//! XMP sidecar import, non-destructive writeback, and local-edit protection.
//!
//! Handles discovery and namespace-aware, non-destructive parsing of `.xmp` sidecar
//! files (compatible with darktable, Adobe Lightroom / Camera Raw, RapidRAW,
//! Capture One, digiKam, etc.).
//!
//! ## Field Mappings
//!
//! | Lap Field | Canonical Namespace URI | XMP Property / Element | Mapping Rules |
//! | :--- | :--- | :--- | :--- |
//! | `afiles.is_favorite` | `https://github.com/julyx10/lap/ns/1.0/` | `lap:Favorite` | Boolean; missing remains `None`; independent of rating, picks and labels. Lap-specific interoperability. |
//! | `afiles.rating` | `http://ns.adobe.com/xap/1.0/` | `xmp:Rating` (`xap:Rating`) | `0..=5` → `Some(0..=5)`; `""` → `Some(0)`; `-1` → `Some(0)` (and sets reject flag if unflagged); missing → `None` |
//! | `afiles.culling_flag` | `http://ns.adobe.com/xmp/1.0/DynamicMedia/`<br>`http://darktable.sf.net/`<br>`http://ns.adobe.com/xap/1.0/` | `xmpDM:pick` / `xmpDM:good`<br>`darktable:rejected`<br>`xmp:Rating="-1"` / `xmp:Label` | `xmpDM:pick="1"` or `xmp:Label="Pick"\|"Select"` → `Some(1)` (Pick)<br>`xmpDM:pick="-1"` or `darktable:rejected="1"` or `xmp:Rating="-1"` or `xmp:Label="Reject"\|"Rejected"` → `Some(2)` (Reject)<br>`xmpDM:pick="0"` or `darktable:rejected="0"` → `Some(0)` (Unflagged)<br>missing → `None` |
//! | `atags` / `afile_tags` | See `t_xmp_compat` | Registered keyword formats | Shared import, path normalization and explicit write capabilities |
//! | `afiles.comments` | `http://purl.org/dc/elements/1.1/`<br>`http://ns.adobe.com/exif/1.0/`<br>`http://ns.adobe.com/photoshop/1.0/` | `dc:description` (`rdf:Alt`)<br>`exif:UserComment` (`rdf:Alt`)<br>`photoshop:Caption` | Priority: `dc:description` (`xml:lang="x-default"` preferred) → `exif:UserComment` → `photoshop:Caption`; empty element/attr → `Some("")`; missing → `None` |

//! Keyword source rules live in `t_xmp_compat`; XML and catalog operations stay here.

use quick_xml::{events::Event, name::ResolveResult, reader::NsReader};
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{
    Mutex,
    atomic::{AtomicBool, AtomicU64, Ordering},
};

use crate::t_sqlite::open_conn;
use crate::t_xmp_compat;
#[cfg(test)]
use crate::t_xmp_compat::{NS_LR, NS_DIGIKAM, NS_ACDSEE};

// Canonical XMP Namespace URIs
/// Lap-owned schema. URI identity is stable and need not resolve to a web page.
pub const NS_LAP: &str = "https://github.com/julyx10/lap/ns/1.0/";
pub const NS_XMP: &str = "http://ns.adobe.com/xap/1.0/";
pub const NS_XMP_DM: &str = "http://ns.adobe.com/xmp/1.0/DynamicMedia/";
pub const NS_DARKTABLE: &str = "http://darktable.sf.net/";
pub const NS_DC: &str = "http://purl.org/dc/elements/1.1/";
pub const NS_EXIF: &str = "http://ns.adobe.com/exif/1.0/";
pub const NS_PHOTOSHOP: &str = "http://ns.adobe.com/photoshop/1.0/";
pub const NS_RDF: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#";

static XMP_IMPORT_ENABLED: AtomicBool = AtomicBool::new(true);

pub fn is_xmp_import_enabled() -> bool {
    XMP_IMPORT_ENABLED.load(Ordering::Relaxed)
}

pub fn set_xmp_import_enabled(enabled: bool) {
    XMP_IMPORT_ENABLED.store(enabled, Ordering::Relaxed);
}

/// Parsed XMP metadata distinguishing missing properties (`None`) from
/// explicitly empty/cleared properties (`Some(0)`, `Some(vec![])`, `Some("")`).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct XmpMetadata {
    /// Missing is distinct from explicit false. Independent of ratings and picks.
    pub favorite: Option<bool>,
    /// Star rating (`0..=5`).
    /// - `None`: rating property is omitted from the sidecar.
    /// - `Some(0)`: explicitly unrated (`0` or `""`, or `-1` when mapped to reject).
    /// - `Some(1..=5)`: star rating.
    pub rating: Option<i32>,
    /// Culling flag (`0` = unflagged, `1` = pick, `2` = reject).
    /// - `None`: no culling/pick/reject property is present in the sidecar.
    /// - `Some(0)`: explicitly unflagged.
    /// - `Some(1)`: pick.
    /// - `Some(2)`: reject.
    pub culling_flag: Option<i32>,
    /// Keywords / subject tags (`dc:subject`, `lr:hierarchicalSubject`, `digiKam:TagsList`).
    /// - `None`: no tag container element exists in the sidecar.
    /// - `Some(vec![])`: tag container exists in the sidecar but contains no user tags.
    /// - `Some(tags)`: non-empty list of deduplicated user tags.
    pub tags: Option<Vec<String>>,
    /// Paths explicitly read from hierarchical keyword fields, not inferred from flat text.
    pub hierarchical_tags: Option<Vec<String>>,
    /// User comment / caption (`dc:description`, `exif:UserComment`, `photoshop:Caption`).
    /// - `None`: no description/caption property exists in the sidecar.
    /// - `Some("")`: property exists in the sidecar, but is explicitly empty.
    /// - `Some(text)`: non-empty description text.
    pub description: Option<String>,
}

/// Controls how imported XMP metadata merges with existing database records.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XmpImportMode {
    /// Used when a new file is first inserted into `afiles`.
    /// Populates all present non-empty fields from the sidecar.
    InitialScan,
    /// Used during folder rescans or `update_file_info` when a file already exists in `afiles`.
    /// Fills empty fields and merges tags only when the corresponding field has
    /// not been edited locally. Explicit local clears are protected as well.
    RescanNonDestructive,
    /// Explicit refresh bypasses fingerprints, preserving local edits.
    RefreshNonDestructive,
    /// Used when the user explicitly selects "Import from XMP" from the context menu.
    /// Applies all `Some(...)` fields from the sidecar (including `Some(0)` or `Some("")` to clear),
    /// while leaving `None` (missing) fields untouched.
    ManualOverwrite,
}

const MAX_XMP_BYTES: usize = 16 * 1024 * 1024;
const MAX_XML_DEPTH: usize = 64;
const MAX_XML_NODES: usize = 100_000;
const MAX_XML_EXPANDED_BYTES: usize = 32 * 1024 * 1024;

static SIDECAR_REVISION: AtomicU64 = AtomicU64::new(0);
type SidecarIndex = (i64, PathBuf, u64, HashMap<String, PathBuf>);

/// Filesystem lifecycle mutations invalidate every thread/session lazily.
/// Update a current, small scan index in-place so batch imports avoid rescanning
/// the whole destination directory for each newly copied sidecar.
pub(crate) fn sidecars_changed(paths: &[PathBuf]) {
    if paths.is_empty() { return; }
    let previous = SIDECAR_REVISION.fetch_add(1, Ordering::AcqRel);
    INDEX.with(|index| {
        let mut index = index.borrow_mut();
        if paths.len() > 64 { *index = None; return; }
        let Some((_, folder, revision, names)) = index.as_mut() else { return; };
        if *revision != previous { return; }
        let canonical_folder = folder.canonicalize().unwrap_or_else(|_| folder.clone());
        let changed: Vec<_> = paths.iter().filter(|path| {
            path.extension().is_some_and(|ext| ext.eq_ignore_ascii_case("xmp"))
                && path.parent().is_some_and(|parent| parent.canonicalize().unwrap_or_else(|_| parent.to_path_buf()) == canonical_folder)
        }).collect();
        for base in changed.iter().filter_map(|path| path.file_stem().and_then(|name| name.to_str())) {
            let chosen = changed.iter().copied().filter(|path| path.file_stem().and_then(|name| name.to_str()) == Some(base))
                .cloned().chain(names.get(base).cloned()).filter(|path| path.is_file()).min();
            if let Some(path) = chosen { names.insert(base.to_owned(), path); }
            else {
                names.remove(base);
                // A differently cased extension may remain outside the cached winner.
                // Leave the revision stale so the next lookup rebuilds the directory.
                return;
            }
        }
        *revision = previous.wrapping_add(1);
    });
}
thread_local! {
    static SCAN: std::cell::Cell<Option<i64>> = const { std::cell::Cell::new(None) };
    static INDEX: std::cell::RefCell<Option<SidecarIndex>> = const { std::cell::RefCell::new(None) };
}
pub struct SidecarScan(Option<i64>);
impl Drop for SidecarScan {
    fn drop(&mut self) {
        SCAN.with(|scan| scan.set(self.0));
    }
}
/// Reuse an index only within this scan's timestamp, never across scan generations.
pub fn sidecar_scan(generation: i64) -> SidecarScan {
    SidecarScan(SCAN.with(|scan| scan.replace(Some(generation))))
}

type SharedRefreshIndex = std::sync::Arc<Mutex<ExportIndex>>;
struct RefreshSession {
    library: String,
    index: SharedRefreshIndex,
    touched: std::time::Instant,
}
static REFRESH_SESSIONS: std::sync::LazyLock<Mutex<HashMap<u64, RefreshSession>>> =
    std::sync::LazyLock::new(|| Mutex::new(HashMap::new()));
static REFRESH_SERIAL: AtomicU64 = AtomicU64::new(1);
thread_local! {
    static REFRESH_INDEX: std::cell::RefCell<Option<SharedRefreshIndex>> = const { std::cell::RefCell::new(None) };
}
pub struct RefreshIndexScope(Option<SharedRefreshIndex>);
impl Drop for RefreshIndexScope {
    fn drop(&mut self) {
        REFRESH_INDEX.with(|index| *index.borrow_mut() = self.0.take());
    }
}
pub fn begin_refresh_batch(library: String) -> Result<u64, String> {
    let mut sessions = REFRESH_SESSIONS.lock().map_err(|e| e.to_string())?;
    // Reclaim abandoned windows; normal completion/cancellation explicitly ends.
    sessions.retain(|_, session| session.touched.elapsed().as_secs() < 3600);
    if sessions.len() >= 16 {
        return Err("Too many metadata refresh batches".into());
    }
    let id = REFRESH_SERIAL.fetch_add(1, Ordering::Relaxed);
    sessions.insert(
        id,
        RefreshSession {
            library,
            index: std::sync::Arc::new(Mutex::new(ExportIndex::default())),
            touched: std::time::Instant::now(),
        },
    );
    Ok(id)
}
pub fn end_refresh_batch(id: u64) {
    if let Ok(mut sessions) = REFRESH_SESSIONS.lock() {
        sessions.remove(&id);
    }
}
pub fn refresh_index_scope(id: Option<u64>, library: &str) -> Result<RefreshIndexScope, String> {
    let index = if let Some(id) = id {
        let mut sessions = REFRESH_SESSIONS.lock().map_err(|e| e.to_string())?;
        let session = sessions
            .get_mut(&id)
            .ok_or("Metadata refresh batch expired")?;
        if session.library != library {
            return Err("Library changed".into());
        }
        session.touched = std::time::Instant::now();
        Some(session.index.clone())
    } else {
        None
    };
    Ok(RefreshIndexScope(
        REFRESH_INDEX.with(|previous| previous.replace(index)),
    ))
}

/// The photo name is case-sensitive; only the sidecar extension is folded.
/// Duplicate extension spellings are resolved deterministically.
pub fn find_xmp_sidecar(image_path: &Path) -> Option<PathBuf> {
    let parent = image_path.parent().unwrap_or(Path::new("."));
    let file_name = image_path.file_name()?.to_str()?;
    let stem = image_path.file_stem()?.to_str()?;
    if let Some(index) = REFRESH_INDEX.with(|index| index.borrow().clone()) {
        let mut cache = index.lock().ok()?;
        let directory = cache.directory(parent).ok()?;
        return directory
            .sidecars
            .get(file_name)
            .or_else(|| directory.sidecars.get(stem))
            .cloned();
    }
    INDEX.with(|cache| {
        let generation = SCAN.with(|scan| scan.get());
        let revision = SIDECAR_REVISION.load(Ordering::Acquire);
        let mut cache = cache.borrow_mut();
        if generation.is_none()
            || cache
                .as_ref()
                .is_none_or(|(scan, dir, old_revision, _)| Some(*scan) != generation || dir != parent || *old_revision != revision)
        {
            let mut names = HashMap::<String, PathBuf>::new();
            for entry in fs::read_dir(parent).ok()?.flatten() {
                let path = entry.path();
                if path
                    .extension()
                    .is_some_and(|e| e.eq_ignore_ascii_case("xmp"))
                    && path.is_file()
                {
                    let name = path.file_stem()?.to_str()?.to_string();
                    let chosen = names.entry(name).or_insert_with(|| path.clone());
                    if path < *chosen {
                        *chosen = path;
                    }
                }
            }
            *cache = Some((generation.unwrap_or(0), parent.to_path_buf(), revision, names));
        }
        let names = &cache.as_ref()?.3;
        names.get(file_name).or_else(|| names.get(stem)).cloned()
    })
}

/// Reads and parses an XMP sidecar file, skipping cloud-only (unhydrated) placeholders.
pub fn read_xmp_file(path: &Path) -> Result<XmpMetadata, String> {
    if let Ok(meta) = fs::metadata(path) {
        if crate::t_utils::is_cloud_only(&meta) {
            return Err(format!(
                "Skipping cloud-only XMP sidecar '{}'",
                path.display()
            ));
        }
    }
    use std::io::Read;
    let file = fs::File::open(path).map_err(|e| e.to_string())?;
    let mut content = Vec::new();
    file.take(MAX_XMP_BYTES as u64 + 1)
        .read_to_end(&mut content)
        .map_err(|e| e.to_string())?;
    if content.len() > MAX_XMP_BYTES {
        return Err("XMP exceeds 16 MiB size limit".into());
    }
    let content = crate::t_metadata::decode_xmp_text(&content)
        .ok_or("Invalid or oversized XMP text encoding")?;
    parse_xmp_str(&content)
}

/// Parse the complete XML document before exposing any metadata to the importer.
pub fn parse_xmp_str(content: &str) -> Result<XmpMetadata, String> {
    let root = parse_xml(content)?;
    let mut properties = Vec::new();
    collect_properties(&root, &mut properties);
    Ok(metadata_from_properties(&properties))
}

fn metadata_from_properties(properties: &[&XmlNode]) -> XmpMetadata {
    let value = |uri: &str, name: &str| {
        properties
            .iter()
            .find(|node| node.uri == uri && node.name == name)
            .map(|n| n.value())
    };
    let mut meta = XmpMetadata::default();
    meta.favorite = value(NS_LAP, "Favorite")
        .as_deref()
        .and_then(parse_favorite);
    let mut rejected = false;
    if let Some(v) = value(NS_XMP, "Rating") {
        if v.trim().is_empty() {
            meta.rating = Some(0);
        } else if let Ok(n) = v.trim().parse::<i32>() {
            if (-1..=5).contains(&n) {
                meta.rating = Some(n.max(0));
                rejected = n == -1;
            }
        }
    }
    let pick = value(NS_XMP_DM, "pick");
    let good = value(NS_XMP_DM, "good");
    let rejected_value = value(NS_DARKTABLE, "rejected");
    let label = value(NS_XMP, "Label");
    meta.culling_flag = culling_from_values(
        pick.as_deref(),
        good.as_deref(),
        rejected_value.as_deref(),
        label.as_deref(),
        if rejected { Some(-1) } else { meta.rating },
    );
    t_xmp_compat::import_properties(properties, &mut meta);
    meta.description = value(NS_DC, "description")
        .or_else(|| value(NS_EXIF, "UserComment"))
        .or_else(|| value(NS_PHOTOSHOP, "Caption"));
    meta
}

/// Standard and referenced extended packets form one logical metadata record.
pub(crate) fn metadata_from_packets(packets: &[String]) -> XmpMetadata {
    let roots: Vec<_> = packets
        .iter()
        .filter_map(|packet| parse_xml(packet).ok())
        .collect();
    let mut properties = Vec::new();
    // Extended packets override standard scalar properties; tags remain a union.
    for root in roots.iter().rev() {
        collect_properties(root, &mut properties);
    }
    metadata_from_properties(&properties)
}

#[cfg(test)]
pub(crate) fn flags_from_packets(packets: &[String]) -> (Option<i32>, Option<bool>) {
    let meta = metadata_from_packets(packets);
    (meta.culling_flag, meta.favorite)
}

fn parse_favorite(value: &str) -> Option<bool> {
    match value.trim().to_ascii_lowercase().as_str() {
        "true" | "1" => Some(true),
        "false" | "0" => Some(false),
        _ => None,
    }
}

#[cfg(test)]
fn culling_from_packets(packets: &[String]) -> Option<i32> {
    flags_from_packets(packets).0
}

/// Shared precedence for sidecar and embedded XMP. Explicit unflagged wins
/// over legacy reject markers. Good=false means Reject, never Unflagged.
pub(crate) fn culling_from_values(
    pick: Option<&str>,
    good: Option<&str>,
    rejected: Option<&str>,
    label: Option<&str>,
    rating: Option<i32>,
) -> Option<i32> {
    pick.and_then(|v| match v.trim() {
        "1" => Some(1),
        "-1" => Some(2),
        "0" | "" => Some(0),
        _ => None,
    })
    .or_else(|| {
        good.and_then(|v| match v.trim().to_ascii_lowercase().as_str() {
            "true" | "1" => Some(1),
            "false" | "0" => Some(2),
            _ => None,
        })
    })
    .or_else(|| {
        rejected.and_then(|v| match v.trim().to_ascii_lowercase().as_str() {
            "true" | "1" => Some(2),
            "false" | "0" | "" => Some(0),
            _ => None,
        })
    })
    .or_else(|| {
        label.and_then(|v| match v.trim().to_ascii_lowercase().as_str() {
            "pick" | "select" => Some(1),
            "reject" | "rejected" => Some(2),
            _ => None,
        })
    })
    .or(if rating == Some(-1) { Some(2) } else { None })
}

// ---------------------------------------------------------------------------
// Database Synchronization Operations
// ---------------------------------------------------------------------------

/// Imports XMP metadata into the database for a specific file using the provided mode.
pub fn import_xmp_for_file_with_conn(
    file_id: i64,
    file_path: &str,
    conn: &Connection,
    mode: XmpImportMode,
) -> Result<bool, String> {
    let sidecar_path = find_xmp_sidecar(Path::new(file_path));
    // The media fingerprint also backfills embedded metadata on existing files
    // once, without repeating deep parsing on every unchanged folder scan.
    // v4 reimports legacy flattened keywords using the tag-tree/vendor schemas.
    let media_info = fs::metadata(file_path).ok();
    if media_info.is_none() && sidecar_path.is_none() {
        return Ok(false);
    }
    let sidecar_info = sidecar_path
        .as_ref()
        .map(fs::metadata)
        .transpose()
        .map_err(|e| e.to_string())?;
    let fingerprint = format!(
        "embedded-v4:{}:{:?}:{}:{:?}:{:?}",
        file_path,
        media_info.as_ref().and_then(|info| info.modified().ok()),
        media_info.map(|info| info.len()).unwrap_or(0),
        sidecar_path,
        sidecar_info.map(|info| (info.modified().ok(), info.len()))
    );
    if !matches!(
        mode,
        XmpImportMode::ManualOverwrite | XmpImportMode::RefreshNonDestructive
    ) {
        let previous: Option<String> = conn
            .query_row(
                "SELECT fingerprint FROM axmp_imports WHERE file_id = ?1",
                [file_id],
                |r| r.get(0),
            )
            .ok()
            .flatten();
        if previous.as_deref() == Some(&fingerprint) {
            return Ok(false);
        }
    }
    // A manual 'Import from XMP' remains a sidecar-only operation. Automatic
    // imports read embedded metadata, then let explicit sidecar fields win.
    let meta = if mode == XmpImportMode::ManualOverwrite {
        let Some(path) = sidecar_path else {
            return Ok(false);
        };
        read_xmp_file(&path)?
    } else {
        let mut embedded = crate::t_metadata::read(Path::new(file_path)).organization;
        if let Some(path) = sidecar_path {
            let sidecar = read_xmp_file(&path)?;
            embedded.favorite = sidecar.favorite.or(embedded.favorite);
            embedded.rating = sidecar.rating.or(embedded.rating);
            embedded.culling_flag = sidecar.culling_flag.or(embedded.culling_flag);
            if sidecar.tags.is_some() {
                embedded.tags = sidecar.tags;
                embedded.hierarchical_tags = sidecar.hierarchical_tags;
            }
            embedded.description = sidecar.description.or(embedded.description);
        }
        embedded
    };
    apply_with_state(file_id, &meta, conn, mode, Some(&fingerprint))
}

/// Scanner-derived descriptions preserve local edits and explicit clears.
pub fn import_scanned_description(
    file_id: i64,
    description: String,
    conn: &Connection,
) -> Result<bool, String> {
    apply_with_state(
        file_id,
        &XmpMetadata {
            description: Some(description),
            ..Default::default()
        },
        conn,
        XmpImportMode::RescanNonDestructive,
        None,
    )
}

/// Installs durable import state and observes local edits, including clears.
/// Import writes restore the prior dirty mask within the same transaction.
pub fn initialize(conn: &Connection) -> Result<(), String> {
    crate::t_tags::initialize(conn)?;
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS axmp_imports (
        file_id INTEGER PRIMARY KEY REFERENCES afiles(id) ON DELETE CASCADE,
        fingerprint TEXT, dirty INTEGER NOT NULL DEFAULT 0
    );",
    )
    .map_err(|e| e.to_string())?;
    for (column, bit) in [
        ("rating", 1),
        ("culling_flag", 2),
        ("comments", 4),
        ("is_favorite", 16),
    ] {
        conn.execute_batch(&format!(
            "CREATE TRIGGER IF NOT EXISTS axmp_local_{column}
        AFTER UPDATE OF {column} ON afiles WHEN NEW.{column} IS NOT OLD.{column}
        BEGIN INSERT INTO axmp_imports(file_id, dirty) VALUES(NEW.id, {bit})
        ON CONFLICT(file_id) DO UPDATE SET dirty = dirty | {bit}; END;"
        ))
        .map_err(|e| e.to_string())?;
    }
    for (operation, row) in [("INSERT", "NEW"), ("DELETE", "OLD")] {
        conn.execute_batch(&format!("CREATE TRIGGER IF NOT EXISTS axmp_local_tags_{operation}
        AFTER {operation} ON afile_tags BEGIN
        INSERT INTO axmp_imports(file_id, dirty) SELECT {row}.file_id, 8 WHERE EXISTS(SELECT 1 FROM afiles WHERE id = {row}.file_id)
        ON CONFLICT(file_id) DO UPDATE SET dirty = dirty | 8; END;" )).map_err(|e| e.to_string())?;
    }
    conn.execute_batch(
        "CREATE TRIGGER IF NOT EXISTS axmp_local_tag_name AFTER UPDATE OF name,parent_id ON atags
        WHEN NEW.name IS NOT OLD.name OR NEW.parent_id IS NOT OLD.parent_id BEGIN
        INSERT INTO axmp_imports(file_id,dirty) SELECT DISTINCT file_id,8 FROM afile_tags
        WHERE tag_id IN (WITH RECURSIVE tree(id) AS (SELECT NEW.id UNION ALL SELECT t.id FROM atags t JOIN tree ON t.parent_id=tree.id) SELECT id FROM tree)
        ON CONFLICT(file_id) DO UPDATE SET dirty=dirty|8; END;",
    )
    .map_err(|e| e.to_string())?;
    conn.execute_batch(
        "CREATE INDEX IF NOT EXISTS idx_atags_name_nocase ON atags(name COLLATE NOCASE)",
    )
    .map_err(|e| e.to_string())?;
    remove_legacy_auto_writeback(conn)?;
    Ok(())
}

#[cfg(test)]
fn apply_xmp_metadata_to_db(
    file_id: i64,
    meta: &XmpMetadata,
    conn: &Connection,
    mode: XmpImportMode,
) -> Result<bool, String> {
    apply_with_state(file_id, meta, conn, mode, None)
}

fn apply_with_state(
    file_id: i64,
    meta: &XmpMetadata,
    conn: &Connection,
    mode: XmpImportMode,
    fingerprint: Option<&str>,
) -> Result<bool, String> {
    let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;
    let dirty: i32 = tx
        .query_row(
            "SELECT dirty FROM axmp_imports WHERE file_id = ?1",
            [file_id],
            |r| r.get(0),
        )
        .unwrap_or(0);
    let result = apply_fields(file_id, meta, &tx, mode, dirty)?;
    tx.execute("INSERT INTO axmp_imports(file_id, fingerprint, dirty) VALUES(?1, ?2, ?3)
        ON CONFLICT(file_id) DO UPDATE SET fingerprint = COALESCE(excluded.fingerprint, fingerprint), dirty = excluded.dirty",
        params![file_id, fingerprint, dirty]).map_err(|e| e.to_string())?;
    tx.commit().map_err(|e| e.to_string())?;
    Ok(result)
}

fn apply_fields(
    file_id: i64,
    meta: &XmpMetadata,
    conn: &Connection,
    mode: XmpImportMode,
    dirty: i32,
) -> Result<bool, String> {
    let (db_rating, db_culling_flag, db_comments, db_favorite): (i32, i32, Option<String>, bool) = conn.query_row(
        "SELECT COALESCE(rating, 0), COALESCE(culling_flag, 0), comments, COALESCE(is_favorite, 0) FROM afiles WHERE id = ?1", [file_id],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
    ).map_err(|e| e.to_string())?;

    if let Some(favorite) = meta.favorite {
        let should_update = match mode {
            XmpImportMode::InitialScan => favorite,
            XmpImportMode::RescanNonDestructive | XmpImportMode::RefreshNonDestructive => {
                dirty & 16 == 0 && !db_favorite && favorite
            }
            XmpImportMode::ManualOverwrite => favorite != db_favorite,
        };
        if should_update {
            conn.execute(
                "UPDATE afiles SET is_favorite=?1 WHERE id=?2",
                params![favorite, file_id],
            )
            .map_err(|e| e.to_string())?;
        }
    }

    // 1. Rating
    if let Some(rating) = meta.rating {
        let clamped = rating.clamp(0, 5);
        let should_update = match mode {
            XmpImportMode::InitialScan => clamped > 0,
            XmpImportMode::RescanNonDestructive | XmpImportMode::RefreshNonDestructive => {
                dirty & 1 == 0 && db_rating == 0 && clamped > 0
            }
            XmpImportMode::ManualOverwrite => clamped != db_rating,
        };
        if should_update {
            conn.execute(
                "UPDATE afiles SET rating = ?1 WHERE id = ?2",
                params![clamped, file_id],
            )
            .map_err(|e| e.to_string())?;
        }
    }

    // 2. Culling flag
    if let Some(flag) = meta.culling_flag {
        let clamped = flag.clamp(0, 2);
        let should_update = match mode {
            XmpImportMode::InitialScan => clamped > 0,
            XmpImportMode::RescanNonDestructive | XmpImportMode::RefreshNonDestructive => {
                dirty & 2 == 0 && db_culling_flag == 0 && clamped > 0
            }
            XmpImportMode::ManualOverwrite => clamped != db_culling_flag,
        };
        if should_update {
            conn.execute(
                "UPDATE afiles SET culling_flag = ?1 WHERE id = ?2",
                params![clamped, file_id],
            )
            .map_err(|e| e.to_string())?;
        }
    }

    // 3. Comments / description
    if let Some(ref desc) = meta.description {
        let trimmed = desc.trim();
        let db_comment_empty = db_comments.as_deref().is_none_or(|c| c.trim().is_empty());

        let should_update = match mode {
            XmpImportMode::InitialScan => !trimmed.is_empty(),
            XmpImportMode::RescanNonDestructive | XmpImportMode::RefreshNonDestructive => {
                dirty & 4 == 0 && db_comment_empty && !trimmed.is_empty()
            }
            XmpImportMode::ManualOverwrite => true,
        };
        if should_update {
            let new_val: Option<&str> = if trimmed.is_empty() {
                None
            } else {
                Some(trimmed)
            };
            conn.execute(
                "UPDATE afiles SET comments = ?1 WHERE id = ?2",
                params![new_val, file_id],
            )
            .map_err(|e| e.to_string())?;
        }
    }

    // 4. Keywords / Tags
    if let Some(ref tags) = meta.tags {
        if matches!(
            mode,
            XmpImportMode::RescanNonDestructive | XmpImportMode::RefreshNonDestructive
        ) && dirty & 8 != 0
        {
            return Ok(true);
        }
        if mode == XmpImportMode::ManualOverwrite {
            conn.execute("DELETE FROM afile_tags WHERE file_id = ?1", [file_id])
                .map_err(|e| e.to_string())?;
            conn.execute("UPDATE afiles SET has_tags = 0 WHERE id = ?1", [file_id])
                .map_err(|e| e.to_string())?;
        }
        if tags.is_empty() {
            if mode == XmpImportMode::ManualOverwrite {
                conn.execute(
                    "DELETE FROM afile_tags WHERE file_id = ?1",
                    params![file_id],
                )
                .map_err(|e| e.to_string())?;
                conn.execute(
                    "UPDATE afiles SET has_tags = 0 WHERE id = ?1",
                    params![file_id],
                )
                .map_err(|e| e.to_string())?;
            }
        } else {
            let paths = meta.hierarchical_tags.as_deref().unwrap_or_default();
            // Flat stores may contain every ancestor, not just the assigned leaf.
            let represented = |tag: &str| paths.iter().any(|path| path.eq_ignore_ascii_case(tag.trim())
                || path.split('|').any(|part| part.trim().eq_ignore_ascii_case(tag.trim())));
            // Only explicit source paths justify replacing old flattened associations.
            // Keep the old root nodes themselves; they may be used on other files.
            if dirty & 8 == 0 {
                for tag in tags.iter().filter(|tag| represented(tag)) {
                    conn.execute("DELETE FROM afile_tags WHERE file_id=?1 AND tag_id IN (SELECT id FROM atags WHERE parent_id IS NULL AND name=?2 COLLATE NOCASE)", params![file_id,tag])
                        .map_err(|e|e.to_string())?;
                }
            }
            let mut added_any = false;
            for value in paths.iter().map(|p| (p, true)).chain(tags.iter()
                .filter(|tag| !represented(tag))
                .map(|tag| (tag, false))) {
                let Some(tag_id) = crate::t_tags::import_path(conn, value.0, value.1)? else { continue };
                conn.execute("INSERT OR IGNORE INTO afile_tags(file_id,tag_id) VALUES(?1,?2)", params![file_id,tag_id])
                    .map_err(|e|e.to_string())?;
                added_any = true;
            }

            if added_any {
                conn.execute(
                    "UPDATE afiles SET has_tags = 1 WHERE id = ?1",
                    params![file_id],
                )
                .map_err(|e| e.to_string())?;
            }
        }
    }

    Ok(true)
}

/// Imports XMP metadata into the database for a file using a fresh connection.
pub fn import_xmp_for_file_with_mode(
    file_id: i64,
    file_path: &str,
    mode: XmpImportMode,
) -> Result<bool, String> {
    let conn = open_conn()?;
    import_xmp_for_file_with_conn(file_id, file_path, &conn, mode)
}

// ---------------------------------------------------------------------------
// Namespace-Aware XML Parsing Helpers
// ---------------------------------------------------------------------------

#[derive(Default)]
pub(crate) struct XmlNode {
    pub(crate) uri: String,
    pub(crate) name: String,
    pub(crate) text: String,
    pub(crate) lang: Option<String>,
    pub(crate) attributes: Vec<XmlNode>,
    pub(crate) children: Vec<XmlNode>,
}
impl XmlNode {
    pub(crate) fn value(&self) -> String {
        if let Some(alt) = self
            .children
            .iter()
            .find(|n| n.uri == NS_RDF && n.name == "Alt")
        {
            let items = || {
                alt.children
                    .iter()
                    .filter(|n| n.uri == NS_RDF && n.name == "li")
            };
            return items()
                .find(|n| n.lang.as_deref() == Some("x-default"))
                .or_else(|| items().next())
                .map(|n| n.text.trim().to_string())
                .unwrap_or_default();
        }
        self.text.trim().to_string()
    }
}
fn collect_properties<'a>(node: &'a XmlNode, out: &mut Vec<&'a XmlNode>) {
    // Retain support for a standalone RDF Description document.
    if node.name.is_empty() && node.children.len() == 1 {
        let root = &node.children[0];
        if root.uri == NS_RDF && root.name == "Description" {
            if !root.attributes.iter().any(|a| {
                a.uri == NS_RDF && (a.name == "nodeID" || (a.name == "about" && !a.text.is_empty()))
            }) {
                out.extend(root.attributes.iter());
                out.extend(root.children.iter());
            }
            return;
        }
    }
    if node.uri == NS_RDF && node.name == "RDF" {
        for description in &node.children {
            if description.uri != NS_RDF || description.name != "Description" {
                continue;
            }
            // Only the document's photo subject, not nested/identified resources.
            if description.attributes.iter().any(|a| {
                a.uri == NS_RDF && (a.name == "nodeID" || (a.name == "about" && !a.text.is_empty()))
            }) {
                continue;
            }
            out.extend(description.attributes.iter());
            out.extend(description.children.iter());
        }
        return;
    }
    for child in &node.children {
        collect_properties(child, out);
    }
}
fn resolved_name(result: quick_xml::name::ResolveResult<'_>) -> Result<String, String> {
    match result {
        quick_xml::name::ResolveResult::Bound(ns) => {
            String::from_utf8(ns.as_ref().to_vec()).map_err(|e| e.to_string())
        }
        quick_xml::name::ResolveResult::Unbound => Ok(String::new()),
        quick_xml::name::ResolveResult::Unknown(prefix) => {
            Err(format!("Unbound XML prefix: {:?}", prefix))
        }
    }
}
pub(crate) fn parse_xml(content: &str) -> Result<XmlNode, String> {
    use quick_xml::{events::Event, reader::NsReader};
    if content.len() > MAX_XMP_BYTES {
        return Err("XMP exceeds 16 MiB size limit".into());
    }
    let mut reader = NsReader::from_str(content);
    let mut nodes = 0usize;
    let mut expanded = 0usize;
    let mut stack = vec![XmlNode::default()];
    loop {
        let event = reader.read_event().map_err(|e| e.to_string())?;
        match event {
            Event::Start(ref e) | Event::Empty(ref e) => {
                if stack.len() > MAX_XML_DEPTH {
                    return Err("XMP nesting depth limit exceeded".into());
                }
                nodes += 1;
                if nodes > MAX_XML_NODES {
                    return Err("XMP node limit exceeded".into());
                }
                let (ns, local) = reader.resolve_element(e.name());
                let mut node = XmlNode {
                    uri: resolved_name(ns)?,
                    name: String::from_utf8_lossy(local.as_ref()).into_owned(),
                    ..Default::default()
                };
                expanded += node.uri.len() + node.name.len();
                if expanded > MAX_XML_EXPANDED_BYTES {
                    return Err("XMP expanded text limit exceeded".into());
                }
                for attr in e.attributes() {
                    let attr = attr.map_err(|e| e.to_string())?;
                    nodes += 1;
                    if nodes > MAX_XML_NODES {
                        return Err("XMP node limit exceeded".into());
                    }
                    if attr.key.as_ref() == b"xmlns" || attr.key.as_ref().starts_with(b"xmlns:") {
                        continue;
                    }
                    let (ns, local) = reader.resolve_attribute(attr.key);
                    let uri = resolved_name(ns)?;
                    let name = String::from_utf8_lossy(local.as_ref()).into_owned();
                    let text = attr
                        .decode_and_unescape_value(reader.decoder())
                        .map_err(|e| e.to_string())?
                        .into_owned();
                    expanded += uri.len() + name.len() + text.len();
                    if expanded > MAX_XML_EXPANDED_BYTES {
                        return Err("XMP expanded text limit exceeded".into());
                    }
                    if uri == "http://www.w3.org/XML/1998/namespace" && name == "lang" {
                        node.lang = Some(text.clone());
                    }
                    node.attributes.push(XmlNode {
                        uri,
                        name,
                        text,
                        ..Default::default()
                    });
                }
                if matches!(event, Event::Empty(_)) {
                    stack.last_mut().unwrap().children.push(node);
                } else {
                    stack.push(node);
                }
            }
            Event::End(_) => {
                if stack.len() <= 1 {
                    return Err("Unexpected XML closing tag".into());
                }
                let node = stack.pop().unwrap();
                stack.last_mut().unwrap().children.push(node);
            }
            Event::Text(e) => {
                let text = e.unescape().map_err(|e| e.to_string())?;
                if stack.len() == 1 && !text.trim().is_empty() {
                    return Err("Text outside XML root".into());
                }
                expanded += text.len();
                if expanded > MAX_XML_EXPANDED_BYTES {
                    return Err("XMP expanded text limit exceeded".into());
                }
                stack.last_mut().unwrap().text.push_str(&text);
            }
            Event::CData(e) => {
                let text = e.decode().map_err(|e| e.to_string())?;
                expanded += text.len();
                if expanded > MAX_XML_EXPANDED_BYTES {
                    return Err("XMP expanded text limit exceeded".into());
                }
                stack.last_mut().unwrap().text.push_str(&text);
            }
            Event::DocType(_) => return Err("DTD is not supported in XMP".into()),
            Event::Eof => break,
            _ => {}
        }
    }
    if stack.len() != 1 || stack[0].children.len() != 1 {
        return Err("Incomplete XML document or multiple roots".into());
    }
    Ok(stack.pop().unwrap())
}

// ---------------------------------------------------------------------------
// Non-destructive sidecar writeback (no media writer or EXIF backend)
// ---------------------------------------------------------------------------

static SERIAL: AtomicU64 = AtomicU64::new(0);
static WRITE_LOCK: Mutex<()> = Mutex::new(());
const ALL: i32 = 1 | 2 | 4 | 8 | 16;

#[derive(Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchResult {
    pub saved_ids: Vec<i64>,
    pub failures: Vec<Failure>,
}
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Failure {
    pub file_id: i64,
    pub error: String,
}

/// Remove the experimental automatic-export schema on existing libraries.
/// Local-edit protection in axmp_imports is retained for automatic import.
fn remove_legacy_auto_writeback(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        "DROP TRIGGER IF EXISTS axmp_write_rating;
        DROP TRIGGER IF EXISTS axmp_write_culling_flag;
        DROP TRIGGER IF EXISTS axmp_write_comments;
        DROP TRIGGER IF EXISTS axmp_write_tags_INSERT;
        DROP TRIGGER IF EXISTS axmp_write_tags_DELETE;
        DROP TRIGGER IF EXISTS axmp_write_tag_name;
        DROP TABLE IF EXISTS axmp_writes;
        DROP TABLE IF EXISTS axmp_write_control;",
    )
    .map_err(|e| e.to_string())
}

fn namespace(result: ResolveResult<'_>) -> Result<String, String> {
    match result {
        ResolveResult::Bound(n) => Ok(String::from_utf8_lossy(n.as_ref()).into()),
        ResolveResult::Unbound => Ok(String::new()),
        ResolveResult::Unknown(_) => Err("Unbound XML namespace".into()),
    }
}
#[derive(Default)]
struct Node {
    uri: String,
    name: String,
    start: usize,
    open_end: usize,
    close: usize,
    end: usize,
    parent: Option<usize>,
    lang: Option<String>,
    about: Option<String>,
}
fn escape(s: &str) -> String {
    quick_xml::escape::escape(s).into_owned()
}

/// Patch owned properties while preserving unrelated elements and language variants.
fn culling_property(uri: &str, name: &[u8], value: &str) -> bool {
    (uri == NS_XMP_DM && matches!(name, b"pick" | b"good"))
        || (uri == NS_DARKTABLE && name == b"rejected")
        || (uri == NS_XMP
            && name == b"Label"
            && matches!(
                value.trim().to_ascii_lowercase().as_str(),
                "pick" | "select" | "reject" | "rejected"
            ))
}

fn property_text(xml: &str, node: &Node) -> Result<String, String> {
    // Text/CDATA scalar properties only. Other managed properties do not need
    // this value, and their nested XML must not be interpreted as a flag.
    if node.uri != NS_XMP || node.name != "Label" {
        return Ok(String::new());
    }
    let mut reader = quick_xml::Reader::from_str(&xml[node.start..node.end]);
    let mut text = String::new();
    loop {
        match reader.read_event().map_err(|e| e.to_string())? {
            Event::Text(e) => text.push_str(&e.unescape().map_err(|e| e.to_string())?),
            Event::CData(e) => text.push_str(&e.decode().map_err(|e| e.to_string())?),
            Event::Eof => break,
            _ => {}
        }
    }
    Ok(text)
}

#[cfg(test)]
fn merge_with_culling(
    xml: &str,
    rating: i32,
    culling: i32,
    tags: &[String],
    description: &str,
    mask: i32,
) -> Result<String, String> {
    merge_with_favorite(xml, rating, culling, false, tags, description, mask & !16)
}

#[cfg(test)]
fn merge_with_favorite(
    xml: &str,
    rating: i32,
    culling: i32,
    favorite: bool,
    tags: &[String],
    description: &str,
    mask: i32,
) -> Result<String, String> {
    merge_with_paths(xml,rating,culling,favorite,tags,None,description,mask)
}

fn merge_with_paths(
    xml: &str, rating: i32, culling: i32, favorite: bool,
    tags: &[String], paths: Option<&[String]>, description: &str, mask: i32,
) -> Result<String,String> {
    parse_xmp_str(xml)?; // reuse bounded, DTD-free validation
    // Rating and culling are one export unit: normalize legacy Rating=-1
    // while preserving the independent Lap star rating and current flag.
    let mask = if mask & 3 != 0 { mask | 3 } else { mask };
    let mut reader = NsReader::from_str(xml);
    let mut nodes: Vec<Node> = Vec::new();
    let mut stack: Vec<usize> = Vec::new();
    let mut patches: Vec<(usize, usize, String)> = Vec::new();
    let mut tag_writer = t_xmp_compat::TagWriter::default();
    let mut had_darktable_rejected = false;
    loop {
        let start = reader.buffer_position() as usize;
        let event = reader.read_event().map_err(|e| e.to_string())?;
        let end = reader.buffer_position() as usize;
        match event {
            Event::Start(ref e) | Event::Empty(ref e) => {
                let (ns, local) = reader.resolve_element(e.name());
                let mut node = Node {
                    uri: namespace(ns)?,
                    name: String::from_utf8_lossy(local.as_ref()).into(),
                    start,
                    open_end: end,
                    close: start,
                    end,
                    parent: stack.last().copied(),
                    ..Default::default()
                };
                let mut filtered = e.to_owned();
                filtered.clear_attributes();
                let mut changed = false;
                let mut retained_property = false;
                for a in e.attributes() {
                    let a = a.map_err(|e| e.to_string())?;
                    let (ns, local) = reader.resolve_attribute(a.key);
                    let uri = namespace(ns)?;
                    let name = String::from_utf8_lossy(local.as_ref());
                    if uri == NS_RDF && name == "about" {
                        node.about = Some(
                            a.decode_and_unescape_value(reader.decoder())
                                .map_err(|e| e.to_string())?
                                .into_owned(),
                        );
                    }
                    if uri == "http://www.w3.org/XML/1998/namespace" && name == "lang" {
                        node.lang = Some(
                            a.decode_and_unescape_value(reader.decoder())
                                .map_err(|e| e.to_string())?
                                .into_owned(),
                        );
                    }
                }
                // Strip managed shorthand attributes, only on the photo's Description.
                if node.uri == NS_RDF
                    && node.name == "Description"
                    && node
                        .parent
                        .is_some_and(|i| nodes[i].uri == NS_RDF && nodes[i].name == "RDF")
                    && node.about.as_deref().unwrap_or("").is_empty()
                {
                    filtered.clear_attributes();
                    for a in e.attributes() {
                        let a = a.map_err(|e| e.to_string())?;
                        let (ns, local) = reader.resolve_attribute(a.key);
                        let uri = namespace(ns)?;
                        let own = (uri == NS_LAP
                            && local.as_ref() == b"Favorite"
                            && mask & 16 != 0)
                            || (uri == NS_XMP && local.as_ref() == b"Rating" && mask & 1 != 0)
                            || (t_xmp_compat::writable_property(&uri, local.as_ref()) && mask & 8 != 0)
                            || (uri == NS_DC && local.as_ref() == b"description" && mask & 4 != 0)
                            || (mask & 2 != 0
                                && culling_property(
                                    &uri,
                                    local.as_ref(),
                                    &a.decode_and_unescape_value(reader.decoder())
                                        .map_err(|e| e.to_string())?,
                                ));
                        if own {
                            tag_writer.observe(&uri, &String::from_utf8_lossy(local.as_ref()));
                            had_darktable_rejected |=
                                uri == NS_DARKTABLE && local.as_ref() == b"rejected";
                            changed = true;
                        } else {
                            // Attribute already contains validated, escaped source bytes.
                            // Preserve those bytes, including namespace declarations.
                            retained_property |= !(a.key.as_ref() == b"xmlns"
                                || a.key.as_ref().starts_with(b"xmlns:")
                                || (uri == NS_RDF && local.as_ref() == b"about"));
                            filtered.push_attribute(a);
                        }
                    }
                    if changed {
                        let mut writer = quick_xml::Writer::new(Vec::new());
                        writer
                            .write_event(if matches!(event, Event::Empty(_)) {
                                Event::Empty(filtered)
                            } else {
                                Event::Start(filtered)
                            })
                            .map_err(|e| e.to_string())?;
                        patches.push((
                            start,
                            end,
                            if matches!(event, Event::Empty(_)) && !retained_property {
                                String::new()
                            } else {
                                String::from_utf8(writer.into_inner()).map_err(|e| e.to_string())?
                            },
                        ));
                    }
                }
                nodes.push(node);
                let index = nodes.len() - 1;
                if matches!(event, Event::Start(_)) {
                    stack.push(index);
                }
            }
            Event::End(_) => {
                let index = stack.pop().ok_or("Unexpected closing tag")?;
                nodes[index].close = start;
                nodes[index].end = end;
            }
            Event::PI(ref e) if e.target() == b"xpacket" => {
                let content = std::str::from_utf8(e.content()).map_err(|e| e.to_string())?;
                let pseudo = format!("<packet {content}/>");
                let mut pi_reader = quick_xml::Reader::from_str(&pseudo);
                if let Event::Empty(packet) = pi_reader.read_event().map_err(|e| e.to_string())? {
                    for attr in packet.attributes() {
                        let attr = attr.map_err(|e| e.to_string())?;
                        if attr.key.as_ref() == b"end" && attr.value.as_ref() == b"r" {
                            return Err("XMP packet is marked read-only".into());
                        }
                    }
                }
            }
            Event::Eof => break,
            _ => {}
        }
    }
    let rdf: Vec<_> = nodes
        .iter()
        .enumerate()
        .filter(|(_, n)| n.uri == NS_RDF && n.name == "RDF")
        .collect();
    if rdf.len() != 1 {
        return Err("Expected one RDF metadata container".into());
    }
    let (rdf_id, rdf) = rdf[0];
    if nodes.iter().any(|n| {
        n.parent == Some(rdf_id)
            && n.uri == NS_RDF
            && n.name == "Description"
            && !n.about.as_deref().unwrap_or("").is_empty()
    }) {
        return Err(
            "Named RDF subjects require manual resolution; original sidecar retained".into(),
        );
    }
    let mut found_description = false;
    for (id, n) in nodes.iter().enumerate() {
        let Some(parent) = n.parent else { continue };
        let p = &nodes[parent];
        if p.uri != NS_RDF
            || p.name != "Description"
            || p.parent != Some(rdf_id)
            || !p.about.as_deref().unwrap_or("").is_empty()
        {
            continue;
        }
        if (mask & 16 != 0 && n.uri == NS_LAP && n.name == "Favorite")
            || (mask & 1 != 0 && n.uri == NS_XMP && n.name == "Rating")
            || (mask & 8 != 0 && t_xmp_compat::writable_property(&n.uri, n.name.as_bytes()))
            || (mask & 2 != 0
                && culling_property(&n.uri, n.name.as_bytes(), &property_text(xml, n)?))
        {
            had_darktable_rejected |= n.uri == NS_DARKTABLE && n.name == "rejected";
            tag_writer.observe(&n.uri, &n.name);
            patches.push((n.start, n.end, String::new()));
        }
        if mask & 4 != 0 && n.uri == NS_DC && n.name == "description" {
            if found_description {
                return Err("Multiple description properties require manual resolution".into());
            }
            found_description = true;
            let alts: Vec<_> = nodes
                .iter()
                .enumerate()
                .filter(|(_, a)| a.parent == Some(id) && a.uri == NS_RDF && a.name == "Alt")
                .collect();
            if alts.len() == 1 && alts[0].1.open_end == alts[0].1.end {
                // A self-closing Alt is a valid empty language container.
                let alt = alts[0].1;
                patches.push((
                    alt.start,
                    alt.end,
                    format!(
                        "<rdf:Alt xmlns:rdf=\"{NS_RDF}\"><rdf:li xml:lang=\"x-default\">{}</rdf:li></rdf:Alt>",
                        escape(description)
                    ),
                ));
            } else if alts.len() == 1 {
                let (alt_id, alt) = alts[0];
                for li in nodes.iter().filter(|li| {
                    li.parent == Some(alt_id)
                        && li.uri == NS_RDF
                        && li.name == "li"
                        && li.lang.as_deref() == Some("x-default")
                }) {
                    patches.push((li.start, li.end, String::new()));
                }
                patches.push((
                    alt.close,
                    alt.close,
                    format!(
                        "<rdf:li xmlns:rdf=\"{NS_RDF}\" xml:lang=\"x-default\">{}</rdf:li>",
                        escape(description)
                    ),
                ));
            } else if nodes.iter().any(|c| c.parent == Some(id)) {
                return Err("Unsupported description structure; original sidecar retained".into());
            } else {
                patches.push((n.start, n.end, description_xml(description)));
            }
        }
    }
    let mut properties = String::new();
    if mask & 16 != 0 {
        properties += &format!(
            "<lap:Favorite xmlns:lap=\"{NS_LAP}\">{}</lap:Favorite>",
            if favorite { "True" } else { "False" }
        );
    }
    if mask & 1 != 0 {
        properties += &format!("<xmp:Rating>{}</xmp:Rating>", rating.clamp(0, 5));
    }
    if mask & 2 != 0 {
        let pick = match culling {
            1 => 1,
            2 => -1,
            _ => 0,
        };
        properties += &format!("<xmpDM:pick>{pick}</xmpDM:pick>");
        if culling == 1 || culling == 2 {
            properties += &format!(
                "<xmpDM:good>{}</xmpDM:good>",
                if culling == 1 { "True" } else { "False" }
            );
        }
        if had_darktable_rejected {
            properties += &format!(
                "<darktable:rejected>{}</darktable:rejected>",
                if culling == 2 { 1 } else { 0 }
            );
        }
    }
    if mask & 8 != 0 {
        properties += &tag_writer.render(tags, paths);
    }
    if mask & 4 != 0 && !found_description {
        properties += &description_xml(description);
    }
    let addition = format!(
        "<rdf:Description xmlns:rdf=\"{NS_RDF}\" xmlns:xmp=\"{NS_XMP}\" xmlns:dc=\"{NS_DC}\" xmlns:xmpDM=\"{NS_XMP_DM}\" xmlns:darktable=\"{NS_DARKTABLE}\" rdf:about=\"\">{properties}</rdf:Description>"
    );
    if rdf.open_end == rdf.end {
        // empty RDF element
        let opening = xml[rdf.start..rdf.end]
            .trim_end_matches('>')
            .trim_end_matches('/');
        let qname = xml[rdf.start + 1..rdf.end]
            .split(|c: char| c.is_whitespace() || c == '/' || c == '>')
            .next()
            .ok_or("Invalid RDF name")?;
        patches.push((
            rdf.start,
            rdf.end,
            format!("{opening}>{addition}</{qname}>"),
        ));
    } else {
        patches.push((rdf.close, rdf.close, addition));
    }
    patches.sort_by_key(|p| (p.0, p.1));
    let mut out = String::new();
    let mut cursor = 0;
    for (start, end, value) in patches {
        if start < cursor {
            return Err("Overlapping XMP properties".into());
        }
        out += &xml[cursor..start];
        out += &value;
        cursor = end;
    }
    out += &xml[cursor..];
    parse_xmp_str(&out)?;
    Ok(out)
}
fn description_xml(value: &str) -> String {
    format!(
        "<dc:description xmlns:dc=\"{NS_DC}\" xmlns:rdf=\"{NS_RDF}\"><rdf:Alt><rdf:li xml:lang=\"x-default\">{}</rdf:li></rdf:Alt></dc:description>",
        escape(value)
    )
}

fn bounded_read(path: &Path) -> Result<Vec<u8>, String> {
    let meta = fs::symlink_metadata(path).map_err(|e| e.to_string())?;
    if !meta.is_file() || meta.file_type().is_symlink() || crate::t_utils::is_cloud_only(&meta) {
        return Err("Sidecar is not an available regular local file".into());
    }
    if meta.permissions().readonly() {
        return Err("Sidecar is read-only".into());
    }
    let mut bytes = Vec::new();
    fs::File::open(path)
        .map_err(|e| e.to_string())?
        .take(16 * 1024 * 1024 + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() > 16 * 1024 * 1024 {
        return Err("XMP exceeds 16 MiB".into());
    }
    Ok(bytes)
}

#[derive(Default)]
struct ExportDirectory {
    sidecars: HashMap<String, PathBuf>,
    media_stems: HashMap<std::ffi::OsString, usize>,
}

impl ExportDirectory {
    fn insert_sidecar(&mut self, path: &Path) {
        let Some(name) = path.file_stem().and_then(|s| s.to_str()) else {
            return;
        };
        let chosen = self
            .sidecars
            .entry(name.to_owned())
            .or_insert_with(|| path.to_path_buf());
        if path < chosen.as_path() {
            *chosen = path.to_path_buf();
        }
    }

    fn read(parent: &Path) -> Result<Self, String> {
        let mut index = Self::default();
        for entry in fs::read_dir(parent).map_err(|e| e.to_string())? {
            let path = entry.map_err(|e| e.to_string())?.path();
            if path
                .extension()
                .is_some_and(|e| e.eq_ignore_ascii_case("xmp"))
            {
                if path.is_file() {
                    index.insert_sidecar(&path);
                }
            } else if path.extension().is_some() {
                if let Some(stem) = path.file_stem() {
                    *index.media_stems.entry(stem.to_owned()).or_default() += 1;
                }
            }
        }
        Ok(index)
    }
}

/// Scoped to one explicit export batch; directories may be interleaved.
#[derive(Default)]
struct ExportIndex(HashMap<PathBuf, Result<ExportDirectory, String>>, u64);
impl ExportIndex {
    fn directory(&mut self, parent: &Path) -> Result<&mut ExportDirectory, String> {
        let revision = SIDECAR_REVISION.load(Ordering::Acquire);
        if self.1 != revision { self.0.clear(); self.1 = revision; }
        self.0
            .entry(parent.to_path_buf())
            .or_insert_with(|| ExportDirectory::read(parent))
            .as_mut()
            .map_err(|e| e.clone())
    }
    fn published(&mut self, path: &Path) {
        if let Some(Ok(directory)) = path.parent().and_then(|p| self.0.get_mut(p)) {
            directory.insert_sidecar(path);
        }
    }
}

fn destination(media: &Path, cache: &mut ExportIndex) -> Result<PathBuf, String> {
    let parent = media.parent().ok_or("Missing parent folder")?;
    let file = media
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or("Invalid media filename")?;
    let stem = media
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or("Invalid media filename")?;
    let directory = cache.directory(parent)?;
    let canonical = parent.join(format!("{file}.xmp"));
    if let Some(existing) = directory.sidecars.get(file) {
        return Ok(existing.clone());
    }
    let Some(existing) = directory.sidecars.get(stem) else {
        return Ok(canonical);
    };
    // Cache the RAW/JPEG shared-stem check together with sidecar discovery.
    if directory
        .media_stems
        .get(std::ffi::OsStr::new(stem))
        .copied()
        .unwrap_or(0)
        > 1
    {
        return Err("The stem-only XMP is shared by multiple files; use a file-specific sidecar before saving".into());
    }
    Ok(existing.clone())
}

/// Preserve the original BOM encoding, including any XML encoding declaration.
fn encode_xmp_text<'a>(text: &'a str, original: Option<&[u8]>) -> std::borrow::Cow<'a, [u8]> {
    let original = original.unwrap_or_default();
    if original.starts_with(&[0xff, 0xfe, 0, 0]) || original.starts_with(&[0, 0, 0xfe, 0xff]) {
        let little = original[0] == 0xff;
        let mut bytes = original[..4].to_vec();
        for character in text.chars() {
            let value = character as u32;
            bytes.extend(if little {
                value.to_le_bytes()
            } else {
                value.to_be_bytes()
            });
        }
        bytes.into()
    } else if original.starts_with(&[0xff, 0xfe]) || original.starts_with(&[0xfe, 0xff]) {
        let little = original[0] == 0xff;
        let mut bytes = original[..2].to_vec();
        for value in text.encode_utf16() {
            bytes.extend(if little {
                value.to_le_bytes()
            } else {
                value.to_be_bytes()
            });
        }
        bytes.into()
    } else {
        if !original.starts_with(&[0xef, 0xbb, 0xbf]) {
            return std::borrow::Cow::Borrowed(text.as_bytes());
        }
        let mut bytes = original[..3].to_vec();
        bytes.extend_from_slice(text.as_bytes());
        bytes.into()
    }
}

fn write_one(conn: &Connection, file_id: i64, cache: &mut ExportIndex) -> Result<(), String> {
    // A consistent catalog snapshot for this explicit manual export.
    let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;
    let (folder, name, rating, culling, description, favorite): (String, String, i32, i32, String, bool) = tx
        .query_row("SELECT f.path,a.name,COALESCE(a.rating,0),COALESCE(a.culling_flag,0),COALESCE(a.comments,''),COALESCE(a.is_favorite,0) FROM afiles a JOIN afolders f ON f.id=a.folder_id WHERE a.id=?1",
            [file_id], |r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?,r.get(5)?)))
        .map_err(|e|e.to_string())?;
    let (tags, paths) = crate::t_tags::keywords_for_file(&tx,file_id)?;
    tx.commit().map_err(|e| e.to_string())?;
    let mut components = Path::new(&name).components();
    if !matches!(components.next(), Some(std::path::Component::Normal(_)))
        || components.next().is_some()
    {
        return Err("Invalid catalog filename".into());
    }
    let media_path = Path::new(&folder).join(&name);
    let media = media_path.as_path();
    let meta = fs::metadata(media).map_err(|e| e.to_string())?;
    if !meta.is_file() || crate::t_utils::is_cloud_only(&meta) {
        return Err("Original media is unavailable locally".into());
    }
    let dest = destination(media, cache)?;
    let before = match fs::symlink_metadata(&dest) {
        Ok(_) => Some(bounded_read(&dest)?),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
        Err(e) => return Err(e.to_string()),
    };
    let xml = match before.as_deref() {
        Some(b) => crate::t_metadata::decode_xmp_text(b)
            .ok_or("Invalid or oversized XMP text encoding")?,
        None => format!(
            "<x:xmpmeta xmlns:x=\"adobe:ns:meta/\"><rdf:RDF xmlns:rdf=\"{NS_RDF}\"></rdf:RDF></x:xmpmeta>"
        ),
    };
    let output = merge_with_paths(&xml, rating, culling, favorite, &tags, Some(&paths), &description, ALL)?;
    let temp = dest.with_file_name(format!(
        ".lap-xmp-{}-{}.tmp",
        std::process::id(),
        SERIAL.fetch_add(1, Ordering::Relaxed)
    ));
    let mut file = fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&temp)
        .map_err(|e| e.to_string())?;
    let result = (|| {
        file.write_all(&encode_xmp_text(&output, before.as_deref()))
            .map_err(|e| e.to_string())?;
        if before.is_some() {
            file.set_permissions(
                fs::metadata(&dest)
                    .map_err(|e| e.to_string())?
                    .permissions(),
            )
            .map_err(|e| e.to_string())?;
        }
        file.sync_all().map_err(|e| e.to_string())?;
        drop(file);
        publish(&temp, &dest, before.as_deref())?;
        Ok(())
    })();
    let _ = fs::remove_file(&temp);
    if result.is_ok() {
        cache.published(&dest);
    }
    result
}

/// Optimistic external-change check immediately before publishing the complete packet.
fn publish(temp: &Path, dest: &Path, before: Option<&[u8]>) -> Result<(), String> {
    if let Some(original) = before {
        if bounded_read(dest)? != original {
            return Err(
                "Sidecar changed externally during export; retry after reviewing it".into(),
            );
        }
        fs::rename(temp, dest).map_err(|e| e.to_string())?;
    } else {
        crate::t_batch_rename::rename_exclusive(temp, dest)?;
    }
    Ok(())
}

fn failure(conn: &Connection, file_id: i64, error: String) -> Failure {
    let name: String = conn
        .query_row("SELECT name FROM afiles WHERE id=?1", [file_id], |r| {
            r.get(0)
        })
        .unwrap_or_else(|_| format!("File #{file_id}"));
    Failure {
        file_id,
        error: format!("{name}: {error}"),
    }
}

pub fn save_batch(conn: &Connection, ids: &[i64]) -> BatchResult {
    let mut result = BatchResult::default();
    let Ok(_guard) = WRITE_LOCK.lock() else {
        result.failures.push(Failure {
            file_id: 0,
            error: "XMP writer unavailable".into(),
        });
        return result;
    };
    let mut cache = ExportIndex::default();
    for &id in ids {
        match write_one(conn, id, &mut cache) {
            Ok(()) => result.saved_ids.push(id),
            Err(error) => result.failures.push(failure(conn, id, error)),
        }
    }
    result
}

#[cfg(test)]
mod import_tests {
    use super::*;
    #[derive(Debug, Default, Serialize)]
    #[serde(rename_all = "camelCase")]
    struct XmpBatchResult {
        pub imported_ids: Vec<i64>,
        pub skipped_ids: Vec<i64>,
        pub failures: Vec<XmpImportFailure>,
    }
    #[derive(Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    struct XmpImportFailure {
        pub file_id: i64,
        pub path: String,
        pub error: String,
    }
    fn import_batch(files: &[(i64, String)], conn: &Connection) -> XmpBatchResult {
        static GENERATION: std::sync::atomic::AtomicI64 = std::sync::atomic::AtomicI64::new(-1);
        let _scan = sidecar_scan(GENERATION.fetch_sub(1, Ordering::Relaxed));
        let mut result = XmpBatchResult::default();
        for (id, path) in files {
            match import_xmp_for_file_with_conn(*id, path, conn, XmpImportMode::ManualOverwrite) {
                Ok(true) => result.imported_ids.push(*id),
                Ok(false) => result.skipped_ids.push(*id),
                Err(error) => result.failures.push(XmpImportFailure {
                    file_id: *id,
                    path: path.clone(),
                    error,
                }),
            }
        }
        result
    }

    use std::time::{SystemTime, UNIX_EPOCH};

    fn create_temp_test_dir(label: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("lap_xmp_test_{}_{}", label, nanos));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn setup_in_memory_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE afiles (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                rating INTEGER NOT NULL DEFAULT 0,
                culling_flag INTEGER NOT NULL DEFAULT 0,
                is_favorite INTEGER DEFAULT 0,
                comments TEXT,
                has_tags INTEGER DEFAULT 0
            );
            CREATE TABLE atags (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                parent_id INTEGER REFERENCES atags(id) ON DELETE CASCADE
            );
            CREATE TABLE afile_tags (
                file_id INTEGER NOT NULL,
                tag_id INTEGER NOT NULL,
                UNIQUE(file_id, tag_id)
            );",
        )
        .unwrap();
        initialize(&conn).unwrap();
        conn
    }

    #[test]
    fn embedded_import_sidecar_priority_and_local_clear() {
        let conn = setup_in_memory_db();
        file(&conn);
        let dir = create_temp_test_dir("embedded");
        let image = dir.join("photo.jpg");
        fs::write(&image, crate::t_metadata::tests::jpeg_with_xmp()).unwrap();
        let path = image.to_str().unwrap();
        import_xmp_for_file_with_conn(1, path, &conn, XmpImportMode::InitialScan).unwrap();
        let rating = || {
            conn.query_row("SELECT rating FROM afiles WHERE id=1", [], |r| {
                r.get::<_, i32>(0)
            })
            .unwrap()
        };
        assert_eq!(rating(), 4);
        let tags: i64 = conn
            .query_row("SELECT COUNT(*) FROM afile_tags WHERE file_id=1", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(tags, 2);
        fs::write(dir.join("photo.jpg.xmp"), format!(r#"<x:xmpmeta xmlns:x="adobe:ns:meta/" xmlns:rdf="{NS_RDF}" xmlns:xmp="{NS_XMP}"><rdf:RDF><rdf:Description xmp:Rating="5"/></rdf:RDF></x:xmpmeta>"#)).unwrap();
        import_xmp_for_file_with_conn(1, path, &conn, XmpImportMode::InitialScan).unwrap();
        assert_eq!(rating(), 5);
        // Local clears remain protected when a sidecar appears later.
        conn.execute("UPDATE afiles SET rating=0 WHERE id=1", [])
            .unwrap();
        fs::write(dir.join("photo.jpg.xmp"), format!(r#"<x:xmpmeta xmlns:x="adobe:ns:meta/" xmlns:rdf="{NS_RDF}" xmlns:xmp="{NS_XMP}"><rdf:RDF><rdf:Description xmp:Rating="5"/></rdf:RDF></x:xmpmeta>"#)).unwrap();
        import_xmp_for_file_with_conn(1, path, &conn, XmpImportMode::RescanNonDestructive).unwrap();
        assert_eq!(rating(), 0);
        // Manual import is explicitly sidecar-only and may overwrite a clear.
        import_xmp_for_file_with_conn(1, path, &conn, XmpImportMode::ManualOverwrite).unwrap();
        assert_eq!(rating(), 5);
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn test_sidecar_naming_conventions() {
        let dir = create_temp_test_dir("naming");
        let raw_path = dir.join("DSC0042.RAF");
        fs::write(&raw_path, b"dummy raw").unwrap();

        // 1. No sidecar initially
        assert_eq!(find_xmp_sidecar(&raw_path), None);

        // 2. <stem>.xmp convention (Lightroom / Camera Raw)
        let stem_xmp = dir.join("DSC0042.xmp");
        fs::write(&stem_xmp, b"<x:xmpmeta/>").unwrap();
        assert_eq!(find_xmp_sidecar(&raw_path), Some(stem_xmp.clone()));

        // 3. <filename>.<ext>.xmp convention (darktable) takes priority when both exist
        let ext_xmp = dir.join("DSC0042.RAF.xmp");
        fs::write(&ext_xmp, b"<x:xmpmeta/>").unwrap();
        assert_eq!(find_xmp_sidecar(&raw_path), Some(ext_xmp.clone()));

        // 4. Uppercase .XMP extension
        fs::remove_file(&stem_xmp).unwrap();
        fs::remove_file(&ext_xmp).unwrap();
        let upper_xmp = dir.join("DSC0042.XMP");
        fs::write(&upper_xmp, b"<x:xmpmeta/>").unwrap();
        assert!(find_xmp_sidecar(&raw_path).is_some());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_hierarchical_tags_and_internal_filtering() {
        let xmp = r#"<?xml version="1.0" encoding="UTF-8"?>
<x:xmpmeta xmlns:x="adobe:ns:meta/" x:xmptk="XMP Core 4.4.0-Exiv2">
 <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
  <rdf:Description rdf:about=""
    xmlns:xmp="http://ns.adobe.com/xap/1.0/"
    xmlns:dc="http://purl.org/dc/elements/1.1/"
    xmlns:lr="http://ns.adobe.com/lightroom/1.0/"
    xmp:Rating="4">
   <dc:subject>
    <rdf:Bag>
     <rdf:li>People</rdf:li>
     <rdf:li>Portrait &amp; Studio</rdf:li>
    </rdf:Bag>
   </dc:subject>
   <lr:hierarchicalSubject>
    <rdf:Bag>
     <rdf:li>People</rdf:li>
     <rdf:li>People|Edith</rdf:li>
     <rdf:li>darktable|format|raf</rdf:li>
     <rdf:li>darktable|exported</rdf:li>
    </rdf:Bag>
   </lr:hierarchicalSubject>
  </rdf:Description>
 </rdf:RDF>
</x:xmpmeta>"#;

        let meta = parse_xmp_str(xmp).unwrap();
        assert_eq!(meta.rating, Some(4));
        assert_eq!(meta.culling_flag, None);
        assert_eq!(
            meta.tags,
            Some(vec![
                "People".to_string(),
                "Portrait & Studio".to_string(),
                "People|Edith".to_string(),
            ])
        );
    }

    #[test]
    fn test_missing_vs_explicitly_empty_values() {
        // Case A: All metadata properties omitted from XMP
        let missing_xmp = r#"<?xml version="1.0" encoding="UTF-8"?>
<x:xmpmeta xmlns:x="adobe:ns:meta/">
 <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
  <rdf:Description rdf:about=""
    xmlns:xmp="http://ns.adobe.com/xap/1.0/"
    xmlns:dc="http://purl.org/dc/elements/1.1/">
  </rdf:Description>
 </rdf:RDF>
</x:xmpmeta>"#;

        let missing_meta = parse_xmp_str(missing_xmp).unwrap();
        assert_eq!(missing_meta.rating, None);
        assert_eq!(missing_meta.culling_flag, None);
        assert_eq!(missing_meta.tags, None);
        assert_eq!(missing_meta.description, None);

        // Case B: Properties explicitly present but empty / zeroed
        let empty_xmp = r#"<?xml version="1.0" encoding="UTF-8"?>
<x:xmpmeta xmlns:x="adobe:ns:meta/">
 <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
  <rdf:Description rdf:about=""
    xmlns:xmp="http://ns.adobe.com/xap/1.0/"
    xmlns:xmpDM="http://ns.adobe.com/xmp/1.0/DynamicMedia/"
    xmlns:dc="http://purl.org/dc/elements/1.1/"
    xmp:Rating="0"
    xmpDM:pick="0">
   <dc:subject>
    <rdf:Bag/>
   </dc:subject>
   <dc:description>
    <rdf:Alt>
     <rdf:li xml:lang="x-default"></rdf:li>
    </rdf:Alt>
   </dc:description>
  </rdf:Description>
 </rdf:RDF>
</x:xmpmeta>"#;

        let empty_meta = parse_xmp_str(empty_xmp).unwrap();
        assert_eq!(empty_meta.rating, Some(0));
        assert_eq!(empty_meta.culling_flag, Some(0));
        assert_eq!(empty_meta.tags, Some(vec![]));
        assert_eq!(empty_meta.description, Some(String::new()));
    }

    #[test]
    fn test_namespace_prefix_resolution_and_culling_flags() {
        // Custom prefixes bound to canonical Adobe/Darktable URIs
        let xmp = r#"<x:xmpmeta xmlns:x="adobe:ns:meta/">
 <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
  <rdf:Description rdf:about=""
    xmlns:adobeXmp="http://ns.adobe.com/xap/1.0/"
    xmlns:dt="http://darktable.sf.net/"
    xmlns:dcore="http://purl.org/dc/elements/1.1/"
    adobeXmp:Rating="3">
   <dt:rejected>1</dt:rejected>
   <dcore:description>
    <rdf:Alt>
     <rdf:li xml:lang="fr">Version française</rdf:li>
     <rdf:li xml:lang="x-default">Default English caption</rdf:li>
    </rdf:Alt>
   </dcore:description>
  </rdf:Description>
 </rdf:RDF>
</x:xmpmeta>"#;

        let meta = parse_xmp_str(xmp).unwrap();
        assert_eq!(meta.rating, Some(3));
        assert_eq!(meta.culling_flag, Some(2)); // Rejected via dt:rejected
        assert_eq!(meta.description.as_deref(), Some("Default English caption"));
    }

    #[test]
    fn test_rescan_preserves_local_edits_in_lap() {
        let conn = setup_in_memory_db();
        conn.execute(
            "INSERT INTO afiles (id, name, rating, culling_flag, comments, has_tags) VALUES (1, 'DSC0001.RAF', 0, 0, NULL, 0)",
            [],
        )
        .unwrap();

        // 1. Initial scan imports metadata from XMP
        let initial_xmp = XmpMetadata {
            hierarchical_tags: None,
            favorite: None,
            rating: Some(2),
            culling_flag: Some(1), // Pick
            tags: Some(vec!["Landscape".to_string()]),
            description: Some("Initial XMP caption".to_string()),
        };
        apply_xmp_metadata_to_db(1, &initial_xmp, &conn, XmpImportMode::InitialScan).unwrap();

        let (rating, flag, comments): (i32, i32, Option<String>) = conn
            .query_row(
                "SELECT rating, culling_flag, comments FROM afiles WHERE id = 1",
                [],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .unwrap();
        assert_eq!(rating, 2);
        assert_eq!(flag, 1);
        assert_eq!(comments.as_deref(), Some("Initial XMP caption"));

        // 2. User edits metadata locally in Lap (changes rating to 5, flag to Reject (2), comment to "Local Lap edit", and adds "Favorite" tag)
        conn.execute(
            "UPDATE afiles SET rating = 5, culling_flag = 2, comments = 'Local Lap edit' WHERE id = 1",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO atags (id, name) VALUES (10, 'Favorite')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO afile_tags (file_id, tag_id) VALUES (1, 10)",
            [],
        )
        .unwrap();

        // 3. Background rescan runs on an updated sidecar (e.g. darktable saved exposure history with old Rating=2, Flag=1, and added "Mountains" tag)
        let rescan_xmp = XmpMetadata {
            hierarchical_tags: None,
            favorite: None,
            rating: Some(2),
            culling_flag: Some(1),
            tags: Some(vec!["Landscape".to_string(), "Mountains".to_string()]),
            description: Some("Initial XMP caption".to_string()),
        };
        apply_xmp_metadata_to_db(1, &rescan_xmp, &conn, XmpImportMode::RescanNonDestructive)
            .unwrap();

        // Verify local edits in Lap were NOT overwritten by the rescan
        let (rating_after, flag_after, comments_after): (i32, i32, Option<String>) = conn
            .query_row(
                "SELECT rating, culling_flag, comments FROM afiles WHERE id = 1",
                [],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .unwrap();
        assert_eq!(rating_after, 5);
        assert_eq!(flag_after, 2);
        assert_eq!(comments_after.as_deref(), Some("Local Lap edit"));

        // Local edits to the tag set prevent automatic additions, including restoring deleted tags.
        let mut stmt = conn
            .prepare(
                "SELECT t.name FROM atags t JOIN afile_tags ft ON t.id = ft.tag_id WHERE ft.file_id = 1 ORDER BY t.name",
            )
            .unwrap();
        let tag_names: Vec<String> = stmt
            .query_map([], |r| r.get(0))
            .unwrap()
            .map(|r| r.unwrap())
            .collect();
        assert_eq!(
            tag_names,
            vec!["Favorite".to_string(), "Landscape".to_string(),]
        );

        // 5. Explicit manual "Import from XMP" overwrites present values from sidecar
        apply_xmp_metadata_to_db(1, &rescan_xmp, &conn, XmpImportMode::ManualOverwrite).unwrap();
        let (manual_rating, manual_flag, manual_comments): (i32, i32, Option<String>) = conn
            .query_row(
                "SELECT rating, culling_flag, comments FROM afiles WHERE id = 1",
                [],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .unwrap();
        assert_eq!(manual_rating, 2);
        assert_eq!(manual_flag, 1);
        assert_eq!(manual_comments.as_deref(), Some("Initial XMP caption"));
    }
    fn xml(body: &str) -> String {
        format!(
            r#"<rdf:RDF xmlns:rdf="{NS_RDF}"><rdf:Description xmlns:xmp="{NS_XMP}" xmlns:dc="{NS_DC}">{body}</rdf:Description></rdf:RDF>"#
        )
    }
    #[test]
    fn parses_xml_text_and_ignores_comments() {
        let meta = parse_xmp_str(&xml(r#"<!-- xmp:Rating="5" --><xmp:Rating>2</xmp:Rating><dc:description><![CDATA[A < B]]> &amp; caf&#233;</dc:description>"#)).unwrap();
        assert_eq!(meta.rating, Some(2));
        assert_eq!(meta.description.as_deref(), Some("A < B & café"));
        let meta = parse_xmp_str(&format!(
            r#"<rdf:Description xmlns:rdf="{NS_RDF}" xmlns:xmp="{NS_XMP}" xmp:Rating = "5"/>"#
        ))
        .unwrap();
        assert_eq!(meta.rating, Some(5));
    }
    #[test]
    fn namespaces_are_scoped_and_language_is_exact() {
        let meta = parse_xmp_str(&xml(r#"<dc:description><rdf:Alt><rdf:li note="x-default" xml:lang="fr">French</rdf:li><rdf:li xml:lang="x-default">Default</rdf:li></rdf:Alt></dc:description><other xmlns:xmp="urn:other" xmp:Rating="5"/><xmp:Rating>3</xmp:Rating>"#)).unwrap();
        assert_eq!(meta.rating, Some(3));
        assert_eq!(meta.description.as_deref(), Some("Default"));
    }
    #[test]
    fn malformed_xml_is_an_error() {
        for input in [
            "",
            "<root>",
            "<root><a></root>",
            "<a/><b/>",
            "<root>&unknown;</root>",
        ] {
            assert!(parse_xmp_str(input).is_err(), "{input}");
        }
    }
    fn file(conn: &Connection) {
        conn.execute("INSERT INTO afiles(id,name) VALUES(1,'photo')", [])
            .unwrap();
    }
    #[test]
    fn rescan_protects_clears_and_removed_tags() {
        let conn = setup_in_memory_db();
        file(&conn);
        let meta = XmpMetadata {
            hierarchical_tags: None,
            favorite: None,
            rating: Some(4),
            culling_flag: Some(1),
            description: Some("caption".into()),
            tags: Some(vec!["tag".into()]),
        };
        apply_xmp_metadata_to_db(1, &meta, &conn, XmpImportMode::InitialScan).unwrap();
        conn.execute(
            "UPDATE afiles SET rating=0,culling_flag=0,comments=NULL WHERE id=1",
            [],
        )
        .unwrap();
        conn.execute("DELETE FROM afile_tags WHERE file_id=1", [])
            .unwrap();
        apply_xmp_metadata_to_db(1, &meta, &conn, XmpImportMode::RescanNonDestructive).unwrap();
        let state: (i32, i32, Option<String>) = conn
            .query_row(
                "SELECT rating,culling_flag,comments FROM afiles WHERE id=1",
                [],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .unwrap();
        assert_eq!(state, (0, 0, None));
        assert_eq!(
            conn.query_row("SELECT COUNT(*) FROM afile_tags", [], |r| r
                .get::<_, i32>(0))
                .unwrap(),
            0
        );
    }
    #[test]
    fn manual_tags_replace_missing_preserves_and_empty_clears() {
        let conn = setup_in_memory_db();
        file(&conn);
        for name in ["old", "new"] {
            apply_xmp_metadata_to_db(
                1,
                &XmpMetadata {
                    tags: Some(vec![name.into()]),
                    ..Default::default()
                },
                &conn,
                XmpImportMode::ManualOverwrite,
            )
            .unwrap();
        }
        apply_xmp_metadata_to_db(
            1,
            &XmpMetadata::default(),
            &conn,
            XmpImportMode::ManualOverwrite,
        )
        .unwrap();
        let tags: String = conn
            .query_row(
                "SELECT group_concat(name) FROM atags JOIN afile_tags ON tag_id=atags.id",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(tags, "new");
        apply_xmp_metadata_to_db(
            1,
            &XmpMetadata {
                tags: Some(vec![]),
                ..Default::default()
            },
            &conn,
            XmpImportMode::ManualOverwrite,
        )
        .unwrap();
        assert_eq!(
            conn.query_row("SELECT has_tags FROM afiles", [], |r| r.get::<_, i32>(0))
                .unwrap(),
            0
        );
    }
    #[test]
    fn failure_rolls_back_all_fields_and_dirty_tracking() {
        let conn = setup_in_memory_db();
        file(&conn);
        conn.execute_batch("CREATE TRIGGER fail_tag BEFORE INSERT ON atags BEGIN SELECT RAISE(ABORT,'test failure'); END;").unwrap();
        let meta = XmpMetadata {
            rating: Some(5),
            tags: Some(vec!["fail".into()]),
            ..Default::default()
        };
        assert!(apply_xmp_metadata_to_db(1, &meta, &conn, XmpImportMode::ManualOverwrite).is_err());
        assert_eq!(
            conn.query_row("SELECT rating FROM afiles", [], |r| r.get::<_, i32>(0))
                .unwrap(),
            0
        );
        assert_eq!(
            conn.query_row("SELECT COUNT(*) FROM axmp_imports", [], |r| r
                .get::<_, i32>(0))
                .unwrap(),
            0
        );
    }
    #[test]
    fn existing_sidecars_import_independently_of_photo_scan_time() {
        let conn = setup_in_memory_db();
        file(&conn);
        let dir = create_temp_test_dir("state");
        let image = dir.join("photo.RAF");
        let sidecar = dir.join("photo.RAF.xmp");
        fs::write(&sidecar, xml("<xmp:Rating>4</xmp:Rating>")).unwrap();
        let path = image.to_str().unwrap();
        assert!(
            import_xmp_for_file_with_conn(1, path, &conn, XmpImportMode::RescanNonDestructive)
                .unwrap()
        );
        assert!(
            !import_xmp_for_file_with_conn(1, path, &conn, XmpImportMode::RescanNonDestructive)
                .unwrap()
        );
        conn.execute("UPDATE afiles SET rating=0 WHERE id=1", [])
            .unwrap();
        fs::write(&sidecar, xml("<xmp:Rating>5</xmp:Rating><!-- changed -->")).unwrap();
        assert!(
            import_xmp_for_file_with_conn(1, path, &conn, XmpImportMode::RescanNonDestructive)
                .unwrap()
        );
        assert_eq!(
            conn.query_row("SELECT rating FROM afiles", [], |r| r.get::<_, i32>(0))
                .unwrap(),
            0
        );
        fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn mixed_case_full_name_takes_priority_and_negative_cache_invalidates() {
        let dir = create_temp_test_dir("cache");
        let image = dir.join("photo.RAF");
        assert!(find_xmp_sidecar(&image).is_none());
        let stem = dir.join("photo.xmp");
        fs::write(&stem, "").unwrap();
        assert_eq!(find_xmp_sidecar(&image), Some(stem));
        let full = dir.join("photo.RAF.Xmp");
        fs::write(&full, "").unwrap();
        assert_eq!(
            fs::canonicalize(find_xmp_sidecar(&image).unwrap()).unwrap(),
            fs::canonicalize(full).unwrap()
        );
        fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn photo_case_is_preserved() {
        let dir = create_temp_test_dir("case");
        fs::write(dir.join("A.RAF.Xmp"), "").unwrap();
        assert!(find_xmp_sidecar(&dir.join("a.RAF")).is_none());
        assert!(find_xmp_sidecar(&dir.join("A.RAF")).is_some());
        fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn new_scan_rebuilds_index_without_directory_timestamp() {
        let dir = create_temp_test_dir("generation");
        let image = dir.join("photo.RAF");
        {
            let _scan = sidecar_scan(101);
            assert!(find_xmp_sidecar(&image).is_none());
            fs::write(dir.join("photo.Xmp"), "").unwrap();
            assert!(find_xmp_sidecar(&image).is_none());
        }
        {
            let _scan = sidecar_scan(102);
            assert!(find_xmp_sidecar(&image).is_some());
        }
        fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn rejects_resource_exhaustion() {
        let deep = format!(
            "{}{}",
            "<a>".repeat(MAX_XML_DEPTH + 1),
            "</a>".repeat(MAX_XML_DEPTH + 1)
        );
        assert!(parse_xmp_str(&deep).unwrap_err().contains("depth"));
        let wide = format!("<root>{}</root>", "<a/>".repeat(MAX_XML_NODES));
        assert!(parse_xmp_str(&wide).unwrap_err().contains("node"));
        assert!(
            parse_xmp_str(&" ".repeat(MAX_XMP_BYTES + 1))
                .unwrap_err()
                .contains("size")
        );
        let amplified = format!(
            "<root xmlns:n='{}'>{}</root>",
            "x".repeat(1024 * 1024),
            "<n:a/>".repeat(40)
        );
        assert!(parse_xmp_str(&amplified).unwrap_err().contains("expanded"));
        let dir = create_temp_test_dir("size");
        let path = dir.join("huge.xmp");
        fs::File::create(&path)
            .unwrap()
            .set_len(MAX_XMP_BYTES as u64 + 1)
            .unwrap();
        assert!(read_xmp_file(&path).is_err());
        fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn batch_continues_after_failure_and_reports_each_result() {
        let conn = setup_in_memory_db();
        let dir = create_temp_test_dir("batch");
        let mut files = Vec::new();
        for id in 1..=4 {
            conn.execute(
                "INSERT INTO afiles(id,name) VALUES(?1,?2)",
                params![id, format!("p{id}")],
            )
            .unwrap();
            let path = dir.join(format!("p{id}.RAF"));
            if id != 4 {
                fs::write(
                    dir.join(format!("p{id}.xmp")),
                    if id == 2 {
                        "<broken>".into()
                    } else {
                        xml("<xmp:Rating>5</xmp:Rating>")
                    },
                )
                .unwrap();
            }
            files.push((id, path.to_string_lossy().into_owned()));
        }
        let result = import_batch(&files, &conn);
        assert_eq!(result.imported_ids, vec![1, 3]);
        assert_eq!(result.skipped_ids, vec![4]);
        assert_eq!(result.failures.len(), 1);
        assert_eq!(result.failures[0].file_id, 2);
        assert_eq!(
            conn.query_row("SELECT rating FROM afiles WHERE id=2", [], |r| r
                .get::<_, i32>(0))
                .unwrap(),
            0
        );
        fs::remove_dir_all(dir).unwrap();
    }
}

#[cfg(test)]
mod export_tests {
    use super::*;
    fn merge(
        xml: &str,
        rating: i32,
        tags: &[String],
        description: &str,
        mask: i32,
    ) -> Result<String, String> {
        merge_with_culling(xml, rating, 0, tags, description, mask)
    }
    fn packet(body: &str) -> String {
        format!(
            "<x:xmpmeta xmlns:x=\"adobe:ns:meta/\"><rdf:RDF xmlns:rdf=\"{NS_RDF}\"><rdf:Description xmlns:d=\"{NS_DC}\" xmlns:r=\"{NS_XMP}\" xmlns:crs=\"http://ns.adobe.com/camera-raw-settings/1.0/\" rdf:about=\"\">{body}</rdf:Description></rdf:RDF></x:xmpmeta>"
        )
    }
    fn fixture() -> (Connection, PathBuf, PathBuf) {
        let folder = std::env::temp_dir().join(format!(
            "lap-xmp-export-{}-{}",
            std::process::id(),
            SERIAL.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&folder).unwrap();
        let media = folder.join("photo.jpg");
        fs::write(&media, b"unchanged-original").unwrap();
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("CREATE TABLE afolders(id INTEGER PRIMARY KEY,path TEXT); INSERT INTO afolders(id) VALUES(1);
            CREATE TABLE afiles(id INTEGER PRIMARY KEY,folder_id INTEGER DEFAULT 1,name TEXT DEFAULT 'photo.jpg',is_favorite INTEGER DEFAULT 0,rating INTEGER DEFAULT 0,culling_flag INTEGER DEFAULT 0,comments TEXT,has_tags INTEGER DEFAULT 0);
            CREATE TABLE atags(id INTEGER PRIMARY KEY,name TEXT,parent_id INTEGER REFERENCES atags(id) ON DELETE CASCADE);CREATE TABLE afile_tags(file_id INTEGER,tag_id INTEGER,UNIQUE(file_id,tag_id));
            INSERT INTO afiles(id,rating,comments) VALUES(1,4,'local caption'); INSERT INTO atags(id,name) VALUES(1,'旅行, travel & <test>');INSERT INTO afile_tags VALUES(1,1);").unwrap();
        conn.execute(
            "UPDATE afolders SET path=?1 WHERE id=1",
            [folder.to_str().unwrap()],
        )
        .unwrap();
        initialize(&conn).unwrap();
        (conn, folder, media)
    }
    #[test]
    fn merge_keeps_unknown_xml_hierarchy_and_other_languages() {
        let unknown = "<crs:Exposure2012>+0.50</crs:Exposure2012><!--keep--><lr:hierarchicalSubject xmlns:lr=\"http://ns.adobe.com/lightroom/1.0/\"><rdf:Bag><rdf:li>Places|Paris</rdf:li></rdf:Bag></lr:hierarchicalSubject>";
        let xml = packet(&format!(
            "<r:Rating>2</r:Rating><d:subject><rdf:Bag><rdf:li>old</rdf:li></rdf:Bag></d:subject><d:description><rdf:Alt><rdf:li xml:lang=\"fr\">Français &amp; texte</rdf:li><rdf:li xml:lang=\"x-default\">old</rdf:li></rdf:Alt></d:description>{unknown}"
        ));
        let out = merge(&xml, 5, &["A,B & <C>".into()], "新备注 & <test>", ALL).unwrap();
        assert!(out.contains("<crs:Exposure2012>+0.50</crs:Exposure2012><!--keep-->"));
        assert!(!out.contains("Places|Paris"));
        assert!(out.contains("<rdf:li xml:lang=\"fr\">Français &amp; texte</rdf:li>"));
        let parsed = parse_xmp_str(&out).unwrap();
        assert_eq!(parsed.rating, Some(5));
        assert_eq!(parsed.description.as_deref(), Some("新备注 & <test>"));
        assert!(parsed.tags.unwrap().contains(&"A,B & <C>".into()));
    }
    #[test]
    fn shorthand_and_explicit_clears() {
        let xml = format!(
            "<rdf:RDF xmlns:rdf=\"{NS_RDF}\"><rdf:Description xmlns:d=\"{NS_DC}\" xmlns:r=\"{NS_XMP}\" r:Rating=\"5\" d:description=\"old\" d:subject=\"old\" custom=\"untouched\"/></rdf:RDF>"
        );
        let out = merge(&xml, 0, &[], "", ALL).unwrap();
        assert!(out.contains("custom=\"untouched\""));
        let m = parse_xmp_str(&out).unwrap();
        assert_eq!(m.rating, Some(0));
        assert_eq!(m.tags, Some(vec![]));
        assert_eq!(m.description, Some(String::new()));
    }
    #[test]
    fn empty_rdf_and_partial_update() {
        let out = merge(&format!("<rdf:RDF xmlns:rdf=\"{NS_RDF}\"/>"), 3, &[], "", 1).unwrap();
        assert_eq!(parse_xmp_str(&out).unwrap().rating, Some(3));
        let out = merge(
            &packet("<r:Rating>4</r:Rating><d:description>original</d:description>"),
            0,
            &["new".into()],
            "",
            8,
        )
        .unwrap();
        let m = parse_xmp_str(&out).unwrap();
        assert_eq!(m.rating, Some(4));
        assert_eq!(m.description.as_deref(), Some("original"));
        assert!(merge("<!DOCTYPE x><x/>", 0, &[], "", ALL).is_err());
        assert!(
            merge(
                &packet("<r:Rating>2</r:Rating>")
                    .replace("rdf:about=\"\"", "rdf:about=\"file:///photo.jpg\""),
                3,
                &[],
                "",
                ALL
            )
            .is_err()
        );
    }
    #[test]
    fn manual_export_round_trip_preserves_original_and_reports_failures() {
        let (conn, folder, media) = fixture();
        let result = save_batch(&conn, &[999, 1]);
        assert_eq!(result.saved_ids, vec![1]);
        assert_eq!(result.failures.len(), 1);
        let sidecar = folder.join("photo.jpg.xmp");
        let m = read_xmp_file(&sidecar).unwrap();
        assert_eq!(m.rating, Some(4));
        assert_eq!(m.description.as_deref(), Some("local caption"));
        assert_eq!(m.tags.unwrap(), vec!["旅行, travel & <test>"]);
        assert_eq!(fs::read(&media).unwrap(), b"unchanged-original");
        fs::write(&sidecar, b"malformed XML").unwrap();
        assert_eq!(save_batch(&conn, &[1]).failures.len(), 1);
        assert_eq!(fs::read(&sidecar).unwrap(), b"malformed XML");
        assert!(
            !fs::read_dir(&folder).unwrap().any(|e| e
                .unwrap()
                .file_name()
                .to_string_lossy()
                .ends_with(".tmp"))
        );
        fs::remove_dir_all(folder).unwrap();
    }
    #[test]
    fn shared_stem_is_rejected_and_full_name_wins() {
        let (conn, folder, _) = fixture();
        fs::write(folder.join("photo.xmp"), packet("")).unwrap();
        fs::write(folder.join("photo.raw"), b"raw").unwrap();
        assert_eq!(save_batch(&conn, &[1]).failures.len(), 1);
        fs::write(folder.join("photo.jpg.XMP"), packet("")).unwrap();
        assert_eq!(save_batch(&conn, &[1]).saved_ids, vec![1]);
        assert_eq!(
            fs::read_to_string(folder.join("photo.xmp")).unwrap(),
            packet("")
        );
        fs::remove_dir_all(folder).unwrap();
    }
    #[test]
    fn local_edits_and_import_never_write_sidecars() {
        let (conn, folder, media) = fixture();
        conn.execute(
            "UPDATE afiles SET rating=0,culling_flag=2,comments='local edit' WHERE id=1",
            [],
        )
        .unwrap();
        conn.execute("UPDATE atags SET name='renamed' WHERE id=1", [])
            .unwrap();
        assert!(!folder.join("photo.jpg.xmp").exists());
        let external = packet("<r:Rating>2</r:Rating>");
        fs::write(folder.join("photo.jpg.xmp"), &external).unwrap();
        import_xmp_for_file_with_conn(
            1,
            media.to_str().unwrap(),
            &conn,
            XmpImportMode::RefreshNonDestructive,
        )
        .unwrap();
        assert_eq!(
            fs::read_to_string(folder.join("photo.jpg.xmp")).unwrap(),
            external
        );
        assert_eq!(save_batch(&conn, &[1]).saved_ids, vec![1]);
        let saved = read_xmp_file(&folder.join("photo.jpg.xmp")).unwrap();
        assert_eq!(saved.rating, Some(0));
        assert_eq!(saved.culling_flag, Some(2));
        assert_eq!(saved.description.as_deref(), Some("local edit"));
        assert_eq!(saved.tags, Some(vec!["renamed".into()]));
        fs::remove_dir_all(folder).unwrap();
    }

    #[test]
    fn failed_manual_export_can_be_retried() {
        let (conn, folder, _) = fixture();
        fs::write(folder.join("photo.jpg.xmp"), "broken").unwrap();
        assert_eq!(save_batch(&conn, &[1]).failures.len(), 1);
        assert_eq!(
            fs::read_to_string(folder.join("photo.jpg.xmp")).unwrap(),
            "broken"
        );
        fs::remove_file(folder.join("photo.jpg.xmp")).unwrap();
        assert_eq!(save_batch(&conn, &[1]).saved_ids, vec![1]);
        fs::remove_dir_all(folder).unwrap();
    }

    #[test]
    fn forced_refresh_bypasses_fingerprint_but_protects_local_clear() {
        let (conn, folder, media) = fixture();
        conn.execute("UPDATE afiles SET rating=0 WHERE id=1", [])
            .unwrap();
        fs::write(
            folder.join("photo.jpg.xmp"),
            packet("<r:Rating>5</r:Rating>"),
        )
        .unwrap();
        import_xmp_for_file_with_conn(
            1,
            media.to_str().unwrap(),
            &conn,
            XmpImportMode::RescanNonDestructive,
        )
        .unwrap();
        conn.execute("UPDATE axmp_imports SET dirty=0 WHERE file_id=1", [])
            .unwrap(); // simulate an unedited empty field
        assert!(
            !import_xmp_for_file_with_conn(
                1,
                media.to_str().unwrap(),
                &conn,
                XmpImportMode::RescanNonDestructive
            )
            .unwrap()
        );
        assert!(
            import_xmp_for_file_with_conn(
                1,
                media.to_str().unwrap(),
                &conn,
                XmpImportMode::RefreshNonDestructive
            )
            .unwrap()
        );
        conn.execute("UPDATE afiles SET rating=0 WHERE id=1", [])
            .unwrap();
        import_xmp_for_file_with_conn(
            1,
            media.to_str().unwrap(),
            &conn,
            XmpImportMode::RefreshNonDestructive,
        )
        .unwrap();
        assert_eq!(
            conn.query_row("SELECT rating FROM afiles WHERE id=1", [], |r| r
                .get::<_, i32>(0))
                .unwrap(),
            0
        );
        conn.execute("DELETE FROM afile_tags WHERE file_id=1", [])
            .unwrap();
        conn.execute("UPDATE afiles SET comments=NULL WHERE id=1", [])
            .unwrap();
        fs::write(folder.join("photo.jpg.xmp"), packet("<d:subject><rdf:Bag><rdf:li>external</rdf:li></rdf:Bag></d:subject><d:description>external</d:description>")).unwrap();
        import_xmp_for_file_with_conn(
            1,
            media.to_str().unwrap(),
            &conn,
            XmpImportMode::RefreshNonDestructive,
        )
        .unwrap();
        assert_eq!(
            conn.query_row("SELECT count(*) FROM afile_tags WHERE file_id=1", [], |r| r
                .get::<_, i32>(0))
                .unwrap(),
            0
        );
        assert_eq!(
            conn.query_row("SELECT comments FROM afiles WHERE id=1", [], |r| r
                .get::<_, Option<String>>(0))
                .unwrap(),
            None
        );
        fs::remove_dir_all(folder).unwrap();
    }

    #[test]
    fn external_change_or_concurrent_creation_is_not_overwritten() {
        let (_conn, folder, _) = fixture();
        let dest = folder.join("test.xmp");
        let temp = folder.join("staged.tmp");
        fs::write(&dest, b"external change").unwrap();
        fs::write(&temp, b"new export").unwrap();
        assert!(publish(&temp, &dest, Some(b"old snapshot")).is_err());
        assert!(publish(&temp, &dest, None).is_err());
        assert_eq!(fs::read(&dest).unwrap(), b"external change");
        assert!(publish(&temp, &dest, Some(b"external change")).is_ok());
        assert_eq!(fs::read(&dest).unwrap(), b"new export");
        fs::remove_dir_all(folder).unwrap();
    }
    #[cfg(unix)]
    #[test]
    fn sidecar_symlink_is_rejected_without_touching_target() {
        let (conn, folder, _) = fixture();
        let target = folder.join("target.xml");
        fs::write(&target, packet("")).unwrap();
        std::os::unix::fs::symlink(&target, folder.join("photo.jpg.xmp")).unwrap();
        assert_eq!(save_batch(&conn, &[1]).failures.len(), 1);
        assert_eq!(fs::read_to_string(&target).unwrap(), packet(""));
        fs::remove_dir_all(folder).unwrap();
    }
    #[test]
    fn legacy_auto_write_schema_is_removed_without_losing_local_values() {
        let (conn, folder, _) = fixture();
        conn.execute_batch("CREATE TABLE axmp_write_control(id INTEGER PRIMARY KEY,enabled INTEGER);INSERT INTO axmp_write_control VALUES(1,1);
            CREATE TABLE axmp_writes(file_id INTEGER PRIMARY KEY,mask INTEGER,revision INTEGER,error TEXT);INSERT INTO axmp_writes VALUES(1,15,1,NULL);
            CREATE TRIGGER axmp_write_rating AFTER UPDATE OF rating ON afiles BEGIN INSERT OR REPLACE INTO axmp_writes VALUES(NEW.id,1,1,NULL);END;").unwrap();
        initialize(&conn).unwrap();
        initialize(&conn).unwrap();
        let legacy:i64=conn.query_row("SELECT count(*) FROM sqlite_master WHERE name IN ('axmp_write_control','axmp_writes','axmp_write_rating')",[],|r|r.get(0)).unwrap();
        assert_eq!(legacy, 0);
        conn.execute("UPDATE afiles SET rating=5 WHERE id=1", [])
            .unwrap();
        assert_eq!(
            conn.query_row(
                "SELECT dirty & 1 FROM axmp_imports WHERE file_id=1",
                [],
                |r| r.get::<_, i32>(0)
            )
            .unwrap(),
            1
        );
        assert!(!folder.join("photo.jpg.xmp").exists());
        assert_eq!(save_batch(&conn, &[1]).saved_ids, vec![1]);
        assert_eq!(
            read_xmp_file(&folder.join("photo.jpg.xmp")).unwrap().rating,
            Some(5)
        );
        fs::remove_dir_all(folder).unwrap();
    }

    #[test]
    fn independent_exiftool_reads_exported_values() {
        if std::env::var("LAP_TEST_EXIFTOOL").as_deref() != Ok("1") {
            return;
        }
        let (conn, folder, _) = fixture();
        assert_eq!(save_batch(&conn, &[1]).saved_ids, vec![1]);
        let result = std::process::Command::new("exiftool")
            .args(["-s3", "-Rating", "-Subject", "-Description"])
            .arg(folder.join("photo.jpg.xmp"))
            .output()
            .expect("ExifTool required with LAP_TEST_EXIFTOOL=1");
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        assert_eq!(
            String::from_utf8(result.stdout)
                .unwrap()
                .lines()
                .collect::<Vec<_>>(),
            vec!["4", "旅行, travel & <test>", "local caption"]
        );
        for (flag, pick) in [(1, "1"), (2, "-1"), (0, "0")] {
            conn.execute("UPDATE afiles SET culling_flag=?1 WHERE id=1", [flag])
                .unwrap();
            assert_eq!(save_batch(&conn, &[1]).saved_ids, vec![1]);
            let actual = std::process::Command::new("exiftool")
                .args(["-s3", "-n", "-XMP-xmp:Rating", "-XMP-xmpDM:Pick"])
                .arg(folder.join("photo.jpg.xmp"))
                .output()
                .unwrap();
            assert!(actual.status.success());
            assert_eq!(
                String::from_utf8(actual.stdout)
                    .unwrap()
                    .lines()
                    .collect::<Vec<_>>(),
                vec!["4", pick]
            );
        }
        fs::remove_dir_all(folder).unwrap();
    }

    #[test]
    fn culling_and_stars_round_trip_for_every_combination() {
        let legacy = format!(
            "<rdf:RDF xmlns:rdf=\"{NS_RDF}\"><rdf:Description xmlns:m=\"{NS_XMP_DM}\" xmlns:r=\"{NS_XMP}\" xmlns:dt=\"{NS_DARKTABLE}\" r:Rating=\"-1\" r:Label=\"Reject\" m:pick=\"-1\" m:good=\"False\" dt:rejected=\"1\"><m:good>False</m:good><r:Label>Pick</r:Label></rdf:Description></rdf:RDF>"
        );
        for rating in 0..=5 {
            for flag in 0..=2 {
                let output = merge_with_culling(&legacy, rating, flag, &[], "", ALL).unwrap();
                let parsed = parse_xmp_str(&output).unwrap();
                assert_eq!(parsed.rating, Some(rating));
                assert_eq!(parsed.culling_flag, Some(flag));
                assert!(!output.contains("<r:Label>Pick</r:Label>"));
                assert!(!output.contains("r:Label=\"Reject\""));
                let bytes = crate::t_metadata::tests::jpeg_with_xmp_packet(output.as_bytes());
                let path = std::env::temp_dir().join(format!(
                    "lap-culling-roundtrip-{}-{}.jpg",
                    std::process::id(),
                    SERIAL.fetch_add(1, Ordering::Relaxed)
                ));
                fs::write(&path, bytes).unwrap();
                let embedded = crate::t_metadata::read(&path).organization;
                assert_eq!(embedded.rating, Some(rating));
                assert_eq!(embedded.culling_flag, Some(flag));
                fs::remove_file(path).unwrap();
                if flag == 0 {
                    assert!(!output.contains("<xmpDM:good>"));
                }
            }
        }
        // A genuine color label is not a culling property.
        let output =
            merge_with_culling(&packet("<r:Label>Red</r:Label>"), 4, 1, &[], "", ALL).unwrap();
        assert!(output.contains("<r:Label>Red</r:Label>"));
    }
    #[test]
    fn culling_manual_export_and_local_unflag_are_protected() {
        let (conn, folder, media) = fixture();
        for flag in [1, 2, 0] {
            conn.execute("UPDATE afiles SET culling_flag=?1 WHERE id=1", [flag])
                .unwrap();
            assert_eq!(save_batch(&conn, &[1]).saved_ids, vec![1]);
            let parsed = read_xmp_file(&folder.join("photo.jpg.xmp")).unwrap();
            assert_eq!(parsed.culling_flag, Some(flag));
            assert_eq!(parsed.rating, Some(4));
            // Even a rating-only edit must retain the current flag.
            conn.execute("UPDATE afiles SET rating=3 WHERE id=1", [])
                .unwrap();
            assert_eq!(save_batch(&conn, &[1]).saved_ids, vec![1]);
            assert_eq!(
                read_xmp_file(&folder.join("photo.jpg.xmp"))
                    .unwrap()
                    .culling_flag,
                Some(flag)
            );
            conn.execute("UPDATE afiles SET rating=4 WHERE id=1", [])
                .unwrap();
            save_batch(&conn, &[1]);
        }
        fs::write(
            folder.join("photo.jpg.xmp"),
            packet("<r:Rating>-1</r:Rating>"),
        )
        .unwrap();
        import_xmp_for_file_with_conn(
            1,
            media.to_str().unwrap(),
            &conn,
            XmpImportMode::RefreshNonDestructive,
        )
        .unwrap();
        assert_eq!(
            conn.query_row("SELECT culling_flag FROM afiles WHERE id=1", [], |r| r
                .get::<_, i32>(0))
                .unwrap(),
            0
        );

        fs::remove_dir_all(folder).unwrap();
    }
    #[test]
    fn empty_description_alt_can_be_updated() {
        for alt in ["<rdf:Alt/>", "<rdf:Alt />", "<rdf:Alt></rdf:Alt>"] {
            let xml = packet(&format!("<d:description>{alt}</d:description>"));
            let output = merge_with_culling(&xml, 4, 1, &[], "new caption", ALL).unwrap();
            let parsed = parse_xmp_str(&output).unwrap();
            assert_eq!(parsed.description.as_deref(), Some("new caption"));
            assert_eq!(parsed.rating, Some(4));
            assert_eq!(parsed.culling_flag, Some(1));
        }
    }

    #[test]
    fn previous_import_version_backfills_culling_once() {
        let (conn, folder, media) = fixture();
        let xml = format!(
            "<rdf:RDF xmlns:rdf=\"{NS_RDF}\"><rdf:Description xmlns:dm=\"{NS_XMP_DM}\" dm:pick=\"1\"/></rdf:RDF>"
        );
        fs::write(
            &media,
            crate::t_metadata::tests::jpeg_with_xmp_packet(xml.as_bytes()),
        )
        .unwrap();
        let info = fs::metadata(&media).unwrap();
        let old = format!(
            "embedded-v1:{}:{:?}:{}:None:None",
            media.display(),
            info.modified().ok(),
            info.len()
        );
        conn.execute(
            "INSERT INTO axmp_imports(file_id,fingerprint,dirty) VALUES(1,?1,0)",
            [&old],
        )
        .unwrap();
        assert!(
            import_xmp_for_file_with_conn(
                1,
                media.to_str().unwrap(),
                &conn,
                XmpImportMode::RescanNonDestructive
            )
            .unwrap()
        );
        assert_eq!(
            conn.query_row("SELECT culling_flag FROM afiles WHERE id=1", [], |r| r
                .get::<_, i32>(0))
                .unwrap(),
            1
        );
        assert!(
            !import_xmp_for_file_with_conn(
                1,
                media.to_str().unwrap(),
                &conn,
                XmpImportMode::RescanNonDestructive
            )
            .unwrap()
        );
        // Even an old-version import must preserve an explicit local unflag.
        conn.execute("UPDATE afiles SET culling_flag=0 WHERE id=1", [])
            .unwrap();
        conn.execute(
            "UPDATE axmp_imports SET fingerprint=?1 WHERE file_id=1",
            [&old],
        )
        .unwrap();
        import_xmp_for_file_with_conn(
            1,
            media.to_str().unwrap(),
            &conn,
            XmpImportMode::RescanNonDestructive,
        )
        .unwrap();
        assert_eq!(
            conn.query_row("SELECT culling_flag FROM afiles WHERE id=1", [], |r| r
                .get::<_, i32>(0))
                .unwrap(),
            0
        );
        fs::remove_dir_all(folder).unwrap();
    }

    #[test]
    fn new_sidecar_is_published_by_exclusive_rename() {
        let (_conn, folder, _) = fixture();
        let temp = folder.join("staged.tmp");
        let dest = folder.join("new.xmp");
        fs::write(&temp, b"complete packet").unwrap();
        publish(&temp, &dest, None).unwrap();
        assert_eq!(fs::read(&dest).unwrap(), b"complete packet");
        assert!(
            !temp.exists(),
            "rename must move the file, not create a hard link"
        );
        fs::write(&temp, b"concurrent packet").unwrap();
        assert!(publish(&temp, &dest, None).is_err());
        assert_eq!(fs::read(&dest).unwrap(), b"complete packet");
        assert_eq!(fs::read(&temp).unwrap(), b"concurrent packet");
        fs::remove_dir_all(folder).unwrap();
    }

    #[test]
    fn sidecar_flag_precedence_and_namespace_aliases() {
        let (conn, folder, media) = fixture();
        let embedded = format!(
            "<rdf:RDF xmlns:rdf=\"{NS_RDF}\"><rdf:Description xmlns:xmpDM=\"{NS_XMP_DM}\" xmpDM:pick=\"1\"/></rdf:RDF>"
        );
        fs::write(
            &media,
            crate::t_metadata::tests::jpeg_with_xmp_packet(embedded.as_bytes()),
        )
        .unwrap();
        let sidecar = format!(
            "<rdf:RDF xmlns:rdf=\"{NS_RDF}\"><rdf:Description xmlns:alias=\"{NS_XMP_DM}\" alias:pick=\"-1\"/></rdf:RDF>"
        );
        fs::write(folder.join("photo.jpg.xmp"), sidecar).unwrap();
        import_xmp_for_file_with_conn(
            1,
            media.to_str().unwrap(),
            &conn,
            XmpImportMode::InitialScan,
        )
        .unwrap();
        assert_eq!(
            conn.query_row("SELECT culling_flag FROM afiles WHERE id=1", [], |r| r
                .get::<_, i32>(0))
                .unwrap(),
            2
        );

        fs::remove_dir_all(folder).unwrap();
    }
    #[test]
    fn cleared_tags_do_not_reappear_from_preserved_hierarchy() {
        let xml = format!(
            "<rdf:RDF xmlns:rdf=\"{NS_RDF}\"><rdf:Description xmlns:lr=\"{NS_LR}\"><lr:hierarchicalSubject><rdf:Bag><rdf:li>Places|Paris</rdf:li></rdf:Bag></lr:hierarchicalSubject></rdf:Description></rdf:RDF>"
        );
        let output = merge_with_culling(&xml, 0, 0, &[], "", ALL).unwrap();
        assert_eq!(parse_xmp_str(&output).unwrap().tags, Some(vec![]));
    }
    #[test]
    fn existing_acdsee_attributes_and_elements_follow_replacements_and_clears() {
        let categories = escape(r#"<Categories><Category Assigned="0">Places<Category Assigned="1">Paris</Category></Category></Categories>"#);
        for description in [
            format!("<rdf:Description xmlns:a=\"{NS_ACDSEE}\" a:categories=\"{categories}\"/>"),
            format!("<rdf:Description xmlns:a=\"{NS_ACDSEE}\"><a:categories>{categories}</a:categories></rdf:Description>"),
        ] {
            let xml = format!("<rdf:RDF xmlns:rdf=\"{NS_RDF}\">{description}</rdf:RDF>");
            let cleared = merge_with_paths(&xml, 0, 0, false, &[], Some(&[]), "", 8).unwrap();
            assert_eq!(parse_xmp_str(&cleared).unwrap().tags, Some(vec![]));
            let paths = vec!["Places|Rome".into()];
            let replaced = merge_with_paths(&xml, 0, 0, false, &["Rome".into()], Some(&paths), "", 8).unwrap();
            assert_eq!(parse_xmp_str(&replaced).unwrap().hierarchical_tags, Some(paths));
            assert!(!replaced.contains("Paris"));
        }
    }
    #[test]
    fn packet_readonly_marker_is_not_caption_text() {
        let xml = format!(
            "<rdf:RDF xmlns:rdf=\"{NS_RDF}\"><rdf:Description xmlns:dc=\"{NS_DC}\"><dc:description><![CDATA[end=\"r\"]]></dc:description></rdf:Description></rdf:RDF>"
        );
        assert!(merge_with_culling(&xml, 3, 0, &[], "", 1).is_ok());
        for ending in ["end='r'", "end = \"r\""] {
            assert!(
                merge_with_culling(&format!("{xml}<?xpacket {ending}?>"), 3, 0, &[], "", 1)
                    .is_err()
            );
        }
    }
    #[test]
    fn all_keyword_stores_follow_replacements_and_clears() {
        for shorthand in [true, false] {
            let properties = if shorthand {
                "l:hierarchicalSubject=\"old\" k:TagsList=\"old\""
            } else {
                ""
            };
            let body = if shorthand {
                ""
            } else {
                "<l:hierarchicalSubject><rdf:Bag><rdf:li>old</rdf:li></rdf:Bag></l:hierarchicalSubject><k:TagsList><rdf:Seq><rdf:li>old</rdf:li></rdf:Seq></k:TagsList>"
            };
            let xml = format!(
                "<rdf:RDF xmlns:rdf=\"{NS_RDF}\"><rdf:Description xmlns:l=\"{NS_LR}\" xmlns:k=\"{NS_DIGIKAM}\" {properties}>{body}</rdf:Description></rdf:RDF>"
            );
            for tags in [vec!["new & tag".to_string()], vec![]] {
                let merged = merge_with_culling(&xml, 0, 0, &tags, "", 8).unwrap();
                assert!(!merged.contains("old"));
                assert_eq!(parse_xmp_str(&merged).unwrap().tags, Some(tags));
                assert!(merged.contains("lr:hierarchicalSubject"));
                assert!(merged.contains("digiKam:TagsList"));
            }
            let rating_only = merge_with_culling(&xml, 4, 0, &[], "", 1).unwrap();
            assert!(rating_only.contains("old"));
        }
    }
    #[test]
    fn unmanaged_shorthand_attributes_survive_repeated_exports() {
        let xml = format!(
            "<rdf:RDF xmlns:rdf=\"{NS_RDF}\"><rdf:Description xmlns:xmp=\"{NS_XMP}\" xmlns:c=\"urn:custom?a=1&amp;b=2\" xmp:Rating=\"2\" c:value=\"A &amp; B &lt;C&gt; &quot;D&quot; &apos;E&apos; 照片\"/></rdf:RDF>"
        );
        let mut merged = xml;
        for _ in 0..3 {
            merged = merge_with_culling(&merged, 4, 0, &[], "", ALL).unwrap();
            let root = parse_xml(&merged).unwrap();
            let mut props = Vec::new();
            collect_properties(&root, &mut props);
            assert_eq!(
                props.iter().find(|n| n.name == "value").unwrap().value(),
                "A & B <C> \"D\" 'E' 照片"
            );
            assert!(!merged.contains("&amp;amp;"));
            assert!(merged.contains("xmlns:c=\"urn:custom?a=1&amp;b=2\""));
        }
    }
    #[test]
    fn darktable_compatibility_is_updated_only_when_present() {
        let base = format!("<rdf:RDF xmlns:rdf=\"{NS_RDF}\"><rdf:Description/></rdf:RDF>");
        assert!(
            !merge_with_culling(&base, 0, 2, &[], "", ALL)
                .unwrap()
                .contains("<darktable:rejected>")
        );
        for property in ["d:rejected=\"1\"", ""] {
            let body = if property.is_empty() {
                "<d:rejected>1</d:rejected>"
            } else {
                ""
            };
            let xml = format!(
                "<rdf:RDF xmlns:rdf=\"{NS_RDF}\"><rdf:Description xmlns:d=\"{NS_DARKTABLE}\" {property}>{body}</rdf:Description></rdf:RDF>"
            );
            let merged = merge_with_culling(&xml, 0, 0, &[], "", ALL).unwrap();
            assert!(merged.contains("<darktable:rejected>0</darktable:rejected>"));
            assert_eq!(parse_xmp_str(&merged).unwrap().culling_flag, Some(0));
        }
    }
    #[test]
    fn extended_values_override_same_standard_property() {
        let packet = |pick| {
            format!(
                "<rdf:RDF xmlns:rdf=\"{NS_RDF}\"><rdf:Description xmlns:d=\"{NS_XMP_DM}\" d:pick=\"{pick}\"/></rdf:RDF>"
            )
        };
        assert_eq!(culling_from_packets(&[packet(-1), packet(1)]), Some(1));
        assert_eq!(culling_from_packets(&[packet(1), packet(0)]), Some(0));
    }
    #[test]
    fn empty_description_is_removed_but_custom_attributes_survive() {
        for (attr, expected) in [("", 1), ("c:x=\"keep\"", 2)] {
            let xml = format!(
                "<rdf:RDF xmlns:rdf=\"{NS_RDF}\"><rdf:Description xmlns:xmp=\"{NS_XMP}\" xmlns:c=\"urn:custom\" xmp:Rating=\"2\" {attr}/></rdf:RDF>"
            );
            let merged = merge_with_culling(&xml, 4, 0, &[], "", ALL).unwrap();
            assert_eq!(merged.matches("<rdf:Description").count(), expected);
            if expected == 2 {
                assert!(merged.contains("c:x=\"keep\""));
            }
        }
    }

    #[test]
    fn export_index_updates_publications_and_checks_shared_stems() {
        let (_, dir, _) = fixture();
        fs::write(dir.join("pair.CR2"), b"raw").unwrap();
        fs::write(dir.join("pair.JPG"), b"jpeg").unwrap();
        fs::write(dir.join("pair.xmp"), packet("")).unwrap();
        let mut cache = ExportIndex::default();
        assert!(destination(&dir.join("pair.CR2"), &mut cache).is_err());
        let specific = dir.join("pair.CR2.XMP");
        fs::write(&specific, packet("")).unwrap();
        cache.published(&specific);
        assert_eq!(
            destination(&dir.join("pair.CR2"), &mut cache).unwrap(),
            specific
        );
        let media = dir.join("new.jpg");
        fs::write(&media, b"jpeg").unwrap();
        let new_sidecar = destination(&media, &mut cache).unwrap();
        fs::write(&new_sidecar, packet("")).unwrap();
        cache.published(&new_sidecar);
        assert_eq!(destination(&media, &mut cache).unwrap(), new_sidecar);
        // A fresh batch must see filesystem changes made after the last batch.
        fs::remove_file(&new_sidecar).unwrap();
        let replacement = dir.join("new.xmp");
        fs::write(&replacement, packet("")).unwrap();
        assert_eq!(
            destination(&media, &mut ExportIndex::default()).unwrap(),
            replacement
        );
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn nested_resource_tags_are_not_photo_keywords() {
        let xml = format!(
            "<rdf:RDF xmlns:rdf=\"{NS_RDF}\"><rdf:Description xmlns:dc=\"{NS_DC}\" xmlns:c=\"urn:custom\"><dc:subject><rdf:Bag><rdf:li>photo</rdf:li></rdf:Bag></dc:subject><c:resource><rdf:Description rdf:about=\"urn:other\"><dc:subject><rdf:Bag><rdf:li>resource</rdf:li></rdf:Bag></dc:subject></rdf:Description></c:resource></rdf:Description></rdf:RDF>"
        );
        let merged = merge_with_culling(&xml, 0, 0, &[], "", ALL).unwrap();
        assert!(merged.contains("<rdf:li>resource</rdf:li>"));
        assert_eq!(parse_xmp_str(&merged).unwrap().tags, Some(vec![]));
    }

    #[test]
    fn refresh_index_is_shared_across_requests_and_bound_to_library() {
        let (_, dir, media) = fixture();
        let id = begin_refresh_batch("test-library".into()).unwrap();
        {
            let _scope = refresh_index_scope(Some(id), "test-library").unwrap();
            assert!(find_xmp_sidecar(&media).is_none());
        }
        let sidecar = dir.join("new-sidecar.xmp");
        fs::write(&sidecar, packet("")).unwrap();
        // Inspect reuse after a different request enters on a fresh worker thread.
        std::thread::spawn(move || {
            let _scope = refresh_index_scope(Some(id), "test-library").unwrap();
            let index = REFRESH_INDEX.with(|v| v.borrow().clone().unwrap());
            assert_eq!(index.lock().unwrap().0.len(), 1);
        })
        .join()
        .unwrap();
        assert!(refresh_index_scope(Some(id), "other-library").is_err());
        end_refresh_batch(id);
        assert!(refresh_index_scope(Some(id), "test-library").is_err());
        fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn hierarchical_catalog_export_roundtrips_without_flat_duplicates_or_group_names() {
        let (conn,dir,media)=fixture();
        conn.execute("DELETE FROM afile_tags",[]).unwrap();
        let xml=packet("<d:subject><rdf:Bag><rdf:li>Paris</rdf:li></rdf:Bag></d:subject><lr:hierarchicalSubject xmlns:lr=\"http://ns.adobe.com/lightroom/1.0/\"><rdf:Bag><rdf:li>Places|France|Paris</rdf:li><rdf:li>Clients|Paris</rdf:li></rdf:Bag></lr:hierarchicalSubject><digiKam:TagsList xmlns:digiKam=\"http://www.digikam.org/ns/1.0/\"><rdf:Seq><rdf:li>Places/France/Paris</rdf:li><rdf:li>Clients/Paris</rdf:li></rdf:Seq></digiKam:TagsList>");
        let meta=parse_xmp_str(&xml).unwrap();
        apply_xmp_metadata_to_db(1,&meta,&conn,XmpImportMode::InitialScan).unwrap();
        assert_eq!(conn.query_row("SELECT COUNT(*) FROM afile_tags WHERE file_id=1",[],|r|r.get::<_,i64>(0)).unwrap(),2);
        assert_eq!(crate::t_tags::find(&conn,"Paris",None).unwrap(),None);
        fs::write(media.with_extension("xmp"), &xml).unwrap();
        let result=save_batch(&conn,&[1]);
        assert_eq!(result.saved_ids,vec![1]);
        assert!(result.failures.is_empty());
        let output=fs::read_to_string(find_xmp_sidecar(&media).unwrap()).unwrap();
        assert!(output.contains("<rdf:li>Places/France/Paris</rdf:li>"));
        assert!(output.contains("<rdf:li>Places|France|Paris</rdf:li>"));
        let reread=parse_xmp_str(&output).unwrap();
        assert_eq!(reread.hierarchical_tags,Some(vec!["Clients|Paris".into(),"Places|France|Paris".into()]));
        conn.execute("INSERT INTO afiles(id) VALUES(2)",[]).unwrap();
        apply_xmp_metadata_to_db(2,&reread,&conn,XmpImportMode::InitialScan).unwrap();
        assert_eq!(conn.query_row("SELECT COUNT(*) FROM afile_tags WHERE file_id=2",[],|r|r.get::<_,i64>(0)).unwrap(),2);
        let country=crate::t_tags::find(&conn,"France",crate::t_tags::find(&conn,"Places",None).unwrap()).unwrap().unwrap();
        conn.execute("UPDATE atags SET name='French places' WHERE id=?1",[country]).unwrap();
        conn.execute("INSERT INTO afiles(id) VALUES(3)",[]).unwrap();
        apply_xmp_metadata_to_db(3,&meta,&conn,XmpImportMode::InitialScan).unwrap();
        let (_,paths)=crate::t_tags::keywords_for_file(&conn,3).unwrap();
        assert!(paths.contains(&"Places|French places|Paris".to_string()));
        assert!(crate::t_tags::find(&conn,"France",crate::t_tags::find(&conn,"Places",None).unwrap()).unwrap().is_none());
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn acdsee_only_tags_and_cross_source_paths_share_the_same_nodes() {
        let (conn, dir, _) = fixture();
        conn.execute("DELETE FROM afile_tags", []).unwrap();
        let categories = r#"<Categories><Category Assigned="0">Places<Category Assigned="1">Paris</Category><Category Assigned="0">Unused</Category></Category></Categories>"#;
        let property = format!(
            "<a:categories xmlns:a=\"{NS_ACDSEE}\">{}</a:categories>",
            escape(categories)
        );
        let meta = parse_xmp_str(&packet(&property)).unwrap();
        apply_xmp_metadata_to_db(1, &meta, &conn, XmpImportMode::InitialScan).unwrap();
        assert_eq!(
            crate::t_tags::keywords_for_file(&conn, 1).unwrap(),
            (vec!["Paris".into()], vec!["Places|Paris".into()])
        );
        assert!(crate::t_tags::find(&conn, "Unused", None)
            .unwrap()
            .is_none());
        let combined = packet(&format!("{property}<lr:hierarchicalSubject xmlns:lr=\"{NS_LR}\"><rdf:Bag><rdf:li>Places|Paris</rdf:li></rdf:Bag></lr:hierarchicalSubject><k:TagsList xmlns:k=\"{NS_DIGIKAM}\"><rdf:Seq><rdf:li>Places/Paris</rdf:li></rdf:Seq></k:TagsList>"));
        let combined = parse_xmp_str(&combined).unwrap();
        conn.execute("INSERT INTO afiles(id) VALUES(2)", [])
            .unwrap();
        for _ in 0..2 {
            apply_xmp_metadata_to_db(2, &combined, &conn, XmpImportMode::InitialScan).unwrap();
            assert_eq!(
                crate::t_tags::keywords_for_file(&conn, 2).unwrap(),
                crate::t_tags::keywords_for_file(&conn, 1).unwrap()
            );
            assert_eq!(
                conn.query_row("SELECT COUNT(*) FROM afile_tags WHERE file_id=2", [], |r| r
                    .get::<_, i64>(0))
                    .unwrap(),
                1
            );
        }
        let broken = packet(&format!("<a:categories xmlns:a=\"{NS_ACDSEE}\">&lt;Categories&gt;</a:categories><d:subject><rdf:Bag><rdf:li>Keep</rdf:li></rdf:Bag></d:subject>"));
        let parsed = parse_xmp_str(&broken).unwrap();
        assert_eq!(parsed.tags, Some(vec!["Keep".into()]));
        assert!(parsed.hierarchical_tags.is_none());
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn tag_lookup_uses_nocase_index() {
        let (conn, dir, _) = fixture();
        let plan: String = conn
            .query_row(
                "EXPLAIN QUERY PLAN SELECT id FROM atags WHERE name=?1 COLLATE NOCASE",
                ["tag"],
                |r| r.get(3),
            )
            .unwrap();
        assert!(
            plan.contains("SEARCH") && plan.contains("idx_atags_name_nocase"),
            "{plan}"
        );
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn favorite_namespace_boolean_and_independence() {
        for (value, expected) in [
            ("True", Some(true)),
            ("false", Some(false)),
            ("1", Some(true)),
            ("0", Some(false)),
            ("", None),
            ("yes", None),
        ] {
            let xml = format!(
                "<rdf:RDF xmlns:rdf=\"{NS_RDF}\"><rdf:Description xmlns:own=\"{NS_LAP}\" own:Favorite=\"{value}\"/></rdf:RDF>"
            );
            assert_eq!(parse_xmp_str(&xml).unwrap().favorite, expected);
            let element = format!(
                "<rdf:RDF xmlns:rdf=\"{NS_RDF}\"><rdf:Description xmlns:own=\"{NS_LAP}\"><own:Favorite>{value}</own:Favorite></rdf:Description></rdf:RDF>"
            );
            assert_eq!(parse_xmp_str(&element).unwrap().favorite, expected);
        }
        let xml = packet(
            "<r:Rating>5</r:Rating><r:Label>Favorite</r:Label><d:subject><rdf:Bag><rdf:li>Favorite</rdf:li></rdf:Bag></d:subject><xmpDM:pick xmlns:xmpDM=\"http://ns.adobe.com/xmp/1.0/DynamicMedia/\">1</xmpDM:pick><lap:Favorite xmlns:lap=\"urn:spoof\">True</lap:Favorite>",
        );
        assert_eq!(parse_xmp_str(&xml).unwrap().favorite, None);
        let standard = packet(&format!(
            "<l:Favorite xmlns:l=\"{NS_LAP}\">True</l:Favorite>"
        ));
        let extended = packet(&format!(
            "<l:Favorite xmlns:l=\"{NS_LAP}\">False</l:Favorite>"
        ));
        assert_eq!(flags_from_packets(&[standard, extended]).1, Some(false));
    }

    #[test]
    fn favorite_export_roundtrip_and_local_unfavorite_protection() {
        let (conn, dir, media) = fixture();
        for favorite in [true, false, true] {
            conn.execute("UPDATE afiles SET is_favorite=?1 WHERE id=1", [favorite])
                .unwrap();
            assert_eq!(save_batch(&conn, &[1]).saved_ids, vec![1]);
            let meta = read_xmp_file(&dir.join("photo.jpg.xmp")).unwrap();
            assert_eq!(meta.favorite, Some(favorite));
            assert_eq!(meta.rating, Some(4));
            if std::env::var("LAP_TEST_EXIFTOOL").as_deref() == Ok("1") {
                let output = std::process::Command::new("exiftool")
                    .args(["-s3", "-Favorite"])
                    .arg(dir.join("photo.jpg.xmp"))
                    .output()
                    .expect("ExifTool required for independent verification");
                assert!(output.status.success());
                assert_eq!(
                    String::from_utf8(output.stdout).unwrap().trim(),
                    if favorite { "True" } else { "False" }
                );
            }
            // Saving does not clear the local-edit protection bit.
            let dirty: i32 = conn
                .query_row("SELECT dirty FROM axmp_imports WHERE file_id=1", [], |r| {
                    r.get(0)
                })
                .unwrap();
            assert_ne!(dirty & 16, 0);
        }
        conn.execute("UPDATE afiles SET is_favorite=0 WHERE id=1", [])
            .unwrap();
        import_xmp_for_file_with_conn(
            1,
            media.to_str().unwrap(),
            &conn,
            XmpImportMode::RefreshNonDestructive,
        )
        .unwrap();
        assert!(
            !conn
                .query_row("SELECT is_favorite FROM afiles WHERE id=1", [], |r| r
                    .get::<_, bool>(0))
                .unwrap()
        );
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn tag_tree_upgrade_reimports_unchanged_sources_once_and_preserves_local_edits() {
        for dirty in [0, 8] {
            let (conn, dir, media) = fixture();
            let categories = escape(r#"<Categories><Category Assigned="1">Vendor</Category></Categories>"#);
            let xml = packet(&format!("<lr:hierarchicalSubject xmlns:lr=\"{NS_LR}\"><rdf:Bag><rdf:li>Places|Paris</rdf:li></rdf:Bag></lr:hierarchicalSubject><a:categories xmlns:a=\"{NS_ACDSEE}\">{categories}</a:categories>"));
            fs::write(media.with_extension("xmp"), &xml).unwrap();
            let import = || import_xmp_for_file_with_conn(
                1, media.to_str().unwrap(), &conn, XmpImportMode::RescanNonDestructive,
            ).unwrap();
            assert!(import());
            // Simulate the migrated catalog and unchanged v3 source fingerprint.
            conn.execute_batch("DELETE FROM afile_tags; DELETE FROM axmp_tag_paths; DELETE FROM atags;
                INSERT INTO atags(id,name) VALUES(1,'Places|Paris');
                INSERT INTO afile_tags VALUES(1,1);
                UPDATE afiles SET has_tags=1 WHERE id=1;
                UPDATE axmp_imports SET fingerprint=REPLACE(fingerprint,'embedded-v4:','embedded-v3:') WHERE file_id=1;").unwrap();
            conn.execute("UPDATE axmp_imports SET dirty=?1 WHERE file_id=1", [dirty]).unwrap();

            assert!(import());
            assert!(!import());
            let (flat, paths) = crate::t_tags::keywords_for_file(&conn, 1).unwrap();
            if dirty == 0 {
                assert_eq!(flat, vec!["Paris", "Vendor"]);
                assert_eq!(paths, vec!["Places|Paris", "Vendor"]);
                assert_eq!(save_batch(&conn, &[1]).saved_ids, vec![1]);
                let output = read_xmp_file(&media.with_extension("xmp")).unwrap();
                assert_eq!(output.hierarchical_tags, Some(paths));
            } else {
                assert_eq!(flat, vec!["Places|Paris"]);
                assert!(paths.is_empty());
                assert_eq!(fs::read_to_string(media.with_extension("xmp")).unwrap(), xml);
            }
            assert_eq!(conn.query_row("SELECT dirty FROM axmp_imports WHERE file_id=1", [], |r| r.get::<_, i32>(0)).unwrap(), dirty);
            assert_eq!(fs::read(&media).unwrap(), b"unchanged-original");
            fs::remove_dir_all(dir).unwrap();
        }
    }

    #[test]
    fn favorite_sidecar_false_overrides_embedded_true_and_upgrade_backfills() {
        let (conn, dir, media) = fixture();
        let xml = packet(&format!(
            "<custom:Favorite xmlns:custom=\"{NS_LAP}\">True</custom:Favorite>"
        ));
        fs::write(
            &media,
            crate::t_metadata::tests::jpeg_with_xmp_packet(xml.as_bytes()),
        )
        .unwrap();
        let sidecar = dir.join("photo.jpg.xmp");
        fs::write(
            &sidecar,
            packet(&format!(
                "<custom:Favorite xmlns:custom=\"{NS_LAP}\">False</custom:Favorite>"
            )),
        )
        .unwrap();
        import_xmp_for_file_with_conn(
            1,
            media.to_str().unwrap(),
            &conn,
            XmpImportMode::InitialScan,
        )
        .unwrap();
        assert!(
            !conn
                .query_row("SELECT is_favorite FROM afiles WHERE id=1", [], |r| r
                    .get::<_, bool>(0))
                .unwrap()
        );
        fs::remove_file(sidecar).unwrap();
        conn.execute("UPDATE axmp_imports SET fingerprint=REPLACE(fingerprint,'embedded-v4:','embedded-v2:') WHERE file_id=1",[]).unwrap();
        assert!(
            import_xmp_for_file_with_conn(
                1,
                media.to_str().unwrap(),
                &conn,
                XmpImportMode::RescanNonDestructive
            )
            .unwrap()
        );
        assert!(
            conn.query_row("SELECT is_favorite FROM afiles WHERE id=1", [], |r| r
                .get::<_, bool>(0))
                .unwrap()
        );
        assert!(
            !import_xmp_for_file_with_conn(
                1,
                media.to_str().unwrap(),
                &conn,
                XmpImportMode::RescanNonDestructive
            )
            .unwrap()
        );
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn favorite_replacement_preserves_unknown_properties_and_absence() {
        let original = packet(&format!(
            "<l:Favorite xmlns:l=\"{NS_LAP}\">True</l:Favorite><crs:Exposure2012>+1</crs:Exposure2012><r:Label>Red</r:Label>"
        ));
        let updated = merge_with_favorite(&original, 0, 0, false, &[], "", 16).unwrap();
        assert_eq!(parse_xmp_str(&updated).unwrap().favorite, Some(false));
        assert!(!updated.contains(">True</l:Favorite>"));
        assert!(updated.contains("<r:Label>Red</r:Label>"));
        assert!(updated.contains("<crs:Exposure2012>+1</crs:Exposure2012>"));
        assert_eq!(
            parse_xmp_str(&merge_with_favorite(&original, 3, 0, false, &[], "", 1).unwrap())
                .unwrap()
                .favorite,
            Some(true)
        );
    }
}

/// Source details are independent of organization import and preserve explicit empty properties.
pub(crate) fn source_details(content: &str) -> Result<crate::t_metadata::Details, String> {
    let root = parse_xml(content)?;
    let mut properties = Vec::new();
    collect_properties(&root, &mut properties);
    let mut details = crate::t_metadata::Details::new();
    for (key, uri, name) in [
        ("title", NS_DC, "title"),
        ("creator", NS_DC, "creator"),
        ("copyright", NS_DC, "rights"),
        ("description", NS_DC, "description"),
        ("credit", "http://ns.adobe.com/photoshop/1.0/", "Credit"),
        ("source", "http://ns.adobe.com/photoshop/1.0/", "Source"),
        (
            "usageTerms",
            "http://ns.adobe.com/xap/1.0/rights/",
            "UsageTerms",
        ),
        (
            "rightsUrl",
            "http://ns.adobe.com/xap/1.0/rights/",
            "WebStatement",
        ),
        ("software", NS_XMP, "CreatorTool"),
    ] {
        if let Some(node) = properties.iter().find(|n| n.uri == uri && n.name == name) {
            let container = node
                .children
                .iter()
                .find(|n| n.uri == NS_RDF && matches!(n.name.as_str(), "Seq" | "Bag" | "Alt"));
            let values = if let Some(container) = container {
                container
                    .children
                    .iter()
                    .filter(|n| n.uri == NS_RDF && n.name == "li")
                    .map(|n| match n.lang.as_deref() {
                        Some(lang) if lang != "x-default" => format!("[{lang}] {}", n.text.trim()),
                        _ => n.text.trim().to_owned(),
                    })
                    .collect()
            } else {
                vec![node
                    .attributes
                    .iter()
                    .find(|a| a.uri == NS_RDF && a.name == "resource")
                    .map(|a| a.text.clone())
                    .unwrap_or_else(|| node.value())]
            };
            details.insert(
                key.into(),
                crate::t_metadata::DetailValue {
                    values,
                    source: format!("Sidecar XMP:{name}"),
                },
            );
        }
    }
    Ok(details)
}

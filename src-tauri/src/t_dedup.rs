use crate::t_sqlite::{AFile, QueryParams};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{self, Read};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::Emitter;

// ----------------------------------------------------------------------------
// Types and Structs
// ----------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DedupScanStatus {
    pub state: String, // "running", "idle", "finished", "error"
    pub processed: u64,
    pub total: u64,
    pub groups: u64,
    pub is_scanning: bool,
    pub skipped: u64, // online-only files left unhashed
}

impl Default for DedupScanStatus {
    fn default() -> Self {
        Self {
            state: "idle".to_string(),
            processed: 0,
            total: 0,
            groups: 0,
            is_scanning: false,
            skipped: 0,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DedupDeleteResult {
    pub deleted_file_ids: Vec<i64>,
    pub affected_folder_paths: Vec<String>,
    pub deleted_count: usize,
    pub failed_count: usize,
    pub errors: Vec<String>,
    pub trash_failed_file_ids: Vec<i64>,
    pub preview: Option<DedupDeletePreview>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DedupDeletePreview {
    pub token: String,
    pub item_count: usize,
    pub raw_companion_count: usize,
    pub bytes: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DedupDeleteRequest {
    pub group_ids: Option<Vec<i64>>,
    pub file_ids: Option<Vec<i64>>,
    pub similar_group_id: Option<i64>,
    pub delete_all: bool,
    pub permanently: bool,
    #[serde(default)]
    pub group_raw_pairs: bool,
    #[serde(default)]
    pub preview: bool,
    pub plan_token: Option<String>,
}

#[derive(Default)]
pub struct DedupState {
    pub is_scanning: Arc<AtomicBool>,
    pub cancel_flag: Arc<AtomicBool>,
    pub status: Arc<Mutex<DedupScanStatus>>,
}

#[derive(Clone)]
struct KeepCandidate {
    id: i64,
    is_raw_pair: bool,
    taken_date: i64,
    created_at: i64,
}

const RAW_PAIR_MEMBER_SQL: &str = "(a.media_subtype IS 'raw_jpeg_pair' AND EXISTS
    (SELECT 1 FROM afiles jpeg WHERE jpeg.id = a.live_photo_video_id))
    OR EXISTS (SELECT 1 FROM afiles raw WHERE raw.media_subtype = 'raw_jpeg_pair'
               AND raw.live_photo_video_id = a.id)";

// ----------------------------------------------------------------------------
// Core Logic
// ----------------------------------------------------------------------------

pub fn start_scan(
    app_handle: tauri::AppHandle,
    dedup_state: tauri::State<'_, DedupState>,
    query_params: Option<QueryParams>,
    collection_id: Option<i64>,
    file_ids: Option<Vec<i64>>,
    keep_rules: Option<KeepRules>,
    group_raw_pairs: bool,
) -> Result<(), String> {
    if file_ids.is_some() && (collection_id.is_some() || query_params.is_some()) {
        return Err("File ID scope cannot be combined with query or collection scope.".into());
    }

    if dedup_state
        .is_scanning
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        return Err("A deduplication scan is already running.".into());
    }
    dedup_state.cancel_flag.store(false, Ordering::SeqCst);

    let status_clone = dedup_state.status.clone();
    let is_scanning_clone = dedup_state.is_scanning.clone();
    let cancel_flag_clone = dedup_state.cancel_flag.clone();

    // Reset status
    {
        let mut status = status_clone.lock().unwrap();
        status.state = "running".to_string();
        status.processed = 0;
        status.total = 0;
        status.groups = 0;
        status.skipped = 0;
        status.is_scanning = true;
    }

    std::thread::spawn(move || {
        let result = scan_and_hash_files(
            &app_handle,
            &status_clone,
            &cancel_flag_clone,
            query_params,
            collection_id,
            file_ids,
            keep_rules,
            group_raw_pairs,
        );

        let mut final_status = status_clone.lock().unwrap();
        match result {
            Ok(_) => {
                if cancel_flag_clone.load(Ordering::SeqCst) {
                    final_status.state = "idle".to_string();
                } else {
                    final_status.state = "finished".to_string();
                }
            }
            Err(e) => {
                eprintln!("Dedup scan error: {}", e);
                final_status.state = "error".to_string();
            }
        }

        is_scanning_clone.store(false, Ordering::SeqCst);
        final_status.is_scanning = false;
        let _ = app_handle.emit("dedup-scan-progress", final_status.clone());
    });

    Ok(())
}

fn scan_and_hash_files(
    app_handle: &tauri::AppHandle,
    status_mutex: &Arc<Mutex<DedupScanStatus>>,
    cancel_flag: &Arc<AtomicBool>,
    query_params: Option<QueryParams>,
    collection_id: Option<i64>,
    file_ids: Option<Vec<i64>>,
    keep_rules: Option<KeepRules>,
    group_raw_pairs: bool,
) -> Result<(), String> {
    let mut conn = get_db_conn()?;
    let has_scope = collection_id.is_some() || file_ids.is_some() || query_params.is_some();

    let query_params = query_params.map(|mut params| {
        params.include_raw_companions = !group_raw_pairs;
        params
    });
    let mut files_to_check = if let Some(file_ids) = file_ids {
        AFile::get_files_by_ids(&file_ids)?
    } else if let Some(collection_id) = collection_id {
        get_files_by_collection(collection_id, query_params.as_ref())?
    } else if let Some(params) = query_params.as_ref() {
        get_files_by_query(params)?
    } else {
        get_files_by_sizes(&conn, group_raw_pairs)?
    };

    if group_raw_pairs && has_scope {
        let companion_ids = files_to_check.iter().filter_map(|file| {
            (file.media_subtype.as_deref() == Some("raw_jpeg_pair"))
                .then_some(file.live_photo_video_id).flatten()
        }).collect::<Vec<_>>();
        let mut seen_ids: HashSet<_> = files_to_check.iter().filter_map(|file| file.id).collect();
        files_to_check.extend(AFile::get_files_by_ids(&companion_ids)?
            .into_iter().filter(|file| file.id.is_some_and(|id| seen_ids.insert(id))));
    }
    let files_to_check = filter_suspicious_files(files_to_check);
    let scoped_file_ids = if has_scope || group_raw_pairs {
        Some(
            files_to_check
                .iter()
                .filter_map(|file| file.id)
                .collect::<Vec<i64>>(),
        )
    } else {
        None
    };
    if files_to_check.is_empty() {
        rebuild_duplicate_groups(&mut conn, scoped_file_ids.as_deref(), None, group_raw_pairs)?;
        return Ok(());
    }

    let total_files = files_to_check.len() as u64;
    {
        let mut status = status_mutex.lock().unwrap();
        status.total = total_files;
        status.processed = 0;
    }
    let _ = app_handle.emit("dedup-scan-progress", status_mutex.lock().unwrap().clone());

    // Step 3: Hash them
    let mut processed = 0;
    let mut skipped = 0u64;

    // We do batch inserts to speed up DB operations
    let tx = conn.transaction().map_err(|e| e.to_string())?;

    for file in files_to_check {
        if cancel_flag.load(Ordering::SeqCst) {
            break;
        }

        // Only hash if mtime changed or hash is missing
        let needs_hash = check_if_needs_hash(&tx, &file)?;

        if needs_hash {
            if let Some(path) = &file.file_path {
                if crate::t_utils::is_cloud_only_path(path) {
                    // Hashing reads the whole file; Lap must not trigger a download.
                    // A hash from before a remote change would still group the file as a duplicate.
                    tx.execute("DELETE FROM file_hashes WHERE file_id = ?1", params![file.id.unwrap()])
                        .map_err(|e| e.to_string())?;
                    skipped += 1;
                } else {
                match compute_blake3_hash(path) {
                    Ok(hash) => {
                        let now = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_secs() as i64;
                        let mtime = file.modified_at.unwrap_or(0);

                        tx.execute(
                            "INSERT OR REPLACE INTO file_hashes (file_id, hash, file_size, mtime, computed_at)
                             VALUES (?1, ?2, ?3, ?4, ?5)",
                            params![file.id.unwrap(), hash, file.size, mtime, now],
                        ).map_err(|e| e.to_string())?;
                    }
                    Err(e) => {
                        tx.execute("DELETE FROM file_hashes WHERE file_id = ?1", params![file.id.unwrap()])
                            .map_err(|e| e.to_string())?;
                        skipped += 1;
                        eprintln!("Failed to hash file {}: {}", path, e);
                    }
                }
                }
            }
        }

        processed += 1;
        if processed % 10 == 0 {
            {
                let mut status = status_mutex.lock().unwrap();
                status.processed = processed;
            }
            let _ = app_handle.emit("dedup-scan-progress", status_mutex.lock().unwrap().clone());
        }
    }

    tx.commit().map_err(|e| e.to_string())?;
    status_mutex.lock().unwrap().skipped = skipped;

    // Step 4: Rebuild duplicate groups
    if !cancel_flag.load(Ordering::SeqCst) {
        rebuild_duplicate_groups(&mut conn, scoped_file_ids.as_deref(), keep_rules, group_raw_pairs)?;

        // Count total groups
        let groups_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM duplicate_groups", [], |row| {
                row.get(0)
            })
            .unwrap_or(0);

        {
            let mut status = status_mutex.lock().unwrap();
            status.processed = processed;
            status.groups = groups_count as u64;
        }
        let _ = app_handle.emit("dedup-scan-progress", status_mutex.lock().unwrap().clone());
    }

    Ok(())
}

fn get_db_conn() -> Result<crate::t_sqlite::PooledConn, String> {
    crate::t_sqlite::open_conn()
}

fn get_files_by_sizes(conn: &Connection, group_raw_pairs: bool) -> Result<Vec<AFile>, String> {
    let mut stmt = conn.prepare(
        "SELECT a.id, a.folder_id,
                a.name, a.name_pinyin, a.size, a.file_type, a.format_label, a.created_at, a.modified_at, a.inode, a.taken_date,
                a.width, a.height, a.duration, a.icc_profile, a.color_space, a.bit_depth,
                a.is_favorite, a.rating, a.rotate, a.comments, a.has_tags, a.has_faces,
                a.e_make, a.e_model, a.e_lens_make, a.e_lens_model,
                a.e_date_time, a.e_exposure_bias, a.e_exposure_time, a.e_f_number, a.e_focal_length, a.e_iso_speed, a.e_flash, a.e_orientation,
                a.e_title, a.e_headline, a.e_description, a.e_artist, a.e_copyright,
                a.e_software,
                a.gps_latitude, a.gps_longitude, a.gps_altitude,
                a.geo_name, a.geo_admin1, a.geo_admin2, a.geo_cc,
                f.path || '/' || a.name as file_path
         FROM afiles a
         JOIN afolders f ON a.folder_id = f.id
         WHERE a.id NOT IN (
             SELECT live_photo_video_id FROM afiles WHERE live_photo_video_id IS NOT NULL
               AND (NOT ?1 OR media_subtype IS NOT 'raw_jpeg_pair')
         )
         AND a.size IN (
             SELECT size FROM afiles
             WHERE id NOT IN (
                 SELECT live_photo_video_id FROM afiles WHERE live_photo_video_id IS NOT NULL
               AND (NOT ?1 OR media_subtype IS NOT 'raw_jpeg_pair')
             )
             GROUP BY size
             HAVING COUNT(size) > 1 AND size > 0
         )
         ORDER BY a.size DESC"
    ).map_err(|e| e.to_string())?;

    let iter = stmt
        .query_map([group_raw_pairs], |row| {
            Ok(AFile {
                id: row.get(0)?,
                folder_id: row.get(1)?,

                name: row.get(2)?,
                name_pinyin: row.get(3)?,
                size: row.get(4)?,
                file_type: row.get(5)?,
                format_label: row.get(6)?,
                created_at: row.get(7)?,
                modified_at: row.get(8)?,
                inode: row.get(9)?,
                taken_date: row.get(10)?,

                width: row.get(11)?,
                height: row.get(12)?,
                duration: row.get(13)?,
                icc_profile: row.get(14)?,
                color_space: row.get(15)?,
                bit_depth: row.get(16)?,

                is_favorite: row.get(17)?,
                rating: row.get(18)?,
                culling_flag: None,
                rotate: row.get(19)?,
                comments: row.get(20)?,
                has_tags: row.get(21)?,
                has_faces: row.get(22)?,

                e_make: row.get(23)?,
                e_model: row.get(24)?,
                e_lens_make: row.get(25)?,
                e_lens_model: row.get(26)?,

                e_date_time: row.get(27)?,
                e_exposure_bias: row.get(28)?,
                e_exposure_time: row.get(29)?,
                e_f_number: row.get(30)?,
                e_focal_length: row.get(31)?,
                e_iso_speed: row.get(32)?,
                e_flash: row.get(33)?,
                e_orientation: row.get(34)?,

                e_title: row.get(35)?,
                e_headline: row.get(36)?,
                e_description: row.get(37)?,
                e_artist: row.get(38)?,
                e_copyright: row.get(39)?,

                e_software: row.get(40)?,

                gps_latitude: row.get(41)?,
                gps_longitude: row.get(42)?,
                gps_altitude: row.get(43)?,

                geo_name: row.get(44)?,
                geo_admin1: row.get(45)?,
                geo_admin2: row.get(46)?,
                geo_cc: row.get(47)?,

                file_path: row.get(48)?,
                album_id: None,
                album_name: None,
                has_thumbnail: None,
                has_collections: None,
                has_embedding: None,
                last_scan_time: Some(0),
                content_identifier: None,
                media_subtype: None,
                live_photo_video_id: None,
                live_photo_video_path: None,
                album_accessible: true,
                album_visible: true,
                motion_photo_offset: None,
                cloud_only: false,
                metadata_pending: false,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut files = Vec::new();
    for f in iter {
        if let Ok(file) = f {
            files.push(file);
        }
    }

    Ok(files)
}

pub fn get_files_by_query(params: &QueryParams) -> Result<Vec<AFile>, String> {
    let mut all_files = Vec::new();
    let mut offset: i64 = 0;
    let chunk_size: i64 = 2000;

    loop {
        let files = AFile::get_query_files(params, offset, chunk_size)?;
        let fetched = files.len() as i64;
        if fetched == 0 {
            break;
        }
        all_files.extend(files);
        if fetched < chunk_size {
            break;
        }
        offset += chunk_size;
    }

    Ok(all_files)
}

pub fn get_files_by_collection(
    collection_id: i64,
    params: Option<&QueryParams>,
) -> Result<Vec<AFile>, String> {
    let params =
        params.ok_or_else(|| "Collection dedup query parameters are required".to_string())?;
    let mut all_files = Vec::new();
    let mut offset: i64 = 0;
    let chunk_size: i64 = 2000;

    loop {
        let files = AFile::get_collection_files(collection_id, params, offset, chunk_size)?;
        let fetched = files.len() as i64;
        if fetched == 0 {
            break;
        }
        all_files.extend(files);
        if fetched < chunk_size {
            break;
        }
        offset += chunk_size;
    }

    Ok(all_files)
}

fn filter_suspicious_files(files: Vec<AFile>) -> Vec<AFile> {
    let mut size_count: HashMap<i64, usize> = HashMap::new();
    for file in &files {
        if file.size > 0 {
            *size_count.entry(file.size).or_insert(0) += 1;
        }
    }

    files
        .into_iter()
        .filter(|file| file.size > 0 && size_count.get(&file.size).copied().unwrap_or(0) > 1)
        .collect()
}

fn check_if_needs_hash(conn: &Connection, file: &AFile) -> Result<bool, String> {
    let mtime = file.modified_at.unwrap_or(0);
    let id = file.id.unwrap();

    let db_mtime: Option<i64> = conn
        .query_row(
            "SELECT mtime FROM file_hashes WHERE file_id = ?1",
            params![id],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| e.to_string())?;

    match db_mtime {
        Some(stored_mtime) => Ok(stored_mtime != mtime),
        None => Ok(true),
    }
}

fn compute_blake3_hash(path: &str) -> Result<String, io::Error> {
    let mut file = File::open(path)?;
    let mut hasher = blake3::Hasher::new();

    // Read in chunks
    let mut buffer = [0; 65536];
    loop {
        let n = file.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }

    Ok(hasher.finalize().to_hex().to_string())
}

fn rebuild_duplicate_groups(
    conn: &mut Connection,
    scope_file_ids: Option<&[i64]>,
    keep_rules: Option<KeepRules>,
    group_raw_pairs: bool,
) -> Result<(), String> {
    let tx = conn.transaction().map_err(|e| e.to_string())?;

    // Remove existing items and groups to rebuild clean
    tx.execute("DELETE FROM duplicate_group_items", [])
        .map_err(|e| e.to_string())?;
    tx.execute("DELETE FROM duplicate_groups", [])
        .map_err(|e| e.to_string())?;

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    if let Some(scope_ids) = scope_file_ids {
        tx.execute("DROP TABLE IF EXISTS temp_scope_ids", [])
            .map_err(|e| e.to_string())?;
        tx.execute(
            "CREATE TEMP TABLE temp_scope_ids (file_id INTEGER PRIMARY KEY)",
            [],
        )
        .map_err(|e| e.to_string())?;

        if scope_ids.is_empty() {
            tx.commit().map_err(|e| e.to_string())?;
            return Ok(());
        }

        let mut insert_stmt = tx
            .prepare("INSERT OR IGNORE INTO temp_scope_ids (file_id) VALUES (?1)")
            .map_err(|e| e.to_string())?;
        for file_id in scope_ids {
            insert_stmt
                .execute(params![file_id])
                .map_err(|e| e.to_string())?;
        }
        drop(insert_stmt);
    }

    // Find dups
    let group_query = if scope_file_ids.is_some() {
        "SELECT fh.hash, fh.file_size, COUNT(fh.file_id) as cnt
         FROM file_hashes fh
         JOIN temp_scope_ids ts ON ts.file_id = fh.file_id
         GROUP BY fh.hash, fh.file_size
         HAVING cnt > 1"
    } else {
        "SELECT hash, file_size, COUNT(file_id) as cnt
         FROM file_hashes
         GROUP BY hash, file_size
         HAVING cnt > 1"
    };

    let mut stmt = tx.prepare(group_query).map_err(|e| e.to_string())?;

    let rows: Vec<(String, i64, i64)> = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, i64>(2)?,
            ))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let scope_join = if scope_file_ids.is_some() {
        "JOIN temp_scope_ids ts ON ts.file_id = a.id"
    } else {
        ""
    };
    let pair_sql = if group_raw_pairs { RAW_PAIR_MEMBER_SQL } else { "0" };
    let item_query = format!(
        "SELECT a.id, a.taken_date, a.created_at, {pair_sql}
         FROM file_hashes fh
         JOIN afiles a ON fh.file_id = a.id
         {scope_join}
         WHERE fh.hash = ?1 AND fh.file_size = ?2"
    );
    let mut f_stmt = tx.prepare(&item_query).map_err(|e| e.to_string())?;

    for (hash, size, count) in rows {
        let total_size = size * count;

        // Insert group
        tx.execute(
            "INSERT INTO duplicate_groups (hash, file_size, file_count, total_size, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![hash, size, count, total_size, now],
        )
        .map_err(|e| e.to_string())?;

        // target_group_id computation is replaced by retrieving it exactly via last_insert_rowid if it works
        // or re-query. We just use last_insert_rowid as an SQLite function on connection, but for tx we do:
        let target_group_id = tx.last_insert_rowid();

        let mut keep_candidates: Vec<KeepCandidate> = Vec::new();
        let iter = f_stmt
            .query_map(params![hash, size], |row| {
                let id: i64 = row.get(0)?;
                let tk: i64 = row.get(1).unwrap_or(0);
                let ca: i64 = row.get(2).unwrap_or(0);
                Ok((id, tk, ca, row.get(3)?))
            })
            .map_err(|e| e.to_string())?;

        for r in iter {
            let (id, tk, ca, is_raw_pair) = r.map_err(|e| e.to_string())?;
            keep_candidates.push(KeepCandidate {
                id,
                is_raw_pair,
                taken_date: tk,
                created_at: ca,
            });
        }

        keep_candidates.sort_by(compare_best_quality);

        let total_candidates = keep_candidates.len() as f64;
        for (i, candidate) in keep_candidates.iter().enumerate() {
            let is_keep = if i == 0 { 1 } else { 0 };
            let score = total_candidates - i as f64;

            tx.execute(
                "INSERT INTO duplicate_group_items (group_id, file_id, is_keep, is_selected, score)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![target_group_id, candidate.id, is_keep, 0, score],
            )
            .map_err(|e| e.to_string())?;
        }
    }
    drop(f_stmt);
    drop(stmt);

    if group_raw_pairs {
        split_raw_pair_groups(&tx)?;
    }
    if let Some(rules) = keep_rules {
        apply_keep_rules(&tx, None, rules, group_raw_pairs)?;
    }
    if group_raw_pairs {
        retain_jpeg_group_keeper(&tx)?;
    }
    tx.commit().map_err(|e| e.to_string())?;
    Ok(())
}

/// RAW matches with different (or unverified) companion content are separate
/// sets. A standalone RAW can be removed through any of those sets.
fn split_raw_pair_groups(conn: &Connection) -> Result<(), String> {
    let mut stmt = conn.prepare(
        "SELECT i.group_id, i.file_id, companion.id, h.hash
         FROM duplicate_group_items i
         JOIN afiles a ON a.id = i.file_id
         LEFT JOIN afiles companion ON a.media_subtype = 'raw_jpeg_pair'
             AND companion.id = a.live_photo_video_id
         LEFT JOIN file_hashes h ON h.file_id = companion.id
             AND h.file_size = companion.size AND h.mtime = COALESCE(companion.modified_at, 0)
         WHERE EXISTS (
             SELECT 1 FROM duplicate_group_items p JOIN afiles raw ON raw.id = p.file_id
             WHERE p.group_id = i.group_id AND raw.media_subtype = 'raw_jpeg_pair'
         )
         ORDER BY i.group_id, i.score DESC",
    ).map_err(|e| e.to_string())?;
    let mut groups = std::collections::BTreeMap::<i64, Vec<_>>::new();
    let rows = stmt.query_map([], |row| Ok((
        row.get::<_, i64>(0)?, row.get::<_, i64>(1)?,
        row.get::<_, Option<i64>>(2)?, row.get::<_, Option<String>>(3)?,
    ))).map_err(|e| e.to_string())?;
    for row in rows {
        let (group, file, companion, hash) = row.map_err(|e| e.to_string())?;
        groups.entry(group).or_default().push((file, companion, hash));
    }
    for (group, members) in groups {
        let mut partitions = Vec::<Vec<i64>>::new();
        let mut by_hash = HashMap::new();
        let mut standalone = Vec::new();
        for (file, companion, hash) in members {
            if companion.is_none() {
                standalone.push(file);
                continue;
            }
            let index = match hash {
                Some(hash) => *by_hash.entry(hash).or_insert(partitions.len()),
                None => partitions.len(),
            };
            if index == partitions.len() {
                partitions.push(Vec::new());
            }
            partitions[index].push(file);
        }
        if partitions.len() <= 1 {
            continue;
        }
        for (index, mut members) in partitions.into_iter().enumerate() {
            let target = if index == 0 {
                members.append(&mut standalone);
                group
            } else {
                conn.execute(
                    "INSERT INTO duplicate_groups(hash,file_size,file_count,total_size,updated_at)
                     SELECT hash || ':' || ?3,file_size,?2,file_size * ?2,updated_at FROM duplicate_groups WHERE id=?1",
                    params![group, members.len(), members[0]],
                ).map_err(|e| e.to_string())?;
                conn.last_insert_rowid()
            };
            for (rank, id) in members.into_iter().enumerate() {
                conn.execute(
                    "UPDATE duplicate_group_items SET group_id=?1, is_keep=?2 WHERE group_id=?3 AND file_id=?4",
                    params![target, rank == 0, group, id],
                ).map_err(|e| e.to_string())?;
            }
        }
    }
    Ok(())
}

/// A JPEG match can protect a full pair, but never authorizes its removal.
/// Keep one surviving companion in each JPEG group and show standalone copies.
fn retain_jpeg_group_keeper(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        "DROP TABLE IF EXISTS temp_dedup_jpeg_keepers;
         CREATE TEMP TABLE temp_dedup_jpeg_keepers AS
         SELECT group_id, file_id FROM (
             SELECT i.group_id, i.file_id,
                 ROW_NUMBER() OVER (PARTITION BY i.group_id ORDER BY i.is_keep DESC, i.score DESC, i.file_id) AS rank
             FROM duplicate_group_items i
             JOIN afiles raw ON raw.live_photo_video_id = i.file_id AND raw.media_subtype = 'raw_jpeg_pair'
             WHERE NOT EXISTS (SELECT 1 FROM duplicate_group_items r WHERE r.file_id = raw.id AND r.is_keep = 0)
         ) WHERE rank = 1;
         DELETE FROM duplicate_group_items AS i
         WHERE EXISTS (SELECT 1 FROM afiles raw WHERE raw.live_photo_video_id = i.file_id
                       AND raw.media_subtype = 'raw_jpeg_pair')
           AND NOT EXISTS (SELECT 1 FROM temp_dedup_jpeg_keepers k
                           WHERE k.group_id = i.group_id AND k.file_id = i.file_id);
         UPDATE duplicate_group_items AS i
         SET is_keep = EXISTS (SELECT 1 FROM temp_dedup_jpeg_keepers k
                               WHERE k.group_id = i.group_id AND k.file_id = i.file_id)
         WHERE EXISTS (SELECT 1 FROM temp_dedup_jpeg_keepers k WHERE k.group_id = i.group_id);
         UPDATE duplicate_group_items
         SET is_keep = 1 WHERE (group_id, file_id) IN (
             SELECT group_id, MIN(file_id) FROM duplicate_group_items
             GROUP BY group_id HAVING MAX(is_keep) = 0
         );
         DELETE FROM duplicate_group_items WHERE group_id IN (
             SELECT group_id FROM duplicate_group_items GROUP BY group_id HAVING COUNT(*) = 1
         );
         DELETE FROM duplicate_groups WHERE id NOT IN (SELECT group_id FROM duplicate_group_items);
         DROP TABLE temp_dedup_jpeg_keepers;",
    ).map_err(|e| e.to_string())
}

fn compare_best_quality(a: &KeepCandidate, b: &KeepCandidate) -> std::cmp::Ordering {
    b.is_raw_pair
        .cmp(&a.is_raw_pair)
        .then_with(|| (b.taken_date > 0).cmp(&(a.taken_date > 0)))
        .then_with(|| a.created_at.cmp(&b.created_at))
        .then_with(|| a.id.cmp(&b.id))
}

// ----------------------------------------------------------------------------
// Retrieval APIs
// ----------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct DedupGroup {
    pub id: i64,
    pub hash: String,
    pub file_size: i64,
    pub file_count: i64,
    pub total_size: i64,
    pub reviewed: i32,
    pub updated_at: i64,
    pub items: Vec<DedupGroupItem>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DedupGroupItem {
    pub group_id: i64,
    pub file_id: i64,
    pub is_keep: i32,
    pub is_selected: i32,
    pub score: f64,
    pub can_set_keep: bool,
    pub can_delete: bool,
    pub companion_size: i64,
    pub file: Option<AFile>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DedupOverview {
    pub total_groups: i64,
    pub total_files: i64,
    pub total_reclaimable_bytes: i64,
}

pub fn get_overview(group_raw_pairs: bool) -> Result<DedupOverview, String> {
    let conn = get_db_conn()?;
    get_overview_on(&conn, group_raw_pairs)
}

fn get_overview_on(conn: &Connection, group_raw_pairs: bool) -> Result<DedupOverview, String> {
    let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;
    let total_groups = tx.query_row(
        "SELECT COUNT(*) FROM (
            SELECT group_id FROM duplicate_group_items GROUP BY group_id HAVING COUNT(*) > 1
        )",
        [],
        |row| row.get(0),
    ).map_err(|e| e.to_string())?;
    let mut files = bulk_duplicate_candidates(&tx)?;
    normalize_delete_candidates(&mut files);
    if group_raw_pairs {
        files = filter_raw_pair_delete_candidates(&tx, files, true)?;
    }
    let total_reclaimable_bytes = walk_delete_candidates(&tx, &files, group_raw_pairs, |_, _, _| {})?;
    tx.commit().map_err(|e| e.to_string())?;

    Ok(DedupOverview {
        total_groups,
        total_files: files.len() as i64,
        total_reclaimable_bytes,
    })
}

pub fn list_groups(
    page: u32,
    page_size: u32,
    sort_by: &str,
    filter: &str,
    group_raw_pairs: bool,
) -> Result<Vec<DedupGroup>, String> {
    let conn = get_db_conn()?;
    let offset = (page.saturating_sub(1)) * page_size;

    let order_clause = match sort_by {
        "size_desc" => "cur_size DESC",
        "size_asc" => "cur_size ASC",
        "count_desc" => "cur_count DESC",
        "count_asc" => "cur_count ASC",
        _ => "cur_size DESC",
    };

    let filter_clause = match filter {
        "unreviewed" => {
            "WHERE reviewed = 0 AND (SELECT COUNT(*) FROM duplicate_group_items WHERE group_id = duplicate_groups.id) > 1"
        }
        "reviewed" => {
            "WHERE reviewed = 1 AND (SELECT COUNT(*) FROM duplicate_group_items WHERE group_id = duplicate_groups.id) > 1"
        }
        _ => {
            "WHERE (SELECT COUNT(*) FROM duplicate_group_items WHERE group_id = duplicate_groups.id) > 1"
        }
    };

    let query = format!(
        "SELECT id, hash, file_size, 
                (SELECT COUNT(*) FROM duplicate_group_items WHERE group_id = duplicate_groups.id) as cur_count,
                ((SELECT COUNT(*) FROM duplicate_group_items WHERE group_id = duplicate_groups.id) * file_size) as cur_size,
                reviewed, updated_at
         FROM duplicate_groups
         {}
         ORDER BY {}
         {}",
        filter_clause,
        order_clause,
        if page_size == 0 { "" } else { "LIMIT ?1 OFFSET ?2" }
    );

    let mut stmt = conn.prepare(&query).map_err(|e| e.to_string())?;
    let query_params = if page_size == 0 {
        Vec::new()
    } else {
        vec![page_size, offset]
    };
    let groups_iter = stmt
        .query_map(rusqlite::params_from_iter(query_params), |row| {
            Ok(DedupGroup {
                id: row.get(0)?,
                hash: row.get(1)?,
                file_size: row.get(2)?,
                file_count: row.get(3)?,
                total_size: row.get(4)?,
                reviewed: row.get(5)?,
                updated_at: row.get(6)?,
                items: Vec::new(),
            })
        })
        .map_err(|e| e.to_string())?;

    let protected_ids = if group_raw_pairs {
        raw_pair_protected_delete_ids(&conn)?
    } else {
        HashSet::new()
    };
    let mut groups = Vec::new();
    for g in groups_iter {
        if let Ok(mut group) = g {
            // Fetch items
            group.items = get_group_items(&conn, group.id, group_raw_pairs, &protected_ids)?;
            groups.push(group);
        }
    }

    Ok(groups)
}

fn get_group_items(
    conn: &Connection,
    group_id: i64,
    group_raw_pairs: bool,
    protected_ids: &HashSet<i64>,
) -> Result<Vec<DedupGroupItem>, String> {
    let keep_locked = pair_keep_locked(conn, group_id)?;
    let mut stmt = conn
        .prepare(
            "SELECT i.group_id, i.file_id, i.is_keep, i.is_selected, i.score, COALESCE(companion.size, 0)
         FROM duplicate_group_items i
         LEFT JOIN afiles raw ON raw.id = i.file_id AND raw.media_subtype = 'raw_jpeg_pair'
         LEFT JOIN afiles companion ON companion.id = raw.live_photo_video_id
         WHERE i.group_id = ?1
         ORDER BY i.is_keep DESC, i.score DESC",
        )
        .map_err(|e| e.to_string())?;

    let iter = stmt
        .query_map(params![group_id], |row| {
            Ok(DedupGroupItem {
                group_id: row.get(0)?,
                file_id: row.get(1)?,
                is_keep: row.get(2)?,
                is_selected: row.get(3)?,
                score: row.get(4)?,
                can_set_keep: !keep_locked,
                can_delete: false,
                companion_size: row.get(5)?,
                file: None, // Will populate shortly
            })
        })
        .map_err(|e| e.to_string())?;

    let mut items = Vec::new();
    for it in iter {
        if let Ok(mut item) = it {
            if let Ok(Some(file_info)) = AFile::get_file_info(item.file_id) {
                item.can_delete = !group_raw_pairs || raw_pair_can_delete(
                    conn,
                    item.file_id,
                    file_info.media_subtype.as_deref() == Some("raw_jpeg_pair"),
                    protected_ids,
                )?;
                item.file = Some(file_info);
            }
            items.push(item);
        }
    }

    Ok(items)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KeepRules {
    pub folder_ids: Vec<i64>,
    pub prefer_format: PreferredFormat,
    pub preferred_date: PreferredDate,
    #[serde(default)]
    pub preferred_size: PreferredSize,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PreferredFormat {
    Ignore,
    Raw,
    JpegHeic,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PreferredSize {
    Ignore,
    #[default]
    LargestDimensions,
    SmallestDimensions,
    LargestFileSize,
    SmallestFileSize,
}

impl PreferredSize {
    fn sort_key(&self, candidate: &RuleCandidate) -> (bool, i64) {
        let amount = match self {
            Self::Ignore => return (false, 0),
            Self::LargestDimensions | Self::SmallestDimensions => {
                candidate.width.max(0).saturating_mul(candidate.height.max(0))
            }
            Self::LargestFileSize | Self::SmallestFileSize => candidate.file_size,
        };
        if amount <= 0 {
            return (true, 0);
        }
        let rank = if matches!(self, Self::LargestDimensions | Self::LargestFileSize) {
            -amount
        } else {
            amount
        };
        (false, rank)
    }
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PreferredDate {
    CreatedEarliest,
    CreatedLatest,
    ModifiedEarliest,
    ModifiedLatest,
}

impl PreferredDate {
    fn sort_key(self, candidate: &RuleCandidate) -> Option<i64> {
        let timestamp = match self {
            Self::CreatedEarliest | Self::CreatedLatest => candidate.created_at,
            Self::ModifiedEarliest | Self::ModifiedLatest => candidate.modified_at,
        };
        if timestamp <= 0 {
            return None;
        }
        Some(match self {
            Self::CreatedLatest | Self::ModifiedLatest => -timestamp,
            _ => timestamp,
        })
    }
}

struct RuleCandidate {
    group_id: i64,
    is_raw_pair: bool,
    file_id: i64,
    is_keep: i64,
    path: String,
    file_type: i64,
    format_label: Option<String>,
    width: i64,
    height: i64,
    created_at: i64,
    modified_at: i64,
    file_size: i64,
}

impl RuleCandidate {
    fn matches_format(&self, format: &PreferredFormat) -> bool {
        match format {
            PreferredFormat::Ignore => false,
            PreferredFormat::Raw => self.file_type == 3,
            PreferredFormat::JpegHeic if self.file_type == 1 => match self.format_label.as_deref() {
                Some(label) if !label.is_empty() => ["JPG", "JPEG", "JPE", "JFIF", "HEIC", "HEIF", "HIF"]
                    .iter()
                    .any(|format| label.eq_ignore_ascii_case(format)),
                _ => {
                    crate::t_image::is_jpeg_path(&self.path) || crate::t_image::is_heic_path(&self.path)
                }
            },
            PreferredFormat::JpegHeic => false,
        }
    }
}

fn folder_contains_file(folder: &str, path: &str) -> bool {
    let normalize = |value: &str| {
        let value = value.replace('\\', "/");
        if cfg!(windows) {
            value.to_lowercase()
        } else {
            value
        }
    };
    let folder = normalize(folder);
    if folder.is_empty() {
        return false;
    }
    let path = normalize(path);
    path.strip_prefix(folder.trim_end_matches('/'))
        .is_some_and(|rest| rest.starts_with('/'))
}

/// Re-select KEEP within existing groups only; no file or catalog metadata changes.
pub(crate) fn apply_keep_rules_on(
    conn: &Connection,
    similar_scope: Option<&str>,
    rules: KeepRules,
) -> Result<usize, String> {
    apply_keep_rules(conn, similar_scope, rules, true)
}

fn apply_keep_rules(
    conn: &Connection,
    similar_scope: Option<&str>,
    rules: KeepRules,
    group_raw_pairs: bool,
) -> Result<usize, String> {
    if similar_scope == Some("") {
        return Err("Missing similar result scope".into());
    }
    let mut preferred_paths = Vec::with_capacity(rules.folder_ids.len());
    if !rules.folder_ids.is_empty() {
        let mut stmt = conn
            .prepare("SELECT path FROM afolders WHERE id = ?1")
            .map_err(|e| e.to_string())?;
        for id in &rules.folder_ids {
            if let Some(path) = stmt
                .query_row([id], |row| row.get::<_, String>(0))
                .optional()
                .map_err(|e| e.to_string())?
            {
                preferred_paths.push(path);
            }
        }
    }
    let (items, groups, scope) = if similar_scope.is_some() {
        (
            "similarity_group_items",
            "similarity_groups",
            " JOIN similarity_scans s ON s.id=g.scan_id WHERE s.scope_key=?1",
        )
    } else {
        (
            "duplicate_group_items",
            "duplicate_groups",
            " WHERE ?1 = ''",
        )
    };
    let pair_sql = if group_raw_pairs && similar_scope.is_none() { RAW_PAIR_MEMBER_SQL } else { "0" };
    let sql = format!(
        "SELECT i.group_id, i.file_id, i.is_keep, f.path || '/' || a.name, a.file_type, COALESCE(a.width,0), COALESCE(a.height,0), COALESCE(a.created_at,0), ({pair_sql}), a.format_label, COALESCE(a.modified_at,0), COALESCE(a.size,0) FROM {items} i JOIN {groups} g ON g.id=i.group_id JOIN afiles a ON a.id=i.file_id JOIN afolders f ON f.id=a.folder_id{scope} ORDER BY i.group_id, i.file_id"
    );
    let rows = {
        let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
        let result = stmt
            .query_map([similar_scope.unwrap_or("")], |r| {
                Ok(RuleCandidate {
                    group_id: r.get(0)?,
                    file_id: r.get(1)?,
                    is_keep: r.get(2)?,
                    path: r.get(3)?,
                    file_type: r.get(4)?,
                    width: r.get(5)?,
                    height: r.get(6)?,
                    created_at: r.get(7)?,
                    is_raw_pair: r.get(8)?,
                    format_label: r.get(9)?,
                    modified_at: r.get(10)?,
                    file_size: r.get(11)?,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        result
    };
    let mut grouped = std::collections::BTreeMap::new();
    for row in rows {
        grouped
            .entry(row.group_id)
            .or_insert_with(Vec::new)
            .push(row);
    }
    let folder_rank = |path: &str| {
        preferred_paths
            .iter()
            .position(|folder| folder_contains_file(folder, path))
            .unwrap_or(usize::MAX)
    };
    let mut changed = 0;
    for (group, mut candidates) in grouped {
        if candidates.len() < 2 {
            continue;
        }
        let similar = similar_scope.is_some();
        let rank = |candidate: &RuleCandidate| {
            let preferred_format = similar && candidate.matches_format(&rules.prefer_format);
            let size_rank = if similar {
                rules.preferred_size.sort_key(candidate)
            } else {
                (false, 0)
            };
            (
                std::cmp::Reverse(candidate.is_raw_pair),
                folder_rank(&candidate.path),
                std::cmp::Reverse(preferred_format),
                size_rank,
            )
        };
        candidates.sort_by_cached_key(rank);
        let best_rank = rank(&candidates[0]);
        let tied = &candidates[..candidates.partition_point(|candidate| rank(candidate) == best_rank)];
        // The preferred date only breaks ties; files without a usable timestamp
        // rank last instead of disabling the rule for the whole tie.
        let winner = tied
            .iter()
            .min_by_key(|candidate| {
                let time = rules.preferred_date.sort_key(candidate);
                (
                    time.is_none(),
                    time.unwrap_or(0),
                    std::cmp::Reverse(candidate.is_keep),
                )
            })
            .unwrap();
        if winner.is_keep == 1 {
            continue;
        }
        let winner = winner.file_id;
        conn.execute(&format!("UPDATE {items} SET is_keep=CASE WHEN file_id=?2 THEN 1 ELSE 0 END WHERE group_id=?1"), params![group,winner]).map_err(|e| e.to_string())?;
        if similar_scope.is_some() {
            conn.execute(
                "UPDATE similarity_groups SET representative_file_id=?2 WHERE id=?1",
                params![group, winner],
            )
            .map_err(|e| e.to_string())?;
        }
        changed += 1;
    }
    Ok(changed)
}

fn pair_keep_locked(conn: &Connection, group_id: i64) -> Result<bool, String> {
    conn.query_row(&format!(
        "SELECT EXISTS (SELECT 1 FROM duplicate_group_items i JOIN afiles a ON a.id = i.file_id
         WHERE i.group_id = ?1 AND i.is_keep = 1 AND ({RAW_PAIR_MEMBER_SQL}))"
    ), [group_id], |row| row.get(0)).map_err(|e| e.to_string())
}

pub fn set_keep(group_id: i64, file_id: i64, group_raw_pairs: bool) -> Result<(), String> {
    let mut conn = get_db_conn()?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;

    if group_raw_pairs && pair_keep_locked(&tx, group_id)? {
        return Err("The complete RAW pair must be kept".into());
    }

    // Clear is_keep for everyone in the group
    tx.execute(
        "UPDATE duplicate_group_items SET is_keep = 0 WHERE group_id = ?1",
        params![group_id],
    )
    .map_err(|e| e.to_string())?;

    // Set the target
    let changed = tx
        .execute(
            "UPDATE duplicate_group_items SET is_keep = 1 WHERE group_id = ?1 AND file_id = ?2",
            params![group_id, file_id],
        )
        .map_err(|e| e.to_string())?;

    if changed == 0 {
        return Err("Item not found in group".into());
    }

    // Mark group as reviewed
    tx.execute(
        "UPDATE duplicate_groups SET reviewed = 1 WHERE id = ?1",
        params![group_id],
    )
    .map_err(|e| e.to_string())?;

    tx.commit().map_err(|e| e.to_string())?;
    Ok(())
}

/// A paired companion is deleted through its RAW. Older scan results can
/// still mark the companion KEEP independently, so protect its RAW as well.
fn raw_pair_protected_delete_ids(conn: &Connection) -> Result<HashSet<i64>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT live_photo_video_id FROM afiles
             WHERE media_subtype = 'raw_jpeg_pair' AND live_photo_video_id IS NOT NULL
             UNION
             SELECT raw.id FROM afiles raw
             JOIN duplicate_group_items keep ON keep.file_id = raw.live_photo_video_id
             WHERE raw.media_subtype = 'raw_jpeg_pair' AND keep.is_keep = 1",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt.query_map([], |row| row.get(0)).map_err(|e| e.to_string())?;
    rows.collect::<Result<HashSet<_>, _>>().map_err(|e| e.to_string())
}

fn raw_pair_has_exact_keep(conn: &Connection, file_id: i64) -> Result<bool, String> {
    let mut stmt = conn.prepare_cached(
        "SELECT NOT EXISTS (
             SELECT 1 FROM afiles raw
             JOIN afiles jpeg ON jpeg.id = raw.live_photo_video_id
             WHERE raw.id = ?1 AND raw.media_subtype = 'raw_jpeg_pair'
               AND NOT EXISTS (
                   SELECT 1 FROM duplicate_group_items dup
                   JOIN duplicate_group_items keep ON keep.group_id = dup.group_id AND keep.is_keep = 1
                   JOIN afiles kept_raw ON kept_raw.id = keep.file_id AND kept_raw.media_subtype = 'raw_jpeg_pair'
                   JOIN afiles kept_jpeg ON kept_jpeg.id = kept_raw.live_photo_video_id
                   JOIN file_hashes h ON h.file_id = jpeg.id AND h.file_size = jpeg.size
                       AND h.mtime = COALESCE(jpeg.modified_at, 0)
                   JOIN file_hashes kh ON kh.file_id = kept_jpeg.id AND kh.file_size = kept_jpeg.size
                       AND kh.mtime = COALESCE(kept_jpeg.modified_at, 0)
                   JOIN file_hashes r ON r.file_id = raw.id AND r.file_size = raw.size
                       AND r.mtime = COALESCE(raw.modified_at, 0)
                   JOIN file_hashes kr ON kr.file_id = kept_raw.id AND kr.file_size = kept_raw.size
                       AND kr.mtime = COALESCE(kept_raw.modified_at, 0)
                   WHERE dup.file_id = raw.id AND h.hash = kh.hash AND h.file_size = kh.file_size
                     AND r.hash = kr.hash AND r.file_size = kr.file_size
               )
         )",
    ).map_err(|e| e.to_string())?;
    stmt.query_row([file_id], |row| row.get(0)).map_err(|e| e.to_string())
}

fn bulk_duplicate_candidates(conn: &Connection) -> Result<Vec<crate::t_cmds::BatchDeleteFile>, String> {
    // A group without a keeper cannot authorize bulk deletion.
    let mut stmt = conn.prepare(
        "SELECT a.id, f.path || '/' || a.name
         FROM duplicate_group_items dgi
         JOIN afiles a ON dgi.file_id = a.id
         JOIN afolders f ON a.folder_id = f.id
         WHERE dgi.is_keep = 0
           AND EXISTS (SELECT 1 FROM duplicate_group_items k
                       WHERE k.group_id = dgi.group_id AND k.is_keep = 1)",
    ).map_err(|e| e.to_string())?;
    let rows = stmt.query_map([], |row| Ok(crate::t_cmds::BatchDeleteFile {
        file_id: row.get(0)?,
        file_path: row.get(1)?,
    })).map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

fn normalize_delete_candidates(files: &mut Vec<crate::t_cmds::BatchDeleteFile>) {
    files.sort_unstable_by_key(|file| file.file_id);
    files.dedup_by_key(|file| file.file_id);
}

pub(crate) fn filter_raw_pair_delete_candidates(
    conn: &Connection,
    files: Vec<crate::t_cmds::BatchDeleteFile>,
    require_exact_keep: bool,
) -> Result<Vec<crate::t_cmds::BatchDeleteFile>, String> {
    let protected_ids = raw_pair_protected_delete_ids(conn)?;
    let mut raw_primaries = HashSet::new();
    if require_exact_keep {
        let mut stmt = conn.prepare(
            "SELECT a.id FROM json_each(?1) candidate
             JOIN afiles a ON a.id = candidate.value
             WHERE a.media_subtype = 'raw_jpeg_pair'",
        ).map_err(|error| error.to_string())?;
        for chunk in files.chunks(2048) {
            let ids = chunk.iter().map(|file| file.file_id).collect::<Vec<_>>();
            let ids = serde_json::to_string(&ids).map_err(|error| error.to_string())?;
            let rows = stmt.query_map([ids], |row| row.get::<_, i64>(0))
                .map_err(|error| error.to_string())?;
            for row in rows {
                raw_primaries.insert(row.map_err(|error| error.to_string())?);
            }
        }
    }
    let mut safe_files = Vec::with_capacity(files.len());
    for file in files {
        if raw_pair_can_delete(conn, file.file_id, raw_primaries.contains(&file.file_id), &protected_ids)? {
            safe_files.push(file);
        }
    }
    Ok(safe_files)
}

fn raw_pair_can_delete(
    conn: &Connection,
    file_id: i64,
    require_exact_keep: bool,
    protected_ids: &HashSet<i64>,
) -> Result<bool, String> {
    Ok(!protected_ids.contains(&file_id)
        && (!require_exact_keep || raw_pair_has_exact_keep(conn, file_id)?))
}

struct DeleteCompanion {
    id: i64,
    path: String,
    size: i64,
    is_raw_pair: bool,
}

fn walk_delete_candidates(
    conn: &Connection,
    files: &[crate::t_cmds::BatchDeleteFile],
    group_raw_pairs: bool,
    mut visit: impl FnMut(&crate::t_cmds::BatchDeleteFile, i64, Option<DeleteCompanion>),
) -> Result<i64, String> {
    let mut stmt = conn.prepare(
        "SELECT a.size, a.media_subtype, c.id, f.path || '/' || c.name, c.size
         FROM json_each(?1) AS candidate
         LEFT JOIN afiles a ON a.id = candidate.value
         LEFT JOIN afiles c ON c.id = a.live_photo_video_id
         LEFT JOIN afolders f ON f.id = c.folder_id
         ORDER BY candidate.key",
    ).map_err(|e| e.to_string())?;
    let primary_ids: HashSet<_> = files.iter().map(|file| file.file_id).collect();
    let mut seen_components = HashSet::new();
    let mut bytes = 0i64;
    for chunk in files.chunks(2048) {
        let ids = chunk.iter().map(|file| file.file_id).collect::<Vec<_>>();
        let ids = serde_json::to_string(&ids).map_err(|error| error.to_string())?;
        let mut rows = stmt.query([ids]).map_err(|error| error.to_string())?;
        for file in chunk {
            let row = rows.next().map_err(|error| error.to_string())?
                .ok_or("Missing dedup candidate metadata")?;
            let size: i64 = row.get(0).map_err(|error| error.to_string())?;
            let subtype: Option<String> = row.get(1).map_err(|error| error.to_string())?;
            let id: Option<i64> = row.get(2).map_err(|error| error.to_string())?;
            let path: Option<String> = row.get(3).map_err(|error| error.to_string())?;
            let companion_size: Option<i64> = row.get(4).map_err(|error| error.to_string())?;
            bytes = bytes.saturating_add(size);

            let is_raw_pair = subtype.as_deref() == Some("raw_jpeg_pair");
            let companion = match (id, path) {
                (Some(id), Some(path)) if (!is_raw_pair || group_raw_pairs)
                    && !primary_ids.contains(&id) && seen_components.insert(id) =>
                {
                    let size = companion_size.unwrap_or(0);
                    bytes = bytes.saturating_add(size);
                    Some(DeleteCompanion { id, path, size, is_raw_pair })
                }
                _ => None,
            };
            visit(file, size, companion);
        }
    }
    Ok(bytes)
}

fn delete_preview(
    conn: &Connection,
    files: &[crate::t_cmds::BatchDeleteFile],
    group_raw_pairs: bool,
) -> Result<(DedupDeletePreview, HashMap<i64, Vec<(i64, String)>>), String> {
    let mut components = HashMap::new();
    let mut raw_companion_count = 0;
    let mut hasher = blake3::Hasher::new();
    hasher.update(&[u8::from(group_raw_pairs)]);
    let bytes = walk_delete_candidates(conn, files, group_raw_pairs, |file, size, companion| {
        hasher.update(&[0]);
        hasher.update(&file.file_id.to_le_bytes());
        hasher.update(&size.to_le_bytes());
        hasher.update(blake3::hash(file.file_path.as_bytes()).as_bytes());
        if let Some(companion) = companion {
            hasher.update(&[1]);
            hasher.update(&companion.id.to_le_bytes());
            hasher.update(&companion.size.to_le_bytes());
            hasher.update(blake3::hash(companion.path.as_bytes()).as_bytes());
            raw_companion_count += usize::from(companion.is_raw_pair);
            components.insert(file.file_id, vec![(companion.id, companion.path)]);
        }
    })?;
    Ok((DedupDeletePreview {
        token: hasher.finalize().to_hex().to_string(),
        item_count: files.len(),
        raw_companion_count,
        bytes,
    }, components))
}

pub(crate) fn similar_delete_candidates(
    conn: &Connection,
    group_id: i64,
    ids: &[i64],
    group_raw_pairs: bool,
) -> Result<Vec<crate::t_cmds::BatchDeleteFile>, String> {
    let mut stmt = conn.prepare(
        "SELECT a.id, f.path || '/' || a.name
         FROM similarity_group_items item
         JOIN similarity_groups g ON g.id = item.group_id
         JOIN afiles a ON a.id = item.file_id
         JOIN afolders f ON f.id = a.folder_id
         WHERE item.group_id = ?1 AND item.file_id = ?2 AND item.is_keep = 0
           AND EXISTS (SELECT 1 FROM similarity_group_items k
                       WHERE k.group_id = item.group_id AND k.is_keep = 1)
           AND NOT EXISTS (
               SELECT 1 FROM similarity_group_items k
               JOIN similarity_groups kg ON kg.id = k.group_id
               WHERE kg.scan_id = g.scan_id AND k.is_keep = 1
                 AND (k.file_id = a.id OR
                      (?3 AND a.media_subtype = 'raw_jpeg_pair' AND k.file_id = a.live_photo_video_id))
           )",
    ).map_err(|e| e.to_string())?;
    let mut files = Vec::new();
    let mut seen_ids = HashSet::new();
    for &id in ids {
        if !seen_ids.insert(id) {
            continue;
        }
        let rows = stmt
            .query_map(params![group_id, id, group_raw_pairs], |row| {
                Ok(crate::t_cmds::BatchDeleteFile {
                    file_id: row.get(0)?,
                    file_path: row.get(1)?,
                })
            })
            .map_err(|e| e.to_string())?;
        for row in rows {
            files.push(row.map_err(|e| e.to_string())?);
        }
    }
    Ok(files)
}

pub fn delete(request: DedupDeleteRequest) -> Result<DedupDeleteResult, String> {
    let mut conn = get_db_conn()?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;

    let mut files_to_delete: Vec<crate::t_cmds::BatchDeleteFile> = Vec::new();
    let mut similar_requested_count = None;

    if let Some(group_id) = request.similar_group_id {
        if request.delete_all || request.group_ids.is_some() {
            return Err("Invalid similar dedup delete request".to_string());
        }
        let ids = request.file_ids.as_deref().ok_or("Missing similar dedup file IDs")?;
        similar_requested_count = Some(ids.iter().collect::<HashSet<_>>().len());
        files_to_delete = similar_delete_candidates(&tx, group_id, ids, request.group_raw_pairs)?;
    } else if request.delete_all {
        files_to_delete = bulk_duplicate_candidates(&tx)?;
    } else if let Some(ids) = request.file_ids {
        let mut stmt = tx
            .prepare(
                "SELECT a.id, f.path || '/' || a.name
                 FROM duplicate_group_items dgi
                 JOIN afiles a ON dgi.file_id = a.id
                 JOIN afolders f ON a.folder_id = f.id
                 WHERE dgi.file_id = ?1 AND dgi.is_keep = 0
                   AND EXISTS (SELECT 1 FROM duplicate_group_items k
                               WHERE k.group_id = dgi.group_id AND k.is_keep = 1)",
            )
            .map_err(|e| e.to_string())?;
        for id in ids {
            let mut iter = stmt
                .query_map(params![id], |row| Ok((row.get(0)?, row.get(1)?)))
                .map_err(|e| e.to_string())?;
            for row in &mut iter {
                let (file_id, file_path) = row.map_err(|e| e.to_string())?;
                files_to_delete.push(crate::t_cmds::BatchDeleteFile { file_id, file_path });
            }
        }
    } else if let Some(gids) = request.group_ids {
        for gid in gids {
            let mut stmt = tx
                .prepare(
                    "SELECT a.id, f.path || '/' || a.name
                    FROM duplicate_group_items dgi
                    JOIN afiles a ON dgi.file_id = a.id
                    JOIN afolders f ON a.folder_id = f.id
                    WHERE dgi.group_id = ?1 AND dgi.is_keep = 0 AND dgi.is_selected = 1",
                )
                .map_err(|e| e.to_string())?;

            let mut iter = stmt
                .query_map(params![gid], |row| Ok((row.get(0)?, row.get(1)?)))
                .map_err(|e| e.to_string())?;
            for row in &mut iter {
                let (file_id, file_path) = row.map_err(|e| e.to_string())?;
                files_to_delete.push(crate::t_cmds::BatchDeleteFile { file_id, file_path });
            }
        }
    } else {
        let mut stmt = tx
            .prepare(
                "SELECT a.id, f.path || '/' || a.name
                 FROM duplicate_group_items dgi
                 JOIN afiles a ON dgi.file_id = a.id
                 JOIN afolders f ON a.folder_id = f.id
                 WHERE dgi.is_keep = 0 AND dgi.is_selected = 1",
            )
            .map_err(|e| e.to_string())?;
        let mut iter = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
            .map_err(|e| e.to_string())?;
        for row in &mut iter {
            let (file_id, file_path) = row.map_err(|e| e.to_string())?;
            files_to_delete.push(crate::t_cmds::BatchDeleteFile { file_id, file_path });
        }
    }

    normalize_delete_candidates(&mut files_to_delete);
    let requested_count = similar_requested_count.unwrap_or(files_to_delete.len());
    if request.group_raw_pairs {
        files_to_delete = filter_raw_pair_delete_candidates(
            &tx, files_to_delete, request.similar_group_id.is_none(),
        )?;
    }
    let skipped_count = requested_count - files_to_delete.len();
    let (preview, components) = delete_preview(&tx, &files_to_delete, request.group_raw_pairs)?;
    if request.preview {
        return Ok(DedupDeleteResult {
            deleted_file_ids: Vec::new(),
            affected_folder_paths: Vec::new(),
            deleted_count: 0,
            failed_count: skipped_count,
            errors: Vec::new(),
            trash_failed_file_ids: Vec::new(),
            preview: Some(preview),
        });
    }
    if request.plan_token.as_deref() != Some(preview.token.as_str()) {
        return Err("dedup_plan_changed: Open the confirmation again.".into());
    }
    tx.commit().map_err(|e| e.to_string())?;

    let candidate_ids: HashSet<i64> = files_to_delete.iter().map(|file| file.file_id).collect();
    let source_folders: Vec<(i64, String)> = files_to_delete
        .iter()
        .filter_map(|file| {
            file.file_path
                .rsplit_once('/')
                .map(|(folder, _)| (file.file_id, folder.to_string()))
        })
        .collect();
    let delete_result = crate::t_cmds::delete_files_with_components(files_to_delete, request.permanently, Some(components))?;
    let deleted_ids: HashSet<i64> = delete_result.deleted_file_ids.iter().copied().collect();
    let affected_folder_paths = source_folders
        .into_iter()
        .filter_map(|(id, path)| deleted_ids.contains(&id).then_some(path))
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();
    let deleted_count = delete_result
        .deleted_file_ids
        .iter()
        .filter(|file_id| candidate_ids.contains(file_id))
        .count();
    let deleted_file_ids = if request.delete_all {
        Vec::new()
    } else {
        delete_result.deleted_file_ids
    };
    let mut failures: Vec<String> = Vec::new();

    // Clean up empty groups. Report cleanup errors without hiding earlier partial deletes.
    match get_db_conn() {
        Ok(conn) => {
            if let Err(e) = conn.execute(
                "DELETE FROM duplicate_groups 
                 WHERE id NOT IN (SELECT DISTINCT group_id FROM duplicate_group_items)",
                [],
            ) {
                failures.push(format!("Failed to clean up empty duplicate groups: {}", e));
            }
        }
        Err(e) => failures.push(format!(
            "Failed to open DB for duplicate group cleanup: {}",
            e
        )),
    }

    Ok(DedupDeleteResult {
        deleted_file_ids,
        affected_folder_paths,
        deleted_count,
        failed_count: skipped_count + delete_result.failed_count + failures.len(),
        errors: failures,
        trash_failed_file_ids: delete_result.trash_failed_file_ids,
        preview: None,
    })
}

#[cfg(test)]
mod keep_rules_tests {
    use super::*;

    #[test]
    fn paired_companions_are_deleted_only_through_a_raw_without_a_keep_companion() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("
            CREATE TABLE afiles(id INTEGER PRIMARY KEY, media_subtype TEXT, live_photo_video_id INTEGER);
            INSERT INTO afiles VALUES (1,'raw_jpeg_pair',2),(2,NULL,NULL),
                (3,'raw_jpeg_pair',4),(4,NULL,NULL),(5,NULL,NULL);
            CREATE TABLE duplicate_group_items(file_id INTEGER, is_keep INTEGER);
            INSERT INTO duplicate_group_items VALUES (1,0),(2,1),(3,0),(4,0),(5,0);
        ").unwrap();
        assert_eq!(
            raw_pair_protected_delete_ids(&conn).unwrap(),
            HashSet::from([1, 2, 4])
        );
    }

    #[test]
    fn similar_delete_protects_keep_items_and_raw_pairs_in_the_current_scan() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE afolders(id INTEGER PRIMARY KEY, path TEXT);
             INSERT INTO afolders VALUES (1,'/photos');
             CREATE TABLE afiles(id INTEGER PRIMARY KEY, folder_id INTEGER, name TEXT,
                                 media_subtype TEXT, live_photo_video_id INTEGER);
             INSERT INTO afiles VALUES (1,1,'a.raw','raw_jpeg_pair',2),(2,1,'a.jpg',NULL,NULL),
                 (3,1,'b.raw','raw_jpeg_pair',4),(4,1,'b.jpg',NULL,NULL),(5,1,'c.jpg',NULL,NULL);
             CREATE TABLE similarity_groups(id INTEGER PRIMARY KEY, scan_id INTEGER);
             INSERT INTO similarity_groups VALUES (10,1),(11,1),(20,2),(30,1);
             CREATE TABLE similarity_group_items(group_id INTEGER, file_id INTEGER, is_keep INTEGER);
             INSERT INTO similarity_group_items VALUES
                 (10,1,0),(10,3,0),(10,5,1),(11,2,1),(20,4,1),(30,3,0);",
        ).unwrap();
        let files = similar_delete_candidates(&conn, 10, &[1, 2, 3, 3, 5, 99], true).unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].file_id, 3);
        assert!(similar_delete_candidates(&conn, 30, &[3], true).unwrap().is_empty());
    }

    #[test]
    fn exact_scan_keeps_pair_for_either_match_and_only_removes_identical_pairs() {
        let mut conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE afolders(id INTEGER PRIMARY KEY,path TEXT);
             INSERT INTO afolders VALUES (1,'/photos');
             CREATE TABLE afiles(id INTEGER PRIMARY KEY,folder_id INTEGER,name TEXT,file_type INTEGER,
                 size INTEGER,width INTEGER,height INTEGER,created_at INTEGER,taken_date INTEGER,
                 media_subtype TEXT,live_photo_video_id INTEGER,format_label TEXT,modified_at INTEGER);
             INSERT INTO afiles VALUES
                 (1,1,'a.raw',3,100,10,10,1,1,'raw_jpeg_pair',2,NULL,1),
                 (2,1,'a.jpg',1,20,10,10,1,1,NULL,NULL,'JPG',1),
                 (3,1,'b.raw',3,100,10,10,2,2,'raw_jpeg_pair',4,NULL,2),
                 (4,1,'b.jpg',1,20,10,10,2,2,NULL,NULL,'JPG',2),
                 (5,1,'c.raw',3,100,10,10,3,3,'raw_jpeg_pair',6,NULL,3),
                 (6,1,'c.jpg',1,20,10,10,3,3,NULL,NULL,'JPG',3),
                 (7,1,'loose.raw',3,100,10,10,0,0,NULL,NULL,NULL,0),
                 (8,1,'loose.jpg',1,20,10,10,0,0,NULL,NULL,'JPG',0),
                 (9,1,'d.raw',3,100,10,10,4,4,'raw_jpeg_pair',10,NULL,4),
                 (10,1,'d.jpg',1,20,10,10,4,4,NULL,NULL,'JPG',4),
                 (11,1,'d-copy.raw',3,100,10,10,5,5,'raw_jpeg_pair',12,NULL,5),
                 (12,1,'d-copy.jpg',1,20,10,10,5,5,NULL,NULL,'JPG',5),
                 (13,1,'other.raw',3,100,10,10,6,6,'raw_jpeg_pair',14,NULL,6),
                 (14,1,'other.jpg',1,20,10,10,6,6,NULL,NULL,'JPG',6);
             CREATE TABLE file_hashes(file_id INTEGER,hash TEXT,file_size INTEGER,mtime INTEGER);
             INSERT INTO file_hashes VALUES (1,'raw',100,1),(2,'jpeg-a',20,1),(3,'raw',100,2),
                 (4,'jpeg-b',20,2),(5,'raw',100,3),(6,'jpeg-a',20,3),(7,'raw',100,0),
                 (8,'jpeg-a',20,0),(9,'raw',100,4),(10,'jpeg-c',20,4),
                 (11,'raw',100,5),(12,'jpeg-c',20,5),(13,'other-raw',100,6),(14,'jpeg-a',20,6);
             CREATE TABLE duplicate_groups(id INTEGER PRIMARY KEY,hash TEXT,file_size INTEGER,
                 file_count INTEGER,total_size INTEGER,updated_at INTEGER,UNIQUE(hash,file_size));
             CREATE TABLE duplicate_group_items(group_id INTEGER,file_id INTEGER,is_keep INTEGER,
                 is_selected INTEGER,score REAL,PRIMARY KEY(group_id,file_id));",
        ).unwrap();

        let rules = KeepRules {
            folder_ids: vec![],
            prefer_format: PreferredFormat::Raw,
            preferred_date: PreferredDate::ModifiedEarliest,
            preferred_size: PreferredSize::LargestDimensions,
        };
        rebuild_duplicate_groups(&mut conn, None, Some(rules), true).unwrap();
        let mut stmt = conn.prepare("SELECT file_id, is_keep FROM duplicate_group_items ORDER BY file_id").unwrap();
        let results = stmt.query_map([], |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?)))
            .unwrap().collect::<Result<Vec<_>, _>>().unwrap();
        assert_eq!(results, vec![(1,1),(2,1),(5,0),(7,0),(8,0),(9,1),(11,0)]);
        assert!(raw_pair_has_exact_keep(&conn, 5).unwrap());
        assert!(!raw_pair_has_exact_keep(&conn, 3).unwrap());

        let files = [crate::t_cmds::BatchDeleteFile { file_id: 5, file_path: "/photos/c.raw".into() }];
        let (enabled, components) = delete_preview(&conn, &files, true).unwrap();
        assert_eq!(enabled.raw_companion_count, 1);
        assert_eq!(enabled.bytes, 120);
        assert_eq!(components[&5], vec![(6, "/photos/c.jpg".into())]);
        let (disabled, components) = delete_preview(&conn, &files, false).unwrap();
        assert_eq!(disabled.raw_companion_count, 0);
        assert_eq!(disabled.bytes, 100);
        assert!(components.is_empty());
        assert_ne!(enabled.token, disabled.token);

        let overview = get_overview_on(&conn, true).unwrap();
        assert_eq!((overview.total_files, overview.total_reclaimable_bytes), (4, 360));
        conn.execute("UPDATE file_hashes SET hash='changed' WHERE file_id=6", []).unwrap();
        let overview = get_overview_on(&conn, true).unwrap();
        assert_eq!((overview.total_files, overview.total_reclaimable_bytes), (3, 240));
    }

    #[test]
    fn exact_duplicates_prefer_an_existing_pair_before_folder_rank() {
        let mut conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("CREATE TABLE afolders(id INTEGER PRIMARY KEY,path TEXT);
            INSERT INTO afolders VALUES(1,'/archive'),(2,'/imports');
            CREATE TABLE afiles(id INTEGER PRIMARY KEY,folder_id INTEGER,name TEXT,file_type INTEGER,width INTEGER,height INTEGER,created_at INTEGER,taken_date INTEGER,media_subtype TEXT,live_photo_video_id INTEGER,format_label TEXT,modified_at INTEGER,size INTEGER);
            INSERT INTO afiles VALUES
                (1,1,'a.raw',3,100,100,10,1,NULL,NULL,NULL,10,100),
                (2,2,'a.raw',3,100,100,20,0,'raw_jpeg_pair',4,NULL,20,100),
                (3,2,'b.raw',3,100,100,5,1,'raw_jpeg_pair',99,NULL,5,100),
                (4,2,'a.jpg',1,100,100,20,0,NULL,NULL,NULL,20,100);
            CREATE TABLE file_hashes(file_id INTEGER,hash TEXT,file_size INTEGER,mtime INTEGER);
            INSERT INTO file_hashes VALUES(1,'raw',100,10),(2,'raw',100,20),(3,'raw',100,5);
            CREATE TABLE duplicate_groups(id INTEGER PRIMARY KEY,hash TEXT,file_size INTEGER,file_count INTEGER,total_size INTEGER,updated_at INTEGER);
            CREATE TABLE duplicate_group_items(group_id INTEGER,file_id INTEGER,is_keep INTEGER,is_selected INTEGER,score REAL);").unwrap();
        let winner = |conn: &Connection| {
            conn.query_row("SELECT file_id FROM duplicate_group_items WHERE is_keep=1", [], |row| row.get::<_, i64>(0)).unwrap()
        };
        rebuild_duplicate_groups(&mut conn, None, None, true).unwrap();
        assert_eq!(winner(&conn), 2);
        rebuild_duplicate_groups(&mut conn, Some(&[1, 2, 3]), None, true).unwrap();
        assert_eq!(winner(&conn), 2);

        let rules = |folder_ids| KeepRules {
            folder_ids,
            prefer_format: PreferredFormat::JpegHeic,
            preferred_date: PreferredDate::CreatedEarliest,
            preferred_size: PreferredSize::LargestDimensions,
        };
        conn.execute("UPDATE afolders SET path = '/renamed' WHERE id = 1", []).unwrap();
        rebuild_duplicate_groups(&mut conn, None, Some(rules(vec![999, 1])), true).unwrap();
        assert_eq!(winner(&conn), 2);
        apply_keep_rules_on(&conn, None, rules(vec![])).unwrap();
        assert_eq!(winner(&conn), 2);
    }

    #[test]
    fn folder_matching_has_boundaries_and_includes_descendants() {
        assert!(folder_contains_file("/photos", "/photos/trip/a.jpg"));
        assert!(!folder_contains_file("/photos", "/photos-old/a.jpg"));
        assert!(folder_contains_file("/", "/photos/a.jpg"));
        assert!(!folder_contains_file("", "/photos/a.jpg"));
    }

    #[test]
    fn apply_rules_is_scoped_and_preserves_keep_on_ties() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("CREATE TABLE afolders(id INTEGER PRIMARY KEY,path TEXT);
            INSERT INTO afolders VALUES(1,'/archive'),(2,'/imports');
            CREATE TABLE afiles(id INTEGER PRIMARY KEY,folder_id INTEGER,name TEXT,file_type INTEGER,width INTEGER,height INTEGER,created_at INTEGER,media_subtype TEXT,live_photo_video_id INTEGER,format_label TEXT,modified_at INTEGER,size INTEGER);
            INSERT INTO afiles VALUES(1,1,'a.jpg',1,100,100,10,NULL,NULL,NULL,50,400),(2,2,'a.raw',3,200,200,20,NULL,NULL,'RAW',20,100),(3,2,'b.png',1,400,400,40,NULL,NULL,'PNG',40,100),(4,2,'b.heic',1,300,300,30,NULL,NULL,'HEIC',25,200);
            CREATE TABLE duplicate_groups(id INTEGER PRIMARY KEY);
            INSERT INTO duplicate_groups VALUES(1);
            CREATE TABLE duplicate_group_items(group_id INTEGER,file_id INTEGER,is_keep INTEGER);
            INSERT INTO duplicate_group_items VALUES(1,1,0),(1,2,1);
            CREATE TABLE similarity_scans(id INTEGER PRIMARY KEY,scope_key TEXT);
            INSERT INTO similarity_scans VALUES(1,'active'),(2,'other');
            CREATE TABLE similarity_groups(id INTEGER PRIMARY KEY,scan_id INTEGER,representative_file_id INTEGER);
            INSERT INTO similarity_groups VALUES(1,1,1),(2,2,1);
            CREATE TABLE similarity_group_items(group_id INTEGER,file_id INTEGER,is_keep INTEGER);
            INSERT INTO similarity_group_items VALUES(1,1,1),(1,2,0),(1,3,0),(1,4,0),(2,1,1),(2,2,0);").unwrap();
        let rule = |folder_ids: Vec<i64>, format, preferred_date| KeepRules {
            folder_ids,
            prefer_format: format,
            preferred_date,
            preferred_size: PreferredSize::LargestDimensions,
        };
        assert_eq!(
            apply_keep_rules_on(&conn, None, rule(vec![], PreferredFormat::Raw, PreferredDate::ModifiedEarliest)).unwrap(),
            0
        );
        assert_eq!(
            apply_keep_rules_on(
                &conn,
                None,
                rule(vec![1], PreferredFormat::JpegHeic, PreferredDate::ModifiedEarliest)
            )
            .unwrap(),
            1
        );
        assert_eq!(
            apply_keep_rules_on(
                &conn,
                Some("active"),
                rule(vec![], PreferredFormat::Raw, PreferredDate::ModifiedEarliest)
            )
            .unwrap(),
            1
        );
        let winner = |conn: &Connection, group| {
            conn.query_row(
                "SELECT file_id FROM similarity_group_items WHERE group_id=?1 AND is_keep=1",
                [group],
                |r| r.get::<_, i64>(0),
            )
            .unwrap()
        };
        assert_eq!(winner(&conn, 1), 2);
        assert_eq!(winner(&conn, 2), 1);
        apply_keep_rules_on(
            &conn,
            Some("active"),
            rule(vec![], PreferredFormat::JpegHeic, PreferredDate::ModifiedEarliest),
        )
        .unwrap();
        assert_eq!(winner(&conn, 1), 4);
        let dates_only = KeepRules {
            preferred_size: PreferredSize::Ignore,
            ..rule(vec![], PreferredFormat::Ignore, PreferredDate::ModifiedEarliest)
        };
        apply_keep_rules_on(&conn, Some("active"), dates_only).unwrap();
        assert_eq!(winner(&conn, 1), 2);

        for (preferred_size, expected) in [
            (PreferredSize::SmallestDimensions, 1),
            (PreferredSize::LargestFileSize, 1),
            (PreferredSize::SmallestFileSize, 4),
        ] {
            let rules = KeepRules {
                preferred_size,
                ..rule(vec![], PreferredFormat::JpegHeic, PreferredDate::ModifiedEarliest)
            };
            apply_keep_rules_on(&conn, Some("active"), rules).unwrap();
            assert_eq!(winner(&conn, 1), expected);
        }
        conn.execute("UPDATE afiles SET width=100, height=100 WHERE id=4", [])
            .unwrap();
        apply_keep_rules_on(
            &conn,
            Some("active"),
            rule(vec![], PreferredFormat::JpegHeic, PreferredDate::CreatedEarliest),
        )
        .unwrap();
        assert_eq!(winner(&conn, 1), 1);
        apply_keep_rules_on(
            &conn,
            Some("active"),
            rule(vec![], PreferredFormat::JpegHeic, PreferredDate::CreatedLatest),
        )
        .unwrap();
        assert_eq!(winner(&conn, 1), 4);
        for (preferred_date, expected) in [(PreferredDate::ModifiedLatest, 1), (PreferredDate::ModifiedEarliest, 4)] {
            apply_keep_rules_on(&conn, Some("active"), rule(vec![], PreferredFormat::JpegHeic, preferred_date)).unwrap();
            assert_eq!(winner(&conn, 1), expected);
        }
        conn.execute("UPDATE afiles SET modified_at=NULL WHERE id=4", []).unwrap();
        apply_keep_rules_on(&conn, Some("active"), rule(vec![], PreferredFormat::JpegHeic, PreferredDate::ModifiedLatest)).unwrap();
        // A missing timestamp ranks last: the tied file that still has one wins.
        assert_eq!(winner(&conn, 1), 1);
    }

    #[test]
    fn missing_timestamps_rank_last_in_date_fallback() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("CREATE TABLE afolders(id INTEGER PRIMARY KEY, path TEXT);
            INSERT INTO afolders VALUES(1,'/photos');
            CREATE TABLE afiles(id INTEGER PRIMARY KEY, folder_id INTEGER, name TEXT, file_type INTEGER, width INTEGER, height INTEGER, created_at INTEGER, media_subtype TEXT, live_photo_video_id INTEGER, format_label TEXT, modified_at INTEGER, size INTEGER);
            INSERT INTO afiles VALUES
                (1,1,'a.jpg',1,100,100,10,NULL,NULL,'JPG',NULL,100),
                (2,1,'b.jpg',1,100,100,10,NULL,NULL,'JPG',50,100);
            CREATE TABLE duplicate_groups(id INTEGER PRIMARY KEY);
            INSERT INTO duplicate_groups VALUES(1);
            CREATE TABLE duplicate_group_items(group_id INTEGER, file_id INTEGER, is_keep INTEGER);
            INSERT INTO duplicate_group_items VALUES(1,1,1),(1,2,0);").unwrap();
        let rules = || KeepRules {
            folder_ids: vec![],
            prefer_format: PreferredFormat::Ignore,
            preferred_date: PreferredDate::ModifiedEarliest,
            preferred_size: PreferredSize::Ignore,
        };
        let winner = |conn: &Connection| {
            conn.query_row(
                "SELECT file_id FROM duplicate_group_items WHERE is_keep=1",
                [],
                |row| row.get::<_, i64>(0),
            )
            .unwrap()
        };
        apply_keep_rules_on(&conn, None, rules()).unwrap();
        assert_eq!(winner(&conn), 2);
        conn.execute("UPDATE afiles SET modified_at=NULL", []).unwrap();
        apply_keep_rules_on(&conn, None, rules()).unwrap();
        assert_eq!(winner(&conn), 2);
    }

    #[test]
    fn raw_first_prefers_raw_pair_over_jpeg_in_similar_groups() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("CREATE TABLE afolders(id INTEGER PRIMARY KEY, path TEXT);
            INSERT INTO afolders VALUES(1,'/photos');
            CREATE TABLE afiles(id INTEGER PRIMARY KEY, folder_id INTEGER, name TEXT, file_type INTEGER, width INTEGER, height INTEGER, created_at INTEGER, media_subtype TEXT, live_photo_video_id INTEGER, format_label TEXT, modified_at INTEGER, size INTEGER);
            INSERT INTO afiles VALUES
                (1,1,'shot.jpg',1,200,200,10,NULL,NULL,'JPG',5,200),
                (2,1,'shot.raw',3,200,200,10,'raw_jpeg_pair',3,NULL,20,300),
                (3,1,'shot.heic',1,200,200,10,NULL,NULL,'HEIC',30,200);
            CREATE TABLE similarity_scans(id INTEGER PRIMARY KEY, scope_key TEXT);
            INSERT INTO similarity_scans VALUES(1,'active');
            CREATE TABLE similarity_groups(id INTEGER PRIMARY KEY, scan_id INTEGER, representative_file_id INTEGER);
            INSERT INTO similarity_groups VALUES(1,1,1);
            CREATE TABLE similarity_group_items(group_id INTEGER, file_id INTEGER, is_keep INTEGER);
            INSERT INTO similarity_group_items VALUES(1,1,1),(1,2,0),(1,3,0);").unwrap();
        apply_keep_rules_on(
            &conn,
            Some("active"),
            KeepRules {
                folder_ids: vec![],
                prefer_format: PreferredFormat::Raw,
                preferred_date: PreferredDate::ModifiedEarliest,
                preferred_size: PreferredSize::Ignore,
            },
        )
        .unwrap();
        let winner: i64 = conn
            .query_row(
                "SELECT file_id FROM similarity_group_items WHERE group_id=1 AND is_keep=1",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(winner, 2);
        let representative: i64 = conn
            .query_row(
                "SELECT representative_file_id FROM similarity_groups WHERE id=1",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(representative, 2);
    }

    #[test]
    fn exact_duplicates_ignore_format_and_size_preferences() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("CREATE TABLE afolders(id INTEGER PRIMARY KEY, path TEXT);
            INSERT INTO afolders VALUES(1,'/photos');
            CREATE TABLE afiles(id INTEGER PRIMARY KEY, folder_id INTEGER, name TEXT, file_type INTEGER, width INTEGER, height INTEGER, created_at INTEGER, media_subtype TEXT, live_photo_video_id INTEGER, format_label TEXT, modified_at INTEGER, size INTEGER);
            INSERT INTO afiles VALUES
                (1,1,'a.jpg',1,100,100,10,NULL,NULL,'JPG',10,100),
                (2,1,'a.raw',3,300,300,10,NULL,NULL,NULL,30,200);
            CREATE TABLE duplicate_groups(id INTEGER PRIMARY KEY);
            INSERT INTO duplicate_groups VALUES(1);
            CREATE TABLE duplicate_group_items(group_id INTEGER, file_id INTEGER, is_keep INTEGER);
            INSERT INTO duplicate_group_items VALUES(1,2,1),(1,1,0);").unwrap();
        apply_keep_rules_on(
            &conn,
            None,
            KeepRules {
                folder_ids: vec![],
                prefer_format: PreferredFormat::Raw,
                preferred_date: PreferredDate::ModifiedEarliest,
                preferred_size: PreferredSize::LargestDimensions,
            },
        )
        .unwrap();
        let winner: i64 = conn
            .query_row(
                "SELECT file_id FROM duplicate_group_items WHERE is_keep=1",
                [],
                |row| row.get(0),
            )
            .unwrap();
        // Format and size are ignored for exact duplicates; the date decides.
        assert_eq!(winner, 1);
    }

    #[test]
    fn preferred_folder_order_outranks_date() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("CREATE TABLE afolders(id INTEGER PRIMARY KEY, path TEXT);
            INSERT INTO afolders VALUES(1,'/first'),(2,'/second');
            CREATE TABLE afiles(id INTEGER PRIMARY KEY, folder_id INTEGER, name TEXT, file_type INTEGER, width INTEGER, height INTEGER, created_at INTEGER, media_subtype TEXT, live_photo_video_id INTEGER, format_label TEXT, modified_at INTEGER, size INTEGER);
            INSERT INTO afiles VALUES
                (1,2,'a.jpg',1,100,100,10,NULL,NULL,'JPG',10,100),
                (2,1,'b.jpg',1,100,100,10,NULL,NULL,'JPG',30,100);
            CREATE TABLE duplicate_groups(id INTEGER PRIMARY KEY);
            INSERT INTO duplicate_groups VALUES(1);
            CREATE TABLE duplicate_group_items(group_id INTEGER, file_id INTEGER, is_keep INTEGER);
            INSERT INTO duplicate_group_items VALUES(1,1,0),(1,2,0);").unwrap();
        let rules = |folder_ids| KeepRules {
            folder_ids,
            prefer_format: PreferredFormat::Ignore,
            preferred_date: PreferredDate::ModifiedEarliest,
            preferred_size: PreferredSize::Ignore,
        };
        let winner = |conn: &Connection| {
            conn.query_row(
                "SELECT file_id FROM duplicate_group_items WHERE is_keep=1",
                [],
                |row| row.get::<_, i64>(0),
            )
            .unwrap()
        };
        apply_keep_rules_on(&conn, None, rules(vec![1, 2])).unwrap();
        assert_eq!(winner(&conn), 2);
        apply_keep_rules_on(&conn, None, rules(vec![2, 1])).unwrap();
        assert_eq!(winner(&conn), 1);
    }

    #[derive(Deserialize)]
    struct SampleSuite {
        samples: std::path::PathBuf,
        cases: Vec<SampleCase>,
    }

    #[derive(Deserialize)]
    struct SampleCase {
        name: String,
        folders: Vec<(i64, String)>,
        files: Vec<SampleFile>,
        rules: KeepRules,
        group_raw_pairs: bool,
        similar: bool,
        expected_keep: Vec<i64>,
        expected_remove: Vec<i64>,
        expected_components: Vec<(i64, i64)>,
    }

    #[derive(Deserialize)]
    struct SampleFile {
        id: i64,
        folder: i64,
        sample: String,
        created: i64,
        modified: i64,
        subtype: Option<String>,
        companion: Option<i64>,
    }

    #[test]
    #[ignore = "run with the external dedup sample suite"]
    fn external_sample_keep_rules() {
        let manifest = std::env::var_os("LAP_DEDUP_SAMPLE_SUITE")
            .expect("set LAP_DEDUP_SAMPLE_SUITE to the fixture manifest");
        let suite: SampleSuite = serde_json::from_slice(&std::fs::read(manifest).unwrap()).unwrap();
        assert!(!suite.cases.is_empty());
        let mut catalog = HashMap::new();

        for case in suite.cases {
            let mut conn = Connection::open_in_memory().unwrap();
            conn.execute_batch(
                "CREATE TABLE afolders(id INTEGER PRIMARY KEY,path TEXT);
                 CREATE TABLE afiles(id INTEGER PRIMARY KEY,folder_id INTEGER,name TEXT,file_type INTEGER,
                     size INTEGER,width INTEGER,height INTEGER,created_at INTEGER,taken_date INTEGER,
                     media_subtype TEXT,live_photo_video_id INTEGER,format_label TEXT,modified_at INTEGER,
                     UNIQUE(folder_id,name));
                 CREATE TABLE file_hashes(file_id INTEGER PRIMARY KEY,hash TEXT,file_size INTEGER,mtime INTEGER);
                 CREATE TABLE duplicate_groups(id INTEGER PRIMARY KEY,hash TEXT,file_size INTEGER,
                     file_count INTEGER,total_size INTEGER,updated_at INTEGER,UNIQUE(hash,file_size));
                 CREATE TABLE duplicate_group_items(group_id INTEGER,file_id INTEGER,is_keep INTEGER,
                     is_selected INTEGER,score REAL,PRIMARY KEY(group_id,file_id));
                 CREATE TABLE similarity_scans(id INTEGER PRIMARY KEY,scope_key TEXT);
                 INSERT INTO similarity_scans VALUES(1,'samples');
                 CREATE TABLE similarity_groups(id INTEGER PRIMARY KEY,scan_id INTEGER,representative_file_id INTEGER);
                 INSERT INTO similarity_groups VALUES(1,1,1);
                 CREATE TABLE similarity_group_items(group_id INTEGER,file_id INTEGER,is_keep INTEGER);",
            ).unwrap();

            let folders: HashMap<_, _> = case.folders.into_iter().collect();
            for (id, path) in &folders {
                conn.execute("INSERT INTO afolders VALUES(?1,?2)", params![id, path])
                    .unwrap();
            }

            let companion_owners: HashMap<_, _> = case
                .files
                .iter()
                .filter_map(|file| file.companion.map(|id| (id, file.id)))
                .collect();
            let mut file_sizes = HashMap::new();
            for file in case.files {
                let owner = companion_owners.get(&file.id).copied().unwrap_or(file.id);
                let name = format!("{owner}-{}", file.sample);
                let (kind, label, size, (width, height), hash) =
                    &*catalog.entry(file.sample.clone()).or_insert_with(|| {
                        let path = suite.samples.join(&file.sample);
                        let path = path.to_str().unwrap();
                        let kind = crate::t_utils::get_file_type(path)
                            .unwrap_or_else(|| panic!("{}: unsupported sample {path}", case.name));
                        let label = crate::t_utils::detect_file_format_label(path, kind);
                        let dimensions = match kind {
                            2 => Ok((0, 0)),
                            3 => crate::t_image::get_raw_dimensions(path),
                            _ => crate::t_image::get_image_dimensions(path),
                        }
                        .unwrap_or_else(|error| {
                            panic!("{}: dimensions for {path}: {error}", case.name)
                        });
                        (
                            kind,
                            label,
                            std::fs::metadata(path).unwrap().len() as i64,
                            dimensions,
                            compute_blake3_hash(path).unwrap(),
                        )
                    });
                conn.execute(
                    "INSERT INTO afiles VALUES(?1,?2,?3,?4,?5,?6,?7,?8,0,?9,?10,?11,?12)",
                    params![
                        file.id,
                        file.folder,
                        name,
                        kind,
                        size,
                        width,
                        height,
                        file.created,
                        file.subtype,
                        file.companion,
                        label,
                        file.modified
                    ],
                )
                .unwrap();
                conn.execute(
                    "INSERT INTO file_hashes VALUES(?1,?2,?3,?4)",
                    params![file.id, hash, size, file.modified],
                )
                .unwrap();
                file_sizes.insert(file.id, *size);
                if case.similar {
                    conn.execute(
                        "INSERT INTO similarity_group_items VALUES(1,?1,?2)",
                        params![file.id, file.id == 1],
                    )
                    .unwrap();
                }
            }

            let items = if case.similar {
                apply_keep_rules_on(&conn, Some("samples"), case.rules).unwrap();
                "similarity_group_items"
            } else {
                let mut stmt = conn
                    .prepare(
                        "SELECT id FROM afiles WHERE id NOT IN (
                    SELECT live_photo_video_id FROM afiles WHERE media_subtype = 'live_photo'
                        AND live_photo_video_id IS NOT NULL
                )",
                    )
                    .unwrap();
                let scope = stmt
                    .query_map([], |row| row.get(0))
                    .unwrap()
                    .collect::<Result<Vec<i64>, _>>()
                    .unwrap();
                drop(stmt);
                rebuild_duplicate_groups(
                    &mut conn,
                    Some(&scope),
                    Some(case.rules),
                    case.group_raw_pairs,
                )
                .unwrap();
                "duplicate_group_items"
            };
            let mut stmt = conn
                .prepare(&format!(
                "SELECT i.file_id, i.is_keep, f.path || '/' || a.name FROM {items} i
                 JOIN afiles a ON a.id=i.file_id JOIN afolders f ON f.id=a.folder_id ORDER BY i.file_id"
            ))
                .unwrap();
            let mut kept = Vec::new();
            let mut removed = Vec::new();
            for row in stmt
                .query_map([], |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, bool>(1)?,
                        row.get::<_, String>(2)?,
                    ))
                })
                .unwrap()
            {
                let (file_id, is_keep, file_path) = row.unwrap();
                if is_keep {
                    kept.push(file_id);
                } else {
                    removed.push(crate::t_cmds::BatchDeleteFile { file_id, file_path });
                }
            }
            assert_eq!(kept, case.expected_keep, "{}: KEEP", case.name);
            assert_eq!(
                removed.iter().map(|file| file.file_id).collect::<Vec<_>>(),
                case.expected_remove,
                "{}: DUP",
                case.name
            );
            if !case.similar && case.group_raw_pairs {
                let protected = raw_pair_protected_delete_ids(&conn).unwrap();
                for file in &removed {
                    assert!(
                        !protected.contains(&file.file_id),
                        "{}: protected DUP",
                        case.name
                    );
                    assert!(
                        raw_pair_has_exact_keep(&conn, file.file_id).unwrap(),
                        "{}: DUP would fail exact-pair deletion check",
                        case.name
                    );
                }
            }
            let (preview, components) = delete_preview(&conn, &removed, case.group_raw_pairs).unwrap();
            let mut component_ids = components
                .iter()
                .flat_map(|(&primary, members)| members.iter().map(move |(id, _)| (primary, *id)))
                .collect::<Vec<_>>();
            component_ids.sort_unstable();
            assert_eq!(
                preview.item_count,
                removed.len(),
                "{}: preview count",
                case.name
            );
            assert_eq!(
                component_ids, case.expected_components,
                "{}: preview companions",
                case.name
            );
            let expected_bytes: i64 = case
                .expected_remove
                .iter()
                .map(|id| file_sizes[id])
                .chain(
                    case.expected_components
                        .iter()
                        .map(|(_, id)| file_sizes[id]),
                )
                .sum();
            assert_eq!(
                preview.bytes, expected_bytes,
                "{}: preview bytes",
                case.name
            );
            if case.similar {
                let representative = conn
                    .query_row(
                        "SELECT representative_file_id FROM similarity_groups",
                        [],
                        |row| row.get::<_, i64>(0),
                    )
                    .unwrap();
                assert_eq!(kept, [representative], "{}: representative", case.name);
            }
            println!("PASS {}", case.name);
        }
    }
}

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
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DedupDeleteRequest {
    pub group_ids: Option<Vec<i64>>,
    pub file_ids: Option<Vec<i64>>,
    pub delete_all: bool,
    pub permanently: bool,
    #[serde(default)]
    pub group_raw_jpeg_pairs: bool,
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
    taken_date: i64,
    created_at: i64,
}

// ----------------------------------------------------------------------------
// Core Logic
// ----------------------------------------------------------------------------

pub fn start_scan(
    app_handle: tauri::AppHandle,
    dedup_state: tauri::State<'_, DedupState>,
    mut query_params: Option<QueryParams>,
    collection_id: Option<i64>,
    file_ids: Option<Vec<i64>>,
    group_raw_jpeg_pairs: bool,
) -> Result<(), String> {
    if let Some(params) = query_params.as_mut() { params.group_raw_jpeg_pairs = group_raw_jpeg_pairs; }
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
            group_raw_jpeg_pairs,
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
    group_raw_jpeg_pairs: bool,
) -> Result<(), String> {
    let mut conn = get_db_conn()?;

    let files_to_check = if let Some(file_ids) = file_ids {
        filter_scope_companions(AFile::get_files_by_ids(&file_ids)?, group_raw_jpeg_pairs)
    } else if let Some(collection_id) = collection_id {
        get_files_by_collection(collection_id, query_params.as_ref())?
    } else if let Some(params) = query_params.as_ref() {
        get_files_by_query(params)?
    } else {
        get_files_by_sizes(&conn, group_raw_jpeg_pairs)?
    };

    // Queries already deduplicate within their result scope. Do not reapply a
    // global exclusion: a companion may be the only matching scope member.
    let files_to_check = filter_suspicious_files(files_to_check);
    let scoped_file_ids = Some(files_to_check.iter().filter_map(|file| file.id).collect::<Vec<i64>>());
    if files_to_check.is_empty() {
        rebuild_duplicate_groups(&mut conn, scoped_file_ids.as_deref())?;
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
                    Err(e) => eprintln!("Failed to hash file {}: {}", path, e),
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
        rebuild_duplicate_groups(&mut conn, scoped_file_ids.as_deref())?;

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

fn filter_scope_companions(files: Vec<AFile>, group_raw_jpeg_pairs: bool) -> Vec<AFile> {
    let hidden: HashSet<i64> = files.iter()
        .filter(|file| file.media_subtype.as_deref() != Some("raw_jpeg_pair") || group_raw_jpeg_pairs)
        .filter_map(|file| file.live_photo_video_id)
        .collect();
    files.into_iter().filter(|file| file.id.map_or(true, |id| !hidden.contains(&id))).collect()
}

#[cfg(test)]
fn hidden_companion_ids(conn: &Connection, group_raw_jpeg_pairs: bool) -> Result<HashSet<i64>, String> {
    let mut stmt = conn.prepare("SELECT live_photo_video_id FROM afiles WHERE live_photo_video_id IS NOT NULL AND (?1 OR COALESCE(media_subtype, '') != 'raw_jpeg_pair')")
        .map_err(|e| e.to_string())?;
    let ids = stmt.query_map(params![group_raw_jpeg_pairs], |row| row.get(0)).map_err(|e| e.to_string())?
        .collect::<Result<HashSet<i64>, _>>().map_err(|e| e.to_string())?;
    Ok(ids)
}

fn get_files_by_sizes(conn: &Connection, group_raw_jpeg_pairs: bool) -> Result<Vec<AFile>, String> {
    let mut stmt = conn.prepare(
        "SELECT a.id, a.folder_id, a.name, a.name_pinyin, a.size, a.file_type, a.format_label,
                a.created_at, a.modified_at, a.inode, a.taken_date, a.width, a.height, a.duration,
                a.is_favorite, a.rating, a.rotate, a.comments, a.has_tags, a.has_faces, 
                a.e_make, a.e_model, a.e_date_time, a.e_software, a.e_artist, a.e_copyright, 
                a.e_description, a.e_lens_make, a.e_lens_model, a.e_exposure_bias, a.e_exposure_time, 
                a.e_f_number, a.e_focal_length, a.e_iso_speed, a.e_flash, a.e_orientation, 
                a.icc_profile, a.color_space, a.bit_depth,
                a.gps_latitude, a.gps_longitude, a.gps_altitude, 
                a.geo_name, a.geo_admin1, a.geo_admin2, a.geo_cc,
                f.path || '/' || a.name as file_path
         FROM afiles a
         JOIN afolders f ON a.folder_id = f.id
         WHERE a.id NOT IN (
             SELECT live_photo_video_id FROM afiles WHERE live_photo_video_id IS NOT NULL
                 AND (?1 OR COALESCE(media_subtype, '') != 'raw_jpeg_pair')
         )
         AND a.size IN (
             SELECT size FROM afiles
             WHERE id NOT IN (
                 SELECT live_photo_video_id FROM afiles WHERE live_photo_video_id IS NOT NULL
                 AND (?1 OR COALESCE(media_subtype, '') != 'raw_jpeg_pair')
             )
             GROUP BY size
             HAVING COUNT(size) > 1 AND size > 0
         )
         ORDER BY a.size DESC"
    ).map_err(|e| e.to_string())?;

    let iter = stmt
        .query_map(params![group_raw_jpeg_pairs], |row| {
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
                is_favorite: row.get(14)?,
                rating: row.get(15)?,
                culling_flag: None,
                rotate: row.get(16)?,
                comments: row.get(17)?,
                has_tags: row.get(18)?,
                has_faces: row.get(19)?,
                e_make: row.get(20)?,
                e_model: row.get(21)?,
                e_date_time: row.get(22)?,
                e_software: row.get(23)?,
                e_artist: row.get(24)?,
                e_copyright: row.get(25)?,
                e_description: row.get(26)?,
                e_lens_make: row.get(27)?,
                e_lens_model: row.get(28)?,
                e_exposure_bias: row.get(29)?,
                e_exposure_time: row.get(30)?,
                e_f_number: row.get(31)?,
                e_focal_length: row.get(32)?,
                e_iso_speed: row.get(33)?,
                e_flash: row.get(34)?,
                e_orientation: row.get(35)?,
                icc_profile: row.get(36)?,
                color_space: row.get(37)?,
                bit_depth: row.get(38)?,
                gps_latitude: row.get(39)?,
                gps_longitude: row.get(40)?,
                gps_altitude: row.get(41)?,
                geo_name: row.get(42)?,
                geo_admin1: row.get(43)?,
                geo_admin2: row.get(44)?,
                geo_cc: row.get(45)?,
                file_path: row.get(46)?,
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
                motion_photo_offset: None,
                album_accessible: true,
                album_visible: true,
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

        // Let's get the files for this group
        let item_query = if scope_file_ids.is_some() {
            "SELECT a.id, a.taken_date, a.created_at
             FROM file_hashes fh
             JOIN afiles a ON fh.file_id = a.id
             JOIN temp_scope_ids ts ON ts.file_id = a.id
             WHERE fh.hash = ?1 AND fh.file_size = ?2"
        } else {
            "SELECT a.id, a.taken_date, a.created_at
             FROM file_hashes fh
             JOIN afiles a ON fh.file_id = a.id
             WHERE fh.hash = ?1 AND fh.file_size = ?2"
        };

        let mut f_stmt = tx.prepare(item_query).map_err(|e| e.to_string())?;

        let mut keep_candidates: Vec<KeepCandidate> = Vec::new();
        let iter = f_stmt
            .query_map(params![hash, size], |row| {
                let id: i64 = row.get(0)?;
                let tk: i64 = row.get(1).unwrap_or(0);
                let ca: i64 = row.get(2).unwrap_or(0);
                Ok((id, tk, ca))
            })
            .map_err(|e| e.to_string())?;

        for r in iter {
            let (id, tk, ca) = r.map_err(|e| e.to_string())?;
            keep_candidates.push(KeepCandidate {
                id,
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
    drop(stmt);

    tx.commit().map_err(|e| e.to_string())?;
    Ok(())
}

fn compare_best_quality(a: &KeepCandidate, b: &KeepCandidate) -> std::cmp::Ordering {
    (b.taken_date > 0)
        .cmp(&(a.taken_date > 0))
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
    pub file: Option<AFile>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DedupOverview {
    pub total_groups: i64,
    pub total_files: i64,
    pub total_reclaimable_bytes: i64,
}

pub fn get_overview() -> Result<DedupOverview, String> {
    let conn = get_db_conn()?;
    let (total_groups, total_files, total_reclaimable_bytes) = conn
        .query_row(
            "SELECT 
                COALESCE(COUNT(*), 0),
                COALESCE(SUM(current_file_count - 1), 0),
                COALESCE(SUM((current_file_count - 1) * file_size), 0)
             FROM (
                SELECT file_size,
                       (SELECT COUNT(*)
                        FROM duplicate_group_items
                        WHERE group_id = duplicate_groups.id) AS current_file_count
                FROM duplicate_groups
             )
             WHERE current_file_count > 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .map_err(|e| e.to_string())?;

    Ok(DedupOverview {
        total_groups,
        total_files,
        total_reclaimable_bytes,
    })
}

pub fn list_groups(
    page: u32,
    page_size: u32,
    sort_by: &str,
    filter: &str,
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

    let mut groups = Vec::new();
    for g in groups_iter {
        if let Ok(mut group) = g {
            // Fetch items
            group.items = get_group_items(&conn, group.id)?;
            groups.push(group);
        }
    }

    Ok(groups)
}

fn get_group_items(conn: &Connection, group_id: i64) -> Result<Vec<DedupGroupItem>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT group_id, file_id, is_keep, is_selected, score
         FROM duplicate_group_items
         WHERE group_id = ?1
         ORDER BY is_keep DESC, score DESC",
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
                file: None, // Will populate shortly
            })
        })
        .map_err(|e| e.to_string())?;

    let mut items = Vec::new();
    for it in iter {
        if let Ok(mut item) = it {
            if let Ok(Some(file_info)) = AFile::get_file_info(item.file_id) {
                item.file = Some(file_info);
            }
            items.push(item);
        }
    }

    Ok(items)
}

pub fn set_keep(group_id: i64, file_id: i64) -> Result<(), String> {
    let mut conn = get_db_conn()?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;

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

pub fn delete(app_handle: &tauri::AppHandle, request: DedupDeleteRequest) -> Result<DedupDeleteResult, String> {
    let _library_guard = crate::t_cmds::FILE_REFRESH_LIBRARY_LOCK.read().map_err(|e| e.to_string())?;
    let mut conn = get_db_conn()?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;

    let mut files_to_delete: Vec<crate::t_cmds::BatchDeleteFile> = Vec::new();

    if request.delete_all {
        // Guard against a corrupted group where every item has is_keep = 0:
        // only delete non-keep items from groups that still have a keep item.
        let mut stmt = tx
            .prepare(
                "SELECT a.id, f.path || '/' || a.name
                 FROM duplicate_group_items dgi
                 JOIN afiles a ON dgi.file_id = a.id
                 JOIN afolders f ON a.folder_id = f.id
                 WHERE dgi.is_keep = 0
                   AND EXISTS (SELECT 1 FROM duplicate_group_items k
                               WHERE k.group_id = dgi.group_id AND k.is_keep = 1)",
            )
            .map_err(|e| e.to_string())?;
        let mut iter = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
            .map_err(|e| e.to_string())?;
        for row in &mut iter {
            let (file_id, file_path) = row.map_err(|e| e.to_string())?;
            files_to_delete.push(crate::t_cmds::BatchDeleteFile { file_id, file_path });
        }
    } else if let Some(ids) = request.file_ids {
        let mut stmt = tx
            .prepare(
                "SELECT a.id, f.path || '/' || a.name
                 FROM duplicate_group_items dgi
                 JOIN afiles a ON dgi.file_id = a.id
                 JOIN afolders f ON a.folder_id = f.id
                 WHERE dgi.file_id = ?1 AND dgi.is_keep = 0",
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
    let delete_result = crate::t_cmds::delete_files_grouped_locked(app_handle, files_to_delete, request.permanently, request.group_raw_jpeg_pairs)?;
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
        failed_count: delete_result.failed_count + failures.len(),
        errors: failures,
        trash_failed_file_ids: delete_result.trash_failed_file_ids,
    })
}

#[cfg(test)]
mod scan_pair_tests {
    use super::*;

    #[test]
    fn scan_snapshot_excludes_live_always_and_raw_only_when_enabled() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("CREATE TABLE afiles (id INTEGER, media_subtype TEXT, live_photo_video_id INTEGER);
            INSERT INTO afiles VALUES (1,'raw_jpeg_pair',2),(2,NULL,NULL),(3,'live_photo',4),(4,NULL,NULL);").unwrap();
        assert_eq!(hidden_companion_ids(&conn, false).unwrap(), HashSet::from([4]));
        assert_eq!(hidden_companion_ids(&conn, true).unwrap(), HashSet::from([2, 4]));
    }

    #[test]
    fn explicit_scope_only_hides_companion_if_primary_is_selected() {
        let raw = || serde_json::from_value::<AFile>(serde_json::json!({"id": 1, "name": "a.CR3", "folder_id": 1, "size": 10, "media_subtype": "raw_jpeg_pair", "live_photo_video_id": 2})).unwrap();
        let jpeg = || serde_json::from_value::<AFile>(serde_json::json!({"id": 2, "name": "a.JPG", "folder_id": 1, "size": 10})).unwrap();
        assert_eq!(filter_scope_companions(vec![jpeg()], true).len(), 1);
        assert_eq!(filter_scope_companions(vec![raw(), jpeg()], true).len(), 1);
        assert_eq!(filter_scope_companions(vec![raw(), jpeg()], false).len(), 2);
    }

    #[test]
    fn rebuild_excludes_cached_hashes_outside_scan_snapshot() {
        let mut conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("CREATE TABLE afiles (id INTEGER PRIMARY KEY, taken_date INTEGER, created_at INTEGER);
            CREATE TABLE file_hashes (file_id INTEGER, hash TEXT, file_size INTEGER);
            CREATE TABLE duplicate_groups (id INTEGER PRIMARY KEY, hash TEXT, file_size INTEGER, file_count INTEGER, total_size INTEGER, updated_at INTEGER);
            CREATE TABLE duplicate_group_items (group_id INTEGER, file_id INTEGER, is_keep INTEGER, is_selected INTEGER, score REAL);
            INSERT INTO afiles VALUES (1,0,1),(2,0,2),(3,0,3);
            INSERT INTO file_hashes VALUES (1,'same',100),(2,'same',100),(3,'same',100);").unwrap();
        rebuild_duplicate_groups(&mut conn, Some(&[1, 2])).unwrap();
        let ids: Vec<i64> = conn.prepare("SELECT file_id FROM duplicate_group_items ORDER BY file_id").unwrap()
            .query_map([], |row| row.get(0)).unwrap().collect::<Result<_, _>>().unwrap();
        assert_eq!(ids, vec![1, 2]);
        rebuild_duplicate_groups(&mut conn, Some(&[1])).unwrap();
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM duplicate_groups", [], |row| row.get(0)).unwrap();
        assert_eq!(count, 0);
    }
}

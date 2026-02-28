use crate::t_sqlite::{AFile, QueryParams};
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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
pub struct DedupScanStatus {
    pub state: String, // "running", "idle", "finished", "error"
    pub processed: u64,
    pub total: u64,
    pub groups: u64,
}

impl Default for DedupScanStatus {
    fn default() -> Self {
        Self {
            state: "idle".to_string(),
            processed: 0,
            total: 0,
            groups: 0,
        }
    }
}

#[derive(Default)]
pub struct DedupState {
    pub is_scanning: Arc<AtomicBool>,
    pub cancel_flag: Arc<AtomicBool>,
    pub status: Arc<Mutex<DedupScanStatus>>,
}

// ----------------------------------------------------------------------------
// Core Logic
// ----------------------------------------------------------------------------

pub fn start_scan(
    app_handle: tauri::AppHandle,
    dedup_state: tauri::State<'_, DedupState>,
    query_params: Option<QueryParams>,
    file_ids: Option<Vec<i64>>,
) -> Result<(), String> {
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
    }

    std::thread::spawn(move || {
        let result = scan_and_hash_files(
            &app_handle,
            &status_clone,
            &cancel_flag_clone,
            query_params,
            file_ids,
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
        let _ = app_handle.emit("dedup-scan-progress", final_status.clone());
    });

    Ok(())
}

fn scan_and_hash_files(
    app_handle: &tauri::AppHandle,
    status_mutex: &Arc<Mutex<DedupScanStatus>>,
    cancel_flag: &Arc<AtomicBool>,
    query_params: Option<QueryParams>,
    file_ids: Option<Vec<i64>>,
) -> Result<(), String> {
    let mut conn = get_db_conn()?;
    let has_scope = query_params.is_some() || file_ids.is_some();

    let files_to_check = if let Some(ids) = file_ids.as_ref() {
        get_files_by_ids(ids)?
    } else if let Some(params) = query_params.as_ref() {
        get_files_by_query(params)?
    } else {
        // Step 1: Find suspicious sizes (sizes shared by >1 file)
        let suspicious_sizes = get_suspicious_file_sizes(&conn)?;
        if suspicious_sizes.is_empty() {
            rebuild_duplicate_groups(&mut conn, None)?;
            return Ok(());
        }
        // Step 2: Get all files with those sizes
        get_files_by_sizes(&conn, &suspicious_sizes)?
    };

    let files_to_check = filter_suspicious_files(files_to_check);
    let scoped_file_ids = if has_scope {
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

fn get_db_conn() -> Result<Connection, String> {
    let path = crate::t_config::get_current_db_path()
        .map_err(|e| format!("Failed to get db path: {}", e))?;
    let conn = Connection::open(&path).map_err(|e| format!("Failed to open db: {}", e))?;
    conn.execute("PRAGMA foreign_keys = ON", [])
        .map_err(|e| format!("Failed to enable foreign keys: {}", e))?;
    Ok(conn)
}

fn get_suspicious_file_sizes(conn: &Connection) -> Result<Vec<i64>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT size 
         FROM afiles 
         GROUP BY size 
         HAVING COUNT(size) > 1 AND size > 0",
        )
        .map_err(|e| e.to_string())?;

    let sizes = stmt
        .query_map([], |row| row.get(0))
        .map_err(|e| e.to_string())?
        .filter_map(Result::ok)
        .collect();

    Ok(sizes)
}

fn get_files_by_sizes(conn: &Connection, _sizes: &[i64]) -> Result<Vec<AFile>, String> {
    // Basic trick: parameterize the sizes. SQLite limits to 999 max vars usually.
    // For safety, let's fetch in chunks if huge. But we'll just do a joined query.

    let mut stmt = conn.prepare(
        "SELECT a.id, a.folder_id, a.name, a.name_pinyin, a.size, a.file_type, 
                a.created_at, a.modified_at, a.taken_date, a.width, a.height, a.duration, 
                a.is_favorite, a.rating, a.rotate, a.comments, a.has_tags, a.has_faces, 
                a.e_make, a.e_model, a.e_date_time, a.e_software, a.e_artist, a.e_copyright, 
                a.e_description, a.e_lens_make, a.e_lens_model, a.e_exposure_bias, a.e_exposure_time, 
                a.e_f_number, a.e_focal_length, a.e_iso_speed, a.e_flash, a.e_orientation, 
                a.gps_latitude, a.gps_longitude, a.gps_altitude, 
                a.geo_name, a.geo_admin1, a.geo_admin2, a.geo_cc,
                f.path || '/' || a.name as file_path
         FROM afiles a
         JOIN afolders f ON a.folder_id = f.id
         WHERE a.size IN (SELECT size FROM afiles GROUP BY size HAVING COUNT(size) > 1 AND size > 0)
         ORDER BY a.size DESC"
    ).map_err(|e| e.to_string())?;

    let iter = stmt
        .query_map([], |row| {
            Ok(AFile {
                id: row.get(0)?,
                folder_id: row.get(1)?,
                name: row.get(2)?,
                name_pinyin: row.get(3)?,
                size: row.get(4)?,
                file_type: row.get(5)?,
                created_at: row.get(6)?,
                modified_at: row.get(7)?,
                taken_date: row.get(8)?,
                width: row.get(9)?,
                height: row.get(10)?,
                duration: row.get(11)?,
                is_favorite: row.get(12)?,
                rating: row.get(13)?,
                rotate: row.get(14)?,
                comments: row.get(15)?,
                has_tags: row.get(16)?,
                has_faces: row.get(17)?,
                e_make: row.get(18)?,
                e_model: row.get(19)?,
                e_date_time: row.get(20)?,
                e_software: row.get(21)?,
                e_artist: row.get(22)?,
                e_copyright: row.get(23)?,
                e_description: row.get(24)?,
                e_lens_make: row.get(25)?,
                e_lens_model: row.get(26)?,
                e_exposure_bias: row.get(27)?,
                e_exposure_time: row.get(28)?,
                e_f_number: row.get(29)?,
                e_focal_length: row.get(30)?,
                e_iso_speed: row.get(31)?,
                e_flash: row.get(32)?,
                e_orientation: row.get(33)?,
                gps_latitude: row.get(34)?,
                gps_longitude: row.get(35)?,
                gps_altitude: row.get(36)?,
                geo_name: row.get(37)?,
                geo_admin1: row.get(38)?,
                geo_admin2: row.get(39)?,
                geo_cc: row.get(40)?,
                file_path: row.get(41)?,
                album_id: None,
                album_name: None,
                has_thumbnail: None,
                has_embedding: None,
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

fn get_files_by_query(params: &QueryParams) -> Result<Vec<AFile>, String> {
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

fn get_files_by_ids(file_ids: &[i64]) -> Result<Vec<AFile>, String> {
    let mut files = Vec::new();
    for file_id in file_ids {
        if *file_id <= 0 {
            continue;
        }
        if let Some(file) = AFile::get_file_info(*file_id)? {
            files.push(file);
        }
    }
    Ok(files)
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

fn rebuild_duplicate_groups(conn: &mut Connection, scope_file_ids: Option<&[i64]>) -> Result<(), String> {
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

    let mut stmt = tx
        .prepare(group_query)
        .map_err(|e| e.to_string())?;

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
            "SELECT a.id, a.width, a.height, a.taken_date, a.modified_at
             FROM file_hashes fh
             JOIN afiles a ON fh.file_id = a.id
             JOIN temp_scope_ids ts ON ts.file_id = a.id
             WHERE fh.hash = ?1 AND fh.file_size = ?2"
        } else {
            "SELECT a.id, a.width, a.height, a.taken_date, a.modified_at
             FROM file_hashes fh
             JOIN afiles a ON fh.file_id = a.id
             WHERE fh.hash = ?1 AND fh.file_size = ?2"
        };

        let mut f_stmt = tx.prepare(item_query).map_err(|e| e.to_string())?;

        // Score each file. Highest score becomes keep file.
        // Higher resolution + newer modified + taken date = higher score
        let mut file_scores = Vec::new();
        let iter = f_stmt
            .query_map(params![hash, size], |row| {
                let id: i64 = row.get(0)?;
                let w: u32 = row.get(1).unwrap_or(0);
                let h: u32 = row.get(2).unwrap_or(0);
                let tk: i64 = row.get(3).unwrap_or(0);
                let mt: i64 = row.get(4).unwrap_or(0);
                Ok((id, w, h, tk, mt))
            })
            .map_err(|e| e.to_string())?;

        for r in iter {
            let (id, w, h, tk, mt) = r.map_err(|e| e.to_string())?;
            let resolution_score = (w as f64) * (h as f64) * 0.001;
            let tk_score = if tk > 0 { 100.0 } else { 0.0 };
            let mt_score = (mt as f64) * 0.000_000_001; // tiny factor to break ties

            let total_score = resolution_score + tk_score + mt_score;
            file_scores.push((id, total_score));
        }

        // highest score first
        file_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        for (i, (file_id, score)) in file_scores.iter().enumerate() {
            let is_keep = if i == 0 { 1 } else { 0 };

            tx.execute(
                "INSERT INTO duplicate_group_items (group_id, file_id, is_keep, is_selected, score)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![target_group_id, file_id, is_keep, 0, score],
            )
            .map_err(|e| e.to_string())?;
        }
    }
    drop(stmt);

    tx.commit().map_err(|e| e.to_string())?;
    Ok(())
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
         LIMIT ?1 OFFSET ?2",
        filter_clause, order_clause
    );

    let mut stmt = conn.prepare(&query).map_err(|e| e.to_string())?;
    let groups_iter = stmt
        .query_map(params![page_size, offset], |row| {
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

pub fn get_group(group_id: i64) -> Result<DedupGroup, String> {
    let conn = get_db_conn()?;
    let mut stmt = conn
        .prepare(
            "SELECT id, hash, file_size, file_count, total_size, reviewed, updated_at
         FROM duplicate_groups WHERE id = ?1",
        )
        .map_err(|e| e.to_string())?;

    let mut group = stmt
        .query_row(params![group_id], |row| {
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

    group.items = get_group_items(&conn, group_id)?;
    Ok(group)
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

pub fn delete_selected(
    group_ids: Option<Vec<i64>>,
    file_ids: Option<Vec<i64>>,
) -> Result<(), String> {
    let mut conn = get_db_conn()?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;

    let mut files_to_delete: Vec<(i64, String)> = Vec::new();

    if let Some(ids) = file_ids {
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
                files_to_delete.push(row.map_err(|e| e.to_string())?);
            }
        }
    } else if let Some(gids) = group_ids {
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
                files_to_delete.push(row.map_err(|e| e.to_string())?);
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
            files_to_delete.push(row.map_err(|e| e.to_string())?);
        }
    }

    tx.commit().map_err(|e| e.to_string())?;

    let mut failures: Vec<String> = Vec::new();
    for (file_id, file_path) in files_to_delete {
        if let Err(e) = trash::delete(&file_path) {
            failures.push(format!("Failed to move to trash: {} ({})", file_path, e));
            continue;
        }
        match AFile::delete(file_id) {
            Ok(0) => failures.push(format!("File not removed from DB: id={}", file_id)),
            Ok(_) => {}
            Err(e) => failures.push(format!("Failed to delete DB row for id={}: {}", file_id, e)),
        }
    }

    // Clean up empty groups
    let conn = get_db_conn()?;
    conn.execute(
        "DELETE FROM duplicate_groups 
         WHERE id NOT IN (SELECT DISTINCT group_id FROM duplicate_group_items)",
        [],
    )
    .map_err(|e| e.to_string())?;

    if !failures.is_empty() {
        return Err(failures.join("; "));
    }

    Ok(())
}

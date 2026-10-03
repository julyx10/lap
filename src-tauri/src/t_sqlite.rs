/**
 * SQLite database operations.
 * project: Lap
 * author:  julyx10
 * date:    2024-08-08
 */
use crate::t_ai;
use crate::t_ai_png;
use crate::t_config;
use crate::t_common;
use crate::t_image;
use crate::t_lens;
use crate::t_libraw;
use crate::t_storage;
use crate::t_utils;
use crate::t_video;
use crate::t_raw_display::RawDisplayOptions;
use base64::{Engine, engine::general_purpose};
use chrono::{Datelike, TimeZone};
use exif::{In, Tag, Value};
use image::{GenericImageView, ImageFormat};
use rusqlite::{Connection, OptionalExtension, Result, ToSql, params, params_from_iter};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::Cursor;
use std::ops::{Deref, DerefMut};
use std::panic::{self, AssertUnwindSafe};
use std::path::{Path, PathBuf};
use std::process;
use std::sync::{Arc, Condvar, Mutex, OnceLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tauri::{Emitter, State};
use tokio::sync::Semaphore;
use walkdir::WalkDir;

static THUMB_GENERATION_LOCKS: OnceLock<ThumbGenerationLocks> = OnceLock::new();
static THUMB_BACKGROUND_TASKS: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();
static THUMB_BACKGROUND_GENERATION_PERMITS: OnceLock<Arc<Semaphore>> = OnceLock::new();
const MAX_BACKGROUND_THUMB_GENERATIONS: usize = 4;

fn subtree_like_pattern(path: &str) -> String {
    let separator = std::path::MAIN_SEPARATOR;
    let prefix = path.trim_end_matches(separator);
    let prefix = if prefix.is_empty() {
        separator.to_string()
    } else {
        format!("{}{}", prefix, separator)
    };
    format!("{}%", prefix.replace('\\', "\\\\").replace('%', "\\%").replace('_', "\\_"))
}

fn populate_selected_file_ids(
    tx: &rusqlite::Transaction<'_>,
    file_ids: &[i64],
) -> Result<(), String> {
    for chunk in file_ids.chunks(500) {
        let placeholders = std::iter::repeat_n("(?)", chunk.len()).collect::<Vec<_>>().join(",");
        tx.execute(
            &format!("INSERT OR IGNORE INTO selected_file_ids (id) VALUES {placeholders}"),
            params_from_iter(chunk.iter()),
        )
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}

struct ThumbGenerationLocks {
    active: Mutex<HashSet<String>>,
    available: Condvar,
}

fn thumb_generation_locks() -> &'static ThumbGenerationLocks {
    THUMB_GENERATION_LOCKS.get_or_init(|| ThumbGenerationLocks {
        active: Mutex::new(HashSet::new()),
        available: Condvar::new(),
    })
}

fn thumb_background_tasks() -> &'static Mutex<HashSet<String>> {
    THUMB_BACKGROUND_TASKS.get_or_init(|| Mutex::new(HashSet::new()))
}

pub(crate) fn thumb_background_generation_permits() -> Arc<Semaphore> {
    THUMB_BACKGROUND_GENERATION_PERMITS
        .get_or_init(|| Arc::new(Semaphore::new(MAX_BACKGROUND_THUMB_GENERATIONS)))
        .clone()
}

pub fn has_active_thumb_background_tasks() -> bool {
    thumb_background_tasks()
        .lock()
        .map(|tasks| !tasks.is_empty())
        .unwrap_or(false)
}

struct ThumbGenerationGuard {
    key: String,
}

impl Drop for ThumbGenerationGuard {
    fn drop(&mut self) {
        let locks = thumb_generation_locks();
        let mut active = locks.active.lock().unwrap_or_else(|e| e.into_inner());
        active.remove(&self.key);
        locks.available.notify_all();
    }
}

/// Face Bounding Box struct (matching JSON storage)
#[derive(Debug, Deserialize)]
struct FaceBBox {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

/// Define the Album struct
#[derive(Debug, Serialize, Deserialize)]
pub struct Album {
    pub id: Option<i64>, // unique id (autoincrement by db)

    // album basic info
    pub name: String,             // album name (default is folder name)
    pub path: String,             // folder path
    pub created_at: Option<i64>,  // folder create time
    pub modified_at: Option<i64>, // folder modified time

    // extra info
    pub display_order_id: Option<i64>, // display order id
    pub cover_file_id: Option<i64>,    // album cover file id
    pub description: Option<String>,   // album description
    pub indexed: Option<u64>,          // indexed files count
    pub total: Option<u64>,            // total files count
    pub skipped_count: Option<u64>, // unsupported files from the last complete scan
    pub skipped_size: Option<u64>,  // total size of unsupported files
    pub failed_count: Option<u64>,  // unreadable files from the last complete scan
    pub failed_size: Option<u64>,   // total size of unreadable files
    pub merged_count: Option<u64>,  // companions merged into logical items
    pub merged_size: Option<u64>,   // total size of merged companions
    pub last_scan_time: Option<i64>,   // last scan time
    #[serde(default = "default_album_accessible")]
    pub is_accessible: bool,
    pub file_types: i64, // 1=photos, 2=videos, 4=RAW
    pub small_image_filter: i64,
    pub excluded_folders: Vec<String>,
}

fn default_album_visible() -> bool {
    true
}

fn default_album_accessible() -> bool {
    true
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlbumDisplayOrder {
    pub id: i64,
    pub display_order: i64,
}

impl Album {
    pub fn allows_file_type(&self, file_type: i64) -> bool {
        let bit = match file_type { 1 => 1, 2 => 2, 3 => 4, _ => return true };
        self.file_types & bit != 0
    }

    pub fn excludes_path(&self, path: &Path) -> bool {
        Self::path_is_excluded(Path::new(&self.path), &self.excluded_folders, path)
    }

    pub(crate) fn path_is_excluded(root: &Path, excluded_folders: &[String], path: &Path) -> bool {
        let Ok(relative) = path.strip_prefix(root) else { return false; };
        relative.components().next().is_some_and(|component| {
            excluded_folders.iter().any(|folder| component.as_os_str() == folder.as_str())
        })
    }

    pub fn edit(id: i64, name: &str, description: &str, file_types: i64,
        small_image_filter: i64, excluded_folders: &[String]) -> Result<usize, String> {
        if !(1..=7).contains(&file_types) || !matches!(small_image_filter, 0 | 160 | 320 | 640) {
            return Err("Invalid album filters".into());
        }
        // Only one normal path component is allowed; never accept a relative traversal or glob.
        if excluded_folders.iter().any(|name| name.is_empty() || name == "." || name == ".."
            || name.contains('/') || name.contains('\\') || name.contains('\0')
            || !matches!(Path::new(name).components().next(), Some(std::path::Component::Normal(_)))) {
            return Err("Excluded folders must be first-level subfolder names".into());
        }
        let mut conn = open_conn()?;
        Self::edit_on(&mut conn, id, name, description, file_types, small_image_filter, excluded_folders)
    }

    fn edit_on(conn: &mut Connection, id: i64, name: &str, description: &str, file_types: i64,
        small_image_filter: i64, excluded_folders: &[String]) -> Result<usize, String> {
        let excluded = serde_json::to_string(excluded_folders).map_err(|e| e.to_string())?;
        let tx = conn.transaction().map_err(|e| e.to_string())?;
        let changed = tx.execute("UPDATE albums SET name=?1, description=?2,
            indexed=CASE WHEN file_types != ?3 OR excluded_folders != ?5 OR COALESCE(small_image_filter, 0) != ?4 THEN 0 ELSE indexed END,
            file_types=?3, small_image_filter=?4, excluded_folders=?5 WHERE id=?6",
            params![name, description, file_types, small_image_filter, excluded, id]).map_err(|e| e.to_string())?;
        // Pairing is derived state. A retained excluded companion must not still
        // be opened through an included RAW or Live Photo primary.
        tx.execute("UPDATE afiles SET media_subtype=NULL, live_photo_video_id=NULL
            WHERE folder_id IN (SELECT id FROM afolders WHERE album_id=?1)
            AND ((media_subtype='raw_jpeg_pair' AND (?2 & 5) != 5)
                OR (media_subtype='live_photo' AND (?2 & 3) != 3))", params![id, file_types])
            .map_err(|e| e.to_string())?;
        tx.execute(&format!("UPDATE afiles AS a SET media_subtype=NULL, live_photo_video_id=NULL
            WHERE folder_id IN (SELECT id FROM afolders WHERE album_id=?1)
            AND live_photo_video_id IS NOT NULL
            AND (NOT ({}) OR EXISTS (SELECT 1 FROM afiles companion
                WHERE companion.id=a.live_photo_video_id AND NOT ({})))",
            AFile::album_scan_predicate("a"), AFile::album_scan_predicate("companion")), params![id])
            .map_err(|e| e.to_string())?;
        tx.commit().map_err(|e| e.to_string())?;
        Ok(changed)
    }

    /// create a new album
    fn new(path: &str) -> Result<Self, String> {
        let file_info = t_utils::FileInfo::new(path)?;
        Ok(Self {
            id: None,
            name: file_info.file_name,
            path: file_info.file_path,
            created_at: file_info.created,
            modified_at: file_info.modified,
            display_order_id: None,
            cover_file_id: None,
            description: Some(String::new()),
            indexed: Some(0),
            total: Some(0),
            skipped_count: Some(0),
            skipped_size: Some(0),
            failed_count: Some(0),
            failed_size: Some(0),
            merged_count: Some(0),
            merged_size: Some(0),
            last_scan_time: Some(0),
            is_accessible: true,
            file_types: 7,
            small_image_filter: 0,
            excluded_folders: Vec::new(),
        })
    }

    /// Function to construct `Self` from a database row
    fn from_row(row: &rusqlite::Row) -> Result<Self, rusqlite::Error> {
        Ok(Self {
            id: Some(row.get(0)?),
            name: row.get(1)?,
            path: row.get(2)?,
            created_at: row.get(3)?,
            modified_at: row.get(4)?,
            display_order_id: row.get(5)?,
            cover_file_id: row.get(6)?,
            description: row.get(7)?,
            indexed: row.get(8)?,
            total: row.get(9)?,
            skipped_count: row.get(10)?,
            skipped_size: row.get(11)?,
            failed_count: row.get(12)?,
            failed_size: row.get(13)?,
            merged_count: row.get(14)?,
            merged_size: row.get(15)?,
            last_scan_time: row.get(16)?,
            is_accessible: true,
            file_types: row.get(17)?,
            small_image_filter: row.get::<_, Option<i64>>(18)?.unwrap_or(0),
            excluded_folders: serde_json::from_str(&row.get::<_, String>(19)?).unwrap_or_default(),
        })
    }

    /// fetch an album from db by path
    fn fetch(path: &str) -> Result<Option<Self>, String> {
        let conn = open_conn()?;
        let result = conn.query_row(
            "SELECT id, name, path, created_at, modified_at, display_order_id, cover_file_id, description, indexed, total, skipped_count, skipped_size, failed_count, failed_size, merged_count, merged_size, last_scan_time, file_types, small_image_filter, excluded_folders
            FROM albums WHERE path = ?1",
            params![path],
            Self::from_row
        ).optional().map_err(|e| e.to_string())?;
        Ok(result)
    }

    /// insert an album into db
    fn insert(&mut self) -> Result<usize, String> {
        let conn = open_conn()?;

        // Determine the next display order id
        self.display_order_id = conn
            .query_row(
                "SELECT COALESCE(MAX(display_order_id), 0) + 1 FROM albums",
                params![],
                |row| row.get(0),
            )
            .map_err(|e| e.to_string())?;

        // Insert the new album into the db
        let result = conn.execute(
            "INSERT INTO albums (name, path, created_at, modified_at, display_order_id, cover_file_id, description, indexed, total, skipped_count, skipped_size, failed_count, failed_size, merged_count, merged_size, last_scan_time, small_image_filter)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, 0)",
            params![
                self.name,
                self.path,
                self.created_at,
                self.modified_at,
                self.display_order_id,
                self.cover_file_id,
                self.description,
                self.indexed,
                self.total,
                self.skipped_count,
                self.skipped_size,
                self.failed_count,
                self.failed_size,
                self.merged_count,
                self.merged_size,
                self.last_scan_time,
            ],
        ).map_err(|e| e.to_string())?;
        Ok(result)
    }

    /// add the album into db if not exists
    pub fn add_album_to_db(path: &str) -> Result<Self, String> {
        // Check if the path already exists
        let existing_album = Self::fetch(path);
        if let Ok(Some(album)) = existing_album {
            return Err(format!(
                "Album '{}' with the path '{}' already exists.",
                album.name, album.path
            ));
        }

        // Reject an album that overlaps an existing album (nested inside it or
        // containing it). A folder can only belong to one album tree;
        // overlapping albums break folder sync/refresh.
        let new_path = Path::new(path);
        for album in Self::get_all_albums()? {
            let existing_path = Path::new(&album.path);
            if new_path != existing_path
                && (new_path.starts_with(existing_path) || existing_path.starts_with(new_path))
            {
                return Err(format!(
                    "Cannot add '{}': it overlaps the existing album '{}' ({}).",
                    path, album.name, album.path
                ));
            }
        }

        // Insert the new album into the database
        Self::new(path)?.insert()?;

        // return the newly inserted album
        let new_album = Self::fetch(path)?;
        new_album.ok_or_else(|| "Inserted album could not be read back".to_string())
    }

    /// delete an album from the db
    pub fn delete_from_db(id: i64) -> Result<usize, String> {
        let conn = open_conn()?;
        let result = conn
            .execute("DELETE FROM albums WHERE id = ?1", params![id])
            .map_err(|e| e.to_string())?;
        Ok(result)
    }

    pub fn reorder_display_order(items: Vec<AlbumDisplayOrder>) -> Result<usize, String> {
        let count = items.len();
        let ids: HashSet<i64> = items.iter().map(|item| item.id).collect();
        let orders: HashSet<i64> = items.iter().map(|item| item.display_order).collect();
        if count == 0 || ids.len() != count || orders.len() != count || items.iter().any(|item| {
            item.id <= 0 || item.display_order < 0 || item.display_order >= count as i64
        }) {
            return Err("Invalid album display order".to_string());
        }

        let mut conn = open_conn()?;
        let tx = conn.transaction().map_err(|e| e.to_string())?;
        let album_count: usize = tx
            .query_row("SELECT COUNT(*) FROM albums", [], |row| row.get(0))
            .map_err(|e| e.to_string())?;
        if album_count != count {
            return Err("Album list changed while reordering".to_string());
        }

        let mut update = tx
            .prepare("UPDATE albums SET display_order_id = ?1 WHERE id = ?2")
            .map_err(|e| e.to_string())?;
        for item in items {
            if update
                .execute(params![item.display_order, item.id])
                .map_err(|e| e.to_string())?
                != 1
            {
                return Err("Album not found while reordering".to_string());
            }
        }
        drop(update);
        tx.commit().map_err(|e| e.to_string())?;
        Ok(count)
    }

    /// Get all albums(album_type = 1) from the db
    pub fn get_all_albums() -> Result<Vec<Self>, String> {
        let conn = open_conn()?;

        let query =
            "SELECT id, name, path, created_at, modified_at, display_order_id, cover_file_id, description, indexed, total, skipped_count, skipped_size, failed_count, failed_size, merged_count, merged_size, last_scan_time, file_types, small_image_filter, excluded_folders
            FROM albums
            ORDER BY display_order_id ASC";

        let mut stmt = conn.prepare(query).map_err(|e| e.to_string())?;

        // Execute the query and map the result to Album structs
        let albums_iter = stmt
            .query_map([], Self::from_row)
            .map_err(|e| e.to_string())?;

        // Collect the results into a Vec<Album>
        let mut albums = Vec::new();
        for album in albums_iter {
            match album {
                Ok(album) => albums.push(album),
                Err(e) => return Err(format!("Failed to retrieve row: {}", e)),
            }
        }
        Ok(albums)
    }

    /// get album info by id
    pub fn get_album_by_id(id: i64) -> Result<Self, String> {
        let conn = open_conn()?;
        let result = conn.query_row(
            "SELECT id, name, path, created_at, modified_at, display_order_id, cover_file_id, description, indexed, total, skipped_count, skipped_size, failed_count, failed_size, merged_count, merged_size, last_scan_time, file_types, small_image_filter, excluded_folders
            FROM albums WHERE id = ?1",
            params![id],
            Self::from_row
        ).map_err(|e| e.to_string())?;
        Ok(result)
    }

    /// update a column value
    pub fn update_column(
        id: i64,
        column: &str,
        value: &dyn rusqlite::ToSql,
    ) -> Result<usize, String> {
        let conn = open_conn()?;
        let query = format!("UPDATE albums SET {} = ?1 WHERE id = ?2", column);
        let result = conn
            .execute(&query, params![value, id])
            .map_err(|e| e.to_string())?;
        Ok(result)
    }

    /// update last scan time
    pub fn update_last_scan_time(album_id: i64, scan_time: i64) -> Result<usize, String> {
        Self::update_column(album_id, "last_scan_time", &scan_time)
    }

    pub fn update_last_scan_results(
        album_id: i64,
        skipped_count: u64,
        skipped_size: u64,
        failed_count: u64,
        failed_size: u64,
        merged_count: u64,
        merged_size: u64,
    ) -> Result<usize, String> {
        let conn = open_conn()?;
        conn.execute(
            "UPDATE albums SET skipped_count = ?1, skipped_size = ?2, failed_count = ?3, failed_size = ?4, merged_count = ?5, merged_size = ?6 WHERE id = ?7",
            params![skipped_count, skipped_size, failed_count, failed_size, merged_count, merged_size, album_id],
        )
        .map_err(|e| e.to_string())
    }

    /// rename the album root metadata and matching folders in one transaction
    pub fn rename_root_folder(old_path: &str, new_path: &str) -> Result<(), String> {
        let new_name = t_utils::get_file_name(new_path);
        let mut conn = open_conn()?;
        let tx = conn.transaction().map_err(|e| e.to_string())?;

        tx.execute(
            "UPDATE albums SET path = ?2 WHERE path = ?1",
            params![old_path, new_path],
        )
        .map_err(|e| e.to_string())?;

        tx.execute(
            "UPDATE afolders
            SET path = CONCAT(?2, SUBSTRING(path, LENGTH(?1) + 1)),
                name = CASE WHEN path = ?1 THEN ?3 ELSE name END
            WHERE path = ?1 OR path LIKE ?4 ESCAPE '\\'",
            params![old_path, new_path, new_name, subtree_like_pattern(old_path)],
        )
        .map_err(|e| e.to_string())?;

        tx.commit().map_err(|e| e.to_string())?;
        Ok(())
    }

    /// update indexed and total progress
    pub fn update_progress(id: i64, indexed: u64, total: u64) -> Result<usize, String> {
        let conn = open_conn()?;
        let result = conn
            .execute(
                "UPDATE albums SET indexed = ?1, total = ?2 WHERE id = ?3",
                params![indexed, total, id],
            )
            .map_err(|e| e.to_string())?;
        Ok(result)
    }

    /// Keep an album cover when it still refers to a displayable file in that
    /// album; otherwise fall back to the first available file or no cover.
    pub fn auto_set_cover(id: i64) -> Result<(), String> {
        let conn = open_conn()?;

        // A cover ID can remain after its file is removed outside Lap. It is
        // valid only when the file still belongs to this album and has a
        // thumbnail that can be displayed in the album list.
        let cover_file_id: Option<i64> = conn
            .query_row(
                "SELECT cover_file_id FROM albums WHERE id = ?1",
                params![id],
                |row| row.get(0),
            )
            .map_err(|e| e.to_string())?;

        if let Some(cover_file_id) = cover_file_id.filter(|file_id| *file_id > 0) {
            let cover_query = format!(
                "SELECT a.id
                     FROM afiles a
                     JOIN afolders b ON a.folder_id = b.id
                     JOIN athumbs c ON a.id = c.file_id
                     WHERE a.id = ?1
                       AND b.album_id = ?2
                       AND (a.file_type IN (1, 2, 3))
                       AND {}",
                format!("{}{}", AFile::live_photo_companion_exclusion_condition(), AFile::album_filter_sql("a")),
            );
            let cover_is_available: Option<i64> = conn
                .query_row(&cover_query, params![cover_file_id, id], |row| row.get(0))
                .optional()
                .map_err(|e| e.to_string())?;
            if cover_is_available.is_some() {
                return Ok(());
            }
        }

        // Choose the first displayable file. A missing candidate intentionally
        // clears the stale cover so the UI uses its default album icon.
        let fallback_query = format!(
            "SELECT a.id
                FROM afiles a
                JOIN afolders b ON a.folder_id = b.id
                JOIN athumbs c ON a.id = c.file_id
                WHERE b.album_id = ?1
                  AND (a.file_type IN (1, 2, 3))
                  AND {}
                ORDER BY a.taken_date ASC
                LIMIT 1",
            format!("{}{}", AFile::live_photo_companion_exclusion_condition(), AFile::album_filter_sql("a")),
        );
        let file_id: Option<i64> = conn
            .query_row(&fallback_query, params![id], |row| row.get(0))
            .optional() // returns Option<i64>
            .map_err(|e| e.to_string())?;

        conn.execute(
            "UPDATE albums SET cover_file_id = ?1 WHERE id = ?2",
            params![file_id, id],
        )
        .map_err(|e| e.to_string())?;

        Ok(())
    }

    /// Recount files for an album from the database and update stored progress.
    /// A completed album stays completed after moving, copying, or deleting
    /// already-indexed files. Partial scan progress is preserved and clamped.
    pub fn recount_album(id: i64) -> Result<Self, String> {
        let conn = open_conn()?;
        let total: i64 = conn
            .query_row(
                &format!(
                    "SELECT COUNT(*) FROM afiles a JOIN afolders b ON a.folder_id = b.id
                    WHERE b.album_id = ?1 AND {} AND {} AND {}",
                    AFile::search_exclusion_condition("b"),
                    AFile::live_photo_companion_exclusion_condition(),
                    AFile::album_scan_predicate("a")
                ),
                params![id],
                |row| row.get(0),
            )
            .map_err(|e| e.to_string())?;
        let (indexed, previous_total): (i64, i64) = conn
            .query_row(
                "SELECT COALESCE(indexed, 0), COALESCE(total, 0)
                 FROM albums WHERE id = ?1",
                params![id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .map_err(|e| e.to_string())?;
        let next_indexed = if indexed >= previous_total {
            total
        } else {
            indexed.min(total).max(0)
        };
        conn.execute(
            "UPDATE albums SET total = ?1, indexed = ?2 WHERE id = ?3",
            params![total, next_indexed, id],
        )
        .map_err(|e| e.to_string())?;
        let result = Self::get_album_by_id(id)?;
        Ok(result)
    }

    /// Count files visible in the current browse filter for every album.
    /// This is read-only; scan completion persists the matching index totals.
    pub fn get_visible_counts() -> Result<HashMap<i64, i64>, String> {
        let conn = open_conn()?;
        let query = format!(
            "SELECT b.album_id, COUNT(DISTINCT a.id)
             FROM afiles a
             JOIN afolders b ON b.id = a.folder_id
             WHERE {} AND {}{}{}
             GROUP BY b.album_id",
            AFile::search_exclusion_condition("b"),
            AFile::live_photo_companion_exclusion_condition(),
            AFile::album_filter_sql("a"),
            AFile::inaccessible_album_filter("b"),
        );
        let mut stmt = conn.prepare(&query).map_err(|e| e.to_string())?;
        let rows = stmt.query_map([], |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?)))
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<HashMap<_, _>, _>>().map_err(|e| e.to_string())
    }

    pub fn merged_file_stats_in_album(album_id: i64) -> Result<(u64, u64), String> {
        let conn = open_conn()?;
        conn.query_row(
            &format!("SELECT COUNT(*), COALESCE(SUM(companion.size), 0) FROM afiles companion
             WHERE companion.id IN (
                 SELECT primary_file.live_photo_video_id FROM afiles primary_file
                 WHERE primary_file.live_photo_video_id IS NOT NULL AND {}
                   AND folder_id IN (SELECT id FROM afolders WHERE album_id = ?1)
             ) AND companion.folder_id IN (SELECT id FROM afolders WHERE album_id = ?1) AND {}",
             AFile::album_scan_predicate("primary_file"), AFile::album_scan_predicate("companion")),
            params![album_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|e| e.to_string())
    }
}

/// Define the album's folder struct
#[derive(Debug, Serialize, Deserialize)]
pub struct AFolder {
    pub id: Option<i64>, // unique id (autoincrement by db)
    pub album_id: i64,   // album id (from albums table)

    // folder basic info
    pub name: String,             // folder name
    pub path: String,             // folder path
    pub created_at: Option<i64>,  // folder create time
    pub modified_at: Option<i64>, // folder modified time

    // extra info
    pub is_favorite: Option<bool>,             // is favorite
    pub is_excluded_from_search: Option<bool>, // exclude folder and children from search
    pub file_count: Option<i64>,               // file count (populated by get_favorite_folders)
    pub has_subfolders: Option<bool>,          // cached direct-subfolder state
}

#[derive(Debug, Clone)]
pub struct FolderSubfolderState {
    pub path: String,
    pub created_at: Option<i64>,
    pub modified_at: Option<i64>,
    pub inode: Option<i64>,
    pub has_subfolders: bool,
}

impl FolderSubfolderState {
    pub fn get_many(paths: &[String]) -> Result<HashMap<String, Self>, String> {
        if paths.is_empty() {
            return Ok(HashMap::new());
        }
        let mut states = HashMap::new();
        let conn = open_conn()?;

        // Keep each query below SQLite's bound-parameter limit.
        for path_chunk in paths.chunks(500) {
            let placeholders = vec!["?"; path_chunk.len()].join(", ");
            let query = format!(
                "SELECT path, modified_at, has_subfolders
                FROM afolders
                WHERE path IN ({})
                  AND has_subfolders IS NOT NULL",
                placeholders
            );
            let mut stmt = conn.prepare(&query).map_err(|e| e.to_string())?;
            let rows = stmt
                .query_map(rusqlite::params_from_iter(path_chunk.iter()), |row| {
                    Ok(Self {
                        path: row.get(0)?,
                        created_at: None,
                        modified_at: row.get(1)?,
                        inode: None,
                        has_subfolders: row.get::<_, i64>(2)? != 0,
                    })
                })
                .map_err(|e| e.to_string())?;
            for state in rows {
                let state = state.map_err(|e| e.to_string())?;
                states.insert(state.path.clone(), state);
            }
        }

        Ok(states)
    }

    /// Persist state after the directory has been fully scanned.  Do not use
    /// this for sidebar probes: advancing `modified_at` before file sync would
    /// make the incremental scanner miss external file changes.
    pub fn update_after_scan(album_id: i64, states: &[Self]) -> Result<(), String> {
        if states.is_empty() {
            return Ok(());
        }
        let mut conn = open_conn()?;
        let tx = conn.transaction().map_err(|e| e.to_string())?;
        {
            let mut stmt = tx
                .prepare(
                    "UPDATE afolders
                    SET modified_at = ?1, has_subfolders = ?2
                    WHERE album_id = ?3 AND path = ?4",
                )
                .map_err(|e| e.to_string())?;
            for state in states {
                stmt.execute(params![
                    state.modified_at,
                    i64::from(state.has_subfolders),
                    album_id,
                    state.path,
                ])
                .map_err(|e| e.to_string())?;
            }
        }
        tx.commit().map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Update only the cached tree shape after a folder-only refresh.  The
    /// directory mtime must remain untouched so a later file sync can still
    /// detect external file changes.
    pub fn update_subfolder_flags(
        album_id: i64,
        states: &[(String, bool)],
    ) -> Result<(), String> {
        if states.is_empty() {
            return Ok(());
        }
        let mut conn = open_conn()?;
        let tx = conn.transaction().map_err(|e| e.to_string())?;
        {
            let mut stmt = tx
                .prepare(
                    "UPDATE afolders
                    SET has_subfolders = ?1
                    WHERE album_id = ?2 AND path = ?3",
                )
                .map_err(|e| e.to_string())?;
            for (path, has_subfolders) in states {
                stmt.execute(params![i64::from(*has_subfolders), album_id, path])
                    .map_err(|e| e.to_string())?;
            }
        }
        tx.commit().map_err(|e| e.to_string())?;
        Ok(())
    }
}

impl AFolder {
    pub fn migrate_path_by_inode(
        album_id: i64,
        path: &str,
        inode: Option<i64>,
    ) -> Result<Option<String>, String> {
        let Some(inode) = inode.filter(|inode| *inode != 0) else {
            return Ok(None);
        };
        let mut conn = open_conn()?;
        let old_path = conn
            .query_row(
                "SELECT path FROM afolders WHERE album_id = ?1 AND inode = ?2 AND path <> ?3",
                params![album_id, inode, path],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .map_err(|e| e.to_string())?;
        match old_path {
            Some(old_path) if old_path != path => {
                Self::apply_path_migrations(
                    &mut conn,
                    album_id,
                    vec![(old_path.clone(), path.to_string())],
                )?;
                Ok(Some(old_path))
            }
            _ => Ok(None),
        }
    }

    fn apply_path_migrations(
        conn: &mut Connection,
        album_id: i64,
        moves: Vec<(String, String)>,
    ) -> Result<(), String> {
        if moves.is_empty() {
            return Ok(());
        }

        let tx = conn.transaction().map_err(|e| e.to_string())?;
        let mut temporary_moves = moves
            .iter()
            .enumerate()
            .map(|(index, (old_path, new_path))| {
                (old_path, new_path, format!("{}.__lap_move_{}_{}", old_path, album_id, index))
            })
            .collect::<Vec<_>>();
        // Apply outer destination paths first so a folder moved into another
        // concurrently moved folder is not removed as stale destination data.
        temporary_moves.sort_by_key(|(_, new_path, _)| new_path.len());
        for (old_path, _, temporary_path) in &temporary_moves {
            tx.execute(
                "UPDATE afolders
                 SET path = CONCAT(?2, SUBSTRING(path, LENGTH(?1) + 1))
                 WHERE album_id = ?3
                   AND (path = ?1 OR SUBSTR(path, 1, LENGTH(?1) + 1) = ?1 || ?4)",
                params![
                    old_path,
                    temporary_path,
                    album_id,
                    std::path::MAIN_SEPARATOR.to_string(),
                ],
            )
            .map_err(|e| e.to_string())?;
        }
        for (_, new_path, temporary_path) in &temporary_moves {
            tx.execute(
                "DELETE FROM afolders
                 WHERE album_id = ?1
                   AND (path = ?2 OR SUBSTR(path, 1, LENGTH(?2) + 1) = ?2 || ?3)",
                params![album_id, new_path, std::path::MAIN_SEPARATOR.to_string()],
            )
            .map_err(|e| e.to_string())?;
            tx.execute(
                "UPDATE afolders
                 SET path = CONCAT(?2, SUBSTRING(path, LENGTH(?1) + 1)),
                     name = CASE WHEN path = ?1 THEN ?3 ELSE name END
                 WHERE album_id = ?4
                   AND (path = ?1 OR SUBSTR(path, 1, LENGTH(?1) + 1) = ?1 || ?5)",
                params![
                    temporary_path,
                    new_path,
                    t_utils::get_file_name(new_path),
                    album_id,
                    std::path::MAIN_SEPARATOR.to_string(),
                ],
            )
            .map_err(|e| e.to_string())?;
        }
        tx.commit().map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Persist every directory observed during a complete album traversal.
    /// This makes the folder tree and mtime sync cover empty directories too,
    /// without adding another filesystem traversal.
    pub fn ensure_subfolder_states(
        album_id: i64,
        states: &[FolderSubfolderState],
    ) -> Result<(), String> {
        if states.is_empty() {
            return Ok(());
        }
        let mut conn = open_conn()?;
        let tx = conn.transaction().map_err(|e| e.to_string())?;
        {
            let mut insert = tx
                .prepare(
                    "INSERT INTO afolders (
                        album_id, name, path, created_at, modified_at,
                        is_favorite, is_excluded_from_search, has_subfolders, inode
                    )
                    VALUES (?1, ?2, ?3, ?4, ?5, NULL, 0, ?6, ?7)",
                )
                .map_err(|e| e.to_string())?;
            let mut update = tx
                .prepare(
                    "UPDATE afolders
                    SET modified_at = ?1, has_subfolders = ?2, inode = ?3
                    WHERE album_id = ?4 AND path = ?5",
                )
                .map_err(|e| e.to_string())?;
            for state in states {
                let updated = update
                    .execute(params![
                        state.modified_at,
                        i64::from(state.has_subfolders),
                        state.inode,
                        album_id,
                        state.path,
                    ])
                    .map_err(|e| e.to_string())?;
                if updated == 0 {
                    insert
                        .execute(params![
                            album_id,
                            t_utils::get_file_name(&state.path),
                            state.path,
                            state.created_at,
                            state.modified_at,
                            i64::from(state.has_subfolders),
                            state.inode,
                        ])
                        .map_err(|e| e.to_string())?;
                }
            }
        }
        tx.commit().map_err(|e| e.to_string())?;
        Ok(())
    }

    /// create a new folder struct
    fn new(album_id: i64, folder_path: &str) -> Result<Self, String> {
        let file_info = t_utils::FileInfo::new(folder_path)?;
        Ok(Self {
            id: None,
            album_id,
            name: file_info.file_name,
            path: folder_path.to_string(),
            created_at: file_info.created,
            modified_at: Some(0), // force first sync
            is_favorite: None,
            is_excluded_from_search: Some(false),
            file_count: None,
            has_subfolders: None,
        })
    }

    /// Function to construct `Self` from a database row
    fn from_row(row: &rusqlite::Row) -> Result<Self, rusqlite::Error> {
        Ok(Self {
            id: Some(row.get(0)?),
            album_id: row.get(1)?,
            name: row.get(2)?,
            path: row.get(3)?,
            created_at: row.get(4)?,
            modified_at: row.get(5)?,
            is_favorite: row.get(6)?,
            is_excluded_from_search: row.get(7)?,
            file_count: None,
            has_subfolders: row.get(8)?,
        })
    }

    /// fetch a folder row from db (by path)
    pub fn fetch(folder_path: &str) -> Result<Option<Self>, String> {
        let conn = open_conn()?;
        Self::fetch_with_conn(&conn, folder_path)
    }

    pub fn get_inode(album_id: i64, folder_path: &str) -> Result<Option<i64>, String> {
        let conn = open_conn()?;
        conn.query_row(
            "SELECT inode FROM afolders WHERE album_id = ?1 AND path = ?2",
            params![album_id, folder_path],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| e.to_string())
    }

    pub fn fetch_with_conn(conn: &Connection, folder_path: &str) -> Result<Option<Self>, String> {
        conn.query_row(
            "SELECT id, album_id, name, path, created_at, modified_at, is_favorite, COALESCE(is_excluded_from_search, 0), has_subfolders
            FROM afolders
            WHERE path = ?1",
            params![folder_path],
            Self::from_row,
        )
        .optional()
        .map_err(|e| e.to_string())
    }

    /// fetch a folder row from db (by id)
    pub fn get_by_id(id: i64) -> Result<Option<Self>, String> {
        let conn = open_conn()?;
        let result = conn
            .query_row(
                "SELECT id, album_id, name, path, created_at, modified_at, is_favorite, COALESCE(is_excluded_from_search, 0), has_subfolders
                FROM afolders
                WHERE id = ?1",
                params![id],
                Self::from_row,
            )
            .optional()
            .map_err(|e| e.to_string())?;
        Ok(result)
    }

    /// fetch all folder rows in the current library database
    pub fn get_all() -> Result<Vec<Self>, String> {
        let conn = open_conn()?;
        let mut stmt = conn
            .prepare(
                "SELECT id, album_id, name, path, created_at, modified_at, is_favorite, COALESCE(is_excluded_from_search, 0), has_subfolders
                FROM afolders",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![], Self::from_row)
            .map_err(|e| e.to_string())?;

        let mut folders = Vec::new();
        for folder in rows {
            folders.push(folder.map_err(|e| e.to_string())?);
        }
        Ok(folders)
    }

    fn insert_with_conn(&self, conn: &Connection) -> Result<usize, String> {
        conn.execute(
            "INSERT INTO afolders (album_id, name, path, created_at, modified_at, is_favorite, is_excluded_from_search, has_subfolders)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                self.album_id, self.name, self.path,
                self.created_at, self.modified_at,
                self.is_favorite, self.is_excluded_from_search, self.has_subfolders
            ],
        )
        .map_err(|e| e.to_string())
    }

    /// insert the folder to db if not exists
    pub fn add_to_db(album_id: i64, folder_path: &str) -> Result<Self, String> {
        let conn = open_conn()?;
        Self::add_to_db_with_conn(&conn, album_id, folder_path)
    }

    pub fn add_to_db_with_conn(
        conn: &Connection,
        album_id: i64,
        folder_path: &str,
    ) -> Result<Self, String> {
        if let Ok(Some(folder)) = Self::fetch_with_conn(conn, folder_path) {
            Self::update_inode_with_conn(conn, album_id, folder_path)?;
            return Ok(folder);
        }
        Self::new(album_id, folder_path)?.insert_with_conn(conn)?;
        Self::update_inode_with_conn(conn, album_id, folder_path)?;
        let new_folder = Self::fetch_with_conn(conn, folder_path)?;
        new_folder.ok_or_else(|| "Inserted folder could not be read back".to_string())
    }

    fn update_inode_with_conn(
        conn: &Connection,
        album_id: i64,
        folder_path: &str,
    ) -> Result<(), String> {
        let inode = t_utils::FileInfo::new(folder_path)
            .ok()
            .map(|info| info.inode as i64)
            .filter(|inode| *inode != 0);
        if let Some(inode) = inode {
            conn.execute(
                "UPDATE afolders SET inode = ?1 WHERE album_id = ?2 AND path = ?3",
                params![inode, album_id, folder_path],
            )
            .map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    fn get_paths_by_album_id(album_id: i64) -> Result<Vec<String>, String> {
        let conn = open_conn()?;
        let mut stmt = conn
            .prepare("SELECT path FROM afolders WHERE album_id = ?1")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![album_id], |row| row.get(0))
            .map_err(|e| e.to_string())?;
        rows.map(|row| row.map_err(|e| e.to_string())).collect()
    }

    /// Remove folders absent from a successful full traversal.  Delete only
    /// missing roots; `delete_folder` removes each root's descendants too.
    pub fn delete_unseen_in_album(
        album_id: i64,
        seen_folders: &[FolderSubfolderState],
    ) -> Result<usize, String> {
        let album = Album::get_album_by_id(album_id)?;
        let seen_paths = seen_folders
            .iter()
            .map(|folder| folder.path.as_str())
            .collect::<HashSet<_>>();
        let missing_paths = Self::get_paths_by_album_id(album_id)?
            .into_iter()
            .filter(|path| !seen_paths.contains(path.as_str()) && !album.excludes_path(Path::new(path)))
            .collect::<Vec<_>>();
        let missing_paths = missing_folder_roots(missing_paths);

        let mut deleted_count = 0;
        for path in missing_paths {
            deleted_count += Self::delete_folder(&path)?;
        }

        Ok(deleted_count)
    }

    /// Remove known direct children of `parent_path` that were not found by a
    /// successful directory read. Descendants are removed together with their
    /// missing direct-child root.
    pub fn delete_unseen_direct_children(
        album_id: i64,
        parent_path: &str,
        seen_paths: &HashSet<String>,
    ) -> Result<usize, String> {
        let album = Album::get_album_by_id(album_id)?;
        let missing_children = Self::get_paths_by_album_id(album_id)?
            .into_iter()
            .filter(|path| {
                Path::new(path).parent() == Some(Path::new(parent_path))
                    && !seen_paths.contains(path)
                    && !album.excludes_path(Path::new(path))
            })
            .collect::<Vec<_>>();

        let folders = Self::get_all()?;
        let mut deleted_count = 0;
        for path in missing_children {
            let descendant_prefix = format!("{}{}", path, std::path::MAIN_SEPARATOR);
            for folder in folders.iter().filter(|folder| {
                folder.album_id == album_id
                    && (folder.path == path || folder.path.starts_with(&descendant_prefix))
            }) {
                if let Some(folder_id) = folder.id {
                    for file in AFile::get_files_by_folder_id(folder_id)? {
                        if let Some(file_id) = file.id {
                            AThumb::delete(file_id)?;
                        }
                    }
                }
            }
            deleted_count += Self::delete_folder(&path)?;
        }
        Ok(deleted_count)
    }

    /// Move a folder subtree by updating its paths and album ID.
    pub fn move_folder(old_path: &str, new_album_id: i64, new_path: &str) -> Result<usize, String> {
        let conn = open_conn()?;
        let result = conn
            .execute(
                "UPDATE afolders
                SET path = CONCAT(?3, SUBSTRING(path, LENGTH(?1) + 1)), album_id = ?2
                WHERE path = ?1 OR path LIKE ?4 ESCAPE '\\'",
                params![
                    old_path,
                    new_album_id,
                    new_path,
                    subtree_like_pattern(old_path),
                ],
            )
            .map_err(|e| e.to_string())?;
        Ok(result)
    }

    /// Replace an existing destination folder subtree and move the source
    /// subtree in one transaction.
    pub fn replace_moved_folder(
        old_path: &str,
        new_album_id: i64,
        new_path: &str,
    ) -> Result<usize, String> {
        let mut conn = open_conn()?;
        let tx = conn.transaction().map_err(|e| e.to_string())?;
        let destination_pattern = subtree_like_pattern(new_path);

        let destination_folder_ids: Vec<i64> = {
            let mut stmt = tx
                .prepare("SELECT id FROM afolders WHERE path = ?1 OR path LIKE ?2 ESCAPE '\\'")
                .map_err(|e| e.to_string())?;
            let rows = stmt
                .query_map(params![new_path, destination_pattern], |row| row.get(0))
                .map_err(|e| e.to_string())?;
            rows.filter_map(|row| row.ok()).collect()
        };

        for folder_id in destination_folder_ids {
            tx.execute(
                "DELETE FROM afiles WHERE folder_id = ?1",
                params![folder_id],
            )
            .map_err(|e| e.to_string())?;
        }
        tx.execute(
            "DELETE FROM afolders WHERE path = ?1 OR path LIKE ?2 ESCAPE '\\'",
            params![new_path, destination_pattern],
        )
        .map_err(|e| e.to_string())?;

        let result = tx
            .execute(
                "UPDATE afolders
                SET path = CONCAT(?3, SUBSTRING(path, LENGTH(?1) + 1)), album_id = ?2
                WHERE path = ?1 OR path LIKE ?4 ESCAPE '\\'",
                params![
                    old_path,
                    new_album_id,
                    new_path,
                    subtree_like_pattern(old_path),
                ],
            )
            .map_err(|e| e.to_string())?;
        tx.commit().map_err(|e| e.to_string())?;
        Ok(result)
    }

    pub fn replace_copied_folder(album_id: i64, folder_path: &str) -> Result<Self, String> {
        let folder = Self::new(album_id, folder_path)?;
        let mut conn = open_conn()?;
        let tx = conn.transaction().map_err(|e| e.to_string())?;
        let destination_pattern = subtree_like_pattern(folder_path);

        let destination_folder_ids: Vec<i64> = {
            let mut stmt = tx
                .prepare("SELECT id FROM afolders WHERE path = ?1 OR path LIKE ?2 ESCAPE '\\'")
                .map_err(|e| e.to_string())?;
            let rows = stmt
                .query_map(params![folder_path, destination_pattern], |row| row.get(0))
                .map_err(|e| e.to_string())?;
            rows.filter_map(|row| row.ok()).collect()
        };
        for folder_id in destination_folder_ids {
            tx.execute(
                "DELETE FROM afiles WHERE folder_id = ?1",
                params![folder_id],
            )
            .map_err(|e| e.to_string())?;
        }
        tx.execute(
            "DELETE FROM afolders WHERE path = ?1 OR path LIKE ?2 ESCAPE '\\'",
            params![folder_path, destination_pattern],
        )
        .map_err(|e| e.to_string())?;
        tx.execute(
            "INSERT INTO afolders
            (album_id, name, path, created_at, modified_at, is_favorite, is_excluded_from_search)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                folder.album_id,
                folder.name,
                folder.path,
                folder.created_at,
                folder.modified_at,
                folder.is_favorite,
                folder.is_excluded_from_search
            ],
        )
        .map_err(|e| e.to_string())?;
        tx.commit().map_err(|e| e.to_string())?;
        Self::fetch(folder_path)?
            .ok_or_else(|| format!("Copied folder missing from DB: {}", folder_path))
    }

    /// delete a folder and all its child folders and files from db
    pub fn delete_folder(folder_path: &str) -> Result<usize, String> {
        let mut conn = open_conn()?;
        let tx = conn.transaction().map_err(|e| e.to_string())?;

        // First, get all folder IDs that will be deleted (the folder itself and all children)
        let folder_ids: Vec<i64> = {
            let mut stmt = tx
                .prepare("SELECT id FROM afolders WHERE path = ?1 OR path LIKE ?2 ESCAPE '\\'")
                .map_err(|e| e.to_string())?;

            let path_pattern = subtree_like_pattern(folder_path);
            let rows = stmt
                .query_map(params![folder_path, path_pattern], |row| row.get(0))
                .map_err(|e| e.to_string())?;

            rows.filter_map(|r| r.ok()).collect()
        };

        // Delete all files in those folders
        for folder_id in &folder_ids {
            tx.execute(
                "DELETE FROM afiles WHERE folder_id = ?1",
                params![folder_id],
            )
            .map_err(|e| e.to_string())?;
        }

        // Delete the folders (the folder and all its children)
        let path_pattern = subtree_like_pattern(folder_path);
        let result = tx
            .execute(
                "DELETE FROM afolders WHERE path = ?1 OR path LIKE ?2 ESCAPE '\\'",
                params![folder_path, path_pattern],
            )
            .map_err(|e| e.to_string())?;

        tx.commit().map_err(|e| e.to_string())?;
        Ok(result)
    }

    // update a column value
    pub fn update_column(
        id: i64,
        column: &str,
        value: &dyn rusqlite::ToSql,
    ) -> Result<usize, String> {
        let conn = open_conn()?;
        let query = format!("UPDATE afolders SET {} = ?1 WHERE id = ?2", column);
        let result = conn
            .execute(&query, params![value, id])
            .map_err(|e| e.to_string())?;
        Ok(result)
    }

    // get a folder's is_favorite status
    pub fn get_is_favorite(folder_path: &str) -> Result<Option<bool>, String> {
        let conn = open_conn()?;
        let result = conn
            .query_row(
                "SELECT is_favorite FROM afolders WHERE path = ?1",
                params![folder_path],
                |row| row.get(0),
            )
            .map_err(|e| e.to_string())?;
        Ok(result)
    }

    // get a folder's is_excluded_from_search status
    pub fn get_is_excluded_from_search(folder_path: &str) -> Result<Option<bool>, String> {
        let conn = open_conn()?;
        let result = conn
            .query_row(
                "SELECT COALESCE(is_excluded_from_search, 0) FROM afolders WHERE path = ?1",
                params![folder_path],
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| e.to_string())?;
        Ok(result)
    }

    // get all favorite folders
    pub fn get_favorite_folders() -> Result<Vec<Self>, String> {
        let conn = open_conn()?;
        let sep = std::path::MAIN_SEPARATOR.to_string().replace('\'', "''");

        let query = format!(
            "SELECT a.id, a.album_id, a.name, a.path, a.created_at, a.modified_at, a.is_favorite,
                EXISTS (
                    SELECT 1 FROM afolders xf
                    WHERE COALESCE(xf.is_excluded_from_search, 0) = 1
                    AND xf.album_id = a.album_id
                    AND (
                        a.path = xf.path
                        OR instr(a.path, xf.path || '{}') = 1
                    )
                ),
                (SELECT COUNT(*) FROM afiles f
                 WHERE f.folder_id = a.id AND f.id NOT IN (
                    SELECT live_photo_video_id FROM afiles WHERE live_photo_video_id IS NOT NULL
                 ))
            FROM afolders a
            WHERE a.is_favorite = 1
            ORDER BY a.name",
            sep
        );

        let mut stmt = conn.prepare(query.as_str()).map_err(|e| e.to_string())?;

        let rows = stmt
            .query_map(params![], |row| {
                Ok(Self {
                    id: Some(row.get(0)?),
                    album_id: row.get(1)?,
                    name: row.get(2)?,
                    path: row.get(3)?,
                    created_at: row.get(4)?,
                    modified_at: row.get(5)?,
                    is_favorite: row.get(6)?,
                    is_excluded_from_search: row.get(7)?,
                    file_count: row.get(8)?,
                    has_subfolders: None,
                })
            })
            .map_err(|e| e.to_string())?;

        let mut folders = Vec::new();
        for folder in rows {
            folders.push(folder.map_err(|e| e.to_string())?);
        }

        Ok(folders)
    }
}

fn missing_folder_roots(mut missing_paths: Vec<String>) -> Vec<String> {
    missing_paths.sort_by_key(|path| path.len());
    let mut roots = Vec::new();
    for path in missing_paths {
        if roots
            .iter()
            .any(|root: &String| {
                Path::new(&path)
                    .strip_prefix(root)
                    .ok()
                    .is_some_and(|relative| !relative.as_os_str().is_empty())
            })
        {
            continue;
        }
        roots.push(path);
    }
    roots
}

pub struct FolderScanState;

impl FolderScanState {
    pub const LIVE_PHOTO_PAIRING: &'static str = "live_photo_pairing";
    pub const LIVE_PHOTO_PAIRING_VERSION: i64 = 5;

    pub fn folders_needing_version(scanner: &str, version: i64) -> Result<HashSet<i64>, String> {
        let conn = open_conn()?;
        let mut stmt = conn
            .prepare(
                "SELECT f.id
                 FROM afolders f
                 LEFT JOIN folder_scan_state state
                   ON state.folder_id = f.id AND state.scanner = ?1
                 WHERE COALESCE(state.version, 0) < ?2",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![scanner, version], |row| row.get(0))
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<HashSet<_>, _>>()
            .map_err(|e| e.to_string())
    }

    pub fn needs_version(folder_id: i64, scanner: &str, version: i64) -> Result<bool, String> {
        let conn = open_conn()?;
        let current = conn
            .query_row(
                "SELECT version FROM folder_scan_state WHERE folder_id = ?1 AND scanner = ?2",
                params![folder_id, scanner],
                |row| row.get::<_, i64>(0),
            )
            .optional()
            .map_err(|e| e.to_string())?
            .unwrap_or(0);
        Ok(current < version)
    }

    pub fn mark_completed(folder_id: i64, scanner: &str, version: i64) -> Result<(), String> {
        let conn = open_conn()?;
        conn.execute(
            "INSERT INTO folder_scan_state (folder_id, scanner, version, updated_at)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(folder_id, scanner) DO UPDATE SET
               version = excluded.version,
               updated_at = excluded.updated_at",
            params![
                folder_id,
                scanner,
                version,
                chrono::Utc::now().timestamp_millis()
            ],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }
}

/// Define the album file struct
#[derive(Debug, Serialize, Deserialize)]
pub struct AFile {
    pub id: Option<i64>, // unique id (autoincrement by db)
    pub folder_id: i64,  // folder id (from folders table)

    // file basic info
    pub name: String,                 // file name
    pub name_pinyin: Option<String>,  // file name pinyin(for sort)
    pub size: i64,                    // file size
    pub file_type: Option<i64>,       // file type (0: all, 1: image, 2: video, 3: audio, 4: other)
    pub format_label: Option<String>, // normalized file format label (from file content)
    pub created_at: Option<i64>,      // file create timestamp
    pub modified_at: Option<i64>,     // file modified timestamp
    pub inode: Option<i64>,           // filesystem inode (for rename detection)
    pub taken_date: Option<i64>,      // taken date timestamp (e_date_time || modified_at)

    // image/video
    pub width: Option<u32>,    // image/video width
    pub height: Option<u32>,   // image/video height
    pub duration: Option<i64>, // video duration

    // extra info
    pub is_favorite: Option<bool>, // is favorite
    pub rating: Option<i32>,       // 0-5 stars
    pub culling_flag: Option<i32>, // 0: unreviewed, 1: pick, 2: reject
    pub rotate: Option<i32>,       // rotate angle (0, 90, 180, 270)
    pub comments: Option<String>,  // comments
    pub has_tags: Option<bool>,    // has tags
    pub has_faces: Option<i32>,    // has faces (0: unprocessed, 1: has faces, 2: no faces)

    // exif info
    pub e_make: Option<String>,  // camera make
    pub e_model: Option<String>, // camera model
    pub e_date_time: Option<String>,
    pub e_software: Option<String>,
    pub e_artist: Option<String>,
    pub e_copyright: Option<String>,
    pub e_description: Option<String>,
    pub e_lens_make: Option<String>,
    pub e_lens_model: Option<String>,
    pub e_exposure_bias: Option<String>,
    pub e_exposure_time: Option<String>,
    pub e_f_number: Option<String>,
    pub e_focal_length: Option<String>,
    pub e_iso_speed: Option<String>,
    pub e_flash: Option<String>,    // flash
    pub e_orientation: Option<u32>, // orientation

    // gps info
    pub gps_latitude: Option<f64>,
    pub gps_longitude: Option<f64>,
    pub gps_altitude: Option<f64>,

    // geo info (from http://www.geonames.org/)
    pub geo_name: Option<String>,   // Location name
    pub geo_admin1: Option<String>, // Administrative district 1
    pub geo_admin2: Option<String>, // Administrative district 2
    pub geo_cc: Option<String>,     // Country code

    // output only
    pub file_path: Option<String>,          // file path (for webview)
    pub album_id: Option<i64>,              // album id (for webview)
    pub album_name: Option<String>,         // album name (for webview)
    pub has_thumbnail: Option<bool>,        // has thumbnail (for webview)
    pub has_collections: Option<bool>,      // belongs to one or more collections
    pub has_embedding: Option<bool>,        // has embedding (for webview)
    pub last_scan_time: Option<i64>,        // last scan timestamp
    pub content_identifier: Option<String>, // Apple Live Photo content identifier
    pub media_subtype: Option<String>,      // live_photo, motion_photo, raw_jpeg_pair, ...
    pub live_photo_video_id: Option<i64>,   // paired Live Photo MOV file id
    pub live_photo_video_path: Option<String>, // paired Live Photo MOV path
    #[serde(default = "default_album_visible")]
    pub album_visible: bool, // output-only: visibility under this album's filters
    pub motion_photo_offset: Option<i64>,   // byte offset of embedded MP4 (Android Motion Photo)
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ACollection {
    pub id: i64,
    pub name: String,
    pub sort_order: i64,
    pub count: i64,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ACollectionOrder {
    pub id: i64,
    pub sort_order: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AFileCollection {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct ACollectionSelectionCount {
    pub collection_id: i64,
    pub count: i64,
}

impl ACollection {
    fn now_ts() -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_secs() as i64)
            .unwrap_or(0)
    }

    fn from_row(row: &rusqlite::Row) -> Result<Self, rusqlite::Error> {
        Ok(Self {
            id: row.get(0)?,
            name: row.get(1)?,
            sort_order: row.get(2)?,
            count: row.get(3)?,
            created_at: row.get(4)?,
            updated_at: row.get(5)?,
        })
    }

    fn ensure_exists(conn: &Connection, id: i64) -> Result<(), String> {
        let exists: Option<i64> = conn
            .query_row(
                "SELECT id FROM acollections WHERE id = ?1",
                params![id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| e.to_string())?;

        if exists.is_some() {
            Ok(())
        } else {
            Err("Collection not found".to_string())
        }
    }

    fn ensure_name_available(conn: &Connection, name: &str, exclude_id: Option<i64>) -> Result<(), String> {
        let existing: Option<i64> = conn
            .query_row(
                "SELECT id FROM acollections WHERE name = ?1 COLLATE NOCASE AND (?2 IS NULL OR id != ?2)",
                params![name, exclude_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| e.to_string())?;
        if existing.is_some() {
            Err("A collection with this name already exists".to_string())
        } else {
            Ok(())
        }
    }

    /// Collection counts are populated lazily after explicit sidebar activation.
    pub fn list() -> Result<Vec<Self>, String> {
        let conn = open_conn()?;
        let query = "SELECT id, name, sort_order, 0 AS count, created_at, updated_at
             FROM acollections
             ORDER BY sort_order ASC, id ASC";
        let mut stmt = conn
            .prepare(query)
            .map_err(|e| e.to_string())?;

        let rows = stmt
            .query_map([], Self::from_row)
            .map_err(|e| e.to_string())?;
        let mut collections = Vec::new();
        for row in rows {
            collections.push(row.map_err(|e| e.to_string())?);
        }
        Ok(collections)
    }

    /// Count visible files for every collection in one grouped query.
    pub fn get_counts() -> Result<HashMap<i64, i64>, String> {
        let conn = open_conn()?;
        let query = format!(
            "SELECT cf.collection_id, COUNT(DISTINCT a.id)
             FROM acollections_files cf
             JOIN afiles a ON a.id = cf.file_id
             JOIN afolders b ON b.id = a.folder_id
             WHERE {}{}{} AND {}
             GROUP BY cf.collection_id",
            AFile::live_photo_companion_exclusion_condition(),
            AFile::album_filter_sql("a"),
            AFile::inaccessible_album_filter("b"),
            AFile::search_exclusion_condition("b"),
        );
        let mut stmt = conn.prepare(&query).map_err(|e| e.to_string())?;
        let rows = stmt.query_map([], |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?)))
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<HashMap<_, _>, _>>().map_err(|e| e.to_string())
    }

    pub fn create(name: &str) -> Result<Self, String> {
        let trimmed = name.trim();
        if trimmed.is_empty() {
            return Err("Collection name cannot be empty".to_string());
        }

        let conn = open_conn()?;
        Self::ensure_name_available(&conn, trimmed, None)?;
        let sort_order: i64 = conn
            .query_row(
                "SELECT COALESCE(MAX(sort_order), -1) + 1 FROM acollections",
                [],
                |row| row.get(0),
            )
            .map_err(|e| e.to_string())?;
        let now = Self::now_ts();
        conn.execute(
            "INSERT INTO acollections (name, sort_order, created_at, updated_at) VALUES (?1, ?2, ?3, ?4)",
            params![trimmed, sort_order, now, now],
        )
        .map_err(|e| e.to_string())?;

        let id = conn.last_insert_rowid();
        Self::get(id)
    }

    pub fn get(id: i64) -> Result<Self, String> {
        let conn = open_conn()?;
        conn.query_row(
            "SELECT c.id, c.name, c.sort_order, COUNT(a.id) AS count, c.created_at, c.updated_at
            FROM acollections c
            LEFT JOIN acollections_files cf ON cf.collection_id = c.id
            LEFT JOIN afiles a ON a.id = cf.file_id
                AND a.id NOT IN (
                    SELECT live_photo_video_id FROM afiles WHERE live_photo_video_id IS NOT NULL
                )
            WHERE c.id = ?1
            GROUP BY c.id",
            params![id],
            Self::from_row,
        )
        .map_err(|e| e.to_string())
    }

    pub fn rename(id: i64, name: &str) -> Result<usize, String> {
        let trimmed = name.trim();
        if trimmed.is_empty() {
            return Err("Collection name cannot be empty".to_string());
        }

        let conn = open_conn()?;
        Self::ensure_name_available(&conn, trimmed, Some(id))?;
        let changed = conn
            .execute(
                "UPDATE acollections SET name = ?1, updated_at = ?2 WHERE id = ?3",
                params![trimmed, Self::now_ts(), id],
            )
            .map_err(|e| e.to_string())?;
        if changed == 0 {
            Err("Collection not found".to_string())
        } else {
            Ok(changed)
        }
    }

    pub fn delete(id: i64) -> Result<usize, String> {
        let conn = open_conn()?;
        let changed = conn
            .execute("DELETE FROM acollections WHERE id = ?1", params![id])
            .map_err(|e| e.to_string())?;
        if changed == 0 {
            Err("Collection not found".to_string())
        } else {
            Ok(changed)
        }
    }

    pub fn reorder(items: Vec<ACollectionOrder>) -> Result<usize, String> {
        let mut conn = open_conn()?;
        let tx = conn.transaction().map_err(|e| e.to_string())?;
        let now = Self::now_ts();
        let mut changed = 0;
        for item in items {
            changed += tx
                .execute(
                    "UPDATE acollections SET sort_order = ?1, updated_at = ?2 WHERE id = ?3",
                    params![item.sort_order, now, item.id],
                )
                .map_err(|e| e.to_string())?;
        }
        tx.commit().map_err(|e| e.to_string())?;
        Ok(changed)
    }

    pub fn add_files(collection_id: i64, file_ids: Vec<i64>) -> Result<(Vec<i64>, Vec<i64>), String> {
        let unique_file_ids: Vec<i64> = file_ids
            .into_iter()
            .filter(|id| *id > 0)
            .collect::<HashSet<i64>>()
            .into_iter()
            .collect();
        if unique_file_ids.is_empty() {
            return Ok((Vec::new(), Vec::new()));
        }

        let mut conn = open_conn()?;
        Self::ensure_exists(&conn, collection_id)?;

        // Find which file ids already exist in this collection
        let placeholders = unique_file_ids
            .iter()
            .map(|_| "?")
            .collect::<Vec<_>>()
            .join(",");
        let existing_sql = format!(
            "SELECT file_id FROM acollections_files WHERE collection_id = ?1 AND file_id IN ({})",
            placeholders
        );
        let mut params_for_existing: Vec<Box<dyn rusqlite::types::ToSql>> =
            vec![Box::new(collection_id)];
        for id in &unique_file_ids {
            params_for_existing.push(Box::new(*id));
        }
        let existing: HashSet<i64> = {
            let mut stmt = conn.prepare(&existing_sql).map_err(|e| e.to_string())?;
            let param_refs: Vec<&dyn rusqlite::types::ToSql> =
                params_for_existing.iter().map(|p| p.as_ref()).collect();
            let rows = stmt
                .query_map(&param_refs[..], |row| row.get::<_, i64>(0))
                .map_err(|e| e.to_string())?;
            let mut set = HashSet::new();
            for row in rows {
                set.insert(row.map_err(|e| e.to_string())?);
            }
            set
        };

        let new_file_ids: Vec<i64> = unique_file_ids
            .iter()
            .filter(|id| !existing.contains(id))
            .copied()
            .collect();
        let skipped_file_ids: Vec<i64> = unique_file_ids
            .iter()
            .filter(|id| existing.contains(id))
            .copied()
            .collect();

        if new_file_ids.is_empty() {
            return Ok((Vec::new(), skipped_file_ids));
        }

        let tx = conn.transaction().map_err(|e| e.to_string())?;
        let now = Self::now_ts();
        let mut added_file_ids = Vec::new();
        for file_id in new_file_ids {
            let added = tx
                .execute(
                    "INSERT OR IGNORE INTO acollections_files (collection_id, file_id, added_at)
                    SELECT ?1, id, ?3 FROM afiles WHERE id = ?2",
                    params![collection_id, file_id, now],
                )
                .map_err(|e| e.to_string())?;
            if added > 0 {
                added_file_ids.push(file_id);
            }
        }
        tx.execute(
            "UPDATE acollections SET updated_at = ?1 WHERE id = ?2",
            params![now, collection_id],
        )
        .map_err(|e| e.to_string())?;
        tx.commit().map_err(|e| e.to_string())?;
        Ok((added_file_ids, skipped_file_ids))
    }

    pub fn remove_files(collection_id: i64, file_ids: Vec<i64>) -> Result<usize, String> {
        let unique_file_ids: HashSet<i64> = file_ids.into_iter().filter(|id| *id > 0).collect();
        if unique_file_ids.is_empty() {
            return Ok(0);
        }

        let mut conn = open_conn()?;
        Self::ensure_exists(&conn, collection_id)?;
        let tx = conn.transaction().map_err(|e| e.to_string())?;
        let now = Self::now_ts();
        let mut changed = 0;
        for file_id in unique_file_ids {
            changed += tx
                .execute(
                    "DELETE FROM acollections_files WHERE collection_id = ?1 AND file_id = ?2",
                    params![collection_id, file_id],
                )
                .map_err(|e| e.to_string())?;
        }
        tx.execute(
            "UPDATE acollections SET updated_at = ?1 WHERE id = ?2",
            params![now, collection_id],
        )
        .map_err(|e| e.to_string())?;
        tx.commit().map_err(|e| e.to_string())?;
        Ok(changed)
    }

    pub fn clear(collection_id: i64) -> Result<usize, String> {
        let conn = open_conn()?;
        Self::ensure_exists(&conn, collection_id)?;
        let changed = conn
            .execute(
                "DELETE FROM acollections_files WHERE collection_id = ?1",
                params![collection_id],
            )
            .map_err(|e| e.to_string())?;
        conn.execute(
            "UPDATE acollections SET updated_at = ?1 WHERE id = ?2",
            params![Self::now_ts(), collection_id],
        )
        .map_err(|e| e.to_string())?;
        Ok(changed)
    }

    pub fn file_ids(collection_id: i64) -> Result<Vec<i64>, String> {
        let conn = open_conn()?;
        Self::ensure_exists(&conn, collection_id)?;
        let mut stmt = conn
            .prepare(
                "SELECT file_id FROM acollections_files
                WHERE collection_id = ?1
                ORDER BY added_at DESC, file_id DESC",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![collection_id], |row| row.get(0))
            .map_err(|e| e.to_string())?;
        let mut ids = Vec::new();
        for id in rows {
            ids.push(id.map_err(|e| e.to_string())?);
        }
        Ok(ids)
    }

    pub fn for_file(file_id: i64) -> Result<Vec<AFileCollection>, String> {
        let conn = open_conn()?;
        let mut stmt = conn
            .prepare(
                "SELECT c.id, c.name
                FROM acollections_files cf
                JOIN acollections c ON c.id = cf.collection_id
                WHERE cf.file_id = ?1
                ORDER BY c.sort_order ASC, c.id ASC",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![file_id], |row| {
                Ok(AFileCollection {
                    id: row.get(0)?,
                    name: row.get(1)?,
                })
            })
            .map_err(|e| e.to_string())?;

        let mut collections = Vec::new();
        for row in rows {
            collections.push(row.map_err(|e| e.to_string())?);
        }
        Ok(collections)
    }

    pub fn get_selection_counts(file_ids: &[i64]) -> Result<Vec<ACollectionSelectionCount>, String> {
        if file_ids.is_empty() {
            return Ok(Vec::new());
        }

        let mut conn = open_conn()?;
        let tx = conn.transaction().map_err(|e| e.to_string())?;
        tx.execute(
            "CREATE TEMP TABLE IF NOT EXISTS selected_file_ids (id INTEGER PRIMARY KEY)",
            [],
        )
        .map_err(|e| e.to_string())?;
        tx.execute("DELETE FROM selected_file_ids", [])
            .map_err(|e| e.to_string())?;
        populate_selected_file_ids(&tx, file_ids)?;

        let counts = {
            let mut stmt = tx
                .prepare(
                    "SELECT cf.collection_id, COUNT(*)
                     FROM acollections_files cf
                     INNER JOIN selected_file_ids selected ON selected.id = cf.file_id
                     GROUP BY cf.collection_id",
                )
                .map_err(|e| e.to_string())?;
            let rows = stmt
                .query_map([], |row| {
                    Ok(ACollectionSelectionCount {
                        collection_id: row.get(0)?,
                        count: row.get(1)?,
                    })
                })
                .map_err(|e| e.to_string())?;
            rows.collect::<Result<Vec<_>, _>>()
                .map_err(|e| e.to_string())?
        };

        tx.execute("DELETE FROM selected_file_ids", [])
            .map_err(|e| e.to_string())?;
        tx.commit().map_err(|e| e.to_string())?;
        Ok(counts)
    }
}

/// Define the timeline marker struct for scrollbar markers
#[derive(Debug, Serialize, Deserialize)]
pub struct ATimeLine {
    pub year: Option<i32>,
    pub month: Option<i32>,
    pub date: Option<i32>,
    pub position: i64, // Row index in the sorted fileList
}

/// Define the query parameters struct for file queries
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct QueryParams {
    pub search_file_name: String, // file name search
    pub search_file_type: i64,
    pub sort_type: i64,
    pub sort_order: i64,
    #[serde(default)]
    pub random_seed: i64,
    pub search_all_subfolders: String,
    pub search_folder: String,
    pub start_date: i64,
    pub end_date: i64,
    pub calendar_sort: i64, // 0=taken asc … 5=modified desc (sort / 2 → column)
    #[serde(default)]
    pub folder_sort: i64, // 0=name asc, 1=name desc, 2=date asc, 3=date desc
    #[serde(default)]
    pub category_sort: i64, // 0=name asc, 1=name desc, 2=count asc, 3=count desc
    pub make: String,
    pub model: String,
    pub lens_make: String,
    pub lens_model: String,
    pub location_admin1: String,
    pub location_name: String,
    pub is_favorite: bool,
    pub rating: i64,
    #[serde(default = "default_culling_flag")]
    pub culling_flag: i64,
    pub tag_id: i64,
    #[serde(default)]
    pub tag_group_id: i64,
    pub person_id: i64,
    // GPS bounding box filter (e.g. for "photos in this map area")
    #[serde(default)]
    pub gps_min_lat: Option<f64>,
    #[serde(default)]
    pub gps_max_lat: Option<f64>,
    #[serde(default)]
    pub gps_min_lon: Option<f64>,
    #[serde(default)]
    pub gps_max_lon: Option<f64>,
    #[serde(default)]
    pub group_by: i64,
}

fn default_culling_flag() -> i64 {
    -1
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SmartRule {
    pub id: String,
    pub field: String,
    pub operator: String,
    pub value: JsonValue,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SmartQueryParams {
    #[serde(default = "default_smart_query_version")]
    pub version: i32,
    #[serde(default = "default_smart_query_match")]
    pub r#match: String,
    #[serde(default)]
    pub rules: Vec<SmartRule>,
    pub sort_type: i64,
    pub sort_order: i64,
    #[serde(default)]
    pub random_seed: i64,
    #[serde(default)]
    pub calendar_sort: i64,
    #[serde(default)]
    pub folder_sort: i64,
    #[serde(default)]
    pub category_sort: i64,
    #[serde(default)]
    pub group_by: i64,
    #[serde(default)]
    pub gps_min_lat: Option<f64>,
    #[serde(default)]
    pub gps_max_lat: Option<f64>,
    #[serde(default)]
    pub gps_min_lon: Option<f64>,
    #[serde(default)]
    pub gps_max_lon: Option<f64>,
}

fn default_smart_query_version() -> i32 {
    1
}

fn default_smart_query_match() -> String {
    "all".to_string()
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupedQueryResult {
    pub rows: Vec<GroupedQueryRow>,
    pub groups: Vec<GroupedQueryGroup>,
    pub total_item_count: i64,
    pub total_row_count: i64,
    pub total_size: i64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupedQueryGroup {
    pub group_id: String,
    pub label: String,
    pub count: i64,
    pub size: i64,
    pub row_index: i64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum GroupedQueryRow {
    #[serde(rename_all = "camelCase")]
    Group {
        row_id: String,
        group_id: String,
        label: String,
        count: i64,
        size: i64,
    },
    #[serde(rename_all = "camelCase")]
    Item {
        row_id: String,
        group_id: String,
        file_index: i64,
        file: AFile,
    },
}

#[derive(Debug, Clone)]
struct QueryGroup {
    id: String,
    label: String,
    count: i64,
    size: i64,
}

const GROUP_BY_FOLDER_PATH: i64 = 1;
const GROUP_BY_DATE_DAY: i64 = 2;
const GROUP_BY_DATE_MONTH: i64 = 3;
const GROUP_BY_RATING: i64 = 4;
const GROUP_BY_LOCATION: i64 = 5;
const GROUP_BY_CAMERA: i64 = 6;
const GROUP_BY_LENS: i64 = 7;
const GROUP_BY_DATE_YEAR: i64 = 8;
const GROUP_BY_FILE_TYPE: i64 = 9;
const GROUP_BY_CULLING: i64 = 10;

/// Define the AI image search parameters struct
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ImageSearchParams {
    pub search_text: String,  // search image text (for AI search)
    pub file_id: Option<i64>, // file id (for similar image search)
    pub threshold: f32,       // search threshold
    #[serde(default)]
    pub file_type: i64, // file type bitmask (0=all, 1=image, 2=video, 4=raw)
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryVisibleCounts {
    pub all: i64, pub favorite: i64, pub today: i64, pub rated: i64, pub unrated: i64,
    pub rating_1: i64, pub rating_2: i64, pub rating_3: i64, pub rating_4: i64, pub rating_5: i64,
    pub pick: i64, pub reject: i64, pub unreviewed: i64,
}

impl AFile {
    pub fn get_library_visible_counts() -> Result<LibraryVisibleCounts, String> {
        let conn = open_conn()?;
        let today = chrono::Local::now().format("%m-%d").to_string();
        let query = format!("SELECT COUNT(*), COALESCE(SUM(a.is_favorite=1),0), COALESCE(SUM(strftime('%m-%d',a.taken_date,'unixepoch','localtime')=?1),0), COALESCE(SUM(a.rating>0),0), COALESCE(SUM(a.rating=0),0), COALESCE(SUM(a.rating=1),0), COALESCE(SUM(a.rating=2),0), COALESCE(SUM(a.rating=3),0), COALESCE(SUM(a.rating=4),0), COALESCE(SUM(a.rating=5),0), COALESCE(SUM(a.culling_flag=1),0), COALESCE(SUM(a.culling_flag=2),0), COALESCE(SUM(a.culling_flag=0),0) FROM afiles a JOIN afolders b ON b.id=a.folder_id WHERE {} AND {}{}{}", Self::search_exclusion_condition("b"), Self::live_photo_companion_exclusion_condition(), Self::album_filter_sql("a"), Self::inaccessible_album_filter("b"));
        conn.query_row(&query, params![today], |r| Ok(LibraryVisibleCounts { all:r.get(0)?, favorite:r.get(1)?, today:r.get(2)?, rated:r.get(3)?, unrated:r.get(4)?, rating_1:r.get(5)?, rating_2:r.get(6)?, rating_3:r.get(7)?, rating_4:r.get(8)?, rating_5:r.get(9)?, pick:r.get(10)?, reject:r.get(11)?, unreviewed:r.get(12)? })).map_err(|e| e.to_string())
    }
    fn inaccessible_album_filter(folder_alias: &str) -> String {
        match t_utils::inaccessible_album_ids().as_slice() {
            [] => String::new(),
            ids => format!(
                " AND {}.album_id NOT IN ({})",
                folder_alias,
                ids.iter().map(i64::to_string).collect::<Vec<_>>().join(",")
            ),
        }
    }

    /// Exclude files whose folder path is the excluded folder itself or one of its children.
    /// The caller must pass the alias for the file's joined afolders row.
    fn search_exclusion_condition(folder_alias: &str) -> String {
        let sep = std::path::MAIN_SEPARATOR.to_string().replace('\'', "''");
        format!(
            "(CASE
                WHEN NOT EXISTS (
                    SELECT 1 FROM afolders
                    WHERE is_excluded_from_search = 1
                )
                THEN 1
                ELSE NOT EXISTS (
                    SELECT 1 FROM afolders xf
                    WHERE xf.is_excluded_from_search = 1
                    AND xf.album_id = {folder_alias}.album_id
                    AND (
                        {folder_alias}.path = xf.path
                        OR instr({folder_alias}.path, xf.path || '{}') = 1
                    )
                )
            END)",
            sep
        )
    }

    fn new(folder_id: i64, file_path: &str, file_type: i64) -> Result<Self, String> {
        Self::new_with_raw_info(folder_id, file_path, file_type, None)
    }

    fn new_with_raw_info(folder_id: i64, file_path: &str, file_type: i64, raw_info: Option<t_libraw::RawInfo>) -> Result<Self, String> {
        let raw_info = if file_type == 3 {
            raw_info.or_else(|| t_libraw::get_raw_info(file_path).ok())
        } else { None };
        let file_info = t_utils::FileInfo::new(file_path)?;

        // get dimensions and duration based on file type
        let (mut width, mut height, mut duration) = (0u32, 0u32, 0u64);

        // Initialize metadata fields
        let mut taken_date: Option<i64> = None;
        let mut e_make: Option<String> = None;
        let mut e_model: Option<String> = None;
        let mut e_date_time: Option<String> = None;
        let mut e_software: Option<String> = None;
        let mut e_artist: Option<String> = None;
        let mut e_copyright: Option<String> = None;
        let mut e_description: Option<String> = None;
        let mut e_lens_make: Option<String> = None;
        let mut e_lens_model: Option<String> = None;
        let mut e_exposure_bias: Option<String> = None;
        let mut e_exposure_time: Option<String> = None;
        let mut e_f_number: Option<String> = None;
        let mut e_focal_length: Option<String> = None;
        let mut e_iso_speed: Option<String> = None;
        let mut e_flash: Option<String> = None;
        let mut e_orientation: Option<u32> = None;
        let mut gps_latitude: Option<f64> = None;
        let mut gps_longitude: Option<f64> = None;
        let mut gps_altitude: Option<f64> = None;
        let mut content_identifier: Option<String> = None;
        let mut media_subtype: Option<String> = None;
        let mut motion_photo_offset: Option<i64> = None;

        // Pre-read file header once for images (saves 3-4 redundant File::open per file).
        let file_header: Option<Vec<u8>> = if file_type == 1 || file_type == 3 {
            std::fs::File::open(file_path).ok().and_then(|mut f| {
                use std::io::Read;
                let mut buf = vec![0u8; 128 * 1024];
                f.read(&mut buf).ok().map(|n| {
                    buf.truncate(n);
                    buf
                })
            })
        } else {
            None
        };
        let file_header_deref = file_header.as_deref();

        if file_type == 1
            && matches!(
                Path::new(file_path)
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .unwrap_or("")
                    .to_ascii_lowercase()
                    .as_str(),
                "heic" | "heif" | "hif" | "jpg" | "jpeg"
            )
        {
            content_identifier = file_header_deref
                .and_then(crate::t_apple_sidecar::apple_content_identifier_from_bytes);
        }

        // Android Motion Photo: JPEG with an embedded MP4 appended after EOI.
        if file_type == 1 && t_image::is_jpeg_path(file_path) {
            // Zero records that this JPEG has been checked and is not a Motion Photo.
            // NULL is reserved for JPEGs indexed before Motion Photo support, so a
            // later rescan can detect them once.
            motion_photo_offset = Some(0);
            if let Some(offset) = crate::t_motion_photo::detect_motion_photo(Path::new(file_path)) {
                media_subtype = Some("motion_photo".to_string());
                motion_photo_offset = Some(offset as i64);
            }
        }

        match file_type {
            1 => {
                let (w, h) = t_image::get_image_dimensions(file_path)?;
                width = w;
                height = h;
            }
            2 => {
                let video_metadata = t_video::get_video_metadata(file_path)?;
                width = video_metadata.width;
                height = video_metadata.height;
                duration = video_metadata.duration;
                e_make = video_metadata.e_make;
                e_model = video_metadata.e_model;
                e_date_time = video_metadata.e_date_time;
                e_software = video_metadata.e_software;
                gps_latitude = video_metadata.gps_latitude;
                gps_longitude = video_metadata.gps_longitude;
                gps_altitude = video_metadata.gps_altitude;
                content_identifier = video_metadata.content_identifier;
            }
            3 => {
                let (w, h) = match raw_info.as_ref().and_then(|info| info.dimensions)
                    .filter(|(w, h)| *w > 0 && *h > 0) {
                    Some(dimensions) => dimensions,
                    None => t_image::get_raw_dimensions(file_path)?,
                };
                width = w;
                height = h;
            }
            _ => {}
        };

        let format_label = if let Some(hdr) = file_header_deref {
            if file_type == 3 {
                Some("RAW".to_string())
            } else {
                t_utils::detect_label_from_header(hdr, file_type)
            }
        } else {
            t_utils::detect_file_format_label(file_path, file_type)
        };

        if file_type == 1 || file_type == 3 {
            // Image file — reuse the pre-read header when it contains EXIF.
            // Some older JPEGs place EXIF after large APP segments (such as an
            // ICC profile), beyond this header buffer. Fall back to scanning
            // the full JPEG so their capture settings are indexed as well.
            let exif = if let Some(hdr) = file_header_deref {
                t_image::read_exif_from_bytes_permissive(hdr).or_else(|| {
                    (file_type == 1 && t_image::is_jpeg_path(file_path))
                        .then(|| t_image::read_exif_permissive(file_path))
                        .flatten()
                })
            } else {
                t_image::read_exif_permissive(file_path)
            };

            // Extracts EXIF orientation field.
            // 1: Horizontal (normal)
            // 2: Mirror horizontal
            // 3: Rotate 180
            // 4: Mirror vertical
            // 5: Mirror horizontal and rotate 270 CW
            // 6: Rotate 90 CW
            // 7: Mirror horizontal and rotate 90 CW
            // 8: Rotate 270 CW
            e_orientation = exif.as_ref().and_then(|exif_data| {
                exif_data
                    .get_field(Tag::Orientation, In::PRIMARY)
                    .or_else(|| exif_data.fields().find(|f| f.tag == Tag::Orientation))
                    .and_then(|field| field.value.get_uint(0))
                    .map(|v| v as u32)
            });

            // 2. Binary Scan Fallback if still None or 1
            if e_orientation.is_none() || e_orientation == Some(1) {
                if let Some(hdr) = file_header_deref {
                    if let Some(bo) = t_image::scan_orientation_binary(hdr) {
                        e_orientation = Some(bo as u32);
                    }
                }
            }

            if e_orientation.is_none() {
                e_orientation = Some(1);
            }

            // Process flash data
            e_flash = exif.as_ref().and_then(|exif_data| {
                exif_data
                    .get_field(Tag::Flash, In::PRIMARY)
                    .and_then(|field| field.value.get_uint(0))
                    .map(|val| {
                        if val & 1 == 1 {
                            "Fired".to_string()
                        } else {
                            "Not fired".to_string()
                        }
                    })
            });

            // Extract GPS data
            let (lat, lon, alt) = Self::extract_gps_data(&exif);
            gps_latitude = lat;
            gps_longitude = lon;
            gps_altitude = alt;

            taken_date = Self::get_exif_field(&exif, Tag::DateTimeOriginal)
                .and_then(|exif_date| t_utils::meta_date_to_timestamp(&exif_date))
                .or(file_info.modified);

            e_make = Self::get_exif_field(&exif, Tag::Make);
            e_model = Self::get_exif_field(&exif, Tag::Model);
            e_date_time = Self::get_exif_field(&exif, Tag::DateTimeOriginal);
            e_software = Self::get_exif_field(&exif, Tag::Software);
            e_artist = Self::get_exif_field(&exif, Tag::Artist);
            e_copyright = Self::get_exif_field(&exif, Tag::Copyright);
            e_description = Self::get_exif_field(&exif, Tag::ImageDescription);
            e_lens_make = Self::get_exif_field(&exif, Tag::LensMake);
            e_lens_model = Self::get_exif_field(&exif, Tag::LensModel);
            e_exposure_bias = Self::get_exif_field(&exif, Tag::ExposureBiasValue);
            e_exposure_time = Self::get_exif_field(&exif, Tag::ExposureTime);
            e_f_number = Self::get_exif_field(&exif, Tag::FNumber);
            e_focal_length = Self::get_exif_field(&exif, Tag::FocalLength);
            e_iso_speed = Self::get_exif_field(&exif, Tag::PhotographicSensitivity);

            // The editor uses little_exif to preserve metadata. Some legacy
            // JPEGs are accepted by that reader but rejected by kamadak-exif,
            // which previously made capture settings appear only after an edit
            // was saved as a new image. Only use the same reader when
            // kamadak-exif found none of the capture settings, so a record
            // never combines partial values from two parsers.
            if t_image::is_jpeg_path(file_path)
                && e_exposure_time.is_none()
                && e_f_number.is_none()
                && e_focal_length.is_none()
                && e_iso_speed.is_none()
            {
                let capture_settings = t_image::read_capture_settings_with_little_exif(file_path);
                e_exposure_time = e_exposure_time.or(capture_settings.exposure_time);
                e_f_number = e_f_number.or(capture_settings.f_number);
                e_focal_length = e_focal_length.or(capture_settings.focal_length);
                e_iso_speed = e_iso_speed.or(capture_settings.iso_speed);
            }

            // Fallback: infer lens make from lens model prefix when LensMake is missing.
            if e_lens_make.is_none() {
                if let Some(model) = e_lens_model.as_deref() {
                    e_lens_make = t_lens::infer_lens_make(model).map(|s| s.to_string());
                }
            }

            // For RAW files, LibRaw is the primary metadata source.
            // It reads the file directly and does not rely on the embedded JPEG
            // that the permissive EXIF reader scans, so it is robust against
            // RAW files whose EXIF data is stored outside the preview image.
            if file_type == 3 {
                if let Some(meta) = raw_info.as_ref().map(|info| info.meta.clone()) {
                    if e_make.is_none() {
                        e_make = meta.make;
                    }
                    if e_model.is_none() {
                        e_model = meta.model;
                    }
                    if e_software.is_none() {
                        e_software = meta.software;
                    }
                    if e_artist.is_none() {
                        e_artist = meta.artist;
                    }
                    if e_description.is_none() {
                        e_description = meta.description;
                    }
                    if e_iso_speed.is_none() {
                        e_iso_speed = meta.iso_speed;
                    }
                    if e_exposure_time.is_none() {
                        e_exposure_time = meta.shutter;
                    }
                    if e_f_number.is_none() {
                        e_f_number = meta.aperture;
                    }
                    if e_focal_length.is_none() {
                        e_focal_length = meta.focal_len;
                    }
                    if e_flash.is_none() {
                        e_flash = meta.flash_used;
                    }
                    if e_lens_make.is_none() {
                        e_lens_make = meta.lens_make;
                    }
                    if e_lens_model.is_none() {
                        e_lens_model = meta.lens_model;
                    }
                    if taken_date == file_info.modified {
                        if let Some(ts) = meta.timestamp {
                            taken_date = Some(ts);
                        }
                    }
                }
            }

            // Binary String Fallback if metadata is still missing (Industry standard for tough files)
            if e_make.is_none()
                || e_model.is_none()
                || e_date_time.is_none()
                || e_software.is_none()
                || e_lens_make.is_none()
                || e_lens_model.is_none()
            {
                if let Some(data) = file_header_deref {
                    let tiff_base = Self::find_tiff_base(data);
                    if e_make.is_none() {
                        e_make = Self::scrape_ascii_from_tag(data, tiff_base, 0x010f);
                    }
                    if e_model.is_none() {
                        e_model = Self::scrape_ascii_from_tag(data, tiff_base, 0x0110);
                    }
                    if e_date_time.is_none() {
                        e_date_time = Self::scrape_ascii_from_tag(data, tiff_base, 0x9003)
                            .or_else(|| Self::scrape_ascii_from_tag(data, tiff_base, 0x0132));
                    }
                    if e_software.is_none() {
                        e_software = Self::scrape_ascii_from_tag(data, tiff_base, 0x0131);
                    }
                    if e_lens_model.is_none() {
                        e_lens_model = Self::scrape_ascii_from_tag(data, tiff_base, 0xa434);
                    }
                    if e_lens_make.is_none() {
                        e_lens_make = Self::scrape_ascii_from_tag(data, tiff_base, 0xa433);
                    }
                    // Extra Orientation fallback for Sony MakerNotes (Tag 0x2000)
                    if e_orientation.is_none() || e_orientation == Some(1) {
                        if let Some(so) = Self::scrape_u16_from_tag(data, 0x2000) {
                            if (1..=8).contains(&so) {
                                e_orientation = Some(so as u32);
                            }
                        }
                    }
                }
            }

            if e_lens_make.is_none() {
                if let Some(model) = e_lens_model.as_deref() {
                    e_lens_make = t_lens::infer_lens_make(model).map(|s| s.to_string());
                }
            }

            // Re-update taken_date if we found e_date_time via binary fallback
            taken_date = Self::capture_date_with_fallback(
                taken_date, file_info.modified, e_date_time.as_deref());
        } else if file_type == 2 {
            taken_date = e_date_time
                .as_ref()
                .and_then(|dt| t_utils::meta_date_to_timestamp(dt))
                .or(file_info.modified);
        }

        // Geocoding based on GPS coordinates from any source
        let (geo_name, geo_admin1, geo_admin2, geo_cc) =
            if let (Some(lat), Some(lon)) = (gps_latitude, gps_longitude) {
                match t_utils::GEOCODER.search((lat, lon)) {
                    Some(result) => (
                        Some(result.record.name.clone()),
                        Some(result.record.admin1.clone()),
                        Some(result.record.admin2.clone()),
                        Some(result.record.cc.clone()),
                    ),
                    None => (None, None, None, None),
                }
            } else {
                (None, None, None, None)
            };

        // RAW and TIFF dimensions already match their decoder output.
        let should_swap_dimensions_for_orientation = file_type != 3
            && !t_image::is_heic_path(file_path)
            && !t_libraw::is_tiff_path(file_path);

        let file = Self {
            id: None,
            folder_id,

            name: file_info.file_name.clone(),
            name_pinyin: Some(t_utils::natural_sort_key(
                &file_info.file_name.to_lowercase(),
            )), // natural sort key (case-insensitive, pinyin + zero-padded numbers)
            size: file_info.file_size,
            file_type: Some(file_type),
            format_label,
            created_at: file_info.created,
            modified_at: file_info.modified,
            inode: Some(file_info.inode as i64),

            taken_date,
            width: e_orientation
                .map(|orientation| {
                    if should_swap_dimensions_for_orientation && orientation > 4 {
                        height
                    } else {
                        width
                    }
                })
                .or(Some(width)),
            height: e_orientation
                .map(|orientation| {
                    if should_swap_dimensions_for_orientation && orientation > 4 {
                        width
                    } else {
                        height
                    }
                })
                .or(Some(height)),
            duration: Some(duration as i64),

            is_favorite: None,
            rating: Some(0),
            culling_flag: Some(0),
            rotate: None,
            comments: t_ai_png::extract_comment(file_path),
            has_tags: Some(false),
            has_faces: Some(0),

            e_make,
            e_model,
            e_date_time,
            e_software,
            e_artist,
            e_copyright,
            e_description,
            e_lens_make,
            e_lens_model,
            e_exposure_bias,
            e_exposure_time,
            e_f_number,
            e_focal_length,
            e_iso_speed,
            e_flash,
            e_orientation,

            gps_latitude,
            gps_longitude,
            gps_altitude,

            geo_name,
            geo_admin1,
            geo_admin2,
            geo_cc,

            file_path: None,
            album_id: None,
            album_name: None,
            has_thumbnail: None,
            has_collections: None,
            has_embedding: None,
            last_scan_time: Some(0),
            content_identifier,
            media_subtype,
            live_photo_video_id: None,
            live_photo_video_path: None,
            motion_photo_offset,
            album_visible: true,
        };

        Ok(file)
    }

    fn capture_date_with_fallback(taken: Option<i64>, modified: Option<i64>, date: Option<&str>) -> Option<i64> {
        if taken == modified {
            date.and_then(t_utils::meta_date_to_timestamp).or(taken)
        } else { taken }
    }

    /// Read the capture timestamp used by Lap before a file is imported.
    /// This keeps date-organized imports consistent with the timestamp shown
    /// after the same file has been indexed.
    pub fn capture_timestamp_with_raw_info(file_path: &str, file_type: i64, raw_info: Option<&t_libraw::RawInfo>) -> Result<i64, String> {
        let modified = t_utils::systemtime_to_timestamp(
            std::fs::metadata(file_path).map_err(|e| e.to_string())?.modified().ok());
        let taken = if file_type == 2 {
            t_video::get_video_metadata(file_path)?.e_date_time
                .as_deref().and_then(t_utils::meta_date_to_timestamp).or(modified)
        } else if file_type == 1 || file_type == 3 {
            use std::io::Read;
            let header = std::fs::File::open(file_path).ok().and_then(|mut file| {
                let mut bytes = vec![0; 128 * 1024];
                file.read(&mut bytes).ok().map(|n| { bytes.truncate(n); bytes })
            });
            let exif = if let Some(bytes) = header.as_deref() {
                t_image::read_exif_from_bytes_permissive(bytes).or_else(|| {
                    (file_type == 1 && t_image::is_jpeg_path(file_path))
                        .then(|| t_image::read_exif_permissive(file_path)).flatten()
                })
            } else { t_image::read_exif_permissive(file_path) };
            let mut date = Self::get_exif_field(&exif, Tag::DateTimeOriginal);
            let mut taken = date.as_deref().and_then(t_utils::meta_date_to_timestamp).or(modified);
            if file_type == 3 && taken == modified {
                let timestamp = match raw_info {
                    Some(info) => info.meta.timestamp,
                    None => t_libraw::get_raw_meta(file_path).ok().and_then(|meta| meta.timestamp),
                };
                taken = timestamp.or(taken);
            }
            if date.is_none() {
                if let Some(bytes) = header.as_deref() {
                    let tiff_base = Self::find_tiff_base(bytes);
                    date = Self::scrape_ascii_from_tag(bytes, tiff_base, 0x9003)
                        .or_else(|| Self::scrape_ascii_from_tag(bytes, tiff_base, 0x0132));
                }
            }
            Self::capture_date_with_fallback(taken, modified, date.as_deref())
        } else { modified };
        taken.ok_or_else(|| format!("Could not read a date from: {}", file_path))
    }

    fn extract_gps_data(exif: &Option<exif::Exif>) -> (Option<f64>, Option<f64>, Option<f64>) {
        let Some(exif_data) = exif else {
            return (None, None, None);
        };

        let lat_val = exif_data
            .get_field(Tag::GPSLatitude, In::PRIMARY)
            .or_else(|| exif_data.fields().find(|f| f.tag == Tag::GPSLatitude))
            .and_then(|f| match &f.value {
                Value::Rational(v) => Some(v.to_vec()),
                _ => None,
            });
        let lat_ref = exif_data
            .get_field(Tag::GPSLatitudeRef, In::PRIMARY)
            .or_else(|| exif_data.fields().find(|f| f.tag == Tag::GPSLatitudeRef))
            .map(|f| f.display_value().to_string());
        let lon_val = exif_data
            .get_field(Tag::GPSLongitude, In::PRIMARY)
            .or_else(|| exif_data.fields().find(|f| f.tag == Tag::GPSLongitude))
            .and_then(|f| match &f.value {
                Value::Rational(v) => Some(v.to_vec()),
                _ => None,
            });
        let lon_ref = exif_data
            .get_field(Tag::GPSLongitudeRef, In::PRIMARY)
            .or_else(|| exif_data.fields().find(|f| f.tag == Tag::GPSLongitudeRef))
            .map(|f| f.display_value().to_string());

        let (gps_lat, gps_lon) = if let (Some(lat_v), Some(lat_r), Some(lon_v), Some(lon_r)) =
            (lat_val, lat_ref, lon_val, lon_ref)
        {
            (
                Self::dms_to_decimal(&lat_v, &lat_r),
                Self::dms_to_decimal(&lon_v, &lon_r),
            )
        } else {
            (None, None)
        };

        let altitude = exif_data
            .get_field(Tag::GPSAltitude, In::PRIMARY)
            .and_then(|field| match &field.value {
                Value::Rational(v) if !v.is_empty() => Some(v[0].num as f64 / v[0].denom as f64),
                _ => None,
            });

        (gps_lat, gps_lon, altitude)
    }

    /// Converts DMS (degrees, minutes, seconds) to decimal degrees.
    fn dms_to_decimal(dms: &[exif::Rational], reference: &str) -> Option<f64> {
        if dms.len() != 3 {
            return None;
        }
        let degrees = dms[0].num as f64 / dms[0].denom as f64;
        let minutes = dms[1].num as f64 / dms[1].denom as f64;
        let seconds = dms[2].num as f64 / dms[2].denom as f64;

        let mut decimal = degrees + minutes / 60.0 + seconds / 3600.0;

        if reference.starts_with("S") || reference.starts_with("W") {
            decimal = -decimal;
        }
        Some(decimal)
    }

    /// Formats DMS coordinates as a string (e.g., "40°42'45\"N").
    // fn format_dms(dms: &[exif::Rational], reference: &str) -> String {
    //     if dms.len() < 3 {
    //         return String::new();
    //     }
    //     let degrees = dms[0].num as f64 / dms[0].denom as f64;
    //     let minutes = dms[1].num as f64 / dms[1].denom as f64;
    //     let seconds = dms[2].num as f64 / dms[2].denom as f64;
    //     format!("{:.0}°{:.0}′{:.0}″{}", degrees, minutes, seconds, reference.trim())
    // }

    /// Extracts an EXIF field as a string.
    pub fn get_exif_field(exif: &Option<exif::Exif>, tag: Tag) -> Option<String> {
        let ex = exif.as_ref()?;
        let field = ex
            .get_field(tag, In::PRIMARY)
            .or_else(|| ex.fields().find(|f| f.tag == tag))?;

        let raw = match &field.value {
            Value::Ascii(vec) => {
                let mut bytes = Vec::new();
                for line in vec {
                    let cleaned: Vec<u8> = line.iter().cloned().take_while(|&b| b != 0).collect();
                    bytes.extend(cleaned);
                }
                String::from_utf8_lossy(&bytes).into_owned()
            }
            _ => field.display_value().with_unit(exif.as_ref()?).to_string(),
        };

        let cleaned = raw
            .replace(['"', '\''], "")
            .lines()
            .map(|line| {
                let mut s = line.trim().to_string();
                while let Some(last) = s.chars().last() {
                    if last.is_ascii_punctuation() && last != ')' && last != '(' {
                        s.pop();
                    } else {
                        break;
                    }
                }
                s
            })
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join(" ");

        let final_str = cleaned.trim();
        if final_str.is_empty() {
            None
        } else {
            Some(final_str.to_string())
        }
    }

    fn find_tiff_base(data: &[u8]) -> Option<usize> {
        data.windows(4).position(|w| w == b"II\x2a\x00" || w == b"MM\x00\x2a")
    }

    fn scrape_ascii_from_tag(data: &[u8], tiff_base: Option<usize>, tag_id: u16) -> Option<String> {
        let tiff_base = tiff_base?;

        let target_le = [(tag_id & 0xFF) as u8, (tag_id >> 8) as u8, 0x02, 0x00];
        let target_be = [(tag_id >> 8) as u8, (tag_id & 0xFF) as u8, 0x00, 0x02];

        for (is_le, target) in [(true, target_le), (false, target_be)] {
            if let Some(pos) = data.windows(12).position(|w| w.starts_with(&target)) {
                let count = if is_le {
                    u32::from_le_bytes(data[pos + 4..pos + 8].try_into().ok()?)
                } else {
                    u32::from_be_bytes(data[pos + 4..pos + 8].try_into().ok()?)
                } as usize;

                if count > 1 && count < 256 {
                    let mut start = if is_le {
                        u32::from_le_bytes(data[pos + 8..pos + 12].try_into().ok()?)
                    } else {
                        u32::from_be_bytes(data[pos + 8..pos + 12].try_into().ok()?)
                    } as usize;

                    // If count <= 4, the value is stored directly in the offset field
                    if count <= 4 {
                        start = pos + 8;
                    } else {
                        // Offset is relative to TIFF header start
                        start += tiff_base;
                    }

                    if start + count <= data.len() {
                        let bytes = &data[start..start + (count - 1).min(count)]; // Skip null terminator
                        let s = String::from_utf8_lossy(bytes)
                            .trim()
                            .trim_matches('\0')
                            .trim()
                            .to_string();
                        if !s.is_empty()
                            && s.chars().all(|c| c.is_ascii_graphic() || c.is_whitespace())
                        {
                            return Some(s);
                        }
                    }
                }
            }
        }
        None
    }

    /// Helper to scrape U16 values (like Orientation) from raw bytes
    fn scrape_u16_from_tag(data: &[u8], tag_id: u16) -> Option<u16> {
        let target_le = [(tag_id & 0xFF) as u8, (tag_id >> 8) as u8, 0x03, 0x00];
        let target_be = [(tag_id >> 8) as u8, (tag_id & 0xFF) as u8, 0x00, 0x03];

        for (is_le, target) in [(true, target_le), (false, target_be)] {
            if let Some(pos) = data.windows(12).position(|w| w.starts_with(&target)) {
                let val = if is_le {
                    u16::from_le_bytes(data[pos + 8..pos + 10].try_into().ok()?)
                } else {
                    u16::from_be_bytes(data[pos + 8..pos + 10].try_into().ok()?)
                };
                return Some(val);
            }
        }
        None
    }

    /// insert a file into db
    fn insert(&self) -> Result<usize, String> {
        let conn = open_conn()?;
        let result = conn.execute(
            "INSERT INTO afiles (
                folder_id, 
                name, name_pinyin, size, file_type, format_label, created_at, modified_at, inode,
                taken_date,
                width, height, duration,
                is_favorite, rating, rotate, comments, has_tags,
                e_make, e_model, e_date_time, e_software, e_artist, e_copyright, e_description, e_lens_make, e_lens_model, e_exposure_bias, e_exposure_time, e_f_number, e_focal_length, e_iso_speed, e_flash, e_orientation,
                gps_latitude, gps_longitude, gps_altitude, geo_name, geo_admin1, geo_admin2, geo_cc,
                last_scan_time, content_identifier, media_subtype, motion_photo_offset
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24, ?25, ?26, ?27, ?28, ?29, ?30, ?31, ?32, ?33, ?34, ?35, ?36, ?37, ?38, ?39, ?40, ?41, ?42, ?43, ?44, ?45)
            ON CONFLICT(folder_id, name) DO NOTHING",
            params![
                self.folder_id,

                self.name,
                self.name_pinyin,
                self.size,
                self.file_type,
                self.format_label,
                self.created_at,
                self.modified_at,
                self.inode,

                self.taken_date,

                self.width,
                self.height,
                self.duration,

                self.is_favorite,
                self.rating,
                self.rotate,
                self.comments,
                self.has_tags,

                self.e_make,
                self.e_model,
                self.e_date_time,
                self.e_software,
                self.e_artist,
                self.e_copyright,
                self.e_description,
                self.e_lens_make,
                self.e_lens_model,
                self.e_exposure_bias,
                self.e_exposure_time,
                self.e_f_number,
                self.e_focal_length,
                self.e_iso_speed,
                self.e_flash,
                self.e_orientation,

                self.gps_latitude,
                self.gps_longitude,
                self.gps_altitude,
                self.geo_name,
                self.geo_admin1,
                self.geo_admin2,
                self.geo_cc,
                self.last_scan_time,
                self.content_identifier,
                self.media_subtype,
                self.motion_photo_offset,
            ]
        ).map_err(|e| e.to_string())?;
        Ok(result)
    }

    /// update a file into db
    pub fn update(file_id: i64, file: &Self) -> Result<usize, String> {
        let conn = open_conn()?;
        let result = conn.execute(
            "UPDATE afiles SET
                name = ?1, name_pinyin = ?2, size = ?3, file_type = ?4, format_label = ?5, created_at = ?6, modified_at = ?7, inode = ?8,
                taken_date = ?9,
                width = ?10, height = ?11, duration = ?12,
                rating = ?13,
                e_make = ?14, e_model = ?15, e_date_time = ?16, e_software = ?17, e_artist = ?18, e_copyright = ?19, e_description = ?20, e_lens_make = ?21, e_lens_model = ?22, e_exposure_bias = ?23, e_exposure_time = ?24, e_f_number = ?25, e_focal_length = ?26, e_iso_speed = ?27, e_flash = ?28, e_orientation = ?29,
                gps_latitude = ?30, gps_longitude = ?31, gps_altitude = ?32, geo_name = ?33, geo_admin1 = ?34, geo_admin2 = ?35, geo_cc = ?36,
                last_scan_time = ?37, content_identifier = ?38, media_subtype = ?39, motion_photo_offset = ?40
            WHERE id = ?41",
            params![
                file.name,
                file.name_pinyin,
                file.size,
                file.file_type,
                file.format_label,
                file.created_at,
                file.modified_at,
                file.inode,

                file.taken_date,

                file.width,
                file.height,
                file.duration,

                file.rating,
                file.e_make,
                file.e_model,
                file.e_date_time,
                file.e_software,
                file.e_artist,
                file.e_copyright,
                file.e_description,
                file.e_lens_make,
                file.e_lens_model,
                file.e_exposure_bias,
                file.e_exposure_time,
                file.e_f_number,
                file.e_focal_length,
                file.e_iso_speed,
                file.e_flash,
                file.e_orientation,

                file.gps_latitude,
                file.gps_longitude,
                file.gps_altitude,
                file.geo_name,
                file.geo_admin1,
                file.geo_admin2,
                file.geo_cc,
                file.last_scan_time,
                file.content_identifier,
                file.media_subtype,
                file.motion_photo_offset,
                file_id,
            ]
        ).map_err(|e| e.to_string())?;
        Ok(result)
    }

    // delete a file from db
    pub fn delete(id: i64) -> Result<usize, String> {
        let mut conn = open_conn()?;
        let tx = conn.transaction().map_err(|e| e.to_string())?;
        tx.execute("DELETE FROM athumbs WHERE file_id = ?1", params![id])
            .map_err(|e| e.to_string())?;
        let result = tx
            .execute("DELETE FROM afiles WHERE id = ?1", params![id])
            .map_err(|e| e.to_string())?;
        tx.commit().map_err(|e| e.to_string())?;
        Ok(result)
    }

    pub fn batch_delete(ids: &[i64]) -> Result<usize, String> {
        if ids.is_empty() {
            return Ok(0);
        }

        let mut conn = open_conn()?;
        let tx = conn.transaction().map_err(|e| e.to_string())?;
        let mut deleted = 0;
        {
            let mut thumb_stmt = tx
                .prepare_cached("DELETE FROM athumbs WHERE file_id = ?1")
                .map_err(|e| e.to_string())?;
            let mut file_stmt = tx
                .prepare_cached("DELETE FROM afiles WHERE id = ?1")
                .map_err(|e| e.to_string())?;
            for id in ids {
                thumb_stmt.execute(params![id]).map_err(|e| e.to_string())?;
                deleted += file_stmt.execute(params![id]).map_err(|e| e.to_string())?;
            }
        }
        tx.commit().map_err(|e| e.to_string())?;
        Ok(deleted)
    }

    pub fn update_moved_file_group(
        file_id: i64,
        component_file_ids: &[i64],
        replaced_file_ids: &[i64],
        new_folder_id: i64,
    ) -> Result<usize, String> {
        let mut conn = open_conn()?;
        let tx = conn.transaction().map_err(|e| e.to_string())?;
        let mut updated = 0usize;

        {
            let mut thumb_delete_stmt = tx
                .prepare_cached("DELETE FROM athumbs WHERE file_id = ?1")
                .map_err(|e| e.to_string())?;
            let mut file_delete_stmt = tx
                .prepare_cached("DELETE FROM afiles WHERE id = ?1")
                .map_err(|e| e.to_string())?;
            for replaced_file_id in replaced_file_ids {
                thumb_delete_stmt
                    .execute(params![replaced_file_id])
                    .map_err(|e| e.to_string())?;
                file_delete_stmt
                    .execute(params![replaced_file_id])
                    .map_err(|e| e.to_string())?;
            }
        }

        {
            let mut update_stmt = tx
                .prepare_cached("UPDATE afiles SET folder_id = ?1 WHERE id = ?2")
                .map_err(|e| e.to_string())?;
            updated += update_stmt
                .execute(params![new_folder_id, file_id])
                .map_err(|e| e.to_string())?;
            for component_file_id in component_file_ids {
                updated += update_stmt
                    .execute(params![new_folder_id, component_file_id])
                    .map_err(|e| e.to_string())?;
            }
        }

        tx.commit().map_err(|e| e.to_string())?;
        Ok(updated)
    }

    /// get all file IDs for a specific album
    /// Returns a map of file path to file ID
    // pub fn get_all_ids_in_album(album_id: i64) -> Result<HashMap<String, i64>, String> {
    //     let conn = open_conn()?;
    //     let mut stmt = conn
    //         .prepare(
    //             "SELECT a.id, b.path, a.name
    //             FROM afiles a
    //             JOIN afolders b ON a.folder_id = b.id
    //             WHERE b.album_id = ?1",
    //         )
    //         .map_err(|e| e.to_string())?;

    //     let rows = stmt
    //         .query_map(params![album_id], |row| {
    //             Ok((
    //                 row.get::<_, i64>(0)?,
    //                 row.get::<_, String>(1)?,
    //                 row.get::<_, String>(2)?,
    //             ))
    //         })
    //         .map_err(|e| e.to_string())?;

    //     let mut files = HashMap::new();
    //     for row in rows {
    //         if let Ok((id, folder_path, name)) = row {
    //             let full_path = t_utils::get_file_path(&folder_path, &name);
    //             files.insert(full_path, id);
    //         }
    //     }
    //     Ok(files)
    // }

    // Helper function to build the count SQL query
    fn build_count_query() -> String {
        let base_query = "SELECT COUNT(*), SUM(a.size)
            FROM afiles a 
            LEFT JOIN afolders b ON a.folder_id = b.id
            LEFT JOIN albums c ON b.album_id = c.id";

        base_query.to_string()
    }

    fn live_photo_companion_exclusion_condition() -> String {
        format!("a.id NOT IN (SELECT primary_file.live_photo_video_id FROM afiles primary_file
            WHERE primary_file.live_photo_video_id IS NOT NULL AND {})", Self::album_scan_predicate("primary_file"))
    }

    // build the base SQL query
    fn build_base_query() -> String {
        format!(
            "SELECT a.id, a.folder_id, 
                a.name, a.name_pinyin, a.size, a.file_type, a.format_label, a.created_at, a.modified_at, a.inode,
                a.taken_date,
                a.width, a.height, a.duration,
                a.is_favorite, a.rating, a.culling_flag, a.rotate, a.comments, a.has_tags,
                a.e_make, a.e_model, a.e_date_time, a.e_software, a.e_artist, a.e_copyright, a.e_description, a.e_lens_make, a.e_lens_model, a.e_exposure_bias, a.e_exposure_time, a.e_f_number, a.e_focal_length, a.e_iso_speed, a.e_flash, a.e_orientation,
                a.gps_latitude, a.gps_longitude, a.gps_altitude, a.geo_name, a.geo_admin1, a.geo_admin2, a.geo_cc,
                b.path,
                c.id AS album_id, c.name AS album_name,
                (SELECT 1 FROM athumbs t WHERE t.file_id = a.id LIMIT 1) AS has_thumbnail,
                (SELECT 1 FROM acollections_files cf WHERE cf.file_id = a.id LIMIT 1) AS has_collections,
                CASE WHEN a.embeds IS NOT NULL THEN 1 ELSE 0 END AS has_embedding,
                a.has_faces,
                a.last_scan_time,
                a.content_identifier,
                a.media_subtype,
                a.live_photo_video_id,
                CASE
                    WHEN lpv.id IS NOT NULL AND lpf.path IS NOT NULL
                    THEN lpf.path || '/' || lpv.name
                    ELSE NULL
                END AS live_photo_video_path,
                a.motion_photo_offset,
                {} AS album_visible
            FROM afiles a
            LEFT JOIN afolders b ON a.folder_id = b.id
            LEFT JOIN albums c ON b.album_id = c.id
            LEFT JOIN afiles lpv ON a.live_photo_video_id = lpv.id
            LEFT JOIN afolders lpf ON lpv.folder_id = lpf.id",
            Self::album_joined_filter_predicate("a", "b", "c")
        )
    }

    // Function to construct `Self` from a database row
    fn from_row(row: &rusqlite::Row) -> Result<Self, rusqlite::Error> {
        Ok(Self {
            id: Some(row.get(0)?),
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
            culling_flag: row.get(16)?,
            rotate: row.get(17)?,
            comments: row.get(18)?,
            has_tags: row.get(19)?,

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

            gps_latitude: row.get(36)?,
            gps_longitude: row.get(37)?,
            gps_altitude: row.get(38)?,
            geo_name: row.get(39)?,
            geo_admin1: row.get(40)?,
            geo_admin2: row.get(41)?,
            geo_cc: row.get(42)?,

            file_path: Some(t_utils::get_file_path(
                row.get::<_, String>(43)?.as_str(),
                row.get::<_, String>(2)?.as_str(),
            )),
            album_id: row.get(44)?,
            album_name: row.get(45)?,
            has_thumbnail: row.get::<_, Option<i64>>(46)?.map(|v| v == 1),
            has_collections: row.get::<_, Option<i64>>(47)?.map(|v| v == 1),
            has_embedding: row.get::<_, Option<i64>>(48)?.map(|v| v == 1),
            has_faces: row.get::<_, Option<i32>>(49)?,
            last_scan_time: row.get(50)?,
            content_identifier: row.get(51)?,
            media_subtype: row.get(52)?,
            live_photo_video_id: row.get(53)?,
            live_photo_video_path: row.get(54)?,
            motion_photo_offset: row.get(55)?,
            album_visible: row.get(56)?,
        })
    }

    // query the count and sum by sql
    fn query_count_and_sum(
        sql: &str,
        params: &[&dyn rusqlite::ToSql],
    ) -> Result<(i64, i64), String> {
        let conn = open_conn()?;
        let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;

        let result = stmt
            .query_row(params, |row| {
                let count: i64 = row.get(0)?;
                let sum: i64 = row.get(1).unwrap_or(0); // Handles NULL from SUM
                Ok((count, sum))
            })
            .map_err(|e| e.to_string())?;

        Ok(result)
    }

    /// query files by sql
    fn query_files(sql: &str, params: &[&dyn rusqlite::ToSql]) -> Result<Vec<Self>, String> {
        let conn = open_conn()?;

        let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;

        let rows = stmt
            .query_map(params, Self::from_row)
            .map_err(|e| e.to_string())?;

        let mut files = Vec::new();
        for file in rows {
            files.push(file.map_err(|e| e.to_string())?);
        }

        Ok(files)
    }

    fn query_file_ids(sql: &str, params: &[&dyn rusqlite::ToSql]) -> Result<Vec<i64>, String> {
        let conn = open_conn()?;
        let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params, |row| row.get::<_, i64>(0))
            .map_err(|e| e.to_string())?;

        let mut ids = Vec::new();
        for id in rows {
            ids.push(id.map_err(|e| e.to_string())?);
        }
        Ok(ids)
    }

    fn append_condition(where_clause: String, condition: &str) -> String {
        if where_clause.trim().is_empty() {
            format!(" WHERE {}", condition)
        } else {
            format!("{} AND {}", where_clause, condition)
        }
    }

    pub fn get_collection_count_and_sum(
        collection_id: i64,
        params: &QueryParams,
    ) -> Result<(i64, i64), String> {
        let (joins, where_clause, sql_params) = Self::build_search_query_parts(params);
        let where_clause = Self::append_condition(where_clause, "cf.collection_id = ?");
        let collection_join = " INNER JOIN acollections_files cf ON a.id = cf.file_id";

        let sql = if params.person_id > 0 {
            format!(
                "SELECT COUNT(*), SUM(size) FROM (SELECT a.id, a.size FROM afiles a
                LEFT JOIN afolders b ON a.folder_id = b.id
                LEFT JOIN albums c ON b.album_id = c.id
                {}{}{} GROUP BY a.id)",
                collection_join, joins, where_clause
            )
        } else {
            format!(
                "{}{}{}{}",
                Self::build_count_query(),
                collection_join,
                joins,
                where_clause
            )
        };

        let mut final_params: Vec<&dyn ToSql> = sql_params.iter().map(|p| p.as_ref()).collect();
        final_params.push(&collection_id);
        Self::query_count_and_sum(&sql, &final_params)
    }

    pub fn get_collection_files(
        collection_id: i64,
        params: &QueryParams,
        offset: i64,
        limit: i64,
    ) -> Result<Vec<Self>, String> {
        let (joins, where_clause, sql_params) = Self::build_search_query_parts(params);
        let where_clause = Self::append_condition(where_clause, "cf.collection_id = ?");

        let mut query = Self::build_base_query();
        query.push_str(" INNER JOIN acollections_files cf ON a.id = cf.file_id");
        query.push_str(&joins);
        query.push_str(&where_clause);

        if params.person_id > 0 {
            query.push_str(" GROUP BY a.id");
        }

        query.push_str(&format!(" ORDER BY {}", Self::build_order_clause(params)));
        query.push_str(" LIMIT ? OFFSET ?");

        let mut final_params: Vec<&dyn ToSql> = sql_params.iter().map(|p| p.as_ref()).collect();
        final_params.push(&collection_id);
        final_params.push(&limit);
        final_params.push(&offset);
        Self::query_files(&query, &final_params)
    }

    fn query_collection_groups(
        collection_id: i64,
        params: &QueryParams,
    ) -> Result<Vec<QueryGroup>, String> {
        let Some((group_id_expr, sort_expr)) =
            Self::group_key_and_sort_expr(params.group_by, params.calendar_sort)
        else {
            return Ok(Vec::new());
        };
        let (joins, where_clause, sql_params) = Self::build_search_query_parts(params);
        let where_clause = Self::append_condition(where_clause, "cf.collection_id = ?");
        let sql = format!(
            "SELECT group_id, group_id AS label, COUNT(*), COALESCE(SUM(size), 0)
             FROM (
                SELECT DISTINCT a.id, a.size, {group_id_expr} AS group_id, {sort_expr} AS group_sort
                FROM afiles a
                LEFT JOIN afolders b ON a.folder_id = b.id
                LEFT JOIN albums c ON b.album_id = c.id
                INNER JOIN acollections_files cf ON a.id = cf.file_id
                {joins}{where_clause}
             )
             GROUP BY group_id
             ORDER BY {}",
            Self::group_order_clause(params)
        );
        let mut final_params: Vec<&dyn ToSql> = sql_params.iter().map(|p| p.as_ref()).collect();
        final_params.push(&collection_id);
        let conn = open_conn()?;
        let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(&final_params[..], |row| {
                Ok(QueryGroup {
                    id: row.get(0)?,
                    label: row.get(1)?,
                    count: row.get(2)?,
                    size: row.get(3)?,
                })
            })
            .map_err(|e| e.to_string())?;

        let mut groups = Vec::new();
        for group in rows {
            groups.push(group.map_err(|e| e.to_string())?);
        }
        Ok(groups)
    }

    fn get_collection_files_in_group(
        collection_id: i64,
        params: &QueryParams,
        group_id: &str,
        offset: i64,
        limit: i64,
    ) -> Result<Vec<Self>, String> {
        if limit <= 0 {
            return Ok(Vec::new());
        }
        let Some((group_id_expr, _)) =
            Self::group_key_and_sort_expr(params.group_by, params.calendar_sort)
        else {
            return Ok(Vec::new());
        };
        let (joins, where_clause, sql_params) = Self::build_search_query_parts(params);
        let where_clause = Self::append_condition(where_clause, "cf.collection_id = ?");

        let mut query = Self::build_base_query();
        query.push_str(" INNER JOIN acollections_files cf ON a.id = cf.file_id");
        query.push_str(&joins);
        query.push_str(&where_clause);
        query.push_str(&format!(" AND {} = ?", group_id_expr));
        query.push_str(" GROUP BY a.id");
        query.push_str(&format!(" ORDER BY {}", Self::build_order_clause(params)));
        query.push_str(" LIMIT ? OFFSET ?");

        let group_id = group_id.to_string();
        let mut final_params: Vec<&dyn ToSql> = sql_params.iter().map(|p| p.as_ref()).collect();
        final_params.push(&collection_id);
        final_params.push(&group_id);
        final_params.push(&limit);
        final_params.push(&offset);
        Self::query_files(&query, &final_params)
    }

    pub fn get_collection_grouped_query_rows(
        collection_id: i64,
        params: &QueryParams,
        offset: i64,
        limit: i64,
    ) -> Result<GroupedQueryResult, String> {
        let groups = Self::query_collection_groups(collection_id, params)?;
        Self::build_grouped_query_result(
            groups,
            offset,
            limit,
            |group, group_file_offset, group_file_limit| {
                Self::get_collection_files_in_group(
                    collection_id,
                    params,
                    &group.id,
                    group_file_offset,
                    group_file_limit,
                )
            },
        )
    }

    pub fn get_collection_group_file_ids(
        collection_id: i64,
        params: &QueryParams,
        group_id: &str,
    ) -> Result<Vec<i64>, String> {
        let Some((group_id_expr, _)) =
            Self::group_key_and_sort_expr(params.group_by, params.calendar_sort)
        else {
            return Ok(Vec::new());
        };
        let (joins, where_clause, sql_params) = Self::build_search_query_parts(params);
        let where_clause = Self::append_condition(where_clause, "cf.collection_id = ?");
        let query = format!(
            "SELECT a.id
             FROM afiles a
             LEFT JOIN afolders b ON a.folder_id = b.id
             LEFT JOIN albums c ON b.album_id = c.id
             INNER JOIN acollections_files cf ON a.id = cf.file_id
             {joins}{where_clause} AND {group_id_expr} = ?
             GROUP BY a.id
             ORDER BY {}",
            Self::build_order_clause(params)
        );
        let group_id = group_id.to_string();
        let mut final_params: Vec<&dyn ToSql> = sql_params.iter().map(|p| p.as_ref()).collect();
        final_params.push(&collection_id);
        final_params.push(&group_id);
        Self::query_file_ids(&query, &final_params)
    }

    pub fn get_collection_query_file_ids(
        collection_id: i64,
        params: &QueryParams,
    ) -> Result<Vec<i64>, String> {
        let (joins, where_clause, sql_params) = Self::build_search_query_parts(params);
        let where_clause = Self::append_condition(where_clause, "cf.collection_id = ?");
        let query = format!(
            "SELECT a.id
             FROM afiles a
             LEFT JOIN afolders b ON a.folder_id = b.id
             LEFT JOIN albums c ON b.album_id = c.id
             INNER JOIN acollections_files cf ON a.id = cf.file_id
             {joins}{where_clause}
             GROUP BY a.id
             ORDER BY {}",
            Self::build_order_clause(params)
        );
        let mut final_params: Vec<&dyn ToSql> = sql_params.iter().map(|p| p.as_ref()).collect();
        final_params.push(&collection_id);
        Self::query_file_ids(&query, &final_params)
    }

    /// fetch a file info from db by folder_id and file name
    pub fn fetch(folder_id: i64, file_path: &str) -> Result<Option<Self>, String> {
        let conn = open_conn()?;
        Self::fetch_with_conn(&conn, folder_id, file_path)
    }

    pub fn fetch_with_conn(
        conn: &Connection,
        folder_id: i64,
        file_path: &str,
    ) -> Result<Option<Self>, String> {
        let sql = format!(
            "{} WHERE a.folder_id = ?1 AND a.name = ?2",
            Self::build_base_query()
        );
        conn.query_row(
            &sql,
            params![folder_id, t_utils::get_file_name(file_path)],
            Self::from_row,
        )
        .optional()
        .map_err(|e| e.to_string())
    }

    fn build_file_type_condition(mask: i64) -> Option<String> {
        if mask <= 0 {
            return None;
        }

        let mut conditions = Vec::new();
        if mask & 1 == 1 {
            conditions.push("a.file_type = 1".to_string());
        }
        if mask & 2 == 2 {
            conditions.push("a.file_type = 2".to_string());
        }
        if mask & 4 == 4 {
            conditions.push("a.file_type = 3".to_string());
        }

        if conditions.is_empty() || conditions.len() == 3 {
            None
        } else {
            Some(format!("({})", conditions.join(" OR ")))
        }
    }

    /// The scan and all query surfaces share the same exclusion rules. Retained
    /// excluded rows are protected from sweep so restoring a rule restores metadata.
    fn album_scan_predicate(alias: &str) -> String {
        Self::album_query_predicate(alias)
    }

    fn album_media_predicate(file: &str, album: &str) -> String {
        format!("(({album}.file_types & CASE {file}.file_type WHEN 1 THEN 1 WHEN 2 THEN 2 WHEN 3 THEN 4 ELSE 0 END) != 0
            AND ({file}.file_type = 2 OR IFNULL({file}.width, 0) <= 0 OR IFNULL({file}.height, 0) <= 0
                OR {file}.width >= COALESCE({album}.small_image_filter, 0)
                OR {file}.height >= COALESCE({album}.small_image_filter, 0)))")
    }

    fn album_folder_excluded_predicate(folder: &str, album: &str) -> String {
        format!("EXISTS (SELECT 1 FROM json_each({album}.excluded_folders) excluded
            WHERE replace({folder}.path, char(92), '/') = rtrim(replace({album}.path, char(92), '/'), '/') || '/' || excluded.value
            OR substr(replace({folder}.path, char(92), '/'), 1,
                length(rtrim(replace({album}.path, char(92), '/'), '/') || '/' || excluded.value || '/'))
                = rtrim(replace({album}.path, char(92), '/'), '/') || '/' || excluded.value || '/')")
    }

    // Reuse the base query's existing joins for output visibility.
    fn album_joined_filter_predicate(file: &str, folder: &str, album: &str) -> String {
        format!("({album}.id IS NOT NULL AND {} AND NOT ({}))",
            Self::album_media_predicate(file, album),
            Self::album_folder_excluded_predicate(folder, album))
    }

    // Single-folder queries should not materialize exclusions across the library.
    fn album_filter_predicate(alias: &str) -> String {
        format!("EXISTS (SELECT 1 FROM afolders scope_folder JOIN albums scope_album
            ON scope_album.id = scope_folder.album_id WHERE scope_folder.id = {alias}.folder_id AND {})",
            Self::album_joined_filter_predicate(alias, "scope_folder", "scope_album"))
    }

    fn album_query_predicate(alias: &str) -> String {
        // Materialize excluded folder IDs once per bulk query, not per file.
        format!("(EXISTS (SELECT 1 FROM afolders scope_folder JOIN albums scope_album
            ON scope_album.id = scope_folder.album_id WHERE scope_folder.id = {alias}.folder_id AND {})
            AND {alias}.folder_id NOT IN (
                SELECT excluded_folder.id FROM afolders excluded_folder JOIN albums excluded_album
                ON excluded_album.id = excluded_folder.album_id
                WHERE excluded_album.excluded_folders != '[]' AND {}))",
            Self::album_media_predicate(alias, "scope_album"),
            Self::album_folder_excluded_predicate("excluded_folder", "excluded_album"))
    }

    pub fn is_album_visible(file_id: i64) -> Result<bool, String> {
        Self::is_album_visible_on(&*open_conn()?, file_id)
    }

    fn is_album_visible_on(conn: &Connection, file_id: i64) -> Result<bool, String> {
        conn.query_row(&format!("SELECT EXISTS (SELECT 1 FROM afiles a
            JOIN afolders b ON b.id=a.folder_id JOIN albums c ON c.id=b.album_id
            WHERE a.id=?1 AND {})", Self::album_joined_filter_predicate("a", "b", "c")),
            params![file_id], |row| row.get(0)).map_err(|e| e.to_string())
    }

    fn append_album_filter(conditions: &mut Vec<String>) {
        conditions.push(Self::album_query_predicate("a"));
    }

    fn album_filter_sql(alias: &str) -> String {
        format!(" AND {}", Self::album_query_predicate(alias))
    }

    /// insert a file into db if not exists
    /// Returns (file, status)
    /// status: 0 - existing, 1 - new, 2 - updated
    pub fn add_to_db(
        folder_id: i64,
        file_path: &str,
        file_type: i64,
        last_scan_time: i64,
    ) -> Result<(Self, i32), String> {
        Self::add_to_db_with_raw_info(folder_id, file_path, file_type, last_scan_time, None)
    }

    pub fn add_to_db_with_raw_info(
        folder_id: i64, file_path: &str, file_type: i64, last_scan_time: i64,
        raw_info: Option<t_libraw::RawInfo>,
    ) -> Result<(Self, i32), String> {
        // Check if the file exists
        let existing_file = Self::fetch(folder_id, file_path)?;
        if let Some(mut file) = existing_file {
            // check file modified time or if thumbnail is missing
            let file_info = t_utils::FileInfo::new(file_path)?;
            let modified = file.modified_at != file_info.modified;
            let missing_thumb = !file.has_thumbnail.unwrap_or(false);
            let needs_tiff_dimension_refresh = t_libraw::is_tiff_path(file_path)
                && file.e_orientation.unwrap_or(1) > 4
                && t_image::get_image_dimensions(file_path).is_ok_and(|(width, height)| {
                    file.width != Some(width) || file.height != Some(height)
                });
            // JPEGs indexed before Motion Photo support have no detection marker.
            // Recheck those files once on their next album scan.
            let needs_motion_photo_detection = file_type == 1
                && t_image::is_jpeg_path(file_path)
                && file.motion_photo_offset.is_none();

            if needs_motion_photo_detection
                && !modified
                && !missing_thumb
                && !needs_tiff_dimension_refresh
            {
                Self::refresh_motion_photo_detection(&mut file, file_path, last_scan_time)?;
                return Ok((file, 2));
            }

            if modified || missing_thumb || needs_tiff_dimension_refresh {
                if let Some(file_id) = file.id {
                    if let Some(mut updated_file) =
                        Self::update_file_info(file_id, file_path, last_scan_time)?
                    {
                        // If modified, delete old thumbnail and remove embeds data
                        if modified || missing_thumb {
                            let _ = AThumb::delete(file_id);
                            // remove embeds data
                            if modified {
                                let conn = open_conn()?;
                                let _ = conn.execute(
                                    "UPDATE afiles SET embeds = NULL WHERE id = ?1",
                                    params![file_id],
                                );
                                updated_file.has_embedding = Some(false);
                            }
                        }
                        return Ok((updated_file, 2));
                    }
                } else {
                    return Err(format!(
                        "Existing DB record is missing file id, skipping '{}'",
                        file_path
                    ));
                }
            } else {
                // Not modified and thumb exists, but we still need to update last_scan_time
                // for the mark-and-sweep deletion logic.
                if let Some(file_id) = file.id {
                    let _ = Self::update_column(file_id, "last_scan_time", &last_scan_time);
                    if file
                        .comments
                        .as_deref()
                        .is_none_or(|comment| comment.trim().is_empty())
                    {
                        if let Some(comment) = t_ai_png::extract_comment(file_path) {
                            let _ = Self::update_column(file_id, "comments", &comment);
                            file.comments = Some(comment);
                        }
                    }
                }
            }
            return Ok((file, 0));
        }

        // insert the new file into the database
        let mut new_file_struct = Self::new_with_raw_info(folder_id, file_path, file_type, raw_info)?;
        new_file_struct.last_scan_time = Some(last_scan_time);
        let inserted = new_file_struct.insert()?;

        // A concurrent folder sync or album scan may have inserted the same
        // path after the SELECT above. Re-enter the existing-file path so the
        // winning row is marked as seen by this scan and receives any required
        // metadata or thumbnail refresh.
        if inserted == 0 {
            return Self::add_to_db(folder_id, file_path, file_type, last_scan_time);
        }

        let new_file = Self::fetch(folder_id, file_path)?;
        new_file
            .map(|f| (f, 1))
            .ok_or_else(|| format!("Inserted file missing from DB: {}", file_path))
    }

    /// get a file info from db by file_id
    pub fn get_file_info(file_id: i64) -> Result<Option<Self>, String> {
        let conn = open_conn()?;

        // Prepare the SQL query using the base query and adding the condition for file ID
        let sql = format!("{} WHERE a.id = ?1", Self::build_base_query());

        // Execute the query with file_id as the parameter
        let result = conn
            .query_row(&sql, params![file_id], Self::from_row)
            .optional()
            .map_err(|e| e.to_string())?;

        Ok(result)
    }

    pub fn get_files_by_ids(file_ids: &[i64]) -> Result<Vec<Self>, String> {
        let conn = open_conn()?;
        let valid_ids = file_ids
            .iter()
            .copied()
            .filter(|id| *id > 0)
            .collect::<Vec<_>>();
        let mut files = Vec::with_capacity(valid_ids.len());

        for ids in valid_ids.chunks(500) {
            let placeholders = std::iter::repeat_n("?", ids.len())
                .collect::<Vec<_>>()
                .join(",");
            let sql = format!(
                "{} WHERE a.id IN ({}){}",
                Self::build_base_query(),
                placeholders,
                match t_utils::inaccessible_album_ids().as_slice() {
                    [] => String::new(),
                    ids => format!(
                        " AND b.album_id NOT IN ({})",
                        ids.iter().map(i64::to_string).collect::<Vec<_>>().join(",")
                    ),
                }
            );
            let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
            let rows = stmt
                .query_map(rusqlite::params_from_iter(ids.iter()), Self::from_row)
                .map_err(|e| e.to_string())?;
            for row in rows {
                files.push(row.map_err(|e| e.to_string())?);
            }
        }

        Ok(files)
    }

    /// update a file info
    pub fn update_file_info(
        file_id: i64,
        file_path: &str,
        last_scan_time: i64,
    ) -> Result<Option<Self>, String> {
        // get old file info
        let old_file_info =
            Self::get_file_info(file_id)?.ok_or_else(|| "File not found".to_string())?;

        // create a new file info
        let mut new_file_info = Self::new(
            old_file_info.folder_id,
            file_path,
            old_file_info.file_type.unwrap_or(0),
        )?;
        new_file_info.id = Some(file_id);
        new_file_info.is_favorite = old_file_info.is_favorite;
        new_file_info.rating = old_file_info.rating;
        new_file_info.rotate = old_file_info.rotate;
        if old_file_info
            .comments
            .as_deref()
            .is_some_and(|comment| !comment.trim().is_empty())
        {
            new_file_info.comments = old_file_info.comments;
        }
        new_file_info.has_tags = old_file_info.has_tags;
        new_file_info.has_thumbnail = old_file_info.has_thumbnail;
        new_file_info.has_embedding = old_file_info.has_embedding;
        // Live Photo / RAW+JPEG pairing is derived from the scan-time pairing
        // pass, not from re-extracting metadata, so retain it when this refresh
        // does not identify an embedded Motion Photo.
        if new_file_info.media_subtype.is_none()
            && matches!(
                old_file_info.media_subtype.as_deref(),
                Some("live_photo" | "raw_jpeg_pair")
            )
        {
            new_file_info.media_subtype = old_file_info.media_subtype.clone();
            new_file_info.live_photo_video_id = old_file_info.live_photo_video_id;
        }
        new_file_info.last_scan_time = Some(last_scan_time);

        // update the file info
        Self::update(file_id, &new_file_info)?;

        Self::get_file_info(file_id)
    }

    /// Backfill Motion Photo metadata without re-extracting all file metadata.
    fn refresh_motion_photo_detection(
        file: &mut Self,
        file_path: &str,
        last_scan_time: i64,
    ) -> Result<(), String> {
        let file_id = file
            .id
            .ok_or_else(|| "File is missing an id".to_string())?;
        if let Some(offset) = crate::t_motion_photo::detect_motion_photo(Path::new(file_path)) {
            file.media_subtype = Some("motion_photo".to_string());
            file.motion_photo_offset = Some(offset as i64);
        } else {
            // Preserve sidecar-derived pairings; they are maintained by the
            // dedicated Live Photo / RAW+JPEG pairing pass.
            if !matches!(
                file.media_subtype.as_deref(),
                Some("live_photo" | "raw_jpeg_pair")
            ) {
                file.media_subtype = None;
            }
            file.motion_photo_offset = Some(0);
        }
        file.last_scan_time = Some(last_scan_time);

        open_conn()?
            .execute(
                "UPDATE afiles
                 SET media_subtype = ?1, motion_photo_offset = ?2, last_scan_time = ?3
                 WHERE id = ?4",
                params![
                    file.media_subtype,
                    file.motion_photo_offset,
                    file.last_scan_time,
                    file_id,
                ],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    /// update a file column value
    pub fn update_column(
        file_id: i64,
        column: &str,
        value: &dyn rusqlite::ToSql,
    ) -> Result<usize, String> {
        let conn = open_conn()?;
        Self::update_column_with_conn(&conn, file_id, column, value)
    }

    pub fn update_column_with_conn(
        conn: &Connection,
        file_id: i64,
        column: &str,
        value: &dyn rusqlite::ToSql,
    ) -> Result<usize, String> {
        let query = format!("UPDATE afiles SET {} = ?1 WHERE id = ?2", column);
        conn.execute(&query, params![value, file_id])
            .map_err(|e| e.to_string())
    }

    pub fn batch_update_names(updates: &[(i64, String, Option<String>)]) -> Result<usize, String> {
        if updates.is_empty() {
            return Ok(0);
        }

        let mut conn = open_conn()?;
        let tx = conn.transaction().map_err(|e| e.to_string())?;
        let mut changed = 0usize;
        for (file_id, name, name_pinyin) in updates {
            changed += tx
                .execute(
                    "UPDATE afiles SET name = ?1, name_pinyin = ?2 WHERE id = ?3",
                    params![name, name_pinyin, file_id],
                )
                .map_err(|e| e.to_string())?;
        }
        tx.commit().map_err(|e| e.to_string())?;
        Ok(changed)
    }

    pub fn batch_update_metadata(
        file_ids: &[i64],
        is_favorite: Option<bool>,
        rating: Option<i32>,
        culling_flag: Option<i32>,
        rotate_delta: Option<i32>,
        comment: Option<&str>,
    ) -> Result<usize, String> {
        if file_ids.is_empty() {
            return Ok(0);
        }

        let mut conn = open_conn()?;
        let tx = conn.transaction().map_err(|e| e.to_string())?;
        let mut updated = 0;

        if let Some(value) = is_favorite {
            let mut stmt = tx
                .prepare_cached("UPDATE afiles SET is_favorite = ?1 WHERE id = ?2")
                .map_err(|e| e.to_string())?;
            for file_id in file_ids {
                updated += stmt
                    .execute(params![value, file_id])
                    .map_err(|e| e.to_string())?;
            }
        }
        if let Some(value) = rating {
            let clamped = value.clamp(0, 5);
            let mut stmt = tx
                .prepare_cached("UPDATE afiles SET rating = ?1 WHERE id = ?2")
                .map_err(|e| e.to_string())?;
            for file_id in file_ids {
                updated += stmt
                    .execute(params![clamped, file_id])
                    .map_err(|e| e.to_string())?;
            }
        }
        if let Some(value) = culling_flag {
            let clamped = value.clamp(0, 2);
            let mut stmt = tx
                .prepare_cached("UPDATE afiles SET culling_flag = ?1 WHERE id = ?2")
                .map_err(|e| e.to_string())?;
            for file_id in file_ids {
                updated += stmt
                    .execute(params![clamped, file_id])
                    .map_err(|e| e.to_string())?;
            }
        }
        if let Some(value) = rotate_delta {
            let mut stmt = tx
                .prepare_cached(
                    "UPDATE afiles
                     SET rotate = ((COALESCE(rotate, 0) + ?1) % 360 + 360) % 360
                     WHERE id = ?2",
                )
                .map_err(|e| e.to_string())?;
            for file_id in file_ids {
                updated += stmt
                    .execute(params![value, file_id])
                    .map_err(|e| e.to_string())?;
            }
        }
        if let Some(value) = comment {
            let mut stmt = tx
                .prepare_cached("UPDATE afiles SET comments = ?1 WHERE id = ?2")
                .map_err(|e| e.to_string())?;
            for file_id in file_ids {
                updated += stmt
                    .execute(params![value, file_id])
                    .map_err(|e| e.to_string())?;
            }
        }

        tx.commit().map_err(|e| e.to_string())?;
        Ok(updated)
    }

    pub fn pair_live_photos_in_folder(
        folder_id: i64,
        affected_names: &HashSet<String>,
        refresh_missing_identifiers: bool,
    ) -> Result<usize, String> {
        if let Some(folder) = AFolder::get_by_id(folder_id)? {
            let album = Album::get_album_by_id(folder.album_id)?;
            if album.excludes_path(Path::new(&folder.path)) { return Ok(0); }
            if album.file_types & 3 != 3 { return Self::clear_live_photo_pairs_in_folder(folder_id); }
        }
        #[derive(Clone)]
        struct Candidate {
            id: i64,
            name: String,
            file_type: i64,
            content_identifier: Option<String>,
            media_subtype: Option<String>,
            live_photo_video_id: Option<i64>,
        }

        fn lower_ext(name: &str) -> String {
            Path::new(name)
                .extension()
                .and_then(|ext| ext.to_str())
                .unwrap_or("")
                .to_ascii_lowercase()
        }

        fn lower_stem(name: &str) -> Option<String> {
            Path::new(name)
                .file_stem()
                .and_then(|stem| stem.to_str())
                .map(|stem| stem.to_ascii_lowercase())
        }

        fn video_match_stems(name: &str) -> Vec<String> {
            let Some(stem) = lower_stem(name) else {
                return Vec::new();
            };
            let mut stems = vec![stem.clone()];
            if let Some(stripped) = stem.strip_suffix("_hevc") {
                if !stripped.is_empty() {
                    stems.push(stripped.to_string());
                }
            }
            stems
        }

        fn is_live_photo_image(candidate: &Candidate) -> bool {
            if candidate.file_type != 1 {
                return false;
            }
            matches!(
                lower_ext(&candidate.name).as_str(),
                "heic" | "heif" | "hif" | "jpg" | "jpeg"
            )
        }

        fn is_live_photo_video(candidate: &Candidate) -> bool {
            if candidate.file_type != 2 || lower_ext(&candidate.name) != "mov" {
                return false;
            }
            has_content_identifier(candidate)
        }

        fn has_filename_video_candidate(
            candidate: &Candidate,
            video_stems: &HashSet<String>,
        ) -> bool {
            is_live_photo_image(candidate)
                && lower_stem(&candidate.name)
                    .map(|stem| video_stems.contains(&stem))
                    .unwrap_or(false)
        }

        fn has_content_identifier(candidate: &Candidate) -> bool {
            content_identifier_key(candidate).is_some()
        }

        fn content_identifier_key(candidate: &Candidate) -> Option<String> {
            let identifier = candidate
                .content_identifier
                .as_ref()
                .map(|identifier| identifier.trim())
                .filter(|identifier| !identifier.is_empty())?;
            Some(identifier.to_ascii_lowercase())
        }

        if affected_names.is_empty() {
            return Ok(0);
        }

        // Restrict pairing to the filename stems touched by this sync. Include
        // both the normal and _HEVC MOV forms for every affected stem.
        let mut affected_stems = HashSet::new();
        for name in affected_names {
            let Some(stem) = lower_stem(name) else {
                continue;
            };
            affected_stems.insert(stem.clone());
            if let Some(stripped) = stem.strip_suffix("_hevc") {
                if !stripped.is_empty() {
                    affected_stems.insert(stripped.to_string());
                }
            }
        }
        if affected_stems.is_empty() {
            return Ok(0);
        }

        let mut candidate_names = HashSet::new();
        for stem in affected_stems {
            for extension in ["heic", "heif", "hif", "jpg", "jpeg", "mov"] {
                candidate_names.insert(format!("{}.{}", stem, extension));
            }
            candidate_names.insert(format!("{}_hevc.mov", stem));
        }

        let conn = open_conn()?;
        {
            let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;
            tx.execute_batch(
                "CREATE TEMP TABLE IF NOT EXISTS live_photo_candidate_names (
                    name TEXT PRIMARY KEY
                );
                DELETE FROM live_photo_candidate_names;",
            )
            .map_err(|e| e.to_string())?;
            let mut stmt = tx
                .prepare_cached(
                    "INSERT OR IGNORE INTO live_photo_candidate_names (name) VALUES (?1)",
                )
                .map_err(|e| e.to_string())?;
            for name in candidate_names {
                stmt.execute(params![name]).map_err(|e| e.to_string())?;
            }
            drop(stmt);
            tx.commit().map_err(|e| e.to_string())?;
        }

        let folder_path: String = conn
            .query_row(
                "SELECT path FROM afolders WHERE id = ?1",
                params![folder_id],
                |row| row.get(0),
            )
            .map_err(|e| e.to_string())?;

        let candidates = {
            let mut stmt = conn
                .prepare(
                    &format!("SELECT a.id, a.name, a.file_type, a.content_identifier,
                            a.media_subtype, a.live_photo_video_id
                     FROM afiles a
                     JOIN live_photo_candidate_names candidates
                       ON a.name = candidates.name COLLATE NOCASE
                     WHERE a.folder_id = ?1 AND {}", Self::album_filter_predicate("a")),
                )
                .map_err(|e| e.to_string())?;
            let rows = stmt
                .query_map(params![folder_id], |row| {
                    Ok(Candidate {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        file_type: row.get::<_, Option<i64>>(2)?.unwrap_or(0),
                        content_identifier: row.get(3)?,
                        media_subtype: row.get(4)?,
                        live_photo_video_id: row.get(5)?,
                    })
                })
                .map_err(|e| e.to_string())?;

            rows.collect::<Result<Vec<_>, _>>()
                .map_err(|e| e.to_string())?
        };

        let mut video_stems = HashSet::<String>::new();
        for candidate in &candidates {
            if candidate.file_type == 2 && lower_ext(&candidate.name) == "mov" {
                for stem in video_match_stems(&candidate.name) {
                    video_stems.insert(stem);
                }
            }
        }

        let mut image_stems = HashSet::<String>::new();
        for candidate in &candidates {
            if is_live_photo_image(candidate) {
                if let Some(stem) = lower_stem(&candidate.name) {
                    image_stems.insert(stem);
                }
            }
        }

        let mut candidates = candidates;
        let mut identifier_updates = Vec::<(i64, String)>::new();
        for candidate in candidates.iter_mut().filter(|candidate| {
            candidate.file_type == 2
                && lower_ext(&candidate.name) == "mov"
                && video_match_stems(&candidate.name)
                    .iter()
                    .any(|stem| image_stems.contains(stem))
        }) {
            if candidate.content_identifier.is_none()
                || (refresh_missing_identifiers && !has_content_identifier(candidate))
            {
                let file_path = PathBuf::from(&folder_path).join(&candidate.name);
                let file_path = file_path.to_string_lossy();
                let identifier = t_video::get_video_metadata(&file_path)
                    .ok()
                    .and_then(|metadata| metadata.content_identifier)
                    .unwrap_or_default();
                identifier_updates.push((candidate.id, identifier.clone()));
                candidate.content_identifier = Some(identifier);
            }
        }

        let mut video_identifiers_by_stem = HashMap::<String, HashSet<String>>::new();
        for video in &candidates {
            if !is_live_photo_video(video) {
                continue;
            }
            let Some(identifier) = content_identifier_key(video) else {
                continue;
            };
            for stem in video_match_stems(&video.name) {
                video_identifiers_by_stem
                    .entry(stem)
                    .or_default()
                    .insert(identifier.clone());
            }
        }

        for image in candidates
            .iter_mut()
            .filter(|candidate| has_filename_video_candidate(candidate, &video_stems))
        {
            let Some(stem) = lower_stem(&image.name) else {
                continue;
            };
            let Some(video_identifiers) = video_identifiers_by_stem.get(&stem) else {
                continue;
            };
            if content_identifier_key(image)
                .is_some_and(|identifier| video_identifiers.contains(&identifier))
            {
                continue;
            }

            let file_path = PathBuf::from(&folder_path).join(&image.name);
            let file_path = file_path.to_string_lossy();
            let identifier = crate::t_apple_sidecar::scan_apple_content_identifiers(&file_path)
                .into_iter()
                .map(|identifier| identifier.to_ascii_lowercase())
                .find(|identifier| video_identifiers.contains(identifier))
                .unwrap_or_default();
            if image.content_identifier.as_deref() != Some(identifier.as_str()) {
                identifier_updates.push((image.id, identifier.clone()));
                image.content_identifier = Some(identifier);
            }
        }

        // Build filename candidates first. Content Identifier only confirms a
        // candidate pair; it must never select a MOV from another filename.
        let mut videos_by_stem = HashMap::<String, Vec<usize>>::new();
        for (index, video) in candidates.iter().enumerate() {
            if is_live_photo_video(video) {
                for stem in video_match_stems(&video.name) {
                    videos_by_stem.entry(stem).or_default().push(index);
                }
            }
        }

        // A copied Live Photo keeps its Content Identifier. Consume each MOV
        // once so copied filename pairs remain independent resources.
        let mut desired_pairs = HashMap::<i64, i64>::new();
        let mut used_video_ids = HashSet::<i64>::new();
        for image in candidates.iter().filter(|candidate| {
            has_filename_video_candidate(candidate, &video_stems)
                && has_content_identifier(candidate)
        }) {
            let Some(image_stem) = lower_stem(&image.name) else {
                continue;
            };
            let Some(image_identifier) = content_identifier_key(image) else {
                continue;
            };
            let matching_video = videos_by_stem
                .get(&image_stem)
                .into_iter()
                .flatten()
                .map(|index| &candidates[*index])
                .filter(|video| {
                    !used_video_ids.contains(&video.id)
                        && content_identifier_key(video).as_deref()
                            == Some(image_identifier.as_str())
                })
                .min_by_key(|video| {
                    (
                        lower_stem(&video.name).as_deref() != Some(image_stem.as_str()),
                        video.id,
                    )
                });
            if let Some(video) = matching_video {
                used_video_ids.insert(video.id);
                desired_pairs.insert(image.id, video.id);
            }
        }

        let changed_candidates = candidates
            .iter()
            .filter(|candidate| is_live_photo_image(candidate))
            .filter_map(|candidate| {
                let desired_video_id = desired_pairs.get(&candidate.id).copied();
                let is_current = match desired_video_id {
                    Some(video_id) => {
                        candidate.live_photo_video_id == Some(video_id)
                            && candidate.media_subtype.as_deref() == Some("live_photo")
                    }
                    None => {
                        candidate.live_photo_video_id.is_none()
                            && candidate.media_subtype.as_deref() != Some("live_photo")
                    }
                };
                (!is_current).then_some((candidate.id, desired_video_id))
            })
            .collect::<Vec<_>>();

        if changed_candidates.is_empty() && identifier_updates.is_empty() {
            return Ok(0);
        }

        let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;
        {
            let mut update_identifier = tx
                .prepare_cached("UPDATE afiles SET content_identifier = ?1 WHERE id = ?2")
                .map_err(|e| e.to_string())?;
            for (file_id, identifier) in &identifier_updates {
                update_identifier
                    .execute(params![identifier, file_id])
                    .map_err(|e| e.to_string())?;
            }
        }
        for (image_id, video_id) in &changed_candidates {
            match video_id {
                Some(video_id) => tx
                    .execute(
                        "UPDATE afiles
                         SET media_subtype = 'live_photo', live_photo_video_id = ?1
                         WHERE id = ?2",
                        params![video_id, image_id],
                    )
                    .map_err(|e| e.to_string())?,
                None => tx
                    .execute(
                        "UPDATE afiles
                         SET media_subtype = NULL, live_photo_video_id = NULL
                         WHERE id = ?1",
                        params![image_id],
                    )
                    .map_err(|e| e.to_string())?,
            };
        }
        tx.commit().map_err(|e| e.to_string())?;
        Ok(changed_candidates.len())
    }

    /// Returns whether a folder has a MOV candidate and any existing Live Photo
    /// associations. This lets a full album scan avoid loading every file in
    /// ordinary folders solely to decide whether pairing is needed.
    pub fn live_photo_folder_state(folder_id: i64) -> Result<(bool, bool), String> {
        let conn = open_conn()?;
        conn.query_row(
            "SELECT
                EXISTS(
                    SELECT 1 FROM afiles
                    WHERE folder_id = ?1
                      AND name LIKE '%.mov' COLLATE NOCASE
                ),
                EXISTS(
                    SELECT 1 FROM afiles
                    WHERE folder_id = ?1
                      AND media_subtype = 'live_photo'
                )",
            params![folder_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|error| error.to_string())
    }

    /// Removes stale image-to-video associations after a companion MOV has
    /// disappeared from a folder.
    pub fn clear_live_photo_pairs_in_folder(folder_id: i64) -> Result<usize, String> {
        let conn = open_conn()?;
        conn.execute(
            "UPDATE afiles
             SET media_subtype = NULL, live_photo_video_id = NULL
             WHERE folder_id = ?1 AND media_subtype = 'live_photo'",
            params![folder_id],
        )
        .map_err(|error| error.to_string())
    }

    pub fn live_photo_component_files(file_id: i64) -> Result<Vec<Self>, String> {
        let Some(file) = Self::get_file_info(file_id)? else {
            return Ok(Vec::new());
        };
        let Some(video_id) = file.live_photo_video_id.filter(|id| *id > 0) else {
            return Ok(Vec::new());
        };
        Self::get_files_by_ids(&[video_id])
    }

    pub fn pair_raw_jpeg_in_folder(folder_id: i64) -> Result<usize, String> {
        if let Some(folder) = AFolder::get_by_id(folder_id)? {
            let album = Album::get_album_by_id(folder.album_id)?;
            if album.excludes_path(Path::new(&folder.path)) { return Ok(0); }
            if album.file_types & 5 != 5 { return Self::clear_raw_jpeg_pairs_in_folder(folder_id); }
        }
        fn stem(name: &str) -> Option<String> { Path::new(name).file_stem()?.to_str().map(|value| value.to_ascii_lowercase()) }
        fn ext(name: &str) -> String { Path::new(name).extension().and_then(|value| value.to_str()).unwrap_or("").to_ascii_lowercase() }
        let files = Self::get_files_by_folder_id(folder_id)?;
        let mut companions = HashMap::<String, Vec<i64>>::new();
        let mut raws = HashMap::<String, usize>::new();
        for file in &files {
            if !file.album_visible { continue; }
            if matches!(ext(&file.name).as_str(), "jpg" | "jpeg" | "heic" | "heif" | "hif") {
                if let (Some(id), Some(file_stem)) = (file.id, stem(&file.name)) { companions.entry(file_stem).or_default().push(id); }
            }
            if t_common::RAW_IMGS.iter().any(|raw| raw.eq_ignore_ascii_case(&ext(&file.name))) {
                if let Some(file_stem) = stem(&file.name) {
                    *raws.entry(file_stem).or_default() += 1;
                }
            }
        }
        let mut updates = Vec::new();
        for file in &files {
            if !t_common::RAW_IMGS.iter().any(|raw| raw.eq_ignore_ascii_case(&ext(&file.name))) { continue; }
            let (Some(id), Some(file_stem)) = (file.id, stem(&file.name)) else { continue; };
            let candidates = companions.get(&file_stem).cloned().unwrap_or_default();
            let desired = (file.album_visible && raws.get(&file_stem) == Some(&1) && candidates.len() == 1)
                .then(|| candidates.first().copied())
                .flatten();
            let current = (file.media_subtype.as_deref() == Some("raw_jpeg_pair")).then_some(file.live_photo_video_id).flatten();
            if desired != current { updates.push((id, desired)); }
        }
        if updates.is_empty() { return Ok(0); }
        let mut conn = open_conn()?; let tx = conn.transaction().map_err(|error| error.to_string())?;
        for (id, companion) in &updates {
            if let Some(companion) = companion { tx.execute("UPDATE afiles SET media_subtype = 'raw_jpeg_pair', live_photo_video_id = ?1 WHERE id = ?2", params![companion, id]).map_err(|error| error.to_string())?; }
            else { tx.execute("UPDATE afiles SET media_subtype = NULL, live_photo_video_id = NULL WHERE id = ?1 AND media_subtype = 'raw_jpeg_pair'", params![id]).map_err(|error| error.to_string())?; }
        }
        tx.commit().map_err(|error| error.to_string())?; Ok(updates.len())
    }

    pub fn clear_raw_jpeg_pairs_in_folder(folder_id: i64) -> Result<usize, String> {
        open_conn()?.execute(
            "UPDATE afiles
             SET media_subtype = NULL, live_photo_video_id = NULL
             WHERE folder_id = ?1 AND media_subtype = 'raw_jpeg_pair'",
            params![folder_id],
        ).map_err(|error| error.to_string())
    }

    pub fn clear_raw_jpeg_pairs_in_album(album_id: i64) -> Result<usize, String> {
        open_conn()?.execute(
            "UPDATE afiles
             SET media_subtype = NULL, live_photo_video_id = NULL
             WHERE media_subtype = 'raw_jpeg_pair'
               AND folder_id IN (SELECT id FROM afolders WHERE album_id = ?1)",
            params![album_id],
        ).map_err(|error| error.to_string())
    }

    pub fn retain_excluded_image(album_id: i64, path: &str, width: u32, height: u32) -> Result<(), String> {
        let path = Path::new(path);
        let parent = path.parent().ok_or("Image has no parent folder")?.to_string_lossy();
        let name = path.file_name().ok_or("Image has no filename")?.to_string_lossy();
        open_conn()?.execute("UPDATE afiles SET width=?1, height=?2
            WHERE name=?3 AND folder_id IN (SELECT id FROM afolders WHERE album_id=?4 AND path=?5)
            AND (width IS NOT ?1 OR height IS NOT ?2)",
            params![width, height, name.as_ref(), album_id, parent.as_ref()]).map_err(|e| e.to_string())?;
        Ok(())
    }

    /// delete unseen files in an album (database only)
    pub fn delete_unseen_in_album(album_id: i64, current_scan_time: i64) -> Result<usize, String> {
        let mut conn = open_conn()?;
        let tx = conn.transaction().map_err(|e| e.to_string())?;
        let query = format!("DELETE FROM afiles AS a
            WHERE last_scan_time < ?1
            AND folder_id IN (SELECT id FROM afolders WHERE album_id = ?2)
            AND {}", Self::album_scan_predicate("a"));
        let result = tx
            .execute(&query, params![current_scan_time, album_id])
            .map_err(|e| e.to_string())?;
        tx.commit().map_err(|e| e.to_string())?;
        Ok(result)
    }

    /// Update last_scan_time for files identified by folder_path and file name in an album
    pub fn touch_last_scan_times(
        album_id: i64,
        entries: &[(String, String)],
        current_scan_time: i64,
    ) -> Result<usize, String> {
        if entries.is_empty() {
            return Ok(0);
        }
        let mut conn = open_conn()?;
        let tx = conn.transaction().map_err(|e| e.to_string())?;
        let touched = Self::touch_last_scan_times_on(&tx, album_id, entries, current_scan_time)?;
        tx.commit().map_err(|e| e.to_string())?;
        Ok(touched)
    }

    /// Update last_scan_time for entries on an existing connection/transaction
    pub fn touch_last_scan_times_on(
        conn: &Connection,
        album_id: i64,
        entries: &[(String, String)],
        current_scan_time: i64,
    ) -> Result<usize, String> {
        if entries.is_empty() {
            return Ok(0);
        }
        let mut touched = 0;
        let mut folder_stmt = conn
            .prepare_cached("SELECT id FROM afolders WHERE album_id = ?1 AND path = ?2")
            .map_err(|e| e.to_string())?;
        let mut update_stmt = conn
            .prepare_cached(
                "UPDATE afiles SET last_scan_time = ?1 WHERE folder_id = ?2 AND name = ?3",
            )
            .map_err(|e| e.to_string())?;

        let mut cached_folder: Option<(String, i64)> = None;

        for (folder_path, file_name) in entries {
            let folder_id = if let Some((ref path, id)) = cached_folder {
                if path == folder_path {
                    id
                } else {
                    let id: Option<i64> = folder_stmt
                        .query_row(params![album_id, folder_path], |row| row.get(0))
                        .optional()
                        .map_err(|e| e.to_string())?;
                    if let Some(id) = id {
                        cached_folder = Some((folder_path.clone(), id));
                        id
                    } else {
                        continue;
                    }
                }
            } else {
                let id: Option<i64> = folder_stmt
                    .query_row(params![album_id, folder_path], |row| row.get(0))
                    .optional()
                    .map_err(|e| e.to_string())?;
                if let Some(id) = id {
                    cached_folder = Some((folder_path.clone(), id));
                    id
                } else {
                    continue;
                }
            };

            let updated = update_stmt
                .execute(params![current_scan_time, folder_id, file_name])
                .map_err(|e| e.to_string())?;
            touched += updated;
        }
        Ok(touched)
    }

    /// Get a file's has_tags status
    pub fn get_has_tags(file_id: i64) -> Result<bool, String> {
        let conn = open_conn()?;
        let result = conn
            .query_row(
                "SELECT has_tags FROM afiles WHERE id = ?1",
                params![file_id],
                |row| row.get(0),
            )
            .map_err(|e| e.to_string())?;
        Ok(result)
    }

    /// get all taken dates from db
    pub fn get_taken_dates(sort: i64) -> Result<Vec<(String, i64)>, String> {
        let conn = open_conn()?;

        // sort encodes both the date column and direction:
        //   sort / 2  →  0=taken_date, 1=created_at, 2=modified_at
        //   sort % 2  →  0=ASC, 1=DESC
        let sort_type = sort / 2;
        let order_clause = if sort % 2 == 0 { "ASC" } else { "DESC" };

        let date_col = match sort_type {
            0 => "a.taken_date",
            1 => "a.created_at",
            2 => "a.modified_at",
            _ => "a.taken_date",
        };

        let date_expr = format!(
            "strftime('%Y-%m-%d', {}, 'unixepoch', 'localtime')",
            date_col
        );
        let query = format!(
            "SELECT {} AS group_date, COUNT(1)
            FROM afiles a
            JOIN afolders b ON a.folder_id = b.id
            WHERE {} IS NOT NULL AND {} >= 86400 AND {}{}{} AND {}
            GROUP BY {}
            ORDER BY group_date {}",
            date_expr,
            date_col,
            date_col,
            Self::live_photo_companion_exclusion_condition(),
            Self::inaccessible_album_filter("b"),
            Self::album_filter_sql("a"),
            Self::search_exclusion_condition("b"),
            date_expr,
            order_clause
        );

        let mut stmt = conn
            .prepare(&query)
            .map_err(|e| format!("Failed to prepare query: {}", e))?;

        // Use collect() to simplify result collection
        let results: Vec<(String, i64)> = stmt
            .query_map(params![], |row| Ok((row.get(0)?, row.get(1)?)))
            .map_err(|e| format!("Query execution failed: {}", e))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Failed to process rows: {}", e))?;

        Ok(results)
    }

    // helper to build search query conditions and params
    // Returns (joins_clause, where_clause, params)
    fn build_search_query_parts(params: &QueryParams) -> (String, String, Vec<Box<dyn ToSql>>) {
        let mut joins = Vec::new();
        let mut conditions: Vec<String> =
            vec![Self::live_photo_companion_exclusion_condition().to_string()];
        let mut sql_params: Vec<Box<dyn ToSql>> = Vec::new();

        let inaccessible_album_ids = t_utils::inaccessible_album_ids();
        if !inaccessible_album_ids.is_empty() {
            conditions.push(format!(
                "b.album_id NOT IN ({})",
                inaccessible_album_ids
                    .iter()
                    .map(i64::to_string)
                    .collect::<Vec<_>>()
                    .join(",")
            ));
        }

        if !params.search_file_name.is_empty() {
            conditions.push("(a.name LIKE ? COLLATE NOCASE OR a.comments LIKE ? COLLATE NOCASE)".to_string());
            let pattern = format!("%{}%", params.search_file_name);
            sql_params.push(Box::new(pattern.clone()));
            sql_params.push(Box::new(pattern));
        }

        if let Some(condition) = Self::build_file_type_condition(params.search_file_type) {
            conditions.push(condition);
        }

        Self::append_album_filter(&mut conditions);

        if !params.search_all_subfolders.is_empty() {
            // Match path that starts with search_folder followed by '/' or end of string
            conditions.push("(b.path = ? OR b.path LIKE ? ESCAPE '\\')".to_string());
            sql_params.push(Box::new(params.search_all_subfolders.clone()));
            sql_params.push(Box::new(subtree_like_pattern(&params.search_all_subfolders)));
        }

        if !params.search_folder.is_empty() {
            conditions.push("(b.path = ?)".to_string());
            sql_params.push(Box::new(params.search_folder.clone()));
        }

        if params.start_date > 0 && params.end_date > 0 {
            let date_col = match params.calendar_sort / 2 {
                0 => "a.taken_date",
                1 => "a.created_at",
                2 => "a.modified_at",
                _ => "a.taken_date",
            };
            conditions.push(format!("{} >= ? AND {} < ?", date_col, date_col));
            sql_params.push(Box::new(params.start_date));
            sql_params.push(Box::new(params.end_date));
        } else if params.start_date == -1 && params.end_date == -1 {
            // "On This Day" feature: find all photos taken on the same month and day as today
            let now = chrono::Local::now();
            let today_month_day = now.format("%m-%d").to_string();
            conditions
                .push("strftime('%m-%d', a.taken_date, 'unixepoch', 'localtime') = ?".to_string());
            sql_params.push(Box::new(today_month_day));
        }

        if !params.make.is_empty() {
            conditions.push("UPPER(a.e_make) = UPPER(?)".to_string());
            sql_params.push(Box::new(params.make.clone()));
            if !params.model.is_empty() {
                conditions.push("a.e_model = ?".to_string());
                sql_params.push(Box::new(params.model.clone()));
            }
        }

        if !params.lens_make.is_empty() {
            conditions.push("UPPER(a.e_lens_make) = UPPER(?)".to_string());
            sql_params.push(Box::new(params.lens_make.clone()));
            if !params.lens_model.is_empty() {
                conditions.push("a.e_lens_model = ?".to_string());
                sql_params.push(Box::new(params.lens_model.clone()));
            }
        }

        if !params.location_admin1.is_empty() {
            conditions.push("a.geo_admin1 = ?".to_string());
            sql_params.push(Box::new(params.location_admin1.clone()));
            if !params.location_name.is_empty() {
                conditions.push("a.geo_name = ?".to_string());
                sql_params.push(Box::new(params.location_name.clone()));
            }
        }

        if let (Some(min_lat), Some(max_lat), Some(min_lon), Some(max_lon)) = (
            params.gps_min_lat,
            params.gps_max_lat,
            params.gps_min_lon,
            params.gps_max_lon,
        ) {
            conditions.push("a.gps_latitude BETWEEN ? AND ?".to_string());
            sql_params.push(Box::new(min_lat));
            sql_params.push(Box::new(max_lat));

            if min_lon <= max_lon {
                conditions.push("a.gps_longitude BETWEEN ? AND ?".to_string());
                sql_params.push(Box::new(min_lon));
                sql_params.push(Box::new(max_lon));
            } else {
                // map view crosses the antimeridian (e.g. min=170, max=-170)
                conditions.push("(a.gps_longitude >= ? OR a.gps_longitude <= ?)".to_string());
                sql_params.push(Box::new(min_lon));
                sql_params.push(Box::new(max_lon));
            }
        }

        if params.is_favorite {
            conditions.push("a.is_favorite = 1".to_string());
        }

        if params.rating == -2 {
            conditions.push("a.rating > 0".to_string());
        } else if params.rating == 0 {
            conditions.push("(a.rating = 0 OR a.rating IS NULL)".to_string());
        } else if params.rating > 0 {
            conditions.push("a.rating = ?".to_string());
            sql_params.push(Box::new(params.rating));
        }

        if (0..=2).contains(&params.culling_flag) {
            conditions.push("COALESCE(a.culling_flag, 0) = ?".to_string());
            sql_params.push(Box::new(params.culling_flag));
        }

        if params.tag_group_id > 0 {
            conditions.push("EXISTS (SELECT 1 FROM afile_tags ft JOIN atags t ON t.id = ft.tag_id WHERE ft.file_id = a.id AND t.group_id = ?)".to_string());
            sql_params.push(Box::new(params.tag_group_id));
        }
        if params.tag_id > 0 {
            joins.push("INNER JOIN afile_tags at ON a.id = at.file_id");
            conditions.push("at.tag_id = ?".to_string());
            sql_params.push(Box::new(params.tag_id));
        }

        if params.person_id > 0 {
            joins.push("INNER JOIN faces f ON a.id = f.file_id");
            conditions.push("f.person_id = ?".to_string());
            sql_params.push(Box::new(params.person_id));
        }

        conditions.push(Self::search_exclusion_condition("b"));

        let joins_clause = if !joins.is_empty() {
            format!(" {}", joins.join(" "))
        } else {
            String::new()
        };

        let where_clause = if !conditions.is_empty() {
            format!(" WHERE {}", conditions.join(" AND "))
        } else {
            String::new()
        };

        (joins_clause, where_clause, sql_params)
    }

    // get query count and sum
    pub fn get_query_count_and_sum(params: &QueryParams) -> Result<(i64, i64), String> {
        let (joins, where_clause, sql_params) = Self::build_search_query_parts(params);

        let sql = if params.person_id > 0 {
            // Use subquery with GROUP BY to handle potential duplicate rows when joining faces
            format!(
                "SELECT COUNT(*), SUM(size) FROM (SELECT a.id, a.size FROM afiles a
                LEFT JOIN afolders b ON a.folder_id = b.id
                LEFT JOIN albums c ON b.album_id = c.id
                {}{} GROUP BY a.id)",
                joins, where_clause
            )
        } else {
            format!("{}{}{}", Self::build_count_query(), joins, where_clause)
        };

        let final_params: Vec<&dyn ToSql> = sql_params.iter().map(|p| p.as_ref()).collect();
        Self::query_count_and_sum(&sql, &final_params)
    }

    // get query files
    pub fn get_query_files(
        params: &QueryParams,
        offset: i64,
        limit: i64,
    ) -> Result<Vec<Self>, String> {
        let (joins, where_clause, sql_params) = Self::build_search_query_parts(params);

        let mut query = Self::build_base_query();
        query.push_str(&joins);
        query.push_str(&where_clause);

        // fix issues that some files have multiple identical person_ids
        if params.person_id > 0 {
            query.push_str(" GROUP BY a.id");
        }

        // sort
        query.push_str(&format!(" ORDER BY {}", Self::build_order_clause(params)));

        // A non-positive limit returns the complete result set. Virtualized views
        // use positive limits; unified search needs all text matches.
        query.push_str(" LIMIT ? OFFSET ?");
        let resolved_limit = if limit > 0 { limit } else { -1 };

        let mut final_params: Vec<&dyn ToSql> = sql_params.iter().map(|p| p.as_ref()).collect();
        final_params.push(&resolved_limit);
        final_params.push(&offset);
        Self::query_files(&query, &final_params)
    }

    fn group_key_and_sort_expr(group_by: i64, calendar_sort: i64) -> Option<(String, String)> {
        let date_col = match calendar_sort / 2 {
            1 => "a.created_at",
            2 => "a.modified_at",
            _ => "a.taken_date",
        };
        match group_by {
            GROUP_BY_FOLDER_PATH => {
                Some((
                    "COALESCE(b.path, 'unknown-folder')".to_string(),
                    "COALESCE(b.modified_at, b.created_at, 0)".to_string(),
                ))
            }
            GROUP_BY_DATE_DAY => {
                let day_bucket = format!("strftime('%s', date({date_col}, 'unixepoch', 'localtime'), 'utc')");
                Some((
                    format!("CASE WHEN {date_col} IS NULL THEN 'unknown-date' ELSE CAST({day_bucket} AS TEXT) END"),
                    "0".to_string(),
                ))
            }
            GROUP_BY_DATE_MONTH => {
                let month_bucket = format!("strftime('%s', date({date_col}, 'unixepoch', 'localtime', 'start of month'), 'utc')");
                Some((
                    format!("CASE WHEN {date_col} IS NULL THEN 'unknown-month' ELSE CAST({month_bucket} AS TEXT) END"),
                    "0".to_string(),
                ))
            }
            GROUP_BY_DATE_YEAR => {
                let year_bucket = format!("strftime('%s', date({date_col}, 'unixepoch', 'localtime', 'start of year'), 'utc')");
                Some((
                    format!("CASE WHEN {date_col} IS NULL THEN 'unknown-year' ELSE CAST({year_bucket} AS TEXT) END"),
                    "0".to_string(),
                ))
            }
            GROUP_BY_RATING => Some((
                "CAST(COALESCE(a.rating, 0) AS TEXT)".to_string(),
                "0".to_string(),
            )),
            GROUP_BY_LOCATION => Some((
                "CASE WHEN COALESCE(a.geo_name, a.geo_admin1, a.geo_cc, '') = '' THEN 'unknown-location' ELSE COALESCE(a.geo_name, a.geo_admin1, a.geo_cc) END".to_string(),
                "COALESCE(a.geo_cc, '')".to_string(),
            )),
            GROUP_BY_CAMERA => Some((
                "CASE WHEN TRIM(COALESCE(a.e_make, '') || ' ' || COALESCE(a.e_model, '')) = '' THEN 'unknown-camera' ELSE TRIM(COALESCE(a.e_make, '') || ' ' || COALESCE(a.e_model, '')) END".to_string(),
                "COALESCE(a.e_model, '')".to_string(),
            )),
            GROUP_BY_LENS => Some((
                "CASE WHEN COALESCE(a.e_lens_model, '') = '' THEN 'unknown-lens' ELSE a.e_lens_model END".to_string(),
                "COALESCE(a.e_lens_model, '')".to_string(),
            )),
            GROUP_BY_FILE_TYPE => Some((
                "CASE a.file_type WHEN 1 THEN 'image' WHEN 3 THEN 'raw' WHEN 2 THEN 'video' ELSE 'other' END".to_string(),
                "0".to_string(),
            )),
            GROUP_BY_CULLING => Some((
                "CAST(COALESCE(a.culling_flag, 0) AS TEXT)".to_string(),
                "0".to_string(),
            )),
            _ => None,
        }
    }

    fn group_order_clause_values(
        group_by: i64,
        folder_sort: i64,
        calendar_sort: i64,
        category_sort: i64,
    ) -> String {
        match group_by {
            GROUP_BY_DATE_DAY | GROUP_BY_DATE_MONTH | GROUP_BY_DATE_YEAR => {
                let dir = if calendar_sort % 2 == 1 {
                    "DESC"
                } else {
                    "ASC"
                };
                format!(
                    "CAST(group_id AS INTEGER) {}, label COLLATE NOCASE ASC",
                    dir
                )
            }
            GROUP_BY_FOLDER_PATH => match folder_sort {
                1 => "label COLLATE NOCASE DESC".to_string(),
                2 => "MAX(group_sort) ASC, label COLLATE NOCASE ASC".to_string(),
                3 => "MAX(group_sort) DESC, label COLLATE NOCASE ASC".to_string(),
                _ => "label COLLATE NOCASE ASC".to_string(),
            },
            GROUP_BY_RATING => "CAST(group_id AS INTEGER) DESC".to_string(),
            GROUP_BY_FILE_TYPE => "CASE group_id WHEN 'image' THEN 0 WHEN 'raw' THEN 1 WHEN 'video' THEN 2 ELSE 3 END".to_string(),
            GROUP_BY_CULLING => "CASE group_id WHEN '1' THEN 0 WHEN '2' THEN 1 ELSE 2 END".to_string(),
            GROUP_BY_LOCATION | GROUP_BY_CAMERA | GROUP_BY_LENS => match category_sort {
                1 => "CASE WHEN group_id LIKE 'unknown-%' THEN 1 ELSE 0 END, MAX(group_sort) COLLATE NOCASE DESC, label COLLATE NOCASE DESC".to_string(),
                2 => "CASE WHEN group_id LIKE 'unknown-%' THEN 1 ELSE 0 END, COUNT(*) ASC, label COLLATE NOCASE ASC".to_string(),
                3 => "CASE WHEN group_id LIKE 'unknown-%' THEN 1 ELSE 0 END, COUNT(*) DESC, label COLLATE NOCASE ASC".to_string(),
                _ => "CASE WHEN group_id LIKE 'unknown-%' THEN 1 ELSE 0 END, MAX(group_sort) COLLATE NOCASE ASC, label COLLATE NOCASE ASC".to_string(),
            },
            _ => "group_id ASC, label COLLATE NOCASE ASC".to_string(),
        }
    }

    fn group_order_clause(params: &QueryParams) -> String {
        Self::group_order_clause_values(
            params.group_by,
            params.folder_sort,
            params.calendar_sort,
            params.category_sort,
        )
    }

    fn smart_group_order_clause(params: &SmartQueryParams) -> String {
        Self::group_order_clause_values(
            params.group_by,
            params.folder_sort,
            params.calendar_sort,
            params.category_sort,
        )
    }

    fn query_groups(params: &QueryParams) -> Result<Vec<QueryGroup>, String> {
        let Some((group_id_expr, sort_expr)) =
            Self::group_key_and_sort_expr(params.group_by, params.calendar_sort)
        else {
            return Ok(Vec::new());
        };
        let (joins, where_clause, sql_params) = Self::build_search_query_parts(params);
        let sql = format!(
            "SELECT group_id, group_id AS label, COUNT(*), COALESCE(SUM(size), 0)
             FROM (
                SELECT DISTINCT a.id, a.size, {group_id_expr} AS group_id, {sort_expr} AS group_sort
                FROM afiles a
                LEFT JOIN afolders b ON a.folder_id = b.id
                LEFT JOIN albums c ON b.album_id = c.id
                {joins}{where_clause}
             )
             GROUP BY group_id
             ORDER BY {}",
            Self::group_order_clause(params)
        );
        let final_params: Vec<&dyn ToSql> = sql_params.iter().map(|p| p.as_ref()).collect();
        let conn = open_conn()?;
        let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(&final_params[..], |row| {
                Ok(QueryGroup {
                    id: row.get(0)?,
                    label: row.get(1)?,
                    count: row.get(2)?,
                    size: row.get(3)?,
                })
            })
            .map_err(|e| e.to_string())?;

        let mut groups = Vec::new();
        for group in rows {
            groups.push(group.map_err(|e| e.to_string())?);
        }
        Ok(groups)
    }

    fn get_query_files_in_group(
        params: &QueryParams,
        group_id: &str,
        offset: i64,
        limit: i64,
    ) -> Result<Vec<Self>, String> {
        if limit <= 0 {
            return Ok(Vec::new());
        }
        let Some((group_id_expr, _)) =
            Self::group_key_and_sort_expr(params.group_by, params.calendar_sort)
        else {
            return Ok(Vec::new());
        };
        let (joins, where_clause, mut sql_params) = Self::build_search_query_parts(params);

        let mut query = Self::build_base_query();
        query.push_str(&joins);
        query.push_str(&where_clause);
        if where_clause.is_empty() {
            query.push_str(&format!(" WHERE {} = ?", group_id_expr));
        } else {
            query.push_str(&format!(" AND {} = ?", group_id_expr));
        }
        sql_params.push(Box::new(group_id.to_string()));

        query.push_str(" GROUP BY a.id");
        query.push_str(&format!(" ORDER BY {}", Self::build_order_clause(params)));
        query.push_str(" LIMIT ? OFFSET ?");

        let mut final_params: Vec<&dyn ToSql> = sql_params.iter().map(|p| p.as_ref()).collect();
        final_params.push(&limit);
        final_params.push(&offset);
        Self::query_files(&query, &final_params)
    }

    fn build_grouped_query_result<F>(
        groups: Vec<QueryGroup>,
        offset: i64,
        limit: i64,
        mut get_files_in_group: F,
    ) -> Result<GroupedQueryResult, String>
    where
        F: FnMut(&QueryGroup, i64, i64) -> Result<Vec<Self>, String>,
    {
        let total_item_count = groups.iter().map(|group| group.count).sum::<i64>();
        let total_size = groups.iter().map(|group| group.size).sum::<i64>();
        let total_row_count = total_item_count + groups.len() as i64;
        let mut grouped_query_groups = Vec::with_capacity(groups.len());
        let mut group_row_cursor = 0_i64;
        for group in &groups {
            grouped_query_groups.push(GroupedQueryGroup {
                group_id: group.id.clone(),
                label: group.label.clone(),
                count: group.count,
                size: group.size,
                row_index: group_row_cursor,
            });
            group_row_cursor += group.count + 1;
        }

        let start = offset.max(0);
        let end = if limit <= 0 {
            start
        } else {
            (start + limit).min(total_row_count)
        };
        let mut rows = Vec::new();
        let mut row_cursor = 0_i64;
        let mut file_index_cursor = 0_i64;

        for group in groups {
            let header_row = row_cursor;
            let file_rows_start = header_row + 1;
            let file_rows_end = file_rows_start + group.count;

            if header_row >= start && header_row < end {
                rows.push(GroupedQueryRow::Group {
                    row_id: format!("group-row-{}", group.id),
                    group_id: group.id.clone(),
                    label: group.label.clone(),
                    count: group.count,
                    size: group.size,
                });
            }

            let file_overlap_start = start.max(file_rows_start);
            let file_overlap_end = end.min(file_rows_end);
            if file_overlap_start < file_overlap_end {
                let group_file_offset = file_overlap_start - file_rows_start;
                let group_file_limit = file_overlap_end - file_overlap_start;
                let files = get_files_in_group(&group, group_file_offset, group_file_limit)?;
                for (index, file) in files.into_iter().enumerate() {
                    let file_index = file_index_cursor + group_file_offset + index as i64;
                    let file_id = file.id.unwrap_or(file_index);
                    rows.push(GroupedQueryRow::Item {
                        row_id: format!("item-row-{}", file_id),
                        group_id: group.id.clone(),
                        file_index,
                        file,
                    });
                }
            }

            row_cursor = file_rows_end;
            file_index_cursor += group.count;
            if row_cursor >= end {
                break;
            }
        }

        Ok(GroupedQueryResult {
            rows,
            groups: grouped_query_groups,
            total_item_count,
            total_row_count,
            total_size,
        })
    }

    pub fn get_grouped_query_rows(
        params: &QueryParams,
        offset: i64,
        limit: i64,
    ) -> Result<GroupedQueryResult, String> {
        let groups = Self::query_groups(params)?;
        Self::build_grouped_query_result(
            groups,
            offset,
            limit,
            |group, group_file_offset, group_file_limit| {
                Self::get_query_files_in_group(
                    params,
                    &group.id,
                    group_file_offset,
                    group_file_limit,
                )
            },
        )
    }

    pub fn get_group_file_ids(params: &QueryParams, group_id: &str) -> Result<Vec<i64>, String> {
        let Some((group_id_expr, _)) =
            Self::group_key_and_sort_expr(params.group_by, params.calendar_sort)
        else {
            return Ok(Vec::new());
        };
        let (joins, where_clause, mut sql_params) = Self::build_search_query_parts(params);
        let mut query = format!(
            "SELECT a.id
             FROM afiles a
             LEFT JOIN afolders b ON a.folder_id = b.id
             LEFT JOIN albums c ON b.album_id = c.id
             {joins}{where_clause}"
        );
        if where_clause.is_empty() {
            query.push_str(&format!(" WHERE {} = ?", group_id_expr));
        } else {
            query.push_str(&format!(" AND {} = ?", group_id_expr));
        }
        sql_params.push(Box::new(group_id.to_string()));
        query.push_str(" GROUP BY a.id");
        query.push_str(&format!(" ORDER BY {}", Self::build_order_clause(params)));

        let final_params: Vec<&dyn ToSql> = sql_params.iter().map(|p| p.as_ref()).collect();
        let conn = open_conn()?;
        let mut stmt = conn.prepare(&query).map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(&final_params[..], |row| row.get::<_, i64>(0))
            .map_err(|e| e.to_string())?;

        let mut ids = Vec::new();
        for id in rows {
            ids.push(id.map_err(|e| e.to_string())?);
        }
        Ok(ids)
    }

    /// Resolve a file's index in the flattened grouped result without loading
    /// the ID list for every preceding group.
    pub fn get_grouped_file_position(
        params: &QueryParams,
        file_id: i64,
    ) -> Result<Option<i64>, String> {
        if file_id <= 0 {
            return Ok(None);
        }
        let Some((group_id_expr, _)) =
            Self::group_key_and_sort_expr(params.group_by, params.calendar_sort)
        else {
            return Ok(None);
        };

        let (joins, where_clause, mut sql_params) = Self::build_search_query_parts(params);
        let mut query = format!(
            "SELECT {group_id_expr}
             FROM afiles a
             LEFT JOIN afolders b ON a.folder_id = b.id
             LEFT JOIN albums c ON b.album_id = c.id
             {joins}{where_clause}"
        );
        if where_clause.is_empty() {
            query.push_str(" WHERE a.id = ?");
        } else {
            query.push_str(" AND a.id = ?");
        }
        sql_params.push(Box::new(file_id));

        let final_params: Vec<&dyn ToSql> = sql_params.iter().map(|p| p.as_ref()).collect();
        let conn = open_conn()?;
        let group_id = conn
            .query_row(&query, &final_params[..], |row| row.get::<_, String>(0))
            .optional()
            .map_err(|e| e.to_string())?;
        let Some(group_id) = group_id else {
            return Ok(None);
        };

        let groups = Self::query_groups(params)?;
        let Some(group_index) = groups.iter().position(|group| group.id == group_id) else {
            return Ok(None);
        };
        let preceding_count = groups[..group_index]
            .iter()
            .map(|group| group.count)
            .sum::<i64>();
        let group_file_ids = Self::get_group_file_ids(params, &group_id)?;
        Ok(group_file_ids
            .iter()
            .position(|id| *id == file_id)
            .map(|index| preceding_count + index as i64))
    }

    pub fn get_query_file_ids(params: &QueryParams) -> Result<Vec<i64>, String> {
        let (joins, where_clause, sql_params) = Self::build_search_query_parts(params);
        let mut query = format!(
            "SELECT a.id
             FROM afiles a
             LEFT JOIN afolders b ON a.folder_id = b.id
             LEFT JOIN albums c ON b.album_id = c.id
             {joins}{where_clause}"
        );
        if params.person_id > 0 {
            query.push_str(" GROUP BY a.id");
        }
        query.push_str(&format!(" ORDER BY {}", Self::build_order_clause(params)));

        let final_params: Vec<&dyn ToSql> = sql_params.iter().map(|p| p.as_ref()).collect();
        let conn = open_conn()?;
        let mut stmt = conn.prepare(&query).map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(&final_params[..], |row| row.get::<_, i64>(0))
            .map_err(|e| e.to_string())?;

        let mut ids = Vec::new();
        for id in rows {
            ids.push(id.map_err(|e| e.to_string())?);
        }
        Ok(ids)
    }

    fn build_order_clause_values(sort_type: i64, sort_order: i64, random_seed: i64) -> String {
        let dir = if sort_order == 1 { "DESC" } else { "ASC" };
        match sort_type {
            0 => format!("a.taken_date {}, a.id {}", dir, dir),
            1 => format!("a.created_at {}, a.id {}", dir, dir),
            2 => format!("a.modified_at {}, a.id {}", dir, dir),
            3 => format!("a.name_pinyin {}, a.id {}", dir, dir),
            4 => format!("a.size {}, a.id {}", dir, dir),
            5 => format!("a.width {}, a.height {}, a.id {}", dir, dir, dir),
            6 => format!("a.duration {}, a.id {}", dir, dir),
            // A seed makes random order stable for every request in one virtualized
            // result set, while a new query can supply a different order.
            7 => {
                // SQLite has no XOR operator. `(x | y) - (x & y)` is XOR for
                // non-negative values, so the seed changes the full ordering
                // instead of merely rotating it.
                let id_hash = "((a.id * 1103515245 + 12345) & 2147483647)";
                let seed = random_seed.rem_euclid(2_147_483_647);
                format!("((({id_hash} | {seed}) - ({id_hash} & {seed}))), a.id")
            }
            9 => "a.id ASC".to_string(), // internal: stable append order during scanning
            _ => format!("a.taken_date {}, a.id {}", dir, dir),
        }
    }

    fn build_order_clause(params: &QueryParams) -> String {
        Self::build_order_clause_values(params.sort_type, params.sort_order, params.random_seed)
    }

    fn smart_rule_string(value: &JsonValue) -> Option<String> {
        value
            .as_str()
            .map(|v| v.trim().to_string())
            .filter(|v| !v.is_empty())
    }

    fn smart_rule_i64(value: &JsonValue) -> Option<i64> {
        if let Some(value) = value.as_i64() {
            return Some(value);
        }
        value.as_str().and_then(|v| v.trim().parse::<i64>().ok())
    }

    fn smart_rule_bool(value: &JsonValue) -> Option<bool> {
        if let Some(value) = value.as_bool() {
            return Some(value);
        }
        match value.as_str()?.trim().to_ascii_lowercase().as_str() {
            "true" | "yes" | "1" => Some(true),
            "false" | "no" | "0" => Some(false),
            _ => None,
        }
    }

    fn smart_rule_array(value: &JsonValue) -> Vec<JsonValue> {
        value
            .as_array()
            .cloned()
            .unwrap_or_else(|| vec![value.clone()])
    }

    fn smart_rule_string_array(value: &JsonValue) -> Vec<String> {
        Self::smart_rule_array(value)
            .iter()
            .filter_map(Self::smart_rule_string)
            .collect()
    }

    fn build_smart_extension_condition(
        value: &JsonValue,
        sql_params: &mut Vec<Box<dyn ToSql>>,
    ) -> Result<String, String> {
        let values = Self::smart_rule_string_array(value);
        if values.is_empty() {
            return Err("Extension value required".to_string());
        }
        let mut parts = Vec::new();
        for ext in values {
            let ext = ext.trim().trim_start_matches('.').to_ascii_lowercase();
            if ext.is_empty() {
                continue;
            }
            parts.push("LOWER(a.name) LIKE ?".to_string());
            sql_params.push(Box::new(format!("%.{}", ext)));
        }
        if parts.is_empty() {
            return Err("Extension value required".to_string());
        }
        Ok(format!("({})", parts.join(" OR ")))
    }

    fn build_smart_name_condition(
        operator: &str,
        value: &JsonValue,
        sql_params: &mut Vec<Box<dyn ToSql>>,
    ) -> Result<String, String> {
        let name =
            Self::smart_rule_string(value).ok_or_else(|| "Name value required".to_string())?;
        match operator {
            "contains" | "has" => {
                sql_params.push(Box::new(format!("%{}%", name)));
                Ok("a.name LIKE ? COLLATE NOCASE".to_string())
            }
            "not_contains" | "not_has" => {
                sql_params.push(Box::new(format!("%{}%", name)));
                Ok("a.name NOT LIKE ? COLLATE NOCASE".to_string())
            }
            _ => Err(format!("Unsupported name operator: {}", operator)),
        }
    }

    fn smart_rule_date_column(field: &str) -> Result<&'static str, String> {
        match field {
            "date_taken" => Ok("a.taken_date"),
            "date_created" => Ok("a.created_at"),
            "date_modified" => Ok("a.modified_at"),
            _ => Err(format!("Unsupported date field: {}", field)),
        }
    }

    fn smart_rule_date_value(value: &JsonValue, key: &str) -> Option<i64> {
        value.get(key).and_then(Self::smart_rule_i64)
    }

    fn smart_rule_relative_date_cutoff(value: &JsonValue) -> Result<i64, String> {
        let amount = value
            .get("amount")
            .and_then(Self::smart_rule_i64)
            .filter(|v| *v > 0)
            .ok_or_else(|| "Relative date amount required".to_string())?;
        let unit = value
            .get("unit")
            .and_then(JsonValue::as_str)
            .unwrap_or("day");
        let days = match unit {
            "day" | "days" => amount,
            "month" | "months" => amount * 30,
            "year" | "years" => amount * 365,
            _ => return Err(format!("Unsupported relative date unit: {}", unit)),
        };
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_secs() as i64;
        Ok(now - days * 86_400)
    }

    fn smart_rule_date_period_range(value: &JsonValue) -> Result<(i64, i64), String> {
        let period = value
            .as_str()
            .ok_or_else(|| "Date period value required".to_string())?;
        let today = chrono::Local::now().date_naive();
        let start_date = match period {
            "this_week" => {
                today - chrono::Duration::days(today.weekday().num_days_from_monday() as i64)
            }
            "this_month" => chrono::NaiveDate::from_ymd_opt(today.year(), today.month(), 1)
                .ok_or_else(|| "Invalid month date".to_string())?,
            "this_year" => chrono::NaiveDate::from_ymd_opt(today.year(), 1, 1)
                .ok_or_else(|| "Invalid year date".to_string())?,
            _ => return Err(format!("Unsupported date period: {}", period)),
        };
        let end_date = today + chrono::Duration::days(1);
        let start = chrono::Local
            .from_local_datetime(
                &start_date
                    .and_hms_opt(0, 0, 0)
                    .ok_or_else(|| "Invalid date period start".to_string())?,
            )
            .earliest()
            .ok_or_else(|| "Invalid local date period start".to_string())?
            .timestamp();
        let end = chrono::Local
            .from_local_datetime(
                &end_date
                    .and_hms_opt(0, 0, 0)
                    .ok_or_else(|| "Invalid date period end".to_string())?,
            )
            .earliest()
            .ok_or_else(|| "Invalid local date period end".to_string())?
            .timestamp();
        Ok((start, end))
    }

    fn build_smart_orientation_condition(value: &JsonValue) -> Result<String, String> {
        let orientation = Self::smart_rule_string(value)
            .ok_or_else(|| "Orientation value required".to_string())?;
        match orientation.as_str() {
            "landscape" => Ok("(a.width > a.height)".to_string()),
            "portrait" => Ok("(a.height > a.width)".to_string()),
            "square" => Ok("(a.width = a.height)".to_string()),
            _ => Err(format!("Unsupported orientation value: {}", orientation)),
        }
    }

    fn push_numeric_rule(
        conditions: &mut Vec<String>,
        sql_params: &mut Vec<Box<dyn ToSql>>,
        column: &str,
        operator: &str,
        value: &JsonValue,
    ) -> Result<(), String> {
        match operator {
            "eq" | "is" => {
                let Some(v) = Self::smart_rule_i64(value) else {
                    return Err("Numeric value required".to_string());
                };
                conditions.push(format!("{} = ?", column));
                sql_params.push(Box::new(v));
            }
            "neq" | "is_not" => {
                let Some(v) = Self::smart_rule_i64(value) else {
                    return Err("Numeric value required".to_string());
                };
                conditions.push(format!("{} != ?", column));
                sql_params.push(Box::new(v));
            }
            "gt" => {
                let Some(v) = Self::smart_rule_i64(value) else {
                    return Err("Numeric value required".to_string());
                };
                conditions.push(format!("{} > ?", column));
                sql_params.push(Box::new(v));
            }
            "gte" => {
                let Some(v) = Self::smart_rule_i64(value) else {
                    return Err("Numeric value required".to_string());
                };
                conditions.push(format!("{} >= ?", column));
                sql_params.push(Box::new(v));
            }
            "lt" => {
                let Some(v) = Self::smart_rule_i64(value) else {
                    return Err("Numeric value required".to_string());
                };
                conditions.push(format!("{} < ?", column));
                sql_params.push(Box::new(v));
            }
            "lte" => {
                let Some(v) = Self::smart_rule_i64(value) else {
                    return Err("Numeric value required".to_string());
                };
                conditions.push(format!("{} <= ?", column));
                sql_params.push(Box::new(v));
            }
            "between" => {
                let start = value
                    .get("min")
                    .or_else(|| value.get("start"))
                    .and_then(Self::smart_rule_i64)
                    .ok_or_else(|| "Range start value required".to_string())?;
                let end = value
                    .get("max")
                    .or_else(|| value.get("end"))
                    .and_then(Self::smart_rule_i64)
                    .ok_or_else(|| "Range end value required".to_string())?;
                conditions.push(format!("{} BETWEEN ? AND ?", column));
                sql_params.push(Box::new(start));
                sql_params.push(Box::new(end));
            }
            "empty" => conditions.push(format!("({} IS NULL OR {} = 0)", column, column)),
            "not_empty" => conditions.push(format!("({} IS NOT NULL AND {} > 0)", column, column)),
            _ => return Err(format!("Unsupported numeric operator: {}", operator)),
        }
        Ok(())
    }

    fn build_smart_rule_condition(
        rule: &SmartRule,
        _joins: &mut Vec<String>,
        _needs_group: &mut bool,
        sql_params: &mut Vec<Box<dyn ToSql>>,
    ) -> Result<String, String> {
        let field = rule.field.as_str();
        let operator = rule.operator.as_str();
        let value = &rule.value;

        match field {
            "name" => Self::build_smart_name_condition(operator, value, sql_params),
            "file_type" => {
                let mask = Self::smart_rule_i64(value)
                    .ok_or_else(|| "File type value required".to_string())?;
                let condition =
                    Self::build_file_type_condition(mask).unwrap_or_else(|| "1 = 1".to_string());
                Ok(if matches!(operator, "is_not" | "neq" | "not_in") {
                    format!("NOT {}", condition)
                } else {
                    condition
                })
            }
            "media_subtype" => {
                let subtype = Self::smart_rule_string(value)
                    .ok_or_else(|| "Media subtype value required".to_string())?;
                // "motion_photo" is the unified "dynamic photo" bucket that
                // covers both Apple Live Photos and Android Motion Photos;
                // "live_photo" maps to the same condition for backward
                // compatibility with previously saved rules.
                let (positive, negative) = match subtype.as_str() {
                    "live_photo" | "motion_photo" => (
                        "a.media_subtype IN ('live_photo', 'motion_photo')",
                        "(a.media_subtype IS NULL OR a.media_subtype NOT IN ('live_photo', 'motion_photo'))",
                    ),
                    "raw_jpeg_pair" => (
                        "a.media_subtype = 'raw_jpeg_pair'",
                        "(a.media_subtype IS NULL OR a.media_subtype != 'raw_jpeg_pair')",
                    ),
                    other => return Err(format!("Unsupported media subtype: {}", other)),
                };
                if matches!(operator, "is_not" | "neq" | "not_in") {
                    Ok(negative.to_string())
                } else if matches!(operator, "is" | "eq" | "in") {
                    Ok(positive.to_string())
                } else {
                    Err(format!("Unsupported media subtype operator: {}", operator))
                }
            }
            "extension" => {
                let condition = Self::build_smart_extension_condition(value, sql_params)?;
                Ok(if matches!(operator, "is_not" | "not_in" | "neq") {
                    format!("NOT {}", condition)
                } else {
                    condition
                })
            }
            "favorite" => {
                let desired = Self::smart_rule_bool(value).unwrap_or(true);
                let is_positive = matches!(operator, "is" | "eq");
                let value = if is_positive { desired } else { !desired };
                Ok(if value {
                    "a.is_favorite = 1".to_string()
                } else {
                    "(a.is_favorite = 0 OR a.is_favorite IS NULL)".to_string()
                })
            }
            "rating" => {
                let mut conditions = Vec::new();
                Self::push_numeric_rule(&mut conditions, sql_params, "a.rating", operator, value)?;
                Ok(conditions.pop().unwrap_or_else(|| "1 = 1".to_string()))
            }
            "culling" => {
                let mut conditions = Vec::new();
                Self::push_numeric_rule(
                    &mut conditions,
                    sql_params,
                    "COALESCE(a.culling_flag, 0)",
                    operator,
                    value,
                )?;
                Ok(conditions.pop().unwrap_or_else(|| "1 = 1".to_string()))
            }
            "date_taken" | "date_created" | "date_modified" => {
                let date_col = Self::smart_rule_date_column(field)?;
                match operator {
                    "is" | "eq" => {
                        let (start, end) = Self::smart_rule_date_period_range(value)?;
                        sql_params.push(Box::new(start));
                        sql_params.push(Box::new(end));
                        Ok(format!("{} >= ? AND {} < ?", date_col, date_col))
                    }
                    "before" | "lt" => {
                        let v = Self::smart_rule_date_value(value, "value")
                            .ok_or_else(|| "Date value required".to_string())?;
                        sql_params.push(Box::new(v));
                        Ok(format!("{} < ?", date_col))
                    }
                    "after" | "gt" => {
                        let v = Self::smart_rule_date_value(value, "value")
                            .ok_or_else(|| "Date value required".to_string())?;
                        sql_params.push(Box::new(v));
                        Ok(format!("{} > ?", date_col))
                    }
                    "between" => {
                        let start = Self::smart_rule_date_value(value, "start")
                            .ok_or_else(|| "Date start value required".to_string())?;
                        let end = Self::smart_rule_date_value(value, "end")
                            .ok_or_else(|| "Date end value required".to_string())?;
                        sql_params.push(Box::new(start));
                        sql_params.push(Box::new(end));
                        Ok(format!("{} >= ? AND {} < ?", date_col, date_col))
                    }
                    "in_last" => {
                        let cutoff = Self::smart_rule_relative_date_cutoff(value)?;
                        sql_params.push(Box::new(cutoff));
                        Ok(format!("{} >= ?", date_col))
                    }
                    "older_than" => {
                        let cutoff = Self::smart_rule_relative_date_cutoff(value)?;
                        sql_params.push(Box::new(cutoff));
                        Ok(format!("{} < ?", date_col))
                    }
                    _ => Err(format!("Unsupported date operator: {}", operator)),
                }
            }
            "size" => {
                let mut conditions = Vec::new();
                Self::push_numeric_rule(&mut conditions, sql_params, "a.size", operator, value)?;
                Ok(conditions.pop().unwrap_or_else(|| "1 = 1".to_string()))
            }
            "width" => {
                let mut conditions = Vec::new();
                Self::push_numeric_rule(&mut conditions, sql_params, "a.width", operator, value)?;
                Ok(conditions.pop().unwrap_or_else(|| "1 = 1".to_string()))
            }
            "height" => {
                let mut conditions = Vec::new();
                Self::push_numeric_rule(&mut conditions, sql_params, "a.height", operator, value)?;
                Ok(conditions.pop().unwrap_or_else(|| "1 = 1".to_string()))
            }
            "duration" => {
                let mut conditions = Vec::new();
                Self::push_numeric_rule(
                    &mut conditions,
                    sql_params,
                    "a.duration",
                    operator,
                    value,
                )?;
                Ok(conditions.pop().unwrap_or_else(|| "1 = 1".to_string()))
            }
            "has_gps" => {
                let desired = Self::smart_rule_bool(value).unwrap_or(true);
                let is_positive = matches!(operator, "is" | "eq");
                let value = if is_positive { desired } else { !desired };
                Ok(if value {
                    "(a.gps_latitude IS NOT NULL AND a.gps_longitude IS NOT NULL)".to_string()
                } else {
                    "(a.gps_latitude IS NULL OR a.gps_longitude IS NULL)".to_string()
                })
            }
            "orientation" => {
                if operator != "is" && operator != "eq" {
                    return Err(format!("Unsupported orientation operator: {}", operator));
                }
                Self::build_smart_orientation_condition(value)
            }
            "tag" => {
                if operator == "empty" {
                    return Ok(
                        "NOT EXISTS (SELECT 1 FROM afile_tags at2 WHERE at2.file_id = a.id)"
                            .to_string(),
                    );
                }
                if operator == "not_empty" {
                    return Ok(
                        "EXISTS (SELECT 1 FROM afile_tags at2 WHERE at2.file_id = a.id)"
                            .to_string(),
                    );
                }
                let id =
                    Self::smart_rule_i64(value).ok_or_else(|| "Tag id required".to_string())?;
                if matches!(operator, "has" | "is" | "eq") {
                    sql_params.push(Box::new(id));
                    Ok("EXISTS (SELECT 1 FROM afile_tags at2 WHERE at2.file_id = a.id AND at2.tag_id = ?)".to_string())
                } else if matches!(operator, "not_has" | "is_not" | "neq") {
                    sql_params.push(Box::new(id));
                    Ok("NOT EXISTS (SELECT 1 FROM afile_tags at2 WHERE at2.file_id = a.id AND at2.tag_id = ?)".to_string())
                } else {
                    Err(format!("Unsupported tag operator: {}", operator))
                }
            }
            "person" => {
                if operator == "empty" {
                    return Ok("NOT EXISTS (SELECT 1 FROM faces f2 WHERE f2.file_id = a.id AND f2.person_id IS NOT NULL)".to_string());
                }
                if operator == "not_empty" {
                    return Ok("EXISTS (SELECT 1 FROM faces f2 WHERE f2.file_id = a.id AND f2.person_id IS NOT NULL)".to_string());
                }
                let id =
                    Self::smart_rule_i64(value).ok_or_else(|| "Person id required".to_string())?;
                if matches!(operator, "has" | "is" | "eq") {
                    sql_params.push(Box::new(id));
                    Ok("EXISTS (SELECT 1 FROM faces f2 WHERE f2.file_id = a.id AND f2.person_id = ?)".to_string())
                } else if matches!(operator, "not_has" | "is_not" | "neq") {
                    sql_params.push(Box::new(id));
                    Ok("NOT EXISTS (SELECT 1 FROM faces f2 WHERE f2.file_id = a.id AND f2.person_id = ?)".to_string())
                } else {
                    Err(format!("Unsupported person operator: {}", operator))
                }
            }
            "album" => {
                let id = Self::smart_rule_i64(value)
                    .ok_or_else(|| "Album id required".to_string())?;
                sql_params.push(Box::new(id));
                if matches!(operator, "is" | "eq") {
                    Ok("b.album_id = ?".to_string())
                } else if matches!(operator, "is_not" | "neq") {
                    Ok("b.album_id != ?".to_string())
                } else {
                    Err(format!("Unsupported album operator: {}", operator))
                }
            }
            "collection" => {
                let id = Self::smart_rule_i64(value)
                    .ok_or_else(|| "Collection id required".to_string())?;
                sql_params.push(Box::new(id));
                let condition = "EXISTS (SELECT 1 FROM acollections_files cf WHERE cf.file_id = a.id AND cf.collection_id = ?)";
                if matches!(operator, "is" | "eq") {
                    Ok(condition.to_string())
                } else if matches!(operator, "is_not" | "neq") {
                    Ok(format!("NOT ({})", condition))
                } else {
                    Err(format!("Unsupported collection operator: {}", operator))
                }
            }
            "camera" | "lens" | "location" => {
                let id = Self::smart_rule_string(value)
                    .ok_or_else(|| format!("{} id required", field))?;
                let parts: Vec<&str> = id.split("||").collect();
                let mut conditions = Vec::new();
                match field {
                    "camera" => {
                        if let Some(make) = parts.get(0).filter(|v| !v.is_empty()) {
                            conditions.push("UPPER(a.e_make) = UPPER(?)".to_string());
                            sql_params.push(Box::new((*make).to_string()));
                        }
                        if let Some(model) = parts.get(1).filter(|v| !v.is_empty()) {
                            conditions.push("a.e_model = ?".to_string());
                            sql_params.push(Box::new((*model).to_string()));
                        }
                    }
                    "lens" => {
                        if let Some(make) = parts.get(0).filter(|v| !v.is_empty()) {
                            conditions.push("UPPER(a.e_lens_make) = UPPER(?)".to_string());
                            sql_params.push(Box::new((*make).to_string()));
                        }
                        if let Some(model) = parts.get(1).filter(|v| !v.is_empty()) {
                            conditions.push("a.e_lens_model = ?".to_string());
                            sql_params.push(Box::new((*model).to_string()));
                        }
                    }
                    "location" => {
                        if let Some(cc) = parts.get(0).filter(|v| !v.is_empty()) {
                            conditions.push("a.geo_cc = ?".to_string());
                            sql_params.push(Box::new((*cc).to_string()));
                        }
                        if let Some(admin1) = parts.get(1).filter(|v| !v.is_empty()) {
                            conditions.push("a.geo_admin1 = ?".to_string());
                            sql_params.push(Box::new((*admin1).to_string()));
                        }
                        if let Some(name) = parts.get(2).filter(|v| !v.is_empty()) {
                            conditions.push("a.geo_name = ?".to_string());
                            sql_params.push(Box::new((*name).to_string()));
                        }
                    }
                    _ => {}
                }
                if conditions.is_empty() {
                    return Err(format!("{} id required", field));
                }
                let condition = format!("({})", conditions.join(" AND "));
                Ok(if matches!(operator, "not_has" | "is_not" | "neq") {
                    format!("NOT {}", condition)
                } else {
                    condition
                })
            }
            _ => Err(format!("Unsupported smart rule field: {}", field)),
        }
    }

    fn build_smart_query_parts(
        params: &SmartQueryParams,
    ) -> Result<(String, String, Vec<Box<dyn ToSql>>, bool), String> {
        if params.rules.is_empty() {
            return Err("Smart query requires at least one rule".to_string());
        }

        let mut joins = Vec::new();
        let mut conditions = Vec::new();
        let mut sql_params: Vec<Box<dyn ToSql>> = Vec::new();
        let mut needs_group = false;

        for rule in &params.rules {
            conditions.push(Self::build_smart_rule_condition(
                rule,
                &mut joins,
                &mut needs_group,
                &mut sql_params,
            )?);
        }

        Self::append_album_filter(&mut conditions);

        let inaccessible_album_ids = t_utils::inaccessible_album_ids();
        if !inaccessible_album_ids.is_empty() {
            conditions.push(format!(
                "b.album_id NOT IN ({})",
                inaccessible_album_ids
                    .iter()
                    .map(i64::to_string)
                    .collect::<Vec<_>>()
                    .join(",")
            ));
        }

        conditions.push(Self::search_exclusion_condition("b"));
        conditions.push(Self::live_photo_companion_exclusion_condition().to_string());

        if let (Some(min_lat), Some(max_lat), Some(min_lon), Some(max_lon)) = (
            params.gps_min_lat,
            params.gps_max_lat,
            params.gps_min_lon,
            params.gps_max_lon,
        ) {
            conditions.push("a.gps_latitude BETWEEN ? AND ?".to_string());
            sql_params.push(Box::new(min_lat));
            sql_params.push(Box::new(max_lat));

            if min_lon <= max_lon {
                conditions.push("a.gps_longitude BETWEEN ? AND ?".to_string());
                sql_params.push(Box::new(min_lon));
                sql_params.push(Box::new(max_lon));
            } else {
                conditions.push("(a.gps_longitude >= ? OR a.gps_longitude <= ?)".to_string());
                sql_params.push(Box::new(min_lon));
                sql_params.push(Box::new(max_lon));
            }
        }

        let joiner = if params.r#match == "any" {
            " OR "
        } else {
            " AND "
        };
        let rule_count = params.rules.len();
        let where_clause = if conditions.is_empty() {
            String::new()
        } else if rule_count > 0 {
            let (rule_conditions, trailing_conditions) = conditions.split_at(rule_count);
            let mut grouped = vec![format!("({})", rule_conditions.join(joiner))];
            grouped.extend(trailing_conditions.iter().cloned());
            format!(" WHERE {}", grouped.join(" AND "))
        } else {
            format!(" WHERE {}", conditions.join(" AND "))
        };

        let joins_clause = if joins.is_empty() {
            String::new()
        } else {
            format!(" {}", joins.join(" "))
        };

        Ok((joins_clause, where_clause, sql_params, needs_group))
    }

    pub fn get_smart_query_count_and_sum(params: &SmartQueryParams) -> Result<(i64, i64), String> {
        let (joins, where_clause, sql_params, needs_group) = Self::build_smart_query_parts(params)?;
        let sql = if needs_group {
            format!(
                "SELECT COUNT(*), SUM(size) FROM (SELECT a.id, a.size FROM afiles a
                LEFT JOIN afolders b ON a.folder_id = b.id
                LEFT JOIN albums c ON b.album_id = c.id
                {}{} GROUP BY a.id)",
                joins, where_clause
            )
        } else {
            format!("{}{}{}", Self::build_count_query(), joins, where_clause)
        };
        let final_params: Vec<&dyn ToSql> = sql_params.iter().map(|p| p.as_ref()).collect();
        Self::query_count_and_sum(&sql, &final_params)
    }

    pub fn get_smart_query_files(
        params: &SmartQueryParams,
        offset: i64,
        limit: i64,
    ) -> Result<Vec<Self>, String> {
        let (joins, where_clause, sql_params, needs_group) = Self::build_smart_query_parts(params)?;
        let mut query = Self::build_base_query();
        query.push_str(&joins);
        query.push_str(&where_clause);
        if needs_group {
            query.push_str(" GROUP BY a.id");
        }
        query.push_str(&format!(
            " ORDER BY {}",
            Self::build_order_clause_values(params.sort_type, params.sort_order, params.random_seed)
        ));
        query.push_str(" LIMIT ? OFFSET ?");

        let mut final_params: Vec<&dyn ToSql> = sql_params.iter().map(|p| p.as_ref()).collect();
        final_params.push(&limit);
        final_params.push(&offset);
        Self::query_files(&query, &final_params)
    }

    fn query_smart_groups(params: &SmartQueryParams) -> Result<Vec<QueryGroup>, String> {
        let Some((group_id_expr, sort_expr)) =
            Self::group_key_and_sort_expr(params.group_by, params.calendar_sort)
        else {
            return Ok(Vec::new());
        };
        let (joins, where_clause, sql_params, _needs_group) =
            Self::build_smart_query_parts(params)?;
        let sql = format!(
            "SELECT group_id, group_id AS label, COUNT(*), COALESCE(SUM(size), 0)
             FROM (
                SELECT DISTINCT a.id, a.size, {group_id_expr} AS group_id, {sort_expr} AS group_sort
                FROM afiles a
                LEFT JOIN afolders b ON a.folder_id = b.id
                LEFT JOIN albums c ON b.album_id = c.id
                {joins}{where_clause}
             )
             GROUP BY group_id
             ORDER BY {}",
            Self::smart_group_order_clause(params)
        );
        let final_params: Vec<&dyn ToSql> = sql_params.iter().map(|p| p.as_ref()).collect();
        let conn = open_conn()?;
        let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(&final_params[..], |row| {
                Ok(QueryGroup {
                    id: row.get(0)?,
                    label: row.get(1)?,
                    count: row.get(2)?,
                    size: row.get(3)?,
                })
            })
            .map_err(|e| e.to_string())?;

        let mut groups = Vec::new();
        for group in rows {
            groups.push(group.map_err(|e| e.to_string())?);
        }
        Ok(groups)
    }

    fn get_smart_query_files_in_group(
        params: &SmartQueryParams,
        group_id: &str,
        offset: i64,
        limit: i64,
    ) -> Result<Vec<Self>, String> {
        if limit <= 0 {
            return Ok(Vec::new());
        }
        let Some((group_id_expr, _)) =
            Self::group_key_and_sort_expr(params.group_by, params.calendar_sort)
        else {
            return Ok(Vec::new());
        };
        let (joins, where_clause, mut sql_params, _needs_group) =
            Self::build_smart_query_parts(params)?;

        let mut query = Self::build_base_query();
        query.push_str(&joins);
        query.push_str(&where_clause);
        if where_clause.is_empty() {
            query.push_str(&format!(" WHERE {} = ?", group_id_expr));
        } else {
            query.push_str(&format!(" AND {} = ?", group_id_expr));
        }
        sql_params.push(Box::new(group_id.to_string()));

        query.push_str(" GROUP BY a.id");
        query.push_str(&format!(
            " ORDER BY {}",
            Self::build_order_clause_values(params.sort_type, params.sort_order, params.random_seed)
        ));
        query.push_str(" LIMIT ? OFFSET ?");

        let mut final_params: Vec<&dyn ToSql> = sql_params.iter().map(|p| p.as_ref()).collect();
        final_params.push(&limit);
        final_params.push(&offset);
        Self::query_files(&query, &final_params)
    }

    pub fn get_smart_grouped_query_rows(
        params: &SmartQueryParams,
        offset: i64,
        limit: i64,
    ) -> Result<GroupedQueryResult, String> {
        let groups = Self::query_smart_groups(params)?;
        Self::build_grouped_query_result(
            groups,
            offset,
            limit,
            |group, group_file_offset, group_file_limit| {
                Self::get_smart_query_files_in_group(
                    params,
                    &group.id,
                    group_file_offset,
                    group_file_limit,
                )
            },
        )
    }

    pub fn get_smart_group_file_ids(
        params: &SmartQueryParams,
        group_id: &str,
    ) -> Result<Vec<i64>, String> {
        let Some((group_id_expr, _)) =
            Self::group_key_and_sort_expr(params.group_by, params.calendar_sort)
        else {
            return Ok(Vec::new());
        };
        let (joins, where_clause, mut sql_params, _needs_group) =
            Self::build_smart_query_parts(params)?;
        let mut query = format!(
            "SELECT a.id
             FROM afiles a
             LEFT JOIN afolders b ON a.folder_id = b.id
             LEFT JOIN albums c ON b.album_id = c.id
             {joins}{where_clause}"
        );
        if where_clause.is_empty() {
            query.push_str(&format!(" WHERE {} = ?", group_id_expr));
        } else {
            query.push_str(&format!(" AND {} = ?", group_id_expr));
        }
        sql_params.push(Box::new(group_id.to_string()));
        query.push_str(" GROUP BY a.id");
        query.push_str(&format!(
            " ORDER BY {}",
            Self::build_order_clause_values(params.sort_type, params.sort_order, params.random_seed)
        ));

        let final_params: Vec<&dyn ToSql> = sql_params.iter().map(|p| p.as_ref()).collect();
        let conn = open_conn()?;
        let mut stmt = conn.prepare(&query).map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(&final_params[..], |row| row.get::<_, i64>(0))
            .map_err(|e| e.to_string())?;

        let mut ids = Vec::new();
        for id in rows {
            ids.push(id.map_err(|e| e.to_string())?);
        }
        Ok(ids)
    }

    pub fn get_smart_query_file_ids(params: &SmartQueryParams) -> Result<Vec<i64>, String> {
        let (joins, where_clause, sql_params, needs_group) = Self::build_smart_query_parts(params)?;
        let mut query = format!(
            "SELECT a.id
             FROM afiles a
             LEFT JOIN afolders b ON a.folder_id = b.id
             LEFT JOIN albums c ON b.album_id = c.id
             {joins}{where_clause}"
        );
        if needs_group {
            query.push_str(" GROUP BY a.id");
        }
        query.push_str(&format!(
            " ORDER BY {}",
            Self::build_order_clause_values(params.sort_type, params.sort_order, params.random_seed)
        ));

        let final_params: Vec<&dyn ToSql> = sql_params.iter().map(|p| p.as_ref()).collect();
        let conn = open_conn()?;
        let mut stmt = conn.prepare(&query).map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(&final_params[..], |row| row.get::<_, i64>(0))
            .map_err(|e| e.to_string())?;

        let mut ids = Vec::new();
        for id in rows {
            ids.push(id.map_err(|e| e.to_string())?);
        }
        Ok(ids)
    }

    pub fn get_smart_query_file_position(
        params: &SmartQueryParams,
        file_id: i64,
    ) -> Result<Option<i64>, String> {
        if file_id <= 0 {
            return Ok(None);
        }
        let (joins, where_clause, sql_params, needs_group) = Self::build_smart_query_parts(params)?;
        let query = format!(
            "WITH ranked_files AS (
                SELECT
                    a.id,
                    ROW_NUMBER() OVER (ORDER BY {}) - 1 AS position
                FROM afiles a
                LEFT JOIN afolders b ON a.folder_id = b.id
                LEFT JOIN albums c ON b.album_id = c.id
                {}
                {}
                {}
            )
            SELECT position FROM ranked_files WHERE id = ?",
            Self::build_order_clause_values(params.sort_type, params.sort_order, params.random_seed),
            joins,
            where_clause,
            if needs_group { " GROUP BY a.id" } else { "" }
        );

        let conn = open_conn()?;
        let mut stmt = conn.prepare(&query).map_err(|e| e.to_string())?;
        let mut final_params: Vec<&dyn ToSql> = sql_params.iter().map(|p| p.as_ref()).collect();
        final_params.push(&file_id);

        stmt.query_row(final_params.as_slice(), |row| row.get(0))
            .optional()
            .map_err(|e| e.to_string())
    }

    pub fn get_smart_query_time_line(params: &SmartQueryParams) -> Result<Vec<ATimeLine>, String> {
        if params.sort_type > 2 {
            return Ok(Vec::new());
        }

        let (joins, where_clause, sql_params, needs_group) = Self::build_smart_query_parts(params)?;
        let (date_field, year_extract, month_extract, date_extract) = match params.sort_type {
            0 => (
                "a.taken_date",
                "CAST(strftime('%Y', a.taken_date, 'unixepoch', 'localtime') AS INTEGER)",
                "CAST(strftime('%m', a.taken_date, 'unixepoch', 'localtime') AS INTEGER)",
                "CAST(strftime('%d', a.taken_date, 'unixepoch', 'localtime') AS INTEGER)",
            ),
            1 => (
                "a.created_at",
                "CAST(strftime('%Y', a.created_at, 'unixepoch', 'localtime') AS INTEGER)",
                "CAST(strftime('%m', a.created_at, 'unixepoch', 'localtime') AS INTEGER)",
                "CAST(strftime('%d', a.created_at, 'unixepoch', 'localtime') AS INTEGER)",
            ),
            2 => (
                "a.modified_at",
                "CAST(strftime('%Y', a.modified_at, 'unixepoch', 'localtime') AS INTEGER)",
                "CAST(strftime('%m', a.modified_at, 'unixepoch', 'localtime') AS INTEGER)",
                "CAST(strftime('%d', a.modified_at, 'unixepoch', 'localtime') AS INTEGER)",
            ),
            _ => unreachable!(),
        };
        let order_clause = if params.sort_order == 0 {
            "ASC"
        } else {
            "DESC"
        };
        let query = format!(
            "WITH ranked_files AS (
                SELECT
                    ROW_NUMBER() OVER (ORDER BY {} {}) - 1 AS position,
                    {} AS year,
                    {} AS month,
                    {} AS date
                FROM afiles a
                LEFT JOIN afolders b ON a.folder_id = b.id
                {}
                {}
                {}
            )
            SELECT year, month, date, MIN(position) as position
            FROM ranked_files
            WHERE year IS NOT NULL
            GROUP BY year, month, date
            ORDER BY position ASC",
            date_field,
            order_clause,
            year_extract,
            month_extract,
            date_extract,
            joins,
            where_clause,
            if needs_group { " GROUP BY a.id" } else { "" }
        );

        let conn = open_conn()?;
        let final_params: Vec<&dyn ToSql> = sql_params.iter().map(|p| p.as_ref()).collect();
        let mut stmt = conn.prepare(&query).map_err(|e| e.to_string())?;
        let timelines = stmt
            .query_map(final_params.as_slice(), |row| {
                Ok(ATimeLine {
                    year: row.get(0)?,
                    month: row.get(1)?,
                    date: row.get(2)?,
                    position: row.get(3)?,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        Ok(timelines)
    }

    pub fn get_query_file_position(
        params: &QueryParams,
        file_id: i64,
    ) -> Result<Option<i64>, String> {
        if file_id <= 0 {
            return Ok(None);
        }

        let (joins, where_clause, sql_params) = Self::build_search_query_parts(params);
        let mut query = format!(
            "WITH ranked_files AS (
                SELECT
                    a.id,
                    ROW_NUMBER() OVER (ORDER BY {}) - 1 AS position
                FROM afiles a
                LEFT JOIN afolders b ON a.folder_id = b.id
                LEFT JOIN albums c ON b.album_id = c.id
                {}
                {}
                {}
            )
            SELECT position FROM ranked_files WHERE id = ?",
            Self::build_order_clause(params),
            joins,
            where_clause,
            if params.person_id > 0 {
                " GROUP BY a.id"
            } else {
                ""
            }
        );

        // Keep SQL clean when where/group are empty to avoid odd spacing.
        query = query.replace("\n                \n", "\n");

        let conn = open_conn()?;
        let mut stmt = conn.prepare(&query).map_err(|e| e.to_string())?;
        let mut final_params: Vec<&dyn ToSql> = sql_params.iter().map(|p| p.as_ref()).collect();
        final_params.push(&file_id);

        stmt.query_row(final_params.as_slice(), |row| row.get(0))
            .optional()
            .map_err(|e| e.to_string())
    }

    // get query timeline markers
    pub fn get_query_time_line(params: &QueryParams) -> Result<Vec<ATimeLine>, String> {
        // Only process for time-based sorts (0=taken_date, 1=created_at, 2=modified_at)
        if params.sort_type > 2 {
            return Ok(Vec::new());
        }

        let (joins, where_clause, sql_params) = Self::build_search_query_parts(params);

        // Determine date field and extraction logic based on sort_type
        let (date_field, year_extract, month_extract, date_extract) = match params.sort_type {
            0 => (
                "a.taken_date",
                "CAST(strftime('%Y', a.taken_date, 'unixepoch', 'localtime') AS INTEGER)",
                "CAST(strftime('%m', a.taken_date, 'unixepoch', 'localtime') AS INTEGER)",
                "CAST(strftime('%d', a.taken_date, 'unixepoch', 'localtime') AS INTEGER)",
            ),
            1 => (
                "a.created_at",
                "CAST(strftime('%Y', a.created_at, 'unixepoch', 'localtime') AS INTEGER)",
                "CAST(strftime('%m', a.created_at, 'unixepoch', 'localtime') AS INTEGER)",
                "CAST(strftime('%d', a.created_at, 'unixepoch', 'localtime') AS INTEGER)",
            ),
            2 => (
                "a.modified_at",
                "CAST(strftime('%Y', a.modified_at, 'unixepoch', 'localtime') AS INTEGER)",
                "CAST(strftime('%m', a.modified_at, 'unixepoch', 'localtime') AS INTEGER)",
                "CAST(strftime('%d', a.modified_at, 'unixepoch', 'localtime') AS INTEGER)",
            ),
            _ => unreachable!(),
        };

        let order_clause = if params.sort_order == 0 {
            "ASC"
        } else {
            "DESC"
        };

        // Build query with ROW_NUMBER to calculate positions
        let query = format!(
            "WITH ranked_files AS (
                SELECT 
                    ROW_NUMBER() OVER (ORDER BY {} {}) - 1 AS position,
                    {} AS year,
                    {} AS month,
                    {} AS date
                FROM afiles a
                LEFT JOIN afolders b ON a.folder_id = b.id
                {}
                {}
            )
            SELECT year, month, date, MIN(position) as position
            FROM ranked_files
            WHERE year IS NOT NULL
            GROUP BY year, month, date
            ORDER BY position ASC",
            date_field,
            order_clause,
            year_extract,
            month_extract,
            date_extract,
            joins,
            where_clause
        );

        let conn = open_conn()?;
        let final_params: Vec<&dyn ToSql> = sql_params.iter().map(|p| p.as_ref()).collect();
        let mut stmt = conn.prepare(&query).map_err(|e| e.to_string())?;

        let timelines = stmt
            .query_map(final_params.as_slice(), |row| {
                Ok(ATimeLine {
                    year: row.get(0)?,
                    month: row.get(1)?,
                    date: row.get(2)?,
                    position: row.get(3)?,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        Ok(timelines)
    }

    // get all files in a folder by folder id (DB only)
    pub fn get_files_by_folder_id(folder_id: i64) -> Result<Vec<Self>, String> {
        let sql = format!(
            "{} WHERE a.folder_id = ?1 ORDER BY a.name ASC",
            Self::build_base_query()
        );
        Self::query_files(&sql, &[&folder_id])
    }

    // --- AI Logic ---

    /// check ai status
    pub fn check_ai_status(state: &State<t_ai::AiState>) -> String {
        let engine = state.0.lock().unwrap();
        if engine.is_loaded() {
            "AI Models Loaded".to_string()
        } else {
            "AI Engine Initialized (Models Not Loaded)".to_string()
        }
    }

    /// get query embedding from search text or similar image id
    pub fn get_query_embedding(
        state: &State<t_ai::AiState>,
        params: &ImageSearchParams,
    ) -> Result<Option<Vec<f32>>, String> {
        if !params.search_text.is_empty() {
            let mut engine = state.0.lock().unwrap();
            Ok(Some(engine.encode_text(&params.search_text)?))
        } else if let Some(file_id) = params.file_id.filter(|&id| id > 0) {
            match Self::get_embedding_by_id(file_id) {
                Ok(emb) => Ok(Some(emb)),
                Err(_) => {
                    Self::generate_embedding(state, file_id)?;
                    Ok(Some(Self::get_embedding_by_id(file_id)?))
                }
            }
        } else {
            Ok(None)
        }
    }

    /// generate embedding for a file
    pub fn generate_embedding(
        state: &State<t_ai::AiState>,
        file_id: i64,
    ) -> Result<String, String> {
        // 1. Fetch file info to get path
        let file_opt = Self::get_file_info(file_id).map_err(|e| e.to_string())?;
        let file = file_opt.ok_or("File not found")?;

        // 2. Check if it's an image
        // file_type: 1 is image, 3 is HEIC
        if file.file_type != Some(1) && file.file_type != Some(3) {
            return Err("File is not an image".to_string());
        }

        let file_path = file.file_path.ok_or("File path not resolved")?;

        // 3. Check if embedding exists
        if let Ok(embeds) = Self::get_embedding_by_id(file_id) {
            if !embeds.is_empty() {
                return Ok("Embedding already exists".to_string());
            }
        }

        // 4. Generate embedding
        let mut engine = state.0.lock().unwrap();

        // Optimized: Use thumbnail if available (much faster than loading original)
        // Fallback to original file if thumbnail is missing or fails to process
        let embedding = match AThumb::fetch(file_id) {
            Ok(Some(thumb)) if thumb.thumb_data.is_some() => {
                let thumb_bytes = thumb.thumb_data.as_ref().unwrap();
                match panic::catch_unwind(AssertUnwindSafe(|| {
                    engine.encode_image_from_bytes(thumb_bytes)
                })) {
                    Ok(res) => res.or_else(|_| {
                        // If thumbnail processing fails (e.g. corrupted), try original
                        match panic::catch_unwind(AssertUnwindSafe(|| {
                            engine.encode_image(&file_path)
                        })) {
                            Ok(res2) => res2,
                            Err(_) => Err(format!(
                                "Embedding panic while encoding original image: {}",
                                file_path
                            )),
                        }
                    }),
                    // If thumbnail path panics, still try original once.
                    Err(_) => match panic::catch_unwind(AssertUnwindSafe(|| {
                        engine.encode_image(&file_path)
                    })) {
                        Ok(res2) => res2,
                        Err(_) => Err(format!(
                            "Embedding panic while encoding original image: {}",
                            file_path
                        )),
                    },
                }
            }
            _ => match panic::catch_unwind(AssertUnwindSafe(|| engine.encode_image(&file_path))) {
                Ok(res) => res,
                Err(_) => Err(format!(
                    "Embedding panic while encoding original image: {}",
                    file_path
                )),
            },
        }?;

        // 5. Save to DB
        let _ =
            Self::update_embedding(file_id, embedding).map_err(|e| format!("DB Error: {}", e))?;

        Ok("Embedding generated and saved".to_string())
    }

    /// Update embedding for a file
    pub fn update_embedding(file_id: i64, embedding: Vec<f32>) -> Result<usize, String> {
        // Convert Vec<f32> to Vec<u8>
        let mut bytes = Vec::with_capacity(embedding.len() * 4);
        for val in embedding {
            bytes.extend_from_slice(&val.to_le_bytes());
        }

        let conn = open_conn()?;
        let result = conn
            .execute(
                "UPDATE afiles SET embeds = ?1 WHERE id = ?2",
                params![bytes, file_id],
            )
            .map_err(|e| e.to_string())?;
        Ok(result)
    }

    pub fn get_embedding_by_id(file_id: i64) -> Result<Vec<f32>, String> {
        let conn = open_conn()?;
        let embeds_blob: Vec<u8> = conn
            .query_row(
                "SELECT embeds FROM afiles WHERE id = ?1 AND embeds IS NOT NULL",
                params![file_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "Image embedding not found".to_string())?;

        let embedding: Vec<f32> = embeds_blob
            .chunks_exact(4)
            .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
            .collect();

        Ok(embedding)
    }

    /// search similar images
    pub fn search_similar_images(
        state: &State<t_ai::AiState>,
        params: ImageSearchParams,
    ) -> Result<Vec<Self>, String> {
        // 1. Determine Target Embedding
        let embedding_opt = Self::get_query_embedding(state, &params)?;
        let embedding =
            embedding_opt.ok_or_else(|| "No file_id or search_text provided".to_string())?;

        // 2. Perform Vector Search
        let conn = open_conn()?;

        let mut query = "SELECT a.id, a.embeds
            FROM afiles a
            LEFT JOIN afolders b ON a.folder_id = b.id
            WHERE a.embeds IS NOT NULL"
            .to_string();

        query.push_str(" AND ");
        query.push_str(&Self::search_exclusion_condition("b"));

        let inaccessible_album_ids = t_utils::inaccessible_album_ids();
        if !inaccessible_album_ids.is_empty() {
            query.push_str(&format!(
                " AND b.album_id NOT IN ({})",
                inaccessible_album_ids
                    .iter()
                    .map(i64::to_string)
                    .collect::<Vec<_>>()
                    .join(",")
            ));
        }

        query.push_str(&Self::album_filter_sql("a"));

        if let Some(ft_condition) = Self::build_file_type_condition(params.file_type) {
            query.push_str(" AND ");
            query.push_str(&ft_condition);
        }

        let mut stmt = conn.prepare(&query).map_err(|e| e.to_string())?;

        let rows = stmt
            .query_map([], |row| {
                let id: i64 = row.get(0)?;
                let embeds_blob: Vec<u8> = row.get(1)?;
                Ok((id, embeds_blob))
            })
            .map_err(|e| e.to_string())?;

        let mut scores: Vec<(i64, f32)> = Vec::new();

        let threshold = params.threshold.clamp(0.0, 1.0);
        let query_norm = embedding
            .iter()
            .map(|value| value * value)
            .sum::<f32>()
            .sqrt();

        // Calculate similarity
        for row in rows {
            let (id, embeds_blob) = row.map_err(|e| e.to_string())?;
            let score = Self::cosine_similarity_blob(&embedding, query_norm, &embeds_blob);

            if score > threshold {
                scores.push((id, score));
            }
        }

        // Sort by score descending
        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Fetch full file info in batches, then restore similarity order.
        let result_ids = scores.iter().map(|(id, _)| *id).collect::<Vec<_>>();
        let files = Self::get_files_by_ids(&result_ids)?;
        let mut files_by_id = files
            .into_iter()
            .filter_map(|file| file.id.map(|id| (id, file)))
            .collect::<HashMap<_, _>>();
        let results = result_ids
            .into_iter()
            .filter_map(|id| files_by_id.remove(&id))
            .collect::<Vec<_>>();

        println!("Returning {} files", results.len());

        Ok(results)
    }

    fn cosine_similarity_blob(query: &[f32], query_norm: f32, blob: &[u8]) -> f32 {
        if query_norm == 0.0 || blob.len() % 4 != 0 {
            return 0.0;
        }

        let file_len = blob.len() / 4;
        if file_len != query.len() {
            return 0.0;
        }

        let mut dot_product = 0.0_f32;
        let mut file_norm_squared = 0.0_f32;
        for (index, chunk) in blob.chunks_exact(4).enumerate() {
            let value = f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
            dot_product += query[index] * value;
            file_norm_squared += value * value;
        }

        let file_norm = file_norm_squared.sqrt();
        if file_norm == 0.0 {
            0.0
        } else {
            dot_product / (query_norm * file_norm)
        }
    }
}

/// Define the album thumbnail struct
#[derive(Debug, Serialize, Deserialize)]
pub struct AThumb {
    pub id: Option<i64>, // unique id (autoincrement by db)
    pub file_id: i64,    // file id (from files table)
    pub error_code: i64, // error code (0: success, 1: error, 2: use original)

    #[serde(skip)]
    pub thumb_data: Option<Vec<u8>>, // thumbnail data (store into db as BLOB)

    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumb_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumb_mtime: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumb_size: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<i64>,

    // output only
    pub thumb_data_base64: Option<String>, // fetch thumbnail data as base64 string (for webview)
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ThumbnailCacheCleanupResult {
    pub files_removed: u64,
    pub bytes_freed: u64,
}

impl AThumb {
    const CACHE_EXTENSIONS: [&'static str; 2] = ["png", "jpg"];
    // Cache files are written before their database row is updated. Keep recent
    // unreferenced files so maintenance cannot race thumbnail generation.
    const CLEANUP_MIN_FILE_AGE: Duration = Duration::from_secs(5 * 60);

    fn should_use_original_image(file_id: i64, file_type: i64, thumbnail_size: u32) -> bool {
        if file_type != 1 || thumbnail_size == 0 {
            return false;
        }

        AFile::get_file_info(file_id)
            .ok()
            .flatten()
            .map(|file| {
                #[cfg(target_os = "linux")]
                if file
                    .file_path
                    .as_deref()
                    .is_some_and(|path| path.to_ascii_lowercase().ends_with(".avif"))
                {
                    return false;
                }

                if file
                    .file_path
                    .as_deref()
                    .is_some_and(t_image::is_ffmpeg_backed_image_path)
                {
                    return false;
                }

                let width = file.width.unwrap_or(0).max(0) as u32;
                let height = file.height.unwrap_or(0).max(0) as u32;
                width > 0 && height > 0 && width <= thumbnail_size && height <= thumbnail_size
            })
            .unwrap_or(false)
    }

    fn is_png_bytes(data: &[u8]) -> bool {
        data.starts_with(&[0x89, 0x50, 0x4E, 0x47])
    }

    fn is_complete_jpeg(data: &[u8]) -> bool {
        data.starts_with(&[0xFF, 0xD8, 0xFF]) && data.ends_with(&[0xFF, 0xD9])
    }

    fn generation_lock_key(file_id: i64, _thumbnail_size: u32) -> String {
        // A file has one active thumbnail record. Serializing every size for a
        // file prevents an older in-flight request from overwriting a newer
        // thumbnail-size selection.
        file_id.to_string()
    }

    fn acquire_generation_guard(file_id: i64, thumbnail_size: u32) -> ThumbGenerationGuard {
        let key = Self::generation_lock_key(file_id, thumbnail_size);
        let locks = thumb_generation_locks();
        let mut active = locks.active.lock().unwrap_or_else(|e| e.into_inner());

        loop {
            if !active.contains(&key) {
                active.insert(key.clone());
                return ThumbGenerationGuard { key };
            }

            active = locks
                .available
                .wait(active)
                .unwrap_or_else(|e| e.into_inner());
        }
    }

    pub(crate) fn try_begin_background_task(file_id: i64, thumbnail_size: u32) -> bool {
        let key = Self::generation_lock_key(file_id, thumbnail_size);
        let Ok(mut tasks) = thumb_background_tasks().lock() else {
            return false;
        };
        if tasks.contains(&key) {
            return false;
        }
        tasks.insert(key);
        true
    }

    pub(crate) fn finish_background_task(file_id: i64, thumbnail_size: u32) {
        let key = Self::generation_lock_key(file_id, thumbnail_size);
        if let Ok(mut tasks) = thumb_background_tasks().lock() {
            tasks.remove(&key);
        }
    }

    fn now_ts() -> i64 {
        chrono::Utc::now().timestamp()
    }

    fn get_source_mtime(file_path: &str) -> Option<i64> {
        fs::metadata(file_path)
            .ok()
            .and_then(|m| m.modified().ok())
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_secs() as i64)
    }

    fn get_current_library_id() -> String {
        t_config::load_app_config()
            .map(|c| c.current_library_id)
            .unwrap_or_else(|_| "default".to_string())
    }

    fn build_thumb_key(
        library_id: &str,
        file_id: i64,
        thumbnail_size: u32,
        source_mtime: Option<i64>,
        orientation: i32,
        raw_thumbnail_source: Option<RawDisplayOptions>,
    ) -> String {
        let mut hasher = blake3::Hasher::new();
        hasher.update(b"lap-thumb-v1");
        hasher.update(library_id.as_bytes());
        hasher.update(&file_id.to_le_bytes());
        hasher.update(&thumbnail_size.to_le_bytes());
        let orientation = if raw_thumbnail_source.is_some() && orientation == 0 { 1 } else { orientation };
        hasher.update(&orientation.to_le_bytes());
        hasher.update(&source_mtime.unwrap_or_default().to_le_bytes());
        if let Some(prefer_embedded) = raw_thumbnail_source {
            hasher.update(prefer_embedded.cache_tag().as_bytes());
            if let Some(companion) = crate::t_raw_display::paired_file(file_id, prefer_embedded) {
                hasher.update(&companion.id.unwrap_or_default().to_le_bytes());
                if let Some(path) = companion.file_path.as_deref() {
                    hasher.update(path.as_bytes());
                    if let Ok(metadata) = fs::metadata(path) {
                        hasher.update(&metadata.len().to_le_bytes());
                        let modified = metadata.modified().ok().and_then(|time| time.duration_since(UNIX_EPOCH).ok());
                        hasher.update(&modified.map(|time| time.as_nanos()).unwrap_or_default().to_le_bytes());
                    }
                }
            }
        }
        let hash = hasher.finalize().to_hex().to_string();
        match raw_thumbnail_source {
            Some(options) => format!("{}{}", if options.embedded() { if options.auto_bright { 'b' } else { 'e' } } else if options.auto_bright { 'a' } else { 'r' }, hash),
            None => hash,
        }
    }

    fn get_file_album_id(file_id: i64) -> Result<Option<i64>, String> {
        AFile::get_file_info(file_id)
            .map(|file| file.and_then(|f| f.album_id))
            .map_err(|e| e.to_string())
    }

    fn get_thumb_cache_path_for_key(
        library_id: &str,
        album_id: i64,
        thumb_key: &str,
        extension: &str,
    ) -> Result<PathBuf, String> {
        if thumb_key.len() < 2 {
            return Err("Invalid thumbnail cache key".to_string());
        }

        let cache_root = t_config::get_app_cache_dir()?
            .join(library_id)
            .join(album_id.to_string());
        Ok(cache_root
            .join(&thumb_key[0..2])
            .join(format!("{}.{}", thumb_key, extension)))
    }

    fn read_thumb_cache_bytes(
        library_id: &str,
        album_id: i64,
        thumb_key: &str,
        preferred_extension: &str,
    ) -> Result<Option<Vec<u8>>, String> {
        let extensions = if preferred_extension == "png" {
            Self::CACHE_EXTENSIONS
        } else {
            ["jpg", "png"]
        };
        for extension in extensions {
            let path =
                Self::get_thumb_cache_path_for_key(library_id, album_id, thumb_key, extension)?;
            if !path.exists() {
                continue;
            }
            let data = fs::read(path).map_err(|e| e.to_string())?;
            if match extension {
                "png" => Self::is_png_bytes(&data),
                "jpg" => Self::is_complete_jpeg(&data),
                _ => false,
            } {
                return Ok(Some(data));
            }
        }
        Ok(None)
    }

    fn write_thumb_cache_bytes(
        library_id: &str,
        album_id: i64,
        thumb_key: &str,
        data: &[u8],
    ) -> Result<PathBuf, String> {
        let extension = if Self::is_png_bytes(data) {
            "png"
        } else if Self::is_complete_jpeg(data) {
            "jpg"
        } else {
            return Err("Invalid thumbnail data".to_string());
        };
        let path = Self::get_thumb_cache_path_for_key(library_id, album_id, thumb_key, extension)?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let temp_path = path.with_extension(format!(
            "{}.{}.tmp",
            process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        fs::write(&temp_path, data).map_err(|e| e.to_string())?;
        if let Err(first_error) = fs::rename(&temp_path, &path) {
            // Windows does not replace an existing destination with rename.
            if !path.exists()
                || fs::remove_file(&path).is_err()
                || fs::rename(&temp_path, &path).is_err()
            {
                let _ = fs::remove_file(&temp_path);
                return Err(first_error.to_string());
            }
        }
        Ok(path)
    }

    fn delete_thumb_cache_for_key(
        library_id: &str,
        album_id: i64,
        thumb_key: &str,
    ) {
        for extension in Self::CACHE_EXTENSIONS {
            if let Ok(path) =
                Self::get_thumb_cache_path_for_key(library_id, album_id, thumb_key, extension)
            {
                let _ = fs::remove_file(path);
            }
        }
    }

    /// Remove cache files no longer referenced by the target library's `athumbs`
    /// table. When `library_id` is None (or empty), falls back to the current
    /// library. Only jpg/png files under that library's cache directory and
    /// older than `CLEANUP_MIN_FILE_AGE` are considered.
    pub fn clean_unused_cache(library_id: Option<&str>) -> Result<ThumbnailCacheCleanupResult, String> {
        // Resolve target library: explicit id (any library) or current (legacy behavior).
        let target_library_id = match library_id {
            Some(id) if !id.is_empty() => id.to_string(),
            _ => Self::get_current_library_id(),
        };

        // Read referenced thumb_keys from the target library's DB. When it's the
        // current library, reuse the pooled connection; otherwise open a transient
        // connection to that library's DB file (same pattern as get_library_info).
        let referenced_keys: HashSet<String> = if target_library_id == Self::get_current_library_id() {
            let conn = open_conn()?;
            let mut stmt = conn
                .prepare("SELECT thumb_key FROM athumbs WHERE thumb_key IS NOT NULL")
                .map_err(|e| e.to_string())?;
            let rows = stmt
                .query_map([], |row| row.get(0))
                .map_err(|e| e.to_string())?;
            rows.map(|row| row.map_err(|e| e.to_string()))
                .collect::<Result<HashSet<String>, String>>()?
        } else {
            let _lease = t_storage::DbConnectionLease::acquire()?;
            let db_path = crate::t_storage::get_library_db_path(&target_library_id)?;
            let conn = rusqlite::Connection::open(&db_path)
                .map_err(|e| format!("Failed to open library DB: {}", e))?;
            let mut stmt = conn
                .prepare("SELECT thumb_key FROM athumbs WHERE thumb_key IS NOT NULL")
                .map_err(|e| e.to_string())?;
            let rows = stmt
                .query_map([], |row| row.get(0))
                .map_err(|e| e.to_string())?;
            rows.map(|row| row.map_err(|e| e.to_string()))
                .collect::<Result<HashSet<String>, String>>()?
        };

        let cache_root = t_config::get_app_cache_dir()?.join(&target_library_id);
        if !cache_root.is_dir() {
            return Ok(ThumbnailCacheCleanupResult {
                files_removed: 0,
                bytes_freed: 0,
            });
        }

        let mut result = ThumbnailCacheCleanupResult {
            files_removed: 0,
            bytes_freed: 0,
        };
        for entry in WalkDir::new(cache_root).into_iter().filter_map(|entry| entry.ok()) {
            if !entry.file_type().is_file()
                || !matches!(entry.path().extension().and_then(|ext| ext.to_str()), Some("jpg" | "png"))
            {
                continue;
            }

            let Some(thumb_key) = entry.path().file_stem().and_then(|stem| stem.to_str()) else {
                continue;
            };
            if referenced_keys.contains(thumb_key) {
                continue;
            }

            let Ok(metadata) = entry.metadata() else {
                continue;
            };
            let is_old_enough = metadata
                .modified()
                .ok()
                .and_then(|modified| SystemTime::now().duration_since(modified).ok())
                .is_some_and(|age| age >= Self::CLEANUP_MIN_FILE_AGE);
            if !is_old_enough {
                continue;
            }

            let byte_count = metadata.len();
            if fs::remove_file(entry.path()).is_ok() {
                result.files_removed += 1;
                result.bytes_freed += byte_count;
            }
        }

        Ok(result)
    }

    fn relocate_thumb_cache_for_key(
        library_id: &str,
        thumb_key: &str,
        old_album_id: i64,
        new_album_id: i64,
    ) -> Result<(), String> {
        for extension in Self::CACHE_EXTENSIONS {
            let old_path = Self::get_thumb_cache_path_for_key(
                library_id,
                old_album_id,
                thumb_key,
                extension,
            )?;
            if !old_path.exists() {
                continue;
            }

            let new_path = Self::get_thumb_cache_path_for_key(
                library_id,
                new_album_id,
                thumb_key,
                extension,
            )?;
            if let Some(parent) = new_path.parent() {
                fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }

            if fs::rename(&old_path, &new_path).is_err() {
                fs::copy(&old_path, &new_path).map_err(|e| e.to_string())?;
                let _ = fs::remove_file(old_path);
            }
        }
        Ok(())
    }

    pub fn get_thumb_keys_in_subtree(folder_path: &str) -> Result<Vec<String>, String> {
        let conn = open_conn()?;
        let mut stmt = conn
            .prepare(
                "SELECT t.thumb_key FROM athumbs t
                 JOIN afiles a ON t.file_id = a.id
                 JOIN afolders b ON a.folder_id = b.id
                 WHERE t.thumb_key IS NOT NULL AND (b.path = ?1 OR b.path LIKE ?2 ESCAPE '\\')",
            )
            .map_err(|e| e.to_string())?;
        let pattern = subtree_like_pattern(folder_path);
        let rows = stmt
            .query_map(params![folder_path, pattern], |row| row.get(0))
            .map_err(|e| e.to_string())?;
        rows.map(|row| row.map_err(|e| e.to_string())).collect()
    }

    pub fn relocate_for_thumb_keys(thumb_keys: &[String], old_album_id: i64, new_album_id: i64) {
        if old_album_id == new_album_id {
            return;
        }

        let library_id = Self::get_current_library_id();
        for thumb_key in thumb_keys {
            if let Err(error) =
                Self::relocate_thumb_cache_for_key(&library_id, thumb_key, old_album_id, new_album_id)
            {
                eprintln!("Error while relocating folder thumbnail cache: {}", error);
            }
        }
    }

    pub fn relocate_for_file(
        file_id: i64,
        old_album_id: i64,
        new_album_id: i64,
    ) -> Result<(), String> {
        if old_album_id == new_album_id {
            return Ok(());
        }

        let Some(thumb_key) = Self::fetch_thumb_key(file_id)? else {
            return Ok(());
        };
        let library_id = Self::get_current_library_id();
        Self::relocate_thumb_cache_for_key(&library_id, &thumb_key, old_album_id, new_album_id)
    }

    /// Create a new thumbnail struct
    fn new_for_library(
        file_id: i64,
        file_path: &str,
        file_type: i64,
        orientation: i32,
        thumbnail_size: u32,
        prefer_embedded_raw_thumbnail: RawDisplayOptions,
        library_id: &str,
        known_duration: Option<u64>,
        seek_percent: Option<u8>,
    ) -> Result<Option<Self>, String> {
        if file_type == 3 {
            if let Some(companion) = crate::t_raw_display::paired_file(file_id, prefer_embedded_raw_thumbnail) {
                if let (Some(companion_id), Some(path)) = (companion.id, companion.file_path.as_deref()) {
                    if let Ok(Some(mut thumb)) = Self::new_for_library(
                        companion_id, path, 1, companion.e_orientation.unwrap_or(1) as i32,
                        thumbnail_size, RawDisplayOptions::default(), library_id, None, None,
                    ) {
                        if thumb.error_code == 0 && thumb.thumb_data.is_some() {
                            thumb.file_id = file_id;
                            thumb.thumb_mtime = Self::get_source_mtime(file_path);
                            thumb.thumb_key = Some(Self::build_thumb_key(library_id, file_id, thumbnail_size,
                                thumb.thumb_mtime, orientation, Some(prefer_embedded_raw_thumbnail)));
                            return Ok(Some(thumb));
                        }
                    }
                }
            }
        }
        let (thumb_data, error_code) = match file_type {
            1 => {
                // image
                if let Some(ext) = t_utils::get_file_extension(file_path) {
                    match ext.to_lowercase().as_str() {
                        "heic" | "heif" | "hif" => {
                            // heic/heif/hif
                            let res = match crate::t_heif::get_heif_thumbnail(
                                file_path,
                                orientation,
                                thumbnail_size,
                            ) {
                                Ok(Some(data)) => (Some(data), 0),
                                Ok(None) => (None, 1), // empty thumb
                                Err(_) => (None, 1),   // error
                            };
                            res
                        }
                        _ => {
                            // other images
                            match t_image::get_image_thumbnail(
                                file_path,
                                orientation,
                                thumbnail_size,
                            ) {
                                Ok(Some(data)) => (Some(data), 0),
                                Ok(None) => (None, 1),
                                Err(_) => (None, 1),
                            }
                        }
                    }
                } else {
                    (None, 1)
                }
            }
            2 => {
                // video
                match t_video::get_video_thumbnail_sync(
                    file_path,
                    thumbnail_size,
                    known_duration,
                    seek_percent,
                ) {
                    Ok(Some(data)) => (Some(data), 0),
                    Ok(None) => (None, 1),
                    Err(_) => (None, 1),
                }
            }
            3 => {
                // raw image
                match t_image::get_raw_thumbnail(
                    file_path,
                    orientation,
                    thumbnail_size,
                    prefer_embedded_raw_thumbnail,
                ) {
                    Ok(Some(data)) => (Some(data), 0),
                    Ok(None) => (None, 1),
                    Err(_) => (None, 1),
                }
            }
            _ => (None, 1),
        };

        let thumb_mtime = Self::get_source_mtime(file_path);
        // Retain the policy key on RAW failures too, so another policy retries
        // while repeated requests for the same broken source remain cached.
        let thumb_key = (file_type == 3 || thumb_data.is_some()).then(|| {
            Self::build_thumb_key(
                library_id,
                file_id,
                thumbnail_size,
                thumb_mtime,
                orientation,
                (file_type == 3).then_some(prefer_embedded_raw_thumbnail),
            )
        });

        Ok(Some(Self {
            id: None,
            file_id,
            error_code,
            thumb_data,
            thumb_key,
            thumb_mtime,
            thumb_size: Some(thumbnail_size as i64),
            updated_at: Some(Self::now_ts()),
            thumb_data_base64: None,
        }))
    }

    // pub fn new(
    //     file_id: i64,
    //     file_path: &str,
    //     file_type: i64,
    //     orientation: i32,
    //     thumbnail_size: u32,
    // ) -> Result<Option<Self>, String> {
    //     let library_id = Self::get_current_library_id();
    //     Self::new_for_library(
    //         file_id,
    //         file_path,
    //         file_type,
    //         orientation,
    //         thumbnail_size,
    //         &library_id,
    //     )
    // }

    /// insert a thumbnail into db
    fn insert(&self) -> Result<usize, String> {
        let conn = open_conn()?;
        let result = conn
            .execute(
                "INSERT OR REPLACE INTO athumbs (file_id, error_code, thumb_data, thumb_key, thumb_mtime, thumb_size, updated_at) 
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    self.file_id,
                    self.error_code,
                    self.thumb_data,
                    self.thumb_key,
                    self.thumb_mtime,
                    self.thumb_size,
                    self.updated_at,
                ],
            )
            .map_err(|e| e.to_string())?;
        Ok(result) // 0: already exists, ignore, 1: inserted
    }

    fn hydrate_output_bytes_for_library(mut thumb: Self, library_id: &str) -> Result<Self, String> {
        if thumb.thumb_data.is_none() {
            if let Some(key) = thumb.thumb_key.as_ref() {
                if let Some(file) = AFile::get_file_info(thumb.file_id)? {
                    if let Some(album_id) = file.album_id {
                        let extension = file
                            .file_path
                            .as_deref()
                            .is_some_and(t_image::should_use_png_thumbnail)
                            .then_some("png")
                            .unwrap_or("jpg");
                        thumb.thumb_data =
                            Self::read_thumb_cache_bytes(library_id, album_id, key, extension)?;
                    }
                }
            }
        }
        thumb.thumb_data_base64 = thumb
            .thumb_data
            .as_ref()
            .map(|data| general_purpose::STANDARD.encode(data));
        Ok(thumb)
    }

    /// fetch a thumbnail from db by file_id
    pub fn fetch(file_id: i64) -> Result<Option<Self>, String> {
        let library_id = Self::get_current_library_id();
        Self::fetch_for_library(file_id, &library_id)
    }

    pub fn fetch_for_library(file_id: i64, library_id: &str) -> Result<Option<Self>, String> {
        let conn = open_conn()?;
        let result = conn
            .query_row(
                "SELECT id, file_id, error_code, thumb_data, thumb_key, thumb_mtime, thumb_size, updated_at
                FROM athumbs WHERE file_id = ?1",
                params![file_id],
                |row| {
                    Ok(Self {
                        id: Some(row.get(0)?),
                        file_id: row.get(1)?,
                        error_code: row.get(2)?,
                        thumb_data: row.get(3)?,
                        thumb_key: row.get(4)?,
                        thumb_mtime: row.get(5)?,
                        thumb_size: row.get(6)?,
                        updated_at: row.get(7)?,
                        thumb_data_base64: None,
                    })
                },
            )
            .optional()
            .map_err(|e| e.to_string())?;
        result
            .map(|thumb| Self::hydrate_output_bytes_for_library(thumb, library_id))
            .transpose()
    }

    pub fn fetch_many(file_ids: &[i64]) -> Result<HashMap<i64, Self>, String> {
        let library_id = Self::get_current_library_id();
        Self::fetch_many_for_library(file_ids, &library_id)
    }

    pub fn fetch_many_for_library(
        file_ids: &[i64],
        library_id: &str,
    ) -> Result<HashMap<i64, Self>, String> {
        if file_ids.is_empty() {
            return Ok(HashMap::new());
        }

        let placeholders = std::iter::repeat("?")
            .take(file_ids.len())
            .collect::<Vec<_>>()
            .join(",");
        let query = format!(
            "SELECT id, file_id, error_code, thumb_data, thumb_key, thumb_mtime, thumb_size, updated_at
            FROM athumbs WHERE file_id IN ({})",
            placeholders
        );
        let conn = open_conn()?;
        let mut stmt = conn.prepare(&query).map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params_from_iter(file_ids.iter()), |row| {
                Ok(Self {
                    id: Some(row.get(0)?),
                    file_id: row.get(1)?,
                    error_code: row.get(2)?,
                    thumb_data: row.get(3)?,
                    thumb_key: row.get(4)?,
                    thumb_mtime: row.get(5)?,
                    thumb_size: row.get(6)?,
                    updated_at: row.get(7)?,
                    thumb_data_base64: None,
                })
            })
            .map_err(|e| e.to_string())?;

        let mut thumbs = HashMap::with_capacity(file_ids.len());
        for row in rows {
            let thumb = Self::hydrate_output_bytes_for_library(
                row.map_err(|e| e.to_string())?,
                library_id,
            )?;
            thumbs.insert(thumb.file_id, thumb);
        }
        Ok(thumbs)
    }

    fn raw_display_is_stale(&self, file_path: &str, orientation: i32, options: RawDisplayOptions) -> bool {
        if t_utils::get_file_type(file_path) != Some(3) {
            return false;
        }
        // Preserve offline-library thumbnails; regenerate once the source returns.
        if !std::path::Path::new(file_path).is_file() { return false; }
        let key = Self::build_thumb_key(&Self::get_current_library_id(), self.file_id,
            self.thumb_size.unwrap_or(512).max(1) as u32, Self::get_source_mtime(file_path),
            orientation, Some(options));
        self.thumb_key.as_deref() != Some(key.as_str())
    }

    fn is_stale(&self, file_path: &str, _thumbnail_size: u32) -> bool {
        // A quality setting change must not invalidate the existing cache while
        // browsing. Explicit refresh actions regenerate at the requested size.
        let current_mtime = Self::get_source_mtime(file_path);
        match (self.thumb_mtime, current_mtime) {
            (Some(cached_mtime), Some(source_mtime)) => cached_mtime != source_mtime,
            (None, Some(_)) => true,
            // The album root may have been temporarily renamed outside Lap.
            // Keep existing thumbnails so they work again when the path returns.
            (_, None) => false,
        }
    }

    /// Whether an explicit album re-scan should regenerate this thumbnail at
    /// the currently selected size and RAW thumbnail source.
    pub fn needs_thumbnail_regeneration(
        file_id: i64,
        thumbnail_size: u32,
        prefer_embedded_raw_thumbnail: RawDisplayOptions,
    ) -> bool {
        Self::fetch(file_id)
            .ok()
            .flatten()
            .is_some_and(|thumbnail| {
                if thumbnail.error_code == 2 {
                    return false;
                }

                let size_changed = thumbnail.thumb_size != Some(thumbnail_size as i64);

                let Ok(Some(file)) = AFile::get_file_info(file_id) else {
                    return size_changed;
                };
                if file.file_type.unwrap_or(0) != 3 {
                    return size_changed;
                }

                if size_changed {
                    return true;
                }

                let expected_key = Self::build_thumb_key(
                    &Self::get_current_library_id(),
                    file_id,
                    thumbnail_size,
                    file.file_path.as_deref().and_then(Self::get_source_mtime),
                    file.e_orientation.unwrap_or(1) as i32,
                    Some(prefer_embedded_raw_thumbnail),
                );
                thumbnail.thumb_key.as_deref() != Some(expected_key.as_str())
            })
    }

    fn fetch_thumb_key(file_id: i64) -> Result<Option<String>, String> {
        let conn = open_conn()?;
        conn.query_row(
            "SELECT thumb_key FROM athumbs WHERE file_id = ?1",
            params![file_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| e.to_string())
    }

    fn persist_cache_and_clear_blob(
        mut thumbnail: Self,
        file_path: &str,
        thumbnail_size: u32,
        orientation: i32,
    ) -> Result<Self, String> {
        let Some(data) = thumbnail.thumb_data.as_ref() else {
            return Self::hydrate_output_bytes_for_library(
                thumbnail,
                &Self::get_current_library_id(),
            );
        };

        let library_id = Self::get_current_library_id();
        let thumb_mtime = Self::get_source_mtime(file_path);
        let now = Self::now_ts();
        let thumb_key = thumbnail.thumb_key.clone().unwrap_or_else(|| {
            Self::build_thumb_key(
                &library_id,
                thumbnail.file_id,
                thumbnail_size,
                thumb_mtime,
                orientation,
                None,
            )
        });

        let album_id = Self::get_file_album_id(thumbnail.file_id)?
            .ok_or_else(|| format!("Album not found for thumbnail file: {}", thumbnail.file_id))?;
        Self::write_thumb_cache_bytes(&library_id, album_id, &thumb_key, data)?;

        let conn = open_conn()?;
        conn.execute(
            "UPDATE athumbs
            SET thumb_key = ?2, thumb_mtime = ?3, thumb_size = ?4, updated_at = ?5, thumb_data = NULL
            WHERE file_id = ?1",
            params![
                thumbnail.file_id,
                thumb_key,
                thumb_mtime,
                thumbnail_size as i64,
                now,
            ],
        )
        .map_err(|e| e.to_string())?;

        thumbnail.thumb_key = Some(thumb_key);
        thumbnail.thumb_mtime = thumb_mtime;
        thumbnail.thumb_size = Some(thumbnail_size as i64);
        thumbnail.updated_at = Some(now);
        Self::hydrate_output_bytes_for_library(thumbnail, &library_id)
    }

    fn ensure_cached(
        thumbnail: Self,
        file_path: &str,
        thumbnail_size: u32,
        orientation: i32,
    ) -> Result<Self, String> {
        if thumbnail.error_code != 0 {
            return Ok(thumbnail);
        }

        if thumbnail.thumb_data.is_some() {
            return Self::persist_cache_and_clear_blob(
                thumbnail,
                file_path,
                thumbnail_size,
                orientation,
            );
        }

        if thumbnail.thumb_key.is_some() {
            return Self::hydrate_output_bytes_for_library(
                thumbnail,
                &Self::get_current_library_id(),
            );
        }

        Ok(thumbnail)
    }

    fn create_cache_backed_thumb_for_library(
        file_id: i64,
        file_path: &str,
        file_type: i64,
        orientation: i32,
        thumbnail_size: u32,
        prefer_embedded_raw_thumbnail: RawDisplayOptions,
        library_id: &str,
        known_duration: Option<u64>,
        seek_percent: Option<u8>,
    ) -> Result<Option<Self>, String> {
        if Self::should_use_original_image(file_id, file_type, thumbnail_size) {
            let athumb = Self {
                id: None,
                file_id,
                error_code: 2,
                thumb_data: None,
                thumb_key: None,
                thumb_mtime: Self::get_source_mtime(file_path),
                thumb_size: Some(thumbnail_size as i64),
                updated_at: Some(Self::now_ts()),
                thumb_data_base64: None,
            };
            athumb.insert()?;
            return Self::fetch_for_library(file_id, library_id);
        }

        let mut athumb = match Self::new_for_library(
            file_id,
            file_path,
            file_type,
            orientation,
            thumbnail_size,
            prefer_embedded_raw_thumbnail,
            library_id,
            known_duration,
            seek_percent,
        ) {
            Ok(Some(athumb)) => athumb,
            _ => Self {
                id: None,
                file_id,
                error_code: 1,
                thumb_data: None,
                thumb_key: None,
                thumb_mtime: Self::get_source_mtime(file_path),
                thumb_size: Some(thumbnail_size as i64),
                updated_at: Some(Self::now_ts()),
                thumb_data_base64: None,
            },
        };

        if athumb.error_code == 0 {
            if let (Some(data), Some(key)) = (athumb.thumb_data.as_ref(), athumb.thumb_key.as_ref())
            {
                let album_id = Self::get_file_album_id(file_id)?
                    .ok_or_else(|| format!("Album not found for thumbnail file: {}", file_id))?;
                Self::write_thumb_cache_bytes(library_id, album_id, key, data)?;
                athumb.thumb_data = None;
            }
        }

        athumb.insert()?;
        Self::fetch_for_library(file_id, library_id)
    }

    fn create_cache_backed_thumb(
        file_id: i64,
        file_path: &str,
        file_type: i64,
        orientation: i32,
        thumbnail_size: u32,
        prefer_embedded_raw_thumbnail: RawDisplayOptions,
        known_duration: Option<u64>,
        seek_percent: Option<u8>,
    ) -> Result<Option<Self>, String> {
        let library_id = Self::get_current_library_id();
        Self::create_cache_backed_thumb_for_library(
            file_id,
            file_path,
            file_type,
            orientation,
            thumbnail_size,
            prefer_embedded_raw_thumbnail,
            &library_id,
            known_duration,
            seek_percent,
        )
    }

    pub fn get_thumb_if_available(
        file_id: i64,
        file_path: &str,
        thumbnail_size: u32,
        orientation: i32,
        prefer_embedded_raw_thumbnail: RawDisplayOptions,
        force_regenerate: bool,
    ) -> Result<Option<Self>, String> {
        if force_regenerate {
            let _ = Self::delete(file_id);
            return Ok(None);
        }

        if let Ok(Some(thumbnail)) = Self::fetch(file_id) {
            if thumbnail.raw_display_is_stale(file_path, orientation, prefer_embedded_raw_thumbnail) {
                let _ = Self::delete(file_id);
                return Ok(None);
            }
            if thumbnail.error_code == 1 {
                if thumbnail.is_stale(file_path, thumbnail_size) {
                    let _ = Self::delete(file_id);
                    return Ok(None);
                }
                return Ok(Some(thumbnail));
            }

            if thumbnail.error_code == 2 {
                return Ok(Some(thumbnail));
            }

            if thumbnail.is_stale(file_path, thumbnail_size) {
                let _ = Self::delete(file_id);
                return Ok(None);
            }

            let hydrated = Self::ensure_cached(thumbnail, file_path, thumbnail_size, orientation)?;
            if hydrated.thumb_data.is_some() {
                return Ok(Some(hydrated));
            }

            let _ = Self::delete(file_id);
        }

        Ok(None)
    }

    pub fn resolve_fetched_thumb_if_available(
        thumbnail: Self,
        file_path: &str,
        thumbnail_size: u32,
        orientation: i32,
        prefer_embedded_raw_thumbnail: RawDisplayOptions,
        force_regenerate: bool,
        trust_cached: bool,
    ) -> Result<Option<Self>, String> {
        if force_regenerate {
            let _ = Self::delete(thumbnail.file_id);
            return Ok(None);
        }

        if thumbnail.raw_display_is_stale(file_path, orientation, prefer_embedded_raw_thumbnail) {
            let _ = Self::delete(thumbnail.file_id);
            return Ok(None);
        }
        if thumbnail.error_code == 1 {
            if thumbnail.is_stale(file_path, thumbnail_size) {
                let _ = Self::delete(thumbnail.file_id);
                return Ok(None);
            }
            return Ok(Some(thumbnail));
        }

        if thumbnail.error_code == 2 {
            return Ok(Some(thumbnail));
        }

        if !trust_cached && thumbnail.is_stale(file_path, thumbnail_size) {
            let _ = Self::delete(thumbnail.file_id);
            return Ok(None);
        }

        let hydrated = Self::ensure_cached(thumbnail, file_path, thumbnail_size, orientation)?;
        if hydrated.thumb_data.is_some() {
            return Ok(Some(hydrated));
        }

        let _ = Self::delete(hydrated.file_id);
        Ok(None)
    }

    pub fn schedule_background_generation_for_library(
        app_handle: tauri::AppHandle,
        file_id: i64,
        file_path: String,
        file_type: i64,
        orientation: i32,
        thumbnail_size: u32,
        prefer_embedded_raw_thumbnail: RawDisplayOptions,
        album_id: i64,
        force_regenerate: bool,
        seek_percent: Option<u8>,
    ) {
        if !Self::try_begin_background_task(file_id, thumbnail_size) {
            return;
        }

        tauri::async_runtime::spawn(async move {
            let Ok(_generation_permit) = thumb_background_generation_permits()
                .acquire_owned()
                .await
            else {
                Self::finish_background_task(file_id, thumbnail_size);
                return;
            };

            let generated = tauri::async_runtime::spawn_blocking(move || {
                let duration = if file_type == 2 {
                    AFile::get_file_info(file_id)
                        .ok()
                        .flatten()
                        .and_then(|f| f.duration.map(|d| d as u64))
                } else {
                    None
                };

                Self::get_or_create_thumb(
                    file_id,
                    &file_path,
                    file_type,
                    orientation,
                    thumbnail_size,
                    prefer_embedded_raw_thumbnail,
                    force_regenerate,
                    duration,
                    seek_percent,
                )
            })
            .await;

            if matches!(generated, Ok(Ok(Some(_)))) && album_id > 0 {
                let _ = app_handle.emit(
                    "thumbnail_ready",
                    serde_json::json!({
                        "album_id": album_id,
                        "file_ids": [file_id],
                        "invalidate": force_regenerate,
                    }),
                );
            }

            Self::finish_background_task(file_id, thumbnail_size);
        });
    }

    /// get or create a thumbnail
    pub fn get_or_create_thumb(
        file_id: i64,
        file_path: &str,
        file_type: i64,
        orientation: i32,
        thumbnail_size: u32,
        prefer_embedded_raw_thumbnail: RawDisplayOptions,
        force_regenerate: bool,
        known_duration: Option<u64>,
        seek_percent: Option<u8>,
    ) -> Result<Option<Self>, String> {
        if force_regenerate {
            let _ = Self::delete(file_id);
        } else if let Some(thumb) =
            Self::get_thumb_if_available(
                file_id,
                file_path,
                thumbnail_size,
                orientation,
                prefer_embedded_raw_thumbnail,
                false,
            )?
        {
            if thumb.error_code != 1 {
                return Ok(Some(thumb));
            }
        }

        let _generation_guard = Self::acquire_generation_guard(file_id, thumbnail_size);

        if !force_regenerate {
            if let Some(hydrated) = Self::get_thumb_if_available(
                file_id,
                file_path,
                thumbnail_size,
                orientation,
                prefer_embedded_raw_thumbnail,
                false,
            )? {
                if hydrated.error_code != 1 {
                    return Ok(Some(hydrated));
                }
            }
        }

        Self::create_cache_backed_thumb(
            file_id,
            file_path,
            file_type,
            orientation,
            thumbnail_size,
            prefer_embedded_raw_thumbnail,
            known_duration,
            seek_percent,
        )
    }

    /// fetch raw thumbnail bytes for protocol handler
    pub fn fetch_raw_for_library(
        file_id: i64,
        library_id: &str,
    ) -> Result<Option<Vec<u8>>, String> {
        let thumb = Self::fetch_for_library(file_id, library_id)?;

        // error_code 2: image is small enough to use the original file directly
        if let Some(ref thumb) = thumb {
            if thumb.error_code == 2 {
                if let Ok(Some(file)) = AFile::get_file_info(file_id) {
                    if let Some(ref file_path) = file.file_path {
                        if let Ok(data) = std::fs::read(file_path) {
                            return Ok(Some(data));
                        }
                    }
                }
            }
        }

        if let Some(thumb) = thumb.filter(|t| t.error_code == 0) {
            if let Some(data) = thumb.thumb_data {
                return Ok(Some(data));
            }

            let file = AFile::get_file_info(file_id)
                .map_err(|e| e.to_string())?
                .ok_or_else(|| format!("File not found for thumbnail: {}", file_id))?;
            let file_path = file
                .file_path
                .ok_or_else(|| format!("File path not found for thumbnail: {}", file_id))?;
            let file_type = file.file_type.unwrap_or(0);
            let orientation = file.e_orientation.unwrap_or(1) as i32;
            let thumbnail_size = thumb.thumb_size.unwrap_or(200).max(1) as u32;
            let prefer_embedded_raw_thumbnail = thumb
                .thumb_key
                .as_deref()
                .map(|key| RawDisplayOptions {
                    mode: if key.starts_with('e') || key.starts_with('b') { crate::t_raw_display::RawPreviewMode::Embedded } else { crate::t_raw_display::RawPreviewMode::Rendered },
                    auto_bright: key.starts_with('a') || key.starts_with('b'),
                    prefer_pair: false,
                })
                .unwrap_or_default();

            return Ok(Self::create_cache_backed_thumb_for_library(
                file_id,
                &file_path,
                file_type,
                orientation,
                thumbnail_size,
                prefer_embedded_raw_thumbnail,
                library_id,
                file.duration.map(|d| d as u64),
                None,
            )?
            .and_then(|thumb| thumb.thumb_data));
        }

        Ok(None)
    }

    /// delete a thumbnail from db
    pub fn delete(file_id: i64) -> Result<usize, String> {
        if let Ok(Some(key)) = Self::fetch_thumb_key(file_id) {
            let library_id = Self::get_current_library_id();
            if let Ok(Some(file)) = AFile::get_file_info(file_id) {
                if let Some(album_id) = file.album_id {
                    Self::delete_thumb_cache_for_key(&library_id, album_id, &key);
                }
            }
        }
        let conn = open_conn()?;
        let result = conn
            .execute("DELETE FROM athumbs WHERE file_id = ?1", params![file_id])
            .map_err(|e| e.to_string())?;
        Ok(result)
    }

    /// get the thumbnail count of the folder
    pub fn get_folder_thumb_count(file_type: i64, folder_id: i64) -> Result<i64, String> {
        let conn = open_conn()?;

        let mut conditions: Vec<String> = Vec::new();
        let mut params: Vec<&dyn rusqlite::ToSql> = Vec::new();

        conditions.push("a.folder_id = ?".to_string());
        params.push(&folder_id);

        if let Some(condition) = AFile::build_file_type_condition(file_type) {
            conditions.push(condition);
        }

        let mut query =
            "SELECT COUNT(b.id) FROM afiles a JOIN athumbs b ON a.id = b.file_id".to_string();
        if !conditions.is_empty() {
            query.push_str(" WHERE ");
            query.push_str(&conditions.join(" AND "));
        }

        let result = conn
            .query_row(&query, rusqlite::params_from_iter(params), |row| row.get(0))
            .map_err(|e| e.to_string())?;

        Ok(result)
    }
}

/// Define the Tag struct
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ATag {
    pub id: i64,
    pub name: String,
    pub count: Option<i64>,
    pub group_id: i64,
    pub group_name: String,
}

#[derive(Debug, Serialize)]
pub struct ATagSelectionCount {
    pub tag_id: i64,
    pub count: i64,
}

#[derive(Debug, Serialize)]
pub struct ATagFileState {
    pub file_id: i64,
    pub has_tags: bool,
}

impl ATag {
    /// Function to construct `Self` from a database row
    fn from_row(row: &rusqlite::Row) -> Result<Self, rusqlite::Error> {
        Ok(Self {
            id: row.get(0)?,
            name: row.get(1)?,
            count: row.get(2)?,
            group_id: row.get(3)?,
            group_name: row.get(4)?,
        })
    }

    pub fn add(name: &str, group_id: Option<i64>) -> Result<Self, String> {
        let trimmed = name.trim();
        if trimmed.is_empty() || trimmed.chars().count() > 255 {
            return Err("Tag name must contain 1–255 characters".to_string());
        }
        let mut conn = open_conn()?;
        let tx = conn.transaction().map_err(|e| e.to_string())?;
        let group_id: i64 = match group_id {
            Some(id) => id,
            None => tx.query_row("SELECT id FROM atag_groups WHERE is_default = 1", [], |r| r.get(0)).map_err(|e| e.to_string())?,
        };
        let exists: bool = tx.query_row("SELECT EXISTS(SELECT 1 FROM atags WHERE name = ? COLLATE NOCASE)", [trimmed], |r| r.get(0)).map_err(|e| e.to_string())?;
        if exists { return Err("A tag with this name already exists".to_string()); }
        tx.execute("INSERT INTO atags(name, group_id) VALUES (?1, ?2)", params![trimmed, group_id]).map_err(|e| e.to_string())?;
        let tag = tx.query_row("SELECT t.id, t.name, 0, t.group_id, g.name FROM atags t JOIN atag_groups g ON g.id = t.group_id WHERE t.id = ?", [tx.last_insert_rowid()], Self::from_row).map_err(|e| e.to_string())?;
        tx.commit().map_err(|e| e.to_string())?;
        Ok(tag)
    }

    /// Get all tags from the db
    pub fn get_all(sort: i64) -> Result<Vec<Self>, String> {
        let conn = open_conn()?;
        // Count-based ordering must match the sidebar badge count (which comes
        // from getQueryCountAndSum with tagId): tagged files excluding Live Photo
        // videos and honoring album exclusions. The displayed count itself is
        // populated lazily, so this expression is only used for ordering.
        let count_expr = format!(
            "(SELECT COUNT(*) FROM afile_tags ft
              JOIN afiles a ON a.id = ft.file_id
              JOIN afolders b ON b.id = a.folder_id
              WHERE ft.tag_id = atags.id AND {}{}{} AND {})",
            AFile::live_photo_companion_exclusion_condition(),
            AFile::album_filter_sql("a"),
            AFile::inaccessible_album_filter("b"),
            AFile::search_exclusion_condition("b"),
        );
        let order_clause = match sort {
            1 => "atags.name DESC".to_string(),
            2 => format!("{count_expr} ASC, atags.name ASC"),
            3 => format!("{count_expr} DESC, atags.name ASC"),
            _ => "atags.name ASC".to_string(),
        };
        let query = format!(
            "SELECT atags.id, atags.name, 0 AS count, atags.group_id, g.name FROM atags JOIN atag_groups g ON g.id = atags.group_id ORDER BY {order_clause}",
        );
        let mut stmt = conn.prepare(query.as_str()).map_err(|e| e.to_string())?;

        let tags_iter = stmt
            .query_map([], Self::from_row)
            .map_err(|e| e.to_string())?;

        let mut tags = Vec::new();
        for tag in tags_iter {
            tags.push(tag.map_err(|e| e.to_string())?);
        }
        Ok(tags)
    }

    /// Count visible files for every tag in one grouped query.
    pub fn get_counts() -> Result<HashMap<i64, i64>, String> {
        let conn = open_conn()?;
        let query = format!(
            "SELECT ft.tag_id, COUNT(DISTINCT a.id)
             FROM afile_tags ft
             JOIN afiles a ON a.id = ft.file_id
             JOIN afolders b ON b.id = a.folder_id
             WHERE {}{}{} AND {}
             GROUP BY ft.tag_id",
            AFile::live_photo_companion_exclusion_condition(),
            AFile::album_filter_sql("a"),
            AFile::inaccessible_album_filter("b"),
            AFile::search_exclusion_condition("b"),
        );
        let mut stmt = conn.prepare(&query).map_err(|e| e.to_string())?;
        let rows = stmt.query_map([], |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?)))
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<HashMap<_, _>, _>>().map_err(|e| e.to_string())
    }

    /// Count distinct visible files for every tag group.
    pub fn get_group_counts() -> Result<HashMap<i64, i64>, String> {
        Self::get_group_counts_on(&*open_conn()?)
    }

    fn get_group_counts_on(conn: &Connection) -> Result<HashMap<i64, i64>, String> {
        let query = format!(
            "SELECT t.group_id, COUNT(DISTINCT a.id)
             FROM afile_tags ft
             JOIN atags t ON t.id = ft.tag_id
             JOIN afiles a ON a.id = ft.file_id
             JOIN afolders b ON b.id = a.folder_id
             WHERE {}{}{} AND {}
             GROUP BY t.group_id",
            AFile::live_photo_companion_exclusion_condition(),
            AFile::album_filter_sql("a"),
            AFile::inaccessible_album_filter("b"),
            AFile::search_exclusion_condition("b"),
        );
        let mut stmt = conn.prepare(&query).map_err(|e| e.to_string())?;
        let rows = stmt.query_map([], |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?)))
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<HashMap<_, _>, _>>().map_err(|e| e.to_string())
    }

    /// Get tag name by id
    pub fn get_name(tag_id: i64, include_group: bool) -> Result<String, String> {
        let conn = open_conn()?;
        let result = conn
            .query_row(
                "SELECT CASE WHEN ?2 THEN g.name || ' > ' || t.name ELSE t.name END
                 FROM atags t LEFT JOIN atag_groups g ON g.id = t.group_id WHERE t.id = ?1",
                params![tag_id, include_group],
                |row| row.get(0),
            )
            .map_err(|e| e.to_string())?;
        Ok(result)
    }

    /// Get all tags for a specific file
    pub fn get_tags_for_file(file_id: i64) -> Result<Vec<Self>, String> {
        let conn = open_conn()?;
        let mut stmt = conn
            .prepare(
                "SELECT t.id, t.name, 0 as count, t.group_id, g.name
                FROM atags t
                JOIN atag_groups g ON g.id = t.group_id
                INNER JOIN afile_tags ft ON t.id = ft.tag_id
                WHERE ft.file_id = ?1
                ORDER BY t.name ASC",
            )
            .map_err(|e| e.to_string())?;

        let tags_iter = stmt
            .query_map(params![file_id], Self::from_row)
            .map_err(|e| e.to_string())?;

        let mut tags = Vec::new();
        for tag in tags_iter {
            tags.push(tag.map_err(|e| e.to_string())?);
        }
        Ok(tags)
    }

    /// Add a tag to a file.
    pub fn add_tag_to_file(file_id: i64, tag_id: i64) -> Result<(), String> {
        let conn = open_conn()?;
        conn.execute(
            "INSERT OR IGNORE INTO afile_tags (file_id, tag_id) VALUES (?1, ?2)",
            params![file_id, tag_id],
        )
        .map_err(|e| e.to_string())?;

        // Update has_tags in afiles table
        conn.execute(
            "UPDATE afiles SET has_tags = 1 WHERE id = ?1",
            params![file_id],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Remove a tag from a file
    pub fn remove_tag_from_file(file_id: i64, tag_id: i64) -> Result<usize, String> {
        let conn = open_conn()?;
        let result = conn
            .execute(
                "DELETE FROM afile_tags WHERE file_id = ?1 AND tag_id = ?2",
                params![file_id, tag_id],
            )
            .map_err(|e| e.to_string())?;

        // Check if the file still has any tags
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM afile_tags WHERE file_id = ?1",
                params![file_id],
                |row| row.get(0),
            )
            .map_err(|e| e.to_string())?;

        if count == 0 {
            // If no tags left, set has_tags to false
            conn.execute(
                "UPDATE afiles SET has_tags = 0 WHERE id = ?1",
                params![file_id],
            )
            .map_err(|e| e.to_string())?;
        }
        Ok(result)
    }

    pub fn get_selection_counts(file_ids: &[i64]) -> Result<Vec<ATagSelectionCount>, String> {
        if file_ids.is_empty() {
            return Ok(Vec::new());
        }

        let mut conn = open_conn()?;
        let tx = conn.transaction().map_err(|e| e.to_string())?;
        tx.execute(
            "CREATE TEMP TABLE IF NOT EXISTS selected_file_ids (id INTEGER PRIMARY KEY)",
            [],
        )
        .map_err(|e| e.to_string())?;
        tx.execute("DELETE FROM selected_file_ids", [])
            .map_err(|e| e.to_string())?;
        populate_selected_file_ids(&tx, file_ids)?;

        let counts = {
            let mut stmt = tx
                .prepare(
                    "SELECT ft.tag_id, COUNT(*)
                     FROM afile_tags ft
                     INNER JOIN selected_file_ids selected ON selected.id = ft.file_id
                     GROUP BY ft.tag_id",
                )
                .map_err(|e| e.to_string())?;
            let rows = stmt
                .query_map([], |row| {
                    Ok(ATagSelectionCount {
                        tag_id: row.get(0)?,
                        count: row.get(1)?,
                    })
                })
                .map_err(|e| e.to_string())?;
            rows.collect::<Result<Vec<_>, _>>()
                .map_err(|e| e.to_string())?
        };

        tx.execute("DELETE FROM selected_file_ids", [])
            .map_err(|e| e.to_string())?;
        tx.commit().map_err(|e| e.to_string())?;
        Ok(counts)
    }

    pub fn apply_to_files(
        file_ids: &[i64],
        add_tag_ids: &[i64],
        remove_tag_ids: &[i64],
    ) -> Result<Vec<ATagFileState>, String> {
        if file_ids.is_empty() {
            return Ok(Vec::new());
        }

        let mut conn = open_conn()?;
        let tx = conn.transaction().map_err(|e| e.to_string())?;
        {
            let mut add_stmt = tx
                .prepare_cached(
                    "INSERT OR IGNORE INTO afile_tags (file_id, tag_id) VALUES (?1, ?2)",
                )
                .map_err(|e| e.to_string())?;
            let mut remove_stmt = tx
                .prepare_cached("DELETE FROM afile_tags WHERE file_id = ?1 AND tag_id = ?2")
                .map_err(|e| e.to_string())?;
            for file_id in file_ids {
                for tag_id in add_tag_ids {
                    add_stmt
                        .execute(params![file_id, tag_id])
                        .map_err(|e| e.to_string())?;
                }
                for tag_id in remove_tag_ids {
                    remove_stmt
                        .execute(params![file_id, tag_id])
                        .map_err(|e| e.to_string())?;
                }
            }
        }

        let mut states = Vec::with_capacity(file_ids.len());
        {
            let mut update_stmt = tx
                .prepare_cached(
                    "UPDATE afiles
                    SET has_tags = EXISTS (
                        SELECT 1 FROM afile_tags WHERE afile_tags.file_id = afiles.id
                    )
                    WHERE id = ?1",
                )
                .map_err(|e| e.to_string())?;
            let mut state_stmt = tx
                .prepare_cached("SELECT COALESCE(has_tags, 0) FROM afiles WHERE id = ?1")
                .map_err(|e| e.to_string())?;
            for file_id in file_ids {
                update_stmt
                    .execute(params![file_id])
                    .map_err(|e| e.to_string())?;
                let has_tags = state_stmt
                    .query_row(params![file_id], |row| row.get(0))
                    .map_err(|e| e.to_string())?;
                states.push(ATagFileState {
                    file_id: *file_id,
                    has_tags,
                });
            }
        }

        tx.commit().map_err(|e| e.to_string())?;
        Ok(states)
    }

    /// Delete a tag from the database. This will also remove all its associations with files.
    pub fn delete(tag_id: i64) -> Result<usize, String> {
        let mut conn = open_conn()?;
        let tx = conn.transaction().map_err(|e| e.to_string())?;
        let file_ids = {
            let mut stmt = tx
                .prepare("SELECT file_id FROM afile_tags WHERE tag_id = ?1")
                .map_err(|e| e.to_string())?;
            let rows = stmt
                .query_map(params![tag_id], |row| row.get::<_, i64>(0))
                .map_err(|e| e.to_string())?;
            rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())?
        };
        let result = tx
            .execute("DELETE FROM atags WHERE id = ?1", params![tag_id])
            .map_err(|e| e.to_string())?;
        if result > 0 {
            let mut stmt = tx
                .prepare_cached(
                    "UPDATE afiles
                     SET has_tags = EXISTS (
                         SELECT 1 FROM afile_tags WHERE afile_tags.file_id = afiles.id
                     )
                     WHERE id = ?1",
                )
                .map_err(|e| e.to_string())?;
            for file_id in file_ids {
                stmt.execute(params![file_id]).map_err(|e| e.to_string())?;
            }
        }
        tx.commit().map_err(|e| e.to_string())?;
        Ok(result)
    }

    /// Rename a tag
    pub fn rename(tag_id: i64, new_name: &str) -> Result<usize, String> {
        let trimmed = new_name.trim();
        if trimmed.is_empty() {
            return Err("Tag name cannot be empty".to_string());
        }
        let conn = open_conn()?;
        let existing: Option<i64> = conn
            .query_row(
                "SELECT id FROM atags WHERE name = ?1 COLLATE NOCASE AND id != ?2",
                params![trimmed, tag_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| e.to_string())?;
        if existing.is_some() {
            return Err("A tag with this name already exists".to_string());
        }
        let result = conn
            .execute(
                "UPDATE atags SET name = ?1 WHERE id = ?2",
                params![trimmed, tag_id],
            )
            .map_err(|e| e.to_string())?;
        Ok(result)
    }
}

/// Person struct for face recognition
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Person {
    pub id: i64,
    pub name: Option<String>,
    pub count: Option<i64>,
    pub thumbnail: Option<String>, // Base64 encoded face thumbnail
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PersonPage {
    pub persons: Vec<Person>,
    pub has_more: bool,
    pub total: usize,
    pub visible_total: Option<usize>,
    pub selected_person_visible: Option<bool>,
}

/// Pagination request for the People sidebar.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PersonPageRequest {
    pub sort: i64,
    pub offset: usize,
    pub limit: usize,
    pub search: String,
    pub refresh_summary: Option<PersonPageRefreshSummary>,
}

/// Aggregate data requested when refreshing the People sidebar.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PersonPageRefreshSummary {
    pub selected_person_id: Option<i64>,
}

impl Person {
    /// Get all persons with face counts and pre-stored thumbnail
    /// Optimized: single query, no runtime image processing
    pub fn get_all(sort: i64) -> Result<Vec<Self>, String> {
        let conn = open_conn()?;

        // Single query with JOIN for count, directly fetch pre-stored thumbnail
        let query = "
            SELECT p.id, p.name, COUNT(f.id) as count, p.thumbnail
            FROM persons p
            LEFT JOIN faces f ON f.person_id = p.id
            GROUP BY p.id
            ORDER BY {order_clause}
        ";
        let order_clause = match sort {
            1 => "p.name DESC",
            2 => "count ASC, p.name ASC",
            3 => "count DESC, p.name ASC",
            _ => "p.name ASC",
        };
        let query = query.replace("{order_clause}", order_clause);
        let mut stmt = conn.prepare(&query).map_err(|e| e.to_string())?;

        let persons_iter = stmt
            .query_map([], |row| {
                let thumb_data: Option<Vec<u8>> = row.get(3)?;
                let thumbnail = thumb_data
                    .as_ref()
                    .map(|data| general_purpose::STANDARD.encode(data));
                Ok(Self {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    count: row.get(2)?,
                    thumbnail,
                })
            })
            .map_err(|e| e.to_string())?;

        let mut persons = Vec::new();
        for person_result in persons_iter {
            persons.push(person_result.map_err(|e| e.to_string())?);
        }

        Ok(persons)
    }

    /// Get a single person's face thumbnail by id (Base64).
    /// Prefers the pre-stored thumbnail; generates it on-the-fly if missing.
    pub fn get_thumbnail(person_id: i64) -> Result<Option<String>, String> {
        let conn = open_conn()?;

        // Prefer the pre-stored thumbnail.
        let stored: Option<Vec<u8>> = conn
            .query_row(
                "SELECT thumbnail FROM persons WHERE id = ?1",
                params![person_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| e.to_string())?;

        if let Some(data) = stored {
            return Ok(Some(general_purpose::STANDARD.encode(data)));
        }

        // Fallback: generate from the person's best face (e.g. the thumbnail
        // was never computed during clustering, or the cover file was missing).
        let cover_face_id: Option<i64> = conn
            .query_row(
                "SELECT cover_face_id FROM persons WHERE id = ?1",
                params![person_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| e.to_string())?
            .flatten();

        let generated = Self::generate_thumbnail(&conn, person_id, cover_face_id)?;
        if let Some(data) = &generated {
            let _ = conn.execute(
                "UPDATE persons SET thumbnail = ?1 WHERE id = ?2",
                params![data, person_id],
            );
        }
        Ok(generated.map(|data| general_purpose::STANDARD.encode(data)))
    }

    pub fn get_page(request: &PersonPageRequest) -> Result<PersonPage, String> {
        let conn = open_conn()?;
        let limit = request.limit.clamp(1, 100);
        let search = request.search.trim();
        let search_pattern = format!(
            "%{}%",
            search
                .replace('\\', "\\\\")
                .replace('%', "\\%")
                .replace('_', "\\_")
        );
        let visible_file_conditions = format!(
            "{}{} AND {} AND {}",
            AFile::album_filter_sql("a"),
            AFile::inaccessible_album_filter("b"),
            AFile::search_exclusion_condition("b"),
            AFile::live_photo_companion_exclusion_condition(),
        );
        let visible_person_condition = format!(
            "EXISTS (SELECT 1 FROM faces f JOIN afiles a ON a.id = f.file_id JOIN afolders b ON b.id = a.folder_id WHERE f.person_id = p.id{})",
            visible_file_conditions,
        );
        let total: i64 = conn
            .query_row(
                &format!(
                    "SELECT COUNT(*) FROM persons p WHERE (?1 = '' OR COALESCE(p.name, '') LIKE ?2 ESCAPE '\\' COLLATE NOCASE) AND {visible_person_condition}"
                ),
                params![search, search_pattern],
                |row| row.get(0),
            )
            .map_err(|e| e.to_string())?;
        let visible_total = if request.refresh_summary.is_some() {
            if search.is_empty() {
                Some(total as usize)
            } else {
                Some(
                    conn.query_row(
                        &format!("SELECT COUNT(*) FROM persons p WHERE {visible_person_condition}"),
                        [],
                        |row| row.get::<_, i64>(0),
                    )
                    .map_err(|e| e.to_string())? as usize,
                )
            }
        } else {
            None
        };
        let selected_person_visible = request.refresh_summary.as_ref()
            .and_then(|summary| summary.selected_person_id)
            .map(|person_id| {
                conn.query_row(
                    &format!(
                        "SELECT EXISTS(SELECT 1 FROM persons p WHERE p.id = ?1 AND {visible_person_condition})"
                    ),
                    params![person_id],
                    |row| row.get::<_, i64>(0),
                )
                .map(|visible| visible != 0)
                .map_err(|e| e.to_string())
            })
            .transpose()?;
        let name_asc = "rtrim(COALESCE(p.name, ''), '0123456789') COLLATE NOCASE ASC, CAST(substr(COALESCE(p.name, ''), length(rtrim(COALESCE(p.name, ''), '0123456789')) + 1) AS INTEGER) ASC, p.name ASC, p.id ASC";
        let name_desc = "rtrim(COALESCE(p.name, ''), '0123456789') COLLATE NOCASE DESC, CAST(substr(COALESCE(p.name, ''), length(rtrim(COALESCE(p.name, ''), '0123456789')) + 1) AS INTEGER) DESC, p.name DESC, p.id ASC";
        let order_clause = match request.sort {
            1 => name_desc,
            2 => "count ASC, p.name ASC, p.id ASC",
            3 => "count DESC, p.name ASC, p.id ASC",
            _ => name_asc,
        };
        let query = format!(
            "SELECT p.id, p.name, COUNT(DISTINCT a.id) as count, p.thumbnail
             FROM persons p
             JOIN faces f ON f.person_id = p.id
             JOIN afiles a ON a.id = f.file_id
             JOIN afolders b ON b.id = a.folder_id
             WHERE (?1 = '' OR COALESCE(p.name, '') LIKE ?2 ESCAPE '\\' COLLATE NOCASE){}
             GROUP BY p.id
             ORDER BY {order_clause}
             LIMIT ?3 OFFSET ?4",
            visible_file_conditions,
        );
        let mut stmt = conn.prepare(&query).map_err(|e| e.to_string())?;
        let persons_iter = stmt
            .query_map(params![search, search_pattern, (limit + 1) as i64, request.offset as i64], |row| {
                let thumb_data: Option<Vec<u8>> = row.get(3)?;
                Ok(Self {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    count: row.get(2)?,
                    thumbnail: thumb_data
                        .as_ref()
                        .map(|data| general_purpose::STANDARD.encode(data)),
                })
            })
            .map_err(|e| e.to_string())?;

        let mut persons = persons_iter
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        let has_more = persons.len() > limit;
        if has_more {
            persons.pop();
        }

        Ok(PersonPage {
            persons,
            has_more,
            total: total as usize,
            visible_total,
            selected_person_visible,
        })
    }

    /// Generate thumbnail for a person from their cover face or best quality face
    /// Returns the thumbnail as JPEG bytes
    fn generate_thumbnail(
        conn: &Connection,
        person_id: i64,
        cover_face_id: Option<i64>,
    ) -> Result<Option<Vec<u8>>, String> {
        // 1. Determine which face to use
        let get_best_face = || -> Result<i64, rusqlite::Error> {
            conn.query_row(
                "SELECT id FROM faces WHERE person_id = ?1 ORDER BY (json_extract(bbox, '$.width') * json_extract(bbox, '$.height')) DESC LIMIT 1",
                params![person_id],
                |row| row.get(0),
            )
        };

        let face_id = if let Some(fid) = cover_face_id {
            // Validate that cover_face_id actually belongs to this person
            let is_valid: bool = conn
                .query_row(
                    "SELECT EXISTS(SELECT 1 FROM faces WHERE id = ?1 AND person_id = ?2)",
                    params![fid, person_id],
                    |row| row.get(0),
                )
                .unwrap_or(false);

            if is_valid {
                fid
            } else {
                match get_best_face() {
                    Ok(fid) => fid,
                    Err(_) => return Ok(None),
                }
            }
        } else {
            match get_best_face() {
                Ok(fid) => fid,
                Err(_) => return Ok(None),
            }
        };

        // 2. Get face info and file info
        let query = "
            SELECT f.id, faces.bbox, f.width, f.height, f.e_orientation, f.name, fd.path
            FROM faces 
            JOIN afiles f ON faces.file_id = f.id
            JOIN afolders fd ON f.folder_id = fd.id
            WHERE faces.id = ?1
        ";

        let row: Result<
            (
                i64,
                String,
                Option<u32>,
                Option<u32>,
                Option<i32>,
                String,
                String,
            ),
            _,
        > = conn.query_row(query, params![face_id], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
                row.get(5)?,
                row.get(6)?,
            ))
        });

        let (file_id, bbox_json, orig_w_opt, orig_h_opt, orientation_opt, file_name, folder_path) =
            match row {
                Ok(r) => r,
                Err(_) => return Ok(None),
            };

        let bbox: FaceBBox = match serde_json::from_str(&bbox_json) {
            Ok(b) => b,
            Err(_) => return Ok(None),
        };

        let orientation = orientation_opt.unwrap_or(1); // Default to Normal

        // 3. Load Image (Original or Thumbnail)
        let full_path = std::path::Path::new(&folder_path).join(&file_name);

        // Helper to load and rotate original image
        let load_original = || -> Option<(image::DynamicImage, u32, u32)> {
            let mut dyn_img = image::open(&full_path).ok()?;
            dyn_img = match orientation {
                3 => dyn_img.rotate180(),
                6 => dyn_img.rotate90(),
                8 => dyn_img.rotate270(),
                _ => dyn_img,
            };
            let (w, h) = dyn_img.dimensions();
            Some((dyn_img, w, h))
        };

        // Helper to load thumbnail from cache-backed thumbnail storage
        let load_thumbnail = || -> Option<(image::DynamicImage, u32, u32)> {
            let data = AThumb::fetch(file_id).ok()??.thumb_data?;
            let img = image::load_from_memory(&data).ok()?;
            let (w, h) = img.dimensions();
            Some((img, w, h))
        };

        let (mut img, img_w, img_h) = match load_original().or_else(load_thumbnail) {
            Some(res) => res,
            None => return Ok(None),
        };

        // 4. Calculate Dimensions & BBox
        let (ref_w, ref_h) = if let (Some(ow), Some(oh)) = (orig_w_opt, orig_h_opt) {
            match orientation {
                6 | 8 => (oh, ow),
                _ => (ow, oh),
            }
        } else {
            (img_w, img_h)
        };

        let transformed_bbox = if orig_w_opt.is_some() && orig_h_opt.is_some() {
            let orig_w = orig_w_opt.unwrap();
            let orig_h = orig_h_opt.unwrap();
            match orientation {
                6 => FaceBBox {
                    x: orig_h as f32 - bbox.y - bbox.height,
                    y: bbox.x,
                    width: bbox.height,
                    height: bbox.width,
                },
                8 => FaceBBox {
                    x: bbox.y,
                    y: orig_w as f32 - bbox.x - bbox.width,
                    width: bbox.height,
                    height: bbox.width,
                },
                3 => FaceBBox {
                    x: orig_w as f32 - bbox.x - bbox.width,
                    y: orig_h as f32 - bbox.y - bbox.height,
                    width: bbox.width,
                    height: bbox.height,
                },
                _ => bbox,
            }
        } else {
            bbox
        };

        // 5. Crop and Resize
        let scale_x = img_w as f32 / ref_w as f32;
        let scale_y = img_h as f32 / ref_h as f32;
        let expansion = 0.2;

        let face_x = transformed_bbox.x * scale_x;
        let face_y = transformed_bbox.y * scale_y;
        let face_w = transformed_bbox.width * scale_x;
        let face_h = transformed_bbox.height * scale_y;

        let crop_x = (face_x - face_w * expansion).max(0.0) as u32;
        let crop_y = (face_y - face_h * expansion).max(0.0) as u32;
        let crop_w =
            (face_w * (1.0 + 2.0 * expansion)).min((img_w.saturating_sub(crop_x)) as f32) as u32;
        let crop_h =
            (face_h * (1.0 + 2.0 * expansion)).min((img_h.saturating_sub(crop_y)) as f32) as u32;

        if crop_w > 0 && crop_h > 0 && crop_x < img_w && crop_y < img_h {
            // Use crop() for DynamicImage type consistency
            let mut cropped = img.crop(
                crop_x,
                crop_y,
                crop_w.min(img_w - crop_x),
                crop_h.min(img_h - crop_y),
            );

            // Resize if too large
            let max_thumb_size = 200;
            if cropped.width() > max_thumb_size || cropped.height() > max_thumb_size {
                cropped = cropped.resize(
                    max_thumb_size,
                    max_thumb_size,
                    image::imageops::FilterType::Lanczos3,
                );
            }

            // Encode to JPEG (with RGB8 conversion for transparency support)
            let rgb_img = cropped.to_rgb8();
            let mut buffer = Cursor::new(Vec::new());
            if rgb_img.write_to(&mut buffer, ImageFormat::Jpeg).is_ok() {
                return Ok(Some(buffer.into_inner()));
            }
        }

        Ok(None)
    }

    /// Update thumbnail for a specific person
    #[allow(dead_code)]
    pub fn update_thumbnail(person_id: i64) -> Result<(), String> {
        let conn = open_conn()?;

        // Get cover_face_id for this person
        let cover_face_id: Option<i64> = conn
            .query_row(
                "SELECT cover_face_id FROM persons WHERE id = ?1",
                params![person_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| e.to_string())?
            .flatten();

        // Generate thumbnail
        let thumbnail = Self::generate_thumbnail(&conn, person_id, cover_face_id)?;

        // Update in database
        conn.execute(
            "UPDATE persons SET thumbnail = ?1 WHERE id = ?2",
            params![thumbnail, person_id],
        )
        .map_err(|e| e.to_string())?;

        Ok(())
    }

    /// Update thumbnails for all persons (called after clustering)
    pub fn update_all_thumbnails() -> Result<(), String> {
        let conn = open_conn()?;

        // Get all person IDs and their cover_face_ids
        let mut stmt = conn
            .prepare("SELECT id, cover_face_id FROM persons")
            .map_err(|e| e.to_string())?;

        let persons: Vec<(i64, Option<i64>)> = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        // Generate and update thumbnail for each person
        for (person_id, cover_face_id) in persons {
            if let Ok(Some(thumbnail)) = Self::generate_thumbnail(&conn, person_id, cover_face_id) {
                let _ = conn.execute(
                    "UPDATE persons SET thumbnail = ?1 WHERE id = ?2",
                    params![thumbnail, person_id],
                );
            }
        }

        Ok(())
    }

    /// Rename a person
    pub fn rename(person_id: i64, new_name: &str) -> Result<usize, String> {
        let conn = open_conn()?;
        let result = conn
            .execute(
                "UPDATE persons SET name = ?1 WHERE id = ?2",
                params![new_name, person_id],
            )
            .map_err(|e| e.to_string())?;
        Ok(result)
    }

    /// Delete a person (faces will have person_id set to NULL)
    pub fn delete(person_id: i64) -> Result<usize, String> {
        let conn = open_conn()?;

        // First, unlink all faces from this person
        conn.execute(
            "UPDATE faces SET person_id = NULL WHERE person_id = ?1",
            params![person_id],
        )
        .map_err(|e| e.to_string())?;

        // Then delete the person
        let result = conn
            .execute("DELETE FROM persons WHERE id = ?1", params![person_id])
            .map_err(|e| e.to_string())?;
        Ok(result)
    }

    /// Create a new person (usually from face clustering)
    pub fn create(name: Option<&str>) -> Result<i64, String> {
        let conn = open_conn()?;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        conn.execute(
            "INSERT INTO persons (name, created_at) VALUES (?1, ?2)",
            params![name, now],
        )
        .map_err(|e| e.to_string())?;

        Ok(conn.last_insert_rowid())
    }
}

/// Face struct for storing detected faces
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Face {
    pub id: i64,
    pub file_id: i64,
    pub bbox: String, // JSON: {"x": f32, "y": f32, "width": f32, "height": f32, "confidence": f32}
    pub embedding: Option<Vec<u8>>, // 512-dimensional float32 embedding as bytes
    pub person_id: Option<i64>,
    pub person_name: Option<String>,
    pub created_at: i64,
}

impl Face {
    /// Add a new face using an existing connection (avoids repeated open_conn during batch indexing)
    pub fn add_with_conn(
        conn: &Connection,
        file_id: i64,
        bbox: &str,
        embedding: &[f32],
    ) -> Result<i64, String> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        // Convert f32 embedding to bytes
        let embedding_bytes: Vec<u8> = embedding.iter().flat_map(|f| f.to_le_bytes()).collect();

        conn.execute(
            "INSERT INTO faces (file_id, bbox, embedding, created_at) VALUES (?1, ?2, ?3, ?4)",
            params![file_id, bbox, embedding_bytes, now],
        )
        .map_err(|e| e.to_string())?;

        Ok(conn.last_insert_rowid())
    }

    /// Check if a file already has faces detected
    /// Check if a file has faces
    #[allow(dead_code)]
    pub fn file_has_faces(file_id: i64) -> Result<bool, String> {
        let conn = open_conn()?;
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM faces WHERE file_id = ?1",
                params![file_id],
                |row| row.get(0),
            )
            .map_err(|e| e.to_string())?;
        Ok(count > 0)
    }

    /// Reset all face data: delete all faces and persons
    pub fn reset_all() -> Result<(), String> {
        let conn = open_conn()?;

        // Use a transaction
        conn.execute("BEGIN TRANSACTION", params![])
            .map_err(|e| e.to_string())?;

        if let Err(e) = conn.execute("DELETE FROM faces", params![]) {
            let _ = conn.execute("ROLLBACK", params![]);
            return Err(e.to_string());
        }

        if let Err(e) = conn.execute("DELETE FROM persons", params![]) {
            let _ = conn.execute("ROLLBACK", params![]);
            return Err(e.to_string());
        }

        // Reset has_faces flag in afiles
        if let Err(e) = conn.execute("UPDATE afiles SET has_faces = 0", params![]) {
            let _ = conn.execute("ROLLBACK", params![]);
            return Err(e.to_string());
        }

        // Vacuum to reclaim space (optional, but good for reset)
        // Note: VACUUM cannot be run inside a transaction in some SQLite versions/modes,
        // but here we just commit first.

        conn.execute("COMMIT", params![])
            .map_err(|e| e.to_string())?;

        Ok(())
    }

    /// Get faces for a specific file
    pub fn get_for_file(file_id: i64) -> Result<Vec<Self>, String> {
        let conn = open_conn()?;
        let mut stmt = conn
            .prepare(
                "SELECT f.id, f.file_id, f.bbox, f.embedding, f.person_id, f.created_at, p.name 
                 FROM faces f
                 LEFT JOIN persons p ON f.person_id = p.id
                 WHERE f.file_id = ?1",
            )
            .map_err(|e| e.to_string())?;

        let faces = stmt
            .query_map([file_id], |row| {
                Ok(Self {
                    id: row.get(0)?,
                    file_id: row.get(1)?,
                    bbox: row.get(2)?,
                    embedding: row.get(3)?,
                    person_id: row.get(4)?,
                    created_at: row.get(5)?,
                    person_name: row.get(6)?,
                })
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        Ok(faces)
    }

    /// Get slim face data for clustering: (face_id, file_id, embedding_bytes)
    /// Avoids loading full Face structs (bbox JSON, person_id, created_at) to reduce memory
    pub fn get_all_for_clustering() -> Result<Vec<(i64, i64, Option<Vec<u8>>)>, String> {
        let conn = open_conn()?;
        let mut stmt = conn
            .prepare("SELECT id, file_id, embedding FROM faces")
            .map_err(|e| e.to_string())?;

        let faces = stmt
            .query_map([], |row| {
                let id: i64 = row.get(0)?;
                let file_id: i64 = row.get(1)?;
                let embedding: Option<Vec<u8>> = row.get(2)?;
                Ok((id, file_id, embedding))
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        Ok(faces)
    }

    /// Reset all face assignments and delete all persons (for re-clustering)
    pub fn reset_all_assignments() -> Result<(), String> {
        let conn = open_conn()?;

        // Clear all person_id from faces
        conn.execute("UPDATE faces SET person_id = NULL", [])
            .map_err(|e| e.to_string())?;

        // Delete all persons
        conn.execute("DELETE FROM persons", [])
            .map_err(|e| e.to_string())?;

        Ok(())
    }

    /// Assign a face to a person
    pub fn assign_to_person(face_id: i64, person_id: i64) -> Result<usize, String> {
        let conn = open_conn()?;
        let result = conn
            .execute(
                "UPDATE faces SET person_id = ?1 WHERE id = ?2",
                params![person_id, face_id],
            )
            .map_err(|e| e.to_string())?;
        Ok(result)
    }

    /// Get all image file IDs that haven't been processed for faces yet
    /// Returns: Vec<(id, file_path, width, height)>
    pub fn get_unprocessed_image_files() -> Result<Vec<(i64, String, i64, i64)>, String> {
        let conn = open_conn()?;
        let mut stmt = conn
            .prepare(
                "SELECT a.id, f.path || '/' || a.name as file_path, a.width, a.height
                 FROM afiles a 
                 JOIN afolders f ON a.folder_id = f.id
                 WHERE a.file_type = 1 
                   AND (a.has_faces IS NULL OR a.has_faces = 0)
                   AND a.width IS NOT NULL AND a.height IS NOT NULL
                 ORDER BY a.id",
            )
            .map_err(|e| e.to_string())?;

        let files = stmt
            .query_map([], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
            })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        Ok(files)
    }

    /// Mark a file as scanned using an existing connection
    pub fn mark_scanned_with_conn(
        conn: &Connection,
        file_id: i64,
        status: i32,
    ) -> Result<(), String> {
        conn.execute(
            "UPDATE afiles SET has_faces = ?1 WHERE id = ?2",
            params![status, file_id],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Get statistics for face indexing
    /// Returns (processed_count, total_faces)
    pub fn get_stats() -> Result<(usize, usize), String> {
        let conn = open_conn()?;

        // Count processed files (has_faces > 0)
        let processed: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM afiles WHERE has_faces > 0 AND file_type = 1",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);

        // Count total faces
        let faces: i64 = conn
            .query_row("SELECT COUNT(*) FROM faces", [], |row| row.get(0))
            .unwrap_or(0);

        Ok((processed as usize, faces as usize))
    }

    /// Get full statistics for face indexing
    /// Returns (total_images, processed_images, unprocessed_images, total_faces)
    pub fn get_stats_full() -> Result<(usize, usize, usize, usize), String> {
        let conn = open_conn()?;
        let visible_file_conditions = format!(
            "{}{} AND {} AND {}",
            AFile::album_filter_sql("a"),
            AFile::inaccessible_album_filter("b"),
            AFile::search_exclusion_condition("b"),
            AFile::live_photo_companion_exclusion_condition(),
        );

        let total: i64 = conn
            .query_row(
                &format!("SELECT COUNT(*) FROM afiles a JOIN afolders b ON b.id = a.folder_id WHERE a.file_type = 1{}", visible_file_conditions),
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);

        let processed: i64 = conn
            .query_row(
                &format!("SELECT COUNT(*) FROM afiles a JOIN afolders b ON b.id = a.folder_id WHERE a.has_faces > 0 AND a.file_type = 1{}", visible_file_conditions),
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);

        let faces: i64 = conn
            .query_row(
                &format!("SELECT COUNT(*) FROM faces f JOIN afiles a ON a.id = f.file_id JOIN afolders b ON b.id = a.folder_id WHERE {}", visible_file_conditions.trim_start_matches(" AND ")),
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);

        let unprocessed = total - processed;

        Ok((
            total as usize,
            processed as usize,
            unprocessed as usize,
            faces as usize,
        ))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ACamera {
    pub make: String,
    pub models: Vec<String>,
    pub counts: Vec<i64>,
}

fn sort_labeled_counts(labels: &mut Vec<String>, counts: &mut Vec<i64>, sort: i64) {
    let mut pairs: Vec<(String, i64)> = labels.drain(..).zip(counts.drain(..)).collect();

    match sort {
        1 => pairs.sort_by(|a, b| b.0.cmp(&a.0)),
        2 => pairs.sort_by(|a, b| a.1.cmp(&b.1).then_with(|| a.0.cmp(&b.0))),
        3 => pairs.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0))),
        _ => pairs.sort_by(|a, b| a.0.cmp(&b.0)),
    }

    for (label, count) in pairs {
        labels.push(label);
        counts.push(count);
    }
}

impl ACamera {
    // get all camera makes and models from db
    pub fn get_from_db(sort: i64) -> Result<Vec<Self>, String> {
        let conn = open_conn()?;
        let query = format!("SELECT UPPER(a.e_make), a.e_model, count(a.id) as count
            FROM afiles a
            JOIN afolders b ON a.folder_id = b.id
            WHERE a.e_make IS NOT NULL AND a.e_model IS NOT NULL
                AND a.id NOT IN (
                    SELECT live_photo_video_id FROM afiles WHERE live_photo_video_id IS NOT NULL
                ){}{} AND {}
            GROUP BY UPPER(a.e_make), a.e_model
            ORDER BY UPPER(a.e_make), a.e_model", AFile::inaccessible_album_filter("b"), AFile::album_filter_sql("a"), AFile::search_exclusion_condition("b"));

        let mut stmt = conn.prepare(query.as_str()).map_err(|e| e.to_string())?;

        let rows = stmt
            .query_map(params![], |row| {
                let make: String = row.get(0)?;
                let model: String = row.get(1)?;
                let count: i64 = row.get(2)?;
                Ok((make, model, count))
            })
            .map_err(|e| e.to_string())?;

        let mut hash_map: HashMap<String, (Vec<String>, Vec<i64>)> = HashMap::new();

        for row_result in rows {
            let (make, model, count) = row_result.map_err(|e| e.to_string())?;
            let entry = hash_map
                .entry(make)
                .or_insert_with(|| (Vec::new(), Vec::new()));
            entry.0.push(model); // Push model to Vec<String>
            entry.1.push(count); // Push count to Vec<i64>
        }

        let mut cameras: Vec<Self> = hash_map
            .into_iter()
            .map(|(make, (mut models, mut counts))| {
                sort_labeled_counts(&mut models, &mut counts, sort);
                Self {
                    make,
                    models,
                    counts,
                }
            })
            .collect();

        match sort {
            1 => cameras.sort_by(|a, b| b.make.cmp(&a.make)),
            2 => cameras.sort_by(|a, b| {
                a.counts
                    .iter()
                    .sum::<i64>()
                    .cmp(&b.counts.iter().sum::<i64>())
                    .then_with(|| a.make.cmp(&b.make))
            }),
            3 => cameras.sort_by(|a, b| {
                b.counts
                    .iter()
                    .sum::<i64>()
                    .cmp(&a.counts.iter().sum::<i64>())
                    .then_with(|| a.make.cmp(&b.make))
            }),
            _ => cameras.sort_by(|a, b| a.make.cmp(&b.make)),
        }

        Ok(cameras)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ALens {
    pub make: String,
    pub models: Vec<String>,
    pub counts: Vec<i64>,
}

impl ALens {
    // get all lens makes and models from db
    pub fn get_from_db(sort: i64) -> Result<Vec<Self>, String> {
        let conn = open_conn()?;
        let query = format!("SELECT UPPER(a.e_lens_make), a.e_lens_model, count(a.id) as count
            FROM afiles a
            JOIN afolders b ON a.folder_id = b.id
            WHERE a.e_lens_make IS NOT NULL AND a.e_lens_model IS NOT NULL
                AND a.id NOT IN (
                    SELECT live_photo_video_id FROM afiles WHERE live_photo_video_id IS NOT NULL
                ){}{} AND {}
            GROUP BY UPPER(a.e_lens_make), a.e_lens_model
            ORDER BY UPPER(a.e_lens_make), a.e_lens_model", AFile::inaccessible_album_filter("b"), AFile::album_filter_sql("a"), AFile::search_exclusion_condition("b"));

        let mut stmt = conn.prepare(query.as_str()).map_err(|e| e.to_string())?;

        let rows = stmt
            .query_map(params![], |row| {
                let make: String = row.get(0)?;
                let model: String = row.get(1)?;
                let count: i64 = row.get(2)?;
                Ok((make, model, count))
            })
            .map_err(|e| e.to_string())?;

        let mut hash_map: HashMap<String, (Vec<String>, Vec<i64>)> = HashMap::new();

        for row_result in rows {
            let (make, model, count) = row_result.map_err(|e| e.to_string())?;
            let entry = hash_map
                .entry(make)
                .or_insert_with(|| (Vec::new(), Vec::new()));
            entry.0.push(model);
            entry.1.push(count);
        }

        let mut lenses: Vec<Self> = hash_map
            .into_iter()
            .map(|(make, (mut models, mut counts))| {
                sort_labeled_counts(&mut models, &mut counts, sort);
                Self {
                    make,
                    models,
                    counts,
                }
            })
            .collect();

        match sort {
            1 => lenses.sort_by(|a, b| b.make.cmp(&a.make)),
            2 => lenses.sort_by(|a, b| {
                a.counts
                    .iter()
                    .sum::<i64>()
                    .cmp(&b.counts.iter().sum::<i64>())
                    .then_with(|| a.make.cmp(&b.make))
            }),
            3 => lenses.sort_by(|a, b| {
                b.counts
                    .iter()
                    .sum::<i64>()
                    .cmp(&a.counts.iter().sum::<i64>())
                    .then_with(|| a.make.cmp(&b.make))
            }),
            _ => lenses.sort_by(|a, b| a.make.cmp(&b.make)),
        }

        Ok(lenses)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ALocation {
    pub cc: String,
    pub admin1: String,
    pub names: Vec<String>,
    pub counts: Vec<i64>,
}

impl ALocation {
    // get all location admin1 and names from db
    pub fn get_from_db(sort: i64) -> Result<Vec<Self>, String> {
        let conn = open_conn()?;

        let query = format!("SELECT COALESCE(a.geo_cc, ''), a.geo_admin1, a.geo_name, count(a.id) as count
            FROM afiles a
            JOIN afolders b ON a.folder_id = b.id
            WHERE COALESCE(a.geo_admin1, '') <> '' AND COALESCE(a.geo_name, '') <> ''
                AND a.id NOT IN (
                    SELECT live_photo_video_id FROM afiles WHERE live_photo_video_id IS NOT NULL
                ){}{} AND {}
            GROUP BY a.geo_cc, a.geo_admin1, a.geo_name
            ORDER BY a.geo_cc, a.geo_admin1, a.geo_name", AFile::inaccessible_album_filter("b"), AFile::album_filter_sql("a"), AFile::search_exclusion_condition("b"));

        let mut stmt = conn.prepare(query.as_str()).map_err(|e| e.to_string())?;

        let rows = stmt
            .query_map(params![], |row| {
                let cc: String = row.get(0)?;
                let admin1: String = row.get(1)?;
                let name: String = row.get(2)?;
                let count: i64 = row.get(3)?;
                Ok((cc, admin1, name, count))
            })
            .map_err(|e| e.to_string())?;

        let mut hash_map: HashMap<(String, String), (Vec<String>, Vec<i64>)> = HashMap::new();

        for row in rows {
            let (cc, admin1, name, count) = row.map_err(|e| e.to_string())?;
            let entry = hash_map
                .entry((cc, admin1))
                .or_insert_with(|| (Vec::new(), Vec::new()));
            entry.0.push(name); // Push name to Vec<String>
            entry.1.push(count); // Push count to Vec<i64>
        }

        let mut locations: Vec<Self> = hash_map
            .into_iter()
            .map(|((cc, admin1), (mut names, mut counts))| {
                sort_labeled_counts(&mut names, &mut counts, sort);
                Self {
                    cc,
                    admin1,
                    names,
                    counts,
                }
            })
            .collect();

        // Sort the locations by admin1
        match sort {
            1 => locations.sort_by(|a, b| b.admin1.cmp(&a.admin1)),
            2 => locations.sort_by(|a, b| {
                a.counts
                    .iter()
                    .sum::<i64>()
                    .cmp(&b.counts.iter().sum::<i64>())
                    .then_with(|| a.admin1.cmp(&b.admin1))
            }),
            3 => locations.sort_by(|a, b| {
                b.counts
                    .iter()
                    .sum::<i64>()
                    .cmp(&a.counts.iter().sum::<i64>())
                    .then_with(|| a.admin1.cmp(&b.admin1))
            }),
            _ => locations.sort_by(|a, b| a.admin1.cmp(&b.admin1)),
        }

        Ok(locations)
    }
}

/// A map thumbnail cluster. `file_id` identifies a representative image for
/// the cluster, while `count` contains every matching photo in the cell.
#[derive(Debug, Serialize, Deserialize)]
pub struct AGpsMapPoint {
    pub lat: f64,
    pub lon: f64,
    pub count: i64,
    pub file_id: i64,
}

impl AGpsMapPoint {
    pub fn get_map_points_from_db(params: &QueryParams) -> Result<Vec<Self>, String> {
        let conn = open_conn()?;
        let (joins, where_clause, sql_params) = AFile::build_search_query_parts(params);
        let query = format!(
            "SELECT AVG(gps_latitude) AS lat, AVG(gps_longitude) AS lon, COUNT(*) AS cnt, MIN(id) AS file_id
             FROM (
                 SELECT a.id, a.gps_latitude, a.gps_longitude
                 FROM afiles a
                 LEFT JOIN afolders b ON a.folder_id = b.id
                 LEFT JOIN albums c ON b.album_id = c.id
                 {}{} AND a.gps_latitude IS NOT NULL AND a.gps_longitude IS NOT NULL
                 GROUP BY a.id
             )
             GROUP BY ROUND(gps_latitude, 2), ROUND(gps_longitude, 2)",
            joins, where_clause,
        );
        let final_params: Vec<&dyn ToSql> = sql_params.iter().map(|p| p.as_ref()).collect();
        let mut stmt = conn.prepare(&query).map_err(|e| e.to_string())?;
        let points = stmt
            .query_map(final_params.as_slice(), |row| {
                Ok(Self {
                    lat: row.get(0)?,
                    lon: row.get(1)?,
                    count: row.get(2)?,
                    file_id: row.get(3)?,
                })
            })
            .map_err(|e| e.to_string())?
            .filter_map(|r| r.ok())
            .collect();
        Ok(points)
    }
}

/// get connection to the db
static CONN_POOL: Mutex<Vec<(String, Connection)>> = Mutex::new(Vec::new());
/// Path of the library db currently known to be corrupt. Corruption state is stored per-path
/// (not a bare bool) so `is_database_corrupted()` is always relative to the *current* library:
/// when the current library changes (switch / remove / hide), a stale mark for another path no
/// longer matches, so a healthy library is never falsely reported as corrupt.
static CORRUPTED_DB_PATH: Mutex<Option<String>> = Mutex::new(None);

/// Message returned when the current library is flagged corrupt. Shared so callers (e.g.
/// switch_library) can decide from the returned error itself instead of re-reading the global flag.
pub const DB_CORRUPTED_MSG: &str = "Database is corrupted. Please switch to another library.";

/// Lock the corruption mark, recovering the guard if the mutex was poisoned. The guarded value is
/// a plain `Option<String>` that is always safe to read, so a poison must never fail *open* into
/// reporting a known-corrupt library as healthy.
fn corrupted_path_guard() -> std::sync::MutexGuard<'static, Option<String>> {
    CORRUPTED_DB_PATH
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// Cheap, path-relative corruption check for the hot `open_conn` path: the caller has already
/// resolved the current db path, so this does no config read (just a mutex + string compare).
fn is_path_corrupted(path: &str) -> bool {
    corrupted_path_guard().as_deref() == Some(path)
}

pub fn is_database_corrupted() -> bool {
    match t_storage::get_current_db_path() {
        Ok(path) => is_path_corrupted(&path),
        Err(e) => {
            // This bool is the sole signal behind the startup guards and the UI banner; a transient
            // config-read failure must be diagnosable rather than silently reported as healthy.
            eprintln!("is_database_corrupted: failed to resolve current db path: {}", e);
            false
        }
    }
}

fn mark_db_corrupted(path: &str) {
    *corrupted_path_guard() = Some(path.to_string());
}

fn clear_db_corrupted(path: &str) {
    let mut guard = corrupted_path_guard();
    if guard.as_deref() == Some(path) {
        *guard = None;
    }
}

/// Check existing data pages before migrations; never modify or rebuild the database.
fn database_is_corrupt(path: &str) -> Result<bool, String> {
    let check = Connection::open_with_flags(path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
        .and_then(|conn| {
            conn.query_row("PRAGMA quick_check(1)", [], |row| row.get::<_, String>(0))
        });
    match check {
        Ok(result) => Ok(result != "ok"),
        Err(error) if matches!(error.sqlite_error_code(),
            Some(rusqlite::ErrorCode::DatabaseCorrupt | rusqlite::ErrorCode::NotADatabase)
        ) => Ok(true),
        Err(error) => Err(error.to_string()),
    }
}

/// Detect corruption errors surfaced by initialization after the integrity check.
fn is_corruption_error(err: &str) -> bool {
    let err = err.to_lowercase();
    err.contains("database disk image is malformed") || err.contains("file is not a database")
}

/// A pooled connection that returns to the global pool on Drop.
pub(crate) struct PooledConn(Option<(String, Connection)>, t_storage::DbConnectionLease);

impl Drop for PooledConn {
    fn drop(&mut self) {
        if let Some(entry) = self.0.take() {
            if let Ok(mut pool) = CONN_POOL.lock() {
                pool.push(entry);
            }
        }
    }
}

impl Deref for PooledConn {
    type Target = Connection;
    fn deref(&self) -> &Connection {
        &self.0.as_ref().unwrap().1
    }
}

impl DerefMut for PooledConn {
    fn deref_mut(&mut self) -> &mut Connection {
        &mut self.0.as_mut().unwrap().1
    }
}

fn setup_conn(conn: &Connection) -> Result<(), String> {
    conn.busy_timeout(Duration::from_secs(5))
        .map_err(|e| format!("Failed to set SQLite busy timeout: {}", e))?;
    conn.query_row("PRAGMA journal_mode = WAL", [], |row| {
        row.get::<_, String>(0)
    })
    .map_err(|e| format!("Failed to enable WAL mode: {}", e))?;
    conn.execute("PRAGMA synchronous = NORMAL", [])
        .map_err(|e| format!("Failed to set SQLite synchronous mode: {}", e))?;
    conn.execute("PRAGMA foreign_keys = ON", [])
        .map_err(|e| format!("Failed to enable foreign keys: {}", e))?;
    Ok(())
}

fn create_conn() -> Result<(String, Connection), String> {
    let path = t_storage::get_current_db_path()
        .map_err(|e| format!("Failed to get the database file path: {}", e))?;
    let conn = Connection::open(&path)
        .map_err(|e| format!("Failed to open database connection: {}", e))?;
    setup_conn(&conn)?;
    Ok((path, conn))
}

pub(crate) fn clear_conn_pool() {
    if let Ok(mut pool) = CONN_POOL.lock() {
        pool.clear();
    }
}

pub(crate) fn open_conn() -> Result<PooledConn, String> {
    let lease = t_storage::DbConnectionLease::acquire()?;
    let current_path = t_storage::get_current_db_path()
        .map_err(|e| format!("Failed to get the database file path: {}", e))?;
    if is_path_corrupted(&current_path) {
        return Err(DB_CORRUPTED_MSG.to_string());
    }
    if let Ok(mut pool) = CONN_POOL.lock() {
        // Only reuse connections pointing to the same DB file
        while let Some((path, conn)) = pool.pop() {
            if path == current_path {
                return Ok(PooledConn(Some((path, conn)), lease));
            }
            // Stale connection for a different library — drop it
        }
    }
    Ok(PooledConn(Some(create_conn()?), lease))
}

/// create all tables if not exists
pub fn create_db() -> Result<(), String> {
    let _lease = t_storage::DbConnectionLease::acquire()?;
    let path = t_storage::get_current_db_path()?;
    // Re-evaluating this library: clear any stale corrupt mark for it before checking.
    clear_db_corrupted(&path);
    if Path::new(&path).exists() {
        // An inconclusive pre-check (Err) must NOT abort: a read-only quick_check can fail on a
        // perfectly healthy WAL db that needs recovery after an unclean shutdown. Fall through to
        // create_db_internal, which opens read-write (running WAL recovery + migrations) and whose
        // is_corruption_error fallback below still catches genuine corruption.
        match database_is_corrupt(&path) {
            Ok(true) => {
                eprintln!("create_db: corruption detected for '{}'", path);
                mark_db_corrupted(&path);
                return Err(DB_CORRUPTED_MSG.to_string());
            }
            Ok(false) => {}
            Err(e) => {
                eprintln!("create_db: integrity pre-check skipped ({}); falling back to migration open", e)
            }
        }
    }
    // Catch page-level corruption that surfaces while the migrations open the db read-write, so
    // the flag is raised and the UI shows the switch-library banner instead of raw errors.
    match create_db_internal() {
        Ok(()) => Ok(()),
        Err(err) if is_corruption_error(&err) => {
            eprintln!("create_db: database corruption detected for '{}': {}", path, err);
            mark_db_corrupted(&path);
            Err(DB_CORRUPTED_MSG.to_string())
        }
        Err(err) => Err(err),
    }
}

fn create_db_internal() -> Result<(), String> {
    let conn = open_conn()?;

    // albums table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS albums (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            path TEXT NOT NULL,
            created_at INTEGER,
            modified_at INTEGER,
            display_order_id INTEGER,
            cover_file_id INTEGER,
            description TEXT,
            indexed INTEGER DEFAULT 0,
            total INTEGER DEFAULT 0,
            skipped_count INTEGER NOT NULL DEFAULT 0,
            skipped_size INTEGER NOT NULL DEFAULT 0,
            failed_count INTEGER NOT NULL DEFAULT 0,
            failed_size INTEGER NOT NULL DEFAULT 0,
            merged_count INTEGER NOT NULL DEFAULT 0,
            merged_size INTEGER NOT NULL DEFAULT 0,
            last_scan_time INTEGER DEFAULT 0
        )",
        [],
    )
    .map_err(|e| e.to_string())?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_albums_name ON albums(name)",
        [],
    )
    .map_err(|e| e.to_string())?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_albums_path ON albums(path)",
        [],
    )
    .map_err(|e| e.to_string())?;

    // folders table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS afolders (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            album_id INTEGER NOT NULL,
            name TEXT NOT NULL,
            path TEXT NOT NULL,
            created_at INTEGER,
            modified_at INTEGER,
            is_favorite INTEGER,
            is_excluded_from_search INTEGER DEFAULT 0,
            has_subfolders INTEGER,
            inode INTEGER,
            FOREIGN KEY (album_id) REFERENCES albums(id) ON DELETE CASCADE
        )",
        [],
    )
    .map_err(|e| e.to_string())?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_afolders_album_id ON afolders(album_id)",
        [],
    )
    .map_err(|e| e.to_string())?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_afolders_name ON afolders(name)",
        [],
    )
    .map_err(|e| e.to_string())?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_afolders_path ON afolders(path)",
        [],
    )
    .map_err(|e| e.to_string())?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_afolders_is_favorite ON afolders(is_favorite)",
        [],
    )
    .map_err(|e| e.to_string())?;

    // files table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS afiles (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            folder_id INTEGER NOT NULL,
            name TEXT NOT NULL,
            name_pinyin TEXT,
            size INTEGER NOT NULL,
            file_type INTEGER,
            format_label TEXT,
            created_at INTEGER,
            modified_at INTEGER,
            inode INTEGER,
            taken_date INTEGER,
            width INTEGER,
            height INTEGER,
            duration INTEGER,
            is_favorite INTEGER,
            rating INTEGER NOT NULL DEFAULT 0,
            culling_flag INTEGER NOT NULL DEFAULT 0,
            rotate INTEGER,
            comments TEXT,
            has_tags INTEGER,
            has_faces INTEGER DEFAULT 0,
            e_make TEXT,
            e_model TEXT,
            e_date_time TEXT,
            e_software TEXT,
            e_artist TEXT,
            e_copyright TEXT,
            e_description TEXT,
            e_lens_make TEXT,
            e_lens_model TEXT,
            e_exposure_bias TEXT,
            e_exposure_time TEXT,
            e_f_number TEXT,
            e_focal_length TEXT,
            e_iso_speed TEXT,
            e_flash TEXT,
            e_orientation INTEGER,
            gps_latitude REAL,
            gps_longitude REAL,
            gps_altitude REAL,
            geo_name TEXT,
            geo_admin1 TEXT,
            geo_admin2 TEXT,
            geo_cc TEXT,
            embeds BLOB,
            last_scan_time INTEGER DEFAULT 0,
            content_identifier TEXT,
            media_subtype TEXT,
            live_photo_video_id INTEGER,
            motion_photo_offset INTEGER,
            FOREIGN KEY (folder_id) REFERENCES afolders(id) ON DELETE CASCADE
        )",
        [],
    )
    .map_err(|e| e.to_string())?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_afiles_folder_id ON afiles(folder_id)",
        [],
    )
    .map_err(|e| e.to_string())?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_afiles_name ON afiles(name)",
        [],
    )
    .map_err(|e| e.to_string())?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_afiles_name_pinyin ON afiles(name_pinyin)",
        [],
    )
    .map_err(|e| e.to_string())?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_afiles_file_type ON afiles(file_type)",
        [],
    )
    .map_err(|e| e.to_string())?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_afiles_taken_date ON afiles(taken_date)",
        [],
    )
    .map_err(|e| e.to_string())?;
    // Map viewport queries filter by both coordinates. Keep non-geotagged media
    // out of this index so large libraries stay responsive while panning.
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_afiles_gps_coordinates
         ON afiles(gps_latitude, gps_longitude)
         WHERE gps_latitude IS NOT NULL AND gps_longitude IS NOT NULL",
        [],
    )
    .map_err(|e| e.to_string())?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_afiles_is_favorite ON afiles(is_favorite)",
        [],
    )
    .map_err(|e| e.to_string())?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_afiles_has_tags ON afiles(has_tags)",
        [],
    )
    .map_err(|e| e.to_string())?;

    // Migration: Add has_faces column if it doesn't exist
    // We try to add it, if it fails it likely exists.
    // Ideally we should check strict versioning but for now this is robust enough for simple addition.
    let _ = conn.execute(
        "ALTER TABLE afiles ADD COLUMN has_faces INTEGER DEFAULT 0",
        [],
    );
    let _ = conn.execute(
        "ALTER TABLE afiles ADD COLUMN rating INTEGER NOT NULL DEFAULT 0",
        [],
    );
    let _ = conn.execute(
        "ALTER TABLE afiles ADD COLUMN culling_flag INTEGER NOT NULL DEFAULT 0",
        [],
    );

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_afiles_rating ON afiles(rating)",
        [],
    )
    .map_err(|e| e.to_string())?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_afiles_culling_flag ON afiles(culling_flag)",
        [],
    )
    .map_err(|e| e.to_string())?;

    // Create index for has_faces
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_afiles_has_faces ON afiles(has_faces)",
        [],
    )
    .map_err(|e| e.to_string())?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_afiles_make_model ON afiles(e_make, e_model)",
        [],
    )
    .map_err(|e| e.to_string())?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_afiles_lens_make_model ON afiles(e_lens_make, e_lens_model)",
        [],
    )
    .map_err(|e| e.to_string())?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_afiles_geo_name ON afiles(geo_name)",
        [],
    )
    .map_err(|e| e.to_string())?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_afiles_geo_admin1 ON afiles(geo_admin1)",
        [],
    )
    .map_err(|e| e.to_string())?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_afiles_geo_admin2 ON afiles(geo_admin2)",
        [],
    )
    .map_err(|e| e.to_string())?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_afiles_geo_cc ON afiles(geo_cc)",
        [],
    )
    .map_err(|e| e.to_string())?;

    // file thumbnail table
    // NOTE: New columns (thumb_key, thumb_mtime, thumb_size, updated_at) are added
    // by migration v3. They are included here so that fresh databases get the full
    // schema immediately; for existing databases CREATE TABLE IF NOT EXISTS is a
    // no-op and migration v3 will ALTER TABLE to add them.
    conn.execute(
        "CREATE TABLE IF NOT EXISTS athumbs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            file_id INTEGER NOT NULL UNIQUE,
            error_code INTEGER NOT NULL,
            thumb_data BLOB,
            thumb_key TEXT,
            thumb_mtime INTEGER,
            thumb_size INTEGER,
            updated_at INTEGER,
            FOREIGN KEY (file_id) REFERENCES afiles(id) ON DELETE CASCADE
        )",
        [],
    )
    .map_err(|e| e.to_string())?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_athumbs_file_id ON athumbs(file_id)",
        [],
    )
    .map_err(|e| e.to_string())?;
    // thumb_key index: may fail on pre-migration DBs where the column doesn't
    // exist yet. Migration v3 will create it after adding the column.
    let _ = conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_athumbs_thumb_key ON athumbs(thumb_key)",
        [],
    );

    // tags table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS atags (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL UNIQUE
        )",
        [],
    )
    .map_err(|e| e.to_string())?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_atags_name ON atags(name)",
        [],
    )
    .map_err(|e| e.to_string())?;

    // file_tags table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS afile_tags (
            file_id INTEGER NOT NULL,
            tag_id INTEGER NOT NULL,
            PRIMARY KEY (file_id, tag_id),
            FOREIGN KEY (file_id) REFERENCES afiles(id) ON DELETE CASCADE,
            FOREIGN KEY (tag_id) REFERENCES atags(id) ON DELETE CASCADE
        )",
        [],
    )
    .map_err(|e| e.to_string())?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_afile_tags_file_id ON afile_tags(file_id)",
        [],
    )
    .map_err(|e| e.to_string())?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_afile_tags_tag_id ON afile_tags(tag_id)",
        [],
    )
    .map_err(|e| e.to_string())?;

    // collections table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS acollections (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            sort_order INTEGER NOT NULL DEFAULT 0,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        )",
        [],
    )
    .map_err(|e| e.to_string())?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_acollections_sort ON acollections(sort_order, id)",
        [],
    )
    .map_err(|e| e.to_string())?;

    // collection files table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS acollections_files (
            collection_id INTEGER NOT NULL,
            file_id INTEGER NOT NULL,
            added_at INTEGER NOT NULL,
            PRIMARY KEY (collection_id, file_id),
            FOREIGN KEY (collection_id) REFERENCES acollections(id) ON DELETE CASCADE,
            FOREIGN KEY (file_id) REFERENCES afiles(id) ON DELETE CASCADE
        )",
        [],
    )
    .map_err(|e| e.to_string())?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_acollections_files_file ON acollections_files(file_id)",
        [],
    )
    .map_err(|e| e.to_string())?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_acollections_files_collection_added ON acollections_files(collection_id, added_at DESC, file_id)",
        [],
    )
    .map_err(|e| e.to_string())?;

    // persons table (for face recognition)
    conn.execute(
        "CREATE TABLE IF NOT EXISTS persons (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT,
            cover_face_id INTEGER,
            thumbnail BLOB,
            created_at INTEGER
        )",
        [],
    )
    .map_err(|e| e.to_string())?;

    // Migration: add thumbnail column if not exists (for existing databases)
    let _ = conn.execute("ALTER TABLE persons ADD COLUMN thumbnail BLOB", []);
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_persons_name ON persons(name)",
        [],
    )
    .map_err(|e| e.to_string())?;

    // faces table (for face recognition)
    conn.execute(
        "CREATE TABLE IF NOT EXISTS faces (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            file_id INTEGER NOT NULL,
            bbox TEXT,
            embedding BLOB,
            person_id INTEGER,
            created_at INTEGER,
            FOREIGN KEY (file_id) REFERENCES afiles(id) ON DELETE CASCADE,
            FOREIGN KEY (person_id) REFERENCES persons(id) ON DELETE SET NULL
        )",
        [],
    )
    .map_err(|e| e.to_string())?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_faces_file_id ON faces(file_id)",
        [],
    )
    .map_err(|e| e.to_string())?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_faces_person_id ON faces(person_id)",
        [],
    )
    .map_err(|e| e.to_string())?;

    // file hashes table (for deduplication)
    conn.execute(
        "CREATE TABLE IF NOT EXISTS file_hashes (
            file_id INTEGER PRIMARY KEY,
            hash TEXT NOT NULL,
            file_size INTEGER NOT NULL,
            mtime INTEGER NOT NULL,
            computed_at INTEGER NOT NULL,
            FOREIGN KEY (file_id) REFERENCES afiles(id) ON DELETE CASCADE
        )",
        [],
    )
    .map_err(|e| e.to_string())?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_file_hashes_hash_size ON file_hashes(hash, file_size)",
        [],
    )
    .map_err(|e| e.to_string())?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_file_hashes_mtime ON file_hashes(mtime)",
        [],
    )
    .map_err(|e| e.to_string())?;

    // duplicate groups table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS duplicate_groups (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            hash TEXT NOT NULL,
            file_size INTEGER NOT NULL,
            file_count INTEGER NOT NULL,
            total_size INTEGER NOT NULL,
            reviewed INTEGER NOT NULL DEFAULT 0,
            updated_at INTEGER NOT NULL
        )",
        [],
    )
    .map_err(|e| e.to_string())?;
    conn.execute(
        "CREATE UNIQUE INDEX IF NOT EXISTS uidx_duplicate_groups_hash_size ON duplicate_groups(hash, file_size)",
        [],
    )
    .map_err(|e| e.to_string())?;

    // duplicate group items table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS duplicate_group_items (
            group_id INTEGER NOT NULL,
            file_id INTEGER NOT NULL,
            is_keep INTEGER NOT NULL DEFAULT 0,
            is_selected INTEGER NOT NULL DEFAULT 0,
            score REAL NOT NULL DEFAULT 0,
            PRIMARY KEY (group_id, file_id),
            FOREIGN KEY (group_id) REFERENCES duplicate_groups(id) ON DELETE CASCADE,
            FOREIGN KEY (file_id) REFERENCES afiles(id) ON DELETE CASCADE
        )",
        [],
    )
    .map_err(|e| e.to_string())?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_dup_items_group ON duplicate_group_items(group_id)",
        [],
    )
    .map_err(|e| e.to_string())?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_dup_items_file ON duplicate_group_items(file_id)",
        [],
    )
    .map_err(|e| e.to_string())?;

    // Run schema migrations after base tables are ensured.
    crate::t_migration::check_and_migrate(&conn)?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_afiles_content_identifier ON afiles(content_identifier)",
        [],
    )
    .map_err(|e| e.to_string())?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_afiles_live_photo_video_id ON afiles(live_photo_video_id)",
        [],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[cfg(test)]
mod tag_group_query_tests {
    use super::*;

    #[test]
    fn group_filter_and_counts_deduplicate_and_honor_visibility() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("CREATE TABLE albums(id INTEGER PRIMARY KEY, path TEXT, file_types INTEGER, small_image_filter INTEGER, excluded_folders TEXT);
            INSERT INTO albums VALUES(1, '/', 7, NULL, '[]');
            CREATE TABLE afolders(id INTEGER PRIMARY KEY, album_id INTEGER, path TEXT, is_excluded_from_search INTEGER);
            INSERT INTO afolders VALUES(1, 1, '/visible', 0), (2, 1, '/excluded', 1);
            CREATE TABLE afiles(id INTEGER PRIMARY KEY, folder_id INTEGER, live_photo_video_id INTEGER, width INTEGER, height INTEGER);
            INSERT INTO afiles VALUES(1,1,NULL,1000,1000), (2,1,NULL,100,100), (3,1,NULL,1000,1000), (4,2,NULL,1000,1000), (5,1,3,1000,1000), (6,1,NULL,1000,1000);
            ALTER TABLE afiles ADD COLUMN file_type INTEGER DEFAULT 1;
            CREATE TABLE atags(id INTEGER PRIMARY KEY, group_id INTEGER);
            INSERT INTO atags VALUES(10,1), (11,1), (12,2);
            CREATE TABLE afile_tags(file_id INTEGER, tag_id INTEGER, PRIMARY KEY(file_id,tag_id));
            INSERT INTO afile_tags VALUES(1,10), (1,11), (2,10), (3,10), (4,10), (5,10), (6,12);").unwrap();
        let params: QueryParams = serde_json::from_value(serde_json::json!({
            "searchFileName":"", "searchFileType":0, "sortType":0, "sortOrder":0,
            "searchAllSubfolders":"", "searchFolder":"", "startDate":0, "endDate":0,
            "calendarSort":0, "make":"", "model":"", "lensMake":"", "lensModel":"",
            "locationAdmin1":"", "locationName":"", "isFavorite":false, "rating":-1,
            "tagId":0, "personId":0, "tagGroupId":1
        })).unwrap();
        for (filter, expected) in [(160, vec![1_i64, 5]), (0, vec![1_i64, 2, 5])] {
            conn.execute("UPDATE albums SET small_image_filter=?1", params![filter]).unwrap();
            let (joins, conditions, values) = AFile::build_search_query_parts(&params);
            let mut stmt = conn.prepare(&format!("SELECT a.id FROM afiles a JOIN afolders b ON b.id = a.folder_id {joins} {conditions} ORDER BY a.id")).unwrap();
            let ids: Vec<i64> = stmt.query_map(rusqlite::params_from_iter(values.iter()), |r| r.get(0)).unwrap().collect::<Result<_, _>>().unwrap();
            assert_eq!(ids, expected);
            let counts = ATag::get_group_counts_on(&conn).unwrap();
            assert_eq!(counts[&1], expected.len() as i64);
            assert_eq!(counts[&2], 1);
        }
    }
}

#[cfg(test)]
mod album_filter_tests {
    use super::*;

    fn fixture() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("CREATE TABLE albums(id INTEGER PRIMARY KEY, path TEXT, file_types INTEGER, small_image_filter INTEGER, excluded_folders TEXT);
            INSERT INTO albums VALUES(1, '/photos', 7, 160, '[\"Exports\",\"100%_done\",\"客户\"]'), (2, '/other', 7, 0, '[]');
            CREATE TABLE afolders(id INTEGER PRIMARY KEY, album_id INTEGER, path TEXT);
            INSERT INTO afolders VALUES(1,1,'/photos'),(2,1,'/photos/Exports/nested'),(3,1,'/photos/Exports-old'),
                (4,1,'/photos/Other/Exports'),(5,1,'/photos/100%_done'),(6,1,'/photos/100XXdone'),
                (7,1,'/photos/客户/深层'),(8,2,'/other');
            CREATE TABLE afiles(id INTEGER PRIMARY KEY, folder_id INTEGER, file_type INTEGER,
                width INTEGER, height INTEGER, live_photo_video_id INTEGER, last_scan_time INTEGER DEFAULT 0, rating INTEGER DEFAULT 5);
            INSERT INTO afiles(id,folder_id,file_type,width,height,live_photo_video_id) VALUES
                (1,1,1,159,159,NULL),(2,1,1,160,100,NULL),(3,1,1,100,160,NULL),(4,1,1,NULL,100,NULL),
                (5,1,1,0,100,NULL),(6,2,1,1000,1000,NULL),(7,3,1,1000,1000,NULL),(8,4,1,1000,1000,NULL),
                (9,5,1,1000,1000,NULL),(10,6,1,1000,1000,NULL),(11,7,1,1000,1000,NULL),
                (12,8,1,10,10,NULL),(13,1,3,1000,1000,14),(14,1,1,1000,1000,NULL),(15,1,2,1000,1000,NULL);").unwrap();
        conn.execute_batch("ALTER TABLE albums ADD COLUMN name TEXT DEFAULT '';
            ALTER TABLE albums ADD COLUMN description TEXT DEFAULT '';
            ALTER TABLE albums ADD COLUMN indexed INTEGER DEFAULT 10;
            ALTER TABLE afiles ADD COLUMN media_subtype TEXT;
            UPDATE afiles SET media_subtype='raw_jpeg_pair' WHERE id=13;").unwrap();
        conn
    }

    fn visible(conn: &Connection) -> Vec<i64> {
        let query = |predicate: String| {
            let sql = format!("SELECT a.id FROM afiles a WHERE {} AND {} ORDER BY a.id",
                AFile::live_photo_companion_exclusion_condition(), predicate);
            conn.prepare(&sql).unwrap().query_map([], |row| row.get(0)).unwrap()
                .collect::<Result<Vec<i64>, _>>().unwrap()
        };
        let bulk = query(AFile::album_query_predicate("a"));
        assert_eq!(bulk, query(AFile::album_filter_predicate("a")));
        let joined_sql = format!("SELECT a.id FROM afiles a LEFT JOIN afolders b ON b.id=a.folder_id
            LEFT JOIN albums c ON c.id=b.album_id WHERE {} AND {} ORDER BY a.id",
            AFile::live_photo_companion_exclusion_condition(), AFile::album_joined_filter_predicate("a", "b", "c"));
        let joined = conn.prepare(&joined_sql).unwrap().query_map([], |row| row.get::<_,i64>(0)).unwrap()
            .collect::<Result<Vec<_>,_>>().unwrap();
        assert_eq!(bulk, joined);
        bulk
    }

    #[test]
    fn lightweight_visibility_matches_album_rules_and_missing_files() {
        let conn = fixture();
        assert!(!AFile::is_album_visible_on(&conn, 999).unwrap());
        assert!(!AFile::is_album_visible_on(&conn, 1).unwrap());
        assert!(!AFile::is_album_visible_on(&conn, 6).unwrap());
        assert!(AFile::is_album_visible_on(&conn, 2).unwrap());
        assert!(AFile::is_album_visible_on(&conn, 12).unwrap());
        conn.execute("UPDATE albums SET file_types=2 WHERE id=1", []).unwrap();
        assert!(!AFile::is_album_visible_on(&conn, 2).unwrap());
        assert!(AFile::is_album_visible_on(&conn, 15).unwrap());
    }

    #[test]
    fn excluded_paths_match_only_the_first_relative_component() {
        let excluded = vec!["Exports".to_string(), "客户".to_string()];
        for path in ["/photos/Exports", "/photos/Exports/nested", "/photos/客户/子目录"] {
            assert!(Album::path_is_excluded(Path::new("/photos"), &excluded, Path::new(path)));
        }
        for path in ["/photos", "/photos/Exports-old", "/photos/Other/Exports", "/other/Exports"] {
            assert!(!Album::path_is_excluded(Path::new("/photos"), &excluded, Path::new(path)));
        }
    }

    #[test]
    fn album_filters_use_own_pixels_and_exact_first_level_folders() {
        let conn = fixture();
        // Pixel thresholds are scoped to each album.
        assert_eq!(visible(&conn), vec![2,3,4,5,7,8,10,12,13,15]);
        conn.execute("UPDATE albums SET small_image_filter=0 WHERE id=1", []).unwrap();
        assert!(visible(&conn).contains(&1));
    }

    #[test]
    fn album_filters_small_images_include_raw_but_never_hide_videos() {
        let conn = fixture();
        conn.execute("UPDATE afiles SET width=100,height=100 WHERE id IN (13,15)", []).unwrap();
        let ids = visible(&conn);
        assert!(!ids.contains(&1)); // Small regular image.
        assert!(!ids.contains(&13)); // Small RAW image.
        assert!(ids.contains(&15)); // Small video stays visible.
        conn.execute("UPDATE albums SET file_types=5 WHERE id=1", []).unwrap();
        assert!(!visible(&conn).contains(&15)); // Video type exclusion still applies.
    }

    #[test]
    fn album_filters_keep_photos_when_paired_raw_is_excluded() {
        let conn = fixture();
        conn.execute("UPDATE albums SET file_types=1 WHERE id=1", []).unwrap();
        let ids = visible(&conn);
        assert!(ids.contains(&14));
        assert!(!ids.contains(&13));
        assert!(!ids.contains(&15));
        conn.execute("UPDATE albums SET file_types=4 WHERE id=1", []).unwrap();
        assert_eq!(visible(&conn), vec![12,13]);
        conn.execute("UPDATE albums SET file_types=2 WHERE id=1", []).unwrap();
        assert_eq!(visible(&conn), vec![12,15]);
    }

    #[test]
    fn album_filters_sweep_preserves_all_excluded_metadata() {
        let conn = fixture();
        conn.execute("UPDATE albums SET file_types=1 WHERE id=1", []).unwrap();
        conn.execute(&format!("DELETE FROM afiles AS a WHERE last_scan_time < 1 AND {}",
            AFile::album_scan_predicate("a")), []).unwrap();
        let retained: Vec<(i64,i64)> = conn.prepare("SELECT id,rating FROM afiles ORDER BY id").unwrap()
            .query_map([], |row| Ok((row.get(0)?,row.get(1)?))).unwrap().collect::<Result<_,_>>().unwrap();
        assert_eq!(retained, vec![(1,5),(6,5),(9,5),(11,5),(13,5),(15,5)]);
        conn.execute("UPDATE albums SET file_types=7,excluded_folders='[]',small_image_filter=0", []).unwrap();
        assert_eq!(visible(&conn), vec![1,6,9,11,13,15]);
    }

    #[test]
    fn album_filters_save_is_atomic_and_preserves_ratings_on_reinclusion() {
        let mut conn = fixture();
        Album::edit_on(&mut conn,1,"Album","Description",1,320,&["Exports".into()]).unwrap();
        assert_eq!(conn.query_row("SELECT indexed FROM albums WHERE id=1", [], |row| row.get::<_,i64>(0)).unwrap(),0);
        assert_eq!(conn.query_row("SELECT COUNT(*) FROM afiles", [], |row| row.get::<_,i64>(0)).unwrap(),15);
        assert!(conn.query_row("SELECT live_photo_video_id FROM afiles WHERE id=13", [], |row| row.get::<_,Option<i64>>(0)).unwrap().is_none());
        Album::edit_on(&mut conn,1,"Album","Description",7,0,&[]).unwrap();
        assert_eq!(conn.query_row("SELECT rating FROM afiles WHERE id=13", [], |row| row.get::<_,i64>(0)).unwrap(),5);
        assert!(visible(&conn).contains(&13));
    }

    #[test]
    fn album_filters_reject_invalid_scope_without_touching_database() {
        assert!(Album::edit(1,"name","",0,0,&[]).is_err());
        assert!(Album::edit(1,"name","",8,0,&[]).is_err());
        assert!(Album::edit(1,"name","",7,200,&[]).is_err());
        for path in ["", ".", "..", "../sibling", "parent/child", "parent\\child", "/absolute"] {
            assert!(Album::edit(1,"name","",7,0,&[path.into()]).is_err());
        }
    }

    #[test]
    fn scan_resume_touch_last_scan_times_preserves_skipped_prefix_from_sweep() {
        let conn = fixture();
        conn.execute("ALTER TABLE afiles ADD COLUMN name TEXT DEFAULT ''", []).unwrap();
        conn.execute("UPDATE afiles SET name = 'img_' || id, last_scan_time = 100 WHERE folder_id = 1", []).unwrap();

        // Simulate an interrupted scan where files were left with old timestamp (100).
        // Next scan runs with current_scan_time = 200.
        // Resume touches prefix files: img_1 and img_2.
        let entries = vec![
            ("/photos".to_string(), "img_1".to_string()),
            ("/photos".to_string(), "img_2".to_string()),
        ];
        let touched = AFile::touch_last_scan_times_on(&conn, 1, &entries, 200).unwrap();
        assert_eq!(touched, 2);

        // Mark-and-sweep deletes files with last_scan_time < 200 in folder 1
        conn.execute(
            "DELETE FROM afiles WHERE last_scan_time < 200 AND folder_id IN (SELECT id FROM afolders WHERE album_id = 1)",
            [],
        ).unwrap();

        // img_1 and img_2 were touched with 200, so they MUST be preserved!
        let remaining: Vec<i64> = conn.prepare("SELECT id FROM afiles WHERE folder_id = 1 ORDER BY id").unwrap()
            .query_map([], |row| row.get(0)).unwrap().collect::<Result<_,_>>().unwrap();
        assert!(remaining.contains(&1));
        assert!(remaining.contains(&2));
        // Other untouched files in folder 1 with last_scan_time 100 were safely swept
        assert!(!remaining.contains(&3));
    }
}

#[cfg(test)]
mod raw_display_cache_tests {
    use super::*;
    use crate::t_raw_display::RawPreviewMode;

    #[test]
    fn raw_display_cache_separates_modes_and_normalizes_missing_orientation() {
        let key = |mode, orientation| AThumb::build_thumb_key(
            "raw-display-test", 42, 512, Some(123), orientation,
            Some(RawDisplayOptions { mode, prefer_pair: false, auto_bright: false }),
        );
        for mode in [RawPreviewMode::Embedded, RawPreviewMode::Rendered] {
            let bright_key = AThumb::build_thumb_key("raw-display-test", 42, 512, Some(123), 1,
                Some(RawDisplayOptions { mode, prefer_pair: false, auto_bright: true }));
            assert_ne!(key(mode, 1), bright_key);
        }
        assert_ne!(key(RawPreviewMode::Embedded, 1), key(RawPreviewMode::Rendered, 1));
        assert_eq!(key(RawPreviewMode::Embedded, 0), key(RawPreviewMode::Embedded, 1));
        assert_ne!(key(RawPreviewMode::Embedded, 6), key(RawPreviewMode::Embedded, 1));
    }
}

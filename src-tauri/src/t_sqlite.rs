/**
 * SQLite database operations.
 * project: Lap
 * author:  julyxx
 * GitHub:  /julyx10
 * date:    2024-08-08
 */
use crate::t_ai;
use crate::t_config;
use crate::t_image;
use crate::t_utils;
use crate::t_video;
use base64::{Engine, engine::general_purpose};
use exif::{In, Reader, Tag, Value};
use rusqlite::{Connection, OptionalExtension, Result, ToSql, params};
use image::{ImageFormat, GenericImageView};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Cursor};
use tauri::State;

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
}

impl Album {
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
        })
    }

    /// fetch an album from db by path
    fn fetch(path: &str) -> Result<Option<Self>, String> {
        let conn = open_conn()?;
        let result = conn.query_row(
            "SELECT id, name, path, created_at, modified_at, display_order_id, cover_file_id, description, indexed, total
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
            "INSERT INTO albums (name, path, created_at, modified_at, display_order_id, cover_file_id, description, indexed, total) 
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
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

        // Insert the new album into the database
        Self::new(path)?.insert()?;

        // return the newly inserted album
        let new_album = Self::fetch(path)?;
        Ok(new_album.unwrap())
    }

    /// delete an album from the db
    pub fn delete_from_db(id: i64) -> Result<usize, String> {
        let conn = open_conn()?;
        let result = conn
            .execute("DELETE FROM albums WHERE id = ?1", params![id])
            .map_err(|e| e.to_string())?;
        Ok(result)
    }

    /// Get all albums(album_type = 1) from the db
    pub fn get_all_albums() -> Result<Vec<Self>, String> {
        let conn = open_conn()?;

        let query = 
            "SELECT id, name, path, created_at, modified_at, display_order_id, cover_file_id, description, indexed, total
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
            "SELECT id, name, path, created_at, modified_at, display_order_id, cover_file_id, description, indexed, total
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

    /// set album cover to the first file (image/video) if not set
    pub fn auto_set_cover(id: i64) -> Result<(), String> {
        let conn = open_conn()?;

        // 1. check if cover_file_id is set
        let cover_file_id: Option<i64> = conn
            .query_row(
                "SELECT cover_file_id FROM albums WHERE id = ?1",
                params![id],
                |row| row.get(0),
            )
            .map_err(|e| e.to_string())?;

        if cover_file_id.unwrap_or(0) > 0 {
            return Ok(());
        }

        // 2. get the first formatted file (image or video)
        let file_id: Option<i64> = conn
            .query_row(
                "SELECT a.id 
                FROM afiles a
                JOIN afolders b ON a.folder_id = b.id
                JOIN athumbs c ON a.id = c.file_id
                WHERE b.album_id = ?1 AND (a.file_type = 1 OR a.file_type = 2)
                ORDER BY a.taken_date ASC
                LIMIT 1",
                params![id],
                |row| row.get(0),
            )
            .optional() // returns Option<i64>
            .map_err(|e| e.to_string())?;

        // 3. update cover_file_id
        if let Some(fid) = file_id {
            let _ = conn
                .execute(
                    "UPDATE albums SET cover_file_id = ?1 WHERE id = ?2",
                    params![fid, id],
                )
                .map_err(|e| e.to_string())?;
        }

        Ok(())
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
    pub is_favorite: Option<bool>, // is favorite
}

impl AFolder {
    /// create a new folder struct
    fn new(album_id: i64, folder_path: &str) -> Result<Self, String> {
        let file_info = t_utils::FileInfo::new(folder_path)?;
        Ok(Self {
            id: None,
            album_id,
            name: file_info.file_name,
            path: folder_path.to_string(),
            created_at: file_info.created,
            modified_at: file_info.modified,
            is_favorite: None,
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
        })
    }

    /// fetch a folder row from db (by path)
    pub fn fetch(folder_path: &str) -> Result<Option<Self>, String> {
        let conn = open_conn()?;
        let result = conn
            .query_row(
                "SELECT id, album_id, name, path, created_at, modified_at, is_favorite
                FROM afolders
                WHERE path = ?1",
                params![folder_path],
                Self::from_row,
            )
            .optional()
            .map_err(|e| e.to_string())?;
        Ok(result)
    }

    /// insert a folder into db
    fn insert(&self) -> Result<usize, String> {
        let conn = open_conn()?;
        let result = conn
            .execute(
                "INSERT INTO afolders (album_id, name, path, created_at, modified_at, is_favorite) 
                VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    self.album_id,
                    self.name,
                    self.path,
                    self.created_at,
                    self.modified_at,
                    self.is_favorite
                ],
            )
            .map_err(|e| e.to_string())?;
        Ok(result)
    }

    /// insert the folder to db if not exists
    pub fn add_to_db(album_id: i64, folder_path: &str) -> Result<Self, String> {
        // Check if the path already exists
        let existing_folder = Self::fetch(folder_path);
        if let Ok(Some(folder)) = existing_folder {
            return Ok(folder);
        }

        // insert the new folder into the database
        // when insert a new folder, save album_id value
        Self::new(album_id, folder_path)?.insert()?;

        // return the newly inserted folder
        let new_folder = Self::fetch(folder_path)?;
        Ok(new_folder.unwrap())
    }

    /// rename a folder (update path and name)
    pub fn rename_folder(old_path: &str, new_path: &str) -> Result<usize, String> {
        // get folder name
        let folder_name = t_utils::get_file_name(new_path);

        let conn = open_conn()?;
        let result = conn
            .execute(
                "UPDATE afolders
                SET path = CONCAT(?2, SUBSTRING(path, LENGTH(?1) + 1)), name = ?3
                WHERE path LIKE ?1 || '%'",
                params![old_path, new_path, folder_name],
            )
            .map_err(|e| e.to_string())?;
        Ok(result)
    }

    /// move a folder (update path and album_id)
    pub fn move_folder(old_path: &str, new_album_id: i64, new_path: &str) -> Result<usize, String> {
        let conn = open_conn()?;
        let result = conn
            .execute(
                "UPDATE afolders
                SET path = CONCAT(?3, SUBSTRING(path, LENGTH(?1) + 1)), album_id = ?2
                WHERE path LIKE ?1 || '%'",
                params![old_path, new_album_id, new_path],
            )
            .map_err(|e| e.to_string())?;
        Ok(result)
    }

    /// delete a folder and all its child folders and files from db
    pub fn delete_folder(folder_path: &str) -> Result<usize, String> {
        let conn = open_conn()?;

        // First, get all folder IDs that will be deleted (the folder itself and all children)
        let folder_ids: Vec<i64> = {
            let mut stmt = conn
                .prepare("SELECT id FROM afolders WHERE path = ?1 OR path LIKE ?2")
                .map_err(|e| e.to_string())?;

            let path_pattern = format!("{}/%", folder_path);
            let rows = stmt
                .query_map(params![folder_path, path_pattern], |row| row.get(0))
                .map_err(|e| e.to_string())?;

            rows.filter_map(|r| r.ok()).collect()
        };

        // Delete all files in those folders
        for folder_id in &folder_ids {
            conn.execute(
                "DELETE FROM afiles WHERE folder_id = ?1",
                params![folder_id],
            )
            .map_err(|e| e.to_string())?;
        }

        // Delete the folders (the folder and all its children)
        let path_pattern = format!("{}/%", folder_path);
        let result = conn
            .execute(
                "DELETE FROM afolders WHERE path = ?1 OR path LIKE ?2",
                params![folder_path, path_pattern],
            )
            .map_err(|e| e.to_string())?;

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

    // get all favorite folders
    pub fn get_favorite_folders() -> Result<Vec<Self>, String> {
        let conn = open_conn()?;

        let query = "SELECT a.id, a.album_id, a.name, a.path, a.created_at, a.modified_at, a.is_favorite
            FROM afolders a
            WHERE a.is_favorite = 1
            ORDER BY a.name".to_string();

        let mut stmt = conn.prepare(query.as_str()).map_err(|e| e.to_string())?;

        let rows = stmt
            .query_map(params![], Self::from_row)
            .map_err(|e| e.to_string())?;

        let mut folders = Vec::new();
        for folder in rows {
            folders.push(folder.unwrap());
        }

        Ok(folders)
    }
}

/// Define the album file struct
#[derive(Debug, Serialize, Deserialize)]
pub struct AFile {
    pub id: Option<i64>, // unique id (autoincrement by db)
    pub folder_id: i64,  // folder id (from folders table)

    // file basic info
    pub name: String,                // file name
    pub name_pinyin: Option<String>, // file name pinyin(for sort)
    pub size: i64,                   // file size
    pub file_type: Option<i64>,      // file type (0: all, 1: image, 2: video, 3: audio, 4: other)
    pub created_at: Option<i64>,     // file create timestamp
    pub modified_at: Option<i64>,    // file modified timestamp
    pub taken_date: Option<i64>,     // taken date timestamp (e_date_time || modified_at)

    // image/video
    pub width: Option<u32>,    // image/video width
    pub height: Option<u32>,   // image/video height
    pub duration: Option<i64>, // video duration

    // extra info
    pub is_favorite: Option<bool>, // is favorite
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
    pub file_path: Option<String>,   // file path (for webview)
    pub album_id: Option<i64>,       // album id (for webview)
    pub album_name: Option<String>,  // album name (for webview)
    pub has_thumbnail: Option<bool>, // has thumbnail (for webview)
    pub has_embedding: Option<bool>, // has embedding (for webview)
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
    pub search_all_subfolders: String,
    pub search_folder: String,
    pub start_date: i64,
    pub end_date: i64,
    pub make: String,
    pub model: String,
    pub location_admin1: String,
    pub location_name: String,
    pub is_favorite: bool,
    pub tag_id: i64,
    pub person_id: i64,
}

/// Define the AI image search parameters struct
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ImageSearchParams {
    pub search_text: String,  // search image text (for AI search)
    pub file_id: Option<i64>, // file id (for similar image search)
    pub threshold: f32,       // search threshold
    pub limit: i64,           // search limit
}

impl AFile {
    fn new(folder_id: i64, file_path: &str, file_type: i64) -> Result<Self, String> {
        let file_info = t_utils::FileInfo::new(file_path)?;

        // get dimensions based on file type
        let (width, height) = match file_type {
            1 | 3 => t_image::get_image_dimensions(file_path)?,
            2 => t_video::get_video_dimensions(file_path)?,
            _ => (0, 0),
        };

        // get duration of video file
        let duration = match file_type {
            2 => t_video::get_video_duration(file_path)? as i64,
            _ => 0,
        };

        // Initialize mutable metadata fields
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

        if file_type == 1 || file_type == 3 {
            // Image file
            // Read EXIF data
            let exif = File::open(file_path)
                .map_err(|e| format!("Error opening file: {}", e))
                .map(|file| {
                    let mut bufreader = BufReader::new(&file);
                    Reader::new().read_from_container(&mut bufreader).ok()
                })?;

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
                    .and_then(|field| field.value.get_uint(0)) // Return u64 directly
                    .or(Some(1)) // If no orientation is found, default to 1 (normal orientation)
            });

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
        } else if file_type == 2 {
            // Video file
            if let Ok(video_metadata) = t_video::get_video_metadata(file_path) {
                e_make = video_metadata.e_make;
                e_model = video_metadata.e_model;
                e_date_time = video_metadata.e_date_time;
                e_software = video_metadata.e_software;
                gps_latitude = video_metadata.gps_latitude;
                gps_longitude = video_metadata.gps_longitude;
                gps_altitude = video_metadata.gps_altitude;
            }

            taken_date = e_date_time
                .as_ref()
                .and_then(|dt| t_utils::meta_date_to_timestamp(dt))
                .or(file_info.modified);
        }

        // Geocoding based on GPS coordinates from any source
        let (geo_name, geo_admin1, geo_admin2, geo_cc) =
            if let (Some(lat), Some(lon)) = (gps_latitude, gps_longitude) {
                let search_result = t_utils::GEOCODER.search((lat, lon));
                (
                    Some(search_result.record.name.clone()),
                    Some(search_result.record.admin1.clone()),
                    Some(search_result.record.admin2.clone()),
                    Some(search_result.record.cc.clone()),
                )
            } else {
                (None, None, None, None)
            };

        let file = Self {
            id: None,
            folder_id,

            name: file_info.file_name.clone(),
            name_pinyin: Some(t_utils::convert_to_pinyin(file_info.file_name.as_str())), // convert to pinyin
            size: file_info.file_size,
            file_type: Some(file_type),
            created_at: file_info.created,
            modified_at: file_info.modified,

            taken_date,
            width: e_orientation
                .map(|orientation| if orientation > 4 { height } else { width })
                .or(Some(width)),
            height: e_orientation
                .map(|orientation| if orientation > 4 { width } else { height })
                .or(Some(height)),
            duration: Some(duration),

            is_favorite: None,
            rotate: None,
            comments: None,
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
            has_embedding: None,
        };

        Ok(file)
    }

    fn extract_gps_data(exif: &Option<exif::Exif>) -> (Option<f64>, Option<f64>, Option<f64>) {
        let Some(exif_data) = exif else {
            return (None, None, None);
        };

        let lat_val = exif_data
            .get_field(Tag::GPSLatitude, In::PRIMARY)
            .and_then(|f| match &f.value {
                Value::Rational(v) => Some(v.to_vec()),
                _ => None,
            });
        let lat_ref = exif_data
            .get_field(Tag::GPSLatitudeRef, In::PRIMARY)
            .map(|f| f.display_value().to_string());
        let lon_val = exif_data
            .get_field(Tag::GPSLongitude, In::PRIMARY)
            .and_then(|f| match &f.value {
                Value::Rational(v) => Some(v.to_vec()),
                _ => None,
            });
        let lon_ref = exif_data
            .get_field(Tag::GPSLongitudeRef, In::PRIMARY)
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
        let field = exif.as_ref()?.get_field(tag, In::PRIMARY)?;

        let raw = match &field.value {
            Value::Ascii(vec) => {
                let mut bytes = Vec::new();
                for line in vec {
                    let cleaned: Vec<u8> = line.iter().cloned().take_while(|&b| b != 0).collect();
                    bytes.extend(cleaned);
                }
                String::from_utf8_lossy(&bytes).into_owned()
            }
            _ => {
                let displayed_value = field.display_value().with_unit(exif.as_ref()?).to_string();
                if displayed_value.starts_with("1/") {
                    let parts: Vec<&str> = displayed_value.split(' ').collect();
                    let fraction_part = &parts[0][2..];

                    let new_fraction_part = if fraction_part.contains('.') {
                        let decimal_part = fraction_part.split('.').nth(1).unwrap_or("");
                        if decimal_part.len() > 2 {
                            if let Ok(num) = fraction_part.parse::<f64>() {
                                format!("{:.2}", num)
                            } else {
                                fraction_part.to_string()
                            }
                        } else {
                            fraction_part.to_string()
                        }
                    } else {
                        fraction_part.to_string()
                    };

                    let mut result = format!("1/{}", new_fraction_part);
                    if parts.len() > 1 {
                        result.push(' ');
                        result.push_str(parts[1]);
                    }
                    result
                } else {
                    let parts: Vec<&str> = displayed_value.split(' ').collect();
                    if parts.is_empty() {
                        return None;
                    }
                    if let Ok(num) = parts[0].parse::<f64>() {
                        let result = if parts[0].contains('.') {
                            let decimal_part = parts[0].split('.').nth(1).unwrap_or("");
                            if decimal_part.len() > 2 {
                                format!("{:.2}", num)
                            } else {
                                parts[0].to_string()
                            }
                        } else {
                            parts[0].to_string()
                        };

                        let mut final_result = result;
                        if parts.len() > 1 {
                            final_result.push(' ');
                            final_result.push_str(parts[1]);
                        }
                        final_result
                    } else {
                        displayed_value
                    }
                }
            }
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

    /// insert a file into db
    fn insert(&self) -> Result<usize, String> {
        let conn = open_conn()?;
        let result = conn.execute(
            "INSERT INTO afiles (
                folder_id, 
                name, name_pinyin, size, file_type, created_at, modified_at, 
                taken_date,
                width, height, duration,
                is_favorite, rotate, comments, has_tags,
                e_make, e_model, e_date_time, e_software, e_artist, e_copyright, e_description, e_lens_make, e_lens_model, e_exposure_bias, e_exposure_time, e_f_number, e_focal_length, e_iso_speed, e_flash, e_orientation,
                gps_latitude, gps_longitude, gps_altitude, geo_name, geo_admin1, geo_admin2, geo_cc
            ) 
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24, ?25, ?26, ?27, ?28, ?29, ?30, ?31, ?32, ?33, ?34, ?35, ?36, ?37, ?38)",
            params![
                self.folder_id,

                self.name,
                self.name_pinyin,
                self.size,
                self.file_type,
                self.created_at,
                self.modified_at,

                self.taken_date,

                self.width,
                self.height,
                self.duration,

                self.is_favorite,
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
            ]
        ).map_err(|e| e.to_string())?;
        Ok(result)
    }

    /// update a file into db
    pub fn update(file_id: i64, file: &Self) -> Result<usize, String> {
        let conn = open_conn()?;
        let result = conn.execute(
            "UPDATE afiles SET
                name = ?1, name_pinyin = ?2, size = ?3, file_type = ?4, created_at = ?5, modified_at = ?6,
                taken_date = ?7,
                width = ?8, height = ?9, duration = ?10,
                e_make = ?11, e_model = ?12, e_date_time = ?13, e_software = ?14, e_artist = ?15, e_copyright = ?16, e_description = ?17, e_lens_make = ?18, e_lens_model = ?19, e_exposure_bias = ?20, e_exposure_time = ?21, e_f_number = ?22, e_focal_length = ?23, e_iso_speed = ?24, e_flash = ?25, e_orientation = ?26,
                gps_latitude = ?27, gps_longitude = ?28, gps_altitude = ?29, geo_name = ?30, geo_admin1 = ?31, geo_admin2 = ?32, geo_cc = ?33
            WHERE id = ?34",
            params![
                file.name,
                file.name_pinyin,
                file.size,
                file.file_type,
                file.created_at,
                file.modified_at,

                file.taken_date,

                file.width,
                file.height,
                file.duration,

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

                file_id,
            ]
        ).map_err(|e| e.to_string())?;
        Ok(result)
    }

    // delete a file from db
    pub fn delete(id: i64) -> Result<usize, String> {
        let conn = open_conn()?;
        let result = conn
            .execute("DELETE FROM afiles WHERE id = ?1", params![id])
            .map_err(|e| e.to_string())?;
        Ok(result)
    }

    /// get all file IDs for a specific album
    /// Returns a map of file path to file ID
    pub fn get_all_ids_in_album(album_id: i64) -> Result<HashMap<String, i64>, String> {
        let conn = open_conn()?;
        let mut stmt = conn
            .prepare(
                "SELECT a.id, b.path, a.name
                FROM afiles a
                JOIN afolders b ON a.folder_id = b.id
                WHERE b.album_id = ?1",
            )
            .map_err(|e| e.to_string())?;

        let rows = stmt
            .query_map(params![album_id], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                ))
            })
            .map_err(|e| e.to_string())?;

        let mut files = HashMap::new();
        for row in rows {
            if let Ok((id, folder_path, name)) = row {
                let full_path = t_utils::get_file_path(&folder_path, &name);
                files.insert(full_path, id);
            }
        }
        Ok(files)
    }
    
    // Helper function to build the count SQL query
    fn build_count_query() -> String {
        let base_query = "SELECT COUNT(*), SUM(a.size)
            FROM afiles a 
            LEFT JOIN afolders b ON a.folder_id = b.id
            LEFT JOIN albums c ON b.album_id = c.id";

        base_query.to_string()
    }

    // build the base SQL query
    fn build_base_query() -> String {
        String::from(
            "SELECT a.id, a.folder_id, 
                a.name, a.name_pinyin, a.size, a.file_type, a.created_at, a.modified_at, 
                a.taken_date,
                a.width, a.height, a.duration,
                a.is_favorite, a.rotate, a.comments, a.has_tags,
                a.e_make, a.e_model, a.e_date_time, a.e_software, a.e_artist, a.e_copyright, a.e_description, a.e_lens_make, a.e_lens_model, a.e_exposure_bias, a.e_exposure_time, a.e_f_number, a.e_focal_length, a.e_iso_speed, a.e_flash, a.e_orientation,
                a.gps_latitude, a.gps_longitude, a.gps_altitude, a.geo_name, a.geo_admin1, a.geo_admin2, a.geo_cc,
                b.path,
                c.id AS album_id, c.name AS album_name,
                (SELECT 1 FROM athumbs t WHERE t.file_id = a.id LIMIT 1) AS has_thumbnail,
                CASE WHEN a.embeds IS NOT NULL THEN 1 ELSE 0 END AS has_embedding,
                a.has_faces
            FROM afiles a 
            LEFT JOIN afolders b ON a.folder_id = b.id
            LEFT JOIN albums c ON b.album_id = c.id"
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
            created_at: row.get(6)?,
            modified_at: row.get(7)?,

            taken_date: row.get(8)?,

            width: row.get(9)?,
            height: row.get(10)?,
            duration: row.get(11)?,

            is_favorite: row.get(12)?,
            rotate: row.get(13)?,
            comments: row.get(14)?,
            has_tags: row.get(15)?,

            e_make: row.get(16)?,
            e_model: row.get(17)?,
            e_date_time: row.get(18)?,
            e_software: row.get(19)?,
            e_artist: row.get(20)?,
            e_copyright: row.get(21)?,
            e_description: row.get(22)?,
            e_lens_make: row.get(23)?,
            e_lens_model: row.get(24)?,
            e_exposure_bias: row.get(25)?,
            e_exposure_time: row.get(26)?,
            e_f_number: row.get(27)?,
            e_focal_length: row.get(28)?,
            e_iso_speed: row.get(29)?,
            e_flash: row.get(30)?,
            e_orientation: row.get(31)?,

            gps_latitude: row.get(32)?,
            gps_longitude: row.get(33)?,
            gps_altitude: row.get(34)?,
            geo_name: row.get(35)?,
            geo_admin1: row.get(36)?,
            geo_admin2: row.get(37)?,
            geo_cc: row.get(38)?,

            file_path: Some(t_utils::get_file_path(
                row.get::<_, String>(39)?.as_str(),
                row.get::<_, String>(2)?.as_str(),
            )),
            album_id: row.get(40)?,
            album_name: row.get(41)?,
            has_thumbnail: row.get::<_, Option<i64>>(42)?.map(|v| v == 1),
            has_embedding: row.get::<_, Option<i64>>(43)?.map(|v| v == 1),
            has_faces: row.get::<_, Option<i32>>(44)?,
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
            files.push(file.unwrap());
        }

        Ok(files)
    }

    /// fetch a file info from db by folder_id and file name
    pub fn fetch(folder_id: i64, file_path: &str) -> Result<Option<Self>, String> {
        let conn = open_conn()?;

        // Prepare the SQL query by using the base query and adding conditions
        let sql = format!(
            "{} WHERE a.folder_id = ?1 AND a.name = ?2",
            Self::build_base_query()
        );

        // Execute the query with folder_id and file name as parameters
        let result = conn
            .query_row(
                &sql,
                params![folder_id, t_utils::get_file_name(file_path)],
                Self::from_row,
            )
            .optional()
            .map_err(|e| e.to_string())?;

        Ok(result)
    }

    /// insert a file into db if not exists
    /// Returns (file, status)
    /// status: 0 - existing, 1 - new, 2 - updated
    pub fn add_to_db(
        folder_id: i64,
        file_path: &str,
        file_type: i64,
    ) -> Result<(Self, i32), String> {
        // Check if the file exists
        let existing_file = Self::fetch(folder_id, file_path)?;
        if let Some(file) = existing_file {
            // check file modified time or if thumbnail is missing
            let file_info = t_utils::FileInfo::new(file_path)?;
            let modified = file.modified_at != file_info.modified;
            let missing_thumb = !file.has_thumbnail.unwrap_or(false);

            if modified || missing_thumb {
                if let Some(mut updated_file) = Self::update_file_info(file.id.unwrap(), file_path)?
                {
                    // If modified, delete old thumbnail and remove embeds data
                    if modified {
                        let _ = AThumb::delete(file.id.unwrap());
                        // remove embeds data
                        let conn = open_conn()?;
                        let _ = conn.execute(
                            "UPDATE afiles SET embeds = NULL WHERE id = ?1",
                            params![file.id.unwrap()],
                        );
                        updated_file.has_embedding = Some(false);
                    }
                    return Ok((updated_file, 2));
                }
            }
            return Ok((file, 0));
        }

        // insert the new file into the database
        Self::new(folder_id, file_path, file_type)?.insert()?;

        // return the newly inserted file
        let new_file = Self::fetch(folder_id, file_path)?;
        Ok((new_file.unwrap(), 1))
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

    /// update a file info
    pub fn update_file_info(file_id: i64, file_path: &str) -> Result<Option<Self>, String> {
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
        new_file_info.rotate = old_file_info.rotate;
        new_file_info.comments = old_file_info.comments;
        new_file_info.has_tags = old_file_info.has_tags;

        // update the file info
        Self::update(file_id, &new_file_info)?;

        Self::get_file_info(file_id)
    }

    /// update a file column value
    pub fn update_column(
        file_id: i64,
        column: &str,
        value: &dyn rusqlite::ToSql,
    ) -> Result<usize, String> {
        let conn = open_conn()?;
        let query = format!("UPDATE afiles SET {} = ?1 WHERE id = ?2", column);
        let result = conn
            .execute(&query, params![value, file_id])
            .map_err(|e| e.to_string())?;
        Ok(result)
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
    pub fn get_taken_dates(ascending: bool) -> Result<Vec<(String, i64)>, String> {
        let conn = open_conn()?;

        let order_clause = if ascending { "ASC" } else { "DESC" };
        let query = format!(
            "SELECT strftime('%Y-%m-%d', a.taken_date, 'unixepoch', 'localtime') AS taken_date, COUNT(1) 
            FROM afiles a
            WHERE a.taken_date IS NOT NULL AND a.taken_date >= 86400
            GROUP BY strftime('%Y-%m-%d', a.taken_date, 'unixepoch', 'localtime')
            ORDER BY taken_date {}",
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

    // get total count and size of files
    pub fn get_total_count_and_sum() -> Result<(i64, i64), String> {
        let sql = Self::build_count_query().to_string();
        Self::query_count_and_sum(&sql, &[])
    }

    // helper to build search query conditions and params
    // Returns (joins_clause, where_clause, params)
    fn build_search_query_parts(
        params: &QueryParams,
    ) -> (String, String, Vec<Box<dyn ToSql>>) {
        let mut joins = Vec::new();
        let mut conditions = Vec::new();
        let mut sql_params: Vec<Box<dyn ToSql>> = Vec::new();

        if !params.search_file_name.is_empty() {
            conditions.push("a.name LIKE ? COLLATE NOCASE");
            sql_params.push(Box::new(format!("%{}%", params.search_file_name)));
        }

        if params.search_file_type > 0 {
            conditions.push("a.file_type = ?");
            sql_params.push(Box::new(params.search_file_type));
        }

        if !params.search_all_subfolders.is_empty() {
            // Match path that starts with search_folder followed by '/' or end of string
            conditions.push("(b.path = ? OR b.path LIKE ?)");
            sql_params.push(Box::new(params.search_all_subfolders.clone()));
            sql_params.push(Box::new(format!("{}/%", params.search_all_subfolders)));
        }

        if !params.search_folder.is_empty() {
            conditions.push("(b.path = ?)");
            sql_params.push(Box::new(params.search_folder.clone()));
        }

        if params.start_date > 0 && params.end_date > 0 {
            conditions.push("a.taken_date >= ? AND a.taken_date < ?");
            sql_params.push(Box::new(params.start_date));
            sql_params.push(Box::new(params.end_date));
        } else if params.start_date == -1 && params.end_date == -1 {
            // "On This Day" feature: find all photos taken on the same month and day as today
            let now = chrono::Local::now();
            let today_month_day = now.format("%m-%d").to_string();
            conditions.push("strftime('%m-%d', a.taken_date, 'unixepoch', 'localtime') = ?");
            sql_params.push(Box::new(today_month_day));
        }

        if !params.make.is_empty() {
            conditions.push("UPPER(a.e_make) = UPPER(?)");
            sql_params.push(Box::new(params.make.clone()));
            if !params.model.is_empty() {
                conditions.push("a.e_model = ?");
                sql_params.push(Box::new(params.model.clone()));
            }
        }

        if !params.location_admin1.is_empty() {
            conditions.push("a.geo_admin1 = ?");
            sql_params.push(Box::new(params.location_admin1.clone()));
            if !params.location_name.is_empty() {
                conditions.push("a.geo_name = ?");
                sql_params.push(Box::new(params.location_name.clone()));
            }
        }

        if params.is_favorite {
            conditions.push("a.is_favorite = 1");
        }

        if params.tag_id > 0 {
            joins.push("INNER JOIN afile_tags at ON a.id = at.file_id");
            conditions.push("at.tag_id = ?");
            sql_params.push(Box::new(params.tag_id));
        }

        if params.person_id > 0 {
            joins.push("INNER JOIN faces f ON a.id = f.file_id");
            conditions.push("f.person_id = ?");
            sql_params.push(Box::new(params.person_id));
        }

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
        let dir = if params.sort_order == 1 { "DESC" } else { "ASC" };
        match params.sort_type {
            0 => query.push_str(&format!(" ORDER BY a.taken_date {}", dir)),
            1 => query.push_str(&format!(" ORDER BY a.name_pinyin {}", dir)),
            2 => query.push_str(&format!(" ORDER BY a.size {}", dir)),
            3 => query.push_str(&format!(" ORDER BY a.width {}, a.height {}", dir, dir)),
            4 => query.push_str(" ORDER BY RANDOM()"),
            _ => query.push_str(&format!(" ORDER BY a.taken_date {}", dir)),
        }

        // paging
        query.push_str(" LIMIT ? OFFSET ?");

        let mut final_params: Vec<&dyn ToSql> = sql_params.iter().map(|p| p.as_ref()).collect();
        final_params.push(&limit);
        final_params.push(&offset);
        Self::query_files(&query, &final_params)
    }

    // get query timeline markers
    pub fn get_query_time_line(params: &QueryParams) -> Result<Vec<ATimeLine>, String> {
        // Only process for time-based sorts (0=taken_date)
        if params.sort_type != 0 {
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
            date_field, order_clause, year_extract, month_extract, date_extract, joins, where_clause
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
                engine.encode_image_from_bytes(thumb_bytes).or_else(|_| {
                    // If thumbnail processing fails (e.g. corrupted), try original
                    engine.encode_image(&file_path)
                })
            }
            _ => engine.encode_image(&file_path),
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

        let query = "SELECT a.id, a.embeds 
            FROM afiles a
            WHERE a.embeds IS NOT NULL".to_string();

        let mut stmt = conn.prepare(&query).map_err(|e| e.to_string())?;

        let rows = stmt
            .query_map([], |row| {
                let id: i64 = row.get(0)?;
                let embeds_blob: Vec<u8> = row.get(1)?;
                Ok((id, embeds_blob))
            })
            .map_err(|e| e.to_string())?;

        let mut scores: Vec<(i64, f32)> = Vec::new();

        // If search_text is present, force threshold to 0.25
        let threshold = if !params.search_text.is_empty() {
            0.25
        } else {
            params.threshold
        };

        // Calculate similarity
        for row in rows {
            let (id, embeds_blob) = row.map_err(|e| e.to_string())?;

            // Convert blob back to Vec<f32>
            let file_embedding: Vec<f32> = embeds_blob
                .chunks_exact(4)
                .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
                .collect();

            let score = Self::cosine_similarity(&embedding, &file_embedding);

            if score > threshold {
                scores.push((id, score));
            }
        }

        // Sort by score descending
        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Limit
        let limit = if params.limit > 0 {
            params.limit as usize
        } else {
            scores.len()
        };

        let final_scores = if limit < scores.len() {
            &scores[..limit]
        } else {
            &scores[..]
        };

        // Fetch full file info
        let mut results = Vec::new();
        for (id, _) in final_scores {
            if let Ok(Some(file)) = Self::get_file_info(*id) {
                results.push(file);
            }
        }

        println!("Returning {} files", results.len());

        Ok(results)
    }

    fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        let dot_product: f32 = a.iter().zip(b).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            0.0
        } else {
            dot_product / (norm_a * norm_b)
        }
    }
}

/// Define the album thumbnail struct
#[derive(Debug, Serialize, Deserialize)]
pub struct AThumb {
    pub id: Option<i64>, // unique id (autoincrement by db)
    pub file_id: i64,    // file id (from files table)
    pub error_code: i64, // error code (0: success, 1: error)

    #[serde(skip)]
    pub thumb_data: Option<Vec<u8>>, // thumbnail data (store into db as BLOB)

    // output only
    pub thumb_data_base64: Option<String>, // fetch thumbnail data as base64 string (for webview)
}

impl AThumb {
    /// Create a new thumbnail struct
    pub fn new(
        file_id: i64,
        file_path: &str,
        file_type: i64,
        orientation: i32,
        thumbnail_size: u32,
    ) -> Result<Option<Self>, String> {
        let (thumb_data, error_code) = match file_type {
            1 => {
                // image
                if let Some(ext) = t_utils::get_file_extension(file_path) {
                    match ext.to_lowercase().as_str() {
                        "heic" | "heif" => {
                            // heic/heif
                            #[cfg(target_os = "macos")]
                            match t_image::get_heic_thumbnail_with_sips(file_path, thumbnail_size) {
                                Ok(Some(data)) => (Some(data), 0),
                                Ok(None) => (None, 1), // empty thumb
                                Err(_) => (None, 1),   // error
                            }
                            #[cfg(not(target_os = "macos"))]
                            match t_video::get_video_thumbnail(file_path, thumbnail_size) {
                                Ok(Some(data)) => (Some(data), 0),
                                Ok(None) => (None, 1), // empty thumb
                                Err(_) => (None, 1),   // error
                            }
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
                match t_video::get_video_thumbnail(file_path, thumbnail_size) {
                    Ok(Some(data)) => (Some(data), 0),
                    Ok(None) => (None, 1),
                    Err(_) => (None, 1),
                }
            }
            _ => (None, 1),
        };

        Ok(Some(Self {
            id: None,
            file_id,
            error_code,
            thumb_data,
            thumb_data_base64: None,
        }))
    }

    /// insert a thumbnail into db
    fn insert(&self) -> Result<usize, String> {
        let conn = open_conn()?;
        let result = conn
            .execute(
                "INSERT OR IGNORE INTO athumbs (file_id, error_code, thumb_data) 
                VALUES (?1, ?2, ?3)",
                params![self.file_id, self.error_code, self.thumb_data,],
            )
            .map_err(|e| e.to_string())?;
        Ok(result) // 0: already exists, ignore, 1: inserted
    }

    /// fetch a thumbnail from db by file_id
    pub fn fetch(file_id: i64) -> Result<Option<Self>, String> {
        let conn = open_conn()?;
        let result = conn
            .query_row(
                "SELECT id, file_id, error_code, thumb_data 
                FROM athumbs WHERE file_id = ?1",
                params![file_id],
                |row| {
                    let thumb_data: Option<Vec<u8>> = row.get(3)?;
                    let thumb_data_base64 = thumb_data
                        .as_ref()
                        .map(|data| general_purpose::STANDARD.encode(data));
                    Ok(Self {
                        id: Some(row.get(0)?),
                        file_id: row.get(1)?,
                        error_code: row.get(2)?,
                        thumb_data,
                        thumb_data_base64,
                    })
                },
            )
            .optional()
            .map_err(|e| e.to_string())?;
        Ok(result)
    }

    /// get or create a thumbnail
    pub fn get_or_create_thumb(
        file_id: i64,
        file_path: &str,
        file_type: i64,
        orientation: i32,
        thumbnail_size: u32,
        force_regenerate: bool,
    ) -> Result<Option<Self>, String> {
        // If force_regenerate is true, delete the existing thumbnail if any
        if force_regenerate {
            let _ = Self::delete(file_id);
        } else {
            // Check if the thumbnail exists
            if let Ok(Some(thumbnail)) = Self::fetch(file_id) {
                return Ok(Some(thumbnail));
            }
        }

        // Try to create a new thumbnail.
        let athumb = match Self::new(file_id, file_path, file_type, orientation, thumbnail_size) {
            Ok(Some(athumb)) => athumb,
            _ => {
                // If `new` fails for any reason, create a thumbnail with an error code.
                Self {
                    id: None,
                    file_id,
                    error_code: 1,
                    thumb_data: None,
                    thumb_data_base64: None,
                }
            }
        };
        athumb.insert()?;

        Self::fetch(file_id)
    }

    /// delete a thumbnail from db
    pub fn delete(file_id: i64) -> Result<usize, String> {
        let conn = open_conn()?;
        let result = conn
            .execute("DELETE FROM athumbs WHERE file_id = ?1", params![file_id])
            .map_err(|e| e.to_string())?;
        Ok(result)
    }

    /// get the thumbnail count of the folder
    pub fn get_folder_thumb_count(file_type: i64, folder_id: i64) -> Result<i64, String> {
        let conn = open_conn()?;

        let mut conditions = Vec::new();
        let mut params: Vec<&dyn rusqlite::ToSql> = Vec::new();

        conditions.push("a.folder_id = ?");
        params.push(&folder_id);

        if file_type > 0 {
            conditions.push("a.file_type = ?");
            params.push(&file_type);
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
}

impl ATag {
    /// Function to construct `Self` from a database row
    fn from_row(row: &rusqlite::Row) -> Result<Self, rusqlite::Error> {
        Ok(Self {
            id: row.get(0)?,
            name: row.get(1)?,
            count: row.get(2)?,
        })
    }

    /// Add a new tag. If the tag already exists, return the existing one.
    pub fn add(name: &str) -> Result<Self, String> {
        let conn = open_conn()?;
        // First, try to fetch the tag to see if it already exists.
        let existing_tag = conn
            .query_row(
                "SELECT id, name, 0 as count FROM atags WHERE name = ?1",
                params![name],
                Self::from_row,
            )
            .optional()
            .map_err(|e| e.to_string())?;

        if let Some(tag) = existing_tag {
            Ok(tag)
        } else {
            // The tag doesn't exist, so insert it.
            conn.execute("INSERT INTO atags (name) VALUES (?1)", params![name])
                .map_err(|e| e.to_string())?;
            let id = conn.last_insert_rowid();
            Ok(Self {
                id,
                name: name.to_string(),
                count: Some(0),
            })
        }
    }

    /// Get all tags from the db
    pub fn get_all() -> Result<Vec<Self>, String> {
        let conn = open_conn()?;
        let query = "SELECT atags.id, atags.name, SUM(CASE WHEN afiles.id IS NOT NULL THEN 1 ELSE 0 END) AS count 
            FROM atags 
            LEFT JOIN afile_tags ON atags.id = afile_tags.tag_id
            LEFT JOIN afiles ON afile_tags.file_id = afiles.id
            GROUP BY atags.id
            ORDER BY atags.name ASC".to_string();
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

    /// Get tag name by id
    pub fn get_name(tag_id: i64) -> Result<String, String> {
        let conn = open_conn()?;
        let result = conn
            .query_row(
                "SELECT name FROM atags WHERE id = ?1",
                params![tag_id],
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
                "SELECT t.id, t.name, 0 as count
                FROM atags t
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

    /// Delete a tag from the database. This will also remove all its associations with files.
    pub fn delete(tag_id: i64) -> Result<usize, String> {
        let conn = open_conn()?;
        let result = conn
            .execute("DELETE FROM atags WHERE id = ?1", params![tag_id])
            .map_err(|e| e.to_string())?;
        Ok(result)
    }

    /// Rename a tag
    pub fn rename(tag_id: i64, new_name: &str) -> Result<usize, String> {
        let conn = open_conn()?;
        let result = conn
            .execute(
                "UPDATE atags SET name = ?1 WHERE id = ?2",
                params![new_name, tag_id],
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

impl Person {
    /// Get all persons with face counts and pre-stored thumbnail
    /// Optimized: single query, no runtime image processing
    pub fn get_all() -> Result<Vec<Self>, String> {
        let conn = open_conn()?;
        
        // Single query with JOIN for count, directly fetch pre-stored thumbnail
        let query = "
            SELECT p.id, p.name, COUNT(f.id) as count, p.thumbnail
            FROM persons p
            LEFT JOIN faces f ON f.person_id = p.id
            GROUP BY p.id
            ORDER BY count DESC, p.name ASC
        ";
        
        let mut stmt = conn.prepare(query).map_err(|e| e.to_string())?;
        
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
    
    /// Generate thumbnail for a person from their cover face or best quality face
    /// Returns the thumbnail as JPEG bytes
    fn generate_thumbnail(conn: &Connection, person_id: i64, cover_face_id: Option<i64>) -> Result<Option<Vec<u8>>, String> {
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
            let is_valid: bool = conn.query_row(
                "SELECT EXISTS(SELECT 1 FROM faces WHERE id = ?1 AND person_id = ?2)",
                params![fid, person_id],
                |row| row.get(0),
            ).unwrap_or(false);
            
            if is_valid { fid } else {
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
        
        let row: Result<(i64, String, Option<u32>, Option<u32>, Option<i32>, String, String), _> = conn.query_row(
            query,
            params![face_id],
            |row| Ok((
                row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, 
                row.get(5)?, row.get(6)?
            )),
        );

        let (file_id, bbox_json, orig_w_opt, orig_h_opt, orientation_opt, file_name, folder_path) = match row {
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

        // Helper to load thumbnail from DB
        let load_thumbnail = || -> Option<(image::DynamicImage, u32, u32)> {
            let thumb_data: Option<Vec<u8>> = conn.query_row(
                "SELECT thumb_data FROM athumbs WHERE file_id = ?1",
                params![file_id],
                |row| row.get(0),
            ).ok()?;
            
            let data = thumb_data?;
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
        let crop_w = (face_w * (1.0 + 2.0 * expansion)).min((img_w.saturating_sub(crop_x)) as f32) as u32;
        let crop_h = (face_h * (1.0 + 2.0 * expansion)).min((img_h.saturating_sub(crop_y)) as f32) as u32;
        
        if crop_w > 0 && crop_h > 0 && crop_x < img_w && crop_y < img_h {
            // Use crop() for DynamicImage type consistency
            let mut cropped = img.crop(crop_x, crop_y, crop_w.min(img_w - crop_x), crop_h.min(img_h - crop_y));
            
            // Resize if too large
            let max_thumb_size = 200;
            if cropped.width() > max_thumb_size || cropped.height() > max_thumb_size {
                cropped = cropped.resize(max_thumb_size, max_thumb_size, image::imageops::FilterType::Lanczos3);
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
        let cover_face_id: Option<i64> = conn.query_row(
            "SELECT cover_face_id FROM persons WHERE id = ?1",
            params![person_id],
            |row| row.get(0),
        ).optional().map_err(|e| e.to_string())?.flatten();
        
        // Generate thumbnail
        let thumbnail = Self::generate_thumbnail(&conn, person_id, cover_face_id)?;
        
        // Update in database
        conn.execute(
            "UPDATE persons SET thumbnail = ?1 WHERE id = ?2",
            params![thumbnail, person_id],
        ).map_err(|e| e.to_string())?;
        
        Ok(())
    }
    
    /// Update thumbnails for all persons (called after clustering)
    pub fn update_all_thumbnails() -> Result<(), String> {
        let conn = open_conn()?;
        
        // Get all person IDs and their cover_face_ids
        let mut stmt = conn.prepare(
            "SELECT id, cover_face_id FROM persons"
        ).map_err(|e| e.to_string())?;
        
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
    pub bbox: String,           // JSON: {"x": f32, "y": f32, "width": f32, "height": f32, "confidence": f32}
    pub embedding: Option<Vec<u8>>, // 512-dimensional float32 embedding as bytes
    pub person_id: Option<i64>,
    pub person_name: Option<String>,
    pub created_at: i64,
}

impl Face {
    /// Add a new face to the database
    pub fn add(file_id: i64, bbox: &str, embedding: &[f32]) -> Result<i64, String> {
        let conn = open_conn()?;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        
        // Convert f32 embedding to bytes
        let embedding_bytes: Vec<u8> = embedding
            .iter()
            .flat_map(|f| f.to_le_bytes())
            .collect();
        
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
        conn.execute("BEGIN TRANSACTION", params![]).map_err(|e| e.to_string())?;
        
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
        
        conn.execute("COMMIT", params![]).map_err(|e| e.to_string())?;
        
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

    /// Get ALL faces (for re-clustering)
    pub fn get_all() -> Result<Vec<Self>, String> {
        let conn = open_conn()?;
        let mut stmt = conn
            .prepare(
                "SELECT id, file_id, bbox, embedding, person_id, created_at FROM faces",
            )
            .map_err(|e| e.to_string())?;
        
        let faces = stmt
            .query_map([], |row| {
                Ok(Self {
                    id: row.get(0)?,
                    file_id: row.get(1)?,
                    bbox: row.get(2)?,
                    embedding: row.get(3)?,
                    person_id: row.get(4)?,
                    created_at: row.get(5)?,
                    person_name: None,
                })
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
            .query_map([], |row| Ok((
                row.get(0)?, 
                row.get(1)?,
                row.get(2)?,
                row.get(3)?
            )))
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        
        Ok(files)
    }

    /// Mark a file as scanned for faces
    /// status: 1 = has faces, 2 = no faces found
    pub fn mark_scanned(file_id: i64, status: i32) -> Result<(), String> {
        let conn = open_conn()?;
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
        let processed: i64 = conn.query_row(
            "SELECT COUNT(*) FROM afiles WHERE has_faces > 0 AND file_type = 1",
            [],
            |row| row.get(0),
        ).unwrap_or(0);

        // Count total faces
        let faces: i64 = conn.query_row(
            "SELECT COUNT(*) FROM faces",
            [],
            |row| row.get(0),
        ).unwrap_or(0);

        Ok((processed as usize, faces as usize))
    }

    /// Get full statistics for face indexing
    /// Returns (total_images, processed_images, unprocessed_images, total_faces)
    pub fn get_stats_full() -> Result<(usize, usize, usize, usize), String> {
        let conn = open_conn()?;
        
        let total: i64 = conn.query_row(
            "SELECT COUNT(*) FROM afiles WHERE file_type = 1",
            [],
            |row| row.get(0),
        ).unwrap_or(0);

        let processed: i64 = conn.query_row(
            "SELECT COUNT(*) FROM afiles WHERE has_faces > 0 AND file_type = 1",
            [],
            |row| row.get(0),
        ).unwrap_or(0);

        let faces: i64 = conn.query_row(
            "SELECT COUNT(*) FROM faces",
            [],
            |row| row.get(0),
        ).unwrap_or(0);

        let unprocessed = total - processed;

        Ok((total as usize, processed as usize, unprocessed as usize, faces as usize))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ACamera {
    pub make: String,
    pub models: Vec<String>,
    pub counts: Vec<i64>,
}

impl ACamera {
    // get all camera makes and models from db
    pub fn get_from_db() -> Result<Vec<Self>, String> {
        let conn = open_conn()?;
        let query = "SELECT UPPER(a.e_make), a.e_model, count(a.id) as count
            FROM afiles a
            WHERE a.e_make IS NOT NULL AND a.e_model IS NOT NULL
            GROUP BY UPPER(a.e_make), a.e_model
            ORDER BY UPPER(a.e_make), a.e_model".to_string();

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
            .map(|(make, (models, counts))| Self {
                make,
                models,
                counts,
            })
            .collect();

        // Sort the cameras by make
        cameras.sort_by(|a, b| a.make.cmp(&b.make));

        Ok(cameras)
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
    pub fn get_from_db() -> Result<Vec<Self>, String> {
        let conn = open_conn()?;

        let query = "SELECT COALESCE(a.geo_cc, ''), a.geo_admin1, a.geo_name, count(a.id) as count
            FROM afiles a
            WHERE COALESCE(a.geo_admin1, '') <> '' AND COALESCE(a.geo_name, '') <> ''
            GROUP BY a.geo_cc, a.geo_admin1, a.geo_name
            ORDER BY a.geo_cc, a.geo_admin1, a.geo_name".to_string();

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
            .map(|((cc, admin1), (names, counts))| Self {
                cc,
                admin1,
                names,
                counts,
            })
            .collect();

        // Sort the locations by admin1
        locations.sort_by(|a, b| a.admin1.cmp(&b.admin1));

        Ok(locations)
    }
}

/// get connection to the db
fn open_conn() -> Result<Connection, String> {
    let path = t_config::get_current_db_path()
        .map_err(|e| format!("Failed to get the database file path: {}", e))?;

    let conn = Connection::open(&path).map_err(|e| format!("Failed to open database connection: {}", e))?;
    
    // Enable foreign key constraints
    conn.execute("PRAGMA foreign_keys = ON", [])
        .map_err(|e| format!("Failed to enable foreign keys: {}", e))?;
    
    Ok(conn)
}

/// create all tables if not exists
pub fn create_db() -> Result<(), String> {
    let conn = open_conn()?;

    // Run migrations
    crate::t_migration::check_and_migrate(&conn)?;

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
            total INTEGER DEFAULT 0
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
            created_at INTEGER,
            modified_at INTEGER,
            taken_date INTEGER,
            width INTEGER,
            height INTEGER,
            duration INTEGER,
            is_favorite INTEGER,
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
            FOREIGN KEY (folder_id) REFERENCES afolders(id) ON DELETE CASCADE
        )",
        [],
    )
    .map_err(|e| e.to_string())?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_afiles_folder_id_name ON afiles(folder_id, name)",
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
    let _ = conn.execute("ALTER TABLE afiles ADD COLUMN has_faces INTEGER DEFAULT 0", []);
    
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
    conn.execute(
        "CREATE TABLE IF NOT EXISTS athumbs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            file_id INTEGER NOT NULL UNIQUE,
            error_code INTEGER NOT NULL,
            thumb_data BLOB,
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

    Ok(())
}

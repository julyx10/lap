/**
 * project: jc-photo
 * author:  julyxx
 * email:   tiangle@gmail.com
 * GitHub:  /julyx10
 * date:    2024-08-08
 */
use std::collections::HashMap;
// use std::fs;
// use dirs;
use base64::{engine::general_purpose, Engine};
use exif::Tag;
use rusqlite::{params, Connection, OptionalExtension, Result};
use serde::{Deserialize, Serialize};

use crate::t_utils;
// use crate::t_opencv;

/// Define the Album struct
#[derive(Debug, Serialize, Deserialize)]
pub struct Album {
    pub id: Option<i64>,               // unique id (autoincrement by db)
    pub name: String,                  // album name (default is folder name)
    pub path: String,                  // folder path
    pub description: Option<String>,   // album description
    pub avatar_id: Option<i64>,        // album avatar id ( from files table)
    pub display_order_id: Option<i64>, // display order id
    pub created_at: Option<u64>,       // folder create time
    pub modified_at: Option<u64>,      // folder modified time
}

impl Album {
    /// create a new album
    fn new(path: &str) -> Result<Self, String> {
        let file_info = t_utils::FileInfo::new(path)?;
        Ok(Self {
            id: None,
            name: file_info.file_name,
            path: file_info.file_path,
            description: None,
            avatar_id: None,
            display_order_id: None,
            created_at: file_info.created,
            modified_at: file_info.modified,
        })
    }

    /// Function to construct `Self` from a database row
    fn from_row(row: &rusqlite::Row) -> Result<Self, rusqlite::Error> {
        Ok(Self {
            id: Some(row.get(0)?),
            name: row.get(1)?,
            path: row.get(2)?,
            description: row.get(3)?,
            avatar_id: row.get(4)?,
            display_order_id: row.get(5)?,
            created_at: row.get(6)?,
            modified_at: row.get(7)?,
        })
    }

    /// fetch an album from db by path
    fn fetch(path: &str) -> Result<Option<Self>, String> {
        let conn = open_conn()?;
        let result = conn.query_row(
            "SELECT id, name, path, description, avatar_id, display_order_id, created_at, modified_at 
            FROM albums WHERE path = ?1",
            params![path],
            |row| Self::from_row(row)
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
            "INSERT INTO albums (name, path, description, avatar_id, display_order_id, created_at, modified_at) 
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                self.name,
                self.path,
                self.description,
                self.avatar_id,
                self.display_order_id,
                self.created_at,
                self.modified_at
            ],
        ).map_err(|e| e.to_string())?;
        Ok(result)
    }

    /// add the album into db if not exists
    pub fn add_to_db(path: &str) -> Result<Self, String> {
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

    /// Get all albums from the db
    pub fn get_all_albums() -> Result<Vec<Self>, String> {
        let conn = open_conn()?;
        let mut stmt = conn.prepare(
            "SELECT id, name, path, description, avatar_id, display_order_id, created_at, modified_at
            FROM albums ORDER BY display_order_id ASC"
        ).map_err(|e| e.to_string())?;

        // Execute the query and map the result to Album structs
        let albums_iter = stmt.query_map([], |row| Self::from_row(row))
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
            "SELECT id, name, path, description, avatar_id, display_order_id, created_at, modified_at
            FROM albums WHERE id = ?1",
            params![id],
            |row| Self::from_row(row)
        ).map_err(|e| e.to_string())?;
        Ok(result)
    }

    /// update a column value
    pub fn update_column(id: i64, column: &str, value: &dyn rusqlite::ToSql) -> Result<usize, String> {
        let conn = open_conn()?;
        let query = format!("UPDATE albums SET {} = ?1 WHERE id = ?2", column);
        let result = conn
            .execute(&query, params![value, id])
            .map_err(|e| e.to_string())?;
        Ok(result)
    }
}

/// Define the album's folder struct
#[derive(Debug, Serialize, Deserialize)]
pub struct AFolder {
    pub id: Option<i64>,          // unique id (autoincrement by db)
    pub album_id: i64,            // album id (from albums table)
    pub parent_id: i64,           // parent folder id
    pub name: String,             // folder name
    pub path: String,             // folder path
    pub is_favorite: Option<bool>,  // is favorite
    pub created_at: Option<u64>,  // folder create time
    pub modified_at: Option<u64>, // folder modified time
}

impl AFolder {
    /// create a new folder struct
    fn new(album_id: i64, parent_id: i64, folder_path: &str) -> Result<Self, String> {
        let file_info = t_utils::FileInfo::new(folder_path)?;
        Ok(Self {
            id: None,
            album_id,
            parent_id,
            name: file_info.file_name,
            path: folder_path.to_string(),
            is_favorite: None,
            created_at: file_info.created,
            modified_at: file_info.modified,
        })
    }

    /// Function to construct `Self` from a database row
    fn from_row(row: &rusqlite::Row) -> Result<Self, rusqlite::Error> {
        Ok(Self {
            id: Some(row.get(0)?),
            album_id: row.get(1)?,
            parent_id: row.get(2)?,
            name: row.get(3)?,
            path: row.get(4)?,
            is_favorite: row.get(5)?,
            created_at: row.get(6)?,
            modified_at: row.get(7)?,
        })
    }
    
    /// fetch a folder row from db (by path)
    fn fetch(folder_path: &str) -> Result<Option<Self>, String> {
        let conn = open_conn()?;
        let result = conn
            .query_row(
                "SELECT a.id, a.album_id, a.parent_id, a.name, a.path, a.is_favorite, a.created_at, a.modified_at
                FROM afolders a
                WHERE a.path = ?1",
                params![folder_path],
                |row| Self::from_row(row)
            ).optional()
            .map_err(|e| e.to_string())?;
        Ok(result)
    }

    /// insert a folder into db
    fn insert(&self) -> Result<usize, String> {
        let conn = open_conn()?;
        let result = conn
            .execute(
                "INSERT INTO afolders (album_id, parent_id, name, path, is_favorite, created_at, modified_at) 
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    self.album_id,
                    self.parent_id,
                    self.name,
                    self.path,
                    self.is_favorite,
                    self.created_at,
                    self.modified_at
                ],
            )
            .map_err(|e| e.to_string())?;
        Ok(result)
    }

    /// insert the folder to db if not exists
    pub fn add_to_db(album_id: i64, parent_id: i64, folder_path: &str) -> Result<Self, String> {
        // Check if the path already exists
        let existing_folder = Self::fetch(folder_path);
        if let Ok(Some(folder)) = existing_folder {
            return Ok(folder);
        }

        // insert the new folder into the database
        // when insert a new folder, save album_id value
        Self::new(album_id, parent_id, folder_path)?.insert()?;

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
            ).map_err(|e| e.to_string())?;
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
            ).map_err(|e| e.to_string())?;
        Ok(result)
    }

    /// delete a folder and all subfolders from db
    pub fn delete_folder(folder_path: &str) -> Result<usize, String> {
        let conn = open_conn()?;
        let result = conn
            .execute(
                "DELETE FROM afolders WHERE path LIKE ?1 || '%'",
                params![folder_path],
            )
            .map_err(|e| e.to_string())?;
        Ok(result)
    }

    // update a column value
    pub fn update_column(id: i64, column: &str, value: &dyn rusqlite::ToSql) -> Result<usize, String> {
        let conn = open_conn()?;
        let query = format!("UPDATE afolders SET {} = ?1 WHERE id = ?2", column);
        let result = conn
            .execute(&query, params![value, id])
            .map_err(|e| e.to_string())?;
        Ok(result)
    }

    // get folder info by id
    // pub fn get_folder_by_id(id: i64) -> Result<Self, String> {
    //     let conn = open_conn()?;
    //     let result = conn
    //         .query_row(
    //             "SELECT id, album_id, parent_id, name, path, is_favorite, created_at, modified_at 
    //             FROM afolders WHERE id = ?1",
    //             params![id],
    //             |row| Self::from_row(row)
    //         ).map_err(|e| e.to_string())?;
    //     Ok(result)
    // }

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
        let mut stmt = conn
            .prepare(
                "SELECT id, album_id, parent_id, name, path, is_favorite, created_at, modified_at 
                FROM afolders WHERE is_favorite = 1",
            )
            .map_err(|e| e.to_string())?;

        let rows = stmt
            .query_map(params![], |row| Self::from_row(row))
            .map_err(|e| e.to_string())?;

        let mut folders = Vec::new();
        for folder in rows {
            folders.push(folder.unwrap());
        }

        Ok(folders)
    }

    // recurse all parent folder id (deprecated)
    // pub fn recurse_all_parents_id(folder_id: i64) -> Result<Vec<i64>, String> {
    //     let conn = open_conn()?;

    //     let mut stmt = conn
    //         .prepare(
    //             "WITH RECURSIVE parent_hierarchy AS (
    //             SELECT parent_id
    //             FROM afolders
    //             WHERE id = ?1
    //             UNION ALL
    //             SELECT f.parent_id
    //             FROM afolders f
    //             INNER JOIN parent_hierarchy ph ON f.id = ph.parent_id
    //             WHERE f.parent_id != 0
    //         )
    //         SELECT parent_id FROM parent_hierarchy;",
    //         )
    //         .map_err(|e| e.to_string())?;

    //     let parent_ids = stmt
    //         .query_map(params![folder_id], |row| row.get(0))
    //         .map_err(|e| e.to_string())?
    //         .collect::<Result<Vec<i64>, _>>()
    //         .map_err(|e| e.to_string())?;

    //     Ok(parent_ids)
    // }

}

/// Define the album file struct
#[derive(Debug, Serialize, Deserialize)]
pub struct AFile {
    pub id: Option<i64>, // unique id (autoincrement by db)
    pub folder_id: i64,  // folder id (from folders table)

    // file basic info
    pub name: String,               // file name
    pub name_pinyin: Option<String>,// file name pinyin(for sort)
    pub size: u64,                  // file size
    pub file_type: Option<i64>,     // file type (0: all, 1: image, 2: video, 3: audio, 4: other)
    pub created_at: Option<u64>,    // file create time
    pub modified_at: Option<u64>,   // file modified time
    pub taken_date: Option<String>, // taken date(yyyy-mm-dd) for calendar view

    // image dimensions
    pub width: Option<u32>,  // image width
    pub height: Option<u32>, // image height

    // extra info
    pub is_favorite: Option<bool>, // is favorite
    pub rotate: Option<i32>,       // rotate angle (0, 90, 180, 270)
    pub comments: Option<String>,  // comments
    pub deleted_at: Option<u64>,   // deleted date

    // exif info
    pub e_make: Option<String>,  // camera make
    pub e_model: Option<String>, // camera model
    pub e_date_time: Option<String>,
    pub e_exposure_time: Option<String>,
    pub e_f_number: Option<String>,
    pub e_focal_length: Option<String>,
    pub e_iso_speed: Option<String>,
    pub e_flash: Option<String>,    // flash
    pub e_orientation: Option<u32>, // orientation

    // gps
    pub gps_latitude: Option<String>,
    pub gps_longitude: Option<String>,
    pub gps_altitude: Option<String>,

    // output only
    pub file_path: Option<String>,  // file path (for webview)
    pub album_id: Option<i64>,      // album id (for webview)
    pub album_name: Option<String>, // album name (for webview)
}

impl AFile {
    /// create a new file struct
    fn new(folder_id: i64, file_path: &str, file_type: i64) -> Result<Self, String> {
        let file_info = t_utils::FileInfo::new(file_path)?;
        let (width, height) = t_utils::get_image_size(file_path)?;

        // open the file
        let file =
            std::fs::File::open(file_path).map_err(|e| format!("Error opening file: {}", e))?;
        // Create a buffered reader
        let mut bufreader = std::io::BufReader::new(&file);
        // Create an EXIF reader and attempt to read EXIF data
        let exifreader = exif::Reader::new();
        let exif = exifreader.read_from_container(&mut bufreader).ok();

        Ok(Self {
            id: None,
            folder_id,
            file_type: Some(file_type),
            name: file_info.file_name.clone(),
            name_pinyin: Some(t_utils::convert_to_pinyin(file_info.file_name.as_str())), // convert to pinyin
            size: file_info.file_size,
            created_at: file_info.created,
            modified_at: file_info.modified,
            taken_date: Self::get_exif_field(&exif, Tag::DateTimeOriginal)
                .map(|exif_date| {
                    if exif_date.len() >= 10 {
                        Some(exif_date[..10].to_string())
                    } else {
                        Some(exif_date) // Fallback to the whole string if it’s shorter than expected
                    }
                })
                .unwrap_or_else(|| file_info.modified_str),

            width: Some(width),
            height: Some(height),

            is_favorite: None,
            rotate: None,
            comments: None,
            deleted_at: None,

            e_make: Self::get_exif_field(&exif, Tag::Make).map(|s| s.to_uppercase()),
            e_model: Self::get_exif_field(&exif, Tag::Model),
            e_date_time: Self::get_exif_field(&exif, Tag::DateTimeOriginal),
            e_exposure_time: Self::get_exif_field(&exif, Tag::ExposureTime),
            e_f_number: Self::get_exif_field(&exif, Tag::FNumber),
            e_focal_length: Self::get_exif_field(&exif, Tag::FocalLength),
            e_iso_speed: Self::get_exif_field(&exif, Tag::PhotographicSensitivity),
            e_flash: Self::get_exif_field(&exif, Tag::Flash),
            e_orientation: Self::get_exif_orientation_field(&exif, Tag::Orientation),

            gps_latitude: Self::get_exif_field(&exif, Tag::GPSLatitude),
            gps_longitude: Self::get_exif_field(&exif, Tag::GPSLongitude),
            gps_altitude: Self::get_exif_field(&exif, Tag::GPSAltitude),

            file_path: None,
            album_id: None,
            album_name: None,
        })
    }

    fn get_exif_field(exif: &Option<exif::Exif>, tag: exif::Tag) -> Option<String> {
        exif.as_ref()
            .and_then(|exif_data| {
                exif_data.get_field(tag, exif::In::PRIMARY).map(|field| {
                    format!("{}", field.display_value().with_unit(exif_data)).replace("\"", "")
                })
            })
            .map(|s| {
                s.trim_end_matches(|c| {
                    char::is_ascii_punctuation(&c) || char::is_ascii_whitespace(&c)
                })
                .to_string()
            }) // trim all trailing commas and spaces,
    }

    fn get_exif_orientation_field(exif: &Option<exif::Exif>, tag: exif::Tag) -> Option<u32> {
        exif.as_ref().and_then(|exif_data| {
            exif_data
                .get_field(tag, exif::In::PRIMARY)
                .and_then(|field| field.value.get_uint(0)) // Return u64 directly
                .or(Some(1)) // If no orientation is found, default to 1 (normal orientation)
        })
    }

    /// insert a file into db
    fn insert(&self) -> Result<usize, String> {
        let conn = open_conn()?;
        let result = conn.execute(
            "INSERT INTO afiles (
                folder_id, 
                name, name_pinyin, size, file_type, created_at, modified_at, taken_date,
                width, height,
                is_favorite, rotate, comments, deleted_at,
                e_make, e_model, e_date_time, e_exposure_time, e_f_number, e_focal_length, e_iso_speed, e_flash, e_orientation,
                gps_latitude, gps_longitude, gps_altitude
            ) 
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24, ?25, ?26)",
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

                self.is_favorite,
                self.rotate,
                self.comments,
                self.deleted_at,

                self.e_make,
                self.e_model,
                self.e_date_time,
                self.e_exposure_time,
                self.e_f_number,
                self.e_focal_length,
                self.e_iso_speed,
                self.e_flash,
                self.e_orientation,

                self.gps_latitude,
                self.gps_longitude,
                self.gps_altitude,
            ]
        ).map_err(|e| e.to_string())?;
        Ok(result)
    }

    // delete a file from db
    pub fn delete(id: i64) -> Result<usize, String> {
        let conn = open_conn()?;
        let result = conn
            .execute(
                "DELETE FROM afiles WHERE id = ?1",
                params![id],
            )
            .map_err(|e| e.to_string())?;
        Ok(result)
    }

    // Helper function to build the count SQL query
    fn build_count_query() -> String {
        String::from("
            SELECT COUNT(*), SUM(a.size)
            FROM afiles a 
            LEFT JOIN afolders b ON a.folder_id = b.id
            LEFT JOIN albums c ON b.album_id = c.id
        ")
    }

    // build the base SQL query
    fn build_base_query() -> String {
        String::from("
            SELECT a.id, a.folder_id, 
                a.name, a.name_pinyin, a.size, a.file_type, a.created_at, a.modified_at, a.taken_date,
                a.width, a.height,
                a.is_favorite, a.rotate, a.comments, a.deleted_at,
                a.e_make, a.e_model, a.e_date_time, a.e_exposure_time, a.e_f_number, a.e_focal_length, a.e_iso_speed, a.e_flash, a.e_orientation,
                a.gps_latitude, a.gps_longitude, a.gps_altitude,
                b.path,
                c.id AS album_id, c.name AS album_name
            FROM afiles a 
            LEFT JOIN afolders b ON a.folder_id = b.id
            LEFT JOIN albums c ON b.album_id = c.id
        ")
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

            is_favorite: row.get(11)?,
            rotate: row.get(12)?,
            comments: row.get(13)?,
            deleted_at: row.get(14)?,

            e_make: row.get(15)?,
            e_model: row.get(16)?,
            e_date_time: row.get(17)?,
            e_exposure_time: row.get(18)?,
            e_f_number: row.get(19)?,
            e_focal_length: row.get(20)?,
            e_iso_speed: row.get(21)?,
            e_flash: row.get(22)?,
            e_orientation: row.get(23)?,

            gps_latitude: row.get(24)?,
            gps_longitude: row.get(25)?,
            gps_altitude: row.get(26)?,

            file_path: Some(t_utils::get_file_path(
                row.get::<_, String>(27)?.as_str(),
                row.get::<_, String>(2)?.as_str(),
            )),
            album_id: row.get(28)?,
            album_name: row.get(29)?,
        })
    }

    // query the count and sum by sql
    fn query_count_and_sum(sql: &str, params: &[&dyn rusqlite::ToSql]) -> Result<(i64, i64), String> {
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
            .query_map(params, |row| Self::from_row(row))
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
                |row| Self::from_row(row),
            )
            .optional()
            .map_err(|e| e.to_string())?;

        Ok(result)
    }

    /// insert a file into db if not exists
    pub fn add_to_db(folder_id: i64, file_path: &str, file_type: i64) -> Result<Self, String> {
        // Check if the file exists
        let existing_file = Self::fetch(folder_id, file_path)?;
        if let Some(file) = existing_file {
            // check file modified time
            let file_info = t_utils::FileInfo::new(file_path)?;
            if file.modified_at != file_info.modified {
                // delete the old file
                Self::delete(folder_id)?;
            } else {
                return Ok(file);
            }
        }

        // insert the new file into the database
        Self::new(folder_id, file_path, file_type)?.insert()?;

        // return the newly inserted file
        let new_file = Self::fetch(folder_id, file_path)?;
        Ok(new_file.unwrap())
    }

    /// get a file info from db by file_id
    pub fn get_file_info(file_id: i64) -> Result<Option<Self>, String> {
        let conn = open_conn()?;

        // Prepare the SQL query using the base query and adding the condition for file ID
        let sql = format!("{} WHERE a.id = ?1", Self::build_base_query());

        // Execute the query with file_id as the parameter
        let result = conn
            .query_row(&sql, params![file_id], |row| Self::from_row(row))
            .optional()
            .map_err(|e| e.to_string())?;

        Ok(result)
    }

    /// update a file column value
    pub fn update_column(file_id: i64, column: &str, value: &dyn rusqlite::ToSql) -> Result<usize, String> {
        let conn = open_conn()?;
        let query = format!("UPDATE afiles SET {} = ?1 WHERE id = ?2", column);
        let result = conn
            .execute(&query, params![value, file_id])
            .map_err(|e| e.to_string())?;
        Ok(result)
    }

    /// get all taken dates from db
    pub fn get_taken_dates() -> Result<Vec<(String, i64)>, String> {
        let conn = open_conn()?;
        let mut stmt = conn
            .prepare(
                "SELECT taken_date, count(1) 
            FROM afiles 
            WHERE taken_date IS NOT NULL
            GROUP BY taken_date
            ORDER BY taken_date ASC",
            )
            .map_err(|e| e.to_string())?;

        // Execute the query and collect results
        let taken_dates_iter = stmt
            .query_map(params![], |row| {
                let date: String = row.get(0)?;
                let count: i64 = row.get(1)?;
                Ok((date, count))
            })
            .map_err(|e| e.to_string())?;

        // Collect the results into a vector
        let mut results = Vec::new();
        for row in taken_dates_iter {
            match row {
                Ok((date, value)) => results.push((date, value)),
                Err(e) => return Err(format!("Failed to retrieve row: {}", e)),
            }
        }

        Ok(results)
    }

    // get total count and size of files
    pub fn get_count_and_sum(
        search_text: &str, search_file_type: i64,
        start_date: &str, end_date: &str,
        make: &str, model: &str,
        is_favorite: bool, is_deleted: bool
    ) -> Result<(i64, i64), String> {
        let mut conditions = Vec::new();
        let mut params: Vec<&dyn rusqlite::ToSql> = Vec::new();
    
        let like_pattern = format!("%{}%", search_text);
        if !search_text.is_empty() {
            conditions.push("a.name LIKE ? COLLATE NOCASE");
            params.push(&like_pattern);
        }
    
        if search_file_type > 0 {
            conditions.push("a.file_type = ?");
            params.push(&search_file_type);
        }
    
        if !start_date.is_empty() {
            if end_date.is_empty() {
                conditions.push("a.taken_date = ?");
                params.push(&start_date);
            } else {
                conditions.push("a.taken_date >= ? AND a.taken_date <= ?");
                params.push(&start_date);
                params.push(&end_date);
            }
        }
    
        if !make.is_empty() {
            conditions.push("a.e_make = ?");
            params.push(&make);
            if !model.is_empty() {
                conditions.push("a.e_model = ?");
                params.push(&model);
            }
        }
    
        if is_favorite {
            conditions.push("a.is_favorite = 1");
        }
    
        if is_deleted {
            conditions.push("a.deleted_at IS NOT NULL");
        } else {
            conditions.push("a.deleted_at IS NULL");
        }
    
        let mut query = Self::build_count_query();
        if !conditions.is_empty() {
            query.push_str(" WHERE ");
            query.push_str(&conditions.join(" AND "));
        }
    
        Self::query_count_and_sum(&query, &params)
    }

    /// get files
    pub fn get_files(
        search_text: &str, search_file_type: i64,
        sort_type: i64, sort_order: i64,
        start_date: &str, end_date: &str,
        make: &str, model: &str,
        is_favorite: bool, is_deleted: bool,
        page_size: i64, offset: i64,
    ) -> Result<Vec<Self>, String> {

        // conditions
        let mut conditions = Vec::new();
        let mut params: Vec<&dyn rusqlite::ToSql> = Vec::new();
    
        let like_pattern = format!("%{}%", search_text);
        if !search_text.is_empty() {
            conditions.push("a.name LIKE ? COLLATE NOCASE");
            params.push(&like_pattern);
        }
    
        if search_file_type > 0 {
            conditions.push("a.file_type = ?");
            params.push(&search_file_type);
        }
    
        if !start_date.is_empty() {
            if end_date.is_empty() {
                conditions.push("a.taken_date = ?");
                params.push(&start_date);
            } else {
                conditions.push("a.taken_date >= ? AND a.taken_date <= ?");
                params.push(&start_date);
                params.push(&end_date);
            }
        }
    
        if !make.is_empty() {
            conditions.push("a.e_make = ?");
            params.push(&make);
            if !model.is_empty() {
                conditions.push("a.e_model = ?");
                params.push(&model);
            }
        }
    
        if is_favorite {
            conditions.push("a.is_favorite = 1");
        }
    
        if is_deleted {
            conditions.push("a.deleted_at IS NOT NULL");
        } else {
            conditions.push("a.deleted_at IS NULL");
        }
    
        let mut query = Self::build_base_query();
        if !conditions.is_empty() {
            query.push_str(" WHERE ");
            query.push_str(&conditions.join(" AND "));
        }
    
        // sort
        match sort_type {
            0 => query.push_str(" ORDER BY a.name_pinyin"),
            1 => query.push_str(" ORDER BY a.size"),
            2 => query.push_str(" ORDER BY a.width, a.height"),     // resolution
            3 => query.push_str(" ORDER BY a.created_at"),
            4 => query.push_str(" ORDER BY a.modified_at"),
            5 => query.push_str(" ORDER BY a.taken_date"),
            _ => query.push_str(" ORDER BY a.name_pinyin"),
        }
        match sort_order {
            0 => query.push_str(" ASC"),
            1 => query.push_str(" DESC"),
            _ => query.push_str(" ASC"),
        }

        // paging
        query.push_str(" LIMIT ? OFFSET ?");
        params.push(&page_size);
        params.push(&offset);

        Self::query_files(&query, &params)
    }
}

/// Define the album thumbnail struct
#[derive(Debug, Serialize, Deserialize)]
pub struct AThumb {
    pub id: Option<i64>, // unique id (autoincrement by db)
    pub file_id: i64,    // file id (from files table)

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
        orientation: i32,
        thumbnail_size: u32,
    ) -> Result<Option<Self>, String> {
        Ok(Some(Self {
            id: None,
            file_id,
            // thumb_data: t_opencv::get_thumbnail(file_path, orientation, thumbnail_size)?,
            thumb_data: t_utils::get_thumbnail(file_path, orientation, thumbnail_size)?,
            thumb_data_base64: None,
        }))
    }

    /// insert a thumbnail into db
    fn insert(&self) -> Result<usize, String> {
        let conn = open_conn()?;
        let result = conn
            .execute(
                "INSERT INTO athumbs (file_id, thumb_data) 
                VALUES (?1, ?2)",
                params![self.file_id, self.thumb_data,],
            )
            .map_err(|e| e.to_string())?;
        Ok(result)
    }

    /// fetch a thumbnail from db by file_id
    fn fetch(file_id: i64) -> Result<Option<Self>, String> {
        let conn = open_conn()?;
        let result = conn
            .query_row(
                "SELECT id, file_id, thumb_data 
                FROM athumbs WHERE file_id = ?1",
                params![file_id],
                |row| {
                    Ok(Self {
                        id: Some(row.get(0)?),
                        file_id: row.get(1)?,
                        thumb_data: None,
                        thumb_data_base64: Some(
                            general_purpose::STANDARD.encode(row.get::<_, Vec<u8>>(2)?),
                        ),
                    })
                },
            )
            .optional()
            .map_err(|e| e.to_string())?;
        Ok(result)
    }

    /// insert a thumbnail into db if not exists
    pub fn add_to_db(
        file_id: i64,
        file_path: &str,
        orientation: i32,
        thumbnail_size: u32,
    ) -> Result<Option<Self>, String> {
        // Check if the thumbnail exists
        let existing_thumbnail = Self::fetch(file_id);
        if let Ok(Some(thumbnail)) = existing_thumbnail {
            return Ok(Some(thumbnail));
        }

        // Insert the new thumbnail into the database
        let new_thumbnail = Self::new(file_id, file_path, orientation, thumbnail_size);
        if let Ok(Some(athumb)) = new_thumbnail {
            athumb.insert()?;
            return Ok(Self::fetch(file_id)?);
        }

        Ok(None)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ACamera {
    pub make: String,
    pub models: Vec<String>,
}

impl ACamera {
    // get all camera makes and models from db
    pub fn get_from_db() -> Result<Vec<Self>, String> {
        let conn = open_conn()?;
        let mut stmt = conn
            .prepare(
                "SELECT e_make, e_model 
                FROM afiles 
                WHERE e_make IS NOT NULL AND e_model IS NOT NULL
                GROUP BY e_make, e_model
                ORDER BY e_make, e_model",
            )
            .map_err(|e| e.to_string())?;

        let rows = stmt
            .query_map(params![], |row| {
                let make: String = row.get(0)?;
                let model: String = row.get(1)?;
                Ok((make, model))
            })
            .map_err(|e| e.to_string())?;

        let mut hash_map: HashMap<String, Vec<String>> = HashMap::new();

        for row in rows {
            let (make, model) = row.unwrap();
            hash_map.entry(make).or_insert_with(Vec::new).push(model);
        }

        let mut cameras: Vec<Self> = hash_map
            .into_iter()
            .map(|(make, models)| Self { make, models })
            .collect();

        // Sort the cameras by make
        cameras.sort_by(|a, b| a.make.cmp(&b.make));

        // // Sort the models within each ACamera instance
        // for camera in &mut cameras {
        //     camera.models.sort();
        // }

        Ok(cameras)
    }
}

/// get connection to the db
fn open_conn() -> Result<Connection, String> {
    let path = t_utils::get_db_file_path()
        .map_err(|e| format!("Failed to get the database file path: {}", e))?;

    Connection::open(&path)
        .map_err(|e| format!("Failed to open database connection: {}", e))
}

/// create all tables if not exists
pub fn create_db() -> Result<String> {
    let conn = open_conn().unwrap();

    // albums table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS albums (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            path TEXT NOT NULL,
            description TEXT,
            avatar_id INTEGER,
            display_order_id INTEGER,
            created_at INTEGER,
            modified_at INTEGER
        )",
        [],
    )?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_albums_name ON albums(name)", [])?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_albums_path ON albums(path)", [])?;

    // folders table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS afolders (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            album_id INTEGER NOT NULL,
            parent_id INTEGER NOT NULL,
            name TEXT NOT NULL,
            path TEXT NOT NULL,
            is_favorite INTEGER,
            created_at INTEGER,
            modified_at INTEGER,
            FOREIGN KEY (album_id) REFERENCES albums(id) ON DELETE CASCADE
        )",
        [],
    )?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_afolders_album_id ON afolders(album_id)", [])?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_afolders_name ON afolders(name)", [])?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_afolders_path ON afolders(path)", [])?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_afolders_is_favorite ON afolders(is_favorite)", [])?;

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
            taken_date TEXT,
            width INTEGER,
            height INTEGER,
            is_favorite INTEGER,
            rotate INTEGER,
            comments TEXT,
            deleted_at INTEGER,
            e_make TEXT,
            e_model TEXT,
            e_date_time TEXT,
            e_exposure_time TEXT,
            e_f_number TEXT,
            e_focal_length TEXT,
            e_iso_speed TEXT,
            e_flash TEXT,
            e_orientation INTEGER,
            gps_latitude TEXT,
            gps_longitude TEXT,
            gps_altitude TEXT,
            FOREIGN KEY (folder_id) REFERENCES afolders(id) ON DELETE CASCADE
        )",
        [],
    )?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_afiles_folder_id ON afiles(folder_id)", [])?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_afiles_name ON afiles(name)", [])?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_afiles_name_pinyin ON afiles(name_pinyin)", [])?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_afiles_file_type ON afiles(file_type)", [])?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_afiles_created_at ON afiles(created_at)", [])?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_afiles_modified_at ON afiles(modified_at)", [])?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_afiles_taken_date ON afiles(taken_date)", [])?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_afiles_is_favorite ON afiles(is_favorite)", [])?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_afiles_deleted_at ON afiles(deleted_at)", [])?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_afiles_make_model ON afiles(e_make, e_model)", [])?;

    // file thumbnail table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS athumbs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            file_id INTEGER NOT NULL,
            thumb_data BLOB NOT NULL,
            FOREIGN KEY (file_id) REFERENCES afiles(id) ON DELETE CASCADE
        )",
        [],
    )?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_athumbs_file_id ON athumbs(file_id)", [])?;

    Ok("Database created successfully.".to_string())
}

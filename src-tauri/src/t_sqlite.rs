/**
 * project: jc-photo
 * author:  julyxx
 * email:   tiangle@gmail.com
 * GitHub:  /julyx10
 * date:    2024-08-08
 */
use std::collections::HashMap;
use trash;
use base64::{engine::general_purpose, Engine};
use exif::{Tag, Value};
use reverse_geocoder::ReverseGeocoder;
use rusqlite::{params, Connection, OptionalExtension, Result};
use serde::{Deserialize, Serialize};

use crate::t_utils;
// use crate::t_opencv;

/// Define the Album struct
#[derive(Debug, Serialize, Deserialize)]
pub struct Album {
    pub id: Option<i64>,               // unique id (autoincrement by db)

    // album basic info
    pub name: String,                  // album name (default is folder name)
    pub path: String,                  // folder path
    pub created_at: Option<u64>,       // folder create time
    pub modified_at: Option<u64>,      // folder modified time
    
    // extra info
    pub display_order_id: Option<i64>, // display order id
    pub avatar_id: Option<i64>,        // album avatar id ( from files table)
    pub description: Option<String>,   // album description
    pub is_hidden: Option<bool>,       // is hidden
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
            avatar_id: None,
            description: None,
            is_hidden: None,
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
            avatar_id: row.get(6)?,
            description: row.get(7)?,
            is_hidden: row.get(8)?,
        })
    }

    /// fetch an album from db by path
    fn fetch(path: &str) -> Result<Option<Self>, String> {
        let conn = open_conn()?;
        let result = conn.query_row(
            "SELECT id, name, path, created_at, modified_at, display_order_id, avatar_id, description, is_hidden 
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
            "INSERT INTO albums (name, path, created_at, modified_at, display_order_id, avatar_id, description, is_hidden) 
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                self.name,
                self.path,
                self.created_at,
                self.modified_at,
                self.display_order_id,
                self.avatar_id,
                self.description,
                self.is_hidden,
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
    pub fn get_all_albums(show_hidden_album: bool) -> Result<Vec<Self>, String> {
        let conn = open_conn()?;
        
        let query = if show_hidden_album {
            "SELECT id, name, path, created_at, modified_at, display_order_id, avatar_id, description, is_hidden
            FROM albums
            ORDER BY display_order_id ASC"
        } else {
            "SELECT id, name, path, created_at, modified_at, display_order_id, avatar_id, description, is_hidden
            FROM albums
            WHERE is_hidden IS NULL OR is_hidden = 0
            ORDER BY display_order_id ASC"
        };
        
        let mut stmt = conn.prepare(query).map_err(|e| e.to_string())?;

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
            "SELECT id, name, path, created_at, modified_at, display_order_id, avatar_id, description, is_hidden
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

    // folder basic info
    pub name: String,             // folder name
    pub path: String,             // folder path
    pub created_at: Option<u64>,  // folder create time
    pub modified_at: Option<u64>, // folder modified time

    // extra info
    pub is_favorite: Option<bool>,  // is favorite
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

    pub fn delete_folder(folder_id: i64) -> Result<usize, String> {
        let conn = open_conn()?;
    
        let sql = "SELECT path FROM afolders WHERE id = ?1";
        let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;
        let path: Option<String> = stmt
            .query_row([folder_id], |row| row.get(0))
            .optional()
            .map_err(|e| e.to_string())?;
    
        if let Some(folder_path) = path {
            trash::delete(&folder_path).map_err(|e| e.to_string())?;
    
            // delete database record
            let result = conn.execute("DELETE FROM afolders WHERE id = ?1", [folder_id])
                .map_err(|e| e.to_string())?;
            return Ok(result);
        } else {
            return Err(format!("Folder with id {} not found", folder_id));
        }
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
    // pub fn get_folder_by_id(id: i64) -> Result<Option<Self>, String> {
    //     let conn = open_conn()?;
    //     let result = conn
    //         .query_row(
    //             "SELECT id, album_id, name, path, created_at, modified_at, is_favorite
    //             FROM afolders WHERE id = ?1",
    //             params![id],
    //             |row| Self::from_row(row)
    //         ).optional().map_err(|e| e.to_string())?;
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
    pub fn get_favorite_folders(is_show_hidden: bool) -> Result<Vec<Self>, String> {
        let conn = open_conn()?;
        let hidden_clause = if is_show_hidden { "" } else { "AND (b.is_hidden IS NULL OR b.is_hidden = 0)" };

        let query = format!(
            "SELECT a.id, a.album_id, a.name, a.path, a.created_at, a.modified_at, a.is_favorite
            FROM afolders a
            LEFT JOIN albums b ON a.album_id = b.id
            WHERE a.is_favorite = 1 {}
            ORDER BY a.name",
            hidden_clause
        );

        let mut stmt = conn
            .prepare(query.as_str())
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

    // file taken date
    pub taken_date: Option<String>, // taken date(yyyy-mm-dd) for calendar view

    // image/video
    pub width: Option<u32>,         // image/video width
    pub height: Option<u32>,        // image/video height
    pub duration: Option<u64>,      // video duration

    // extra info
    pub is_favorite: Option<bool>, // is favorite
    pub rotate: Option<i32>,       // rotate angle (0, 90, 180, 270)
    pub comments: Option<String>,  // comments
    pub has_tags: Option<bool>,    // has tags

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
    pub gps_location: Option<String>,
    pub gps_country: Option<String>,
    pub gps_city: Option<String>,

    // output only
    pub file_path: Option<String>,  // file path (for webview)
    pub album_id: Option<i64>,      // album id (for webview)
    pub album_name: Option<String>, // album name (for webview)
}

impl AFile {
    fn new(folder_id: i64, file_path: &str, file_type: i64) -> Result<Self, String> {
        let file_info = t_utils::FileInfo::new(file_path)?;
        let mut duration = 0;
        let (width, height) = 
            match file_type {
                1 => t_utils::get_image_dimensions(file_path)?,
                2 => {
                    duration = t_utils::get_video_duration(file_path)?;
                    t_utils::get_video_dimensions(file_path)?
                },
                _ => (0, 0),
            };

        // open the file
        let file =
            std::fs::File::open(file_path).map_err(|e| format!("Error opening file: {}", e))?;
        // Create a buffered reader
        let mut bufreader = std::io::BufReader::new(&file);
        // Create an EXIF reader and attempt to read EXIF data
        let exifreader = exif::Reader::new();
        let exif = exifreader.read_from_container(&mut bufreader).ok();

        let (gps_latitude, gps_longitude, gps_location, gps_country, gps_city, gps_altitude) = if let Some(exif_data) = &exif {
            let lat_val = exif_data.get_field(Tag::GPSLatitude, exif::In::PRIMARY).and_then(|f| if let Value::Rational(v) = &f.value { Some(v.to_vec()) } else { None });
            let lat_ref = exif_data.get_field(Tag::GPSLatitudeRef, exif::In::PRIMARY).map(|f| f.display_value().to_string());
            let lon_val = exif_data.get_field(Tag::GPSLongitude, exif::In::PRIMARY).and_then(|f| if let Value::Rational(v) = &f.value { Some(v.to_vec()) } else { None });
            let lon_ref = exif_data.get_field(Tag::GPSLongitudeRef, exif::In::PRIMARY).map(|f| f.display_value().to_string());

            let format_dms = |dms: &[exif::Rational], reference: &str| -> String {
                let degrees = dms[0].num as f64 / dms[0].denom as f64;
                let minutes = dms[1].num as f64 / dms[1].denom as f64;
                let seconds = dms[2].num as f64 / dms[2].denom as f64;
                format!("{:.0}°{:.0}′{:.0}″{}", degrees, minutes, seconds, reference.trim())
            };

            let (gps_lat_str, gps_lon_str, location, country, city) = if let (Some(lat_v), Some(lat_r), Some(lon_v), Some(lon_r)) = (lat_val, lat_ref, lon_val, lon_ref) {
                let dec_lat = Self::dms_to_decimal(&lat_v, &lat_r);
                let dec_lon = Self::dms_to_decimal(&lon_v, &lon_r);

                let lat_str = format_dms(&lat_v, &lat_r);
                let lon_str = format_dms(&lon_v, &lon_r);

                let (location, country, city) = if let (Some(lat), Some(lon)) = (dec_lat, dec_lon) {
                    let geocoder = ReverseGeocoder::new();
                    let search_result = geocoder.search((lat, lon));

                    let location = format!("{}, {}", search_result.record.admin2, search_result.record.admin1);
                    let country = Some(search_result.record.cc.clone());
                    let city = Some(search_result.record.name.clone());
                
                    (Some(location), country, city)
                } else {
                    (None, None, None)
                };
                (Some(lat_str), Some(lon_str), location, country, city)
            } else {
                (None, None, None, None, None)
            };

            let altitude = exif_data.get_field(Tag::GPSAltitude, exif::In::PRIMARY)
                .and_then(|field| match &field.value {
                    Value::Rational(v) if !v.is_empty() => {
                        let alt = v[0].num as f64 / v[0].denom as f64;
                        Some(format!("{:.2}", alt))
                    },
                    _ => None,
                });

            (gps_lat_str, gps_lon_str, location, country, city, altitude)
        } else {
            (None, None, None, None, None, None)
        };

        let e_flash = if let Some(exif_data) = &exif {
            exif_data.get_field(Tag::Flash, exif::In::PRIMARY)
                .and_then(|field| field.value.get_uint(0))
                .map(|val| if val & 1 == 1 { "Fired".to_string() } else { "Not fired".to_string() })
        } else {
            None
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
            duration: Some(duration),

            is_favorite: None,
            rotate: None,
            comments: None,
            has_tags: Some(false),

            e_make: Self::get_exif_field(&exif, Tag::Make).map(|s| s.to_uppercase()),
            e_model: Self::get_exif_field(&exif, Tag::Model),
            e_date_time: Self::get_exif_field(&exif, Tag::DateTimeOriginal),
            e_exposure_time: Self::get_exif_field(&exif, Tag::ExposureTime),
            e_f_number: Self::get_exif_field(&exif, Tag::FNumber),
            e_focal_length: Self::get_exif_field(&exif, Tag::FocalLength),
            e_iso_speed: Self::get_exif_field(&exif, Tag::PhotographicSensitivity),
            e_flash,
            e_orientation: Self::get_exif_orientation_field(&exif, Tag::Orientation),

            gps_latitude,
            gps_longitude,
            gps_altitude,
            gps_location,
            gps_country,
            gps_city,

            file_path: None,
            album_id: None,
            album_name: None,
        };

        Ok(file)
    }

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
                name, name_pinyin, size, file_type, created_at, modified_at, 
                taken_date,
                width, height, duration,
                is_favorite, rotate, comments, has_tags,
                e_make, e_model, e_date_time, e_exposure_time, e_f_number, e_focal_length, e_iso_speed, e_flash, e_orientation,
                gps_latitude, gps_longitude, gps_altitude, gps_location, gps_country, gps_city
            ) 
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24, ?25, ?26, ?27, ?28, ?29, ?30)",
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
                self.e_exposure_time,
                self.e_f_number,
                self.e_focal_length,
                self.e_iso_speed,
                self.e_flash,
                self.e_orientation,

                self.gps_latitude,
                self.gps_longitude,
                self.gps_altitude,
                self.gps_location,
                self.gps_country,
                self.gps_city,
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

    pub fn delete_file(file_id: i64) -> Result<usize, String> {
        let conn = open_conn()?;
    
        let sql = 
            "SELECT b.path, a.name 
            FROM afiles a 
            LEFT JOIN afolders b ON a.folder_id = b.id 
            WHERE a.id = ?1";
        let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;
        let file_info: Option<(String, String)> = stmt
            .query_row([file_id], |row| Ok((row.get(0)?, row.get(1)?)))
            .optional()
            .map_err(|e| e.to_string())?;
    
        if let Some((path, name)) = file_info {
            let file_path = format!("{}/{}", path, name);
            trash::delete(&file_path).map_err(|e| e.to_string())?;
    
            // delete database record
            let result = conn.execute("DELETE FROM afiles WHERE id = ?1", [file_id])
                .map_err(|e| e.to_string())?;
            return Ok(result);
        } else {
            return Err(format!("File with id {} not found", file_id));
        }
    }

    // Helper function to build the count SQL query
    fn build_count_query() -> String {
        let base_query = "SELECT COUNT(*), SUM(a.size)
            FROM afiles a 
            LEFT JOIN afolders b ON a.folder_id = b.id
            LEFT JOIN albums c ON b.album_id = c.id";
        
        format!("{}", base_query)
    }

    // build the base SQL query
    fn build_base_query() -> String {
        String::from(
            "SELECT a.id, a.folder_id, 
                a.name, a.name_pinyin, a.size, a.file_type, a.created_at, a.modified_at, 
                a.taken_date,
                a.width, a.height, a.duration,
                a.is_favorite, a.rotate, a.comments, a.has_tags,
                a.e_make, a.e_model, a.e_date_time, a.e_exposure_time, a.e_f_number, a.e_focal_length, a.e_iso_speed, a.e_flash, a.e_orientation,
                a.gps_latitude, a.gps_longitude, a.gps_altitude, a.gps_location, a.gps_country, a.gps_city,
                b.path,
                c.id AS album_id, c.name AS album_name
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
            e_exposure_time: row.get(19)?,
            e_f_number: row.get(20)?,
            e_focal_length: row.get(21)?,
            e_iso_speed: row.get(22)?,
            e_flash: row.get(23)?,
            e_orientation: row.get(24)?,

            gps_latitude: row.get(25)?,
            gps_longitude: row.get(26)?,
            gps_altitude: row.get(27)?,
            gps_location: row.get(28)?,
            gps_country: row.get(29)?,
            gps_city: row.get(30)?,

            file_path: Some(t_utils::get_file_path(
                row.get::<_, String>(31)?.as_str(),
                row.get::<_, String>(2)?.as_str(),
            )),
            album_id: row.get(32)?,
            album_name: row.get(33)?,
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
    
    /// Get a file's has_tags status
    pub fn get_has_tags(file_id: i64) -> Result<bool, String> {
        let conn = open_conn()?;
        let result = conn.query_row(
            "SELECT has_tags FROM afiles WHERE id = ?1",
            params![file_id],
            |row| row.get(0),
        ).map_err(|e| e.to_string())?;
        Ok(result)
    }

    /// get all taken dates from db
    pub fn get_taken_dates(ascending: bool, is_show_hidden: bool) -> Result<Vec<(String, i64)>, String> {
        let conn = open_conn()?;
        
        let hidden_clause = if is_show_hidden { "" } else { "AND (c.is_hidden IS NULL OR c.is_hidden = 0)" };
        let order_clause = if ascending { "ASC" } else { "DESC" };
        let query = format!(
            "SELECT a.taken_date, COUNT(1) 
            FROM afiles a
            LEFT JOIN afolders b ON a.folder_id = b.id
            LEFT JOIN albums c ON b.album_id = c.id
            WHERE a.taken_date IS NOT NULL {}
            GROUP BY a.taken_date
            ORDER BY a.taken_date {}",
            hidden_clause, order_clause
        );

        let mut stmt = conn
            .prepare(&query)
            .map_err(|e| format!("Failed to prepare query: {}", e))?;

        // Use collect() to simplify result collection
        let results: Vec<(String, i64)> = stmt
            .query_map(params![], |row| {
                Ok((row.get(0)?, row.get(1)?))
            })
            .map_err(|e| format!("Query execution failed: {}", e))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Failed to process rows: {}", e))?;

        Ok(results)
    }

    // get total count and size of files
    pub fn get_count_and_sum() -> Result<(i64, i64), String> {
        let sql = format!("{}", Self::build_count_query());
        Self::query_count_and_sum(&sql, &[])
    }

    /// get files
    pub fn get_files(
        search_text: &str, search_file_type: i64,
        sort_type: i64, sort_order: i64,
        search_folder: &str,
        start_date: &str, end_date: &str,
        make: &str, model: &str,
        is_favorite: bool, 
        is_show_hidden: bool,
        tag_id: i64,
        offset: i64, page_size: i64,
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
    
        let folder_exact_pattern = format!("{}/%", search_folder);
        if !search_folder.is_empty() {
            // Match path that starts with search_folder followed by '/' or end of string
            conditions.push("(b.path = ? OR b.path LIKE ?)");
            params.push(&search_folder);
            params.push(&folder_exact_pattern);
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
    
        if tag_id > 0 {
            conditions.push("a.id IN (SELECT file_id FROM afile_tags WHERE tag_id = ?)");
            params.push(&tag_id);
        }
    
        if !is_show_hidden {
            conditions.push("(c.is_hidden IS NULL OR c.is_hidden = 0)");
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
            2 => query.push_str(" ORDER BY a.width, a.height"),
            3 => query.push_str(" ORDER BY a.duration"),
            4 => query.push_str(" ORDER BY a.created_at"),
            5 => query.push_str(" ORDER BY a.modified_at"),
            6 => query.push_str(" ORDER BY a.taken_date"),
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
    pub error_code: i64,  // error code (0: success, 1: error)

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
            1 => { // image
                if let Some(ext) = t_utils::get_file_extension(file_path) {
                    match ext.to_lowercase().as_str() {
                        "heic" | "heif" => { // heic/heif
                            match t_utils::get_video_thumbnail(file_path, thumbnail_size) {
                                Ok(Some(data)) => (Some(data), 0),
                                Ok(None) => (None, 1), // empty thumb
                                Err(_) => (None, 1), // error
                            }
                        },
                        _ => { // other images
                            match t_utils::get_image_thumbnail(file_path, orientation, thumbnail_size) {
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
            2 => { // video
                match t_utils::get_video_thumbnail(file_path, thumbnail_size) {
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
                "INSERT INTO athumbs (file_id, error_code, thumb_data) 
                VALUES (?1, ?2, ?3)",
                params![self.file_id, self.error_code, self.thumb_data,],
            )
            .map_err(|e| e.to_string())?;
        Ok(result)
    }

    /// fetch a thumbnail from db by file_id
    fn fetch(file_id: i64) -> Result<Option<Self>, String> {
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

    /// insert a thumbnail into db if not exists
    pub fn add_to_db(
        file_id: i64,
        file_path: &str,
        file_type: i64,
        orientation: i32,
        thumbnail_size: u32,
    ) -> Result<Option<Self>, String> {
        // Check if the thumbnail exists
        if let Ok(Some(thumbnail)) = Self::fetch(file_id) {
            return Ok(Some(thumbnail));
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

    /// get the thumbnail count of the folder
    pub fn get_folder_thumb_count(search_text: &str, search_file_type: i64, folder_id: i64) -> Result<i64, String> {
        let conn = open_conn()?;

        let mut conditions = Vec::new();
        let mut params: Vec<&dyn rusqlite::ToSql> = Vec::new();

        conditions.push("a.folder_id = ?");
        params.push(&folder_id);

        let like_pattern = format!("%{}%", search_text);
        if !search_text.is_empty() {
            conditions.push("a.name LIKE ? COLLATE NOCASE");
            params.push(&like_pattern);
        }

        if search_file_type > 0 {
            conditions.push("a.file_type = ?");
            params.push(&search_file_type);
        }

        let mut query = "SELECT COUNT(b.id) FROM afiles a JOIN athumbs b ON a.id = b.file_id".to_string();
        if !conditions.is_empty() {
            query.push_str(" WHERE ");
            query.push_str(&conditions.join(" AND "));
        }

        let result = conn.query_row(
            &query,
            rusqlite::params_from_iter(params),
            |row| row.get(0)
        ).map_err(|e| e.to_string())?;

        Ok(result)
    }
}


/// Define the Tag struct
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ATag {
    pub id: i64,
    pub name: String,
}

impl ATag {
    /// Function to construct `Self` from a database row
    fn from_row(row: &rusqlite::Row) -> Result<Self, rusqlite::Error> {
        Ok(Self {
            id: row.get(0)?,
            name: row.get(1)?,
        })
    }

    /// Add a new tag. If the tag already exists, return the existing one.
    pub fn add(name: &str) -> Result<Self, String> {
        let conn = open_conn()?;
        // First, try to fetch the tag to see if it already exists.
        let existing_tag = conn.query_row(
            "SELECT id, name FROM atags WHERE name = ?1",
            params![name],
            Self::from_row
        ).optional().map_err(|e| e.to_string())?;

        if let Some(tag) = existing_tag {
            Ok(tag)
        } else {
            // The tag doesn't exist, so insert it.
            conn.execute(
                "INSERT INTO atags (name) VALUES (?1)",
                params![name],
            ).map_err(|e| e.to_string())?;
            let id = conn.last_insert_rowid();
            Ok(Self { id, name: name.to_string() })
        }
    }

    /// Get all tags from the db
    pub fn get_all() -> Result<Vec<Self>, String> {
        let conn = open_conn()?;
        let mut stmt = conn.prepare("SELECT id, name FROM atags ORDER BY name ASC")
            .map_err(|e| e.to_string())?;

        let tags_iter = stmt.query_map([], |row| Self::from_row(row))
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
        let result = conn.query_row("SELECT name FROM atags WHERE id = ?1", params![tag_id], |row| row.get(0))
            .map_err(|e| e.to_string())?;
        Ok(result)
    }

    /// Get all tags for a specific file
    pub fn get_tags_for_file(file_id: i64) -> Result<Vec<Self>, String> {
        let conn = open_conn()?;
        let mut stmt = conn.prepare(
            "SELECT t.id, t.name
             FROM atags t
             INNER JOIN afile_tags ft ON t.id = ft.tag_id
             WHERE ft.file_id = ?1
             ORDER BY t.name ASC"
        ).map_err(|e| e.to_string())?;

        let tags_iter = stmt.query_map(params![file_id], |row| Self::from_row(row))
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
        ).map_err(|e| e.to_string())?;

        // Update has_tags in afiles table
        conn.execute("UPDATE afiles SET has_tags = 1 WHERE id = ?1", params![file_id])
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Remove a tag from a file
    pub fn remove_tag_from_file(file_id: i64, tag_id: i64) -> Result<usize, String> {
        let conn = open_conn()?;
        let result = conn.execute(
            "DELETE FROM afile_tags WHERE file_id = ?1 AND tag_id = ?2",
            params![file_id, tag_id],
        ).map_err(|e| e.to_string())?;

        // Check if the file still has any tags
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM afile_tags WHERE file_id = ?1",
            params![file_id],
            |row| row.get(0),
        ).map_err(|e| e.to_string())?;

        if count == 0 {
            // If no tags left, set has_tags to false
            conn.execute("UPDATE afiles SET has_tags = 0 WHERE id = ?1", params![file_id])
                .map_err(|e| e.to_string())?;
        }
        Ok(result)
    }

    /// Delete a tag from the database. This will also remove all its associations with files.
    pub fn delete(tag_id: i64) -> Result<usize, String> {
        let conn = open_conn()?;
        let result = conn.execute(
            "DELETE FROM atags WHERE id = ?1",
            params![tag_id],
        ).map_err(|e| e.to_string())?;
        Ok(result)
    }

    /// Rename a tag
    pub fn rename(tag_id: i64, new_name: &str) -> Result<usize, String> {
        let conn = open_conn()?;
        let result = conn.execute(
            "UPDATE atags SET name = ?1 WHERE id = ?2",
            params![new_name, tag_id],
        ).map_err(|e| e.to_string())?;
        Ok(result)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ACamera {
    pub make: String,
    pub models: Vec<String>,
}

impl ACamera {
    // get all camera makes and models from db
    pub fn get_from_db(is_show_hidden: bool) -> Result<Vec<Self>, String> {
        let conn = open_conn()?;
        let hidden_clause = if is_show_hidden { "" } else { "AND (c.is_hidden IS NULL OR c.is_hidden = 0)" };

        let query = format!(
            "SELECT a.e_make, a.e_model 
            FROM afiles a
            LEFT JOIN afolders b ON a.folder_id = b.id
            LEFT JOIN albums c ON b.album_id = c.id
            WHERE a.e_make IS NOT NULL AND a.e_model IS NOT NULL {}
            GROUP BY a.e_make, a.e_model
            ORDER BY a.e_make, a.e_model",
            hidden_clause
        );

        let mut stmt = conn
            .prepare(query.as_str())
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
pub fn create_db() -> Result<(), String> {
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
            avatar_id INTEGER,
            description TEXT,
            is_hidden INTEGER
        )",
        [],
    ).map_err(|e| e.to_string())?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_albums_name ON albums(name)", []).map_err(|e| e.to_string())?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_albums_path ON albums(path)", []).map_err(|e| e.to_string())?;

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
    ).map_err(|e| e.to_string())?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_afolders_album_id ON afolders(album_id)", []).map_err(|e| e.to_string())?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_afolders_name ON afolders(name)", []).map_err(|e| e.to_string())?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_afolders_path ON afolders(path)", []).map_err(|e| e.to_string())?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_afolders_is_favorite ON afolders(is_favorite)", []).map_err(|e| e.to_string())?;

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
            duration INTEGER,
            is_favorite INTEGER,
            rotate INTEGER,
            comments TEXT,
            has_tags INTEGER,
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
            gps_location TEXT,
            gps_country TEXT,
            gps_city TEXT,
            FOREIGN KEY (folder_id) REFERENCES afolders(id) ON DELETE CASCADE
        )",
        [],
    ).map_err(|e| e.to_string())?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_afiles_folder_id ON afiles(folder_id)", []).map_err(|e| e.to_string())?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_afiles_name ON afiles(name)", []).map_err(|e| e.to_string())?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_afiles_name_pinyin ON afiles(name_pinyin)", []).map_err(|e| e.to_string())?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_afiles_file_type ON afiles(file_type)", []).map_err(|e| e.to_string())?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_afiles_created_at ON afiles(created_at)", []).map_err(|e| e.to_string())?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_afiles_modified_at ON afiles(modified_at)", []).map_err(|e| e.to_string())?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_afiles_taken_date ON afiles(taken_date)", []).map_err(|e| e.to_string())?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_afiles_is_favorite ON afiles(is_favorite)", []).map_err(|e| e.to_string())?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_afiles_has_tags ON afiles(has_tags)", []).map_err(|e| e.to_string())?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_afiles_make_model ON afiles(e_make, e_model)", []).map_err(|e| e.to_string())?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_afiles_gps_location ON afiles(gps_location)", []).map_err(|e| e.to_string())?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_afiles_gps_country ON afiles(gps_country)", []).map_err(|e| e.to_string())?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_afiles_gps_city ON afiles(gps_city)", []).map_err(|e| e.to_string())?;

    // file thumbnail table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS athumbs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            file_id INTEGER NOT NULL,
            error_code INTEGER NOT NULL,
            thumb_data BLOB,
            FOREIGN KEY (file_id) REFERENCES afiles(id) ON DELETE CASCADE
        )",
        [],
    ).map_err(|e| e.to_string())?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_athumbs_file_id ON athumbs(file_id)", []).map_err(|e| e.to_string())?;

    // tags table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS atags (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL UNIQUE
        )",
        [],
    ).map_err(|e| e.to_string())?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_atags_name ON atags(name)", []).map_err(|e| e.to_string())?;

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
    ).map_err(|e| e.to_string())?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_afile_tags_file_id ON afile_tags(file_id)", []).map_err(|e| e.to_string())?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_afile_tags_tag_id ON afile_tags(tag_id)", []).map_err(|e| e.to_string())?;

    Ok(())
}

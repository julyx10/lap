/**
 * The db.rs file is used to create a database and a table in it.
 * 
 */
use std::io::Cursor;
use rusqlite::{ params, Connection, Result, OptionalExtension };
use serde::{Serialize, Deserialize};
use exif::Tag;
use tauri::api::file;

use crate::t_utils;


/// Define the Album struct
#[derive(Debug, Serialize, Deserialize)]
pub struct Album {
    pub id:                 Option<i64>,    // unique id (autoincrement by db)
    pub name:               String,         // album name (default is folder name)
    pub path:               String,         // folder path
    pub description:        Option<String>, // album description
    pub avatar_id:          Option<i64>,    // album avatar id ( from files table)
    pub display_order_id:   Option<i64>,    // display order id
    pub created_at:         Option<u64>,    // folder create time
    pub modified_at:        Option<u64>,    // folder modified time
}


impl Album {

    /// create a new album
    fn new(path: &str, ) -> Result<Self, String> {
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

    /// fetch an album from db by path
    fn fetch(path: &str) -> Result<Option<Self>, String> {
        let conn = get_conn();
        let result = conn.query_row(
            "SELECT id, name, path, description, avatar_id, display_order_id, created_at, modified_at FROM albums WHERE path = ?1",
            params![path],
            |row| {
                Ok(Self {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    path: row.get(2)?,
                    description: row.get(3)?,
                    avatar_id: row.get(4)?,
                    display_order_id: row.get(5)?,
                    created_at: row.get(6)?,
                    modified_at: row.get(7)?
                })
            }
        ).optional().map_err(|e| e.to_string())?;
        Ok(result)
    }

    /// insert an album into db
    fn insert(&mut self) -> Result<usize, String> {
        let conn = get_conn();

        // Determine the next display order id
        self.display_order_id = conn.query_row(
            "SELECT COALESCE(MAX(display_order_id), 0) + 1 FROM albums",
            params![],
            |row| row.get(0)
        ).map_err(|e| e.to_string())?;

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
            return Err(format!("Album '{}' with the path '{}' already exists.", album.name, album.path));
        }

        // Insert the new album into the database
        Self::new(path)?.insert()?;

        // return the newly inserted album
        let new_album = Self::fetch(path);
        Ok(new_album.unwrap().unwrap())
    }

    /// delete an album from the db
    pub fn delete_from_db(id: i64) -> Result<usize, String> {
        let conn = get_conn();
        let result = conn.execute(
            "DELETE FROM albums WHERE id = ?1",
            params![id],
        ).map_err(|e| e.to_string())?;
        Ok(result)
    }

    /// Get all albums from the db
    pub fn get_all_albums() -> Result<Vec<Self>> {
        let conn = get_conn();
        let mut stmt = conn.prepare(
            "SELECT id, name, path, description, avatar_id, display_order_id, created_at, modified_at
            FROM albums ORDER BY display_order_id ASC"
        )?;
        
        // Execute the query and map the result to Album structs
        let albums_iter = stmt.query_map([], |row| {
            Ok(Self {
                id: row.get(0)?,
                name: row.get(1)?,
                path: row.get(2)?,
                description: row.get(3)?,
                avatar_id: row.get(4)?,
                display_order_id: row.get(5)?,
                created_at: row.get(6)?,
                modified_at: row.get(7)?
            })
        })?;
        
        // Collect the results into a Vec<Album>
        let mut albums = Vec::new();
        for album in albums_iter {
            albums.push(album?);
        }
        Ok(albums)
    }
    
}


/// Define the album's folder struct
#[derive(Debug, Serialize, Deserialize)]
pub struct AFolder {
    pub id:             Option<i64>,    // unique id (autoincrement by db)
    pub album_id:       i64,            // album id (from albums table)
    pub parent_id:      i64,            // parent folder id
    pub name:           String,         // folder name
    pub path:           String,         // folder path
    pub created_at:     Option<u64>,    // folder create time
    pub modified_at:    Option<u64>,    // folder modified time
}


impl AFolder {

    /// create a new folder struct
    fn new(album_id: i64, parent_id: i64, path: &str) -> Result<Self, String> {
        let file_info = t_utils::FileInfo::new(path)?;
        Ok(Self {
            id: None,
            album_id,
            parent_id,
            name: file_info.file_name,
            path: file_info.file_path,
            created_at: file_info.created,
            modified_at: file_info.modified,
        })
    }

    /// fetch a folder row from db by path
    fn fetch(path: &str) -> Result<Option<Self>, String> {
        let conn = get_conn();
        let result = conn.query_row(
            "SELECT id, album_id, parent_id, name, path, created_at, modified_at FROM afolders WHERE path = ?1",
            params![path],
            |row| {
                Ok(Self {
                    id: Some(row.get(0)?),
                    album_id: row.get(1)?,
                    parent_id: row.get(2)?,
                    name: row.get(3)?,
                    path: row.get(4)?,
                    created_at: row.get(5)?,
                    modified_at: row.get(6)?,
                })
            }
        ).optional().map_err(|e| e.to_string())?;
        Ok(result)
    }

    /// insert a folder into db
    fn insert(&self) -> Result<usize, String> {
        let conn = get_conn();
        let result = conn.execute(
            "INSERT INTO afolders (album_id, parent_id, name, path, created_at, modified_at) 
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                self.album_id,
                self.parent_id,
                self.name,
                self.path,
                self.created_at,
                self.modified_at
            ],
        ).map_err(|e| e.to_string())?;
        Ok(result)
    }

    /// insert the folder to db if not exists
    pub fn add_to_db(album_id: i64, parent_id: i64, path: &str) -> Result<Self, String> {
        // Check if the path already exists
        let existing_folder = Self::fetch(path);
        if let Ok(Some(folder)) = existing_folder {
            return Ok(folder);
        }

        // insert the new folder into the database
        Self::new(album_id, parent_id, path)?.insert()?;

        // return the newly inserted folder
        let new_folder = Self::fetch(path)?;
        Ok(new_folder.unwrap())
    }

    // /// delete a folder from db
    // pub fn delete_from_db(id: i64) -> Result<()> {
    //     let conn = get_conn()?;
    //     conn.execute(
    //         "DELETE FROM afolders WHERE id = ?1",
    //         params![id],
    //     )?;
    //     Ok(())
    // }
}


/// Define the album file struct
#[derive(Debug, Serialize, Deserialize)]
pub struct AFile {
    pub id:             Option<i64>,    // unique id (autoincrement by db)
    pub folder_id:      i64,            // folder id (from folders table)

    // file basic info
    pub name:           String,         // file name
    pub size:           u64,            // file size
    pub created_at:     Option<u64>,    // file create time
    pub modified_at:    Option<u64>,    // file modified time

    // exif info
    pub e_make:           Option<String>,   // camera make
    pub e_model:          Option<String>,   // camera model
    pub e_date_time:      Option<String>,   
    pub e_exposure_time:  Option<String>,    
    pub e_f_number:       Option<String>,
    pub e_iso_speed:      Option<String>,
    pub e_focal_length:   Option<String>,
}


impl AFile {

    /// create a new file struct
    fn new(folder_id: i64, path: &str) -> Result<Self, String> {
        let file_info = t_utils::FileInfo::new(path)?;

        // Attempt to open the file
        let file = std::fs::File::open(path).map_err(|e| format!("Error opening file: {}", e))?;
        // Create a buffered reader
        let mut bufreader = std::io::BufReader::new(&file);
        // Create an EXIF reader and attempt to read EXIF data
        let exifreader = exif::Reader::new();
        let exif = exifreader.read_from_container(&mut bufreader).ok();

        Ok(Self {
            id: None,
            folder_id,
            
            name: file_info.file_name,
            size: file_info.file_size,
            created_at:  file_info.created,
            modified_at: file_info.modified,

            e_make: Self::get_exif_field(&exif, Tag::Make),
            e_model: Self::get_exif_field(&exif, Tag::Model),
            e_date_time: Self::get_exif_field(&exif, Tag::DateTime),
            e_exposure_time: Self::get_exif_field(&exif, Tag::ExposureTime),
            e_f_number: Self::get_exif_field(&exif, Tag::FNumber),
            e_iso_speed: Self::get_exif_field(&exif, Tag::PhotographicSensitivity),
            e_focal_length: Self::get_exif_field(&exif, Tag::FocalLength),
        })
    }

    fn get_exif_field(exif: &Option<exif::Exif>, tag: exif::Tag) -> Option<String> {
        exif.as_ref().and_then(|exif_data| {
            exif_data.get_field(tag, exif::In::PRIMARY)
                .map(|field| format!("{}", field.display_value().with_unit(exif_data)).replace("\"", ""))
        })
    }
    
    /// fetch a file info from db by folder_id and file name
    fn fetch(folder_id: i64, path: &str) -> Result<Option<Self>, String> {
        let conn = get_conn();
        let result = conn.query_row(
            "SELECT id, folder_id, name, size, created_at, modified_at, 
            e_make, e_model, e_date_time, e_exposure_time, e_f_number, e_iso_speed, e_focal_length 
            FROM afiles WHERE folder_id = ?1 AND name = ?2",
            params![folder_id, t_utils::get_path_name(path)],
            |row| {
                Ok(Self {
                    id: Some(row.get(0)?),
                    folder_id: row.get(1)?,
                    name: row.get(2)?,
                    size: row.get(3)?,
                    created_at: row.get(4)?,
                    modified_at: row.get(5)?,
                    e_make: row.get(6)?,
                    e_model: row.get(7)?,
                    e_date_time: row.get(8)?,
                    e_exposure_time: row.get(9)?,
                    e_f_number: row.get(10)?,
                    e_iso_speed: row.get(11)?,
                    e_focal_length: row.get(12)?,
                })
            }
        ).optional().map_err(|e| e.to_string())?;
        Ok(result)
    }

    /// insert a file into db
    fn insert(&self) -> Result<usize, String> {
        let conn = get_conn();
        let result = conn.execute(
            "INSERT INTO afiles (folder_id, name, size, created_at, modified_at, 
            e_make, e_model, e_date_time, e_exposure_time, e_f_number, e_iso_speed, e_focal_length) 
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                self.folder_id,
                self.name,
                self.size,
                self.created_at,
                self.modified_at,
                self.e_make,
                self.e_model,
                self.e_date_time,
                self.e_exposure_time,
                self.e_f_number,
                self.e_iso_speed,
                self.e_focal_length,
            ],
        ).map_err(|e| e.to_string())?;
        Ok(result)
    }

    /// delete a file from db
    fn delete(folder_id: i64) -> Result<usize, String> {
        let conn = get_conn();
        let result = conn.execute(
            "DELETE FROM afiles WHERE folder_id = ?1",
            params![folder_id],
        ).map_err(|e| e.to_string())?;
        Ok(result)
    }

    /// insert a file into db if not exists
    pub fn add_to_db(folder_id: i64, path: &str) -> Result<Self, String> {
        // Check if the file exists
        let existing_file = Self::fetch(folder_id, path)?;
        if let Some(file) = existing_file {
            // check file modified time
            let file_info = t_utils::FileInfo::new(path)?;
            if file.modified_at != file_info.modified {
                // delete the old file
                Self::delete(folder_id)?;
            } else {
                return Ok(file);
            }
        }

        // insert the new file into the database
        Self::new(folder_id, path)?.insert()?;

        // return the newly inserted file
        let new_file = Self::fetch(folder_id, path)?;
        Ok(new_file.unwrap())
    }

    // /// Get all files from the db by folder_id
    // pub fn get_all_files(folder_id: i64) -> Result<Vec<Self>> {
    //     let conn = get_conn();
        
    //     // Prepare the SQL query to fetch all files by folder_id
    //     let mut stmt = conn.prepare(
    //         "SELECT id, folder_id, name, size, created_at, modified_at, 
    //         e_make, e_model, e_date_time, e_exposure_time, e_f_number, e_iso_speed, e_focal_length 
    //         FROM afiles WHERE folder_id = ?1"
    //     )?;
        
    //     // Execute the query and map the result to AFile structs
    //     let files_iter = stmt.query_map(params![folder_id], |row| {
    //         Ok(Self {
    //             id: row.get(0)?,
    //             folder_id: row.get(1)?,
    //             name: row.get(2)?,
    //             size: row.get(3)?,
    //             created_at: row.get(4)?,
    //             modified_at: row.get(5)?,
    //             e_make: row.get(6)?,
    //             e_model: row.get(7)?,
    //             e_date_time: row.get(8)?,
    //             e_exposure_time: row.get(9)?,
    //             e_f_number: row.get(10)?,
    //             e_iso_speed: row.get(11)?,
    //             e_focal_length: row.get(12)?,
    //         })
    //     })?;
        
    //     // Collect the results into a Vec<AFile>
    //     let mut files = Vec::new();
    //     for file in files_iter {
    //         files.push(file?);
    //     }
    //     Ok(files)
    // }

}


/// Define the album thumbnail struct
pub struct AThumb {
    pub id:             Option<i64>,     // unique id (autoincrement by db)
    pub file_id:        i64,             // file id (from files table)
    pub thumb_data:     Option<Vec<u8>>, // thumbnail data
}


impl AThumb {

    /// create a new thumbnail struct
    fn new(file_id: i64, path: &str) -> Result<Option<Self>, String> {
       match Self::get_thumbnail(path, 320) {
            Ok(data) => {
                Ok(Some(Self {
                    id: None,
                    file_id,
                    thumb_data: data,
                }))
            },
            Err(_) => Ok(None),
        }
    }

    /// fetch a thumbnail from db by file_id
    fn fetch(file_id: i64) -> Result<Option<Self>, String> {
        let conn = get_conn();
        let result = conn.query_row(
            "SELECT id, file_id, thumb_data FROM athumbs WHERE file_id = ?1",
            params![file_id],
            |row| {
                Ok(Self {
                    id: Some(row.get(0)?),
                    file_id: row.get(1)?,
                    thumb_data: row.get(2)?,
                })
            }
        ).optional().map_err(|e| e.to_string())?;
        Ok(result)
    }

    /// insert a thumbnail into db
    fn insert(&self) -> Result<usize, String> {
        let conn = get_conn();
        let result = conn.execute(
            "INSERT INTO athumbs (file_id, thumb_data) 
            VALUES (?1, ?2)",
            params![
                self.file_id,
                self.thumb_data,
            ],
        ).map_err(|e| e.to_string())?;
        Ok(result)
    }

    /// insert a thumbnail into db if not exists
    pub fn add_to_db(file_id: i64, path: &str) -> Result<Option<Self>, String> {
        // Check if the thumbnail exists
        let existing_thumbnail = Self::fetch(file_id);
        if let Ok(Some(thumbnail)) = existing_thumbnail {
            return Ok(Some(thumbnail));
        }

        // Insert the new thumbnail into the database
        let new_thumbnail = Self::new(file_id, path);
        if let Ok(Some(athumb)) = new_thumbnail {
            athumb.insert();
            Ok(Self::fetch(file_id)?)
        } else {
            Ok(None)
        }

        // match Self::new(file_id, path) {
        //     Ok(Some(data)) => {
        //         data.insert()?;

        //         // return the newly inserted thumbnail
        //         let new_thumbnail = Self::fetch(file_id)?;
        //         Ok(new_thumbnail)
        //     },
        //     Err(_) => Ok(None)
        // }
    }

    /// Resize an image to create a thumbnail and return it as a vector of bytes
    fn get_thumbnail(file_path: &str, thumbnail_size: u32) -> Result<Option<Vec<u8>>, String> {
        // Attempt to open the image file
        let img = match image::open(file_path) {
            Ok(image) => image,
            Err(_) => return Ok(None), // Return Ok(None) if the image fails to open
        };

        // Resize the image to a thumbnail
        let thumbnail = img.thumbnail(thumbnail_size, thumbnail_size);

        // Save the thumbnail to an in-memory buffer as a JPEG
        let mut buf = Vec::new();
        match thumbnail.write_to(&mut Cursor::new(&mut buf), image::ImageFormat::Jpeg) {
            Ok(()) => Ok(Some(buf)),  // Return Ok(Some(buf)) if writing is successful
            Err(_) => Ok(None)        // Return Ok(None) if writing fails
        }
    }
    
}


/// get connection to the db
fn get_conn() -> Connection {
    let conn = Connection::open("./main.db").map_err(|e| e.to_string()).unwrap();
    conn
}


/// create all tables if not exists
pub fn create_db() -> Result<String> {
    let conn = get_conn();

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

    // album folders table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS afolders (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            album_id INTEGER NOT NULL,
            parent_id INTEGER NOT NULL,
            name TEXT NOT NULL,
            path TEXT NOT NULL,
            created_at INTEGER,
            modified_at INTEGER,
            FOREIGN KEY (album_id) REFERENCES albums(id) ON DELETE CASCADE
        )",
        [],
    )?;

    // album files table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS afiles (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            folder_id INTEGER NOT NULL,
            name TEXT NOT NULL,
            size INTEGER NOT NULL,
            created_at INTEGER,
            modified_at INTEGER,
            e_make TEXT,
            e_model TEXT,
            e_date_time TEXT,
            e_exposure_time TEXT,
            e_f_number TEXT,
            e_iso_speed TEXT,
            e_focal_length TEXT,
            FOREIGN KEY (folder_id) REFERENCES afolders(id) ON DELETE CASCADE
        )",
        [],
    )?;

    // album thumbnail table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS athumbs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            file_id INTEGER NOT NULL,
            thumb_data BLOB,
            FOREIGN KEY (file_id) REFERENCES afiles(id) ON DELETE CASCADE
        )",
        [],
    )?;
    
    Ok("Database created successfully.".to_string())

}


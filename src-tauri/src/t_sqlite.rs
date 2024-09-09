/**
 * The db.rs file is used to create a database and a table in it.
 * 
 */
use std::path::Path;

use rusqlite::{ params, Connection, Result, OptionalExtension };
use serde::{Serialize, Deserialize};
use exif::{In, Reader, Tag};

use crate::t_utils::{self, FileInfo};


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

    /// fetch an album from db by path
    fn fetch(conn: &Connection, path: &str) -> Result<Option<Album>, String> {
        let album = conn.query_row(
            "SELECT id, name, path, description, avatar_id, display_order_id, created_at, modified_at FROM albums WHERE path = ?1",
            params![path],
            |row| {
                Ok(Album {
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
        Ok(album)
    }

    /// insert an album into db
    fn insert(&self, conn: &Connection) -> Result<usize, String> {
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
    pub fn add_to_db(&mut self) -> Result<Album, String> {
        let conn = get_conn().map_err(|e| e.to_string())?;
        
        // Check if the path already exists
        let existing_album = Album::fetch(&conn, self.path.as_str());
        if let Ok(Some(album)) = existing_album {
            return Err(format!("Album '{}' with the path '{}' already exists.", album.name, album.path));
        }
                    
        // Determine the next display order id
        self.display_order_id = conn.query_row(
            "SELECT COALESCE(MAX(display_order_id), 0) + 1 FROM albums",
            params![],
            |row| row.get(0)
        ).map_err(|e| e.to_string())?;

        // Insert the new album into the database
        Album::insert(&self, &conn)?;

        // return the newly inserted album
        let new_album = Album::fetch(&conn, self.path.as_str());
        Ok(new_album.unwrap().unwrap())
    }

    /// delete an album from the db
    pub fn delete_from_db(id: i64) -> Result<()> {
        let conn = get_conn()?;
        conn.execute(
            "DELETE FROM albums WHERE id = ?1",
            params![id],
        )?;
        Ok(())
    }

    /// Get all albums from the db
    pub fn get_all_albums() -> Result<Vec<Album>> {
        let conn = get_conn()?;
        
        // Prepare the SQL query to fetch all albums
        let mut stmt = conn.prepare(
            "SELECT id, name, path, description, avatar_id, display_order_id, created_at, modified_at
            FROM albums ORDER BY display_order_id ASC"
        )?;
        
        // Execute the query and map the result to Album structs
        let albums_iter = stmt.query_map([], |row| {
            Ok(Album {
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

    /// fetch a folder row from db by path
    fn fetch(conn: &Connection, path: &str) -> Result<Option<AFolder>, String> {
        let folder = conn.query_row(
            "SELECT id, album_id, parent_id, name, path, created_at, modified_at FROM afolders WHERE path = ?1",
            params![path],
            |row| {
                Ok(AFolder {
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
        Ok(folder)
    }

    /// insert a folder into db
    fn insert(&self, conn: &Connection) -> Result<usize, String> {
        // Insert the new folder into the db
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
    pub fn add_to_db(&self) -> Result<AFolder, String> {
        let conn = get_conn().map_err(|e| e.to_string())?;
        
        // Check if the path already exists
        let existing_folder = AFolder::fetch(&conn, self.path.as_str());
        if let Ok(Some(folder)) = existing_folder {
            return Ok(folder);
        }

        // insert the new folder into the database
        AFolder::insert(&self, &conn)?;

        // return the newly inserted folder
        let new_folder = AFolder::fetch(&conn, self.path.as_str());
        Ok(new_folder.unwrap().unwrap())
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

    /// create a new file info struct
    pub fn new(folder_id: i64, path: &str) -> Result<Self, String> {
        // file info
        let file_info = FileInfo::new(path);

        // Attempt to open the file
        let file = std::fs::File::open(path).map_err(|e| {
            format!("Error opening file: {}", e)
        })?;

        // Create a buffered reader
        let mut bufreader = std::io::BufReader::new(&file);

        // Create an EXIF reader and attempt to read EXIF data
        let exifreader = exif::Reader::new();
        let exif = match exifreader.read_from_container(&mut bufreader) {
            Ok(exif) => exif,
            Err(e) => {
                return Err(format!("Error reading EXIF data: {}", e)); // Return the error
            }
        };

        Ok(AFile {
            id: None,
            folder_id: folder_id,
            name: file_info.file_name,
            size: file_info.file_size,
            created_at:  file_info.created,
            modified_at: file_info.modified,

            e_make: exif.get_field(Tag::Make, In::PRIMARY)
                .map(|field| field.display_value().with_unit(&exif).to_string().replace("\"", "")),
            e_model: exif.get_field(Tag::Model, In::PRIMARY)
                .map(|field| field.display_value().with_unit(&exif).to_string().replace("\"", "")),
            e_date_time: exif.get_field(Tag::DateTime, In::PRIMARY)
                .map(|field| field.display_value().with_unit(&exif).to_string()),
            e_exposure_time: exif.get_field(Tag::ExposureTime, In::PRIMARY)
                .map(|field| field.display_value().with_unit(&exif).to_string()),
            e_f_number: exif.get_field(Tag::FNumber, In::PRIMARY)
                .map(|field| field.display_value().with_unit(&exif).to_string()),
            e_iso_speed: exif.get_field(Tag::PhotographicSensitivity, In::PRIMARY)
                .map(|field| field.display_value().with_unit(&exif).to_string()),
            e_focal_length: exif.get_field(Tag::FocalLength, In::PRIMARY)
                .map(|field| field.display_value().with_unit(&exif).to_string()),            
        })
    }

    /// fetch a file info from db by folder_id and file name
    fn fetch(conn: &Connection, folder_id: i64, file_name: &str) -> Result<Option<AFile>, String> {
        let file = conn.query_row(
            "SELECT id, folder_id, name, size, created_at, modified_at, 
            e_make, e_model, e_date_time, e_exposure_time, e_f_number, e_iso_speed, e_focal_length 
            FROM afiles WHERE folder_id = ?1 AND name = ?2",
            params![folder_id, file_name],
            |row| {
                Ok(AFile {
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
        Ok(file)
    }

    /// insert a file into db
    fn insert(&self, conn: &Connection) -> Result<usize, String> {
        // Insert the new file into the db
        let result = conn.execute(
            "INSERT INTO  afiles (folder_id, name, size, created_at, modified_at, 
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

    /// insert a file into db if not exists
    pub fn add_to_db(&self) -> Result<AFile, String> {
        let conn = get_conn().map_err(|e| e.to_string())?;
        
        // Check if the file exists
        let existing_file = AFile::fetch(&conn, self.folder_id, self.name.as_str());
        if let Ok(Some(file)) = existing_file {
            return Ok(file);
        }

        // insert the new file into the database
        AFile::insert(&self, &conn)?;

        // return the newly inserted file
        let new_file = AFile::fetch(&conn, self.folder_id, self.name.as_str());
        Ok(new_file.unwrap().unwrap())
    }

    /// Get all files from the db by folder_id
    pub fn get_all_files(folder_id: i64) -> Result<Vec<AFile>> {
        let conn = get_conn()?;
        
        // Prepare the SQL query to fetch all files by folder_id
        let mut stmt = conn.prepare(
            "SELECT id, folder_id, name, size, created_at, modified_at, 
            e_make, e_model, e_date_time, e_exposure_time, e_f_number, e_iso_speed, e_focal_length 
            FROM afiles WHERE folder_id = ?1"
        )?;
        
        // Execute the query and map the result to AFile structs
        let files_iter = stmt.query_map(params![folder_id], |row| {
            Ok(AFile {
                id: row.get(0)?,
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
        })?;
        
        // Collect the results into a Vec<AFile>
        let mut files = Vec::new();
        for file in files_iter {
            files.push(file?);
        }
        Ok(files)
    }

}


/// Define the album thumbnail struct
pub struct AThumb {
    pub id:             Option<i64>,    // unique id (autoincrement by db)
    pub file_id:        i64,            // file id (from files table)
    pub thumb_data:     Vec<u8>,        // thumbnail data
}


impl AThumb {

    /// fetch a thumbnail from db by file_id
    fn fetch(conn: &Connection, file_id: i64) -> Result<Option<AThumb>, String> {
        let thumbnail = conn.query_row(
            "SELECT id, file_id, thumb_data FROM athumbs WHERE file_id = ?1",
            params![file_id],
            |row| {
                Ok(AThumb {
                    id: Some(row.get(0)?),
                    file_id: row.get(1)?,
                    thumb_data: row.get(2)?,
                })
            }
        ).optional().map_err(|e| e.to_string())?;
        Ok(thumbnail)
    }

    /// insert a thumbnail into db if not exists
    pub fn add_to_db(&self) -> Result<AThumb, String> {
        let conn = get_conn().map_err(|e| e.to_string())?;
        
        // Check if the thumbnail exists
        let existing_thumbnail = AThumb::fetch(&conn, self.file_id);
        if let Ok(Some(thumbnail)) = existing_thumbnail {
            return Ok(thumbnail);
        }

        // Insert the new thumbnail into the database
        conn.execute(
            "INSERT INTO athumbs (file_id, thumb_data) 
            VALUES (?1, ?2)",
            params![
                self.file_id,
                self.thumb_data,
            ],
        ).map_err(|e| e.to_string())?;

        // return the newly inserted thumbnail
        let new_thumbnail = AThumb::fetch(&conn, self.file_id);
        Ok(new_thumbnail.unwrap().unwrap())
    }
    
}



/// get connection to the db
fn get_conn() -> Result<Connection> {
    let conn = Connection::open("./main.db")?;
    Ok(conn)
}


/// create all tables if not exists
pub fn create_db() -> Result<String> {
    let conn = get_conn().map_err(|e| e.to_string()).unwrap();

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
            thumb_data BLOB NOT NULL,
            FOREIGN KEY (file_id) REFERENCES afiles(id) ON DELETE CASCADE
        )",
        [],
    )?;
    
    Ok("Database created successfully.".to_string())
}


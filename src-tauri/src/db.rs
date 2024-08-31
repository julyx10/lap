/**
 * The db.rs file is used to create a database and a table in it.
 * 
 */

use rusqlite::{ params, Connection, Result, OptionalExtension };
use serde::{Serialize, Deserialize};

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

    /// add a new album
    pub fn add_album(&self) -> Result<Album, String> {
        let conn = get_conn().map_err(|e| e.to_string())?;

        // Check if the path already exists
        let album_exists: bool = conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM albums WHERE path = ?1)",
            params![self.path],
            |row| row.get(0)
        ).map_err(|e| e.to_string())?;

        if album_exists {
            return Err("The selected album already exists.".to_string());
        }

        // Determine the next display order id
        let next_display_order_id: i64 = conn.query_row(
            "SELECT COALESCE(MAX(display_order_id), 0) + 1 FROM albums",
            params![],
            |row| row.get(0)
        ).map_err(|e| e.to_string())?;

        // Insert the new album into the database
        conn.execute(
            "INSERT INTO albums (name, path, description, avatar_id, display_order_id, created_at, modified_at) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                self.name,
                self.path,
                self.description,
                self.avatar_id,
                next_display_order_id,
                self.created_at,
                self.modified_at
            ],
        ).map_err(|e| e.to_string())?;

        // Get the ID of the newly inserted album
        let last_id: i64 = conn.last_insert_rowid();

        // Fetch the newly inserted album from the database
        let mut stmt = conn.prepare(
            "SELECT id, name, path, description, avatar_id, display_order_id, created_at, modified_at FROM albums
            WHERE id = ?1"
        ).map_err(|e| e.to_string())?;
        
        let album = stmt.query_row(params![last_id], |row| {
            Ok(Album {
                id:  row.get(0)?,
                name: row.get(1)?,
                path: row.get(2)?,
                description: row.get(3)?,
                avatar_id: row.get(4)?,
                display_order_id: row.get(5)?,
                created_at: row.get(6)?,
                modified_at: row.get(7)?
            })
        }).map_err(|e| e.to_string())?;

        Ok(album)
    }

    /// delete an album
    pub fn delete_album(id: i64) -> Result<()> {
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


/// Define the Folder struct
#[derive(Debug, Serialize, Deserialize)]
pub struct Folder {
    pub id:             Option<i64>,    // unique id (autoincrement by db)
    pub album_id:       i64,            // album id (from albums table)
    pub parent_id:      i64,            // parent folder id
    pub name:           String,         // folder name
    pub path:           String,         // folder path
    pub created_at:     Option<u64>,    // folder create time
    pub modified_at:    Option<u64>,    // folder modified time
}


impl Folder {

    /// add a new folder
    pub fn add_folder(&self) -> Result<Folder, String> {
        let conn = get_conn().map_err(|e| e.to_string())?;
        
        // Check if the folder exists
        let existing_folder = conn.query_row(
            "SELECT id, album_id, parent_id, name, path, created_at, modified_at FROM folders WHERE path = ?1",
            params![self.path],
            |row| {
                Ok(Folder {
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

        // If folder exists, return it
        if let Some(folder) = existing_folder {
            return Ok(folder);
        }

        // Insert the new folder into the database
        conn.execute(
            "INSERT INTO folders (album_id, parent_id, name, path, created_at, modified_at) 
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

        // Get the ID of the newly inserted folder
        let last_id: i64 = conn.last_insert_rowid();

        // Fetch the newly inserted folder from the database
        let mut stmt = conn.prepare(
            "SELECT id, album_id, parent_id, name, path, created_at, modified_at FROM folders WHERE id = ?1"
        ).map_err(|e| e.to_string())?;

        let new_folder = stmt.query_row(params![last_id], |row| {
            Ok(Folder {
                id: Some(row.get(0)?),
                album_id: row.get(1)?,
                parent_id: row.get(2)?,
                name: row.get(3)?,
                path: row.get(4)?,
                created_at: row.get(5)?,
                modified_at: row.get(6)?,
            })
        }).map_err(|e| e.to_string())?;

        Ok(new_folder)
    }


    /// delete a folder
    pub fn delete_folder(id: i64) -> Result<()> {
        let conn = get_conn()?;
        conn.execute(
            "DELETE FROM folders WHERE id = ?1",
            params![id],
        )?;
        Ok(())
    }

    /// delete all folders in an album
    pub fn delete_all_folders(album_id: i64) -> Result<()> {
        let conn = get_conn()?;
        conn.execute(
            "DELETE FROM folders WHERE album_id = ?1",
            params![album_id],
        )?;
        Ok(())
    }

}


/// Define the file struct
#[allow(dead_code)]
pub struct File {
    pub id:             Option<i64>,    // unique id (autoincrement by db)
    pub folder_id:      i64,            // folder id (from folders table)
    pub exifdata_id:    i64,            // exif metadata (from exif_data table)
    pub name:           String,         // file name
    pub size:           i64,            // file size
    pub created_at:     Option<u64>,    // file create time
    pub modified_at:    Option<u64>,    // file modified time
}


/// Define the exif struct
pub struct ExifData {
    pub id:             Option<i64>,        // unique id (autoincrement by db)
    pub file_id:        i64,                // file id (from files table)
    pub make:           Option<String>,     // camera make
    pub model:          Option<String>,     // camera model
    pub date_time:      Option<String>,  
    pub exposure_time:  Option<String>,
    pub f_number:       Option<String>,
    pub iso_speed:      Option<String>,
    pub focal_length:   Option<String>,
}


impl ExifData {

    /// save a new exif data to the db
    pub fn save_to_db(&self) -> Result<()> {
        let conn = get_conn()?;
        conn.execute(
            "INSERT INTO exif_data (file_id, make, model, date_time, exposure_time, f_number, iso_speed, focal_length) 
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                self.file_id,
                self.make,
                self.model,
                self.date_time,
                self.exposure_time,
                self.f_number,
                self.iso_speed,
                self.focal_length,
            ],
        )?;
        Ok(())
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

    // folders table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS folders (
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

    // files table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS files (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            folder_id INTEGER NOT NULL,
            name TEXT NOT NULL,
            size INTEGER NOT NULL,
            created_at INTEGER,
            modified_at INTEGER,
            FOREIGN KEY (folder_id) REFERENCES folders(id) ON DELETE CASCADE
        )",
        [],
    )?;

    // exif_data table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS exif_data (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            file_id INTEGER NOT NULL,
            make TEXT,
            model TEXT,
            date_time TEXT,
            exposure_time TEXT,
            f_number TEXT,
            iso_speed TEXT,
            focal_length TEXT,
            FOREIGN KEY (file_id) REFERENCES files(id) ON DELETE CASCADE
        )",
        [],
    )?;

    Ok("Database created successfully.".to_string())
}


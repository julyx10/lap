/**
 * The db.rs file is used to create a database and a table in it.
 * 
 */

use rusqlite::{ params, Connection, Result };
use serde::{Serialize, Deserialize};


/// Define the Album struct
#[derive(Debug, Serialize, Deserialize)]
pub struct Album {
    pub id:             Option<i64>,    // unique id (autoincrement by db)
    pub order_id:       Option<i64>,    // display order id
    pub name:           String,         // folder name
    pub path:           String,         // folder path
    pub description:    Option<String>, // folder description
    pub created_at:     Option<i64>,    // auto update by db
    pub updated_at:     Option<i64>,    // auto update by db
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

        // Determine the next order_id
        let next_order_id: i64 = conn.query_row(
            "SELECT COALESCE(MAX(order_id), 0) + 1 FROM albums",
            params![],
            |row| row.get(0)
        ).map_err(|e| e.to_string())?;

        // Insert the new album into the database
        conn.execute(
            "INSERT INTO albums (order_id, name, path, description) 
             VALUES (?1, ?2, ?3, ?4)",
            params![
                next_order_id,
                self.name,
                self.path,
                self.description
            ],
        ).map_err(|e| e.to_string())?;

        // Get the ID of the newly inserted album
        let last_id: i64 = conn.last_insert_rowid();

        // Fetch the newly inserted album from the database
        let mut stmt = conn.prepare(
            "SELECT id, order_id, name, path, description, created_at, updated_at FROM albums
            WHERE id = ?1 
            ORDER BY order_id"
        ).map_err(|e| e.to_string())?;
        
        let album = stmt.query_row(params![last_id], |row| {
            Ok(Album {
                id: Some(row.get(0)?),
                order_id: row.get(1)?,
                name: row.get(2)?,
                path: row.get(3)?,
                description: row.get(4)?,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
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
            "SELECT id, order_id, name, path, description, created_at, updated_at FROM albums"
        )?;
        
        // Execute the query and map the result to Album structs
        let albums_iter = stmt.query_map([], |row| {
            Ok(Album {
                id: row.get(0)?,
                order_id: row.get(1)?,
                name: row.get(2)?,
                path: row.get(3)?,
                description: row.get(4)?,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
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
    pub album_id:       i64,            // album id from albums table
    pub parent_id:      i64,            // parent folder id
    pub name:           String,         // folder name
    pub path:           String,         // folder path
    pub created_at:     Option<i64>,    // auto update by db
    pub updated_at:     Option<i64>,    // auto update by db
}


impl Folder {

    /// add a new folder
    pub fn add_folder(&self) -> Result<Folder, String> {
        let conn = get_conn().map_err(|e| e.to_string())?;

        // Check if the path already exists
        let folder_exists: bool = conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM folders WHERE path = ?1)",
            params![self.path],
            |row| row.get(0)
        ).map_err(|e| e.to_string())?;

        if folder_exists {
            return Err("The selected folder already exists.".to_string());
        }

        // Insert the new folder into the database
        conn.execute(
            "INSERT INTO folders (album_id, parent_id, name, path) 
            VALUES (?1, ?2, ?3, ?4)",
            params![
                self.album_id,
                self.parent_id,
                self.name,
                self.path
            ],
        ).map_err(|e| e.to_string())?;

        // Get the ID of the newly inserted folder
        let last_id: i64 = conn.last_insert_rowid();

        // Fetch the newly inserted folder from the database
        let mut stmt = conn.prepare(
            "SELECT id, album_id, parent_id, name, path, created_at, updated_at FROM folders
            WHERE id = ?1"
        ).map_err(|e| e.to_string())?;

        let folder = stmt.query_row(params![last_id], |row| {
            Ok(Folder {
                id: Some(row.get(0)?),
                album_id: row.get(1)?,
                parent_id: row.get(2)?,
                name: row.get(3)?,
                path: row.get(4)?,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        }).map_err(|e| e.to_string())?;

        Ok(folder)
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


/// Define the Thumbnail struct
#[allow(dead_code)]
pub struct Thumbnail {
    id:             Option<i64>,    // unique id
    album_id:       i64,
    name:           String,     // file name
    path:           String,     // file path
    size:           i64,        // file size
    exifdata_id:    i64,        // exif metadata
    created_at:     i64,        // utc timestamp
    updated_at:     i64,        // utc timestamp
}


pub struct ExifData {
    pub id:             Option<i64>,        // unique id
    pub thumbnail_id:   Option<i64>,
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
            "INSERT INTO exif_data (thumbnail_id, make, model, date_time, exposure_time, f_number, iso_speed, focal_length) 
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                self.thumbnail_id,
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
            order_id INTEGER NOT NULL,
            name TEXT NOT NULL,
            path TEXT NOT NULL,
            description TEXT,
            created_at INTEGER DEFAULT (strftime('%s', 'now')),
            updated_at INTEGER DEFAULT (strftime('%s', 'now'))
        )",
        [],
    )?;

    // create a trigger to automatically update the updated_at column
    conn.execute(
        "CREATE TRIGGER IF NOT EXISTS update_album_updated_at
         AFTER UPDATE ON albums
         FOR EACH ROW
         BEGIN
             UPDATE albums
             SET updated_at = strftime('%s', 'now')
             WHERE id = OLD.id;
         END;",
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
            created_at INTEGER DEFAULT (strftime('%s', 'now')),
            updated_at INTEGER DEFAULT (strftime('%s', 'now')),
            FOREIGN KEY (album_id) REFERENCES albums(id)
        )",
        [],
    )?;

    conn.execute(
        "CREATE TRIGGER IF NOT EXISTS update_folder_updated_at
         AFTER UPDATE ON folders
         FOR EACH ROW
         BEGIN
             UPDATE folders
             SET updated_at = strftime('%s', 'now')
             WHERE id = OLD.id;
         END;",
        [],
    )?;

    
    // exif_data table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS exif_data (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            thumbnail_id INTEGER NOT NULL,
            make TEXT,
            model TEXT,
            date_time TEXT,
            exposure_time TEXT,
            f_number TEXT,
            iso_speed TEXT,
            focal_length TEXT
        )",
        [],
    )?;

    conn.execute(
        "CREATE TRIGGER IF NOT EXISTS update_exif_data_updated_at
         AFTER UPDATE ON exif_data
         FOR EACH ROW
         BEGIN
             UPDATE exif_data
             SET updated_at = strftime('%s', 'now')
             WHERE id = OLD.id;
         END;",
        [],
    )?;

    // thumbnails table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS thumbnails (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            album_id INTEGER NOT NULL,
            name TEXT NOT NULL,
            path TEXT NOT NULL,
            size INTEGER NOT NULL,
            exifdata_id INTEGER,
            created_at INTEGER DEFAULT (strftime('%s', 'now')),
            updated_at INTEGER DEFAULT (strftime('%s', 'now')),
            FOREIGN KEY (album_id) REFERENCES albums(id),
            FOREIGN KEY (exifdata_id) REFERENCES exif_data(id)
        )",
        [],
    )?;

    conn.execute(
        "CREATE TRIGGER IF NOT EXISTS update_thumbnail_updated_at
         AFTER UPDATE ON thumbnails
         FOR EACH ROW
         BEGIN
             UPDATE thumbnails
             SET updated_at = strftime('%s', 'now')
             WHERE id = OLD.id;
         END;",
        [],
    )?;

    Ok("Database created successfully.".to_string())
}


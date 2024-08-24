/**
 * The db.rs file is used to create a database and a table in it.
 * 
 */

use rusqlite::{ params, Connection, Result };
use serde::{Serialize, Deserialize};
// use chrono::Utc;


/// Define the Album struct
#[derive(Debug, Serialize, Deserialize)]
pub struct Album {
    // pub index:          i64,         // order in the list
    pub id:             Option<i64>,    // unique id
    pub name:           String,         // folder name
    pub path:           String,         // folder path
    pub description:    Option<String>, // folder description
    pub created_at:     i64,            // utc timestamp
    pub updated_at:     i64,            // utc timestamp
}

impl Album {

    /// add a new album
    pub fn add_album(&self) -> Result<()> {
        let conn = get_conn()?;
        conn.execute(
            "INSERT INTO albums (name, path, description, created_at, updated_at) 
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                self.name,
                self.path,
                self.description,
                self.created_at,
                self.updated_at,
            ],
        ).expect("error while instert the table.");
        Ok(())
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
        let mut stmt = conn.prepare("SELECT id, name, path, description, created_at, updated_at FROM albums")?;
        
        // Execute the query and map the result to Album structs
        let albums_iter = stmt.query_map([], |row| {
            Ok(Album {
                id: row.get(0)?,
                name: row.get(1)?,
                path: row.get(2)?,
                description: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
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
pub fn create_db() -> Result<()> {
    let conn = get_conn()?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS albums (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            path TEXT NOT NULL,
            description TEXT,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS thumbnails (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            album_id INTEGER NOT NULL,
            name TEXT NOT NULL,
            path TEXT NOT NULL,
            size INTEGER NOT NULL,
            exifdata_id INTEGER,
            created_at INTEGER,
            updated_at INTEGER,
            FOREIGN KEY (album_id) REFERENCES albums(id),
            FOREIGN KEY (exifdata_id) REFERENCES exif_data(id)
        )",
        [],
    )?;

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

    Ok(())
}


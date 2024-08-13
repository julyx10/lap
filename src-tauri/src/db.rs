/**
 * The db.rs file is used to create a database and a table in it.
 * 
 */

use rusqlite::{ params, Connection, Result };
use serde::{Serialize, Deserialize};


// Define the Album struct
#[derive(Debug, Serialize, Deserialize)]
pub struct Album {
    // pub index:          i32,         // order in the list
    pub name:           String,         // folder name
    pub description:    Option<String>,
    pub location:       String,         // folder location
    pub created_at:     i64,            // utc timestamp
    pub updated_at:     i64,            // utc timestamp
}

impl Album {
    /**
     * save a new album to the database
     */
    pub fn save_to_db(&self) -> Result<()> {
        let conn = get_conn()?;
        conn.execute(
            "INSERT INTO albums (name, description, location, created_at, updated_at) 
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                self.name,
                self.description,
                self.location,
                self.created_at,
                self.updated_at,
            ],
        ).expect("error while instert the table.");
        Ok(())
    }

    /**
     * get all albums from the database
     */
    pub fn get_all_albums() -> Result<Vec<Album>> {
        let conn = get_conn()?;
        
        // Prepare the SQL query to fetch all albums
        let mut stmt = conn.prepare("SELECT name, description, location, created_at, updated_at FROM albums")?;
        
        // Execute the query and map the result to Album structs
        let albums_iter = stmt.query_map([], |row| {
            Ok(Album {
                name: row.get(0)?,
                description: row.get(1)?,
                location: row.get(2)?,
                created_at: row.get(3)?,
                updated_at: row.get(4)?,
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

// Define the Thumbnail struct
pub struct Thumbnail {
    // id: i32,
    album_id:       i32,
    name:           String,     // file name
    size:           i32,        // file size
    location:       String,     // file location
    exifdata_id:    i32,        // exif metadata
    created_at:     i64,        // utc timestamp
    updated_at:     i64,        // utc timestamp
}

pub struct ExifData {
    // id: i32,
    pub thumbnail_id:   i32,
    pub make:           Option<String>,     // camera make
    pub model:          Option<String>,     // camera model
    pub date_time:      Option<String>,  
    pub exposure_time:  Option<String>,
    pub f_number:       Option<String>,
    pub iso_speed:      Option<String>,
    pub focal_length:   Option<String>,
}

impl ExifData {
    /**
     * save a new exif data to the database
     */
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


/**
 * get connection to the database
 */
fn get_conn() -> Result<Connection> {
    let conn = Connection::open("./main.db")?;
    Ok(conn)
}


/**
 * create all tables if not exists
 */
pub fn create_db() -> Result<()> {
    let conn = get_conn()?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS albums (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            description TEXT,
            location TEXT NOT NULL,
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
            size INTEGER NOT NULL,
            location TEXT NOT NULL,
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


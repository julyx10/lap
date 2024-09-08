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

    /// fetch a folder row from db by path
    fn fetch(conn: &Connection, path: &str) -> Result<Option<Folder>, String> {
        let folder = conn.query_row(
            "SELECT id, album_id, parent_id, name, path, created_at, modified_at FROM folders WHERE path = ?1",
            params![path],
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
        Ok(folder)
    }

    /// insert a folder into db
    fn insert(&self, conn: &Connection) -> Result<usize, String> {
        // Insert the new folder into the db
        let result = conn.execute(
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
        Ok(result)
    }

    /// insert the folder to db if not exists
    pub fn add_to_db(&self) -> Result<Folder, String> {
        let conn = get_conn().map_err(|e| e.to_string())?;
        
        // Check if the path already exists
        let existing_folder = Folder::fetch(&conn, self.path.as_str());
        if let Ok(Some(folder)) = existing_folder {
            return Ok(folder);
        }

        // insert the new folder into the database
        Folder::insert(&self, &conn)?;

        // return the newly inserted folder
        let new_folder = Folder::fetch(&conn, self.path.as_str());
        Ok(new_folder.unwrap().unwrap())
    }

    // /// delete a folder from db
    // pub fn delete_from_db(id: i64) -> Result<()> {
    //     let conn = get_conn()?;
    //     conn.execute(
    //         "DELETE FROM folders WHERE id = ?1",
    //         params![id],
    //     )?;
    //     Ok(())
    // }
}


/// Define the file struct
#[allow(dead_code)]
pub struct File {
    pub id:             Option<i64>,    // unique id (autoincrement by db)
    pub folder_id:      i64,            // folder id (from folders table)
    pub name:           String,         // file name
    pub size:           i64,            // file size
    pub created_at:     Option<u64>,    // file create time
    pub modified_at:    Option<u64>,    // file modified time
}


impl File {

    /// fetch a file from db by folder_id and name
    fn fetch(conn: &Connection, folder_id: i64, name: &str) -> Result<Option<File>, String> {
        let file = conn.query_row(
            "SELECT id, folder_id, name, size, created_at, modified_at FROM files WHERE folder_id = ?1 AND name = ?2",
            params![folder_id, name],
            |row| {
                Ok(File {
                    id: Some(row.get(0)?),
                    folder_id: row.get(1)?,
                    name: row.get(2)?,
                    size: row.get(3)?,
                    created_at: row.get(4)?,
                    modified_at: row.get(5)?,
                })
            }
        ).optional().map_err(|e| e.to_string())?;
        Ok(file)
    }

    /// insert a file into db
    fn insert(&self, conn: &Connection) -> Result<usize, String> {
        // Insert the new file into the db
        let result = conn.execute(
            "INSERT INTO files (folder_id, name, size, created_at, modified_at) 
            VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                self.folder_id,
                self.name,
                self.size,
                self.created_at,
                self.modified_at
            ],
        ).map_err(|e| e.to_string())?;
        Ok(result)
    }

    /// insert a file info into db if not exists
    pub fn add_to_db(&self) -> Result<File, String> {
        let conn = get_conn().map_err(|e| e.to_string())?;
        
        // Check if the file exists
        let existing_file = File::fetch(&conn, self.folder_id, self.name.as_str());
        if let Ok(Some(file)) = existing_file {
            return Ok(file);
        }

        // If file exists, return it
        if let Ok(Some(file)) = existing_file {
            return Ok(file);
        }

        // insert the new file into the database
        File::insert(&self, &conn)?;

        // return the newly inserted file
        let new_file = File::fetch(&conn, self.folder_id, self.name.as_str());
        Ok(new_file.unwrap().unwrap())
    }

}


pub struct ThumbNail {
    pub id:             Option<i64>,    // unique id (autoincrement by db)
    pub file_id:        i64,            // file id (from files table)
    pub thumb_data:     Vec<u8>,        // thumbnail data
}


impl ThumbNail {

    /// fetch a thumbnail from db by file_id
    fn fetch(conn: &Connection, file_id: i64) -> Result<Option<ThumbNail>, String> {
        let thumbnail = conn.query_row(
            "SELECT id, file_id, thumb_data FROM thumbnails WHERE file_id = ?1",
            params![file_id],
            |row| {
                Ok(ThumbNail {
                    id: Some(row.get(0)?),
                    file_id: row.get(1)?,
                    thumb_data: row.get(2)?,
                })
            }
        ).optional().map_err(|e| e.to_string())?;
        Ok(thumbnail)
    }

    /// insert a thumbnail into db if not exists
    pub fn add_to_db(&self) -> Result<ThumbNail, String> {
        let conn = get_conn().map_err(|e| e.to_string())?;
        
        // Check if the thumbnail exists
        let existing_thumbnail = ThumbNail::fetch(&conn, self.file_id);
        if let Ok(Some(thumbnail)) = existing_thumbnail {
            return Ok(thumbnail);
        }

        // Insert the new thumbnail into the database
        conn.execute(
            "INSERT INTO thumbnails (file_id, thumb_data) 
            VALUES (?1, ?2)",
            params![
                self.file_id,
                self.thumb_data,
            ],
        ).map_err(|e| e.to_string())?;

        // return the newly inserted thumbnail
        let new_thumbnail = ThumbNail::fetch(&conn, self.file_id);
        Ok(new_thumbnail.unwrap().unwrap())
    }
    
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

    /// insert exif data into db
    pub fn update_db(&self) -> Result<()> {
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

    // thumbnail table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS thumbnails (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            file_id INTEGER NOT NULL,
            thumb_data BLOB NOT NULL,
            FOREIGN KEY (file_id) REFERENCES files(id) ON DELETE CASCADE
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


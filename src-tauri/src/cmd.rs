/**
 * This file is used to define the commands that can be called from the JS side of the application.
 * Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
 */

use std::fs;
use std::io::BufReader;
use native_dialog::FileDialog;
use exif::{In, Reader, Tag};
use crate::db;
use crate::utils;
// use crate::utils::systemtime_to_u64;


/// get all albums
#[tauri::command]
pub fn get_albums() -> Result<Vec<db::Album>, String> {
    // Call the database function and handle errors
    match db::Album::get_all_albums() {
        Ok(albums) => Ok(albums),
        Err(e) => Err(format!("Error fetching albums: {}", e)),
    }
}


/// add an album
#[tauri::command]
pub fn add_album(window: tauri::Window, title: &str) -> Result<db::Album, String> {
    // Show open folder dialog
    let result = FileDialog::new()
        .set_title(title)
        .set_owner(&window)
        .show_open_single_dir();

    match result {
        Ok(Some(path)) => {
            let file_info = utils::FileInfo::new(path.to_str().unwrap());
            let mut album = db::Album {
                id: None,
                name: utils::get_path_name(path.to_str().unwrap()).to_string(),
                path: path.to_string_lossy().to_string(),
                description: None,
                avatar_id: None,
                display_order_id: None,
                created_at: file_info.created,
                modified_at: file_info.modified,
            };

            // Add the album to the database and return the result
            album.add_to_db()
                .map_err(|e| format!("Error while adding album to DB: {}", e))
        },
        Ok(None) => Err("No folder selected".to_string()),
        Err(_) => Err("Failed to open folder dialog".to_string()),
    }
}


/// delete an album
#[tauri::command]
pub fn delete_album(id: i64) -> Result<i64, String> {
    db::Album::delete_from_db(id).map_err(|e| {
        format!("Error while deleting album with id {}: {}", id, e.to_string())
    })?;

    // return the album id
    Ok(id)
}


// click a sub-folder under an album to add the folder to db
#[tauri::command]
pub fn add_folder(album_id: i64, parent_id: i64, name: &str, path: &str) -> Result<db::Folder, String> {
    let file_info = utils::FileInfo::new(path);

    let folder = db::Folder {
        id: None,
        album_id,
        parent_id,  
        name: name.to_string(),
        path: path.to_string(),
        created_at: file_info.created,
        modified_at: file_info.modified,
    };

    folder.add_to_db()
        .map_err(|e| format!("Error while adding folder to DB: {}", e))
}

/// expand folder from a path and build a FileNode
#[tauri::command]
pub fn expand_folder(path: &str) -> Result<utils::FileNode, String> {
    utils::FileNode::build_nodes(path)
}

/// read image files
#[tauri::command]
pub fn read_image_files(path: &str) -> Result<Vec<utils::FileInfo>, String> {
    utils::list_image_files(path)
}


/// open a picture file
#[tauri::command]
pub fn open_file() -> Option<String> {
    let result = FileDialog::new()
        .set_title("open a picture file")
        .add_filter("JPEG Image", &["jpg", "jpeg"])
        .add_filter("PNG Image", &["png"])
        .show_open_single_file();

    match result {
        Ok(Some(path)) => {
            let file = fs::File::open(&path).ok()?;

            // create an exif reader and read the exif data
            let mut bufreader = BufReader::new(file);
            let exif = Reader::new().read_from_container(&mut bufreader).ok()?;

            let exit_data = db::ExifData {
                id: None,
                file_id: 0,
                make: exif.get_field(Tag::Make, In::PRIMARY)
                    .map(|field| field.display_value().with_unit(&exif).to_string().replace("\"", "")),
                model: exif.get_field(Tag::Model, In::PRIMARY)
                    .map(|field| field.display_value().with_unit(&exif).to_string().replace("\"", "")),
                date_time: exif.get_field(Tag::DateTime, In::PRIMARY)
                    .map(|field| field.display_value().with_unit(&exif).to_string()),
                exposure_time: exif.get_field(Tag::ExposureTime, In::PRIMARY)
                    .map(|field| field.display_value().with_unit(&exif).to_string()),
                f_number: exif.get_field(Tag::FNumber, In::PRIMARY)
                    .map(|field| field.display_value().with_unit(&exif).to_string()),
                iso_speed: exif.get_field(Tag::PhotographicSensitivity, In::PRIMARY)
                    .map(|field| field.display_value().with_unit(&exif).to_string()),
                focal_length: exif.get_field(Tag::FocalLength, In::PRIMARY)
                    .map(|field| field.display_value().with_unit(&exif).to_string()),
            };

            exit_data.update_db().expect("error while saving to db");
            Some(path.to_str().unwrap().to_string())
        },
        Ok(None) => None,
        Err(_) => None,
    }
}
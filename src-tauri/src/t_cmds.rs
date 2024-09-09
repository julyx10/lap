/**
 * This file is used to define the commands that can be called from the JS side of the application.
 * Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
 */

use std::fs;
use std::io::BufReader;
use native_dialog::FileDialog;
use walkdir::{WalkDir, DirEntry}; // https://docs.rs/walkdir/2.5.0/walkdir/
use exif::{In, Reader, Tag};
use crate::t_sqlite::{ Album, AFolder, AFile };
use crate::t_utils;


/// get all albums
#[tauri::command]
pub fn get_albums() -> Result<Vec<Album>, String> {
    // Call the database function and handle errors
    match Album::get_all_albums() {
        Ok(albums) => Ok(albums),
        Err(e) => Err(format!("Error fetching albums: {}", e)),
    }
}


/// add an album
#[tauri::command]
pub fn add_album(window: tauri::Window, title: &str) -> Result<Album, String> {
    // Show open folder dialog
    let result = FileDialog::new()
        .set_title(title)
        .set_owner(&window)
        .show_open_single_dir();

    match result {
        Ok(Some(path)) => {
            let file_info = t_utils::FileInfo::new(path.to_str().unwrap());
            let mut album = Album {
                id: None,
                name: t_utils::get_path_name(path.to_str().unwrap()).to_string(),
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
    Album::delete_from_db(id).map_err(|e| {
        format!("Error while deleting album with id {}: {}", id, e.to_string())
    })?;

    // return the album id
    Ok(id)
}


// click a sub-folder under an album to add the folder to db
#[tauri::command]
pub fn add_folder(album_id: i64, parent_id: i64, name: &str, path: &str) -> Result<AFolder, String> {
    let file_info = t_utils::FileInfo::new(path);

    let folder = AFolder {
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


/// expand folder to recurse sub-folders and build a FileNode
#[tauri::command]
pub fn expand_folder(path: &str) -> Result<t_utils::FileNode, String> {
    t_utils::FileNode::build_nodes(path)
}


/// add image files
#[tauri::command]
pub fn add_files(folder_id: i64, path: &str) -> Result<Vec<AFile>, String> {
    let mut files: Vec<AFile> = Vec::new();

    // Use WalkDir to iterate over directory entries
    for entry in WalkDir::new(path)
        .min_depth(1)
        .max_depth(1)
        .into_iter().filter_map(|e| e.ok()) {
        
        let entry_path = entry.path();
        if entry_path.is_file() {
            if let Some(extension) = entry_path.extension().and_then(|ext| ext.to_str()) {
                if t_utils::is_image_extension(extension) {
                    let file = AFile::new(folder_id, entry_path.to_str().unwrap()).map_err(|e| format!("Error while creating file: {}", e))?;
                    file.add_to_db().map_err(|e| format!("Error while adding file to DB: {}", e))?;

                    files.push(file);
                }
            }
        }
    }

    Ok(files)
}


/// get files by folder id
#[tauri::command]
pub fn get_files(folder_id: i64) -> Result<Vec<AFile>, String> {
    AFile::get_all_files(folder_id)
        .map_err(|e| format!("Error while getting files: {}", e))
}
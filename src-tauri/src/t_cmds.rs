/**
 * This file is used to define the commands that can be called from the JS side of the application.
 * Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
 */

use native_dialog::FileDialog;
use walkdir::{WalkDir, DirEntry}; // https://docs.rs/walkdir/2.5.0/walkdir/
use base64::{Engine, engine::general_purpose};
use crate::t_sqlite::{ AFile, AFolder, AThumb, Album };
use crate::t_utils;


/// get all albums
#[tauri::command]
pub fn get_albums() -> Result<Vec<Album>, String> {
    Album::get_all_albums().map_err(|e| format!("Error while fetching albums: {}", e))
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
            // Add the album to the database and return the result
            Album::add_to_db(path.to_string_lossy().into_owned().as_str())
                .map_err(|e| format!("Error while adding album to DB: {}", e))
        },
        Ok(None) => Err("No folder selected".to_string()),
        Err(_) => Err("Failed to open folder dialog".to_string()),
    }
}


/// delete an album
#[tauri::command]
pub fn delete_album(id: i64) -> Result<usize, String> {
    let result = Album::delete_from_db(id).map_err(|e| {
        format!("Error while deleting album with id {}: {}", id, e.to_string())
    })?;

    Ok(result)
}


// click a sub-folder under an album
#[tauri::command]
pub fn select_folder(album_id: i64, parent_id: i64, path: &str) -> Result<AFolder, String> {
    AFolder::add_to_db(album_id, parent_id, path)
        .map_err(|e| format!("Error while adding folder to DB: {}", e))
}


/// expand folder to recurse sub-folders and build a FileNode
#[tauri::command]
pub fn expand_folder(path: &str, is_recursive: bool) -> Result<t_utils::FileNode, String> {
    t_utils::FileNode::build_nodes(path, is_recursive)
}


/// list image files
#[tauri::command]
pub fn get_files(folder_id: i64, path: &str) -> Result<Vec<AFile>, String> {
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
                    let file_path = entry_path.to_str().unwrap();

                    // Create a new AFile instance and add it to the database
                    let file = AFile::add_to_db(folder_id, file_path).map_err(|e| format!("Error while adding file to DB: {}", e))?;

                    files.push(file);
                }
            }
        }
    }

    Ok(files)
}


/// get a file's thumb image 
#[tauri::command]
pub async fn get_file_thumb(file_id: i64, path: &str) -> Result<String, String> {
    // Add the thumb to the database and return the thumb data
    match AThumb::add_to_db(file_id, path) {
        Ok(thumb) => {
            match thumb {
                Some(t) => {
                    match(t.thumb_data) {
                        Some(data) => {
                            let base64_thumbnail = general_purpose::STANDARD.encode(data);
                            Ok(base64_thumbnail)
                        },
                        None => Err("No thumbnail data found".to_string())
                    }
                },
                None => {
                    Err("No thumbnail data found".to_string())
                }
            }
        },
        Err(e) => {
            Err(format!("Error while adding thumb to DB: {}", e))
        }
    }
}


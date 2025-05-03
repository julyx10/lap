/**
 * project: jc-photo
 * author:  julyxx
 * email:   tiangle@gmail.com
 * GitHub:  /julyx10
 * date:    2024-08-08
 */
use base64::{engine::general_purpose, Engine};
use crate::t_sqlite::{ACamera, AFile, AFolder, AThumb, Album};
use crate::t_utils;

/// get all albums
#[tauri::command]
pub fn get_all_albums() -> Result<Vec<Album>, String> {
    Album::get_all_albums()
        .map_err(|e| format!("Error while getting all albums: {}", e))
}

/// get one album
#[tauri::command]
pub fn get_album(album_id: i64) -> Result<Album, String> {
    Album::get_album_by_id(album_id)
        .map_err(|e| format!("Error while getting one album: {}", e))
}

/// add an album
#[tauri::command]
pub fn add_album(folder_path: &str) -> Result<Album, String> {
    Album::add_to_db(folder_path)
        .map_err(|e| format!("Error while adding an album to DB: {}", e))
}

/// edit an album
#[tauri::command]
pub fn edit_album(id: i64, name: &str, description: &str) -> Result<usize, String> {
    let _ = Album::update_column(id, "name", &name)
        .map_err(|e| format!("Error while editing album with id {}: {}", id, e));

    Album::update_column(id, "description", &description)
        .map_err(|e| format!("Error while editing album with id {}: {}", id, e))
}

/// rename an album
// #[tauri::command]
// pub fn rename_album(id: i64, name: &str) -> Result<usize, String> {
//     Album::update_column(id, "name", &name)
//         .map_err(|e| format!("Error while edit album with id {}: {}", id, e))
// }

/// remove an album
#[tauri::command]
pub fn remove_album(id: i64) -> Result<usize, String> {
    Album::delete_from_db(id)
        .map_err(|e| format!("Error while removing album with id {}: {}", id, e))
}

/// set album display order
#[tauri::command]
pub fn set_album_display_order(id: i64, display_order: i32) -> Result<usize, String> {
    Album::update_column(id, "display_order_id", &display_order)
        .map_err(|e| format!("Error while setting album display order: {}", e))
}

/// get all favorite folders
#[tauri::command]
pub fn get_favorite_folders() -> Result<Vec<AFolder>, String> {
    AFolder::get_favorite_folders()
        .map_err(|e| format!("Error while getting favorite folders: {}", e))
}

// click to select a sub-folder under an album
#[tauri::command]
pub fn select_folder(album_id: i64, parent_id: i64, folder_path: &str) -> Result<AFolder, String> {
    AFolder::add_to_db(album_id, parent_id, folder_path)
        .map_err(|e| format!("Error while adding folder to DB: {}", e))
}

/// fetch folder and build a FileNode
#[tauri::command]
pub fn fetch_folder(path: &str, is_recursive: bool) -> Result<t_utils::FileNode, String> {
    t_utils::FileNode::build_nodes(path, is_recursive)
}

/// count all files in a folder
#[tauri::command]
pub fn count_folder(path: &str) -> (u64, u64, u64, u64, u64) {
    t_utils::count_folder_files(path) 
}

/// create a new folder
#[tauri::command]
pub fn create_folder(path: &str, folder_name: &str) -> Option<String> {
    let folder_path = t_utils::get_file_path(path, folder_name);
    t_utils::create_new_folder(&folder_path)
}

/// rename a folder
#[tauri::command]
pub fn rename_folder(folder_path: &str, new_folder_name: &str) -> Option<String> {
    let new_folder_path = t_utils::rename_folder(folder_path, new_folder_name);

    match new_folder_path {
        Some(new_path) => {
            if let Err(e) = AFolder::rename_folder(folder_path, &new_path) {
                eprintln!("Error while renaming folder: {}", e);
                return None;
            }
            Some(new_path)
        }
        None => None,
    }
}

/// move a folder
#[tauri::command]
pub fn move_folder(folder_path: &str, new_album_id: i64, new_folder_path: &str) -> Option<String> {
    // Move the folder in the file system
    let result = t_utils::move_folder(folder_path, new_folder_path);

    match result {
        Some(new_path) => {
            // Update the folder path in the database
            let _ = AFolder::move_folder(folder_path, new_album_id, &new_path)
                .map_err(|e| format!("Error while moving folder in DB: {}", e));
            Some(new_path)
        }
        None => None
    } 
}

/// copy a folder
#[tauri::command]
pub fn copy_folder(folder_path: &str, new_folder_path: &str) -> Option<String> {
    t_utils::copy_folder(folder_path, new_folder_path)
}

/// delete a folder
/// return the number of files and folders deleted
#[tauri::command]
pub fn delete_folder(folder_path: &str) -> Result<usize, String> {
    // Delete the folder from the file system
    if t_utils::delete_folder(folder_path) {
        // delete the folder from db
        AFolder::delete_folder(folder_path)
            .map_err(|e| format!("Failed to delete folder from database: {}", e))
    } else {
        Err("Failed to delete folder from file system".to_string())
    }
}

/// get a folder's favorite
#[tauri::command]
pub fn get_folder_favorite(folder_path: &str) -> Result<bool, String> {
    let is_favorite_opt = AFolder::get_is_favorite(folder_path)
        .map_err(|e| format!("Error while getting folder favorite: {}", e))?;

    match is_favorite_opt {
        Some(val) => Ok(val),
        None => Ok(false), // Default to false if not found
    }
}

/// set a folder's favorite
#[tauri::command]
pub fn set_folder_favorite(folder_id: i64, is_favorite: bool) -> Result<usize, String> {
    AFolder::update_column(folder_id, "is_favorite", &is_favorite)
        .map_err(|e| format!("Error while setting folder favorite: {}", e))
}

/// reveal a folder in the file explorer( or finder)
#[tauri::command]
pub fn reveal_folder(folder_path: &str) -> Result<(), String> {
    opener::open(folder_path).map_err(|e| e.to_string())
}

/// get all files from the folder
#[tauri::command]
pub fn get_folder_files(folder_id: i64, path: &str) -> Vec<AFile> {
    t_utils::get_folder_files(folder_id, path)
}

/// rename a file
#[tauri::command]
pub fn rename_file(file_id: i64, file_path: &str, new_name: &str) -> Option<String> {
    match t_utils::rename_file(file_path, new_name) {
        Some(new_file_path) => {
            match AFile::update_column(file_id, "name", &new_name) {
                Ok(_) => {
                    Some(new_file_path)
                },
                Err(e) => {
                    eprintln!("Error while renaming file in DB: {}", e);
                    None
                }
            }
        },
        None => None
    }
}

/// move a file to dest folder
#[tauri::command]
pub fn move_file(file_id: i64, file_path: &str, new_folder_id: i64, new_folder_path: &str) -> Option<String> {
    let moved_file = t_utils::move_file(file_path, new_folder_path);
    match moved_file {
        Some(new_path) => {
            // update the file's folder_id in the database
            let _ = AFile::update_column(file_id, "folder_id", &new_folder_id)
                .map_err(|e| format!("Error while moving file in DB: {}", e));
            Some(new_path)
        }
        None => None,
    }
}

/// copy a file to dest folder
#[tauri::command]
pub fn copy_file(file_path: &str, new_folder_path: &str) -> Option<String> {
    t_utils::copy_file(file_path, new_folder_path)
}

/// delete files
#[tauri::command]
pub fn delete_file(file_id: i64, file_path: &str) -> Option<String> {
    let deleted_file = t_utils::delete_file(file_path);
    match deleted_file {
        Some(new_path) => {
            // delete the file from the database
            let _ = AFile::delete(file_id)
                .map_err(|e| format!("Error while deleting file in DB: {}", e));
            Some(new_path)
        },
        None => None
    }
}

/// get a file's thumb image
#[tauri::command]
pub async fn get_file_thumb(file_id: i64, file_path: &str, orientation: i32, thumbnail_size: u32) -> Result<Option<AThumb>, String> {
    AThumb::add_to_db(file_id, file_path, orientation, thumbnail_size)
        .map_err(|e| format!("Error while adding thumbnail to DB: {}", e))
}

/// get a file's info
#[tauri::command]
pub fn get_file_info(file_id: i64) -> Result<Option<AFile>, String> {
    AFile::get_file_info(file_id)
        .map_err(|e| format!("Error while getting file info: {}", e))
}

/// get a file's image
#[tauri::command]
pub async fn get_file_image(file_path: String) -> Result<String, String> {
    match tokio::fs::read(file_path).await {
        Ok(image_data) => Ok(general_purpose::STANDARD.encode(image_data)),
        Err(e) => Err(format!("Failed to read the image: {}", e)),
    }
}

/// set a file's favorite status
#[tauri::command]
pub fn set_file_favorite(file_id: i64, is_favorite: bool) -> Result<usize, String> {
    AFile::update_column(file_id, "is_favorite", &is_favorite)
        .map_err(|e| format!("Error while setting file favorite: {}", e))
}

/// set a file's rotate status
#[tauri::command]
pub fn set_file_rotate(file_id: i64, rotate: i32) -> Result<usize, String> {
    AFile::update_column(file_id, "rotate", &rotate)
        .map_err(|e| format!("Error while setting file rotate: {}", e))
}

/// set a file's delete status: write the deleted_at timestamp
/// if deleted_at is 0 or null, it means the file is not deleted
#[tauri::command]
pub fn set_file_delete(file_id: i64, deleted_at: u64) -> Result<usize, String> {
    AFile::update_column(file_id, "deleted_at", &deleted_at)
        .map_err(|e| format!("Error while setting file delete: {}", e))
}

/// get camera's taken dates
#[tauri::command]
pub fn get_taken_dates() -> Result<Vec<(String, i64)>, String> {
    AFile::get_taken_dates()
        .map_err(|e| format!("Error while getting taken dates: {}", e))
}

/// get all files
#[tauri::command]
pub fn get_all_files(is_favorite: bool, offset: i64, page_size: i64) -> Result<Vec<AFile>, String> {
    AFile::get_all_files(is_favorite, offset, page_size)
        .map_err(|e| format!("Error while getting all files: {}", e))
}

/// get files by date
#[tauri::command]
pub fn get_files_by_date(date: &str) -> Result<Vec<AFile>, String> {
    AFile::get_files_by_date(date)
        .map_err(|e| format!("Error while getting files by date: {}", e))
}

/// get files by date range
/// start_date and end_date format: yyyy-mm-dd
#[tauri::command]
pub fn get_files_by_date_range(start_date: &str, end_date: &str) -> Result<Vec<AFile>, String> {
    AFile::get_files_by_date_range(start_date, end_date)
        .map_err(|e| format!("Error while getting files by date range: {}", e))
}

/// get a file's camera make and model info
#[tauri::command]
pub fn get_camera_info() -> Result<Vec<ACamera>, String> {
    ACamera::get_from_db()
        .map_err(|e| format!("Error while getting camera info: {}", e))
}

/// get files from db by camera make and model
#[tauri::command]
pub fn get_camera_files(make: &str, model: &str) -> Result<Vec<AFile>, String> {
    AFile::get_files_by_camera(make, model)
        .map_err(|e| format!("Error while getting camera files: {}", e))
}

/// print an image: uses platform-specific commands to print an image.
#[tauri::command]
pub fn print_image(image_path: String) -> Result<(), String> {
    t_utils::print_image(image_path)
        .map_err(|e| format!("Error while printing image: {}", e))
}
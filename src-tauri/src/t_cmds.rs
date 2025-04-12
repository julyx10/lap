/**
 * project: jc-photo
 * author:  julyxx
 * email:   tiangle@gmail.com
 * GitHub:  /julyx10
 * date:    2024-08-08
 */
use base64::{engine::general_purpose, Engine};
use walkdir::WalkDir; // https://docs.rs/walkdir/2.5.0/walkdir/
use crate::t_sqlite::{ACamera, AFile, AFolder, AThumb, Album};
use crate::t_utils;
use std::process::Command;

/// get all albums
#[tauri::command]
pub fn get_all_albums() -> Result<Vec<Album>, String> {
    Album::get_all_albums().map_err(|e| format!("Error while fetching albums: {}", e))
}

/// get one album
#[tauri::command]
pub fn get_album(album_id: i64) -> Result<Album, String> {
    Album::get_album_by_id(album_id).map_err(|e| format!("Error while fetching album info: {}", e))
}

/// add an album
#[tauri::command]
pub fn add_album(folder_path: &str) -> Result<Album, String> {
    Album::add_to_db(folder_path)
        .map_err(|e| format!("Error while adding album to DB: {}", e))
}
// pub fn add_album(_window: tauri::Window, title: &str) -> Result<Album, String> {
//     // Show open folder dialog
//     let result = FileDialog::new().set_title(title).show_open_single_dir();

//     match result {
//         Ok(Some(path)) => {
//             // Add the album to the database and return the result
//             Album::add_to_db(path.to_string_lossy().into_owned().as_str())
//                 .map_err(|e| format!("Error while adding album to DB: {}", e))
//         }
//         Ok(None) => Err("No folder selected".to_string()),
//         Err(_) => Err("Failed to open folder dialog".to_string()),
//     }
// }

/// rename an album
#[tauri::command]
pub fn rename_album(id: i64, name: &str) -> Result<usize, String> {
    Album::update_column(id, "name", &name).map_err(|e| {
        format!(
            "Error while renaming album with id {}: {}",
            id,
            e.to_string()
        )
    })
}

/// remove an album
#[tauri::command]
pub fn remove_album(id: i64) -> Result<usize, String> {
    Album::delete_from_db(id).map_err(|e| {
        format!(
            "Error while removing album with id {}: {}",
            id,
            e.to_string()
        )
    })
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
        .map_err(|e| format!("Error while fetching favorite folders: {}", e))
}

/// create a new folder
#[tauri::command]
pub fn create_folder(path: &str, folder_name: &str) -> Option<String> {
    let folder_path = t_utils::get_file_path(path, folder_name);
    return t_utils::create_new_folder(&folder_path);
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

// get all parent folders of a folder
// #[tauri::command]
// pub fn get_folder_parents(folder_id: i64) -> Result<Vec<i64>, String> {
//     AFolder::recurse_all_parents_id(folder_id)
//         .map_err(|e| format!("Error while recursing all parent folders from DB: {}", e))
// }

// click to select a sub-folder under an album
#[tauri::command]
pub fn select_folder(album_id: i64, parent_id: i64, folder_path: &str) -> Result<AFolder, String> {
    AFolder::add_to_db(album_id, parent_id, folder_path)
        .map_err(|e| format!("Error while adding folder to DB: {}", e))
}

/// expand folder to recurse sub-folders and build a FileNode
#[tauri::command]
pub fn expand_folder(path: &str, is_recursive: bool) -> Result<t_utils::FileNode, String> {
    t_utils::FileNode::build_nodes(path, is_recursive)
}

/// get a folder's favorite
#[tauri::command]
pub fn get_folder_favorite(folder_path: &str) -> Result<bool, String> {
    let is_favorite_opt = AFolder::get_is_favorite(folder_path)
        .map_err(|e| format!("Error while fetching folder favorite: {}", e))?;

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

/// get all files from the folder
#[tauri::command]
pub fn get_folder_files(folder_id: i64, path: &str) -> Result<Vec<AFile>, String> {
    let mut files: Vec<AFile> = Vec::new();

    // Use WalkDir to iterate over directory entries
    for entry in WalkDir::new(path)
        .min_depth(1)
        .max_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let entry_path = entry.path();
        if entry_path.is_file() {
            if let Some(extension) = entry_path.extension().and_then(|ext| ext.to_str()) {
                if t_utils::is_image_extension(extension) {
                    let file_path = entry_path.to_str().unwrap();

                    // Create a new AFile instance and add it to the database
                    let file = AFile::add_to_db(folder_id, file_path)
                        .map_err(|e| format!("Error while adding file to DB: {}", e))?;

                    files.push(file);
                }
            }
        }
    }

    Ok(files)
}

/// get a file's thumb image
#[tauri::command]
pub async fn get_file_thumb(
    file_id: i64,
    file_path: &str,
    orientation: i32,
    thumbnail_size: u32,
) -> Result<Option<AThumb>, String> {
    AThumb::add_to_db(file_id, file_path, orientation, thumbnail_size)
        .map_err(|e| format!("Error while adding thumbnail to DB: {}", e))
}

/// get a file's info
#[tauri::command]
pub fn get_file_info(file_id: i64) -> Result<Option<AFile>, String> {
    AFile::get_file_info(file_id).map_err(|e| format!("Error while fetching file info: {}", e))
}

/// get a file's image
#[tauri::command]
// pub fn get_file_image(file_path: &str) -> Result<String, String> {
//     match std::fs::read(file_path) {
//         Ok(image_data) => Ok(general_purpose::STANDARD.encode(image_data)),
//         Err(e) => Err(format!("Failed to read the image: {}", e)),
//     }
// }
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
    AFile::get_taken_dates().map_err(|e| format!("Error while fetching taken dates: {}", e))
}

/// get all files
#[tauri::command]
pub fn get_all_files(is_favorite: bool, offset: i64, page_size: i64) -> Result<Vec<AFile>, String> {
    AFile::get_all_files(is_favorite, offset, page_size)
        .map_err(|e| format!("Error while fetching all files: {}", e))
}

/// get files by date
#[tauri::command]
pub fn get_files_by_date(date: &str) -> Result<Vec<AFile>, String> {
    AFile::get_files_by_date(date).map_err(|e| format!("Error while fetching files by date: {}", e))
}

/// get files by date range
/// start_date and end_date format: yyyy-mm-dd
#[tauri::command]
pub fn get_files_by_date_range(start_date: &str, end_date: &str) -> Result<Vec<AFile>, String> {
    AFile::get_files_by_date_range(start_date, end_date)
        .map_err(|e| format!("Error while fetching files by date range: {}", e))
}

/// get a file's camera make and model info
#[tauri::command]
pub fn get_camera_info() -> Result<Vec<ACamera>, String> {
    ACamera::get_from_db().map_err(|e| format!("Error while fetching camera info: {}", e))
}

/// get files from db by camera make and model
#[tauri::command]
pub fn get_camera_files(make: &str, model: &str) -> Result<Vec<AFile>, String> {
    AFile::get_files_by_camera(make, model)
        .map_err(|e| format!("Error while fetching camera files: {}", e))
}

/// print an image: uses platform-specific commands to print an image.
#[tauri::command]
pub fn print_image(image_path: String) -> Result<(), String> {
    // Platform-specific printing logic
    let output = if cfg!(target_os = "windows") {
        Command::new("mspaint")
            .arg("/p")
            .arg(image_path)
            .output()
            .map_err(|e| e.to_string())?
    } else if cfg!(target_os = "macos") {
        Command::new("lp")
            .arg(image_path)
            .output()
            .map_err(|e| e.to_string())?
    } else {
        return Err("Unsupported OS".to_string());
    };

    if !output.status.success() {
        return Err(format!("Failed to print image: {}", String::from_utf8_lossy(&output.stderr)));
    }

    Ok(())
}
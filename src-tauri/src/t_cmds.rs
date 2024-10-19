/**
 * project: jc-photo
 * author:  julyxx
 * email:   tiangle@gmail.com
 * GitHub:  /julyx10
 * date:    2024-08-08
 */

use native_dialog::FileDialog;
use walkdir::WalkDir; // https://docs.rs/walkdir/2.5.0/walkdir/
use base64::{ Engine, engine::general_purpose };
use crate::t_sqlite::{ ACamera, AFile, AFolder, AThumb, Album };
use crate::t_utils;


/// save the app's configuration
#[tauri::command]
pub fn save_config(config: t_utils::AppConfig) -> Result<(), String> {
    // Get the app's data directory
    let app_dir = std::env::current_exe().map_err(|e| format!("Failed to get current exe path: {}", e))?;
    let config_path = app_dir.join("config.json");

    // Serialize the config to JSON and write it to the file
    let config_json = serde_json::to_string(&config).map_err(|e| e.to_string())?;

    std::fs::write(config_path, config_json).map_err(|e| e.to_string())?;

    Ok(())
}


/// load the app's configuration
#[tauri::command]
pub fn load_config() -> Result<t_utils::AppConfig, String> {
    // Get the app's data directory
    let app_dir = std::env::current_exe().map_err(|e| format!("Failed to get current exe path: {}", e))?;
    let config_path = app_dir.join("config.json");

    // If the config file doesn't exist, return the default config
    if !config_path.exists() {
        return Ok(t_utils::AppConfig::default());
    }

    // Read the config file and deserialize the JSON
    let config_json = std::fs::read_to_string(config_path).map_err(|e| e.to_string())?;
    let config: t_utils::AppConfig = serde_json::from_str(&config_json).map_err(|e| e.to_string())?;

    Ok(config)
}


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
pub async fn get_file_thumb(file_id: i64, file_path: &str, orientation: i32, thumbnail_size: u32) -> Result<Option<AThumb>, String> {
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
pub fn get_file_image(file_path: &str) -> Result<String, String> {
  match std::fs::read(file_path) {
    Ok(image_data) => Ok(general_purpose::STANDARD.encode(image_data)),
    Err(e) => Err(format!("Failed to read the image: {}", e)),
  }
}


/// set a file's favorite status
#[tauri::command]
pub fn set_file_favorite(file_id: i64, is_favorite: bool) -> Result<usize, String> {
    AFile::set_favorite(file_id, is_favorite).map_err(|e| format!("Error while setting file favorite: {}", e))
}


/// get camera's taken dates
#[tauri::command]
pub fn get_taken_dates() -> Result<Vec<(String, i64)>, String> {
    AFile::get_taken_dates().map_err(|e| format!("Error while fetching taken dates: {}", e))
}

/// get files by date
#[tauri::command]
pub fn get_files_by_date(year: i64, month: i64, date: i64) -> Result<Vec<AFile>, String> {
    AFile::get_files_by_date(year, month, date).map_err(|e| format!("Error while fetching files by date: {}", e))
}

/// get files by date range
/// start_date and end_date format: yyyy-mm-dd
#[tauri::command]
pub fn get_files_by_date_range(start_date: &str, end_date: &str ) -> Result<Vec<AFile>, String> {
    AFile::get_files_by_date_range(start_date, end_date).map_err(|e| format!("Error while fetching files by date range: {}", e))
}

/// get a file's camera make and model info
#[tauri::command]
pub fn get_camera_info() -> Result<Vec<ACamera>, String> {
    ACamera::get_from_db().map_err(|e| format!("Error while fetching camera info: {}", e))
}


/// get files from db by camera make and model
#[tauri::command]
pub fn get_camera_files(make: &str, model: &str) -> Result<Vec<AFile>, String> {
    AFile::get_files_by_camera(make, model).map_err(|e| format!("Error while fetching camera files: {}", e))
}


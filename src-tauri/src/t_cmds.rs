/**
 * Tauri commands for frontend-backend communication.
 * project: Lap
 * author:  julyx10
 * date:    2024-08-08
 */
use crate::t_config::{self, AppConfig, Library, LibraryInfo, LibraryState};
use crate::t_face;
use crate::t_image;
use crate::t_sqlite::{
    ACamera, AFile, AFolder, ALens, ALocation, ATag, AThumb, ATimeLine, Album, ImageSearchParams,
    Person, QueryParams,
};
use crate::t_utils;
use crate::{t_ai, t_sqlite};

use serde::Serialize;
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};
use tauri::State;

// cancellation token for indexing
pub struct IndexCancellation(pub Arc<Mutex<HashMap<i64, bool>>>);

// active indexing workers
pub struct IndexActivity(pub Arc<Mutex<HashMap<i64, usize>>>);

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IndexingActivity {
    pub running: bool,
    pub album_id: Option<i64>,
}

// build info
include!(concat!(env!("OUT_DIR"), "/build_info.rs"));

// library

/// get app config (libraries list and current library)
#[tauri::command]
pub fn get_app_config() -> Result<AppConfig, String> {
    t_config::load_app_config()
}

#[tauri::command]
pub fn add_library(name: &str) -> Result<Library, String> {
    ensure_library_storage_mutation_allowed()?;
    t_config::add_library(name)
}

/// hide a library
#[tauri::command]
pub fn hide_library(id: &str, hidden: bool) -> Result<(), String> {
    ensure_library_storage_mutation_allowed()?;
    t_config::hide_library(id, hidden)
}

/// reorder libraries
#[tauri::command]
pub fn reorder_libraries(ids: Vec<String>) -> Result<(), String> {
    ensure_library_storage_mutation_allowed()?;
    t_config::reorder_libraries(ids)
}

/// edit library name
#[tauri::command]
pub fn edit_library(id: &str, name: &str) -> Result<(), String> {
    ensure_library_storage_mutation_allowed()?;
    t_config::edit_library(id, name)
}

/// remove a library (also deletes the database file)
#[tauri::command]
pub fn remove_library(id: &str) -> Result<(), String> {
    ensure_library_storage_mutation_allowed()?;
    t_config::remove_library(id)
}

/// switch to a different library
#[tauri::command]
pub fn switch_library(app_handle: tauri::AppHandle, id: &str) -> Result<(), String> {
    ensure_library_storage_mutation_allowed()?;
    t_sqlite::create_db_for_library(id)?;
    t_config::switch_library(id)?;
    t_utils::restore_album_scopes(&app_handle)
}

/// move a library database to a different storage directory
#[tauri::command]
pub async fn move_library_storage(
    app_handle: tauri::AppHandle,
    index_activity_state: State<'_, IndexActivity>,
    dedup_state: State<'_, crate::t_dedup::DedupState>,
    face_status_state: State<'_, t_face::FaceIndexingStatus>,
    id: String,
    storage_dir: Option<String>,
) -> Result<LibraryInfo, String> {
    ensure_library_storage_move_idle(&index_activity_state, &dedup_state, &face_status_state)?;
    tauri::async_runtime::spawn_blocking(move || {
        t_config::move_library_storage(&app_handle, &id, storage_dir)
    })
    .await
    .map_err(|e| format!("Failed to join library storage move task: {}", e))?
}

fn ensure_library_storage_move_idle(
    index_activity_state: &State<'_, IndexActivity>,
    dedup_state: &State<'_, crate::t_dedup::DedupState>,
    face_status_state: &State<'_, t_face::FaceIndexingStatus>,
) -> Result<(), String> {
    let indexing_running = !index_activity_state.0.lock().unwrap().is_empty();
    let dedup_running = dedup_state.is_scanning.load(Ordering::SeqCst);
    let face_running = *face_status_state.0.lock().unwrap();

    if indexing_running || dedup_running || face_running {
        return Err(
            "Cannot move library storage while indexing, dedup, or face indexing is still running."
                .to_string(),
        );
    }

    Ok(())
}

fn ensure_library_storage_mutation_allowed() -> Result<(), String> {
    t_config::ensure_library_storage_write_allowed()
}

fn ensure_library_storage_mutation_allowed_or_none() -> bool {
    match ensure_library_storage_mutation_allowed() {
        Ok(_) => true,
        Err(error) => {
            eprintln!("{}", error);
            false
        }
    }
}

#[tauri::command]
pub fn cancel_library_storage_move(id: &str) -> Result<(), String> {
    t_config::cancel_library_storage_move(id)
}

/// get library statistics
#[tauri::command]
pub async fn get_library_info(id: String) -> Result<LibraryInfo, String> {
    tauri::async_runtime::spawn_blocking(move || t_config::get_library_info(&id))
        .await
        .map_err(|e| format!("Failed to join library info task: {}", e))?
}

/// save library state
#[tauri::command]
pub fn save_library_state(id: &str, state: LibraryState) -> Result<(), String> {
    ensure_library_storage_mutation_allowed()?;
    t_config::save_library_state(id, state)
}

/// get library state
#[tauri::command]
pub fn get_library_state(id: &str) -> Result<LibraryState, String> {
    t_config::get_library_state(id)
}

/// get current library state
#[tauri::command]
pub fn get_current_library_state() -> Result<LibraryState, String> {
    t_config::get_current_library_state()
}

// album

/// get all albums
#[tauri::command]
pub fn get_all_albums() -> Result<Vec<Album>, String> {
    Album::get_all_albums().map_err(|e| format!("Error while getting all albums: {}", e))
}

/// get one album
#[tauri::command]
pub fn get_album(album_id: i64) -> Result<Album, String> {
    Album::get_album_by_id(album_id).map_err(|e| format!("Error while getting one album: {}", e))
}

/// recount files for an album and return updated album
#[tauri::command]
pub fn recount_album(album_id: i64) -> Result<Album, String> {
    ensure_library_storage_mutation_allowed()?;
    Album::recount_album(album_id).map_err(|e| format!("Error while recounting album: {}", e))
}

/// add an album
#[tauri::command]
pub fn add_album(app_handle: tauri::AppHandle, folder_path: &str) -> Result<Album, String> {
    ensure_library_storage_mutation_allowed()?;
    t_utils::authorize_directory_scope(&app_handle, folder_path).map_err(|e| {
        format!(
            "Error while authorizing album folder '{}': {}",
            folder_path, e
        )
    })?;

    Album::add_album_to_db(folder_path)
        .map_err(|e| format!("Error while adding an album to DB: {}", e))
}

/// edit an album
#[tauri::command]
pub fn edit_album(id: i64, name: &str, description: &str) -> Result<usize, String> {
    ensure_library_storage_mutation_allowed()?;
    let _ = Album::update_column(id, "name", &name)
        .map_err(|e| format!("Error while editing album with id {}: {}", id, e));

    Album::update_column(id, "description", &description)
        .map_err(|e| format!("Error while editing album with id {}: {}", id, e))
}

/// remove an album
#[tauri::command]
pub fn remove_album(id: i64) -> Result<usize, String> {
    ensure_library_storage_mutation_allowed()?;
    Album::delete_from_db(id)
        .map_err(|e| format!("Error while removing album with id {}: {}", id, e))
}

/// set album display order
#[tauri::command]
pub fn set_album_display_order(id: i64, display_order: i32) -> Result<usize, String> {
    ensure_library_storage_mutation_allowed()?;
    Album::update_column(id, "display_order_id", &display_order)
        .map_err(|e| format!("Error while setting album display order: {}", e))
}

/// set album cover
#[tauri::command]
pub fn set_album_cover(id: i64, file_id: i64) -> Result<usize, String> {
    ensure_library_storage_mutation_allowed()?;
    Album::update_column(id, "cover_file_id", &file_id)
        .map_err(|e| format!("Error while setting album cover: {}", e))
}

/// index album
#[tauri::command]
pub fn index_album(
    app_handle: tauri::AppHandle,
    state: State<IndexCancellation>,
    activity_state: State<IndexActivity>,
    album_id: i64,
    thumbnail_size: u32,
    skip_file_path: Option<String>,
) -> Result<(), String> {
    ensure_library_storage_mutation_allowed()?;
    // Reset cancellation flag
    state.0.lock().unwrap().insert(album_id, false);
    let cancellation_token = state.0.clone();
    let cancellation_cleanup_token = state.0.clone();
    {
        let mut activity = activity_state.0.lock().unwrap();
        *activity.entry(album_id).or_insert(0) += 1;
    }
    let activity_token = activity_state.0.clone();

    tauri::async_runtime::spawn(async move {
        if let Err(e) = t_utils::index_album_worker(
            &app_handle,
            cancellation_token,
            album_id,
            thumbnail_size,
            skip_file_path,
        )
        .await
        {
            eprintln!("Error indexing album {}: {}", album_id, e);
        }
        cancellation_cleanup_token.lock().unwrap().remove(&album_id);
        let mut activity = activity_token.lock().unwrap();
        if let Some(count) = activity.get_mut(&album_id) {
            if *count <= 1 {
                activity.remove(&album_id);
            } else {
                *count -= 1;
            }
        }
    });
    Ok(())
}

/// cancel indexing
#[tauri::command]
pub fn cancel_indexing(state: State<IndexCancellation>, album_id: i64) -> Result<(), String> {
    state.0.lock().unwrap().insert(album_id, true);
    Ok(())
}

#[tauri::command]
pub fn get_indexing_activity(state: State<IndexActivity>) -> Result<IndexingActivity, String> {
    let activity = state.0.lock().unwrap();
    let album_id = activity.keys().next().copied();
    Ok(IndexingActivity {
        running: !activity.is_empty(),
        album_id,
    })
}

#[tauri::command]
pub fn get_index_recovery_info() -> Option<crate::t_utils::IndexRecoveryInfo> {
    crate::t_utils::read_index_trace()
}

#[tauri::command]
pub fn clear_index_recovery_info() -> Result<(), String> {
    t_utils::clear_index_trace();
    Ok(())
}

// folder

// click to select a sub-folder under an album
#[tauri::command]
pub fn select_folder(
    app_handle: tauri::AppHandle,
    album_id: i64,
    folder_path: &str,
) -> Result<AFolder, String> {
    ensure_library_storage_mutation_allowed()?;
    t_utils::authorize_directory_scope(&app_handle, folder_path)
        .map_err(|e| format!("Error while authorizing folder '{}': {}", folder_path, e))?;

    AFolder::add_to_db(album_id, folder_path)
        .map_err(|e| format!("Error while adding folder to DB: {}", e))
}

/// fetch folder and build a FileNode
#[tauri::command]
pub fn fetch_folder(path: &str, is_recursive: bool) -> Result<t_utils::FileNode, String> {
    t_utils::FileNode::build_nodes(path, is_recursive)
}

/// count all files in a folder (include all sub-folders)
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
    if !ensure_library_storage_mutation_allowed_or_none() {
        return None;
    }
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
    if !ensure_library_storage_mutation_allowed_or_none() {
        return None;
    }
    // Move the folder in the file system
    let result = t_utils::move_folder(folder_path, new_folder_path);

    match result {
        Some(new_path) => {
            // Update the folder path in the database
            let _ = AFolder::move_folder(folder_path, new_album_id, &new_path)
                .map_err(|e| format!("Error while moving folder in DB: {}", e));
            Some(new_path)
        }
        None => None,
    }
}

/// copy a folder
#[tauri::command]
pub fn copy_folder(folder_path: &str, new_folder_path: &str) -> Option<String> {
    t_utils::copy_folder(folder_path, new_folder_path)
}

/// delete a folder
#[tauri::command]
pub fn delete_folder(folder_path: &str) -> Result<usize, String> {
    ensure_library_storage_mutation_allowed()?;
    // trash the folder
    trash::delete(folder_path).map_err(|e| e.to_string())?;

    // delete the folder and all children from db
    AFolder::delete_folder(folder_path)
        .map_err(|e| format!("Error while deleting folder from DB: {}", e))
}

/// reveal a file or folder in the file explorer (or finder)
#[tauri::command]
pub fn reveal_path(path: &str) -> Result<(), String> {
    t_utils::reveal_path(path)
}

/// open an external URL or app-specific deep link
#[tauri::command]
pub fn open_external_url(url: &str) -> Result<(), String> {
    opener::open(url).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_external_app_display_name(app_path: &str) -> Result<String, String> {
    t_utils::get_external_app_display_name(app_path)
}

/// open a file with a specific external application
#[tauri::command]
pub fn open_file_with_app(file_path: &str, app_path: &str) -> Result<(), String> {
    if file_path.is_empty() || app_path.is_empty() {
        return Err("Missing file path or app path".to_string());
    }

    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg("-a")
            .arg(app_path)
            .arg(file_path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }

    #[cfg(not(target_os = "macos"))]
    {
        Command::new(app_path)
            .arg(file_path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

// file

/// get total file count and sum
#[tauri::command]
pub fn get_total_count_and_sum() -> Result<(i64, i64), String> {
    AFile::get_total_count_and_sum()
        .map_err(|e| format!("Error while getting all files count: {}", e))
}

/// get query count and sum
#[tauri::command]
pub fn get_query_count_and_sum(params: QueryParams) -> Result<(i64, i64), String> {
    AFile::get_query_count_and_sum(&params)
        .map_err(|e| format!("Error while getting query files count: {}", e))
}

/// get query time line
#[tauri::command]
pub fn get_query_time_line(params: QueryParams) -> Result<Vec<ATimeLine>, String> {
    AFile::get_query_time_line(&params)
        .map_err(|e| format!("Error while getting query timeline: {}", e))
}

/// get query file
#[tauri::command]
pub fn get_query_files(params: QueryParams, offset: i64, limit: i64) -> Result<Vec<AFile>, String> {
    AFile::get_query_files(&params, offset, limit)
        .map_err(|e| format!("Error while getting query files: {}", e))
}

#[tauri::command]
pub fn get_query_file_position(params: QueryParams, file_id: i64) -> Result<Option<i64>, String> {
    AFile::get_query_file_position(&params, file_id)
        .map_err(|e| format!("Error while getting query file position: {}", e))
}

/// get all files from the folder
#[tauri::command]
pub fn get_folder_files(
    file_type: i64,
    sort_type: i64,
    sort_order: i64,
    folder_id: i64,
    folder_path: &str,
    from_db_only: Option<bool>,
) -> (Vec<AFile>, u32, u32) {
    t_utils::get_folder_files(
        file_type,
        sort_type,
        sort_order,
        folder_id,
        folder_path,
        from_db_only.unwrap_or(false),
    )
}

/// get the thumbnail count of the folder
#[tauri::command]
pub fn get_folder_thumb_count(file_type: i64, folder_id: i64) -> i64 {
    AThumb::get_folder_thumb_count(file_type, folder_id).unwrap_or_default()
}

/// edit an image
#[tauri::command]
pub async fn edit_image(params: t_image::EditParams) -> Result<bool, String> {
    tokio::task::spawn_blocking(move || Ok(t_image::edit_image(params)))
        .await
        .map_err(|e| format!("Task error: {}", e))?
}

/// copy an edited image to clipboard
#[tauri::command]
pub async fn copy_edited_image(params: t_image::EditParams) -> Result<bool, String> {
    tokio::task::spawn_blocking(move || Ok(t_image::copy_edited_image_to_clipboard(params)))
        .await
        .map_err(|e| format!("Task error: {}", e))?
}

/// copy image to clipboard
#[tauri::command]
pub async fn copy_image(file_path: String) -> Result<bool, String> {
    tokio::task::spawn_blocking(move || {
        if let Ok(img) = image::open(Path::new(&file_path)) {
            Ok(t_image::copy_image_to_clipboard(img))
        } else {
            Err(format!("Failed to open image: {}", file_path))
        }
    })
    .await
    .map_err(|e| format!("Task error: {}", e))?
}

/// rename a file
#[tauri::command]
pub fn rename_file(file_id: i64, file_path: &str, new_name: &str) -> Option<String> {
    if !ensure_library_storage_mutation_allowed_or_none() {
        return None;
    }
    match t_utils::rename_file(file_path, new_name) {
        Some(new_file_path) => {
            let name_pinyin = t_utils::convert_to_pinyin(new_name);
            if let Err(e) = AFile::update_column(file_id, "name_pinyin", &name_pinyin) {
                eprintln!("Error while renaming file in DB: {}", e);
                return None;
            }

            match AFile::update_column(file_id, "name", &new_name) {
                Ok(_) => Some(new_file_path),
                Err(e) => {
                    eprintln!("Error while renaming file in DB: {}", e);
                    None
                }
            }
        }
        None => None,
    }
}

/// move a file to dest folder
#[tauri::command]
pub fn move_file(
    file_id: i64,
    file_path: &str,
    new_folder_id: i64,
    new_folder_path: &str,
) -> Option<String> {
    if !ensure_library_storage_mutation_allowed_or_none() {
        return None;
    }
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

/// delete a file
#[tauri::command]
pub fn delete_file(file_id: i64, file_path: &str) -> Result<usize, String> {
    ensure_library_storage_mutation_allowed()?;
    // trash the file
    trash::delete(file_path).map_err(|e| e.to_string())?;

    // delete the file from db
    AFile::delete(file_id).map_err(|e| format!("Error while deleting file from DB: {}", e))
}

/// delete a file from db
#[tauri::command]
pub fn delete_db_file(file_id: i64) -> Result<usize, String> {
    ensure_library_storage_mutation_allowed()?;
    // delete the file from db
    AFile::delete(file_id).map_err(|e| format!("Error while deleting file from DB: {}", e))
}

/// edit a file's comment
#[tauri::command]
pub fn edit_file_comment(file_id: i64, comment: &str) -> Result<usize, String> {
    ensure_library_storage_mutation_allowed()?;
    AFile::update_column(file_id, "comments", &comment)
        .map_err(|e| format!("Error while editing file comment: {}", e))
}

/// get a file's thumb image, if not exist, create a new one
#[tauri::command]
pub async fn get_file_thumb(
    file_id: i64,
    file_path: &str,
    file_type: i64,
    orientation: i32,
    thumbnail_size: u32,
    force_regenerate: bool,
) -> Result<Option<AThumb>, String> {
    ensure_library_storage_mutation_allowed()?;
    AThumb::get_or_create_thumb(
        file_id,
        file_path,
        file_type,
        orientation,
        thumbnail_size,
        force_regenerate,
    )
    .map_err(|e| format!("Error while getting or creating thumbnail: {}", e))
}

/// get a file's info
#[tauri::command]
pub fn get_file_info(file_id: i64) -> Result<Option<AFile>, String> {
    AFile::get_file_info(file_id).map_err(|e| format!("Error while getting file info: {}", e))
}

/// update a file's info
#[tauri::command]
pub fn update_file_info(file_id: i64, file_path: &str) -> Result<Option<AFile>, String> {
    ensure_library_storage_mutation_allowed()?;
    AFile::update_file_info(file_id, file_path)
        .map_err(|e| format!("Error while updating file info: {}", e))
}

/// add or refresh a file in db and return the indexed file info
#[tauri::command]
pub fn add_file_to_db(folder_id: i64, file_path: &str) -> Result<Option<AFile>, String> {
    ensure_library_storage_mutation_allowed()?;
    let file_type = t_utils::get_file_type(file_path)
        .ok_or_else(|| format!("Unsupported file type: {}", file_path))?;
    let (file, _) = AFile::add_to_db(folder_id, file_path, file_type)?;
    Ok(Some(file))
}

/// check if file exists
#[tauri::command]
pub fn check_file_exists(file_path: &str) -> bool {
    Path::new(file_path).exists()
}

/// set a file's rotate status
#[tauri::command]
pub fn set_file_rotate(file_id: i64, rotate: i32) -> Result<usize, String> {
    ensure_library_storage_mutation_allowed()?;
    AFile::update_column(file_id, "rotate", &rotate)
        .map_err(|e| format!("Error while setting file rotate: {}", e))
}

/// get a file's has_tags status (true or false)
#[tauri::command]
pub fn get_file_has_tags(file_id: i64) -> Result<bool, String> {
    AFile::get_has_tags(file_id)
        .map_err(|e| format!("Error while getting file has_tags status: {}", e))
}

// favorite

/// get all favorite folders
#[tauri::command]
pub fn get_favorite_folders() -> Result<Vec<AFolder>, String> {
    AFolder::get_favorite_folders()
        .map_err(|e| format!("Error while getting favorite folders: {}", e))
}

/// get a folder's favorite status (true or false)
#[tauri::command]
pub fn get_folder_favorite(folder_path: &str) -> Result<bool, String> {
    let is_favorite_opt = AFolder::get_is_favorite(folder_path)
        .map_err(|e| format!("Error while getting folder favorite: {}", e))?;

    match is_favorite_opt {
        Some(val) => Ok(val),
        None => Ok(false), // Default to false if not found
    }
}

/// set a folder's favorite status (true or false)
#[tauri::command]
pub fn set_folder_favorite(folder_id: i64, is_favorite: bool) -> Result<usize, String> {
    ensure_library_storage_mutation_allowed()?;
    AFolder::update_column(folder_id, "is_favorite", &is_favorite)
        .map_err(|e| format!("Error while setting folder favorite: {}", e))
}

/// set a file's favorite status (true or false)
#[tauri::command]
pub fn set_file_favorite(file_id: i64, is_favorite: bool) -> Result<usize, String> {
    ensure_library_storage_mutation_allowed()?;
    AFile::update_column(file_id, "is_favorite", &is_favorite)
        .map_err(|e| format!("Error while setting file favorite: {}", e))
}

/// set a file's rating (0-5)
#[tauri::command]
pub fn set_file_rating(file_id: i64, rating: i32) -> Result<usize, String> {
    ensure_library_storage_mutation_allowed()?;
    let clamped = rating.clamp(0, 5);
    AFile::update_column(file_id, "rating", &clamped)
        .map_err(|e| format!("Error while setting file rating: {}", e))
}

// tag

/// get all tags
#[tauri::command]
pub fn get_all_tags() -> Result<Vec<ATag>, String> {
    ATag::get_all().map_err(|e| format!("Error while getting all tags: {}", e))
}

/// get tag name by id
#[tauri::command]
pub fn get_tag_name(tag_id: i64) -> Result<String, String> {
    ATag::get_name(tag_id).map_err(|e| format!("Error while getting tag name: {}", e))
}

/// create a new tag
#[tauri::command]
pub fn create_tag(name: &str) -> Result<ATag, String> {
    ensure_library_storage_mutation_allowed()?;
    ATag::add(name).map_err(|e| format!("Error while creating tag: {}", e))
}

/// rename a tag
#[tauri::command]
pub fn rename_tag(tag_id: i64, new_name: &str) -> Result<usize, String> {
    ensure_library_storage_mutation_allowed()?;
    ATag::rename(tag_id, new_name).map_err(|e| format!("Error while renaming tag: {}", e))
}

/// delete a tag
#[tauri::command]
pub fn delete_tag(tag_id: i64) -> Result<usize, String> {
    ensure_library_storage_mutation_allowed()?;
    ATag::delete(tag_id).map_err(|e| format!("Error while deleting tag: {}", e))
}

/// get all tags for a specific file
#[tauri::command]
pub fn get_tags_for_file(file_id: i64) -> Result<Vec<ATag>, String> {
    ATag::get_tags_for_file(file_id)
        .map_err(|e| format!("Error while getting tags for file: {}", e))
}

/// add a tag to a file
#[tauri::command]
pub fn add_tag_to_file(file_id: i64, tag_id: i64) -> Result<(), String> {
    ensure_library_storage_mutation_allowed()?;
    ATag::add_tag_to_file(file_id, tag_id)
        .map_err(|e| format!("Error while adding tag to file: {}", e))
}

/// remove a tag from a file
#[tauri::command]
pub fn remove_tag_from_file(file_id: i64, tag_id: i64) -> Result<usize, String> {
    ensure_library_storage_mutation_allowed()?;
    ATag::remove_tag_from_file(file_id, tag_id)
        .map_err(|e| format!("Error while removing tag from file: {}", e))
}

// calendar

/// get camera's taken dates
#[tauri::command]
pub fn get_taken_dates(ascending: bool) -> Result<Vec<(String, i64)>, String> {
    AFile::get_taken_dates(ascending).map_err(|e| format!("Error while getting taken dates: {}", e))
}

// camera

/// get a file's camera make and model info
#[tauri::command]
pub fn get_camera_info() -> Result<Vec<ACamera>, String> {
    ACamera::get_from_db().map_err(|e| format!("Error while getting camera info: {}", e))
}

/// get a file's lens make and model info
#[tauri::command]
pub fn get_lens_info() -> Result<Vec<ALens>, String> {
    ALens::get_from_db().map_err(|e| format!("Error while getting lens info: {}", e))
}

// location

/// get a file's location info
#[tauri::command]
pub fn get_location_info() -> Result<Vec<ALocation>, String> {
    ALocation::get_from_db().map_err(|e| format!("Error while getting location info: {}", e))
}

// settings

/// get package info
#[tauri::command]
pub fn get_package_info() -> t_utils::PackageInfo {
    t_utils::PackageInfo::new()
}

/// get the build time
#[tauri::command]
pub fn get_build_time() -> u64 {
    BUILD_UNIX_TIME
}

/// get db file info
#[tauri::command]
pub fn get_storage_file_info() -> Result<t_utils::FileInfo, String> {
    // Get the database file path
    let db_file_path = t_config::get_current_db_path()
        .map_err(|e| format!("Failed to get the database file path: {}", e))?;

    match t_utils::FileInfo::new(&db_file_path) {
        Ok(info) => Ok(info),
        Err(e) => Err(format!("Failed to get the database file size: {}", e)),
    }
}

// image search

/// check ai status
#[tauri::command]
pub fn check_ai_status(state: State<t_ai::AiState>) -> String {
    AFile::check_ai_status(&state)
}

/// generate embedding for a file
#[tauri::command]
pub fn generate_embedding(state: State<t_ai::AiState>, file_id: i64) -> Result<String, String> {
    ensure_library_storage_mutation_allowed()?;
    AFile::generate_embedding(&state, file_id)
}

// search similar images
#[tauri::command]
pub fn search_similar_images(
    state: State<t_ai::AiState>,
    params: ImageSearchParams,
) -> Result<Vec<AFile>, String> {
    AFile::search_similar_images(&state, params)
        .map_err(|e| format!("Error while searching similar images: {}", e))
}

// face recognition

/// index faces for all images in the current library
#[tauri::command]
pub fn index_faces(
    app_handle: tauri::AppHandle,
    state: State<t_face::FaceState>,
    cancel_state: State<t_face::FaceIndexCancellation>,
    status_state: State<t_face::FaceIndexingStatus>,
    progress_state: State<t_face::FaceIndexProgressState>,
    cluster_epsilon: Option<f32>,
) -> Result<(), String> {
    ensure_library_storage_mutation_allowed()?;
    t_face::run_face_indexing(
        app_handle,
        (*state).clone(),
        (*cancel_state).clone(),
        (*status_state).clone(),
        (*progress_state).clone(),
        cluster_epsilon,
    )
}

/// get face indexing stats
#[tauri::command]
pub fn get_face_stats() -> Result<t_face::FaceStats, String> {
    let (total, processed, unprocessed, faces) = t_sqlite::Face::get_stats_full()
        .map_err(|e| format!("Error while getting face stats: {}", e))?;

    Ok(t_face::FaceStats {
        total,
        processed,
        unprocessed,
        faces,
    })
}

/// cancel face indexing
#[tauri::command]
pub fn cancel_face_index(state: State<t_face::FaceIndexCancellation>) -> Result<(), String> {
    *state.0.lock().unwrap() = true;

    Ok(())
}

/// reset all faces (delete all faces and persons)
#[tauri::command]
pub fn reset_faces() -> Result<(), String> {
    ensure_library_storage_mutation_allowed()?;
    t_sqlite::Face::reset_all().map_err(|e| format!("Error while resetting faces: {}", e))
}

/// check if face indexing is running, return (is_running, progress)
#[tauri::command]
pub fn is_face_indexing(
    status_state: State<t_face::FaceIndexingStatus>,
    progress_state: State<t_face::FaceIndexProgressState>,
) -> Result<(bool, Option<t_face::FaceIndexProgress>), String> {
    let is_running = *status_state.0.lock().unwrap();
    let progress = if is_running {
        Some(progress_state.0.lock().unwrap().clone())
    } else {
        None
    };
    Ok((is_running, progress))
}

/// get all persons with face counts
#[tauri::command]
pub fn get_persons() -> Result<Vec<Person>, String> {
    Person::get_all().map_err(|e| format!("Error while getting persons: {}", e))
}

/// rename a person
#[tauri::command]
pub fn rename_person(person_id: i64, name: String) -> Result<usize, String> {
    ensure_library_storage_mutation_allowed()?;
    Person::rename(person_id, &name).map_err(|e| format!("Error while renaming person: {}", e))
}

/// delete a person
#[tauri::command]
pub fn delete_person(person_id: i64) -> Result<usize, String> {
    ensure_library_storage_mutation_allowed()?;
    Person::delete(person_id).map_err(|e| format!("Error while deleting person: {}", e))
}

/// get faces for a file
#[tauri::command]
pub fn get_faces_for_file(file_id: i64) -> Result<Vec<t_sqlite::Face>, String> {
    t_sqlite::Face::get_for_file(file_id)
        .map_err(|e| format!("Error while getting faces for file: {}", e))
}

// ----------------------------------------------------------------------------
// Deduplication Commands
// ----------------------------------------------------------------------------

#[tauri::command]
pub fn dedup_start_scan(
    app_handle: tauri::AppHandle,
    state: tauri::State<'_, crate::t_dedup::DedupState>,
    params: Option<crate::t_sqlite::QueryParams>,
) -> Result<(), String> {
    ensure_library_storage_mutation_allowed()?;
    crate::t_dedup::start_scan(app_handle, state, params)
}

#[tauri::command]
pub fn dedup_get_scan_status(
    state: tauri::State<'_, crate::t_dedup::DedupState>,
) -> Result<crate::t_dedup::DedupScanStatus, String> {
    let status = state.status.lock().unwrap();
    Ok(status.clone())
}

#[tauri::command]
pub fn dedup_cancel_scan(
    state: tauri::State<'_, crate::t_dedup::DedupState>,
) -> Result<(), String> {
    state
        .cancel_flag
        .store(true, std::sync::atomic::Ordering::SeqCst);
    Ok(())
}

#[tauri::command]
pub fn dedup_list_groups(
    page: u32,
    page_size: u32,
    sort_by: String, // E.g., "size_desc", "count_desc"
    filter: String,  // E.g., "all", "unreviewed"
) -> Result<Vec<crate::t_dedup::DedupGroup>, String> {
    crate::t_dedup::list_groups(page, page_size, &sort_by, &filter)
}

#[tauri::command]
pub fn dedup_get_overview() -> Result<crate::t_dedup::DedupOverview, String> {
    crate::t_dedup::get_overview()
}

#[tauri::command]
pub fn dedup_get_group(group_id: i64) -> Result<crate::t_dedup::DedupGroup, String> {
    crate::t_dedup::get_group(group_id)
}

#[tauri::command]
pub fn dedup_set_keep(group_id: i64, file_id: i64) -> Result<(), String> {
    ensure_library_storage_mutation_allowed()?;
    crate::t_dedup::set_keep(group_id, file_id)
}

#[tauri::command]
pub fn dedup_delete_selected(
    group_ids: Option<Vec<i64>>,
    file_ids: Option<Vec<i64>>,
) -> Result<(), String> {
    ensure_library_storage_mutation_allowed()?;
    crate::t_dedup::delete_selected(group_ids, file_ids)
}

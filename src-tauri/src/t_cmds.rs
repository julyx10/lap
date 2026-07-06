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
    ACamera, ACollection, ACollectionOrder, AFile, AFolder, ALens, ALocation, ATag, ATagFileState,
    ATagSelectionCount, AThumb, ATimeLine, Album, GroupedQueryResult, ImageSearchParams, Person,
    QueryParams, SmartQueryParams,
};
use crate::t_storage;
use crate::t_utils;
use crate::{t_ai, t_common, t_sqlite};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Cursor;
use std::path::Path;
use std::process::Command;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, State};

// cancellation token for indexing
pub struct IndexCancellation(pub Arc<Mutex<HashMap<i64, bool>>>);

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThumbRequest {
    pub file_id: i64,
    pub file_path: Option<String>,
    pub file_type: Option<i64>,
    pub orientation: Option<i32>,
    pub album_id: Option<i64>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionOption {
    pub value: String,
    pub label: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SupportedFormatExtensions {
    pub image: Vec<String>,
    pub raw: Vec<String>,
    pub video: Vec<String>,
    pub options: Vec<ExtensionOption>,
}

// build info
include!(concat!(env!("OUT_DIR"), "/build_info.rs"));

// library

/// get app config (libraries list and current library)
#[tauri::command]
pub fn get_app_config() -> Result<AppConfig, String> {
    t_config::load_app_config()
}

fn extension_description(ext: &str) -> &'static str {
    match ext {
        "3fr" => "Hasselblad RAW",
        "3gp" => "3GPP Multimedia",
        "arw" => "Sony Alpha RAW",
        "asf" => "Advanced Systems Format",
        "avi" => "Audio Video Interleave",
        "avif" => "AV1 Image File Format",
        "bmp" => "Bitmap Image File",
        "cr2" => "Canon RAW 2",
        "cr3" => "Canon RAW 3",
        "crw" => "Canon RAW",
        "dcr" => "Kodak RAW",
        "dds" => "DirectDraw Surface",
        "dng" => "Digital Negative",
        "dpx" => "Digital Picture Exchange",
        "erf" => "Epson RAW",
        "exr" => "OpenEXR High Dynamic Range Image",
        "flv" => "Flash Video",
        "gif" => "Graphics Interchange Format",
        "hdr" => "Radiance High Dynamic Range Image",
        "heic" => "High Efficiency Image Container",
        "heif" => "High Efficiency Image File",
        "hevc" => "High Efficiency Video Coding",
        "hif" => "HEIF Image File",
        "j2c" => "JPEG 2000 Codestream",
        "j2k" => "JPEG 2000 Codestream",
        "jp2" => "JPEG 2000 Image",
        "jpc" => "JPEG 2000 Codestream",
        "jpeg" => "Joint Photographic Experts Group",
        "jpf" => "JPEG 2000 Image",
        "jpg" => "Joint Photographic Experts Group",
        "jpx" => "JPEG 2000 Extended Image",
        "jxl" => "JPEG XL",
        "kdc" => "Kodak Digital Camera RAW",
        "m2ts" => "Blu-ray BDAV Video",
        "m4v" => "MPEG-4 Video",
        "mdc" => "Minolta RAW",
        "mef" => "Mamiya RAW",
        "mkv" => "Matroska Video",
        "mod" => "Camcorder Video",
        "mos" => "Leaf RAW",
        "mov" => "QuickTime Movie",
        "mp4" => "MPEG-4 Video",
        "mpeg" => "Moving Picture Experts Group Video",
        "mpg" => "MPEG Video",
        "mrw" => "Minolta RAW",
        "mts" => "AVCHD Video",
        "nef" => "Nikon Electronic Format",
        "nrw" => "Nikon RAW",
        "orf" => "Olympus RAW Format",
        "pef" => "Pentax Electronic File",
        "png" => "Portable Network Graphics",
        "psd" => "Photoshop Document",
        "qoi" => "Quite OK Image",
        "raf" => "Fujifilm RAW",
        "raw" => "Generic Camera RAW",
        "rgbe" => "Radiance RGBE Image",
        "rw2" => "Panasonic RAW 2",
        "rwl" => "Leica RAW",
        "sr2" => "Sony RAW 2",
        "srf" => "Sony RAW Format",
        "srw" => "Samsung RAW",
        "tga" => "Truevision Targa Image",
        "tif" => "Tagged Image File",
        "tiff" => "Tagged Image File Format",
        "tod" => "JVC HD Camcorder Video",
        "ts" => "MPEG Transport Stream",
        "webm" => "WebM Video",
        "webp" => "Web Picture Format",
        "wmv" => "Windows Media Video",
        _ => "File Format",
    }
}

fn extension_option(ext: &str) -> ExtensionOption {
    ExtensionOption {
        value: ext.to_string(),
        label: format!(
            "{} ({})",
            ext.to_ascii_uppercase(),
            extension_description(ext)
        ),
    }
}

/// get supported file format extensions for smart album filters
#[tauri::command]
pub fn get_supported_format_extensions() -> SupportedFormatExtensions {
    let image: Vec<String> = t_common::NORMAL_IMGS
        .iter()
        .chain(t_common::FFMPEG_BACKED_IMGS.iter())
        .map(|ext| ext.to_string())
        .collect();
    let raw: Vec<String> = t_common::RAW_IMGS
        .iter()
        .map(|ext| ext.to_string())
        .collect();
    let video: Vec<String> = t_common::VIDEOS.iter().map(|ext| ext.to_string()).collect();
    let mut options: Vec<ExtensionOption> = image
        .iter()
        .chain(raw.iter())
        .chain(video.iter())
        .map(|ext| extension_option(ext))
        .collect();
    options.sort_by(|a, b| a.value.cmp(&b.value));

    SupportedFormatExtensions {
        image,
        raw,
        video,
        options,
    }
}

/// set last selected item index
#[tauri::command]
pub fn set_last_selected_item_index(index: i64) -> Result<(), String> {
    let mut config = t_config::load_app_config()?;
    config.last_selected_item_index = index;
    t_config::save_app_config(&config)
}

#[tauri::command]
pub fn get_db_storage_dir() -> Result<String, String> {
    t_storage::get_db_storage_dir()
}

#[tauri::command]
pub fn is_using_custom_db_storage() -> Result<bool, String> {
    t_storage::is_using_custom_db_storage()
}

fn ensure_db_storage_change_allowed(
    status_state: &State<t_face::FaceIndexingStatus>,
) -> Result<(), String> {
    if t_storage::is_db_migration_in_progress() {
        return Err("Database storage migration is already in progress.".to_string());
    }

    let is_library_indexing = t_config::get_current_library_state()
        .map(|state| state.index.status == 1)
        .unwrap_or(false);
    if is_library_indexing {
        return Err(
            "Cannot change database storage while library indexing is running.".to_string(),
        );
    }

    if *status_state.0.lock().unwrap() {
        return Err("Cannot change database storage while face indexing is running.".to_string());
    }

    if t_sqlite::has_active_thumb_background_tasks() {
        return Err(
            "Cannot change database storage while thumbnails are still being generated."
                .to_string(),
        );
    }

    Ok(())
}

#[tauri::command]
pub fn change_db_storage_dir(
    new_dir: &str,
    status_state: State<t_face::FaceIndexingStatus>,
) -> Result<String, String> {
    ensure_db_storage_change_allowed(&status_state)?;
    t_storage::change_db_storage_dir(new_dir)
}

#[tauri::command]
pub fn reset_db_storage_dir(
    status_state: State<t_face::FaceIndexingStatus>,
) -> Result<String, String> {
    ensure_db_storage_change_allowed(&status_state)?;
    t_storage::reset_db_storage_dir()
}

#[tauri::command]
pub fn add_library(name: &str) -> Result<Library, String> {
    t_config::add_library(name)
}

/// hide a library
#[tauri::command]
pub fn hide_library(id: &str, hidden: bool) -> Result<(), String> {
    t_config::hide_library(id, hidden)
}

/// reorder libraries
#[tauri::command]
pub fn reorder_libraries(ids: Vec<String>) -> Result<(), String> {
    t_config::reorder_libraries(ids)
}

/// edit library name
#[tauri::command]
pub fn edit_library(id: &str, name: &str) -> Result<(), String> {
    t_config::edit_library(id, name)
}

/// remove a library (also deletes the database file)
#[tauri::command]
pub fn remove_library(id: &str) -> Result<(), String> {
    t_config::remove_library(id)
}

/// switch to a different library
#[tauri::command]
pub async fn switch_library(app_handle: tauri::AppHandle, id: String) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || -> Result<(), String> {
        t_config::switch_library(&id)?;
        t_sqlite::clear_conn_pool();
        t_sqlite::create_db()?;
        Ok(())
    })
    .await
    .map_err(|e| format!("Failed to join switch library task: {}", e))??;

    t_utils::restore_album_scopes(&app_handle)?;
    t_utils::start_folder_mtime_sync(app_handle);
    Ok(())
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

/// batch-generate thumbnails for a directory into an output folder
#[tauri::command]
pub fn generate_directory_thumbnails(
    dir_path: &str,
    output_dir: &str,
    thumbnail_size: u32,
) -> Result<t_image::BatchThumbnailStats, String> {
    t_image::generate_directory_thumbnails(dir_path, output_dir, thumbnail_size)
}

/// get one album
#[tauri::command]
pub fn get_album(album_id: i64) -> Result<Album, String> {
    Album::get_album_by_id(album_id).map_err(|e| format!("Error while getting one album: {}", e))
}

/// recount files for an album and return updated album
#[tauri::command]
pub fn recount_album(album_id: i64) -> Result<Album, String> {
    Album::recount_album(album_id).map_err(|e| format!("Error while recounting album: {}", e))
}

/// add an album
#[tauri::command]
pub fn add_album(app_handle: tauri::AppHandle, folder_path: &str) -> Result<Album, String> {
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
    let _ = Album::update_column(id, "name", &name)
        .map_err(|e| format!("Error while editing album with id {}: {}", id, e));

    Album::update_column(id, "description", &description)
        .map_err(|e| format!("Error while editing album with id {}: {}", id, e))
}

/// remove an album
#[tauri::command]
pub fn remove_album(id: i64) -> Result<usize, String> {
    let result = Album::delete_from_db(id)
        .map_err(|e| format!("Error while removing album with id {}: {}", id, e))?;

    let library_id = crate::t_config::load_app_config()
        .map(|c| c.current_library_id)
        .unwrap_or_else(|_| "default".to_string());
    let album_cache_dir = crate::t_config::get_app_cache_dir()
        .map(|dir| dir.join(library_id).join(id.to_string()))
        .map_err(|e| format!("Error while resolving album thumbnail cache path: {}", e))?;
    if album_cache_dir.exists() {
        std::fs::remove_dir_all(&album_cache_dir)
            .map_err(|e| format!("Error while removing album thumbnail cache: {}", e))?;
    }

    Ok(result)
}

/// set album display order
#[tauri::command]
pub fn set_album_display_order(id: i64, display_order: i32) -> Result<usize, String> {
    Album::update_column(id, "display_order_id", &display_order)
        .map_err(|e| format!("Error while setting album display order: {}", e))
}

/// set album cover
#[tauri::command]
pub fn set_album_cover(id: i64, file_id: i64) -> Result<usize, String> {
    Album::update_column(id, "cover_file_id", &file_id)
        .map_err(|e| format!("Error while setting album cover: {}", e))
}

/// index album
#[tauri::command]
pub fn index_album(
    app_handle: tauri::AppHandle,
    state: State<IndexCancellation>,
    album_id: i64,
    thumbnail_size: u32,
    skip_file_path: Option<String>,
) -> Result<(), String> {
    // Reset cancellation flag
    state.0.lock().unwrap().insert(album_id, false);
    let cancellation_token = state.0.clone();

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
    t_utils::authorize_directory_scope(&app_handle, folder_path)
        .map_err(|e| format!("Error while authorizing folder '{}': {}", folder_path, e))?;

    AFolder::add_to_db(album_id, folder_path)
        .map_err(|e| format!("Error while adding folder to DB: {}", e))
}

/// fetch folder and build a FileNode
#[tauri::command]
pub fn fetch_folder(
    path: &str,
    is_recursive: bool,
    sort: i64,
) -> Result<t_utils::FileNode, String> {
    t_utils::FileNode::build_nodes(path, is_recursive, sort)
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
    let new_folder_path = t_utils::rename_folder(folder_path, new_folder_name);

    match new_folder_path {
        Some(new_path) => {
            if let Err(e) = Album::rename_root_folder(folder_path, &new_path) {
                eprintln!("Error while renaming root folder in DB: {}", e);
                return None;
            }
            Some(new_path)
        }
        None => None,
    }
}

/// move a folder
#[tauri::command]
pub fn move_folder(
    folder_path: &str,
    new_album_id: i64,
    new_folder_path: &str,
    conflict_policy: &str,
) -> Result<String, String> {
    let transfer = t_utils::move_folder_with_policy(
        folder_path,
        new_folder_path,
        t_utils::FileConflictPolicy::from_str(conflict_policy),
    )?;
    let new_path = transfer.path.clone();
    let db_result = if conflict_policy == "replace" {
        AFolder::replace_moved_folder(folder_path, new_album_id, &new_path)
    } else {
        AFolder::move_folder(folder_path, new_album_id, &new_path)
    };
    if let Err(error) = db_result {
        let rollback_error = transfer.rollback_move(Path::new(folder_path)).err();
        return Err(match rollback_error {
            Some(rollback_error) => format!(
                "Error while moving folder in DB: {}; rollback also failed: {}",
                error, rollback_error
            ),
            None => format!("Error while moving folder in DB: {}", error),
        });
    }
    transfer.finalize()
}

/// move a folder outside the library and remove its database records
#[tauri::command]
pub fn move_folder_outside_library(
    folder_path: &str,
    new_folder_path: &str,
    conflict_policy: &str,
) -> Result<String, String> {
    let transfer = t_utils::move_folder_with_policy(
        folder_path,
        new_folder_path,
        t_utils::FileConflictPolicy::from_str(conflict_policy),
    )?;

    if let Err(error) = AFolder::delete_folder(folder_path) {
        let rollback_error = transfer.rollback_move(Path::new(folder_path)).err();
        return Err(match rollback_error {
            Some(rollback_error) => format!(
                "Error while removing folder from DB: {}; rollback also failed: {}",
                error, rollback_error
            ),
            None => format!("Error while removing folder from DB: {}", error),
        });
    }

    transfer.finalize()
}

/// copy a folder
#[tauri::command]
pub fn copy_folder(
    folder_path: &str,
    new_folder_path: &str,
    new_album_id: i64,
    conflict_policy: &str,
) -> Result<String, String> {
    let transfer = t_utils::copy_folder_with_policy(
        folder_path,
        new_folder_path,
        t_utils::FileConflictPolicy::from_str(conflict_policy),
    )?;
    let new_path = transfer.path.clone();
    if new_album_id > 0 {
        let db_result = if conflict_policy == "replace" {
            AFolder::replace_copied_folder(new_album_id, &new_path)
        } else {
            AFolder::add_to_db(new_album_id, &new_path)
        };
        if let Err(error) = db_result {
            let rollback_error = transfer.rollback_copy().err();
            return Err(match rollback_error {
                Some(rollback_error) => format!(
                    "Error while adding copied folder to DB: {}; rollback also failed: {}",
                    error, rollback_error
                ),
                None => format!("Error while adding copied folder to DB: {}", error),
            });
        }
    }
    transfer.finalize()
}

/// delete a folder (move to trash)
#[tauri::command]
pub fn delete_folder(folder_path: &str) -> Result<usize, String> {
    // trash the folder
    t_utils::trash_path(folder_path)?;

    // delete the folder and all children from db
    AFolder::delete_folder(folder_path)
        .map_err(|e| format!("Error while deleting folder from DB: {}", e))
}

/// permanently delete a folder (skip trash)
#[tauri::command]
pub fn delete_folder_permanently(folder_path: &str) -> Result<usize, String> {
    t_utils::delete_folder_permanently(folder_path)?;

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
    open_files_with_app(vec![file_path.to_string()], app_path)
}

/// open one or more files with a specific external application
#[tauri::command]
pub fn open_files_with_app(file_paths: Vec<String>, app_path: &str) -> Result<(), String> {
    let file_paths: Vec<String> = file_paths
        .into_iter()
        .filter(|p| !p.is_empty())
        .collect();

    if file_paths.is_empty() || app_path.is_empty() {
        return Err("Missing file path or app path".to_string());
    }

    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg("-a")
            .arg(app_path)
            .args(&file_paths)
            .spawn()
            .map_err(|e| e.to_string())?;
    }

    #[cfg(not(target_os = "macos"))]
    {
        Command::new(app_path)
            .args(&file_paths)
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
pub async fn get_query_count_and_sum(params: QueryParams) -> Result<(i64, i64), String> {
    AFile::get_query_count_and_sum(&params)
        .map_err(|e| format!("Error while getting query files count: {}", e))
}

/// get query time line
#[tauri::command]
pub async fn get_query_time_line(params: QueryParams) -> Result<Vec<ATimeLine>, String> {
    AFile::get_query_time_line(&params)
        .map_err(|e| format!("Error while getting query timeline: {}", e))
}

/// get query file
#[tauri::command]
pub async fn get_query_files(params: QueryParams, offset: i64, limit: i64) -> Result<Vec<AFile>, String> {
    AFile::get_query_files(&params, offset, limit)
        .map_err(|e| format!("Error while getting query files: {}", e))
}

/// Get grouped render rows for a normal query.
/// The result includes group header rows, file item rows, group metadata, and row counts for virtual scrolling.
#[tauri::command]
pub async fn get_grouped_query_rows(
    params: QueryParams,
    offset: i64,
    limit: i64,
) -> Result<GroupedQueryResult, String> {
    AFile::get_grouped_query_rows(&params, offset, limit)
        .map_err(|e| format!("Error while getting grouped query rows: {}", e))
}

/// Get all file ids in one group for a normal query.
/// Used by the group header checkbox so selecting a group is not limited to loaded rows.
#[tauri::command]
pub async fn get_group_file_ids(params: QueryParams, group_id: String) -> Result<Vec<i64>, String> {
    AFile::get_group_file_ids(&params, &group_id)
        .map_err(|e| format!("Error while getting group file ids: {}", e))
}

/// Get all file ids in the current normal query.
/// Used by Select All to support large virtualized result sets without loading every file object.
#[tauri::command]
pub async fn get_query_file_ids(params: QueryParams) -> Result<Vec<i64>, String> {
    AFile::get_query_file_ids(&params)
        .map_err(|e| format!("Error while getting query file ids: {}", e))
}

#[tauri::command]
pub async fn get_query_file_position(params: QueryParams, file_id: i64) -> Result<Option<i64>, String> {
    AFile::get_query_file_position(&params, file_id)
        .map_err(|e| format!("Error while getting query file position: {}", e))
}

// collection

#[tauri::command]
pub fn list_collections() -> Result<Vec<ACollection>, String> {
    ACollection::list().map_err(|e| format!("Error while listing collections: {}", e))
}

#[tauri::command]
pub fn create_collection(name: &str) -> Result<ACollection, String> {
    ACollection::create(name).map_err(|e| format!("Error while creating collection: {}", e))
}

#[tauri::command]
pub fn rename_collection(id: i64, name: &str) -> Result<usize, String> {
    ACollection::rename(id, name).map_err(|e| format!("Error while renaming collection: {}", e))
}

#[tauri::command]
pub fn delete_collection(id: i64) -> Result<usize, String> {
    ACollection::delete(id).map_err(|e| format!("Error while deleting collection: {}", e))
}

#[tauri::command]
pub fn reorder_collections(items: Vec<ACollectionOrder>) -> Result<usize, String> {
    ACollection::reorder(items).map_err(|e| format!("Error while reordering collections: {}", e))
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CollectionAddResult {
    pub added: usize,
    pub skipped: usize,
}

#[tauri::command]
pub fn add_files_to_collection(
    collection_id: i64,
    file_ids: Vec<i64>,
) -> Result<CollectionAddResult, String> {
    ACollection::add_files(collection_id, file_ids)
        .map(|(added, skipped)| CollectionAddResult { added, skipped })
        .map_err(|e| format!("Error while adding files to collection: {}", e))
}

#[tauri::command]
pub fn remove_files_from_collection(
    collection_id: i64,
    file_ids: Vec<i64>,
) -> Result<usize, String> {
    ACollection::remove_files(collection_id, file_ids)
        .map_err(|e| format!("Error while removing files from collection: {}", e))
}

#[tauri::command]
pub fn clear_collection(collection_id: i64) -> Result<usize, String> {
    ACollection::clear(collection_id).map_err(|e| format!("Error while clearing collection: {}", e))
}

#[tauri::command]
pub fn get_collection_file_ids(collection_id: i64) -> Result<Vec<i64>, String> {
    ACollection::file_ids(collection_id)
        .map_err(|e| format!("Error while getting collection file ids: {}", e))
}

#[tauri::command]
pub fn get_collection_count_and_sum(
    collection_id: i64,
    params: QueryParams,
) -> Result<(i64, i64), String> {
    AFile::get_collection_count_and_sum(collection_id, &params)
        .map_err(|e| format!("Error while getting collection count: {}", e))
}

#[tauri::command]
pub fn get_collection_files(
    collection_id: i64,
    params: QueryParams,
    offset: i64,
    limit: i64,
) -> Result<Vec<AFile>, String> {
    AFile::get_collection_files(collection_id, &params, offset, limit)
        .map_err(|e| format!("Error while getting collection files: {}", e))
}

#[tauri::command]
pub async fn get_smart_query_count_and_sum(params: SmartQueryParams) -> Result<(i64, i64), String> {
    AFile::get_smart_query_count_and_sum(&params)
        .map_err(|e| format!("Error while getting smart query files count: {}", e))
}

#[tauri::command]
pub async fn get_smart_query_time_line(params: SmartQueryParams) -> Result<Vec<ATimeLine>, String> {
    AFile::get_smart_query_time_line(&params)
        .map_err(|e| format!("Error while getting smart query timeline: {}", e))
}

#[tauri::command]
pub async fn get_smart_query_files(
    params: SmartQueryParams,
    offset: i64,
    limit: i64,
) -> Result<Vec<AFile>, String> {
    AFile::get_smart_query_files(&params, offset, limit)
        .map_err(|e| format!("Error while getting smart query files: {}", e))
}

/// Get grouped render rows for a smart query.
/// The result includes group header rows, file item rows, group metadata, and row counts for virtual scrolling.
#[tauri::command]
pub async fn get_smart_grouped_query_rows(
    params: SmartQueryParams,
    offset: i64,
    limit: i64,
) -> Result<GroupedQueryResult, String> {
    AFile::get_smart_grouped_query_rows(&params, offset, limit)
        .map_err(|e| format!("Error while getting smart grouped query rows: {}", e))
}

/// Get all file ids in one group for a smart query.
/// Used by the group header checkbox so selecting a group is not limited to loaded rows.
#[tauri::command]
pub async fn get_smart_group_file_ids(
    params: SmartQueryParams,
    group_id: String,
) -> Result<Vec<i64>, String> {
    AFile::get_smart_group_file_ids(&params, &group_id)
        .map_err(|e| format!("Error while getting smart group file ids: {}", e))
}

/// Get all file ids in the current smart query.
/// Used by Select All to support large virtualized result sets without loading every file object.
#[tauri::command]
pub async fn get_smart_query_file_ids(params: SmartQueryParams) -> Result<Vec<i64>, String> {
    AFile::get_smart_query_file_ids(&params)
        .map_err(|e| format!("Error while getting smart query file ids: {}", e))
}

#[tauri::command]
pub async fn get_smart_query_file_position(
    params: SmartQueryParams,
    file_id: i64,
) -> Result<Option<i64>, String> {
    AFile::get_smart_query_file_position(&params, file_id)
        .map_err(|e| format!("Error while getting smart query file position: {}", e))
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

/// sync a single folder's mtime and DB records with the filesystem
#[tauri::command]
pub async fn sync_album_folder_mtimes(
    app_handle: tauri::AppHandle,
    album_id: i64,
    folder_id: i64,
    folder_path: String,
) -> Result<crate::t_utils::FolderMtimeSyncResult, String> {
    tauri::async_runtime::spawn_blocking(move || {
        crate::t_utils::sync_single_folder(&app_handle, album_id, folder_id, &folder_path)
    })
    .await
    .map_err(|e| format!("folder sync task failed: {}", e))?
}

#[tauri::command]
pub fn is_directory_accessible(path: &str) -> bool {
    t_utils::directory_accessible(path)
}

/// get the thumbnail count of the folder
#[tauri::command]
pub fn get_folder_thumb_count(file_type: i64, folder_id: i64) -> i64 {
    AThumb::get_folder_thumb_count(file_type, folder_id).unwrap_or_default()
}

/// edit an image
#[tauri::command]
pub async fn edit_image(params: t_image::EditParams) -> Result<bool, String> {
    Ok(t_image::edit_image(params).await)
}

/// copy an edited image to clipboard
#[tauri::command]
pub async fn copy_edited_image(params: t_image::EditParams) -> Result<bool, String> {
    Ok(t_image::copy_edited_image_to_clipboard(params).await)
}

/// copy up to 10 files to the clipboard
#[tauri::command]
pub async fn copy_images(
    app_handle: tauri::AppHandle,
    file_paths: Vec<String>,
) -> Result<usize, String> {
    t_image::copy_files_to_clipboard(&app_handle, file_paths).await
}

/// rename a file
#[tauri::command]
pub fn rename_file(file_id: i64, file_path: &str, new_name: &str) -> Option<String> {
    match t_utils::rename_file(file_path, new_name) {
        Some(new_file_path) => {
            let name_pinyin = t_utils::natural_sort_key(&new_name.to_lowercase());
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
    conflict_policy: &str,
) -> Result<String, String> {
    let old_album_id = AFile::get_file_info(file_id)
        .ok()
        .flatten()
        .and_then(|file| file.album_id);
    let new_album_id = AFolder::get_by_id(new_folder_id)
        .ok()
        .flatten()
        .map(|folder| folder.album_id);
    let replaced_file_id = if conflict_policy == "replace" {
        AFile::fetch(new_folder_id, file_path)
            .ok()
            .flatten()
            .and_then(|file| file.id)
            .filter(|id| *id != file_id)
    } else {
        None
    };

    let transfer = t_utils::move_file_with_policy(
        file_path,
        new_folder_path,
        t_utils::FileConflictPolicy::from_str(conflict_policy),
    )?;
    let db_result = if let Some(replaced_file_id) = replaced_file_id {
        AFile::replace_moved_file(file_id, replaced_file_id, new_folder_id)
    } else {
        AFile::update_column(file_id, "folder_id", &new_folder_id)
    };
    if let Err(error) = db_result {
        let rollback_error = transfer.rollback_move(Path::new(file_path)).err();
        return Err(match rollback_error {
            Some(rollback_error) => format!(
                "Error while moving file in DB: {}; rollback also failed: {}",
                error, rollback_error
            ),
            None => format!("Error while moving file in DB: {}", error),
        });
    }
    if let (Some(old_album_id), Some(new_album_id)) = (old_album_id, new_album_id) {
        let _ = AThumb::relocate_for_file(file_id, old_album_id, new_album_id)
            .map_err(|e| format!("Error while relocating thumbnail cache: {}", e));
    }
    transfer.finalize()
}

/// move a file outside the library and remove its database record
#[tauri::command]
pub fn move_file_outside_library(
    file_id: i64,
    file_path: &str,
    new_folder_path: &str,
    conflict_policy: &str,
) -> Result<String, String> {
    let transfer = t_utils::move_file_with_policy(
        file_path,
        new_folder_path,
        t_utils::FileConflictPolicy::from_str(conflict_policy),
    )?;

    if let Err(error) = AFile::delete(file_id) {
        let rollback_error = transfer.rollback_move(Path::new(file_path)).err();
        return Err(match rollback_error {
            Some(rollback_error) => format!(
                "Error while removing file from DB: {}; rollback also failed: {}",
                error, rollback_error
            ),
            None => format!("Error while removing file from DB: {}", error),
        });
    }

    transfer.finalize()
}

/// copy a file to dest folder
#[tauri::command]
pub fn copy_file(
    file_path: &str,
    new_folder_path: &str,
    conflict_policy: &str,
) -> Result<String, String> {
    t_utils::copy_file_with_policy(
        file_path,
        new_folder_path,
        t_utils::FileConflictPolicy::from_str(conflict_policy),
    )?
    .finalize()
}

/// import a file into a folder preserving the original file name
#[tauri::command]
pub fn import_file(
    file_path: &str,
    folder_id: i64,
    folder_path: &str,
) -> Result<Option<AFile>, String> {
    // Validate the source is a supported type *before* copying.
    t_utils::get_file_type(file_path)
        .ok_or_else(|| format!("Unsupported file type: {}", file_path))?;

    let new_path = t_utils::import_file(file_path, folder_path)
        .ok_or_else(|| format!("Failed to copy file: {}", file_path))?;
    let file_type = t_utils::get_file_type(&new_path).ok_or_else(|| {
        // The renamed file should have a valid extension; if not, remove
        // the orphan so the album folder stays clean.
        let _ = std::fs::remove_file(&new_path);
        format!("Unsupported file type after copy: {}", new_path)
    })?;
    let now = chrono::Utc::now().timestamp_millis();
    let (file, _) = AFile::add_to_db(folder_id, &new_path, file_type, now)?;
    Ok(Some(file))
}

/// import an image from a URL into a folder preserving the original file name when possible
#[tauri::command]
pub async fn import_url(
    url: &str,
    folder_id: i64,
    folder_path: String,
) -> Result<Option<AFile>, String> {
    import_url_inner(url, folder_id, folder_path).await
}

/// Import an image from the macOS drag pasteboard (for browser-sourced drags
/// where Tauri cannot provide file paths).
#[tauri::command]
pub async fn import_from_drag(
    folder_id: i64,
    folder_path: String,
) -> Result<Option<AFile>, String> {
    let url = crate::t_pasteboard::get_drag_image_url()
        .ok_or_else(|| "No image URL found in drag pasteboard".to_string())?;
    import_url_inner(&url, folder_id, folder_path).await
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DragPayload {
    pub file_paths: Vec<String>,
    pub url: Option<String>,
}

#[tauri::command]
pub fn get_drag_payload() -> DragPayload {
    DragPayload {
        file_paths: crate::t_pasteboard::get_drag_file_paths(),
        url: crate::t_pasteboard::get_drag_image_url(),
    }
}

async fn import_url_inner(
    url: &str,
    folder_id: i64,
    folder_path: String,
) -> Result<Option<AFile>, String> {
    let response = reqwest::get(url)
        .await
        .map_err(|e| format!("Failed to download image: {}", e))?;

    // Reject HTTP error statuses
    let status = response.status();
    if !status.is_success() {
        return Err(format!(
            "Server returned {} {}",
            status.as_u16(),
            status.canonical_reason().unwrap_or("")
        ));
    }

    // Require a supported image content type — validate via the shared
    // MIME→extension table so the response form the importer can name.
    let mime = {
        let ct = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| "Response missing Content-Type header".to_string())?;
        let m = ct.split(';').next().unwrap_or(ct).trim().to_string();
        t_utils::image_mime_to_ext(&m).ok_or_else(|| format!("Unsupported image format: {}", m))?;
        m
    };
    let original_name = response
        .headers()
        .get("content-disposition")
        .and_then(|value| value.to_str().ok())
        .and_then(filename_from_content_disposition)
        .or_else(|| filename_from_url(response.url().as_str()))
        .or_else(|| filename_from_url(url));

    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("Failed to read response: {}", e))?;

    let dest_folder = folder_path.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let new_path = original_name
            .and_then(|name| {
                t_utils::save_downloaded_bytes_with_name(&bytes, &mime, &name, &dest_folder)
            })
            .or_else(|| t_utils::save_bytes_to_folder(&bytes, &mime, &dest_folder))
            .ok_or_else(|| "Failed to save downloaded image".to_string())?;
        let file_type = t_utils::get_file_type(&new_path)
            .ok_or_else(|| format!("Unsupported file type: {}", new_path))?;
        let now = chrono::Utc::now().timestamp_millis();
        let (file, _) = AFile::add_to_db(folder_id, &new_path, file_type, now)?;
        Ok(Some(file))
    })
    .await
    .map_err(|e| format!("Failed to save file: {}", e))?
}

fn filename_from_content_disposition(value: &str) -> Option<String> {
    for part in value.split(';') {
        if let Some((key, raw_value)) = part.trim().split_once('=') {
            if key.trim().eq_ignore_ascii_case("filename*") {
                let raw_value = raw_value.trim().trim_matches('"');
                let encoded = raw_value
                    .split_once("''")
                    .map_or(raw_value, |(_, value)| value);
                return clean_import_filename(&percent_decode_utf8(encoded));
            }
        }
    }

    for part in value.split(';') {
        if let Some((key, raw_value)) = part.trim().split_once('=') {
            if key.trim().eq_ignore_ascii_case("filename") {
                return clean_import_filename(raw_value.trim().trim_matches('"'));
            }
        }
    }

    None
}

fn filename_from_url(url: &str) -> Option<String> {
    let without_query = url.split('?').next().unwrap_or(url);
    let without_fragment = without_query.split('#').next().unwrap_or(without_query);
    let last_segment = without_fragment.trim_end_matches('/').rsplit('/').next()?;
    clean_import_filename(&percent_decode_utf8(last_segment))
}

fn clean_import_filename(name: &str) -> Option<String> {
    let normalized = name.replace('\\', "/");
    let filename = Path::new(&normalized)
        .file_name()
        .and_then(|item| item.to_str())?
        .trim();
    if filename.is_empty() || filename.contains('\0') {
        None
    } else {
        Some(filename.to_string())
    }
}

fn percent_decode_utf8(value: &str) -> String {
    let bytes = value.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;

    while index < bytes.len() {
        if bytes[index] == b'%' && index + 2 < bytes.len() {
            if let Ok(hex) = u8::from_str_radix(&value[index + 1..index + 3], 16) {
                decoded.push(hex);
                index += 3;
                continue;
            }
        }

        decoded.push(bytes[index]);
        index += 1;
    }

    String::from_utf8(decoded).unwrap_or_else(|_| value.to_string())
}

/// Import a file from raw bytes (used by DOM-based drag-drop on Windows
/// where Tauri native file paths are unavailable).
#[tauri::command]
pub async fn import_file_bytes(
    bytes: Vec<u8>,
    name: String,
    folder_id: i64,
    folder_path: String,
) -> Result<Option<AFile>, String> {
    if bytes.is_empty() {
        return Err("Dropped file is empty".to_string());
    }
    let dest_folder = folder_path.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let new_path = t_utils::save_bytes_with_name(&bytes, &name, &dest_folder)
            .ok_or_else(|| "Failed to save dropped file".to_string())?;
        let file_type = t_utils::get_file_type(&new_path).ok_or_else(|| {
            let _ = std::fs::remove_file(&new_path);
            format!("Unsupported file type: {}", new_path)
        })?;
        let now = chrono::Utc::now().timestamp_millis();
        let (file, _) = AFile::add_to_db(folder_id, &new_path, file_type, now)?;
        Ok(Some(file))
    })
    .await
    .map_err(|e| format!("Failed to save file: {}", e))?
}

fn import_clipboard_file(
    file_path: &Path,
    folder_id: i64,
    folder_path: &str,
) -> Result<AFile, String> {
    let source = file_path
        .to_str()
        .ok_or_else(|| format!("Invalid clipboard file path: {}", file_path.display()))?;
    t_utils::get_file_type(source)
        .ok_or_else(|| format!("Unsupported file type: {}", file_path.display()))?;
    let new_path = t_utils::import_file(source, folder_path)
        .ok_or_else(|| format!("Failed to copy file: {}", file_path.display()))?;
    let file_type = t_utils::get_file_type(&new_path).ok_or_else(|| {
        let _ = std::fs::remove_file(&new_path);
        format!("Unsupported file type after copy: {}", new_path)
    })?;
    let now = chrono::Utc::now().timestamp_millis();
    match AFile::add_to_db(folder_id, &new_path, file_type, now) {
        Ok((file, _)) => Ok(file),
        Err(error) => {
            let _ = std::fs::remove_file(&new_path);
            Err(error)
        }
    }
}

#[tauri::command]
pub async fn has_importable_clipboard(_app_handle: tauri::AppHandle) -> bool {
    #[cfg(target_os = "linux")]
    {
        let Ok(clipboard_data) = crate::t_pasteboard::get_clipboard_import_data(&_app_handle).await
        else {
            return false;
        };

        if clipboard_data.file_paths.iter().any(|path| {
            std::path::Path::new(path).is_file() && t_utils::get_file_type(path).is_some()
        }) {
            return true;
        }

        if clipboard_data.png.is_some() {
            return true;
        }
    }

    let Ok(mut clipboard) = arboard::Clipboard::new() else {
        return false;
    };

    if let Ok(file_paths) = clipboard.get().file_list() {
        if file_paths
            .iter()
            .any(|path| path.is_file() && path.to_str().and_then(t_utils::get_file_type).is_some())
        {
            return true;
        }
    }

    clipboard.get_image().is_ok()
}

/// Import copied image files while preserving their original format and
/// metadata. Fall back to a PNG only when the clipboard contains pixels
/// without a backing file, such as a screenshot.
#[tauri::command]
pub async fn import_clipboard(
    _app_handle: tauri::AppHandle,
    folder_id: i64,
    folder_path: &str,
) -> Result<Vec<AFile>, String> {
    #[cfg(target_os = "linux")]
    {
        let clipboard_data = crate::t_pasteboard::get_clipboard_import_data(&_app_handle).await?;
        if !clipboard_data.file_paths.is_empty() {
            let mut imported = Vec::new();
            for path in clipboard_data
                .file_paths
                .iter()
                .map(std::path::Path::new)
                .filter(|path| path.is_file())
            {
                let supported = path.to_str().and_then(t_utils::get_file_type).is_some();
                if !supported {
                    continue;
                }
                match import_clipboard_file(path, folder_id, folder_path) {
                    Ok(file) => imported.push(file),
                    Err(error) => eprintln!(
                        "Failed to import clipboard file {}: {}",
                        path.display(),
                        error
                    ),
                }
            }
            if !imported.is_empty() {
                return Ok(imported);
            }
        }

        if let Some(png) = clipboard_data.png {
            let new_path = t_utils::save_bytes_to_folder(&png, "image/png", folder_path)
                .ok_or_else(|| "Failed to save clipboard image".to_string())?;
            let file_type = t_utils::get_file_type(&new_path).ok_or_else(|| {
                let _ = std::fs::remove_file(&new_path);
                format!("Unsupported file type: {}", new_path)
            })?;
            let now = chrono::Utc::now().timestamp_millis();
            return match AFile::add_to_db(folder_id, &new_path, file_type, now) {
                Ok((file, _)) => Ok(vec![file]),
                Err(error) => {
                    let _ = std::fs::remove_file(&new_path);
                    Err(error)
                }
            };
        }

        if !clipboard_data.file_paths.is_empty() {
            return Err("Clipboard does not contain supported image files".to_string());
        }
    }

    let mut clipboard =
        arboard::Clipboard::new().map_err(|e| format!("Failed to open clipboard: {}", e))?;
    if let Ok(file_paths) = clipboard.get().file_list() {
        if !file_paths.is_empty() {
            let mut imported = Vec::new();
            for path in file_paths.iter().filter(|path| path.is_file()) {
                let supported = path.to_str().and_then(t_utils::get_file_type).is_some();
                if !supported {
                    continue;
                }
                match import_clipboard_file(path, folder_id, folder_path) {
                    Ok(file) => imported.push(file),
                    Err(error) => eprintln!(
                        "Failed to import clipboard file {}: {}",
                        path.display(),
                        error
                    ),
                }
            }
            if !imported.is_empty() {
                return Ok(imported);
            }
            return Err("Clipboard does not contain supported image files".to_string());
        }
    }

    let clipboard_image = clipboard
        .get_image()
        .map_err(|e| format!("No image found in clipboard: {}", e))?;
    let width = u32::try_from(clipboard_image.width)
        .map_err(|_| "Clipboard image width is too large".to_string())?;
    let height = u32::try_from(clipboard_image.height)
        .map_err(|_| "Clipboard image height is too large".to_string())?;
    let rgba = image::RgbaImage::from_raw(width, height, clipboard_image.bytes.into_owned())
        .ok_or_else(|| "Clipboard image data is invalid".to_string())?;

    let mut bytes = Cursor::new(Vec::new());
    image::DynamicImage::ImageRgba8(rgba)
        .write_to(&mut bytes, image::ImageFormat::Png)
        .map_err(|e| format!("Failed to encode clipboard image: {}", e))?;
    let new_path = t_utils::save_bytes_to_folder(bytes.get_ref(), "image/png", folder_path)
        .ok_or_else(|| "Failed to save clipboard image".to_string())?;
    let file_type = t_utils::get_file_type(&new_path).ok_or_else(|| {
        let _ = std::fs::remove_file(&new_path);
        format!("Unsupported file type: {}", new_path)
    })?;
    let now = chrono::Utc::now().timestamp_millis();
    match AFile::add_to_db(folder_id, &new_path, file_type, now) {
        Ok((file, _)) => Ok(vec![file]),
        Err(error) => {
            let _ = std::fs::remove_file(&new_path);
            Err(error)
        }
    }
}

/// delete a file
#[tauri::command]
pub fn delete_file(file_id: i64, file_path: &str) -> Result<usize, String> {
    // trash the file
    t_utils::trash_path(file_path)?;

    // delete the file from db
    AFile::delete(file_id).map_err(|e| format!("Error while deleting file from DB: {}", e))
}

/// delete a file permanently
#[tauri::command]
pub fn delete_file_permanently(file_id: i64, file_path: &str) -> Result<usize, String> {
    // delete the file from disk first
    t_utils::delete_file_permanently(file_path)?;

    // delete the file from db
    AFile::delete(file_id).map_err(|e| format!("Error while deleting file from DB: {}", e))
}

/// delete a file from db
#[tauri::command]
pub fn delete_db_file(file_id: i64) -> Result<usize, String> {
    // delete the file from db
    AFile::delete(file_id).map_err(|e| format!("Error while deleting file from DB: {}", e))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchDeleteFile {
    pub file_id: i64,
    pub file_path: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchDeleteResult {
    pub deleted_file_ids: Vec<i64>,
    pub failed_count: usize,
}

#[tauri::command]
pub async fn batch_delete_files(
    files: Vec<BatchDeleteFile>,
    permanently: bool,
) -> Result<BatchDeleteResult, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let mut deleted_file_ids = Vec::with_capacity(files.len());
        for file in &files {
            let result = if permanently {
                t_utils::delete_file_permanently(&file.file_path)
            } else {
                t_utils::trash_path(&file.file_path)
            };
            if result.is_ok() {
                deleted_file_ids.push(file.file_id);
            }
        }

        AFile::batch_delete(&deleted_file_ids)
            .map_err(|e| format!("Error while deleting files from DB: {}", e))?;
        Ok(BatchDeleteResult {
            failed_count: files.len().saturating_sub(deleted_file_ids.len()),
            deleted_file_ids,
        })
    })
    .await
    .map_err(|e| format!("Failed to run batch delete: {}", e))?
}

/// edit a file's comment
#[tauri::command]
pub fn edit_file_comment(file_id: i64, comment: &str) -> Result<usize, String> {
    AFile::update_column(file_id, "comments", &comment)
        .map_err(|e| format!("Error while editing file comment: {}", e))
}

/// get a file's thumb image, if not exist, create a new one
#[tauri::command]
pub async fn get_file_thumb(
    app_handle: tauri::AppHandle,
    file_id: i64,
    file_path: &str,
    file_type: i64,
    orientation: i32,
    thumbnail_size: u32,
    force_regenerate: bool,
    thumbnail_seek_percent: Option<u8>,
) -> Result<Option<AThumb>, String> {
    if let Some(thumb) = AThumb::get_thumb_if_available(
        file_id,
        file_path,
        thumbnail_size,
        orientation,
        force_regenerate,
    )
    .map_err(|e| format!("Error while getting thumbnail: {}", e))?
    {
        return Ok(Some(thumb));
    }

    let album_id = AFile::get_file_info(file_id)
        .map_err(|e| format!("Error while getting file info for thumbnail: {}", e))?
        .and_then(|file| file.album_id)
        .unwrap_or(0);

    AThumb::schedule_background_generation_for_library(
        app_handle,
        file_id,
        file_path.to_string(),
        file_type,
        orientation,
        thumbnail_size,
        album_id,
        force_regenerate,
        thumbnail_seek_percent,
    );

    Ok(None)
}

/// get a file's thumb image by id, if not exist, create a new one in background
#[tauri::command]
pub async fn get_file_thumb_by_id(
    app_handle: tauri::AppHandle,
    file_id: i64,
    thumbnail_size: u32,
    force_regenerate: bool,
) -> Result<Option<AThumb>, String> {
    let Some(file) = AFile::get_file_info(file_id)
        .map_err(|e| format!("Error while getting file info for thumbnail: {}", e))?
    else {
        return Ok(None);
    };

    let Some(file_path) = file.file_path.clone() else {
        return Ok(None);
    };

    let file_type = file.file_type.unwrap_or(0);
    let orientation = file.e_orientation.unwrap_or(1) as i32;

    if let Some(thumb) = AThumb::get_thumb_if_available(
        file_id,
        &file_path,
        thumbnail_size,
        orientation,
        force_regenerate,
    )
    .map_err(|e| format!("Error while getting thumbnail: {}", e))?
    {
        return Ok(Some(thumb));
    }

    AThumb::schedule_background_generation_for_library(
        app_handle,
        file_id,
        file_path,
        file_type,
        orientation,
        thumbnail_size,
        file.album_id.unwrap_or(0),
        force_regenerate,
        None,
    );

    Ok(None)
}

/// get multiple thumbnails in one IPC call; missing thumbnails are generated in background
#[tauri::command]
pub async fn get_file_thumbs(
    app_handle: tauri::AppHandle,
    files: Vec<ThumbRequest>,
    thumbnail_size: u32,
    force_regenerate: bool,
) -> Result<Vec<Option<AThumb>>, String> {
    let mut thumbs = Vec::with_capacity(files.len());
    let file_ids: Vec<i64> = files
        .iter()
        .map(|request| request.file_id)
        .filter(|file_id| *file_id > 0)
        .collect();
    let mut fetched_thumbs = if force_regenerate {
        HashMap::new()
    } else {
        AThumb::fetch_many(&file_ids)
            .map_err(|e| format!("Error while fetching thumbnails: {}", e))?
    };

    for request in files {
        if request.file_id <= 0 {
            thumbs.push(None);
            continue;
        }

        let mut file_path = request.file_path;
        let mut file_type = request.file_type.unwrap_or(0);
        let mut orientation = request.orientation.unwrap_or(1);
        let mut album_id = request.album_id.unwrap_or(0);

        if file_path.is_none()
            || request.file_type.is_none()
            || request.orientation.is_none()
            || album_id <= 0
        {
            if let Some(file) = AFile::get_file_info(request.file_id)
                .map_err(|e| format!("Error while getting file info for thumbnail: {}", e))?
            {
                if file_path.is_none() {
                    file_path = file.file_path;
                }
                if request.file_type.is_none() {
                    file_type = file.file_type.unwrap_or(0);
                }
                if request.orientation.is_none() {
                    orientation = file.e_orientation.unwrap_or(1) as i32;
                }
                if album_id <= 0 {
                    album_id = file.album_id.unwrap_or(0);
                }
            }
        }

        let Some(file_path) = file_path else {
            thumbs.push(None);
            continue;
        };

        if let Some(fetched_thumb) = fetched_thumbs.remove(&request.file_id) {
            if let Some(thumb) = AThumb::resolve_fetched_thumb_if_available(
                fetched_thumb,
                &file_path,
                thumbnail_size,
                orientation,
                force_regenerate,
            )
            .map_err(|e| format!("Error while getting thumbnail: {}", e))?
            {
                thumbs.push(Some(thumb));
                continue;
            }
        }

        AThumb::schedule_background_generation_for_library(
            app_handle.clone(),
            request.file_id,
            file_path,
            file_type,
            orientation,
            thumbnail_size,
            album_id,
            force_regenerate,
            None,
        );

        thumbs.push(None);
    }

    Ok(thumbs)
}

/// get a file's info
#[tauri::command]
pub fn get_file_info(file_id: i64) -> Result<Option<AFile>, String> {
    AFile::get_file_info(file_id).map_err(|e| format!("Error while getting file info: {}", e))
}

/// update a file's info
#[tauri::command]
pub fn update_file_info(file_id: i64, file_path: &str) -> Result<Option<AFile>, String> {
    let now = chrono::Utc::now().timestamp_millis();
    AFile::update_file_info(file_id, file_path, now)
        .map_err(|e| format!("Error while updating file info: {}", e))
}

/// add or refresh a file in db and return the indexed file info
#[tauri::command]
pub fn add_file_to_db(folder_id: i64, file_path: &str) -> Result<Option<AFile>, String> {
    let file_type = t_utils::get_file_type(file_path)
        .ok_or_else(|| format!("Unsupported file type: {}", file_path))?;
    let now = chrono::Utc::now().timestamp_millis();
    let (file, _) = AFile::add_to_db(folder_id, file_path, file_type, now)?;
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
    AFolder::update_column(folder_id, "is_favorite", &is_favorite)
        .map_err(|e| format!("Error while setting folder favorite: {}", e))
}

/// get a folder's search exclusion status (true or false)
#[tauri::command]
pub fn get_folder_search_excluded(folder_path: &str) -> Result<bool, String> {
    let is_excluded_opt = AFolder::get_is_excluded_from_search(folder_path)
        .map_err(|e| format!("Error while getting folder search exclusion: {}", e))?;

    match is_excluded_opt {
        Some(is_excluded) => Ok(is_excluded),
        None => Ok(false),
    }
}

/// set a folder's search exclusion status (true or false)
#[tauri::command]
pub fn set_folder_search_excluded(
    album_id: i64,
    folder_path: &str,
    is_excluded: bool,
) -> Result<usize, String> {
    let folder = AFolder::add_to_db(album_id, folder_path)
        .map_err(|e| format!("Error while ensuring folder in DB: {}", e))?;
    let folder_id = folder
        .id
        .ok_or_else(|| "Folder was saved without an id".to_string())?;
    AFolder::update_column(folder_id, "is_excluded_from_search", &is_excluded)
        .map_err(|e| format!("Error while setting folder search exclusion: {}", e))
}

/// set a file's favorite status (true or false)
#[tauri::command]
pub fn set_file_favorite(file_id: i64, is_favorite: bool) -> Result<usize, String> {
    AFile::update_column(file_id, "is_favorite", &is_favorite)
        .map_err(|e| format!("Error while setting file favorite: {}", e))
}

/// set a file's rating (0-5)
#[tauri::command]
pub fn set_file_rating(file_id: i64, rating: i32) -> Result<usize, String> {
    let clamped = rating.clamp(0, 5);
    AFile::update_column(file_id, "rating", &clamped)
        .map_err(|e| format!("Error while setting file rating: {}", e))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchFileMetadataUpdate {
    pub file_ids: Vec<i64>,
    pub is_favorite: Option<bool>,
    pub rating: Option<i32>,
    pub rotate_delta: Option<i32>,
    pub comment: Option<String>,
}

#[tauri::command]
pub fn batch_update_file_metadata(params: BatchFileMetadataUpdate) -> Result<usize, String> {
    AFile::batch_update_metadata(
        &params.file_ids,
        params.is_favorite,
        params.rating,
        params.rotate_delta,
        params.comment.as_deref(),
    )
    .map_err(|e| format!("Error while updating file metadata: {}", e))
}

// tag

/// get all tags
#[tauri::command]
pub fn get_all_tags(sort: i64) -> Result<Vec<ATag>, String> {
    ATag::get_all(sort).map_err(|e| format!("Error while getting all tags: {}", e))
}

/// get tag name by id
#[tauri::command]
pub fn get_tag_name(tag_id: i64) -> Result<String, String> {
    ATag::get_name(tag_id).map_err(|e| format!("Error while getting tag name: {}", e))
}

/// create a new tag
#[tauri::command]
pub fn create_tag(name: &str) -> Result<ATag, String> {
    ATag::add(name).map_err(|e| format!("Error while creating tag: {}", e))
}

/// rename a tag
#[tauri::command]
pub fn rename_tag(tag_id: i64, new_name: &str) -> Result<usize, String> {
    ATag::rename(tag_id, new_name).map_err(|e| format!("Error while renaming tag: {}", e))
}

/// delete a tag
#[tauri::command]
pub fn delete_tag(tag_id: i64) -> Result<usize, String> {
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
    ATag::add_tag_to_file(file_id, tag_id)
        .map_err(|e| format!("Error while adding tag to file: {}", e))
}

/// remove a tag from a file
#[tauri::command]
pub fn remove_tag_from_file(file_id: i64, tag_id: i64) -> Result<usize, String> {
    ATag::remove_tag_from_file(file_id, tag_id)
        .map_err(|e| format!("Error while removing tag from file: {}", e))
}

#[tauri::command]
pub fn get_tag_selection_counts(file_ids: Vec<i64>) -> Result<Vec<ATagSelectionCount>, String> {
    ATag::get_selection_counts(&file_ids)
        .map_err(|e| format!("Error while getting tag selection counts: {}", e))
}

#[tauri::command]
pub fn apply_tags_to_files(
    file_ids: Vec<i64>,
    add_tag_ids: Vec<i64>,
    remove_tag_ids: Vec<i64>,
) -> Result<Vec<ATagFileState>, String> {
    ATag::apply_to_files(&file_ids, &add_tag_ids, &remove_tag_ids)
        .map_err(|e| format!("Error while applying tags to files: {}", e))
}

// calendar

/// get camera's taken dates
#[tauri::command]
pub fn get_taken_dates(sort: i64) -> Result<Vec<(String, i64)>, String> {
    AFile::get_taken_dates(sort).map_err(|e| format!("Error while getting taken dates: {}", e))
}

// camera

/// get a file's camera make and model info
#[tauri::command]
pub fn get_camera_info(sort: i64) -> Result<Vec<ACamera>, String> {
    ACamera::get_from_db(sort).map_err(|e| format!("Error while getting camera info: {}", e))
}

/// get a file's lens make and model info
#[tauri::command]
pub fn get_lens_info(sort: i64) -> Result<Vec<ALens>, String> {
    ALens::get_from_db(sort).map_err(|e| format!("Error while getting lens info: {}", e))
}

// location

/// get a file's location info
#[tauri::command]
pub fn get_location_info(sort: i64) -> Result<Vec<ALocation>, String> {
    ALocation::get_from_db(sort).map_err(|e| format!("Error while getting location info: {}", e))
}

/// get GPS coordinates aggregated into grid cells for heatmap rendering
#[tauri::command]
pub fn get_gps_heatmap_points() -> Result<Vec<t_sqlite::AGpsHeatPoint>, String> {
    t_sqlite::AGpsHeatPoint::get_heatmap_from_db()
        .map_err(|e| format!("Error while getting GPS heatmap points: {}", e))
}

// settings

/// get package info
#[tauri::command]
pub fn get_package_info() -> t_utils::PackageInfo {
    t_utils::PackageInfo::new(GIT_COMMIT_HASH)
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
    let db_file_path = t_storage::get_current_db_path()
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

#[tauri::command]
pub fn get_image_search_model_status(
    app_handle: AppHandle,
    state: State<t_ai::AiState>,
) -> t_ai::ImageSearchModelStatus {
    let ai_engine = state.0.lock().unwrap();
    ai_engine.model_status(&app_handle)
}

#[tauri::command]
pub async fn set_image_search_model(
    app_handle: AppHandle,
    state: State<'_, t_ai::AiState>,
    model: i64,
) -> Result<t_ai::ImageSearchModelStatus, String> {
    let mut ai_engine = state.0.lock().unwrap();
    ai_engine.set_text_model(&app_handle, t_ai::ImageSearchTextModel::from_i64(model))?;
    Ok(ai_engine.model_status(&app_handle))
}

#[tauri::command]
pub async fn download_multilingual_image_search_model(app_handle: AppHandle) -> Result<(), String> {
    t_ai::download_multilingual_text_model(app_handle).await
}

#[tauri::command]
pub async fn cancel_multilingual_image_search_model_download(
    app_handle: AppHandle,
) -> Result<(), String> {
    t_ai::cancel_multilingual_text_model_download(app_handle).await
}

/// generate embedding for a file
#[tauri::command]
pub fn generate_embedding(state: State<t_ai::AiState>, file_id: i64) -> Result<String, String> {
    AFile::generate_embedding(&state, file_id)
}

// search similar images
#[tauri::command]
pub async fn search_similar_images(
    state: State<'_, t_ai::AiState>,
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
pub fn get_persons(sort: i64) -> Result<Vec<Person>, String> {
    Person::get_all(sort).map_err(|e| format!("Error while getting persons: {}", e))
}

/// rename a person
#[tauri::command]
pub fn rename_person(person_id: i64, name: String) -> Result<usize, String> {
    Person::rename(person_id, &name).map_err(|e| format!("Error while renaming person: {}", e))
}

/// delete a person
#[tauri::command]
pub fn delete_person(person_id: i64) -> Result<usize, String> {
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
    collection_id: Option<i64>,
    file_ids: Option<Vec<i64>>,
) -> Result<(), String> {
    crate::t_dedup::start_scan(app_handle, state, params, collection_id, file_ids)
}

#[tauri::command]
pub fn dedup_get_scan_status(
    state: tauri::State<'_, crate::t_dedup::DedupState>,
) -> Result<crate::t_dedup::DedupScanStatus, String> {
    let mut status = state.status.lock().unwrap().clone();
    status.is_scanning = state.is_scanning.load(std::sync::atomic::Ordering::SeqCst);
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
pub fn dedup_set_keep(group_id: i64, file_id: i64) -> Result<(), String> {
    crate::t_dedup::set_keep(group_id, file_id)
}

#[tauri::command]
pub fn dedup_delete_selected(
    group_ids: Option<Vec<i64>>,
    file_ids: Option<Vec<i64>>,
) -> Result<crate::t_dedup::DedupDeleteResult, String> {
    crate::t_dedup::delete_selected(group_ids, file_ids)
}

// ----------------------------------------------------------------------------
// Backup / Restore Commands
// ----------------------------------------------------------------------------

#[tauri::command]
pub fn get_db_storage_info() -> Result<Vec<t_storage::DbStorageInfo>, String> {
    t_storage::get_db_storage_info()
}

#[tauri::command]
pub fn backup_databases(
    library_ids: Vec<String>,
    dest_path: String,
) -> Result<t_storage::BackupResult, String> {
    t_storage::backup_databases(&library_ids, &dest_path)
}

#[tauri::command]
pub fn parse_backup_file(path: String) -> Result<t_storage::BackupMetaData, String> {
    t_storage::parse_backup_file(&path)
}

#[tauri::command]
pub fn restore_databases(
    backup_path: String,
    selections: Vec<t_storage::RestoreSelection>,
) -> Result<t_storage::RestoreResult, String> {
    t_storage::restore_databases(&backup_path, &selections)
}

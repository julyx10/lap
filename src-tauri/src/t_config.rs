/**
 * App configuration management.
 * Handles app-config.json for multi-library support.
 * project: Lap
 * author:  julyx10
 * date:    2026-01-15
 */
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use uuid::Uuid;
use crate::t_storage;

static APP_IDENTIFIER: OnceLock<String> = OnceLock::new();
static CONFIG_IO_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

pub fn set_app_identifier(identifier: &str) {
    let _ = APP_IDENTIFIER.set(identifier.to_string());
}

fn config_io_lock() -> &'static Mutex<()> {
    CONFIG_IO_LOCK.get_or_init(|| Mutex::new(()))
}

// ============================================================================
// LibraryState sub-structs for per-library config persistence
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AlbumState {
    pub id: i64,
    #[serde(alias = "folder_id")]
    pub folder_id: Option<i64>,
    #[serde(alias = "folder_path")]
    pub folder_path: String,
    pub selected: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FavoriteState {
    #[serde(default = "default_favorite_tab")]
    pub tab: String,
    #[serde(alias = "album_id")]
    pub album_id: Option<i64>,
    #[serde(alias = "folder_id")]
    pub folder_id: i64,
    #[serde(alias = "folder_path")]
    pub folder_path: Option<String>,
    #[serde(default)]
    pub rating: Option<i32>,
}

fn default_favorite_tab() -> String {
    "favorite".to_string()
}

impl Default for FavoriteState {
    fn default() -> Self {
        Self {
            tab: default_favorite_tab(),
            album_id: None,
            folder_id: 0,
            folder_path: None,
            rating: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TagState {
    pub id: Option<i64>,
    pub smart_id: Option<String>,
    #[serde(default = "default_tag_tab")]
    pub tab: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CalendarState {
    pub year: Option<i32>,
    pub month: Option<i32>,
    pub date: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CameraState {
    #[serde(default = "default_camera_tab")]
    pub tab: String,
    pub make: Option<String>,
    pub model: Option<String>,
    pub lens_make: Option<String>,
    pub lens_model: Option<String>,
}

fn default_camera_tab() -> String {
    "camera".to_string()
}

fn default_tag_tab() -> String {
    "custom".to_string()
}

impl Default for TagState {
    fn default() -> Self {
        Self {
            id: None,
            smart_id: None,
            tab: default_tag_tab(),
        }
    }
}

impl Default for CameraState {
    fn default() -> Self {
        Self {
            tab: default_camera_tab(),
            make: None,
            model: None,
            lens_make: None,
            lens_model: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct LocationState {
    pub cc: Option<String>,
    pub admin1: Option<String>,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SearchHistoryEntry {
    Legacy(String),
    Rich(SearchHistoryItem),
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SearchHistoryItem {
    pub text: String,
    pub file_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SearchState {
    #[serde(default, alias = "search_type")]
    pub search_type: i32,
    #[serde(alias = "search_text")]
    pub search_text: String,
    #[serde(alias = "search_history")]
    pub search_history: Vec<SearchHistoryEntry>,
    #[serde(alias = "search_history_index")]
    pub search_history_index: i32,
    #[serde(alias = "similar_image_history")]
    pub similar_image_history: Vec<i64>,
    #[serde(alias = "similar_image_history_index")]
    pub similar_image_history_index: i32,
    #[serde(alias = "file_name")]
    pub file_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct DestFolderState {
    #[serde(alias = "album_id")]
    pub album_id: Option<i64>,
    #[serde(alias = "folder_id")]
    pub folder_id: Option<i64>,
    #[serde(alias = "folder_path")]
    pub folder_path: Option<String>,
    pub selected: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct IndexState {
    pub status: i32,
    #[serde(alias = "album_queue")]
    pub album_queue: Vec<i64>,
    #[serde(alias = "paused_album_ids", default)]
    pub paused_album_ids: Vec<i64>,
    #[serde(alias = "album_name")]
    pub album_name: String,
    pub indexed: i64,
    pub total: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PersonState {
    pub id: Option<i64>,
    pub name: Option<String>,
}

/// Per-library state that persists across sessions
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct LibraryState {
    pub album: AlbumState,
    pub favorite: FavoriteState,
    pub tag: TagState,
    pub calendar: CalendarState,
    pub camera: CameraState,
    pub location: LocationState,
    #[serde(default)]
    pub person: PersonState,
    pub search: SearchState,
    #[serde(alias = "dest_folder")]
    pub dest_folder: DestFolderState,
    pub index: IndexState,
}

// ============================================================================
// Library and AppConfig structs
// ============================================================================

/// Library entry in the config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Library {
    pub id: String,
    pub name: String,
    pub created_at: i64,
    #[serde(default)]
    pub state: LibraryState,
    #[serde(default)]
    pub hidden: bool,
}

/// App configuration stored in app-config.json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default)]
    pub debug: bool,
    #[serde(default = "default_last_selected_item_index")]
    pub last_selected_item_index: i64,
    #[serde(default)]
    pub db_storage_dir: Option<String>,
    pub current_library_id: String,
    pub libraries: Vec<Library>,
}

fn default_last_selected_item_index() -> i64 {
    -1
}

impl Default for AppConfig {
    fn default() -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            debug: false,
            last_selected_item_index: default_last_selected_item_index(),
            db_storage_dir: None,
            current_library_id: "default".to_string(),
            libraries: vec![Library {
                id: "default".to_string(),
                name: "Default Library".to_string(),
                created_at: now,
                state: LibraryState::default(),
                hidden: false,
            }],
        }
    }
}

/// Get the AppData directory for app
pub fn get_app_data_dir() -> Result<PathBuf, String> {
    let app_dir_name = get_app_data_folder_name();
    dirs::data_local_dir()
        .ok_or_else(|| "Failed to get the local AppData directory".to_string())
        .map(|p| p.join(app_dir_name))
}

fn get_app_data_folder_name() -> String {
    let identifier = APP_IDENTIFIER
        .get()
        .cloned()
        .unwrap_or_else(|| "com.julyx10.lap".to_string());

    if cfg!(debug_assertions) {
        format!("{}.debug", identifier)
    } else {
        identifier
    }
}

/// Get the cache directory for app-managed temporary data.
/// macos: ~/Library/Caches/com.julyx10.lap
/// windows: C:\Users\<username>\AppData\Local\com.julyx10.lap\cache
/// linux: ~/.cache/com.julyx10.lap
pub fn get_app_cache_dir() -> Result<PathBuf, String> {
    let app_dir_name = get_app_data_folder_name();
    dirs::cache_dir()
        .ok_or_else(|| "Failed to get the local cache directory".to_string())
        .map(|p| p.join(app_dir_name))
}

/// Get the libraries directory path
pub fn get_libraries_dir() -> Result<PathBuf, String> {
    let app_dir = get_app_data_dir()?;
    let lib_dir = app_dir.join("libraries");
    fs::create_dir_all(&lib_dir)
        .map_err(|e| format!("Failed to create libraries directory: {}", e))?;
    Ok(lib_dir)
}

/// Get the app-config.json file path
pub fn get_config_file_path() -> Result<PathBuf, String> {
    let app_dir = get_app_data_dir()?;
    fs::create_dir_all(&app_dir)
        .map_err(|e| format!("Failed to create AppData directory: {}", e))?;
    Ok(app_dir.join("app-config.json"))
}

/// Load app config from file, create default if not exists
pub fn load_app_config() -> Result<AppConfig, String> {
    let _guard = config_io_lock()
        .lock()
        .map_err(|_| "Config lock poisoned".to_string())?;
    load_app_config_locked()
}

fn load_app_config_locked() -> Result<AppConfig, String> {
    let config_path = get_config_file_path()?;

    if config_path.exists() {
        // Retry briefly in case another process is atomically replacing the file.
        let mut last_err: Option<String> = None;
        for _ in 0..3 {
            let content = fs::read_to_string(&config_path)
                .map_err(|e| format!("Failed to read config file: {}", e))?;

            match serde_json::from_str::<AppConfig>(&content) {
                Ok(config) => return Ok(config),
                Err(parse_err) => {
                    // Handle rare corruption patterns (e.g. concatenated JSON objects)
                    if let Some(config) = parse_first_json_object::<AppConfig>(&content) {
                        let _ = save_app_config_locked(&config);
                        return Ok(config);
                    }

                    last_err = Some(format!("Failed to parse config file: {}", parse_err));
                    thread::sleep(Duration::from_millis(30));
                }
            }
        }

        // Keep the bad file for diagnosis and recover as much as possible.
        let backup_msg = backup_corrupt_config(&config_path)
            .map(|p| format!(" Backed up to '{}'.", p.display()))
            .unwrap_or_else(|e| format!(" Backup failed: {}.", e));
        let parse_msg = last_err.unwrap_or_else(|| "Failed to parse config file".to_string());

        match recover_app_config_from_library_dbs() {
            Ok(recovered) => {
                eprintln!(
                    "{}{} Recovered app config from existing library database files.",
                    parse_msg, backup_msg
                );
                save_app_config_locked(&recovered)?;
                Ok(recovered)
            }
            Err(recover_err) => {
                eprintln!(
                    "{}{} Failed to recover from library DB files: {}. Falling back to default config.",
                    parse_msg, backup_msg, recover_err
                );
                let config = AppConfig::default();
                save_app_config_locked(&config)?;
                Ok(config)
            }
        }
    } else {
        // Create default config
        let config = AppConfig::default();
        save_app_config_locked(&config)?;
        Ok(config)
    }
}

/// Save app config to file
pub fn save_app_config(config: &AppConfig) -> Result<(), String> {
    let _guard = config_io_lock()
        .lock()
        .map_err(|_| "Config lock poisoned".to_string())?;
    save_app_config_locked(config)
}

fn save_app_config_locked(config: &AppConfig) -> Result<(), String> {
    let config_path = get_config_file_path()?;
    let content = serde_json::to_string_pretty(config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;
    write_atomic(&config_path, &content)
        .map_err(|e| format!("Failed to write config file: {}", e))?;
    Ok(())
}

fn parse_first_json_object<T: for<'de> Deserialize<'de>>(content: &str) -> Option<T> {
    let mut stream = serde_json::Deserializer::from_str(content).into_iter::<T>();
    match stream.next() {
        Some(Ok(value)) => Some(value),
        _ => None,
    }
}

fn write_atomic(path: &Path, content: &str) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| "Config path has no parent directory".to_string())?;
    fs::create_dir_all(parent).map_err(|e| format!("Failed to create parent directory: {}", e))?;

    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| format!("Failed to get timestamp: {}", e))?
        .as_nanos();
    let tmp_path = parent.join(format!(
        ".{}.tmp-{}-{}",
        path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("app-config.json"),
        std::process::id(),
        stamp
    ));

    let mut tmp_file =
        fs::File::create(&tmp_path).map_err(|e| format!("Failed to create temp file: {}", e))?;
    tmp_file
        .write_all(content.as_bytes())
        .map_err(|e| format!("Failed to write temp file: {}", e))?;
    tmp_file
        .sync_all()
        .map_err(|e| format!("Failed to sync temp config file: {}", e))?;
    drop(tmp_file);

    match fs::rename(&tmp_path, path) {
        Ok(_) => {
            // Best-effort directory sync to persist rename metadata.
            if let Ok(dir_file) = fs::File::open(parent) {
                let _ = dir_file.sync_all();
            }
            Ok(())
        }
        Err(rename_err) => {
            // Windows fallback: rename may fail when target exists.
            if path.exists() {
                let _ = fs::remove_file(path);
                if fs::rename(&tmp_path, path).is_ok() {
                    if let Ok(dir_file) = fs::File::open(parent) {
                        let _ = dir_file.sync_all();
                    }
                    return Ok(());
                }
            }
            let _ = fs::remove_file(&tmp_path);
            Err(format!(
                "Failed to atomically replace config file: {}",
                rename_err
            ))
        }
    }
}

fn backup_corrupt_config(path: &Path) -> Result<PathBuf, String> {
    let parent = path
        .parent()
        .ok_or_else(|| "Config path has no parent directory".to_string())?;
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("app-config");
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| format!("Failed to get timestamp: {}", e))?
        .as_secs();
    let backup_path = parent.join(format!("{}-corrupt-{}.json", stem, stamp));
    fs::rename(path, &backup_path)
        .or_else(|_| fs::copy(path, &backup_path).map(|_| ()))
        .map_err(|e| format!("Failed to backup corrupted config file: {}", e))?;
    Ok(backup_path)
}

fn recover_app_config_from_library_dbs() -> Result<AppConfig, String> {
    let lib_dir = get_libraries_dir()?;
    let read_dir = fs::read_dir(&lib_dir).map_err(|e| {
        format!(
            "Failed to read libraries directory '{}': {}",
            lib_dir.display(),
            e
        )
    })?;

    let mut libraries: Vec<Library> = Vec::new();
    let now = chrono::Utc::now().timestamp();

    for entry in read_dir {
        let entry =
            entry.map_err(|e| format!("Failed to read libraries directory entry: {}", e))?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("db") {
            continue;
        }

        let Some(id) = path.file_stem().and_then(|s| s.to_str()) else {
            continue;
        };
        if id.trim().is_empty() {
            continue;
        }

        let default_name = if id == "default" {
            "Default Library".to_string()
        } else {
            format!("Library {}", &id.chars().take(8).collect::<String>())
        };
        let created_at = fs::metadata(&path)
            .ok()
            .and_then(|m| m.created().or_else(|_| m.modified()).ok())
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_secs() as i64)
            .unwrap_or(now);

        libraries.push(Library {
            id: id.to_string(),
            name: default_name,
            created_at,
            state: LibraryState::default(),
            hidden: false,
        });
    }

    if libraries.is_empty() {
        return Err("No library database files found".to_string());
    }

    libraries.sort_by(|a, b| a.created_at.cmp(&b.created_at));

    let current_library_id = if libraries.iter().any(|l| l.id == "default") {
        "default".to_string()
    } else {
        libraries[0].id.clone()
    };

    Ok(AppConfig {
        debug: false,
        last_selected_item_index: default_last_selected_item_index(),
        db_storage_dir: None,
        current_library_id,
        libraries,
    })
}

/// Add a new library
pub fn add_library(name: &str) -> Result<Library, String> {
    let mut config = load_app_config()?;

    // Check for duplicate names
    if config.libraries.iter().any(|l| l.name == name) {
        return Err("Library name already exists".to_string());
    }

    // Generate unique ID
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().timestamp();

    let library = Library {
        id: id.clone(),
        name: name.to_string(),
        created_at: now,

        state: LibraryState::default(),
        hidden: false,
    };

    config.libraries.push(library.clone());
    save_app_config(&config)?;

    Ok(library)
}

/// Edit library name
pub fn edit_library(id: &str, new_name: &str) -> Result<(), String> {
    let mut config = load_app_config()?;

    // Check for duplicate names (excluding current library)
    if config
        .libraries
        .iter()
        .any(|l| l.name == new_name && l.id != id)
    {
        return Err("Library name already exists".to_string());
    }

    // Find and update the library
    if let Some(lib) = config.libraries.iter_mut().find(|l| l.id == id) {
        lib.name = new_name.to_string();
        save_app_config(&config)?;
        Ok(())
    } else {
        Err("Library not found".to_string())
    }
}

/// Remove a library (also deletes the database file)
pub fn remove_library(id: &str) -> Result<(), String> {
    let mut config = load_app_config()?;

    // Cannot remove the only remaining library
    if config.libraries.len() <= 1 {
        return Err("Cannot remove the last library".to_string());
    }

    // Find and remove the library
    let original_len = config.libraries.len();
    config.libraries.retain(|l| l.id != id);

    if config.libraries.len() == original_len {
        return Err("Library not found".to_string());
    }

    // If removing current library, switch to first available
    if config.current_library_id == id {
        config.current_library_id = config.libraries[0].id.clone();
    }

    save_app_config(&config)?;

    // Delete the database file and SQLite WAL/SHM companion files
    let db_path = t_storage::get_library_db_path(id)?;
    let db = std::path::Path::new(&db_path);
    for path in [
        db.to_path_buf(),
        db.with_extension("db-wal"),
        db.with_extension("db-shm"),
    ] {
        if path.exists() {
            let _ = fs::remove_file(&path);
        }
    }

    let thumb_cache_dir = get_app_cache_dir()?.join(id);
    if thumb_cache_dir.exists() {
        let _ = fs::remove_dir_all(&thumb_cache_dir);
    }

    Ok(())
}

/// Switch to a different library
pub fn switch_library(id: &str) -> Result<(), String> {
    let mut config = load_app_config()?;

    // Verify library exists
    if !config.libraries.iter().any(|l| l.id == id) {
        return Err("Library not found".to_string());
    }

    config.current_library_id = id.to_string();
    save_app_config(&config)?;

    Ok(())
}

/// Get library info
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryInfo {
    pub db_file_size: i64,
    pub db_file_path: String,
    pub file_count: i64,
    pub total_size: i64,
}

pub fn get_library_info(id: &str) -> Result<LibraryInfo, String> {
    let db_path = t_storage::get_library_db_path(id)?;

    // Get db file size
    let db_file_size = if std::path::Path::new(&db_path).exists() {
        fs::metadata(&db_path).map(|m| m.len() as i64).unwrap_or(0)
    } else {
        0
    };

    // Open connection to the library's DB to get file stats
    let conn = rusqlite::Connection::open(&db_path)
        .map_err(|e| format!("Failed to open library DB: {}", e))?;

    let (file_count, total_size): (i64, i64) = conn
        .query_row(
            "SELECT COUNT(id), COALESCE(SUM(size), 0) FROM afiles",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap_or((0, 0));

    Ok(LibraryInfo {
        db_file_size,
        db_file_path: db_path,
        file_count,
        total_size,
    })
}

/// Save library state
pub fn save_library_state(id: &str, state: LibraryState) -> Result<(), String> {
    let mut config = load_app_config()?;

    if let Some(lib) = config.libraries.iter_mut().find(|l| l.id == id) {
        lib.state = state;
        save_app_config(&config)?;
        Ok(())
    } else {
        Err("Library not found".to_string())
    }
}

/// Get library state
pub fn get_library_state(id: &str) -> Result<LibraryState, String> {
    let config = load_app_config()?;

    if let Some(lib) = config.libraries.iter().find(|l| l.id == id) {
        Ok(lib.state.clone())
    } else {
        Err("Library not found".to_string())
    }
}

/// Get current library state
pub fn get_current_library_state() -> Result<LibraryState, String> {
    let config = load_app_config()?;
    get_library_state(&config.current_library_id)
}

/// Hide/Show a library
pub fn hide_library(id: &str, hidden: bool) -> Result<(), String> {
    let mut config = load_app_config()?;

    // Cannot hide the current library
    // if config.current_library_id == id && hidden {
    //     return Err("Cannot hide the current library".to_string());
    // }

    if let Some(lib) = config.libraries.iter_mut().find(|l| l.id == id) {
        lib.hidden = hidden;
        save_app_config(&config)?;
        Ok(())
    } else {
        Err("Library not found".to_string())
    }
}

/// Reorder libraries
pub fn reorder_libraries(ids: Vec<String>) -> Result<(), String> {
    let mut config = load_app_config()?;

    // Create a map for quick lookup
    let mut lib_map: std::collections::HashMap<String, Library> = config
        .libraries
        .drain(..)
        .map(|l| (l.id.clone(), l))
        .collect();

    let mut new_libraries = Vec::new();

    // Rebuild the list based on the new order
    for id in ids {
        if let Some(lib) = lib_map.remove(&id) {
            new_libraries.push(lib);
        }
    }

    // Append any remaining libraries (shouldn't happen if frontend is correct, but safe fallback)
    for (_, lib) in lib_map {
        new_libraries.push(lib);
    }

    config.libraries = new_libraries;
    save_app_config(&config)?;

    Ok(())
}

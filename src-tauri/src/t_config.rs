/**
 * App configuration management.
 * Handles app-config.json for multi-library support.
 * project: Lap
 * author:  julyx10
 * date:    2026-01-15
 */
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock, RwLock};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use uuid::Uuid;
use crate::t_storage;

static APP_IDENTIFIER: OnceLock<String> = OnceLock::new();
static CONFIG_IO_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
static CURRENT_LIBRARY_ID: RwLock<Option<String>> = RwLock::new(None);

pub fn current_library_id() -> Result<String, String> {
    if let Some(id) = CURRENT_LIBRARY_ID.read().map_err(|e| e.to_string())?.clone() {
        return Ok(id);
    }
    Ok(load_app_config()?.current_library_id)
}


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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SmartAlbumState {
    #[serde(default = "default_smart_album_type")]
    pub r#type: String,
    pub id: Option<Value>,
}

fn default_smart_album_type() -> String {
    "system".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SmartAlbumRuleState {
    pub id: String,
    pub field: String,
    pub operator: String,
    pub value: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SmartAlbumQueryState {
    #[serde(default = "default_smart_album_query_version")]
    pub version: i32,
    #[serde(default = "default_smart_album_match")]
    pub r#match: String,
    #[serde(default)]
    pub rules: Vec<SmartAlbumRuleState>,
}

fn default_smart_album_query_version() -> i32 {
    1
}

fn default_smart_album_match() -> String {
    "all".to_string()
}

impl Default for SmartAlbumQueryState {
    fn default() -> Self {
        Self {
            version: default_smart_album_query_version(),
            r#match: default_smart_album_match(),
            rules: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SmartAlbumSortState {
    pub r#type: i64,
    pub order: i64,
}

impl Default for SmartAlbumSortState {
    fn default() -> Self {
        Self {
            r#type: 0,
            order: 1,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SmartAlbumGroupState {
    pub r#type: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CustomSmartAlbumState {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default = "default_smart_album_source")]
    pub source: String,
    #[serde(default)]
    pub query: SmartAlbumQueryState,
    #[serde(default)]
    pub sort: SmartAlbumSortState,
    #[serde(default)]
    pub group: SmartAlbumGroupState,
    #[serde(default)]
    pub cover_file_id: Option<i64>,
    #[serde(default)]
    pub count: Option<i64>,
    #[serde(default)]
    pub created_at: i64,
    #[serde(default)]
    pub updated_at: i64,
}

fn default_smart_album_source() -> String {
    "rules".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryPanelState {
    #[serde(default = "default_library_item")]
    pub item: String,
    #[serde(default, alias = "smart_id")]
    pub smart_id: Option<String>,
    #[serde(default = "default_ratings_expanded", alias = "ratings_expanded")]
    pub ratings_expanded: bool,
    #[serde(default = "default_culling_expanded", alias = "culling_expanded")]
    pub culling_expanded: bool,
    #[serde(default = "default_subjects_expanded", alias = "subjects_expanded")]
    pub subjects_expanded: bool,
    #[serde(default)]
    pub subject_counts: HashMap<String, i64>,
}

fn default_library_item() -> String {
    "all-files".to_string()
}

fn default_subjects_expanded() -> bool {
    true
}

fn default_culling_expanded() -> bool {
    true
}

fn default_ratings_expanded() -> bool {
    true
}

impl Default for LibraryPanelState {
    fn default() -> Self {
        Self {
            item: default_library_item(),
            smart_id: None,
            ratings_expanded: default_ratings_expanded(),
            culling_expanded: default_culling_expanded(),
            subjects_expanded: default_subjects_expanded(),
            subject_counts: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RatingState {
    pub item: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CullingState {
    #[serde(default = "default_culling_item")]
    pub item: String,
}

fn default_culling_item() -> String {
    "pick".to_string()
}

impl Default for CullingState {
    fn default() -> Self {
        Self {
            item: default_culling_item(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TagState {
    pub id: Option<i64>,
    #[serde(default)]
    pub collapsed_tag_ids: Vec<i64>,
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
    pub make: Option<String>,
    pub model: Option<String>,
    pub lens_make: Option<String>,
    pub lens_model: Option<String>,
}

impl Default for TagState {
    fn default() -> Self {
        Self {
            id: None,
            collapsed_tag_ids: Vec::new(),
        }
    }
}

impl Default for CameraState {
    fn default() -> Self {
        Self {
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
    #[serde(default)]
    pub count: Option<i64>,
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
pub struct CollectionState {
    #[serde(default, alias = "selected_id")]
    pub selected_id: Option<i64>,
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
    #[serde(default)]
    pub library: LibraryPanelState,
    pub album: AlbumState,
    #[serde(default)]
    pub smart_album: SmartAlbumState,
    #[serde(default)]
    pub smart_albums: Vec<CustomSmartAlbumState>,
    #[serde(default)]
    pub rating: RatingState,
    #[serde(default)]
    pub culling: CullingState,
    pub tag: TagState,
    pub calendar: CalendarState,
    pub camera: CameraState,
    pub location: LocationState,
    #[serde(default)]
    pub person: PersonState,
    #[serde(default)]
    pub collection: CollectionState,
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
    #[serde(default)]
    pub thumbnail_cache_dir: Option<String>,
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
            thumbnail_cache_dir: None,
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
    let config = load_app_config_locked()?;
    *CURRENT_LIBRARY_ID.write().map_err(|e| e.to_string())? = Some(config.current_library_id.clone());
    Ok(config)
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

/// Serialize the whole read/modify/write, not just the individual file accesses.
pub(crate) fn update_app_config<T>(
    update: impl FnOnce(&mut AppConfig) -> Result<T, String>,
) -> Result<T, String> {
    update_config_with(
        config_io_lock(),
        load_app_config_locked,
        save_app_config_locked,
        update,
    )
}

fn update_config_with<T>(
    lock: &Mutex<()>,
    load: impl FnOnce() -> Result<AppConfig, String>,
    save: impl FnOnce(&AppConfig) -> Result<(), String>,
    update: impl FnOnce(&mut AppConfig) -> Result<T, String>,
) -> Result<T, String> {
    let _guard = lock
        .lock()
        .map_err(|_| "Config lock poisoned".to_string())?;
    let mut config = load()?;
    let result = update(&mut config)?;
    save(&config)?;
    Ok(result)
}

pub(crate) fn set_thumbnail_cache_dir(directory: Option<String>) -> Result<(), String> {
    update_app_config(|config| {
        config.thumbnail_cache_dir = directory;
        Ok(())
    })
}

fn save_app_config_locked(config: &AppConfig) -> Result<(), String> {
    let config_path = get_config_file_path()?;
    let content = serde_json::to_string_pretty(config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;
    write_atomic(&config_path, &content)
        .map_err(|e| format!("Failed to write config file: {}", e))?;
    *CURRENT_LIBRARY_ID.write().map_err(|e| e.to_string())? = Some(config.current_library_id.clone());
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
        thumbnail_cache_dir: None,
        current_library_id,
        libraries,
    })
}

/// Add a new library
pub fn add_library(name: &str) -> Result<Library, String> {
    update_app_config(|config| {
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

        Ok(library)
    })
}

/// Edit library name
pub fn edit_library(id: &str, new_name: &str) -> Result<(), String> {
    update_app_config(|config| {
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
            Ok(())
        } else {
            Err("Library not found".to_string())
        }
    })
}

/// Remove a library (also deletes the database file)
pub fn remove_library(id: &str) -> Result<(), String> {
    update_app_config(|config| {
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

        Ok(())
    })?;

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

    for warning in crate::t_cache_storage::remove_library_cache(id) {
        eprintln!("Library {id} cache cleanup failed: {warning}");
    }

    Ok(())
}

/// Switch to a different library
pub fn switch_library(id: &str) -> Result<(), String> {
    update_app_config(|config| {
        // Verify library exists
        if !config.libraries.iter().any(|l| l.id == id) {
            return Err("Library not found".to_string());
        }

        config.current_library_id = id.to_string();

        Ok(())
    })
}

/// Get library info
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryInfo {
    pub db_file_size: i64,
    pub db_file_path: String,
    pub file_count: i64,
    pub total_size: i64,
    pub thumb_cache_size: i64,
}

pub fn get_library_info(id: &str) -> Result<LibraryInfo, String> {
    let _lease = t_storage::DbConnectionLease::acquire()?;
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

    drop(conn);
    drop(_lease);

    // Sum thumbnail cache size (jpg/png files under cache_dir/{id}/).
    // Best-effort: any IO error just contributes 0 rather than failing the call.
    let thumb_cache_size = get_thumb_cache_size(id);

    Ok(LibraryInfo {
        db_file_size,
        db_file_path: db_path,
        file_count,
        total_size,
        thumb_cache_size,
    })
}

fn get_thumb_cache_size(library_id: &str) -> i64 {
    let Ok(_cache_access) = crate::t_cache_storage::access() else { return 0; };
    let Ok(cache_root) = crate::t_cache_storage::root().map(|p| p.join(library_id)) else {
        return 0;
    };
    if !cache_root.is_dir() {
        return 0;
    }
    let mut total: i64 = 0;
    for entry in walkdir::WalkDir::new(&cache_root).into_iter().filter_map(|e| e.ok()) {
        if !entry.file_type().is_file() {
            continue;
        }
        let is_thumb = matches!(
            entry.path().extension().and_then(|ext| ext.to_str()),
            Some("jpg" | "png")
        );
        if !is_thumb {
            continue;
        }
        if let Ok(md) = entry.metadata() {
            total = total.saturating_add(md.len() as i64);
        }
    }
    total
}

/// Save library state
pub fn save_library_state(id: &str, state: LibraryState) -> Result<(), String> {
    update_app_config(|config| {
        if let Some(lib) = config.libraries.iter_mut().find(|l| l.id == id) {
            lib.state = state;
            Ok(())
        } else {
            Err("Library not found".to_string())
        }
    })
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
    config
        .libraries
        .iter()
        .find(|lib| lib.id == config.current_library_id)
        .map(|lib| lib.state.clone())
        .ok_or_else(|| "Library not found".to_string())
}

/// Hide/Show a library
pub fn hide_library(id: &str, hidden: bool) -> Result<(), String> {
    update_app_config(|config| {
        // Cannot hide the current library
        // if config.current_library_id == id && hidden {
        //     return Err("Cannot hide the current library".to_string());
        // }

        if let Some(lib) = config.libraries.iter_mut().find(|l| l.id == id) {
            lib.hidden = hidden;
            Ok(())
        } else {
            Err("Library not found".to_string())
        }
    })
}

/// Reorder libraries
pub fn reorder_libraries(ids: Vec<String>) -> Result<(), String> {
    update_app_config(|config| {
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

        Ok(())
    })
}

#[cfg(test)]
mod config_update_tests {
    use super::*;

    #[test]
    fn tag_state_preserves_tree_folding_and_accepts_legacy_groups() {
        let legacy: TagState = serde_json::from_value(serde_json::json!({
            "id": 10, "groupId": 2, "collapsedGroupIds": [2]
        }))
        .unwrap();
        assert_eq!(legacy.id, Some(10));
        assert!(legacy.collapsed_tag_ids.is_empty());
        let state = TagState {
            id: Some(10),
            collapsed_tag_ids: vec![10, 20],
        };
        let saved = serde_json::to_value(&state).unwrap();
        assert_eq!(
            saved,
            serde_json::json!({"id": 10, "collapsedTagIds": [10, 20]})
        );
        let restored: TagState = serde_json::from_value(saved).unwrap();
        assert_eq!(restored.collapsed_tag_ids, state.collapsed_tag_ids);
        assert!(TagState::default().collapsed_tag_ids.is_empty());
    }
    use std::sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc,
    };

    #[test]
    fn concurrent_updates_read_after_the_previous_commit() {
        let lock = Arc::new(Mutex::new(()));
        let stored = Arc::new(Mutex::new(AppConfig::default()));
        let (started_tx, started_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let writer_lock = lock.clone();
        let writer_stored = stored.clone();
        let writer = std::thread::spawn(move || {
            update_config_with(
                &writer_lock,
                || Ok(writer_stored.lock().unwrap().clone()),
                |config| {
                    *writer_stored.lock().unwrap() = config.clone();
                    Ok(())
                },
                |config| {
                    config.thumbnail_cache_dir = Some("/new-cache".into());
                    started_tx.send(()).unwrap();
                    release_rx.recv().unwrap();
                    Ok(())
                },
            )
        });
        started_rx.recv_timeout(Duration::from_secs(2)).unwrap();
        let read_started = Arc::new(AtomicBool::new(false));
        let reader_started = read_started.clone();
        let reader_lock = lock.clone();
        let reader_stored = stored.clone();
        let (attempt_tx, attempt_rx) = mpsc::channel();
        let reader = std::thread::spawn(move || {
            attempt_tx.send(()).unwrap();
            update_config_with(
                &reader_lock,
                || {
                    reader_started.store(true, Ordering::SeqCst);
                    Ok(reader_stored.lock().unwrap().clone())
                },
                |config| {
                    *reader_stored.lock().unwrap() = config.clone();
                    Ok(())
                },
                |config| {
                    config.last_selected_item_index = 42;
                    Ok(())
                },
            )
        });
        attempt_rx.recv_timeout(Duration::from_secs(2)).unwrap();
        assert!(!read_started.load(Ordering::SeqCst));
        release_tx.send(()).unwrap();
        writer.join().unwrap().unwrap();
        reader.join().unwrap().unwrap();
        let final_config = stored.lock().unwrap();
        assert_eq!(
            final_config.thumbnail_cache_dir.as_deref(),
            Some("/new-cache")
        );
        assert_eq!(final_config.last_selected_item_index, 42);
        assert_eq!(final_config.libraries.len(), 1);
    }

    #[test]
    fn rejected_update_never_saves_partial_changes() {
        let lock = Mutex::new(());
        let saved = AtomicBool::new(false);
        let result = update_config_with(
            &lock,
            || Ok(AppConfig::default()),
            |_| {
                saved.store(true, Ordering::SeqCst);
                Ok(())
            },
            |config| {
                config.thumbnail_cache_dir = Some("/new-cache".into());
                Err::<(), _>("rejected".into())
            },
        );
        assert!(result.is_err());
        assert!(!saved.load(Ordering::SeqCst));
    }
}

/**
 * App configuration management.
 * Handles app-config.json for multi-library support.
 * project: Lap
 * author:  julyx10
 * date:    2026-01-15
 */
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tauri::Emitter;
use uuid::Uuid;

static APP_IDENTIFIER: OnceLock<String> = OnceLock::new();
static CONFIG_IO_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
static STORAGE_MOVE_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
static LIBRARY_STORAGE_MOVE_STATE: OnceLock<Mutex<LibraryStorageMoveState>> = OnceLock::new();
static APP_DATA_DIR_OVERRIDE: OnceLock<Mutex<Option<PathBuf>>> = OnceLock::new();
#[cfg(test)]
static CONFIG_WRITE_FAIL_OVERRIDE: OnceLock<Mutex<bool>> = OnceLock::new();

pub const LIBRARY_STORAGE_MOVE_IN_PROGRESS_ERROR: &str = "Library storage move in progress.";

pub fn set_app_identifier(identifier: &str) {
    let _ = APP_IDENTIFIER.set(identifier.to_string());
}

fn config_io_lock() -> &'static Mutex<()> {
    CONFIG_IO_LOCK.get_or_init(|| Mutex::new(()))
}

fn storage_move_lock() -> &'static Mutex<()> {
    STORAGE_MOVE_LOCK.get_or_init(|| Mutex::new(()))
}

fn storage_move_state() -> &'static Mutex<LibraryStorageMoveState> {
    LIBRARY_STORAGE_MOVE_STATE.get_or_init(|| Mutex::new(LibraryStorageMoveState::default()))
}

fn app_data_dir_override() -> &'static Mutex<Option<PathBuf>> {
    APP_DATA_DIR_OVERRIDE.get_or_init(|| Mutex::new(None))
}

#[cfg(test)]
fn config_write_fail_override() -> &'static Mutex<bool> {
    CONFIG_WRITE_FAIL_OVERRIDE.get_or_init(|| Mutex::new(false))
}

#[cfg(test)]
fn should_fail_config_write() -> bool {
    config_write_fail_override()
        .lock()
        .map(|flag| *flag)
        .unwrap_or(false)
}

#[cfg(test)]
pub(crate) fn set_test_config_write_failure(enabled: bool) {
    if let Ok(mut flag) = config_write_fail_override().lock() {
        *flag = enabled;
    }
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
pub struct FavoriteState {
    #[serde(alias = "album_id")]
    pub album_id: Option<i64>,
    #[serde(alias = "folder_id")]
    pub folder_id: i64,
    #[serde(alias = "folder_path")]
    pub folder_path: Option<String>,
    #[serde(default)]
    pub rating: i32,
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
    #[serde(default)]
    pub storage_dir: Option<String>,
    #[serde(default)]
    pub metadata_initialized: bool,
}

/// App configuration stored in app-config.json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default)]
    pub debug: bool,
    pub current_library_id: String,
    pub libraries: Vec<Library>,
}

impl Default for AppConfig {
    fn default() -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            debug: false,
            current_library_id: "default".to_string(),
            libraries: vec![Library {
                id: "default".to_string(),
                name: "Default Library".to_string(),
                created_at: now,
                state: LibraryState::default(),
                hidden: false,
                storage_dir: None,
                metadata_initialized: false,
            }],
        }
    }
}

/// Get the AppData directory for app
pub fn get_app_data_dir() -> Result<PathBuf, String> {
    if let Ok(override_dir) = app_data_dir_override().lock() {
        if let Some(path) = override_dir.as_ref() {
            return Ok(path.clone());
        }
    }

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

fn normalize_storage_dir_value(storage_dir: Option<String>) -> Option<String> {
    storage_dir.and_then(|dir| {
        let trimmed = dir.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}

fn canonicalize_existing_directory(path: &Path, label: &str) -> Result<PathBuf, String> {
    if !path.is_absolute() {
        return Err(format!(
            "{} must be an absolute path: {}",
            label,
            path.display()
        ));
    }

    if !path.exists() {
        return Err(format!("{} does not exist: {}", label, path.display()));
    }

    if !path.is_dir() {
        return Err(format!("{} is not a directory: {}", label, path.display()));
    }

    fs::canonicalize(path).map_err(|e| {
        format!(
            "Failed to canonicalize {} '{}': {}",
            label,
            path.display(),
            e
        )
    })
}

fn canonicalize_custom_storage_dir(storage_dir: Option<String>) -> Result<Option<String>, String> {
    match normalize_storage_dir_value(storage_dir) {
        Some(dir) => {
            let canonical =
                canonicalize_existing_directory(Path::new(&dir), "Target storage directory")?;
            Ok(Some(canonical.to_string_lossy().into_owned()))
        }
        None => Ok(None),
    }
}

fn get_effective_storage_dir(library: &Library) -> Result<(PathBuf, bool), String> {
    match normalize_storage_dir_value(library.storage_dir.clone()) {
        Some(dir) => Ok((PathBuf::from(dir), false)),
        None => Ok((get_libraries_dir()?, true)),
    }
}

fn ensure_storage_dir_ready(library: &Library) -> Result<PathBuf, String> {
    let (dir, uses_default) = get_effective_storage_dir(library)?;

    if uses_default {
        fs::create_dir_all(&dir).map_err(|e| {
            format!(
                "Failed to create default libraries directory '{}': {}",
                dir.display(),
                e
            )
        })?;
        return Ok(dir);
    }

    if !dir.exists() {
        return Err(format!(
            "Library storage directory is unavailable: {}",
            dir.display()
        ));
    }

    if !dir.is_dir() {
        return Err(format!(
            "Library storage path is not a directory: {}",
            dir.display()
        ));
    }

    Ok(dir)
}

fn is_storage_dir_available(library: &Library) -> Result<bool, String> {
    let (dir, uses_default) = get_effective_storage_dir(library)?;
    if uses_default {
        return Ok(true);
    }

    Ok(dir.exists() && dir.is_dir())
}

fn library_has_metadata_files(library: &Library) -> Result<bool, String> {
    if !is_storage_dir_available(library)? {
        return Ok(false);
    }

    let db_path = library_db_path_from_library(library)?;
    Ok(!existing_sqlite_files(&db_path).is_empty())
}

fn library_effective_metadata_initialized(library: &Library) -> Result<bool, String> {
    if library.metadata_initialized {
        return Ok(true);
    }

    library_has_metadata_files(library)
}

fn library_allows_disconnected_metadata_operation(library: &Library) -> Result<bool, String> {
    Ok(!library_effective_metadata_initialized(library)?)
}

fn library_db_path_from_library(library: &Library) -> Result<PathBuf, String> {
    let (dir, _) = get_effective_storage_dir(library)?;
    Ok(dir.join(format!("{}.db", library.id)))
}

fn find_library<'a>(config: &'a AppConfig, library_id: &str) -> Result<&'a Library, String> {
    config
        .libraries
        .iter()
        .find(|library| library.id == library_id)
        .ok_or_else(|| "Library not found".to_string())
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

fn update_app_config<R, F>(mutator: F) -> Result<R, String>
where
    F: FnOnce(&mut AppConfig) -> Result<R, String>,
{
    let _guard = config_io_lock()
        .lock()
        .map_err(|_| "Config lock poisoned".to_string())?;
    let mut config = load_app_config_locked()?;
    let result = mutator(&mut config)?;
    save_app_config_locked(&config)?;
    Ok(result)
}

fn update_library_entry<R, F>(library_id: &str, mutator: F) -> Result<R, String>
where
    F: FnOnce(&mut Library) -> Result<R, String>,
{
    update_app_config(|config| {
        let library = config
            .libraries
            .iter_mut()
            .find(|library| library.id == library_id)
            .ok_or_else(|| "Library not found".to_string())?;
        mutator(library)
    })
}

pub fn ensure_library_storage_write_allowed() -> Result<(), String> {
    let move_state = storage_move_state()
        .lock()
        .map_err(|_| "Storage move state lock poisoned".to_string())?;

    if move_state.library_id.is_some() && move_state.status == "running" {
        return Err(LIBRARY_STORAGE_MOVE_IN_PROGRESS_ERROR.to_string());
    }

    Ok(())
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
    #[cfg(test)]
    if should_fail_config_write() {
        return Err("Injected config write failure".to_string());
    }

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
            storage_dir: None,
            metadata_initialized: true,
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
        current_library_id,
        libraries,
    })
}

/// Get the database file path for a library and ensure the storage directory is ready
pub fn get_library_db_path_for_open(library_id: &str) -> Result<String, String> {
    let config = load_app_config()?;
    let library = find_library(&config, library_id)?;
    let storage_dir = ensure_storage_dir_ready(library)?;
    let db_path = storage_dir.join(format!("{}.db", library.id));
    Ok(db_path.to_string_lossy().into_owned())
}

/// Get the current library's database file path
pub fn get_current_db_path() -> Result<String, String> {
    let config = load_app_config()?;
    let library = find_library(&config, &config.current_library_id)?;
    let db_path = library_db_path_from_library(library)?;
    Ok(db_path.to_string_lossy().into_owned())
}

/// Get the current library's database file path and ensure the storage directory is ready
pub fn get_current_db_path_for_open() -> Result<String, String> {
    let config = load_app_config()?;
    let library = find_library(&config, &config.current_library_id)?;
    let storage_dir = ensure_storage_dir_ready(library)?;
    let db_path = storage_dir.join(format!("{}.db", library.id));
    Ok(db_path.to_string_lossy().into_owned())
}

pub fn mark_library_metadata_initialized(library_id: &str) -> Result<(), String> {
    update_library_entry(library_id, |library| {
        library.metadata_initialized = true;
        Ok(())
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
        storage_dir: None,
        metadata_initialized: false,
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
    let original_config = config.clone();

    // Cannot remove the only remaining library
    if config.libraries.len() <= 1 {
        return Err("Cannot remove the last library".to_string());
    }

    let library = config
        .libraries
        .iter()
        .find(|l| l.id == id)
        .cloned()
        .ok_or_else(|| "Library not found".to_string())?;
    let storage_available = is_storage_dir_available(&library)?;
    if !storage_available && !library_allows_disconnected_metadata_operation(&library)? {
        return Err(format!(
            "Library storage is disconnected and still contains metadata. Reconnect the storage before deleting this library: {}",
            library_db_path_from_library(&library)?.display()
        ));
    }
    let db_path = library_db_path_from_library(&library)?;
    let existing_paths = if storage_available {
        existing_sqlite_files(&db_path)
    } else {
        Vec::new()
    };
    let quarantine_stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| format!("Failed to get timestamp for library deletion: {}", e))?
        .as_nanos();
    let quarantined_paths = move_paths_to_quarantine(&existing_paths, quarantine_stamp)?;

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

    if let Err(save_error) = save_app_config(&config) {
        let rollback_error = rollback_quarantined_paths(&quarantined_paths);
        return Err(match rollback_error {
            Ok(_) => save_error,
            Err(file_error) => format!("{} Rollback failed: {}", save_error, file_error),
        });
    }

    if let Err(cleanup_error) = cleanup_quarantined_paths(&quarantined_paths) {
        let config_rollback_error = save_app_config(&original_config).err();
        let file_rollback_error = rollback_quarantined_paths(&quarantined_paths).err();
        let mut errors = vec![cleanup_error];
        if let Some(error) = config_rollback_error {
            errors.push(format!("Config rollback failed: {}", error));
        }
        if let Some(error) = file_rollback_error {
            errors.push(format!("File rollback failed: {}", error));
        }
        return Err(errors.join(" | "));
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
    pub storage_dir: Option<String>,
    pub uses_default_storage: bool,
    pub storage_available: bool,
    pub metadata_initialized: bool,
}

#[derive(Debug, Clone, Default)]
struct LibraryStorageMoveState {
    library_id: Option<String>,
    phase: String,
    status: String,
    cancellable: bool,
    cancel_requested: bool,
}

pub fn get_library_info(id: &str) -> Result<LibraryInfo, String> {
    let config = load_app_config()?;
    let library = find_library(&config, id)?;
    let db_path = library_db_path_from_library(library)?;
    let (storage_dir, uses_default_storage) = get_effective_storage_dir(library)?;
    let storage_available = is_storage_dir_available(library)?;
    let metadata_initialized = library_effective_metadata_initialized(library)?;
    let db_path_ref = Path::new(&db_path);

    if storage_available && db_path_ref.exists() && !library.metadata_initialized {
        let _ = mark_library_metadata_initialized(id);
    }

    // Get db file size
    let db_file_size = if db_path_ref.exists() {
        fs::metadata(&db_path).map(|m| m.len() as i64).unwrap_or(0)
    } else {
        0
    };

    let (file_count, total_size): (i64, i64) = if storage_available && db_path_ref.exists() {
        let conn = rusqlite::Connection::open(&db_path)
            .map_err(|e| format!("Failed to open library DB: {}", e))?;

        conn.query_row(
            "SELECT COUNT(id), COALESCE(SUM(size), 0) FROM afiles",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap_or((0, 0))
    } else {
        (0, 0)
    };

    Ok(LibraryInfo {
        db_file_size,
        db_file_path: db_path.to_string_lossy().into_owned(),
        file_count,
        total_size,
        storage_dir: Some(storage_dir.to_string_lossy().into_owned()),
        uses_default_storage,
        storage_available,
        metadata_initialized,
    })
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryStorageMoveProgress {
    pub library_id: String,
    pub phase: String,
    pub bytes_copied: u64,
    pub total_bytes: u64,
    pub percent: u64,
    pub message: String,
    pub cancellable: bool,
    pub cancel_requested: bool,
    pub status: String,
}

fn emit_library_storage_move_progress(
    app_handle: &tauri::AppHandle,
    library_id: &str,
    phase: &str,
    bytes_copied: u64,
    total_bytes: u64,
    cancellable: bool,
    status: &str,
    message: impl Into<String>,
) {
    let percent = if total_bytes == 0 {
        if phase == "finished" { 100 } else { 0 }
    } else {
        ((bytes_copied.saturating_mul(100)) / total_bytes).min(100)
    };

    let move_state = {
        let mut move_state = storage_move_state()
            .lock()
            .map_err(|_| "Storage move state lock poisoned".to_string())
            .ok();

        if let Some(ref mut move_state) = move_state {
            move_state.library_id = Some(library_id.to_string());
            move_state.phase = phase.to_string();
            move_state.status = status.to_string();
            move_state.cancellable = cancellable;
            move_state.clone()
        } else {
            LibraryStorageMoveState {
                library_id: Some(library_id.to_string()),
                phase: phase.to_string(),
                status: status.to_string(),
                cancellable,
                cancel_requested: false,
            }
        }
    };

    let _ = app_handle.emit(
        "library-storage-move-progress",
        LibraryStorageMoveProgress {
            library_id: library_id.to_string(),
            phase: phase.to_string(),
            bytes_copied,
            total_bytes,
            percent,
            message: message.into(),
            cancellable: move_state.cancellable,
            cancel_requested: move_state.cancel_requested,
            status: move_state.status,
        },
    );
}

fn sqlite_related_paths(db_path: &Path) -> Vec<PathBuf> {
    vec![
        db_path.to_path_buf(),
        PathBuf::from(format!("{}-wal", db_path.to_string_lossy())),
        PathBuf::from(format!("{}-shm", db_path.to_string_lossy())),
    ]
}

fn existing_sqlite_files(db_path: &Path) -> Vec<PathBuf> {
    sqlite_related_paths(db_path)
        .into_iter()
        .filter(|path| path.exists())
        .collect()
}

fn temp_storage_path(path: &Path, stamp: u128) -> PathBuf {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("library.db");
    path.with_file_name(format!(
        "{}.moving-{}-{}",
        file_name,
        std::process::id(),
        stamp
    ))
}

fn temp_delete_path(path: &Path, stamp: u128) -> PathBuf {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("library.db");
    path.with_file_name(format!(
        ".{}.delete-{}-{}",
        file_name,
        std::process::id(),
        stamp
    ))
}

fn cleanup_paths(paths: &[PathBuf]) {
    for path in paths {
        let _ = fs::remove_file(path);
    }
}

fn move_paths_to_quarantine(
    paths: &[PathBuf],
    stamp: u128,
) -> Result<Vec<(PathBuf, PathBuf)>, String> {
    let mut quarantined = Vec::new();

    for original_path in paths {
        let quarantine_path = temp_delete_path(original_path, stamp);
        fs::rename(original_path, &quarantine_path).map_err(|error| {
            let _ = rollback_quarantined_paths(&quarantined);
            format!(
                "Failed to quarantine '{}' before library deletion: {}",
                original_path.display(),
                error
            )
        })?;
        quarantined.push((original_path.clone(), quarantine_path));
    }

    Ok(quarantined)
}

fn rollback_quarantined_paths(paths: &[(PathBuf, PathBuf)]) -> Result<(), String> {
    let mut errors = Vec::new();

    for (original_path, quarantine_path) in paths.iter().rev() {
        if !quarantine_path.exists() {
            continue;
        }

        if let Err(error) = fs::rename(quarantine_path, original_path) {
            errors.push(format!(
                "Failed to restore '{}' from '{}': {}",
                original_path.display(),
                quarantine_path.display(),
                error
            ));
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join(" | "))
    }
}

fn quarantine_delete_priority(path: &Path) -> u8 {
    let path_string = path.to_string_lossy();
    if path_string.contains("-wal") {
        0
    } else if path_string.contains("-shm") {
        1
    } else {
        2
    }
}

fn cleanup_quarantined_paths(paths: &[(PathBuf, PathBuf)]) -> Result<(), String> {
    let mut ordered_paths = paths.to_vec();
    ordered_paths.sort_by_key(|(_, quarantine_path)| quarantine_delete_priority(quarantine_path));

    for (_, quarantine_path) in ordered_paths {
        if !quarantine_path.exists() {
            continue;
        }

        fs::remove_file(&quarantine_path).map_err(|error| {
            format!(
                "Failed to delete quarantined library file '{}': {}",
                quarantine_path.display(),
                error
            )
        })?;
    }

    Ok(())
}

const LIBRARY_STORAGE_MOVE_CANCELLED_ERROR: &str = "Library storage move cancelled.";

struct LibraryStorageMoveStateGuard;

impl Drop for LibraryStorageMoveStateGuard {
    fn drop(&mut self) {
        if let Ok(mut move_state) = storage_move_state().lock() {
            *move_state = LibraryStorageMoveState::default();
        }
    }
}

fn start_library_storage_move(library_id: &str) -> Result<LibraryStorageMoveStateGuard, String> {
    let mut move_state = storage_move_state()
        .lock()
        .map_err(|_| "Storage move state lock poisoned".to_string())?;
    *move_state = LibraryStorageMoveState {
        library_id: Some(library_id.to_string()),
        phase: String::new(),
        status: "running".to_string(),
        cancellable: false,
        cancel_requested: false,
    };
    Ok(LibraryStorageMoveStateGuard)
}

fn is_library_storage_move_cancel_requested(library_id: &str) -> Result<bool, String> {
    let move_state = storage_move_state()
        .lock()
        .map_err(|_| "Storage move state lock poisoned".to_string())?;
    Ok(move_state.library_id.as_deref() == Some(library_id) && move_state.cancel_requested)
}

pub fn cancel_library_storage_move(library_id: &str) -> Result<(), String> {
    let mut move_state = storage_move_state()
        .lock()
        .map_err(|_| "Storage move state lock poisoned".to_string())?;

    if move_state.library_id.as_deref() != Some(library_id) {
        return Err("No active storage move for this library".to_string());
    }

    if !move_state.cancellable {
        return Err("Library storage move can no longer be cancelled".to_string());
    }

    move_state.cancel_requested = true;
    Ok(())
}

fn ensure_library_storage_move_not_cancelled(
    library_id: &str,
    app_handle: &tauri::AppHandle,
    phase: &str,
    bytes_copied: u64,
    total_bytes: u64,
) -> Result<(), String> {
    if is_library_storage_move_cancel_requested(library_id)? {
        emit_library_storage_move_progress(
            app_handle,
            library_id,
            "cancelled",
            bytes_copied,
            total_bytes,
            false,
            "cancelled",
            format!("Library storage move cancelled during {}", phase),
        );
        return Err(LIBRARY_STORAGE_MOVE_CANCELLED_ERROR.to_string());
    }

    Ok(())
}

fn hash_file_blake3(path: &Path, library_id: &str) -> Result<String, String> {
    let mut file = fs::File::open(path)
        .map_err(|e| format!("Failed to open '{}' for hashing: {}", path.display(), e))?;
    let mut hasher = blake3::Hasher::new();
    let mut buffer = [0u8; 1024 * 1024];

    loop {
        let read = file
            .read(&mut buffer)
            .map_err(|e| format!("Failed to read '{}' for hashing: {}", path.display(), e))?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
        if is_library_storage_move_cancel_requested(library_id)? {
            return Err(LIBRARY_STORAGE_MOVE_CANCELLED_ERROR.to_string());
        }
    }

    Ok(hasher.finalize().to_hex().to_string())
}

fn copy_file_with_progress(
    source: &Path,
    target: &Path,
    copied_so_far: &mut u64,
    total_bytes: u64,
    app_handle: &tauri::AppHandle,
    library_id: &str,
) -> Result<(), String> {
    let mut reader = fs::File::open(source)
        .map_err(|e| format!("Failed to open '{}' for copying: {}", source.display(), e))?;
    let mut writer = fs::File::create(target)
        .map_err(|e| format!("Failed to create '{}' for copying: {}", target.display(), e))?;
    let mut buffer = [0u8; 1024 * 1024];

    loop {
        ensure_library_storage_move_not_cancelled(
            library_id,
            app_handle,
            "copying",
            *copied_so_far,
            total_bytes,
        )?;
        let read = reader
            .read(&mut buffer)
            .map_err(|e| format!("Failed to read '{}' while copying: {}", source.display(), e))?;
        if read == 0 {
            break;
        }

        writer.write_all(&buffer[..read]).map_err(|e| {
            format!(
                "Failed to write '{}' while copying: {}",
                target.display(),
                e
            )
        })?;
        *copied_so_far = copied_so_far.saturating_add(read as u64);
        emit_library_storage_move_progress(
            app_handle,
            library_id,
            "copying",
            *copied_so_far,
            total_bytes,
            true,
            "running",
            format!("Copying {}", source.display()),
        );
        ensure_library_storage_move_not_cancelled(
            library_id,
            app_handle,
            "copying",
            *copied_so_far,
            total_bytes,
        )?;
    }

    writer
        .sync_all()
        .map_err(|e| format!("Failed to sync '{}' after copying: {}", target.display(), e))?;

    Ok(())
}

fn checkpoint_sqlite_db(db_path: &Path) -> Result<(), String> {
    if !db_path.exists() {
        return Ok(());
    }

    let conn = rusqlite::Connection::open(db_path).map_err(|e| {
        format!(
            "Failed to open '{}' for checkpoint: {}",
            db_path.display(),
            e
        )
    })?;
    conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")
        .map_err(|e| format!("Failed to checkpoint '{}': {}", db_path.display(), e))?;
    Ok(())
}

pub fn move_library_storage(
    app_handle: &tauri::AppHandle,
    id: &str,
    storage_dir: Option<String>,
) -> Result<LibraryInfo, String> {
    let _guard = storage_move_lock()
        .lock()
        .map_err(|_| "Storage move lock poisoned".to_string())?;

    let config = load_app_config()?;
    let library = config
        .libraries
        .iter()
        .find(|library| library.id == id)
        .cloned()
        .ok_or_else(|| "Library not found".to_string())?;
    let next_storage_dir = canonicalize_custom_storage_dir(storage_dir)?;

    let (current_storage_dir, _) = get_effective_storage_dir(&library)?;
    let source_storage_available = is_storage_dir_available(&library)?;
    let current_compare_dir = if source_storage_available {
        fs::canonicalize(&current_storage_dir).unwrap_or(current_storage_dir.clone())
    } else {
        current_storage_dir.clone()
    };
    let (target_storage_dir, target_uses_default) = match next_storage_dir.clone() {
        Some(dir) => (PathBuf::from(dir), false),
        None => {
            let default_dir = get_libraries_dir()?;
            (fs::canonicalize(&default_dir).unwrap_or(default_dir), true)
        }
    };

    if current_compare_dir == target_storage_dir {
        return get_library_info(id);
    }

    let _move_state_guard = start_library_storage_move(id)?;

    let source_db_path = library_db_path_from_library(&library)?;
    let target_db_path = target_storage_dir.join(format!("{}.db", library.id));
    let target_related_paths = sqlite_related_paths(&target_db_path);

    if target_related_paths
        .iter()
        .any(|path| path.exists() && !path.eq(&source_db_path))
    {
        return Err(format!(
            "Target storage already contains database files for this library: {}",
            target_db_path.display()
        ));
    }

    if !source_storage_available {
        if library_allows_disconnected_metadata_operation(&library)? {
            emit_library_storage_move_progress(
                app_handle,
                id,
                "committing",
                0,
                0,
                false,
                "running",
                if target_uses_default {
                    "Updating library storage to default location".to_string()
                } else {
                    format!(
                        "Updating library storage to {}",
                        target_storage_dir.display()
                    )
                },
            );

            update_library_entry(id, |library| {
                library.storage_dir = next_storage_dir.clone();
                Ok(())
            })?;

            emit_library_storage_move_progress(
                app_handle,
                id,
                "finished",
                0,
                0,
                false,
                "finished",
                "Library storage location updated successfully".to_string(),
            );
            return get_library_info(id);
        }

        return Err(format!(
            "Library storage directory is unavailable and already contains metadata. Reconnect the storage first: {}",
            current_storage_dir.display()
        ));
    }

    let source_files = existing_sqlite_files(&source_db_path);
    if source_files.is_empty() {
        emit_library_storage_move_progress(
            app_handle,
            id,
            "committing",
            0,
            0,
            false,
            "running",
            if target_uses_default {
                "Updating library storage to default location".to_string()
            } else {
                format!(
                    "Updating library storage to {}",
                    target_storage_dir.display()
                )
            },
        );

        update_library_entry(id, |library| {
            library.storage_dir = next_storage_dir.clone();
            Ok(())
        })?;

        emit_library_storage_move_progress(
            app_handle,
            id,
            "finished",
            0,
            0,
            false,
            "finished",
            "Library storage location updated successfully".to_string(),
        );
        return get_library_info(id);
    }

    checkpoint_sqlite_db(&source_db_path)?;

    let source_files = existing_sqlite_files(&source_db_path);
    let total_bytes = source_files
        .iter()
        .filter_map(|path| fs::metadata(path).ok().map(|meta| meta.len()))
        .sum::<u64>();
    emit_library_storage_move_progress(
        app_handle,
        id,
        "copying",
        0,
        total_bytes,
        true,
        "running",
        if target_uses_default {
            "Copying metadata to default storage".to_string()
        } else {
            format!("Copying metadata to {}", target_storage_dir.display())
        },
    );
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| format!("Failed to get timestamp for storage move: {}", e))?
        .as_nanos();
    let temp_targets = source_files
        .iter()
        .map(|source_path| {
            let suffix = source_path
                .file_name()
                .and_then(|name| name.to_str())
                .and_then(|name| name.strip_prefix(&format!("{}.db", library.id)))
                .unwrap_or("");
            temp_storage_path(
                &target_storage_dir.join(format!("{}.db{}", library.id, suffix)),
                stamp,
            )
        })
        .collect::<Vec<_>>();

    let mut copied = 0u64;
    for (source_path, temp_target) in source_files.iter().zip(temp_targets.iter()) {
        if let Err(error) = copy_file_with_progress(
            source_path,
            temp_target,
            &mut copied,
            total_bytes,
            app_handle,
            id,
        ) {
            cleanup_paths(&temp_targets);
            return Err(error);
        }
    }

    emit_library_storage_move_progress(
        app_handle,
        id,
        "verifying",
        total_bytes,
        total_bytes,
        true,
        "running",
        "Verifying copied data".to_string(),
    );

    if let Err(error) = ensure_library_storage_move_not_cancelled(
        id,
        app_handle,
        "verifying",
        total_bytes,
        total_bytes,
    ) {
        cleanup_paths(&temp_targets);
        return Err(error);
    }

    for (source_path, temp_target) in source_files.iter().zip(temp_targets.iter()) {
        let source_hash = match hash_file_blake3(source_path, id) {
            Ok(hash) => hash,
            Err(error) => {
                cleanup_paths(&temp_targets);
                if error == LIBRARY_STORAGE_MOVE_CANCELLED_ERROR {
                    emit_library_storage_move_progress(
                        app_handle,
                        id,
                        "cancelled",
                        total_bytes,
                        total_bytes,
                        false,
                        "cancelled",
                        "Library storage move cancelled during verification".to_string(),
                    );
                }
                return Err(error);
            }
        };
        let target_hash = match hash_file_blake3(temp_target, id) {
            Ok(hash) => hash,
            Err(error) => {
                cleanup_paths(&temp_targets);
                if error == LIBRARY_STORAGE_MOVE_CANCELLED_ERROR {
                    emit_library_storage_move_progress(
                        app_handle,
                        id,
                        "cancelled",
                        total_bytes,
                        total_bytes,
                        false,
                        "cancelled",
                        "Library storage move cancelled during verification".to_string(),
                    );
                }
                return Err(error);
            }
        };
        if source_hash != target_hash {
            cleanup_paths(&temp_targets);
            return Err(format!(
                "Integrity verification failed for '{}'",
                source_path.display()
            ));
        }
    }

    if let Err(error) = ensure_library_storage_move_not_cancelled(
        id,
        app_handle,
        "verifying",
        total_bytes,
        total_bytes,
    ) {
        cleanup_paths(&temp_targets);
        return Err(error);
    }

    emit_library_storage_move_progress(
        app_handle,
        id,
        "committing",
        total_bytes,
        total_bytes,
        false,
        "running",
        "Finalizing storage move".to_string(),
    );

    let final_targets = source_files
        .iter()
        .map(|source_path| {
            let suffix = source_path
                .file_name()
                .and_then(|name| name.to_str())
                .and_then(|name| name.strip_prefix(&format!("{}.db", library.id)))
                .unwrap_or("");
            target_storage_dir.join(format!("{}.db{}", library.id, suffix))
        })
        .collect::<Vec<_>>();

    for target in &final_targets {
        if target.exists() {
            cleanup_paths(&temp_targets);
            return Err(format!(
                "Target storage already contains '{}'",
                target.display()
            ));
        }
    }

    for (temp_target, final_target) in temp_targets.iter().zip(final_targets.iter()) {
        if let Err(e) = fs::rename(temp_target, final_target) {
            cleanup_paths(&temp_targets);
            cleanup_paths(&final_targets);
            return Err(format!(
                "Failed to finalize storage move '{}': {}",
                final_target.display(),
                e
            ));
        }
    }

    if let Err(error) = update_library_entry(id, |library| {
        library.storage_dir = next_storage_dir.clone();
        Ok(())
    }) {
        cleanup_paths(&final_targets);
        return Err(error);
    }

    for source_path in &source_files {
        if source_path.exists() {
            if let Err(error) = fs::remove_file(source_path) {
                eprintln!(
                    "Warning: failed to remove original storage file '{}' after move: {}",
                    source_path.display(),
                    error
                );
            }
        }
    }

    emit_library_storage_move_progress(
        app_handle,
        id,
        "finished",
        total_bytes,
        total_bytes,
        false,
        "finished",
        "Library metadata moved successfully".to_string(),
    );

    get_library_info(id)
}

/// Save library state
pub fn save_library_state(id: &str, state: LibraryState) -> Result<(), String> {
    update_library_entry(id, |library| {
        library.state = state;
        Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;

    static TEST_MUTEX: OnceLock<Mutex<()>> = OnceLock::new();

    fn test_lock() -> &'static Mutex<()> {
        TEST_MUTEX.get_or_init(|| Mutex::new(()))
    }

    struct TestEnv {
        _guard: std::sync::MutexGuard<'static, ()>,
        root: PathBuf,
    }

    impl TestEnv {
        fn new() -> Self {
            let guard = test_lock().lock().unwrap();
            let stamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let root = std::env::temp_dir().join(format!(
                "lap-t-config-tests-{}-{}",
                std::process::id(),
                stamp
            ));
            fs::create_dir_all(&root).unwrap();
            *app_data_dir_override().lock().unwrap() = Some(root.clone());
            Self {
                _guard: guard,
                root,
            }
        }
    }

    impl Drop for TestEnv {
        fn drop(&mut self) {
            set_test_config_write_failure(false);
            *app_data_dir_override().lock().unwrap() = None;
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    fn library_fixture(
        id: &str,
        storage_dir: Option<String>,
        metadata_initialized: bool,
    ) -> Library {
        Library {
            id: id.to_string(),
            name: format!("Library {}", id),
            created_at: 0,
            state: LibraryState::default(),
            hidden: false,
            storage_dir,
            metadata_initialized,
        }
    }

    #[test]
    fn legacy_library_config_deserializes_without_storage_fields() {
        let json = r#"{
          "debug": false,
          "current_library_id": "default",
          "libraries": [{
            "id": "default",
            "name": "Default Library",
            "created_at": 123,
            "hidden": false
          }]
        }"#;

        let config: AppConfig = serde_json::from_str(json).unwrap();
        let library = &config.libraries[0];
        assert_eq!(library.storage_dir, None);
        assert!(!library.metadata_initialized);
    }

    #[test]
    fn canonicalize_custom_storage_dir_requires_absolute_existing_directory() {
        let env = TestEnv::new();
        let custom_dir = env.root.join("external");
        fs::create_dir_all(&custom_dir).unwrap();

        let canonical =
            canonicalize_custom_storage_dir(Some(custom_dir.to_string_lossy().into_owned()))
                .unwrap()
                .unwrap();
        assert_eq!(
            PathBuf::from(canonical),
            fs::canonicalize(&custom_dir).unwrap()
        );

        let relative_error =
            canonicalize_custom_storage_dir(Some("relative/path".to_string())).unwrap_err();
        assert!(relative_error.contains("absolute path"));
    }

    #[test]
    fn remove_library_rejects_disconnected_initialized_storage() {
        let _env = TestEnv::new();
        let unavailable_dir = std::env::temp_dir().join("lap-unavailable-storage");

        let config = AppConfig {
            debug: false,
            current_library_id: "default".to_string(),
            libraries: vec![
                library_fixture("default", None, false),
                library_fixture(
                    "external",
                    Some(unavailable_dir.to_string_lossy().into_owned()),
                    true,
                ),
            ],
        };
        save_app_config(&config).unwrap();

        let error = remove_library("external").unwrap_err();
        assert!(error.contains("Reconnect the storage before deleting"));

        let reloaded = load_app_config().unwrap();
        assert!(
            reloaded
                .libraries
                .iter()
                .any(|library| library.id == "external")
        );
    }

    #[test]
    fn remove_library_allows_disconnected_uninitialized_storage() {
        let _env = TestEnv::new();
        let unavailable_dir = std::env::temp_dir().join("lap-unavailable-storage-empty");

        let config = AppConfig {
            debug: false,
            current_library_id: "default".to_string(),
            libraries: vec![
                library_fixture("default", None, false),
                library_fixture(
                    "external",
                    Some(unavailable_dir.to_string_lossy().into_owned()),
                    false,
                ),
            ],
        };
        save_app_config(&config).unwrap();

        remove_library("external").unwrap();

        let reloaded = load_app_config().unwrap();
        assert!(
            !reloaded
                .libraries
                .iter()
                .any(|library| library.id == "external")
        );
    }

    #[test]
    fn disconnected_metadata_policy_is_shared() {
        let unavailable_dir = std::env::temp_dir().join("lap-disconnected-policy");
        let empty_library = library_fixture(
            "empty",
            Some(unavailable_dir.to_string_lossy().into_owned()),
            false,
        );
        let initialized_library = library_fixture(
            "initialized",
            Some(unavailable_dir.to_string_lossy().into_owned()),
            true,
        );

        assert!(library_allows_disconnected_metadata_operation(&empty_library).unwrap());
        assert!(!library_allows_disconnected_metadata_operation(&initialized_library).unwrap());
    }

    #[test]
    fn library_storage_write_gate_blocks_while_move_is_active() {
        let _env = TestEnv::new();
        let _move_guard = start_library_storage_move("default").unwrap();

        let error = ensure_library_storage_write_allowed().unwrap_err();
        assert_eq!(error, LIBRARY_STORAGE_MOVE_IN_PROGRESS_ERROR);
    }

    #[test]
    fn remove_library_rolls_back_quarantined_files_when_config_save_fails() {
        let _env = TestEnv::new();
        let config = AppConfig {
            debug: false,
            current_library_id: "default".to_string(),
            libraries: vec![
                library_fixture("default", None, false),
                library_fixture("external", None, true),
            ],
        };
        save_app_config(&config).unwrap();

        let libraries_dir = get_libraries_dir().unwrap();
        fs::create_dir_all(&libraries_dir).unwrap();
        let db_path = libraries_dir.join("external.db");
        let wal_path = PathBuf::from(format!("{}-wal", db_path.to_string_lossy()));
        let shm_path = PathBuf::from(format!("{}-shm", db_path.to_string_lossy()));
        fs::write(&db_path, b"db").unwrap();
        fs::write(&wal_path, b"wal").unwrap();
        fs::write(&shm_path, b"shm").unwrap();

        set_test_config_write_failure(true);
        let error = remove_library("external").unwrap_err();
        set_test_config_write_failure(false);

        assert!(error.contains("Injected config write failure"));
        let reloaded = load_app_config().unwrap();
        assert!(
            reloaded
                .libraries
                .iter()
                .any(|library| library.id == "external")
        );
        assert!(db_path.exists());
        assert!(wal_path.exists());
        assert!(shm_path.exists());
    }

    #[test]
    fn create_db_fails_if_metadata_initialized_cannot_be_persisted() {
        let _env = TestEnv::new();
        let config = AppConfig {
            debug: false,
            current_library_id: "default".to_string(),
            libraries: vec![library_fixture("default", None, false)],
        };
        save_app_config(&config).unwrap();

        set_test_config_write_failure(true);
        let error = crate::t_sqlite::create_db().unwrap_err();
        set_test_config_write_failure(false);

        assert!(error.contains("Injected config write failure"));
        let db_path = get_current_db_path().unwrap();
        assert!(Path::new(&db_path).exists());

        let reloaded = load_app_config().unwrap();
        let library = reloaded
            .libraries
            .iter()
            .find(|library| library.id == "default")
            .unwrap();
        assert!(!library.metadata_initialized);
    }
}

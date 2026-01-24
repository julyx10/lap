/**
 * App configuration management.
 * Handles app-config.json for multi-library support.
 * project: Lap
 * author:  julyxx
 * date:    2025-01-15
 */
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

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
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct TagState {
    pub id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CalendarState {
    pub year: Option<i32>,
    pub month: Option<i32>,
    pub date: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CameraState {
    pub make: Option<String>,
    pub model: Option<String>,
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
    pub current_library_id: String,
    pub libraries: Vec<Library>,
}

impl Default for AppConfig {
    fn default() -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
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
    dirs::data_local_dir()
        .ok_or_else(|| "Failed to get the local AppData directory".to_string())
        .map(|p| p.join("com.julyxx.lap"))
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
    let config_path = get_config_file_path()?;

    if config_path.exists() {
        let content = fs::read_to_string(&config_path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;
        let config: AppConfig = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse config file: {}", e))?;
        Ok(config)
    } else {
        // Create default config
        let config = AppConfig::default();
        save_app_config(&config)?;
        Ok(config)
    }
}

/// Save app config to file
pub fn save_app_config(config: &AppConfig) -> Result<(), String> {
    let config_path = get_config_file_path()?;
    let content = serde_json::to_string_pretty(config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;
    fs::write(&config_path, content).map_err(|e| format!("Failed to write config file: {}", e))?;
    Ok(())
}

/// Get the database file path for a library
pub fn get_library_db_path(library_id: &str) -> Result<String, String> {
    let lib_dir = get_libraries_dir()?;
    let db_path = lib_dir.join(format!("{}.db", library_id));
    Ok(db_path.to_string_lossy().into_owned())
}

/// Get the current library's database file path
pub fn get_current_db_path() -> Result<String, String> {
    let config = load_app_config()?;
    get_library_db_path(&config.current_library_id)
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

    // Delete the database file
    let db_path = get_library_db_path(id)?;
    if std::path::Path::new(&db_path).exists() {
        fs::remove_file(&db_path).map_err(|e| format!("Failed to delete database file: {}", e))?;
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
pub struct LibraryInfo {
    pub db_file_size: i64,
    pub db_file_path: String,
}

pub fn get_library_info(id: &str) -> Result<LibraryInfo, String> {
    let db_path = get_library_db_path(id)?;

    // Get db file size
    let db_file_size = if std::path::Path::new(&db_path).exists() {
        fs::metadata(&db_path).map(|m| m.len() as i64).unwrap_or(0)
    } else {
        0
    };

    Ok(LibraryInfo {
        db_file_size,
        db_file_path: db_path,
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
    if config.current_library_id == id && hidden {
        return Err("Cannot hide the current library".to_string());
    }

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

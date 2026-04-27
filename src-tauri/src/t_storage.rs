/**
 * Database storage, backup, and restore operations.
 * Handles custom DB storage directory, migration, backup/restore of library databases.
 * project: Lap
 * author:  julyx10
 * date:    2026-01-15
 */
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use uuid::Uuid;
use zip::read::ZipArchive;
use zip::write::FileOptions;
use zip::ZipWriter;

use crate::t_config::{self, AppConfig, Library, LibraryState};

static DB_MIGRATION_IN_PROGRESS: AtomicBool = AtomicBool::new(false);

struct DbMigrationGuard;

impl DbMigrationGuard {
    fn acquire() -> Result<Self, String> {
        DB_MIGRATION_IN_PROGRESS
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .map_err(|_| "Database storage migration is already in progress.".to_string())?;
        Ok(Self)
    }
}

impl Drop for DbMigrationGuard {
    fn drop(&mut self) {
        DB_MIGRATION_IN_PROGRESS.store(false, Ordering::SeqCst);
    }
}

fn get_db_storage_dir_from_config(config: &AppConfig) -> Result<PathBuf, String> {
    if let Some(dir) = config.db_storage_dir.as_deref() {
        let path = PathBuf::from(dir);
        fs::create_dir_all(&path)
            .map_err(|e| format!("Failed to create database storage directory: {}", e))?;
        Ok(path)
    } else {
        t_config::get_libraries_dir()
    }
}

fn get_library_db_path_from_config(config: &AppConfig, library_id: &str) -> Result<String, String> {
    let db_dir = get_db_storage_dir_from_config(config)?;
    Ok(db_dir
        .join(format!("{}.db", library_id))
        .to_string_lossy()
        .into_owned())
}

pub fn get_db_storage_dir() -> Result<String, String> {
    let config = t_config::load_app_config()?;
    get_db_storage_dir_from_config(&config).map(|p| p.to_string_lossy().into_owned())
}

pub fn is_using_custom_db_storage() -> Result<bool, String> {
    let config = t_config::load_app_config()?;
    Ok(config.db_storage_dir.is_some())
}

pub fn is_db_migration_in_progress() -> bool {
    DB_MIGRATION_IN_PROGRESS.load(Ordering::SeqCst)
}

/// Get the database file path for a library
pub fn get_library_db_path(library_id: &str) -> Result<String, String> {
    if is_db_migration_in_progress() {
        return Err("Database storage migration is in progress.".to_string());
    }
    let config = t_config::load_app_config()?;
    get_library_db_path_from_config(&config, library_id)
}

/// Get the current library's database file path
pub fn get_current_db_path() -> Result<String, String> {
    if is_db_migration_in_progress() {
        return Err("Database storage migration is in progress.".to_string());
    }
    let config = t_config::load_app_config()?;
    get_library_db_path_from_config(&config, &config.current_library_id)
}

fn checkpoint_db(path: &Path) -> Result<(), String> {
    if !path.exists() {
        return Ok(());
    }

    let conn = rusqlite::Connection::open(path)
        .map_err(|e| format!("Failed to open database for checkpoint: {}", e))?;
    conn.busy_timeout(Duration::from_secs(5))
        .map_err(|e| format!("Failed to set SQLite busy timeout: {}", e))?;

    let run_checkpoint = |mode: &str| -> Result<(), String> {
        let pragma = format!("PRAGMA wal_checkpoint({})", mode);
        conn.query_row(&pragma, [], |_| Ok(()))
            .map_err(|e| format!("Failed to checkpoint database with mode {}: {}", mode, e))
    };

    if let Err(truncate_err) = run_checkpoint("TRUNCATE") {
        eprintln!("{}", truncate_err);
        run_checkpoint("RESTART")?;
    }

    Ok(())
}

pub fn change_db_storage_dir(new_dir: &str) -> Result<String, String> {
    let _migration_guard = DbMigrationGuard::acquire()?;
    let mut config = t_config::load_app_config()?;
    let target_dir = PathBuf::from(new_dir);

    fs::create_dir_all(&target_dir)
        .map_err(|e| format!("Failed to create target database directory: {}", e))?;

    let current_dir = get_db_storage_dir_from_config(&config)?;
    let current_dir_canon = fs::canonicalize(&current_dir).unwrap_or(current_dir.clone());
    let target_dir_canon = fs::canonicalize(&target_dir).unwrap_or(target_dir.clone());
    if current_dir_canon == target_dir_canon {
        return Ok(target_dir_canon.to_string_lossy().into_owned());
    }

    for library in &config.libraries {
        let source_path = PathBuf::from(get_library_db_path_from_config(&config, &library.id)?);
        let target_path = target_dir.join(format!("{}.db", library.id));

        if !source_path.exists() {
            continue;
        }

        checkpoint_db(&source_path)?;

        if target_path.exists() {
            fs::remove_file(&target_path)
                .map_err(|e| format!("Failed to replace existing target database: {}", e))?;
        }

        fs::copy(&source_path, &target_path)
            .map_err(|e| format!("Failed to migrate database '{}': {}", library.name, e))?;
    }

    config.db_storage_dir = Some(target_dir_canon.to_string_lossy().into_owned());
    t_config::save_app_config(&config)?;

    for library in &config.libraries {
        let source_path = PathBuf::from(current_dir.join(format!("{}.db", library.id)));
        if source_path.exists() {
            let _ = fs::remove_file(&source_path);
        }
        let wal_path = current_dir.join(format!("{}.db-wal", library.id));
        if wal_path.exists() {
            let _ = fs::remove_file(&wal_path);
        }
        let shm_path = current_dir.join(format!("{}.db-shm", library.id));
        if shm_path.exists() {
            let _ = fs::remove_file(&shm_path);
        }
    }

    Ok(target_dir_canon.to_string_lossy().into_owned())
}

pub fn reset_db_storage_dir() -> Result<String, String> {
    let _migration_guard = DbMigrationGuard::acquire()?;
    let mut config = t_config::load_app_config()?;
    let target_dir = t_config::get_libraries_dir()?;
    let current_dir = get_db_storage_dir_from_config(&config)?;
    let current_dir_canon = fs::canonicalize(&current_dir).unwrap_or(current_dir.clone());
    let target_dir_canon = fs::canonicalize(&target_dir).unwrap_or(target_dir.clone());

    if current_dir_canon == target_dir_canon {
        config.db_storage_dir = None;
        t_config::save_app_config(&config)?;
        return Ok(target_dir_canon.to_string_lossy().into_owned());
    }

    for library in &config.libraries {
        let source_path = PathBuf::from(get_library_db_path_from_config(&config, &library.id)?);
        let target_path = target_dir.join(format!("{}.db", library.id));

        if !source_path.exists() {
            continue;
        }

        checkpoint_db(&source_path)?;

        if target_path.exists() {
            fs::remove_file(&target_path)
                .map_err(|e| format!("Failed to replace existing target database: {}", e))?;
        }

        fs::copy(&source_path, &target_path)
            .map_err(|e| format!("Failed to migrate database '{}': {}", library.name, e))?;
    }

    config.db_storage_dir = None;
    t_config::save_app_config(&config)?;

    for library in &config.libraries {
        let source_path = PathBuf::from(current_dir.join(format!("{}.db", library.id)));
        if source_path.exists() {
            let _ = fs::remove_file(&source_path);
        }
        let wal_path = current_dir.join(format!("{}.db-wal", library.id));
        if wal_path.exists() {
            let _ = fs::remove_file(&wal_path);
        }
        let shm_path = current_dir.join(format!("{}.db-shm", library.id));
        if shm_path.exists() {
            let _ = fs::remove_file(&shm_path);
        }
    }

    Ok(target_dir_canon.to_string_lossy().into_owned())
}

// ============================================================================
// Backup / Restore
// ============================================================================

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DbStorageInfo {
    pub library_id: String,
    pub library_name: String,
    pub db_file_size: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupResult {
    pub file_path: String,
    pub file_size: i64,
    pub library_count: usize,
    pub library_names: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupMetaLibrary {
    pub name: String,
    pub db_size: i64,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupMetaData {
    pub created_at: i64,
    pub libraries: Vec<BackupMetaLibrary>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreSelection {
    pub library_name: String,
    pub should_rename: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreResult {
    pub restored_count: usize,
    pub restored_names: Vec<String>,
}

pub fn get_db_storage_info() -> Result<Vec<DbStorageInfo>, String> {
    let config = t_config::load_app_config()?;

    let mut results = Vec::new();
    for lib in &config.libraries {
        let db_path = get_library_db_path_from_config(&config, &lib.id)?;
        let db_file_size = {
            let p = Path::new(&db_path);
            if p.exists() {
                fs::metadata(p).ok().map(|m| m.len() as i64).unwrap_or(0)
            } else {
                0
            }
        };

        results.push(DbStorageInfo {
            library_id: lib.id.clone(),
            library_name: lib.name.clone(),
            db_file_size,
        });
    }

    Ok(results)
}

pub fn backup_databases(library_ids: &[String], dest_path: &str) -> Result<BackupResult, String> {
    let _migration_guard = DbMigrationGuard::acquire()?;
    let config = t_config::load_app_config()?;

    let selected: Vec<&Library> = library_ids
        .iter()
        .filter_map(|id| config.libraries.iter().find(|l| l.id == *id))
        .collect();

    if selected.is_empty() {
        return Err("No libraries selected for backup.".to_string());
    }

    for lib in &selected {
        let db_path = get_library_db_path_from_config(&config, &lib.id)?;
        checkpoint_db(Path::new(&db_path))?;
    }

    let file = fs::File::create(dest_path)
        .map_err(|e| format!("Failed to create backup file: {}", e))?;
    let mut zip = ZipWriter::new(file);

    let backup_info = serde_json::json!({
        "createdAt": SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0),
        "libraries": selected.iter().map(|lib| {
            let db_path = get_library_db_path_from_config(&config, &lib.id).ok();
            let size = db_path.as_ref()
                .and_then(|p| fs::metadata(p).ok())
                .map(|m| m.len() as i64)
                .unwrap_or(0);
            serde_json::json!({ "name": lib.name, "dbSize": size })
        }).collect::<Vec<_>>()
    });

    let info_json = serde_json::to_string_pretty(&backup_info)
        .map_err(|e| format!("Failed to serialize backup info: {}", e))?;

    let options = FileOptions::<'_, ()>::default()
        .compression_method(zip::CompressionMethod::Deflated);
    zip.start_file("backup-info.json", options)
        .map_err(|e| format!("Failed to write backup-info.json to zip: {}", e))?;
    zip.write_all(info_json.as_bytes())
        .map_err(|e| format!("Failed to write backup-info.json content: {}", e))?;

    let app_config_path = t_config::get_config_file_path()?;
    if app_config_path.exists() {
        let mut config_content = String::new();
        fs::File::open(&app_config_path)
            .map_err(|e| format!("Failed to read config file: {}", e))?
            .read_to_string(&mut config_content)
            .map_err(|e| format!("Failed to read config file: {}", e))?;

        let options = FileOptions::<'_, ()>::default()
            .compression_method(zip::CompressionMethod::Deflated);
        zip.start_file("app-config.json", options)
            .map_err(|e| format!("Failed to write app-config.json to zip: {}", e))?;
        zip.write_all(config_content.as_bytes())
            .map_err(|e| format!("Failed to write app-config.json content: {}", e))?;
    }

    let mut library_names = Vec::new();
    for lib in &selected {
        let db_path = get_library_db_path_from_config(&config, &lib.id)?;
        let path = Path::new(&db_path);
        if !path.exists() {
            continue;
        }

        let mut db_content = Vec::new();
        fs::File::open(path)
            .map_err(|e| format!("Failed to read database '{}': {}", lib.name, e))?
            .read_to_end(&mut db_content)
            .map_err(|e| format!("Failed to read database '{}': {}", lib.name, e))?;

        let safe_name = sanitize_filename(&lib.name);
        let zip_path = format!("{}.db", safe_name);

        let options = FileOptions::<'_, ()>::default()
            .compression_method(zip::CompressionMethod::Deflated);
        zip.start_file(&zip_path, options)
            .map_err(|e| format!("Failed to write {} to zip: {}", zip_path, e))?;
        zip.write_all(&db_content)
            .map_err(|e| format!("Failed to write {} content: {}", zip_path, e))?;

        library_names.push(lib.name.clone());
    }

    let file = zip
        .finish()
        .map_err(|e| format!("Failed to finalize backup zip: {}", e))?;
    let final_size = file
        .metadata()
        .map(|m| m.len() as i64)
        .unwrap_or(0);

    Ok(BackupResult {
        file_path: dest_path.to_string(),
        file_size: final_size,
        library_count: library_names.len(),
        library_names,
    })
}

pub fn parse_backup_file(path: &str) -> Result<BackupMetaData, String> {
    let file = fs::File::open(path)
        .map_err(|e| format!("Failed to open backup file: {}", e))?;
    let mut archive = ZipArchive::new(file)
        .map_err(|e| format!("Failed to read backup zip: {}", e))?;

    let info_index = archive.index_for_name("backup-info.json")
        .ok_or_else(|| "Invalid backup file: missing backup-info.json".to_string())?;
    let mut info_file = archive.by_index(info_index)
        .map_err(|e| format!("Failed to read backup-info.json: {}", e))?;
    let mut info_content = String::new();
    info_file.read_to_string(&mut info_content)
        .map_err(|e| format!("Failed to read backup-info.json: {}", e))?;

    let info: BackupMetaData = serde_json::from_str(&info_content)
        .map_err(|e| format!("Failed to parse backup-info.json: {}", e))?;

    Ok(info)
}

pub fn restore_databases(
    backup_path: &str,
    selections: &[RestoreSelection],
) -> Result<RestoreResult, String> {
    let _migration_guard = DbMigrationGuard::acquire()?;
    let mut config = t_config::load_app_config()?;

    let file = fs::File::open(backup_path)
        .map_err(|e| format!("Failed to open backup file: {}", e))?;
    let mut archive = ZipArchive::new(file)
        .map_err(|e| format!("Failed to read backup zip: {}", e))?;

    let mut db_entries: std::collections::HashMap<String, Vec<u8>> = std::collections::HashMap::new();
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)
            .map_err(|e| format!("Failed to read zip entry: {}", e))?;
        let entry_name = entry.name().to_string();
        if !entry_name.ends_with(".db") {
            continue;
        }
        let lib_name = entry_name.trim_end_matches(".db").to_string();
        let mut content = Vec::new();
        entry.read_to_end(&mut content)
            .map_err(|e| format!("Failed to read zip entry '{}': {}", entry_name, e))?;
        db_entries.insert(lib_name, content);
    }

    let mut restored_names = Vec::new();
    let mut existing_names: std::collections::HashSet<String> = config
        .libraries
        .iter()
        .map(|l| l.name.clone())
        .collect();

    for selection in selections {
        let zip_lib_name = sanitize_filename(&selection.library_name);
        let Some(db_content) = db_entries.remove(&zip_lib_name) else {
            continue;
        };

        let final_lib_name = if selection.should_rename {
            resolve_unique_name(&selection.library_name, &existing_names)
        } else {
            selection.library_name.clone()
        };

        let lib_id = Uuid::new_v4().to_string();
        let now = chrono::Utc::now().timestamp();
        let new_lib = Library {
            id: lib_id.clone(),
            name: final_lib_name.clone(),
            created_at: now,
            state: LibraryState::default(),
            hidden: false,
        };

        let db_path = get_library_db_path_from_config(&config, &lib_id)?;
        let db_path_obj = Path::new(&db_path);
        if let Some(parent) = db_path_obj.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create directory for library: {}", e))?;
        }
        fs::write(db_path_obj, &db_content)
            .map_err(|e| format!("Failed to write database file for '{}': {}", final_lib_name, e))?;

        config.libraries.push(new_lib);
        existing_names.insert(final_lib_name.clone());
        restored_names.push(final_lib_name);
    }

    t_config::save_app_config(&config)?;
    Ok(RestoreResult {
        restored_count: restored_names.len(),
        restored_names,
    })
}

fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' || c == ' ' || c == '.' {
                c
            } else {
                '_'
            }
        })
        .collect::<String>()
        .trim()
        .to_string()
}

fn resolve_unique_name(name: &str, existing: &std::collections::HashSet<String>) -> String {
    if !existing.contains(name) {
        return name.to_string();
    }
    for i in 1..=999 {
        let candidate = format!("{} ({})", name, i);
        if !existing.contains(&candidate) {
            return candidate;
        }
    }
    format!("{} ({})", name, rand::random::<u16>())
}

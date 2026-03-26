/**
 * General utility functions.
 * project: Lap
 * author:  julyx10
 * date:    2024-08-08
 */
use crate::t_common;
use crate::t_sqlite::{AFile, Album};
use chrono::{DateTime, Local, TimeZone};
use once_cell::sync::Lazy;
use pinyin::ToPinyin;
use reverse_geocoder::ReverseGeocoder;
use std::collections::HashMap;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{Emitter, Manager, State};
use walkdir::WalkDir; // https://docs.rs/walkdir/2.5.0/walkdir/

// reverse geocoder
pub static GEOCODER: Lazy<ReverseGeocoder> = Lazy::new(|| {
    println!("Initializing ReverseGeocoder...");
    ReverseGeocoder::new()
});

// #[cfg(target_os = "windows")]
// use std::os::windows::fs::MetadataExt; // Windows-specific extensions

#[derive(serde::Serialize)]
pub struct PackageInfo {
    name: String,
    version: String,
    description: String,
    authors: Vec<String>,
    repository: Option<String>,
    license: Option<String>,
    homepage: Option<String>,
}

impl PackageInfo {
    pub fn new() -> Self {
        Self {
            name: env!("CARGO_PKG_NAME").to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            description: env!("CARGO_PKG_DESCRIPTION").to_string(),
            authors: env!("CARGO_PKG_AUTHORS")
                .split(':')
                .map(|s| s.to_string())
                .collect(),
            repository: option_env!("CARGO_PKG_REPOSITORY").map(|s| s.to_string()),
            license: option_env!("CARGO_PKG_LICENSE").map(|s| s.to_string()),
            homepage: option_env!("CARGO_PKG_HOMEPAGE").map(|s| s.to_string()),
        }
    }
}

/// FileNode struct to represent a file system node
#[derive(serde::Serialize)]
pub struct FileNode {
    id: Option<i64>, // unique id(in database)
    name: String,    // folder name
    path: String,    // folder path
    is_dir: bool,    // is directory
    is_expanded: bool,
    children: Option<Vec<Self>>,
}

impl FileNode {
    /// Create a new FileNode
    fn new(path: &str, is_dir: bool, is_expanded: bool) -> Self {
        FileNode {
            id: None,
            name: get_file_name(path),
            path: path.to_string(),
            is_dir,
            is_expanded,
            children: None,
        }
    }

    /// Read folders from a path and build a FileNode
    pub fn build_nodes(path: &str, is_recursive: bool) -> Result<Self, String> {
        let root_path = Path::new(&path);

        // Check if the path exists and is a directory
        if !root_path.exists() {
            return Err(format!("Path does not exist: {}", path));
        }

        if !root_path.is_dir() {
            return Err(format!("Path is not a directory: {}", path));
        }

        // Create the root FileNode
        let mut root_node = FileNode::new(path, root_path.is_dir(), false);

        // Recursively read subfolders and files
        root_node.children = Some(Self::recurse_nodes(root_path, is_recursive)?);

        Ok(root_node)
    }

    /// Recurse sub-folders
    fn recurse_nodes(path: &Path, is_recursive: bool) -> Result<Vec<Self>, String> {
        let mut nodes: Vec<FileNode> = Vec::new();

        for entry in WalkDir::new(path)
            .min_depth(1)
            .max_depth(1)
            .into_iter()
            .filter_entry(|e| !Self::is_hidden(e))
        {
            let entry = entry.map_err(|e| e.to_string())?;
            let entry_path = entry.path();
            let path_str = entry_path.to_string_lossy().to_string();

            if entry.file_type().is_dir() {
                let mut node = FileNode::new(&path_str, true, false);

                if is_recursive {
                    node.children = Some(Self::recurse_nodes(entry_path, is_recursive)?);
                }

                nodes.push(node);
            }
        }

        // Sort the nodes by name
        nodes.sort_by(|a, b| {
            convert_to_pinyin(&a.name)
                .to_lowercase()
                .cmp(&convert_to_pinyin(&b.name).to_lowercase())
        });
        Ok(nodes)
    }

    fn is_hidden(entry: &walkdir::DirEntry) -> bool {
        entry
            .file_name()
            .to_str()
            .map(|s| s.starts_with('.'))
            .unwrap_or(false)
    }
}

// file metadata struct
#[derive(serde::Serialize)]
pub struct FileInfo {
    pub file_path: String,
    pub file_name: String,
    pub file_type: Option<String>, // file type (dir, file)
    pub created: Option<i64>,
    pub modified: Option<i64>, // modified date as a timestamp
    pub file_size: i64,
}

impl FileInfo {
    /// Get file info from a folder/file path (on Windows)
    pub fn new(file_path: &str) -> Result<Self, String> {
        // Convert the string path into a Path object
        let path = Path::new(file_path);
        let metadata = fs::metadata(path).map_err(|e| e.to_string())?;

        Ok(FileInfo {
            file_path: file_path.to_string(),
            file_name: get_file_name(file_path),
            file_type: metadata.file_type().is_dir().then(|| "dir".to_string()),
            created: systemtime_to_timestamp(metadata.created().ok()),
            modified: systemtime_to_timestamp(metadata.modified().ok()),
            file_size: metadata.len() as i64,
        })
    }
}

pub fn authorize_directory_scope(
    app_handle: &tauri::AppHandle,
    dir_path: &str,
) -> Result<(), String> {
    app_handle
        .state::<tauri::Scopes>()
        .allow_directory(dir_path, true)
        .map_err(|e| format!("Failed to authorize directory '{}': {}", dir_path, e))
}

pub fn restore_album_scopes(app_handle: &tauri::AppHandle) -> Result<(), String> {
    let albums = Album::get_all_albums()?;

    for album in albums {
        authorize_directory_scope(app_handle, &album.path)?;
    }

    Ok(())
}

fn normalize_external_app_name(name: &str) -> String {
    let known_suffixes = [
        ".appimage",
        ".desktop",
        ".bundle",
        ".app",
        ".exe",
        ".lnk",
        ".cmd",
        ".bat",
    ];

    let trimmed_name = name.trim();
    let lower_name = trimmed_name.to_lowercase();
    for suffix in known_suffixes {
        if lower_name.ends_with(suffix) {
            let trimmed = &trimmed_name[..trimmed_name.len() - suffix.len()];
            return trimmed.to_string();
        }
    }

    trimmed_name.to_string()
}

fn fallback_external_app_name(app_path: &str) -> String {
    let clean_path = app_path.trim().trim_end_matches(['/', '\\']);
    let fallback = Path::new(clean_path)
        .file_name()
        .or_else(|| Path::new(clean_path).file_stem())
        .and_then(|name| name.to_str())
        .unwrap_or(clean_path);

    normalize_external_app_name(fallback)
}

fn command_stdout(mut command: Command) -> Option<String> {
    let output = command.output().ok()?;
    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if stdout.is_empty() || stdout == "(null)" {
        None
    } else {
        Some(stdout)
    }
}

#[cfg(target_os = "macos")]
fn resolve_external_app_display_name(app_path: &str) -> Option<String> {
    command_stdout({
        let mut command = Command::new("mdls");
        command.args(["-name", "kMDItemDisplayName", "-raw", app_path]);
        command
    })
}

#[cfg(target_os = "windows")]
fn resolve_external_app_display_name(app_path: &str) -> Option<String> {
    let script = r#"
$path = $env:LAP_APP_PATH
if (-not $path) { exit 1 }

function Get-ResolvedTarget([string]$candidate) {
  if (-not $candidate) { return $null }
  if ($candidate.ToLower().EndsWith('.lnk')) {
    try {
      $shell = New-Object -ComObject WScript.Shell
      $shortcut = $shell.CreateShortcut($candidate)
      if ($shortcut.TargetPath) { return $shortcut.TargetPath }
    } catch {}
  }
  return $candidate
}

$candidate = Get-ResolvedTarget $path
if (-not $candidate) { exit 1 }

try {
  $item = Get-Item -LiteralPath $candidate -ErrorAction Stop
  $info = $item.VersionInfo
  if ($info) {
    if ($info.FileDescription) {
      [Console]::OutputEncoding = [System.Text.Encoding]::UTF8
      Write-Output $info.FileDescription
      exit 0
    }
    if ($info.ProductName) {
      [Console]::OutputEncoding = [System.Text.Encoding]::UTF8
      Write-Output $info.ProductName
      exit 0
    }
  }
} catch {}

exit 1
"#;

    command_stdout({
        let mut command = Command::new("powershell");
        command
            .args(["-NoProfile", "-Command", script])
            .env("LAP_APP_PATH", app_path);
        command
    })
}

#[cfg(target_os = "linux")]
fn resolve_external_app_display_name(app_path: &str) -> Option<String> {
    let path = Path::new(app_path);
    let lower_path = app_path.to_lowercase();
    if lower_path.ends_with(".desktop") {
        let desktop_file = fs::read_to_string(path).ok()?;
        for line in desktop_file.lines() {
            if let Some(name) = line.strip_prefix("Name=") {
                let trimmed = name.trim();
                if !trimmed.is_empty() {
                    return Some(trimmed.to_string());
                }
            }
        }
    }

    None
}

#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
fn resolve_external_app_display_name(_app_path: &str) -> Option<String> {
    None
}

pub fn get_external_app_display_name(app_path: &str) -> Result<String, String> {
    if app_path.trim().is_empty() {
        return Err("Missing app path".to_string());
    }

    Ok(resolve_external_app_display_name(app_path)
        .filter(|name| !name.trim().is_empty())
        .map(|name| normalize_external_app_name(&name))
        .unwrap_or_else(|| fallback_external_app_name(app_path)))
}

/// create a new folder at a given path
/// Returns the folder path if successful
pub fn create_new_folder(folder_path: &str) -> Option<String> {
    let path = Path::new(folder_path);

    if path.exists() {
        eprintln!("Folder already exists: {}", folder_path);
        return None;
    }

    match fs::create_dir_all(path) {
        Ok(_) => {
            println!("Folder created successfully: {}", folder_path);
            let folder_name = path.to_string_lossy().into_owned();
            Some(folder_name)
        }
        Err(e) => {
            eprintln!("Failed to create folder '{}': {}", folder_path, e);
            None
        }
    }
}

/// Renames a folder
/// Returns the new folder path if successful
pub fn rename_folder(folder_path: &str, new_folder_name: &str) -> Option<String> {
    let path = Path::new(folder_path);

    // Check if the folder exists
    if !path.exists() {
        eprintln!("Folder does not exist: {}", folder_path);
        return None;
    }

    // Construct the new folder path
    let mut new_folder_path = PathBuf::from(path);
    new_folder_path.set_file_name(new_folder_name);

    // Check if the new folder name already exists
    if new_folder_path.exists() {
        eprintln!(
            "A file or folder with the name '{}' already exists at: {}",
            new_folder_name,
            new_folder_path.display()
        );
        return None;
    }

    // Attempt to rename the folder
    match fs::rename(path, &new_folder_path) {
        Ok(_) => {
            let new_path_str = new_folder_path.to_string_lossy().into_owned();
            println!("Folder renamed successfully: {}", new_path_str);
            Some(new_path_str)
        }
        Err(e) => {
            eprintln!("Failed to rename folder '{}': {}", folder_path, e);
            None
        }
    }
}

/// Checks if a path exists, and if so, returns a new unique path
/// by appending a number like (1), (2), etc.
fn get_unique_path(path: PathBuf) -> PathBuf {
    if !path.exists() {
        return path;
    }

    let parent_dir = path.parent().unwrap_or_else(|| Path::new(""));
    let stem_os = path.file_stem().unwrap_or_default();
    let stem = stem_os.to_string_lossy();
    let ext_os = path.extension().unwrap_or_default();
    let ext = ext_os.to_string_lossy();

    let mut i = 1;
    loop {
        let new_name = if ext.is_empty() {
            format!("{}({})", stem, i)
        } else {
            format!("{}({}).{}", stem, i, ext)
        };

        let new_path = parent_dir.join(&new_name);

        if !new_path.exists() {
            return new_path;
        }
        i += 1;
    }
}

/// move a folder to a new location
/// Returns the new folder path if successful
pub fn move_folder(folder_path: &str, dest_folder: &str) -> Option<String> {
    let path = Path::new(folder_path);
    let mut destination = Path::new(dest_folder).to_path_buf();

    // Ensure the source folder exists
    if !path.exists() {
        eprintln!("Folder does not exist: {}", folder_path);
        return None;
    }

    // Append the folder name to the new folder path
    if let Some(folder_name) = path.file_name() {
        destination.push(folder_name);
    } else {
        eprintln!("Invalid folder name: {}", folder_path);
        return None;
    }

    let destination = get_unique_path(destination);

    // Attempt to move the folder and return result
    fs::rename(path, &destination).map_or_else(
        |e| {
            eprintln!("Failed to move folder: {}", e);
            None
        },
        |_| {
            println!("Folder moved to: {}", destination.display());
            Some(destination.to_string_lossy().into_owned())
        },
    )
}

/// Recursively copies a folder and all its contents to a new location.
/// Returns Some(new_folder_path) if successful, or None on failure.
pub fn copy_folder(folder_path: &str, dest_folder: &str) -> Option<String> {
    let src = Path::new(folder_path);
    let mut dst = Path::new(dest_folder).to_path_buf();

    // Check if the source folder exists and is a directory
    if !src.exists() || !src.is_dir() {
        eprintln!(
            "Source folder does not exist or is not a directory: {}",
            folder_path
        );
        return None;
    }

    // Get the name of the folder from folder_path
    if let Some(folder_name) = src.file_name() {
        // Append the folder name to the new folder path
        dst.push(folder_name);
    } else {
        eprintln!("Failed to get the folder name from path: {}", folder_path);
        return None;
    }

    let dst = get_unique_path(dst);

    // Create the destination folder if it does not exist
    if let Err(e) = fs::create_dir_all(&dst) {
        eprintln!(
            "Failed to create destination folder '{}': {}",
            dst.display(),
            e
        );
        return None;
    }

    // Walk through the source folder and copy its contents
    for entry in WalkDir::new(src) {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                eprintln!("Error reading entry: {}", e);
                return None;
            }
        };

        let relative_path = match entry.path().strip_prefix(src) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Error computing relative path: {}", e);
                return None;
            }
        };

        let dest_path = dst.join(relative_path);

        if entry.file_type().is_dir() {
            if let Err(e) = fs::create_dir_all(&dest_path) {
                eprintln!(
                    "Failed to create directory '{}': {}",
                    dest_path.display(),
                    e
                );
                return None;
            }
        } else if let Err(e) = fs::copy(entry.path(), &dest_path) {
            eprintln!("Failed to copy file to '{}': {}", dest_path.display(), e);
            return None;
        }
    }

    println!("Folder copied successfully to: {}", dst.display());
    Some(dst.to_string_lossy().into_owned())
}

/// move file to dest folder, if dest file already exists, find a new name
/// Returns the new file path if successful, or None on failure.
pub fn move_file(file_path: &str, dest_folder: &str) -> Option<String> {
    let source = Path::new(file_path);
    let file_name = source.file_name()?;
    let mut destination = PathBuf::from(dest_folder);
    destination.push(file_name);

    let destination = get_unique_path(destination);

    // Try to move the file
    match fs::rename(source, &destination) {
        Ok(_) => {
            println!("File moved successfully: {}", destination.display());
            destination.to_str().map(|s| s.to_string())
        }
        Err(e) => {
            eprintln!("Failed to move file: {}", e);
            None
        }
    }
}

/// copy file to dest folder, if dest file already exists, find a new name
/// Returns the new file path if successful, or None on failure.
pub fn copy_file(file_path: &str, dest_folder: &str) -> Option<String> {
    let source = Path::new(file_path);
    let file_name = source.file_name()?;
    let mut destination = PathBuf::from(dest_folder);
    destination.push(file_name);

    let destination = get_unique_path(destination);

    // Try to copy the file
    match fs::copy(source, &destination) {
        Ok(_) => {
            println!("File copied successfully: {}", destination.display());
            destination.to_str().map(|s| s.to_string())
        }
        Err(e) => {
            eprintln!("Failed to copy file: {}", e);
            None
        }
    }
}

/// rename a file
pub fn rename_file(file_path: &str, new_file_name: &str) -> Option<String> {
    let path = Path::new(file_path);

    // Check if the file exists
    if !path.exists() {
        eprintln!("File does not exist: {}", file_path);
        return None;
    }

    // Construct the new file path
    let mut new_file_path = path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf();
    new_file_path.push(new_file_name);

    // Check if the new file name already exists
    if new_file_path.exists() {
        eprintln!(
            "Target file already exists: {}",
            new_file_path.to_string_lossy()
        );
        return None;
    }

    // Attempt to rename the file
    match fs::rename(path, &new_file_path) {
        Ok(_) => {
            let new_path_str = new_file_path.to_string_lossy().into_owned();
            println!("File renamed successfully: {}", new_path_str);
            Some(new_path_str)
        }
        Err(e) => {
            eprintln!("Failed to rename file '{}': {}", file_path, e);
            None
        }
    }
}

/// Get all files in a folder(not include sub-folders)
/// Returns (files, new_count, updated_count)
pub fn get_folder_files(
    file_type: i64,
    sort_type: i64,
    sort_order: i64,
    folder_id: i64,
    folder_path: &str,
    from_db_only: bool,
) -> (Vec<AFile>, u32, u32) {
    fn matches_file_type_filter(filter: i64, file_type: i64) -> bool {
        if filter <= 0 {
            return true;
        }

        let bit = match file_type {
            1 => 1,
            2 => 2,
            3 => 4,
            _ => 0,
        };

        bit > 0 && (filter & bit) == bit
    }

    let mut new_count = 0;
    let mut updated_count = 0;

    let mut files: Vec<AFile> = if from_db_only {
        match AFile::get_files_by_folder_id(folder_id) {
            Ok(files) => files
                .into_iter()
                .filter(|file| matches_file_type_filter(file_type, file.file_type.unwrap_or_default()))
                .collect(),
            Err(e) => {
                eprintln!("Failed to get files from DB: {}", e);
                Vec::new()
            }
        }
    } else {
        let mut file_list = Vec::new();
        for entry in WalkDir::new(folder_path)
            .min_depth(1)
            .max_depth(1)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|entry| entry.file_type().is_file())
        {
            let path = entry.path();
            let file_path_str = match path.to_str() {
                Some(p) => p,
                None => continue,
            };

            if let Some(ftype) = get_file_type(file_path_str) {
                if matches_file_type_filter(file_type, ftype) {
                    match AFile::add_to_db(folder_id, file_path_str, ftype) {
                        Ok((file, status)) => {
                            if status == 1 {
                                new_count += 1;
                            } else if status == 2 {
                                updated_count += 1;
                            }
                            file_list.push(file);
                        }
                        Err(e) => {
                            eprintln!("Failed to add file to DB: {} ({})", file_path_str, e);
                        }
                    }
                }
            }
        }
        file_list
    };

    // sort
    if sort_type == 4 {
        // random
        use rand::seq::SliceRandom;
        let mut rng = rand::thread_rng();
        files.shuffle(&mut rng);
    } else {
        files.sort_by(|a, b| {
            let ordering = match sort_type {
                0 => a.taken_date.cmp(&b.taken_date), // Time
                1 => convert_to_pinyin(&a.name.to_lowercase()) // name
                    .cmp(&convert_to_pinyin(&b.name.to_lowercase())), // support pinyin
                2 => a.size.cmp(&b.size),             // size
                3 => {
                    if a.width == b.width {
                        a.height.cmp(&b.height)
                    } else {
                        a.width.cmp(&b.width)
                    }
                } // dimension
                _ => a.taken_date.cmp(&b.taken_date), // Default to taken date
            };
            if sort_order == 1 {
                ordering.reverse()
            } else {
                ordering
            }
        });
    }

    (files, new_count, updated_count)
}

/// get folder and file count and total file size (include all sub-folders)
pub fn count_folder_files(path: &str) -> (u64, u64, u64, u64, u64) {
    let mut folder_count = 0;
    let mut image_file_count = 0;
    let mut total_image_size = 0;
    let mut video_file_count = 0;
    let mut total_video_size = 0;

    // Use WalkDir to iterate over directory entries
    for entry in WalkDir::new(path).into_iter().filter_map(Result::ok) {
        let entry_type = entry.file_type();

        if entry_type.is_dir() {
            folder_count += 1;
        } else if entry_type.is_file() {
            if let Some(file_ext_type) = get_file_type(entry.path().to_str().unwrap_or("")) {
                let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
                match file_ext_type {
                    1 | 3 => {
                        image_file_count += 1;
                        total_image_size += size;
                    }
                    2 => {
                        video_file_count += 1;
                        total_video_size += size;
                    }
                    _ => {}
                }
            }
        }
    }

    (
        folder_count,
        image_file_count,
        total_image_size,
        video_file_count,
        total_video_size,
    )
}

/// Get the file extension from a file path
pub fn get_file_extension(file_path: &str) -> Option<String> {
    let path = Path::new(file_path);
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|s| s.to_string())
}

/// get file type by extension (1: image, 2: video, 3: raw image)
pub fn get_file_type(file_path: &str) -> Option<i64> {
    let ext = Path::new(file_path)
        .extension()
        .and_then(|ext| ext.to_str())?;
    let contains_ext = |exts: &[&str]| exts.iter().any(|item| item.eq_ignore_ascii_case(ext));

    if contains_ext(t_common::NORMAL_IMGS) {
        return Some(1);
    }

    if contains_ext(t_common::VIDEOS) {
        return Some(2);
    }

    if contains_ext(t_common::RAW_IMGS) {
        return Some(3);
    }

    None
}

fn normalize_format_label(label: &str) -> String {
    match label.to_ascii_uppercase().as_str() {
        "JPEG" | "JPE" | "JFIF" => "JPG".to_string(),
        "TIF" => "TIFF".to_string(),
        "MPG" => "MPEG".to_string(),
        "M4V" => "MP4".to_string(),
        other => other.to_string(),
    }
}

fn detect_label_from_header(header: &[u8], file_type: i64) -> Option<String> {
    // JPEG
    if header.len() >= 3 && header[0] == 0xFF && header[1] == 0xD8 && header[2] == 0xFF {
        return Some("JPG".to_string());
    }
    // PNG
    if header.starts_with(&[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]) {
        return Some("PNG".to_string());
    }
    // GIF
    if header.starts_with(b"GIF87a") || header.starts_with(b"GIF89a") {
        return Some("GIF".to_string());
    }
    // BMP
    if header.starts_with(b"BM") {
        return Some("BMP".to_string());
    }
    // RIFF family: WEBP / AVI
    if header.len() >= 12 && &header[0..4] == b"RIFF" {
        if &header[8..12] == b"WEBP" {
            return Some("WEBP".to_string());
        }
        if &header[8..11] == b"AVI" {
            return Some("AVI".to_string());
        }
    }
    // TIFF / DNG
    if header.len() >= 4 {
        let is_tiff = (&header[0..4] == b"II*\0") || (&header[0..4] == b"MM\0*");
        if is_tiff {
            if file_type == 3 {
                return Some("RAW".to_string());
            }
            return Some("TIFF".to_string());
        }
    }
    // Matroska / WebM (EBML)
    if header.starts_with(&[0x1A, 0x45, 0xDF, 0xA3]) {
        return Some("MKV".to_string());
    }
    // FLV
    if header.starts_with(b"FLV") {
        return Some("FLV".to_string());
    }
    // ASF/WMV
    if header.starts_with(&[
        0x30, 0x26, 0xB2, 0x75, 0x8E, 0x66, 0xCF, 0x11, 0xA6, 0xD9, 0x00, 0xAA, 0x00, 0x62, 0xCE,
        0x6C,
    ]) {
        return Some("ASF".to_string());
    }
    // ISO BMFF family (MP4/MOV/HEIF/AVIF/3GP...)
    if header.len() >= 12 && &header[4..8] == b"ftyp" {
        let brand = String::from_utf8_lossy(&header[8..12]).to_ascii_lowercase();
        if brand.starts_with("3gp") {
            return Some("3GP".to_string());
        }
        if brand == "qt  " || brand == "qt" {
            return Some("MOV".to_string());
        }
        if ["heic", "heix", "hevc", "hevx", "mif1", "msf1", "heif"].contains(&brand.as_str()) {
            return Some("HEIC".to_string());
        }
        if ["avif", "avis"].contains(&brand.as_str()) {
            return Some("AVIF".to_string());
        }
        return Some("MP4".to_string());
    }

    None
}

pub fn detect_file_format_label(file_path: &str, file_type: i64) -> Option<String> {
    if file_type == 3 {
        return Some("RAW".to_string());
    }

    let mut file = fs::File::open(file_path).ok()?;
    let mut header = [0u8; 512];
    let n = file.read(&mut header).ok()?;
    let header = &header[..n];

    if let Some(label) = detect_label_from_header(header, file_type) {
        return Some(normalize_format_label(&label));
    }

    let ext = get_file_extension(file_path)?;
    Some(normalize_format_label(&ext))
}

/// Get the name from a folder or file path
pub fn get_file_name(path: &str) -> String {
    let path = Path::new(path);

    // Extract the file name or last component of the path
    match path.file_name() {
        Some(name) => name.to_string_lossy().into_owned(),
        None => String::new(), // Return an empty string if there is no valid file name
    }
}

/// Get the full path by joining a folder path and a file name
pub fn get_file_path(path: &str, name: &str) -> String {
    let file_path: PathBuf = Path::new(path).join(name);
    file_path.to_string_lossy().to_string() // Convert PathBuf to String
}

pub fn convert_to_pinyin(s: &str) -> String {
    s.chars()
        .flat_map(|c| match c.to_pinyin() {
            Some(p) => p.plain().chars().collect::<Vec<_>>(),
            None => vec![c],
        })
        .collect()
}

/// Convert a SystemTime to a i64 timestamp (in seconds since UNIX_EPOCH)
pub fn systemtime_to_timestamp(time: Option<SystemTime>) -> Option<i64> {
    match time {
        Some(t) => {
            // Calculate the duration since UNIX_EPOCH
            match t.duration_since(UNIX_EPOCH) {
                Ok(duration) => Some(duration.as_secs() as i64),
                Err(_) => {
                    // pre-1970
                    match UNIX_EPOCH.duration_since(t) {
                        Ok(duration) => Some(-(duration.as_secs() as i64)),
                        Err(_) => None,
                    }
                }
            }
        }
        None => None, // Return None if the input is None
    }
}

/// Convert an EXIF or ISO 8601 date string to a i64 timestamp
pub fn meta_date_to_timestamp(date: &str) -> Option<i64> {
    // Try to parse as ISO 8601 (RFC 3339) first, which video metadata often uses
    if let Ok(datetime) = DateTime::parse_from_rfc3339(date) {
        return Some(datetime.timestamp());
    }

    // Fallback to EXIF format: YYYY:MM:DD HH:MM:SS
    // Some EXIF dates might use different separators or formats, so we can try to be a bit more robust
    // Standard EXIF is "YYYY:MM:DD HH:MM:SS"
    let parts: Vec<&str> = date.split(' ').collect();
    if parts.len() < 2 {
        return None;
    }

    let date_part = parts[0];
    let time_part = parts[1];

    let date_fields: Vec<&str> = date_part.split(':').collect();
    let time_fields: Vec<&str> = time_part.split(':').collect();

    if date_fields.len() != 3 || time_fields.len() != 3 {
        return None;
    }

    let year = date_fields[0].parse::<i32>().ok()?;
    let month = date_fields[1].parse::<u32>().ok()?;
    let day = date_fields[2].parse::<u32>().ok()?;
    let hour = time_fields[0].parse::<u32>().ok()?;
    let minute = time_fields[1].parse::<u32>().ok()?;
    let second = time_fields[2].parse::<u32>().ok()?;

    let dt =
        chrono::NaiveDate::from_ymd_opt(year, month, day)?.and_hms_opt(hour, minute, second)?;

    // Treat EXIF time as local time (without timezone information)
    let local_dt = Local.from_local_datetime(&dt).single()?;
    Some(local_dt.timestamp())
}

/// EXIF GPS data is often stored in a format that includes degrees, minutes, and seconds (DMS),
/// which requires conversion to decimal format for easier use
#[allow(dead_code)]
pub fn dms_to_decimal(degrees: f64, minutes: f64, seconds: f64, direction: Option<&str>) -> f64 {
    let decimal = degrees + (minutes / 60.0) + (seconds / 3600.0);
    if let Some(dir) = direction {
        if dir == "S" || dir == "W" {
            return -decimal; // Convert to negative if South or West
        }
    }
    decimal
}

#[derive(serde::Serialize, Clone)]
struct ProgressPayload {
    album_id: i64,
    current: u64,
    total: u64,
}

#[derive(serde::Serialize, Clone)]
struct FinishedPayload {
    album_id: i64,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Default)]
pub struct IndexRecoveryInfo {
    pub album_id: i64,
    pub file_name: String,
    pub file_path: String,
    pub time: String,
}

fn get_index_trace_path() -> Result<PathBuf, String> {
    let app_dir = crate::t_config::get_app_data_dir()?;
    fs::create_dir_all(&app_dir)
        .map_err(|e| format!("Failed to create AppData directory for index recovery: {}", e))?;
    
    // Make trace library-specific so switching libraries doesn't overwrite it
    let config = crate::t_config::load_app_config().map_err(|e| e.to_string())?;
    let prefix = if config.current_library_id.is_empty() {
        "default".to_string()
    } else {
        config.current_library_id
    };
    
    Ok(app_dir.join(format!("index-recovery-{}.json", prefix)))
}

fn write_index_trace(album_id: i64, file_path: &str) {
    let payload = IndexRecoveryInfo {
        album_id,
        file_name: get_file_name(file_path),
        file_path: file_path.to_string(),
        time: Local::now().format("%Y-%m-%d %H:%M:%S%.3f").to_string(),
    };
    if let (Ok(path), Ok(content)) = (get_index_trace_path(), serde_json::to_string(&payload)) {
        let _ = fs::write(path, content);
    }
}

pub fn read_index_trace() -> Option<IndexRecoveryInfo> {
    let path = get_index_trace_path().ok()?;
    fs::read_to_string(path)
        .ok()
        .and_then(|content| serde_json::from_str::<IndexRecoveryInfo>(&content).ok())
}

pub fn clear_index_trace() {
    if let Ok(path) = get_index_trace_path() {
        let _ = fs::remove_file(path);
    }
}

pub async fn index_album_worker(
    app_handle: &tauri::AppHandle,
    cancellation_token: Arc<Mutex<HashMap<i64, bool>>>,
    album_id: i64,
    thumbnail_size: u32,
    skip_file_path: Option<String>,
) -> Result<(), String> {
    // 1. Get album info
    let album = Album::get_album_by_id(album_id).map_err(|e| e.to_string())?;

    // 2. Count total files
    let (_folders, image_count, _image_size, video_count, _video_size) =
        count_folder_files(&album.path);
    let total_files = image_count + video_count;

    // Resume only when totals match and previous indexed is a valid in-progress value.
    // This avoids breaking normal re-scan behavior after a completed run.
    let previous_indexed = album.indexed.unwrap_or(0);
    let previous_total = album.total.unwrap_or(0);
    let resume_from = if previous_total == total_files
        && previous_indexed > 0
        && previous_indexed < total_files
    {
        previous_indexed
    } else {
        0
    };

    // Get all existing files in DB for this album to track deletions
    let mut all_files_map = AFile::get_all_ids_in_album(album_id).unwrap_or_default();

    // 3. Emit start progress
    let mut current_progress = resume_from;
    app_handle
        .emit(
            "index_progress",
            ProgressPayload {
                album_id,
                current: current_progress,
                total: total_files,
            },
        )
        .map_err(|e| e.to_string())?;

    // update progress to db
    let _ = Album::update_progress(album_id, current_progress, total_files);

    // 4. Traverse and index
    let mut is_cancelled = false;
    let mut traversed_count = 0u64;
    for entry in WalkDir::new(&album.path).into_iter().filter_map(Result::ok) {
        // Check cancellation
        if let Some(&true) = cancellation_token.lock().unwrap().get(&album_id) {
            println!("Indexing cancelled for album {}", album_id);
            is_cancelled = true;
            break;
        }

        if entry.file_type().is_file() {
            let path_str = entry.path().to_string_lossy().to_string();
            if let Some(ftype) = get_file_type(&path_str) {
                // Resume mode: skip already-indexed prefix files and mark them as existing
                // so they are not considered deleted in the final cleanup step.
                if traversed_count < resume_from {
                    all_files_map.remove(&path_str);
                    traversed_count += 1;
                    continue;
                }

                // Persist current file pointer for post-crash diagnosis.
                write_index_trace(album_id, &path_str);

                if skip_file_path.as_deref() == Some(path_str.as_str()) {
                    eprintln!(
                        "Skipping suspected problematic file during recovered indexing: {}",
                        path_str
                    );
                    all_files_map.remove(&path_str);
                    current_progress += 1;
                    let _ = app_handle.emit(
                        "index_progress",
                        ProgressPayload {
                            album_id,
                            current: current_progress,
                            total: total_files,
                        },
                    );
                    traversed_count += 1;
                    continue;
                }

                let parent_path = entry
                    .path()
                    .parent()
                    .unwrap_or(Path::new(&album.path))
                    .to_string_lossy()
                    .to_string();

                if let Ok(folder) = crate::t_sqlite::AFolder::add_to_db(album_id, &parent_path) {
                    if let Some(folder_id) = folder.id {
                        if let Ok((file, _)) =
                            crate::t_sqlite::AFile::add_to_db(folder_id, &path_str, ftype)
                        {
                            if let Some(file_id) = file.id {
                                // Generate thumbnail
                                let _ = crate::t_sqlite::AThumb::get_or_create_thumb(
                                    file_id,
                                    &path_str,
                                    ftype,
                                    file.e_orientation.unwrap_or(1) as i32,
                                    thumbnail_size,
                                    false,
                                );

                                // Generate embedding
                                let ai_state: State<crate::t_ai::AiState> = app_handle.state();
                                let _ =
                                    crate::t_sqlite::AFile::generate_embedding(&ai_state, file_id);

                                // Remove from map to mark as exists
                                all_files_map.remove(&path_str);
                            } else {
                                eprintln!(
                                    "Indexed file has no id, skipping follow-up tasks: {}",
                                    path_str
                                );
                            }
                        }
                    } else {
                        eprintln!("Indexed folder has no id, skipping file: {}", parent_path);
                    }
                }

                current_progress += 1;
                let _ = app_handle.emit(
                    "index_progress",
                    ProgressPayload {
                        album_id,
                        current: current_progress,
                        total: total_files,
                    },
                );

                // update progress to db every 50 items
                if current_progress % 50 == 0 {
                    let _ = Album::update_progress(album_id, current_progress, total_files);
                }

                traversed_count += 1;
            }
        }
    }

    clear_index_trace();

    // Delete files that are in DB but not in file system
    if !is_cancelled && !all_files_map.is_empty() {
        println!(
            "Deleting {} files from DB that are no longer in file system for album {}",
            all_files_map.len(),
            album_id
        );
        for (_, file_id) in all_files_map {
            let _ = AFile::delete(file_id);
        }
    }

    // index finished
    let _ = Album::update_progress(album_id, current_progress, total_files);

    // 5. Set album cover if needed (must happen before index_finished event)
    // so frontend refresh gets the latest cover_file_id immediately.
    let _ = Album::auto_set_cover(album_id);

    // 6. Emit finished
    app_handle
        .emit("index_finished", FinishedPayload { album_id })
        .map_err(|e| e.to_string())?;

    Ok(())
}

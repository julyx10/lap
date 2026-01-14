/**
 * General utility functions.
 * project: jc-photo
 * author:  julyxx
 * email:   tiangle@gmail.com
 * GitHub:  /julyx10
 * date:    2024-08-08
 */
use crate::t_sqlite::{AFile, Album};
use chrono::{DateTime, Local, TimeZone};
use once_cell::sync::Lazy;
use pinyin::ToPinyin;
use reverse_geocoder::ReverseGeocoder;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
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
            return None;
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
        } else {
            if let Err(e) = fs::copy(entry.path(), &dest_path) {
                eprintln!("Failed to copy file to '{}': {}", dest_path.display(), e);
                return None;
            }
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
    match fs::rename(&source, &destination) {
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
    match fs::copy(&source, &destination) {
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
    let mut new_count = 0;
    let mut updated_count = 0;

    let mut files: Vec<AFile> = if from_db_only {
        match AFile::get_files_by_folder_id(folder_id) {
            Ok(files) => files,
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
                if file_type == 0 || file_type == ftype {
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
                    1 => {
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

/// get file type by extension (1: image, 2: video, 3: raw)
pub fn get_file_type(file_path: &str) -> Option<i64> {
    let ext = get_file_extension(file_path)?.to_lowercase();

    // Normal images
    let normal_imgs = [
        "jpg", "jpeg", "png", "gif", "bmp", "tif", "tiff", "webp", "avif", "heic", "heif",
    ];

    // RAW formats
    let raw_imgs = [
        "cr2", "cr3", "nef", "nrw", "arw", "srf", "sr2", "raf", "rw2", "orf", "pef", "dng",
    ];

    // Video formats
    let videos = [
        "mpg", "mpeg", "mp4", "mkv", "avi", "mov", "webm", "flv", "wmv", "3gp", "m4v", "hevc",
        "asf", "mts", "m2ts", "mod", "tod", "ts",
    ];

    if normal_imgs.contains(&ext.as_str()) {
        return Some(1);
    }

    if raw_imgs.contains(&ext.as_str()) {
        return Some(3);
    }

    if videos.contains(&ext.as_str()) {
        return Some(2);
    }

    None
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

/// get main.db file path
pub fn get_db_file_path() -> Result<String, String> {
    // Get the local AppData directory
    let app_data_dir = dirs::data_local_dir()
        .ok_or_else(|| "Failed to get the local AppData directory".to_string())?
        .join("jc-photo");

    // Ensure the directory exists
    fs::create_dir_all(&app_data_dir)
        .map_err(|e| format!("Failed to create AppData directory: {}", e))?;

    // Construct the path for main.db
    let db_path = app_data_dir.join("main.db");

    Ok(db_path.to_string_lossy().into_owned())
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

pub async fn index_album_worker(
    app_handle: &tauri::AppHandle,
    cancellation_token: Arc<Mutex<HashMap<i64, bool>>>,
    album_id: i64,
    thumbnail_size: u32,
) -> Result<(), String> {
    // 1. Get album info
    let album = Album::get_album_by_id(album_id).map_err(|e| e.to_string())?;

    // 2. Count total files
    let (_folders, image_count, _image_size, video_count, _video_size) =
        count_folder_files(&album.path);
    let total_files = image_count + video_count;

    // 3. Emit start progress
    let mut current_progress = 0;
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
    let _ = Album::update_progress(album_id, 0, total_files);

    // 4. Traverse and index
    for entry in WalkDir::new(&album.path).into_iter().filter_map(Result::ok) {
        // Check cancellation
        if let Some(&true) = cancellation_token.lock().unwrap().get(&album_id) {
            println!("Indexing cancelled for album {}", album_id);
            break;
        }

        if entry.file_type().is_file() {
            let path_str = entry.path().to_string_lossy().to_string();
            if let Some(ftype) = get_file_type(&path_str) {
                let parent_path = entry
                    .path()
                    .parent()
                    .unwrap_or(Path::new(&album.path))
                    .to_string_lossy()
                    .to_string();

                if let Ok(folder) = crate::t_sqlite::AFolder::add_to_db(album_id, &parent_path) {
                    if let Ok((file, _)) =
                        crate::t_sqlite::AFile::add_to_db(folder.id.unwrap(), &path_str, ftype)
                    {
                        // Generate thumbnail
                        let _ = crate::t_sqlite::AThumb::get_or_create_thumb(
                            file.id.unwrap(),
                            &path_str,
                            ftype,
                            file.e_orientation.unwrap_or(1) as i32,
                            thumbnail_size,
                            false,
                        );

                        // Generate embedding
                        let ai_state: State<crate::t_ai::AiState> = app_handle.state();
                        let _ =
                            crate::t_sqlite::AFile::generate_embedding(&ai_state, file.id.unwrap());
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
            }
        }
    }

    // index finished
    let _ = Album::update_progress(album_id, current_progress, total_files);

    // 5. Emit finished
    app_handle
        .emit("index_finished", FinishedPayload { album_id })
        .map_err(|e| e.to_string())?;

    // 6. Set album cover if needed
    let _ = Album::auto_set_cover(album_id);

    Ok(())
}

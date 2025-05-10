
/**
 * project: jc-photo
 * author:  julyxx
 * email:   tiangle@gmail.com
 * GitHub:  /julyx10
 * date:    2024-08-08
 */
use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use walkdir::WalkDir; // https://docs.rs/walkdir/2.5.0/walkdir/
use chrono::{DateTime, Utc};
use image::ImageReader;
use crate::t_sqlite::AFile;
use std::process::Command;

// #[cfg(target_os = "windows")]
// use std::os::windows::fs::MetadataExt; // Windows-specific extensions

// #[derive(serde::Serialize, serde::Deserialize, Debug)]
// pub struct AppConfig {
//     pub app_name: String,
//     pub theme: String,
//     pub language: String,
// }

// impl Default for AppConfig {
//     fn default() -> Self {
//         Self {
//             app_name: "jc-photo".into(),
//             theme: "dark".into(),
//             language: "en".into(),
//         }
//     }
// }
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

        // Use WalkDir to iterate over directory entries
        for entry in WalkDir::new(path).min_depth(1).max_depth(1).into_iter()
        // .filter_entry(|e| !is_hidden(e))
        {
            let entry = entry.map_err(|e| e.to_string())?;
            if entry.file_type().is_dir() {
                let mut node = FileNode::new(entry.path().to_str().unwrap(), true, false);

                // Recursively process subdirectories
                if is_recursive {
                    node.children = Some(Self::recurse_nodes(entry.path(), is_recursive)?);
                }

                nodes.push(node);
            }
        }

        Ok(nodes)
    }
}

// file metadata struct
#[derive(serde::Serialize)]
pub struct FileInfo {
    pub file_path: String,
    pub file_name: String,
    pub file_type: Option<String>,
    pub created: Option<u64>,
    pub modified: Option<u64>,        // modified date as a timestamp
    pub modified_str: Option<String>, // modified date as a string (YYYY-MM-DD)
    // pub accessed:  Option<u64>,

    // Windows-specific attributes
    // pub file_attributes: u32,
    // volume_serial_number: u32,  // identifies the disk or partition where the file is stored
    // number_of_links: u32,
    // file_index: u64,   // uid of the file
    pub file_size: u64,
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
            created: systemtime_to_u64(metadata.created().ok()),
            modified: systemtime_to_u64(metadata.modified().ok()),
            modified_str: systemtime_to_string(metadata.modified().ok()),
            // accessed: systemtime_to_string(metadata.accessed().ok()),
            // file_attributes: metadata.file_attributes(),
            // volume_serial_number: metadata.volume_serial_number(),
            // number_of_links: metadata.number_of_links(),
            // file_index: metadata.file_index(),
            file_size: metadata.len(),
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
        },
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

/// Deletes a folder and all its contents.
/// Returns `true` if deletion was successful, otherwise `false`.
pub fn delete_folder(folder_path: &str) -> bool {
    let path = Path::new(folder_path);

    if !path.exists() {
        eprintln!("Folder does not exist: {}", folder_path);
        return false;
    }

    // Use remove_dir_all to recursively delete the entire folder
    match fs::remove_dir_all(path) {
        Ok(_) => {
            println!("Folder deleted successfully: {}", folder_path);
            true
        }
        Err(e) => {
            eprintln!("Failed to delete folder '{}': {}", folder_path, e);
            false
        }
    }
}

/// move a folder to a new location
/// Returns the new folder path if successful
pub fn move_folder(folder_path: &str, new_folder_path: &str) -> Option<String> {
    let path = Path::new(folder_path);
    let mut new_path = Path::new(new_folder_path).to_path_buf();

    // Ensure the source folder exists
    if !path.exists() {
        eprintln!("Folder does not exist: {}", folder_path);
        return None;
    }

    // Append the folder name to the new folder path
    if let Some(folder_name) = path.file_name() {
        new_path.push(folder_name);
    } else {
        eprintln!("Invalid folder name: {}", folder_path);
        return None;
    }

    // Attempt to move the folder and return result
    fs::rename(path, &new_path).map_or_else(
        |e| {
            eprintln!("Failed to move folder: {}", e);
            None
        },
        |_| {
            println!("Folder moved to: {}", new_path.display());
            Some(new_path.to_string_lossy().into_owned())
        },
    )
}

/// Recursively copies a folder and all its contents to a new location.
/// Returns Some(new_folder_path) if successful, or None on failure.
pub fn copy_folder(folder_path: &str, new_folder_path: &str) -> Option<String> {
    let src = Path::new(folder_path);
    let mut dst = Path::new(new_folder_path).to_path_buf();

    // Check if the source folder exists and is a directory
    if !src.exists() || !src.is_dir() {
        eprintln!("Source folder does not exist or is not a directory: {}", folder_path);
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

    // Create the destination folder if it does not exist
    if let Err(e) = fs::create_dir_all(&dst) {
        eprintln!("Failed to create destination folder '{}': {}", dst.display(), e);
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
                eprintln!("Failed to create directory '{}': {}", dest_path.display(), e);
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

/// move file to a new location
pub fn move_file(file_path: &str, new_folder_path: &str) -> Option<String> {
    let source = Path::new(file_path);
    let file_name = source.file_name()?;
    let mut destination = PathBuf::from(new_folder_path);
    destination.push(file_name);

    // Do not overwrite if destination already exists
    if destination.exists() {
        eprintln!("Destination file already exists: {}", destination.display());
        return None;
    }

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

/// copy file to a new location
pub fn copy_file(file_path: &str, new_folder_path: &str) -> Option<String> {
    let source = Path::new(file_path);
    let file_name = source.file_name()?;
    let mut destination = PathBuf::from(new_folder_path);
    destination.push(file_name);

    // Do not overwrite if destination already exists
    if destination.exists() {
        eprintln!("Destination file already exists: {}", destination.display());
        return None;
    }

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
    let mut new_file_path = path.parent().unwrap_or_else(|| Path::new(".")).to_path_buf();
    new_file_path.push(new_file_name);
    
    // Check if the new file name already exists
    if new_file_path.exists() {
        eprintln!("Target file already exists: {}", new_file_path.to_string_lossy());
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

/// delete a file
pub fn delete_file(file_path: &str) -> Option<String> {
    let path = Path::new(file_path);

    // Check if the file exists
    if !path.exists() {
        eprintln!("File does not exist: {}", file_path);
        return None;
    }

    // Attempt to delete the file
    match fs::remove_file(path) {
        Ok(_) => {
            println!("File deleted successfully: {}", file_path);
            Some(file_path.to_string())
        }
        Err(e) => {
            eprintln!("Failed to delete file '{}': {}", file_path, e);
            None
        }
    }
}

/// Get all files in a folder(not include sub-folders)
/// Returns a vector of AFile instances
pub fn get_folder_files(
    folder_id: i64,
    folder_path: &str,
    filter_file_name: &str,
    filter_file_type: i64,     // 0: all, 1: image, 2: video, 3: music
) -> Vec<AFile> {
    WalkDir::new(folder_path)
        .min_depth(1)
        .max_depth(1)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .filter_map(|entry| {
            let path = entry.path();

            let file_path = path.to_str()?;
            let file_name = path.file_name()?.to_str()?;
            let file_ext = path.extension()?.to_str()?;

            // filter by file name
            if !filter_file_name.is_empty() && !file_name.contains(filter_file_name) {
                return None;
            }

            if let Some(file_type) = get_file_type(file_ext) {
                if filter_file_type == 0 || filter_file_type == file_type {
                    match AFile::add_to_db(folder_id, file_path, file_type) {
                        Ok(file) => Some(file),
                        Err(e) => {
                            eprintln!("Failed to add file to DB: {} ({})", file_path, e);
                            None
                        }
                    }
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect()
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
        let entry_path = entry.path();
    
        if entry_type.is_dir() {
            folder_count += 1;
        } else if entry_type.is_file() {
            if let Some(extension) = entry_path.extension().and_then(|ext| ext.to_str()) {
                if let Some(file_type) = get_file_type(extension) {
                    let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
                    match file_type {
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
    }

    (folder_count, image_file_count, total_image_size, video_file_count, total_video_size)
}

/// get file type by extension (1: image, 2: video, 3: music)
pub fn get_file_type(extension: &str) -> Option<i64> {
    match extension.to_lowercase().as_str() {
        "jpg" | "jpeg" | "png" | "gif" | "bmp" | "tiff" | "webp" | "heic" | "heif" => Some(1),   // image
        "mpg" | "mpeg" | "mp4" | "mkv" | "avi" | "mov" | "webm" | "flv" | "wmv" | "3gp" => Some(2), // video
        "mp3" | "flac" | "wav" | "m4a" | "ogg" | "wma" | "aac" | "ac3" | "alac"| "aiff" => Some(3), // music
        _ => None,
    }
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

/// Convert a SystemTime to a u64 timestamp (in seconds since UNIX_EPOCH)
pub fn systemtime_to_u64(time: Option<SystemTime>) -> Option<u64> {
    match time {
        Some(t) => {
            // Calculate the duration since UNIX_EPOCH
            match t.duration_since(UNIX_EPOCH) {
                Ok(duration) => Some(duration.as_secs()),
                Err(_) => None, // Handle the case where `SystemTime` is before UNIX_EPOCH
            }
        }
        None => None, // Return None if the input is None
    }
}

/// Convert a SystemTime to a string
pub fn systemtime_to_string(time: Option<SystemTime>) -> Option<String> {
    match time {
        Some(t) => {
            // Calculate the duration since UNIX_EPOCH
            let duration = t.duration_since(UNIX_EPOCH).ok()?;
            // Convert to DateTime<Utc> using the duration
            let datetime = DateTime::<Utc>::from(UNIX_EPOCH + duration);
            // Format the DateTime to a readable string
            Some(datetime.format("%Y-%m-%d").to_string())
        }
        None => None, // Return None if the input is None
    }
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

/// Quick probing of image dimensions without loading the entire file
pub fn get_image_size(file_path: &str) -> Result<(u32, u32), String> {
    // Use imagesize to get width and height
    let dimensions = imagesize::size(file_path).map_err(|e| e.to_string())?; // Map error to String if any

    // Return the dimensions as (width, height)
    Ok((dimensions.width as u32, dimensions.height as u32))
}

/// Get a thumbnail image from a file path
pub fn get_thumbnail(
    file_path: &str,
    orientation: i32,
    thumbnail_size: u32,
) -> Result<Option<Vec<u8>>, String> {
    // Open and decode the image
    let img_reader =
        ImageReader::open(file_path).map_err(|e| format!("Failed to open image: {}", e))?;
    let img_format = img_reader.format().ok_or("Could not detect image format")?;
    let img = img_reader
        .decode()
        .map_err(|e| format!("Failed to decode image: {}", e))?;
    let thumbnail = img.thumbnail(thumbnail_size, thumbnail_size);

    // Adjust the image orientation based on the EXIF orientation value
    let adjusted_thumbnail = match orientation {
        3 => thumbnail.rotate180(),
        6 => thumbnail.rotate90(),
        8 => thumbnail.rotate270(),
        _ => thumbnail,
    };

    // Save the thumbnail to an in-memory buffer as a JPEG
    let mut buf = Vec::new();
    match adjusted_thumbnail.write_to(&mut Cursor::new(&mut buf), img_format) {
        Ok(()) => Ok(Some(buf)),
        Err(_) => Ok(None),
    }
}

/// Print an image using the default system printer
/// This function is platform-specific and may need to be adjusted for different operating systems
pub fn print_image(image_path: String) -> Result<(), String> {
    // Platform-specific printing logic
    let output = if cfg!(target_os = "windows") {
        Command::new("mspaint")
            .arg("/p")
            .arg(image_path)
            .output()
            .map_err(|e| e.to_string())?
    } else if cfg!(target_os = "macos") {
        Command::new("lp")
            .arg(image_path)
            .output()
            .map_err(|e| e.to_string())?
    } else {
        return Err("Unsupported OS".to_string());
    };

    if !output.status.success() {
        return Err(format!("Failed to print image: {}", String::from_utf8_lossy(&output.stderr)));
    }

    Ok(())
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

/**
 * project: jc-photo
 * author:  julyxx
 * email:   tiangle@gmail.com
 * GitHub:  /julyx10
 * date:    2024-08-08
 */

use std::fs;
use std::io::Cursor;
use std::os::windows::fs::MetadataExt; // Windows-specific extensions
use std::path::{ Path, PathBuf };
use std::time::{ SystemTime, UNIX_EPOCH };
use chrono::{ DateTime, Utc };
use walkdir::WalkDir; // https://docs.rs/walkdir/2.5.0/walkdir/
use image::GenericImageView;


#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct AppConfig {
    pub app_name: String,
    pub theme: String,
    pub language: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            app_name: "jc-photo".into(),
            theme: "dark".into(),
            language: "en".into(),
        }
    }
}


/// FileNode struct to represent a file system node
#[derive(serde::Serialize)]
pub struct FileNode {
    id:   Option<i64>,      // unique id(in database)
    name: String,           // folder name
    path: String,           // folder path
    is_dir: bool,           // is directory
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
        for entry in WalkDir::new(path)
            .min_depth(1)
            .max_depth(1)
            .into_iter()
            // .filter_entry(|e| !is_hidden(e))
        {
            let entry = entry.map_err(|e| e.to_string())?;
            if entry.file_type().is_dir() {
                let mut node = FileNode::new(
                    entry.path().to_str().unwrap(),
                    true,
                    false,
                );
                
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
    pub created:   Option<u64>,
    pub modified:  Option<u64>,         // modified date as a timestamp
    pub modified_str: Option<String>,  // modified date as a string (YYYY-MM-DD)
    // pub accessed:  Option<u64>,

    // Windows-specific attributes
    pub file_attributes: u32,
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
            created:  systemtime_to_u64(metadata.created().ok()),
            modified: systemtime_to_u64(metadata.modified().ok()),
            modified_str: systemtime_to_string(metadata.modified().ok()),
            // accessed: systemtime_to_string(metadata.accessed().ok()),

            file_attributes: metadata.file_attributes(),
            // volume_serial_number: metadata.volume_serial_number(),
            // number_of_links: metadata.number_of_links(),
            // file_index: metadata.file_index(),
            file_size: metadata.len(),
        })
    }
}


/// ImageInfo struct to represent image metadata
#[derive(serde::Serialize)]
pub struct ImageInfo {
    pub width:      u32,
    pub height:     u32,
    pub color_type: String,
    pub bit_depth:  u16,
    pub has_alpha:  bool,
}

impl ImageInfo {
    /// Get image info from a file path
    pub fn new(file_path: &str) -> Result<Self, String> {
        let img = image::open(file_path).map_err(|e| e.to_string())?;
        let (width, height) = img.dimensions();
        let color_type = img.color();
        let bit_depth = color_type.bits_per_pixel();
        let has_alpha = color_type.has_alpha();

        Ok(ImageInfo {
            width,
            height,
            color_type: format!("{:?}", color_type),
            bit_depth,
            has_alpha,
        })
    }
}


/// Check if a file extension is an image extension
pub fn is_image_extension(extension: &str) -> bool {
    match extension.to_lowercase().as_str() {
        "jpg" | "jpeg" | "png" | "gif" | "bmp" | "tiff" | "webp" | "heic" => true,
        _ => false,
    }
}

/// Check if a file extension is a video extension
pub fn is_video_extension(extension: &str) -> bool {
    match extension.to_lowercase().as_str() {
        "mp4" | "mkv" | "avi" | "mov" | "webm" | "flv" | "wmv" | "3gp" => true,
        _ => false,
    }
}

/// Check if a file extension is a music extension
pub fn is_music_extension(extension: &str) -> bool {
    match extension.to_lowercase().as_str() {
        "mp3" | "wav" | "flac" | "m4a" | "ogg" | "wma" | "aac" | "ac3" => true,
        _ => false,
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
    file_path.to_string_lossy().to_string()  // Convert PathBuf to String
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


/// Quick probing of image dimensions without loading the entire file
pub fn get_image_size(file_path: &str) -> Result<(u32, u32), String> {
    // Use imagesize to get width and height
    let dimensions = imagesize::size(file_path)
        .map_err(|e| e.to_string())?; // Map error to String if any

    // Return the dimensions as (width, height)
    Ok((dimensions.width as u32, dimensions.height as u32))
}


/// Get a thumbnail image from a file path
pub fn get_thumbnail(file_path: &str, orientation: i32, thumbnail_size: u32) -> Result<Option<Vec<u8>>, String> {
    let img = image::open(file_path).expect("Failed to open image");
    
    // Adjust the image orientation based on the EXIF orientation value
    let adjusted_img = match orientation {
        3 => img.rotate180(),
        6 => img.rotate90(),
        8 => img.rotate270(),
        _ => img,
    };

    // Resize the image to a thumbnail
    let thumbnail = adjusted_img.thumbnail(thumbnail_size, thumbnail_size);

    // Save the thumbnail to an in-memory buffer as a JPEG
    let mut buf = Vec::new();
    match thumbnail.write_to(&mut Cursor::new(&mut buf), image::ImageFormat::Jpeg) {
        Ok(()) => Ok(Some(buf)),
        Err(_) => Ok(None)
    }
}


/// EXIF GPS data is often stored in a format that includes degrees, minutes, and seconds (DMS),
/// which requires conversion to decimal format for easier use
pub fn dms_to_decimal(degrees: f64, minutes: f64, seconds: f64, direction: Option<&str>) -> f64 {
    let decimal = degrees + (minutes / 60.0) + (seconds / 3600.0);
    if let Some(dir) = direction {
        if dir == "S" || dir == "W" {
            return -decimal; // Convert to negative if South or West
        }
    }
    decimal
}
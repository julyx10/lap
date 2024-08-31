use std::fs;
use std::io;
use std::time::{SystemTime, UNIX_EPOCH};
use std::os::windows::fs::MetadataExt; // Windows-specific extensions
use std::path::Path;
use walkdir::{WalkDir, DirEntry}; // https://docs.rs/walkdir/2.5.0/walkdir/

// file metadata struct
#[derive(serde::Serialize)]
pub struct FileInfo {
    pub file_size: u64,
    pub created: Option<SystemTime>,
    pub modified: Option<SystemTime>,
    pub accessed: Option<SystemTime>,
    pub file_attributes: u32,
    // volume_serial_number: u32,  // identifies the disk or partition where the file is stored
    // number_of_links: u32,
    // file_index: u64,   // uid of the file
}


/// get file info from a folder/file path (on Windows)
pub fn get_file_info(path: String) -> io::Result<FileInfo> {
    // Convert the string path into a Path object
    let path = Path::new(&path);

    let metadata = fs::metadata(path)?;

    let file_size = metadata.len();
    let created = metadata.created().ok();
    let modified = metadata.modified().ok();
    let accessed = metadata.accessed().ok();

    // Windows-specific attributes
    let file_attributes = metadata.file_attributes();
    // let volume_serial_number = metadata.volume_serial_number();
    // let number_of_links = metadata.number_of_links();
    // let file_index = metadata.file_index();

    Ok(FileInfo {
        file_size,
        created,
        modified,
        accessed,
        file_attributes,
        // volume_serial_number,
        // number_of_links,
        // file_index,
    })
}


/// Check if the entry is hidden (starts with a dot)
// fn is_hidden(entry: &DirEntry) -> bool {
    
//     entry.file_name().to_string_lossy().starts_with('.')
// }


/// Check if a file extension is an image extension
fn is_image_extension(extension: &str) -> bool {
    match extension.to_lowercase().as_str() {
        "jpg" | "jpeg" | "png" | "gif" | "bmp" | "tiff" | "webp" => true,
        _ => false,
    }
}


/// Get the name from a folder or file path
pub fn get_path_name(path: String) -> String {
    // Convert the String into a Path object
    let path = Path::new(&path);
    
    // Extract the file name or last component of the path
    match path.file_name() {
        Some(name) => name.to_string_lossy().into_owned(),
        None => String::new(), // Return an empty string if there is no valid file name
    }
}


/// Convert a SystemTime to a u64 timestamp (in seconds since UNIX_EPOCH)
pub fn systemtime_to_u64(time: Option<SystemTime>) -> Option<u64> {
    time.and_then(|t| {
        t.duration_since(UNIX_EPOCH)
            .ok() // Convert Result to Option
            .map(|d| d.as_secs()) // Convert Duration to u64 (in seconds)
    })
}


/// Format a SystemTime into a human-readable string
pub fn format_time(time: Option<SystemTime>) -> String {
    match time {
        Some(time) => {
            let datetime: chrono::DateTime<chrono::Local> = time.into();
            datetime.format("%Y-%m-%d %H:%M").to_string()
        },
        None => "N/A".to_string(),
    }
}


/// Format a file size into a human-readable string
pub fn format_file_size(size: u64) -> String {
    const KIB: u64 = 1024; // Kibibyte
    const MIB: u64 = KIB * 1024; // Mebibyte
    const GIB: u64 = MIB * 1024; // Gibibyte

    if size < KIB {
        format!("{} bytes", size)
    } else if size < MIB {
        format!("{:.2} KB", size as f64 / KIB as f64)
    } else if size < GIB {
        format!("{:.2} MB", size as f64 / MIB as f64)
    } else {
        format!("{:.2} GB", size as f64 / GIB as f64)
    }
}


/// List image files in a path
pub fn list_image_files(path: String) -> Result<Vec<FileInfo>, String> {
    let mut file_info: Vec<FileInfo> = Vec::new();

    for entry in WalkDir::new(path)
        .min_depth(1)
        .max_depth(1)
        .into_iter().filter_map(|e| e.ok()) {
        
        let path = entry.path();
        if path.is_file() {
            if let Some(extension) = path.extension().and_then(|ext| ext.to_str()) {
                if is_image_extension(extension) {
                    file_info.push(get_file_info(path.to_str().unwrap().to_string()).unwrap());
                }
            }
        }
    }

    Ok(file_info)
}


/// FileNode struct to represent a file system node
#[derive(serde::Serialize)]
pub struct FileNode {
    id:   Option<i64>,      // unique id(in database)
    name: String,           // folder name
    path: String,           // folder path
    is_dir: bool,           // is directory
    is_expanded: bool,
    children: Option<Vec<FileNode>>,
}

impl FileNode {
    
    /// Create a new FileNode
    fn new(name: String, path: String, is_dir: bool, is_expanded: bool) -> Self {
        FileNode {
            id: None,
            name,
            path,
            is_dir,
            is_expanded,
            children: None,
        }
    }

    /// Read folders from a path and build a FileNode
    pub fn read_folders(path: String) -> Result<FileNode, String> {
        let root_path = Path::new(&path);

        // Check if the path exists and is a directory
        if !root_path.exists() {
            return Err(format!("Path does not exist: {}", path));
        }

        if !root_path.is_dir() {
            return Err(format!("Path is not a directory: {}", path));
        }

        // Create the root FileNode
        let root_name = root_path.file_name()
            .unwrap_or_else(|| root_path.as_os_str())
            .to_string_lossy()
            .into_owned();

        let mut root_node = FileNode::new(root_name, path.clone(), true, false);

        // Recursively read subfolders and files
        root_node.children = Some(Self::recurse_folders(root_path)?);

        Ok(root_node)
    }

    /// Recurse sub-folders 
    fn recurse_folders(path: &Path) -> Result<Vec<FileNode>, String> {
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
                let node = Self::build_file_node(&entry)?;

                // Recursively process subdirectories
                // node.children = Some(Self::recurse_folders(entry.path())?);
                nodes.push(node);
            }
        }

        Ok(nodes)
    }

    /// Build a FileNode from a DirEntry
    fn build_file_node(entry: &DirEntry) -> Result<FileNode, String> {
        let path = entry.path().to_str().ok_or("Invalid path")?.to_string();
        let name = entry.file_name().to_string_lossy().into_owned();
        let is_dir = entry.file_type().is_dir();
        // let id = next_id();

        Ok(FileNode::new(name, path, is_dir, false))
    }

}


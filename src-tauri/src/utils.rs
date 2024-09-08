use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};
use std::os::windows::fs::MetadataExt; // Windows-specific extensions
use std::path::Path;
use walkdir::{WalkDir, DirEntry}; // https://docs.rs/walkdir/2.5.0/walkdir/


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
    fn new(name: &str, path: &str, is_dir: bool, is_expanded: bool) -> Self {
        FileNode {
            id: None,
            name: name.to_string(),
            path: path.to_string(),
            is_dir,
            is_expanded,
            children: None,
        }
    }

    /// Read folders from a path and build a FileNode
    pub fn build_nodes(path: &str) -> Result<FileNode, String> {
        let root_path = Path::new(&path);

        // Check if the path exists and is a directory
        if !root_path.exists() {
            return Err(format!("Path does not exist: {}", path));
        }

        if !root_path.is_dir() {
            return Err(format!("Path is not a directory: {}", path));
        }

        // // Create the root FileNode
        let mut root_node = FileNode::new(
            get_path_name(path).as_str(), 
            path, 
            true,
            false
        );

        // Recursively read subfolders and files
        root_node.children = Some(Self::recurse_nodes(root_path)?);

        Ok(root_node)
    }

    /// Recurse sub-folders 
    fn recurse_nodes(path: &Path) -> Result<Vec<FileNode>, String> {
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
                // let node = Self::build_file_node(&entry)?;
                let node = FileNode::new(
                    entry.file_name().to_string_lossy().as_ref(),
                    entry.path().to_str().unwrap(),
                    entry.file_type().is_dir(),
                    false,
                );

                // Recursively process subdirectories
                // node.children = Some(Self::recurse_nodes(entry.path())?);
                nodes.push(node);
            }
        }

        Ok(nodes)
    }

}


// file metadata struct
#[derive(serde::Serialize)]
pub struct FileInfo {
    pub file_name: String,
    pub file_type: Option<String>,
    pub created:   Option<u64>,
    pub modified:  Option<u64>,
    // pub accessed:  Option<u64>,
    pub file_attributes: u32,
    // volume_serial_number: u32,  // identifies the disk or partition where the file is stored
    // number_of_links: u32,
    // file_index: u64,   // uid of the file
    pub file_size: u64,
}

impl FileInfo {

    /// Get file info from a folder/file path (on Windows)
    pub fn new(path: &str) -> Self {
        // Convert the string path into a Path object
        let path = Path::new(path);
        let metadata = fs::metadata(path).unwrap();

        let file_name = path.file_name().unwrap().to_string_lossy().into_owned();
        let file_type = metadata.file_type().is_dir().then(|| "dir".to_string());
        let created = metadata.created().ok();
        let modified = metadata.modified().ok();
        // let accessed = metadata.accessed().ok();
        
        // Windows-specific attributes
        let file_attributes = metadata.file_attributes();
        // let volume_serial_number = metadata.volume_serial_number();
        // let number_of_links = metadata.number_of_links();
        // let file_index = metadata.file_index();
        let file_size = metadata.len();

        FileInfo {
            file_name,
            file_type,
            created:  systemtime_to_u64(created),
            modified: systemtime_to_u64(modified),
            // accessed: systemtime_to_u64(accessed),
            file_attributes,
            // volume_serial_number,
            // number_of_links,
            // file_index,
            file_size,
        }
    }
    
}


/// Check if a file extension is an image extension
fn is_image_extension(extension: &str) -> bool {
    match extension.to_lowercase().as_str() {
        "jpg" | "jpeg" | "png" | "gif" | "bmp" | "tiff" | "webp" => true,
        _ => false,
    }
}


/// List image files in a path
pub fn list_image_files(path: &str) -> Result<Vec<FileInfo>, String> {
    let mut files: Vec<FileInfo> = Vec::new();

    for entry in WalkDir::new(path)
        .min_depth(1)
        .max_depth(1)
        .into_iter().filter_map(|e| e.ok()) {
        
        let path = entry.path();
        if path.is_file() {
            if let Some(extension) = path.extension().and_then(|ext| ext.to_str()) {
                if is_image_extension(extension) {
                    files.push(FileInfo::new(path.to_str().unwrap()));
                }
            }
        }
    }

    Ok(files)
}


/// Get the name from a folder or file path
pub fn get_path_name(path: &str) -> String {
    // Convert the String into a Path object
    let path = Path::new(path);
    
    // Extract the file name or last component of the path
    match path.file_name() {
        Some(name) => name.to_string_lossy().into_owned(),
        None => String::new(), // Return an empty string if there is no valid file name
    }
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

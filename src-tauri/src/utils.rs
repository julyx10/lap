use std::fs;
use std::io;
use std::time::SystemTime;
use std::os::windows::fs::MetadataExt; // Windows-specific extensions
use std::path::Path;
use std::sync::atomic::{AtomicI64, Ordering};
use walkdir::{WalkDir, DirEntry}; // https://docs.rs/walkdir/2.5.0/walkdir/

// file metadata struct
#[derive(serde::Serialize)]
pub struct FileInfo {
    file_name: String,
    file_size: String,
    created: String,
    modified: String,
    accessed: String,

    file_attributes: u32,
    // volume_serial_number: u32,  // identifies the disk or partition where the file is stored
    // number_of_links: u32,
    // file_index: u64,   // uid of the file
}

// retrieve file info on windows os
fn get_file_info(file_path: String) -> io::Result<FileInfo> {
    let metadata = fs::metadata(&file_path)?;

    let file_name = get_file_name(&file_path).unwrap_or_default().to_string();
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
        file_name,
        file_size: format_file_size(file_size),
        created: get_format_time(created),
        modified: get_format_time(modified),
        accessed: get_format_time(accessed),
        file_attributes,
        // volume_serial_number,
        // number_of_links,
        // file_index,
    })
}


static ID_COUNTER: AtomicI64 = AtomicI64::new(1);

fn next_id() -> i64 {
    ID_COUNTER.fetch_add(1, Ordering::Relaxed)
}

/// Check if the entry is hidden (starts with a dot)
fn is_hidden(entry: &DirEntry) -> bool {
    
    entry.file_name().to_string_lossy().starts_with('.')
}

/// Check if a file extension is an image extension
fn is_image_extension(extension: &str) -> bool {
    match extension.to_lowercase().as_str() {
        "jpg" | "jpeg" | "png" | "gif" | "bmp" | "tiff" | "webp" => true,
        _ => false,
    }
}

/// Get the file name from a file path
fn get_file_name(file_path: &str) -> Option<&str> {
    let path = Path::new(file_path);
    path.file_name()?.to_str()
}

fn get_format_time(time: Option<SystemTime>) -> String {
    match time {
        Some(time) => {
            let datetime: chrono::DateTime<chrono::Local> = time.into();
            datetime.format("%Y-%m-%d %H:%M").to_string()
        },
        None => "N/A".to_string(),
    }
}


fn format_file_size(size: u64) -> String {
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


#[derive(serde::Serialize)]
pub struct FileNode {
    id:   i64,      // unique id
    name: String,   // folder name
    path: String,   // folder path
    is_dir: bool,   // is directory
    is_expanded: bool,
    children: Option<Vec<FileNode>>,
}

impl FileNode {
    
    fn new(id: i64, name: String, path: String, is_dir: bool, is_expanded: bool) -> Self {
        FileNode {
            id,
            name,
            path,
            is_dir,
            is_expanded,
            children: None,
        }
    }

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

        let mut root_node = FileNode::new(next_id(), root_name, path.clone(), true, false);

        // Recursively read subfolders and files
        root_node.children = Some(Self::recurse_folders(root_path)?);

        Ok(root_node)
    }

    /// Function to build a FileNode from a DirEntry
    fn recurse_folders(path: &Path) -> Result<Vec<FileNode>, String> {
        let mut nodes: Vec<FileNode> = Vec::new();

        // Use WalkDir to iterate over directory entries
        for entry in WalkDir::new(path)
            .min_depth(1)
            .max_depth(1)
            .into_iter()
            .filter_entry(|e| !is_hidden(e))
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

    fn build_file_node(entry: &DirEntry) -> Result<FileNode, String> {
        let path = entry.path().to_str().ok_or("Invalid path")?.to_string();
        let name = entry.file_name().to_string_lossy().into_owned();
        let is_dir = entry.file_type().is_dir();
        let id = next_id();

        Ok(FileNode::new(id, name, path, is_dir, false))
    }

}


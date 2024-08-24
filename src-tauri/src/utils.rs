use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicI64, Ordering};


#[derive(serde::Serialize)]
pub struct FileNode {
    id:   i64,      // unique id
    name: String,   // folder name
    path: String,   // folder path
    is_expanded: bool,
    children: Option<Vec<FileNode>>,
}

// Create a static atomic counter for generating unique IDs
static ID_COUNTER: AtomicI64 = AtomicI64::new(1);

impl FileNode {
    // Function to generate a new unique ID
    fn new_id() -> i64 {
        ID_COUNTER.fetch_add(1, Ordering::SeqCst)
    }
}

/// read a folder 
pub fn read_folder(path: String) -> Result<FileNode, String> {
    fn build_tree(path: &Path) -> Result<FileNode, String> {
        // generate a unique ID
        let id = FileNode::new_id();
        
        // get the file or directory metadata
        let metadata = fs::metadata(path).map_err(|e| e.to_string())?;
        let is_dir = metadata.is_dir();

        // extract the file name and path
        let name = path.file_name()
            .ok_or_else(|| "Failed to get file name".to_string())?
            .to_string_lossy()
            .to_string();

        // convert the path to a string
        let path_str = path.to_string_lossy().to_string();

        // initialize the expanded flag
        let is_expanded = false;

        let children = if is_dir {
            let mut nodes = vec![];
            for entry in fs::read_dir(path).map_err(|e| e.to_string())? {
                let entry = entry.map_err(|e| e.to_string())?;
                let child_path = entry.path();
                if child_path.is_dir() {
                    nodes.push(build_tree(&child_path)?);
                }
            }
            Some(nodes)
        } else {
            None
        };

        Ok(FileNode {
            id,
            name,
            path: path_str,
            is_expanded,
            children,
        })
    }

    let path = Path::new(&path);
    build_tree(path)
}

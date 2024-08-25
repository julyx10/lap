// use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicI64, Ordering};
use walkdir::{WalkDir, DirEntry}; // https://docs.rs/walkdir/2.5.0/walkdir/


static ID_COUNTER: AtomicI64 = AtomicI64::new(1);

fn next_id() -> i64 {
    ID_COUNTER.fetch_add(1, Ordering::Relaxed)
}

fn is_hidden(entry: &DirEntry) -> bool {
    // Check if the entry is hidden (starts with a dot)
    entry.file_name().to_string_lossy().starts_with('.')
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

    // Function to generate a new unique ID
    // fn new_id() -> i64 {
    //     ID_COUNTER.fetch_add(1, Ordering::SeqCst)
    // }

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
                let mut node = Self::build_file_node(&entry)?;

                // Recursively process subdirectories
                node.children = Some(Self::recurse_folders(entry.path())?);
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

/// read all folders and the path
// pub fn read_folders(path: String) -> Result<FileNode, String> {
//     fn build_tree(path: &Path) -> Result<FileNode, String> {
//         // generate a unique ID
//         let id = FileNode::new_id();
        
//         // get the file or directory metadata
//         let metadata = fs::metadata(path).map_err(|e| e.to_string())?;
//         let is_dir = metadata.is_dir();

//         // extract the file name and path
//         let name = path.file_name()
//             .ok_or_else(|| "Failed to get file name".to_string())?
//             .to_string_lossy()
//             .to_string();

//         // convert the path to a string
//         let path_str = path.to_string_lossy().to_string();

//         // initialize the expanded flag
//         let is_expanded = false;

//         let children = if is_dir {
//             let mut nodes = vec![];
//             for entry in fs::read_dir(path).map_err(|e| e.to_string())? {
//                 let entry = entry.map_err(|e| e.to_string())?;
//                 let child_path = entry.path();
//                 if child_path.is_dir() {
//                     nodes.push(build_tree(&child_path)?);
//                 }
//             }
//             Some(nodes)
//         } else {
//             None
//         };

//         Ok(FileNode {
//             id,
//             name,
//             path: path_str,
//             is_dir,
//             is_expanded,
//             children,
//         })
//     }

//     let path = Path::new(&path);
//     build_tree(path)
// }


/// read all image files in a directory
// pub fn read_image_files(path: String) -> Result<FileNode, String> {
//     Ok(vec![])
// }

fn is_image_extension(extension: &std::ffi::OsStr) -> bool {
    match extension.to_str() {
        Some("jpg") | Some("jpeg") | Some("png")| Some("gif") => true,
        _ => false,
    }
}
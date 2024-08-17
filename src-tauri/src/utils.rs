use std::fs;
use std::path::Path;


#[derive(serde::Serialize)]
pub struct FileNode {
    name: String,   // folder name
    path: String,   // folder path
    is_dir: bool,
    is_callapsed: bool,
    children: Option<Vec<FileNode>>,
}


/// read a folder 
pub fn read_folder(path: String) -> Result<FileNode, String> {
    fn build_tree(path: &Path) -> Result<FileNode, String> {
        let metadata = fs::metadata(path).map_err(|e| e.to_string())?;
        let is_dir = metadata.is_dir();
        let name = path.file_name()
            .ok_or_else(|| "Failed to get file name".to_string())?
            .to_string_lossy()
            .to_string();
        let path_str = path.to_string_lossy().to_string();
        let is_callapsed = false;

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
            name,
            path: path_str,
            is_dir,
            is_callapsed,
            children,
        })
    }

    let path = Path::new(&path);
    build_tree(path)
}

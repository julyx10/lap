/**
 * General utility functions.
 * project: Lap
 * author:  julyx10
 * date:    2024-08-08
 */
use crate::t_common;
use crate::t_apple_sidecar;
use crate::t_sqlite::{AFile, AFolder, AThumb, Album, FolderScanState, FolderSubfolderState};
use crate::t_raw_display::RawDisplayOptions;
use chrono::{DateTime, Local, TimeZone, Utc};
use once_cell::sync::Lazy;
use pinyin::ToPinyin;
use rstar::{AABB, PointDistance, RTree, RTreeObject};
use serde::Serialize;
use std::collections::{HashMap, HashSet, VecDeque};
use std::fs;
use std::io::Read;
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;
use std::panic::{self, AssertUnwindSafe};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tauri::{Emitter, Manager, State};
use tokio::sync::Semaphore;
use tokio::task::JoinSet;
use walkdir::WalkDir; // https://docs.rs/walkdir/2.5.0/walkdir/

#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

pub fn path_exists(path: &str) -> bool {
    fs::symlink_metadata(path).is_ok()
}

pub fn directory_accessible(path: &str) -> bool {
    if !fs::metadata(path)
        .map(|metadata| metadata.is_dir())
        .unwrap_or(false)
    {
        return false;
    }

    match fs::read_dir(path) {
        Ok(mut entries) => !matches!(entries.next(), Some(Err(_))),
        Err(_) => false,
    }
}

pub fn trash_path(path: &str) -> Result<(), String> {
    if !path_exists(path) {
        return Err(format!("Path does not exist: {}", path));
    }

    move_to_trash(path)?;

    if path_exists(path) {
        return Err(format!(
            "Failed to move path to Trash. The path still exists on disk: {}",
            path
        ));
    }

    Ok(())
}

#[cfg(target_os = "macos")]
fn move_to_trash(path: &str) -> Result<(), String> {
    use trash::macos::{DeleteMethod, TrashContextExtMacos};

    // NsFileManager is fast but requires Automation permission that users may deny.
    // When denied, Put Back metadata is lost. Fall back to Finder (AppleScript) which
    // always preserves Put Back metadata and does not require special permissions.
    let mut context = trash::TrashContext::default();
    context.set_delete_method(DeleteMethod::NsFileManager);
    if let Err(_error) = context.delete(path) {
        let mut fallback = trash::TrashContext::default();
        fallback.set_delete_method(DeleteMethod::Finder);
        fallback.delete(path).map_err(|fallback_error| fallback_error.to_string())
    } else {
        Ok(())
    }
}

#[cfg(all(not(target_os = "macos")))]
fn move_to_trash(path: &str) -> Result<(), String> {
    trash::delete(path).map_err(|error| error.to_string())
}

pub fn delete_file_permanently(path: &str) -> Result<(), String> {
    let metadata = fs::symlink_metadata(path).map_err(|e| {
        format!(
            "Failed to read metadata before permanent delete: {} ({})",
            path, e
        )
    })?;

    if metadata.is_dir() {
        return Err(format!(
            "Permanent delete only supports files, not directories: {}",
            path
        ));
    }

    fs::remove_file(path)
        .map_err(|e| format!("Failed to permanently delete file: {} ({})", path, e))?;

    if path_exists(path) {
        return Err(format!(
            "Failed to permanently delete file. The path still exists on disk: {}",
            path
        ));
    }

    Ok(())
}

pub fn delete_folder_permanently(path: &str) -> Result<(), String> {
    let metadata = fs::symlink_metadata(path).map_err(|e| {
        format!(
            "Failed to read metadata before permanent delete: {} ({})",
            path, e
        )
    })?;

    if !metadata.is_dir() {
        return Err(format!(
            "Permanent folder delete only supports directories, not files: {}",
            path
        ));
    }

    fs::remove_dir_all(path)
        .map_err(|e| format!("Failed to permanently delete folder: {} ({})", path, e))?;

    if path_exists(path) {
        return Err(format!(
            "Failed to permanently delete folder. The path still exists on disk: {}",
            path
        ));
    }

    Ok(())
}

// reverse geocoder
#[derive(serde::Deserialize)]
pub struct GeoRecord {
    pub lat: f64,
    pub lon: f64,
    pub name: String,
    pub admin1: String,
    pub admin2: String,
    pub cc: String,
}

struct CityPoint {
    xyz: [f64; 3],
    idx: u32,
}

impl RTreeObject for CityPoint {
    type Envelope = AABB<[f64; 3]>;
    fn envelope(&self) -> Self::Envelope {
        AABB::from_point(self.xyz)
    }
}

impl PointDistance for CityPoint {
    fn distance_2(&self, point: &[f64; 3]) -> f64 {
        let dx = self.xyz[0] - point[0];
        let dy = self.xyz[1] - point[1];
        let dz = self.xyz[2] - point[2];
        dx * dx + dy * dy + dz * dz
    }
}

fn lat_lon_to_xyz(lat: f64, lon: f64) -> [f64; 3] {
    let lat = lat.to_radians();
    let lon = lon.to_radians();
    [lat.cos() * lon.cos(), lat.cos() * lon.sin(), lat.sin()]
}

pub struct ReverseGeocoder {
    records: Vec<GeoRecord>,
    tree: RTree<CityPoint>,
}

pub struct SearchResult<'a> {
    pub record: &'a GeoRecord,
}

impl ReverseGeocoder {
    pub fn new() -> ReverseGeocoder {
        let cities = include_str!("../data/cities.csv");
        let mut reader = csv::ReaderBuilder::new()
            .has_headers(true)
            .from_reader(cities.as_bytes());
        let mut records: Vec<GeoRecord> = Vec::with_capacity(150_000);
        let mut points: Vec<CityPoint> = Vec::with_capacity(150_000);
        for row in reader.deserialize() {
            let r: GeoRecord = row.expect("malformed cities.csv row");
            points.push(CityPoint {
                xyz: lat_lon_to_xyz(r.lat, r.lon),
                idx: records.len() as u32,
            });
            records.push(r);
        }
        ReverseGeocoder {
            records,
            tree: RTree::bulk_load(points),
        }
    }

    pub fn search(&self, loc: (f64, f64)) -> Option<SearchResult<'_>> {
        if self.tree.size() == 0 {
            return None;
        }
        if loc.0.is_nan() || loc.1.is_nan() {
            return None;
        }
        let query = lat_lon_to_xyz(loc.0, loc.1);
        // rstar 0.12 has an internal unwrap that can panic on certain tree states;
        // catch_unwind ensures the app survives even if the R-tree is corrupted.
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            self.tree.nearest_neighbor(&query)
        }));
        match result {
            Ok(Some(nearest)) => Some(SearchResult {
                record: &self.records[nearest.idx as usize],
            }),
            Ok(None) => {
                eprintln!(
                    "Geocoder: no nearest neighbor found for ({}, {})",
                    loc.0, loc.1
                );
                None
            }
            Err(_) => {
                eprintln!(
                    "Geocoder: rstar nearest_neighbor panicked for ({}, {})",
                    loc.0, loc.1
                );
                None
            }
        }
    }
}

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
    commit_hash: Option<String>,
    description: String,
    authors: Vec<String>,
    repository: Option<String>,
    license: Option<String>,
    homepage: Option<String>,
}

impl PackageInfo {
    pub fn new(commit_hash: &str) -> Self {
        Self {
            name: env!("CARGO_PKG_NAME").to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            commit_hash: (!commit_hash.is_empty()).then(|| commit_hash.to_string()),
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
    created_at: Option<i64>,
    modified_at: Option<i64>,
    is_dir: bool, // is directory
    is_expanded: bool,
    has_subfolders: bool,
    children: Option<Vec<Self>>,
}

impl FileNode {
    /// Create a new FileNode
    fn new(
        path: &str,
        is_dir: bool,
        is_expanded: bool,
        created_at: Option<i64>,
        modified_at: Option<i64>,
    ) -> Self {
        FileNode {
            id: None,
            name: get_file_name(path),
            path: path.to_string(),
            created_at,
            modified_at,
            is_dir,
            is_expanded,
            has_subfolders: false,
            children: None,
        }
    }

    /// Read folders from a path and build a FileNode
    pub fn build_nodes(path: &str, is_recursive: bool, sort: i64) -> Result<Self, String> {
        let root_path = Path::new(&path);

        // Check if the path exists and is a directory
        if !root_path.exists() {
            return Err(format!("Path does not exist: {}", path));
        }

        if !root_path.is_dir() {
            return Err(format!("Path is not a directory: {}", path));
        }

        // Create the root FileNode
        let root_meta = fs::metadata(root_path).ok();
        let mut root_node = FileNode::new(
            path,
            root_path.is_dir(),
            false,
            root_meta
                .as_ref()
                .and_then(|m| systemtime_to_timestamp(m.created().ok())),
            root_meta
                .as_ref()
                .and_then(|m| systemtime_to_timestamp(m.modified().ok())),
        );

        // Recursively read subfolders and files
        let album = Album::get_all_albums()?.into_iter()
            .filter(|album| root_path.starts_with(&album.path))
            .max_by_key(|album| Path::new(&album.path).components().count());
        root_node.children = Some(Self::recurse_nodes(root_path, is_recursive, sort, album.as_ref())?);
        root_node.has_subfolders = root_node
            .children
            .as_ref()
            .is_some_and(|children| !children.is_empty());

        Ok(root_node)
    }

    /// Recurse sub-folders
    fn recurse_nodes(path: &Path, is_recursive: bool, sort: i64, album: Option<&Album>) -> Result<Vec<Self>, String> {
        let mut nodes: Vec<FileNode> = Vec::new();

        for entry in WalkDir::new(path)
            .min_depth(1)
            .max_depth(1)
            .into_iter()
            .filter_entry(|entry| is_visible_or_root(entry) && !album.is_some_and(|album| album.excludes_path(entry.path())))
        {
            let entry = entry.map_err(|e| e.to_string())?;
            let entry_path = entry.path();
            let path_str = entry_path.to_string_lossy().to_string();

            if entry.file_type().is_dir() {
                let metadata = fs::metadata(entry_path).ok();
                let mut node = FileNode::new(
                    &path_str,
                    true,
                    false,
                    metadata
                        .as_ref()
                        .and_then(|m| systemtime_to_timestamp(m.created().ok())),
                    metadata
                        .as_ref()
                        .and_then(|m| systemtime_to_timestamp(m.modified().ok())),
                );

                if is_recursive {
                    node.children = Some(Self::recurse_nodes(entry_path, is_recursive, sort, album)?);
                    node.has_subfolders = node
                        .children
                        .as_ref()
                        .is_some_and(|children| !children.is_empty());
                }

                nodes.push(node);
            }
        }

        if !is_recursive {
            Self::hydrate_subfolder_states(&mut nodes);
        }

        match sort {
            1 => nodes.sort_by(|a, b| {
                natural_sort_key(&b.name.to_lowercase())
                    .cmp(&natural_sort_key(&a.name.to_lowercase()))
            }),
            2 => nodes.sort_by(|a, b| {
                a.modified_at
                    .or(a.created_at)
                    .unwrap_or(0)
                    .cmp(&b.modified_at.or(b.created_at).unwrap_or(0))
            }),
            3 => nodes.sort_by(|a, b| {
                b.modified_at
                    .or(b.created_at)
                    .unwrap_or(0)
                    .cmp(&a.modified_at.or(a.created_at).unwrap_or(0))
            }),
            _ => nodes.sort_by(|a, b| {
                natural_sort_key(&a.name.to_lowercase())
                    .cmp(&natural_sort_key(&b.name.to_lowercase()))
            }),
        }
        Ok(nodes)
    }

    /// Resolve direct-child state from the cache in one database query.  A
    /// shallow directory read is only needed when a folder has changed since
    /// it was indexed (or has not yet been indexed).
    fn hydrate_subfolder_states(nodes: &mut [Self]) {
        let paths = nodes.iter().map(|node| node.path.clone()).collect::<Vec<_>>();
        let cached = FolderSubfolderState::get_many(&paths).unwrap_or_else(|error| {
            eprintln!("Failed to read folder subfolder-state cache: {}", error);
            HashMap::new()
        });
        for node in nodes {
            let cached_state = cached.get(&node.path);
            if let (Some(current_mtime), Some(state)) = (node.modified_at, cached_state) {
                if state.modified_at == Some(current_mtime) {
                    node.has_subfolders = state.has_subfolders;
                    continue;
                }
            }

            // An unreadable directory is not known to be a leaf. Keep its
            // expand control available so a transient permission or network
            // error does not make the branch inaccessible from the sidebar.
            node.has_subfolders = detect_visible_subfolder(Path::new(&node.path)).unwrap_or(true);
        }
    }
}

/// True when the file name starts with `.` (Unix dotfile convention).
/// Non-UTF-8 names are silently treated as not-hidden.
fn is_dotfile(name: &std::ffi::OsStr) -> bool {
    name.to_str().map(|s| s.starts_with('.')).unwrap_or(false)
}

/// True on Windows when `FILE_ATTRIBUTE_HIDDEN` is set; always false on other platforms.
#[cfg(target_os = "windows")]
fn has_hidden_attribute(metadata: &std::fs::Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;
    const FILE_ATTRIBUTE_HIDDEN: u32 = 0x2;
    (metadata.file_attributes() & FILE_ATTRIBUTE_HIDDEN) != 0
}

#[cfg(not(target_os = "windows"))]
fn has_hidden_attribute(_metadata: &std::fs::Metadata) -> bool {
    false
}

/// True on Windows for a cloud-only placeholder (OneDrive Files On-Demand):
/// reading its content downloads it, reading this attribute does not.
#[cfg(target_os = "windows")]
pub fn is_cloud_only(metadata: &std::fs::Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;
    const FILE_ATTRIBUTE_RECALL_ON_DATA_ACCESS: u32 = 0x40_0000;
    (metadata.file_attributes() & FILE_ATTRIBUTE_RECALL_ON_DATA_ACCESS) != 0
}

#[cfg(not(target_os = "windows"))]
pub fn is_cloud_only(_metadata: &std::fs::Metadata) -> bool {
    false
}

pub fn is_hidden(entry: &walkdir::DirEntry) -> bool {
    is_dotfile(entry.file_name())
        || entry
            .metadata()
            .ok()
            .map_or(false, |m| has_hidden_attribute(&m))
}

/// Predicate for `WalkDir::filter_entry`. Always keeps the walk root (depth 0)
/// so an explicitly-selected NTFS volume root — which can report
/// `Hidden | System` attributes — is still traversed. Hidden filtering applies
/// only below the root.
pub fn is_visible_or_root(entry: &walkdir::DirEntry) -> bool {
    entry.depth() == 0 || !is_hidden(entry)
}

/// Hidden-file check for `std::fs::DirEntry` (used by fs::read_dir callers).
/// Note: `file_name()` returns an owned `OsString` (unlike `walkdir::DirEntry`'s `&OsStr`).
pub fn is_fs_entry_hidden(entry: &std::fs::DirEntry) -> bool {
    is_dotfile(&entry.file_name())
        || entry
            .metadata()
            .ok()
            .map_or(false, |m| has_hidden_attribute(&m))
}

/// Detect whether a directory contains a visible direct subdirectory.
/// This is only used to fill a missing or stale cache entry.
pub fn detect_visible_subfolder(path: &Path) -> std::io::Result<bool> {
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        if entry
            .file_type()
            .map(|file_type| file_type.is_dir())
            .unwrap_or(false)
            && !is_fs_entry_hidden(&entry)
        {
            return Ok(true);
        }
    }
    Ok(false)
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
    pub inode: u64,
}

impl FileInfo {
    /// Get file info from a folder/file path (on Windows)
    pub fn new(file_path: &str) -> Result<Self, String> {
        // Convert the string path into a Path object
        let path = Path::new(file_path);
        let metadata = fs::metadata(path).map_err(|e| e.to_string())?;

        let inode = file_id(path).unwrap_or(0);

        Ok(FileInfo {
            file_path: file_path.to_string(),
            file_name: get_file_name(file_path),
            file_type: metadata.file_type().is_dir().then(|| "dir".to_string()),
            created: systemtime_to_timestamp(metadata.created().ok()),
            modified: systemtime_to_timestamp(metadata.modified().ok()),
            file_size: metadata.len() as i64,
            inode,
        })
    }
}

/// Return a stable filesystem identifier for rename detection.
#[cfg(unix)]
fn file_id(path: &Path) -> Option<u64> {
    use std::os::unix::fs::MetadataExt;
    fs::metadata(path).ok().map(|m| m.ino())
}

#[cfg(windows)]
fn file_id(path: &Path) -> Option<u64> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use win32_imports::*;

    let path_wide: Vec<u16> = OsStr::new(path)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    unsafe {
        let handle = CreateFileW(
            path_wide.as_ptr(),
            0,   // dwDesiredAccess — 0 is sufficient for metadata
            0x7, // FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE
            core::ptr::null_mut(),
            3,           // OPEN_EXISTING
            0x0200_0000, // FILE_FLAG_BACKUP_SEMANTICS (required for directories)
            core::ptr::null_mut(),
        );
        if handle == INVALID_HANDLE_VALUE {
            return None;
        }
        let mut info = core::mem::zeroed::<BY_HANDLE_FILE_INFORMATION>();
        let ok = GetFileInformationByHandle(handle, &mut info);
        CloseHandle(handle);
        if ok == 0 {
            return None;
        }
        Some(((info.nFileIndexHigh as u64) << 32) | (info.nFileIndexLow as u64))
    }
}

#[cfg(windows)]
mod win32_imports {
    #![allow(non_camel_case_types, non_snake_case, unused)]

    pub type HANDLE = isize;

    #[repr(C)]
    pub struct FILETIME {
        pub dwLowDateTime: u32,
        pub dwHighDateTime: u32,
    }

    #[repr(C)]
    pub struct BY_HANDLE_FILE_INFORMATION {
        pub dwFileAttributes: u32,
        pub ftCreationTime: FILETIME,
        pub ftLastAccessTime: FILETIME,
        pub ftLastWriteTime: FILETIME,
        pub dwVolumeSerialNumber: u32,
        pub nFileSizeHigh: u32,
        pub nFileSizeLow: u32,
        pub nNumberOfLinks: u32,
        pub nFileIndexHigh: u32,
        pub nFileIndexLow: u32,
    }

    pub const INVALID_HANDLE_VALUE: HANDLE = -1isize;

    #[link(name = "kernel32")]
    unsafe extern "system" {
        pub fn CreateFileW(
            lpFileName: *const u16,
            dwDesiredAccess: u32,
            dwShareMode: u32,
            lpSecurityAttributes: *mut core::ffi::c_void,
            dwCreationDisposition: u32,
            dwFlagsAndAttributes: u32,
            hTemplateFile: *mut core::ffi::c_void,
        ) -> HANDLE;

        pub fn GetFileInformationByHandle(
            hFile: HANDLE,
            lpFileInformation: *mut BY_HANDLE_FILE_INFORMATION,
        ) -> i32;

        pub fn CloseHandle(hObject: HANDLE) -> i32;
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
            .env("LAP_APP_PATH", app_path)
            .creation_flags(CREATE_NO_WINDOW);
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

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FileConflictPolicy {
    KeepBoth,
    Replace,
}

impl FileConflictPolicy {
    pub fn from_str(value: &str) -> Self {
        match value {
            "replace" => Self::Replace,
            _ => Self::KeepBoth,
        }
    }
}

fn transfer_temp_path(destination: &Path, label: &str) -> PathBuf {
    let parent = destination.parent().unwrap_or_else(|| Path::new(""));
    parent.join(format!(".lap-{}-{}", label, uuid::Uuid::new_v4()))
}

fn resolve_transfer_destination(
    source: &Path,
    dest_folder: &str,
    policy: FileConflictPolicy,
) -> Result<PathBuf, String> {
    let mut destination = PathBuf::from(dest_folder);
    destination.push(
        source
            .file_name()
            .ok_or_else(|| format!("Invalid source path: {}", source.display()))?,
    );
    Ok(
        if destination.exists() && policy == FileConflictPolicy::KeepBoth {
            get_unique_path(destination)
        } else {
            destination
        },
    )
}

pub fn resolve_file_transfer_destination(
    file_path: &str,
    dest_folder: &str,
    policy: FileConflictPolicy,
) -> Result<PathBuf, String> {
    resolve_transfer_destination(Path::new(file_path), dest_folder, policy)
}

fn paths_refer_to_same_item(left: &Path, right: &Path) -> bool {
    match (left.canonicalize(), right.canonicalize()) {
        (Ok(left), Ok(right)) => left == right,
        _ => left == right,
    }
}

#[derive(Debug)]
pub struct TransferResult {
    pub path: String,
    backup: Option<PathBuf>,
}

impl TransferResult {
    fn new(destination: &Path, backup: Option<PathBuf>) -> Self {
        Self {
            path: destination.to_string_lossy().into_owned(),
            backup,
        }
    }

    pub fn finalize(self) -> Result<String, String> {
        if let Some(backup) = self.backup {
            let result = if backup.is_dir() {
                fs::remove_dir_all(&backup)
            } else {
                fs::remove_file(&backup)
            };
            if let Err(error) = result {
                eprintln!(
                    "Transfer completed but backup cleanup failed for '{}': {}",
                    backup.display(),
                    error
                );
            }
        }
        Ok(self.path)
    }

    pub fn rollback_move(self, source: &Path) -> Result<(), String> {
        let destination = PathBuf::from(&self.path);
        if fs::rename(&destination, source).is_err() {
            if destination.is_dir() {
                copy_folder_to(&destination, source).map_err(|e| {
                    format!(
                        "Failed to restore source folder '{}' from '{}': {}",
                        source.display(),
                        destination.display(),
                        e
                    )
                })?;
                fs::remove_dir_all(&destination)
                    .map_err(|e| format!("Failed to clean rollback source folder: {}", e))?;
            } else {
                copy_file_to_path(&destination, source).map_err(|e| {
                    format!(
                        "Failed to restore source file '{}' from '{}': {}",
                        source.display(),
                        destination.display(),
                        e
                    )
                })?;
                fs::remove_file(&destination)
                    .map_err(|e| format!("Failed to clean rollback source file: {}", e))?;
            }
        }
        self.restore_backup(&destination)
    }

    pub fn rollback_copy(self) -> Result<(), String> {
        let destination = PathBuf::from(&self.path);
        if destination.is_dir() {
            fs::remove_dir_all(&destination)
        } else {
            fs::remove_file(&destination)
        }
        .map_err(|e| format!("Failed to remove copied destination: {}", e))?;
        self.restore_backup(&destination)
    }

    fn restore_backup(self, destination: &Path) -> Result<(), String> {
        if let Some(backup) = self.backup {
            fs::rename(&backup, destination).map_err(|e| {
                format!(
                    "Failed to restore previous destination '{}' from '{}': {}",
                    destination.display(),
                    backup.display(),
                    e
                )
            })?;
        }
        Ok(())
    }
}

fn replace_path_with_staged(
    staged: &Path,
    destination: &Path,
) -> Result<Option<PathBuf>, std::io::Error> {
    if !destination.exists() {
        fs::rename(staged, destination)?;
        return Ok(None);
    }

    let backup = transfer_temp_path(destination, "backup");
    fs::rename(destination, &backup)?;
    if let Err(error) = fs::rename(staged, destination) {
        let _ = fs::rename(&backup, destination);
        return Err(error);
    }
    Ok(Some(backup))
}

pub fn move_folder_with_policy(
    folder_path: &str,
    dest_folder: &str,
    policy: FileConflictPolicy,
) -> Result<TransferResult, String> {
    let path = Path::new(folder_path);
    let destination_parent = Path::new(dest_folder);

    // Ensure the source folder exists
    if !path.exists() {
        return Err(format!("Folder does not exist: {}", folder_path));
    }
    if destination_parent.starts_with(path) || path.parent() == Some(destination_parent) {
        return Err(format!("Invalid folder move destination: {}", dest_folder));
    }

    let destination = resolve_transfer_destination(path, dest_folder, policy)?;
    if paths_refer_to_same_item(path, &destination) {
        return Err("Source and destination are the same folder".to_string());
    }

    if policy != FileConflictPolicy::Replace || !destination.exists() {
        if fs::rename(path, &destination).is_ok() {
            println!("Folder moved to: {}", destination.display());
            return Ok(TransferResult::new(&destination, None));
        }
    }

    let staged = transfer_temp_path(&destination, "move");
    if let Err(copy_error) = copy_folder_to(path, &staged) {
        let _ = fs::remove_dir_all(&staged);
        return Err(format!("Failed to stage folder move: {}", copy_error));
    }

    let backup = replace_path_with_staged(&staged, &destination).map_err(|replace_error| {
        let _ = fs::remove_dir_all(&staged);
        format!("Failed to finalize folder move: {}", replace_error)
    })?;

    if let Err(remove_error) = fs::remove_dir_all(path) {
        let rollback_error = TransferResult::new(&destination, backup)
            .rollback_move(path)
            .err();
        return Err(match rollback_error {
            Some(rollback_error) => format!(
                "Folder source cleanup failed: {}; rollback also failed: {}",
                remove_error, rollback_error
            ),
            None => format!(
                "Folder source cleanup failed; move was rolled back: {}",
                remove_error
            ),
        });
    }

    println!("Folder moved to: {}", destination.display());
    Ok(TransferResult::new(&destination, backup))
}

fn copy_folder_to(src: &Path, dst: &Path) -> Result<(), std::io::Error> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let source_path = entry.path();
        let destination_path = dst.join(entry.file_name());
        if source_path.is_dir() {
            copy_folder_to(&source_path, &destination_path)?;
        } else if let Err(error) = fs::copy(&source_path, &destination_path) {
            return Err(error);
        }
    }
    Ok(())
}

pub fn copy_folder_with_policy(
    folder_path: &str,
    dest_folder: &str,
    policy: FileConflictPolicy,
) -> Result<TransferResult, String> {
    let src = Path::new(folder_path);
    let destination_parent = Path::new(dest_folder);

    if !src.exists() || !src.is_dir() {
        return Err(format!(
            "Source folder does not exist or is not a directory: {}",
            folder_path
        ));
    }
    if destination_parent.starts_with(src) {
        return Err(format!("Invalid folder copy destination: {}", dest_folder));
    }

    let destination = resolve_transfer_destination(src, dest_folder, policy)?;
    if policy == FileConflictPolicy::Replace && paths_refer_to_same_item(src, &destination) {
        return Err("Cannot replace a folder with itself".to_string());
    }
    let staged = transfer_temp_path(&destination, "copy");
    if let Err(copy_error) = copy_folder_to(src, &staged) {
        let _ = fs::remove_dir_all(&staged);
        return Err(format!("Failed to copy folder: {}", copy_error));
    }

    let backup = replace_path_with_staged(&staged, &destination).map_err(|replace_error| {
        let _ = fs::remove_dir_all(&staged);
        format!("Failed to finalize folder copy: {}", replace_error)
    })?;

    println!("Folder copied successfully to: {}", destination.display());
    Ok(TransferResult::new(&destination, backup))
}

fn copy_file_to_path(source: &Path, destination: &Path) -> Result<(), std::io::Error> {
    if let Err(error) = fs::copy(source, destination) {
        let _ = fs::remove_file(destination);
        return Err(error);
    }
    Ok(())
}

pub fn move_file_to_path_with_policy(
    file_path: &str,
    destination: &Path,
    policy: FileConflictPolicy,
) -> Result<TransferResult, String> {
    let source = Path::new(file_path);
    if paths_refer_to_same_item(source, destination) {
        return Err("Source and destination are the same file".to_string());
    }
    if policy != FileConflictPolicy::Replace && destination.exists() {
        return Err(format!(
            "Destination file already exists: {}",
            destination.display()
        ));
    }

    if policy != FileConflictPolicy::Replace || !destination.exists() {
        if fs::rename(source, &destination).is_ok() {
            println!("File moved successfully: {}", destination.display());
            return Ok(TransferResult::new(&destination, None));
        }
    }

    let staged = transfer_temp_path(&destination, "move");
    if let Err(copy_error) = copy_file_to_path(source, &staged) {
        return Err(format!("Failed to stage file move: {}", copy_error));
    }
    let backup = replace_path_with_staged(&staged, &destination).map_err(|replace_error| {
        let _ = fs::remove_file(&staged);
        format!("Failed to finalize file move: {}", replace_error)
    })?;
    if let Err(remove_error) = fs::remove_file(source) {
        let rollback_error = TransferResult::new(&destination, backup)
            .rollback_move(source)
            .err();
        return Err(match rollback_error {
            Some(rollback_error) => format!(
                "File source cleanup failed: {}; rollback also failed: {}",
                remove_error, rollback_error
            ),
            None => format!(
                "File source cleanup failed; move was rolled back: {}",
                remove_error
            ),
        });
    }

    println!("File moved successfully: {}", destination.display());
    Ok(TransferResult::new(&destination, backup))
}

pub fn copy_file_to_path_with_policy(
    file_path: &str,
    destination: &Path,
    policy: FileConflictPolicy,
) -> Result<TransferResult, String> {
    let source = Path::new(file_path);
    if policy == FileConflictPolicy::Replace && paths_refer_to_same_item(source, destination) {
        return Err("Cannot replace a file with itself".to_string());
    }
    if policy != FileConflictPolicy::Replace && destination.exists() {
        return Err(format!(
            "Destination file already exists: {}",
            destination.display()
        ));
    }
    let staged = transfer_temp_path(&destination, "copy");
    if let Err(copy_error) = copy_file_to_path(source, &staged) {
        return Err(format!("Failed to copy file: {}", copy_error));
    }
    let backup = replace_path_with_staged(&staged, &destination).map_err(|replace_error| {
        let _ = fs::remove_file(&staged);
        format!("Failed to finalize file copy: {}", replace_error)
    })?;

    println!("File copied successfully: {}", destination.display());
    Ok(TransferResult::new(&destination, backup))
}

/// Import a file into a destination folder preserving the original file name.
/// If the target already exists, append a numeric suffix like `(1)`.
pub fn import_file(source_path: &str, dest_folder: &str) -> Option<String> {
    let source = Path::new(source_path);
    let mut destination = PathBuf::from(dest_folder);
    destination.push(source.file_name()?);
    // Reserve the destination atomically; concurrent imports must never overwrite it.
    let mut input = fs::File::open(source).ok()?;
    let permissions = input.metadata().ok()?.permissions();
    loop {
        let candidate = get_unique_path(destination.clone());
        let mut output = match fs::OpenOptions::new().write(true).create_new(true).open(&candidate) {
            Ok(file) => file,
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => {
                eprintln!("Failed to create imported file: {}", error);
                return None;
            }
        };
        let result = std::io::copy(&mut input, &mut output)
            .and_then(|_| output.set_permissions(permissions.clone()));
        if let Err(error) = result {
            drop(output);
            let _ = fs::remove_file(&candidate);
            eprintln!("Failed to import file: {}", error);
            return None;
        }
        return candidate.to_str().map(|path| path.to_string());
    }
}

// Compare in fixed-size chunks; unreadable files must never be treated as duplicates.
fn import_files_equal(left: &Path, right: &Path, cancelled: &impl Fn() -> bool) -> std::io::Result<bool> {
    let mut left = fs::File::open(left)?;
    let mut right = fs::File::open(right)?;
    let size = left.metadata()?.len();
    if size != right.metadata()?.len() { return Ok(false); }
    let mut remaining = size;
    let mut a = [0u8; 65536];
    let mut b = [0u8; 65536];
    while remaining > 0 {
        if cancelled() { return Ok(false); }
        let len = remaining.min(a.len() as u64) as usize;
        left.read_exact(&mut a[..len])?;
        right.read_exact(&mut b[..len])?;
        if a[..len] != b[..len] { return Ok(false); }
        remaining -= len as u64;
    }
    Ok(true)
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportOrganizeProgress {
    pub phase: String,
    pub processed: usize,
    pub total: usize,
    pub total_size: u64,
    pub imported: usize,
    pub skipped: usize,
    pub failed: usize,
    pub cancelled: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportOrganizeResult {
    pub completed_paths: Vec<String>,
    pub total: usize,
    pub imported: usize,
    pub skipped: usize,
    pub failed: usize,
    pub cancelled: bool,
}

/// Copy supported media from a source folder into a folder in an album and
/// create the corresponding folder/file records. `layout` is day, month, year,
/// or none.
pub fn import_and_organize<F, C>(
    album_id: i64,
    source_path: &str,
    destination_path: &str,
    layout: &str,
    completed_paths: HashSet<String>,
    mut report_progress: F,
    is_cancelled: C,
) -> Result<ImportOrganizeResult, String>
where
    F: FnMut(ImportOrganizeProgress),
    C: Fn() -> bool,
{
    let started = Instant::now();
    let mut aae_cache = t_apple_sidecar::ImportAaeCache::default();
    let album = Album::get_album_by_id(album_id)?;
    let source = fs::canonicalize(source_path)
        .map_err(|e| format!("Cannot access source folder: {}", e))?;
    if !source.is_dir() {
        return Err("The import source must be a folder".to_string());
    }
    let album_root = fs::canonicalize(&album.path)
        .map_err(|e| format!("Cannot access album folder: {}", e))?;
    let destination_folder = fs::canonicalize(destination_path)
        .map_err(|e| format!("Cannot access destination folder: {}", e))?;
    if !destination_folder.starts_with(&album_root) {
        return Err("The destination folder must be inside the album".to_string());
    }
    if source == destination_folder
        || destination_folder.starts_with(&source)
        || source.starts_with(&destination_folder)
    {
        return Err(
            "The import source and destination folders cannot contain one another".to_string(),
        );
    }
    let destination_components = destination_folder
        .strip_prefix(&album_root)
        .map_err(|_| "The destination folder must be inside the album".to_string())?
        .components()
        .map(|component| component.as_os_str().to_string_lossy().to_string())
        .collect::<Vec<_>>();

    report_progress(ImportOrganizeProgress {
        phase: "preparing".to_string(), processed: 0, total: 0, total_size: 0,
        imported: 0, skipped: 0, failed: 0, cancelled: false,
    });
    // Resume only files successfully imported by this dialog, including renamed collisions.
    let mut completed_paths = completed_paths;
    let mut last_progress = Instant::now();
    let mut files: Vec<(PathBuf, i64, String, Option<crate::t_libraw::RawInfo>)> = Vec::new();
    let mut failed = 0;
    let mut total_size: u64 = 0;
    for entry in WalkDir::new(&source)
        .into_iter()
        .filter_entry(is_visible_or_root)
    {
        if last_progress.elapsed() >= Duration::from_millis(100) {
            report_progress(ImportOrganizeProgress {
                phase: "preparing".to_string(), processed: files.len() + failed,
                total: files.len() + failed, total_size, imported: 0, skipped: 0, failed, cancelled: false,
            });
            last_progress = Instant::now();
        }
        if is_cancelled() {
            report_progress(ImportOrganizeProgress {
                phase: "preparing".to_string(), processed: files.len() + failed, total: files.len() + failed, total_size,
                imported: 0, skipped: 0, failed, cancelled: true,
            });
            return Ok(ImportOrganizeResult { completed_paths: completed_paths.into_iter().collect(), total: files.len() + failed, imported: 0, skipped: 0, failed, cancelled: true });
        }
        let Ok(entry) = entry else { continue; };
        if !entry.file_type().is_file() { continue; }
        let path = entry.into_path();
        let Some(file_type) = get_file_type(&path.to_string_lossy()) else { continue; };
        let file_size = path.metadata().map(|m| m.len()).unwrap_or(0);
        let path_string = path.to_string_lossy().to_string();
        if completed_paths.contains(&path_string) { continue; }
        total_size += file_size;
        let raw_info = if file_type == 3 && layout != "none" {
            crate::t_libraw::get_raw_info(&path_string).ok()
        } else { None };
        let relative_folder = match (|| -> Result<String, String> {
            if layout == "none" { return Ok(String::new()); }
            let timestamp = AFile::capture_timestamp_with_raw_info(&path_string, file_type, raw_info.as_ref())?;
            let date = Local.timestamp_opt(timestamp, 0).single()
                .ok_or_else(|| format!("Invalid capture date: {}", path_string))?;
            Ok(match layout {
                "day" => date.format("%Y/%Y-%m-%d").to_string(),
                "month" => date.format("%Y/%Y-%m").to_string(),
                "year" => date.format("%Y").to_string(),
                _ => return Err("Invalid import folder layout".to_string()),
            })
        })() {
            Ok(relative_folder) => relative_folder,
            Err(_) => {
                failed += 1;
                continue;
            }
        };
        files.push((path, file_type, relative_folder, raw_info));
    }
    eprintln!("Import preparation: {:?}, files: {}", started.elapsed(), files.len() + failed);
    let import_started = Instant::now();
    let mut compare_time = Duration::ZERO;
    let mut copy_time = Duration::ZERO;
    let mut database_time = Duration::ZERO;
    let total = files.len() + failed;
    let mut processed = failed;
    let mut imported = 0;
    let mut skipped = 0;
    let mut folder_ids: HashMap<PathBuf, i64> = HashMap::new();
    let mut candidates: HashMap<PathBuf, HashMap<u64, Vec<PathBuf>>> = HashMap::new();
    report_progress(ImportOrganizeProgress {
        phase: "importing".to_string(), processed,
        total, total_size, imported, skipped, failed, cancelled: false,
    });

    for (path, file_type, relative_folder, raw_info) in files {
        if is_cancelled() {
            report_progress(ImportOrganizeProgress {
                phase: "importing".to_string(), processed, total, total_size, imported, skipped, failed, cancelled: true,
            });
            return Ok(ImportOrganizeResult { completed_paths: completed_paths.into_iter().collect(), total, imported, skipped, failed, cancelled: true });
        }

        let path_string = path.to_string_lossy().to_string();
        let target_folder = destination_folder.join(&relative_folder);
        let result = (|| -> Result<bool, String> {
            // Cache file sizes once per target directory. Include successful copies below.
            if !candidates.contains_key(&target_folder) {
                let mut by_size: HashMap<u64, Vec<PathBuf>> = HashMap::new();
                if target_folder.exists() {
                    for entry in fs::read_dir(&target_folder).map_err(|e| e.to_string())? {
                        let entry = entry.map_err(|e| e.to_string())?;
                        let metadata = entry.metadata().map_err(|e| e.to_string())?;
                        if metadata.is_file() && get_file_type(&entry.path().to_string_lossy()).is_some() {
                            by_size.entry(metadata.len()).or_default().push(entry.path());
                        }
                    }
                }
                candidates.insert(target_folder.clone(), by_size);
            }
            let size = path.metadata().map_err(|e| e.to_string())?.len();
            let folder_id = if let Some(&id) = folder_ids.get(&target_folder) {
                id
            } else {
                fs::create_dir_all(&target_folder).map_err(|e| format!("Cannot create destination folder: {}", e))?;

                let mut folder = AFolder::add_to_db(album_id, &album_root.to_string_lossy())?;
                for component in destination_components.iter().map(String::as_str)
                    .chain(relative_folder.split('/').filter(|component| !component.is_empty()))
                {
                    let parent = Path::new(&folder.path).join(component);
                    folder = AFolder::add_to_db(album_id, &parent.to_string_lossy())?;
                }
                let id = folder.id.ok_or_else(|| "Imported folder is missing its database id".to_string())?;
                folder_ids.insert(target_folder.clone(), id);
                id
            };
            if let Some(matches) = candidates[&target_folder].get(&size) {
                for candidate in matches {
                    if is_cancelled() { return Err("Import cancelled".to_string()); }
                    let compare_started = Instant::now();
                    let duplicate = import_files_equal(&path, candidate, &is_cancelled).unwrap_or(false)
                        && {
                            let left = aae_cache.paths(&path);
                            let right = aae_cache.paths(candidate);
                            left.len() == right.len() && left.iter().zip(&right)
                                .all(|(a, b)| import_files_equal(a, b, &is_cancelled).unwrap_or(false))
                        };
                    compare_time += compare_started.elapsed();
                    if duplicate {
                        let database_started = Instant::now();
                        let destination = candidate.to_string_lossy().to_string();
                        if AFile::fetch(folder_id, &destination)?.is_none() {
                            let candidate_type = get_file_type(&destination)
                                .ok_or_else(|| format!("Unsupported import candidate: {}", destination))?;
                            AFile::add_to_db(folder_id, &destination, candidate_type, chrono::Utc::now().timestamp_millis())?;
                        }
                        database_time += database_started.elapsed();
                        return Ok(true);
                    }
                }
            }
            if is_cancelled() { return Err("Import cancelled".to_string()); }
            let copy_started = Instant::now();
            let destination = import_file(&path_string, &target_folder.to_string_lossy())
                .ok_or_else(|| format!("Failed to copy file: {}", path_string))?;
            let copied_sidecars = match t_apple_sidecar::copy_apple_aae_paths_for_import(
                &path_string,
                &destination,
                aae_cache.paths(&path),
            ) {
                Ok(sidecars) => sidecars,
                Err(error) => {
                    let _ = fs::remove_file(&destination);
                    return Err(error);
                }
            };
            copy_time += copy_started.elapsed();
            let database_started = Instant::now();
            if let Err(error) = AFile::add_to_db_with_raw_info(folder_id, &destination, file_type, chrono::Utc::now().timestamp_millis(), raw_info) {
                let _ = fs::remove_file(&destination);
                for sidecar in copied_sidecars {
                    let _ = fs::remove_file(sidecar);
                }
                return Err(error);
            }
            database_time += database_started.elapsed();
            aae_cache.record(&copied_sidecars);
            candidates.get_mut(&target_folder).unwrap().entry(size).or_default()
                .push(PathBuf::from(&destination));
            Ok(false)
        })();
        if result.is_err() && is_cancelled() {
            report_progress(ImportOrganizeProgress {
                phase: "importing".to_string(), processed,
                total, total_size, imported, skipped, failed, cancelled: true,
            });
            return Ok(ImportOrganizeResult { completed_paths: completed_paths.into_iter().collect(), total, imported, skipped, failed, cancelled: true });
        }
        match result {
            Ok(duplicate) => {
                if duplicate { skipped += 1; } else { imported += 1; }
                completed_paths.insert(path_string);
            }
            Err(_) => {
                failed += 1;
            }
        };
        processed += 1;
        report_progress(ImportOrganizeProgress {
            phase: "importing".to_string(), processed, total, total_size, imported, skipped, failed, cancelled: false,
        });
    }

    eprintln!("Import successful steps: comparison {:?}, copy/sidecars {:?}, file database {:?}", compare_time, copy_time, database_time);
    eprintln!("Import processing: {:?}, imported: {}, skipped: {}, failed: {}", import_started.elapsed(), imported, skipped, failed);
    Ok(ImportOrganizeResult { completed_paths: completed_paths.into_iter().collect(), total, imported, skipped, failed, cancelled: false })
}

/// Map an image MIME type to the canonical file extension.
/// Returns `None` for unsupported formats so callers can reject them
/// before writing a file.
pub fn image_mime_to_ext(content_type: &str) -> Option<&'static str> {
    let mime = content_type
        .split(';')
        .next()
        .unwrap_or(content_type)
        .trim();
    match mime {
        "image/jpeg" => Some("jpg"),
        "image/png" => Some("png"),
        "image/gif" => Some("gif"),
        "image/webp" => Some("webp"),
        "image/avif" => Some("avif"),
        "image/bmp" => Some("bmp"),
        _ => None,
    }
}

/// Validate that `bytes` starts with the expected magic bytes for `ext`.
fn validate_image_bytes(ext: &str, bytes: &[u8]) -> bool {
    match ext {
        "jpg" => bytes.len() >= 3 && bytes[0] == 0xFF && bytes[1] == 0xD8 && bytes[2] == 0xFF,
        "png" => bytes.len() >= 8 && &bytes[..8] == b"\x89PNG\r\n\x1a\n",
        "gif" => bytes.len() >= 4 && (&bytes[..3] == b"GIF"),
        "webp" => bytes.len() >= 12 && &bytes[..4] == b"RIFF" && &bytes[8..12] == b"WEBP",
        "bmp" => bytes.len() >= 2 && &bytes[..2] == b"BM",
        "avif" => {
            // ISOBMFF: 4 bytes size, then 'ftyp' box with 'avif' or 'avis' brand
            bytes.len() >= 12
                && &bytes[4..8] == b"ftyp"
                && (&bytes[8..12] == b"avif" || &bytes[8..12] == b"avis")
        }
        _ => false,
    }
}

/// Save bytes to a folder preserving the original file name.
/// If the target already exists, append a numeric suffix like `(1)`.
pub fn save_bytes_with_name(bytes: &[u8], name: &str, dest_folder: &str) -> Option<String> {
    let filename = Path::new(name)
        .file_name()
        .and_then(|item| item.to_str())
        .filter(|item| !item.trim().is_empty())?;
    let mut destination = PathBuf::from(dest_folder);
    destination.push(filename);
    let destination = get_unique_path(destination);

    let mut file = fs::File::create(&destination)
        .map_err(|e| eprintln!("Failed to create file: {}", e))
        .ok()?;
    std::io::Write::write_all(&mut file, bytes)
        .map_err(|e| eprintln!("Failed to write file: {}", e))
        .ok()?;
    println!("File saved with name: {}", destination.display());
    destination.to_str().map(|s| s.to_string())
}

/// Save downloaded bytes to a folder preserving the original file name when possible.
/// MIME and magic-byte validation are performed before writing.
pub fn save_downloaded_bytes_with_name(
    bytes: &[u8],
    content_type: &str,
    name: &str,
    dest_folder: &str,
) -> Option<String> {
    let ext = image_mime_to_ext(content_type)?;
    if !validate_image_bytes(ext, bytes) {
        eprintln!("Downloaded bytes do not match expected format for .{}", ext);
        return None;
    }

    let filename = Path::new(name)
        .file_name()
        .and_then(|item| item.to_str())
        .filter(|item| !item.trim().is_empty())?;
    let filename_path = Path::new(filename);
    let filename = match filename_path.extension().and_then(|item| item.to_str()) {
        Some(file_ext) if !file_ext.trim().is_empty() && get_file_type(filename).is_some() => {
            filename.to_string()
        }
        Some(_) => {
            let stem = filename_path
                .file_stem()
                .and_then(|item| item.to_str())
                .filter(|item| !item.trim().is_empty())
                .unwrap_or("image");
            format!("{}.{}", stem, ext)
        }
        _ => format!("{}.{}", filename, ext),
    };

    let mut destination = PathBuf::from(dest_folder);
    destination.push(filename);
    let destination = get_unique_path(destination);

    let mut file = fs::File::create(&destination)
        .map_err(|e| eprintln!("Failed to create file: {}", e))
        .ok()?;
    std::io::Write::write_all(&mut file, bytes)
        .map_err(|e| eprintln!("Failed to write file: {}", e))
        .ok()?;
    println!("File saved from URL: {}", destination.display());
    destination.to_str().map(|s| s.to_string())
}

/// Save downloaded bytes to a folder with auto-generated name.
/// Content type is used to determine the file extension.
/// Byte-level magic-number validation is performed before writing.
pub fn save_bytes_to_folder(bytes: &[u8], content_type: &str, dest_folder: &str) -> Option<String> {
    let ext = image_mime_to_ext(content_type)?;
    if !validate_image_bytes(ext, bytes) {
        eprintln!("Downloaded bytes do not match expected format for .{}", ext);
        return None;
    }

    let now = chrono::Local::now();
    let name = format!("IMG_{}.{}", now.format("%Y%m%d_%H%M%S"), ext);
    let mut destination = PathBuf::from(dest_folder);
    destination.push(&name);
    let destination = get_unique_path(destination);

    let mut file = fs::File::create(&destination)
        .map_err(|e| eprintln!("Failed to create file: {}", e))
        .ok()?;
    std::io::Write::write_all(&mut file, bytes)
        .map_err(|e| eprintln!("Failed to write file: {}", e))
        .ok()?;
    println!("File saved from URL: {}", destination.display());
    destination.to_str().map(|s| s.to_string())
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

/// reveal a file or folder in the file explorer (or finder)
pub fn reveal_path(path: &str) -> Result<(), String> {
    if path.trim().is_empty() {
        return Err("Missing path".to_string());
    }

    let is_dir = Path::new(path).is_dir();

    #[cfg(target_os = "macos")]
    {
        let mut cmd = Command::new("open");
        if !is_dir {
            cmd.arg("-R");
        }
        cmd.arg(path).spawn().map_err(|e| e.to_string())?;
    }

    #[cfg(target_os = "windows")]
    {
        let mut cmd = Command::new("explorer");
        if is_dir {
            cmd.arg(path);
        } else {
            cmd.arg("/select,").arg(path);
        }
        cmd.spawn().map_err(|e| e.to_string())?;
    }

    #[cfg(target_os = "linux")]
    {
        let target = Path::new(path);
        let reveal_target = if target.is_dir() {
            path.to_string()
        } else {
            target
                .parent()
                .and_then(|parent| parent.to_str())
                .ok_or_else(|| "Failed to resolve parent directory".to_string())?
                .to_string()
        };

        opener::open(reveal_target).map_err(|e| e.to_string())?;
    }

    Ok(())
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

    fn hide_live_photo_companion_videos(files: Vec<AFile>) -> Vec<AFile> {
        let companion_ids: HashSet<i64> = files
            .iter()
            .filter_map(|file| file.live_photo_video_id)
            .collect();

        if companion_ids.is_empty() {
            return files;
        }

        files
            .into_iter()
            .filter(|file| {
                file.id
                    .map(|id| !companion_ids.contains(&id))
                    .unwrap_or(true)
            })
            .collect()
    }

    let mut new_count = 0;
    let mut updated_count = 0;
    let folder = match AFolder::fetch(folder_path) {
        Ok(Some(folder)) if !album_removal_pending(folder.album_id) => folder,
        _ => return (Vec::new(), new_count, updated_count),
    };
    let album = match Album::get_album_by_id(folder.album_id) {
        Ok(album) if !album.excludes_path(Path::new(folder_path)) => album,
        _ => return (Vec::new(), 0, 0),
    };
    let album_sync_lock = album_sync_lock(folder.album_id);
    let _album_sync_guard = match album_sync_lock.lock() {
        Ok(guard) => guard,
        Err(_) => return (Vec::new(), new_count, updated_count),
    };
    let resolved_folder_id = match AFolder::fetch(folder_path) {
        Ok(Some(folder)) if !album_removal_pending(folder.album_id) => {
            let database_folder_id = folder.id.unwrap_or(folder_id);
            if database_folder_id != folder_id {
                eprintln!(
                    "get_folder_files: using folder id {} from DB for path {} instead of stale id {}",
                    database_folder_id, folder_path, folder_id
                );
            }
            database_folder_id
        }
        _ => return (Vec::new(), new_count, updated_count),
    };

    let mut files: Vec<AFile> = if from_db_only {
        match AFile::get_files_by_folder_id(resolved_folder_id) {
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
            .filter_entry(is_visible_or_root)
            .filter_map(Result::ok)
            .filter(|entry| entry.file_type().is_file())
        {
            let path = entry.path();
            let file_path_str = match path.to_str() {
                Some(p) => p,
                None => continue,
            };

            if let Some(ftype) = get_file_type(file_path_str) {
                if !album.allows_file_type(ftype) { continue; }
                match skip_excluded_image(&album, file_path_str, ftype) {
                    Ok(true) => continue,
                    Ok(false) => {},
                    Err(error) => {
                        eprintln!("Failed to retain excluded image: {error}");
                        continue;
                    }
                }
                let now = Utc::now().timestamp_millis();
                match AFile::add_to_db(resolved_folder_id, file_path_str, ftype, now) {
                    Ok((file, status)) => {
                        if status == 1 {
                            new_count += 1;
                        } else if status == 2 {
                            updated_count += 1;
                        }
                        if matches_file_type_filter(file_type, ftype) {
                            file_list.push(file);
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to add file to DB: {} ({})", file_path_str, e);
                    }
                }
            }
        }
        file_list
    };

    files.retain(|file| album.allows_file_type(file.file_type.unwrap_or_default()));
    files.retain(|file| {
        if file.file_type == Some(2) { return true; }
        let threshold = album.small_image_filter as u32;
        let width = file.width.unwrap_or(0);
        let height = file.height.unwrap_or(0);
        width == 0 || height == 0 || width >= threshold || height >= threshold
    });
    files = hide_live_photo_companion_videos(files);
    files.retain(|file| matches_file_type_filter(file_type, file.file_type.unwrap_or_default()));

    files.sort_by(|a, b| {
        let ordering = match sort_type {
            0 => a.taken_date.cmp(&b.taken_date),   // Taken Date
            1 => a.created_at.cmp(&b.created_at),   // Created Date
            2 => a.modified_at.cmp(&b.modified_at), // Modified Date
            3 => natural_sort_key(&a.name.to_lowercase()) // name
                .cmp(&natural_sort_key(&b.name.to_lowercase())), // support pinyin
            4 => a.size.cmp(&b.size),               // size
            5 => {
                if a.width == b.width {
                    a.height.cmp(&b.height)
                } else {
                    a.width.cmp(&b.width)
                }
            } // dimension
            6 => a.duration.cmp(&b.duration),       // duration
            9 => a.id.cmp(&b.id),                   // ID sort
            _ => a.taken_date.cmp(&b.taken_date),   // Default to taken date
        };
        if sort_order == 1 {
            ordering.reverse()
        } else {
            ordering
        }
    });

    (files, new_count, updated_count)
}

#[derive(serde::Serialize, Default)]
pub struct FolderMtimeSyncResult {
    pub dirty_folder_count: u32,
    pub new_folder_count: u32,
    pub new_file_count: u32,
    pub updated_file_count: u32,
    pub deleted_file_count: u32,
    pub rename_count: u32,
    pub deleted_folder_count: u32,
    pub folder_path_migrations: Vec<FolderPathMigration>,
}

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FolderPathMigration {
    pub album_id: i64,
    pub old_path: String,
    pub new_path: String,
}

struct SyncedFileTask {
    file_id: i64,
    file_path: String,
    file_type: i64,
    orientation: i32,
    album_id: i64,
    invalidate_thumbnail: bool,
}

#[derive(Default)]
struct FolderSyncOutcome {
    new_file_count: u32,
    updated_file_count: u32,
    deleted_file_count: u32,
    rename_count: u32,
    tasks: Vec<SyncedFileTask>,
}

const FOLDER_SYNC_THUMBNAIL_SIZE: u32 = 512;

fn is_path_not_found(err: &str) -> bool {
    err.contains("No such file or directory")
        || err.contains("The system cannot find the file specified")
}

/// Guard to ensure only one folder sync runs at a time. When a new sync
/// starts the previous one is cancelled (its generation is invalidated).
static FOLDER_SYNC_GENERATION: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
static ACTIVE_ALBUM_SCANS: Lazy<Mutex<HashSet<i64>>> = Lazy::new(|| Mutex::new(HashSet::new()));
#[derive(Default)]
struct AlbumAccessibilityCache {
    generation: u64,
    ids: HashSet<i64>,
}

impl AlbumAccessibilityCache {
    fn replace_if_current(&mut self, generation: u64, ids: HashSet<i64>) {
        if self.generation == generation { self.ids = ids; }
    }

    fn clear(&mut self) {
        self.generation = self.generation.wrapping_add(1);
        self.ids.clear();
    }
}

static INACCESSIBLE_ALBUM_IDS: Lazy<Mutex<AlbumAccessibilityCache>> =
    Lazy::new(|| Mutex::new(AlbumAccessibilityCache::default()));
static ALBUM_SYNC_LOCKS: Lazy<Mutex<HashMap<i64, Arc<Mutex<()>>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

static REMOVING_ALBUMS: Lazy<Mutex<HashMap<i64, usize>>> = Lazy::new(|| Mutex::new(HashMap::new()));

struct AlbumScanGuard {
    album_id: i64,
}

impl AlbumScanGuard {
    fn acquire(album_id: i64) -> Result<Self, String> {
        if album_removal_pending(album_id) {
            return Err(format!("Album {} is being removed", album_id));
        }
        let mut active = ACTIVE_ALBUM_SCANS.lock().unwrap();
        if !active.insert(album_id) {
            return Err(format!("Album {} is already being scanned", album_id));
        }
        Ok(Self { album_id })
    }
}

impl Drop for AlbumScanGuard {
    fn drop(&mut self) {
        ACTIVE_ALBUM_SCANS.lock().unwrap().remove(&self.album_id);
    }
}

/// Marks an album as being removed so no new scan can start while it is being
/// deleted. Cleared automatically on drop.
pub(crate) struct AlbumRemovalGuard {
    album_id: i64,
}

impl AlbumRemovalGuard {
    pub(crate) fn acquire(album_id: i64) -> Self {
        *REMOVING_ALBUMS
            .lock()
            .unwrap()
            .entry(album_id)
            .or_default() += 1;
        Self { album_id }
    }
}

impl Drop for AlbumRemovalGuard {
    fn drop(&mut self) {
        let mut removing = REMOVING_ALBUMS.lock().unwrap();
        if removing.get(&self.album_id).copied().unwrap_or(0) <= 1 {
            removing.remove(&self.album_id);
        } else if let Some(count) = removing.get_mut(&self.album_id) {
            *count -= 1;
        }
    }
}

fn album_removal_pending(album_id: i64) -> bool {
    REMOVING_ALBUMS.lock().unwrap().contains_key(&album_id)
}

fn album_scan_active(album_id: i64) -> bool {
    ACTIVE_ALBUM_SCANS.lock().unwrap().contains(&album_id)
}

pub fn album_sync_lock(album_id: i64) -> Arc<Mutex<()>> {
    ALBUM_SYNC_LOCKS
        .lock()
        .unwrap()
        .entry(album_id)
        .or_insert_with(|| Arc::new(Mutex::new(())))
        .clone()
}

/// Drop the per-album sync lock entry after deletion. Any stale queued sync
/// revalidates the album before writing and exits safely.
pub fn release_album_sync_lock(album_id: i64) {
    ALBUM_SYNC_LOCKS.lock().unwrap().remove(&album_id);
}

pub async fn wait_for_album_scan_end(album_id: i64) -> Result<(), String> {
    // Bounded wait: the scan honors the cancellation flag, so it should finish
    // shortly. A deadline prevents `remove_album` from hanging forever if the
    // scan is stuck.
    let deadline = Instant::now() + Duration::from_secs(30);
    while album_scan_active(album_id) {
        if Instant::now() >= deadline {
            return Err(format!(
                "Timed out waiting for album {} scan to finish",
                album_id
            ));
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    Ok(())
}

fn sync_generation_valid(generation: u64) -> bool {
    FOLDER_SYNC_GENERATION.load(std::sync::atomic::Ordering::SeqCst) == generation
}

pub fn start_folder_mtime_sync(app_handle: tauri::AppHandle) {
    let generation = FOLDER_SYNC_GENERATION
        .fetch_add(1, std::sync::atomic::Ordering::SeqCst)
        .wrapping_add(1);

    tauri::async_runtime::spawn_blocking(move || {
        match sync_dirty_folders_by_mtime(generation) {
            Ok((result, tasks)) => {
                if !sync_generation_valid(generation) {
                    return;
                }
                if !result.folder_path_migrations.is_empty() {
                    let _ = app_handle.emit(
                        "album-folder-paths-migrated",
                        &result.folder_path_migrations,
                    );
                }
                for task in tasks {
                    schedule_synced_file_processing(app_handle.clone(), task);
                }
                if result.dirty_folder_count > 0 || result.new_folder_count > 0 {
                    let _ = app_handle.emit("library-folder-sync-finished", &result);
                    let _ = app_handle.emit("library-total-refreshed", ());
                }
            }
            Err(e) => eprintln!("folder mtime sync failed: {}", e),
        }
    });
}

/// Check every known folder in the current library using directory mtime.
/// Dirty folders are synced in the background without touching unrelated views.
fn sync_dirty_folders_by_mtime(
    generation: u64,
) -> Result<(FolderMtimeSyncResult, Vec<SyncedFileTask>), String> {
    let mut dirty_folder_count = 0u32;
    let mut new_folder_count = 0u32;
    let mut new_file_count = 0u32;
    let mut updated_file_count = 0u32;
    let mut deleted_file_count = 0u32;
    let mut rename_count = 0u32;
    let mut deleted_folder_count = 0u32;
    let mut folder_path_migrations = Vec::new();
    let mut tasks = Vec::new();
    let mut queue = Vec::new();
    let mut album_accessibility = HashMap::new();
    let pending_live_photo_folders = FolderScanState::folders_needing_version(
        FolderScanState::LIVE_PHOTO_PAIRING,
        FolderScanState::LIVE_PHOTO_PAIRING_VERSION,
    )?;

    for folder in AFolder::get_all()? {
        if !sync_generation_valid(generation) {
            return Ok((FolderMtimeSyncResult::default(), Vec::new()));
        }
        if album_scan_active(folder.album_id) {
            continue;
        }
        let root_accessible = *album_accessibility
            .entry(folder.album_id)
            .or_insert_with(|| {
                Album::get_album_by_id(folder.album_id)
                    .map(|album| directory_accessible(&album.path))
                    .unwrap_or(false)
            });
        if !root_accessible {
            continue;
        }
        let info = match FileInfo::new(&folder.path) {
            Ok(info) => info,
            Err(e) => {
                if is_path_not_found(&e) {
                    // A missing folder may have been renamed or moved outside
                    // its parent. Keep its metadata until an explicit parent
                    // refresh can reconcile or remove it.
                } else {
                    eprintln!(
                        "sync_dirty_folders_by_mtime: failed to stat {} ({})",
                        folder.path, e
                    );
                }
                continue;
            }
        };
        let needs_live_photo_reindex = folder
            .id
            .map(|id| pending_live_photo_folders.contains(&id))
            .unwrap_or(true);
        if info.modified != folder.modified_at || needs_live_photo_reindex {
            queue.push((folder, info.modified, needs_live_photo_reindex));
        }
    }

    while let Some((folder, latest_mtime, needs_live_photo_reindex)) = queue.pop() {
        if !sync_generation_valid(generation) {
            return Ok((FolderMtimeSyncResult::default(), Vec::new()));
        }
        if album_scan_active(folder.album_id) || album_removal_pending(folder.album_id) {
            continue;
        }
        let album_sync_lock = album_sync_lock(folder.album_id);
        let _album_sync_guard = album_sync_lock
            .lock()
            .map_err(|_| format!("Album sync lock poisoned: {}", folder.album_id))?;
        if !sync_generation_valid(generation)
            || album_scan_active(folder.album_id)
            || album_removal_pending(folder.album_id)
        {
            continue;
        }
        if Album::get_album_by_id(folder.album_id).is_err() {
            continue;
        }
        let folder_id = match folder.id {
            Some(id) => id,
            None => continue,
        };
        dirty_folder_count += 1;

        let child_scan = scan_new_child_folders(folder.album_id, &folder.path, false)?;
        new_folder_count += child_scan.folders.len() as u32;
        deleted_folder_count += child_scan.deleted_folder_count;
        folder_path_migrations.extend(child_scan.folder_path_migrations);
        for child in child_scan.folders {
            queue.push((child, None, true));
        }

        let outcome = sync_folder_direct_files(
            folder_id,
            folder.album_id,
            &folder.path,
            generation,
            needs_live_photo_reindex,
        )?;
        new_file_count += outcome.new_file_count;
        updated_file_count += outcome.updated_file_count;
        deleted_file_count += outcome.deleted_file_count;
        rename_count += outcome.rename_count;
        tasks.extend(outcome.tasks);

        let modified_at = latest_mtime.or_else(|| {
            FileInfo::new(&folder.path)
                .ok()
                .and_then(|info| info.modified)
        });
        let _ = FolderSubfolderState::update_after_scan(folder.album_id, &[FolderSubfolderState {
            path: folder.path,
            created_at: None,
            modified_at,
            inode: None,
            has_subfolders: child_scan.has_subfolders,
        }]);
    }

    Ok((
        FolderMtimeSyncResult {
            dirty_folder_count,
            new_folder_count,
            new_file_count,
            updated_file_count,
            deleted_file_count,
            rename_count,
            deleted_folder_count,
            folder_path_migrations,
        },
        tasks,
    ))
}

struct ChildFolderScan {
    folders: Vec<AFolder>,
    child_paths: Vec<String>,
    has_subfolders: bool,
    deleted_folder_count: u32,
    folder_path_migrations: Vec<FolderPathMigration>,
}

fn scan_new_child_folders(
    album_id: i64,
    folder_path: &str,
    reconcile_removed_children: bool,
) -> Result<ChildFolderScan, String> {
    let album = Album::get_album_by_id(album_id)?;
    if album.excludes_path(Path::new(folder_path)) {
        return Ok(ChildFolderScan { folders: Vec::new(), child_paths: Vec::new(), has_subfolders: false,
            deleted_folder_count: 0, folder_path_migrations: Vec::new() });
    }
    let entries = fs::read_dir(folder_path).map_err(|e| e.to_string())?;
    let mut has_subfolders = false;
    let mut seen_paths = HashSet::new();

    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        if is_fs_entry_hidden(&entry) {
            continue;
        }
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        if !file_type.is_dir() {
            continue;
        }
        if album.excludes_path(&entry.path()) { continue; }
        has_subfolders = true;

        let child_path = entry.path().to_string_lossy().to_string();
        seen_paths.insert(child_path.clone());
    }

    let mut new_folders = Vec::new();
    let mut folder_path_migrations = Vec::new();
    for child_path in &seen_paths {
        if AFolder::fetch(&child_path)?.is_some() {
            continue;
        }
        let inode = FileInfo::new(child_path)
            .ok()
            .map(|info| info.inode as i64)
            .filter(|inode| *inode != 0);
        if let Some(old_path) = AFolder::migrate_path_by_inode(album_id, child_path, inode)? {
            folder_path_migrations.push(FolderPathMigration {
                album_id,
                old_path,
                new_path: child_path.clone(),
            });
        }
        if AFolder::fetch(&child_path)?.is_some() {
            continue;
        }
        new_folders.push(AFolder::add_to_db(album_id, child_path)?);
    }

    // The background mtime sync visits every known folder but preserves
    // missing paths. Avoid loading every album path once per dirty folder;
    // reconciliation is needed only for an explicit subfolder refresh.
    let deleted_folder_count = if reconcile_removed_children {
        AFolder::delete_unseen_direct_children(album_id, folder_path, &seen_paths)? as u32
    } else {
        0
    };

    Ok(ChildFolderScan {
        folders: new_folders,
        child_paths: seen_paths.into_iter().collect(),
        has_subfolders,
        deleted_folder_count,
        folder_path_migrations,
    })
}

fn is_live_photo_candidate_name(name: &str) -> bool {
    matches!(
        Path::new(name)
            .extension()
            .and_then(|extension| extension.to_str())
            .unwrap_or("")
            .to_ascii_lowercase()
            .as_str(),
        "heic" | "heif" | "hif" | "jpg" | "jpeg" | "mov"
    )
}

fn sync_folder_direct_files(
    folder_id: i64,
    album_id: i64,
    folder_path: &str,
    generation: u64,
    full_live_photo_reindex: bool,
) -> Result<FolderSyncOutcome, String> {
    let album = Album::get_album_by_id(album_id)?;
    if album.excludes_path(Path::new(folder_path)) { return Ok(FolderSyncOutcome::default()); }
    let scan_time = Utc::now().timestamp_millis();
    let mut seen_names = HashSet::new();
    let mut seen_inodes = HashSet::new();
    let mut new_count = 0u32;
    let mut updated_count = 0u32;
    let mut rename_count = 0u32;
    let mut tasks = Vec::new();
    let mut live_photo_affected_names = HashSet::new();

    // Helper to check background cancellation (generation 0 = foreground, never cancels).
    let is_cancelled =
        || generation != 0 && (!sync_generation_valid(generation) || album_scan_active(album_id));

    // Build a map of file_id → (db_id, db_name) for rename detection.
    // Use the stored inode from the DB record — this works even when the
    // file has been renamed and the old path no longer exists on disk.
    let db_inodes: HashMap<u64, (i64, String)> = {
        let mut map = HashMap::new();
        if let Ok(files) = AFile::get_files_by_folder_id(folder_id) {
            for file in files {
                if let (Some(id), Some(inode)) = (file.id, file.inode) {
                    if inode > 0 {
                        map.insert(inode as u64, (id, file.name.clone()));
                    }
                }
            }
        }
        map
    };

    for entry in WalkDir::new(folder_path)
        .min_depth(1)
        .max_depth(1)
        .into_iter()
        .filter_entry(is_visible_or_root)
    {
        let entry = entry.map_err(|e| e.to_string())?;
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        let file_path_str = match path.to_str() {
            Some(p) => p,
            None => continue,
        };
        let file_name = get_file_name(file_path_str);
        seen_names.insert(file_name.clone());

        // Record file_id for rename detection.
        if let Some(fid) = file_id(path) {
            seen_inodes.insert(fid);
        }

        if let Some(ftype) = get_file_type(file_path_str) {
            if !album.allows_file_type(ftype) || skip_excluded_image(&album, file_path_str, ftype)? { continue; }
            // Check for rename: a file whose file_id matches a known DB record
            // with a different name.
            let renamed: Option<(i64, String)> = file_id(path).and_then(|fid| {
                if fid > 0 {
                    db_inodes.get(&fid).and_then(|(db_id, db_name)| {
                        if db_name != &file_name {
                            Some((*db_id, db_name.clone()))
                        } else {
                            None
                        }
                    })
                } else {
                    None
                }
            });

            if let Some((db_id, old_file_name)) = renamed {
                if is_cancelled() {
                    return Ok(FolderSyncOutcome::default());
                }
                // Update the existing DB record to point to the new name/path.
                if let Ok(Some(updated_file)) =
                    AFile::update_file_info(db_id, file_path_str, scan_time)
                {
                    rename_count += 1;
                    if is_live_photo_candidate_name(&file_name) {
                        live_photo_affected_names.insert(file_name.clone());
                    }
                    if is_live_photo_candidate_name(&old_file_name) {
                        live_photo_affected_names.insert(old_file_name);
                    }
                    if should_process_synced_file(&updated_file, ftype, album.small_image_filter) {
                        tasks.push(SyncedFileTask {
                            file_id: db_id,
                            file_path: file_path_str.to_string(),
                            file_type: ftype,
                            orientation: updated_file.e_orientation.unwrap_or(1) as i32,
                            album_id,
                            invalidate_thumbnail: false,
                        });
                    }
                }
            } else {
                if is_cancelled() {
                    return Ok(FolderSyncOutcome::default());
                }
                match AFile::add_to_db(folder_id, file_path_str, ftype, scan_time) {
                    Ok((file, status)) => {
                        if status == 1 {
                            new_count += 1;
                            if is_live_photo_candidate_name(&file_name) {
                                live_photo_affected_names.insert(file_name.clone());
                            }
                        } else if status == 2 {
                            updated_count += 1;
                            if is_live_photo_candidate_name(&file_name) {
                                live_photo_affected_names.insert(file_name.clone());
                            }
                        }
                        if should_process_synced_file(&file, ftype, album.small_image_filter) {
                            if let Some(file_id) = file.id {
                                tasks.push(SyncedFileTask {
                                    file_id,
                                    file_path: file_path_str.to_string(),
                                    file_type: ftype,
                                    orientation: file.e_orientation.unwrap_or(1) as i32,
                                    album_id,
                                    invalidate_thumbnail: status == 2,
                                });
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!(
                            "sync_folder: failed to add file to DB: {} ({})",
                            file_path_str, e
                        );
                    }
                }
            }
        }
    }

    // Delete DB records that are truly gone (name not on disk AND file_id not seen).
    // Uses the stored inode so rename detection works even when the old path
    // no longer exists on disk.
    if is_cancelled() {
        return Ok(FolderSyncOutcome::default());
    }
    let album = Album::get_album_by_id(album_id).map_err(|e| e.to_string())?;
    if !directory_accessible(&album.path) || !directory_accessible(folder_path) {
        return Err(format!(
            "Folder became inaccessible during sync: {}",
            folder_path
        ));
    }
    let deleted_count: u32 = {
        let mut count = 0u32;
        if let Ok(files) = AFile::get_files_by_folder_id(folder_id) {
            for file in files {
                if !file.album_visible || seen_names.contains(&file.name) {
                    continue;
                }
                let still_exists = file
                    .inode
                    .map(|ino| ino > 0 && seen_inodes.contains(&(ino as u64)))
                    .unwrap_or(false);
                if still_exists {
                    continue;
                }
                if let Some(id) = file.id {
                    if is_cancelled() {
                        return Ok(FolderSyncOutcome::default());
                    }
                    if AFile::delete(id).is_ok() {
                        count += 1;
                        if is_live_photo_candidate_name(&file.name) {
                            live_photo_affected_names.insert(file.name.clone());
                        }
                    }
                }
            }
        }
        count
    };

    if full_live_photo_reindex {
        let files = AFile::get_files_by_folder_id(folder_id).map_err(|error| {
            format!(
                "Failed to load Live Photo reindex candidates for folder {}: {}",
                folder_path, error
            )
        })?;
        for file in files {
            if is_live_photo_candidate_name(&file.name) {
                live_photo_affected_names.insert(file.name);
            }
        }
    }

    let pairing_result =
        AFile::pair_live_photos_in_folder(
            folder_id,
            &live_photo_affected_names,
            full_live_photo_reindex,
        );
    if full_live_photo_reindex {
        pairing_result.map_err(|error| {
            format!(
                "Failed to complete Live Photo reindex for folder {}: {}",
                folder_path, error
            )
        })?;
        FolderScanState::mark_completed(
            folder_id,
            FolderScanState::LIVE_PHOTO_PAIRING,
            FolderScanState::LIVE_PHOTO_PAIRING_VERSION,
        )
        .map_err(|error| {
            format!(
                "Failed to mark Live Photo reindex complete for folder {}: {}",
                folder_path, error
            )
        })?;
    } else if let Err(error) = pairing_result {
        eprintln!(
            "Failed to pair Live Photos in folder {}: {}",
            folder_path, error
        );
    }

    Ok(FolderSyncOutcome {
        new_file_count: new_count,
        updated_file_count: updated_count,
        deleted_file_count: deleted_count,
        rename_count,
        tasks,
    })
}

fn pair_live_photos_after_album_index(album_id: i64) -> Result<(), String> {
    for folder in AFolder::get_all()?.into_iter().filter(|folder| folder.album_id == album_id) {
        let folder_id = folder
            .id
            .ok_or_else(|| format!("Folder has no id: {}", folder.path))?;

        // A Live Photo always has a MOV companion. Avoid loading every file
        // and constructing pairing candidates for ordinary folders, while
        // still clearing associations whose companion MOV was removed.
        let (has_mov, has_live_photo_pairs) = AFile::live_photo_folder_state(folder_id)?;
        if !has_mov {
            if has_live_photo_pairs {
                AFile::clear_live_photo_pairs_in_folder(folder_id)?;
            }
            continue;
        }

        let files = AFile::get_files_by_folder_id(folder_id)?;
        let affected_names = files
            .into_iter()
            .filter_map(|file| is_live_photo_candidate_name(&file.name).then_some(file.name))
            .collect::<HashSet<_>>();

        let refresh_missing_identifiers = FolderScanState::needs_version(
            folder_id,
            FolderScanState::LIVE_PHOTO_PAIRING,
            FolderScanState::LIVE_PHOTO_PAIRING_VERSION,
        )?;
        AFile::pair_live_photos_in_folder(
            folder_id,
            &affected_names,
            refresh_missing_identifiers,
        )?;
        FolderScanState::mark_completed(
            folder_id,
            FolderScanState::LIVE_PHOTO_PAIRING,
            FolderScanState::LIVE_PHOTO_PAIRING_VERSION,
        )?;
    }
    Ok(())
}

/// RAW+JPEG pairing is derived from the complete folder index, just like Live
/// Photo pairing.  Doing this after an album scan covers files which already
/// existed before grouping was enabled and folders not touched by an
/// incremental mtime sync.
fn reconcile_raw_jpeg_pairs_after_album_index(
    album_id: i64,
    group_raw_jpeg_pairs: bool,
) -> Result<(), String> {
    if !group_raw_jpeg_pairs {
        AFile::clear_raw_jpeg_pairs_in_album(album_id)?;
        return Ok(());
    }
    for folder in AFolder::get_all()?
        .into_iter()
        .filter(|folder| folder.album_id == album_id)
    {
        let folder_id = folder
            .id
            .ok_or_else(|| format!("Folder has no id: {}", folder.path))?;
        AFile::pair_raw_jpeg_in_folder(folder_id)?;
    }
    Ok(())
}

/// Sync a single folder when its directory mtime has changed.
/// Returns counts and schedules thumbnail/embedding generation.
pub fn sync_single_folder(
    app_handle: &tauri::AppHandle,
    album_id: i64,
    folder_id: i64,
    folder_path: &str,
    group_raw_jpeg_pairs: bool,
) -> Result<FolderMtimeSyncResult, String> {
    // A complete album scan owns folder and file reconciliation for this
    // album. Skip foreground refreshes until it finishes to avoid concurrent
    // writes based on different filesystem snapshots.
    if album_scan_active(album_id) || album_removal_pending(album_id) {
        return Ok(FolderMtimeSyncResult::default());
    }
    let album_sync_lock = album_sync_lock(album_id);
    let _album_sync_guard = album_sync_lock
        .lock()
        .map_err(|_| format!("Album sync lock poisoned: {}", album_id))?;
    if album_scan_active(album_id) || album_removal_pending(album_id) {
        return Ok(FolderMtimeSyncResult::default());
    }

    let album = Album::get_album_by_id(album_id).map_err(|e| e.to_string())?;
    if !directory_accessible(&album.path) {
        return Err(format!("Album folder is not accessible: {}", album.path));
    }

    let info = match FileInfo::new(folder_path) {
        Ok(info) => info,
        Err(e) => {
            if is_path_not_found(&e) {
                // Keep missing folders until an explicit parent refresh can
                // reconcile a rename or remove an intentionally deleted path.
                return Ok(FolderMtimeSyncResult {
                    dirty_folder_count: 0,
                    new_folder_count: 0,
                    new_file_count: 0,
                    updated_file_count: 0,
                    deleted_file_count: 0,
                    rename_count: 0,
                    deleted_folder_count: 0,
                    folder_path_migrations: Vec::new(),
                });
            }
            return Err(e);
        }
    };
    let folder =
        AFolder::fetch(folder_path)?.ok_or_else(|| format!("Folder not found: {}", folder_path))?;

    let resolved_folder_id = folder
        .id
        .ok_or_else(|| format!("Folder has no id: {}", folder_path))?;
    if resolved_folder_id != folder_id {
        eprintln!(
            "sync_single_folder: using DB folder id {} for path {} instead of stale id {}",
            resolved_folder_id, folder_path, folder_id
        );
    }
    if folder.album_id != album_id {
        return Err(format!(
            "Folder album_id mismatch: expected {}, found {}",
            album_id, folder.album_id
        ));
    }

    // Always reconcile direct child additions before deciding whether file
    // metadata needs an mtime-driven update.
    let child_scan = scan_new_child_folders(album_id, folder_path, false)?;
    let new_folder_count = child_scan.folders.len() as u32;

    let needs_live_photo_reindex = FolderScanState::needs_version(
        resolved_folder_id,
        FolderScanState::LIVE_PHOTO_PAIRING,
        FolderScanState::LIVE_PHOTO_PAIRING_VERSION,
    )?;
    if info.modified == folder.modified_at && !needs_live_photo_reindex {
        // An unchanged mtime-driven sync still reconciles the existing pair state.
        // This is intentionally the only incremental path that receives the
        // setting; changing Settings alone does not mutate the database.
        let raw_pairing_changed = if group_raw_jpeg_pairs {
            AFile::pair_raw_jpeg_in_folder(resolved_folder_id)? > 0
        } else {
            AFile::clear_raw_jpeg_pairs_in_folder(resolved_folder_id)? > 0
        };
        let folder_layout_changed = new_folder_count > 0 || child_scan.deleted_folder_count > 0;
        let _ = FolderSubfolderState::update_after_scan(album_id, &[FolderSubfolderState {
            path: folder_path.to_string(),
            created_at: None,
            modified_at: info.modified,
            inode: None,
            has_subfolders: child_scan.has_subfolders,
        }]);
        return Ok(FolderMtimeSyncResult {
            // The frontend uses this to decide whether it needs to reload the
            // current list.  A newly created or cleared association changes
            // which companion rows are visible.
            dirty_folder_count: u32::from(raw_pairing_changed || folder_layout_changed),
            new_folder_count,
            new_file_count: 0,
            updated_file_count: 0,
            deleted_file_count: 0,
            rename_count: 0,
            deleted_folder_count: child_scan.deleted_folder_count,
            folder_path_migrations: child_scan.folder_path_migrations,
        });
    }

    let outcome = sync_folder_direct_files(
        resolved_folder_id,
        album_id,
        folder_path,
        0,
        needs_live_photo_reindex,
    )?; // 0 = foreground, never cancel
    if group_raw_jpeg_pairs {
        AFile::pair_raw_jpeg_in_folder(resolved_folder_id)?;
    } else {
        AFile::clear_raw_jpeg_pairs_in_folder(resolved_folder_id)?;
    }
    for task in outcome.tasks {
        schedule_synced_file_processing(app_handle.clone(), task);
    }

    let mtime = info.modified;
    let _ = FolderSubfolderState::update_after_scan(album_id, &[FolderSubfolderState {
        path: folder_path.to_string(),
        created_at: None,
        modified_at: mtime,
        inode: None,
        has_subfolders: child_scan.has_subfolders,
    }]);

    Ok(FolderMtimeSyncResult {
        dirty_folder_count: 1,
        new_folder_count,
        new_file_count: outcome.new_file_count,
        updated_file_count: outcome.updated_file_count,
        deleted_file_count: outcome.deleted_file_count,
        rename_count: outcome.rename_count,
        deleted_folder_count: child_scan.deleted_folder_count,
        folder_path_migrations: child_scan.folder_path_migrations,
    })
}

/// Recursively reconcile the folder hierarchy below `folder_path` without
/// enumerating or updating any file records.
pub fn refresh_album_subfolders(
    album_id: i64,
    folder_path: &str,
) -> Result<Vec<FolderPathMigration>, String> {
    if album_scan_active(album_id) {
        return Err("Album scan is already in progress.".to_string());
    }
    if album_removal_pending(album_id) {
        return Err("Album is being removed.".to_string());
    }

    let album_sync_lock = album_sync_lock(album_id);
    let _album_sync_guard = album_sync_lock
        .lock()
        .map_err(|_| format!("Album sync lock poisoned: {album_id}"))?;
    if album_scan_active(album_id) || album_removal_pending(album_id) {
        return Err("Album is unavailable for subfolder refresh.".to_string());
    }

    let folder = AFolder::fetch(folder_path)?
        .ok_or_else(|| format!("Folder not found: {folder_path}"))?;
    if folder.album_id != album_id {
        return Err(format!("Folder does not belong to album: {folder_path}"));
    }
    if !directory_accessible(folder_path) {
        return Err(format!("Folder is not accessible: {folder_path}"));
    }

    let mut pending = vec![folder_path.to_string()];
    let mut migrations = Vec::new();
    let mut subfolder_flags = Vec::new();
    while let Some(path) = pending.pop() {
        let scan = scan_new_child_folders(album_id, &path, true)?;
        migrations.extend(scan.folder_path_migrations);
        subfolder_flags.push((path, scan.has_subfolders));
        pending.extend(scan.child_paths);
    }
    FolderSubfolderState::update_subfolder_flags(album_id, &subfolder_flags)?;
    Ok(migrations)
}

fn should_process_synced_file(file: &AFile, file_type: i64, threshold: i64) -> bool {
    if !file.album_visible || dimensions_excluded(file_type, file.width.unwrap_or(0), file.height.unwrap_or(0), threshold) { return false; }
    if !file.has_thumbnail.unwrap_or(false) {
        return true;
    }
    matches!(file_type, 1 | 3) && !file.has_embedding.unwrap_or(false)
}

fn schedule_synced_file_processing(app_handle: tauri::AppHandle, task: SyncedFileTask) {
    tauri::async_runtime::spawn(async move {
        if album_removal_pending(task.album_id) {
            return;
        }
        let file_path_for_thumb = task.file_path.clone();
        let file_type = task.file_type;
        let orientation = task.orientation;
        let file_id = task.file_id;
        let album_id = task.album_id;
        let thumb_result = tauri::async_runtime::spawn_blocking(move || {
            let album_sync_lock = album_sync_lock(album_id);
            let _album_sync_guard = match album_sync_lock.lock() {
                Ok(guard) => guard,
                Err(_) => return Ok(None),
            };
            if album_removal_pending(album_id) || Album::get_album_by_id(album_id).is_err() {
                return Ok(None);
            }
            if !AFile::is_album_visible(file_id)? {
                return Ok(None);
            }
            AThumb::get_or_create_thumb(
                file_id,
                &file_path_for_thumb,
                file_type,
                orientation,
                FOLDER_SYNC_THUMBNAIL_SIZE,
                RawDisplayOptions::rendered_bright(),
                false,
                None,
                None,
            )
        })
        .await;

        if !matches!(thumb_result, Ok(Ok(Some(_)))) || album_removal_pending(task.album_id) {
            return;
        }

        let _ = app_handle.emit(
            "thumbnail_ready",
            serde_json::json!({
                "album_id": task.album_id,
                "file_ids": [task.file_id],
                "invalidate": task.invalidate_thumbnail,
            }),
        );

        if !matches!(task.file_type, 1 | 3) {
            return;
        }

        let app_handle_for_embedding = app_handle.clone();
        let file_path = task.file_path.clone();
        let album_id = task.album_id;
        let _ = tauri::async_runtime::spawn_blocking(move || {
            let album_sync_lock = album_sync_lock(album_id);
            let _album_sync_guard = match album_sync_lock.lock() {
                Ok(guard) => guard,
                Err(_) => return,
            };
            if album_removal_pending(album_id) || Album::get_album_by_id(album_id).is_err() {
                return;
            }
            if !AFile::is_album_visible(task.file_id).unwrap_or(false) {
                return;
            }
            if let Err(error) = crate::t_startup::ensure_ai(&app_handle_for_embedding, None) {
                eprintln!("Failed to initialize AI for embedding: {}", error);
                return;
            }
            let ai_state: tauri::State<crate::t_ai::AiState> = app_handle_for_embedding.state();
            if let Err(e) = AFile::generate_embedding(&ai_state, task.file_id) {
                eprintln!("Failed to generate embedding for {}: {}", file_path, e);
            }
        })
        .await;
    });
}

fn dimensions_excluded(file_type: i64, width: u32, height: u32, threshold: i64) -> bool {
    matches!(threshold, 160 | 320 | 640) && matches!(file_type, 1 | 3)
        && width > 0 && height > 0 && width < threshold as u32 && height < threshold as u32
}

#[derive(Clone)]
struct CachedFilterDimensions {
    size: u64,
    modified: SystemTime,
    dimensions: Option<(u32, u32)>,
}

#[derive(Default)]
struct FilterDimensionsCache {
    entries: HashMap<Arc<str>, CachedFilterDimensions>,
    order: VecDeque<Arc<str>>,
    bytes: usize,
}

static FILTER_DIMENSIONS_CACHE: Lazy<Mutex<FilterDimensionsCache>> =
    Lazy::new(|| Mutex::new(FilterDimensionsCache::default()));

/// Reuse header reads between previews, pre-counting and indexing. File size and
/// modification time invalidate changed files; the FIFO cache has a 32 MiB budget.
fn filter_image_dimensions(path: &str, file_type: i64) -> Option<(u32, u32)> {
    let metadata = fs::metadata(path).ok()?;
    let modified = metadata.modified().ok()?;
    if let Ok(cache) = FILTER_DIMENSIONS_CACHE.lock() {
        if let Some(entry) = cache.entries.get(path) {
            if entry.size == metadata.len() && entry.modified == modified {
                return entry.dimensions;
            }
        }
    }
    // Never hold the cache mutex during disk reads or external metadata probes.
    let dimensions = if file_type == 3 {
        crate::t_image::get_raw_dimensions(path)
    } else {
        crate::t_image::get_image_dimensions(path)
    }.ok();
    if let Ok(mut cache) = FILTER_DIMENSIONS_CACHE.lock() {
        let key: Arc<str> = Arc::from(path);
        if !cache.entries.contains_key(path) {
            cache.bytes += path.len() + 192;
            cache.order.push_back(key.clone());
        }
        cache.entries.insert(key, CachedFilterDimensions { size: metadata.len(), modified, dimensions });
        while cache.bytes > 32 * 1024 * 1024 {
            let Some(oldest) = cache.order.pop_front() else { break; };
            cache.entries.remove(&oldest);
            cache.bytes = cache.bytes.saturating_sub(oldest.len() + 192);
        }
    }
    dimensions
}

/// Unknown dimensions fail open, and video is never excluded by image dimensions.
fn excluded_image_dimensions(path: &str, file_type: i64, threshold: i64) -> Option<(u32, u32)> {
    if !matches!(threshold, 160 | 320 | 640) || !matches!(file_type, 1 | 3) {
        return None;
    }
    let dimensions = filter_image_dimensions(path, file_type)?;
    let (width, height) = dimensions;
    dimensions_excluded(file_type, width, height, threshold).then_some(dimensions)
}

fn skip_excluded_image(album: &Album, path: &str, file_type: i64) -> Result<bool, String> {
    if let Some((width, height)) = excluded_image_dimensions(path, file_type, album.small_image_filter) {
        // Do not insert new excluded files. For previously indexed files, retain
        // ratings/tags and update dimensions so the query exclusion stays accurate.
        AFile::retain_excluded_image(album.id.ok_or("Album has no id")?, path, width, height)?;
        return Ok(true);
    }
    Ok(false)
}

/// Get folder, media, and scan candidate totals (including all sub-folders).
pub fn count_folder_files(path: &str) -> (u64, u64, u64, u64, u64, u64, u64) {
    count_folder_files_filtered(path, 7, &[], 0)
}

pub fn count_folder_files_filtered(path: &str, file_types: i64, excluded_folders: &[String], small_image_filter: i64) -> (u64, u64, u64, u64, u64, u64, u64) {
    count_folder_files_cancellable(path, file_types, excluded_folders, small_image_filter, || false).unwrap_or_default()
}

fn count_folder_files_cancellable(path: &str, file_types: i64, excluded_folders: &[String],
    small_image_filter: i64, is_cancelled: impl Fn() -> bool) -> Option<(u64, u64, u64, u64, u64, u64, u64)> {
    let mut folder_count = 0;
    let mut image_file_count = 0;
    let mut total_image_size = 0;
    let mut video_file_count = 0;
    let mut total_video_size = 0;
    let mut scan_file_count = 0;
    let mut total_scan_size = 0;

    // Use WalkDir to iterate over directory entries
    for entry in WalkDir::new(path)
        .into_iter()
        .filter_entry(|entry| is_visible_or_root(entry)
            && !Album::path_is_excluded(Path::new(path), excluded_folders, entry.path()))
        .filter_map(Result::ok)
    {
        if is_cancelled() { return None; }
        let entry_type = entry.file_type();

        if entry_type.is_dir() {
            folder_count += 1;
        } else if entry_type.is_file() {
            let kind = get_file_type(entry.path().to_str().unwrap_or(""));
            let bit = match kind { Some(1) => 1, Some(2) => 2, Some(3) => 4, _ => 0 };
            if bit != 0 && file_types & bit == 0 { continue; }
            if kind.is_some_and(|kind| excluded_image_dimensions(&entry.path().to_string_lossy(), kind, small_image_filter).is_some()) { continue; }
            let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
            if !is_ignored_scan_sidecar(entry.path()) {
                scan_file_count += 1;
                total_scan_size += size;
            }
            if let Some(file_ext_type) = get_file_type(entry.path().to_str().unwrap_or("")) {
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

    if is_cancelled() { return None; }
    Some((
        folder_count,
        image_file_count,
        total_image_size,
        video_file_count,
        total_video_size,
        scan_file_count,
        total_scan_size,
    ))
}

fn is_ignored_scan_sidecar(path: &Path) -> bool {
    matches!(
        path.extension()
            .and_then(|extension| extension.to_str())
            .map(|extension| extension.to_ascii_lowercase())
            .as_deref(),
        Some("xmp" | "aae")
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

    if contains_ext(t_common::NORMAL_IMGS) || contains_ext(t_common::FFMPEG_BACKED_IMGS) {
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

pub(crate) fn detect_label_from_header(header: &[u8], file_type: i64) -> Option<String> {
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
    // JPEG XL codestream
    if header.starts_with(&[0xFF, 0x0A]) {
        return Some("JXL".to_string());
    }
    // JPEG XL container
    if header.len() >= 12
        && &header[0..4] == [0x00, 0x00, 0x00, 0x0C]
        && &header[4..8] == b"JXL "
        && &header[8..12] == [0x0D, 0x0A, 0x87, 0x0A]
    {
        return Some("JXL".to_string());
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

/// Convert to pinyin and zero-pad ALL digit sequences for natural sort order.
/// Every group of digits is padded to 10 digits so "Page_2" < "Page_10".
pub fn natural_sort_key(s: &str) -> String {
    use std::fmt::Write;

    let pinyin: String = s
        .chars()
        .flat_map(|c| match c.to_pinyin() {
            Some(p) => p.plain().chars().collect::<Vec<_>>(),
            None => vec![c],
        })
        .collect();

    let mut result = String::with_capacity(pinyin.len() + 32);
    let mut digits = String::new();

    for c in pinyin.chars() {
        if c.is_ascii_digit() {
            digits.push(c);
        } else {
            if !digits.is_empty() {
                let _ = write!(result, "{:0>10}", digits);
                digits.clear();
            }
            result.push(c);
        }
    }
    if !digits.is_empty() {
        let _ = write!(result, "{:0>10}", digits);
    }

    result
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

    // QuickTime commonly stores a numeric UTC offset without a colon, such as
    // `2025-12-31T13:36:26+0100`.
    if let Ok(datetime) = DateTime::parse_from_str(date, "%Y-%m-%dT%H:%M:%S%z") {
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

/// Keep the recording wall clock, matching timezone-free photo EXIF dates.
pub fn metadata_wallclock_date(date: &str) -> Option<String> {
    DateTime::parse_from_rfc3339(date)
        .or_else(|_| DateTime::parse_from_str(date, "%Y-%m-%dT%H:%M:%S%z"))
        .ok()
        .map(|dt| dt.format("%Y:%m:%d %H:%M:%S").to_string())
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
    phase: String,
    current: u64,
    discovered: u64,
    processed: u64,
    search_ready: u64,
    total: u64,
    search_total: u64,
    current_size: u64,
    skipped: u64,
    skipped_size: u64,
    failed: u64,
    failed_size: u64,
    scan_total: u64,
    scan_total_size: u64,
}

#[derive(serde::Serialize, Clone)]
struct FinishedPayload {
    album_id: i64,
    phase: String,
    indexed: u64,
    processed: u64,
    search_ready: u64,
    total: u64,
    search_total: u64,
    skipped: u64,
    skipped_size: u64,
    failed: u64,
    failed_size: u64,
    scan_total: u64,
    scan_total_size: u64,
}

#[derive(serde::Serialize, Clone)]
struct ThumbnailReadyPayload {
    album_id: i64,
    file_ids: Vec<i64>,
    invalidate: bool,
}

pub fn file_accessible(path: &str) -> bool {
    // Opening a cloud-only file downloads it; its metadata is enough here.
    std::fs::metadata(path).is_ok_and(|m| m.is_file() && (is_cloud_only(&m) || std::fs::File::open(path).is_ok()))
}

pub fn album_accessible(album_id: i64) -> bool {
    !INACCESSIBLE_ALBUM_IDS.lock().unwrap().ids.contains(&album_id)
}

pub fn refresh_album_accessibility(album: &mut Album) {
    let generation = album_accessibility_generation();
    album.is_accessible = directory_accessible(&album.path);
    if let Some(album_id) = album.id {
        let mut inaccessible = INACCESSIBLE_ALBUM_IDS.lock().unwrap();
        if inaccessible.generation != generation { return; }
        if album.is_accessible {
            inaccessible.ids.remove(&album_id);
        } else {
            inaccessible.ids.insert(album_id);
        }
    }
}

pub fn apply_album_accessibility(album: &mut Album) {
    album.is_accessible = album
        .id
        .is_none_or(|album_id| !INACCESSIBLE_ALBUM_IDS.lock().unwrap().ids.contains(&album_id));
}

pub fn album_accessibility_generation() -> u64 {
    INACCESSIBLE_ALBUM_IDS.lock().unwrap().generation
}

pub fn refresh_all_album_accessibility(albums: &mut [Album]) {
    refresh_all_album_accessibility_if_current(albums, album_accessibility_generation());
}

pub fn refresh_all_album_accessibility_if_current(albums: &mut [Album], generation: u64) {
    let mut inaccessible = HashSet::new();
    for album in albums {
        album.is_accessible = directory_accessible(&album.path);
        if !album.is_accessible {
            if let Some(album_id) = album.id {
                inaccessible.insert(album_id);
            }
        }
    }
    INACCESSIBLE_ALBUM_IDS.lock().unwrap().replace_if_current(generation, inaccessible);
}

pub fn clear_album_accessibility() {
    INACCESSIBLE_ALBUM_IDS.lock().unwrap().clear();
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
    fs::create_dir_all(&app_dir).map_err(|e| {
        format!(
            "Failed to create AppData directory for index recovery: {}",
            e
        )
    })?;

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

#[derive(Clone)]
struct ThumbnailTask {
    file_id: i64,
    file_path: String,
    file_type: i64,
    orientation: i32,
    thumbnail_size: u32,
    prefer_embedded_raw_thumbnail: RawDisplayOptions,
    file_size: u64,
    duration: Option<u64>,
    is_heavy: bool,
    processed_already_ready: bool,
    force_regenerate: bool,
}

struct FileIndexOutcome {
    excluded: bool,
    task: Option<ThumbnailTask>,
    processed_immediately: bool,
    search_ready_immediately: bool,
}

#[derive(Clone)]
struct ProcessingBudget {
    normal_thumb: Arc<Semaphore>,
    heavy_thumb: Arc<Semaphore>,
    embedding: Arc<Semaphore>,
}

impl ProcessingBudget {
    fn new() -> Self {
        let logical_cores = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4);
        let total_budget = ((logical_cores as f64) * 0.7).floor().max(1.0) as usize;
        let heavy_budget = if logical_cores <= 8 { 1 } else { 2 }.min(total_budget);
        let normal_budget = total_budget.saturating_sub(heavy_budget).max(1);
        Self {
            normal_thumb: Arc::new(Semaphore::new(normal_budget)),
            heavy_thumb: Arc::new(Semaphore::new(heavy_budget)),
            embedding: Arc::new(Semaphore::new(1)),
        }
    }
}

#[derive(Clone, Default)]
struct ProgressSnapshot {
    discovered: u64,
    processed: u64,
    search_ready: u64,
    total: u64,
    search_total: u64,
    current_size: u64,
    skipped: u64,
    skipped_size: u64,
    failed: u64,
    failed_size: u64,
    scan_total: u64,
    scan_total_size: u64,
}

impl ProgressSnapshot {
    fn phase(&self) -> &'static str {
        if self.discovered < self.total {
            "discovering"
        } else if self.processed < self.total {
            "preparing_previews"
        } else if self.search_ready < self.search_total {
            "preparing_search"
        } else {
            "complete"
        }
    }

    fn to_payload(&self, album_id: i64) -> ProgressPayload {
        ProgressPayload {
            album_id,
            phase: self.phase().to_string(),
            current: self.processed,
            discovered: self.discovered,
            processed: self.processed,
            search_ready: self.search_ready,
            total: self.total,
            search_total: self.search_total,
            current_size: self.current_size,
            skipped: self.skipped,
            skipped_size: self.skipped_size,
            failed: self.failed,
            failed_size: self.failed_size,
            scan_total: self.scan_total,
            scan_total_size: self.scan_total_size,
        }
    }
}

struct ProgressTracker {
    album_id: i64,
    app_handle: tauri::AppHandle,
    flush_interval: Duration,
    last_emit_at: Option<Instant>,
    last_phase: String,
    snapshot: ProgressSnapshot,
}

impl ProgressTracker {
    fn new(
        app_handle: &tauri::AppHandle,
        album_id: i64,
        total: u64,
        search_total: u64,
        discovered: u64,
        scan_total: u64,
        scan_total_size: u64,
    ) -> Self {
        let snapshot = ProgressSnapshot {
            discovered,
            processed: 0,
            search_ready: 0,
            total,
            search_total,
            current_size: 0,
            skipped: 0,
            skipped_size: 0,
            failed: 0,
            failed_size: 0,
            scan_total,
            scan_total_size,
        };
        Self {
            album_id,
            app_handle: app_handle.clone(),
            flush_interval: Duration::from_millis(150),
            last_emit_at: None,
            last_phase: snapshot.phase().to_string(),
            snapshot,
        }
    }

    fn snapshot(&self) -> ProgressSnapshot {
        self.snapshot.clone()
    }

    fn modify<F>(&mut self, mutator: F)
    where
        F: FnOnce(&mut ProgressSnapshot),
    {
        mutator(&mut self.snapshot);
    }

    fn maybe_emit(&mut self) {
        let now = Instant::now();
        let phase = self.snapshot.phase().to_string();
        let should_emit = self
            .last_emit_at
            .map(|last| now.duration_since(last) >= self.flush_interval)
            .unwrap_or(true)
            || phase != self.last_phase;

        if should_emit {
            self.emit_now();
        }
    }

    fn emit_now(&mut self) {
        let payload = self.snapshot.to_payload(self.album_id);
        let _ = self.app_handle.emit("index_progress", payload);
        self.last_emit_at = Some(Instant::now());
        self.last_phase = self.snapshot.phase().to_string();
    }
}

fn with_progress_tracker<T, F>(tracker: &Arc<Mutex<ProgressTracker>>, update: F) -> T
where
    F: FnOnce(&mut ProgressTracker) -> T,
{
    let mut guard = tracker.lock().unwrap();
    update(&mut guard)
}

fn should_use_heavy_lane(
    file_type: i64,
    file_path: &str,
    file_size: u64,
    width: u32,
    height: u32,
) -> bool {
    if file_type == 2 || file_type == 3 {
        return true;
    }

    if crate::t_image::is_ffmpeg_backed_image_path(file_path) {
        return true;
    }

    if let Some(ext) = Path::new(file_path)
        .extension()
        .and_then(|ext| ext.to_str())
    {
        return matches!(
            ext.to_ascii_lowercase().as_str(),
            "heic" | "heif" | "tif" | "tiff" | "jxl"
        );
    }

    if file_size >= 50 * 1024 * 1024 {
        return true;
    }

    let pixel_count = (width as u64).saturating_mul(height as u64);
    pixel_count >= 40_000_000 || width >= 8000 || height >= 8000
}

fn index_single_file(
    album_path: &str,
    album_id: i64,
    path_str: &str,
    ftype: i64,
    thumbnail_size: u32,
    prefer_embedded_raw_thumbnail: RawDisplayOptions,
    last_scan_time: i64,
    small_image_filter: i64,
) -> Option<FileIndexOutcome> {
    let result = panic::catch_unwind(AssertUnwindSafe(|| {
        let parent_path = Path::new(path_str)
            .parent()
            .unwrap_or(Path::new(album_path))
            .to_string_lossy()
            .to_string();

        if let Ok(folder) = crate::t_sqlite::AFolder::add_to_db(album_id, &parent_path) {
            if let Some(folder_id) = folder.id {
                if let Ok((file, _)) =
                    crate::t_sqlite::AFile::add_to_db(folder_id, path_str, ftype, last_scan_time)
                {
                    if dimensions_excluded(ftype, file.width.unwrap_or(0), file.height.unwrap_or(0), small_image_filter) {
                        return Some(FileIndexOutcome { excluded: true, task: None,
                            processed_immediately: false, search_ready_immediately: false });
                    }
                    if let Some(file_id) = file.id {
                        let has_thumbnail = file.has_thumbnail.unwrap_or(false);
                        let needs_thumbnail_regeneration = has_thumbnail
                            && crate::t_sqlite::AThumb::needs_thumbnail_regeneration(
                                file_id,
                                thumbnail_size,
                                prefer_embedded_raw_thumbnail,
                            );
                        let thumbnail_ready = has_thumbnail && !needs_thumbnail_regeneration;
                        let has_embedding = file.has_embedding.unwrap_or(false);
                        let processed_immediately = thumbnail_ready;
                        let search_ready_immediately = match ftype {
                            1 | 3 => thumbnail_ready && has_embedding,
                            _ => false,
                        };
                        let fully_indexed = match ftype {
                            1 | 3 => search_ready_immediately,
                            2 => processed_immediately,
                            _ => false,
                        };

                        let task = if fully_indexed {
                            None
                        } else {
                            Some(ThumbnailTask {
                                file_id,
                                file_path: path_str.to_string(),
                                file_type: ftype,
                                orientation: file.e_orientation.unwrap_or(1) as i32,
                                thumbnail_size,
                                prefer_embedded_raw_thumbnail,
                                file_size: file.size.max(0) as u64,
                                duration: file.duration.map(|d| d as u64),
                                is_heavy: should_use_heavy_lane(
                                    ftype,
                                    path_str,
                                    file.size.max(0) as u64,
                                    file.width.unwrap_or(0),
                                    file.height.unwrap_or(0),
                                ),
                                processed_already_ready: thumbnail_ready,
                                force_regenerate: needs_thumbnail_regeneration,
                            })
                        };

                        return Some(FileIndexOutcome {
                            excluded: false,
                            task,
                            processed_immediately,
                            search_ready_immediately,
                        });
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
        None
    }));

    match result {
        Ok(task) => task,
        Err(_) => {
            eprintln!("Panic while indexing file, skipping: {}", path_str);
            None
        }
    }
}

async fn process_thumbnail_task(
    app_handle: tauri::AppHandle,
    task: ThumbnailTask,
    budget: ProcessingBudget,
    tracker: Arc<Mutex<ProgressTracker>>,
) -> Result<bool, String> {
    let thumb_semaphore = if task.is_heavy {
        budget.heavy_thumb.clone()
    } else {
        budget.normal_thumb.clone()
    };

    let _thumb_permit = thumb_semaphore
        .acquire()
        .await
        .map_err(|e| format!("Failed to acquire thumbnail permit: {}", e))?;

    let task_for_thumb = task.clone();
    let thumb_ok = tauri::async_runtime::spawn_blocking(move || {
        match crate::t_sqlite::AThumb::get_or_create_thumb(
            task_for_thumb.file_id,
            &task_for_thumb.file_path,
            task_for_thumb.file_type,
            task_for_thumb.orientation,
            task_for_thumb.thumbnail_size,
            task_for_thumb.prefer_embedded_raw_thumbnail,
            task_for_thumb.force_regenerate,
            task_for_thumb.duration,
            None,
        ) {
            Ok(Some(thumb)) if thumb.error_code == 0 => true,
            Ok(Some(_)) => false,
            Ok(None) => false,
            Err(e) => {
                eprintln!(
                    "Failed to generate thumb for {}: {}",
                    task_for_thumb.file_path, e
                );
                false
            }
        }
    })
    .await
    .map_err(|e| format!("Thumbnail task failed: {}", e))?;

    if !thumb_ok {
        // The file record was created successfully. A preview failure must not
        // be counted as a scan failure, otherwise it overlaps with Indexed.
        return Ok(false);
    }

    let _ = app_handle.emit(
        "thumbnail_ready",
        ThumbnailReadyPayload {
            album_id: with_progress_tracker(&tracker, |tracker| tracker.album_id),
            file_ids: vec![task.file_id],
            invalidate: task.force_regenerate,
        },
    );

    if !task.processed_already_ready {
        with_progress_tracker(&tracker, |tracker| {
            tracker.modify(|snapshot| {
                snapshot.processed += 1;
            });
            tracker.maybe_emit();
        });
    }

    if !matches!(task.file_type, 1 | 3) {
        return Ok(true);
    }

    let _embedding_permit = budget
        .embedding
        .acquire()
        .await
        .map_err(|e| format!("Failed to acquire embedding permit: {}", e))?;

    let app_handle_for_embedding = app_handle.clone();
    let file_id = task.file_id;
    let file_path = task.file_path.clone();
    let embedding_ok = tauri::async_runtime::spawn_blocking(move || {
        if let Err(error) = crate::t_startup::ensure_ai(&app_handle_for_embedding, None) {
            eprintln!("Failed to initialize AI for embedding: {}", error);
            return false;
        }
        let ai_state: State<crate::t_ai::AiState> = app_handle_for_embedding.state();
        match crate::t_sqlite::AFile::generate_embedding(&ai_state, file_id) {
            Ok(_) => true,
            Err(e) => {
                eprintln!("Failed to generate embedding for {}: {}", file_path, e);
                false
            }
        }
    })
    .await
    .map_err(|e| format!("Embedding task failed: {}", e))?;

    if embedding_ok {
        with_progress_tracker(&tracker, |tracker| {
            tracker.modify(|snapshot| {
                snapshot.search_ready += 1;
            });
            tracker.maybe_emit();
        });
        Ok(true)
    } else {
        // Embedding is supplementary; the file remains indexed and visible.
        Ok(false)
    }
}

pub async fn index_album_worker(
    app_handle: &tauri::AppHandle,
    cancellation_token: Arc<Mutex<HashMap<i64, bool>>>,
    album_id: i64,
    thumbnail_size: u32,
    prefer_embedded_raw_thumbnail: RawDisplayOptions,
    skip_file_path: Option<String>,
    group_raw_jpeg_pairs: bool,
) -> Result<(), String> {
    let _album_scan_guard = AlbumScanGuard::acquire(album_id)?;
    let scan_start = std::time::Instant::now();
    // Generate a unique scan time for this session (current timestamp)
    let current_scan_time = Utc::now().timestamp_millis();
    let processing_budget = ProcessingBudget::new();
    // 1. Get album info
    let album = Album::get_album_by_id(album_id).map_err(|e| e.to_string())?;
    let previous_indexed = album.indexed.unwrap_or(0).max(0) as u64;
    let previous_total = album.total.unwrap_or(0).max(0) as u64;
    if !directory_accessible(&album.path) {
        app_handle
            .emit(
                "index_finished",
                FinishedPayload {
                    album_id,
                    phase: "complete".to_string(),
                    indexed: previous_indexed,
                    processed: previous_indexed,
                    search_ready: previous_indexed,
                    total: previous_total,
                    search_total: previous_total,
                    skipped: 0,
                    skipped_size: 0,
                    failed: 1,
                    failed_size: 0,
                    scan_total: previous_total,
                    scan_total_size: 0,
                },
            )
            .map_err(|e| e.to_string())?;
        return Ok(());
    }

    // Never persist a partial pre-count when cancellation interrupts traversal.
    let counts = count_folder_files_cancellable(&album.path, album.file_types, &album.excluded_folders, album.small_image_filter,
        || cancellation_token.lock().map(|flags| flags.get(&album_id) == Some(&true)).unwrap_or(true));
    let Some((_folders, image_count, _image_size, video_count, _video_size, scan_total, scan_total_size)) = counts else {
        app_handle.emit("index_finished", FinishedPayload {
            album_id,
            phase: "discovering".to_string(),
            indexed: previous_indexed,
            processed: previous_indexed,
            search_ready: previous_indexed,
            total: previous_total,
            search_total: previous_total,
            skipped: album.skipped_count.unwrap_or(0),
            skipped_size: album.skipped_size.unwrap_or(0),
            failed: album.failed_count.unwrap_or(0),
            failed_size: album.failed_size.unwrap_or(0),
            scan_total: previous_total,
            scan_total_size: 0,
        }).map_err(|e| e.to_string())?;
        return Ok(());
    };
    let total_files = image_count + video_count;
    let search_total = image_count;

    // Resume only when totals match and previous indexed is a valid in-progress value.
    // This avoids breaking normal re-scan behavior after a completed run.
    let resume_from = if previous_total == total_files
        && previous_indexed > 0
        && previous_indexed < total_files
    {
        previous_indexed
    } else {
        0
    };

    // 3. Emit start progress
    let tracker = Arc::new(Mutex::new(ProgressTracker::new(
        app_handle,
        album_id,
        total_files,
        search_total,
        resume_from,
        scan_total,
        scan_total_size,
    )));
    with_progress_tracker(&tracker, |tracker| tracker.emit_now());

    // update progress to db
    let _ = Album::update_progress(album_id, 0, total_files);

    // 4. Traverse and index
    let mut is_cancelled = false;
    let mut traversal_failed = false;
    let mut traversed_count = 0u64;
    // WalkDir already visits every directory during indexing.  Record the
    // direct-parent relationship here so the sidebar never needs to rescan an
    // unchanged directory just to decide whether to show its expand arrow.
    let mut folder_subfolder_states: Vec<FolderSubfolderState> = Vec::new();
    let mut directory_state_stack: Vec<usize> = Vec::new();
    let mut thumbnail_join_set: JoinSet<Result<bool, String>> = JoinSet::new();
    for entry in WalkDir::new(&album.path)
        .into_iter()
        .filter_entry(|entry| is_visible_or_root(entry) && !album.excludes_path(entry.path()))
    {
        // Check cancellation
        if let Some(&true) = cancellation_token.lock().unwrap().get(&album_id) {
            println!("Indexing cancelled for album {}", album_id);
            is_cancelled = true;
            thumbnail_join_set.abort_all();
            break;
        }

        let entry = match entry {
            Ok(entry) => entry,
            Err(error) => {
                eprintln!("Album traversal failed for {}: {}", album.path, error);
                traversal_failed = true;
                thumbnail_join_set.abort_all();
                break;
            }
        };

        if entry.file_type().is_dir() {
            let directory_path = entry.path().to_string_lossy().to_string();
            let metadata = entry.metadata().ok();
            let modified_at = metadata
                .as_ref()
                .and_then(|metadata| systemtime_to_timestamp(metadata.modified().ok()));
            let created_at = metadata
                .as_ref()
                .and_then(|metadata| systemtime_to_timestamp(metadata.created().ok()));
            #[cfg(unix)]
            let inode = {
                use std::os::unix::fs::MetadataExt;
                metadata
                    .as_ref()
                    .map(|metadata| metadata.ino() as i64)
                    .filter(|inode| *inode != 0)
            };
            #[cfg(windows)]
            let inode = FileInfo::new(&directory_path)
                .ok()
                .map(|info| info.inode as i64)
                .filter(|inode| *inode != 0);
            AFolder::migrate_path_by_inode(album_id, &directory_path, inode)?;
            let depth = entry.depth();
            directory_state_stack.truncate(depth);
            if let Some(&parent_index) = directory_state_stack.last() {
                folder_subfolder_states[parent_index].has_subfolders = true;
            }
            directory_state_stack.push(folder_subfolder_states.len());
            folder_subfolder_states.push(FolderSubfolderState {
                path: directory_path,
                created_at,
                modified_at,
                inode,
                has_subfolders: false,
            });
        }

        if entry.file_type().is_file() {
            let path_str = entry.path().to_string_lossy().to_string();
            if let Some(ftype) = get_file_type(&path_str) {
                if !album.allows_file_type(ftype) || skip_excluded_image(&album, &path_str, ftype)? { continue; }
                // Resume mode: skip already-indexed prefix files.
                if traversed_count < resume_from {
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
                    let file_size = std::fs::metadata(&path_str).map(|m| m.len()).unwrap_or(0);
                    with_progress_tracker(&tracker, |tracker| {
                        tracker.modify(|snapshot| {
                            snapshot.discovered += 1;
                            snapshot.failed += 1;
                            snapshot.failed_size += file_size;
                            snapshot.current_size += file_size;
                        });
                        tracker.maybe_emit();
                    });
                    traversed_count += 1;
                    continue;
                }

                if let Some(outcome) = index_single_file(
                    &album.path,
                    album_id,
                    &path_str,
                    ftype,
                    thumbnail_size,
                    prefer_embedded_raw_thumbnail,
                    current_scan_time,
                    album.small_image_filter,
                ) {
                    let file_size = outcome
                        .task
                        .as_ref()
                        .map(|task| task.file_size)
                        .unwrap_or_else(|| {
                            std::fs::metadata(&path_str).map(|m| m.len()).unwrap_or(0)
                        });
                    if outcome.excluded {
                        // Some formats reveal dimensions only during metadata reading.
                        // Remove those candidates from all progress totals consistently.
                        with_progress_tracker(&tracker, |tracker| {
                            tracker.modify(|snapshot| {
                                snapshot.total = snapshot.total.saturating_sub(1);
                                snapshot.search_total = snapshot.search_total.saturating_sub(1);
                                snapshot.scan_total = snapshot.scan_total.saturating_sub(1);
                                snapshot.scan_total_size = snapshot.scan_total_size.saturating_sub(file_size);
                            });
                            tracker.maybe_emit();
                        });
                        traversed_count += 1;
                        continue;
                    }
                    if let Some(task) = outcome.task {
                        thumbnail_join_set.spawn(process_thumbnail_task(
                            app_handle.clone(),
                            task,
                            processing_budget.clone(),
                            tracker.clone(),
                        ));
                    }
                    with_progress_tracker(&tracker, |tracker| {
                        tracker.modify(|snapshot| {
                            snapshot.discovered += 1;
                            snapshot.current_size += file_size;
                            if outcome.processed_immediately {
                                snapshot.processed += 1;
                            }
                            if outcome.search_ready_immediately {
                                snapshot.search_ready += 1;
                            }
                        });
                        tracker.maybe_emit();
                    });
                    let processed_now =
                        with_progress_tracker(&tracker, |tracker| tracker.snapshot.processed);
                    let discovered_now =
                        with_progress_tracker(&tracker, |tracker| tracker.snapshot.discovered);
                    if discovered_now % 50 == 0 || processed_now % 50 == 0 {
                        let current_total = with_progress_tracker(&tracker, |tracker| tracker.snapshot.total);
                        let _ = Album::update_progress(album_id, processed_now, current_total);
                    }
                } else {
                    let file_size = std::fs::metadata(&path_str).map(|m| m.len()).unwrap_or(0);
                    with_progress_tracker(&tracker, |tracker| {
                        tracker.modify(|snapshot| {
                            snapshot.discovered += 1;
                            snapshot.failed += 1;
                            snapshot.failed_size += file_size;
                            snapshot.current_size += file_size;
                        });
                        tracker.maybe_emit();
                    });
                }

                traversed_count += 1;
            } else if !is_ignored_scan_sidecar(entry.path()) {
                let file_size = entry.metadata().map(|metadata| metadata.len()).unwrap_or(0);
                with_progress_tracker(&tracker, |tracker| {
                    tracker.modify(|snapshot| {
                        snapshot.skipped += 1;
                        snapshot.skipped_size += file_size;
                        snapshot.current_size += file_size;
                    });
                    tracker.maybe_emit();
                });
            }
        }
    }

    while let Some(result) = thumbnail_join_set.join_next().await {
        match result {
            Ok(Ok(_)) => {}
            Ok(Err(e)) => {
                eprintln!("Processing task failed: {}", e);
            }
            Err(e) => {
                if !e.is_cancelled() {
                    eprintln!("Processing task join failed: {}", e);
                }
            }
        }
    }

    with_progress_tracker(&tracker, |tracker| tracker.emit_now());
    let mut final_snapshot = with_progress_tracker(&tracker, |tracker| tracker.snapshot());
    let scan_failed = traversal_failed || !directory_accessible(&album.path);
    let scan_complete = !is_cancelled && !scan_failed;
    if scan_complete {
        let _ = Album::update_progress(album_id, final_snapshot.processed, final_snapshot.total);
    } else if scan_failed {
        final_snapshot.failed += 1;
        let _ = Album::update_progress(album_id, previous_indexed, previous_total);
    }

    clear_index_trace();

    // Delete files that are in DB but not in file system (Mark-and-Sweep)
    if scan_complete {
        if let Err(error) = AFolder::ensure_subfolder_states(album_id, &folder_subfolder_states) {
            eprintln!(
                "Failed to persist folder subfolder states for album {}: {}",
                album_id, error
            );
        }
        if let Err(error) = AFolder::delete_unseen_in_album(
            album_id,
            &folder_subfolder_states,
        ) {
            eprintln!("Failed to remove stale folders from album {}: {}", album_id, error);
        }

        println!("Cleaning up removed files from DB for album {}", album_id);
        let deleted_count = AFile::delete_unseen_in_album(album_id, current_scan_time).unwrap_or(0);
        if deleted_count > 0 {
            println!("Deleted {} stale records from DB.", deleted_count);
        }

        if let Err(error) = pair_live_photos_after_album_index(album_id) {
            eprintln!(
                "Failed to pair Live Photos after indexing album {}: {}",
                album_id, error
            );
        }
        if let Err(error) = reconcile_raw_jpeg_pairs_after_album_index(
            album_id,
            group_raw_jpeg_pairs,
        ) {
            eprintln!(
                "Failed to pair RAW+JPEG files after indexing album {}: {}",
                album_id, error
            );
        }
    }

    let (merged_count, merged_size) = if scan_complete {
        Album::merged_file_stats_in_album(album_id).unwrap_or((0, 0))
    } else {
        (0, 0)
    };

    if scan_complete {
        // Update last scan time only after a complete filesystem traversal.
        let _ = Album::update_last_scan_time(album_id, current_scan_time);
        let _ = Album::update_last_scan_results(
            album_id,
            final_snapshot.skipped,
            final_snapshot.skipped_size,
            final_snapshot.failed,
            final_snapshot.failed_size,
            merged_count,
            merged_size,
        );

        // Recount only after a complete traversal so an offline disk cannot
        // replace the existing album totals with a partial result.
        let _ = Album::recount_album(album_id);
    }

    // After a clean completed scan, align indexed with total so the next
    // scan does not incorrectly resume from a progress-milestone value
    // (update_progress writes indexed every 50 files during the scan,
    // and recount_album clamps without distinguishing complete vs partial).
    if scan_complete {
        if let Ok(album) = Album::get_album_by_id(album_id) {
            let total = album.total.unwrap_or(0) as u64;
            let _ = Album::update_progress(album_id, total, total);
        }
    }

    // 5. Set album cover if needed (must happen before index_finished event)
    // so frontend refresh gets the latest cover_file_id immediately.
    if scan_complete {
        let _ = Album::auto_set_cover(album_id);
    }

    // Summary log
    let elapsed = scan_start.elapsed().as_secs_f64();
    println!(
        "[scan] album={} folder='{}' files={} skipped={} merged={} failed={} time={:.1}s",
        album_id,
        album.path,
        final_snapshot.processed,
        final_snapshot.skipped,
        merged_count,
        final_snapshot.failed,
        elapsed
    );

    // 6. Emit finished
    app_handle
        .emit(
            "index_finished",
            FinishedPayload {
                album_id,
                phase: final_snapshot.phase().to_string(),
                indexed: if scan_failed {
                    previous_indexed
                } else {
                    final_snapshot.processed
                },
                processed: if scan_failed {
                    previous_indexed
                } else {
                    final_snapshot.processed
                },
                search_ready: final_snapshot.search_ready,
                total: if scan_failed {
                    previous_total
                } else {
                    final_snapshot.total
                },
                search_total: final_snapshot.search_total,
                skipped: final_snapshot.skipped,
                skipped_size: final_snapshot.skipped_size,
                failed: final_snapshot.failed,
                failed_size: final_snapshot.failed_size,
                scan_total: final_snapshot.scan_total,
                scan_total_size: final_snapshot.scan_total_size,
            },
        )
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[cfg(test)]
mod album_scan_filter_tests {
    use super::*;

    #[test]
    fn album_filters_probe_pixels_before_indexing_and_counting() {
        let root = std::env::temp_dir().join(format!("lap-pixel-exclusions-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&root).unwrap();
        image::RgbImage::new(159, 159).save(root.join("small.png")).unwrap();
        image::RgbImage::new(160, 100).save(root.join("boundary.png")).unwrap();
        fs::write(root.join("unknown.jpg"), b"unknown dimensions").unwrap();
        fs::write(root.join("video.mp4"), b"video").unwrap();
        let small = root.join("small.png").to_string_lossy().to_string();
        assert_eq!(excluded_image_dimensions(&small, 1, 160), Some((159,159)));
        assert_eq!(excluded_image_dimensions(&small, 2, 160), None);
        assert_eq!(excluded_image_dimensions(&small, 1, 0), None);
        assert_eq!(excluded_image_dimensions(&root.join("boundary.png").to_string_lossy(), 1, 160), None);
        assert_eq!(excluded_image_dimensions(&root.join("unknown.jpg").to_string_lossy(), 1, 160), None);
        let counts = count_folder_files_filtered(root.to_str().unwrap(), 7, &[], 160);
        assert_eq!((counts.1, counts.3, counts.5), (2,1,3));
        let counts = count_folder_files_filtered(root.to_str().unwrap(), 7, &[], 0);
        assert_eq!((counts.1, counts.3, counts.5), (3,1,4));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn pixel_preview_cache_invalidates_when_file_changes() {
        let root = std::env::temp_dir().join(format!("lap-pixel-cache-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&root).unwrap();
        let path = root.join("image.png");
        image::RgbImage::new(100, 100).save(&path).unwrap();
        assert_eq!(filter_image_dimensions(&path.to_string_lossy(), 1), Some((100, 100)));
        image::RgbImage::new(800, 600).save(&path).unwrap();
        assert_eq!(filter_image_dimensions(&path.to_string_lossy(), 1), Some((800, 600)));
        assert_eq!(excluded_image_dimensions(&path.to_string_lossy(), 1, 160), None);
        assert_eq!(count_folder_files_cancellable(root.to_str().unwrap(), 7, &[], 160, || true), None);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn cancelled_precount_does_not_return_partial_totals() {
        let root = std::env::temp_dir().join(format!("lap-cancel-count-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&root).unwrap();
        for i in 0..5 { fs::write(root.join(format!("{i}.jpg")), b"123").unwrap(); }
        let visited = std::cell::Cell::new(0);
        let counts = count_folder_files_cancellable(root.to_str().unwrap(), 7, &[], 0, || {
            visited.set(visited.get() + 1);
            visited.get() > 3
        });
        assert_eq!(counts, None);
        assert_eq!(count_folder_files(root.to_str().unwrap()).5, 5);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn album_filters_count_only_selected_media_and_prune_subtrees() {
        let root = std::env::temp_dir().join(format!("lap-album-filters-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(root.join("Exports/nested")).unwrap();
        fs::create_dir_all(root.join("Other/Exports")).unwrap();
        for name in ["image.jpg", "raw.cr2", "video.mp4", "Exports/nested/excluded.jpg", "Other/Exports/included.jpg"] {
            fs::write(root.join(name), b"123").unwrap();
        }
        let counts = count_folder_files_filtered(root.to_str().unwrap(), 1, &["Exports".into()], 0);
        assert_eq!((counts.1,counts.3,counts.5,counts.6),(2,0,2,6));
        let counts = count_folder_files_filtered(root.to_str().unwrap(), 4, &[], 0);
        assert_eq!((counts.1,counts.3,counts.5),(1,0,1));
        let counts = count_folder_files_filtered(root.to_str().unwrap(), 2, &[], 0);
        assert_eq!((counts.1,counts.3,counts.5),(0,1,1));
        let counts = count_folder_files(root.to_str().unwrap());
        assert_eq!((counts.1,counts.3,counts.5),(4,1,5));
        fs::remove_dir_all(root).unwrap();
    }
}

#[cfg(test)]
mod cloud_only_tests {
    use super::*;

    #[test]
    fn local_file_is_not_cloud_only() {
        let path = std::env::temp_dir().join(format!("lap-cloud-only-{}", uuid::Uuid::new_v4()));
        fs::write(&path, b"local").unwrap();
        assert!(!is_cloud_only(&fs::metadata(&path).unwrap()));
        fs::remove_file(path).unwrap();
    }

    /// Set LAP_CLOUD_ONLY_FILE to a OneDrive "online-only" file to run this check.
    #[test]
    fn cloud_only_sample_is_detected() {
        let Ok(path) = std::env::var("LAP_CLOUD_ONLY_FILE") else { return };
        assert!(is_cloud_only(&fs::metadata(&path).unwrap()));
        assert!(file_accessible(&path));
        assert!(is_cloud_only(&fs::metadata(&path).unwrap()), "the accessibility check downloaded the file");
    }
}

#[cfg(test)]
mod meta_date_tests {
    use super::*;

    #[test]
    fn video_wallclock_matches_photo_date() {
        let photo = "2025:12:31 13:36:26";
        for video in ["2025-12-31T13:36:26+01:00", "2025-12-31T13:36:26+0100", "2025-12-31T13:36:26Z"] {
            let date = metadata_wallclock_date(video).unwrap();
            assert_eq!(date, photo);
            assert_eq!(meta_date_to_timestamp(&date), meta_date_to_timestamp(photo));
        }
        assert_eq!(metadata_wallclock_date("invalid"), None);
    }
}

#[cfg(test)]
mod album_accessibility_cache_tests {
    use super::*;
    #[test]
    fn stale_probe_cannot_overwrite_new_library_cache() {
        let mut cache = AlbumAccessibilityCache::default();
        let old_generation = cache.generation;
        cache.clear();
        cache.replace_if_current(cache.generation, HashSet::from([2]));
        cache.replace_if_current(old_generation, HashSet::from([1]));
        assert_eq!(cache.ids, HashSet::from([2]));
    }
}

#[cfg(test)]
mod import_file_concurrency_tests {
    use super::import_file;
    use std::{fs, sync::{Arc, Barrier}, thread};

    #[test]
    fn concurrent_imports_preserve_every_file_and_existing_destination() {
        let root = std::env::temp_dir().join(format!("lap-import-test-{}", uuid::Uuid::new_v4()));
        let destination = root.join("destination");
        fs::create_dir_all(&destination).unwrap();
        fs::write(destination.join("photo.jpg"), b"existing").unwrap();
        let barrier = Arc::new(Barrier::new(16));
        let mut workers = Vec::new();
        for index in 0..16 {
            let source = root.join(format!("source-{index}"));
            fs::create_dir_all(&source).unwrap();
            let path = source.join("photo.jpg");
            let content = format!("photo-{index}").repeat(8192);
            fs::write(&path, &content).unwrap();
            let destination = destination.clone();
            let barrier = barrier.clone();
            workers.push(thread::spawn(move || {
                barrier.wait();
                let imported = import_file(path.to_str().unwrap(), destination.to_str().unwrap()).unwrap();
                assert_eq!(fs::read_to_string(&imported).unwrap(), content);
                imported
            }));
        }
        let mut paths = std::collections::HashSet::new();
        for worker in workers { assert!(paths.insert(worker.join().unwrap())); }
        assert_eq!(paths.len(), 16);
        assert_eq!(fs::read(destination.join("photo.jpg")).unwrap(), b"existing");
        assert_eq!(fs::read_dir(&destination).unwrap().count(), 17);
        fs::remove_dir_all(root).unwrap();
    }
}

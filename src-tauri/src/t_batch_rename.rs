//! Batch rename planning and rollback. Commands live in t_cmds.
use crate::{
    t_apple_sidecar::{build_apple_sidecar_rename_plan_cached, ImportAaeCache},
    t_sqlite::AFile,
    t_utils,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
    sync::Mutex,
};

static RENAME_LOCK: Mutex<()> = Mutex::new(());

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RenameOptions {
    pub name: String,
    pub start: u32,
    pub digits: usize,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RenameItem {
    pub id: i64,
    pub old_name: String,
    pub new_name: String,
    pub companions: usize,
}

#[derive(Serialize)]
pub struct RenamePreview {
    pub items: Vec<RenameItem>,
    pub errors: Vec<String>,
}

struct RenameMove {
    id: Option<i64>,
    old: PathBuf,
    new: PathBuf,
    old_name: String,
    new_name: String,
}

fn validate_name(name: &str) -> Result<(), String> {
    if name.trim().is_empty()
        || name.ends_with([' ', '.'])
        || name
            .chars()
            .any(|c| c.is_control() || "\\/:*?\"<>|".contains(c))
    {
        return Err(
            "Enter a valid file name without path separators or trailing spaces/dots.".into(),
        );
    }
    let first = name.split('.').next().unwrap_or("").to_ascii_uppercase();
    if ["CON", "PRN", "AUX", "NUL"].contains(&first.as_str())
        || (first.len() == 4
            && (first.starts_with("COM") || first.starts_with("LPT"))
            && matches!(first.as_bytes()[3], b'1'..=b'9'))
    {
        return Err(format!("Reserved file name: {name}"));
    }
    if name.encode_utf16().count() > 255 || name.len() > 255 {
        return Err("The generated file name is too long.".into());
    }
    Ok(())
}

fn numbered_name(old_name: &str, options: &RenameOptions, index: usize) -> Result<String, String> {
    if options.digits > 10 || options.start == 0 {
        return Err("Use a positive start number and 0–10 numbering digits.".into());
    }
    validate_name(&options.name)?;
    let number = options
        .start
        .checked_add(u32::try_from(index).map_err(|_| "Too many files")?)
        .ok_or("Sequence number overflow")?;
    let stem = format!("{}_{number:0width$}", options.name, width = options.digits);
    let extension = Path::new(old_name)
        .extension()
        .and_then(|value| value.to_str());
    let name = match extension {
        Some(ext) => format!("{stem}.{ext}"),
        None => stem,
    };
    validate_name(&name)?;
    Ok(name)
}

fn normalized_path(
    path: &Path,
    parents: &mut HashMap<PathBuf, PathBuf>,
) -> Result<PathBuf, String> {
    let parent = path.parent().ok_or("File has no parent directory")?;
    if !parents.contains_key(parent) {
        parents.insert(
            parent.to_path_buf(),
            parent.canonicalize().map_err(|error| error.to_string())?,
        );
    }
    Ok(parents[parent].join(path.file_name().ok_or("File has no name")?))
}

fn path_key(path: &Path) -> String {
    path.to_string_lossy().to_lowercase()
}

fn build_plan(
    ids: &[i64],
    options: &RenameOptions,
    group_raw_jpeg_pairs: bool,
) -> Result<(RenamePreview, Vec<RenameMove>), String> {
    if ids.is_empty() {
        return Err("Select at least one file.".into());
    }
    let unique: HashSet<_> = ids.iter().copied().collect();
    if unique.len() != ids.len() || ids.iter().any(|id| *id <= 0) {
        return Err("Invalid file selection.".into());
    }
    let mut files = HashMap::new();
    for chunk in ids.chunks(500) {
        for file in AFile::get_files_by_ids(chunk)? {
            if let Some(id) = file.id {
                files.insert(id, file);
            }
        }
    }
    if files.len() != ids.len() {
        return Err("Some selected files no longer exist in the library.".into());
    }
    let companion_ids: Vec<_> = files
        .values()
        .filter(|file| AFile::operation_includes_component(file.media_subtype.as_deref(), group_raw_jpeg_pairs))
        .filter_map(|file| file.live_photo_video_id)
        .filter(|id| *id > 0 && !files.contains_key(id))
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();
    for file in AFile::get_files_by_ids(&companion_ids)? {
        if let Some(id) = file.id {
            files.insert(id, file);
        }
    }
    let mut sidecar_cache = ImportAaeCache::default();
    plan_files(ids, options, &files, group_raw_jpeg_pairs, |id, path, name| {
        let file = &files[&id];
        let component = file.live_photo_video_id.and_then(|id| files.get(&id))
            .filter(|component| AFile::operation_component_matches(file, component, group_raw_jpeg_pairs));
        build_apple_sidecar_rename_plan_cached(path, name, component, &mut sidecar_cache)
    })
}

fn plan_files(
    ids: &[i64],
    options: &RenameOptions,
    files: &HashMap<i64, AFile>,
    group_raw_jpeg_pairs: bool,
    mut sidecar_plan: impl FnMut(
        i64,
        &str,
        &str,
    ) -> Result<Vec<crate::t_apple_sidecar::SidecarRenamePlan>, String>,
) -> Result<(RenamePreview, Vec<RenameMove>), String> {
    // A stale cross-directory Live Photo link cannot be safely registered by
    // this folder operation; fail before changing either file.
    for id in ids {
        let file = &files[id];
        if group_raw_jpeg_pairs && file.media_subtype.as_deref() == Some("raw_jpeg_pair") {
            let valid = file.live_photo_video_id.and_then(|id| files.get(&id))
                .is_some_and(|component| AFile::operation_component_matches(file, component, true));
            if !valid { return Err("RAW pair is stale; refresh the library before retrying".into()); }
        }
        if file.media_subtype.as_deref() == Some("live_photo") {
            if let Some(component) = file.live_photo_video_id.and_then(|id| files.get(&id)) {
                if component.folder_id != file.folder_id {
                    return Err("Live Photo companion is in a different folder; refresh the library before retrying".into());
                }
            }
        }
    }
    // A selected RAW/JPEG or Live Photo companion follows its primary, rather
    // than receiving a different sequence number and breaking the pair.
    let companions: HashSet<i64> = files
        .values()
        .filter_map(|file| file.live_photo_video_id.and_then(|id| files.get(&id)
            .filter(|component| AFile::operation_component_matches(file, component, group_raw_jpeg_pairs))
            .map(|_| id)))
        .collect();
    let roots: Vec<_> = ids.iter().filter(|id| !companions.contains(id)).collect();
    if roots.is_empty() {
        return Err("Invalid paired file selection.".into());
    }
    let mut preview = RenamePreview {
        items: Vec::new(),
        errors: Vec::new(),
    };
    let mut moves = Vec::new();
    let mut parents = HashMap::new();
    let mut xmp_cache = crate::t_xmp_files::Cache::default();
    for (index, id) in roots.iter().enumerate() {
        let file = &files[*id];
        let old_path = file
            .file_path
            .as_deref()
            .ok_or("File path is unavailable")?;
        let old = normalized_path(Path::new(old_path), &mut parents)?;
        let new_name = numbered_name(&file.name, options, index)?;
        let new = old.with_file_name(&new_name);
        let sidecars = sidecar_plan(**id, old_path, &new_name)?;
        preview.items.push(RenameItem {
            id: **id,
            old_name: file.name.clone(),
            new_name: new_name.clone(),
            companions: sidecars.len() + xmp_cache.count(
                std::iter::once((old.as_path(), new.as_path()))
                    .chain(sidecars.iter().map(|p| (p.old_path.as_path(), p.new_path.as_path()))),
            )?,
        });
        moves.push(RenameMove {
            id: Some(**id),
            old,
            new,
            old_name: file.name.clone(),
            new_name,
        });
        for sidecar in sidecars {
            validate_name(&sidecar.new_name)?;
            let old = normalized_path(&sidecar.old_path, &mut parents)?;
            let new = normalized_path(&sidecar.new_path, &mut parents)?;
            let old_name = old.file_name().unwrap().to_string_lossy().into_owned();
            moves.push(RenameMove {
                id: sidecar.file_id,
                old,
                new,
                old_name,
                new_name: sidecar.new_name,
            });
        }
    }
    preview.errors = check_plan(&moves);
    preview.errors.extend(xmp_cache.conflicts(moves.iter().map(|p| (p.old.as_path(), p.new.as_path())))?
        .into_iter().map(|path| format!("Destination XMP already exists: {}", path.display())));
    Ok((preview, moves))
}

fn check_plan(moves: &[RenameMove]) -> Vec<String> {
    let mut errors = Vec::new();
    let mut sources = HashSet::new();
    let mut targets = HashSet::new();
    for entry in moves {
        if !sources.insert(path_key(&entry.old)) {
            errors.push(format!(
                "A file belongs to more than one rename group: {}",
                entry.old.display()
            ));
        }
        if !targets.insert(path_key(&entry.new)) {
            errors.push(format!("Duplicate target name: {}", entry.new.display()));
        }
        if !fs::symlink_metadata(&entry.old).is_ok_and(|metadata| !metadata.is_dir()) {
            errors.push(format!(
                "Source file is unavailable: {}",
                entry.old.display()
            ));
        }
    }
    for entry in moves {
        if fs::symlink_metadata(&entry.new).is_ok() && !sources.contains(&path_key(&entry.new)) {
            errors.push(format!("A file already exists: {}", entry.new.display()));
        }
    }
    errors
}

pub fn preview(ids: &[i64], options: &RenameOptions, group_raw_jpeg_pairs: bool) -> Result<RenamePreview, String> {
    let _guard = RENAME_LOCK.lock().map_err(|error| error.to_string())?;
    build_plan(ids, options, group_raw_jpeg_pairs).map(|(preview, _)| preview)
}

/// Use platform rename-without-replacement, including for rollback. Files never
/// fall back to an overwriting rename when the filesystem lacks this operation.
// METADATA_HARNESS_RENAME_BEGIN
pub(crate) fn rename_exclusive(old: &Path, new: &Path) -> Result<(), String> {
    #[cfg(unix)]
    {
        use std::{ffi::CString, os::unix::ffi::OsStrExt};
        let old = CString::new(old.as_os_str().as_bytes()).map_err(|error| error.to_string())?;
        let new = CString::new(new.as_os_str().as_bytes()).map_err(|error| error.to_string())?;
        // SAFETY: both paths are valid null-terminated strings for the call.
        #[cfg(target_os = "macos")]
        let result = unsafe { libc::renamex_np(old.as_ptr(), new.as_ptr(), libc::RENAME_EXCL) };
        #[cfg(target_os = "linux")]
        let result = unsafe {
            libc::renameat2(
                libc::AT_FDCWD,
                old.as_ptr(),
                libc::AT_FDCWD,
                new.as_ptr(),
                libc::RENAME_NOREPLACE,
            )
        };
        #[cfg(not(any(target_os = "macos", target_os = "linux")))]
        return Err("Batch rename is unsupported on this platform.".into());
        #[cfg(any(target_os = "macos", target_os = "linux"))]
        if result == 0 {
            Ok(())
        } else {
            Err(std::io::Error::last_os_error().to_string())
        }
    }
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt;
        use windows::{
            core::PCWSTR,
            Win32::Storage::FileSystem::{MoveFileExW, MOVE_FILE_FLAGS},
        };
        let old: Vec<u16> = old.as_os_str().encode_wide().chain(Some(0)).collect();
        let new: Vec<u16> = new.as_os_str().encode_wide().chain(Some(0)).collect();
        // SAFETY: paths remain valid null-terminated strings throughout the call.
        // No MOVEFILE_REPLACE_EXISTING: an occupied destination must fail.
        unsafe {
            MoveFileExW(
                PCWSTR(old.as_ptr()),
                PCWSTR(new.as_ptr()),
                MOVE_FILE_FLAGS(0),
            )
        }
        .map_err(|error| error.to_string())
    }
    #[cfg(not(any(unix, windows)))]
    Err("Batch rename is unsupported on this platform.".into())
}

// METADATA_HARNESS_RENAME_END

fn apply_moves(
    moves: &[RenameMove],
    commit: impl FnOnce() -> Result<(), String>,
) -> Result<(), String> {
    let changes: Vec<_> = moves
        .iter()
        .filter(|entry| entry.old != entry.new)
        .collect();
    let temps: Vec<_> = changes
        .iter()
        .map(|entry| {
            entry
                .old
                .with_file_name(format!(".lap-rename-{}.tmp", uuid::Uuid::new_v4()))
        })
        .collect();
    let mut states = vec![0u8; changes.len()]; // original / temporary / final
    let result = (|| {
        for (index, entry) in changes.iter().enumerate() {
            rename_exclusive(&entry.old, &temps[index])?;
            states[index] = 1;
        }
        for (index, entry) in changes.iter().enumerate() {
            rename_exclusive(&temps[index], &entry.new)?;
            states[index] = 2;
        }
        commit()
    })();
    if let Err(error) = result {
        let mut rollback_errors = Vec::new();
        // Vacate all final names first, so swaps/cycles can also roll back.
        for (index, entry) in changes.iter().enumerate() {
            if states[index] == 2 {
                match rename_exclusive(&entry.new, &temps[index]) {
                    Ok(()) => states[index] = 1,
                    Err(error) => rollback_errors.push(format!("{}: {error}", entry.new.display())),
                }
            }
        }
        for (index, entry) in changes.iter().enumerate() {
            if states[index] == 1 {
                if let Err(error) = rename_exclusive(&temps[index], &entry.old) {
                    rollback_errors.push(format!("{}: {error}", temps[index].display()));
                }
            }
        }
        return Err(if rollback_errors.is_empty() {
            error
        } else {
            format!("{error}; could not restore: {}", rollback_errors.join("; "))
        });
    }
    Ok(())
}

pub fn execute(
    ids: &[i64],
    options: &RenameOptions,
    expected: &[RenameItem],
    group_raw_jpeg_pairs: bool,
) -> Result<RenamePreview, String> {
    let _guard = RENAME_LOCK.lock().map_err(|error| error.to_string())?;
    let (preview, moves) = build_plan(ids, options, group_raw_jpeg_pairs)?;
    if preview.items.len() != expected.len()
        || preview.items.iter().zip(expected).any(|(a, b)| {
            a.id != b.id
                || a.old_name != b.old_name
                || a.new_name != b.new_name
                || a.companions != b.companions
        })
    {
        return Err(
            "The file selection or names changed. Refresh the preview before renaming.".into(),
        );
    }
    if !preview.errors.is_empty() {
        return Err(preview.errors.join("\n"));
    }
    let updates: Vec<_> = moves
        .iter()
        .filter_map(|entry| {
            entry.id.map(|id| {
                (
                    id,
                    entry.old_name.clone(),
                    entry.new_name.clone(),
                    t_utils::natural_sort_key(&entry.new_name.to_lowercase()),
                )
            })
        })
        .collect();
    let xmp = crate::t_xmp_files::Guard::new(
        moves.iter().map(|p| (p.old.as_path(), p.new.as_path())), crate::t_xmp_files::Operation::Move,
    )?;
    apply_moves(&moves, || AFile::batch_rename_names(&updates))?;
    xmp.commit();
    Ok(preview)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Fixture(PathBuf);
    impl Fixture {
        fn new() -> Self {
            let path =
                std::env::temp_dir().join(format!("lap-rename-test-{}", uuid::Uuid::new_v4()));
            fs::create_dir(&path).unwrap();
            Self(path)
        }
        fn entry(&self, old: &str, new: &str) -> RenameMove {
            RenameMove {
                id: None,
                old: self.0.join(old),
                new: self.0.join(new),
                old_name: old.into(),
                new_name: new.into(),
            }
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn operation_pair_policy_keeps_live_photos_and_gates_raw() {
        for enabled in [false, true] {
            assert!(AFile::operation_includes_component(Some("live_photo"), enabled));
            assert_eq!(AFile::operation_includes_component(Some("raw_jpeg_pair"), enabled), enabled);
            assert!(!AFile::operation_includes_component(None, enabled));
        }
    }

    #[test]
    fn raw_selected_companions_are_independent_when_grouping_is_off() {
        let f = Fixture::new();
        let mut files = HashMap::new();
        for (id, name, subtype, companion) in [(1, "a.CR3", Some("raw_jpeg_pair"), Some(2)), (2, "a.JPG", None, None)] {
            fs::write(f.0.join(name), b"test").unwrap();
            let file: AFile = serde_json::from_value(serde_json::json!({
                "id": id, "name": name, "folder_id": 1, "size": 1, "file_type": if id == 1 { 3 } else { 1 },
                "file_path": f.0.join(name).to_str().unwrap(),
                "media_subtype": subtype, "live_photo_video_id": companion
            })).unwrap();
            files.insert(id, file);
        }
        let options = RenameOptions { name: "Trip".into(), start: 1, digits: 3 };
        let (ungrouped, _) = plan_files(&[1, 2], &options, &files, false, |_, _, _| Ok(vec![])).unwrap();
        let (grouped, _) = plan_files(&[1, 2], &options, &files, true, |_, _, _| Ok(vec![])).unwrap();
        assert_eq!(ungrouped.items.len(), 2);
        assert_eq!(grouped.items.len(), 1);
        files.get_mut(&2).unwrap().name = "other.JPG".into();
        assert!(!AFile::operation_component_matches(&files[&1], &files[&2], true));
        files.get_mut(&2).unwrap().name = "a.png".into();
        assert!(!AFile::operation_component_matches(&files[&1], &files[&2], true));
        files.get_mut(&2).unwrap().name = "A.jpg".into();
        assert!(AFile::operation_component_matches(&files[&1], &files[&2], true));
        files.get_mut(&2).unwrap().live_photo_video_id = Some(9);
        assert!(!AFile::operation_component_matches(&files[&1], &files[&2], true));
        files.get_mut(&2).unwrap().live_photo_video_id = None;
        files.get_mut(&2).unwrap().album_visible = false;
        assert!(!AFile::operation_component_matches(&files[&1], &files[&2], true));
        files.get_mut(&2).unwrap().album_visible = true;
        files.get_mut(&2).unwrap().media_subtype = Some("live_photo".into());
        assert!(!AFile::operation_component_matches(&files[&1], &files[&2], true));
        files.get_mut(&2).unwrap().media_subtype = None;
        files.get_mut(&2).unwrap().folder_id = 2;
        assert!(!AFile::operation_component_matches(&files[&1], &files[&2], true));
        assert!(plan_files(&[1, 2], &options, &files, true, |_, _, _| Ok(vec![])).is_err());
        assert!(plan_files(&[1], &options, &HashMap::from([(1, files.remove(&1).unwrap())]), true, |_, _, _| Ok(vec![])).is_err());
        let raw: AFile = serde_json::from_value(serde_json::json!({"id": 1, "name": "a.CR3", "folder_id": 1, "size": 1, "file_type": 3, "file_path": f.0.join("a.CR3").to_str().unwrap(), "media_subtype": "raw_jpeg_pair", "live_photo_video_id": 2})).unwrap();
        files.insert(1, raw);
        files.get_mut(&1).unwrap().media_subtype = Some("live_photo".into());
        assert!(!AFile::operation_component_matches(&files[&1], &files[&2], true));
        assert!(plan_files(&[1, 2], &options, &files, true, |_, _, _| Ok(vec![])).is_err());
        files.get_mut(&2).unwrap().folder_id = 1;
        assert!(AFile::operation_component_matches(&files[&1], &files[&2], true));
    }

    #[test]
    fn names_preserve_extensions_and_validate_sequences() {
        let options = RenameOptions {
            name: "Holiday".into(),
            start: 7,
            digits: 3,
        };
        assert_eq!(
            numbered_name("IMG.JPG", &options, 0).unwrap(),
            "Holiday_007.JPG"
        );
        assert_eq!(
            numbered_name("IMG.CR3", &options, 1).unwrap(),
            "Holiday_008.CR3"
        );
        let options = RenameOptions {
            digits: 0,
            start: 1,
            ..options
        };
        assert_eq!(numbered_name("photo", &options, 0).unwrap(), "Holiday_1");
        for name in ["", "../path", "CON", "a.", "a ", "a\n"] {
            assert!(validate_name(name).is_err(), "{name}");
        }
        assert!(numbered_name(
            "x.jpg",
            &RenameOptions {
                start: u32::MAX,
                ..options
            },
            1
        )
        .is_err());
    }

    #[test]
    fn preflight_rejects_existing_and_duplicate_targets_but_allows_swaps() {
        let f = Fixture::new();
        fs::write(f.0.join("a.jpg"), b"A").unwrap();
        fs::write(f.0.join("b.jpg"), b"B").unwrap();
        assert!(!check_plan(&[f.entry("a.jpg", "b.jpg")]).is_empty());
        assert!(
            !check_plan(&[f.entry("a.jpg", "new.jpg"), f.entry("b.jpg", "NEW.jpg")]).is_empty()
        );
        assert!(check_plan(&[f.entry("a.jpg", "b.jpg"), f.entry("b.jpg", "a.jpg")]).is_empty());
    }

    #[test]
    fn two_phase_rename_handles_cycles_and_database_failure_rolls_back() {
        let f = Fixture::new();
        fs::write(f.0.join("a.jpg"), b"A").unwrap();
        fs::write(f.0.join("b.jpg"), b"B").unwrap();
        let moves = vec![f.entry("a.jpg", "b.jpg"), f.entry("b.jpg", "a.jpg")];
        assert!(apply_moves(&moves, || Err("DB failure".into())).is_err());
        assert_eq!(fs::read(f.0.join("a.jpg")).unwrap(), b"A");
        assert_eq!(fs::read(f.0.join("b.jpg")).unwrap(), b"B");
        apply_moves(&moves, || Ok(())).unwrap();
        assert_eq!(fs::read(f.0.join("a.jpg")).unwrap(), b"B");
        assert_eq!(fs::read(f.0.join("b.jpg")).unwrap(), b"A");
        assert_eq!(fs::read_dir(&f.0).unwrap().count(), 2);
    }

    #[test]
    fn a_late_collision_never_overwrites_and_restores_sources() {
        let f = Fixture::new();
        fs::write(f.0.join("a.jpg"), b"A").unwrap();
        fs::write(f.0.join("b.jpg"), b"B").unwrap();
        fs::write(f.0.join("taken.jpg"), b"existing").unwrap();
        assert!(apply_moves(
            &[f.entry("a.jpg", "new.jpg"), f.entry("b.jpg", "taken.jpg")],
            || Ok(())
        )
        .is_err());
        assert_eq!(fs::read(f.0.join("taken.jpg")).unwrap(), b"existing");
        assert_eq!(fs::read(f.0.join("a.jpg")).unwrap(), b"A");
        assert_eq!(fs::read(f.0.join("b.jpg")).unwrap(), b"B");
        assert!(!f.0.join("new.jpg").exists());
    }

    #[test]
    fn paired_files_follow_one_sequence_and_database_updates_are_atomic() {
        use rusqlite::Connection;
        let f = Fixture::new();
        for name in [
            "first.CR3",
            "first.JPG",
            "second.HEIC",
            "second.MOV",
            "second.AAE",
        ] {
            fs::write(f.0.join(name), name.as_bytes()).unwrap();
        }
        let mut files = HashMap::new();
        for (id, name, companion) in [
            (1, "first.CR3", Some(2)),
            (2, "first.JPG", None),
            (3, "second.HEIC", Some(4)),
            (4, "second.MOV", None),
        ] {
            let file: AFile = serde_json::from_value(serde_json::json!({
                "id": id, "name": name, "folder_id": 1, "size": 1, "file_type": if id == 1 { 3 } else if id == 4 { 2 } else { 1 },
                "file_path": f.0.join(name).to_str().unwrap(), "live_photo_video_id": companion,
                "media_subtype": if id == 1 { Some("raw_jpeg_pair") } else if id == 3 { Some("live_photo") } else { None }
            }))
            .unwrap();
            files.insert(id, file);
        }
        let options = RenameOptions {
            name: "Trip".into(),
            start: 7,
            digits: 3,
        };
        let mut sidecar_cache = ImportAaeCache::default();
        let (preview, moves) = plan_files(&[3, 4, 1, 2], &options, &files, true, |id, path, name| {
            let component = files[&id].live_photo_video_id.and_then(|id| files.get(&id));
            build_apple_sidecar_rename_plan_cached(path, name, component, &mut sidecar_cache)
        })
        .unwrap();
        assert!(preview.errors.is_empty());
        assert_eq!(
            preview
                .items
                .iter()
                .map(|item| item.new_name.as_str())
                .collect::<Vec<_>>(),
            ["Trip_007.HEIC", "Trip_008.CR3"]
        );
        let mut conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("CREATE TABLE afiles(id INTEGER PRIMARY KEY, name TEXT, name_pinyin TEXT, rating INTEGER, folder_id INTEGER DEFAULT 1);
            CREATE UNIQUE INDEX uidx_afiles_folder_id_name ON afiles(folder_id, name);
            INSERT INTO afiles(id,name,name_pinyin,rating) VALUES(1,'first.CR3',NULL,5),(2,'first.JPG',NULL,4),(3,'second.HEIC',NULL,3),(4,'second.MOV',NULL,2);").unwrap();
        let updates: Vec<_> = moves
            .iter()
            .filter_map(|entry| {
                entry.id.map(|id| {
                    (
                        id,
                        entry.old_name.clone(),
                        entry.new_name.clone(),
                        entry.new_name.to_lowercase(),
                    )
                })
            })
            .collect();
        let mut stale = updates.clone();
        stale.last_mut().unwrap().1 = "stale.JPG".into();
        assert!(apply_moves(&moves, || AFile::batch_rename_names_with_conn(
            &mut conn, &stale
        ))
        .is_err());
        for entry in &moves {
            assert!(entry.old.exists());
            assert!(!entry.new.exists());
        }
        assert_eq!(
            conn.query_row("SELECT name FROM afiles WHERE id=3", [], |row| row
                .get::<_, String>(0))
                .unwrap(),
            "second.HEIC"
        );
        apply_moves(&moves, || {
            AFile::batch_rename_names_with_conn(&mut conn, &updates)
        })
        .unwrap();
        for entry in &moves {
            assert!(entry.new.exists());
            assert!(!entry.old.exists());
        }
        assert_eq!(
            conn.query_row("SELECT name FROM afiles WHERE id=2", [], |row| row
                .get::<_, String>(0))
                .unwrap(),
            "Trip_008.JPG"
        );
        assert_eq!(
            conn.query_row("SELECT SUM(rating) FROM afiles", [], |row| row
                .get::<_, i64>(0))
                .unwrap(),
            14
        );
        assert_eq!(fs::read_dir(&f.0).unwrap().count(), 5);
    }

    #[test]
    fn preflight_rejects_directories_and_missing_sources() {
        let f = Fixture::new();
        fs::create_dir(f.0.join("folder.jpg")).unwrap();
        assert!(!check_plan(&[f.entry("folder.jpg", "new.jpg")]).is_empty());
        assert!(!check_plan(&[f.entry("missing.jpg", "new.jpg")]).is_empty());
    }
}

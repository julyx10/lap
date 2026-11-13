//! XMP ownership and transactional file operations. No XML parsing or catalog writes.
use std::{
    collections::{HashMap, HashSet},
    ffi::OsString,
    fs, io,
    path::{Path, PathBuf},
};

// Explicit UI batches own their indexes; commands in one batch are serialized so
// shared RAW/JPEG sidecar ownership is updated before the next transfer plans.
type TransferCache = std::sync::Arc<std::sync::Mutex<Cache>>;
struct TransferSession {
    cache: TransferCache,
    touched: std::time::Instant,
}
static TRANSFERS: std::sync::LazyLock<std::sync::Mutex<HashMap<u64, TransferSession>>> =
    std::sync::LazyLock::new(|| std::sync::Mutex::new(HashMap::new()));
static NEXT_TRANSFER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

pub(crate) fn begin_transfer_batch() -> Result<u64, String> {
    let mut batches = TRANSFERS.lock().map_err(|e| e.to_string())?;
    create_transfer_batch(&mut batches)
}
fn create_transfer_batch(batches: &mut HashMap<u64, TransferSession>) -> Result<u64, String> {
    batches.retain(|_, session| session.touched.elapsed().as_secs() < 3600);
    if batches.len() >= 32 {
        return Err("Too many active file transfer batches".into());
    }
    let id = NEXT_TRANSFER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    batches.insert(
        id,
        TransferSession {
            cache: Default::default(),
            touched: std::time::Instant::now(),
        },
    );
    Ok(id)
}
pub(crate) fn end_transfer_batch(id: u64) {
    if let Ok(mut batches) = TRANSFERS.lock() {
        batches.remove(&id);
    }
}
pub(crate) fn with_transfer_cache<T>(
    id: Option<u64>,
    action: impl FnOnce(&mut Cache) -> Result<T, String>,
) -> Result<T, String> {
    let Some(id) = id else {
        return action(&mut Cache::default());
    };
    let cache = {
        let mut batches = TRANSFERS.lock().map_err(|e| e.to_string())?;
        let session = batches
            .get_mut(&id)
            .ok_or("File transfer batch has ended")?;
        session.touched = std::time::Instant::now();
        session.cache.clone()
    };
    let mut cache = cache.lock().map_err(|e| e.to_string())?;
    let result = action(&mut cache);
    if result.is_err() {
        cache.directories.clear();
    }
    result
}

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Operation {
    Move,
    Copy,
}

#[cfg(test)]
thread_local! {
    static DIRECTORY_READS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// Directory metadata detects entry changes without enumerating its contents.
/// Unix ctime also catches changes when a tool restores the directory mtime.
#[derive(PartialEq, Eq)]
struct DirectoryStamp {
    modified: Option<std::time::SystemTime>,
    created: Option<std::time::SystemTime>,
    #[cfg(unix)]
    identity_and_change: (u64, u64, i64, i64),
}
fn directory_stamp(path: &Path) -> Option<DirectoryStamp> {
    let metadata = fs::metadata(path).ok()?;
    if !metadata.is_dir() {
        return None;
    }
    Some(DirectoryStamp {
        modified: Some(metadata.modified().ok()?),
        created: metadata.created().ok(),
        #[cfg(unix)]
        identity_and_change: {
            use std::os::unix::fs::MetadataExt;
            (
                metadata.dev(),
                metadata.ino(),
                metadata.ctime(),
                metadata.ctime_nsec(),
            )
        },
    })
}

#[derive(Default)]
struct Directory {
    stamp: Option<DirectoryStamp>,
    sidecars: HashMap<OsString, Vec<PathBuf>>,
    owners: HashMap<OsString, Vec<PathBuf>>,
}
impl Directory {
    fn read(parent: &Path) -> Result<Self, String> {
        #[cfg(test)]
        DIRECTORY_READS.with(|reads| reads.set(reads.get() + 1));
        // Capture before enumeration so a concurrent entry change is detected
        // by the next lookup rather than stamped as already indexed.
        let mut index = Self {
            stamp: directory_stamp(parent),
            ..Default::default()
        };
        let entries = match fs::read_dir(parent) {
            Ok(entries) => entries,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(index),
            Err(e) => {
                return Err(format!(
                    "Cannot inspect XMP directory '{}': {e}",
                    parent.display()
                ));
            }
        };
        for entry in entries {
            let entry = entry.map_err(|e| e.to_string())?;
            let path = entry.path();
            let kind = entry.file_type().map_err(|e| e.to_string())?;
            if kind.is_dir() {
                continue;
            }
            if path
                .extension()
                .is_some_and(|e| e.eq_ignore_ascii_case("xmp"))
            {
                if let Some(base) = path.file_stem() {
                    index.sidecars.entry(base.into()).or_default().push(path);
                }
            } else if !path
                .extension()
                .is_some_and(|e| e.eq_ignore_ascii_case("aae"))
            {
                // Unknown formats can also own a sidecar. Prefer retaining metadata.
                for base in [path.file_name(), path.file_stem()].into_iter().flatten() {
                    let owners = index.owners.entry(base.into()).or_default();
                    if !owners.contains(&path) {
                        owners.push(path.clone());
                    }
                }
            }
        }
        for paths in index.sidecars.values_mut() {
            paths.sort();
        }
        Ok(index)
    }
    fn associated(&self, media: &Path) -> Vec<PathBuf> {
        let mut paths = Vec::new();
        for base in [media.file_name(), media.file_stem()].into_iter().flatten() {
            if let Some(found) = self.sidecars.get(base) {
                for path in found {
                    if !paths.contains(path) {
                        paths.push(path.clone());
                    }
                }
            }
        }
        paths
    }
}

/// Reused only for the lifetime of an explicit operation or transfer batch.
#[derive(Default)]
pub(crate) struct Cache {
    directories: HashMap<PathBuf, Directory>,
}
impl Cache {
    pub(crate) fn ensure_directory(&mut self, folder: &Path) -> Result<(), String> {
        let stamp = directory_stamp(folder);
        if self
            .directories
            .get(folder)
            .is_none_or(|index| stamp.is_none() || index.stamp != stamp)
        {
            self.directories
                .insert(folder.to_path_buf(), Directory::read(folder)?);
        }
        Ok(())
    }
    /// Match the per-file copy policy: an independent full-name XMP excludes
    /// a stem still shared with another original, which stays at the source.
    pub(crate) fn import_sidecars(&mut self, media: &Path) -> Result<Vec<PathBuf>, String> {
        let folder = parent(media);
        self.ensure_directory(folder)?;
        let directory = &self.directories[folder];
        let own_full = media
            .file_name()
            .is_some_and(|name| directory.sidecars.contains_key(name));
        Ok(directory
            .associated(media)
            .into_iter()
            .filter(|sidecar| {
                let shared = sidecar
                    .file_stem()
                    .and_then(|base| directory.owners.get(base))
                    .is_some_and(|owners| owners.iter().any(|owner| key(owner) != key(media)));
                sidecar.file_stem() == media.file_name() || !own_full || !shared
            })
            .collect())
    }
    pub(crate) fn record(&mut self, media: &Path, sidecars: &[PathBuf]) {
        if let Some(directory) = self.directories.get_mut(parent(media)) {
            for base in [media.file_name(), media.file_stem()].into_iter().flatten() {
                let owners = directory.owners.entry(base.into()).or_default();
                if !owners.contains(&media.to_path_buf()) {
                    owners.push(media.to_path_buf());
                }
            }
            for path in sidecars {
                if let Some(base) = path.file_stem() {
                    let paths = directory.sidecars.entry(base.into()).or_default();
                    if !paths.contains(path) {
                        paths.push(path.clone());
                        paths.sort();
                    }
                }
            }
            directory.stamp = directory_stamp(parent(media));
        }
    }
    fn update(&mut self, paths: &[PathBuf]) {
        for path in paths {
            let Some(directory) = self.directories.get_mut(parent(path)) else {
                continue;
            };
            if path
                .extension()
                .is_some_and(|e| e.eq_ignore_ascii_case("xmp"))
            {
                if let Some(base) = path.file_stem() {
                    let sidecars = directory.sidecars.entry(base.into()).or_default();
                    sidecars.retain(|old| old != path);
                    if fs::symlink_metadata(path).is_ok() {
                        sidecars.push(path.clone());
                        sidecars.sort();
                    }
                    if sidecars.is_empty() {
                        directory.sidecars.remove(base);
                    }
                }
            } else if !path
                .extension()
                .is_some_and(|e| e.eq_ignore_ascii_case("aae"))
            {
                for base in [path.file_name(), path.file_stem()].into_iter().flatten() {
                    let owners = directory.owners.entry(base.into()).or_default();
                    owners.retain(|old| old != path);
                    if fs::symlink_metadata(path).is_ok() {
                        owners.push(path.clone());
                    }
                    if owners.is_empty() {
                        directory.owners.remove(base);
                    }
                }
            }
        }
        // Only known successful mutations are acknowledged here. The next plan
        // stats each relevant directory again and rebuilds it on outside changes.
        let folders: HashSet<_> = paths.iter().map(|path| parent(path)).collect();
        for folder in folders {
            if let Some(directory) = self.directories.get_mut(folder) {
                directory.stamp = directory_stamp(folder);
            }
        }
    }
    pub(crate) fn count<'a>(
        &mut self,
        mappings: impl IntoIterator<Item = (&'a Path, &'a Path)>,
    ) -> Result<usize, String> {
        Ok(plan(mappings, self)?.plans.len())
    }
    pub(crate) fn conflicts<'a>(
        &mut self,
        mappings: impl IntoIterator<Item = (&'a Path, &'a Path)>,
    ) -> Result<Vec<PathBuf>, String> {
        Ok(plan(mappings, self)?.conflicts)
    }
}

#[derive(Clone)]
struct Plan {
    old: PathBuf,
    new: PathBuf,
    copy_only: bool,
}
struct Planning {
    plans: Vec<Plan>,
    conflicts: Vec<PathBuf>,
}
fn parent(path: &Path) -> &Path {
    path.parent().unwrap_or(Path::new("."))
}
fn key(path: &Path) -> PathBuf {
    let path = parent(path)
        .canonicalize()
        .unwrap_or_else(|_| parent(path).to_path_buf())
        .join(path.file_name().unwrap_or_default());
    #[cfg(any(windows, target_os = "macos"))]
    {
        PathBuf::from(path.to_string_lossy().to_lowercase())
    }
    #[cfg(not(any(windows, target_os = "macos")))]
    {
        path
    }
}
// Only media paths participate; non-media mappings (XMP/AAE) are filtered out.
fn plan<'a>(
    mappings: impl IntoIterator<Item = (&'a Path, &'a Path)>,
    cache: &mut Cache,
) -> Result<Planning, String> {
    let mappings: Vec<_> = mappings
        .into_iter()
        .filter(|(old, _)| crate::t_utils::get_file_type(&old.to_string_lossy()).is_some())
        .map(|(a, b)| (a.to_path_buf(), b.to_path_buf()))
        .collect();
    for (old, new) in &mappings {
        for folder in [parent(old), parent(new)] {
            cache.ensure_directory(folder)?;
        }
    }
    let directories = &mut cache.directories;
    let destinations: HashMap<_, _> = mappings.iter().map(|(a, b)| (key(a), b)).collect();
    let target_media: HashSet<_> = mappings.iter().map(|(_, b)| key(b)).collect();
    let mut plans = Vec::<Plan>::new();
    let mut seen_plans = HashSet::new();
    for (old, new) in &mappings {
        if old == new {
            continue;
        }
        let directory = &directories[parent(old)];
        let own_full = old
            .file_name()
            .and_then(|n| directory.sidecars.get(n))
            .is_some();
        for sidecar in directory.associated(old) {
            let metadata = fs::symlink_metadata(&sidecar).map_err(|e| e.to_string())?;
            if !metadata.is_file() {
                return Err(format!(
                    "XMP sidecar is not a regular file: {}",
                    sidecar.display()
                ));
            }
            let full = sidecar.file_stem() == old.file_name();
            let base = sidecar.file_stem().unwrap();
            let owners = directory.owners.get(base).cloned().unwrap_or_default();
            let shared = owners.iter().any(|owner| {
                destinations.get(&key(owner)).is_none_or(|target| {
                    parent(target) != parent(new) || target.file_stem() != new.file_stem()
                })
            });
            if !full && own_full && shared {
                continue;
            }
            let target_directory = &directories[parent(new)];
            let target_shared = new
                .file_stem()
                .and_then(|stem| target_directory.owners.get(stem))
                .is_some_and(|owners| {
                    owners
                        .iter()
                        .any(|owner| !target_media.contains(&key(owner)))
                });
            let mut name = if full || shared || target_shared {
                new.file_name()
            } else {
                new.file_stem()
            }
            .ok_or("File has no name")?
            .to_os_string();
            name.push(".");
            name.push(sidecar.extension().unwrap());
            let target = parent(new).join(name);
            if sidecar == target {
                continue;
            }
            if seen_plans.insert((sidecar.clone(), target.clone())) {
                plans.push(Plan {
                    old: sidecar,
                    new: target,
                    copy_only: shared,
                });
            }
        }
    }
    let mut counts = HashMap::<PathBuf, usize>::new();
    for p in &plans {
        *counts.entry(key(&p.old)).or_default() += 1;
    }
    for p in &mut plans {
        if counts[&key(&p.old)] > 1 {
            p.copy_only = true;
        }
    }
    let movable: HashSet<_> = plans
        .iter()
        .filter(|p| !p.copy_only)
        .map(|p| key(&p.old))
        .collect();
    let mut targets = HashSet::new();
    let mut conflicts = Vec::new();
    for p in &plans {
        if !targets.insert(key(&p.new)) {
            return Err(format!("Several XMP sidecars map to '{}'", p.new.display()));
        }
        if fs::symlink_metadata(&p.new).is_ok() && !movable.contains(&key(&p.new)) {
            conflicts.push(p.new.clone());
        }
    }
    let target_bases: HashSet<_> = plans
        .iter()
        .map(|p| (parent(&p.new), p.new.file_stem()))
        .collect();
    for (old, new) in &mappings {
        if old == new {
            continue;
        }
        let full_target = target_bases.contains(&(parent(new), new.file_name()));
        for existing in directories[parent(new)].associated(new) {
            if full_target && existing.file_stem() != new.file_name() {
                continue;
            }
            if !movable.contains(&key(&existing)) {
                conflicts.push(existing);
            }
        }
    }
    conflicts.sort();
    conflicts.dedup();
    Ok(Planning { plans, conflicts })
}

fn copy_new(old: &Path, new: &Path) -> Result<(), String> {
    let mut input = crate::t_utils::open_local_content(old).map_err(|e| e.to_string())?;
    let info = input.metadata().map_err(|e| e.to_string())?;
    let mut output = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(new)
        .map_err(|e| format!("Cannot create XMP '{}': {e}", new.display()))?;
    let result = io::copy(&mut input, &mut output)
        .and_then(|_| output.set_permissions(info.permissions()))
        .and_then(|_| {
            info.modified()
                .and_then(|time| output.set_times(fs::FileTimes::new().set_modified(time)))
        });
    if let Err(error) = result {
        drop(output);
        let _ = fs::remove_file(new);
        return Err(error.to_string());
    }
    Ok(())
}
fn move_new(old: &Path, new: &Path) -> Result<(), String> {
    if crate::t_batch_rename::rename_exclusive(old, new).is_ok() {
        return Ok(());
    }
    // Cross-device move: reserve the destination exclusively before removing the source.
    copy_new(old, new)?;
    if let Err(error) = fs::remove_file(old) {
        let _ = fs::remove_file(new);
        return Err(error.to_string());
    }
    Ok(())
}
struct Entry {
    plan: Plan,
    temp: PathBuf,
    copied: bool,
    state: u8,
    fingerprint: (u64, Option<std::time::SystemTime>),
}
pub(crate) struct Guard {
    entries: Vec<Entry>,
    committed: bool,
    cleanup_sources: bool,
    affected: Vec<PathBuf>,
}
impl Guard {
    pub(crate) fn new<'a>(
        mappings: impl IntoIterator<Item = (&'a Path, &'a Path)>,
        operation: Operation,
    ) -> Result<Self, String> {
        Self::with_cache(mappings, operation, &mut Cache::default())
    }
    pub(crate) fn paths(&self) -> Vec<PathBuf> {
        self.entries
            .iter()
            .map(|entry| entry.plan.new.clone())
            .collect()
    }
    pub(crate) fn with_cache<'a>(
        mappings: impl IntoIterator<Item = (&'a Path, &'a Path)>,
        operation: Operation,
        cache: &mut Cache,
    ) -> Result<Self, String> {
        let mappings: Vec<_> = mappings.into_iter().collect();
        let planning = plan(mappings.iter().copied(), cache)?;
        let mut affected: Vec<_> = mappings
            .iter()
            .flat_map(|(old, new)| [old.to_path_buf(), new.to_path_buf()])
            .collect();
        affected.extend(
            planning
                .plans
                .iter()
                .flat_map(|entry| [entry.old.clone(), entry.new.clone()]),
        );
        affected.sort();
        affected.dedup();
        if !planning.conflicts.is_empty() {
            return Err(format!(
                "Destination XMP already exists: {}",
                planning
                    .conflicts
                    .iter()
                    .map(|p| p.display().to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        if operation == Operation::Copy
            && planning
                .plans
                .iter()
                .any(|p| fs::symlink_metadata(&p.new).is_ok())
        {
            return Err("Destination XMP already exists".into());
        }
        let mut guard = Self {
            entries: Vec::new(),
            committed: false,
            cleanup_sources: operation == Operation::Move,
            affected,
        };
        for plan in planning.plans {
            let info = fs::symlink_metadata(&plan.old).map_err(|e| e.to_string())?;
            if !info.is_file() {
                return Err(format!(
                    "XMP sidecar is not a regular file: {}",
                    plan.old.display()
                ));
            }
            let copied = operation == Operation::Copy || plan.copy_only;
            let temp = (if copied {
                parent(&plan.new)
            } else {
                parent(&plan.old)
            })
            .join(format!(".lap-xmp-{}.tmp", uuid::Uuid::new_v4()));
            guard.entries.push(Entry {
                plan,
                temp,
                copied,
                state: 0,
                fingerprint: (info.len(), info.modified().ok()),
            });
        }
        for entry in &mut guard.entries {
            if entry.copied {
                copy_new(&entry.plan.old, &entry.temp)?;
            } else {
                move_new(&entry.plan.old, &entry.temp)?;
            }
            entry.state = 1;
        }
        for entry in &mut guard.entries {
            move_new(&entry.temp, &entry.plan.new)?;
            entry.state = 2;
        }
        crate::metadata::xmp::sidecars_changed(&guard.affected);
        Ok(guard)
    }
    pub(crate) fn commit_with_cache(self, cache: &mut Cache) {
        cache.update(&self.affected);
        self.finish(Some(cache));
    }
    pub(crate) fn commit(self) {
        self.finish(None);
    }
    fn finish(mut self, mut cache: Option<&mut Cache>) {
        self.committed = true;
        if !self.cleanup_sources {
            crate::metadata::xmp::sidecars_changed(&self.affected);
            return;
        }
        // When every original owner moved, a copied shared stem is now orphaned.
        let mut local = HashMap::new();
        let directories = match cache.as_deref_mut() {
            Some(cache) => &mut cache.directories,
            None => &mut local,
        };
        for entry in self.entries.iter().filter(|e| e.copied && e.plan.copy_only) {
            let folder = parent(&entry.plan.old);
            if !directories.contains_key(folder) {
                if let Ok(directory) = Directory::read(folder) {
                    directories.insert(folder.to_path_buf(), directory);
                }
            }
            if let (Some(directory), Ok(info)) = (
                directories.get(folder),
                fs::symlink_metadata(&entry.plan.old),
            ) {
                let orphan = entry
                    .plan
                    .old
                    .file_stem()
                    .is_some_and(|base| directory.owners.get(base).is_none_or(Vec::is_empty));
                if orphan
                    && info.is_file()
                    && (info.len(), info.modified().ok()) == entry.fingerprint
                {
                    if let Err(error) = fs::remove_file(&entry.plan.old) {
                        eprintln!(
                            "Could not clean moved XMP '{}': {error}",
                            entry.plan.old.display()
                        );
                    }
                }
            }
        }
        if let Some(cache) = cache {
            cache.update(&self.affected);
        }
        crate::metadata::xmp::sidecars_changed(&self.affected);
    }
}
impl Drop for Guard {
    fn drop(&mut self) {
        if self.committed {
            return;
        }
        // Vacate all final names first, permitting swaps and cycles to roll back.
        for entry in &mut self.entries {
            if entry.state == 2 {
                match move_new(&entry.plan.new, &entry.temp) {
                    Ok(()) => entry.state = 1,
                    Err(error) => eprintln!(
                        "Could not roll back XMP '{}': {error}",
                        entry.plan.new.display()
                    ),
                }
            }
        }
        let changed = self.entries.iter().any(|entry| entry.state != 0);
        for entry in &self.entries {
            if entry.state != 1 {
                continue;
            }
            let result = if entry.copied {
                fs::remove_file(&entry.temp).map_err(|e| e.to_string())
            } else {
                move_new(&entry.temp, &entry.plan.old)
            };
            if let Err(error) = result {
                eprintln!(
                    "Could not restore XMP '{}': {error}",
                    entry.plan.old.display()
                );
            }
        }
        if changed {
            crate::metadata::xmp::sidecars_changed(&self.affected);
        }
    }
}

/// Called after media deletions; shared XMP survives until its last owner is gone.
pub(crate) fn delete_orphans<'a>(
    media: impl IntoIterator<Item = &'a Path>,
    permanently: bool,
) -> Result<(), String> {
    let mut directories = HashMap::new();
    let mut seen = HashSet::new();
    let mut errors = Vec::new();
    let mut affected = Vec::new();
    for media in media {
        affected.push(media.to_path_buf());
        if fs::symlink_metadata(media).is_ok() {
            continue;
        }
        let folder = parent(media);
        if !directories.contains_key(folder) {
            match Directory::read(folder) {
                Ok(directory) => {
                    directories.insert(folder.to_path_buf(), directory);
                }
                Err(error) => {
                    errors.push(error);
                    continue;
                }
            }
        }
        let directory = &directories[folder];
        for sidecar in directory.associated(media) {
            if !seen.insert(key(&sidecar)) {
                continue;
            }
            if sidecar.file_stem().is_some_and(|base| {
                directory
                    .owners
                    .get(base)
                    .is_some_and(|owners| !owners.is_empty())
            }) {
                continue;
            }
            affected.push(sidecar.clone());
            let result = (|| {
                let info = fs::symlink_metadata(&sidecar).map_err(|e| e.to_string())?;
                if !info.is_file() {
                    return Err(format!(
                        "XMP sidecar is not a regular file: {}",
                        sidecar.display()
                    ));
                }
                if permanently {
                    crate::t_utils::delete_file_permanently(&sidecar.to_string_lossy())
                } else {
                    crate::t_utils::trash_path(&sidecar.to_string_lossy())
                }
            })();
            if let Err(error) = result {
                errors.push(error);
            }
        }
    }
    crate::metadata::xmp::sidecars_changed(&affected);
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("; "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Fixture(PathBuf);
    impl Fixture {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!("lap-xmp-files-{}", uuid::Uuid::new_v4()));
            fs::create_dir_all(&path).unwrap();
            Self(path)
        }
        fn path(&self, name: &str) -> PathBuf {
            self.0.join(name)
        }
        fn write(&self, name: &str, bytes: &[u8]) -> PathBuf {
            let path = self.path(name);
            fs::create_dir_all(parent(&path)).unwrap();
            fs::write(&path, bytes).unwrap();
            path
        }
        fn read(&self, name: &str) -> Vec<u8> {
            fs::read(self.path(name)).unwrap()
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn rename_preserves_both_naming_forms_and_extension_case() {
        let f = Fixture::new();
        let old = f.write("a.jpg", b"image");
        let new = f.path("b.jpg");
        f.write("a.xmp", b"stem");
        f.write("a.jpg.XmP", b"full");
        let guard = Guard::new([(&*old, &*new)], Operation::Move).unwrap();
        fs::rename(&old, &new).unwrap();
        guard.commit();
        assert_eq!(f.read("b.xmp"), b"stem");
        assert_eq!(f.read("b.jpg.XmP"), b"full");
        assert!(!f.path("a.xmp").exists());
        assert!(!f.path("a.jpg.XmP").exists());
    }
    #[test]
    fn rollback_restores_xmp_after_a_failed_primary_operation() {
        let f = Fixture::new();
        let old = f.write("a.jpg", b"image");
        let new = f.path("b.jpg");
        f.write("a.xmp", b"metadata");
        {
            let _guard = Guard::new([(&*old, &*new)], Operation::Move).unwrap();
            assert_eq!(f.read("b.xmp"), b"metadata");
        }
        assert_eq!(f.read("a.xmp"), b"metadata");
        assert!(!f.path("b.xmp").exists());
        assert_eq!(fs::read_dir(&f.0).unwrap().count(), 2);
    }
    #[test]
    fn shared_raw_jpeg_stem_is_copied_without_harming_remaining_owner() {
        let f = Fixture::new();
        let raw = f.write("a.nef", b"raw");
        f.write("a.jpg", b"jpeg");
        f.write("a.xmp", b"shared");
        let new = f.path("b.nef");
        let guard = Guard::new([(&*raw, &*new)], Operation::Move).unwrap();
        fs::rename(raw, new).unwrap();
        guard.commit();
        assert_eq!(f.read("a.xmp"), b"shared");
        assert_eq!(f.read("b.nef.xmp"), b"shared");
    }
    #[test]
    fn paired_files_move_one_shared_stem_together() {
        let f = Fixture::new();
        let raw = f.write("a.nef", b"raw");
        let jpg = f.write("a.jpg", b"jpeg");
        f.write("a.xmp", b"pair");
        let r = f.path("b.nef");
        let j = f.path("b.jpg");
        let guard = Guard::new([(&*raw, &*r), (&*jpg, &*j)], Operation::Move).unwrap();
        fs::rename(raw, r).unwrap();
        fs::rename(jpg, j).unwrap();
        guard.commit();
        assert_eq!(f.read("b.xmp"), b"pair");
        assert!(!f.path("a.xmp").exists());
    }
    #[test]
    fn split_batch_rename_copies_shared_metadata_and_cleans_orphan_after_commit() {
        let f = Fixture::new();
        let raw = f.write("a.nef", b"raw");
        let jpg = f.write("a.jpg", b"jpeg");
        f.write("a.xmp", b"shared");
        let r = f.path("b.nef");
        let j = f.path("c.jpg");
        let guard = Guard::new([(&*raw, &*r), (&*jpg, &*j)], Operation::Move).unwrap();
        fs::rename(raw, r).unwrap();
        fs::rename(jpg, j).unwrap();
        guard.commit();
        assert_eq!(f.read("b.nef.xmp"), b"shared");
        assert_eq!(f.read("c.jpg.xmp"), b"shared");
        assert!(!f.path("a.xmp").exists());
    }
    #[test]
    fn copy_retains_source_and_preserves_sidecar_timestamp() {
        let f = Fixture::new();
        let old = f.write("a.jpg", b"image");
        let xmp = f.write("a.jpg.xmp", b"full");
        let new = f.path("b.jpg");
        let stamp = fs::metadata(&xmp).unwrap().modified().unwrap();
        let guard = Guard::new([(&*old, &*new)], Operation::Copy).unwrap();
        fs::copy(old, new).unwrap();
        guard.commit();
        assert_eq!(f.read("a.jpg.xmp"), b"full");
        assert_eq!(f.read("b.jpg.xmp"), b"full");
        assert_eq!(
            fs::metadata(f.path("b.jpg.xmp"))
                .unwrap()
                .modified()
                .unwrap(),
            stamp
        );
    }
    #[test]
    fn conflicts_never_overwrite_existing_or_orphan_metadata() {
        let f = Fixture::new();
        let old = f.write("a.jpg", b"image");
        let new = f.path("b.jpg");
        f.write("a.xmp", b"source");
        f.write("b.xmp", b"existing");
        assert!(Guard::new([(&*old, &*new)], Operation::Move).is_err());
        assert_eq!(f.read("a.xmp"), b"source");
        assert_eq!(f.read("b.xmp"), b"existing");
        fs::remove_file(f.path("a.xmp")).unwrap();
        assert!(Guard::new([(&*old, &*new)], Operation::Copy).is_err());
        assert_eq!(f.read("b.xmp"), b"existing");
    }
    #[test]
    fn deleting_one_owner_preserves_shared_xmp_until_last_owner_is_deleted() {
        let f = Fixture::new();
        let raw = f.write("a.nef", b"raw");
        let jpg = f.write("a.jpg", b"jpeg");
        f.write("a.xmp", b"shared");
        f.write("a.nef.xmp", b"own");
        delete_orphans([raw.as_path()], true).unwrap();
        assert!(f.path("a.nef.xmp").exists());
        fs::remove_file(&raw).unwrap();
        delete_orphans([raw.as_path()], true).unwrap();
        assert!(!f.path("a.nef.xmp").exists());
        assert!(f.path("a.xmp").exists());
        fs::remove_file(&jpg).unwrap();
        delete_orphans([jpg.as_path()], true).unwrap();
        assert!(!f.path("a.xmp").exists());
    }
    #[test]
    fn publish_failure_restores_every_source_and_removes_partial_targets() {
        let f = Fixture::new();
        let a = f.write("a.jpg", b"a");
        let b = f.write("b.jpg", b"b");
        f.write("a.xmp", b"A");
        f.write("b.xmp", b"B");
        let x = f.path("x.jpg");
        let y = f.path("missing/y.jpg");
        assert!(Guard::new([(&*a, &*x), (&*b, &*y)], Operation::Move).is_err());
        assert_eq!(f.read("a.xmp"), b"A");
        assert_eq!(f.read("b.xmp"), b"B");
        assert!(!f.path("x.xmp").exists());
        assert_eq!(fs::read_dir(&f.0).unwrap().count(), 4);
    }
    #[test]
    fn cyclic_rename_and_rollback_preserve_distinct_metadata() {
        let f = Fixture::new();
        let a = f.write("a.jpg", b"a");
        let b = f.write("b.jpg", b"b");
        f.write("a.xmp", b"A");
        f.write("b.xmp", b"B");
        {
            let _guard = Guard::new([(&*a, &*b), (&*b, &*a)], Operation::Move).unwrap();
            assert_eq!(f.read("a.xmp"), b"B");
        }
        assert_eq!(f.read("a.xmp"), b"A");
        assert_eq!(f.read("b.xmp"), b"B");
    }
    #[test]
    fn full_name_metadata_is_independent_from_siblings_shared_stem() {
        let f = Fixture::new();
        let a = f.write("a.nef", b"raw");
        f.write("a.jpg", b"jpeg");
        f.write("a.nef.xmp", b"own");
        f.write("a.xmp", b"shared");
        let b = f.path("b.nef");
        let guard = Guard::new([(&*a, &*b)], Operation::Move).unwrap();
        fs::rename(a, b).unwrap();
        guard.commit();
        assert_eq!(f.read("b.nef.xmp"), b"own");
        assert_eq!(f.read("a.xmp"), b"shared");
    }
    #[test]
    fn destination_peers_require_a_full_name_sidecar() {
        let f = Fixture::new();
        let a = f.write("a.nef", b"raw");
        f.write("a.xmp", b"own");
        f.write("target/b.jpg", b"peer");
        let b = f.path("target/b.nef");
        let guard = Guard::new([(&*a, &*b)], Operation::Copy).unwrap();
        fs::copy(a, b).unwrap();
        guard.commit();
        assert_eq!(f.read("target/b.nef.xmp"), b"own");
        assert!(!f.path("target/b.xmp").exists());
    }
    #[test]
    fn expired_transfer_batches_release_slots_without_removing_active_ids() {
        let mut batches = HashMap::new();
        let active = create_transfer_batch(&mut batches).unwrap();
        for _ in 1..32 {
            create_transfer_batch(&mut batches).unwrap();
        }
        assert!(create_transfer_batch(&mut batches).is_err());
        let expired = std::time::Instant::now() - std::time::Duration::from_secs(3601);
        for (&id, session) in &mut batches {
            if id != active {
                session.touched = expired;
            }
        }
        for _ in 0..31 {
            create_transfer_batch(&mut batches).unwrap();
        }
        assert_eq!(batches.len(), 32);
        assert!(batches.contains_key(&active));
        assert!(batches.values().all(|s| s.touched > expired));
        assert!(create_transfer_batch(&mut batches).is_err());
    }

    #[test]
    fn reusing_transfer_batch_updates_its_activity_time() {
        let batch = begin_transfer_batch().unwrap();
        let previous = std::time::Instant::now() - std::time::Duration::from_secs(3599);
        TRANSFERS.lock().unwrap().get_mut(&batch).unwrap().touched = previous;
        with_transfer_cache(Some(batch), |_| Ok(())).unwrap();
        assert!(TRANSFERS.lock().unwrap()[&batch].touched > previous);
        end_transfer_batch(batch);
    }

    #[test]
    fn transfer_batch_reads_each_directory_once_and_updates_shared_ownership() {
        let f = Fixture::new();
        fs::create_dir(f.path("target")).unwrap();
        for i in 0..32 {
            f.write(&format!("source/{i}.nef"), b"raw");
            f.write(&format!("source/{i}.jpg"), b"jpeg");
            f.write(&format!("source/{i}.xmp"), b"metadata");
        }
        let batch = begin_transfer_batch().unwrap();
        DIRECTORY_READS.with(|reads| reads.set(0));
        for i in 0..32 {
            for ext in ["nef", "jpg"] {
                let old = f.path(&format!("source/{i}.{ext}"));
                let new = f.path(&format!("target/{i}.{ext}"));
                with_transfer_cache(Some(batch), |cache| {
                    assert!(cache.conflicts([(&*old, &*new)])?.is_empty());
                    let guard = Guard::with_cache([(&*old, &*new)], Operation::Move, cache)?;
                    fs::rename(&old, &new).map_err(|e| e.to_string())?;
                    guard.commit_with_cache(cache);
                    Ok(())
                })
                .unwrap();
                assert_eq!(f.read(&format!("target/{i}.{ext}.xmp")), b"metadata");
            }
            assert!(!f.path(&format!("source/{i}.xmp")).exists());
        }
        assert_eq!(DIRECTORY_READS.with(|reads| reads.get()), 2);
        end_transfer_batch(batch);
        assert!(with_transfer_cache(Some(batch), |_| Ok(())).is_err());
    }

    #[test]
    fn import_copies_acknowledge_known_entries_without_rescanning() {
        let f = Fixture::new();
        fs::create_dir(f.path("target")).unwrap();
        for i in 0..32 {
            f.write(&format!("source/{i}.jpg"), b"image");
            f.write(&format!("source/{i}.xmp"), b"metadata");
        }
        let mut cache = Cache::default();
        DIRECTORY_READS.with(|reads| reads.set(0));
        for i in 0..32 {
            let old = f.path(&format!("source/{i}.jpg"));
            let new = f.path(&format!("target/{i}.jpg"));
            cache.ensure_directory(parent(&new)).unwrap();
            fs::copy(&old, &new).unwrap();
            cache.record(&new, &[]);
            let guard = Guard::with_cache([(&*old, &*new)], Operation::Copy, &mut cache).unwrap();
            cache.record(&new, &guard.paths());
            guard.commit();
            assert_eq!(f.read(&format!("target/{i}.xmp")), b"metadata");
        }
        assert_eq!(DIRECTORY_READS.with(|reads| reads.get()), 2);
    }

    #[test]
    fn transfer_batch_detects_new_owner_between_commands() {
        let f = Fixture::new();
        let old = f.write("source/a.nef", b"raw");
        f.write("source/a.xmp", b"metadata");
        fs::create_dir(f.path("target")).unwrap();
        let new = f.path("target/a.nef");
        let batch = begin_transfer_batch().unwrap();
        with_transfer_cache(Some(batch), |cache| {
            cache.conflicts([(&*old, &*new)])?;
            Ok(())
        })
        .unwrap();
        // A file manager or another action adds a JPEG after the snapshot.
        f.write("source/a.jpg", b"jpeg");
        with_transfer_cache(Some(batch), |cache| {
            let guard = Guard::with_cache([(&*old, &*new)], Operation::Move, cache)?;
            fs::rename(&old, &new).map_err(|e| e.to_string())?;
            guard.commit_with_cache(cache);
            Ok(())
        })
        .unwrap();
        end_transfer_batch(batch);
        assert_eq!(f.read("source/a.xmp"), b"metadata");
        assert_eq!(f.read("target/a.nef.xmp"), b"metadata");
    }

    #[test]
    fn transfer_batch_detects_new_source_and_destination_sidecars() {
        let f = Fixture::new();
        let old = f.write("source/a.jpg", b"image");
        fs::create_dir(f.path("target")).unwrap();
        let new = f.path("target/a.jpg");
        let batch = begin_transfer_batch().unwrap();
        with_transfer_cache(Some(batch), |cache| {
            cache.conflicts([(&*old, &*new)])?;
            Ok(())
        })
        .unwrap();
        let orphan = f.write("target/a.xmp", b"unrelated metadata");
        with_transfer_cache(Some(batch), |cache| {
            assert_eq!(cache.conflicts([(&*old, &*new)])?, vec![orphan.clone()]);
            Ok(())
        })
        .unwrap();
        fs::remove_file(orphan).unwrap();
        f.write("source/a.xmp", b"new metadata");
        with_transfer_cache(Some(batch), |cache| {
            let guard = Guard::with_cache([(&*old, &*new)], Operation::Move, cache)?;
            fs::rename(&old, &new).map_err(|e| e.to_string())?;
            guard.commit_with_cache(cache);
            Ok(())
        })
        .unwrap();
        end_transfer_batch(batch);
        assert_eq!(f.read("target/a.xmp"), b"new metadata");
        assert!(!f.path("source/a.xmp").exists());
    }

    #[cfg(unix)]
    #[test]
    fn directory_change_is_detected_even_when_mtime_is_restored() {
        let f = Fixture::new();
        let old = f.write("source/a.jpg", b"image");
        let new = f.path("target/a.jpg");
        let source = f.path("source");
        let before = fs::metadata(&source).unwrap().modified().unwrap();
        let mut cache = Cache::default();
        assert!(cache.import_sidecars(&old).unwrap().is_empty());
        f.write("source/a.xmp", b"metadata");
        fs::File::open(&source)
            .unwrap()
            .set_times(fs::FileTimes::new().set_modified(before))
            .unwrap();
        assert_eq!(fs::metadata(&source).unwrap().modified().unwrap(), before);
        assert_eq!(
            cache.import_sidecars(&old).unwrap(),
            vec![f.path("source/a.xmp")]
        );
        assert!(cache.conflicts([(&*old, &*new)]).unwrap().is_empty());
    }

    #[test]
    fn transfer_batch_copy_updates_conflicts_and_failure_rebuilds_cache() {
        let f = Fixture::new();
        let old = f.write("source/a.jpg", b"image");
        f.write("source/a.xmp", b"metadata");
        fs::create_dir(f.path("target")).unwrap();
        let new = f.path("target/a.jpg");
        let batch = begin_transfer_batch().unwrap();
        with_transfer_cache(Some(batch), |cache| {
            let guard = Guard::with_cache([(&*old, &*new)], Operation::Copy, cache)?;
            fs::copy(&old, &new).map_err(|e| e.to_string())?;
            guard.commit_with_cache(cache);
            Ok(())
        })
        .unwrap();
        let error: Result<(), String> = with_transfer_cache(Some(batch), |cache| {
            assert!(!cache.conflicts([(&*old, &*new)])?.is_empty());
            Err("simulate failed transfer".into())
        });
        assert!(error.is_err());
        with_transfer_cache(Some(batch), |cache| {
            assert!(cache.directories.is_empty());
            assert!(!cache.conflicts([(&*old, &*new)])?.is_empty());
            Ok(())
        })
        .unwrap();
        end_transfer_batch(batch);
        assert_eq!(f.read("source/a.xmp"), b"metadata");
        assert_eq!(f.read("target/a.xmp"), b"metadata");
    }

    #[test]
    fn import_dedup_uses_the_same_sidecars_that_copy_carries() {
        let f = Fixture::new();
        let old = f.write("source/a.nef", b"image");
        f.write("source/a.jpg", b"peer");
        f.write("source/a.nef.xmp", b"own metadata");
        f.write("source/a.xmp", b"shared metadata");
        fs::create_dir(f.path("target")).unwrap();
        let new = f.path("target/a.nef");
        let guard = Guard::new([(&*old, &*new)], Operation::Copy).unwrap();
        fs::copy(&old, &new).unwrap();
        guard.commit();
        let mut cache = Cache::default();
        let left = cache.import_sidecars(&old).unwrap();
        let right = cache.import_sidecars(&new).unwrap();
        assert_eq!(left.len(), 1);
        assert_eq!(left.len(), right.len());
        assert!(
            left.iter()
                .zip(&right)
                .all(|(a, b)| fs::read(a).unwrap() == fs::read(b).unwrap())
        );
        assert_eq!(f.read("source/a.xmp"), b"shared metadata");
    }

    #[test]
    fn scan_and_refresh_lookups_see_publication_before_commit_and_rollback() {
        let f = Fixture::new();
        let old = f.write("source/a.jpg", b"image");
        f.write("source/a.xmp", b"metadata");
        fs::create_dir(f.path("target")).unwrap();
        let new = f.path("target/b.jpg");
        {
            let _scan = crate::metadata::xmp::sidecar_scan(991);
            assert!(crate::metadata::xmp::find_xmp_sidecar(&new).is_none());
            let guard = Guard::new([(&*old, &*new)], Operation::Copy).unwrap();
            assert_eq!(
                crate::metadata::xmp::find_xmp_sidecar(&new),
                Some(f.path("target/b.xmp"))
            );
            drop(guard);
            assert!(crate::metadata::xmp::find_xmp_sidecar(&new).is_none());
        }
        let batch = crate::metadata::xmp::begin_refresh_batch("lifecycle-test".into()).unwrap();
        {
            let _scope = crate::metadata::xmp::refresh_index_scope(Some(batch), "lifecycle-test").unwrap();
            assert!(crate::metadata::xmp::find_xmp_sidecar(&new).is_none());
            let guard = Guard::new([(&*old, &*new)], Operation::Copy).unwrap();
            assert_eq!(
                crate::metadata::xmp::find_xmp_sidecar(&new),
                Some(f.path("target/b.xmp"))
            );
            drop(guard);
            assert!(crate::metadata::xmp::find_xmp_sidecar(&new).is_none());
        }
        crate::metadata::xmp::end_refresh_batch(batch);
    }

    #[test]
    fn another_threads_scan_observes_publication_and_rollback() {
        let f = Fixture::new();
        let old = f.write("source/a.jpg", b"image");
        f.write("source/a.xmp", b"metadata");
        fs::create_dir(f.path("target")).unwrap();
        let new = f.path("target/b.jpg");
        let target = new.clone();
        let (ready, wait_ready) = std::sync::mpsc::channel();
        let (advance, wait_advance) = std::sync::mpsc::channel();
        let worker = std::thread::spawn(move || {
            let _scan = crate::metadata::xmp::sidecar_scan(992);
            assert!(crate::metadata::xmp::find_xmp_sidecar(&target).is_none());
            ready.send(()).unwrap();
            wait_advance.recv().unwrap();
            assert!(crate::metadata::xmp::find_xmp_sidecar(&target).is_some());
            ready.send(()).unwrap();
            wait_advance.recv().unwrap();
            assert!(crate::metadata::xmp::find_xmp_sidecar(&target).is_none());
        });
        wait_ready.recv().unwrap();
        let guard = Guard::new([(&*old, &*new)], Operation::Copy).unwrap();
        advance.send(()).unwrap();
        wait_ready.recv().unwrap();
        drop(guard);
        advance.send(()).unwrap();
        worker.join().unwrap();
    }

    #[test]
    fn scan_lookup_drops_deleted_sidecars() {
        let f = Fixture::new();
        let image = f.write("a.jpg", b"image");
        f.write("a.xmp", b"metadata");
        let _scan = crate::metadata::xmp::sidecar_scan(993);
        assert!(crate::metadata::xmp::find_xmp_sidecar(&image).is_some());
        fs::remove_file(&image).unwrap();
        delete_orphans([image.as_path()], true).unwrap();
        assert!(crate::metadata::xmp::find_xmp_sidecar(&image).is_none());
    }

    #[test]
    fn unsupported_siblings_conservatively_retain_shared_metadata() {
        let f = Fixture::new();
        let image = f.write("a.jpg", b"image");
        f.write("a.uncommonraw", b"other");
        f.write("a.xmp", b"shared");
        fs::remove_file(&image).unwrap();
        delete_orphans([image.as_path()], true).unwrap();
        assert_eq!(f.read("a.xmp"), b"shared");
    }

    #[cfg(unix)]
    #[test]
    fn copy_does_not_require_a_writable_source_directory() {
        use std::os::unix::fs::PermissionsExt;
        let f = Fixture::new();
        let old = f.write("source/a.jpg", b"image");
        f.write("source/a.xmp", b"metadata");
        fs::create_dir(f.path("target")).unwrap();
        let new = f.path("target/a.jpg");
        let permissions = fs::metadata(parent(&old)).unwrap().permissions();
        fs::set_permissions(parent(&old), fs::Permissions::from_mode(0o555)).unwrap();
        let result = Guard::new([(&*old, &*new)], Operation::Copy);
        fs::set_permissions(parent(&old), permissions).unwrap();
        let guard = result.unwrap();
        fs::copy(old, new).unwrap();
        guard.commit();
        assert_eq!(f.read("source/a.xmp"), b"metadata");
        assert_eq!(f.read("target/a.xmp"), b"metadata");
    }

    #[cfg(unix)]
    #[test]
    fn symlink_sidecars_are_not_followed_and_dangling_targets_are_conflicts() {
        let f = Fixture::new();
        let a = f.write("a.jpg", b"image");
        let b = f.path("b.jpg");
        let external = f.write("external", b"private");
        std::os::unix::fs::symlink(&external, f.path("a.xmp")).unwrap();
        assert!(Guard::new([(&*a, &*b)], Operation::Copy).is_err());
        assert_eq!(f.read("external"), b"private");
        fs::remove_file(f.path("a.xmp")).unwrap();
        f.write("a.xmp", b"source");
        std::os::unix::fs::symlink(f.path("absent"), f.path("b.xmp")).unwrap();
        assert!(Guard::new([(&*a, &*b)], Operation::Move).is_err());
        assert_eq!(f.read("a.xmp"), b"source");
    }
}

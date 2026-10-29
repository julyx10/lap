//! Apple-specific file helpers: Live Photo sidecar discovery, AAE sidecar
//! management, and grouped file-transfer planning.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

use crate::t_sqlite::{AFile, AFolder};
use crate::t_utils;

// ---------------------------------------------------------------------------
// Structs
// ---------------------------------------------------------------------------

pub(crate) struct SidecarRenamePlan {
    pub old_path: PathBuf,
    pub new_path: PathBuf,
    pub new_name: String,
    pub file_id: Option<i64>,
}

#[derive(Clone)]
pub(crate) struct SidecarTransferPlan {
    pub old_path: PathBuf,
    pub new_path: PathBuf,
    pub file_id: Option<i64>,
}

#[derive(Clone)]
pub(crate) enum SidecarTransferSource {
    LivePhoto {
        old_path: PathBuf,
        file_id: Option<i64>,
    },
    AppleAae {
        old_path: PathBuf,
    },
}

// ---------------------------------------------------------------------------
// Rename helpers
// ---------------------------------------------------------------------------

pub(crate) fn build_apple_sidecar_rename_plan(
    file_id: i64,
    file_path: &str,
    new_name: &str,
    group_raw_jpeg_pairs: bool,
) -> Result<Vec<SidecarRenamePlan>, String> {
    let components = AFile::operation_component_files(file_id, group_raw_jpeg_pairs)?;
    build_rename_plan(file_path, new_name, &components, apple_aae_sidecar_paths(file_path))
}

pub(crate) fn build_apple_sidecar_rename_plan_cached(
    file_path: &str,
    new_name: &str,
    component: Option<&AFile>,
    cache: &mut ImportAaeCache,
) -> Result<Vec<SidecarRenamePlan>, String> {
    let components = component.map(std::slice::from_ref).unwrap_or(&[]);
    build_rename_plan(file_path, new_name, components, cache.paths(Path::new(file_path)))
}

fn build_rename_plan(
    file_path: &str,
    new_name: &str,
    components: &[AFile],
    aae_paths: Vec<PathBuf>,
) -> Result<Vec<SidecarRenamePlan>, String> {
    let primary_path = Path::new(file_path);
    let new_stem = Path::new(new_name)
        .file_stem()
        .and_then(|stem| stem.to_str())
        .ok_or_else(|| format!("Invalid new file name: {}", new_name))?;
    let mut plans = Vec::new();
    let mut seen_sidecar_paths = HashSet::new();

    for component in components {
        if let Some(old_path) = component.file_path.as_deref() {
            if let Some(plan) =
                build_sidecar_rename_plan(Path::new(old_path), new_stem, component.id)
            {
                if seen_sidecar_paths.insert(normalize_sidecar_path_key(&plan.old_path)) {
                    plans.push(plan);
                }
            }
        }
    }

    if let Some(parent) = primary_path.parent() {
        for old_path in aae_paths {
            if let Some(plan) =
                build_aae_rename_plan(primary_path, parent, &old_path, new_name, new_stem)
            {
                if seen_sidecar_paths.insert(normalize_sidecar_path_key(&plan.old_path)) {
                    plans.push(plan);
                }
            }
        }
    }

    Ok(plans)
}

pub(crate) fn preflight_rename_plan(
    file_path: &str,
    new_name: &str,
    sidecar_plans: &[SidecarRenamePlan],
) -> bool {
    let primary_path = Path::new(file_path);
    let primary_target = primary_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(new_name);
    if primary_target.exists() {
        eprintln!("Target file already exists: {}", primary_target.display());
        return false;
    }

    for plan in sidecar_plans {
        if !plan.old_path.exists() {
            eprintln!("Sidecar file does not exist: {}", plan.old_path.display());
            return false;
        }
        if plan.new_path.exists() {
            eprintln!("Target sidecar already exists: {}", plan.new_path.display());
            return false;
        }
    }

    true
}

pub(crate) fn rollback_renamed_sidecars(renamed_sidecars: Vec<(PathBuf, PathBuf)>) {
    for (old_path, new_path) in renamed_sidecars.into_iter().rev() {
        if let Err(error) = fs::rename(&new_path, &old_path) {
            eprintln!(
                "Failed to rollback sidecar rename '{}' to '{}': {}",
                new_path.display(),
                old_path.display(),
                error
            );
        }
    }
}

pub(crate) fn collect_original_rename_db_names(
    file_id: i64,
    sidecar_plans: &[SidecarRenamePlan],
) -> HashMap<i64, (String, Option<String>)> {
    let mut originals = HashMap::new();
    if let Ok(Some(file)) = AFile::get_file_info(file_id) {
        originals.insert(file_id, (file.name, file.name_pinyin));
    }
    for plan in sidecar_plans {
        let Some(sidecar_file_id) = plan.file_id else {
            continue;
        };
        if originals.contains_key(&sidecar_file_id) {
            continue;
        }
        if let Ok(Some(file)) = AFile::get_file_info(sidecar_file_id) {
            originals.insert(sidecar_file_id, (file.name, file.name_pinyin));
        }
    }
    originals
}

pub(crate) fn rollback_rename_changes(
    old_file_path: &str,
    new_file_path: Option<&str>,
    renamed_sidecars: Vec<(PathBuf, PathBuf)>,
    original_db_names: &HashMap<i64, (String, Option<String>)>,
) {
    if let Some(new_file_path) = new_file_path {
        if let Err(error) = fs::rename(new_file_path, old_file_path) {
            eprintln!(
                "Failed to rollback primary rename '{}' to '{}': {}",
                new_file_path, old_file_path, error
            );
        }
    }
    rollback_renamed_sidecars(renamed_sidecars);
    let db_updates = original_db_names
        .iter()
        .map(|(file_id, (name, name_pinyin))| (*file_id, name.clone(), name_pinyin.clone()))
        .collect::<Vec<_>>();
    if let Err(error) = AFile::batch_update_names(&db_updates) {
        eprintln!("Failed to rollback file names in DB: {}", error);
    }
}

// ---------------------------------------------------------------------------
// AAE sidecar helpers
// ---------------------------------------------------------------------------

/// Operation-local directory cache; imports update it after successful copies.
#[derive(Default)]
pub(crate) struct ImportAaeCache(HashMap<PathBuf, HashMap<String, Vec<PathBuf>>>);

impl ImportAaeCache {
    pub(crate) fn paths(&mut self, path: &Path) -> Vec<PathBuf> {
        let Some(parent) = path.parent() else {
            return Vec::new();
        };
        let entries = self.0.entry(parent.to_path_buf()).or_insert_with(|| {
            let mut index = HashMap::<String, Vec<PathBuf>>::new();
            for entry in fs::read_dir(parent)
                .into_iter()
                .flatten()
                .filter_map(Result::ok)
            {
                let path = entry.path();
                if path.is_file()
                    && path
                        .extension()
                        .is_some_and(|ext| ext.eq_ignore_ascii_case("aae"))
                {
                    let key = entry.file_name().to_string_lossy().to_ascii_lowercase();
                    index.entry(key).or_default().push(path);
                }
            }
            index
        });
        let stem = format!(
            "{}.aae",
            path.file_stem().unwrap_or_default().to_string_lossy()
        )
        .to_ascii_lowercase();
        let full = format!(
            "{}.aae",
            path.file_name().unwrap_or_default().to_string_lossy()
        )
        .to_ascii_lowercase();
        let mut matches = entries.get(&stem).cloned().unwrap_or_default();
        if full != stem {
            if let Some(paths) = entries.get(&full) {
                matches.extend(paths.iter().cloned());
            }
        }
        matches.sort();
        matches
    }

    pub(crate) fn record(&mut self, paths: &[PathBuf]) {
        for path in paths {
            if let Some(entries) = path.parent().and_then(|parent| self.0.get_mut(parent)) {
                if let Some(name) = path.file_name() {
                    let values = entries
                        .entry(name.to_string_lossy().to_ascii_lowercase())
                        .or_default();
                    if !values.contains(path) {
                        values.push(path.clone());
                    }
                }
            }
        }
    }
}

pub(crate) fn apple_aae_sidecar_paths(file_path: &str) -> Vec<PathBuf> {
    let path = Path::new(file_path);
    let Some(parent) = path.parent() else {
        return Vec::new();
    };
    let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) else {
        return Vec::new();
    };
    let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
        return Vec::new();
    };
    let direct_name = format!("{}.aae", stem).to_ascii_lowercase();
    let full_name = format!("{}.aae", file_name).to_ascii_lowercase();

    let mut paths = Vec::new();
    let mut seen = HashSet::new();
    let Ok(entries) = fs::read_dir(parent) else {
        return Vec::new();
    };

    for entry in entries.flatten() {
        let sidecar = entry.path();
        if !sidecar.is_file() {
            continue;
        }
        let Some(ext) = sidecar.extension().and_then(|ext| ext.to_str()) else {
            continue;
        };
        if !ext.eq_ignore_ascii_case("aae") {
            continue;
        }
        let Some(name) = sidecar.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        let normalized_name = name.to_ascii_lowercase();
        if (normalized_name == direct_name || normalized_name == full_name)
            && seen.insert(normalize_sidecar_path_key(&sidecar))
        {
            paths.push(sidecar);
        }
    }
    paths
}

pub(crate) fn delete_apple_aae_sidecars(file_path: &str, permanently: bool) -> Result<(), String> {
    for sidecar in apple_aae_sidecar_paths(file_path) {
        let sidecar_path = sidecar.to_string_lossy();
        if permanently {
            t_utils::delete_file_permanently(&sidecar_path)?;
        } else {
            t_utils::trash_path(&sidecar_path)?;
        }
    }
    Ok(())
}

/// Copy Apple edit sidecars alongside an imported file. The destination file
/// name may have received a conflict suffix, so derive each sidecar name from
/// the final destination rather than copying its source name verbatim.
pub(crate) fn copy_apple_aae_paths_for_import(
    source_file_path: &str,
    destination_file_path: &str,
    sidecars: Vec<PathBuf>,
) -> Result<Vec<PathBuf>, String> {
    let source = Path::new(source_file_path);
    let destination = Path::new(destination_file_path);
    let mut copied = Vec::new();

    for sidecar in sidecars {
        let Some(name) = build_aae_transfer_target_name(
            source,
            &sidecar,
            destination.file_name().and_then(|name| name.to_str()).ok_or_else(|| {
                format!("Invalid import destination: {}", destination.display())
            })?,
            destination.file_stem().and_then(|stem| stem.to_str()).ok_or_else(|| {
                format!("Invalid import destination: {}", destination.display())
            })?,
        ) else {
            continue;
        };
        let target = destination
            .parent()
            .ok_or_else(|| format!("Invalid import destination: {}", destination.display()))?
            .join(name);
        if target.exists() {
            for copied_path in copied {
                let _ = fs::remove_file(copied_path);
            }
            return Err(format!("Destination sidecar already exists: {}", target.display()));
        }
        if let Err(error) = fs::copy(&sidecar, &target) {
            for copied_path in copied {
                let _ = fs::remove_file(copied_path);
            }
            return Err(format!("Failed to copy sidecar '{}': {}", sidecar.display(), error));
        }
        copied.push(target);
    }

    Ok(copied)
}

// ---------------------------------------------------------------------------
// Transfer (move / copy) helpers
// ---------------------------------------------------------------------------

fn file_record_for_path(file_path: &str) -> Result<Option<AFile>, String> {
    let path = Path::new(file_path);
    let parent = path
        .parent()
        .and_then(|parent| parent.to_str())
        .ok_or_else(|| format!("Invalid file path: {}", file_path))?;
    let Some(folder) = AFolder::fetch(parent)? else {
        return Ok(None);
    };
    let Some(folder_id) = folder.id else {
        return Ok(None);
    };
    AFile::fetch(folder_id, file_path)
}

fn live_photo_component_files_for_path(file_path: &str, group_raw_jpeg_pairs: bool) -> Result<Vec<AFile>, String> {
    let Some(file) = file_record_for_path(file_path)? else {
        return Ok(Vec::new());
    };
    let Some(file_id) = file.id else {
        return Ok(Vec::new());
    };
    AFile::operation_component_files(file_id, group_raw_jpeg_pairs)
}

fn resolve_apple_sidecar_transfer_sources(
    file_id: Option<i64>,
    file_path: &str,
    group_raw_jpeg_pairs: bool,
) -> Result<Vec<SidecarTransferSource>, String> {
    let mut seen_paths = HashSet::new();

    let component_files = if let Some(file_id) = file_id {
        AFile::operation_component_files(file_id, group_raw_jpeg_pairs)?
    } else {
        live_photo_component_files_for_path(file_path, group_raw_jpeg_pairs)?
    };

    let mut sources = Vec::new();
    for component in component_files {
        if let Some(path) = component.file_path.as_deref() {
            let old_path = PathBuf::from(path);
            if seen_paths.insert(normalize_sidecar_path_key(&old_path)) {
                sources.push(SidecarTransferSource::LivePhoto {
                    old_path,
                    file_id: component.id,
                });
            }
        }
    }

    for sidecar in apple_aae_sidecar_paths(file_path) {
        if seen_paths.insert(normalize_sidecar_path_key(&sidecar)) {
            sources.push(SidecarTransferSource::AppleAae { old_path: sidecar });
        }
    }

    Ok(sources)
}

fn build_apple_sidecar_transfer_plan_from_sources(
    file_path: &str,
    primary_target: &Path,
    sources: &[SidecarTransferSource],
) -> Result<Vec<SidecarTransferPlan>, String> {
    let primary_path = Path::new(file_path);
    let target_parent = primary_target
        .parent()
        .ok_or_else(|| format!("Invalid target path: {}", primary_target.display()))?;
    let target_stem = primary_target
        .file_stem()
        .and_then(|stem| stem.to_str())
        .ok_or_else(|| format!("Invalid target path: {}", primary_target.display()))?;
    let target_name = primary_target
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| format!("Invalid target path: {}", primary_target.display()))?;

    let mut plans = Vec::new();
    for source in sources {
        match source {
            SidecarTransferSource::LivePhoto { old_path, file_id } => {
                let Some(ext) = old_path.extension().and_then(|ext| ext.to_str()) else {
                    continue;
                };
                let new_path = target_parent.join(format!("{}.{}", target_stem, ext));
                if old_path != &new_path {
                    plans.push(SidecarTransferPlan {
                        old_path: old_path.clone(),
                        new_path,
                        file_id: *file_id,
                    });
                }
            }
            SidecarTransferSource::AppleAae { old_path } => {
                if let Some(new_name) =
                    build_aae_transfer_target_name(primary_path, old_path, target_name, target_stem)
                {
                    let new_path = target_parent.join(new_name);
                    if old_path != &new_path {
                        plans.push(SidecarTransferPlan {
                            old_path: old_path.clone(),
                            new_path,
                            file_id: None,
                        });
                    }
                }
            }
        }
    }
    Ok(plans)
}

fn build_aae_transfer_target_name(
    primary_path: &Path,
    old_path: &Path,
    target_name: &str,
    target_stem: &str,
) -> Option<String> {
    let old_sidecar_name = old_path.file_name().and_then(|name| name.to_str())?;
    let old_primary_name = primary_path.file_name().and_then(|name| name.to_str())?;
    let old_primary_stem = primary_path.file_stem().and_then(|stem| stem.to_str())?;
    let ext = old_path.extension().and_then(|ext| ext.to_str())?;
    let old_name_lower = old_sidecar_name.to_ascii_lowercase();

    if old_name_lower == format!("{}.{}", old_primary_stem, ext).to_ascii_lowercase() {
        Some(format!("{}.{}", target_stem, ext))
    } else if old_name_lower == format!("{}.{}", old_primary_name, ext).to_ascii_lowercase() {
        Some(format!("{}.{}", target_name, ext))
    } else {
        None
    }
}

fn group_target_conflicts(primary_target: &Path, sidecar_plans: &[SidecarTransferPlan]) -> bool {
    primary_target.exists() || sidecar_plans.iter().any(|plan| plan.new_path.exists())
}

fn sibling_candidate_path(base: &Path, index: usize) -> PathBuf {
    if index == 0 {
        return base.to_path_buf();
    }

    let parent = base.parent().unwrap_or_else(|| Path::new(""));
    let stem = base
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or_default();
    let ext = base.extension().and_then(|ext| ext.to_str()).unwrap_or("");
    let name = if ext.is_empty() {
        format!("{}({})", stem, index)
    } else {
        format!("{}({}).{}", stem, index, ext)
    };
    parent.join(name)
}

pub(crate) fn resolve_group_primary_target(
    file_id: Option<i64>,
    file_path: &str,
    new_folder_path: &str,
    policy: t_utils::FileConflictPolicy,
    xmp_cache: &mut crate::t_xmp_files::Cache,
    group_raw_jpeg_pairs: bool,
) -> Result<(PathBuf, Vec<SidecarTransferPlan>), String> {
    let base_target =
        t_utils::resolve_file_transfer_destination(file_path, new_folder_path, policy)?;
    let sidecar_sources = resolve_apple_sidecar_transfer_sources(file_id, file_path, group_raw_jpeg_pairs)?;
    if policy == t_utils::FileConflictPolicy::Replace {
        let sidecar_plans = build_apple_sidecar_transfer_plan_from_sources(
            file_path,
            &base_target,
            &sidecar_sources,
        )?;
        let conflicts = xmp_cache.conflicts(
            std::iter::once((Path::new(file_path), base_target.as_path()))
                .chain(sidecar_plans.iter().map(|p| (p.old_path.as_path(), p.new_path.as_path()))),
        )?;
        if !conflicts.is_empty() { return Err(format!("Destination XMP already exists: {}", conflicts[0].display())); }
        return Ok((base_target, sidecar_plans));
    }

    for index in 0..10_000 {
        let primary_target = sibling_candidate_path(&base_target, index);
        let sidecar_plans = build_apple_sidecar_transfer_plan_from_sources(
            file_path,
            &primary_target,
            &sidecar_sources,
        )?;
        if !group_target_conflicts(&primary_target, &sidecar_plans)
            && xmp_cache.conflicts(std::iter::once((Path::new(file_path), primary_target.as_path()))
                .chain(sidecar_plans.iter().map(|p| (p.old_path.as_path(), p.new_path.as_path()))))?.is_empty() {
            return Ok((primary_target, sidecar_plans));
        }
    }
    Err(format!(
        "Failed to resolve unique target path for {}",
        file_path
    ))
}

fn push_replaced_file_id(
    file_id: i64,
    source_file_ids: &HashSet<i64>,
    replaced_file_ids: &mut Vec<i64>,
) {
    if source_file_ids.contains(&file_id) || replaced_file_ids.contains(&file_id) {
        return;
    }
    replaced_file_ids.push(file_id);
}

pub(crate) fn collect_replaced_file_ids_for_targets<'a, I>(
    folder_id: i64,
    targets: I,
    source_file_ids: &HashSet<i64>,
    replaced_file_ids: &mut Vec<i64>,
) where
    I: IntoIterator<Item = &'a PathBuf>,
{
    collect_replaced_targets(targets, source_file_ids, replaced_file_ids, |path| {
        AFile::fetch(folder_id, path.to_str()?).ok().flatten()?.id
    });
}

fn collect_replaced_targets<'a, I>(
    targets: I,
    source_file_ids: &HashSet<i64>,
    replaced_file_ids: &mut Vec<i64>,
    mut lookup: impl FnMut(&Path) -> Option<i64>,
) where I: IntoIterator<Item = &'a PathBuf> {
    // Only actual destination paths are replaced on disk. Associated files
    // which are not transfer targets must remain in the catalog as well.
    for target in targets {
        if let Some(id) = lookup(target) {
            push_replaced_file_id(id, source_file_ids, replaced_file_ids);
        }
    }
}

pub(crate) fn rollback_copied_transfers(transfers: Vec<t_utils::TransferResult>) {
    for transfer in transfers.into_iter().rev() {
        if let Err(error) = transfer.rollback_copy() {
            eprintln!("Failed to rollback copied sidecar: {}", error);
        }
    }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

fn normalize_sidecar_path_key(path: &Path) -> String {
    path.to_string_lossy().to_ascii_lowercase()
}

fn build_sidecar_rename_plan(
    old_path: &Path,
    new_stem: &str,
    file_id: Option<i64>,
) -> Option<SidecarRenamePlan> {
    let parent = old_path.parent()?;
    let ext = old_path.extension().and_then(|ext| ext.to_str())?;
    let new_name = format!("{}.{}", new_stem, ext);
    let new_path = parent.join(&new_name);
    if old_path == new_path {
        return None;
    }
    Some(SidecarRenamePlan {
        old_path: old_path.to_path_buf(),
        new_path,
        new_name,
        file_id,
    })
}

fn build_aae_rename_plan(
    primary_path: &Path,
    parent: &Path,
    old_path: &Path,
    new_name: &str,
    new_stem: &str,
) -> Option<SidecarRenamePlan> {
    let old_sidecar_name = old_path.file_name().and_then(|name| name.to_str())?;
    let old_primary_name = primary_path.file_name().and_then(|name| name.to_str())?;
    let old_primary_stem = primary_path.file_stem().and_then(|stem| stem.to_str())?;
    let ext = old_path.extension().and_then(|ext| ext.to_str())?;
    let old_name_lower = old_sidecar_name.to_ascii_lowercase();
    let new_sidecar_name =
        if old_name_lower == format!("{}.{}", old_primary_stem, ext).to_ascii_lowercase() {
            format!("{}.{}", new_stem, ext)
        } else if old_name_lower == format!("{}.{}", old_primary_name, ext).to_ascii_lowercase() {
            format!("{}.{}", new_name, ext)
        } else {
            return None;
        };
    let new_path = parent.join(&new_sidecar_name);
    if old_path == new_path {
        return None;
    }
    Some(SidecarRenamePlan {
        old_path: old_path.to_path_buf(),
        new_path,
        new_name: new_sidecar_name,
        file_id: None,
    })
}

// ---------------------------------------------------------------------------
// Content identifier helpers (Apple Live Photo pairing)
// ---------------------------------------------------------------------------

/// Read the XMP area of a filename-matched image candidate. Apple Photos can
/// place HEIF XMP metadata beyond the small header used for regular EXIF work.
pub(crate) fn scan_apple_content_identifiers(file_path: &str) -> Vec<String> {
    let Ok(mut file) = fs::File::open(file_path) else {
        return Vec::new();
    };
    let mut buf = vec![0u8; 2 * 1024 * 1024];
    let Ok(n) = file.read(&mut buf) else {
        return Vec::new();
    };
    buf.truncate(n);
    apple_content_identifiers_from_bytes(&buf)
}

pub(crate) fn apple_content_identifier_from_bytes(buf: &[u8]) -> Option<String> {
    apple_content_identifiers_from_bytes(buf).into_iter().next()
}

pub(crate) fn apple_content_identifiers_from_bytes(buf: &[u8]) -> Vec<String> {
    let text = String::from_utf8_lossy(&buf);
    let mut identifiers = Vec::new();
    for marker in [
        "com.apple.quicktime.content.identifier",
        "ContentIdentifier",
    ] {
        if let Some(identifier) = identifier_after_marker(&text, marker) {
            identifiers.push(identifier);
        }
    }

    // Some Photos-exported HEIC files keep the identifier in an HEIF metadata
    // item without the XMP property name as a plain-text neighbour. The caller
    // only accepts this value when it exactly matches the filename-paired MOV.
    for token in text.split(|c: char| !(c.is_ascii_alphanumeric() || c == '-' || c == '_')) {
        let mut add_identifier = |identifier: &str| {
            if looks_like_content_identifier(identifier)
                && !identifiers
                    .iter()
                    .any(|existing| existing.eq_ignore_ascii_case(identifier))
            {
                identifiers.push(identifier.to_string());
            }
        };
        add_identifier(token);

        // HEIF item payloads can prefix the UUID with a non-metadata byte
        // that still decodes as ASCII. Search the token for an embedded UUID.
        for start in 0..token.len().saturating_sub(35) {
            add_identifier(&token[start..start + 36]);
        }
    }
    identifiers
}

fn identifier_after_marker(text: &str, marker: &str) -> Option<String> {
    let marker_offset = text.find(marker)? + marker.len();
    let value_area = text.get(marker_offset..)?;
    let value_area = &value_area[..value_area.len().min(256)];
    value_area
        .split(|c: char| !(c.is_ascii_alphanumeric() || c == '-' || c == '_'))
        .find(|token| looks_like_content_identifier(token))
        .map(str::to_string)
}

pub(crate) fn looks_like_content_identifier(value: &str) -> bool {
    let len = value.len();
    if !(32..=64).contains(&len) {
        return false;
    }

    let uuid_like = len == 36
        && value.chars().enumerate().all(|(idx, ch)| match idx {
            8 | 13 | 18 | 23 => ch == '-',
            _ => ch.is_ascii_hexdigit(),
        });
    if uuid_like {
        return true;
    }

    len >= 32
        && value
            .chars()
            .all(|ch| ch.is_ascii_hexdigit() || ch == '-' || ch == '_')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replacement_collects_only_actual_targets_and_skips_sources() {
        let raw = PathBuf::from("/dest/a.CR3");
        let jpeg = PathBuf::from("/dest/a.JPG");
        let source = PathBuf::from("/dest/source.JPG");
        let lookup = |path: &Path| match path.to_str().unwrap() {
            "/dest/a.CR3" => Some(1),
            "/dest/a.JPG" => Some(2),
            "/dest/source.JPG" => Some(3),
            _ => None,
        };
        let mut replaced = vec![];
        collect_replaced_targets([&raw], &HashSet::new(), &mut replaced, lookup);
        assert_eq!(replaced, vec![1]); // The unoverwritten JPEG remains indexed.
        replaced.clear();
        collect_replaced_targets([&raw, &jpeg, &jpeg, &source], &HashSet::from([3]), &mut replaced, lookup);
        assert_eq!(replaced, vec![1, 2]);
    }

    #[test]
    fn indexed_aae_lookup_matches_both_conventions_and_records_new_imports() {
        let directory =
            std::env::temp_dir().join(format!("lap-aae-cache-test-{}", uuid::Uuid::new_v4()));
        fs::create_dir(&directory).unwrap();
        struct Cleanup(PathBuf);
        impl Drop for Cleanup {
            fn drop(&mut self) {
                let _ = fs::remove_dir_all(&self.0);
            }
        }
        let _cleanup = Cleanup(directory.clone());
        for name in ["IMG.AAE", "IMG.HEIC.AAE", "OTHER.AAE"] {
            fs::write(directory.join(name), b"aae").unwrap();
        }
        let mut cache = ImportAaeCache::default();
        let photo = directory.join("IMG.HEIC");
        let mut expected = apple_aae_sidecar_paths(photo.to_str().unwrap());
        expected.sort();
        assert_eq!(cache.paths(&photo), expected);
        assert_eq!(
            cache.paths(&directory.join("UNKNOWN.JPG")),
            Vec::<PathBuf>::new()
        );
        let imported = directory.join("NEW.AAE");
        fs::write(&imported, b"aae").unwrap();
        cache.record(&[imported.clone(), imported.clone()]);
        assert_eq!(cache.paths(&directory.join("NEW.JPG")), vec![imported]);
        assert_eq!(cache.0.len(), 1);
    }
}

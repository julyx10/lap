//! Read-only, on-demand MakerNote AF areas. Unknown status is never failure.
use exiftool_rs::{ExifTool, Options, Tag};
use serde::Serialize;
use std::{
    collections::VecDeque,
    sync::{Arc, Mutex, OnceLock},
    time::SystemTime,
};

#[derive(Clone, Debug, Serialize)]
pub struct AfArea {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    // None is a selected area without a confirmed result, never a failure.
    pub focused: Option<bool>,
}

#[tauri::command]
pub async fn get_af_areas(file_path: String) -> Result<Vec<AfArea>, String> {
    tauri::async_runtime::spawn_blocking(move || read(&file_path))
        .await
        .map_err(|e| e.to_string())
}

const CACHE_CAPACITY: usize = 32;
type CachedAreas = Arc<OnceLock<Result<Vec<AfArea>, ()>>>;
#[derive(Clone, PartialEq, Eq)]
struct CacheKey {
    path: String,
    modified: SystemTime,
    size: u64,
}
#[derive(Default)]
struct AreaCache(VecDeque<(CacheKey, CachedAreas)>);
impl AreaCache {
    fn entry(&mut self, key: CacheKey) -> CachedAreas {
        let entry = self
            .0
            .iter()
            .position(|(cached, _)| cached == &key)
            .and_then(|i| self.0.remove(i))
            .unwrap_or_else(|| (key.clone(), Arc::new(OnceLock::new())));
        // A new file version replaces its older cached result.
        self.0.retain(|(cached, _)| cached.path != key.path);
        let cell = entry.1.clone();
        self.0.push_front(entry);
        self.0.truncate(CACHE_CAPACITY);
        cell
    }
}
static CACHE: OnceLock<Mutex<AreaCache>> = OnceLock::new();
thread_local! {
    static READER: ExifTool = ExifTool::with_options(Options {
        duplicates: true,
        requested_tags: vec!["Make".into(), "Model".into(), "Orientation".into(), "MakerNotes:*".into()],
        ..Default::default()
    });
}

fn read(path: &str) -> Vec<AfArea> {
    let Ok(info) = std::fs::metadata(path) else {
        return vec![];
    };
    if !info.is_file() || crate::t_utils::is_cloud_only(&info) {
        return vec![];
    }
    let Ok(modified) = info.modified() else {
        return read_uncached(path).unwrap_or_default();
    };
    let key = CacheKey {
        path: path.into(),
        modified,
        size: info.len(),
    };
    let cache = CACHE.get_or_init(|| Mutex::new(AreaCache::default()));
    let cell = cache.lock().unwrap_or_else(|e| e.into_inner()).entry(key);
    // Only callers for the same version of the same file wait for one another.
    // No global cache lock is held while accessing a local disk or NAS.
    let result = cell.get_or_init(|| read_uncached(path));
    if result.is_err() {
        cache
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .0
            .retain(|(_, entry)| !Arc::ptr_eq(entry, &cell));
    }
    result.clone().unwrap_or_default()
}

fn read_uncached(path: &str) -> Result<Vec<AfArea>, ()> {
    let tags = READER
        .with(|reader| reader.extract_info(path))
        .map_err(|_| ())?;
    let regions = from_tags(&tags);
    if !regions.is_empty() || !needs_nikon_fallback(&tags) {
        return Ok(regions);
    }
    // Fall back only when the metadata reader did not expose Nikon AF areas.
    let exif = crate::t_image::read_exif_permissive(path)
        .filter(|e| {
            e.get_field(exif::Tag::MakerNote, exif::In::PRIMARY)
                .is_some()
        })
        .or_else(|| {
            // RAW MakerNotes may exceed the importer's 128 KiB prefix. Read a
            // bounded metadata prefix, never the entire RAW pixel payload.
            use std::io::Read;
            let file = std::fs::File::open(path).ok()?;
            let mut bytes = Vec::new();
            file.take(16 * 1024 * 1024).read_to_end(&mut bytes).ok()?;
            crate::t_image::read_exif_from_bytes_permissive(&bytes)
        });
    let Some(exif) = exif else {
        return Ok(vec![]);
    };
    let Some(field) = exif.get_field(exif::Tag::MakerNote, exif::In::PRIMARY) else {
        return Ok(vec![]);
    };
    let exif::Value::Undefined(data, _) = &field.value else {
        return Ok(vec![]);
    };
    let orientation = exif
        .get_field(exif::Tag::Orientation, exif::In::PRIMARY)
        .and_then(|f| f.value.get_uint(0))
        .unwrap_or(1) as u8;
    Ok(nikon_maker_note(data, orientation).into_iter().collect())
}

fn needs_nikon_fallback(tags: &[Tag]) -> bool {
    let nikon = tags.iter().any(|tag| {
        (tag.name == "Make" && tag.print_value.to_ascii_uppercase().starts_with("NIKON"))
            || tag.group.family1.starts_with("Nikon")
    });
    nikon
        && !tags.iter().any(|tag| {
            (tag.name == "AFInfo2Version"
                && !matches!(tag.print_value.as_str(), "0400" | "0401" | "0402"))
                || (tag.name == "AFCoordinatesAvailable" && tag.print_value == "No")
        })
}

// Nikon type-3 MakerNotes contain their own TIFF header at byte 10. All offsets
// are relative to that header; never scan arbitrary image bytes for signatures.
fn nikon_maker_note(data: &[u8], orientation: u8) -> Option<AfArea> {
    if !data.starts_with(b"Nikon\0\x02") {
        return None;
    }
    let tiff = data.get(10..)?;
    let le = match tiff.get(..2)? {
        b"II" => true,
        b"MM" => false,
        _ => return None,
    };
    let u16_at = |d: &[u8], p: usize| -> Option<u16> {
        let a = d.get(p..p.checked_add(2)?)?.try_into().ok()?;
        Some(if le {
            u16::from_le_bytes(a)
        } else {
            u16::from_be_bytes(a)
        })
    };
    let u32_at = |p: usize| -> Option<u32> {
        let a = tiff.get(p..p.checked_add(4)?)?.try_into().ok()?;
        Some(if le {
            u32::from_le_bytes(a)
        } else {
            u32::from_be_bytes(a)
        })
    };
    if u16_at(tiff, 2)? != 42 {
        return None;
    }
    let ifd = u32_at(4)? as usize;
    let count = u16_at(tiff, ifd)? as usize;
    for i in 0..count.min(4096) {
        let p = ifd.checked_add(2 + i * 12)?;
        if u16_at(tiff, p)? != 0xb7 {
            continue;
        }
        if u16_at(tiff, p + 2)? != 7 {
            return None;
        }
        let len = u32_at(p + 4)? as usize;
        let offset = u32_at(p + 8)? as usize;
        let af = tiff.get(offset..offset.checked_add(len)?)?;
        // AFInfo2 0400/0401/0402 (Expeed 7), see Nikon AFInfo2V0400.
        if !matches!(af.get(..4)?, b"0400" | b"0401" | b"0402") || *af.get(7)? != 1 {
            return None;
        }
        let num = |p| u16_at(af, p).map(f64::from);
        let (w, h, x, y, rw, rh) = (
            num(0x3e)?,
            num(0x40)?,
            num(0x42)?,
            num(0x44)?,
            num(0x46)?,
            num(0x48)?,
        );
        let focused = match af.get(0x4a)? {
            0 => false,
            1 => true,
            _ => return None,
        };
        if w == 0.0 || h == 0.0 {
            return None;
        }
        return region(
            (x - rw / 2.0) / w,
            (y - rh / 2.0) / h,
            rw / w,
            rh / h,
            Some(focused),
            orientation,
        );
    }
    None
}

fn from_tags(tags: &[Tag]) -> Vec<AfArea> {
    // Only use camera metadata, not user-supplied XMP regions or preview tags.
    let text = |name: &str| {
        tags.iter()
            .find(|t| t.name == name && t.group.family0 == "MakerNotes")
            .map(|t| t.print_value.clone())
    };
    let numbers = |name: &str| -> Vec<f64> {
        text(name)
            .unwrap_or_default()
            .split(|c: char| c.is_whitespace() || c == ',')
            .filter_map(|v| v.parse::<f64>().ok())
            .collect()
    };
    let scalar = |name: &str| numbers(name).first().copied();
    let canon = tags.iter().any(|tag| {
        (tag.name == "Make"
            && tag.group.family1 == "IFD0"
            && tag.print_value.trim().eq_ignore_ascii_case("Canon"))
            || (tag.group.family0 == "MakerNotes" && tag.group.family1.starts_with("Canon"))
    });
    // The original EOS 5D records AFImage dimensions in a different reference
    // frame. Use Canon's image dimensions for this model only, not later 5Ds.
    let original_5d = canon
        && tags.iter().any(|tag| {
            tag.name == "Model"
                && tag.group.family1 == "IFD0"
                && tag.print_value.trim().eq_ignore_ascii_case("Canon EOS 5D")
        });
    let (width_tag, height_tag) = if original_5d {
        ("CanonImageWidth", "CanonImageHeight")
    } else {
        ("AFImageWidth", "AFImageHeight")
    };
    let Some(w) = scalar(width_tag).filter(|n| *n > 0.0) else {
        return vec![];
    };
    let Some(h) = scalar(height_tag).filter(|n| *n > 0.0) else {
        return vec![];
    };
    let orientation = tags
        .iter()
        .find(|tag| tag.name == "Orientation" && tag.group.family1 == "IFD0")
        .and_then(|tag| tag.raw_value.as_u64())
        .filter(|value| (1..=8).contains(value))
        .unwrap_or(1) as u8;
    let mut regions = vec![];
    // Nikon contrast/hybrid AF: absolute sensor coordinates with explicit result.
    if text("AFCoordinatesAvailable").as_deref() == Some("Yes") {
        let status = text("FocusResult").and_then(|s| match s.as_str() {
            "Focus" | "In Focus" => Some(true),
            "Out of Focus" | "Not in Focus" => Some(false),
            _ => None,
        });
        if let (Some(x), Some(y), Some(rw), Some(rh)) = (
            scalar("AFAreaXPosition"),
            scalar("AFAreaYPosition"),
            scalar("AFAreaWidth"),
            scalar("AFAreaHeight"),
        ) {
            if let Some(r) = region(
                (x - rw / 2.0) / w,
                (y - rh / 2.0) / h,
                rw / w,
                rh / h,
                status,
                orientation,
            ) {
                regions.push(r);
            }
        }
    }
    // Canon centers are relative to the sensor center. Compact cameras have
    // downward-positive Y coordinates, while EOS cameras use upward-positive Y.
    let y_direction =
        if canon && text("CameraType").is_some_and(|value| value.eq_ignore_ascii_case("Compact")) {
            1.0
        } else {
            -1.0
        };
    let xs = numbers("AFAreaXPositions");
    let ys = numbers("AFAreaYPositions");
    let widths = numbers("AFAreaWidths");
    let heights = numbers("AFAreaHeights");
    let indices = |name: &str| -> Option<Vec<usize>> {
        let value = text(name)?;
        if value == "(none)" || value == "None" {
            return Some(vec![]);
        }
        value
            .split(|c: char| c.is_whitespace() || c == ',')
            .filter(|s| !s.is_empty())
            .map(|s| s.parse().ok())
            .collect()
    };
    let focused = indices("AFPointsInFocus").unwrap_or_default();
    let selected = indices("AFPointsSelected").unwrap_or_default();
    for (i, (&x, &y)) in xs.iter().zip(&ys).enumerate().take(4096) {
        if !focused.contains(&i) && !selected.contains(&i) {
            continue;
        }
        let rw = widths.get(i).copied().or_else(|| scalar("AFAreaWidth"));
        let rh = heights.get(i).copied().or_else(|| scalar("AFAreaHeight"));
        if let (Some(rw), Some(rh)) = (rw, rh) {
            if let Some(r) = region(
                0.5 + (x - rw / 2.0) / w,
                0.5 + (y * y_direction - rh / 2.0) / h,
                rw / w,
                rh / h,
                focused.contains(&i).then_some(true),
                orientation,
            ) {
                regions.push(r);
            }
        }
    }
    regions
}

fn region(
    x: f64,
    y: f64,
    w: f64,
    h: f64,
    focused: Option<bool>,
    orientation: u8,
) -> Option<AfArea> {
    if ![x, y, w, h].iter().all(|n| n.is_finite())
        || w <= 0.0
        || h <= 0.0
        || x < 0.0
        || y < 0.0
        || x + w > 1.001
        || y + h > 1.001
    {
        return None;
    }
    let (x, y, w, h) = match orientation {
        2 => (1.0 - x - w, y, w, h),
        3 => (1.0 - x - w, 1.0 - y - h, w, h),
        4 => (x, 1.0 - y - h, w, h),
        5 => (y, x, h, w),
        6 => (1.0 - y - h, x, h, w),
        7 => (1.0 - y - h, 1.0 - x - w, h, w),
        8 => (y, 1.0 - x - w, h, w),
        _ => (x, y, w, h),
    };
    Some(AfArea {
        x,
        y,
        width: w,
        height: h,
        focused,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn validates_and_rotates() {
        assert!(region(f64::NAN, 0.0, 0.1, 0.1, Some(true), 1).is_none());
        assert!(region(0.9, 0.0, 0.2, 0.1, Some(true), 1).is_none());
        let r = region(0.1, 0.2, 0.3, 0.4, Some(false), 6).unwrap();
        assert!((r.x - 0.4).abs() < 1e-10 && (r.y - 0.1).abs() < 1e-10);
        assert_eq!(r.focused, Some(false));
    }
    #[test]
    fn absent_metadata_is_empty() {
        assert!(from_tags(&[]).is_empty());
    }
    fn tag(name: &str, value: &str) -> Tag {
        Tag {
            id: exiftool_rs::tag::TagId::Text(name.into()),
            name: name.into(),
            description: name.into(),
            group: exiftool_rs::tag::TagGroup {
                family0: "MakerNotes".into(),
                ..Default::default()
            },
            raw_value: exiftool_rs::Value::String(value.into()),
            print_value: value.into(),
            priority: 0,
        }
    }
    #[test]
    fn canon_only_marks_selected_or_confirmed_points() {
        let mut tags = vec![
            tag("AFImageWidth", "1000"),
            tag("AFImageHeight", "800"),
            tag("AFAreaXPositions", "-100 0 100"),
            tag("AFAreaYPositions", "0 100 0"),
            tag("AFAreaWidth", "50"),
            tag("AFAreaHeight", "40"),
            tag("AFPointsSelected", "0,1"),
            tag("AFPointsInFocus", "1"),
        ];
        let r = from_tags(&tags);
        assert_eq!(r.len(), 2);
        assert_eq!(r[0].focused, None);
        assert_eq!(r[1].focused, Some(true));
        assert!((r[1].y - 0.35).abs() < 1e-10);
        tags.pop(); // A selected point alone does not prove failure.
        let selected = from_tags(&tags);
        assert_eq!(selected.len(), 2);
        assert!(selected.iter().all(|area| area.focused.is_none()));
    }
    fn camera_tag(name: &str, value: &str) -> Tag {
        let mut tag = tag(name, value);
        tag.group.family0 = "EXIF".into();
        tag.group.family1 = "IFD0".into();
        tag
    }

    #[test]
    fn canon_compact_y_axis_differs_from_eos_without_affecting_other_makers() {
        let mut tags = vec![
            camera_tag("Make", "Canon"),
            tag("CameraType", "Compact"),
            tag("AFImageWidth", "4352"),
            tag("AFImageHeight", "2904"),
            tag("AFAreaXPositions", "163"),
            tag("AFAreaYPositions", "-489"),
            tag("AFAreaWidths", "783"),
            tag("AFAreaHeights", "590"),
            tag("AFPointsInFocus", "0"),
        ];
        let compact = from_tags(&tags);
        assert_eq!(compact.len(), 1);
        assert!((compact[0].y - (0.5 + (-489.0 - 295.0) / 2904.0)).abs() < 1e-10);
        assert_eq!(compact[0].focused, Some(true));
        tags[1].print_value = "EOS High-end".into();
        let eos = from_tags(&tags);
        assert!((eos[0].y - (0.5 + (489.0 - 295.0) / 2904.0)).abs() < 1e-10);
        tags[0].print_value = "Other camera".into();
        tags[1].print_value = "Compact".into();
        assert!((from_tags(&tags)[0].y - eos[0].y).abs() < 1e-10);
    }

    #[test]
    fn only_original_canon_5d_uses_canon_image_dimensions() {
        let mut tags = vec![
            camera_tag("Make", "Canon"),
            camera_tag("Model", "Canon EOS 5D"),
            tag("AFImageWidth", "4992"),
            tag("AFImageHeight", "3328"),
            tag("CanonImageWidth", "4368"),
            tag("CanonImageHeight", "2912"),
            tag("AFAreaXPositions", "0"),
            tag("AFAreaYPositions", "0"),
            tag("AFAreaWidth", "119"),
            tag("AFAreaHeight", "119"),
            tag("AFPointsInFocus", "0"),
        ];
        let original = from_tags(&tags);
        assert_eq!(original.len(), 1);
        assert!((original[0].width - 119.0 / 4368.0).abs() < 1e-10);
        assert!((original[0].height - 119.0 / 2912.0).abs() < 1e-10);
        for model in [
            "Canon EOS 5D Mark II",
            "Canon EOS 5D Mark III",
            "Canon EOS 5D Mark IV",
        ] {
            tags[1].print_value = model.into();
            assert!((from_tags(&tags)[0].width - 119.0 / 4992.0).abs() < 1e-10);
        }
        tags[1].print_value = "Canon EOS 5D".into();
        tags[4].print_value = "0".into();
        assert!(from_tags(&tags).is_empty()); // Missing reference size must not produce a guessed box.
    }

    fn nikon_note() -> Vec<u8> {
        let mut data = b"Nikon\0\x02\x10\0\0II\x2a\0\x08\0\0\0".to_vec();
        data.extend_from_slice(&1u16.to_le_bytes());
        data.extend_from_slice(&0xb7u16.to_le_bytes());
        data.extend_from_slice(&7u16.to_le_bytes());
        data.extend_from_slice(&80u32.to_le_bytes());
        data.extend_from_slice(&26u32.to_le_bytes());
        data.extend_from_slice(&0u32.to_le_bytes());
        let start = data.len();
        data.resize(start + 80, 0);
        data[start..start + 4].copy_from_slice(b"0402");
        data[start + 7] = 1;
        for (p, v) in [
            (0x3e, 1000u16),
            (0x40, 800),
            (0x42, 500),
            (0x44, 400),
            (0x46, 100),
            (0x48, 80),
        ] {
            data[start + p..start + p + 2].copy_from_slice(&v.to_le_bytes());
        }
        data
    }
    #[test]
    fn nikon_binary_status_and_missing_data() {
        let mut data = nikon_note();
        let start = data.len() - 80;
        assert_eq!(nikon_maker_note(&data, 1).unwrap().focused, Some(false));
        data[start + 0x4a] = 1;
        assert_eq!(nikon_maker_note(&data, 1).unwrap().focused, Some(true));
        data[start + 0x4a] = 2;
        assert!(nikon_maker_note(&data, 1).is_none());
        data[start + 7] = 0;
        assert!(nikon_maker_note(&data, 1).is_none());
        for len in 0..data.len() {
            assert!(nikon_maker_note(&data[..len], 1).is_none());
        }
    }
    #[test]
    fn fallback_skips_other_cameras_and_known_unsupported_nikon_data() {
        assert!(!needs_nikon_fallback(&[tag("Make", "Canon")]));
        let mut tags = vec![
            tag("Make", "NIKON CORPORATION"),
            tag("AFInfo2Version", "0100"),
        ];
        assert!(!needs_nikon_fallback(&tags));
        tags[1].print_value = "0401".into();
        assert!(needs_nikon_fallback(&tags));
        tags.push(tag("AFCoordinatesAvailable", "No"));
        assert!(!needs_nikon_fallback(&tags));
    }

    fn key(path: &str, version: u64) -> CacheKey {
        CacheKey {
            path: path.into(),
            modified: SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(version),
            size: 100,
        }
    }

    #[test]
    fn cache_reuses_empty_results_and_invalidates_versions() {
        let mut cache = AreaCache::default();
        let original = cache.entry(key("photo.nef", 1));
        original.set(Ok(vec![])).unwrap();
        assert!(Arc::ptr_eq(&original, &cache.entry(key("photo.nef", 1))));
        let changed = cache.entry(key("photo.nef", 2));
        assert!(!Arc::ptr_eq(&original, &changed));
        assert!(changed.get().is_none());
        let mut resized = key("photo.nef", 2);
        resized.size = 101;
        assert!(!Arc::ptr_eq(&changed, &cache.entry(resized)));
        assert_eq!(cache.0.len(), 1);
        for i in 0..CACHE_CAPACITY + 1 {
            cache.entry(key(&format!("{i}.nef"), 1));
        }
        assert_eq!(cache.0.len(), CACHE_CAPACITY);
    }

    #[test]
    fn concurrent_panes_parse_the_same_file_once() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        let mut cache = AreaCache::default();
        let calls = AtomicUsize::new(0);
        let cell = cache.entry(key("photo.nef", 1));
        std::thread::scope(|scope| {
            for _ in 0..4 {
                let cell = cell.clone();
                let calls = &calls;
                scope.spawn(move || {
                    cell.get_or_init(|| {
                        calls.fetch_add(1, Ordering::SeqCst);
                        Ok(vec![])
                    });
                });
            }
        });
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn all_exif_orientations_keep_area_geometry_and_unknown_state() {
        let expected = [
            (0.1, 0.2, 0.3, 0.4),
            (0.6, 0.2, 0.3, 0.4),
            (0.6, 0.4, 0.3, 0.4),
            (0.1, 0.4, 0.3, 0.4),
            (0.2, 0.1, 0.4, 0.3),
            (0.4, 0.1, 0.4, 0.3),
            (0.4, 0.6, 0.4, 0.3),
            (0.2, 0.6, 0.4, 0.3),
        ];
        for (i, (x, y, w, h)) in expected.into_iter().enumerate() {
            let area = region(0.1, 0.2, 0.3, 0.4, None, (i + 1) as u8).unwrap();
            assert!((area.x - x).abs() < 1e-10);
            assert!((area.y - y).abs() < 1e-10);
            assert!((area.width - w).abs() < 1e-10);
            assert!((area.height - h).abs() < 1e-10);
            assert!(area.focused.is_none());
        }
    }

    #[test]
    fn nikon_coordinates_require_availability_and_do_not_invent_a_result() {
        let mut tags = vec![
            tag("AFImageWidth", "1000"),
            tag("AFImageHeight", "800"),
            tag("AFAreaXPosition", "500"),
            tag("AFAreaYPosition", "400"),
            tag("AFAreaWidth", "100"),
            tag("AFAreaHeight", "80"),
        ];
        assert!(from_tags(&tags).is_empty());
        tags.push(tag("AFCoordinatesAvailable", "Yes"));
        assert_eq!(from_tags(&tags).len(), 1);
        assert_eq!(from_tags(&tags)[0].focused, None);
        tags.push(tag("FocusResult", "Out of Focus"));
        assert_eq!(from_tags(&tags)[0].focused, Some(false));
        tags.last_mut().unwrap().print_value = "Focus".into();
        assert_eq!(from_tags(&tags)[0].focused, Some(true));
    }

    fn nikon_tiff() -> Vec<u8> {
        let mut note = nikon_note();
        let start = note.len() - 80;
        note[start + 0x4a] = 1;
        let mut data = b"II\x2a\0\x08\0\0\0".to_vec();
        data.extend_from_slice(&3u16.to_le_bytes());
        for (tag, kind, count, value) in [
            (0x10fu16, 2u16, 6u32, 50u32),
            (0x112, 3, 1, 6),
            (0x8769, 4, 1, 56),
        ] {
            data.extend_from_slice(&tag.to_le_bytes());
            data.extend_from_slice(&kind.to_le_bytes());
            data.extend_from_slice(&count.to_le_bytes());
            data.extend_from_slice(&value.to_le_bytes());
        }
        data.extend_from_slice(&0u32.to_le_bytes());
        data.extend_from_slice(b"NIKON\0");
        data.extend_from_slice(&1u16.to_le_bytes());
        data.extend_from_slice(&0x927cu16.to_le_bytes());
        data.extend_from_slice(&7u16.to_le_bytes());
        data.extend_from_slice(&(note.len() as u32).to_le_bytes());
        data.extend_from_slice(&74u32.to_le_bytes());
        data.extend_from_slice(&0u32.to_le_bytes());
        data.extend_from_slice(&note);
        data
    }

    #[test]
    fn reads_nikon_jpeg_and_raw_with_current_metadata_backend() {
        let tiff = nikon_tiff();
        let mut jpeg = vec![0xff, 0xd8, 0xff, 0xe1];
        jpeg.extend_from_slice(&((tiff.len() + 8) as u16).to_be_bytes());
        jpeg.extend_from_slice(b"Exif\0\0");
        jpeg.extend_from_slice(&tiff);
        jpeg.extend_from_slice(&[0xff, 0xd9]);
        for (suffix, bytes) in [("JPG", &jpeg), ("NEF", &tiff)] {
            let path =
                std::env::temp_dir().join(format!("lap-af-{}.{suffix}", uuid::Uuid::new_v4()));
            std::fs::write(&path, bytes).unwrap();
            let regions = read(path.to_str().unwrap());
            std::fs::remove_file(path).unwrap();
            assert_eq!(regions.len(), 1, "{suffix}");
            assert_eq!(regions[0].focused, Some(true));
            assert!((regions[0].x - 0.45).abs() < 1e-10);
        }
    }

    #[test]
    fn sample_directory_stress() {
        let Ok(dir) = std::env::var("LAP_AF_AREA_SAMPLE_DIR") else {
            return;
        };
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if !path.is_file() {
                continue;
            }
            eprintln!("AF Area stress: {}", path.display());
            let start = std::time::Instant::now();
            let areas = read(&path.to_string_lossy());
            eprintln!(
                "  {} areas, {:?}, {:?}",
                areas.len(),
                start.elapsed(),
                areas
            );
        }
    }
    #[test]
    fn sample_nikon() {
        let Ok(dir) = std::env::var("LAP_AF_AREA_SAMPLE_DIR") else {
            return;
        };
        for name in ["DSC_0028.JPG", "DSC_0028.NEF"] {
            let regions = read(&std::path::Path::new(&dir).join(name).to_string_lossy());
            assert_eq!(regions.len(), 1, "{name}");
            assert_eq!(regions[0].focused, Some(true), "{name}");
            assert!((regions[0].x - (4461.0 - 170.0) / 6048.0).abs() < 1e-6);
        }
    }
}

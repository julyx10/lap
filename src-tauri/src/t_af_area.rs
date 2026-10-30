//! Read-only, on-demand MakerNote AF areas. Unknown status is never failure.
use exiftool_rs::{ExifTool, Tag};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct AfArea {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub focused: bool,
}

#[tauri::command]
pub async fn get_af_areas(file_path: String) -> Result<Vec<AfArea>, String> {
    tauri::async_runtime::spawn_blocking(move || read(&file_path))
        .await
        .map_err(|e| e.to_string())
}

fn read(path: &str) -> Vec<AfArea> {
    let Ok(info) = std::fs::metadata(path) else {
        return vec![];
    };
    if !info.is_file() || crate::t_utils::is_cloud_only(&info) {
        return vec![];
    }
    let Ok(tags) = ExifTool::new().extract_info(path) else {
        return vec![];
    };
    let regions = from_tags(&tags);
    if !regions.is_empty() {
        return regions;
    }
    // exiftool-rs does not yet decode modern Nikon AFInfo2 payloads.
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
        return vec![];
    };
    let Some(field) = exif.get_field(exif::Tag::MakerNote, exif::In::PRIMARY) else {
        return vec![];
    };
    let exif::Value::Undefined(data, _) = &field.value else {
        return vec![];
    };
    let orientation = exif
        .get_field(exif::Tag::Orientation, exif::In::PRIMARY)
        .and_then(|f| f.value.get_uint(0))
        .unwrap_or(1) as u8;
    nikon_maker_note(data, orientation).into_iter().collect()
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
            focused,
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
    let Some(w) = scalar("AFImageWidth").filter(|n| *n > 0.0) else {
        return vec![];
    };
    let Some(h) = scalar("AFImageHeight").filter(|n| *n > 0.0) else {
        return vec![];
    };
    let orientation = tags
        .iter()
        .find(|t| t.name == "Orientation" && t.group.family1 == "IFD0")
        .and_then(|t| match t.print_value.clone().as_str() {
            "Mirror horizontal" => Some(2),
            "Rotate 180" => Some(3),
            "Mirror vertical" => Some(4),
            "Mirror horizontal and rotate 270 CW" => Some(5),
            "Rotate 90 CW" => Some(6),
            "Mirror horizontal and rotate 90 CW" => Some(7),
            "Rotate 270 CW" => Some(8),
            _ => None,
        })
        .unwrap_or(1);
    let mut regions = vec![];
    // Nikon contrast/hybrid AF: absolute sensor coordinates with explicit result.
    if text("AFCoordinatesAvailable").as_deref() != Some("No") {
        let status = text("FocusResult").and_then(|s| match s.as_str() {
            "Focus" | "In Focus" => Some(true),
            "Out of Focus" | "Not in Focus" => Some(false),
            _ => None,
        });
        if let (Some(x), Some(y), Some(rw), Some(rh), Some(ok)) = (
            scalar("AFAreaXPosition"),
            scalar("AFAreaYPosition"),
            scalar("AFAreaWidth"),
            scalar("AFAreaHeight"),
            status,
        ) {
            if let Some(r) = region(
                (x - rw / 2.0) / w,
                (y - rh / 2.0) / h,
                rw / w,
                rh / h,
                ok,
                orientation,
            ) {
                regions.push(r);
            }
        }
    }
    // Canon AFInfo/AFInfo2: centers are relative to the sensor center, Y upwards.
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
    if let Some(focused) = indices("AFPointsInFocus") {
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
                    0.5 - (y + rh / 2.0) / h,
                    rw / w,
                    rh / h,
                    focused.contains(&i),
                    orientation,
                ) {
                    regions.push(r);
                }
            }
        }
    }
    regions
}

fn region(x: f64, y: f64, w: f64, h: f64, focused: bool, orientation: u8) -> Option<AfArea> {
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
        assert!(region(f64::NAN, 0.0, 0.1, 0.1, true, 1).is_none());
        assert!(region(0.9, 0.0, 0.2, 0.1, true, 1).is_none());
        let r = region(0.1, 0.2, 0.3, 0.4, false, 6).unwrap();
        assert!((r.x - 0.4).abs() < 1e-10 && (r.y - 0.1).abs() < 1e-10);
        assert!(!r.focused);
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
        assert!(!r[0].focused && r[1].focused);
        assert!((r[1].y - 0.35).abs() < 1e-10);
        tags.pop(); // A selected point alone does not prove failure.
        assert!(from_tags(&tags).is_empty());
    }
    #[test]
    fn nikon_binary_status_and_missing_data() {
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
        assert!(!nikon_maker_note(&data, 1).unwrap().focused);
        data[start + 0x4a] = 1;
        assert!(nikon_maker_note(&data, 1).unwrap().focused);
        data[start + 0x4a] = 2;
        assert!(nikon_maker_note(&data, 1).is_none());
        data[start + 7] = 0;
        assert!(nikon_maker_note(&data, 1).is_none());
        for len in 0..data.len() {
            assert!(nikon_maker_note(&data[..len], 1).is_none());
        }
    }
    #[test]
    fn sample_directory_stress() {
        let Ok(dir) = std::env::var("LAP_AF_AREA_SAMPLE_DIR") else { return; };
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if !path.is_file() { continue; }
            eprintln!("AF Area stress: {}", path.display());
            let _ = read(&path.to_string_lossy());
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
            assert!(regions[0].focused, "{name}");
            assert!((regions[0].x - (4461.0 - 170.0) / 6048.0).abs() < 1e-6);
        }
    }
}

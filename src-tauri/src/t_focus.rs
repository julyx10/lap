//! Nikon AFInfo2 coordinates in unrotated AF image space.
use serde::Serialize;
use std::{io::BufReader, path::Path};

#[derive(Debug, Serialize)]
pub struct FocusRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    #[serde(skip)]
    af_dimensions: (usize, usize),
}

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FocusResult {
    OutOfFocus,
    Focus,
}

#[derive(Debug, Serialize)]
pub struct FocusInfo {
    pub focus: Option<FocusRect>,
    pub focus_result: Option<FocusResult>,
    pub orientation: u32,
}

#[derive(Default)]
struct NikonFocus {
    focus: Option<FocusRect>,
    focus_result: Option<FocusResult>,
}

fn u16_at(data: &[u8], pos: usize, little: bool) -> Option<u16> {
    let bytes: [u8; 2] = data.get(pos..pos.checked_add(2)?)?.try_into().ok()?;
    Some(if little {
        u16::from_le_bytes(bytes)
    } else {
        u16::from_be_bytes(bytes)
    })
}
fn u32_at(data: &[u8], pos: usize, little: bool) -> Option<u32> {
    let bytes: [u8; 4] = data.get(pos..pos.checked_add(4)?)?.try_into().ok()?;
    Some(if little {
        u32::from_le_bytes(bytes)
    } else {
        u32::from_be_bytes(bytes)
    })
}

// Nikon type 3 MakerNote: offsets in its IFD are relative to its TIFF header.
fn nikon_focus(note: &[u8]) -> Option<NikonFocus> {
    if !note.starts_with(b"Nikon\0\x02") {
        return None;
    }
    let tiff = note.get(10..)?;
    let little = match tiff.get(..2)? {
        b"II" => true,
        b"MM" => false,
        _ => return None,
    };
    if u16_at(tiff, 2, little)? != 42 {
        return None;
    }
    let ifd = u32_at(tiff, 4, little)? as usize;
    let count = u16_at(tiff, ifd, little)? as usize;
    for i in 0..count {
        let entry = ifd.checked_add(2)?.checked_add(i.checked_mul(12)?)?;
        if u16_at(tiff, entry, little)? != 0x00b7 {
            continue;
        }
        // AFInfo2 is an UNDEFINED byte array.
        if u16_at(tiff, entry + 2, little)? != 7 {
            return None;
        }
        let len = u32_at(tiff, entry + 4, little)? as usize;
        let offset = u32_at(tiff, entry + 8, little)? as usize;
        let block = tiff.get(offset..offset.checked_add(len)?)?;
        return decode_af_info2(block, little);
    }
    None
}

fn decode_af_info2(block: &[u8], little: bool) -> Option<NikonFocus> {
    // Version 04xx: Nikon Z Expeed 7; other layouts are deliberately excluded.
    if !matches!(block.get(..4)?, b"0400" | b"0401" | b"0402") {
        return None;
    }
    let focus_result = match block.get(0x4a) {
        Some(0) => Some(FocusResult::OutOfFocus),
        Some(1) => Some(FocusResult::Focus),
        _ => None,
    };
    Some(NikonFocus {
        focus: decode_coordinates(block, little),
        focus_result,
    })
}

fn decode_coordinates(block: &[u8], little: bool) -> Option<FocusRect> {
    if *block.get(7)? != 1 {
        return None;
    }
    let iw = u16_at(block, 0x3e, little)? as f64;
    let ih = u16_at(block, 0x40, little)? as f64;
    let x = u16_at(block, 0x42, little)? as f64;
    let y = u16_at(block, 0x44, little)? as f64;
    let w = u16_at(block, 0x46, little)? as f64;
    let h = u16_at(block, 0x48, little)? as f64;
    // (0,0) is Nikon's no-target sentinel in auto-area / tracking modes.
    if iw == 0.0
        || ih == 0.0
        || w == 0.0
        || h == 0.0
        || (x == 0.0 && y == 0.0)
        || x > iw
        || y > ih
        || w > iw
        || h > ih
    {
        return None;
    }
    Some(FocusRect {
        x: x / iw,
        y: y / ih,
        width: w / iw,
        height: h / ih,
        af_dimensions: (iw as usize, ih as usize),
    })
}

fn jpeg_dimensions_match(rect: &FocusRect, dimensions: Option<(usize, usize)>) -> bool {
    dimensions == Some(rect.af_dimensions)
}

// Match only an unambiguous same-folder, same-stem RAW; never hydrate cloud files.
fn paired_raw(path: &Path) -> Option<std::path::PathBuf> {
    let stem = path.file_stem()?.to_str()?;
    let mut matches = std::fs::read_dir(path.parent()?).ok()?.filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|candidate| {
            candidate.file_stem().and_then(|s| s.to_str())
                .is_some_and(|s| s.eq_ignore_ascii_case(stem))
                && candidate.extension().and_then(|s| s.to_str())
                    .is_some_and(|ext| crate::t_common::RAW_IMGS.iter().any(|raw| ext.eq_ignore_ascii_case(raw)))
        });
    let candidate = matches.next()?;
    if matches.next().is_some() { return None; }
    let meta = std::fs::metadata(&candidate).ok()?;
    (meta.is_file() && !crate::t_utils::is_cloud_only(&meta)).then_some(candidate)
}

pub fn read_focus(path: &Path) -> Result<FocusInfo, String> {
    let mut info = read_focus_file(path)?;
    let is_jpeg = path.extension().and_then(|s| s.to_str())
        .is_some_and(|ext| ["jpg", "jpeg", "jfif"].iter().any(|e| ext.eq_ignore_ascii_case(e)));
    if is_jpeg && (info.focus.is_none() || info.focus_result.is_none()) {
        if let Some(raw) = paired_raw(path) {
            if let Ok(raw_info) = read_focus_file(&raw) {
                let dimensions = imagesize::size(path).ok().map(|s| (s.width, s.height));
                // Keep the displayed JPEG's orientation and crop safety check.
                if let Some(rect) = raw_info.focus.filter(|r| jpeg_dimensions_match(r, dimensions)) {
                    if info.focus.is_none() {
                        info.focus = Some(rect);
                        info.focus_result = raw_info.focus_result;
                    } else if info.focus_result.is_none() {
                        info.focus_result = raw_info.focus_result;
                    }
                }
            }
        }
    }
    Ok(info)
}

fn read_focus_file(path: &Path) -> Result<FocusInfo, String> {
    let mut file = std::fs::File::open(path).map_err(|e| e.to_string())?;
    use std::io::{Read, Seek};
    let mut signature = [0u8; 2];
    let is_jpeg = file.read_exact(&mut signature).is_ok() && signature == [0xff, 0xd8];
    file.rewind().map_err(|e| e.to_string())?;
    let reader = exif::Reader::new().read_from_container(&mut BufReader::new(file));
    let Ok(exif) = reader else {
        return Ok(FocusInfo {
            focus: None,
            focus_result: None,
            orientation: 1,
        });
    };
    let orientation = exif
        .get_field(exif::Tag::Orientation, exif::In::PRIMARY)
        .and_then(|f| f.value.get_uint(0))
        .filter(|v| (1..=8).contains(v))
        .unwrap_or(1);
    let NikonFocus {
        focus,
        focus_result,
    } = exif
        .get_field(exif::Tag::MakerNote, exif::In::PRIMARY)
        .and_then(|f| match &f.value {
            exif::Value::Undefined(bytes, _) => nikon_focus(bytes),
            _ => None,
        })
        .unwrap_or_default();
    // JPEG raster mismatch may indicate a crop or edit; suppress the overlay.
    let focus = if is_jpeg {
        let dimensions = imagesize::size(path).ok().map(|s| (s.width, s.height));
        focus.filter(|rect| jpeg_dimensions_match(rect, dimensions))
    } else {
        focus
    };
    Ok(FocusInfo {
        focus,
        focus_result,
        orientation,
    })
}

#[tauri::command]
pub async fn get_focus_info(path: String) -> Result<FocusInfo, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let path = Path::new(&path);
        let meta = std::fs::metadata(path).map_err(|e| e.to_string())?;
        if crate::t_utils::is_cloud_only(&meta) {
            return Err("File is not available locally".into());
        }
        read_focus(path)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn af_info_coordinates_and_validation() {
        let mut block = vec![0; 74];
        block[..4].copy_from_slice(b"0402");
        block[7] = 1;
        for (offset, value) in [
            (62, 6048u16),
            (64, 4032),
            (66, 4461),
            (68, 2404),
            (70, 340),
            (72, 342),
        ] {
            block[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
        }
        let parsed = decode_af_info2(&block, true).unwrap();
        assert_eq!(parsed.focus_result, None); // truncated before the result byte
        let rect = parsed.focus.unwrap();
        assert_eq!(rect.x, 4461.0 / 6048.0);
        assert_eq!(rect.height, 342.0 / 4032.0);
        assert!(jpeg_dimensions_match(&rect, Some((6048, 4032))));
        assert!(!jpeg_dimensions_match(&rect, Some((5000, 4032))));
        assert!(!jpeg_dimensions_match(&rect, None));
        block[7] = 0;
        assert!(decode_coordinates(&block, true).is_none());
        block[7] = 1;
        block[..4].copy_from_slice(b"9999");
        assert!(decode_af_info2(&block, true).is_none());
        assert!(decode_coordinates(&block[..20], true).is_none());
    }
    #[test]
    fn malformed_and_big_endian_blocks() {
        assert!(nikon_focus(b"Nikon\0\x02").is_none());
        let mut block = vec![0; 74];
        block[..4].copy_from_slice(b"0400");
        block[7] = 1;
        for (offset, value) in [
            (62, 1000u16),
            (64, 800),
            (66, 500),
            (68, 400),
            (70, 100),
            (72, 80),
        ] {
            block[offset..offset + 2].copy_from_slice(&value.to_be_bytes());
        }
        let r = decode_coordinates(&block, false).unwrap();
        assert_eq!((r.x, r.y, r.width, r.height), (0.5, 0.5, 0.1, 0.1));
        block[66..68].copy_from_slice(&1001u16.to_be_bytes());
        assert!(decode_coordinates(&block, false).is_none());
        block[66..68].copy_from_slice(&0u16.to_be_bytes());
        block[68..70].copy_from_slice(&0u16.to_be_bytes());
        assert!(decode_coordinates(&block, false).is_none());
    }
    #[test]
    fn result_is_independent_of_coordinates_and_handles_unknowns() {
        let mut block = vec![0; 75];
        block[..4].copy_from_slice(b"0402");
        block[0x4a] = 1;
        let parsed = decode_af_info2(&block, true).unwrap();
        assert!(parsed.focus.is_none());
        assert_eq!(parsed.focus_result, Some(FocusResult::Focus));
        block[0x4a] = 0;
        assert_eq!(
            decode_af_info2(&block, true).unwrap().focus_result,
            Some(FocusResult::OutOfFocus)
        );
        block[0x4a] = 2;
        assert_eq!(decode_af_info2(&block, true).unwrap().focus_result, None);
        assert_eq!(
            decode_af_info2(&block[..74], true).unwrap().focus_result,
            None
        );
        block[..4].copy_from_slice(b"9999");
        assert!(decode_af_info2(&block, true).is_none());
        assert!(decode_af_info2(&block[..3], true).is_none());
        let json = serde_json::to_value(FocusInfo {
            focus: None,
            focus_result: None,
            orientation: 1,
        })
        .unwrap();
        assert_eq!(
            json,
            serde_json::json!({"focus": null, "focus_result": null, "orientation": 1})
        );
        assert_eq!(serde_json::to_value(FocusResult::Focus).unwrap(), "focus");
        assert_eq!(
            serde_json::to_value(FocusResult::OutOfFocus).unwrap(),
            "out_of_focus"
        );
    }
    #[test]
    fn jpeg_without_makernote_uses_raw_if_available() {
        let Ok(dir) = std::env::var("LAP_TEST_NIKON_DIR") else { return; };
        let temp = std::env::temp_dir().join(format!("lap-focus-{}-{}", std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        std::fs::create_dir(&temp).unwrap();
        let jpeg = temp.join("dsc_0028.jpg");
        let raw = temp.join("DSC_0028.NEF");
        std::fs::copy(Path::new(&dir).join("DSC_0028.NEF"), &raw).unwrap();
        // A metadata-free JPEG models exported/stripped companions.
        image::RgbImage::new(6048, 4032).save(&jpeg).unwrap();
        assert!(read_focus_file(&jpeg).unwrap().focus.is_none());
        let info = read_focus(&jpeg).unwrap();
        assert_eq!(info.focus.unwrap().x, 4461.0 / 6048.0);
        assert_eq!(info.focus_result, Some(FocusResult::Focus));
        assert_eq!(info.orientation, 1);
        image::RgbImage::new(100, 100).save(&jpeg).unwrap();
        assert!(read_focus(&jpeg).unwrap().focus.is_none());
        std::fs::copy(&raw, temp.join("DSC_0028.dng")).unwrap();
        assert!(paired_raw(&jpeg).is_none());
        std::fs::remove_dir_all(&temp).unwrap();
    }

    #[test]
    fn nikon_sample_if_available() {
        let Ok(dir) = std::env::var("LAP_TEST_NIKON_DIR") else {
            return;
        };
        for (number, expected_result, x, y) in [
            ("0028", FocusResult::Focus, 4461.0, 2404.0),
            ("0291", FocusResult::OutOfFocus, 3990.0, 1866.0),
        ] {
            for ext in ["JPG", "NEF"] {
                let path = Path::new(&dir).join(format!("DSC_{number}.{ext}"));
                let info = read_focus(&path).unwrap();
                assert_eq!(info.orientation, 1);
                assert_eq!(
                    info.focus_result.as_ref(),
                    Some(&expected_result),
                    "{}",
                    path.display()
                );
                let focus = info
                    .focus
                    .expect("Nikon sample should contain AF coordinates");
                assert_eq!(focus.x, x / 6048.0);
                assert_eq!(focus.y, y / 4032.0);
                let (width, height) = if number == "0028" {
                    (340.0, 342.0)
                } else {
                    (538.0, 534.0)
                };
                assert_eq!(focus.width, width / 6048.0);
                assert_eq!(focus.height, height / 4032.0);
            }
        }
    }
}

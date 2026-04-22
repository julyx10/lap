/**
 * Image processing utilities.
 * project: Lap
 * author:  julyx10
 * date:    2024-08-08
 */
use arboard::Clipboard;
use exif::{In, Reader, Tag};
use fast_image_resize as fir;
use image::{DynamicImage, GenericImageView, ImageReader, RgbImage};
use little_exif::filetype::FileExtension;
use little_exif::ifd::ExifTagGroup;
use little_exif::metadata::Metadata as LittleExifMetadata;
use once_cell::sync::Lazy;

use rusqlite::Result;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::fs;
use std::fs::File;
use std::io::{BufReader, Cursor, Read};
use std::panic::{self, AssertUnwindSafe};
use std::path::{Path, PathBuf};
#[cfg(target_os = "macos")]
use std::process::Command;
use std::sync::Mutex;
use std::time::UNIX_EPOCH;
use uuid::Uuid;
use walkdir::WalkDir;

use crate::{t_jxl, t_libraw, t_utils};
#[cfg(not(target_os = "macos"))]
use crate::t_video;

/// Quick probing of image dimensions without loading the entire file
pub fn get_image_dimensions(file_path: &str) -> Result<(u32, u32), String> {
    if t_jxl::is_jxl_path(file_path) {
        return t_jxl::get_jxl_dimensions(file_path);
    }

    // Catch potential panics in the third-party imagesize crate
    let result = panic::catch_unwind(|| imagesize::size(file_path));

    match result {
        Ok(Ok(dimensions)) => {
            let width = dimensions.width as u32;
            let height = dimensions.height as u32;

            if crate::t_libraw::is_tiff_path(file_path) {
                if let Ok((raw_width, raw_height)) = crate::t_libraw::get_raw_dimensions(file_path)
                {
                    if raw_width > width || raw_height > height {
                        return Ok((raw_width, raw_height));
                    }
                }
            }

            Ok((width, height))
        }
        Ok(Err(e)) => Err(e.to_string()),
        Err(_) => {
            eprintln!("Panic caught while getting dimensions for: {}", file_path);
            Err(
                "Failed to parse image dimensions due to panic (corrupt or invalid file)"
                    .to_string(),
            )
        }
    }
}

fn get_raw_dimensions_from_exif(file_path: &str) -> Result<Option<(u32, u32)>, String> {
    let exif = match read_exif_permissive(file_path) {
        Some(exif) => exif,
        None => return Ok(None),
    };

    let dimension_tag_pairs = [
        (Tag::PixelXDimension, Tag::PixelYDimension),
        (Tag::ImageWidth, Tag::ImageLength),
    ];

    for (width_tag, height_tag) in dimension_tag_pairs {
        let width = exif
            .get_field(width_tag, In::PRIMARY)
            .and_then(|field| field.value.get_uint(0));
        let height = exif
            .get_field(height_tag, In::PRIMARY)
            .and_then(|field| field.value.get_uint(0));

        if let (Some(width), Some(height)) = (width, height) {
            if width > 0 && height > 0 {
                return Ok(Some((width, height)));
            }
        }
    }

    Ok(None)
}

pub fn read_exif_from_bytes_permissive(data: &[u8]) -> Option<exif::Exif> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let mut cursor = Cursor::new(data);
        if let Ok(exif) = Reader::new().read_from_container(&mut cursor) {
            return Some(exif);
        }
        if let Some(pos) = data.windows(6).position(|w| w == b"Exif\0\0") {
            let exif_start = pos + 6;
            if exif_start < data.len() {
                if let Ok(exif) = Reader::new().read_raw(data[exif_start..].to_vec()) {
                    return Some(exif);
                }
            }
        }
        for sig in [b"II\x2a\x00", b"MM\x00\x2a"] {
            if let Some(pos) = data.windows(4).position(|w| w == sig) {
                if let Ok(exif) = Reader::new().read_raw(data[pos..].to_vec()) {
                    return Some(exif);
                }
            }
        }
        None
    }))
    .unwrap_or_else(|_| None)
}

/// A very aggressive binary scanner that looks for the EXIF Orientation tag (0x0112)
/// directly in the byte stream. This is used as a final fallback for non-standard devices.
pub fn scan_orientation_binary(data: &[u8]) -> Option<i32> {
    // Orientation tag is 0x0112. In TIFF, it's a Short (3) with Count 1.
    // Little Endian: 12 01 03 00 01 00 00 00 [Value] 00 00 00
    // Big Endian:    01 12 00 03 00 00 00 01 00 [Value] 00 00
    
    // Little Endian search
    if let Some(pos) = data.windows(12).position(|w| {
        w[0..8] == [0x12, 0x01, 0x03, 0x00, 0x01, 0x00, 0x00, 0x00]
    }) {
        let val = data[pos + 8] as i32;
        if (1..=8).contains(&val) { return Some(val); }
    }
    
    // Big Endian search
    if let Some(pos) = data.windows(12).position(|w| {
        w[0..8] == [0x01, 0x12, 0x00, 0x03, 0x00, 0x00, 0x00, 0x01]
    }) {
        let val = data[pos + 9] as i32;
        if (1..=8).contains(&val) { return Some(val); }
    }
    
    None
}

pub fn read_exif_permissive(file_path: &str) -> Option<exif::Exif> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        use std::io::{Read, Seek};
        let mut file = File::open(file_path).ok()?;
        let mut header = [0u8; 2];
        file.read_exact(&mut header).ok()?;
        
        if header != [0xFF, 0xD8] { return None; }

        loop {
            let mut marker = [0u8; 2];
            if file.read_exact(&mut marker).is_err() { break; }
            if marker[0] != 0xFF { break; }
            if marker[1] == 0xD9 || marker[1] == 0xDA { break; } 

            let mut len_bytes = [0u8; 2];
            if file.read_exact(&mut len_bytes).is_err() { break; }
            let len = u16::from_be_bytes(len_bytes) as usize;
            if len < 2 { break; }
            let segment_len = len - 2;
            
            if marker[1] == 0xE1 { // APP1
                let mut segment_data = vec![0u8; segment_len];
                if file.read_exact(&mut segment_data).is_err() { break; }
                if segment_data.starts_with(b"Exif\0\0") {
                    if let Ok(exif) = Reader::new().read_raw(segment_data[6..].to_vec()) {
                        return Some(exif);
                    }
                }
            } else {
                if file.seek(std::io::SeekFrom::Current(segment_len as i64)).is_err() { break; }
            }
        }
        None
    }))
    .unwrap_or_else(|_| None)
    .or_else(|| {
        let mut file = File::open(file_path).ok()?;
        let mut buffer = vec![0u8; 128 * 1024]; 
        let n = file.read(&mut buffer).unwrap_or(0);
        read_exif_from_bytes_permissive(&buffer[..n])
    })
}

pub fn get_image_orientation(file_path: &str) -> i32 {
    let data = match File::open(file_path) {
        Ok(mut f) => {
            let mut buf = vec![0u8; 128 * 1024];
            let n = f.read(&mut buf).unwrap_or(0);
            buf.truncate(n);
            buf
        }
        Err(_) => return 1,
    };

    // 1. Try modern logic
    if let Some(exif) = read_exif_from_bytes_permissive(&data) {
        let orient = exif.get_field(Tag::Orientation, In::PRIMARY)
            .or_else(|| exif.fields().find(|f| f.tag == Tag::Orientation))
            .and_then(|field| field.value.get_uint(0))
            .map(|value| value as i32);
        
        if let Some(o) = orient { return o; }
    }

    // 2. Industry Fallback: Binary Scan
    // This handles K800i and other phones with broken IFD chains
    scan_orientation_binary(&data).unwrap_or(1)
}

fn apply_orientation(img: DynamicImage, orientation: i32) -> DynamicImage {
    match orientation {
        2 => img.fliph(),
        3 => img.rotate180(),
        4 => img.flipv(),
        5 => img.rotate90().fliph(),
        6 => img.rotate90(),
        7 => img.rotate270().fliph(),
        8 => img.rotate270(),
        _ => img,
    }
}

fn compute_thumbnail_dimensions(width: u32, height: u32, thumbnail_size: u32) -> (u32, u32) {
    if width == 0 || height == 0 || thumbnail_size == 0 {
        return (1, 1);
    }

    if width <= thumbnail_size && height <= thumbnail_size {
        return (width.max(1), height.max(1));
    }

    let max_edge = width.max(height) as f32;
    let scale = thumbnail_size as f32 / max_edge;
    let dst_w = ((width as f32) * scale).round().max(1.0) as u32;
    let dst_h = ((height as f32) * scale).round().max(1.0) as u32;
    (dst_w, dst_h)
}

fn encode_jpeg_rgb8(rgb: &image::RgbImage) -> Result<Vec<u8>, String> {
    crate::t_jpeg::encode_rgb8(rgb, 85)
        .map_err(|e| format!("Failed to encode JPEG thumbnail: {}", e))
}

fn resize_rgb_image_to_jpeg(
    rgb: image::RgbImage,
    thumbnail_size: u32,
) -> Result<Vec<u8>, String> {
    let (src_w, src_h) = rgb.dimensions();
    let (dst_w, dst_h) = compute_thumbnail_dimensions(src_w, src_h, thumbnail_size);

    if src_w == dst_w && src_h == dst_h {
        return encode_jpeg_rgb8(&rgb);
    }

    let src_image = fir::images::Image::from_vec_u8(
        src_w,
        src_h,
        rgb.into_raw(),
        fir::PixelType::U8x3,
    )
    .map_err(|e| format!("Failed to prepare RGB source image for resize: {}", e))?;
    let mut dst_image = fir::images::Image::new(dst_w, dst_h, fir::PixelType::U8x3);
    let mut resizer = fir::Resizer::new();
    let options =
        fir::ResizeOptions::new().resize_alg(fir::ResizeAlg::Convolution(fir::FilterType::Bilinear));

    resizer
        .resize(&src_image, &mut dst_image, &options)
        .map_err(|e| format!("Failed to resize RGB thumbnail: {}", e))?;

    let resized = image::RgbImage::from_raw(dst_w, dst_h, dst_image.into_vec())
        .ok_or_else(|| "Failed to build resized RGB image".to_string())?;
    encode_jpeg_rgb8(&resized)
}

fn is_jpeg_path(file_path: &str) -> bool {
    Path::new(file_path)
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| matches!(ext.to_ascii_lowercase().as_str(), "jpg" | "jpeg" | "jpe"))
        .unwrap_or(false)
}


fn decode_scaled_jpeg_image(
    file_path: &str,
    _orientation: i32,
    thumbnail_size: u32,
) -> Result<Option<DynamicImage>, String> {
    if !is_jpeg_path(file_path) || thumbnail_size == 0 {
        return Ok(None);
    }

    // Logic: We pass the thumbnail size to libjpeg-turbo, which picks the best 1/8, 1/4, 1/2 scale.
    match crate::t_jpeg::decode_rgb8_scaled(file_path, thumbnail_size, thumbnail_size) {
        Ok((pixels, w, h)) => {
            let img = RgbImage::from_raw(w, h, pixels)
                .ok_or_else(|| "Failed to build RGB image from turbo pixels".to_string())?;
            Ok(Some(DynamicImage::ImageRgb8(img)))
        }
        Err(e) => {
            eprintln!("libjpeg-turbo scaled decode failed for {}: {}", file_path, e);
            Ok(None) // Fallback to standard decode
        }
    }
}

pub(crate) fn resize_dynamic_image_to_jpeg(
    img: DynamicImage,
    orientation: i32,
    thumbnail_size: u32,
) -> Result<Vec<u8>, String> {
    let adjusted = apply_orientation(img, orientation);

    if !adjusted.color().has_alpha() {
        return resize_rgb_image_to_jpeg(adjusted.to_rgb8(), thumbnail_size);
    }

    let rgba = adjusted.to_rgba8();
    let (src_w, src_h) = rgba.dimensions();
    let (dst_w, dst_h) = compute_thumbnail_dimensions(src_w, src_h, thumbnail_size);

    if src_w == dst_w && src_h == dst_h {
        return encode_jpeg_rgb8(&DynamicImage::ImageRgba8(rgba).to_rgb8());
    }

    let src_image = fir::images::Image::from_vec_u8(src_w, src_h, rgba.into_raw(), fir::PixelType::U8x4)
        .map_err(|e| format!("Failed to prepare source image for resize: {}", e))?;
    let mut dst_image = fir::images::Image::new(dst_w, dst_h, fir::PixelType::U8x4);
    let mut resizer = fir::Resizer::new();
    let options =
        fir::ResizeOptions::new().resize_alg(fir::ResizeAlg::Convolution(fir::FilterType::Bilinear));

    resizer
        .resize(&src_image, &mut dst_image, &options)
        .map_err(|e| format!("Failed to resize thumbnail: {}", e))?;

    let resized = image::RgbaImage::from_raw(dst_w, dst_h, dst_image.into_vec())
        .ok_or_else(|| "Failed to build resized RGBA image".to_string())?;
    encode_jpeg_rgb8(&DynamicImage::ImageRgba8(resized).to_rgb8())
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchThumbnailStats {
    pub processed: usize,
    pub succeeded: usize,
    pub failed: usize,
}

pub fn generate_directory_thumbnails(
    dir_path: &str,
    output_dir: &str,
    thumbnail_size: u32,
) -> Result<BatchThumbnailStats, String> {
    let dir_root = Path::new(dir_path);
    let files: Vec<PathBuf> = WalkDir::new(dir_path)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_file())
        .map(|entry| entry.into_path())
        .collect();

    fs::create_dir_all(output_dir)
        .map_err(|e| format!("Failed to create thumbnail output directory: {}", e))?;

    let processed = files.len();
    let results: Vec<bool> = files
        .iter()
        .map(|path| {
            let path_str = path.to_string_lossy().to_string();
            let Some(file_type) = t_utils::get_file_type(&path_str) else {
                return false;
            };
            if file_type != 1 && file_type != 3 {
                return false;
            }

            let orientation = get_image_orientation(&path_str);
            let thumb = if file_type == 3 {
                get_raw_thumbnail(&path_str, orientation, thumbnail_size)
            } else {
                get_image_thumbnail(&path_str, orientation, thumbnail_size)
            };

            let Ok(Some(data)) = thumb else {
                return false;
            };

            let relative_path = path.strip_prefix(dir_root).ok().unwrap_or(path.as_path());
            let output_path = Path::new(output_dir).join(relative_path).with_extension("jpg");
            if let Some(parent) = output_path.parent() {
                if fs::create_dir_all(parent).is_err() {
                    return false;
                }
            }
            fs::write(output_path, data).is_ok()
        })
        .collect();

    let succeeded = results.iter().filter(|ok| **ok).count();
    Ok(BatchThumbnailStats {
        processed,
        succeeded,
        failed: processed.saturating_sub(succeeded),
    })
}

/// Get a thumbnail from an image file path
pub fn get_image_thumbnail(
    file_path: &str,
    orientation: i32,
    thumbnail_size: u32,
) -> Result<Option<Vec<u8>>, String> {
    if t_jxl::is_jxl_path(file_path) {
        return t_jxl::get_jxl_thumbnail(file_path, thumbnail_size);
    }

    if crate::t_libraw::is_tiff_path(file_path) {
        if let Ok(Some(data)) = crate::t_libraw::get_raw_thumbnail(file_path, thumbnail_size) {
            return Ok(Some(data));
        }
    }

    let result = panic::catch_unwind(|| {
        let img = if let Some(img) = decode_scaled_jpeg_image(file_path, orientation, thumbnail_size)? {
            img
        } else {
            let img_reader =
                ImageReader::open(file_path).map_err(|e| format!("Failed to open image: {}", e))?;

            match img_reader.decode() {
                Ok(img) => img,
                Err(e) => {
                    // Some formats/variants (notably AVIF) may fail to decode via `image` depending on
                    // the underlying codec support. On macOS, fall back to `sips` which supports
                    // more system formats and returns a JPEG directly.
                    #[cfg(target_os = "macos")]
                    if let Ok(Some(data)) = get_thumbnail_with_sips(file_path, thumbnail_size) {
                        return Ok(Some(data));
                    }
                    // On other platforms, fall back to the bundled FFmpeg sidecar when available.
                    // This is already used for HEIC/HEIF on non-macOS and tends to support more
                    // real-world AVIF variants than the pure-Rust decode path.
                    #[cfg(not(target_os = "macos"))]
                    {
                        if let Ok(Some(data)) =
                            crate::t_video::get_video_thumbnail_sync(file_path, thumbnail_size, None)
                        {
                            return Ok(Some(data));
                        }
                    }
                    return Err(format!("Failed to decode image: {}", e));
                }
            }
        };
        resize_dynamic_image_to_jpeg(img, orientation, thumbnail_size).map(Some)
    });

    match result {
        Ok(v) => v,
        Err(_) => {
            eprintln!(
                "Panic caught while creating image thumbnail for: {}",
                file_path
            );
            Ok(None)
        }
    }
}

#[derive(Debug)]
struct EmbeddedJpegCandidate {
    data: Vec<u8>,
    width: u32,
    height: u32,
    max_edge: u32,
}

fn collect_embedded_jpeg_candidates(file_path: &str) -> Result<Vec<EmbeddedJpegCandidate>, String> {
    let exif = match read_exif_permissive(file_path) {
        Some(exif) => exif,
        None => return Ok(Vec::new()),
    };

    let buf = exif.buf();
    let mut candidates: Vec<EmbeddedJpegCandidate> = Vec::new();

    // The parser caps IFD count at 8. Scan all possible IFDs for embedded JPEGs.
    for ifd_index in 0u16..8u16 {
        let ifd = In(ifd_index);
        let offset = exif
            .get_field(Tag::JPEGInterchangeFormat, ifd)
            .and_then(|field| field.value.get_uint(0))
            .map(|value| value as usize);
        let len = exif
            .get_field(Tag::JPEGInterchangeFormatLength, ifd)
            .and_then(|field| field.value.get_uint(0))
            .map(|value| value as usize);

        let (offset, len) = match (offset, len) {
            (Some(offset), Some(len)) if len > 4 => (offset, len),
            _ => continue,
        };

        let end = offset.saturating_add(len);
        if end > buf.len() {
            continue;
        }

        let candidate = &buf[offset..end];
        // Basic JPEG signature check to avoid selecting non-JPEG payloads.
        if !(candidate.starts_with(&[0xFF, 0xD8])) {
            continue;
        }

        let data = candidate.to_vec();
        let (width, height, max_edge) = match image::load_from_memory(&data) {
            Ok(image) => {
                let (width, height) = image.dimensions();
                (width, height, width.max(height))
            }
            Err(_) => continue,
        };

        if max_edge == 0 {
            continue;
        }

        candidates.push(EmbeddedJpegCandidate {
            data,
            width,
            height,
            max_edge,
        });
    }

    Ok(candidates)
}

fn select_embedded_jpeg_for_preview(file_path: &str) -> Result<Option<Vec<u8>>, String> {
    let candidates = collect_embedded_jpeg_candidates(file_path)?;
    let (raw_width, raw_height) = t_libraw::get_raw_dimensions(file_path)?;
    let mut selected: Option<EmbeddedJpegCandidate> = None;

    for candidate in candidates {
        let width_delta = candidate.width.abs_diff(raw_width);
        let height_delta = candidate.height.abs_diff(raw_height);
        let is_fullsize = width_delta.saturating_mul(100) <= raw_width.max(1)
            && height_delta.saturating_mul(100) <= raw_height.max(1);

        if !is_fullsize {
            continue;
        }

        match &selected {
            Some(best) if candidate.max_edge <= best.max_edge => {}
            _ => selected = Some(candidate),
        }
    }

    Ok(selected.map(|item| item.data))
}

fn select_embedded_jpeg_for_thumbnail(
    file_path: &str,
    thumbnail_size: u32,
) -> Result<Option<Vec<u8>>, String> {
    let candidates = collect_embedded_jpeg_candidates(file_path)?;
    if candidates.is_empty() {
        return Ok(None);
    }

    let mut best_not_smaller: Option<EmbeddedJpegCandidate> = None;
    let mut best_smaller: Option<EmbeddedJpegCandidate> = None;

    for candidate in candidates {
        if candidate.max_edge >= thumbnail_size {
            match &best_not_smaller {
                Some(best) if candidate.max_edge >= best.max_edge => {}
                _ => best_not_smaller = Some(candidate),
            }
        } else {
            match &best_smaller {
                Some(best) if candidate.max_edge <= best.max_edge => {}
                _ => best_smaller = Some(candidate),
            }
        }
    }

    Ok(best_not_smaller.or(best_smaller).map(|item| item.data))
}

fn get_jpeg_orientation_from_bytes(data: &[u8]) -> i32 {
    let exif = match read_exif_from_bytes_permissive(data) {
        Some(exif) => exif,
        None => return 1,
    };

    exif.get_field(Tag::Orientation, In::PRIMARY)
        .and_then(|field| field.value.get_uint(0))
        .map(|value| value as i32)
        .unwrap_or(1)
}

pub fn get_raw_preview_image(file_path: &str) -> Result<Option<Vec<u8>>, String> {
    // Primary: LibRaw handles extraction and rotation
    if let Ok(Some(data)) = t_libraw::get_raw_preview_image(file_path) {
        return Ok(Some(data));
    }

    // Fallback: EXIF-based embedded JPEG extraction
    if let Ok(Some(preview)) = select_embedded_jpeg_for_preview(file_path) {
        let image = image::load_from_memory(&preview)
            .map_err(|e| format!("Failed to decode embedded RAW preview: {}", e))?;
        let image = apply_orientation(image, get_jpeg_orientation_from_bytes(&preview));
        let buf = crate::t_jpeg::encode_rgb8(&image.to_rgb8(), 85)
            .map_err(|e| format!("Failed to encode embedded RAW preview: {}", e))?;
        return Ok(Some(buf));
    }

    #[cfg(target_os = "macos")]
    if let Ok(Some(data)) = get_thumbnail_with_sips(file_path, 4096) {
        return Ok(Some(data));
    }

    let orientation = get_image_orientation(file_path);

    // Final fallback for formats that can be decoded directly by `image`.
    if let Ok(Some(data)) = get_image_thumbnail(file_path, orientation, 4096) {
        return Ok(Some(data));
    }

    Ok(None)
}

pub fn get_raw_dimensions(file_path: &str) -> Result<(u32, u32), String> {
    let should_swap_dimensions = |orientation: i32, raw_flip: i32| -> bool {
        match orientation {
            5 | 6 | 7 | 8 => true,
            1 => matches!(raw_flip, 5 | 6),
            _ => false,
        }
    };

    if let Ok((mut width, mut height, raw_flip)) = t_libraw::get_raw_dimensions_with_flip(file_path)
    {
        let orientation = get_image_orientation(file_path);
        if should_swap_dimensions(orientation, raw_flip) {
            std::mem::swap(&mut width, &mut height);
        }
        if width > 0 && height > 0 {
            return Ok((width, height));
        }
    }

    if let Ok((width, height)) = get_image_dimensions(file_path) {
        if width > 0 && height > 0 {
            return Ok((width, height));
        }
    }

    if let Ok(Some((width, height))) = get_raw_dimensions_from_exif(file_path) {
        return Ok((width, height));
    }

    #[cfg(target_os = "macos")]
    if let Ok(Some((width, height))) = get_dimensions_with_sips(file_path) {
        return Ok((width, height));
    }

    if let Ok(Some(preview)) = select_embedded_jpeg_for_preview(file_path) {
        if let Ok(image) = image::load_from_memory(&preview) {
            return Ok(image.dimensions());
        }
    }

    Err("Failed to resolve RAW dimensions".to_string())
}

pub fn get_raw_thumbnail(
    file_path: &str,
    orientation: i32,
    thumbnail_size: u32,
) -> Result<Option<Vec<u8>>, String> {
    // Primary: LibRaw handles extraction and rotation
    if let Ok(Some(data)) = t_libraw::get_raw_thumbnail(file_path, thumbnail_size) {
        return Ok(Some(data));
    }

    // Fallback: EXIF-based embedded JPEG extraction
    if let Ok(Some(preview)) = select_embedded_jpeg_for_thumbnail(file_path, thumbnail_size) {
        let img = image::load_from_memory(&preview)
            .map_err(|e| format!("Failed to decode RAW preview image: {}", e))?;
        return resize_dynamic_image_to_jpeg(
            img,
            get_jpeg_orientation_from_bytes(&preview),
            thumbnail_size,
        )
        .map(Some);
    }

    #[cfg(target_os = "macos")]
    if let Ok(Some(data)) = get_thumbnail_with_sips(file_path, thumbnail_size) {
        return Ok(Some(data));
    }

    // Fallback for formats that can be decoded directly by `image`.
    get_image_thumbnail(file_path, orientation, thumbnail_size)
}

/// edit image impl

/// crop data
#[derive(Debug, Deserialize, Serialize)]
pub struct CropData {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

/// resize data
#[derive(Debug, Deserialize, Serialize)]
pub struct ResizeData {
    width: Option<u32>,
    height: Option<u32>,
}

/// edit params
#[derive(Debug, Deserialize, Serialize)]
pub struct EditParams {
    #[serde(rename = "sourceFilePath")]
    source_file_path: String,
    #[serde(rename = "destFilePath")]
    dest_file_path: String,
    #[serde(rename = "outputFormat")]
    output_format: String,
    orientation: i32, // exif orientation value
    #[serde(rename = "flipHorizontal")]
    flip_horizontal: bool,
    #[serde(rename = "flipVertical")]
    flip_vertical: bool,
    rotate: i32,
    crop: CropData,
    resize: ResizeData,
    quality: Option<u8>,
    // New adjustments
    filter: Option<String>,  // "grayscale", "sepia", "invert"
    brightness: Option<i32>, // -100 to 100
    contrast: Option<f32>,   // -100.0 to 100.0
    blur: Option<f32>,       // sigma > 0
    hue_rotate: Option<i32>, // degrees
    saturation: Option<f32>, // multiplier, 1.0 is normal
}

/// edit an image and save to dest file
pub async fn edit_image(params: EditParams) -> bool {
    if let Ok(img) = get_edited_image(&params).await {
        let path = Path::new(&params.dest_file_path);
        let format = match params.output_format.as_str() {
            "png" => image::ImageFormat::Png,
            "webp" => image::ImageFormat::WebP,
            _ => image::ImageFormat::Jpeg,
        };

        let quality = params.quality.unwrap_or(80);
        let save_ok = if format == image::ImageFormat::Jpeg {
            if let Ok(file) = std::fs::File::create(path) {
                let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(file, quality);
                encoder.encode_image(&img).is_ok()
            } else {
                false
            }
        } else {
            img.save_with_format(path, format).is_ok()
        };

        if !save_ok {
            return false;
        }

        if format == image::ImageFormat::Jpeg {
            let metadata_backup_path = match prepare_metadata_backup_path(
                &params.source_file_path,
                &params.dest_file_path,
            ) {
                Ok(path) => path,
                Err(_) => return false,
            };

            let metadata_source = metadata_backup_path
                .as_ref()
                .map(|p| p.as_path())
                .unwrap_or_else(|| Path::new(&params.source_file_path));

            if let Err(_) = copy_metadata_to_output(metadata_source, path) {
                if metadata_backup_path.is_some() {
                    let _ = fs::copy(metadata_source, path);
                } else {
                    let _ = fs::remove_file(path);
                }
                cleanup_metadata_backup(&metadata_backup_path);
                return false;
            }

            cleanup_metadata_backup(&metadata_backup_path);
        }

        return true;
    }
    false
}

fn prepare_metadata_backup_path(source_file_path: &str, dest_file_path: &str) -> Result<Option<PathBuf>, String> {
    if source_file_path != dest_file_path {
        return Ok(None);
    }

    let source_path = Path::new(source_file_path);
    let extension = source_path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("tmp");
    let backup_path = std::env::temp_dir().join(format!(
        "lap-edit-metadata-{}.{}",
        Uuid::new_v4(),
        extension
    ));

    fs::copy(source_path, &backup_path)
        .map_err(|e| format!("Failed to create metadata backup: {}", e))?;

    Ok(Some(backup_path))
}

fn cleanup_metadata_backup(path: &Option<PathBuf>) {
    if let Some(path) = path {
        let _ = fs::remove_file(path);
    }
}

/// Copies metadata from source to destination.
/// Prefers little_exif for JPEGs and falls back to kamadak-exif for RAW formats.
fn copy_metadata_to_output(source_path: &Path, dest_path: &Path) -> Result<(), String> {
    let source_path_buf = source_path.to_path_buf();

    // Check file type to detect RAW formats
    let file_type = crate::t_utils::get_file_type(source_path.to_str().unwrap_or_default()).unwrap_or(0);
    let is_raw = file_type == 3;

    let mut little_exif_worked = false;
    let mut little_exif_error = String::new();

    if !is_raw {
        // Use little_exif for standard formats (JPEG/WebP).
        // Wrapped in catch_unwind as little_exif can panic on malformed data.
        let result = panic::catch_unwind(AssertUnwindSafe(|| {
            LittleExifMetadata::new_from_path(&source_path_buf)
        }));

        match result {
            Ok(Ok(mut metadata)) => {
                sanitize_edit_output_metadata(&mut metadata);
                if let Err(e) = metadata.write_to_file(dest_path) {
                    little_exif_error = format!("little_exif write failed: {}", e);
                } else {
                    little_exif_worked = true;
                }
            }
            Ok(Err(e)) => {
                little_exif_error = format!("little_exif read failed: {}", e);
            }
            Err(_) => {
                little_exif_error = "little_exif panicked".to_string();
            }
        }
    }

    if little_exif_worked {
        return Ok(());
    }

    // Fallback: use kamadak-exif which has broader RAW support
    match copy_metadata_from_raw_to_jpeg(source_path, dest_path) {
        Ok(()) => Ok(()),
        Err(raw_error) => {
            if is_raw {
                Err(format!("RAW metadata extraction failed: {}", raw_error))
            } else {
                Err(format!(
                    "Metadata copy failed. little_exif: {}; kamadak: {}",
                    little_exif_error, raw_error
                ))
            }
        }
    }
}

/// Removes tags that shouldn't be copied to the edited output (like original orientation and dimensions).
fn sanitize_edit_output_metadata(metadata: &mut LittleExifMetadata) {
    metadata.remove_tag_by_hex_group(0x0112, ExifTagGroup::GENERIC); // Orientation
    metadata.remove_tag_by_hex_group(0x0100, ExifTagGroup::GENERIC); // ImageWidth
    metadata.remove_tag_by_hex_group(0x0101, ExifTagGroup::GENERIC); // ImageLength
    metadata.remove_tag_by_hex_group(0xA002, ExifTagGroup::EXIF); // PixelXDimension
    metadata.remove_tag_by_hex_group(0xA003, ExifTagGroup::EXIF); // PixelYDimension
    metadata.remove_tag_by_hex_group(0x0201, ExifTagGroup::GENERIC); // JPEGInterchangeFormat
    metadata.remove_tag_by_hex_group(0x0202, ExifTagGroup::GENERIC); // JPEGInterchangeFormatLength
}

/// Filter for EXIF fields to copy. 
/// We mainly copy PRIMARY IFD and exclude pointers or hardware-specific tags that
/// might be invalidated by the image edit (like StripOffsets or Orientation).
fn should_copy_exif_field(field: &exif::Field) -> bool {
    if field.ifd_num != In::PRIMARY {
        return false;
    }

    !matches!(
        field.tag,
        Tag::ExifIFDPointer
            | Tag::GPSInfoIFDPointer
            | Tag::InteropIFDPointer
            | Tag::StripOffsets
            | Tag::StripByteCounts
            | Tag::TileOffsets
            | Tag::TileByteCounts
            | Tag::JPEGInterchangeFormat
            | Tag::JPEGInterchangeFormatLength
            | Tag::Orientation
            | Tag::ImageWidth
            | Tag::ImageLength
            | Tag::PixelXDimension
            | Tag::PixelYDimension
    )
}

/// Extracts EXIF from a (potentially RAW) source and injects it into a JPEG destination.
/// Includes a three-pass reduction logic to ensure metadata fits within the 64KB JPEG segment limit.
fn copy_metadata_from_raw_to_jpeg(source_path: &Path, dest_path: &Path) -> Result<(), String> {
    let file =
        File::open(source_path).map_err(|e| format!("Failed to open source metadata file: {}", e))?;
    let mut reader = BufReader::new(file);
    
    // Try read_from_container first (handles JPEG/TIFF/RAW-TIFF)
    let exif = match Reader::new().read_from_container(&mut reader) {
        Ok(exif) => exif,
        Err(_) => {
            // Fallback: try raw TIFF read (some RAWs are just TIFF structures)
            let data = fs::read(source_path).map_err(|e| e.to_string())?;
            match Reader::new().read_raw(data) {
                Ok(exif) => exif,
                Err(_) => return Ok(()), // Truly no metadata found or unreadable, skip
            }
        }
    };

    // Helper to attempt encoding a set of fields and check if it fits in 64KB
    let encode_and_check = |fields: Vec<&exif::Field>| -> Option<Vec<u8>> {
        let mut writer = exif::experimental::Writer::new();
        for field in fields {
            writer.push_field(field);
        }
        let mut tiff_cursor = Cursor::new(Vec::new());
        if writer.write(&mut tiff_cursor, exif.little_endian()).is_ok() {
            let data = tiff_cursor.into_inner();
            if data.len() <= 65527 { // JPEG APP1 max is 65535, minus 8 bytes header
                return Some(data);
            }
        }
        None
    };

    // Pass 1: Attempt to copy all standard fields
    let initial_fields: Vec<&exif::Field> = exif.fields().filter(|f| should_copy_exif_field(f)).collect();
    let mut exif_data = encode_and_check(initial_fields);

    // Pass 2: If too large, strip typically large vendor blocks (MakerNote, UserComment)
    if exif_data.is_none() {
        let reduced_fields: Vec<&exif::Field> = exif.fields()
            .filter(|f| should_copy_exif_field(f))
            .filter(|f| !matches!(f.tag, Tag::MakerNote | Tag::UserComment))
            .collect();
        exif_data = encode_and_check(reduced_fields);
    }

    // Pass 3: If still too large, keep only the most essential photography and GPS tags
    if exif_data.is_none() {
        let essential_fields: Vec<&exif::Field> = exif.fields()
            .filter(|f| should_copy_exif_field(f))
            .filter(|f| matches!(f.tag, 
                Tag::Make | Tag::Model | Tag::DateTimeOriginal | Tag::DateTimeDigitized |
                Tag::ExposureTime | Tag::FNumber | Tag::PhotographicSensitivity | Tag::FocalLength |
                Tag::LensMake | Tag::LensModel | Tag::ExposureBiasValue |
                Tag::GPSLatitudeRef | Tag::GPSLatitude | Tag::GPSLongitudeRef | Tag::GPSLongitude | Tag::GPSAltitudeRef | Tag::GPSAltitude
            ))
            .collect();
        exif_data = encode_and_check(essential_fields);
    }

    if let Some(data) = exif_data {
        write_jpeg_exif_block(dest_path, &data)
    } else {
        eprintln!("EXIF metadata still too large even after stripping, skipping");
        Ok(())
    }
}

/// Manually injects a TIFF-formatted EXIF block into a JPEG file's APP1 segment.
/// This is used when high-level libraries (like little_exif) fail or are not applicable.
fn write_jpeg_exif_block(dest_path: &Path, exif_tiff_data: &[u8]) -> Result<(), String> {
    let mut file_buffer =
        fs::read(dest_path).map_err(|e| format!("Failed to read destination JPEG: {}", e))?;

    // Clear existing metadata using little_exif if possible
    let _ = LittleExifMetadata::clear_metadata(&mut file_buffer, FileExtension::JPEG);

    if file_buffer.len() < 2 || file_buffer[0] != 0xFF || file_buffer[1] != 0xD8 {
        return Err("Destination file is not a valid JPEG".to_string());
    }

    // Prepare APP1 segment: FF E1 + Length + "Exif\0\0" + TIFF data
    let app1_length = (2 + 6 + exif_tiff_data.len()) as u16;
    let mut app1_segment = Vec::with_capacity(2 + 2 + 6 + exif_tiff_data.len());
    app1_segment.extend_from_slice(&[0xFF, 0xE1]);
    app1_segment.extend_from_slice(&app1_length.to_be_bytes());
    app1_segment.extend_from_slice(b"Exif\0\0");
    app1_segment.extend_from_slice(exif_tiff_data);

    // Reconstruct file: FF D8 + New APP1 + Remainder of file
    let mut output = Vec::with_capacity(file_buffer.len() + app1_segment.len());
    output.extend_from_slice(&file_buffer[..2]);
    output.extend_from_slice(&app1_segment);
    output.extend_from_slice(&file_buffer[2..]);

    fs::write(dest_path, output).map_err(|e| format!("Failed to write destination JPEG: {}", e))
}

/// copy an image to clipboard
pub fn copy_image_to_clipboard(img: DynamicImage) -> bool {
    let (width, height) = img.dimensions();
    let rgba = img.to_rgba8();
    let bytes = rgba.into_raw();

    if let Ok(mut clipboard) = Clipboard::new() {
        let image_data = arboard::ImageData {
            width: width as usize,
            height: height as usize,
            bytes: std::borrow::Cow::Owned(bytes),
        };
        return clipboard.set_image(image_data).is_ok();
    }
    false
}

fn is_heic_path(file_path: &str) -> bool {
    matches!(
        t_utils::get_file_extension(file_path)
            .unwrap_or_default()
            .to_lowercase()
            .as_str(),
        "heic" | "heif"
    )
}

fn should_generate_preview_for_file(file_path: &str, file_type: i64) -> bool {
    file_type == 3
        || crate::t_libraw::is_tiff_path(file_path)
        || t_jxl::is_jxl_path(file_path)
        || is_heic_path(file_path)
}

async fn get_generated_preview_bytes(file_path: &str) -> Result<Option<Vec<u8>>, String> {
    let file_type = t_utils::get_file_type(file_path).unwrap_or(0);

    if file_type == 3 {
        return get_raw_preview_image(file_path);
    }

    if t_jxl::is_jxl_path(file_path) {
        return t_jxl::get_jxl_preview_image(file_path, 4096);
    }

    if crate::t_libraw::is_tiff_path(file_path) {
        return match get_raw_preview_image(file_path) {
            Ok(Some(data)) => Ok(Some(data)),
            _ => {
                #[cfg(target_os = "macos")]
                {
                    get_thumbnail_with_sips(file_path, 4096)
                }
                #[cfg(not(target_os = "macos"))]
                {
                    Ok(None)
                }
            }
        };
    }

    if is_heic_path(file_path) {
        #[cfg(target_os = "macos")]
        {
            return get_thumbnail_with_sips(file_path, 4096);
        }
        #[cfg(not(target_os = "macos"))]
        {
            return t_video::get_video_thumbnail(file_path, 4096, None).await;
        }
    }

    Ok(None)
}

pub async fn copy_file_to_clipboard(file_path: &str) -> Result<bool, String> {
    if should_generate_preview_for_file(file_path, t_utils::get_file_type(file_path).unwrap_or(0))
    {
        let preview = get_generated_preview_bytes(file_path).await?
            .ok_or_else(|| format!("Failed to resolve preview image: {}", file_path))?;
        let img = image::load_from_memory(&preview)
            .map_err(|e| format!("Failed to decode preview image: {}", e))?;
        return Ok(copy_image_to_clipboard(img));
    }

    let img = image::open(Path::new(file_path))
        .map_err(|e| format!("Failed to open image: {}", e))?;
    Ok(copy_image_to_clipboard(img))
}

/// copy an edited image to clipboard
pub async fn copy_edited_image_to_clipboard(params: EditParams) -> bool {
    if let Ok(img) = get_edited_image(&params).await {
        return copy_image_to_clipboard(img);
    }
    false
}

/// get an edited image
async fn get_edited_image(params: &EditParams) -> Result<DynamicImage, String> {
    let file_type = t_utils::get_file_type(&params.source_file_path).unwrap_or(0);
    let mut img = if should_generate_preview_for_file(&params.source_file_path, file_type) {
        let preview = get_generated_preview_bytes(&params.source_file_path).await?
            .ok_or_else(|| "Failed to resolve editable preview image".to_string())?;
        image::load_from_memory(&preview)
            .map_err(|e| format!("Failed to decode editable preview image: {}", e))?
    } else {
        let path = Path::new(&params.source_file_path);
        let mut img = image::open(path).map_err(|e| e.to_string())?;
        // orientation adjustment based on exif orientation value
        img = apply_orientation(img, params.orientation);
        img
    };

    // 1. Flip
    if params.flip_horizontal {
        img = img.fliph();
    }
    if params.flip_vertical {
        img = img.flipv();
    }

    // 2. Rotate
    match params.rotate {
        90 => img = img.rotate90(),
        180 => img = img.rotate180(),
        270 => img = img.rotate270(),
        -90 => img = img.rotate270(),
        -180 => img = img.rotate180(),
        -270 => img = img.rotate90(),
        _ => {}
    }

    // 3. Crop
    if params.crop.width > 0 && params.crop.height > 0 {
        img = img.crop_imm(
            params.crop.x,
            params.crop.y,
            params.crop.width,
            params.crop.height,
        );
    }

    // 4. Resize
    if let (Some(w), Some(h)) = (params.resize.width, params.resize.height) {
        if w > 0 && h > 0 {
            img = img.resize_exact(w, h, image::imageops::FilterType::Lanczos3);
        }
    }

    // 5. Adjustments & Filters
    // NOTE: These implementations match CSS filter spec so preview == saved result.

    // Brightness: CSS brightness(X%) multiplies each channel by X/100.
    // Frontend sends -100..100, meaning factor = (100 + b) / 100.
    if let Some(b) = params.brightness {
        if b != 0 {
            let factor = (100 + b) as f32 / 100.0;
            let mut rgba = img.to_rgba8();
            for pixel in rgba.pixels_mut() {
                pixel[0] = (pixel[0] as f32 * factor).clamp(0.0, 255.0) as u8;
                pixel[1] = (pixel[1] as f32 * factor).clamp(0.0, 255.0) as u8;
                pixel[2] = (pixel[2] as f32 * factor).clamp(0.0, 255.0) as u8;
            }
            img = DynamicImage::ImageRgba8(rgba);
        }
    }

    // Contrast: CSS contrast(X%) scales deviation from 128 gray by X/100.
    // Frontend sends -100..100, meaning factor = (100 + c) / 100.
    if let Some(c) = params.contrast {
        if c != 0.0 {
            let factor = (100.0 + c) / 100.0;
            let mut rgba = img.to_rgba8();
            for pixel in rgba.pixels_mut() {
                pixel[0] = ((pixel[0] as f32 - 128.0) * factor + 128.0).clamp(0.0, 255.0) as u8;
                pixel[1] = ((pixel[1] as f32 - 128.0) * factor + 128.0).clamp(0.0, 255.0) as u8;
                pixel[2] = ((pixel[2] as f32 - 128.0) * factor + 128.0).clamp(0.0, 255.0) as u8;
            }
            img = DynamicImage::ImageRgba8(rgba);
        }
    }

    // Blur
    if let Some(sigma) = params.blur {
        if sigma > 0.0 {
            img = img.blur(sigma);
        }
    }

    // Hue Rotate
    if let Some(hue) = params.hue_rotate {
        if hue != 0 {
            img = img.huerotate(hue);
        }
    }

    // Saturation
    if let Some(saturation) = params.saturation {
        if (saturation - 1.0).abs() > f32::EPSILON {
            let mut rgba = img.to_rgba8();
            for pixel in rgba.pixels_mut() {
                let r = pixel[0] as f32;
                let g = pixel[1] as f32;
                let b = pixel[2] as f32;

                // Simple saturation: blend pixel with its grayscale value
                // Luma = 0.299 * R + 0.587 * G + 0.114 * B
                let luma = 0.299 * r + 0.587 * g + 0.114 * b;

                let new_r = (luma + saturation * (r - luma)).clamp(0.0, 255.0) as u8;
                let new_g = (luma + saturation * (g - luma)).clamp(0.0, 255.0) as u8;
                let new_b = (luma + saturation * (b - luma)).clamp(0.0, 255.0) as u8;

                pixel[0] = new_r;
                pixel[1] = new_g;
                pixel[2] = new_b;
            }
            img = DynamicImage::ImageRgba8(rgba);
        }
    }

    // Filter
    if let Some(filter) = &params.filter {
        match filter.as_str() {
            "grayscale" => {
                img = DynamicImage::ImageLuma8(img.to_luma8());
            }
            "invert" => {
                img.invert();
            }
            "sepia" => {
                // Manual Sepia implementation since standard image crate might not have it exposed simply
                // Formula:
                // R = (r * 0.393) + (g * 0.769) + (b * 0.189)
                // G = (r * 0.349) + (g * 0.686) + (b * 0.168)
                // B = (r * 0.272) + (g * 0.534) + (b * 0.131)
                let mut rgba = img.to_rgba8();
                for pixel in rgba.pixels_mut() {
                    let r = pixel[0] as f32;
                    let g = pixel[1] as f32;
                    let b = pixel[2] as f32;

                    let new_r = (r * 0.393 + g * 0.769 + b * 0.189).min(255.0) as u8;
                    let new_g = (r * 0.349 + g * 0.686 + b * 0.168).min(255.0) as u8;
                    let new_b = (r * 0.272 + g * 0.534 + b * 0.131).min(255.0) as u8;

                    pixel[0] = new_r;
                    pixel[1] = new_g;
                    pixel[2] = new_b;
                    // alpha unchanged
                }
                img = DynamicImage::ImageRgba8(rgba);
            }
            _ => {}
        }
    }

    Ok(img)
}

#[cfg(target_os = "macos")]
pub fn get_thumbnail_with_sips(
    file_path: &str,
    thumbnail_size: u32,
) -> Result<Option<Vec<u8>>, String> {
    use std::fs;
    use std::process::Command;
    use std::time::{SystemTime, UNIX_EPOCH};

    let temp_dir = std::env::temp_dir();
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| format!("System clock error: {}", e))?
        .subsec_nanos();
    let temp_file = temp_dir.join(format!("thumb_{}.jpg", nanos));
    let temp_output = temp_file.to_str().ok_or("Invalid temp path")?;

    let output = Command::new("sips")
        .arg("--resampleHeight")
        .arg(thumbnail_size.to_string())
        .arg("-s")
        .arg("format")
        .arg("jpeg")
        .arg(file_path)
        .arg("--out")
        .arg(temp_output)
        .output()
        .map_err(|e| format!("Failed to run sips: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "sips failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let data = fs::read(&temp_file).map_err(|e| format!("Failed to read temp file: {}", e))?;
    let _ = fs::remove_file(temp_file);

    Ok(Some(data))
}

#[cfg(target_os = "macos")]
pub fn get_dimensions_with_sips(file_path: &str) -> Result<Option<(u32, u32)>, String> {
    let output = Command::new("sips")
        .arg("-g")
        .arg("pixelWidth")
        .arg("-g")
        .arg("pixelHeight")
        .arg(file_path)
        .output()
        .map_err(|e| format!("Failed to run sips for dimensions: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "sips dimension probe failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut width: Option<u32> = None;
    let mut height: Option<u32> = None;

    for line in stdout.lines() {
        let line = line.trim();
        if let Some(value) = line.strip_prefix("pixelWidth:") {
            width = value.trim().parse::<u32>().ok();
        } else if let Some(value) = line.strip_prefix("pixelHeight:") {
            height = value.trim().parse::<u32>().ok();
        }
    }

    match (width, height) {
        (Some(width), Some(height)) if width > 0 && height > 0 => Ok(Some((width, height))),
        _ => Ok(None),
    }
}

#[cfg(target_os = "macos")]
pub fn get_heic_thumbnail_with_sips(
    file_path: &str,
    thumbnail_size: u32,
) -> Result<Option<Vec<u8>>, String> {
    get_thumbnail_with_sips(file_path, thumbnail_size)
}

const FILE_IMAGE_RESULT_CACHE_MAX: usize = 8;

#[derive(Clone)]
struct FileImageCacheEntry {
    signature: (u64, u128),
    data: Vec<u8>,
}

struct FileImageResultCache {
    entries: HashMap<String, FileImageCacheEntry>,
    order: VecDeque<String>,
}

impl FileImageResultCache {
    fn new() -> Self {
        Self {
            entries: HashMap::new(),
            order: VecDeque::new(),
        }
    }

    fn get(&mut self, file_path: &str, signature: (u64, u128)) -> Option<Vec<u8>> {
        let entry = self.entries.get(file_path)?;
        if entry.signature != signature {
            self.entries.remove(file_path);
            self.order.retain(|item| item != file_path);
            return None;
        }

        self.order.retain(|item| item != file_path);
        self.order.push_back(file_path.to_string());
        Some(entry.data.clone())
    }

    fn insert(&mut self, file_path: String, signature: (u64, u128), data: Vec<u8>) {
        self.entries
            .insert(file_path.clone(), FileImageCacheEntry { signature, data });
        self.order.retain(|item| item != &file_path);
        self.order.push_back(file_path);

        while self.order.len() > FILE_IMAGE_RESULT_CACHE_MAX {
            if let Some(oldest) = self.order.pop_front() {
                self.entries.remove(&oldest);
            }
        }
    }
}

static FILE_IMAGE_RESULT_CACHE: Lazy<Mutex<FileImageResultCache>> =
    Lazy::new(|| Mutex::new(FileImageResultCache::new()));

fn get_file_signature(file_path: &str) -> Result<(u64, u128), String> {
    let metadata = fs::metadata(file_path)
        .map_err(|e| format!("Failed to read file metadata for cache: {}", e))?;
    let modified = metadata
        .modified()
        .map_err(|e| format!("Failed to read file modified time for cache: {}", e))?
        .duration_since(UNIX_EPOCH)
        .map_err(|e| format!("Invalid file modified time for cache: {}", e))?
        .as_millis();
    Ok((metadata.len(), modified))
}

pub async fn get_file_image_bytes_cached(file_path: &str) -> Result<Vec<u8>, String> {
    let file_type = t_utils::get_file_type(file_path).unwrap_or(0);
    let cache_signature = if should_generate_preview_for_file(file_path, file_type) {
        Some(get_file_signature(file_path)?)
    } else {
        None
    };

    if let Some(signature) = cache_signature {
        if let Ok(mut cache) = FILE_IMAGE_RESULT_CACHE.lock() {
            if let Some(cached) = cache.get(file_path, signature) {
                return Ok(cached);
            }
        }
    }

    let image_data = if file_type == 3 {
        get_raw_preview_image(file_path)?
            .ok_or_else(|| format!("Failed to resolve RAW preview image: {}", file_path))?
    } else if t_jxl::is_jxl_path(file_path) {
        t_jxl::get_jxl_preview_image(file_path, 4096)?
            .ok_or_else(|| format!("Failed to resolve JXL preview image: {}", file_path))?
    } else if is_heic_path(file_path) {
        #[cfg(target_os = "macos")]
        {
            get_thumbnail_with_sips(file_path, 4096)?
                .ok_or_else(|| format!("Failed to resolve HEIC preview image: {}", file_path))?
        }
        #[cfg(not(target_os = "macos"))]
        {
            t_video::get_video_thumbnail(file_path, 4096, None).await?
                .ok_or_else(|| format!("Failed to resolve HEIC preview image: {}", file_path))?
        }
    } else if crate::t_libraw::is_tiff_path(file_path) {
        match get_raw_preview_image(file_path) {
            Ok(Some(data)) => data,
            _ => tokio::fs::read(file_path)
                .await
                .map_err(|e| format!("Failed to read the image: {}", e))?,
        }
    } else {
        tokio::fs::read(file_path)
            .await
            .map_err(|e| format!("Failed to read the image: {}", e))?
    };

    if let Some(signature) = cache_signature {
        if let Ok(mut cache) = FILE_IMAGE_RESULT_CACHE.lock() {
            cache.insert(file_path.to_string(), signature, image_data.clone());
        }
    }

    Ok(image_data)
}

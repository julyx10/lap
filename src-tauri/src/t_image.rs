/**
 * Image processing utilities.
 * project: Lap
 * author:  julyx10
 * date:    2024-08-08
 */
use arboard::Clipboard;
use exif::{In, Reader, Tag};
use image::{DynamicImage, GenericImageView, ImageFormat, ImageReader};
use once_cell::sync::Lazy;
use rusqlite::Result;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::io::Cursor;
use std::panic;
use std::path::Path;
use std::sync::Mutex;
use std::time::UNIX_EPOCH;

#[cfg(target_os = "macos")]
use std::process::Command;

use crate::{t_jxl, t_libraw, t_utils};

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
    let file = File::open(file_path).map_err(|e| format!("Failed to open RAW image: {}", e))?;
    let mut reader = BufReader::new(file);
    let exif = Reader::new()
        .read_from_container(&mut reader)
        .map_err(|e| format!("Failed to parse EXIF dimensions: {}", e))?;

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

pub fn get_image_orientation(file_path: &str) -> i32 {
    let file = match File::open(file_path) {
        Ok(file) => file,
        Err(_) => return 1,
    };
    let mut reader = BufReader::new(file);
    let exif = match Reader::new().read_from_container(&mut reader) {
        Ok(exif) => exif,
        Err(_) => return 1,
    };

    exif.get_field(Tag::Orientation, In::PRIMARY)
        .and_then(|field| field.value.get_uint(0))
        .map(|value| value as i32)
        .unwrap_or(1)
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
        // Open and decode the image
        let img_reader =
            ImageReader::open(file_path).map_err(|e| format!("Failed to open image: {}", e))?;
        let img_format = img_reader.format().ok_or("Could not detect image format")?;
        let img = img_reader
            .decode()
            .map_err(|e| format!("Failed to decode image: {}", e))?;
        let thumbnail = img.thumbnail(u32::MAX, thumbnail_size);

        // Adjust the image orientation based on the EXIF orientation value
        let adjusted_thumbnail = apply_orientation(thumbnail, orientation);

        // Determine output format based on input format
        let output_format = if img_format == ImageFormat::Png {
            ImageFormat::Png
        } else {
            ImageFormat::Jpeg
        };

        // Save the thumbnail to an in-memory buffer
        let mut buf = Vec::new();

        if output_format == ImageFormat::Jpeg {
            // For JPEG, convert to RGB8 to remove alpha channel
            let rgb_image = adjusted_thumbnail.to_rgb8();
            match rgb_image.write_to(&mut Cursor::new(&mut buf), output_format) {
                Ok(()) => Ok(Some(buf)),
                Err(e) => {
                    eprintln!(
                        "Failed to write thumbnail to buffer as {:?}: {}",
                        output_format, e
                    );
                    Ok(None)
                }
            }
        } else {
            // For PNG, keep RGBA8 to preserve alpha channel
            let rgba_image = adjusted_thumbnail.to_rgba8();
            match rgba_image.write_to(&mut Cursor::new(&mut buf), output_format) {
                Ok(()) => Ok(Some(buf)),
                Err(e) => {
                    eprintln!(
                        "Failed to write thumbnail to buffer as {:?}: {}",
                        output_format, e
                    );
                    Ok(None)
                }
            }
        }
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
    let file = File::open(file_path).map_err(|e| format!("Failed to open RAW image: {}", e))?;
    let mut reader = BufReader::new(file);
    let exif = Reader::new()
        .read_from_container(&mut reader)
        .map_err(|e| format!("Failed to parse EXIF preview: {}", e))?;

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
    let mut reader = Cursor::new(data);
    let exif = match Reader::new().read_from_container(&mut reader) {
        Ok(exif) => exif,
        Err(_) => return 1,
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
        let mut buf = Vec::new();
        image
            .to_rgb8()
            .write_to(&mut Cursor::new(&mut buf), ImageFormat::Jpeg)
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
        let thumbnail = img.thumbnail(u32::MAX, thumbnail_size);
        let adjusted_thumbnail =
            apply_orientation(thumbnail, get_jpeg_orientation_from_bytes(&preview));

        let rgb_image = adjusted_thumbnail.to_rgb8();
        let mut buf = Vec::new();
        rgb_image
            .write_to(&mut Cursor::new(&mut buf), ImageFormat::Jpeg)
            .map_err(|e| format!("Failed to encode RAW thumbnail as JPEG: {}", e))?;
        return Ok(Some(buf));
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
pub fn edit_image(params: EditParams) -> bool {
    if let Ok(img) = get_edited_image(&params) {
        let path = Path::new(&params.dest_file_path);
        let format = match params.output_format.as_str() {
            "png" => image::ImageFormat::Png,
            "webp" => image::ImageFormat::WebP,
            _ => image::ImageFormat::Jpeg,
        };

        let quality = params.quality.unwrap_or(80);

        if format == image::ImageFormat::Jpeg {
            if let Ok(file) = std::fs::File::create(path) {
                let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(file, quality);
                let _ = encoder.encode_image(&img);
                return true;
            }
        } else {
            return img.save_with_format(path, format).is_ok();
        }

        return false;
    }
    false
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

/// copy an edited image to clipboard
pub fn copy_edited_image_to_clipboard(params: EditParams) -> bool {
    if let Ok(img) = get_edited_image(&params) {
        return copy_image_to_clipboard(img);
    }
    false
}

/// get an edited image
fn get_edited_image(params: &EditParams) -> Result<DynamicImage, String> {
    let mut img = if t_utils::get_file_type(&params.source_file_path).unwrap_or(0) == 3
        || crate::t_libraw::is_tiff_path(&params.source_file_path)
    {
        let preview = get_raw_preview_image(&params.source_file_path)?
            .ok_or_else(|| "Failed to resolve editable RAW preview".to_string())?;
        image::load_from_memory(&preview).map_err(|e| e.to_string())?
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

fn should_cache_file_image_result(file_path: &str, file_type: i64) -> bool {
    file_type == 3 || crate::t_libraw::is_tiff_path(file_path) || t_jxl::is_jxl_path(file_path)
}

pub async fn get_file_image_bytes_cached(file_path: &str) -> Result<Vec<u8>, String> {
    let file_type = t_utils::get_file_type(file_path).unwrap_or(0);
    let cache_signature = if should_cache_file_image_result(file_path, file_type) {
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

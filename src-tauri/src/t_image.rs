/**
 * Image processing utilities.
 * project: jc-photo
 * author:  julyxx
 * email:   tiangle@gmail.com
 * GitHub:  /julyx10
 * date:    2024-08-08
 */
use arboard::Clipboard;
use image::{DynamicImage, GenericImageView, ImageFormat, ImageReader};
use rusqlite::Result;
use serde::{Deserialize, Serialize};
use std::io::Cursor;
use std::path::Path;
use std::process::Command;

/// Quick probing of image dimensions without loading the entire file
pub fn get_image_dimensions(file_path: &str) -> Result<(u32, u32), String> {
    // Use imagesize to get width and height
    let dimensions = imagesize::size(file_path).map_err(|e| e.to_string())?; // Map error to String if any

    // Return the dimensions as (width, height)
    Ok((dimensions.width as u32, dimensions.height as u32))
}

/// Get a thumbnail from an image file path
pub fn get_image_thumbnail(
    file_path: &str,
    orientation: i32,
    thumbnail_size: u32,
) -> Result<Option<Vec<u8>>, String> {
    // Open and decode the image
    let img_reader =
        ImageReader::open(file_path).map_err(|e| format!("Failed to open image: {}", e))?;
    let img_format = img_reader.format().ok_or("Could not detect image format")?;
    let img = img_reader
        .decode()
        .map_err(|e| format!("Failed to decode image: {}", e))?;
    let thumbnail = img.thumbnail(thumbnail_size, thumbnail_size);

    // Adjust the image orientation based on the EXIF orientation value
    let adjusted_thumbnail = match orientation {
        3 => thumbnail.rotate180(),
        6 => thumbnail.rotate90(),
        8 => thumbnail.rotate270(),
        _ => thumbnail,
    };

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
}

/// Print an image using the default system printer
/// This function is platform-specific and may need to be adjusted for different operating systems
pub fn print_image(image_path: &str) -> Result<(), String> {
    // Platform-specific printing logic
    let output = if cfg!(target_os = "windows") {
        Command::new("mspaint")
            .arg("/p")
            .arg(image_path)
            .output()
            .map_err(|e| e.to_string())?
    } else if cfg!(target_os = "macos") {
        Command::new("lp")
            .arg(image_path)
            .output()
            .map_err(|e| e.to_string())?
    } else {
        return Err("Unsupported OS".to_string());
    };

    if !output.status.success() {
        return Err(format!(
            "Failed to print image: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    Ok(())
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
    let path = Path::new(&params.source_file_path);
    let mut img = image::open(path).map_err(|e| e.to_string())?;

    // orientaion adjustment based on exif orientation value
    img = match params.orientation {
        3 => img.rotate180(),
        6 => img.rotate90(),
        8 => img.rotate270(),
        _ => img,
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

    // Brightness: -100 to 100.
    if let Some(b) = params.brightness {
        if b != 0 {
            img = img.brighten(b);
        }
    }

    // Contrast: -100.0 to 100.0.
    if let Some(c) = params.contrast {
        if c != 0.0 {
            img = img.adjust_contrast(c);
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
pub fn get_heic_thumbnail_with_sips(
    file_path: &str,
    thumbnail_size: u32,
) -> Result<Option<Vec<u8>>, String> {
    use std::fs;
    use std::process::Command;
    use std::time::{SystemTime, UNIX_EPOCH};

    let temp_dir = std::env::temp_dir();
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    let temp_file = temp_dir.join(format!("thumb_{}.jpg", nanos));
    let temp_output = temp_file.to_str().ok_or("Invalid temp path")?;

    let output = Command::new("sips")
        .arg("-Z")
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

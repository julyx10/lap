/**
 * project: jc-photo
 * author:  julyxx
 * email:   tiangle@gmail.com
 * GitHub:  /julyx10
 * date:    2024-08-08
 */

use std::process::Command;
use std::io::Cursor;
use ffmpeg_next as ffmpeg;
use image::{DynamicImage, RgbImage, ImageFormat, ImageReader, GenericImageView};
use rusqlite::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;
use arboard::Clipboard;

/// Quick probing of image dimensions without loading the entire file
pub fn get_image_dimensions(file_path: &str) -> Result<(u32, u32), String> {
    // Use imagesize to get width and height
    let dimensions = imagesize::size(file_path).map_err(|e| e.to_string())?; // Map error to String if any

    // Return the dimensions as (width, height)
    Ok((dimensions.width as u32, dimensions.height as u32))
}

/// Get video dimensions using ffmpeg
pub fn get_video_dimensions(file_path: &str) -> Result<(u32, u32), String> {
    ffmpeg_next::init().map_err(|e| format!("ffmpeg init error: {:?}", e))?;
    match ffmpeg_next::format::input(&file_path) {
        Ok(ictx) => {
            let input = ictx
                .streams()
                .best(ffmpeg_next::media::Type::Video)
                .ok_or("No video stream found")?;
            let context = ffmpeg_next::codec::context::Context::from_parameters(input.parameters())
                .map_err(|e| format!("Failed to get codec context: {}", e))?;
            let decoder = context.decoder();
            let video = decoder.video()
                .map_err(|e| format!("Failed to get video decoder: {}", e))?;
            Ok((video.width(), video.height()))
        }
        Err(e) => Err(format!("Failed to open file: {:?}", e)),
    }
}

/// get video duration using ffmpeg
pub fn get_video_duration(file_path: &str) -> Result<u64, String> {
    ffmpeg_next::init().map_err(|e| format!("ffmpeg init error: {:?}", e))?;
    let ictx = ffmpeg_next::format::input(file_path).map_err(|e| format!("Failed to open file: {e}"))?;
    let duration = ictx.duration();
    let duration_seconds = if duration > 0 {
        (duration as f64 / ffmpeg_next::ffi::AV_TIME_BASE as f64) as u64  // Convert from AV_TIME_BASE to seconds
    } else {
        0
    };
    Ok(duration_seconds)
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
                eprintln!("Failed to write thumbnail to buffer as {:?}: {}", output_format, e);
                Ok(None)
            },
        }
    } else {
        // For PNG, keep RGBA8 to preserve alpha channel
        let rgba_image = adjusted_thumbnail.to_rgba8();
        match rgba_image.write_to(&mut Cursor::new(&mut buf), output_format) {
            Ok(()) => Ok(Some(buf)),
            Err(e) => {
                eprintln!("Failed to write thumbnail to buffer as {:?}: {}", output_format, e);
                Ok(None)
            },
        }
    }
}

/// Get a thumbnail from a video or heic file path
pub fn get_video_thumbnail(
    file_path: &str,
    thumbnail_size: u32,
) -> Result<Option<Vec<u8>>, String> {
    ffmpeg::init().map_err(|e| format!("ffmpeg init error: {e}"))?;

    let mut ictx = ffmpeg::format::input(file_path)
        .map_err(|e| format!("Failed to open file: {e}"))?;

    let input_stream = ictx
        .streams()
        .best(ffmpeg::media::Type::Video)
        .ok_or("No video stream found")?;

    let stream_index = input_stream.index();

    let rotation = input_stream
        .metadata()
        .get("rotate")
        .and_then(|v| v.parse::<i32>().ok())
        .unwrap_or(0);

    let mut decoder = ffmpeg::codec::context::Context::from_parameters(input_stream.parameters())
        .map_err(|e| format!("Failed to get decoder context: {e}"))?
        .decoder()
        .video()
        .map_err(|e| format!("Decoder error: {e}"))?;

    // For video files, seek to 10% of the duration
    // For HEIC files, we may not have a duration, so we skip seeking
    if ictx.duration() > 0 {
        ictx.seek(ictx.duration() / 10 , ..)
            .map_err(|e| format!("Seek error: {e}"))?;
    }

    for (stream, packet) in ictx.packets() {
        if stream.index() != stream_index {
            continue;
        }

        decoder
            .send_packet(&packet)
            .map_err(|e| format!("Send packet error: {e}"))?;

        let mut frame = ffmpeg::util::frame::Video::empty();
        if decoder.receive_frame(&mut frame).is_ok() {
            let width = frame.width();
            let height = frame.height();

            // Convert to RGB
            let mut rgb_frame = ffmpeg::util::frame::Video::empty();
            let mut scaler = ffmpeg::software::scaling::context::Context::get(
                decoder.format(),
                width,
                height,
                ffmpeg::format::Pixel::RGB24,
                width,
                height,
                ffmpeg::software::scaling::flag::Flags::BILINEAR,
            )
            .map_err(|e| format!("Scaler creation error: {e}"))?;

            scaler
                .run(&frame, &mut rgb_frame)
                .map_err(|e| format!("Scaling error: {e}"))?;

            // avoid stride error
            let stride = rgb_frame.stride(0) as usize;
            let mut buf = Vec::with_capacity((width * height * 3) as usize);
            for y in 0..height as usize {
                let start = y * stride;
                let end = start + (width as usize * 3);
                buf.extend_from_slice(&rgb_frame.data(0)[start..end]);
            }
            
            // Create DynamicImage
            let rgb_image = RgbImage::from_raw(width, height, buf)
                .ok_or("Failed to create image buffer")?;
            let dyn_image = DynamicImage::ImageRgb8(rgb_image);

            // Resize while keeping aspect ratio
            let thumbnail = dyn_image.thumbnail(thumbnail_size, thumbnail_size);

            let adjusted_thumbnail = match rotation {
                90 => thumbnail.rotate90(),
                180 => thumbnail.rotate180(),
                270 => thumbnail.rotate270(),
                _ => thumbnail,
            };

            // Encode JPEG to memory
            let mut buffer = Cursor::new(Vec::new());
            adjusted_thumbnail
                .write_to(&mut buffer, ImageFormat::Jpeg)
                .map_err(|e| format!("Image encode error: {e}"))?;

            return Ok(Some(buffer.into_inner()));
        }
    }

    Ok(None)
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
        return Err(format!("Failed to print image: {}", String::from_utf8_lossy(&output.stderr)));
    }

    Ok(())
}

/// edit image impl

/// crop data
#[derive(Debug, Deserialize, Serialize)]
pub struct CropData {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
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
    orientation: i32,   // exif orientation value
    #[serde(rename = "flipHorizontal")]
    flip_horizontal: bool,
    #[serde(rename = "flipVertical")]
    flip_vertical: bool,
    rotate: i32,
    crop: CropData,
    resize: ResizeData,
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

        return img.save_with_format(path, format).is_ok();
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

    // Flip
    if params.flip_horizontal {
        img = img.fliph();
    }
    if params.flip_vertical {
        img = img.flipv();
    }

    // Rotate
    match params.rotate {
        90 => img = img.rotate90(),
        180 => img = img.rotate180(),
        270 => img = img.rotate270(),
        -90 => img = img.rotate270(),
        -180 => img = img.rotate180(),
        -270 => img = img.rotate90(),
        _ => {},
    }

    // Crop
    img = img.crop_imm(
        params.crop.x.round() as u32,
        params.crop.y.round() as u32,
        params.crop.width.round() as u32,
        params.crop.height.round() as u32,
    );

    // Resize
    if let (Some(w), Some(h)) = (params.resize.width, params.resize.height) {
        if w > 0 && h > 0 {
            img = img.resize_exact(w, h, image::imageops::FilterType::Lanczos3);
        }
    }

    Ok(img)
}

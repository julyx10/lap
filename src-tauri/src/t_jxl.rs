use exif::{In, Reader, Tag};
use image::{DynamicImage, GenericImageView, ImageDecoder, ImageFormat};
use jxl_oxide::integration::JxlDecoder;
use std::fs::File;
use std::io::BufReader;
use std::io::Cursor;
use std::path::Path;

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

pub fn is_jxl_path(file_path: &str) -> bool {
    matches!(
        Path::new(file_path)
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_ascii_lowercase())
            .as_deref(),
        Some("jxl")
    )
}

fn open_decoder(
    file_path: &str,
) -> Result<JxlDecoder<BufReader<File>>, String> {
    let file = File::open(file_path).map_err(|e| format!("Failed to open JXL image: {}", e))?;
    let reader = BufReader::new(file);
    JxlDecoder::new(reader).map_err(|e| format!("Failed to initialize JXL decoder: {}", e))
}

fn get_jxl_orientation(file_path: &str) -> i32 {
    let mut decoder = match open_decoder(file_path) {
        Ok(decoder) => decoder,
        Err(_) => return 1,
    };

    let exif = match decoder.exif_metadata() {
        Ok(Some(exif)) => exif,
        _ => return 1,
    };

    Reader::new()
        .read_raw(exif)
        .ok()
        .and_then(|exif| exif.get_field(Tag::Orientation, In::PRIMARY).and_then(|field| field.value.get_uint(0)))
        .map(|value| value as i32)
        .unwrap_or(1)
}

fn decode_jxl_image(file_path: &str) -> Result<DynamicImage, String> {
    let decoder = open_decoder(file_path)?;
    let image = DynamicImage::from_decoder(decoder)
        .map_err(|e| format!("Failed to decode JXL image: {}", e))?;
    Ok(apply_orientation(image, get_jxl_orientation(file_path)))
}

pub fn get_jxl_dimensions(file_path: &str) -> Result<(u32, u32), String> {
    let decoder = open_decoder(file_path)?;
    let (mut width, mut height) = decoder.dimensions();
    if matches!(get_jxl_orientation(file_path), 5 | 6 | 7 | 8) {
        std::mem::swap(&mut width, &mut height);
    }
    Ok((width, height))
}

pub fn get_jxl_preview_image(file_path: &str, max_edge: u32) -> Result<Option<Vec<u8>>, String> {
    let image = decode_jxl_image(file_path)?;
    let image = if max_edge > 0 {
        image.resize(max_edge, max_edge, image::imageops::FilterType::Lanczos3)
    } else {
        image
    };

    let mut buf = Vec::new();
    image
        .to_rgb8()
        .write_to(&mut Cursor::new(&mut buf), ImageFormat::Jpeg)
        .map_err(|e| format!("Failed to encode JXL preview: {}", e))?;
    Ok(Some(buf))
}

pub fn get_jxl_thumbnail(file_path: &str, thumbnail_size: u32) -> Result<Option<Vec<u8>>, String> {
    let image = decode_jxl_image(file_path)?;
    let thumbnail = image.thumbnail(u32::MAX, thumbnail_size);

    let mut buf = Vec::new();
    thumbnail
        .to_rgb8()
        .write_to(&mut Cursor::new(&mut buf), ImageFormat::Jpeg)
        .map_err(|e| format!("Failed to encode JXL thumbnail: {}", e))?;
    Ok(Some(buf))
}

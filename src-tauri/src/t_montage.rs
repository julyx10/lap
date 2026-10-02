// Photo montage: composes several photos onto one page (grid, mosaic, pile).
// The layout comes from the frontend in page-relative coordinates, so the
// preview and this full-resolution render share one source of truth.

use fast_image_resize as fir;
use image::{DynamicImage, ImageFormat, Rgba, RgbaImage};
use serde::Deserialize;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

use crate::t_image;

const MAX_PAGE_SIDE: u32 = 12000;
// drop shadow, relative to the page width (same values as the preview in Montage.vue)
const SHADOW_OFFSET: f32 = 0.004;
const SHADOW_BLUR: f32 = 0.004; // gaussian sigma
const SHADOW_OPACITY: f32 = 0.45;

#[derive(Debug, Clone, Deserialize)]
pub struct MontageItem {
    path: String,
    orientation: i32, // exif orientation value
    rotate: i32,      // file rotation stored by Lap (0, 90, 180, 270)
    // frame (border included), relative to the page: 0..1
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    rotation: f32, // degrees, clockwise
    border: f32,   // border width, relative to the page width
}

#[derive(Debug, Deserialize)]
pub struct MontageParams {
    width: u32,
    height: u32,
    background: String, // "#rrggbb"
    #[serde(rename = "borderColor")]
    border_color: String, // "#rrggbb"
    shadow: bool,
    items: Vec<MontageItem>,
    #[serde(rename = "destFilePath")]
    dest_file_path: String,
    #[serde(rename = "outputFormat")]
    output_format: String,
    quality: Option<u8>,
}

fn parse_hex_color(color: &str) -> Result<Rgba<u8>, String> {
    let hex = color.trim_start_matches('#');
    if hex.len() != 6 {
        return Err(format!("Invalid color: {}", color));
    }
    let channel = |i: usize| {
        u8::from_str_radix(&hex[i..i + 2], 16).map_err(|_| format!("Invalid color: {}", color))
    };
    Ok(Rgba([channel(0)?, channel(2)?, channel(4)?, 255]))
}

/// Center-crop `img` to the aspect ratio of `dst_w`x`dst_h`, then resize it to that size.
fn cover_resize(img: DynamicImage, dst_w: u32, dst_h: u32) -> Result<RgbaImage, String> {
    let (src_w, src_h) = (img.width().max(1), img.height().max(1));
    let dst_ratio = dst_w as f64 / dst_h as f64;
    let (crop_w, crop_h) = if src_w as f64 / src_h as f64 > dst_ratio {
        (((src_h as f64 * dst_ratio).round() as u32).clamp(1, src_w), src_h)
    } else {
        (src_w, ((src_w as f64 / dst_ratio).round() as u32).clamp(1, src_h))
    };
    let cropped = img
        .crop_imm((src_w - crop_w) / 2, (src_h - crop_h) / 2, crop_w, crop_h)
        .into_rgba8();

    let src_image = fir::images::Image::from_vec_u8(crop_w, crop_h, cropped.into_raw(), fir::PixelType::U8x4)
        .map_err(|e| format!("Failed to prepare montage image for resize: {}", e))?;
    let mut dst_image = fir::images::Image::new(dst_w, dst_h, fir::PixelType::U8x4);
    let options = fir::ResizeOptions::new()
        .resize_alg(fir::ResizeAlg::Convolution(fir::FilterType::Lanczos3));
    fir::Resizer::new()
        .resize(&src_image, &mut dst_image, &options)
        .map_err(|e| format!("Failed to resize montage image: {}", e))?;
    RgbaImage::from_raw(dst_w, dst_h, dst_image.into_vec())
        .ok_or_else(|| "Failed to build resized montage image".to_string())
}

/// Blur the tile's silhouette into a dark shadow and blend it below the tile,
/// offset down-right in page coordinates.
fn draw_shadow(canvas: &mut RgbaImage, tile: &RgbaImage, left: i64, top: i64) {
    let page_w = canvas.width() as f32;
    let sigma = (page_w * SHADOW_BLUR).max(0.5);
    let pad = (sigma * 3.0).ceil() as u32;
    let mut shadow = RgbaImage::new(tile.width() + 2 * pad, tile.height() + 2 * pad);
    for (x, y, px) in tile.enumerate_pixels() {
        shadow.put_pixel(x + pad, y + pad, Rgba([0, 0, 0, (px[3] as f32 * SHADOW_OPACITY).round() as u8]));
    }
    let shadow = image::imageops::fast_blur(&shadow, sigma);
    let offset = (page_w * SHADOW_OFFSET).round() as i64;
    image::imageops::overlay(canvas, &shadow, left - pad as i64 + offset, top - pad as i64 + offset);
}

/// Draw one photo onto the page: fill its frame (cover), add the border,
/// rotate it, and alpha-blend it (and its shadow) centered on the frame's center.
fn draw_item(
    canvas: &mut RgbaImage,
    img: DynamicImage,
    item: &MontageItem,
    border_color: Rgba<u8>,
    shadow: bool,
) -> Result<(), String> {
    let (page_w, page_h) = (canvas.width() as f32, canvas.height() as f32);
    let frame_w = (item.w * page_w).round().max(1.0) as u32;
    let frame_h = (item.h * page_h).round().max(1.0) as u32;
    let border = ((item.border * page_w).round() as u32).min((frame_w.min(frame_h) - 1) / 2);

    let photo = cover_resize(img, frame_w - 2 * border, frame_h - 2 * border)?;
    let tile = if border > 0 {
        let mut framed = RgbaImage::from_pixel(frame_w, frame_h, border_color);
        image::imageops::replace(&mut framed, &photo, border as i64, border as i64);
        framed
    } else {
        photo
    };
    let tile = if item.rotation != 0.0 {
        t_image::rotate_arbitrary(DynamicImage::ImageRgba8(tile), item.rotation).into_rgba8()
    } else {
        tile
    };

    let center_x = (item.x + item.w / 2.0) * page_w;
    let center_y = (item.y + item.h / 2.0) * page_h;
    let left = (center_x - tile.width() as f32 / 2.0).round() as i64;
    let top = (center_y - tile.height() as f32 / 2.0).round() as i64;
    if shadow {
        draw_shadow(canvas, &tile, left, top);
    }
    image::imageops::overlay(canvas, &tile, left, top);
    Ok(())
}

fn apply_rotate(img: DynamicImage, rotate: i32) -> DynamicImage {
    match rotate.rem_euclid(360) {
        90 => img.rotate90(),
        180 => img.rotate180(),
        270 => img.rotate270(),
        _ => img,
    }
}

/// Encode into a temporary file next to the destination, then rename it,
/// so a failed save never leaves a partial file behind.
fn write_montage(canvas: RgbaImage, dest: &Path, format: &str, quality: u8) -> Result<(), String> {
    let rgb = DynamicImage::ImageRgba8(canvas).into_rgb8();
    let tmp_path = dest.with_file_name(format!(
        "{}.part",
        dest.file_name().and_then(|n| n.to_str()).unwrap_or("montage")
    ));

    let result = (|| {
        let mut writer = BufWriter::new(File::create(&tmp_path).map_err(|e| e.to_string())?);
        match format {
            "png" => rgb.write_to(&mut writer, ImageFormat::Png),
            "webp" => rgb.write_to(&mut writer, ImageFormat::WebP),
            _ => image::codecs::jpeg::JpegEncoder::new_with_quality(&mut writer, quality).encode_image(&rgb),
        }
        .map_err(|e| e.to_string())?;
        writer.into_inner().map_err(|e| e.to_string())?; // flush, reporting write errors
        std::fs::rename(&tmp_path, dest).map_err(|e| e.to_string())
    })();

    if result.is_err() {
        let _ = std::fs::remove_file(&tmp_path);
    }
    result
}

/// Render the montage at full resolution from the original files and save it.
pub async fn render_montage(params: MontageParams) -> Result<(), String> {
    if params.width == 0 || params.height == 0 || params.width > MAX_PAGE_SIDE || params.height > MAX_PAGE_SIDE {
        return Err(format!("Invalid montage size: {}x{}", params.width, params.height));
    }
    if params.items.is_empty() {
        return Err("No photos in the montage".to_string());
    }
    let dest = Path::new(&params.dest_file_path).to_path_buf();
    if dest.exists() {
        return Err(format!("File already exists: {}", params.dest_file_path));
    }

    let mut canvas = RgbaImage::from_pixel(params.width, params.height, parse_hex_color(&params.background)?);
    let border_color = parse_hex_color(&params.border_color)?;
    let shadow = params.shadow;

    // One photo at a time: only a single decoded original is in memory.
    for item in params.items {
        let img = t_image::load_oriented_image(&item.path, item.orientation)
            .await
            .map_err(|e| format!("{}: {}", item.path, e))?;
        canvas = tauri::async_runtime::spawn_blocking(move || {
            let img = apply_rotate(img, item.rotate);
            draw_item(&mut canvas, img, &item, border_color, shadow).map_err(|e| format!("{}: {}", item.path, e))?;
            Ok::<RgbaImage, String>(canvas)
        })
        .await
        .map_err(|e| format!("Failed to join montage task: {}", e))??;
    }

    let quality = params.quality.unwrap_or(80);
    tauri::async_runtime::spawn_blocking(move || write_montage(canvas, &dest, &params.output_format, quality))
        .await
        .map_err(|e| format!("Failed to join montage task: {}", e))?
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(x: f32, y: f32, w: f32, h: f32, rotation: f32, border: f32) -> MontageItem {
        MontageItem { path: String::new(), orientation: 1, rotate: 0, x, y, w, h, rotation, border }
    }

    const WHITE: Rgba<u8> = Rgba([255, 255, 255, 255]);

    fn solid(w: u32, h: u32, color: [u8; 4]) -> DynamicImage {
        DynamicImage::ImageRgba8(RgbaImage::from_pixel(w, h, Rgba(color)))
    }

    #[test]
    fn parses_hex_colors() {
        assert_eq!(parse_hex_color("#102030").unwrap(), Rgba([16, 32, 48, 255]));
        assert!(parse_hex_color("#12").is_err());
        assert!(parse_hex_color("#zzzzzz").is_err());
    }

    #[test]
    fn draws_photo_in_its_frame_and_keeps_background_elsewhere() {
        let mut canvas = RgbaImage::from_pixel(100, 100, Rgba([0, 0, 255, 255]));
        // wide red source into a square frame on the right half: cover-cropped, no distortion issue
        draw_item(&mut canvas, solid(300, 100, [255, 0, 0, 255]), &item(0.5, 0.0, 0.5, 0.5, 0.0, 0.0), WHITE, false).unwrap();

        assert_eq!(canvas.get_pixel(75, 25), &Rgba([255, 0, 0, 255])); // frame center
        assert_eq!(canvas.get_pixel(25, 25), &Rgba([0, 0, 255, 255])); // left of the frame
        assert_eq!(canvas.get_pixel(75, 75), &Rgba([0, 0, 255, 255])); // below the frame
    }

    #[test]
    fn draws_white_border_around_photo() {
        let mut canvas = RgbaImage::from_pixel(100, 100, Rgba([0, 0, 0, 255]));
        draw_item(&mut canvas, solid(50, 50, [255, 0, 0, 255]), &item(0.0, 0.0, 1.0, 1.0, 0.0, 0.1), WHITE, false).unwrap();

        assert_eq!(canvas.get_pixel(5, 50), &Rgba([255, 255, 255, 255])); // inside the 10 px border
        assert_eq!(canvas.get_pixel(50, 50), &Rgba([255, 0, 0, 255]));
    }

    #[test]
    fn rotated_photo_stays_centered_on_its_frame() {
        let mut canvas = RgbaImage::from_pixel(200, 200, Rgba([0, 0, 0, 255]));
        draw_item(&mut canvas, solid(40, 40, [0, 255, 0, 255]), &item(0.4, 0.4, 0.2, 0.2, 45.0, 0.0), WHITE, false).unwrap();

        assert_eq!(canvas.get_pixel(100, 100), &Rgba([0, 255, 0, 255]));
        // a corner of the unrotated frame falls outside the rotated square
        assert_eq!(canvas.get_pixel(81, 81), &Rgba([0, 0, 0, 255]));
    }

    #[test]
    fn draws_border_in_the_given_color() {
        let mut canvas = RgbaImage::from_pixel(100, 100, Rgba([255, 255, 255, 255]));
        let gray = Rgba([224, 224, 224, 255]);
        draw_item(&mut canvas, solid(50, 50, [255, 0, 0, 255]), &item(0.0, 0.0, 1.0, 1.0, 0.0, 0.1), gray, false).unwrap();
        assert_eq!(canvas.get_pixel(5, 50), &gray);
    }

    #[test]
    fn shadow_darkens_the_page_beside_the_photo_only_when_enabled() {
        // 1000 px page: offset 4 px, sigma 4 px; just right of / below the frame's corner
        for shadow in [false, true] {
            let mut canvas = RgbaImage::from_pixel(1000, 1000, Rgba([255, 255, 255, 255]));
            draw_item(&mut canvas, solid(40, 40, [255, 0, 0, 255]), &item(0.4, 0.4, 0.2, 0.2, 0.0, 0.0), WHITE, shadow).unwrap();
            let beside = canvas.get_pixel(603, 603)[0];
            assert_eq!(beside < 255, shadow, "pixel beside the photo: {}", beside);
            assert_eq!(canvas.get_pixel(100, 100), &WHITE); // far from the photo
            assert_eq!(canvas.get_pixel(500, 500), &Rgba([255, 0, 0, 255])); // the photo stays on top
        }
    }

    #[test]
    fn applies_lap_rotation() {
        let img = apply_rotate(solid(30, 10, [0, 0, 0, 255]), 90);
        assert_eq!((img.width(), img.height()), (10, 30));
        assert_eq!(apply_rotate(solid(30, 10, [0, 0, 0, 255]), -180).width(), 30);
    }
}

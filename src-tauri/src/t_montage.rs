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

/// Center-crop `img` to the aspect ratio of `dst_w`x`dst_h` and resize it to that size,
/// producing only the `part` (x, y, w, h) of the result.
fn cover_resize(img: DynamicImage, dst_w: u32, dst_h: u32, part: (u32, u32, u32, u32)) -> Result<RgbaImage, String> {
    let src = img.into_rgba8();
    let (src_w, src_h) = (src.width().max(1), src.height().max(1));
    let dst_ratio = dst_w as f64 / dst_h as f64;
    let (crop_w, crop_h) = if src_w as f64 / src_h as f64 > dst_ratio {
        (((src_h as f64 * dst_ratio).round() as u32).clamp(1, src_w), src_h)
    } else {
        (src_w, ((src_w as f64 / dst_ratio).round() as u32).clamp(1, src_h))
    };
    let (scale_x, scale_y) = (crop_w as f64 / dst_w as f64, crop_h as f64 / dst_h as f64);
    let (x, y, w, h) = part;
    // the part of the source; min() keeps float rounding from pushing it past the edge
    let left = ((src_w - crop_w) / 2) as f64 + x as f64 * scale_x;
    let top = ((src_h - crop_h) / 2) as f64 + y as f64 * scale_y;
    let width = (w as f64 * scale_x).min(src_w as f64 - left);
    let height = (h as f64 * scale_y).min(src_h as f64 - top);

    let src_image = fir::images::Image::from_vec_u8(src.width(), src.height(), src.into_raw(), fir::PixelType::U8x4)
        .map_err(|e| format!("Failed to prepare montage image for resize: {}", e))?;
    let mut dst_image = fir::images::Image::new(w, h, fir::PixelType::U8x4);
    let options = fir::ResizeOptions::new()
        .resize_alg(fir::ResizeAlg::Convolution(fir::FilterType::Lanczos3))
        .crop(left, top, width, height);
    fir::Resizer::new()
        .resize(&src_image, &mut dst_image, &options)
        .map_err(|e| format!("Failed to resize montage image: {}", e))?;
    RgbaImage::from_raw(w, h, dst_image.into_vec())
        .ok_or_else(|| "Failed to build resized montage image".to_string())
}

/// Error function (Abramowitz & Stegun 7.1.26, error below 1.5e-7).
fn erf(x: f32) -> f32 {
    let t = 1.0 / (1.0 + 0.327_591_1 * x.abs());
    let poly = ((((1.061_405_4 * t - 1.453_152) * t + 1.421_413_8) * t - 0.284_496_74) * t + 0.254_829_6) * t;
    (1.0 - poly * (-x * x).exp()).copysign(x)
}

/// Darken the page with the drop shadow of a `frame_w`x`frame_h` frame centered on
/// (`center_x`, `center_y`) and rotated by (`sin`, `cos`), offset down-right.
/// A gaussian blur of a rectangle is the product of two erf differences along its
/// own axes, so the shadow is computed exactly, page pixel by page pixel, without
/// building and blurring a silhouette.
fn draw_shadow(canvas: &mut RgbaImage, center_x: f32, center_y: f32, frame_w: f32, frame_h: f32, sin: f32, cos: f32) {
    let (page_w, page_h) = (canvas.width() as f32, canvas.height() as f32);
    let sigma = (page_w * SHADOW_BLUR).max(0.5);
    let offset = (page_w * SHADOW_OFFSET).round();
    let (cx, cy) = (center_x + offset, center_y + offset);
    let (half_w, half_h) = (frame_w / 2.0, frame_h / 2.0);
    let scale = 1.0 / (sigma * std::f32::consts::SQRT_2);
    // fraction of the blurred edge interval [-half, half] seen at distance u
    let coverage = |u: f32, half: f32| 0.5 * (erf((half - u) * scale) + erf((half + u) * scale));

    // bounding box of the rotated frame, widened by 3 sigma, on the page
    let reach_x = half_w * cos.abs() + half_h * sin.abs() + 3.0 * sigma;
    let reach_y = half_w * sin.abs() + half_h * cos.abs() + 3.0 * sigma;
    let x0 = (cx - reach_x).floor().clamp(0.0, page_w) as u32;
    let x1 = (cx + reach_x).ceil().clamp(0.0, page_w) as u32;
    let y0 = (cy - reach_y).floor().clamp(0.0, page_h) as u32;
    let y1 = (cy + reach_y).ceil().clamp(0.0, page_h) as u32;
    // the shadow's center, in the frame's own axes, relative to the frame's center
    let (shift_u, shift_v) = (offset * (cos + sin), offset * (cos - sin));
    for y in y0..y1 {
        let dy = y as f32 + 0.5 - cy;
        for x in x0..x1 {
            let dx = x as f32 + 0.5 - cx;
            // into the frame's own axes: the clockwise rotation, inverted
            let (u, v) = (dx * cos + dy * sin, -dx * sin + dy * cos);
            // hidden under the photo, which is drawn next (1 px kept for its anti-aliased edge)
            if (u + shift_u).abs() < half_w - 1.0 && (v + shift_v).abs() < half_h - 1.0 {
                continue;
            }
            let alpha = SHADOW_OPACITY * coverage(u, half_w) * coverage(v, half_h);
            if alpha > 0.001 {
                let px = canvas.get_pixel_mut(x, y);
                for c in 0..3 {
                    px[c] = (px[c] as f32 * (1.0 - alpha)).round() as u8;
                }
            }
        }
    }
}

/// Draw one photo onto the page: fill its frame (cover), add the border,
/// rotate it, and alpha-blend it (and its shadow) centered on the frame's center.
/// Only the part of the frame that can reach the page is built, so a photo
/// enlarged far beyond the page costs no more than the page itself.
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
    let center_x = (item.x + item.w / 2.0) * page_w;
    let center_y = (item.y + item.h / 2.0) * page_h;
    let (sin, cos) = item.rotation.to_radians().sin_cos();

    if shadow {
        draw_shadow(canvas, center_x, center_y, frame_w as f32, frame_h as f32, sin, cos);
    }

    // the page, widened by the anti-aliased edge, mapped into the frame:
    // the clockwise rotation around the centers, inverted
    let margin = 2.0;
    let (mut min_x, mut min_y, mut max_x, mut max_y) = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
    for (px, py) in [(-margin, -margin), (page_w + margin, -margin), (-margin, page_h + margin), (page_w + margin, page_h + margin)] {
        let (dx, dy) = (px - center_x, py - center_y);
        let fx = dx * cos + dy * sin + frame_w as f32 / 2.0;
        let fy = -dx * sin + dy * cos + frame_h as f32 / 2.0;
        (min_x, min_y, max_x, max_y) = (min_x.min(fx), min_y.min(fy), max_x.max(fx), max_y.max(fy));
    }
    let left = min_x.floor().clamp(0.0, frame_w as f32) as u32;
    let top = min_y.floor().clamp(0.0, frame_h as f32) as u32;
    let right = max_x.ceil().clamp(0.0, frame_w as f32) as u32;
    let bottom = max_y.ceil().clamp(0.0, frame_h as f32) as u32;
    if left >= right || top >= bottom {
        return Ok(()); // entirely off the page
    }

    // that part of the frame: border color, then the part of the photo inside it
    let mut tile = RgbaImage::from_pixel(right - left, bottom - top, border_color);
    let (photo_left, photo_top) = (left.max(border), top.max(border));
    let (photo_right, photo_bottom) = (right.min(frame_w - border), bottom.min(frame_h - border));
    if photo_left < photo_right && photo_top < photo_bottom {
        let photo = cover_resize(
            img,
            frame_w - 2 * border,
            frame_h - 2 * border,
            (photo_left - border, photo_top - border, photo_right - photo_left, photo_bottom - photo_top),
        )?;
        image::imageops::replace(&mut tile, &photo, (photo_left - left) as i64, (photo_top - top) as i64);
    }
    let tile = if item.rotation != 0.0 {
        t_image::rotate_arbitrary(DynamicImage::ImageRgba8(tile), item.rotation).into_rgba8()
    } else {
        tile
    };

    // the part's center, rotated around the frame's center, places it on the page
    let dx = (left + right) as f32 / 2.0 - frame_w as f32 / 2.0;
    let dy = (top + bottom) as f32 / 2.0 - frame_h as f32 / 2.0;
    let left = (center_x + dx * cos - dy * sin - tile.width() as f32 / 2.0).round() as i64;
    let top = (center_y + dx * sin + dy * cos - tile.height() as f32 / 2.0).round() as i64;
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

    /// smooth gradient, so a misplaced or misscaled part shows as a color difference
    fn gradient(w: u32, h: u32) -> DynamicImage {
        DynamicImage::ImageRgba8(RgbaImage::from_fn(w, h, |x, y| Rgba([(x * 2) as u8, (y * 2) as u8, 128, 255])))
    }

    #[test]
    fn a_photo_cut_by_the_page_edge_looks_as_if_drawn_whole() {
        // the same 120 px rotated frame: inside a 200 px page, and across the corner of a
        // 100 px page that is the top-left quarter of it (coordinates are page-relative)
        let mut whole = RgbaImage::from_pixel(200, 200, Rgba([0, 0, 0, 255]));
        draw_item(&mut whole, gradient(120, 90), &item(0.1, 0.1, 0.6, 0.6, 20.0, 0.02), WHITE, false).unwrap();
        let mut cut = RgbaImage::from_pixel(100, 100, Rgba([0, 0, 0, 255]));
        draw_item(&mut cut, gradient(120, 90), &item(0.2, 0.2, 1.2, 1.2, 20.0, 0.04), WHITE, false).unwrap();

        // both renders round the photo's position to whole pixels, so edges may differ
        // by up to half a pixel: compare away from them (where neighbors differ sharply)
        let diff = |a: &Rgba<u8>, b: &Rgba<u8>| (0..3).map(|i| (a[i] as i32 - b[i] as i32).abs()).max().unwrap();
        let mut worst = 0;
        for (x, y, px) in cut.enumerate_pixels() {
            let other = whole.get_pixel(x, y);
            let near_edge = [(x.saturating_sub(1), y), (x + 1, y), (x, y.saturating_sub(1)), (x, y + 1)]
                .iter()
                .any(|&(nx, ny)| diff(whole.get_pixel(nx, ny), other) > 20);
            if !near_edge {
                worst = worst.max(diff(px, other));
            }
        }
        assert!(worst <= 4, "largest channel difference: {}", worst);
    }

    #[test]
    fn a_part_reaching_the_photo_edge_stays_inside_the_source() {
        // floating-point rounding must not push the crop box past the source's edge
        for (src_w, src_h) in [(400, 300), (300, 400), (123, 98), (37, 23)] {
            for dst in (50..400).step_by(3) {
                let (dst_w, dst_h) = (dst, dst * 2 / 3 + 1);
                let part = (dst_w / 3, dst_h / 3, dst_w - dst_w / 3, dst_h - dst_h / 3);
                if let Err(e) = cover_resize(solid(src_w, src_h, [0, 0, 0, 255]), dst_w, dst_h, part) {
                    panic!("{}x{} into {}x{}: {}", src_w, src_h, dst_w, dst_h, e);
                }
            }
        }
    }

    #[test]
    fn a_photo_off_the_page_draws_nothing() {
        let mut canvas = RgbaImage::from_pixel(100, 100, Rgba([0, 0, 255, 255]));
        draw_item(&mut canvas, solid(40, 40, [255, 0, 0, 255]), &item(1.5, 1.5, 0.5, 0.5, 30.0, 0.0), WHITE, true).unwrap();
        assert!(canvas.pixels().all(|px| px == &Rgba([0, 0, 255, 255])));
    }

    #[test]
    fn a_photo_four_times_the_page_fills_it() {
        let mut canvas = RgbaImage::from_pixel(100, 100, Rgba([0, 0, 255, 255]));
        draw_item(&mut canvas, solid(40, 40, [255, 0, 0, 255]), &item(-1.5, -1.5, 4.0, 4.0, 30.0, 0.01), WHITE, true).unwrap();
        assert!(canvas.pixels().all(|px| px == &Rgba([255, 0, 0, 255])));
    }

    #[test]
    fn rotated_edges_are_anti_aliased() {
        let rotated = t_image::rotate_arbitrary(solid(40, 40, [255, 0, 0, 255]), 30.0).into_rgba8();
        let alphas: Vec<u8> = rotated.pixels().map(|px| px[3]).collect();
        assert!(alphas.contains(&0) && alphas.contains(&255));
        assert!(alphas.iter().any(|&a| a > 0 && a < 255), "no partly transparent edge pixel");
    }

    #[test]
    fn shadow_matches_a_blurred_silhouette() {
        // reference: the silhouette of the frame, blurred (the previous implementation), photo on top.
        // Unrotated, the reference is exact: only its blur's approximation differs. Rotated, its
        // bounding box and position are rounded to whole pixels: up to one pixel of shift, i.e.
        // the steepest slope of the shadow's edge (11 levels per pixel on a 1000 px page).
        for (page, angle, tolerance) in [(400u32, 0.0f32, 3), (1000, 25.0, 12)] {
            let k = page as f32 / 400.0;
            let (frame_w, frame_h) = ((120.0 * k) as u32, (80.0 * k) as u32);
            let (center_x, center_y) = (200.0 * k, 190.0 * k);
            let sigma = page as f32 * SHADOW_BLUR;
            let offset = (page as f32 * SHADOW_OFFSET).round() as i64;
            let tile = if angle != 0.0 {
                t_image::rotate_arbitrary(solid(frame_w, frame_h, [255, 255, 255, 255]), angle).into_rgba8()
            } else {
                RgbaImage::from_pixel(frame_w, frame_h, WHITE)
            };
            let pad = (sigma * 3.0).ceil() as u32;
            let mut silhouette = RgbaImage::new(tile.width() + 2 * pad, tile.height() + 2 * pad);
            for (x, y, px) in tile.enumerate_pixels() {
                silhouette.put_pixel(x + pad, y + pad, Rgba([0, 0, 0, (px[3] as f32 * SHADOW_OPACITY).round() as u8]));
            }
            let blurred = image::imageops::blur(&silhouette, sigma);
            let mut expected = RgbaImage::from_pixel(page, page, WHITE);
            let left = (center_x - tile.width() as f32 / 2.0).round() as i64 - pad as i64 + offset;
            let top = (center_y - tile.height() as f32 / 2.0).round() as i64 - pad as i64 + offset;
            image::imageops::overlay(&mut expected, &blurred, left, top);

            let mut canvas = RgbaImage::from_pixel(page, page, WHITE);
            let (sin, cos) = angle.to_radians().sin_cos();
            draw_shadow(&mut canvas, center_x, center_y, frame_w as f32, frame_h as f32, sin, cos);
            // then the photo on top, as in a montage: it hides the part of the shadow under it
            let photo_left = (center_x - tile.width() as f32 / 2.0).round() as i64;
            let photo_top = (center_y - tile.height() as f32 / 2.0).round() as i64;
            let gray = |img: &RgbaImage| RgbaImage::from_fn(img.width(), img.height(), |x, y| {
                let a = img.get_pixel(x, y)[3];
                Rgba([128, 128, 128, a])
            });
            image::imageops::overlay(&mut canvas, &gray(&tile), photo_left, photo_top);
            image::imageops::overlay(&mut expected, &gray(&tile), photo_left, photo_top);

            let worst = canvas.pixels().zip(expected.pixels()).map(|(a, b)| (a[0] as i32 - b[0] as i32).abs()).max().unwrap();
            assert!(worst <= tolerance, "page {page}, angle {angle}: largest difference {worst}");
        }
    }

    #[test]
    fn applies_lap_rotation() {
        let img = apply_rotate(solid(30, 10, [0, 0, 0, 255]), 90);
        assert_eq!((img.width(), img.height()), (10, 30));
        assert_eq!(apply_rotate(solid(30, 10, [0, 0, 0, 255]), -180).width(), 30);
    }
}

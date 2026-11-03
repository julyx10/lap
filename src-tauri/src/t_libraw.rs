use crate::t_raw_display::RawDisplayOptions;

use exif;
use image::{DynamicImage, ImageBuffer, Luma, Rgb, Rgba};
use std::ffi::CStr;
use std::fs;
use std::io::Cursor;
use std::os::raw::{c_char, c_int, c_void};
use std::path::Path;

const LIBRAW_THUMBNAIL_JPEG: i32 = 1;

const LIBRAW_IMAGE_JPEG: i32 = 1;
const LIBRAW_IMAGE_BITMAP: i32 = 2;

#[repr(C)]
struct LapLibRawImage {
    data: *mut u8,
    len: u32,
    format: c_int,
    width: u16,
    height: u16,
    colors: u16,
    bits: u16,
    flip: c_int,
}

#[repr(C)]
struct LapLibRawMeta {
    make: [c_char; 128],
    model: [c_char; 128],
    software: [c_char; 128],
    artist: [c_char; 64],
    desc: [c_char; 512],
    timestamp: i64,
    iso_speed: f32,
    shutter: f32,
    aperture: f32,
    focal_len: f32,
    flash_used: f32,
    lens_make: [c_char; 128],
    lens_model: [c_char; 128],
    min_focal: f32,
    max_focal: f32,
    max_ap_min_focal: f32,
    max_ap_max_focal: f32,
}

#[link(name = "lap_libraw_shim", kind = "static")]
unsafe extern "C" {
    fn lap_libraw_open_buffer(data: *const u8, len: usize, err: *mut c_int) -> *mut c_void;
    fn lap_libraw_close(raw: *mut c_void);
    fn lap_libraw_strerror(code: c_int) -> *const c_char;
    fn lap_libraw_get_dimensions(
        raw: *mut c_void,
        width: *mut u32,
        height: *mut u32,
        flip: *mut c_int,
    ) -> c_int;
    fn lap_libraw_get_meta(raw: *mut c_void, out: *mut LapLibRawMeta) -> c_int;
    fn lap_libraw_get_thumbnail_count(raw: *mut c_void) -> c_int;
    fn lap_libraw_extract_thumbnail(
        raw: *mut c_void,
        index: c_int,
        out: *mut LapLibRawImage,
    ) -> c_int;
    fn lap_libraw_render_preview(
        raw: *mut c_void,
        half_size: c_int,
        strict_data_error: c_int,
        auto_bright: c_int,
        out: *mut LapLibRawImage,
    ) -> c_int;
    fn lap_libraw_free_buffer(data: *mut u8);
}

#[derive(Clone, Debug)]
struct RawImageBlob {
    format: i32,
    width: u32,
    height: u32,
    colors: u16,
    bits: u16,
    flip: i32,
    data: Vec<u8>,
}

struct RawHandle {
    raw: *mut c_void,
    _bytes: Vec<u8>,
}

impl Drop for RawHandle {
    fn drop(&mut self) {
        unsafe { lap_libraw_close(self.raw) };
    }
}

fn file_extension(file_path: &str) -> Option<String> {
    Path::new(file_path)
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_ascii_lowercase())
}

fn encode_as_jpeg(img: &DynamicImage) -> Result<Vec<u8>, String> {
    crate::t_jpeg::encode_rgb8(&img.to_rgb8(), 85)
        .map_err(|e| format!("Failed to encode image as JPEG: {}", e))
}

fn orient_image(img: DynamicImage, orientation: i32) -> DynamicImage {
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

fn decode_bitmap_image(
    width: u32,
    height: u32,
    colors: u16,
    bits: u16,
    data: &[u8],
) -> Result<DynamicImage, String> {
    match (colors, bits) {
        (1, 8) => {
            let image = ImageBuffer::<Luma<u8>, _>::from_raw(width, height, data.to_vec())
                .ok_or("Failed to create grayscale image buffer")?;
            Ok(DynamicImage::ImageLuma8(image))
        }
        (3, 8) => {
            let image = ImageBuffer::<Rgb<u8>, _>::from_raw(width, height, data.to_vec())
                .ok_or("Failed to create RGB image buffer")?;
            Ok(DynamicImage::ImageRgb8(image))
        }
        (4, 8) => {
            let image = ImageBuffer::<Rgba<u8>, _>::from_raw(width, height, data.to_vec())
                .ok_or("Failed to create RGBA image buffer")?;
            Ok(DynamicImage::ImageRgba8(image))
        }
        (1, 16) => {
            let data = data
                .chunks_exact(2)
                .map(|chunk| chunk[1])
                .collect::<Vec<u8>>();
            let image = ImageBuffer::<Luma<u8>, _>::from_raw(width, height, data)
                .ok_or("Failed to create 16-bit grayscale image buffer")?;
            Ok(DynamicImage::ImageLuma8(image))
        }
        (3, 16) => {
            let data = data
                .chunks_exact(2)
                .map(|chunk| chunk[1])
                .collect::<Vec<u8>>();
            let image = ImageBuffer::<Rgb<u8>, _>::from_raw(width, height, data)
                .ok_or("Failed to create 16-bit RGB image buffer")?;
            Ok(DynamicImage::ImageRgb8(image))
        }
        (4, 16) => {
            let data = data
                .chunks_exact(2)
                .map(|chunk| chunk[1])
                .collect::<Vec<u8>>();
            let image = ImageBuffer::<Rgba<u8>, _>::from_raw(width, height, data)
                .ok_or("Failed to create 16-bit RGBA image buffer")?;
            Ok(DynamicImage::ImageRgba8(image))
        }
        _ => Err(format!(
            "Unsupported bitmap payload: colors={}, bits={}",
            colors, bits
        )),
    }
}

fn decode_processed_image(image: &RawImageBlob) -> Result<DynamicImage, String> {
    match image.format {
        LIBRAW_IMAGE_JPEG => image::load_from_memory(&image.data)
            .map_err(|e| format!("Failed to decode processed RAW JPEG preview: {}", e)),
        LIBRAW_IMAGE_BITMAP => decode_bitmap_image(
            image.width,
            image.height,
            image.colors,
            image.bits,
            &image.data,
        ),
        _ => Err(format!(
            "Unsupported processed RAW image format: {}",
            image.format
        )),
    }
}

fn libraw_error(code: i32, context: &str) -> String {
    let message = unsafe {
        let ptr = lap_libraw_strerror(code);
        if ptr.is_null() {
            None
        } else {
            CStr::from_ptr(ptr).to_str().ok().map(str::to_string)
        }
    }
    .unwrap_or_else(|| format!("LibRaw error {}", code));

    format!("{}: {}", context, message)
}

impl RawHandle {
    fn open(file_path: &str) -> Result<Self, String> {
        let bytes = fs::read(file_path).map_err(|e| format!("Failed to read RAW file: {}", e))?;
        let mut err = 0;
        let raw = unsafe { lap_libraw_open_buffer(bytes.as_ptr(), bytes.len(), &mut err) };
        if raw.is_null() {
            return Err(libraw_error(err, "Failed to open RAW file with LibRaw"));
        }

        Ok(Self { raw, _bytes: bytes })
    }

    fn dimensions(&self) -> Result<(u32, u32), String> {
        let mut width = 0;
        let mut height = 0;
        let mut flip = 0;
        let ret =
            unsafe { lap_libraw_get_dimensions(self.raw, &mut width, &mut height, &mut flip) };
        if ret != 0 {
            return Err(libraw_error(ret, "Failed to resolve RAW dimensions"));
        }
        if width == 0 || height == 0 {
            return Err("LibRaw resolved empty RAW dimensions".to_string());
        }
        Ok((width, height))
    }

    fn dimensions_with_flip(&self) -> Result<(u32, u32, i32), String> {
        let mut width = 0;
        let mut height = 0;
        let mut flip = 0;
        let ret =
            unsafe { lap_libraw_get_dimensions(self.raw, &mut width, &mut height, &mut flip) };
        if ret != 0 {
            return Err(libraw_error(ret, "Failed to resolve RAW dimensions"));
        }
        if width == 0 || height == 0 {
            return Err("LibRaw resolved empty RAW dimensions".to_string());
        }
        Ok((width, height, flip))
    }

    fn meta(&self) -> Result<RawMeta, String> {
        let mut out = LapLibRawMeta {
            make: [0; 128],
            model: [0; 128],
            software: [0; 128],
            artist: [0; 64],
            desc: [0; 512],
            timestamp: 0,
            iso_speed: 0.0,
            shutter: 0.0,
            aperture: 0.0,
            focal_len: 0.0,
            flash_used: 0.0,
            lens_make: [0; 128],
            lens_model: [0; 128],
            min_focal: 0.0,
            max_focal: 0.0,
            max_ap_min_focal: 0.0,
            max_ap_max_focal: 0.0,
        };
        let ret = unsafe { lap_libraw_get_meta(self.raw, &mut out) };
        if ret != 0 {
            return Err(libraw_error(ret, "Failed to extract RAW metadata"));
        }

        Ok(RawMeta {
            make: c_char_array_to_string(&out.make),
            model: c_char_array_to_string(&out.model),
            software: c_char_array_to_string(&out.software),
            artist: c_char_array_to_string(&out.artist),
            description: c_char_array_to_string(&out.desc),
            timestamp: (out.timestamp > 0).then_some(out.timestamp),
            iso_speed: (out.iso_speed > 0.0).then(|| out.iso_speed.to_string()),
            shutter: (out.shutter > 0.0).then(|| format_shutter_speed(out.shutter)),
            aperture: (out.aperture > 0.0).then(|| format!("f/{}", out.aperture)),
            focal_len: (out.focal_len > 0.0).then(|| format!("{} mm", out.focal_len)),
            flash_used: (out.flash_used != 0.0).then(|| {
                if out.flash_used > 0.0 {
                    "Fired".to_string()
                } else {
                    "Not fired".to_string()
                }
            }),
            lens_make: c_char_array_to_string(&out.lens_make),
            lens_model: c_char_array_to_string(&out.lens_model),
        })
    }

    fn thumbnail_count(&self) -> i32 {
        unsafe { lap_libraw_get_thumbnail_count(self.raw) }
    }

    fn extract_thumbnail(&mut self, index: Option<i32>) -> Result<RawImageBlob, String> {
        let mut out = LapLibRawImage {
            data: std::ptr::null_mut(),
            len: 0,
            format: 0,
            width: 0,
            height: 0,
            colors: 0,
            bits: 0,
            flip: 0,
        };

        let ret = unsafe { lap_libraw_extract_thumbnail(self.raw, index.unwrap_or(-1), &mut out) };
        if ret != 0 {
            return Err(libraw_error(ret, "Failed to extract embedded RAW preview"));
        }

        let data = if out.data.is_null() || out.len == 0 {
            Vec::new()
        } else {
            let data = unsafe { std::slice::from_raw_parts(out.data, out.len as usize).to_vec() };
            unsafe { lap_libraw_free_buffer(out.data) };
            data
        };

        Ok(RawImageBlob {
            format: out.format,
            width: out.width as u32,
            height: out.height as u32,
            colors: out.colors,
            bits: out.bits,
            flip: out.flip,
            data,
        })
    }

    fn extract_thumbnails(&mut self) -> Vec<RawImageBlob> {
        let count = self.thumbnail_count();
        let mut thumbs = Vec::new();

        if count > 0 {
            for index in 0..count {
                if let Ok(thumb) = self.extract_thumbnail(Some(index)) {
                    thumbs.push(thumb);
                }
            }
        }

        if thumbs.is_empty() {
            if let Ok(thumb) = self.extract_thumbnail(None) {
                thumbs.push(thumb);
            }
        }

        thumbs
    }

    fn render_preview(&mut self, options: RawDisplayOptions, half_size: bool) -> Result<RawImageBlob, String> {
        let mut out = LapLibRawImage {
            data: std::ptr::null_mut(),
            len: 0,
            format: 0,
            width: 0,
            height: 0,
            colors: 0,
            bits: 0,
            flip: 0,
        };

        let ret = unsafe { lap_libraw_render_preview(self.raw, half_size as c_int, 0, options.auto_bright as c_int, &mut out) };
        if ret != 0 {
            return Err(libraw_error(ret, "Failed to process RAW preview"));
        }

        let data = if out.data.is_null() || out.len == 0 {
            Vec::new()
        } else {
            let data = unsafe { std::slice::from_raw_parts(out.data, out.len as usize).to_vec() };
            unsafe { lap_libraw_free_buffer(out.data) };
            data
        };

        Ok(RawImageBlob {
            format: out.format,
            width: out.width as u32,
            height: out.height as u32,
            colors: out.colors,
            bits: out.bits,
            flip: out.flip,
            data,
        })
    }
}

/// Metadata extracted from a RAW file via LibRaw.
#[derive(Clone)]
pub struct RawMeta {
    pub make: Option<String>,
    pub model: Option<String>,
    pub software: Option<String>,
    pub artist: Option<String>,
    pub description: Option<String>,
    pub timestamp: Option<i64>,
    pub iso_speed: Option<String>,
    pub shutter: Option<String>,
    pub aperture: Option<String>,
    pub focal_len: Option<String>,
    pub flash_used: Option<String>,
    pub lens_make: Option<String>,
    pub lens_model: Option<String>,
}

fn c_char_array_to_string(bytes: &[c_char]) -> Option<String> {
    let ptr = bytes.as_ptr();
    if ptr.is_null() || bytes.first().copied().unwrap_or_default() == 0 {
        return None;
    }

    unsafe { CStr::from_ptr(ptr) }
        .to_str()
        .ok()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
}

fn format_shutter_speed(shutter: f32) -> String {
    if shutter >= 1.0 {
        format!("{} s", shutter)
    } else {
        format!("1/{} s", 1.0 / shutter)
    }
}

fn render_processed_preview(file_path: &str, max_edge: u32, options: RawDisplayOptions, half_size: bool) -> Result<Vec<u8>, String> {
    let mut raw = RawHandle::open(file_path)?;
    let rendered = raw.render_preview(options, half_size)?;
    let image = decode_processed_image(&rendered)?;
    let image = if max_edge > 0 && image.width().max(image.height()) > max_edge {
        image.resize(max_edge, max_edge, if half_size { image::imageops::FilterType::Triangle } else { image::imageops::FilterType::Lanczos3 })
    } else {
        image
    };
    encode_as_jpeg(&image)
}

pub fn get_raw_dimensions(file_path: &str) -> Result<(u32, u32), String> {
    RawHandle::open(file_path)?.dimensions()
}

pub fn get_raw_dimensions_with_flip(file_path: &str) -> Result<(u32, u32, i32), String> {
    RawHandle::open(file_path)?.dimensions_with_flip()
}

/// Small owned metadata snapshot; never retains a LibRaw decoder or pixel buffer.
#[derive(Clone)]
pub struct RawInfo {
    pub dimensions: Option<(u32, u32)>,
    pub meta: RawMeta,
}

pub fn get_raw_info(file_path: &str) -> Result<RawInfo, String> {
    let raw = RawHandle::open(file_path)?;
    Ok(RawInfo { dimensions: raw.dimensions().ok(), meta: raw.meta()? })
}

pub fn get_raw_meta(file_path: &str) -> Result<RawMeta, String> {
    RawHandle::open(file_path)?.meta()
}

/// Read the EXIF Orientation tag from in-memory JPEG bytes.
fn jpeg_exif_orientation(data: &[u8]) -> Option<i32> {
    let mut cursor = Cursor::new(data);
    let exif = exif::Reader::new().read_from_container(&mut cursor).ok()?;
    exif.get_field(exif::Tag::Orientation, exif::In::PRIMARY)
        .and_then(|field| field.value.get_uint(0))
        .map(|v| v as i32)
        .filter(|orientation| (1..=8).contains(orientation))
}

/// Convert LibRaw's RAW-pixel orientation into an EXIF orientation value.
/// RAW and embedded thumbnails have separate flip values; prefer the latter
/// for embedded pixels when no JPEG EXIF orientation is available.
fn raw_flip_to_exif_orientation(flip: i32) -> Option<i32> {
    match flip {
        0 | 360 => Some(1),
        3 | 180 => Some(3),
        5 | 270 => Some(8), // 90° counter-clockwise
        6 | 90 => Some(6),  // 90° clockwise
        _ => None,
    }
}

// Embedded pixels may already be oriented independently of the RAW data.
fn embedded_jpeg_orientation(
    jpeg_orientation: Option<i32>, thumbnail_flip: i32,
    jpeg_size: (u32, u32), raw_size: (u32, u32, i32),
) -> i32 {
    if let Some(orientation) = jpeg_orientation { return orientation; }
    let raw_orientation = raw_flip_to_exif_orientation(raw_size.2).unwrap_or(1);
    let orientation = raw_flip_to_exif_orientation(thumbnail_flip).unwrap_or(raw_orientation);
    // TIFF thumbnail flips can be default zero or stale after the JPEG was
    // rotated. Resolve quarter-turn conflicts against LibRaw's oriented size,
    // allowing small crop/border differences without decoding RAW pixels.
    if matches!(raw_orientation, 6 | 8) && jpeg_size.0 > 0 && jpeg_size.1 > 0
        && raw_size.0 > 0 && raw_size.1 > 0 {
        let target_ratio = raw_size.0 as f64 / raw_size.1 as f64;
        let matches_size = |orientation| {
            let ratio = if matches!(orientation, 5..=8) {
                jpeg_size.1 as f64 / jpeg_size.0 as f64
            } else { jpeg_size.0 as f64 / jpeg_size.1 as f64 };
            (ratio / target_ratio - 1.0).abs() <= 0.10
        };
        if !matches_size(orientation) {
            if matches_size(raw_orientation) { return raw_orientation; }
            if matches_size(1) { return 1; }
        }
    }
    orientation
}

pub(crate) fn embedded_preview_is_large_enough(width: u32, height: u32, raw_width: u32, raw_height: u32) -> bool {
    width.max(height) >= raw_width.max(raw_height).min(1024)
        && width.min(height) >= raw_width.min(raw_height).min(512)
}

pub(crate) fn get_embedded_raw_preview_image(file_path: &str) -> Result<Option<Vec<u8>>, String> {
    let mut raw = RawHandle::open(file_path)?;
    let (raw_width, raw_height, raw_flip) = raw.dimensions_with_flip()?;
    let mut thumbs = raw.extract_thumbnails();
    thumbs.sort_by_key(|thumb| std::cmp::Reverse(thumb.width as u64 * thumb.height as u64));

    // Use the largest decodable embedded JPEG, including reduced previews.
    for thumb in &thumbs {
        if thumb.format == LIBRAW_THUMBNAIL_JPEG {
            // Prefer the JPEG's own orientation, then its thumbnail flip;
            // RAW orientation is only a fallback for unknown thumbnail flips.
            if let Ok(image) = image::load_from_memory(&thumb.data) {
                let orient = embedded_jpeg_orientation(jpeg_exif_orientation(&thumb.data), thumb.flip,
                    (image.width(), image.height()), (raw_width, raw_height, raw_flip));
                if !embedded_preview_is_large_enough(image.width(), image.height(), raw_width, raw_height) { continue; }
                let image = orient_image(image, orient);
                return encode_as_jpeg(&image).map(Some);
            }
        }
    }

    Ok(None)
}

pub fn get_raw_preview_image(
    file_path: &str,
    options: RawDisplayOptions,
) -> Result<Option<Vec<u8>>, String> {
    get_raw_preview_with_source(file_path, options).map(|(data, _, _)| Some(data))
}

pub fn get_raw_preview_with_source(
    file_path: &str,
    options: RawDisplayOptions,
) -> Result<(Vec<u8>, &'static str, bool), String> {
    resolve_raw_preview(file_path, options, 4096, false)
}

pub fn get_raw_preview_with_size(file_path: &str, options: RawDisplayOptions, full_size: bool) -> Result<(Vec<u8>, &'static str, bool), String> {
    resolve_raw_preview(file_path, options, if full_size { 0 } else { 4096 }, !full_size)
}

fn resolve_raw_preview(file_path: &str, options: RawDisplayOptions, max_edge: u32, half_size: bool) -> Result<(Vec<u8>, &'static str, bool), String> {
    if options.embedded() {
        if let Ok(Some(preview)) = get_embedded_raw_preview_image(file_path) {
            return Ok((preview, "embedded", false));
        }
    }
    let preview = render_processed_preview(file_path, max_edge, options, half_size)?;
    Ok((preview, if options.auto_bright { "brightened" } else { "rendered" }, options.embedded()))
}

fn get_processed_raw_thumbnail(file_path: &str, thumbnail_size: u32, options: RawDisplayOptions) -> Result<Option<Vec<u8>>, String> {
    let raw = RawHandle::open(file_path)?;
    let mut out = LapLibRawImage {
        data: std::ptr::null_mut(),
        len: 0,
        format: 0,
        width: 0,
        height: 0,
        colors: 0,
        bits: 0,
        flip: 0,
    };

    let ret = unsafe { lap_libraw_render_preview(raw.raw, 1, 1, options.auto_bright as c_int, &mut out) };
    if ret == 0 && !out.data.is_null() && out.len > 0 {
        // Copy C buffer into Rust Vec, then free the C allocation immediately.
        let data = unsafe { std::slice::from_raw_parts(out.data, out.len as usize).to_vec() };
        unsafe { lap_libraw_free_buffer(out.data) };

        // data is now owned by blob (Vec<u8>); if decode_processed_image fails
        // below, blob is dropped and the Vec is freed automatically — no leak.
        let blob = RawImageBlob {
            format: out.format,
            width: out.width as u32,
            height: out.height as u32,
            colors: out.colors,
            bits: out.bits,
            flip: out.flip,
            data,
        };

        if let Ok(image) = decode_processed_image(&blob) {
            let thumbnail = image.thumbnail(u32::MAX, thumbnail_size);
            return encode_as_jpeg(&thumbnail).map(Some);
        } else {
            eprintln!("LibRaw decode_processed_image failed for {}", file_path);
        }
    } else {
        if !out.data.is_null() {
            unsafe { lap_libraw_free_buffer(out.data) };
        }
        eprintln!("LibRaw dcraw_process failed for {} (likely HE/HE* NEF)", file_path);
    }

    Ok(None)
}

fn get_embedded_jpeg_thumbnail(file_path: &str, thumbnail_size: u32) -> Result<Option<Vec<u8>>, String> {
    let mut raw = RawHandle::open(file_path)?;
    let raw_size = raw.dimensions_with_flip().unwrap_or((0, 0, -1));
    let thumbs = raw.extract_thumbnails();
    let best = thumbs
        .iter()
        .filter(|thumb| thumb.format == LIBRAW_THUMBNAIL_JPEG && !thumb.data.is_empty())
        .max_by_key(|thumb| {
            let max_edge = thumb.width.max(thumb.height);
            (max_edge >= thumbnail_size, max_edge)
        });

    if let Some(thumb) = best {
        if let Ok(image) = image::load_from_memory(&thumb.data) {
            let orient = embedded_jpeg_orientation(jpeg_exif_orientation(&thumb.data), thumb.flip,
                (image.width(), image.height()), raw_size);
            let image = orient_image(image, orient);
            let thumbnail = image.thumbnail(u32::MAX, thumbnail_size);
            return encode_as_jpeg(&thumbnail).map(Some);
        }
    }

    Ok(None)
}

pub fn get_raw_thumbnail(
    file_path: &str,
    thumbnail_size: u32,
    prefer_embedded_jpeg: RawDisplayOptions,
) -> Result<Option<Vec<u8>>, String> {
    if prefer_embedded_jpeg.embedded() {
        if let Ok(Some(thumbnail)) = get_embedded_jpeg_thumbnail(file_path, thumbnail_size) {
            return Ok(Some(thumbnail));
        }
    }

    if let Ok(Some(thumbnail)) = get_processed_raw_thumbnail(file_path, thumbnail_size, prefer_embedded_jpeg) {
        return Ok(Some(thumbnail));
    }

    if !prefer_embedded_jpeg.embedded() {
        return get_embedded_jpeg_thumbnail(file_path, thumbnail_size);
    }

    Ok(None)
}

pub fn is_tiff_path(file_path: &str) -> bool {
    matches!(
        file_extension(file_path).as_deref(),
        Some("tif") | Some("tiff")
    )
}

#[cfg(test)]
pub(crate) fn inspect_raw_fixture_orientation(path: &str) {
    let Ok(mut raw) = RawHandle::open(path) else { return; };
    eprintln!("RAW dimensions {:?}", raw.dimensions_with_flip());
    for thumb in raw.extract_thumbnails() {
        let pixels = image::load_from_memory(&thumb.data).ok().map(|img| (img.width(), img.height()));
        eprintln!("  embedded format={} metadata={}x{} flip={} jpeg_exif={:?} pixels={pixels:?}",
            thumb.format, thumb.width, thumb.height, thumb.flip, jpeg_exif_orientation(&thumb.data));
    }
}

#[cfg(test)]
mod raw_display_render_tests {
    use super::*;
    use crate::t_raw_display::RawPreviewMode;

    #[test]
    fn embedded_thumbnail_orientation_resolves_default_and_stale_flips() {
        assert_eq!(embedded_jpeg_orientation(None, 0, (480, 640), (1924, 2576, 5)), 1); // E5700
        assert_eq!(embedded_jpeg_orientation(None, 0, (3008, 2000), (2014, 3039, 5)), 8); // D70
        assert_eq!(embedded_jpeg_orientation(None, 0, (4928, 3264), (3284, 4950, 6)), 6); // K5IIS
        assert_eq!(embedded_jpeg_orientation(None, 6, (332, 244), (5344, 4008, 6)), 1); // Leaf
        assert_eq!(embedded_jpeg_orientation(Some(6), 0, (480, 640), (1924, 2576, 5)), 6); // JPEG wins
        assert_eq!(embedded_jpeg_orientation(None, -1, (300, 200), (200, 300, 5)), 8);
        assert_eq!(embedded_jpeg_orientation(None, 3, (300, 200), (300, 200, 3)), 3);
        assert_eq!(embedded_jpeg_orientation(None, 0, (300, 200), (0, 0, -1)), 1);
        assert_eq!(embedded_jpeg_orientation(None, 0, (100, 100), (200, 300, 5)), 1); // ambiguous
    }

    // Small uncompressed CFA DNG, generated in memory so the test does not
    // depend on downloaded camera samples or user photos.
    fn dark_dng() -> Vec<u8> {
        let short = |v: u16| v.to_le_bytes().to_vec();
        let long = |v: u32| v.to_le_bytes().to_vec();
        let rational = |v: i32| [v.to_le_bytes(), 1i32.to_le_bytes()].concat();
        let mut tags: Vec<(u16, u16, u32, Vec<u8>)> = vec![
            (256, 4, 1, long(128)), (257, 4, 1, long(128)),
            (258, 3, 1, short(16)), (259, 3, 1, short(1)),
            (262, 3, 1, short(32803)), (271, 2, 4, b"Lap\0".to_vec()),
            (272, 2, 5, b"Test\0".to_vec()), (273, 4, 1, long(0)),
            (274, 3, 1, short(1)), (277, 3, 1, short(1)),
            (278, 4, 1, long(128)), (279, 4, 1, long(128 * 128 * 2)),
            (284, 3, 1, short(1)), (33421, 3, 2, [short(2), short(2)].concat()),
            (33422, 1, 4, vec![0, 1, 1, 2]),
            (50706, 1, 4, vec![1, 4, 0, 0]), (50707, 1, 4, vec![1, 1, 0, 0]),
            (50708, 2, 9, b"Lap Test\0".to_vec()),
            (50714, 5, 1, rational(0)), (50717, 4, 1, long(4095)),
            (50721, 10, 9, [1,0,0,0,1,0,0,0,1].into_iter().flat_map(rational).collect()),
            (50728, 5, 3, [1,1,1].into_iter().flat_map(rational).collect()),
            (50778, 3, 1, short(21)),
        ];
        tags.sort_by_key(|tag| tag.0);
        let mut bytes = b"II\x2a\0\x08\0\0\0".to_vec();
        bytes.extend_from_slice(&(tags.len() as u16).to_le_bytes());
        let mut extras = Vec::new();
        let base = 8 + 2 + tags.len() * 12 + 4;
        let mut strip_offset_position = 0;
        for (tag, kind, count, data) in tags {
            bytes.extend_from_slice(&tag.to_le_bytes());
            bytes.extend_from_slice(&kind.to_le_bytes());
            bytes.extend_from_slice(&count.to_le_bytes());
            if tag == 273 { strip_offset_position = bytes.len(); }
            if data.len() <= 4 {
                bytes.extend_from_slice(&data);
                bytes.resize(bytes.len() + 4 - data.len(), 0);
            } else {
                bytes.extend_from_slice(&((base + extras.len()) as u32).to_le_bytes());
                extras.extend_from_slice(&data);
                if extras.len() % 2 != 0 { extras.push(0); }
            }
        }
        bytes.extend_from_slice(&0u32.to_le_bytes());
        bytes.extend_from_slice(&extras);
        let offset = (bytes.len() as u32).to_le_bytes();
        bytes[strip_offset_position..strip_offset_position + 4].copy_from_slice(&offset);
        for i in 0..128 * 128 { bytes.extend_from_slice(&(32 + (i % 96) as u16).to_le_bytes()); }
        bytes
    }

    #[test]
    fn raw_display_brightness_and_missing_embedded_preview() {
        let path = std::env::temp_dir().join(format!("lap-raw-display-{}.dng", std::process::id()));
        std::fs::write(&path, dark_dng()).unwrap();
        let path_str = path.to_str().unwrap();
        let options = |mode| RawDisplayOptions { mode, prefer_pair: false, auto_bright: false };
        let render = |mode| get_raw_preview_image(path_str, options(mode)).unwrap().unwrap();
        let normal = render(RawPreviewMode::Rendered);
        let fit = get_raw_preview_with_size(path_str, options(RawPreviewMode::Rendered), false).unwrap().0;
        let full = get_raw_preview_with_size(path_str, options(RawPreviewMode::Rendered), true).unwrap().0;
        assert_eq!(image::load_from_memory(&fit).unwrap().width(), 64);
        assert_eq!(image::load_from_memory(&full).unwrap().width(), 128);
        let bright_options = RawDisplayOptions { auto_bright: true, ..options(RawPreviewMode::Rendered) };
        let bright = get_raw_preview_image(path_str, bright_options).unwrap().unwrap();
        let (fallback_bright, source, unavailable) = get_raw_preview_with_source(path_str, RawDisplayOptions { mode: RawPreviewMode::Embedded, ..bright_options }).unwrap();
        assert_eq!(fallback_bright, bright);
        assert_eq!(source, "brightened");
        assert!(unavailable);
        let embedded_fallback = render(RawPreviewMode::Embedded);
        let mean = |data: &[u8]| {
            let rgb = image::load_from_memory(data).unwrap().to_rgb8();
            rgb.as_raw().iter().map(|v| *v as f64).sum::<f64>() / rgb.as_raw().len() as f64
        };
        assert!(mean(&bright) > mean(&normal) + 20.0);
        assert_eq!(normal, embedded_fallback);
        for mode in [RawPreviewMode::Rendered, RawPreviewMode::Embedded] {
            assert!(get_raw_thumbnail(path_str, 64, options(mode)).unwrap().is_some());
        }
        std::fs::remove_file(path).unwrap();
    }
}

#[cfg(test)]
mod embedded_preview_size_tests {
    use super::embedded_preview_is_large_enough as usable;
    #[test]
    fn rejects_tiny_previews_but_accepts_reduced_and_rotated_images() {
        assert!(!usable(160, 120, 6000, 4000));
        assert!(usable(1536, 1024, 6000, 4000));
        assert!(usable(1024, 1536, 6000, 4000));
        assert!(usable(640, 480, 640, 480));
    }
}

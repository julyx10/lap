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
    _flip: i32,
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

fn is_same_size_embedded_jpeg(thumb: &RawImageBlob, raw_width: u32, raw_height: u32) -> bool {
    if thumb.format != LIBRAW_THUMBNAIL_JPEG {
        return false;
    }

    let width_delta = thumb.width.abs_diff(raw_width);
    let height_delta = thumb.height.abs_diff(raw_height);

    width_delta.saturating_mul(100) <= raw_width.max(1)
        && height_delta.saturating_mul(100) <= raw_height.max(1)
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
            iso_speed: (out.iso_speed > 0.0).then(|| format_float(out.iso_speed)),
            shutter: (out.shutter > 0.0).then(|| format_shutter_speed(out.shutter)),
            aperture: (out.aperture > 0.0).then(|| format!("f/{}", format_float(out.aperture))),
            focal_len: (out.focal_len > 0.0).then(|| format!("{} mm", format_float(out.focal_len))),
            flash_used: (out.flash_used != 0.0).then(|| {
                if out.flash_used > 0.0 {
                    "Fired".to_string()
                } else {
                    "Not fired".to_string()
                }
            }),
            lens_make: c_char_array_to_string(&out.lens_make),
            lens_model: c_char_array_to_string(&out.lens_model).or_else(|| {
                format_lens_model_from_numbers(
                    out.min_focal,
                    out.max_focal,
                    out.max_ap_min_focal,
                    out.max_ap_max_focal,
                )
            }),
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
            _flip: out.flip,
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

    fn render_preview(&mut self) -> Result<RawImageBlob, String> {
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

        let ret = unsafe { lap_libraw_render_preview(self.raw, 0, 0, &mut out) };
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
            _flip: out.flip,
            data,
        })
    }
}

/// Metadata extracted from a RAW file via LibRaw.
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

fn format_float(value: f32) -> String {
    if value.fract().abs() < 0.05 {
        format!("{:.0}", value)
    } else {
        format!("{:.1}", value)
    }
}

fn format_shutter_speed(shutter: f32) -> String {
    if shutter >= 1.0 {
        format!("{} s", format_float(shutter))
    } else {
        let denom = (1.0 / shutter).round();
        format!("1/{} s", denom)
    }
}

fn format_lens_model_from_numbers(
    min_focal: f32,
    max_focal: f32,
    max_ap_min_focal: f32,
    max_ap_max_focal: f32,
) -> Option<String> {
    if min_focal <= 0.0 || max_focal <= 0.0 || max_ap_min_focal <= 0.0 || max_ap_max_focal <= 0.0 {
        return None;
    }

    let focal = if (min_focal - max_focal).abs() < 0.05 {
        format!("{} mm", format_float(min_focal))
    } else {
        format!("{}-{} mm", format_float(min_focal), format_float(max_focal))
    };
    let aperture = if (max_ap_min_focal - max_ap_max_focal).abs() < 0.05 {
        format!("f/{}", format_float(max_ap_min_focal))
    } else {
        format!(
            "f/{}-{}",
            format_float(max_ap_min_focal),
            format_float(max_ap_max_focal)
        )
    };
    Some(format!("{} {}", focal, aperture))
}

fn render_processed_preview(file_path: &str, max_edge: u32) -> Result<Vec<u8>, String> {
    let mut raw = RawHandle::open(file_path)?;
    let rendered = raw.render_preview()?;
    let image = decode_processed_image(&rendered)?;
    let image = if max_edge > 0 {
        image.resize(max_edge, max_edge, image::imageops::FilterType::Lanczos3)
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

pub fn get_raw_meta(file_path: &str) -> Result<RawMeta, String> {
    RawHandle::open(file_path)?.meta()
}

/// Read the EXIF Orientation tag from in-memory JPEG bytes.
/// Returns 1 (normal) when absent or unparseable.
fn jpeg_exif_orientation(data: &[u8]) -> i32 {
    let mut cursor = Cursor::new(data);
    let exif = match exif::Reader::new().read_from_container(&mut cursor) {
        Ok(exif) => exif,
        Err(_) => return 1,
    };
    exif.get_field(exif::Tag::Orientation, exif::In::PRIMARY)
        .and_then(|field| field.value.get_uint(0))
        .map(|v| v as i32)
        .unwrap_or(1)
}

pub fn get_raw_preview_image(file_path: &str) -> Result<Option<Vec<u8>>, String> {
    let mut raw = RawHandle::open(file_path)?;
    let (raw_width, raw_height, _) = raw.dimensions_with_flip()?;
    let thumbs = raw.extract_thumbnails();

    // Try embedded full-size JPEG first (camera-processed, correct colors)
    for thumb in &thumbs {
        if is_same_size_embedded_jpeg(thumb, raw_width, raw_height) {
            // Use the JPEG's own EXIF orientation — most reliable source
            let orient = jpeg_exif_orientation(&thumb.data);
            if let Ok(image) = image::load_from_memory(&thumb.data) {
                let image = orient_image(image, orient);
                return encode_as_jpeg(&image).map(Some);
            }
        }
    }

    // Processed preview: LibRaw dcraw_process auto-rotates, correct WB
    match render_processed_preview(file_path, 4096) {
        Ok(bytes) => Ok(Some(bytes)),
        Err(_) => Ok(None),
    }
}

pub fn get_raw_thumbnail(file_path: &str, thumbnail_size: u32) -> Result<Option<Vec<u8>>, String> {
    // Always use dcraw_process with half_size for thumbnails.
    // - Guaranteed correct rotation (LibRaw auto-rotates)
    // - Guaranteed correct colors (full WB pipeline)
    // - 4x faster than full decode (half_size=1)
    // Embedded thumbnails have unreliable rotation across camera brands.
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

    let ret = unsafe { lap_libraw_render_preview(raw.raw, 1, 1, &mut out) };
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
            _flip: out.flip,
            data,
        };

        if let Ok(image) = decode_processed_image(&blob) {
            let thumbnail = image.thumbnail(u32::MAX, thumbnail_size);
            return encode_as_jpeg(&thumbnail).map(Some);
        } else {
            eprintln!(
                "LibRaw decode_processed_image failed for {}, falling back to embedded thumbnail",
                file_path
            );
        }
    } else {
        if !out.data.is_null() {
            unsafe { lap_libraw_free_buffer(out.data) };
        }
        eprintln!(
            "LibRaw dcraw_process failed for {} (likely HE/HE* NEF), falling back to embedded thumbnail",
            file_path
        );
    }

    // Reopen the file for thumbnail extraction — the handle used for
    // render_preview may be in an undefined state after a data error.
    let mut raw = RawHandle::open(file_path)?;
    let thumbs = raw.extract_thumbnails();
    let best = thumbs
        .iter()
        .filter(|thumb| thumb.format == LIBRAW_THUMBNAIL_JPEG && !thumb.data.is_empty())
        .max_by_key(|thumb| {
            let max_edge = thumb.width.max(thumb.height);
            (max_edge >= thumbnail_size, max_edge)
        });

    if let Some(thumb) = best {
        let orient = jpeg_exif_orientation(&thumb.data);
        if let Ok(image) = image::load_from_memory(&thumb.data) {
            let image = orient_image(image, orient);
            let thumbnail = image.thumbnail(u32::MAX, thumbnail_size);
            return encode_as_jpeg(&thumbnail).map(Some);
        }
    }

    Ok(None)
}

pub fn is_tiff_path(file_path: &str) -> bool {
    matches!(
        file_extension(file_path).as_deref(),
        Some("tif") | Some("tiff")
    )
}

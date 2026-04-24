use std::ffi::CString;
use std::fs;
use std::os::raw::{c_char, c_int, c_void};
use std::ptr;
use std::slice;

use image::DynamicImage;

use crate::t_image::resize_dynamic_image_to_jpeg;

// Minimal libheif FFI for decoding the primary image to RGB.
// We keep this narrow to avoid pulling bindgen into the build.

#[repr(C)]
#[derive(Clone, Copy)]
struct HeifError {
    code: c_int,
    subcode: c_int,
    message: *const c_char,
}

#[repr(C)]
struct HeifContext(c_void);
#[repr(C)]
struct HeifImageHandle(c_void);
#[repr(C)]
struct HeifImage(c_void);

// libheif v1.21.x enums from heif_image.h
// heif_colorspace_RGB
const HEIF_COLORSPACE_RGB: c_int = 1;
// heif_chroma_interleaved_RGB
const HEIF_CHROMA_INTERLEAVED_RGB: c_int = 10;
// heif_chroma_interleaved_RGBA
const HEIF_CHROMA_INTERLEAVED_RGBA: c_int = 11;
// heif_channel_interleaved
const HEIF_CHANNEL_INTERLEAVED: c_int = 10;

unsafe extern "C" {
    fn heif_context_alloc() -> *mut HeifContext;
    fn heif_context_free(ctx: *mut HeifContext);
    fn heif_context_read_from_file(
        ctx: *mut HeifContext,
        filename: *const c_char,
        options: *const c_void,
    ) -> HeifError;
    fn heif_context_get_primary_image_handle(
        ctx: *mut HeifContext,
        handle: *mut *mut HeifImageHandle,
    ) -> HeifError;
    fn heif_image_handle_release(handle: *mut HeifImageHandle);
    fn heif_image_handle_get_width(handle: *const HeifImageHandle) -> c_int;
    fn heif_image_handle_get_height(handle: *const HeifImageHandle) -> c_int;
    fn heif_image_handle_has_alpha_channel(handle: *const HeifImageHandle) -> c_int;

    fn heif_decode_image(
        handle: *const HeifImageHandle,
        out_img: *mut *mut HeifImage,
        colorspace: c_int,
        chroma: c_int,
        options: *const c_void,
    ) -> HeifError;
    fn heif_image_release(img: *mut HeifImage);

    fn heif_image_get_width(img: *const HeifImage, channel: c_int) -> c_int;
    fn heif_image_get_height(img: *const HeifImage, channel: c_int) -> c_int;
    fn heif_image_get_plane_readonly(
        img: *const HeifImage,
        channel: c_int,
        out_stride: *mut c_int,
    ) -> *const u8;
}

fn fmt_heif_error(err: HeifError) -> String {
    if err.code == 0 {
        return "ok".to_string();
    }
    unsafe {
        let msg: String = if err.message.is_null() {
            String::new()
        } else {
            std::ffi::CStr::from_ptr(err.message)
                .to_string_lossy()
                .into_owned()
        };
        format!(
            "libheif error code={} subcode={} msg={}",
            err.code, err.subcode, msg
        )
    }
}

fn mark_heif_hit(kind: &str, file_path: &str) {
    let path = std::env::temp_dir().join(format!("lap-heif-{}-hit.txt", kind));
    let _ = fs::write(path, file_path);
}

fn decode_primary_rgb(file_path: &str) -> Result<(Vec<u8>, u32, u32, u32), String> {
    let c_path = CString::new(file_path).map_err(|_| "Invalid file path".to_string())?;
    unsafe {
        let ctx = heif_context_alloc();
        if ctx.is_null() {
            return Err("Failed to allocate heif context".to_string());
        }
        struct CtxGuard(*mut HeifContext);
        impl Drop for CtxGuard {
            fn drop(&mut self) {
                unsafe { heif_context_free(self.0) }
            }
        }
        let _ctx_guard = CtxGuard(ctx);

        let err = heif_context_read_from_file(ctx, c_path.as_ptr(), ptr::null());
        if err.code != 0 {
            return Err(fmt_heif_error(err));
        }

        let mut handle: *mut HeifImageHandle = ptr::null_mut();
        let err = heif_context_get_primary_image_handle(ctx, &mut handle);
        if err.code != 0 || handle.is_null() {
            return Err(fmt_heif_error(err));
        }
        struct HandleGuard(*mut HeifImageHandle);
        impl Drop for HandleGuard {
            fn drop(&mut self) {
                unsafe { heif_image_handle_release(self.0) }
            }
        }
        let _handle_guard = HandleGuard(handle);

        let handle_w = heif_image_handle_get_width(handle);
        let handle_h = heif_image_handle_get_height(handle);
        let has_alpha = heif_image_handle_has_alpha_channel(handle) != 0;
        eprintln!("[heif] file: {}", file_path);
        eprintln!(
            "[heif] handle: {}x{}, has_alpha: {}",
            handle_w, handle_h, has_alpha
        );

        let mut img: *mut HeifImage = ptr::null_mut();
        let chroma = if has_alpha {
            HEIF_CHROMA_INTERLEAVED_RGBA
        } else {
            HEIF_CHROMA_INTERLEAVED_RGB
        };
        let err = heif_decode_image(handle, &mut img, HEIF_COLORSPACE_RGB, chroma, ptr::null());
        eprintln!("[heif] decode err.code: {}", err.code);
        if err.code != 0 || img.is_null() {
            return Err(fmt_heif_error(err));
        }
        struct ImgGuard(*mut HeifImage);
        impl Drop for ImgGuard {
            fn drop(&mut self) {
                unsafe { heif_image_release(self.0) }
            }
        }
        let _img_guard = ImgGuard(img);

        let width = heif_image_get_width(img, HEIF_CHANNEL_INTERLEAVED).max(0) as u32;
        let height = heif_image_get_height(img, HEIF_CHANNEL_INTERLEAVED).max(0) as u32;
        if width == 0 || height == 0 {
            return Err("libheif returned empty dimensions".to_string());
        }

        let mut stride: c_int = 0;
        let ptr_plane = heif_image_get_plane_readonly(img, HEIF_CHANNEL_INTERLEAVED, &mut stride);
        if ptr_plane.is_null() || stride <= 0 {
            return Err("libheif returned empty plane".to_string());
        }
        eprintln!(
            "[heif] decoded: {}x{}, stride: {}, bpp: {}",
            width,
            height,
            stride,
            if has_alpha { 4 } else { 3 }
        );
        if handle_w != width as c_int || handle_h != height as c_int {
            eprintln!(
                "[heif] size mismatch: handle={}x{} decoded={}x{}",
                handle_w, handle_h, width, height
            );
        }

        let stride_u = stride as u32;
        let decoded_row_bytes = width.saturating_mul(if has_alpha { 4 } else { 3 });
        if stride_u < decoded_row_bytes {
            return Err("libheif returned invalid stride".to_string());
        }

        let src = slice::from_raw_parts(ptr_plane, (stride_u * height) as usize);
        let mut out = vec![0u8; (width * height * 3) as usize];
        for y in 0..height {
            let src_off = (y * stride_u) as usize;
            let dst_off = (y * width * 3) as usize;
            if has_alpha {
                let src_row = &src[src_off..src_off + decoded_row_bytes as usize];
                let dst_row = &mut out[dst_off..dst_off + (width * 3) as usize];
                for (i, pixel) in src_row.chunks_exact(4).enumerate() {
                    dst_row[i * 3..i * 3 + 3].copy_from_slice(&pixel[0..3]);
                }
            } else {
                out[dst_off..dst_off + (width * 3) as usize]
                    .copy_from_slice(&src[src_off..src_off + (width * 3) as usize]);
            }
        }

        Ok((out, width, height, width * 3))
    }
}

pub fn get_heif_thumbnail(
    file_path: &str,
    _orientation: i32,
    thumbnail_size: u32,
) -> Result<Option<Vec<u8>>, String> {
    mark_heif_hit("thumbnail", file_path);
    let (rgb, width, height, _row_bytes) = decode_primary_rgb(file_path)?;
    // Build a DynamicImage to reuse existing orientation + alpha handling logic.
    // libheif decode gives us RGB, no alpha here.
    let img = image::RgbImage::from_raw(width, height, rgb)
        .ok_or_else(|| "Failed to build RGB image from libheif buffer".to_string())?;
    let dyn_img = DynamicImage::ImageRgb8(img);
    // libheif already applies HEIF geometric transformations (rotation/mirroring/crop).
    resize_dynamic_image_to_jpeg(dyn_img, 1, thumbnail_size).map(Some)
}

pub fn get_heif_preview(
    file_path: &str,
    _orientation: i32,
    max_size: u32,
) -> Result<Option<Vec<u8>>, String> {
    mark_heif_hit("preview", file_path);
    let (rgb, width, height, _row_bytes) = decode_primary_rgb(file_path)?;
    let img = image::RgbImage::from_raw(width, height, rgb)
        .ok_or_else(|| "Failed to build RGB image from libheif buffer".to_string())?;
    // Preview path: keep it JPEG encoded at up to max_size (same as thumbnail sizing semantics).
    // libheif already applies HEIF geometric transformations (rotation/mirroring/crop).
    resize_dynamic_image_to_jpeg(DynamicImage::ImageRgb8(img), 1, max_size).map(Some)
}

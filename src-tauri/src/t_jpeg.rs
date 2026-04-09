use image::RgbImage;
use std::ffi::CStr;
use std::os::raw::{c_char, c_int, c_ulong};

#[link(name = "lap_libraw_shim", kind = "static")]
unsafe extern "C" {
    fn lap_jpeg_encode_rgb8(
        rgb_data: *const u8,
        width: u32,
        height: u32,
        quality: c_int,
        out_data: *mut *mut u8,
        out_len: *mut c_ulong,
        err_buf: *mut c_char,
        err_buf_len: usize,
    ) -> c_int;
    fn lap_jpeg_free_buffer(data: *mut u8);
}

pub fn encode_rgb8(rgb: &RgbImage, quality: u8) -> Result<Vec<u8>, String> {
    let mut out_data: *mut u8 = std::ptr::null_mut();
    let mut out_len: c_ulong = 0;
    let mut err_buf = vec![0 as c_char; 512];

    let ok = unsafe {
        lap_jpeg_encode_rgb8(
            rgb.as_raw().as_ptr(),
            rgb.width(),
            rgb.height(),
            quality as c_int,
            &mut out_data,
            &mut out_len,
            err_buf.as_mut_ptr(),
            err_buf.len(),
        )
    };

    if ok == 0 {
        let err = unsafe { CStr::from_ptr(err_buf.as_ptr()) }
            .to_string_lossy()
            .trim()
            .to_string();
        return Err(if err.is_empty() {
            "libjpeg-turbo encode failed".to_string()
        } else {
            format!("libjpeg-turbo encode failed: {}", err)
        });
    }

    if out_data.is_null() || out_len == 0 {
        return Err("libjpeg-turbo returned empty JPEG output".to_string());
    }

    let bytes = unsafe {
        let slice = std::slice::from_raw_parts(out_data, out_len as usize);
        let result = slice.to_vec();
        lap_jpeg_free_buffer(out_data);
        result
    };

    Ok(bytes)
}

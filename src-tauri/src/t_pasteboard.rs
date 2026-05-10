/// macOS drag‑pasteboard image URL extraction.
///
/// On macOS we read `NSPasteboardNameDrag` which is scoped to the
/// current drag operation — it never returns stale data.

#[cfg(target_os = "macos")]
mod imp {
    use std::ffi::{c_char, CStr};

    unsafe extern "C" {
        fn lap_get_drag_image_url() -> *const c_char;
        fn lap_free_string(ptr: *const c_char);
    }

    pub fn get_drag_image_url() -> Option<String> {
        let ptr = unsafe { lap_get_drag_image_url() };
        if ptr.is_null() {
            return None;
        }
        let url = unsafe { CStr::from_ptr(ptr) }.to_string_lossy().to_string();
        unsafe { lap_free_string(ptr) };
        Some(url)
    }
}

#[cfg(not(target_os = "macos"))]
mod imp {
    pub fn get_drag_image_url() -> Option<String> {
        None
    }
}

pub fn get_drag_image_url() -> Option<String> {
    imp::get_drag_image_url()
}

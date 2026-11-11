use tauri::{Builder, Wry};

use crate::{t_image, t_sqlite};
use crate::t_raw_display::{RawDisplayOptions, RawPreviewMode};

fn text_response(status: http::StatusCode, body: &str) -> http::Response<Vec<u8>> {
    http::Response::builder()
        .status(status)
        .header(http::header::CONTENT_TYPE, "text/plain")
        .header(http::header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
        .header(http::header::ACCESS_CONTROL_ALLOW_METHODS, "GET, OPTIONS")
        .header(http::header::ACCESS_CONTROL_ALLOW_HEADERS, "*")
        .body(body.as_bytes().to_vec())
        .unwrap()
}

pub(crate) fn detect_image_mime(data: &[u8]) -> &'static str {
    if data.starts_with(&[0x89, 0x50, 0x4E, 0x47]) {
        "image/png"
    } else if data.starts_with(&[0xFF, 0xD8, 0xFF]) {
        "image/jpeg"
    } else if data.starts_with(&[0x47, 0x49, 0x46, 0x38]) {
        "image/gif"
    } else if data.starts_with(&[0x49, 0x49, 0x2A, 0x00])
        || data.starts_with(&[0x4D, 0x4D, 0x00, 0x2A])
        || data.starts_with(&[0x4D, 0x4D, 0x00, 0x2B])
        || data.starts_with(&[0x49, 0x49, 0x2B, 0x00])
    {
        "image/tiff"
    } else if data.starts_with(b"BM") {
        "image/bmp"
    } else if data.starts_with(b"RIFF") && data.get(8..12) == Some(b"WEBP") {
        "image/webp"
    } else {
        detect_isobmff_mime(data)
    }
}

fn detect_isobmff_mime(data: &[u8]) -> &'static str {
    if data.len() < 16 || &data[4..8] != b"ftyp" {
        return "application/octet-stream";
    }

    let box_size = u32::from_be_bytes([data[0], data[1], data[2], data[3]]) as usize;
    if box_size < 16 || box_size > data.len() {
        return "application/octet-stream";
    }

    match &data[8..12] {
        b"avif" | b"avis" => "image/avif",
        b"heic" | b"heix" | b"hevc" | b"hevx" => "image/heic",
        b"mif1" | b"heif" => "image/heif",
        _ => {
            for brand in data[16..box_size].chunks(4) {
                if brand.len() < 4 {
                    break;
                }
                match brand {
                    b"avif" | b"avis" => return "image/avif",
                    b"heic" | b"heix" | b"hevc" | b"hevx" => {
                        return "image/heic";
                    }
                    b"mif1" | b"heif" => return "image/heif",
                    _ => {}
                }
            }
            "application/octet-stream"
        }
    }
}

fn image_response(data: Vec<u8>) -> http::Response<Vec<u8>> {
    http::Response::builder()
        .status(http::StatusCode::OK)
        .header(http::header::CONTENT_TYPE, detect_image_mime(&data))
        .header(http::header::CACHE_CONTROL, "max-age=31536000, immutable")
        .header(http::header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
        .header(http::header::ACCESS_CONTROL_ALLOW_METHODS, "GET, OPTIONS")
        .header(http::header::ACCESS_CONTROL_ALLOW_HEADERS, "*")
        .body(data)
        .unwrap()
}

fn raw_display_options(query: Option<&str>) -> RawDisplayOptions {
    let mut options = RawDisplayOptions::default();
    for pair in query.unwrap_or_default().split('&') {
        match pair {
            "rawPreviewMode=embedded" => options.mode = RawPreviewMode::Embedded,
            "rawPreviewMode=rendered" => options.mode = RawPreviewMode::Rendered,
            "rawAutoBright=true" => options.auto_bright = true,
            "rawPairDisplay=jpeg" => options.prefer_pair = true,
            _ => {}
        }
    }
    options
}

fn raw_image_response(data: Vec<u8>) -> http::Response<Vec<u8>> {
    let mut response = image_response(data);
    // Revalidate external companion changes even when the RAW itself is unchanged.
    response.headers_mut().insert(http::header::CACHE_CONTROL, http::HeaderValue::from_static("no-cache"));
    response
}

fn raw_preview_response(data: Vec<u8>, source: &'static str, unavailable: bool, pair: &'static str) -> http::Response<Vec<u8>> {
    let mut response = raw_image_response(data);
    let headers = response.headers_mut();
    headers.insert("X-Raw-Source", http::HeaderValue::from_static(source));
    headers.insert("X-Raw-Embedded-Unavailable", http::HeaderValue::from_static(if unavailable { "true" } else { "false" }));
    headers.insert("X-Raw-Pair", http::HeaderValue::from_static(pair));
    headers.insert(http::header::ACCESS_CONTROL_EXPOSE_HEADERS, http::HeaderValue::from_static("X-Raw-Source, X-Raw-Embedded-Unavailable, X-Raw-Pair"));
    response
}

/// Parses the first range of a `Range: bytes=...` header into an inclusive
/// `(start, end)` pair. `None` means the range cannot be satisfied.
fn parse_byte_range(header: &str, len: u64) -> Option<(u64, u64)> {
    let spec = header.trim().strip_prefix("bytes=")?.split(',').next()?.trim();
    let (start, end) = spec.split_once('-')?;
    if len == 0 {
        return None;
    }
    let (start, end) = if start.is_empty() {
        let suffix: u64 = end.trim().parse().ok()?;
        if suffix == 0 {
            return None;
        }
        (len.saturating_sub(suffix), len - 1)
    } else {
        let start: u64 = start.trim().parse().ok()?;
        let end = if end.trim().is_empty() { len - 1 } else { end.trim().parse::<u64>().ok()?.min(len - 1) };
        (start, end)
    };
    (start < len && start <= end).then_some((start, end))
}

/// Serves `asset://` like Tauri's built-in handler, but off the UI thread.
/// The built-in handler blocks the scheme callback until the file is read; on
/// macOS that callback runs on the main thread, so an online-only (iCloud)
/// original froze the whole app while it downloaded.
fn asset_file_response(path: &str, range: Option<&str>, head: bool, origin: &str) -> http::Response<Vec<u8>> {
    use std::io::{Read, Seek, SeekFrom};
    const MAX_RANGE_LEN: u64 = 1000 * 1024;

    let status_only = |status: http::StatusCode| {
        http::Response::builder()
            .status(status)
            .header(http::header::ACCESS_CONTROL_ALLOW_ORIGIN, origin)
            .body(Vec::new())
            .unwrap()
    };
    let mut file = match std::fs::File::open(path) {
        Ok(file) => file,
        Err(e) => {
            return status_only(match e.kind() {
                std::io::ErrorKind::NotFound => http::StatusCode::NOT_FOUND,
                std::io::ErrorKind::PermissionDenied => http::StatusCode::FORBIDDEN,
                _ => http::StatusCode::INTERNAL_SERVER_ERROR,
            });
        }
    };
    let read = |file: &mut std::fs::File, start: u64, nbytes: u64| -> std::io::Result<Vec<u8>> {
        let mut buf = Vec::with_capacity(nbytes as usize);
        file.seek(SeekFrom::Start(start))?;
        file.take(nbytes).read_to_end(&mut buf)?;
        Ok(buf)
    };
    let Ok(len) = file.metadata().map(|m| m.len()) else {
        return status_only(http::StatusCode::INTERNAL_SERVER_ERROR);
    };
    let Ok(magic) = read(&mut file, 0, len.min(8192)) else {
        return status_only(http::StatusCode::INTERNAL_SERVER_ERROR);
    };
    let mime = tauri::utils::mime_type::MimeType::parse(&magic, path);
    let builder = http::Response::builder()
        .header(http::header::ACCESS_CONTROL_ALLOW_ORIGIN, origin)
        .header(http::header::CONTENT_TYPE, mime);

    if let Some(range) = range {
        let Some((start, end)) = parse_byte_range(range, len) else {
            return http::Response::builder()
                .status(http::StatusCode::RANGE_NOT_SATISFIABLE)
                .header(http::header::ACCESS_CONTROL_ALLOW_ORIGIN, origin)
                .header(http::header::CONTENT_RANGE, format!("bytes */{len}"))
                .body(Vec::new())
                .unwrap();
        };
        let end = start + (end - start).min(MAX_RANGE_LEN - 1);
        let Ok(buf) = read(&mut file, start, end + 1 - start) else {
            return status_only(http::StatusCode::INTERNAL_SERVER_ERROR);
        };
        return builder
            .status(http::StatusCode::PARTIAL_CONTENT)
            .header(http::header::ACCEPT_RANGES, "bytes")
            .header(http::header::ACCESS_CONTROL_EXPOSE_HEADERS, "content-range")
            .header(http::header::CONTENT_RANGE, format!("bytes {start}-{end}/{len}"))
            .header(http::header::CONTENT_LENGTH, end + 1 - start)
            .body(buf)
            .unwrap();
    }
    if head {
        return builder.header(http::header::CONTENT_LENGTH, len).body(Vec::new()).unwrap();
    }
    let body = if len <= 8192 { Ok(magic) } else { read(&mut file, 0, len) };
    match body {
        Ok(buf) => builder.header(http::header::CONTENT_LENGTH, len).body(buf).unwrap(),
        Err(_) => status_only(http::StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub fn register_protocols(builder: Builder<Wry>) -> Builder<Wry> {
    builder
        .register_asynchronous_uri_scheme_protocol("asset", |ctx, request, responder| {
            use tauri::Manager;
            // Registering "asset" replaces Tauri's built-in handler; keep its
            // URL format, scope check and range behaviour.
            let raw_path = request.uri().path();
            let path = percent_encoding::percent_decode(raw_path.strip_prefix('/').unwrap_or(raw_path).as_bytes())
                .decode_utf8_lossy()
                .to_string();
            let origin = request.headers().get(http::header::ORIGIN)
                .and_then(|v| v.to_str().ok()).unwrap_or("*").to_string();
            let traversal = std::path::Path::new(&path).components()
                .any(|c| matches!(c, std::path::Component::ParentDir));
            if path.is_empty() || traversal || !ctx.app_handle().asset_protocol_scope().is_allowed(&path) {
                responder.respond(http::Response::builder()
                    .status(http::StatusCode::FORBIDDEN)
                    .header(http::header::ACCESS_CONTROL_ALLOW_ORIGIN, origin)
                    .body(Vec::new())
                    .unwrap());
                return;
            }
            let range = request.headers().get(http::header::RANGE)
                .and_then(|v| v.to_str().ok()).map(str::to_string);
            let head = request.method() == http::Method::HEAD;
            tauri::async_runtime::spawn(async move {
                let fallback_origin = origin.clone();
                let response = tauri::async_runtime::spawn_blocking(move || {
                    asset_file_response(&path, range.as_deref(), head, &origin)
                }).await.unwrap_or_else(|_| http::Response::builder()
                    .status(http::StatusCode::INTERNAL_SERVER_ERROR)
                    .header(http::header::ACCESS_CONTROL_ALLOW_ORIGIN, fallback_origin)
                    .body(Vec::new())
                    .unwrap());
                responder.respond(response);
            });
        })
        .register_asynchronous_uri_scheme_protocol("thumb", |_ctx, request, responder| {
            // URL format: thumb://localhost/{library_id}/{file_id}
            // library_id is also used for cache namespace isolation.
            let path = request.uri().path();
            let mut segments = path.trim_start_matches('/').split('/');
            let library_id = segments.next().unwrap_or("default").to_string();
            let file_id_str = segments.next().unwrap_or("");
            let file_id: i64 = file_id_str.parse().unwrap_or(0);

            if library_id.is_empty() || file_id <= 0 {
                responder.respond(text_response(
                    http::StatusCode::BAD_REQUEST,
                    "invalid file_id",
                ));
                return;
            }

            let options = raw_display_options(request.uri().query());
            let query = request.uri().query().unwrap_or_default();
            let thumbnail_size = query.split('&').find_map(|p| p.strip_prefix("size=").and_then(|v| v.parse::<u32>().ok()))
                .unwrap_or(512).clamp(64,2048);
            let repair = query.split('&').any(|p| p == "repair=true");
            let app = _ctx.app_handle().clone();
            tauri::async_runtime::spawn(async move {
                if !repair {
                    let lookup_library = library_id.clone();
                    let cached = tauri::async_runtime::spawn_blocking(move || {
                        crate::t_cmds::with_current_library(&lookup_library, || {
                            let file = t_sqlite::AFile::get_file_info(file_id)?.ok_or("File not found")?;
                            let path = file.file_path.as_deref().unwrap_or_default();
                            if let Some(thumb) = t_sqlite::AThumb::get_thumb_if_available(file_id, path, false, file.album_accessible)? {
                                if let Some(data) = thumb.thumb_data { return Ok(Some((data, file.file_type == Some(3)))); }
                            }
                            Ok(None)
                        })
                    }).await;
                    if let Ok(Ok(Some((data, raw)))) = cached { responder.respond(if raw { raw_image_response(data) } else { image_response(data) }); return; }
                }
                let permits = t_sqlite::thumb_background_generation_permits();
                let Ok(permit) = permits.acquire_owned().await else {
                    responder.respond(text_response(http::StatusCode::SERVICE_UNAVAILABLE, "thumbnail queue closed")); return;
                };
                let result = tauri::async_runtime::spawn_blocking(move || {
                    let _permit = permit;
                    crate::t_cmds::with_current_library(&library_id, || {
                        let file = t_sqlite::AFile::get_file_info(file_id)?.ok_or("File not found")?;
                        let previous = t_sqlite::AThumb::fetch_record(file_id)?.and_then(|(thumb, _)| thumb.id);
                        let thumb = t_sqlite::AThumb::get_or_create_thumb(file_id, file.file_path.as_deref().unwrap_or_default(),
                            file.file_type.unwrap_or(0), file.e_orientation.unwrap_or(1) as i32,
                            thumbnail_size, options, repair && t_sqlite::AThumb::auto_repair_allowed(&file, thumbnail_size, options), file.duration.map(|d| d as u64), None, repair)?;
                        if repair && thumb.as_ref().is_some_and(|thumb| thumb.error_code == 0 && thumb.thumb_data.is_some()
                            && previous != thumb.id) {
                            use tauri::Emitter;
                            let _ = app.emit("thumbnail_ready", serde_json::json!({ "library_id": library_id, "album_id": file.album_id,
                                "file_ids": [file_id], "invalidate": true, "metadata_changed": false }));
                        }
                        if let Some(thumb) = thumb {
                            if let Some(data) = thumb.thumb_data { return Ok(Some((data, file.file_type == Some(3)))); }
                        }
                        Ok(None)
                    })
                }).await;
                responder.respond(match result {
                    Ok(Ok(Some((data, raw)))) => if raw { raw_image_response(data) } else { image_response(data) },
                    _ => text_response(http::StatusCode::NOT_FOUND, "thumbnail not found"),
                });
            });
        })
        .register_asynchronous_uri_scheme_protocol("preview", |_ctx, request, responder| {
            // URL format: preview://localhost/{library_id}/{file_id}?rawPreviewMode=embedded
            // library_id isolates browser caches and guards stale requests after a library switch
            let path = request.uri().path();
            let file_id_str = path.rsplit('/').next().unwrap_or("");
            let file_id: i64 = file_id_str.parse().unwrap_or(0);
            let options = raw_display_options(request.uri().query());
            let cached_only = request.uri().query().unwrap_or_default().split('&')
                .any(|param| param == "cachedOnly=true");
            let for_editing = request.uri().query().unwrap_or_default().split('&')
                .any(|param| param == "forEditing=true");
            let raw_request = request.uri().query().unwrap_or_default().split('&')
                .find_map(|param| param.strip_prefix("rawViewerId="))
                .map(t_image::register_raw_preview_request);

            if file_id <= 0 {
                responder.respond(text_response(
                    http::StatusCode::BAD_REQUEST,
                    "invalid file_id",
                ));
                return;
            }

            let library_id = path.trim_start_matches('/').split('/').next().unwrap_or("default").to_string();
            tauri::async_runtime::spawn(async move {
                // Scheme callbacks can run on the UI thread. Never wait for a
                // library lock, configuration IO or SQLite there.
                let lookup = tauri::async_runtime::spawn_blocking(move || {
                    crate::t_cmds::with_current_library(&library_id, || {
                        let file = t_sqlite::AFile::get_file_info(file_id)?
                            .ok_or_else(|| "File not found".to_string())?;
                        let companion = if !options.prefer_pair {
                            None
                        } else if cached_only && file.media_subtype.as_deref() == Some("raw_jpeg_pair") {
                            file.live_photo_video_id.and_then(|id| t_sqlite::AFile::get_file_info(id).ok().flatten())
                        } else {
                            crate::t_raw_display::paired_file(file_id, options)
                        };
                        Ok((file, companion))
                    })
                }).await;
                let (file, companion) = match lookup {
                    Ok(Ok(files)) => files,
                    _ => {
                        responder.respond(text_response(http::StatusCode::NOT_FOUND, "file not found"));
                        return;
                    }
                };
                let is_raw = file.file_type == Some(3);
                let file_path = match file.file_path {
                    Some(path) if !path.is_empty() => path,
                    _ => {
                        responder.respond(text_response(http::StatusCode::NOT_FOUND, "file path not found"));
                        return;
                    }
                };
                if cached_only {
                    let paired_cache = options.prefer_pair.then(|| companion.as_ref()
                        .and_then(|file| file.file_path.as_deref())
                        .and_then(|path| t_image::get_cached_preview_bytes(path, options))).flatten();
                    let data = paired_cache.or_else(|| t_image::get_cached_preview_bytes(&file_path, options));
                    let mut response = match data {
                        Some(data) => image_response(data),
                        None => text_response(http::StatusCode::NOT_FOUND, "cached preview not found"),
                    };
                    response.headers_mut().insert(http::header::CACHE_CONTROL, http::HeaderValue::from_static("no-store"));
                    responder.respond(response);
                    return;
                }
                if for_editing {
                    // Use exactly the same pixels (including decoder fallbacks)
                    // that get_edited_image applies the crop and adjustments to.
                    let result = tauri::async_runtime::spawn_blocking(move || {
                        tauri::async_runtime::block_on(t_image::get_generated_preview_bytes(&file_path))
                    }).await;
                    responder.respond(match result {
                        Ok(Ok(Some(data))) => image_response(data),
                        _ => text_response(http::StatusCode::NOT_FOUND, "editable preview not found"),
                    });
                    return;
                }
                let pair_label = companion.as_ref().and_then(|file| file.file_path.as_ref()).map(|path| {
                    if t_image::is_heic_path(path) { "HEIC" } else { "JPEG" }
                }).unwrap_or("");
                if options.prefer_pair {
                    if let Some(path) = companion.and_then(|file| file.file_path) {
                        if let Ok(data) = t_image::get_file_image_bytes_cached(&path, options).await {
                            if image::load_from_memory(&data).is_ok() {
                                responder.respond(raw_preview_response(data, "pair", false, pair_label));
                                return;
                            }
                        }
                    }
                }
                if is_raw {
                    let full_size = request.uri().query().unwrap_or_default().split('&').any(|param| param == "rawFullSize=true");
                    let response = match t_image::get_raw_preview_cached(&file_path, options, full_size, raw_request).await {
                        Ok((data, source, unavailable)) => raw_preview_response(data, source, unavailable, pair_label),
                        Err(_) => text_response(http::StatusCode::NOT_FOUND, "RAW preview not found"),
                    };
                    responder.respond(response);
                    return;
                }
                let response = match t_image::get_file_image_bytes_cached(&file_path, options).await {
                    Ok(data) => image_response(data),
                    Err(_) => text_response(http::StatusCode::NOT_FOUND, "preview not found"),
                };
                responder.respond(response);
            });
        })
}

#[cfg(test)]
mod raw_display_tests {
    use super::*;

    #[test]
    fn raw_display_protocol_parses_modes_and_pair_policy() {
        let options = raw_display_options(Some("v=42&rawPreviewMode=rendered&rawPairDisplay=jpeg"));
        assert_eq!(options.mode, RawPreviewMode::Rendered);
        assert!(options.prefer_pair);
        assert_eq!(raw_display_options(Some("rawPreviewMode=embedded")).mode, RawPreviewMode::Embedded);
        assert!(raw_display_options(Some("rawPreviewMode=embedded&rawAutoBright=true")).auto_bright);
        assert!(!raw_display_options(None).prefer_pair);
    }
}

#[cfg(test)]
mod asset_range_tests {
    use super::parse_byte_range;

    #[test]
    fn parses_ranges_like_the_built_in_asset_handler() {
        assert_eq!(parse_byte_range("bytes=0-499", 1000), Some((0, 499)));
        assert_eq!(parse_byte_range("bytes=500-", 1000), Some((500, 999)));
        assert_eq!(parse_byte_range("bytes=-200", 1000), Some((800, 999)));
        assert_eq!(parse_byte_range("bytes=0-5000", 1000), Some((0, 999)));
        assert_eq!(parse_byte_range("bytes=0-1, 5-9", 1000), Some((0, 1)));
        assert_eq!(parse_byte_range("bytes=1000-", 1000), None);
        assert_eq!(parse_byte_range("bytes=9-3", 1000), None);
        assert_eq!(parse_byte_range("bytes=0-", 0), None);
        assert_eq!(parse_byte_range("items=0-1", 1000), None);
    }
}

use tauri::{Builder, Wry};

use crate::{t_image, t_sqlite};

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

fn detect_image_mime(data: &[u8]) -> &'static str {
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
        b"heic" | b"heix" | b"hevc" | b"hevx" | b"mif1" | b"heif" => "image/heic",
        _ => {
            for brand in data[16..box_size].chunks(4) {
                if brand.len() < 4 {
                    break;
                }
                match brand {
                    b"avif" | b"avis" => return "image/avif",
                    b"heic" | b"heix" | b"hevc" | b"hevx" | b"mif1" | b"heif" => {
                        return "image/heic";
                    }
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

pub fn register_protocols(builder: Builder<Wry>) -> Builder<Wry> {
    builder
        .register_asynchronous_uri_scheme_protocol("thumb", |_ctx, request, responder| {
            // URL format: thumb://localhost/{library_id}/{file_id}
            // library_id is also used for cache namespace isolation.
            let path = request.uri().path();
            let mut segments = path.trim_start_matches('/').split('/');
            let library_id = segments.next().unwrap_or("default").to_string();
            let file_id_str = segments.next().unwrap_or("");
            let file_id: i64 = file_id_str.parse().unwrap_or(0);

            if library_id.is_empty() || file_id <= 0 {
                responder.respond(text_response(http::StatusCode::BAD_REQUEST, "invalid file_id"));
                return;
            }

            let app_handle = _ctx.app_handle().clone();
            tauri::async_runtime::spawn(async move {
                let response = match t_sqlite::AThumb::fetch_raw_for_library(file_id, &library_id) {
                    Ok(Some(data)) => image_response(data),
                    _ => {
                        if let Ok(Some(file)) = t_sqlite::AFile::get_file_info(file_id) {
                            if let Some(file_path) = file.file_path.clone() {
                                let file_type = file.file_type.unwrap_or(0);
                                let orientation = file.e_orientation.unwrap_or(1) as i32;
                                let album_id = file.album_id.unwrap_or(0);
                                let thumbnail_size = 200;
                                t_sqlite::AThumb::schedule_background_generation_for_library(
                                    app_handle,
                                    file_id,
                                    file_path,
                                    file_type,
                                    orientation,
                                    thumbnail_size,
                                    album_id,
                                    false,
                                );
                            }
                        }
                        text_response(http::StatusCode::NOT_FOUND, "thumbnail not found")
                    }
                };
                responder.respond(response);
            });
        })
        .register_asynchronous_uri_scheme_protocol("preview", |_ctx, request, responder| {
            // URL format: preview://localhost/{library_id}/{file_id}
            // library_id is for browser cache isolation only; file_id is the last segment
            let path = request.uri().path();
            let file_id_str = path.rsplit('/').next().unwrap_or("");
            let file_id: i64 = file_id_str.parse().unwrap_or(0);

            if file_id <= 0 {
                responder.respond(text_response(
                    http::StatusCode::BAD_REQUEST,
                    "invalid file_id",
                ));
                return;
            }

            let file = match t_sqlite::AFile::get_file_info(file_id) {
                Ok(Some(file)) => file,
                _ => {
                    responder.respond(text_response(http::StatusCode::NOT_FOUND, "file not found"));
                    return;
                }
            };

            let file_path = match file.file_path {
                Some(path) if !path.is_empty() => path,
                _ => {
                    responder.respond(text_response(
                        http::StatusCode::NOT_FOUND,
                        "file path not found",
                    ));
                    return;
                }
            };

            tauri::async_runtime::spawn(async move {
                let response = match t_image::get_file_image_bytes_cached(&file_path).await {
                    Ok(data) => image_response(data),
                    Err(_) => text_response(http::StatusCode::NOT_FOUND, "preview not found"),
                };
                responder.respond(response);
            });
        })
}

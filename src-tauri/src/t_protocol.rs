use tauri::{Builder, Wry};

use crate::{t_image, t_sqlite};

fn text_response(status: http::StatusCode, body: &str) -> http::Response<Vec<u8>> {
    http::Response::builder()
        .status(status)
        .header(http::header::CONTENT_TYPE, "text/plain")
        .body(body.as_bytes().to_vec())
        .unwrap()
}

fn detect_image_mime(data: &[u8]) -> &'static str {
    if data.starts_with(&[0x89, 0x50, 0x4E, 0x47]) {
        "image/png"
    } else if data.starts_with(&[0xFF, 0xD8, 0xFF]) {
        "image/jpeg"
    } else if data.starts_with(&[0x49, 0x49, 0x2A, 0x00])
        || data.starts_with(&[0x4D, 0x4D, 0x00, 0x2A])
        || data.starts_with(&[0x4D, 0x4D, 0x00, 0x2B])
        || data.starts_with(&[0x49, 0x49, 0x2B, 0x00])
    {
        "image/tiff"
    } else {
        "application/octet-stream"
    }
}

pub fn register_protocols(builder: Builder<Wry>) -> Builder<Wry> {
    builder
        .register_uri_scheme_protocol("thumb", |_ctx, request| {
            // URL format: thumb://localhost/{library_id}/{file_id}
            // library_id is for browser cache isolation only; file_id is the last segment
            let path = request.uri().path();
            let file_id_str = path.rsplit('/').next().unwrap_or("");
            let file_id: i64 = file_id_str.parse().unwrap_or(0);

            if file_id <= 0 {
                return text_response(http::StatusCode::BAD_REQUEST, "invalid file_id");
            }

            match t_sqlite::AThumb::fetch_raw(file_id) {
                Ok(Some(data)) => {
                    http::Response::builder()
                        .status(http::StatusCode::OK)
                        .header(http::header::CONTENT_TYPE, detect_image_mime(&data))
                        .header(http::header::CACHE_CONTROL, "max-age=31536000, immutable")
                        .body(data)
                        .unwrap()
                }
                _ => text_response(http::StatusCode::NOT_FOUND, "thumbnail not found"),
            }
        })
        .register_asynchronous_uri_scheme_protocol("preview", |_ctx, request, responder| {
            // URL format: preview://localhost/{library_id}/{file_id}
            // library_id is for browser cache isolation only; file_id is the last segment
            let path = request.uri().path();
            let file_id_str = path.rsplit('/').next().unwrap_or("");
            let file_id: i64 = file_id_str.parse().unwrap_or(0);

            if file_id <= 0 {
                responder.respond(text_response(http::StatusCode::BAD_REQUEST, "invalid file_id"));
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
                    responder
                        .respond(text_response(http::StatusCode::NOT_FOUND, "file path not found"));
                    return;
                }
            };

            tauri::async_runtime::spawn(async move {
                let response = match t_image::get_file_image_bytes_cached(&file_path).await {
                    Ok(data) => http::Response::builder()
                        .status(http::StatusCode::OK)
                        .header(http::header::CONTENT_TYPE, detect_image_mime(&data))
                        .header(http::header::CACHE_CONTROL, "max-age=31536000, immutable")
                        .body(data)
                        .unwrap(),
                    Err(_) => text_response(http::StatusCode::NOT_FOUND, "preview not found"),
                };
                responder.respond(response);
            });
        })
}

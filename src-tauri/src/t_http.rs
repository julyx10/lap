/**
 * t_http.rs - Local HTTP Server for Media Streaming
 *
 * This module provides a simple, high-performance local HTTP server specifically
 * for serving video files on Linux platforms. It bypasses WebView (WebKitGTK)
 * limitations regarding range requests and asset protocols.
 *
 * Performance:
 * - Uses tokio::io::copy for efficient, zero-buffer streaming from disk to socket.
 * - Supports HTTP 206 Partial Content for precise seeking.
 */
#[cfg(target_os = "linux")]
use once_cell::sync::OnceCell;
#[cfg(target_os = "linux")]
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
#[cfg(target_os = "linux")]
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};

#[cfg(target_os = "linux")]
static VIDEO_HTTP_BASE_URL: OnceCell<String> = OnceCell::new();

#[cfg(target_os = "linux")]
pub fn init_video_http_server() {
    if VIDEO_HTTP_BASE_URL.get().is_some() {
        return;
    }

    let (tx, rx) = std::sync::mpsc::sync_channel(1);
    tauri::async_runtime::spawn(async move {
        let listener = match tokio::net::TcpListener::bind(("127.0.0.1", 0)).await {
            Ok(listener) => listener,
            Err(_) => {
                let _ = tx.send(None);
                return;
            }
        };

        let base_url = match listener.local_addr() {
            Ok(addr) => format!("http://127.0.0.1:{}", addr.port()),
            Err(_) => {
                let _ = tx.send(None);
                return;
            }
        };
        let _ = tx.send(Some(base_url));

        loop {
            let Ok((stream, _)) = listener.accept().await else {
                continue;
            };
            tauri::async_runtime::spawn(async move {
                let _ = handle_video_http_connection(stream).await;
            });
        }
    });

    if let Ok(Some(base_url)) = rx.recv() {
        let _ = VIDEO_HTTP_BASE_URL.set(base_url);
    }
}

#[cfg(target_os = "linux")]
fn video_http_base_url() -> Result<&'static str, String> {
    VIDEO_HTTP_BASE_URL
        .get()
        .map(|s| s.as_str())
        .ok_or_else(|| "video HTTP server not initialized".to_string())
}

#[cfg(target_os = "linux")]
pub fn make_video_stream_url(file_path: &str) -> Result<String, String> {
    let base_url = video_http_base_url()?;
    let encoded = URL_SAFE_NO_PAD.encode(file_path.as_bytes());
    Ok(format!("{}/video?path={}", base_url, encoded))
}

#[cfg(target_os = "linux")]
fn parse_http_range(range_header: &str, file_size: u64) -> Option<(u64, u64)> {
    let range = range_header
        .strip_prefix("bytes=")?
        .split(',')
        .next()?
        .trim();
    let (start_str, end_str) = range.split_once('-')?;

    if start_str.is_empty() {
        let suffix_len = end_str.parse::<u64>().ok()?;
        if suffix_len == 0 {
            return None;
        }
        let start = file_size.saturating_sub(suffix_len);
        let end = file_size.saturating_sub(1);
        return Some((start, end));
    }

    let start = start_str.parse::<u64>().ok()?;
    if start >= file_size {
        return None;
    }

    let end = if end_str.is_empty() {
        file_size.saturating_sub(1)
    } else {
        end_str
            .parse::<u64>()
            .ok()?
            .min(file_size.saturating_sub(1))
    };

    if start > end {
        return None;
    }

    Some((start, end))
}

#[cfg(target_os = "linux")]
fn video_content_type(file_path: &str) -> &'static str {
    match std::path::Path::new(file_path)
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_ascii_lowercase()
        .as_str()
    {
        "mp4" | "m4v" => "video/mp4",
        "mov" => "video/quicktime",
        "webm" => "video/webm",
        "ogv" => "video/ogg",
        _ => "application/octet-stream",
    }
}

#[cfg(target_os = "linux")]
async fn write_http_headers(
    stream: &mut tokio::net::TcpStream,
    status: &str,
    headers: &[(&str, String)],
) -> std::io::Result<()> {
    let mut response = format!("HTTP/1.1 {}\r\n", status);
    for (name, value) in headers {
        response.push_str(name);
        response.push_str(": ");
        response.push_str(value);
        response.push_str("\r\n");
    }
    response.push_str("\r\n");
    stream.write_all(response.as_bytes()).await?;
    Ok(())
}

#[cfg(target_os = "linux")]
async fn handle_video_http_connection(mut stream: tokio::net::TcpStream) -> std::io::Result<()> {
    let mut buf = vec![0u8; 16 * 1024];
    let n = stream.read(&mut buf).await?;
    if n == 0 {
        return Ok(());
    }
    let request = String::from_utf8_lossy(&buf[..n]);
    let mut lines = request.split("\r\n");
    let Some(request_line) = lines.next() else {
        return Ok(());
    };
    let mut request_parts = request_line.split_whitespace();
    let method = request_parts.next().unwrap_or("");
    let target = request_parts.next().unwrap_or("");

    if method == "OPTIONS" {
        return write_http_headers(
            &mut stream,
            "204 No Content",
            &[
                ("Access-Control-Allow-Origin", "*".to_string()),
                (
                    "Access-Control-Allow-Methods",
                    "GET, HEAD, OPTIONS".to_string(),
                ),
                (
                    "Access-Control-Allow-Headers",
                    "Range, Content-Type".to_string(),
                ),
                ("Content-Length", "0".to_string()),
            ],
        )
        .await;
    }

    if method != "GET" && method != "HEAD" {
        return write_http_headers(
            &mut stream,
            "405 Method Not Allowed",
            &[("Content-Length", "0".to_string())],
        )
        .await;
    }

    let Some(path_query) = target.strip_prefix("/video?path=") else {
        return write_http_headers(
            &mut stream,
            "404 Not Found",
            &[("Content-Length", "0".to_string())],
        )
        .await;
    };

    let encoded_path = path_query.split('&').next().unwrap_or("");
    let Ok(path_bytes) = URL_SAFE_NO_PAD.decode(encoded_path) else {
        return write_http_headers(
            &mut stream,
            "400 Bad Request",
            &[("Content-Length", "0".to_string())],
        )
        .await;
    };
    let Ok(file_path) = String::from_utf8(path_bytes) else {
        return write_http_headers(
            &mut stream,
            "400 Bad Request",
            &[("Content-Length", "0".to_string())],
        )
        .await;
    };

    let Ok(metadata) = tokio::fs::metadata(&file_path).await else {
        return write_http_headers(
            &mut stream,
            "404 Not Found",
            &[("Content-Length", "0".to_string())],
        )
        .await;
    };
    let file_size = metadata.len();
    let content_type = video_content_type(&file_path).to_string();
    let range_header = lines.find_map(|line| {
        let (name, value) = line.split_once(':')?;
        if name.eq_ignore_ascii_case("range") {
            Some(value.trim().to_string())
        } else {
            None
        }
    });

    let Ok(mut file) = tokio::fs::File::open(&file_path).await else {
        return write_http_headers(
            &mut stream,
            "404 Not Found",
            &[("Content-Length", "0".to_string())],
        )
        .await;
    };

    if let Some(range_header) = range_header {
        let Some((start, end)) = parse_http_range(&range_header, file_size) else {
            return write_http_headers(
                &mut stream,
                "416 Range Not Satisfiable",
                &[
                    ("Access-Control-Allow-Origin", "*".to_string()),
                    ("Accept-Ranges", "bytes".to_string()),
                    ("Content-Range", format!("bytes */{}", file_size)),
                    ("Content-Length", "0".to_string()),
                ],
            )
            .await;
        };

        let chunk_len = end - start + 1;
        write_http_headers(
            &mut stream,
            "206 Partial Content",
            &[
                ("Content-Type", content_type),
                ("Access-Control-Allow-Origin", "*".to_string()),
                ("Accept-Ranges", "bytes".to_string()),
                (
                    "Content-Range",
                    format!("bytes {}-{}/{}", start, end, file_size),
                ),
                ("Content-Length", chunk_len.to_string()),
                ("Cache-Control", "no-store".to_string()),
                ("Connection", "close".to_string()),
            ],
        )
        .await?;

        if method != "HEAD" {
            file.seek(std::io::SeekFrom::Start(start)).await?;
            let mut reader = file.take(chunk_len);
            tokio::io::copy(&mut reader, &mut stream).await?;
        }
        return Ok(());
    }

    write_http_headers(
        &mut stream,
        "200 OK",
        &[
            ("Content-Type", content_type),
            ("Access-Control-Allow-Origin", "*".to_string()),
            ("Accept-Ranges", "bytes".to_string()),
            ("Content-Length", file_size.to_string()),
            ("Cache-Control", "no-store".to_string()),
            ("Connection", "close".to_string()),
        ],
    )
    .await?;

    if method != "HEAD" {
        tokio::io::copy(&mut file, &mut stream).await?;
    }

    Ok(())
}

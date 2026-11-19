//! Loopback HTTP transport for original images and Range-capable video playback.
use once_cell::sync::OnceCell;
use std::io::{self, SeekFrom};
use tauri::Manager;
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};
use tokio::net::TcpStream;

static ORIGINAL_MEDIA_BASE_URL: OnceCell<String> = OnceCell::new();

pub fn init_original_media_server(app: &tauri::AppHandle) -> io::Result<()> {
    let runtime = tauri::async_runtime::handle();
    let _runtime = runtime.inner().enter();
    let listener = std::net::TcpListener::bind(("127.0.0.1", 0))?;
    listener.set_nonblocking(true)?;
    let addr = listener.local_addr()?;
    let route = format!("/media/{}/", uuid::Uuid::new_v4().simple());
    let scope = app.asset_protocol_scope();
    let listener = tokio::net::TcpListener::from_std(listener)?;
    let _ = ORIGINAL_MEDIA_BASE_URL.set(format!("http://{addr}{route}"));

    tauri::async_runtime::spawn(async move {
        let route: std::sync::Arc<str> = route.into();
        while let Ok((stream, _)) = listener.accept().await {
            let scope = scope.clone();
            let route = route.clone();
            tokio::spawn(async move {
                let _ = handle_connection(stream, &route, move |path| scope.is_allowed(path)).await;
            });
        }
    });
    Ok(())
}

#[tauri::command]
pub fn original_media_base_url() -> Result<&'static str, String> {
    ORIGINAL_MEDIA_BASE_URL
        .get()
        .map(String::as_str)
        .ok_or_else(|| "Original media server not initialized".to_string())
}

pub fn make_video_stream_url(file_path: &str) -> Result<String, String> {
    let encoded =
        percent_encoding::utf8_percent_encode(file_path, percent_encoding::NON_ALPHANUMERIC);
    Ok(format!("{}{}", original_media_base_url()?, encoded))
}

fn parse_http_range(range_header: &str, file_size: u64) -> Option<(u64, u64)> {
    if file_size == 0 {
        return None;
    }
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

async fn write_headers(
    stream: &mut TcpStream,
    status: &str,
    headers: &[(&str, String)],
) -> io::Result<()> {
    let mut response = format!(
        "HTTP/1.1 {status}\r\nConnection: close\r\nReferrer-Policy: no-referrer\r\nContent-Security-Policy: sandbox; default-src 'none'\r\nX-Content-Type-Options: nosniff\r\n"
    );
    for (name, value) in headers {
        response.push_str(&format!("{name}: {value}\r\n"));
    }
    response.push_str("\r\n");
    stream.write_all(response.as_bytes()).await
}

async fn reject(stream: &mut TcpStream, status: &str) -> io::Result<()> {
    write_headers(stream, status, &[("Content-Length", "0".into())]).await
}

async fn read_request(stream: &mut TcpStream) -> io::Result<Option<String>> {
    let mut request = Vec::with_capacity(2048);
    let mut chunk = [0u8; 2048];
    while request.len() < 16 * 1024 {
        let count = stream.read(&mut chunk).await?;
        if count == 0 {
            return Ok(None);
        }
        request.extend_from_slice(&chunk[..count]);
        if request.windows(4).any(|bytes| bytes == b"\r\n\r\n") {
            return Ok(String::from_utf8(request).ok());
        }
    }
    Ok(None)
}

async fn handle_connection(
    mut stream: TcpStream,
    route: &str,
    is_allowed: impl FnOnce(&std::path::Path) -> bool + Send + 'static,
) -> io::Result<()> {
    let Ok(Ok(Some(request))) =
        tokio::time::timeout(std::time::Duration::from_secs(5), read_request(&mut stream)).await
    else {
        return Ok(());
    };
    let mut lines = request.split("\r\n");
    let mut parts = lines.next().unwrap_or_default().split_whitespace();
    let method = parts.next().unwrap_or_default();
    let target = parts
        .next()
        .unwrap_or_default()
        .split('?')
        .next()
        .unwrap_or_default();
    let Some(encoded) = target.strip_prefix(route) else {
        return reject(&mut stream, "404 Not Found").await;
    };
    let origin = lines
        .clone()
        .take_while(|line| !line.is_empty())
        .find_map(|line| {
            let (name, value) = line.split_once(':')?;
            name.eq_ignore_ascii_case("origin").then(|| value.trim())
        });
    let mut cors = vec![("Vary", "Origin".into())];
    if let Some(origin) = origin {
        // WKWebView can serialize a custom-scheme origin as opaque.
        // Every request still requires the per-process route token.
        let trusted = matches!(
            origin,
            "tauri://localhost" | "http://tauri.localhost" | "https://tauri.localhost" | "null"
        ) || (cfg!(debug_assertions) && origin == "http://localhost:3580");
        if !trusted {
            return reject(&mut stream, "403 Forbidden").await;
        }
        cors.push(("Access-Control-Allow-Origin", origin.into()));
    }
    if method == "OPTIONS" {
        cors.extend([
            ("Access-Control-Allow-Methods", "GET, HEAD, OPTIONS".into()),
            ("Access-Control-Allow-Headers", "Range".into()),
            ("Content-Length", "0".into()),
        ]);
        return write_headers(&mut stream, "204 No Content", &cors).await;
    }
    if method != "GET" && method != "HEAD" {
        return reject(&mut stream, "405 Method Not Allowed").await;
    }
    let Ok(path) = percent_encoding::percent_decode_str(encoded).decode_utf8() else {
        return reject(&mut stream, "400 Bad Request").await;
    };
    let Ok(path) = tauri::path::SafePathBuf::new(std::path::PathBuf::from(path.as_ref())) else {
        return reject(&mut stream, "403 Forbidden").await;
    };
    let opened = tokio::task::spawn_blocking(move || {
        let path = path.as_ref();
        if !path.is_absolute() || !is_allowed(path) {
            return Err("403 Forbidden");
        }
        let metadata = std::fs::metadata(path).map_err(|_| "404 Not Found")?;
        if !metadata.is_file() || crate::t_utils::is_cloud_only(&metadata) {
            return Err("403 Forbidden");
        }
        let file = std::fs::File::open(path).map_err(|_| "404 Not Found")?;
        Ok((
            tokio::fs::File::from_std(file),
            path.to_string_lossy().into_owned(),
        ))
    })
    .await;
    let (mut file, path) = match opened {
        Ok(Ok(opened)) => opened,
        Ok(Err(status)) => return reject(&mut stream, status).await,
        Err(_) => return reject(&mut stream, "500 Internal Server Error").await,
    };
    let file_size = file.metadata().await?.len();
    let range = lines.take_while(|line| !line.is_empty()).find_map(|line| {
        let (name, value) = line.split_once(':')?;
        name.eq_ignore_ascii_case("range").then(|| value.trim())
    });
    let mut headers = cors;
    headers.extend([
        (
            "Access-Control-Expose-Headers",
            "Content-Range, Accept-Ranges".into(),
        ),
        ("Accept-Ranges", "bytes".into()),
        ("Cache-Control", "no-store".into()),
    ]);
    let (status, offset, length) = if let Some(range) = range {
        let Some((start, end)) = parse_http_range(range, file_size) else {
            headers.push(("Content-Range", format!("bytes */{file_size}")));
            headers.push(("Content-Length", "0".into()));
            return write_headers(&mut stream, "416 Range Not Satisfiable", &headers).await;
        };
        headers.push(("Content-Range", format!("bytes {start}-{end}/{file_size}")));
        ("206 Partial Content", start, end - start + 1)
    } else {
        ("200 OK", 0, file_size)
    };

    let content_type = match std::path::Path::new(&path)
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase()
        .as_str()
    {
        "mp4" | "m4v" => "video/mp4".into(),
        "mov" => "video/quicktime".into(),
        "webm" => "video/webm".into(),
        "ogv" => "video/ogg".into(),
        _ => {
            let mut prefix = Vec::with_capacity(8192);
            (&mut file).take(8192).read_to_end(&mut prefix).await?;
            tauri_utils::mime_type::MimeType::parse(&prefix, &path)
        }
    };
    headers.push(("Content-Type", content_type));
    headers.push(("Content-Length", length.to_string()));
    if method != "HEAD" {
        file.seek(SeekFrom::Start(offset)).await?;
    }
    write_headers(&mut stream, status, &headers).await?;
    if method != "HEAD" && length > 0 {
        let capacity = length.min(128 * 1024) as usize;
        let mut body = tokio::io::BufReader::with_capacity(capacity, file.take(length));
        tokio::io::copy_buf(&mut body, &mut stream).await?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn streams_full_files_and_ranges_only_with_the_route_and_scope() {
        let path = std::env::temp_dir().join(format!("Lap {} # 示例.mp4", uuid::Uuid::new_v4()));
        std::fs::write(&path, b"0123456789").unwrap();
        let encoded = percent_encoding::utf8_percent_encode(
            path.to_str().unwrap(),
            percent_encoding::NON_ALPHANUMERIC,
        );
        let target = format!("/media/test/{encoded}?v=1");

        async fn request(
            target: &str,
            method: &str,
            range: &str,
            permitted: Option<std::path::PathBuf>,
        ) -> (String, Vec<u8>) {
            let listener = tokio::net::TcpListener::bind(("127.0.0.1", 0))
                .await
                .unwrap();
            let mut client = TcpStream::connect(listener.local_addr().unwrap())
                .await
                .unwrap();
            let (server, _) = listener.accept().await.unwrap();
            let serve = tokio::spawn(handle_connection(server, "/media/test/", move |path| {
                permitted.as_deref() == Some(path)
            }));
            client
                .write_all(format!("{method} {target} HTTP/1.1\r\n").as_bytes())
                .await
                .unwrap();
            client
                .write_all(format!("Host: localhost\r\n{range}\r\n").as_bytes())
                .await
                .unwrap();
            let mut response = Vec::new();
            client.read_to_end(&mut response).await.unwrap();
            serve.await.unwrap().unwrap();
            let separator = response
                .windows(4)
                .position(|chunk| chunk == b"\r\n\r\n")
                .unwrap();
            (
                String::from_utf8(response[..separator].to_vec()).unwrap(),
                response[separator + 4..].to_vec(),
            )
        }

        let (headers, body) = request(&target, "GET", "", Some(path.clone())).await;
        assert!(headers.starts_with("HTTP/1.1 200"));
        assert!(headers.contains("Content-Type: video/mp4"));
        assert_eq!(body, b"0123456789");

        let (headers, body) =
            request(&target, "GET", "Range: bytes=2-4\r\n", Some(path.clone())).await;
        assert!(headers.starts_with("HTTP/1.1 206"));
        assert!(headers.contains("Content-Range: bytes 2-4/10"));
        assert_eq!(body, b"234");

        let (headers, body) =
            request(&target, "HEAD", "Range: bytes=-2\r\n", Some(path.clone())).await;
        assert!(headers.starts_with("HTTP/1.1 206"));
        assert!(headers.contains("Content-Range: bytes 8-9/10"));
        assert!(body.is_empty());

        let (headers, _) =
            request(&target, "GET", "Range: bytes=10-\r\n", Some(path.clone())).await;
        assert!(headers.starts_with("HTTP/1.1 416"));

        let (headers, _) = request(&target, "GET", "", None).await;
        assert!(headers.starts_with("HTTP/1.1 403"));
        let (headers, _) = request("/media/incorrect/file", "GET", "", Some(path.clone())).await;
        assert!(headers.starts_with("HTTP/1.1 404"));

        let (headers, body) =
            request(&target, "OPTIONS", "Origin: tauri://localhost\r\n", None).await;
        assert!(headers.starts_with("HTTP/1.1 204"));
        assert!(headers.contains("Access-Control-Allow-Origin: tauri://localhost"));
        assert!(body.is_empty());

        let (headers, _) = request(
            &target,
            "GET",
            "Origin: https://example.com\r\n",
            Some(path.clone()),
        )
        .await;
        assert!(headers.starts_with("HTTP/1.1 403"));

        let (headers, body) = request(&target, "HEAD", "", Some(path.clone())).await;
        assert!(headers.contains("Content-Length: 10"));
        assert!(body.is_empty());

        let (headers, _) = request(
            "/media/test/relative.mp4",
            "GET",
            "",
            Some("relative.mp4".into()),
        )
        .await;
        assert!(headers.starts_with("HTTP/1.1 403"));

        std::fs::write(&path, b"").unwrap();
        let (headers, body) = request(&target, "GET", "", Some(path.clone())).await;
        assert!(headers.contains("Content-Length: 0"));
        assert!(body.is_empty());
        let (headers, _) = request(&target, "GET", "Range: bytes=0-\r\n", Some(path.clone())).await;
        assert!(headers.starts_with("HTTP/1.1 416"));

        let image_path = path.with_extension("png");
        let image = b"\x89PNG\r\n\x1a\n";
        std::fs::write(&image_path, image).unwrap();
        let encoded = percent_encoding::utf8_percent_encode(
            image_path.to_str().unwrap(),
            percent_encoding::NON_ALPHANUMERIC,
        );
        let (headers, body) = request(
            &format!("/media/test/{encoded}"),
            "GET",
            "",
            Some(image_path.clone()),
        )
        .await;
        assert!(headers.contains("Content-Type: image/png"));
        assert_eq!(body, image);

        std::fs::remove_file(image_path).unwrap();
        std::fs::remove_file(path).unwrap();
    }
}

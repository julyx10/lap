/**
 * Video processing utilities.
 * project: Lap
 * author:  julyx10
 * date:    2024-08-08
 */
use ffmpeg_next as ffmpeg;
use image::{DynamicImage, ImageFormat, RgbImage};
use std::collections::HashMap;
use std::io::Cursor;
use std::panic::{self, AssertUnwindSafe};

/// Upper bound on demuxed packets while extracting one video thumbnail.
/// Corrupt or pathological files can otherwise iterate packets indefinitely.
const MAX_THUMB_DECODE_PACKETS: usize = 24_000;

/// Prefix for errors emitted only when Rust panics inside FFmpeg wrappers (not normal I/O failures).
const FFMPEG_PANIC_ERR_PREFIX: &str = "LAP_FFMPEG_PANIC:";

/// Wraps FFmpeg work so a Rust panic in `ffmpeg-next` does not tear down the whole app.
/// Note: `abort()` from libav/ffmpeg C code cannot be caught here; only unwinding panics.
fn ffmpeg_catch_panic<T>(
    file_path: &str,
    context: &str,
    op: impl FnOnce() -> Result<T, String>,
) -> Result<T, String> {
    let path = file_path.to_string();
    match panic::catch_unwind(AssertUnwindSafe(op)) {
        Ok(inner) => inner,
        Err(_) => {
            eprintln!("Panic caught during {} for video file: {}", context, path);
            Err(format!("{}{}: {}", FFMPEG_PANIC_ERR_PREFIX, context, path))
        }
    }
}

fn is_ffmpeg_panic_err(e: &str) -> bool {
    e.starts_with(FFMPEG_PANIC_ERR_PREFIX)
}

/// Extract rotation from a video stream.
/// First checks the legacy `rotate` metadata tag, then falls back to the
/// display‑matrix side‑data that modern iPhone MOV files use.
fn get_stream_rotation(stream: &ffmpeg::format::stream::Stream) -> i32 {
    // 1. Legacy metadata tag (older containers)
    if let Some(rot) = stream
        .metadata()
        .get("rotate")
        .and_then(|v| v.parse::<i32>().ok())
    {
        if rot != 0 {
            return rot;
        }
    }

    // 2. Display‑matrix side data (modern MOV/MP4)
    unsafe {
        let raw_stream = stream.as_ptr();
        if raw_stream.is_null() {
            return 0;
        }
        let codecpar = (*raw_stream).codecpar;
        if codecpar.is_null() {
            return 0;
        }
        let n = (*codecpar).nb_coded_side_data;
        let side_data_ptr = (*codecpar).coded_side_data;
        if side_data_ptr.is_null() || n <= 0 {
            return 0;
        }
        for i in 0..n as isize {
            let entry = &*side_data_ptr.offset(i);
            if entry.type_ == ffmpeg::ffi::AVPacketSideDataType::AV_PKT_DATA_DISPLAYMATRIX
                && entry.size >= 36
                && !entry.data.is_null()
            {
                let angle = ffmpeg::ffi::av_display_rotation_get(entry.data as *const i32);
                let rounded = (-angle).round() as i32;
                return ((rounded % 360) + 360) % 360;
            }
        }
    }

    0
}

/// Get video dimensions using ffmpeg (accounts for rotation)
pub fn get_video_dimensions(file_path: &str) -> Result<(u32, u32), String> {
    match ffmpeg_catch_panic(file_path, "reading dimensions", || {
        get_video_dimensions_inner(file_path)
    }) {
        Ok(dims) => Ok(dims),
        Err(e) if is_ffmpeg_panic_err(&e) => {
            eprintln!(
                "Degrading video dimensions to 0×0 after FFmpeg panic: {}",
                e
            );
            Ok((0, 0))
        }
        Err(e) => Err(e),
    }
}

fn get_video_dimensions_inner(file_path: &str) -> Result<(u32, u32), String> {
    ffmpeg_next::init().map_err(|e| format!("ffmpeg init error: {:?}", e))?;
    match ffmpeg_next::format::input(&file_path) {
        Ok(ictx) => {
            let input = ictx
                .streams()
                .best(ffmpeg_next::media::Type::Video)
                .ok_or("No video stream found")?;
            let rotation = get_stream_rotation(&input);
            let context = ffmpeg_next::codec::context::Context::from_parameters(input.parameters())
                .map_err(|e| format!("Failed to get codec context: {}", e))?;
            let decoder = context.decoder();
            let video = decoder
                .video()
                .map_err(|e| format!("Failed to get video decoder: {}", e))?;
            let (w, h) = (video.width(), video.height());
            if rotation == 90 || rotation == 270 {
                Ok((h, w))
            } else {
                Ok((w, h))
            }
        }
        Err(e) => Err(format!("Failed to open file: {:?}", e)),
    }
}

/// get video duration using ffmpeg
pub fn get_video_duration(file_path: &str) -> Result<u64, String> {
    match ffmpeg_catch_panic(file_path, "reading duration", || {
        get_video_duration_inner(file_path)
    }) {
        Ok(d) => Ok(d),
        Err(e) if is_ffmpeg_panic_err(&e) => {
            eprintln!("Degrading video duration to 0 after FFmpeg panic: {}", e);
            Ok(0)
        }
        Err(e) => Err(e),
    }
}

fn get_video_duration_inner(file_path: &str) -> Result<u64, String> {
    ffmpeg_next::init().map_err(|e| format!("ffmpeg init error: {:?}", e))?;
    let ictx =
        ffmpeg_next::format::input(file_path).map_err(|e| format!("Failed to open file: {e}"))?;
    let duration = ictx.duration();
    let duration_seconds = if duration > 0 {
        (duration as f64 / ffmpeg_next::ffi::AV_TIME_BASE as f64) as u64 // Convert from AV_TIME_BASE to seconds
    } else {
        0
    };
    Ok(duration_seconds)
}

/// Get a thumbnail from a video or heic file path
pub fn get_video_thumbnail(
    file_path: &str,
    thumbnail_size: u32,
) -> Result<Option<Vec<u8>>, String> {
    match ffmpeg_catch_panic(file_path, "thumbnail extraction", || {
        get_video_thumbnail_internal(file_path, thumbnail_size, true)
    }) {
        Ok(v) => Ok(v),
        Err(e) if is_ffmpeg_panic_err(&e) => {
            eprintln!("Skipping video thumbnail after FFmpeg panic: {}", e);
            Ok(None)
        }
        Err(e) => Err(e),
    }
}

fn get_video_thumbnail_internal(
    file_path: &str,
    thumbnail_size: u32,
    seek_to_ten_percent: bool,
) -> Result<Option<Vec<u8>>, String> {
    ffmpeg::init().map_err(|e| format!("ffmpeg init error: {e}"))?;

    let mut ictx =
        ffmpeg::format::input(file_path).map_err(|e| format!("Failed to open file: {e}"))?;

    let input_stream = ictx
        .streams()
        .best(ffmpeg::media::Type::Video)
        .ok_or("No video stream found")?;

    let stream_index = input_stream.index();

    let rotation = get_stream_rotation(&input_stream);

    let mut decoder = ffmpeg::codec::context::Context::from_parameters(input_stream.parameters())
        .map_err(|e| format!("Failed to get decoder context: {e}"))?
        .decoder()
        .video()
        .map_err(|e| format!("Decoder error: {e}"))?;

    // For video files, seek to 10% of the duration.
    // If seek fails on malformed containers, fallback to decoding from start.
    if seek_to_ten_percent && ictx.duration() > 0 {
        if let Err(e) = ictx.seek(ictx.duration() / 10, ..) {
            eprintln!(
                "Seek failed for '{}': {}. Falling back to decode from start.",
                file_path, e
            );
            return get_video_thumbnail_internal(file_path, thumbnail_size, false);
        }
    }

    let mut packet_count: usize = 0;
    for (stream, packet) in ictx.packets() {
        packet_count = packet_count.saturating_add(1);
        if packet_count > MAX_THUMB_DECODE_PACKETS {
            eprintln!(
                "Video thumbnail: packet limit ({}) reached for '{}', giving up",
                MAX_THUMB_DECODE_PACKETS, file_path
            );
            return Ok(None);
        }

        if stream.index() != stream_index {
            continue;
        }

        decoder
            .send_packet(&packet)
            .map_err(|e| format!("Send packet error: {e}"))?;

        let mut frame = ffmpeg::util::frame::Video::empty();
        if decoder.receive_frame(&mut frame).is_ok() {
            let width = frame.width();
            let height = frame.height();

            // Convert to RGB
            let mut rgb_frame = ffmpeg::util::frame::Video::empty();
            let mut scaler = ffmpeg::software::scaling::context::Context::get(
                decoder.format(),
                width,
                height,
                ffmpeg::format::Pixel::RGB24,
                width,
                height,
                ffmpeg::software::scaling::flag::Flags::BILINEAR,
            )
            .map_err(|e| format!("Scaler creation error: {e}"))?;

            scaler
                .run(&frame, &mut rgb_frame)
                .map_err(|e| format!("Scaling error: {e}"))?;

            // avoid stride error
            let stride = rgb_frame.stride(0);
            let row_bytes = width as usize * 3;
            if stride < row_bytes {
                eprintln!(
                    "Invalid video frame stride for '{}': stride={} < row_bytes={}",
                    file_path, stride, row_bytes
                );
                return Ok(None);
            }

            let frame_data = rgb_frame.data(0);
            let mut buf = Vec::with_capacity((width * height * 3) as usize);
            for y in 0..height as usize {
                let start = y.saturating_mul(stride);
                let end = start.saturating_add(row_bytes);
                if end > frame_data.len() {
                    eprintln!(
                        "Video frame buffer out-of-range for '{}': y={}, start={}, end={}, len={}",
                        file_path,
                        y,
                        start,
                        end,
                        frame_data.len()
                    );
                    return Ok(None);
                }
                buf.extend_from_slice(&frame_data[start..end]);
            }

            // Create DynamicImage
            let rgb_image =
                RgbImage::from_raw(width, height, buf).ok_or("Failed to create image buffer")?;
            let dyn_image = DynamicImage::ImageRgb8(rgb_image);

            // Resize while keeping aspect ratio
            let thumbnail = dyn_image.thumbnail(u32::MAX, thumbnail_size);

            let adjusted_thumbnail = match rotation {
                90 => thumbnail.rotate90(),
                180 => thumbnail.rotate180(),
                270 => thumbnail.rotate270(),
                -90 => thumbnail.rotate270(),
                -180 => thumbnail.rotate180(),
                -270 => thumbnail.rotate90(),
                _ => thumbnail,
            };

            // Encode JPEG to memory
            let mut buffer = Cursor::new(Vec::new());
            adjusted_thumbnail
                .write_to(&mut buffer, ImageFormat::Jpeg)
                .map_err(|e| format!("Image encode error: {e}"))?;

            return Ok(Some(buffer.into_inner()));
        }
    }

    Ok(None)
}

/// Video metadata struct
#[derive(Default, Debug)]
pub struct VideoMetadata {
    pub e_make: Option<String>,
    pub e_model: Option<String>,
    pub e_date_time: Option<String>,
    pub e_software: Option<String>,
    pub gps_latitude: Option<f64>,
    pub gps_longitude: Option<f64>,
    pub gps_altitude: Option<f64>,
}

pub fn get_video_metadata(file_path: &str) -> Result<VideoMetadata, String> {
    match ffmpeg_catch_panic(file_path, "reading metadata", || {
        get_video_metadata_inner(file_path)
    }) {
        Ok(m) => Ok(m),
        Err(e) if is_ffmpeg_panic_err(&e) => {
            eprintln!(
                "Degrading video metadata to empty after FFmpeg panic: {}",
                e
            );
            Ok(VideoMetadata::default())
        }
        Err(e) => Err(e),
    }
}

fn get_video_metadata_inner(file_path: &str) -> Result<VideoMetadata, String> {
    ffmpeg::init().map_err(|e| format!("ffmpeg init error: {:?}", e))?;

    let ictx =
        ffmpeg::format::input(&file_path).map_err(|e| format!("Failed to open file: {:?}", e))?;

    let mut meta = HashMap::<String, String>::new();

    // ---- Collect container metadata -----------------------------------------
    for (k, v) in ictx.metadata().iter() {
        meta.insert(k.to_lowercase(), v.to_string());
    }

    // ---- Collect best video stream metadata ---------------------------------
    if let Some(stream) = ictx.streams().best(ffmpeg::media::Type::Video) {
        for (k, v) in stream.metadata().iter() {
            meta.insert(k.to_lowercase(), v.to_string());
        }
    }

    let mut m = VideoMetadata::default();

    // -------------------------------------------------------------------------
    //  Common metadata field variations (Make / Model / Software)
    // -------------------------------------------------------------------------
    m.e_make = first_exist(
        &meta,
        &[
            "make",
            "camera_make",
            "com.apple.quicktime.make",
            "com.android.capture.camera.make",
        ],
    );

    m.e_model = first_exist(
        &meta,
        &[
            "model",
            "camera_model",
            "com.apple.quicktime.model",
            "com.android.capture.camera.model",
        ],
    );

    m.e_software = first_exist(
        &meta,
        &[
            "software",
            "com.apple.quicktime.software",
            "com.android.capture.camera.software",
            "encoder",
        ],
    );

    // -------------------------------------------------------------------------
    //   Creation Time (several different tags across platforms)
    // -------------------------------------------------------------------------
    m.e_date_time = first_exist(
        &meta,
        &[
            "com.apple.quicktime.creationdate", // Apple
            "com.android.capture.framedate",    // Android
            "creation_time",                    // ffmpeg standard
            "media_time",                       // Some MP4 variants
            "date",                             // Some MKV
            "datetimeoriginal",                 // EXIF pulled through ffmpeg
        ],
    );

    // -------------------------------------------------------------------------
    //   GPS Parsing — Multiple vendor formats
    // -------------------------------------------------------------------------

    // --- Apple ISO6709 style: +37.3317-122.0302/
    if let Some(loc) = first_exist(
        &meta,
        &[
            "location", // generic
            "location-eng",
            "com.apple.quicktime.location.iso6709", // Apple
        ],
    ) {
        parse_apple_iso6709(&loc, &mut m);
    }

    // --- DJI / GoPro often use: gps_latitude, gps_longitude, gps_altitude
    if let Some(lat) = meta.get("gps_latitude") {
        m.gps_latitude = lat.parse().ok();
    }
    if let Some(lon) = meta.get("gps_longitude") {
        m.gps_longitude = lon.parse().ok();
    }
    if let Some(alt) = meta.get("gps_altitude") {
        m.gps_altitude = alt.parse().ok();
    }

    // --- Some devices use: com.dji.gpslatitude, com.dji.gpslongitude
    if let Some(lat) = meta.get("com.dji.gpslatitude") {
        m.gps_latitude = lat.parse().ok();
    }
    if let Some(lon) = meta.get("com.dji.gpslongitude") {
        m.gps_longitude = lon.parse().ok();
    }

    Ok(m)
}

/// Pick the first valid entry from a list of possible tag keys
fn first_exist(meta: &HashMap<String, String>, keys: &[&str]) -> Option<String> {
    for key in keys {
        if let Some(v) = meta.get(&key.to_string()) {
            return Some(v.clone());
        }
    }
    None
}

/// Parse Apple's ISO6709 location format: "+37.3317-122.0302+12.3/"
fn parse_apple_iso6709(raw: &str, m: &mut VideoMetadata) {
    let s = raw.trim().trim_end_matches('/');
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut first = true;

    for ch in s.chars() {
        if (ch == '+' || ch == '-') && !first {
            parts.push(current.clone());
            current.clear();
        }
        current.push(ch);
        first = false;
    }
    if !current.is_empty() {
        parts.push(current);
    }

    if !parts.is_empty() {
        m.gps_latitude = parts[0].parse().ok();
    }
    if parts.len() >= 2 {
        m.gps_longitude = parts[1].parse().ok();
    }
    if parts.len() >= 3 {
        m.gps_altitude = parts[2].parse().ok();
    }
}

/**
 * Video processing utilities.
 * project: jc-photo
 * author:  julyxx
 * email:   tiangle@gmail.com
 * GitHub:  /julyx10
 * date:    2024-08-08
 */
use ffmpeg_next as ffmpeg;
use image::{DynamicImage, ImageFormat, RgbImage};
use rusqlite::Result;
use std::collections::HashMap;
use std::io::Cursor;
/// Get video dimensions using ffmpeg
pub fn get_video_dimensions(file_path: &str) -> Result<(u32, u32), String> {
    ffmpeg_next::init().map_err(|e| format!("ffmpeg init error: {:?}", e))?;
    match ffmpeg_next::format::input(&file_path) {
        Ok(ictx) => {
            let input = ictx
                .streams()
                .best(ffmpeg_next::media::Type::Video)
                .ok_or("No video stream found")?;
            let context = ffmpeg_next::codec::context::Context::from_parameters(input.parameters())
                .map_err(|e| format!("Failed to get codec context: {}", e))?;
            let decoder = context.decoder();
            let video = decoder
                .video()
                .map_err(|e| format!("Failed to get video decoder: {}", e))?;
            Ok((video.width(), video.height()))
        }
        Err(e) => Err(format!("Failed to open file: {:?}", e)),
    }
}

/// get video duration using ffmpeg
pub fn get_video_duration(file_path: &str) -> Result<u64, String> {
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
    ffmpeg::init().map_err(|e| format!("ffmpeg init error: {e}"))?;

    let mut ictx =
        ffmpeg::format::input(file_path).map_err(|e| format!("Failed to open file: {e}"))?;

    let input_stream = ictx
        .streams()
        .best(ffmpeg::media::Type::Video)
        .ok_or("No video stream found")?;

    let stream_index = input_stream.index();

    let rotation = input_stream
        .metadata()
        .get("rotate")
        .and_then(|v| v.parse::<i32>().ok())
        .unwrap_or(0);

    let mut decoder = ffmpeg::codec::context::Context::from_parameters(input_stream.parameters())
        .map_err(|e| format!("Failed to get decoder context: {e}"))?
        .decoder()
        .video()
        .map_err(|e| format!("Decoder error: {e}"))?;

    // For video files, seek to 10% of the duration
    // For HEIC files, we may not have a duration, so we skip seeking
    if ictx.duration() > 0 {
        ictx.seek(ictx.duration() / 10, ..)
            .map_err(|e| format!("Seek error: {e}"))?;
    }

    for (stream, packet) in ictx.packets() {
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
            let stride = rgb_frame.stride(0) as usize;
            let mut buf = Vec::with_capacity((width * height * 3) as usize);
            for y in 0..height as usize {
                let start = y * stride;
                let end = start + (width as usize * 3);
                buf.extend_from_slice(&rgb_frame.data(0)[start..end]);
            }

            // Create DynamicImage
            let rgb_image =
                RgbImage::from_raw(width, height, buf).ok_or("Failed to create image buffer")?;
            let dyn_image = DynamicImage::ImageRgb8(rgb_image);

            // Resize while keeping aspect ratio
            let thumbnail = dyn_image.thumbnail(thumbnail_size, thumbnail_size);

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

    if parts.len() >= 1 {
        m.gps_latitude = parts[0].parse().ok();
    }
    if parts.len() >= 2 {
        m.gps_longitude = parts[1].parse().ok();
    }
    if parts.len() >= 3 {
        m.gps_altitude = parts[2].parse().ok();
    }
}

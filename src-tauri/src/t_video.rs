/**
 * t_video.rs - Core Video Processing Pipeline
 *
 * This module handles video metadata probing, thumbnail extraction, and playback
 * preparation (transcoding/remuxing).
 *
 * Design Principles:
 * 1. Fully Asynchronous: All CLI calls (ffprobe/ffmpeg) use `tokio::process` with hard timeouts.
 * 2. Thread-Safe Bridging: Provides sync wrappers for legacy calls (e.g., database indexing)
 *    using `block_in_place` to prevent runtime deadlocks and UI freezes.
 * 3. Robustness:
 *    - `kill_on_drop(true)`: Ensures orphan processes are reclaimed immediately when a user
 *      switches videos or cancels a task.
 *    - Automatic Fallback: If Remuxing (e.g., for AVI/WMV) fails, it automatically triggers
 *      a full Transcoding fallback to ensure the video is always playable.
 *    - Metadata-based Cache: Deterministic hashing using path, size, and mtime.
 * 4. IO Isolation: Long-running filesystem operations (like cleanup) are offloaded to
 *    background tokio tasks.
 */
use once_cell::sync::OnceCell;
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};
use tokio::sync::Mutex;

static SIDE_CAR_DIR: OnceCell<PathBuf> = OnceCell::new();
const PROCESS_TIMEOUT_SECS: u64 = 30;
const EXTERNAL_PLAYER_REQUIRED_ERROR: &str = "video_requires_external_player";

fn thumbnail_ffmpeg_threads() -> usize {
    let logical_cores = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);
    if logical_cores <= 8 { 2 } else { 4 }
}

pub fn init_ffmpeg_path(app: &AppHandle) {
    // 1. In development, prioritize the source directory
    #[cfg(debug_assertions)]
    {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let dev_path = PathBuf::from(manifest_dir).join("resources").join("ffmpeg");
        if dev_path.exists() {
            let _ = SIDE_CAR_DIR.set(dev_path);
            return;
        }
    }

    // 2. Production: use resource_dir
    if let Ok(res_dir) = app.path().resource_dir() {
        let _ = SIDE_CAR_DIR.set(res_dir.join("ffmpeg"));
    }
}

/// Get video url for different platforms
fn platform_video_url(file_path: &str) -> Result<String, String> {
    #[cfg(target_os = "linux")]
    {
        crate::t_http::make_video_stream_url(file_path)
    }
    #[cfg(not(target_os = "linux"))]
    {
        Ok(file_path.to_string())
    }
}

/// Manages active FFmpeg/FFprobe processes, allowing per-player cancellation.
pub struct VideoManager {
    pub active_processes: Mutex<HashMap<String, (u64, tokio::process::Child)>>,
    pub task_counter: std::sync::atomic::AtomicU64,
}

impl Default for VideoManager {
    fn default() -> Self {
        Self {
            active_processes: Mutex::new(HashMap::new()),
            task_counter: std::sync::atomic::AtomicU64::new(0),
        }
    }
}

fn ffmpeg_sidecar_name() -> &'static str {
    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    {
        "ffmpeg-x86_64-apple-darwin"
    }
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    {
        "ffmpeg-aarch64-apple-darwin"
    }
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    {
        "ffmpeg-x86_64-unknown-linux-gnu"
    }
    #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
    {
        "ffmpeg-aarch64-unknown-linux-gnu"
    }
    #[cfg(target_os = "windows")]
    {
        "ffmpeg-x86_64-pc-windows-msvc.exe"
    }
}

fn ffprobe_sidecar_name() -> &'static str {
    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    {
        "ffprobe-x86_64-apple-darwin"
    }
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    {
        "ffprobe-aarch64-apple-darwin"
    }
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    {
        "ffprobe-x86_64-unknown-linux-gnu"
    }
    #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
    {
        "ffprobe-aarch64-unknown-linux-gnu"
    }
    #[cfg(target_os = "windows")]
    {
        "ffprobe-x86_64-pc-windows-msvc.exe"
    }
}

fn find_sidecar(name: &str) -> Option<PathBuf> {
    SIDE_CAR_DIR.get().and_then(|dir| {
        let path = dir.join(name);
        if path.exists() { Some(path) } else { None }
    })
}

fn sidecar_command(sidecar_name: &str, fallback: &str) -> tokio::process::Command {
    let bin = find_sidecar(sidecar_name).unwrap_or_else(|| PathBuf::from(fallback));
    #[cfg(unix)]
    {
        let mut cmd = tokio::process::Command::new("nice");
        cmd.arg("-n").arg("19").arg(bin);
        cmd
    }
    #[cfg(not(unix))]
    {
        tokio::process::Command::new(bin)
    }
}

fn ffprobe_command() -> tokio::process::Command {
    sidecar_command(ffprobe_sidecar_name(), "ffprobe")
}
fn ffmpeg_command() -> tokio::process::Command {
    sidecar_command(ffmpeg_sidecar_name(), "ffmpeg")
}

fn debug_stderr() -> std::process::Stdio {
    if cfg!(debug_assertions) {
        std::process::Stdio::piped()
    } else {
        std::process::Stdio::null()
    }
}

/// Internal: Get probe JSON via async command with 30s hard timeout.
/// This prevents malformed headers from hanging the backend.
async fn probe_json_async(file_path: &str) -> Result<Value, String> {
    let mut cmd = ffprobe_command();

    cmd.args([
        "-v",
        "quiet",
        "-show_format",
        "-show_streams",
        "-of",
        "json",
        "-threads",
        "1",
        file_path,
    ]);

    // Hide console window on Windows.
    #[cfg(target_os = "windows")]
    {
        cmd.creation_flags(0x08000000);
    }

    let probe_child = cmd
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(debug_stderr())
        .kill_on_drop(true)
        .spawn()
        .map_err(|e| format!("ffprobe failed to spawn: {}", e))?;

    // Hard timeout for probe stage.
    match tokio::time::timeout(
        std::time::Duration::from_secs(30),
        probe_child.wait_with_output(),
    )
    .await
    {
        Ok(Ok(out)) => {
            if out.status.success() {
                serde_json::from_slice(&out.stdout).map_err(|e| e.to_string())
            } else {
                let err = String::from_utf8_lossy(&out.stderr);
                Err(format!(
                    "ffprobe exited with code {}: {}",
                    out.status.code().unwrap_or(-1),
                    err
                ))
            }
        }
        Ok(Err(e)) => Err(format!("ffprobe execution failed: {}", e)),
        Err(_) => Err("ffprobe timed out after 30s".to_string()),
    }
}

/// Get video duration asynchronously.
#[allow(dead_code)]
pub async fn get_video_duration(file_path: &str) -> Result<u64, String> {
    let json = probe_json_async(file_path).await?;
    Ok(json["format"]["duration"]
        .as_str()
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(0.0) as u64)
}

#[derive(Clone, Copy)]
#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
enum ThumbnailStrategy {
    FastSeek(u64),
    SlowSeek(u64),
    FirstFrame,
}

struct ThumbnailProbeInfo {
    fmt: String,
    v_codec: String,
}

impl ThumbnailProbeInfo {
    fn from_probe(probe: &Value) -> Self {
        let fmt = probe["format"]["format_name"]
            .as_str()
            .unwrap_or("")
            .to_lowercase();
        let v_codec = probe["streams"]
            .as_array()
            .and_then(|streams| {
                streams.iter().find_map(|s| {
                    if s["codec_type"].as_str().unwrap_or("") == "video" {
                        Some(s["codec_name"].as_str().unwrap_or("").to_lowercase())
                    } else {
                        None
                    }
                })
            })
            .unwrap_or_default();

        Self { fmt, v_codec }
    }
}

fn choose_thumbnail_strategy(
    _probe_info: Option<&ThumbnailProbeInfo>,
    seek_time: u64,
) -> ThumbnailStrategy {
    if seek_time == 0 {
        return ThumbnailStrategy::FirstFrame;
    }

    #[cfg(target_os = "windows")]
    {
        if let Some(info) = _probe_info {
            if info.fmt.contains("mov") && info.v_codec == "hevc" {
                return ThumbnailStrategy::SlowSeek(seek_time);
            }
        }
    }

    ThumbnailStrategy::FastSeek(seek_time)
}

fn should_retry_thumbnail_with_slow_seek(
    probe_info: Option<&ThumbnailProbeInfo>,
    primary_strategy: &ThumbnailStrategy,
    seek_time: u64,
    err: &str,
) -> bool {
    if seek_time == 0 {
        return false;
    }

    if matches!(primary_strategy, ThumbnailStrategy::SlowSeek(_)) {
        return false;
    }

    let err = err.to_lowercase();
    if err.contains("moov atom not found") {
        return false;
    }

    let seek_like_failure = [
        "could not seek",
        "error while seeking",
        "partial file",
        "invalid data found when processing input",
        "non monotonically increasing dts",
    ]
    .iter()
    .any(|pattern| err.contains(pattern));

    if !seek_like_failure {
        return false;
    }

    let Some(info) = probe_info else {
        return false;
    };

    info.fmt.contains("mov") || info.v_codec == "hevc"
}

async fn run_thumbnail_command(
    file_path: &str,
    ffmpeg_threads: &str,
    filter: &str,
    strategy: ThumbnailStrategy,
    timeout_secs: u64,
) -> Result<Option<Vec<u8>>, String> {
    let mut cmd = ffmpeg_command();

    // Fast seek: -ss before -i (input-side seeking, near-instant but less precise)
    if let ThumbnailStrategy::FastSeek(t) = strategy {
        cmd.args(["-ss", &t.to_string()]);
    }
    cmd.args(["-i", file_path]);
    // Slow seek: -ss after -i (output-side seeking, precise but decodes from start)
    if let ThumbnailStrategy::SlowSeek(t) = strategy {
        cmd.args(["-ss", &t.to_string()]);
    }
    cmd.args([
        "-map",
        "0:v:0",
        "-vframes",
        "1",
        "-an",
        "-sn",
        "-vf",
        filter,
        "-c:v",
        "mjpeg",
        "-f",
        "image2",
        "-threads",
        ffmpeg_threads,
        "pipe:1",
    ]);

    #[cfg(target_os = "windows")]
    {
        cmd.creation_flags(0x08000000);
    }

    let child = cmd
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .map_err(|e| e.to_string())?;

    match tokio::time::timeout(
        std::time::Duration::from_secs(timeout_secs),
        child.wait_with_output(),
    )
    .await
    {
        Ok(Ok(output)) if output.status.success() && !output.stdout.is_empty() => {
            Ok(Some(output.stdout))
        }
        Ok(Ok(output)) if output.status.success() => {
            Err("thumbnail extraction produced empty output".to_string())
        }
        Ok(Ok(output)) => {
            let err = String::from_utf8_lossy(&output.stderr);
            Err(err.to_string())
        }
        Ok(Err(e)) => Err(e.to_string()),
        Err(_) => Err(format!(
            "thumbnail extraction timed out after {}s",
            timeout_secs
        )),
    }
}

/// Extracts a video thumbnail.
pub async fn get_video_thumbnail(
    file_path: &str,
    thumbnail_size: u32,
    known_duration: Option<u64>,
) -> Result<Option<Vec<u8>>, String> {
    let probe = probe_json_async(file_path).await.ok();
    let probe_info = probe.as_ref().map(ThumbnailProbeInfo::from_probe);
    let duration = if let Some(d) = known_duration {
        d
    } else {
        probe
            .as_ref()
            .and_then(|json| json["format"]["duration"].as_str())
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(0.0) as u64
    };
    let seek_time = if duration > 10 { duration / 10 } else { 0 };
    let ffmpeg_threads = thumbnail_ffmpeg_threads().to_string();
    let filter = format!(
        "scale=w={}:h={}:force_original_aspect_ratio=decrease",
        thumbnail_size, thumbnail_size
    );

    let primary_strategy = probe
        .as_ref()
        .map(|_| choose_thumbnail_strategy(probe_info.as_ref(), seek_time))
        .unwrap_or_else(|| {
            if seek_time > 0 {
                ThumbnailStrategy::FastSeek(seek_time)
            } else {
                ThumbnailStrategy::FirstFrame
            }
        });

    match run_thumbnail_command(file_path, &ffmpeg_threads, &filter, primary_strategy, 20).await {
        Ok(result) => return Ok(result),
        Err(err) => {
            if should_retry_thumbnail_with_slow_seek(
                probe_info.as_ref(),
                &primary_strategy,
                seek_time,
                &err,
            ) {
                if let Ok(result) = run_thumbnail_command(
                    file_path,
                    &ffmpeg_threads,
                    &filter,
                    ThumbnailStrategy::SlowSeek(seek_time),
                    20,
                )
                .await
                {
                    return Ok(result);
                }
            }
        }
    }

    match run_thumbnail_command(
        file_path,
        &ffmpeg_threads,
        &filter,
        ThumbnailStrategy::FirstFrame,
        15,
    )
    .await
    {
        Ok(result) => Ok(result),
        Err(err) => {
            eprintln!(
                "FFmpeg total thumbnail failure for {}. Err: {}",
                file_path, err
            );
            Ok(None)
        }
    }
}

pub fn get_video_thumbnail_sync(
    file_path: &str,
    sz: u32,
    known_duration: Option<u64>,
) -> Result<Option<Vec<u8>>, String> {
    let handle = tokio::runtime::Handle::try_current().map_err(|_| "No runtime handle")?;
    tokio::task::block_in_place(|| {
        handle.block_on(get_video_thumbnail(file_path, sz, known_duration))
    })
}

#[derive(Default)]
pub struct VideoMetadata {
    pub width: u32,
    pub height: u32,
    pub duration: u64,
    pub e_make: Option<String>,
    pub e_model: Option<String>,
    pub e_date_time: Option<String>,
    pub e_software: Option<String>,
    pub gps_latitude: Option<f64>,
    pub gps_longitude: Option<f64>,
    pub gps_altitude: Option<f64>,
}

pub async fn get_video_metadata_async(file_path: &str) -> Result<VideoMetadata, String> {
    let json = probe_json_async(file_path).await?;

    // Extract stream info
    let streams = json["streams"]
        .as_array()
        .ok_or("No streams found in video")?;
    let video = streams
        .iter()
        .find(|s| s["codec_type"] == "video")
        .ok_or("No video stream found")?;

    let mut w = video["width"].as_u64().unwrap_or(0) as u32;
    let mut h = video["height"].as_u64().unwrap_or(0) as u32;

    // Handle rotation
    let mut rotation = 0;
    if let Some(tags) = video["tags"].as_object() {
        if let Some(rot) = tags
            .get("rotate")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<i32>().ok())
        {
            rotation = rot;
        }
    }
    if rotation == 0 {
        if let Some(side_data) = video["side_data_list"].as_array() {
            for entry in side_data {
                if entry["side_data_type"] == "Display Matrix" {
                    if let Some(rot) = entry["rotation"].as_i64() {
                        rotation = (((-rot % 360) + 360) % 360) as i32;
                        break;
                    }
                }
            }
        }
    }
    if rotation == 90 || rotation == 270 {
        std::mem::swap(&mut w, &mut h);
    }

    // Extract duration
    let duration = json["format"]["duration"]
        .as_str()
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(0.0) as u64;

    // Extract tags
    let meta = if let Some(tags) = json["format"]["tags"].as_object() {
        tags.iter()
            .map(|(k, v)| (k.to_lowercase(), v.as_str().unwrap_or("").to_string()))
            .collect::<HashMap<String, String>>()
    } else {
        HashMap::new()
    };

    Ok(VideoMetadata {
        width: w,
        height: h,
        duration,
        e_make: first_exist(&meta, &["make", "camera_make"]),
        e_model: first_exist(&meta, &["model", "camera_model"]),
        e_software: first_exist(&meta, &["software", "encoder"]),
        e_date_time: first_exist(&meta, &["creation_time"]),
        gps_latitude: None,
        gps_longitude: None,
        gps_altitude: None,
    })
}

/// Checks if MOOV atom is within the first 1MB of the file (faststart).
/// Correctly parses MP4 boxes by jumping boundaries to detect MOOV before MDAT (faststart).
async fn is_moov_at_start_async(file_path: &str) -> Result<bool, String> {
    use tokio::io::AsyncReadExt;
    let mut file = tokio::fs::File::open(file_path)
        .await
        .map_err(|e| format!("failed to open video for moov scan: {}", e))?;
    let mut buf = vec![0u8; 1024 * 1024]; // 1MB scan buffer
    let n = file
        .read(&mut buf)
        .await
        .map_err(|e| format!("failed to read video for moov scan: {}", e))?;

    let mut pos = 0usize;
    while pos.checked_add(8).map_or(false, |end| end <= n) {
        let size32 = u32::from_be_bytes(buf[pos..pos + 4].try_into().unwrap_or([0; 4]));
        let box_type = &buf[pos + 4..pos + 8];

        // ISO 14496-12: size==1 means 64-bit extended size follows, size==0 means box extends to EOF
        let size = if size32 == 1 {
            if pos + 16 > n {
                break;
            }
            u64::from_be_bytes(buf[pos + 8..pos + 16].try_into().unwrap_or([0; 8])) as usize
        } else if size32 < 8 {
            break;
        } else {
            size32 as usize
        };

        if box_type == b"moov" {
            return Ok(true);
        }
        if box_type == b"mdat" {
            return Ok(false);
        }

        pos = match pos.checked_add(size) {
            Some(p) => p,
            None => break,
        };
    }
    Ok(false)
}

/// Decisions strategy based on OS and probe info.
async fn analyze_strategy(json: &Value, file_path: &str) -> VideoAction {
    let fmt = json["format"]["format_name"]
        .as_str()
        .unwrap_or("")
        .to_lowercase();
    let streams = match json["streams"].as_array() {
        Some(s) => s,
        None => return VideoAction::Transcode,
    };

    let mut v_codec = "";
    let mut a_codec = "";
    let mut a_channels = 0;
    let mut a_profile_str = "";

    for s in streams {
        let codec_type = s["codec_type"].as_str().unwrap_or("");
        if codec_type == "video" {
            v_codec = s["codec_name"].as_str().unwrap_or("");
        } else if codec_type == "audio" {
            a_codec = s["codec_name"].as_str().unwrap_or("");
            a_channels = s["channels"].as_i64().unwrap_or(0);
            a_profile_str = s["profile"].as_str().unwrap_or("");
        }
    }

    // 2. Audio Codec Support (Corrected: ffprobe profile is a string)
    let has_audio = !a_codec.is_empty();
    let a_ok = !has_audio
        || match a_codec {
            "aac" => matches!(a_profile_str, "LC" | "Main" | "") && a_channels <= 2,
            "mp3" => a_channels <= 2,
            "ac3" | "eac3" => cfg!(target_os = "macos"),
            _ => false,
        };

    // 3. Container & Platform Specifics
    let is_mp4 = fmt.contains("mp4") || fmt.contains("m4v");
    let is_mov = fmt.contains("mov");
    let is_safe_mp4_audio = a_ok && (is_mp4 || is_mov);

    #[cfg(target_os = "macos")]
    {
        let direct_ok = match v_codec {
            "h264" => is_safe_mp4_audio,
            "hevc" => is_safe_mp4_audio,
            "vp9" => is_safe_mp4_audio && is_mp4,
            _ => false,
        };

        if direct_ok {
            if (is_mp4 || is_mov) && is_moov_at_start_async(file_path).await.unwrap_or(false) {
                return VideoAction::Direct;
            }
            return VideoAction::Remux;
        }
    }

    #[cfg(any(target_os = "windows", target_os = "linux"))]
    {
        if v_codec == "h264" && is_safe_mp4_audio {
            if is_mp4 && is_moov_at_start_async(file_path).await.unwrap_or(false) {
                return VideoAction::Direct;
            }
            return if is_mp4 || is_mov {
                VideoAction::Remux
            } else {
                VideoAction::Transcode
            };
        }
        #[cfg(target_os = "windows")]
        if v_codec == "hevc" && (is_mp4 || is_mov) && a_ok {
            // Edge WebView2 can accept the container while still failing video decode,
            // which shows up as audio-only playback for some MOV/HEVC files.
            return VideoAction::Transcode;
        }

        #[cfg(target_os = "linux")]
        if v_codec == "hevc" && (is_mp4 || is_mov) && a_ok {
            // Linux WebKitGTK support varies by distro/codec pack. Keep the cheaper remux path
            // and let the frontend fall back to an external player if decode still fails.
            return VideoAction::Remux;
        }
    }

    VideoAction::Transcode
}

pub fn get_video_metadata(file_path: &str) -> Result<VideoMetadata, String> {
    let handle = tokio::runtime::Handle::try_current().map_err(|_| "No runtime handle")?;
    tokio::task::block_in_place(|| handle.block_on(get_video_metadata_async(file_path)))
}

fn first_exist(meta: &HashMap<String, String>, keys: &[&str]) -> Option<String> {
    for k in keys {
        if let Some(v) = meta.get(*k) {
            return Some(v.clone());
        }
    }
    None
}

#[derive(Serialize)]
pub struct VideoPrepareResult {
    pub url: String,
    pub action: VideoAction,
}

#[derive(Debug, PartialEq, Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum VideoAction {
    Direct,
    Remux,
    Transcode,
}

pub fn init_video_cache(app: &AppHandle) {
    if let Ok(d) = app.path().app_cache_dir().map(|d| d.join("video_cache")) {
        if !d.exists() {
            let _ = std::fs::create_dir_all(&d);
        }
    }
}

#[tauri::command]
pub async fn clear_video_cache(
    app: AppHandle,
    state: tauri::State<'_, VideoManager>,
) -> Result<(), String> {
    {
        let mut guard = state.active_processes.lock().await;
        guard.clear();
    }

    // Windows Fix: Give the OS a moment to release file handles after killing processes
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;

    if let Ok(d) = app.path().app_cache_dir().map(|d| d.join("video_cache")) {
        if d.exists() {
            let _ = tokio::fs::remove_dir_all(&d).await;
            let _ = tokio::fs::create_dir_all(&d).await;
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn cancel_video_prepare(
    state: tauri::State<'_, VideoManager>,
    player_id: String,
) -> Result<(), String> {
    let mut guard = state.active_processes.lock().await;
    guard.remove(&player_id);
    Ok(())
}

/// Prepares a video for playback with a single probe and an optimized retry loop.
#[tauri::command]
pub async fn prepare_video(
    app: AppHandle,
    state: tauri::State<'_, VideoManager>,
    file_path: String,
    player_id: String,
    force: Option<String>,
) -> Result<VideoPrepareResult, String> {
    {
        state.active_processes.lock().await.remove(&player_id);
    }

    // Global deadline: user should never wait more than PROCESS_TIMEOUT_SECS.
    // This is a photo management app for phone/camera clips, not a full video player.
    let deadline =
        tokio::time::Instant::now() + std::time::Duration::from_secs(PROCESS_TIMEOUT_SECS);

    // Step 1: Single Probe
    let action = match probe_json_async(&file_path).await {
        Ok(json) => analyze_strategy(&json, &file_path).await,
        Err(e) => return Err(format!("Probe failed: {}", e)),
    };

    let force_process = force.as_deref() == Some("process");
    let force_fallback = force.as_deref() == Some("fallback");

    if action == VideoAction::Direct && !force_process && !force_fallback {
        return Ok(VideoPrepareResult {
            url: platform_video_url(&file_path)?,
            action: VideoAction::Direct,
        });
    }

    let mut current_action = if force_process {
        VideoAction::Transcode
    } else if force_fallback {
        match action {
            VideoAction::Direct => VideoAction::Remux,
            VideoAction::Remux => VideoAction::Transcode,
            VideoAction::Transcode => VideoAction::Transcode,
        }
    } else {
        action
    };
    let mut try_software_enc = false;

    let temp_dir = app
        .path()
        .app_cache_dir()
        .map_err(|e| e.to_string())?
        .join("video_cache");
    if !temp_dir.exists() {
        let _ = tokio::fs::create_dir_all(&temp_dir).await;
    }

    let ext = "mp4"; // Use MP4 for platforms with reliable built-in H.264 playback in the embedded WebView
    let cache_name = get_cache_filename_async(&file_path, ext).await;
    let output_path = temp_dir.join(&cache_name);
    // Fixed Concurrency: include player_id in tmp name to avoid contention
    let tmp_path = temp_dir.join(format!("{}.{}.tmp", cache_name, hash_id(&player_id)));

    // Cache validation: check if exists and size > 32KB (avoid broken/truncated files)
    if let Ok(meta) = tokio::fs::metadata(&output_path).await {
        if meta.len() > 32 * 1024 && force.is_none() {
            return Ok(VideoPrepareResult {
                url: platform_video_url(output_path.to_string_lossy().as_ref())?,
                action: current_action,
            });
        }
        // If file is broken (<32KB) or we are forcing, remove it to regenerate
        let _ = tokio::fs::remove_file(&output_path).await;
    }

    // Processing loop with global deadline — all stages (remux, transcode, fallback) share one budget.
    loop {
        let mut cmd = ffmpeg_command();

        cmd.arg("-i").arg(&file_path);
        cmd.args(["-map", "0:v:0", "-map", "0:a:0?"]);
        if current_action == VideoAction::Remux {
            cmd.args([
                "-c:v",
                "copy",
                "-c:a",
                "copy",
                "-movflags",
                "+faststart",
                "-f",
                "mp4",
                "-y",
            ]);
        } else {
            // Transcode: macOS tries hardware encoding first, all others use libx264
            #[cfg(target_os = "macos")]
            if !try_software_enc {
                cmd.args(["-c:v", "h264_videotoolbox", "-b:v", "4000k"]);
            } else {
                cmd.args(["-c:v", "libx264", "-preset", "superfast", "-crf", "23"]);
            }
            #[cfg(not(target_os = "macos"))]
            cmd.args(["-c:v", "libx264", "-preset", "superfast", "-crf", "23"]);
            cmd.args([
                "-c:a",
                "aac",
                "-b:a",
                "192k",
                "-movflags",
                "+faststart",
                "-f",
                "mp4",
                "-y",
            ]);
        }
        cmd.arg(&tmp_path);
        #[cfg(target_os = "windows")]
        {
            cmd.creation_flags(0x08000000);
        }

        cmd.stderr(debug_stderr());

        let this_task_id = state
            .task_counter
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        #[allow(unused_mut)]
        let mut t_child = cmd
            .stdin(std::process::Stdio::null())
            .kill_on_drop(true)
            .spawn()
            .map_err(|e| e.to_string())?;

        #[cfg(debug_assertions)]
        let stderr_task = t_child.stderr.take().map(|mut stderr| {
            tokio::spawn(async move {
                use tokio::io::AsyncReadExt;
                let mut err_log = String::new();
                let _ = stderr.read_to_string(&mut err_log).await;
                err_log
            })
        });

        {
            state
                .active_processes
                .lock()
                .await
                .insert(player_id.clone(), (this_task_id, t_child));
        }

        let final_status = loop {
            // Priority Check: Abort if global deadline passed → tell user to use external player
            if tokio::time::Instant::now() > deadline {
                let mut guard = state.active_processes.lock().await;
                guard.remove(&player_id);
                let _ = tokio::fs::remove_file(&tmp_path).await;
                return Err(EXTERNAL_PLAYER_REQUIRED_ERROR.to_string());
            }

            {
                let mut guard = state.active_processes.lock().await;
                if let Some((pid, child)) = guard.get_mut(&player_id) {
                    if *pid != this_task_id {
                        return Err("Task preempted".to_string());
                    }

                    // DO NOT use .wait() here while holding the lock! It will deadlock other cmds.
                    match child.try_wait() {
                        Ok(Some(s)) => {
                            guard.remove(&player_id);
                            break s; // Task finished
                        }
                        Ok(None) => {} // Still running, release lock and sleep
                        Err(e) => {
                            guard.remove(&player_id);
                            return Err(format!("Process crashed: {}", e));
                        }
                    }
                } else {
                    let _ = tokio::fs::remove_file(&tmp_path).await;
                    return Err("Cancelled".to_string());
                }
            } // Lock released here

            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        };

        if final_status.success() {
            // Check if another task already promoted this cache
            if output_path.exists() {
                let _ = tokio::fs::remove_file(&tmp_path).await;
                return Ok(VideoPrepareResult {
                    url: platform_video_url(output_path.to_string_lossy().as_ref())?,
                    action: current_action,
                });
            }

            if tokio::fs::rename(&tmp_path, &output_path).await.is_ok() {
                let td = temp_dir.clone();
                tokio::spawn(async move {
                    auto_cleanup_video_cache_async(td).await;
                });
                return Ok(VideoPrepareResult {
                    url: platform_video_url(output_path.to_string_lossy().as_ref())?,
                    action: current_action,
                });
            }
            return Err("Rename failed".to_string());
        } else {
            // Trace failure cause in debug mode
            #[cfg(debug_assertions)]
            if let Some(stderr_task) = stderr_task {
                if let Ok(err_log) = stderr_task.await {
                    eprintln!(
                        "FFmpeg stage failed ({:?}) output:\n{}",
                        current_action, err_log
                    );
                }
            }

            let _ = tokio::fs::remove_file(&tmp_path).await;
            if current_action == VideoAction::Remux {
                // FALLBACK PASS 1: Remux failed, transition to Transcode
                current_action = VideoAction::Transcode;
                continue;
            }
            #[cfg(target_os = "macos")]
            if current_action == VideoAction::Transcode && !try_software_enc {
                // FALLBACK PASS 2: macOS Hardware encoding failed, switch to Software (libx264)
                try_software_enc = true;
                continue;
            }
            return Err(format!("FFmpeg failed: {}", final_status));
        }
    }
}

async fn auto_cleanup_video_cache_async(dir: PathBuf) {
    let max: u64 = 10 * 1024 * 1024 * 1024;
    let target: u64 = 7 * 1024 * 1024 * 1024;
    let Ok(mut entries) = tokio::fs::read_dir(dir).await else {
        return;
    };
    let mut files = Vec::new();
    let mut total = 0;
    while let Ok(Some(e)) = entries.next_entry().await {
        if let Ok(m) = e.metadata().await {
            let name = e.file_name();
            if m.is_file() && !name.to_string_lossy().ends_with(".tmp") {
                total += m.len();
                files.push((
                    e.path(),
                    m.len(),
                    m.modified().unwrap_or(std::time::SystemTime::UNIX_EPOCH),
                ));
            }
        }
    }
    if total <= max {
        return;
    }
    files.sort_by_key(|f| f.2);
    for (p, s, _) in files {
        if total <= target {
            break;
        }
        if tokio::fs::remove_file(&p).await.is_ok() {
            total -= s;
        }
    }
}

async fn get_cache_filename_async(p: &str, ext: &str) -> String {
    let meta = tokio::fs::metadata(p).await.ok();
    let size = meta.as_ref().map(|m| m.len()).unwrap_or(0);
    let mtime = meta
        .as_ref()
        .and_then(|m| m.modified().ok())
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("{:x}_{}_{}.{}", hash_id(p), size, mtime, ext)
}

fn hash_id(value: &str) -> u64 {
    // FNV-1a: deterministic across Rust versions, unlike DefaultHasher
    let mut h: u64 = 0xcbf29ce484222325;
    for b in value.as_bytes() {
        h ^= *b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

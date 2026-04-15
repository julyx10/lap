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

use std::collections::HashMap;
use std::path::PathBuf;
use tokio::sync::Mutex;
use tauri::{AppHandle, Manager};
use once_cell::sync::Lazy;

/// Manages active FFmpeg/FFprobe processes, allowing per-player cancellation.
pub struct VideoManager { 
    pub active_processes: Mutex<HashMap<String, (u64, tokio::process::Child)>>,
    pub task_counter: std::sync::atomic::AtomicU64
}

impl Default for VideoManager { 
    fn default() -> Self { 
        Self { 
            active_processes: Mutex::new(HashMap::new()),
            task_counter: std::sync::atomic::AtomicU64::new(0)
        } 
    } 
}

static FFMPEG_SIDECAR_DIR: Lazy<Option<PathBuf>> = Lazy::new(|| {
    // 1. In development, prioritize the source directory to avoid using stale 'cached' files in the target folder.
    #[cfg(debug_assertions)]
    {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let dev_path = PathBuf::from(manifest_dir).join("resources").join("ffmpeg");
        if dev_path.exists() {
            return Some(dev_path);
        }
    }

    // 2. Try to find sidecar in standard resource locations (Production/Bundled):
    // - macOS: inside .app bundle
    // - Linux/Windows: relative to executable
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            // Check common patterns (Windows/Linux)
            let candidates: Vec<PathBuf> = vec![
                exe_dir.join("resources").join("ffmpeg"),
                exe_dir.join("ffmpeg"),
            ];
            for candidate in candidates {
                if candidate.exists() {
                    return Some(candidate);
                }
            }
            // macOS bundle Resources pattern: Contents/Resources/ffmpeg
            if let Some(parent) = exe_dir.parent() {
                let candidate = parent.join("Resources").join("ffmpeg");
                if candidate.exists() {
                    return Some(candidate);
                }
            }
        }
    }
    None
});

fn ffmpeg_sidecar_name() -> &'static str {
    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    { "ffmpeg-x86_64-apple-darwin" }
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    { "ffmpeg-aarch64-apple-darwin" }
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    { "ffmpeg-x86_64-unknown-linux-gnu" }
    #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
    { "ffmpeg-aarch64-unknown-linux-gnu" }
    #[cfg(target_os = "windows")]
    { "ffmpeg-x86_64-pc-windows-msvc.exe" }
}

fn ffprobe_sidecar_name() -> &'static str {
    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    { "ffprobe-x86_64-apple-darwin" }
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    { "ffprobe-aarch64-apple-darwin" }
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    { "ffprobe-x86_64-unknown-linux-gnu" }
    #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
    { "ffprobe-aarch64-unknown-linux-gnu" }
    #[cfg(target_os = "windows")]
    { "ffprobe-x86_64-pc-windows-msvc.exe" }
}

fn find_sidecar(name: &str) -> Option<PathBuf> {
    FFMPEG_SIDECAR_DIR.as_ref().and_then(|dir| {
        let path = dir.join(name);
        if path.exists() { Some(path) } else { None }
    })
}

fn ffprobe_command() -> tokio::process::Command {
    #[cfg(unix)]
    {
        let mut cmd = tokio::process::Command::new("nice");
        cmd.arg("-n").arg("19");
        if let Some(path) = find_sidecar(ffprobe_sidecar_name()) {
            cmd.arg(path);
        } else {
            cmd.arg("ffprobe");
        }
        cmd
    }
    #[cfg(not(unix))]
    {
        if let Some(path) = find_sidecar(ffprobe_sidecar_name()) {
            tokio::process::Command::new(path)
        } else {
            tokio::process::Command::new("ffprobe")
        }
    }
}

fn ffmpeg_command() -> tokio::process::Command {
    #[cfg(unix)]
    {
        let mut cmd = tokio::process::Command::new("nice");
        cmd.arg("-n").arg("19");
        if let Some(path) = find_sidecar(ffmpeg_sidecar_name()) {
            cmd.arg(path);
        } else {
            cmd.arg("ffmpeg");
        }
        cmd
    }
    #[cfg(not(unix))]
    {
        if let Some(path) = find_sidecar(ffmpeg_sidecar_name()) {
            tokio::process::Command::new(path)
        } else {
            tokio::process::Command::new("ffmpeg")
        }
    }
}

/// Internal: Get probe JSON via async command with 10s hard timeout.
/// This prevents malformed headers from hanging the backend.
async fn probe_json_async(file_path: &str) -> Result<serde_json::Value, String> {
    let mut cmd = ffprobe_command();

    cmd.args(["-v", "quiet", "-show_format", "-show_streams", "-of", "json", file_path]);
    
    // Hide console window on Windows.
    #[cfg(target_os = "windows")] { cmd.creation_flags(0x08000000); }

    let child = cmd
        .stdin(std::process::Stdio::null()) // Prevent ffprobe from waiting on stdin.
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .kill_on_drop(true)
        .spawn()
        .map_err(|e| format!("ffprobe failed to spawn: {}", e))?;

    // Hard timeout for probe stage.
    match tokio::time::timeout(std::time::Duration::from_secs(10), child.wait_with_output()).await {
        Ok(Ok(out)) if out.status.success() => {
            serde_json::from_slice(&out.stdout).map_err(|e| e.to_string())
        }
        _ => Err("ffprobe timeout or unsupported format".to_string())
    }
}

/// Get video dimensions asynchronously.
pub async fn get_video_dimensions_async(file_path: &str) -> Result<(u32, u32), String> {
    let json = probe_json_async(file_path).await?;
    let streams = json["streams"].as_array().ok_or("No streams")?;
    let video = streams.iter().find(|s| s["codec_type"] == "video").ok_or("No video stream")?;
    
    let w = video["width"].as_u64().unwrap_or(0) as u32;
    let h = video["height"].as_u64().unwrap_or(0) as u32;
    
    let mut rotation = 0;
    if let Some(tags) = video["tags"].as_object() {
        if let Some(rot) = tags.get("rotate").and_then(|v| v.as_str()).and_then(|s| s.parse::<i32>().ok()) {
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
    
    if rotation == 90 || rotation == 270 { Ok((h, w)) } else { Ok((w, h)) }
}

/// Sync shim for DB calls.
pub fn get_video_dimensions(file_path: &str) -> Result<(u32, u32), String> {
    let handle = tokio::runtime::Handle::try_current().map_err(|_| "No runtime handle")?;
    tokio::task::block_in_place(|| handle.block_on(get_video_dimensions_async(file_path)))
}

/// Get video duration asynchronously.
pub async fn get_video_duration(file_path: &str) -> Result<u64, String> {
    let json = probe_json_async(file_path).await?;
    Ok(json["format"]["duration"].as_str().and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0) as u64)
}

pub fn get_video_duration_sync(file_path: &str) -> Result<u64, String> {
    let handle = tokio::runtime::Handle::try_current().map_err(|_| "No runtime handle")?;
    tokio::task::block_in_place(|| handle.block_on(get_video_duration(file_path)))
}

/// Extracts a video thumbnail.
pub async fn get_video_thumbnail(file_path: &str, thumbnail_size: u32, known_duration: Option<u64>) -> Result<Option<Vec<u8>>, String> {
    let duration = if let Some(d) = known_duration { d } else { get_video_duration(file_path).await.unwrap_or(0) };
    let seek_time = if duration > 10 { duration / 10 } else { 0 };

    let mut cmd = ffmpeg_command();

    cmd.args(["-ss", &seek_time.to_string(), "-i", file_path, "-vframes", "1", "-f", "image2", "-update", "1",
              "-vf", &format!("scale=w={}:h={}:force_original_aspect_ratio=increase,crop={}:{}", thumbnail_size, thumbnail_size, thumbnail_size, thumbnail_size),
              "-c:v", "mjpeg", "pipe:1"]);
    #[cfg(target_os = "windows")] { cmd.creation_flags(0x08000000); }

    let child = cmd
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .kill_on_drop(true).spawn().map_err(|e| e.to_string())?;
    
    match tokio::time::timeout(std::time::Duration::from_secs(10), child.wait_with_output()).await {
        Ok(Ok(output)) if output.status.success() => Ok(Some(output.stdout)),
        _ => {
            let mut fallback = ffmpeg_command();
            fallback.args(["-i", file_path, "-vframes", "1", "-f", "image2", "-update", "1",
                           "-vf", &format!("scale=w={}:h={}:force_original_aspect_ratio=increase,crop={}:{}", thumbnail_size, thumbnail_size, thumbnail_size, thumbnail_size),
                           "-c:v", "mjpeg", "pipe:1"]);
            #[cfg(target_os = "windows")] { fallback.creation_flags(0x08000000); }
            let f_child = fallback.stdin(std::process::Stdio::null()).stdout(std::process::Stdio::piped()).stderr(std::process::Stdio::null()).kill_on_drop(true).spawn().map_err(|e| e.to_string())?;
            match tokio::time::timeout(std::time::Duration::from_secs(5), f_child.wait_with_output()).await {
                Ok(Ok(o)) if o.status.success() => Ok(Some(o.stdout)),
                _ => Ok(None)
            }
        }
    }
}

pub fn get_video_thumbnail_sync(file_path: &str, sz: u32, known_duration: Option<u64>) -> Result<Option<Vec<u8>>, String> {
    let handle = tokio::runtime::Handle::try_current().map_err(|_| "No runtime handle")?;
    tokio::task::block_in_place(|| handle.block_on(get_video_thumbnail(file_path, sz, known_duration)))
}

#[derive(Default)]
pub struct VideoMetadata {
    pub e_make: Option<String>, pub e_model: Option<String>, pub e_date_time: Option<String>,
    pub e_software: Option<String>, pub gps_latitude: Option<f64>, pub gps_longitude: Option<f64>, pub gps_altitude: Option<f64>,
}

pub async fn get_video_metadata_async(file_path: &str) -> Result<VideoMetadata, String> {
    let json = probe_json_async(file_path).await.unwrap_or_default();
    let meta = if let Some(tags) = json["format"]["tags"].as_object() {
        tags.iter().map(|(k, v)| (k.to_lowercase(), v.as_str().unwrap_or("").to_string())).collect::<HashMap<String, String>>()
    } else { HashMap::new() };
    Ok(VideoMetadata {
        e_make: first_exist(&meta, &["make", "camera_make"]),
        e_model: first_exist(&meta, &["model", "camera_model"]),
        e_software: first_exist(&meta, &["software", "encoder"]),
        e_date_time: first_exist(&meta, &["creation_time"]),
        gps_latitude: None, gps_longitude: None, gps_altitude: None
    })
}

pub fn get_video_metadata(file_path: &str) -> Result<VideoMetadata, String> {
    let handle = tokio::runtime::Handle::try_current().map_err(|_| "No runtime handle")?;
    tokio::task::block_in_place(|| handle.block_on(get_video_metadata_async(file_path)))
}

fn first_exist(meta: &HashMap<String, String>, keys: &[&str]) -> Option<String> {
    for k in keys { if let Some(v) = meta.get(*k) { return Some(v.clone()); } }
    None
}

#[derive(serde::Serialize)]
pub struct VideoPrepareResult { pub url: String, pub is_remuxed: bool }

#[derive(Debug, PartialEq, Clone, Copy)]
enum VideoAction { Direct, Remux, Transcode }

pub fn init_video_cache(app: &AppHandle) {
    if let Ok(d) = app.path().app_cache_dir().map(|d| d.join("video_cache")) { if !d.exists() { let _ = std::fs::create_dir_all(&d); } }
}

#[tauri::command]
pub async fn clear_video_cache(app: AppHandle, state: tauri::State<'_, VideoManager>) -> Result<(), String> {
    { let mut guard = state.active_processes.lock().await; guard.clear(); }
    if let Ok(d) = app.path().app_cache_dir().map(|d| d.join("video_cache")) {
        if d.exists() { 
            let _ = tokio::fs::remove_dir_all(&d).await;
            let _ = tokio::fs::create_dir_all(&d).await;
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn cancel_video_prepare(state: tauri::State<'_, VideoManager>, player_id: String) -> Result<(), String> {
    let mut guard = state.active_processes.lock().await;
    guard.remove(&player_id);
    Ok(())
}

/// Prepares a video for playback with a single probe and an optimized retry loop.
#[tauri::command]
pub async fn prepare_video(app: AppHandle, state: tauri::State<'_, VideoManager>, file_path: String, player_id: String, force: Option<String>) -> Result<VideoPrepareResult, String> {
    { state.active_processes.lock().await.remove(&player_id); }

    // Step 1: Single Probe
    let (action, duration) = match probe_json_async(&file_path).await {
        Ok(json) => {
            let dur = json["format"]["duration"].as_str().and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0) as u64;
            (analyze_probe_info(&json), dur)
        },
        Err(e) => return Err(format!("Probe failed: {}", e)),
    };

    // Rule: Reject transcoding for videos longer than 10 minutes (600s).
    if (action == VideoAction::Transcode || force.as_deref() == Some("process")) && duration > 600 {
        return Err("Video too long for transcoding (>10 min).".to_string());
    }

    if action == VideoAction::Direct && force.as_deref() != Some("process") {
        return Ok(VideoPrepareResult { url: file_path, is_remuxed: false });
    }

    let mut current_action = if force.as_deref() == Some("process") { VideoAction::Transcode } else { action };
    
    let temp_dir = app.path().app_cache_dir().map_err(|e| e.to_string())?.join("video_cache");
    if !temp_dir.exists() { let _ = tokio::fs::create_dir_all(&temp_dir).await; }
    
    let cache_name = get_cache_filename_async(&file_path).await;
    let output_path = temp_dir.join(&cache_name);
    let tmp_path = temp_dir.join(format!("{}.tmp", cache_name));

    if tokio::fs::metadata(&output_path).await.is_ok() { 
        return Ok(VideoPrepareResult { url: output_path.to_string_lossy().to_string(), is_remuxed: true }); 
    }

    // SHARED DEADLINE: The entire prepare_video operation must finish within 30 seconds.
    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(30);

    // Loop for automatic Remux-to-Transcode fallback without re-probing.
    loop {
        let mut cmd = ffmpeg_command();

        cmd.arg("-i").arg(&file_path);
        if current_action == VideoAction::Remux { 
            cmd.args(["-c:v", "copy", "-c:a", "aac", "-b:a", "192k", "-movflags", "faststart", "-f", "mp4", "-y"]); 
        } else {
            #[cfg(target_os = "macos")] { cmd.args(["-c:v", "h264_videotoolbox", "-b:v", "4000k"]); }
            #[cfg(not(target_os = "macos"))] { cmd.args(["-c:v", "libx264", "-preset", "superfast", "-crf", "23"]); }
            cmd.args(["-c:a", "aac", "-b:a", "192k", "-movflags", "faststart", "-f", "mp4", "-y"]);
        }
        cmd.arg(&tmp_path);
        #[cfg(target_os = "windows")] { cmd.creation_flags(0x08000000); }

        let this_task_id = state.task_counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let t_child = cmd.stdin(std::process::Stdio::null()).stderr(std::process::Stdio::null()).kill_on_drop(true).spawn().map_err(|e| e.to_string())?;
        { state.active_processes.lock().await.insert(player_id.clone(), (this_task_id, t_child)); }

        let final_status = loop {
            if tokio::time::Instant::now() > deadline {
                state.active_processes.lock().await.remove(&player_id);
                let _ = tokio::fs::remove_file(&tmp_path).await;
                return Err("Process timeout (30s limit for entire operation)".to_string());
            }
            let mut guard = state.active_processes.lock().await;
            if let Some((stashed_id, c)) = guard.get_mut(&player_id) {
                if *stashed_id != this_task_id {
                    // This task has been superseded by a newer one.
                    let _ = tokio::fs::remove_file(&tmp_path).await;
                    return Err("Cancelled".to_string());
                }
                match c.try_wait() {
                    Ok(Some(s)) => { guard.remove(&player_id); break s; }
                    Ok(None) => {}
                    Err(_) => { guard.remove(&player_id); return Err("Process crash".to_string()); }
                }
            } else {
                let _ = tokio::fs::remove_file(&tmp_path).await;
                return Err("Cancelled".to_string());
            }
            drop(guard);
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        };

        if final_status.success() {
            if tokio::fs::rename(&tmp_path, &output_path).await.is_ok() {
                let td = temp_dir.clone();
                tokio::spawn(async move { auto_cleanup_video_cache_async(td).await; });
                return Ok(VideoPrepareResult { url: output_path.to_string_lossy().to_string(), is_remuxed: true });
            }
            return Err("Rename failed".to_string());
        } else {
            let _ = tokio::fs::remove_file(&tmp_path).await;
            if current_action == VideoAction::Remux {
                // FALLBACK: Remux failed, transition to Transcode and RETRY loop.
                current_action = VideoAction::Transcode;
                continue;
            }
            return Err(format!("FFmpeg failed: {}", final_status));
        }
    }
}

fn analyze_probe_info(json: &serde_json::Value) -> VideoAction {
    let fmt = json["format"]["format_name"].as_str().unwrap_or("").to_lowercase();
    let is_native = (fmt.contains("mp4") || fmt.contains("mov") || fmt.contains("m4v")) && !fmt.contains("mpegts") && !fmt.contains("mpeg");
    let streams = json["streams"].as_array();
    if streams.is_none() { return VideoAction::Transcode; }
    let mut v_ok = false; let mut a_ok = true;
    for s in streams.unwrap() {
        let t = s["codec_type"].as_str().unwrap_or("");
        let n = s["codec_name"].as_str().unwrap_or("");
        if t == "video" { v_ok = match n { "h264" => true, "hevc" | "vp9" => cfg!(target_os = "macos"), _ => false }; }
        else if t == "audio" { a_ok = match n { "aac" | "mp3" => true, _ => false }; }
    }
    if is_native && v_ok && a_ok { VideoAction::Direct } else if v_ok { VideoAction::Remux } else { VideoAction::Transcode }
}

async fn auto_cleanup_video_cache_async(dir: PathBuf) {
    let max = 10 * 1024 * 1024 * 1024;
    let target = 7 * 1024 * 1024 * 1024;
    let Ok(mut entries) = tokio::fs::read_dir(dir).await else { return; };
    let mut files = Vec::new(); let mut total = 0;
    while let Ok(Some(e)) = entries.next_entry().await {
        if let Ok(m) = e.metadata().await { 
            let name = e.file_name();
            if m.is_file() && !name.to_string_lossy().ends_with(".tmp") { 
                total += m.len(); files.push((e.path(), m.len(), m.modified().unwrap_or(std::time::SystemTime::UNIX_EPOCH))); 
            } 
        }
    }
    if total <= max { return; }
    files.sort_by_key(|f| f.2);
    for (p, s, _) in files { if total <= target { break; } if tokio::fs::remove_file(&p).await.is_ok() { total -= s; } }
}

async fn get_cache_filename_async(p: &str) -> String {
    let meta = tokio::fs::metadata(p).await.ok();
    let size = meta.as_ref().map(|m| m.len()).unwrap_or(0);
    let mtime = meta.as_ref().and_then(|m| m.modified().ok()).and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok()).map(|d| d.as_secs()).unwrap_or(0);
    let path_sum: u64 = p.bytes().fold(5381u64, |acc, b| acc.wrapping_mul(33).wrapping_add(b as u64));
    let len_mix = (p.len() as u64).wrapping_mul(0x517cc1b727220a95);
    format!("{:x}_{}_{}.mp4", path_sum ^ len_mix, size, mtime)
}

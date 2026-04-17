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
use once_cell::sync::OnceCell;
use serde::Serialize;
use serde_json::Value;

static SIDE_CAR_DIR: OnceCell<PathBuf> = OnceCell::new();

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
    SIDE_CAR_DIR.get().and_then(|dir| {
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
async fn probe_json_async(file_path: &str) -> Result<Value, String> {
    let mut cmd = ffprobe_command();

    cmd.args(["-v", "quiet", "-show_format", "-show_streams", "-of", "json", "-threads", "1", file_path]);
    
    // Hide console window on Windows.
    #[cfg(target_os = "windows")] { cmd.creation_flags(0x08000000); }

    let probe_child = cmd
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(if cfg!(debug_assertions) { std::process::Stdio::piped() } else { std::process::Stdio::null() })
        .kill_on_drop(true)
        .spawn()
        .map_err(|e| format!("ffprobe failed to spawn: {}", e))?;

    // Hard timeout for probe stage.
    match tokio::time::timeout(std::time::Duration::from_secs(30), probe_child.wait_with_output()).await {
        Ok(Ok(out)) => {
            if out.status.success() {
                serde_json::from_slice(&out.stdout).map_err(|e| e.to_string())
            } else {
                let err = String::from_utf8_lossy(&out.stderr);
                Err(format!("ffprobe exited with code {}: {}", out.status.code().unwrap_or(-1), err))
            }
        }
        Ok(Err(e)) => Err(format!("ffprobe execution failed: {}", e)),
        Err(_) => Err("ffprobe timed out after 30s".to_string()),
    }
}


/// Get video duration asynchronously.
pub async fn get_video_duration(file_path: &str) -> Result<u64, String> {
    let json = probe_json_async(file_path).await?;
    Ok(json["format"]["duration"].as_str().and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0) as u64)
}



/// Extracts a video thumbnail.
pub async fn get_video_thumbnail(file_path: &str, thumbnail_size: u32, known_duration: Option<u64>) -> Result<Option<Vec<u8>>, String> {
    let duration = if let Some(d) = known_duration {
        d
    } else {
        get_video_duration(file_path).await.unwrap_or(0)
    };
    let seek_time = if duration > 10 { duration / 10 } else { 0 };
    let ffmpeg_threads = thumbnail_ffmpeg_threads().to_string();

    let mut cmd = ffmpeg_command();

    let args = ["-ss", &seek_time.to_string(), "-i", file_path, "-vframes", "1", "-f", "image2", "-update", "1",
              "-vf", &format!("scale=w={}:h={}:force_original_aspect_ratio=increase,crop={}:{}", thumbnail_size, thumbnail_size, thumbnail_size, thumbnail_size),
              "-c:v", "mjpeg", "-threads", ffmpeg_threads.as_str(), "pipe:1"];
    cmd.args(args);
    #[cfg(target_os = "windows")] { cmd.creation_flags(0x08000000); }

    let child = cmd
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .kill_on_drop(true).spawn().map_err(|e| e.to_string())?;
    
    match tokio::time::timeout(std::time::Duration::from_secs(30), child.wait_with_output()).await {
        Ok(Ok(output)) if output.status.success() => Ok(Some(output.stdout)),
        Ok(Ok(output)) => {
            let err_msg = String::from_utf8_lossy(&output.stderr);
            eprintln!("FFmpeg failed for {}. Stderr: {}", file_path, err_msg);
            
            let mut fallback = ffmpeg_command();
            fallback.args(["-i", file_path, "-vframes", "1", "-f", "image2", "-update", "1",
                           "-vf", &format!("scale=w={}:h={}:force_original_aspect_ratio=increase,crop={}:{}", thumbnail_size, thumbnail_size, thumbnail_size, thumbnail_size),
                           "-c:v", "mjpeg", "-threads", ffmpeg_threads.as_str(), "pipe:1"]);
            #[cfg(target_os = "windows")] { fallback.creation_flags(0x08000000); }
            let f_child = fallback.stdin(std::process::Stdio::null()).stdout(std::process::Stdio::piped()).stderr(std::process::Stdio::piped()).kill_on_drop(true).spawn().map_err(|e| e.to_string())?;
            match tokio::time::timeout(std::time::Duration::from_secs(15), f_child.wait_with_output()).await {
                Ok(Ok(o)) if o.status.success() => Ok(Some(o.stdout)),
                Ok(Ok(o)) => {
                    let f_err = String::from_utf8_lossy(&o.stderr);
                    eprintln!("FFmpeg fallback failed for {}. Stderr: {}", file_path, f_err);
                    Ok(None)
                }
                _ => Ok(None)
            }
        }
        _ => {
            eprintln!("FFmpeg timed out for {}", file_path);
            Ok(None)
        }
    }
}

pub fn get_video_thumbnail_sync(file_path: &str, sz: u32, known_duration: Option<u64>) -> Result<Option<Vec<u8>>, String> {
    let handle = tokio::runtime::Handle::try_current().map_err(|_| "No runtime handle")?;
    tokio::task::block_in_place(|| handle.block_on(get_video_thumbnail(file_path, sz, known_duration)))
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
    let streams = json["streams"].as_array().ok_or("No streams found in video")?;
    let video = streams.iter().find(|s| s["codec_type"] == "video").ok_or("No video stream found")?;
    
    let mut w = video["width"].as_u64().unwrap_or(0) as u32;
    let mut h = video["height"].as_u64().unwrap_or(0) as u32;
    
    // Handle rotation
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
    if rotation == 90 || rotation == 270 {
        std::mem::swap(&mut w, &mut h);
    }

    // Extract duration
    let duration = json["format"]["duration"].as_str().and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0) as u64;

    // Extract tags
    let meta = if let Some(tags) = json["format"]["tags"].as_object() {
        tags.iter().map(|(k, v)| (k.to_lowercase(), v.as_str().unwrap_or("").to_string())).collect::<HashMap<String, String>>()
    } else { HashMap::new() };

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
        gps_altitude: None
    })
}

/// Checks if MOOV atom is within the first 1MB of the file (faststart).
/// Correctly parses MP4 boxes by jumping boundaries to detect MOOV before MDAT (faststart).
async fn is_moov_at_start_async(file_path: &str) -> bool {
    use tokio::io::AsyncReadExt;
    let Ok(mut file) = tokio::fs::File::open(file_path).await else { return false; };
    let mut buf = vec![0u8; 1024 * 1024]; // 1MB scan buffer
    let Ok(n) = file.read(&mut buf).await else { return false; };

    let mut pos = 0usize;
    while pos.checked_add(8).map_or(false, |end| end <= n) {
        let size_bytes = &buf[pos..pos+4].try_into().ok().and_then(|b| Some(u32::from_be_bytes(b)));
        let size = match size_bytes {
            Some(s) => *s as usize,
            None => break,
        };
        let box_type = &buf[pos+4..pos+8];

        if box_type == b"moov" { return true; }
        if box_type == b"mdat" { return false; } 
        
        if size < 8 { break; } 
        pos = match pos.checked_add(size) {
            Some(p) => p,
            None => break,
        };
    }
    false
}

/// Decisions strategy based on OS and probe info.
async fn analyze_strategy(json: &Value, file_path: &str) -> VideoAction {
    let fmt = json["format"]["format_name"].as_str().unwrap_or("").to_lowercase();
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

    // 1. Video Codec Support logic
    let v_ok = match v_codec {
        "h264" => !cfg!(target_os = "linux"), // Linux WebKitGTK often lacks H.264, force Remux
        "hevc" | "vp9" => cfg!(target_os = "macos"), 
        _ => false,
    };

    // 2. Audio Codec Support (Corrected: ffprobe profile is a string)
    let a_ok = match a_codec {
        "aac" => matches!(a_profile_str, "LC" | "Main" | "") && a_channels <= 2,
        "mp3" => a_channels <= 2,
        "ac3" | "eac3" => cfg!(target_os = "macos"),
        _ => false,
    };

    if !v_ok {
        return VideoAction::Transcode;
    }

    // 3. Container & Platform Specifics
    let is_mp4 = fmt.contains("mp4") || fmt.contains("m4v");
    let is_mov = fmt.contains("mov");

    if a_ok {
        if is_mp4 {
            if is_moov_at_start_async(file_path).await { return VideoAction::Direct; }
        } else if is_mov {
            if cfg!(target_os = "macos") {
                if is_moov_at_start_async(file_path).await { return VideoAction::Direct; }
            } else {
                // Windows/Linux: MOV support is inconsistent in WebViews, force Remux to MP4
                return VideoAction::Remux;
            }
        }
        // Linux: if we reached here, it's probably H.264 which we disabled v_ok for earlier
        // Force Remux to standard MP4 with faststart
        return VideoAction::Remux;
    }

    // Video OK but Audio Incompatible -> Remux (FFmpeg CLI will encode to AAC)
    // For Linux, if video wasn't v_ok, it already went to Transcode above.
    VideoAction::Remux
}

pub fn get_video_metadata(file_path: &str) -> Result<VideoMetadata, String> {
    let handle = tokio::runtime::Handle::try_current().map_err(|_| "No runtime handle")?;
    tokio::task::block_in_place(|| handle.block_on(get_video_metadata_async(file_path)))
}

fn first_exist(meta: &HashMap<String, String>, keys: &[&str]) -> Option<String> {
    for k in keys { if let Some(v) = meta.get(*k) { return Some(v.clone()); } }
    None
}

#[derive(Serialize)]
pub struct VideoPrepareResult { 
    pub url: String, 
    pub action: VideoAction 
}

#[derive(Debug, PartialEq, Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum VideoAction { Direct, Remux, Transcode }

pub fn init_video_cache(app: &AppHandle) {
    if let Ok(d) = app.path().app_cache_dir().map(|d| d.join("video_cache")) { if !d.exists() { let _ = std::fs::create_dir_all(&d); } }
}

#[tauri::command]
pub async fn clear_video_cache(app: AppHandle, state: tauri::State<'_, VideoManager>) -> Result<(), String> {
    { let mut guard = state.active_processes.lock().await; guard.clear(); }
    
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
            (analyze_strategy(&json, &file_path).await, dur)
        },
        Err(e) => return Err(format!("Probe failed: {}", e)),
    };

    // Only reject explicit user-forced processing for very long videos.
    // Automatic platform compatibility processing must remain available.
    if force.as_deref() == Some("process") && duration > 900 {
        return Err("Video too long for transcoding (>15 min).".to_string());
    }

    if action == VideoAction::Direct && force.as_deref() != Some("process") {
        return Ok(VideoPrepareResult { 
            url: file_path, 
            action: VideoAction::Direct 
        });
    }

    let mut current_action = if force.as_deref() == Some("process") { VideoAction::Transcode } else { action };
    let mut try_software_enc = false;
    
    let temp_dir = app.path().app_cache_dir().map_err(|e| e.to_string())?.join("video_cache");
    if !temp_dir.exists() { let _ = tokio::fs::create_dir_all(&temp_dir).await; }
    
    let ext = if current_action == VideoAction::Transcode && cfg!(target_os = "linux") { "webm" } else { "mp4" };
    let cache_name = get_cache_filename_async(&file_path, ext).await;
    let output_path = temp_dir.join(&cache_name);
    // Fixed Concurrency: include player_id in tmp name to avoid contention
    let tmp_path = temp_dir.join(format!("{}.{}.tmp", cache_name, &player_id));

    // Cache validation: check if exists and size > 1KB (avoid broken files)
    if let Ok(meta) = tokio::fs::metadata(&output_path).await {
        if meta.len() > 1024 && force.is_none() {
            return Ok(VideoPrepareResult { 
                url: output_path.to_string_lossy().to_string(), 
                action: current_action 
            });
        }
        // If file is broken (<1KB) or we are forcing, remove it to regenerate
        let _ = tokio::fs::remove_file(&output_path).await;
    }

    // STAGED DEADLINE: Separate Remux and Transcode timeouts.
    loop {
        let mut cmd = ffmpeg_command();
        
        // Define deadline for THIS specific stage (Dynamic based on duration for Transcode)
        let stage_timeout_secs = if current_action == VideoAction::Remux {
            40
        } else {
            // Give automatic compatibility transcodes enough time for long videos.
            // Keep user-forced processing more conservative to avoid runaway jobs.
            let max_transcode_timeout = if force.as_deref() == Some("process") { 900 } else { 3600 };
            let mut base = (duration * 30 / 60).clamp(120, max_transcode_timeout);
            
            // macOS: Software encoding (libx264) is significantly slower than hardware (videotoolbox)
            #[cfg(target_os = "macos")]
            if try_software_enc { base = (base * 3).min(max_transcode_timeout); }
            
            base
        };
        let stage_deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(stage_timeout_secs);

        cmd.arg("-i").arg(&file_path);
        if current_action == VideoAction::Remux { 
            cmd.args(["-c:v", "copy", "-c:a", "aac", "-b:a", "192k", "-movflags", "+faststart", "-f", "mp4", "-y"]); 
        } else {
            // Linux: Use WebM/VP8 for maximum compatibility as H.264 support is unreliable
            #[cfg(target_os = "linux")]
            {
                cmd.args(["-c:v", "libvpx", "-b:v", "2M", "-crf", "10", "-quality", "good", "-cpu-used", "5", "-c:a", "libvorbis", "-f", "webm", "-y"]);
            }
            // macOS: Use Hardware acceleration
            #[cfg(target_os = "macos")]
            {
                if try_software_enc {
                    cmd.args(["-c:v", "libx264", "-preset", "superfast", "-crf", "23", "-c:a", "aac", "-b:a", "192k", "-movflags", "+faststart", "-f", "mp4", "-y"]);
                } else {
                    cmd.args(["-c:v", "h264_videotoolbox", "-b:v", "4000k", "-c:a", "aac", "-b:a", "192k", "-movflags", "+faststart", "-f", "mp4", "-y"]);
                }
            }
            // Windows: Use standard libx264
            #[cfg(target_os = "windows")]
            { 
                cmd.args(["-c:v", "libx264", "-preset", "superfast", "-crf", "23", "-c:a", "aac", "-b:a", "192k", "-movflags", "+faststart", "-f", "mp4", "-y"]); 
            }
        }
        cmd.arg(&tmp_path);
        #[cfg(target_os = "windows")] { cmd.creation_flags(0x08000000); }
        
        #[cfg(debug_assertions)]
        cmd.stderr(std::process::Stdio::piped());
        #[cfg(not(debug_assertions))]
        cmd.stderr(std::process::Stdio::null());

        let this_task_id = state.task_counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        #[allow(unused_mut)]
        let mut t_child = cmd.stdin(std::process::Stdio::null()).kill_on_drop(true).spawn().map_err(|e| e.to_string())?;
        
        #[cfg(debug_assertions)]
        let stderr_pipe = t_child.stderr.take();

        { state.active_processes.lock().await.insert(player_id.clone(), (this_task_id, t_child)); }

        let final_status = loop {
            // Priority Check: Abort if deadline passed
            if tokio::time::Instant::now() > stage_deadline {
                let mut guard = state.active_processes.lock().await;
                guard.remove(&player_id);
                let _ = tokio::fs::remove_file(&tmp_path).await;
                return Err(format!("{} stage timed out", if current_action == VideoAction::Remux { "Remux" } else { "Transcode" }));
            }

            {
                let mut guard = state.active_processes.lock().await;
                if let Some((pid, child)) = guard.get_mut(&player_id) {
                    if *pid != this_task_id { return Err("Task preempted".to_string()); }
                    
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
                    url: output_path.to_string_lossy().to_string(), 
                    action: current_action 
                });
            }

            if tokio::fs::rename(&tmp_path, &output_path).await.is_ok() {
                let td = temp_dir.clone();
                tokio::spawn(async move { auto_cleanup_video_cache_async(td).await; });
                return Ok(VideoPrepareResult { 
                    url: output_path.to_string_lossy().to_string(), 
                    action: current_action 
                });
            }
            return Err("Rename failed".to_string());
        } else {
            // Trace failure cause in debug mode
            #[cfg(debug_assertions)]
            if let Some(mut reader) = stderr_pipe {
                use tokio::io::AsyncReadExt;
                let mut err_log = String::new();
                let _ = reader.read_to_string(&mut err_log).await;
                eprintln!("FFmpeg stage failed ({:?}) output:\n{}", current_action, err_log);
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

async fn get_cache_filename_async(p: &str, ext: &str) -> String {
    let meta = tokio::fs::metadata(p).await.ok();
    let size = meta.as_ref().map(|m| m.len()).unwrap_or(0);
    let mtime = meta.as_ref().and_then(|m| m.modified().ok()).and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok()).map(|d| d.as_secs()).unwrap_or(0);
    let path_sum: u64 = p.bytes().fold(5381u64, |acc, b| acc.wrapping_mul(33).wrapping_add(b as u64));
    let len_mix = (p.len() as u64).wrapping_mul(0x517cc1b727220a95);
    format!("{:x}_{}_{}.{}", path_sum ^ len_mix, size, mtime, ext)
}

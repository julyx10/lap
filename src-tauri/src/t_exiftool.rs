/// ExifTool process runner for extended Fujifilm MakerNotes inspection.
///
/// This module provides Tier-2 deep metadata extraction by invoking `exiftool`
/// on-demand with targeted flags (`-json -s -fast -FujiFilm:all`) and a 3-second timeout safeguard.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExifToolStatus {
    pub available: bool,
    pub version: Option<String>,
    pub binary_path: Option<String>,
}

fn find_executable_in_path(name: &str) -> Option<PathBuf> {
    let path_var = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path_var) {
        let full = dir.join(name);
        if full.is_file() {
            return Some(full);
        }
    }
    None
}

/// Resolves the executable path to use for ExifTool.
/// Checks user-configured custom path first, standard paths, then system PATH.
pub fn resolve_exiftool_bin(custom_path: Option<&str>) -> Option<PathBuf> {
    if let Some(custom) = custom_path {
        let trimmed = custom.trim();
        if !trimmed.is_empty() {
            let path = PathBuf::from(trimmed);
            if path.is_file() {
                return Some(path);
            }
        }
    }

    // Common standard installation paths on macOS / Linux
    let standard_paths = [
        "/etc/profiles/per-user/fmontagut/bin/exiftool",
        "/opt/homebrew/bin/exiftool",
        "/usr/local/bin/exiftool",
        "/usr/bin/exiftool",
    ];

    for p in standard_paths {
        let pb = PathBuf::from(p);
        if pb.is_file() {
            return Some(pb);
        }
    }

    // Fall back to searching system PATH
    #[cfg(target_os = "windows")]
    {
        find_executable_in_path("exiftool.exe").or_else(|| find_executable_in_path("exiftool"))
    }
    #[cfg(not(target_os = "windows"))]
    {
        find_executable_in_path("exiftool")
    }
}

/// Checks whether ExifTool is installed and queries its version.
pub async fn probe_exiftool_status(custom_path: Option<&str>) -> ExifToolStatus {
    let Some(bin_path) = resolve_exiftool_bin(custom_path) else {
        return ExifToolStatus {
            available: false,
            version: None,
            binary_path: None,
        };
    };

    let cmd_result = tokio::time::timeout(
        Duration::from_secs(2),
        Command::new(&bin_path).arg("-ver").output(),
    )
    .await;

    match cmd_result {
        Ok(Ok(output)) if output.status.success() => {
            let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
            ExifToolStatus {
                available: true,
                version: Some(version),
                binary_path: Some(bin_path.to_string_lossy().to_string()),
            }
        }
        _ => ExifToolStatus {
            available: false,
            version: None,
            binary_path: None,
        },
    }
}

/// Runs `exiftool -json -s -fast -FujiFilm:all <file>` on demand and returns the raw JSON object.
pub async fn extract_fuji_raw_tags(
    file_path: &Path,
    custom_bin: Option<&str>,
) -> Result<serde_json::Value, String> {
    if !file_path.exists() {
        return Err(format!("File does not exist: {}", file_path.display()));
    }

    let bin_path = resolve_exiftool_bin(custom_bin).ok_or_else(|| {
        "ExifTool is not installed or not found on PATH".to_string()
    })?;

    let child = Command::new(&bin_path)
        .arg("-json")
        .arg("-s")
        .arg("-fast")
        .arg("-FujiFilm:all")
        .arg(file_path)
        .output();

    let output = tokio::time::timeout(Duration::from_secs(4), child)
        .await
        .map_err(|_| "ExifTool process timed out after 4 seconds".to_string())?
        .map_err(|e| format!("Failed to spawn ExifTool: {e}"))?;

    if !output.status.success() {
        let err_msg = String::from_utf8_lossy(&output.stderr);
        return Err(format!("ExifTool exited with error: {}", err_msg.trim()));
    }

    let parsed: Vec<serde_json::Value> = serde_json::from_slice(&output.stdout)
        .map_err(|e| format!("Failed to parse ExifTool JSON output: {e}"))?;

    let first = parsed
        .into_iter()
        .next()
        .unwrap_or_else(|| serde_json::json!({}));

    Ok(first)
}

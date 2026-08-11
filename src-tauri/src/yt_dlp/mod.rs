#![allow(dead_code)]

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use tauri::{AppHandle, Manager};
use tokio::process::Child;

mod commands;
mod models;

pub use commands::*;
pub use models::*;

/// Application state for managing active downloads.
#[derive(Clone)]
pub struct YtDlpState {
    pub active_downloads: Arc<Mutex<HashMap<u64, Child>>>,
    pub download_counter: Arc<Mutex<u64>>,
}

impl YtDlpState {
    pub fn new() -> Self {
        Self {
            active_downloads: Arc::new(Mutex::new(HashMap::new())),
            download_counter: Arc::new(Mutex::new(0)),
        }
    }
}

/// Get the yt-dlp binary path for the current platform.
pub fn get_binary_path(app_handle: &AppHandle) -> Result<PathBuf, String> {
    let target = tauri::utils::platform::target_triple()
        .map_err(|e| format!("Failed to get target triple: {}", e))?;

    let binary_name = format!("yt-dlp_{}", target);

    // In development, look in the binaries directory
    #[cfg(debug_assertions)]
    {
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
            .map_err(|e| format!("Failed to get manifest dir: {}", e))?;
        let manifest_path = PathBuf::from(&manifest_dir).join("binaries");
        let path = manifest_path.join(&binary_name);
        if path.exists() {
            return Ok(path);
        }

        // Fallback to plain name
        let path = manifest_path.join("yt-dlp");
        if path.exists() {
            return Ok(path);
        }
    }

    // In production, use the sidecar
    let resource_dir = app_handle.path().resource_dir()
        .map_err(|e| format!("Failed to get resource dir: {}", e))?;

    let path = resource_dir.join(&binary_name);
    if path.exists() {
        return Ok(path);
    }

    // Fallback: try system yt-dlp
    Ok(PathBuf::from("yt-dlp"))
}

/// Validate custom arguments against the denied list.
pub fn validate_custom_args(args: &str) -> Result<Vec<String>, String> {
    const DENIED: &[&str] = &[
        "--alias",
        "--config-location",
        "--config-locations",
        "--downloader",
        "--downloader-args",
        "--exec",
        "--exec-before-download",
        "--external-downloader",
        "--external-downloader-args",
        "--ffmpeg-location",
        "--plugin-dirs",
        "--remote-components",
    ];

    let parsed: Vec<String> = args
        .split_whitespace()
        .map(String::from)
        .collect();

    for arg in &parsed {
        let arg_lower = arg.to_lowercase();
        for denied in DENIED {
            if arg_lower.starts_with(denied) {
                return Err(format!("Argument '{}' is not allowed", arg));
            }
        }
    }

    Ok(parsed)
}

/// Parse yt-dlp progress output line.
pub fn parse_progress_line(line: &str) -> Option<(f64, Option<String>, Option<String>)> {
    // [download]  12.3% of ~156.78MiB at  5.12MiB/s ETA 00:25
    let line = line.trim();
    if !line.starts_with("[download]") {
        return None;
    }

    let mut percent = 0.0;
    let mut speed = None;
    let mut eta = None;

    // Extract percentage
    if let Some(pct_start) = line.find('%') {
        let before_pct = &line[..pct_start];
        if let Some(num_start) = before_pct.rfind(|c: char| c.is_ascii_digit() || c == '.') {
            let num_str = &before_pct[..=num_start];
            if let Ok(p) = num_str.trim().parse::<f64>() {
                percent = p;
            }
        }
    }

    // Extract speed
    if let Some(speed_idx) = line.find("at ") {
        let after_at = &line[speed_idx + 3..];
        if let Some(end) = after_at.find(|c: char| c == ' ' && !c.is_ascii_digit()) {
            speed = Some(after_at[..end].trim().to_string());
        } else {
            speed = Some(after_at.trim().to_string());
        }
    }

    // Extract ETA
    if let Some(eta_idx) = line.find("ETA ") {
        let after_eta = &line[eta_idx + 4..];
        eta = Some(after_eta.trim().to_string());
    }

    Some((percent, speed, eta))
}

/// Parse destination from yt-dlp output.
pub fn parse_destination(line: &str) -> Option<String> {
    // [download] Destination: /path/to/file.mp4
    // [ExtractAudio] Destination: /path/to/file.mp3
    // [Merger] Merging formats into "/path/to/file.mp4"
    if let Some(idx) = line.find("Destination: ") {
        let dest = &line[idx + 13..];
        return Some(dest.trim().to_string());
    }

    if let Some(idx) = line.find("Merging formats into \"") {
        let dest = &line[idx + 22..];
        if let Some(end) = dest.rfind('"') {
            return Some(dest[..end].to_string());
        }
    }

    None
}

use std::process::Stdio;

use tauri::{AppHandle, Emitter, State};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

use crate::yt_dlp::{
    get_binary_path, parse_destination, parse_progress_line, validate_custom_args, YtDlpState,
};

/// Get video info from yt-dlp.
#[tauri::command]
pub async fn yt_dlp_get_info(
    app_handle: AppHandle,
    url: String,
) -> Result<serde_json::Value, String> {
    let binary_path = get_binary_path(&app_handle)?;

    let output = Command::new(&binary_path)
        .args(["--dump-json", "--no-download", &url])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(|e| format!("Failed to run yt-dlp: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("yt-dlp error: {}", stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout)
        .map_err(|e| format!("Failed to parse yt-dlp output: {}", e))?;

    Ok(json)
}

/// Get playback info (streaming formats) from yt-dlp.
#[tauri::command]
pub async fn yt_dlp_get_playback_info(
    app_handle: AppHandle,
    url: String,
) -> Result<serde_json::Value, String> {
    let binary_path = get_binary_path(&app_handle)?;

    let output = Command::new(&binary_path)
        .args([
            "--dump-json",
            "--no-download",
            "--no-warnings",
            &url,
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(|e| format!("Failed to run yt-dlp: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("yt-dlp error: {}", stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout)
        .map_err(|e| format!("Failed to parse yt-dlp output: {}", e))?;

    Ok(json)
}

/// Start a yt-dlp download.
#[tauri::command]
pub async fn yt_dlp_download(
    app_handle: AppHandle,
    args: crate::yt_dlp::YtDlpDownloadArgs,
    state: State<'_, YtDlpState>,
) -> Result<u64, String> {
    let binary_path = get_binary_path(&app_handle)?;

    // Build command arguments
    let mut cmd_args: Vec<String> = Vec::new();

    // Output template
    let output_template = args
        .filename_template
        .clone()
        .unwrap_or_else(|| "%(title)s-[%(id)s].%(ext)s".to_string());
    cmd_args.extend_from_slice(&["-o".to_string(), output_template]);

    // Mode-specific args
    match args.mode {
        crate::yt_dlp::DownloadMode::Video => {
            if let Some(quality) = &args.quality {
                cmd_args.extend_from_slice(&[
                    "-f".to_string(),
                    format!("bestvideo[height<={}]+bestaudio/best[height<={}]", quality, quality),
                ]);
            }
            if let Some(format) = &args.video_format {
                cmd_args.extend_from_slice(&["--merge-output-format".to_string(), format.clone()]);
            }
            if let Some(codec) = &args.video_codec {
                cmd_args.extend_from_slice(&[
                    "--recode-video".to_string(),
                    match codec.as_str() {
                        "h264" => "mp4".to_string(),
                        "h265" => "mp4".to_string(),
                        "vp9" => "webm".to_string(),
                        "av1" => "webm".to_string(),
                        _ => codec.clone(),
                    },
                ]);
            }
        }
        crate::yt_dlp::DownloadMode::Audio => {
            cmd_args.push("-x".to_string());
            cmd_args.push("--audio-quality".to_string());
            cmd_args.push("0".to_string());

            if let Some(format) = &args.audio_format {
                cmd_args.push("--audio-format".to_string());
                cmd_args.push(format.clone());
            }
        }
        crate::yt_dlp::DownloadMode::Custom => {
            // Custom mode uses default behavior
        }
    }

    // Time range
    if let Some(start) = &args.start_time {
        cmd_args.extend_from_slice(&["--download-sections".to_string(), format!("*{}..{}", start, args.end_time.as_deref().unwrap_or("inf"))]);
    }

    // Split chapters
    if args.split_chapters {
        cmd_args.push("--split-chapters".to_string());
    }

    // SponsorBlock
    if args.remove_sponsorblock {
        cmd_args.push("--sponsorblock-remove".to_string());
        if !args.sponsor_block_categories.is_empty() {
            cmd_args.push(args.sponsor_block_categories.join(","));
        }
    }

    // Subtitles
    if args.include_subtitles {
        cmd_args.push("--write-subs".to_string());
        if let Some(langs) = &args.subtitle_languages {
            cmd_args.extend_from_slice(&["--sub-langs".to_string(), langs.clone()]);
        }
        if args.embed_subtitles {
            cmd_args.push("--embed-subs".to_string());
        }
    }

    // Thumbnail
    if args.embed_thumbnail {
        cmd_args.push("--embed-thumbnail".to_string());
    }

    // Metadata
    if args.embed_metadata {
        cmd_args.push("--embed-metadata".to_string());
    }

    // Custom args (validated)
    if let Some(custom) = &args.custom_args {
        let validated = validate_custom_args(custom)?;
        cmd_args.extend(validated);
    }

    // URL(s)
    let url = format!("https://www.youtube.com/watch?v={}", args.video_id);
    cmd_args.push(url);

    // Spawn the download process
    let child = Command::new(&binary_path)
        .args(&cmd_args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn yt-dlp: {}", e))?;

    // Get the download ID
    let mut counter = state.download_counter.lock().map_err(|e| e.to_string())?;
    *counter += 1;
    let download_id = *counter;
    drop(counter);

    // Store the child process
    state
        .active_downloads
        .lock()
        .map_err(|e| e.to_string())?
        .insert(download_id, child);

    // Spawn progress monitoring
    let app_handle_clone = app_handle.clone();
    let state_clone = state.inner().clone();
    tauri::async_runtime::spawn(async move {
        monitor_download(download_id, app_handle_clone, state_clone).await;
    });

    Ok(download_id)
}

/// Cancel a download.
#[tauri::command]
pub async fn yt_dlp_cancel(
    app_handle: AppHandle,
    id: u64,
    state: State<'_, YtDlpState>,
) -> Result<(), String> {
    // Take the child out of the map to release the lock before awaiting
    let child = {
        let mut active = state.active_downloads.lock().map_err(|e| e.to_string())?;
        active.remove(&id)
    };

    if let Some(mut child) = child {
        child
            .kill()
            .await
            .map_err(|e| format!("Failed to kill process: {}", e))?;

        // Emit cancellation event
        let _ = app_handle.emit("yt-dlp-cancelled", id);
    }

    Ok(())
}

/// List active downloads.
#[tauri::command]
pub async fn yt_dlp_list(
    state: State<'_, YtDlpState>,
) -> Result<Vec<u64>, String> {
    let active = state.active_downloads.lock().map_err(|e| e.to_string())?;
    Ok(active.keys().copied().collect())
}

/// Monitor download progress.
async fn monitor_download(
    id: u64,
    app_handle: AppHandle,
    state: YtDlpState,
) {
    // Take the child out of the active downloads map so we don't hold the lock
    // across await points. The child is removed from the map for the duration
    // of monitoring and re-inserted only if monitoring fails unexpectedly.
    let child = {
        let mut active = state.active_downloads.lock().unwrap();
        active.remove(&id)
    };

    let Some(mut child) = child else {
        return;
    };

    let stdout = child.stdout.take();
    let stderr = child.stderr.take();

    // Read stdout for progress
    if let Some(stdout) = stdout {
        let reader = BufReader::new(stdout);
        let mut lines = reader.lines();

        while let Ok(Some(line)) = lines.next_line().await {
            if let Some((percent, speed, eta)) = parse_progress_line(&line) {
                let _ = app_handle.emit(
                    "yt-dlp-progress",
                    serde_json::json!({
                        "id": id,
                        "percent": percent,
                        "speed": speed,
                        "eta": eta,
                    }),
                );
            } else if let Some(dest) = parse_destination(&line) {
                let _ = app_handle.emit(
                    "yt-dlp-destination",
                    serde_json::json!({
                        "id": id,
                        "destination": dest,
                    }),
                );
            }
        }
    }

    // Check exit status
    if let Ok(status) = child.wait().await {
        if status.success() {
            let _ = app_handle.emit(
                "yt-dlp-complete",
                serde_json::json!({ "id": id }),
            );
        } else {
            let error_msg = if let Some(stderr) = stderr {
                let reader = BufReader::new(stderr);
                let mut lines = reader.lines();
                let mut msg = String::new();
                while let Ok(Some(line)) = lines.next_line().await {
                    msg.push_str(&line);
                    msg.push('\n');
                }
                msg
            } else {
                "Unknown error".to_string()
            };

            let _ = app_handle.emit(
                "yt-dlp-error",
                serde_json::json!({ "id": id, "error": error_msg }),
            );
        }
    }
}

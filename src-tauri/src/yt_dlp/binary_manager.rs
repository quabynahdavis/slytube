use std::path::{Path, PathBuf};
use std::time::Duration;

use reqwest::Client;
use tauri::{AppHandle, Emitter, Manager};
use tokio::fs;
use tokio::io::AsyncWriteExt;

use crate::yt_dlp::models::{BinaryType, DownloadMetadata, Platform};

/// Base URL for yt-dlp GitHub releases.
const YT_DLP_RELEASE_BASE: &str =
    "https://github.com/yt-dlp/yt-dlp/releases/latest/download";

/// Base URL for FFmpeg builds (BtbN provides cross-platform master-latest builds).
const FFMPEG_RELEASE_BASE: &str =
    "https://github.com/BtbN/FFmpeg-Builds/releases/download/latest";

/// Errors that can occur during binary download/install.
#[derive(Debug, thiserror::Error)]
pub enum BinaryError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("{0}")]
    Other(String),
}

impl From<BinaryError> for String {
    fn from(err: BinaryError) -> Self {
        err.to_string()
    }
}

/// Manages downloading, installing, and updating yt-dlp and FFmpeg binaries.
pub struct BinaryManager {
    client: Client,
    platform: Platform,
    binaries_dir: PathBuf,
}

impl BinaryManager {
    /// Create a new BinaryManager for the current platform.
    ///
    /// Binaries are stored in the app's data directory under `binaries/`.
    pub fn new(app_handle: &AppHandle) -> Result<Self, BinaryError> {
        let client = Client::builder()
            .timeout(Duration::from_secs(300))
            .connect_timeout(Duration::from_secs(30))
            .redirect(reqwest::redirect::Policy::limited(10))
            .user_agent("SlyTube/0.1.0 (https://github.com/slytube)")
            .build()?;

        let app_data = app_handle
            .path()
            .app_data_dir()
            .map_err(|e| BinaryError::Other(format!("Failed to get app data dir: {}", e)))?;
        let binaries_dir = app_data.join("binaries");

        Ok(Self {
            client,
            platform: Platform::current(),
            binaries_dir,
        })
    }

    /// Construct a BinaryManager with an explicit binaries directory (for testing).
    #[cfg(test)]
    pub fn with_dir(client: Client, platform: Platform, binaries_dir: PathBuf) -> Self {
        Self {
            client,
            platform,
            binaries_dir,
        }
    }

    /// Get the download URL for a given binary type.
    pub fn download_url(&self, binary_type: BinaryType) -> String {
        match binary_type {
            BinaryType::YtDlp => {
                format!("{}/{}", YT_DLP_RELEASE_BASE, self.platform.yt_dlp_name())
            }
            BinaryType::FFmpeg => {
                format!(
                    "{}/{}",
                    FFMPEG_RELEASE_BASE,
                    self.platform.ffmpeg_name()
                )
            }
        }
    }

    /// Get the path where a binary would be installed after download.
    pub fn install_path(&self, binary_type: BinaryType) -> PathBuf {
        match binary_type {
            BinaryType::YtDlp => self.binaries_dir.join(self.platform.sidecar_name()),
            BinaryType::FFmpeg => {
                self.binaries_dir
                    .join(self.platform.ffmpeg_binary_name())
            }
        }
    }

    /// Get the `.download.json` metadata sidecar path for a binary.
    fn metadata_path(&self, binary_type: BinaryType) -> PathBuf {
        self.install_path(binary_type)
            .with_extension("download.json")
    }

    /// Get the `.part` temporary download path for a binary.
    fn part_path(&self, binary_type: BinaryType) -> PathBuf {
        let base = self.install_path(binary_type);
        let name = base.file_name().unwrap_or_default().to_string_lossy();
        base.with_file_name(format!("{}.part", name))
    }

    /// Check if a binary is already installed.
    pub fn is_installed(&self, binary_type: BinaryType) -> bool {
        self.install_path(binary_type).exists()
    }

    /// Load stored download metadata for a binary, if present.
    async fn load_metadata(
        &self,
        binary_type: BinaryType,
    ) -> Result<Option<DownloadMetadata>, BinaryError> {
        let path = self.metadata_path(binary_type);
        if !path.exists() {
            return Ok(None);
        }
        let contents = fs::read_to_string(&path).await?;
        let meta = serde_json::from_str(&contents)?;
        Ok(Some(meta))
    }

    /// Save download metadata to the sidecar file.
    async fn save_metadata(
        &self,
        binary_type: BinaryType,
        meta: &DownloadMetadata,
    ) -> Result<(), BinaryError> {
        fs::create_dir_all(&self.binaries_dir).await?;
        let path = self.metadata_path(binary_type);
        let contents = serde_json::to_string_pretty(meta)?;
        fs::write(&path, contents).await?;
        Ok(())
    }

    /// Check whether a binary needs to be (re)downloaded using conditional
    /// HTTP headers (ETag / Last-Modified).
    ///
    /// Returns `true` if the binary should be downloaded, `false` if the
    /// cached version is up-to-date.
    pub async fn needs_download(&self, binary_type: BinaryType) -> Result<bool, BinaryError> {
        let installed = self.install_path(binary_type);
        if !installed.exists() {
            return Ok(true);
        }

        let stored = self.load_metadata(binary_type).await?;
        let stored = match stored {
            Some(m) => m,
            None => return Ok(true),
        };

        // Issue a HEAD request to check current remote headers.
        let url = self.download_url(binary_type);
        let resp = self.client.head(&url).send().await?;

        let etag = resp
            .headers()
            .get("etag")
            .and_then(|v| v.to_str().ok())
            .map(String::from);
        let last_modified = resp
            .headers()
            .get("last-modified")
            .and_then(|v| v.to_str().ok())
            .map(String::from);

        Ok(!is_up_to_date(&stored, etag.as_deref(), last_modified.as_deref()))
    }

    /// Download a binary and install it atomically.
    ///
    /// For yt-dlp, the binary is downloaded directly. For FFmpeg, the archive
    /// is downloaded and the `ffmpeg` binary is extracted from it.
    ///
    /// Progress events are emitted via the provided `AppHandle`.
    pub async fn download_binary(
        &self,
        binary_type: BinaryType,
        app_handle: &AppHandle,
    ) -> Result<PathBuf, BinaryError> {
        let url = self.download_url(binary_type);
        let install_path = self.install_path(binary_type);
        let part_path = self.part_path(binary_type);

        // Ensure the target directory exists.
        fs::create_dir_all(&self.binaries_dir).await?;

        // Emit 0% progress.
        emit_progress(app_handle, binary_type, 0.0);

        // Stream the download to a `.part` file.
        let resp = self.client.get(&url).send().await.map_err(|e| {
            emit_error(app_handle, binary_type, &format!("Download failed: {}", e));
            BinaryError::Http(e)
        })?;

        if !resp.status().is_success() {
            let msg = format!("HTTP {} for {}", resp.status(), url);
            emit_error(app_handle, binary_type, &msg);
            return Err(BinaryError::Other(msg));
        }

        let total_size = resp.content_length();
        let mut file = fs::File::create(&part_path).await.map_err(|e| {
            emit_error(app_handle, binary_type, &format!("Failed to create file: {}", e));
            BinaryError::Io(e)
        })?;

        let mut downloaded: u64 = 0;
        let mut stream = resp.bytes_stream();

        while let Some(chunk) = futures_util::StreamExt::next(&mut stream).await {
            let chunk = chunk.map_err(|e| {
                emit_error(app_handle, binary_type, &format!("Stream error: {}", e));
                BinaryError::Http(e)
            })?;
            file.write_all(&chunk).await.map_err(|e| {
                emit_error(app_handle, binary_type, &format!("Write error: {}", e));
                BinaryError::Io(e)
            })?;
            downloaded += chunk.len() as u64;

            if let Some(total) = total_size {
                if total > 0 {
                    let percent = (downloaded as f64 / total as f64) * 100.0;
                    emit_progress(app_handle, binary_type, percent);
                }
            }
        }

        file.flush().await?;
        drop(file);

        // Install the downloaded file.
        match binary_type {
            BinaryType::YtDlp => {
                // yt-dlp is a standalone binary — rename .part → final path.
                atomic_install(&part_path, &install_path).await?;
            }
            BinaryType::FFmpeg => {
                // FFmpeg comes as an archive — extract the binary.
                extract_ffmpeg(&part_path, &install_path).await?;
                // Clean up the archive after extraction.
                let _ = fs::remove_file(&part_path).await;
            }
        }

        // Extract metadata from a HEAD request (to store ETag for next time).
        let meta = self.fetch_metadata(&url).await?;
        self.save_metadata(binary_type, &meta).await?;

        emit_progress(app_handle, binary_type, 100.0);
        emit_complete(app_handle, binary_type);

        Ok(install_path)
    }

    /// Fetch download metadata (ETag / Last-Modified) via HEAD request.
    async fn fetch_metadata(&self, url: &str) -> Result<DownloadMetadata, BinaryError> {
        let resp = self.client.head(url).send().await?;

        let etag = resp
            .headers()
            .get("etag")
            .and_then(|v| v.to_str().ok())
            .map(String::from);
        let last_modified = resp
            .headers()
            .get("last-modified")
            .and_then(|v| v.to_str().ok())
            .map(String::from);

        Ok(DownloadMetadata {
            etag,
            last_modified,
            downloaded_at: chrono::Utc::now().to_rfc3339(),
            version: None,
        })
    }

    /// Get the version string of an installed binary by running `--version`.
    pub async fn get_version(binary_path: &Path) -> Result<String, String> {
        if !binary_path.exists() {
            return Err("Binary not found".to_string());
        }

        let output = tokio::process::Command::new(binary_path)
            .arg("--version")
            .output()
            .await
            .map_err(|e| format!("Failed to run binary: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Binary returned error: {}", stderr));
        }

        let version = String::from_utf8_lossy(&output.stdout);
        // Take the first line — yt-dlp reports "YYYY.MM.DD" on stdout.
        Ok(version.lines().next().unwrap_or("").trim().to_string())
    }
}

// ─── Archive extraction ──────────────────────────────────────────────────────

/// Extract the ffmpeg binary from a BtbN archive to the install path.
///
/// BtbN archives have the structure:
/// ```text
/// ffmpeg-master-latest-{platform}-gpl/
///   bin/
///     ffmpeg[.exe]
///     ffprobe[.exe]
///     ...
/// ```
///
/// We use the system `tar` command which handles both tar.xz (Unix) and zip
/// (Windows, modern macOS/Linux) formats.
async fn extract_ffmpeg(archive_path: &Path, install_path: &Path) -> Result<(), BinaryError> {
    let dest_dir = archive_path.parent().unwrap_or(Path::new("."));
    let temp_extract = dest_dir.join(".ffmpeg-extract-temp");

    // Clean up any previous partial extraction.
    if temp_extract.exists() {
        fs::remove_dir_all(&temp_extract).await?;
    }
    fs::create_dir_all(&temp_extract).await?;

    // Determine archive type and extract.
    let archive_str = archive_path.to_string_lossy();
    let extracted = if archive_str.ends_with(".zip") {
        extract_zip(&archive_str, temp_extract.as_path()).await?
    } else {
        // tar.xz — use system tar (auto-detects compression).
        extract_tar(&archive_str, temp_extract.as_path()).await?
    };

    if !extracted {
        return Err(BinaryError::Other(format!(
            "Failed to extract archive: {}",
            archive_str
        )));
    }

    // Find the ffmpeg binary inside the extracted directory tree.
    let expected_name = if install_path.to_string_lossy().ends_with(".exe") {
        "ffmpeg.exe"
    } else {
        "ffmpeg"
    };

    let ffmpeg_src = find_file_in_dir(&temp_extract, expected_name).await.ok_or_else(|| {
        BinaryError::Other(format!(
            "Could not find {} in extracted archive",
            expected_name
        ))
    })?;

    // Copy the binary to the install path and set permissions.
    fs::copy(&ffmpeg_src, install_path).await?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(install_path).await?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(install_path, perms).await?;
    }

    // Clean up temp directory.
    let _ = fs::remove_dir_all(&temp_extract).await;

    Ok(())
}

/// Extract a tar archive using the system `tar` command.
async fn extract_tar(archive: &str, dest: &Path) -> Result<bool, BinaryError> {
    let output = tokio::process::Command::new("tar")
        .args(["-xf", archive, "-C", dest.to_str().unwrap_or(".")])
        .output()
        .await
        .map_err(|e| BinaryError::Other(format!("tar command failed: {}", e)))?;

    Ok(output.status.success())
}

/// Extract a zip archive using the system `unzip` command (Unix) or `tar` (Windows).
async fn extract_zip(archive: &str, dest: &Path) -> Result<bool, BinaryError> {
    #[cfg(windows)]
    {
        // Windows 10+ ships with tar that handles zip.
        let output = tokio::process::Command::new("tar")
            .args(["-xf", archive, "-C", dest.to_str().unwrap_or(".")])
            .output()
            .await
            .map_err(|e| BinaryError::Other(format!("tar command failed: {}", e)))?;
        Ok(output.status.success())
    }
    #[cfg(not(windows))]
    {
        // Unix: prefer unzip, fall back to tar.
        let output = tokio::process::Command::new("unzip")
            .args(["-o", archive, "-d", dest.to_str().unwrap_or(".")])
            .output()
            .await
            .map_err(|e| BinaryError::Other(format!("unzip command failed: {}", e)))?;
        Ok(output.status.success())
    }
}

/// Recursively search for a file by name within a directory.
async fn find_file_in_dir(dir: &Path, name: &str) -> Option<PathBuf> {
    let mut entries = match fs::read_dir(dir).await {
        Ok(e) => e,
        Err(_) => return None,
    };

    while let Ok(Some(entry)) = entries.next_entry().await {
        let path = entry.path();
        if path.is_dir() {
            if let found @ Some(_) = Box::pin(find_file_in_dir(&path, name)).await {
                return found;
            }
        } else if path.file_name().and_then(|n| n.to_str()) == Some(name) {
            return Some(path);
        }
    }

    None
}

// ─── Free functions ───────────────────────────────────────────────────────────

/// Determine whether a stored download is still up-to-date based on
/// conditional HTTP headers.
///
/// ETag takes precedence; Last-Modified is the fallback when either side
/// lacks an ETag.
pub fn is_up_to_date(
    stored: &DownloadMetadata,
    remote_etag: Option<&str>,
    remote_last_modified: Option<&str>,
) -> bool {
    // Prefer ETag comparison when both sides have one.
    if let (Some(stored_etag), Some(remote_etag)) = (stored.etag.as_deref(), remote_etag) {
        return stored_etag == remote_etag;
    }

    // Fall back to Last-Modified.
    if let (Some(stored_lm), Some(remote_lm)) = (stored.last_modified.as_deref(), remote_last_modified) {
        return !stored_lm.is_empty() && stored_lm == remote_lm;
    }

    false
}

/// Atomically install a downloaded `.part` file to its final path.
///
/// On Unix, sets executable permissions (0o755).
async fn atomic_install(part_path: &Path, final_path: &Path) -> Result<(), BinaryError> {
    // Remove any existing final file to avoid rename errors.
    if final_path.exists() {
        fs::remove_file(final_path).await?;
    }

    // Rename .part → final (atomic on the same filesystem).
    fs::rename(part_path, final_path).await?;

    // Set executable permissions on Unix.
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(final_path).await?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(final_path, perms).await?;
    }

    Ok(())
}

// ─── Event emitters ───────────────────────────────────────────────────────────

fn emit_progress(app_handle: &AppHandle, binary_type: BinaryType, percent: f64) {
    let _ = app_handle.emit(
        "yt-dlp-binary-progress",
        serde_json::json!({
            "percent": percent,
            "binaryType": binary_type.as_str(),
        }),
    );
}

fn emit_complete(app_handle: &AppHandle, binary_type: BinaryType) {
    let _ = app_handle.emit(
        "yt-dlp-binary-complete",
        serde_json::json!({
            "binaryType": binary_type.as_str(),
        }),
    );
}

fn emit_error(app_handle: &AppHandle, binary_type: BinaryType, error: &str) {
    let _ = app_handle.emit(
        "yt-dlp-binary-error",
        serde_json::json!({
            "binaryType": binary_type.as_str(),
            "error": error,
        }),
    );
}

// ─── Unit tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod binary_manager_tests {
    use super::*;

    // ─── is_up_to_date ────────────────────────────────────────────────────

    #[test]
    fn test_is_up_to_date_matching_etag() {
        let stored = DownloadMetadata {
            etag: Some("\"abc123\"".to_string()),
            last_modified: None,
            downloaded_at: "2024-01-01T00:00:00Z".to_string(),
            version: None,
        };
        assert!(is_up_to_date(&stored, Some("\"abc123\""), None));
    }

    #[test]
    fn test_is_up_to_date_different_etag() {
        let stored = DownloadMetadata {
            etag: Some("\"abc123\"".to_string()),
            last_modified: Some("Mon, 01 Jan 2024 00:00:00 GMT".to_string()),
            downloaded_at: "2024-01-01T00:00:00Z".to_string(),
            version: None,
        };
        // ETag differs → not up-to-date, even though Last-Modified matches.
        assert!(!is_up_to_date(
            &stored,
            Some("\"xyz789\""),
            Some("Mon, 01 Jan 2024 00:00:00 GMT")
        ));
    }

    #[test]
    fn test_is_up_to_date_fallback_last_modified() {
        let stored = DownloadMetadata {
            etag: None,
            last_modified: Some("Mon, 01 Jan 2024 00:00:00 GMT".to_string()),
            downloaded_at: "2024-01-01T00:00:00Z".to_string(),
            version: None,
        };
        assert!(is_up_to_date(&stored, None, Some("Mon, 01 Jan 2024 00:00:00 GMT")));
    }

    #[test]
    fn test_is_up_to_date_no_stored_etag_fallback_lm() {
        let stored = DownloadMetadata {
            etag: Some("\"abc123\"".to_string()),
            last_modified: Some("Mon, 01 Jan 2024 00:00:00 GMT".to_string()),
            downloaded_at: "2024-01-01T00:00:00Z".to_string(),
            version: None,
        };
        // Stored has ETag but remote does not → fall through to Last-Modified.
        assert!(is_up_to_date(&stored, None, Some("Mon, 01 Jan 2024 00:00:00 GMT")));
    }

    #[test]
    fn test_is_up_to_date_empty_headers() {
        let stored = DownloadMetadata {
            etag: None,
            last_modified: None,
            downloaded_at: "2024-01-01T00:00:00Z".to_string(),
            version: None,
        };
        assert!(!is_up_to_date(&stored, None, None));
    }

    #[test]
    fn test_is_up_to_date_empty_last_modified_no_match() {
        let stored = DownloadMetadata {
            etag: Some("\"abc\"".to_string()),
            last_modified: Some("".to_string()),
            downloaded_at: "2024-01-01T00:00:00Z".to_string(),
            version: None,
        };
        // ETag present on both → uses ETag path, not Last-Modified.
        assert!(is_up_to_date(&stored, Some("\"abc\""), Some("something")));
    }

    // ─── download_url ────────────────────────────────────────────────────

    #[test]
    fn test_download_url_yt_dlp_linux_x64() {
        let client = Client::new();
        let mgr =
            BinaryManager::with_dir(client, Platform::LinuxX64, PathBuf::from("/tmp/test"));
        assert_eq!(
            mgr.download_url(BinaryType::YtDlp),
            "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp_linux"
        );
    }

    #[test]
    fn test_download_url_yt_dlp_windows() {
        let client = Client::new();
        let mgr =
            BinaryManager::with_dir(client, Platform::WinX64, PathBuf::from("/tmp/test"));
        assert_eq!(
            mgr.download_url(BinaryType::YtDlp),
            "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp.exe"
        );
    }

    #[test]
    fn test_download_url_ffmpeg_linux_x64() {
        let client = Client::new();
        let mgr =
            BinaryManager::with_dir(client, Platform::LinuxX64, PathBuf::from("/tmp/test"));
        assert_eq!(
            mgr.download_url(BinaryType::FFmpeg),
            "https://github.com/BtbN/FFmpeg-Builds/releases/download/latest/ffmpeg-master-latest-linux64-gpl.tar.xz"
        );
    }

    #[test]
    fn test_download_url_ffmpeg_windows() {
        let client = Client::new();
        let mgr =
            BinaryManager::with_dir(client, Platform::WinX64, PathBuf::from("/tmp/test"));
        assert_eq!(
            mgr.download_url(BinaryType::FFmpeg),
            "https://github.com/BtbN/FFmpeg-Builds/releases/download/latest/ffmpeg-master-latest-win64-gpl.zip"
        );
    }

    // ─── install_path / part_path / metadata_path ─────────────────────────

    #[test]
    fn test_install_path_yt_dlp() {
        let client = Client::new();
        let mgr =
            BinaryManager::with_dir(client, Platform::LinuxX64, PathBuf::from("/tmp/test"));
        assert_eq!(
            mgr.install_path(BinaryType::YtDlp),
            PathBuf::from("/tmp/test/yt-dlp_x86_64-unknown-linux-gnu")
        );
    }

    #[test]
    fn test_install_path_ffmpeg() {
        let client = Client::new();
        let mgr =
            BinaryManager::with_dir(client, Platform::LinuxX64, PathBuf::from("/tmp/test"));
        assert_eq!(
            mgr.install_path(BinaryType::FFmpeg),
            PathBuf::from("/tmp/test/ffmpeg")
        );
    }

    #[test]
    fn test_install_path_ffmpeg_windows() {
        let client = Client::new();
        let mgr =
            BinaryManager::with_dir(client, Platform::WinX64, PathBuf::from("/tmp/test"));
        assert_eq!(
            mgr.install_path(BinaryType::FFmpeg),
            PathBuf::from("/tmp/test/ffmpeg.exe")
        );
    }

    #[test]
    fn test_part_path() {
        let client = Client::new();
        let mgr =
            BinaryManager::with_dir(client, Platform::LinuxX64, PathBuf::from("/tmp/test"));
        assert_eq!(
            mgr.part_path(BinaryType::YtDlp),
            PathBuf::from("/tmp/test/yt-dlp_x86_64-unknown-linux-gnu.part")
        );
    }

    #[test]
    fn test_metadata_path() {
        let client = Client::new();
        let mgr =
            BinaryManager::with_dir(client, Platform::LinuxX64, PathBuf::from("/tmp/test"));
        assert_eq!(
            mgr.metadata_path(BinaryType::YtDlp),
            PathBuf::from("/tmp/test/yt-dlp_x86_64-unknown-linux-gnu.download.json")
        );
    }

    // ─── is_installed ────────────────────────────────────────────────────

    #[test]
    fn test_is_installed_false_when_missing() {
        let client = Client::new();
        let mgr = BinaryManager::with_dir(
            client,
            Platform::LinuxX64,
            PathBuf::from("/tmp/nonexistent_dir_12345"),
        );
        assert!(!mgr.is_installed(BinaryType::YtDlp));
    }

    // ─── find_file_in_dir ────────────────────────────────────────────────

    #[tokio::test]
    async fn test_find_file_in_dir_finds_nested() {
        let temp = tempfile::tempdir().unwrap();
        let nested = temp.path().join("a").join("b");
        fs::create_dir_all(&nested).await.unwrap();
        fs::write(nested.join("target.bin"), b"data").await.unwrap();

        let found = find_file_in_dir(temp.path(), "target.bin").await;
        assert!(found.is_some());
        assert_eq!(found.unwrap().file_name().unwrap(), "target.bin");
    }

    #[tokio::test]
    async fn test_find_file_in_dir_missing() {
        let temp = tempfile::tempdir().unwrap();
        fs::write(temp.path().join("other.bin"), b"data").await.unwrap();

        let found = find_file_in_dir(temp.path(), "missing.bin").await;
        assert!(found.is_none());
    }

    // ─── atomic_install (rename logic) ───────────────────────────────────

    #[tokio::test]
    async fn test_atomic_install_renames_part_to_final() {
        let temp = tempfile::tempdir().unwrap();
        let part = temp.path().join("test.bin.part");
        let final_path = temp.path().join("test.bin");

        // Create a .part file with some content.
        fs::write(&part, b"binary content").await.unwrap();

        atomic_install(&part, &final_path).await.unwrap();

        assert!(final_path.exists(), "final path should exist after install");
        assert!(!part.exists(), ".part file should be removed after install");

        let contents = fs::read(&final_path).await.unwrap();
        assert_eq!(contents, b"binary content");
    }

    #[tokio::test]
    async fn test_atomic_install_overwrites_existing() {
        let temp = tempfile::tempdir().unwrap();
        let part = temp.path().join("test.bin.part");
        let final_path = temp.path().join("test.bin");

        // Create both .part and existing final file.
        fs::write(&part, b"new content").await.unwrap();
        fs::write(&final_path, b"old content").await.unwrap();

        atomic_install(&part, &final_path).await.unwrap();

        let contents = fs::read(&final_path).await.unwrap();
        assert_eq!(contents, b"new content");
    }

    // ─── metadata round-trip ─────────────────────────────────────────────

    #[tokio::test]
    async fn test_save_and_load_metadata() {
        let temp = tempfile::tempdir().unwrap();
        let client = Client::new();
        let mgr = BinaryManager::with_dir(
            client,
            Platform::LinuxX64,
            temp.path().to_path_buf(),
        );

        let meta = DownloadMetadata {
            etag: Some("\"test-etag\"".to_string()),
            last_modified: Some("Mon, 01 Jan 2024 00:00:00 GMT".to_string()),
            downloaded_at: "2024-01-01T00:00:00Z".to_string(),
            version: Some("2024.01.01".to_string()),
        };

        mgr.save_metadata(BinaryType::YtDlp, &meta).await.unwrap();
        let loaded = mgr.load_metadata(BinaryType::YtDlp).await.unwrap();

        assert_eq!(loaded, Some(meta));
    }
}

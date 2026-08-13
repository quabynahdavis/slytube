use serde::{Deserialize, Serialize};

/// Download mode for yt-dlp.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DownloadMode {
    Video,
    Audio,
    Custom,
}

/// Arguments for starting a yt-dlp download.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct YtDlpDownloadArgs {
    pub video_id: String,
    #[serde(default)]
    pub video_ids: Option<Vec<String>>,
    #[serde(default)]
    pub playlist_id: Option<String>,
    pub mode: DownloadMode,
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

/// Status of a yt-dlp download.
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

/// Video info from yt-dlp.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoInfo {
    pub id: String,
    pub title: String,
    pub duration: Option<f64>,
    pub thumbnail: Option<String>,
    pub description: Option<String>,
    pub uploader: Option<String>,
    pub uploader_id: Option<String>,
    pub view_count: Option<u64>,
    pub formats: Option<Vec<serde_json::Value>>,
}

/// Allowed video formats.
pub const VIDEO_FORMATS: &[&str] = &["mp4", "mkv", "webm"];
pub const VIDEO_CODECS: &[&str] = &["h264", "h265", "vp9", "av1"];
pub const AUDIO_FORMATS: &[&str] = &["mp3", "m4a", "opus", "flac"];
pub const SPONSORBLOCK_CATEGORIES: &[&str] = &[
    "sponsor",
    "intro",
    "outro",
    "selfpromo",
    "interaction",
    "music_offtopic",
    "preview",
    "filler",
];

/// Supported platforms for binary downloads.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Platform {
    LinuxX64,
    LinuxArm64,
    MacOX64,
    MacOArm64,
    WinX64,
    WinArm64,
}

impl Platform {
    /// Detect the current platform from compile-time constants.
    pub fn current() -> Self {
        match (std::env::consts::OS, std::env::consts::ARCH) {
            ("linux", "x86_64") => Platform::LinuxX64,
            ("linux", "aarch64") => Platform::LinuxArm64,
            ("macos", "x86_64") => Platform::MacOX64,
            ("macos", "aarch64") => Platform::MacOArm64,
            ("windows", "x86_64") => Platform::WinX64,
            ("windows", "aarch64") => Platform::WinArm64,
            (os, arch) => {
                // Fallback for unknown platforms: assume Linux x64.
                // This should never happen on supported platforms.
                tracing::warn!(
                    "Unknown platform (os={}, arch={}), falling back to LinuxX64",
                    os,
                    arch
                );
                Platform::LinuxX64
            }
        }
    }

    /// Construct a Platform from explicit OS/arch strings (for testing).
    pub fn from_parts(os: &str, arch: &str) -> Option<Self> {
        match (os, arch) {
            ("linux", "x86_64") => Some(Platform::LinuxX64),
            ("linux", "aarch64") => Some(Platform::LinuxArm64),
            ("macos", "x86_64") => Some(Platform::MacOX64),
            ("macos", "aarch64") => Some(Platform::MacOArm64),
            ("windows", "x86_64") => Some(Platform::WinX64),
            ("windows", "aarch64") => Some(Platform::WinArm64),
            _ => None,
        }
    }

    /// The yt-dlp binary name for this platform on GitHub releases.
    pub fn yt_dlp_name(&self) -> &str {
        match self {
            Platform::LinuxX64 => "yt-dlp_linux",
            Platform::LinuxArm64 => "yt-dlp_linux_aarch64",
            // macOS uses a universal binary for both architectures
            Platform::MacOX64 | Platform::MacOArm64 => "yt-dlp_macos",
            Platform::WinX64 | Platform::WinArm64 => "yt-dlp.exe",
        }
    }

    /// The FFmpeg archive name for this platform on GitHub releases.
    pub fn ffmpeg_name(&self) -> &str {
        match self {
            Platform::LinuxX64 => "ffmpeg-master-latest-linux64-gpl.tar.xz",
            Platform::LinuxArm64 => "ffmpeg-master-latest-linuxarm64-gpl.tar.xz",
            Platform::MacOX64 => "ffmpeg-master-latest-macos64-gpl.tar.xz",
            Platform::MacOArm64 => "ffmpeg-master-latest-macosarm64-gpl.tar.xz",
            Platform::WinX64 => "ffmpeg-master-latest-win64-gpl.zip",
            Platform::WinArm64 => "ffmpeg-master-latest-winarm64-gpl.zip",
        }
    }

    /// The Rust target triple for Tauri sidecar naming convention.
    pub fn target_triple(&self) -> &str {
        match self {
            Platform::LinuxX64 => "x86_64-unknown-linux-gnu",
            Platform::LinuxArm64 => "aarch64-unknown-linux-gnu",
            Platform::MacOX64 => "x86_64-apple-darwin",
            Platform::MacOArm64 => "aarch64-apple-darwin",
            Platform::WinX64 => "x86_64-pc-windows-msvc",
            Platform::WinArm64 => "aarch64-pc-windows-msvc",
        }
    }

    /// The full sidecar binary name for the yt-dlp binary.
    pub fn sidecar_name(&self) -> String {
        let triple = self.target_triple();
        if self.is_windows() {
            format!("yt-dlp_{}.exe", triple)
        } else {
            format!("yt-dlp_{}", triple)
        }
    }

    /// The final installed FFmpeg binary name.
    pub fn ffmpeg_binary_name(&self) -> &str {
        if self.is_windows() {
            "ffmpeg.exe"
        } else {
            "ffmpeg"
        }
    }

    pub fn is_windows(&self) -> bool {
        matches!(self, Platform::WinX64 | Platform::WinArm64)
    }

    pub fn is_unix(&self) -> bool {
        !self.is_windows()
    }
}

/// Type of binary to download.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BinaryType {
    YtDlp,
    FFmpeg,
}

impl BinaryType {
    pub fn as_str(&self) -> &str {
        match self {
            BinaryType::YtDlp => "yt_dlp",
            BinaryType::FFmpeg => "ffmpeg",
        }
    }
}

impl std::str::FromStr for BinaryType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "yt_dlp" => Ok(BinaryType::YtDlp),
            "ffmpeg" => Ok(BinaryType::FFmpeg),
            _ => Err(format!("Unknown binary type: {}", s)),
        }
    }
}

/// Metadata stored alongside a downloaded binary (`.download.json` sidecar file).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DownloadMetadata {
    /// The ETag from the HTTP response, if available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub etag: Option<String>,
    /// The Last-Modified header, used as fallback when ETag is absent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_modified: Option<String>,
    /// ISO-8601 timestamp of when this download completed.
    pub downloaded_at: String,
    /// The reported version of the binary (from `--version` output), if known.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

/// Result of checking binary availability and versions.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BinaryVersionInfo {
    pub yt_dlp_available: bool,
    pub ffmpeg_available: bool,
    pub yt_dlp_version: String,
    pub ffmpeg_version: String,
}

// ─── Platform unit tests ──────────────────────────────────────────────────────

#[cfg(test)]
mod platform_tests {
    use super::*;

    #[test]
    fn test_from_parts_linux_x64() {
        assert_eq!(Platform::from_parts("linux", "x86_64"), Some(Platform::LinuxX64));
    }

    #[test]
    fn test_from_parts_linux_arm64() {
        assert_eq!(Platform::from_parts("linux", "aarch64"), Some(Platform::LinuxArm64));
    }

    #[test]
    fn test_from_parts_macos_x64() {
        assert_eq!(Platform::from_parts("macos", "x86_64"), Some(Platform::MacOX64));
    }

    #[test]
    fn test_from_parts_macos_arm64() {
        assert_eq!(Platform::from_parts("macos", "aarch64"), Some(Platform::MacOArm64));
    }

    #[test]
    fn test_from_parts_windows_x64() {
        assert_eq!(Platform::from_parts("windows", "x86_64"), Some(Platform::WinX64));
    }

    #[test]
    fn test_from_parts_windows_arm64() {
        assert_eq!(Platform::from_parts("windows", "aarch64"), Some(Platform::WinArm64));
    }

    #[test]
    fn test_from_parts_unknown() {
        assert_eq!(Platform::from_parts("freebsd", "x86_64"), None);
        assert_eq!(Platform::from_parts("linux", "i686"), None);
    }

    #[test]
    fn test_yt_dlp_name() {
        assert_eq!(Platform::LinuxX64.yt_dlp_name(), "yt-dlp_linux");
        assert_eq!(Platform::LinuxArm64.yt_dlp_name(), "yt-dlp_linux_aarch64");
        assert_eq!(Platform::MacOX64.yt_dlp_name(), "yt-dlp_macos");
        assert_eq!(Platform::MacOArm64.yt_dlp_name(), "yt-dlp_macos");
        assert_eq!(Platform::WinX64.yt_dlp_name(), "yt-dlp.exe");
        assert_eq!(Platform::WinArm64.yt_dlp_name(), "yt-dlp.exe");
    }

    #[test]
    fn test_target_triple() {
        assert_eq!(Platform::LinuxX64.target_triple(), "x86_64-unknown-linux-gnu");
        assert_eq!(Platform::LinuxArm64.target_triple(), "aarch64-unknown-linux-gnu");
        assert_eq!(Platform::MacOX64.target_triple(), "x86_64-apple-darwin");
        assert_eq!(Platform::MacOArm64.target_triple(), "aarch64-apple-darwin");
        assert_eq!(Platform::WinX64.target_triple(), "x86_64-pc-windows-msvc");
        assert_eq!(Platform::WinArm64.target_triple(), "aarch64-pc-windows-msvc");
    }

    #[test]
    fn test_sidecar_name() {
        assert_eq!(
            Platform::LinuxX64.sidecar_name(),
            "yt-dlp_x86_64-unknown-linux-gnu"
        );
        assert_eq!(
            Platform::LinuxArm64.sidecar_name(),
            "yt-dlp_aarch64-unknown-linux-gnu"
        );
        assert_eq!(
            Platform::MacOX64.sidecar_name(),
            "yt-dlp_x86_64-apple-darwin"
        );
        assert_eq!(
            Platform::MacOArm64.sidecar_name(),
            "yt-dlp_aarch64-apple-darwin"
        );
        assert_eq!(
            Platform::WinX64.sidecar_name(),
            "yt-dlp_x86_64-pc-windows-msvc.exe"
        );
        assert_eq!(
            Platform::WinArm64.sidecar_name(),
            "yt-dlp_aarch64-pc-windows-msvc.exe"
        );
    }

    #[test]
    fn test_is_windows() {
        assert!(Platform::WinX64.is_windows());
        assert!(Platform::WinArm64.is_windows());
        assert!(!Platform::LinuxX64.is_windows());
        assert!(!Platform::MacOX64.is_windows());
    }

    #[test]
    fn test_is_unix() {
        assert!(Platform::LinuxX64.is_unix());
        assert!(Platform::MacOX64.is_unix());
        assert!(!Platform::WinX64.is_unix());
    }

    #[test]
    fn test_binary_type_from_str() {
        assert_eq!("yt_dlp".parse::<BinaryType>(), Ok(BinaryType::YtDlp));
        assert_eq!("ffmpeg".parse::<BinaryType>(), Ok(BinaryType::FFmpeg));
        assert!("unknown".parse::<BinaryType>().is_err());
    }

    #[test]
    fn test_binary_type_as_str() {
        assert_eq!(BinaryType::YtDlp.as_str(), "yt_dlp");
        assert_eq!(BinaryType::FFmpeg.as_str(), "ffmpeg");
    }
}

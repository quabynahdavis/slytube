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

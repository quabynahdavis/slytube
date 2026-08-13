pub mod dearrow;
pub mod ryd;
pub mod sponsorblock;

use thiserror::Error;
use tauri::State;

use crate::http_client::SharedHttpClient;

/// Default User-Agent for community API requests (SponsorBlock / DeArrow).
///
/// Uses the OpenTubeX identifier for compatibility with these services.
pub const DEFAULT_USER_AGENT: &str = "OpenTubeX/0.31.0";

/// Errors from community API integrations (SponsorBlock, DeArrow, RYD).
#[derive(Debug, Error)]
pub enum CommunityError {
    #[error("HTTP request failed: {0}")]
    Http(String),

    #[error("Failed to parse response: {0}")]
    Parse(String),

    #[error("Invalid video ID: {0}")]
    InvalidVideoId(String),
}

/// Computes the SponsorBlock hash prefix for a video ID.
///
/// Returns the first 4 hex characters of the SHA-256 hash of the video ID,
/// as required by the SponsorBlock and DeArrow APIs.
pub fn hash_video_id(video_id: &str) -> String {
    use sha2::{Digest, Sha256};
    let hash = Sha256::digest(video_id.as_bytes());
    hex::encode(&hash[..2]) // first 2 bytes = 4 hex chars
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_video_id_never_gonna() {
        // SHA-256("dQw4w9WgXcQ") starts with 5f6b...
        // Verified via: echo -n "dQw4w9WgXcQ" | sha256sum
        let result = hash_video_id("dQw4w9WgXcQ");
        assert_eq!(result.len(), 4);
        // First 4 hex chars of SHA-256("dQw4w9WgXcQ")
        assert_eq!(result, "5f6b");
    }

    #[test]
    fn test_hash_video_id_short() {
        let result = hash_video_id("abc");
        assert_eq!(result.len(), 4);
    }

    #[test]
    fn test_hash_video_id_deterministic() {
        let a = hash_video_id("test_video_123");
        let b = hash_video_id("test_video_123");
        assert_eq!(a, b);
    }

    #[test]
    fn test_hash_video_id_different_ids() {
        let a = hash_video_id("aaaa");
        let b = hash_video_id("bbbb");
        assert_ne!(a, b);
    }
}

// ─── Tauri commands ───────────────────────────────────────────────────────────

/// Tauri command: get SponsorBlock segments for a video.
#[tauri::command]
pub async fn sponsorblock_get_segments(
    http_client: State<'_, SharedHttpClient>,
    video_id: String,
    categories: Vec<String>,
    action_types: Vec<String>,
) -> Result<Vec<sponsorblock::SponsorBlockSegment>, String> {
    sponsorblock::get_segments(&http_client, &video_id, &categories, &action_types).await
}

/// Tauri command: get SponsorBlock labels for a video.
#[tauri::command]
pub async fn sponsorblock_get_labels(
    http_client: State<'_, SharedHttpClient>,
    video_id: String,
) -> Result<Option<sponsorblock::SponsorBlockSegment>, String> {
    sponsorblock::get_video_labels(&http_client, &video_id).await
}

/// Tauri command: submit SponsorBlock segments for a video.
#[tauri::command]
pub async fn sponsorblock_submit_segments(
    http_client: State<'_, SharedHttpClient>,
    video_id: String,
    video_duration: f64,
    user_id: String,
    segments: Vec<sponsorblock::SponsorBlockSegment>,
) -> Result<(), String> {
    sponsorblock::submit_segments(&http_client, &video_id, video_duration, &user_id, &segments).await
}

/// Tauri command: vote on a SponsorBlock segment.
#[tauri::command]
pub async fn sponsorblock_vote(
    http_client: State<'_, SharedHttpClient>,
    video_id: String,
    uuid: String,
    user_id: String,
    vote_type: i32,
) -> Result<(), String> {
    sponsorblock::vote_on_segment(&http_client, &video_id, &uuid, &user_id, vote_type).await
}

/// Tauri command: get DeArrow branding data for a video.
#[tauri::command]
pub async fn dearrow_get_data(
    http_client: State<'_, SharedHttpClient>,
    video_id: String,
) -> Result<Option<dearrow::DeArrowData>, String> {
    dearrow::get_dearrow_data(&http_client, &video_id).await
}

/// Tauri command: get DeArrow thumbnail URL for a video at a timestamp.
#[tauri::command]
pub async fn dearrow_get_thumbnail(
    http_client: State<'_, SharedHttpClient>,
    video_id: String,
    timestamp: f64,
) -> Result<Option<String>, String> {
    dearrow::get_thumbnail(&http_client, &video_id, timestamp).await
}

/// Tauri command: get Return YouTube Dislike count for a video.
#[tauri::command]
pub async fn ryd_get_dislikes(
    http_client: State<'_, SharedHttpClient>,
    video_id: String,
    base_url: Option<String>,
) -> Result<u64, String> {
    ryd::get_dislikes(&http_client, &video_id, base_url.as_deref()).await
}

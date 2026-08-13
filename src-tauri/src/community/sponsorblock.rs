use std::sync::Mutex;

use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::community::hash_video_id;
use crate::http_client::SharedHttpClient;

/// Default SponsorBlock API base URL.
pub const SPONSORBLOCK_URL: &str = "https://sponsor.ajay.app";

/// SponsorBlock segment as returned by the API.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SponsorBlockSegment {
    pub uuid: String,
    pub category: String,
    #[serde(rename = "actionType")]
    pub action_type: String,
    pub description: String,
    pub segment: [f64; 2],
    pub video_duration: f64,
    pub votes: i32,
    pub locked: bool,
}

/// A single segment entry inside the API response.
#[derive(Debug, Deserialize)]
struct SegmentInfo {
    #[serde(rename = "category")]
    category: String,
    #[serde(rename = "actionType")]
    action_type: String,
    #[serde(default)]
    description: String,
    segment: [f64; 2],
    #[serde(rename = "videoDuration")]
    video_duration: f64,
    votes: i32,
    locked: bool,
    uuid: String,
}

impl From<SegmentInfo> for SponsorBlockSegment {
    fn from(info: SegmentInfo) -> Self {
        Self {
            uuid: info.uuid,
            category: info.category,
            action_type: info.action_type,
            description: info.description,
            segment: info.segment,
            video_duration: info.video_duration,
            votes: info.votes,
            locked: info.locked,
        }
    }
}

/// Builds the SponsorBlock segments URL with query parameters.
fn build_segments_url(
    base_url: &str,
    video_id: &str,
    categories: &[String],
    action_types: &[String],
) -> String {
    let hash = hash_video_id(video_id);
    let mut url = format!("{base_url}/api/skipSegments/{hash}");

    let mut params = Vec::new();
    if !categories.is_empty() {
        let cats = serde_json::to_string(categories).unwrap_or_default();
        params.push(format!("categories={}", urlencoding::encode(&cats)));
    }
    if !action_types.is_empty() {
        let actions = serde_json::to_string(action_types).unwrap_or_default();
        params.push(format!("actionTypes={}", urlencoding::encode(&actions)));
    }

    if !params.is_empty() {
        url.push('?');
        url.push_str(&params.join("&"));
    }

    url
}

/// Fetches skip segments for a video from the SponsorBlock API.
///
/// Returns an empty vector if the video has no segments (HTTP 404).
pub async fn get_segments(
    http_client: &SharedHttpClient,
    video_id: &str,
    categories: &[String],
    action_types: &[String],
) -> Result<Vec<SponsorBlockSegment>, String> {
    let url = build_segments_url(SPONSORBLOCK_URL, video_id, categories, action_types);
    tracing::debug!("SponsorBlock get_segments: {}", url);

    let response = http_client.client().get(&url).send().await;

    match response {
        Ok(resp) => {
            let status = resp.status();
            if status.as_u16() == 404 {
                // No segments for this video
                return Ok(Vec::new());
            }
            if !status.is_success() {
                let body = resp.text().await.unwrap_or_default();
                return Err(format!("SponsorBlock returned {status}: {body}"));
            }
            let segments: Vec<SegmentInfo> = resp
                .json()
                .await
                .map_err(|e| format!("Failed to parse SponsorBlock response: {e}"))?;
            Ok(segments.into_iter().map(Into::into).collect())
        }
        Err(e) => Err(format!("SponsorBlock request failed: {e}")),
    }
}

/// Fetches full-video labels (sponsor/selfpromo/exclusive_access) for a video.
///
/// Returns `None` if no labels exist (HTTP 404).
pub async fn get_video_labels(
    http_client: &SharedHttpClient,
    video_id: &str,
) -> Result<Option<SponsorBlockSegment>, String> {
    let hash = hash_video_id(video_id);
    let url = format!("{SPONSORBLOCK_URL}/api/videoLabels/{hash}");
    tracing::debug!("SponsorBlock get_video_labels: {}", url);

    let response = http_client.client().get(&url).send().await;

    match response {
        Ok(resp) => {
            let status = resp.status();
            if status.as_u16() == 404 {
                return Ok(None);
            }
            if !status.is_success() {
                let body = resp.text().await.unwrap_or_default();
                return Err(format!("SponsorBlock returned {status}: {body}"));
            }
            // videoLabels returns an array of segment-like objects
            let segments: Vec<SegmentInfo> = resp
                .json()
                .await
                .map_err(|e| format!("Failed to parse videoLabels response: {e}"))?;
            Ok(segments.into_iter().next().map(Into::into))
        }
        Err(e) => Err(format!("SponsorBlock request failed: {e}")),
    }
}

/// Submits segments for a video to the SponsorBlock API.
pub async fn submit_segments(
    http_client: &SharedHttpClient,
    video_id: &str,
    video_duration: f64,
    user_id: &str,
    segments: &[SponsorBlockSegment],
) -> Result<(), String> {
    let url = format!("{SPONSORBLOCK_URL}/api/skipSegments");

    let body = serde_json::json!({
        "videoID": video_id,
        "userID": user_id,
        "videoDuration": video_duration,
        "userAgent": crate::community::DEFAULT_USER_AGENT,
        "segments": segments.iter().map(|s| serde_json::json!({
            "category": s.category,
            "actionType": s.action_type,
            "segment": s.segment,
            "description": s.description,
        })).collect::<Vec<_>>(),
    });

    tracing::debug!("SponsorBlock submit_segments for {}", video_id);

    let resp = http_client
        .client()
        .post(&url)
        .header("Accept", "application/json")
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("SponsorBlock submit failed: {e}"))?;

    let status = resp.status();
    if !status.is_success() {
        let body_text = resp.text().await.unwrap_or_default();
        return Err(format!("SponsorBlock submit returned {status}: {body_text}"));
    }

    Ok(())
}

/// Votes on a segment (up/down/undo).
///
/// `vote_type`: 0 = downvote, 1 = upvote, 20 = undo.
pub async fn vote_on_segment(
    http_client: &SharedHttpClient,
    video_id: &str,
    uuid: &str,
    user_id: &str,
    vote_type: i32,
) -> Result<(), String> {
    let url = format!("{SPONSORBLOCK_URL}/api/voteOnSponsorTime");

    let body = serde_json::json!({
        "UUID": uuid,
        "videoID": video_id,
        "userID": user_id,
        "type": vote_type,
    });

    tracing::debug!("SponsorBlock vote_on_segment for uuid {}", uuid);

    let resp = http_client
        .client()
        .post(&url)
        .header("Accept", "application/json")
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("SponsorBlock vote failed: {e}"))?;

    let status = resp.status();
    if !status.is_success() {
        let body_text = resp.text().await.unwrap_or_default();
        return Err(format!("SponsorBlock vote returned {status}: {body_text}"));
    }

    Ok(())
}

/// Manages the local SponsorBlock user ID.
///
/// The user ID is a 30-character random string that persists across sessions.
pub struct UserIdStore {
    id: Mutex<String>,
}

impl UserIdStore {
    pub fn new() -> Self {
        Self {
            id: Mutex::new(generate_user_id()),
        }
    }

    pub fn get(&self) -> String {
        self.id.lock().unwrap().clone()
    }

    pub fn set(&self, id: String) {
        *self.id.lock().unwrap() = id;
    }
}

/// Generates a random 30-character hex user ID.
fn generate_user_id() -> String {
    let mut bytes = vec![0u8; 15]; // 15 bytes = 30 hex chars
    rand::RngCore::fill_bytes(&mut rand::thread_rng(), &mut bytes);
    hex::encode(bytes)
}

/// Strips DeArrow formatting markers from a title.
///
/// DeArrow titles may contain `(^|\s)>(\S)` markers that indicate formatting.
/// This function strips them to produce a clean title.
pub fn strip_dearrow_formatting(title: &str) -> String {
    // The regex `(^|\s)>(\S)` captures a `>` following start-of-string or
    // whitespace, followed by a non-whitespace character. We replace with
    // the whitespace (if any) and the non-whitespace character.
    let re = Regex::new(r"(^|\s)>(\S)").unwrap();
    re.replace_all(title, "$1$2").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_segments_url_no_params() {
        let url = build_segments_url(SPONSORBLOCK_URL, "dQw4w9WgXcQ", &[], &[]);
        assert_eq!(url, format!("{SPONSORBLOCK_URL}/api/skipSegments/5f6b"));
    }

    #[test]
    fn test_build_segments_url_with_categories() {
        let cats = vec!["sponsor".to_string(), "intro".to_string()];
        let url = build_segments_url(SPONSORBLOCK_URL, "test", &cats, &[]);
        assert!(url.contains("/api/skipSegments/"));
        assert!(url.contains("categories="));
        assert!(url.contains("sponsor"));
    }

    #[test]
    fn test_build_segments_url_with_action_types() {
        let actions = vec!["skip".to_string()];
        let url = build_segments_url(SPONSORBLOCK_URL, "test", &[], &actions);
        assert!(url.contains("actionTypes="));
        assert!(url.contains("skip"));
    }

    #[test]
    fn test_build_segments_url_with_both() {
        let cats = vec!["sponsor".to_string()];
        let actions = vec!["skip".to_string(), "mute".to_string()];
        let url = build_segments_url(SPONSORBLOCK_URL, "test", &cats, &actions);
        assert!(url.contains("categories="));
        assert!(url.contains("&"));
        assert!(url.contains("actionTypes="));
    }

    #[test]
    fn test_strip_dearrow_formatting_start() {
        let title = ">Sponsor message here";
        assert_eq!(strip_dearrow_formatting(title), "Sponsor message here");
    }

    #[test]
    fn test_strip_dearrow_formatting_space() {
        let title = "Hello >World rest";
        assert_eq!(strip_dearrow_formatting(title), "Hello World rest");
    }

    #[test]
    fn test_strip_dearrow_formatting_multiple() {
        let title = ">Hello >World";
        assert_eq!(strip_dearrow_formatting(title), "Hello World");
    }

    #[test]
    fn test_strip_dearrow_formatting_no_markers() {
        let title = "Normal Title Here";
        assert_eq!(strip_dearrow_formatting(title), "Normal Title Here");
    }

    #[test]
    fn test_generate_user_id_length() {
        let id = generate_user_id();
        assert_eq!(id.len(), 30);
    }

    #[test]
    fn test_generate_user_id_is_hex() {
        let id = generate_user_id();
        assert!(id.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_generate_user_id_unique() {
        let a = generate_user_id();
        let b = generate_user_id();
        assert_ne!(a, b);
    }

    #[test]
    fn test_user_id_store() {
        let store = UserIdStore::new();
        let id = store.get();
        assert_eq!(id.len(), 30);

        store.set("custom_id_123456789012345".to_string());
        assert_eq!(store.get(), "custom_id_123456789012345");
    }

    #[test]
    fn test_segment_info_into() {
        let info = SegmentInfo {
            uuid: "abc-123".to_string(),
            category: "sponsor".to_string(),
            action_type: "skip".to_string(),
            description: "Sponsor".to_string(),
            segment: [10.0, 20.0],
            video_duration: 100.0,
            votes: 5,
            locked: false,
        };
        let seg: SponsorBlockSegment = info.into();
        assert_eq!(seg.uuid, "abc-123");
        assert_eq!(seg.category, "sponsor");
        assert_eq!(seg.action_type, "skip");
        assert_eq!(seg.segment, [10.0, 20.0]);
        assert_eq!(seg.votes, 5);
    }
}

use serde::Deserialize;

use crate::community::hash_video_id;
use crate::http_client::SharedHttpClient;

/// Default DeArrow thumbnail generator URL.
pub const DEARROW_THUMBNAIL_URL: &str = "https://dearrow-thumb.ajay.app";

/// DeArrow branding data for a video.
#[derive(Debug, Clone, serde::Serialize)]
pub struct DeArrowData {
    pub titles: Vec<DeArrowTitle>,
    pub thumbnails: Vec<DeArrowThumbnail>,
    pub video_duration: Option<f64>,
    pub random_time: f64,
}

/// A single DeArrow title submission.
#[derive(Debug, Clone, serde::Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeArrowTitle {
    pub title: String,
    pub locked: bool,
    pub votes: i32,
}

/// A single DeArrow thumbnail submission.
#[derive(Debug, Clone, serde::Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeArrowThumbnail {
    pub timestamp: f64,
    pub locked: bool,
    pub votes: i32,
}

/// Raw response from the DeArrow branding API.
#[derive(Debug, Deserialize)]
struct DeArrowResponse {
    titles: Vec<DeArrowTitle>,
    thumbnails: Vec<DeArrowThumbnail>,
    #[serde(rename = "videoDuration")]
    video_duration: Option<f64>,
    #[serde(rename = "randomTime")]
    random_time: f64,
}

impl From<DeArrowResponse> for DeArrowData {
    fn from(resp: DeArrowResponse) -> Self {
        Self {
            titles: resp.titles,
            thumbnails: resp.thumbnails,
            video_duration: resp.video_duration,
            random_time: resp.random_time,
        }
    }
}

/// Builds the DeArrow branding URL for a video.
fn build_dearrow_url(base_url: &str, video_id: &str) -> String {
    let hash = hash_video_id(video_id);
    format!("{base_url}/api/branding/{hash}")
}

/// Fetches DeArrow branding data (titles, thumbnails) for a video.
///
/// Returns `None` if no branding exists (HTTP 404).
pub async fn get_dearrow_data(
    http_client: &SharedHttpClient,
    video_id: &str,
) -> Result<Option<DeArrowData>, String> {
    let url = build_dearrow_url(crate::community::sponsorblock::SPONSORBLOCK_URL, video_id);
    tracing::debug!("DeArrow get_dearrow_data: {}", url);

    let response = http_client.client().get(&url).send().await;

    match response {
        Ok(resp) => {
            let status = resp.status();
            if status.as_u16() == 404 {
                return Ok(None);
            }
            if !status.is_success() {
                let body = resp.text().await.unwrap_or_default();
                return Err(format!("DeArrow returned {status}: {body}"));
            }
            let data: DeArrowResponse = resp
                .json()
                .await
                .map_err(|e| format!("Failed to parse DeArrow response: {e}"))?;
            Ok(Some(data.into()))
        }
        Err(e) => Err(format!("DeArrow request failed: {e}")),
    }
}

/// Fetches the DeArrow thumbnail URL for a video at a given timestamp.
///
/// Follows redirects to get the final thumbnail URL.
/// Returns `None` if no custom thumbnail (HTTP 204).
pub async fn get_thumbnail(
    _http_client: &SharedHttpClient,
    video_id: &str,
    timestamp: f64,
) -> Result<Option<String>, String> {
    let url = format!(
        "{}/api/v1/getThumbnail?videoID={}&time={}",
        DEARROW_THUMBNAIL_URL, video_id, timestamp
    );
    tracing::debug!("DeArrow get_thumbnail: {}", url);

    // Use a client that follows redirects to capture the final URL.
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::limited(5))
        .build()
        .map_err(|e| format!("Failed to build thumbnail client: {e}"))?;

    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("DeArrow thumbnail request failed: {e}"))?;

    let status = resp.status();
    if status.as_u16() == 204 {
        // No custom thumbnail
        return Ok(None);
    }
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("DeArrow thumbnail returned {status}: {body}"));
    }

    // Return the final URL after redirects
    Ok(Some(resp.url().to_string()))
}

/// Selects the best DeArrow title from the list.
///
/// Returns the first title if it's locked or has non-negative votes.
pub fn select_title(titles: &[DeArrowTitle]) -> Option<&DeArrowTitle> {
    titles.first().filter(|t| t.locked || t.votes >= 0)
}

/// Selects the best DeArrow thumbnail timestamp from the list.
///
/// Returns the first thumbnail's timestamp if locked or votes >= 0,
/// otherwise falls back to `video_duration * random_time`.
pub fn select_thumbnail_timestamp(
    thumbnails: &[DeArrowThumbnail],
    video_duration: Option<f64>,
    random_time: f64,
) -> Option<f64> {
    if let Some(thumb) = thumbnails.first() {
        if thumb.locked || thumb.votes >= 0 {
            return Some(thumb.timestamp);
        }
    }
    // Fallback: use random time within the video
    video_duration.map(|d| d * random_time)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_dearrow_url() {
        let url = build_dearrow_url("https://sponsor.ajay.app", "dQw4w9WgXcQ");
        assert_eq!(url, "https://sponsor.ajay.app/api/branding/5f6b");
    }

    #[test]
    fn test_select_title_locked() {
        let titles = vec![
            DeArrowTitle {
                title: "Locked Title".to_string(),
                locked: true,
                votes: -5,
            },
            DeArrowTitle {
                title: "Other".to_string(),
                locked: false,
                votes: 10,
            },
        ];
        let selected = select_title(&titles);
        assert!(selected.is_some());
        assert_eq!(selected.unwrap().title, "Locked Title");
    }

    #[test]
    fn test_select_title_positive_votes() {
        let titles = vec![DeArrowTitle {
            title: "Good Title".to_string(),
            locked: false,
            votes: 3,
        }];
        let selected = select_title(&titles);
        assert_eq!(selected.unwrap().title, "Good Title");
    }

    #[test]
    fn test_select_title_negative_votes_not_locked() {
        let titles = vec![DeArrowTitle {
            title: "Bad Title".to_string(),
            locked: false,
            votes: -2,
        }];
        let selected = select_title(&titles);
        assert!(selected.is_none());
    }

    #[test]
    fn test_select_title_empty() {
        let titles: Vec<DeArrowTitle> = vec![];
        assert!(select_title(&titles).is_none());
    }

    #[test]
    fn test_select_thumbnail_timestamp_locked() {
        let thumbs = vec![DeArrowThumbnail {
            timestamp: 42.5,
            locked: true,
            votes: -10,
        }];
        let ts = select_thumbnail_timestamp(&thumbs, Some(100.0), 0.5);
        assert_eq!(ts, Some(42.5));
    }

    #[test]
    fn test_select_thumbnail_timestamp_positive_votes() {
        let thumbs = vec![DeArrowThumbnail {
            timestamp: 30.0,
            locked: false,
            votes: 5,
        }];
        let ts = select_thumbnail_timestamp(&thumbs, Some(100.0), 0.5);
        assert_eq!(ts, Some(30.0));
    }

    #[test]
    fn test_select_thumbnail_timestamp_fallback() {
        let thumbs = vec![DeArrowThumbnail {
            timestamp: 30.0,
            locked: false,
            votes: -5,
        }];
        let ts = select_thumbnail_timestamp(&thumbs, Some(100.0), 0.5);
        assert_eq!(ts, Some(50.0)); // 100.0 * 0.5
    }

    #[test]
    fn test_select_thumbnail_timestamp_no_video_duration() {
        let thumbs = vec![DeArrowThumbnail {
            timestamp: 30.0,
            locked: false,
            votes: -5,
        }];
        let ts = select_thumbnail_timestamp(&thumbs, None, 0.5);
        assert!(ts.is_none());
    }

    #[test]
    fn test_select_thumbnail_timestamp_empty() {
        let thumbs: Vec<DeArrowThumbnail> = vec![];
        let ts = select_thumbnail_timestamp(&thumbs, Some(100.0), 0.25);
        assert_eq!(ts, Some(25.0));
    }

    #[test]
    fn test_select_thumbnail_timestamp_empty_no_duration() {
        let thumbs: Vec<DeArrowThumbnail> = vec![];
        let ts = select_thumbnail_timestamp(&thumbs, None, 0.25);
        assert!(ts.is_none());
    }

    #[test]
    fn test_dearrow_response_into() {
        let resp = DeArrowResponse {
            titles: vec![DeArrowTitle {
                title: "Test".to_string(),
                locked: true,
                votes: 5,
            }],
            thumbnails: vec![DeArrowThumbnail {
                timestamp: 10.0,
                locked: false,
                votes: 2,
            }],
            video_duration: Some(120.0),
            random_time: 0.5,
        };
        let data: DeArrowData = resp.into();
        assert_eq!(data.titles.len(), 1);
        assert_eq!(data.thumbnails.len(), 1);
        assert_eq!(data.video_duration, Some(120.0));
        assert_eq!(data.random_time, 0.5);
    }
}

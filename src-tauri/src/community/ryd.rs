use serde::Deserialize;

use crate::http_client::SharedHttpClient;

/// Public Return YouTube Dislike API instances.
pub const RYD_INSTANCES: &[&str] = &[
    "https://ryd-proxy.kavin.rocks",
    "https://returnyoutubedislikeapi.com",
];

/// Default RYD API base URL.
pub const RYD_DEFAULT_URL: &str = "https://ryd-proxy.kavin.rocks";

/// Response from the RYD API.
#[derive(Debug, Deserialize)]
struct RydResponse {
    #[serde(default)]
    dislikes: i64,
}

/// Determines if the given base URL is the legacy `returnyoutubedislikeapi.com` host.
///
/// The legacy host uses a different URL scheme: `/Votes?videoId={id}`
/// All other instances use: `/votes/{id}`
fn is_legacy_host(base_url: &str) -> bool {
    // Extract hostname
    let after_scheme = base_url.splitn(2, "://").nth(1).unwrap_or(base_url);
    let host = after_scheme
        .find(|c| c == '/' || c == '?' || c == '#')
        .map(|i| &after_scheme[..i])
        .unwrap_or(after_scheme);
    // Strip port
    let host = host.split(':').next().unwrap_or(host);
    host == "returnyoutubedislikeapi.com"
}

/// Builds the RYD API URL for a video.
///
/// - Legacy host (`returnyoutubedislikeapi.com`): `{base}/Votes?videoId={id}`
/// - All other instances: `{base}/votes/{id}`
fn build_ryd_url(base_url: &str, video_id: &str) -> String {
    if is_legacy_host(base_url) {
        format!("{base_url}/Votes?videoId={video_id}")
    } else {
        format!("{base_url}/votes/{video_id}")
    }
}

/// Fetches the dislike count for a video from the RYD API.
///
/// Returns 0 if the response contains NaN or a negative value.
pub async fn get_dislikes(
    http_client: &SharedHttpClient,
    video_id: &str,
    base_url: Option<&str>,
) -> Result<u64, String> {
    let base = base_url.unwrap_or(RYD_DEFAULT_URL);
    let url = build_ryd_url(base, video_id);
    tracing::debug!("RYD get_dislikes: {}", url);

    let response = http_client
        .get_json(&url)
        .await
        .map_err(|e| format!("RYD request failed: {e}"))?;

    // Parse the response - handle both {dislikes: number} and direct number
    let dislikes = response
        .get("dislikes")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);

    // NaN → 0 fallback (NaN would not parse as i64, defaults to 0)
    // Also clamp negative values to 0
    Ok(dislikes.max(0) as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_legacy_host_yes() {
        assert!(is_legacy_host("https://returnyoutubedislikeapi.com"));
        assert!(is_legacy_host("https://returnyoutubedislikeapp.com") == false);
    }

    #[test]
    fn test_is_legacy_host_no() {
        assert!(!is_legacy_host("https://ryd-proxy.kavin.rocks"));
        assert!(!is_legacy_host("https://example.com"));
        assert!(!is_legacy_host("https://www.returnyoutubedislikeapi.com"));
    }

    #[test]
    fn test_is_legacy_host_with_path() {
        assert!(is_legacy_host("https://returnyoutubedislikeapi.com/Votes"));
    }

    #[test]
    fn test_is_legacy_host_with_port() {
        assert!(is_legacy_host("https://returnyoutubedislikeapi.com:8080"));
        assert!(!is_legacy_host("https://ryd-proxy.kavin.rocks:8080"));
    }

    #[test]
    fn test_build_ryd_url_modern() {
        let url = build_ryd_url("https://ryd-proxy.kavin.rocks", "dQw4w9WgXcQ");
        assert_eq!(url, "https://ryd-proxy.kavin.rocks/votes/dQw4w9WgXcQ");
    }

    #[test]
    fn test_build_ryd_url_legacy() {
        let url = build_ryd_url("https://returnyoutubedislikeapi.com", "dQw4w9WgXcQ");
        assert_eq!(url, "https://returnyoutubedislikeapi.com/Votes?videoId=dQw4w9WgXcQ");
    }

    #[test]
    fn test_build_ryd_url_custom_instance() {
        let url = build_ryd_url("https://my-ryd.example.com", "abc123");
        assert_eq!(url, "https://my-ryd.example.com/votes/abc123");
    }

    #[test]
    fn test_ryd_instances_list() {
        // Ensure the default URL is in the list
        assert!(RYD_INSTANCES.contains(&RYD_DEFAULT_URL));
        assert_eq!(RYD_INSTANCES.len(), 2);
    }

    #[test]
    fn test_ryd_response_parse() {
        let json = serde_json::json!({ "dislikes": 42 });
        let parsed: RydResponse = serde_json::from_value(json).unwrap();
        assert_eq!(parsed.dislikes, 42);
    }

    #[test]
    fn test_ryd_response_parse_zero() {
        let json = serde_json::json!({ "dislikes": 0 });
        let parsed: RydResponse = serde_json::from_value(json).unwrap();
        assert_eq!(parsed.dislikes, 0);
    }

    #[test]
    fn test_ryd_response_parse_missing_field() {
        let json = serde_json::json!({});
        let parsed: RydResponse = serde_json::from_value(json).unwrap();
        assert_eq!(parsed.dislikes, 0);
    }
}

use tauri::{AppHandle, Emitter};

/// Handle `opentubex://` deep links.
///
/// Supported URL patterns:
/// - `opentubex://watch?v=VIDEO_ID`
/// - `opentubex://channel/CHANNEL_ID`
/// - `opentubex://playlist/PLAYLIST_ID`
/// - `opentubex://search?q=QUERY`
pub fn handle_deep_link(app: &AppHandle, url: String) {
    tracing::info!("Received deep link: {}", url);

    // Parse the URL and emit a structured event
    if let Some(parsed) = parse_deep_link(&url) {
        let _ = app.emit("deep-link", parsed);
    } else {
        tracing::warn!("Failed to parse deep link: {}", url);
        let _ = app.emit("deep-link-raw", url);
    }
}

/// Parse a deep link URL into a structured event payload.
fn parse_deep_link(url: &str) -> Option<DeepLinkEvent> {
    let url = url.strip_prefix("opentubex://")?;

    let parts: Vec<&str> = url.splitn(2, '?').collect();
    let path = parts[0];
    let query = parts.get(1).copied().unwrap_or("");

    if path == "watch" {
        let video_id = query
            .split('&')
            .find_map(|pair| {
                let mut kv = pair.splitn(2, '=');
                if kv.next()? == "v" {
                    kv.next()
                } else {
                    None
                }
            })
            .map(String::from)?;

        Some(DeepLinkEvent::Watch { video_id })
    } else if let Some(channel_id) = path.strip_prefix("channel/") {
        Some(DeepLinkEvent::Channel {
            channel_id: channel_id.to_string(),
        })
    } else if let Some(playlist_id) = path.strip_prefix("playlist/") {
        Some(DeepLinkEvent::Playlist {
            playlist_id: playlist_id.to_string(),
        })
    } else if path == "search" {
        let search_query = query
            .split('&')
            .find_map(|pair| {
                let mut kv = pair.splitn(2, '=');
                if kv.next()? == "q" {
                    kv.next()
                } else {
                    None
                }
            })
            .map(String::from)?;

        Some(DeepLinkEvent::Search {
            query: search_query,
        })
    } else {
        None
    }
}

/// Structured deep link event payload.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DeepLinkEvent {
    Watch { video_id: String },
    Channel { channel_id: String },
    Playlist { playlist_id: String },
    Search { query: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_watch_url() {
        let result = parse_deep_link("opentubex://watch?v=abc123");
        assert!(matches!(
            result,
            Some(DeepLinkEvent::Watch { video_id }) if video_id == "abc123"
        ));
    }

    #[test]
    fn test_parse_channel_url() {
        let result = parse_deep_link("opentubex://channel/UC123456");
        assert!(matches!(
            result,
            Some(DeepLinkEvent::Channel { channel_id }) if channel_id == "UC123456"
        ));
    }

    #[test]
    fn test_parse_playlist_url() {
        let result = parse_deep_link("opentubex://playlist/PL123456");
        assert!(matches!(
            result,
            Some(DeepLinkEvent::Playlist { playlist_id }) if playlist_id == "PL123456"
        ));
    }

    #[test]
    fn test_parse_search_url() {
        let result = parse_deep_link("opentubex://search?q=rust+tauri");
        assert!(matches!(
            result,
            Some(DeepLinkEvent::Search { query }) if query == "rust+tauri"
        ));
    }

    #[test]
    fn test_parse_invalid_url() {
        let result = parse_deep_link("https://example.com");
        assert!(result.is_none());
    }
}

use tauri::State;

use crate::http_client::SharedHttpClient;

const YOUTUBE_PLAYER_URL: &str = "https://www.youtube.com/youtubei/v1/player";
const YOUTUBE_SEARCH_URL: &str = "https://www.youtube.com/youtubei/v1/search";
const YOUTUBE_BROWSE_URL: &str = "https://www.youtube.com/youtubei/v1/browse";
const YOUTUBE_NEXT_URL: &str = "https://www.youtube.com/youtubei/v1/next";
const YOUTUBE_GUIDE_URL: &str = "https://www.youtube.com/youtubei/v1/guide";

fn build_web_context() -> serde_json::Value {
    serde_json::json!({
        "client": {
            "clientName": "WEB",
            "clientVersion": "2.20240101.01.00",
            "hl": "en",
            "gl": "US",
            "userAgent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
            "browserName": "Chrome",
            "browserVersion": "120.0.0.0",
            "osName": "Windows",
            "osVersion": "10.0",
            "platform": "DESKTOP",
            "clientFormFactor": "UNKNOWN_FORM_FACTOR",
        }
    })
}

/// Build a WEB context with PoToken support for playback requests.
fn build_web_context_with_potoken(potoken: Option<&str>) -> serde_json::Value {
    match potoken {
        Some(token) => serde_json::json!({
            "client": {
                "clientName": "WEB",
                "clientVersion": "2.20240101.01.00",
                "hl": "en",
                "gl": "US",
                "userAgent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
                "browserName": "Chrome",
                "browserVersion": "120.0.0.0",
                "osName": "Windows",
                "osVersion": "10.0",
                "platform": "DESKTOP",
                "clientFormFactor": "UNKNOWN_FORM_FACTOR",
            },
            "serviceIntegrityDimensions": {
                "poToken": token
            }
        }),
        None => build_web_context(),
    }
}

#[tauri::command]
pub async fn get_video_info(
    http_client: State<'_, SharedHttpClient>,
    video_id: String,
    potoken: Option<String>,
) -> Result<serde_json::Value, String> {
    let context = build_web_context_with_potoken(potoken.as_deref());
    let body = serde_json::json!({
        "context": context,
        "videoId": video_id,
        "playbackContext": {
            "contentPlaybackContext": {
                "html5Preference": "HTML5_PREF_WANTS",
                "signatureTimestamp": 19171
            }
        },
        "racyCheckOk": true,
        "contentCheckOk": true,
    });

    http_client.post_json(YOUTUBE_PLAYER_URL, &body).await
}

#[tauri::command]
pub async fn search_videos(
    http_client: State<'_, SharedHttpClient>,
    query: String,
    filters: Option<serde_json::Value>,
    continuation: Option<String>,
) -> Result<serde_json::Value, String> {
    let body = if let Some(token) = continuation {
        // Continuation request
        serde_json::json!({
            "context": build_web_context(),
            "continuation": token,
        })
    } else {
        // New search request
        let mut query_obj = serde_json::json!({
            "context": build_web_context(),
            "query": query,
        });

        // Add optional filters
        if let Some(f) = filters {
            if let Some(obj) = query_obj.as_object_mut() {
                if let Some(params) = f.get("params").and_then(|v| v.as_str()) {
                    obj.insert("params".to_string(), serde_json::json!(params));
                }
            }
        }
        query_obj
    };

    http_client.post_json(YOUTUBE_SEARCH_URL, &body).await
}

#[tauri::command]
pub async fn get_trending(
    http_client: State<'_, SharedHttpClient>,
    category: Option<String>,
) -> Result<serde_json::Value, String> {
    let (browse_id, params) = match category.as_deref() {
        Some("gaming") => (
            "UCOpNcN46UbXVtpKMrmU4Abg",
            Some("Egh0cmVuZGluZ7gBAJIDAPIGBAoCMgA"),
        ),
        Some("music") => (
            "FEtrending",
            Some("4gINGgt5dG1diixub3BfbGVhc3VyZRAB"), // Music tab params
        ),
        Some("movies") => (
            "FEtrending",
            Some("4gIKGgh0cmFpbGVycxAB"), // Movies tab params
        ),
        _ => ("FEtrending", None),
    };

    let mut body = serde_json::json!({
        "context": build_web_context(),
        "browseId": browse_id,
    });

    if let Some(p) = params {
        if let Some(obj) = body.as_object_mut() {
            obj.insert("params".to_string(), serde_json::json!(p));
        }
    }

    http_client.post_json(YOUTUBE_BROWSE_URL, &body).await
}

#[tauri::command]
pub async fn get_channel_info(
    http_client: State<'_, SharedHttpClient>,
    channel_id: String,
    tab: Option<String>,
) -> Result<serde_json::Value, String> {
    let (params, browse_id) = match tab.as_deref() {
        Some("videos") => (Some("EgZ2aWRlb3PyBgQKAjoA"), channel_id.clone()),
        Some("shorts") => (Some("EgZzaG9ydHPyBgQKAjoA"), channel_id.clone()),
        Some("live") => (Some("EgdzdHJlYW1z8gYECgJ6AA=="), channel_id.clone()),
        Some("community") => (Some("EgVwb3N0c_IGBAoCSgA="), channel_id.clone()),
        Some("playlists") => (Some("EglwbGF5bGlzdHPyBgQKAkIA"), channel_id.clone()),
        _ => (None, channel_id.clone()),
    };

    let mut body = serde_json::json!({
        "context": build_web_context(),
        "browseId": browse_id,
    });

    if let Some(p) = params {
        if let Some(obj) = body.as_object_mut() {
            obj.insert("params".to_string(), serde_json::json!(p));
        }
    }

    http_client.post_json(YOUTUBE_BROWSE_URL, &body).await
}

#[tauri::command]
pub async fn get_channel_videos(
    http_client: State<'_, SharedHttpClient>,
    channel_id: String,
    continuation: Option<String>,
) -> Result<serde_json::Value, String> {
    let body = if let Some(token) = continuation {
        serde_json::json!({
            "context": build_web_context(),
            "continuation": token,
        })
    } else {
        serde_json::json!({
            "context": build_web_context(),
            "browseId": channel_id,
            "params": "EgZ2aWRlb3PyBgQKAjoA",
        })
    };

    http_client.post_json(YOUTUBE_BROWSE_URL, &body).await
}

#[tauri::command]
pub async fn get_comments(
    http_client: State<'_, SharedHttpClient>,
    video_id: String,
    continuation: Option<String>,
) -> Result<serde_json::Value, String> {
    let body = if let Some(token) = continuation {
        serde_json::json!({
            "context": build_web_context(),
            "continuation": token,
        })
    } else {
        serde_json::json!({
            "context": build_web_context(),
            "videoId": video_id,
        })
    };

    http_client.post_json(YOUTUBE_NEXT_URL, &body).await
}

#[tauri::command]
pub async fn get_search_suggestions(
    http_client: State<'_, SharedHttpClient>,
    query: String,
) -> Result<serde_json::Value, String> {
    let url = format!(
        "https://suggestqueries-clients6.youtube.com/complete/search?client=youtube&ds=yt&q={}",
        urlencoding::encode(&query)
    );

    // Route through the hardened client — YouTube image/CDN domains get
    // automatic Referer/Origin headers via request_internal.
    let text = http_client.get_text(&url).await?;

    // Response format is JSON array wrapped in a callback: `window.google.ach(..., [[...]])`
    // Extract the JSON array from the response
    let json_start = text.find('[').ok_or("Invalid suggestions response format")?;
    let json_end = text.rfind(']').ok_or("Invalid suggestions response format")?;
    let json_str = &text[json_start..=json_end];

    serde_json::from_str(json_str)
        .map_err(|e| format!("Failed to parse suggestions JSON: {}", e))
}

#[tauri::command]
pub async fn get_playlist_info(
    http_client: State<'_, SharedHttpClient>,
    playlist_id: String,
    continuation: Option<String>,
) -> Result<serde_json::Value, String> {
    let body = if let Some(token) = continuation {
        serde_json::json!({
            "context": build_web_context(),
            "continuation": token,
        })
    } else {
        serde_json::json!({
            "context": build_web_context(),
            "browseId": format!("VL{}", playlist_id),
        })
    };

    http_client.post_json(YOUTUBE_BROWSE_URL, &body).await
}

#[tauri::command]
pub async fn get_community_posts(
    http_client: State<'_, SharedHttpClient>,
    channel_id: String,
    continuation: Option<String>,
) -> Result<serde_json::Value, String> {
    let body = if let Some(token) = continuation {
        serde_json::json!({
            "context": build_web_context(),
            "continuation": token,
        })
    } else {
        serde_json::json!({
            "context": build_web_context(),
            "browseId": channel_id,
            "params": "EgVwb3N0c_IGBAoCSgA=",
        })
    };

    http_client.post_json(YOUTUBE_BROWSE_URL, &body).await
}

#[tauri::command]
pub async fn get_hashtag(
    http_client: State<'_, SharedHttpClient>,
    hashtag: String,
) -> Result<serde_json::Value, String> {
    let clean_tag = hashtag.trim_start_matches('#').to_string();
    let body = serde_json::json!({
        "context": build_web_context(),
        "browseId": format!("FEhashtag_{}", clean_tag),
    });

    http_client.post_json(YOUTUBE_BROWSE_URL, &body).await
}

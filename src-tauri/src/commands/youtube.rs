use tauri::State;

use crate::http_client::SharedHttpClient;

const YOUTUBE_PLAYER_URL: &str = "https://www.youtube.com/youtubei/v1/player";
const YOUTUBE_SEARCH_URL: &str = "https://www.youtube.com/youtubei/v1/search";
const YOUTUBE_BROWSE_URL: &str = "https://www.youtube.com/youtubei/v1/browse";
const YOUTUBE_NEXT_URL: &str = "https://www.youtube.com/youtubei/v1/next";

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

#[tauri::command]
pub async fn get_video_info(
    http_client: State<'_, SharedHttpClient>,
    video_id: String,
) -> Result<serde_json::Value, String> {
    let body = serde_json::json!({
        "context": build_web_context(),
        "videoId": video_id,
        "playbackContext": {
            "contentPlaybackContext": {
                "html5Preference": "HTML5_PREF_WANTS"
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
) -> Result<serde_json::Value, String> {
    let body = serde_json::json!({
        "context": build_web_context(),
        "query": query,
    });

    http_client.post_json(YOUTUBE_SEARCH_URL, &body).await
}

#[tauri::command]
pub async fn get_trending(
    http_client: State<'_, SharedHttpClient>,
) -> Result<serde_json::Value, String> {
    let body = serde_json::json!({
        "context": build_web_context(),
        "browseId": "FEtrending",
    });

    http_client.post_json(YOUTUBE_BROWSE_URL, &body).await
}

#[tauri::command]
pub async fn get_channel_info(
    http_client: State<'_, SharedHttpClient>,
    channel_id: String,
) -> Result<serde_json::Value, String> {
    let body = serde_json::json!({
        "context": build_web_context(),
        "browseId": channel_id,
    });

    http_client.post_json(YOUTUBE_BROWSE_URL, &body).await
}

#[tauri::command]
pub async fn get_channel_videos(
    http_client: State<'_, SharedHttpClient>,
    channel_id: String,
) -> Result<serde_json::Value, String> {
    let body = serde_json::json!({
        "context": build_web_context(),
        "browseId": channel_id,
        "params": "EgZ2aWRlb3PyBgQKAkIA",
    });

    http_client.post_json(YOUTUBE_BROWSE_URL, &body).await
}

#[tauri::command]
pub async fn get_comments(
    http_client: State<'_, SharedHttpClient>,
    video_id: String,
) -> Result<serde_json::Value, String> {
    let body = serde_json::json!({
        "context": build_web_context(),
        "videoId": video_id,
    });

    http_client.post_json(YOUTUBE_NEXT_URL, &body).await
}

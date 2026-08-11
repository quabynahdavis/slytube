use tauri::State;

use crate::http_client::SharedHttpClient;

const INSTANCES_API: &str = "https://api.invidious.io/instances.json";

const FALLBACK_INSTANCES: &[&str] = &[
    "https://inv.nadeko.net",
    "https://invidious.nerdvpn.de",
    "https://yewtu.be",
    "https://invidious.private.coffee",
    "https://invidious.jing.rocks",
];

async fn try_instances<F>(http_client: &SharedHttpClient, build_url: F) -> Result<serde_json::Value, String>
where
    F: Fn(&str) -> String,
{
    let mut last_error = String::new();

    for instance in FALLBACK_INSTANCES {
        let url = build_url(instance);
        match http_client.get_json(&url).await {
            Ok(result) => return Ok(result),
            Err(e) => {
                tracing::warn!("Instance {} failed: {}", instance, e);
                last_error = e;
                continue;
            }
        }
    }

    Err(format!("All Invidious instances failed. Last error: {}", last_error))
}

fn build_api_url(instance: &str, resource: &str, id: &str, params: &[(&str, &str)]) -> String {
    let base = if id.is_empty() {
        format!("{}/api/v1/{}", instance, resource)
    } else {
        format!("{}/api/v1/{}/{}", instance, resource, id)
    };

    if params.is_empty() {
        base
    } else {
        let query: String = params
            .iter()
            .map(|(k, v)| format!("{}={}", k, urlencoding::encode(v)))
            .collect::<Vec<_>>()
            .join("&");
        format!("{}?{}", base, query)
    }
}

#[tauri::command]
pub async fn invidious_get_video(
    http_client: State<'_, SharedHttpClient>,
    video_id: String,
) -> Result<serde_json::Value, String> {
    try_instances(&http_client, |instance| {
        build_api_url(instance, "videos", &video_id, &[])
    }).await
}

#[tauri::command]
pub async fn invidious_search(
    http_client: State<'_, SharedHttpClient>,
    query: String,
) -> Result<serde_json::Value, String> {
    try_instances(&http_client, |instance| {
        build_api_url(instance, "search", "", &[
            ("q", &query),
            ("page", "1"),
        ])
    }).await
}

#[tauri::command]
pub async fn invidious_get_trending(
    http_client: State<'_, SharedHttpClient>,
) -> Result<serde_json::Value, String> {
    try_instances(&http_client, |instance| {
        build_api_url(instance, "trending", "", &[
            ("type", "news"),
        ])
    }).await
}

#[tauri::command]
pub async fn invidious_get_channel(
    http_client: State<'_, SharedHttpClient>,
    channel_id: String,
) -> Result<serde_json::Value, String> {
    try_instances(&http_client, |instance| {
        build_api_url(instance, "channels", &channel_id, &[])
    }).await
}

#[tauri::command]
pub async fn invidious_get_playlist(
    http_client: State<'_, SharedHttpClient>,
    playlist_id: String,
) -> Result<serde_json::Value, String> {
    try_instances(&http_client, |instance| {
        build_api_url(instance, "playlists", &playlist_id, &[])
    }).await
}

#[tauri::command]
pub async fn invidious_get_comments(
    http_client: State<'_, SharedHttpClient>,
    video_id: String,
) -> Result<serde_json::Value, String> {
    try_instances(&http_client, |instance| {
        build_api_url(instance, "comments", &video_id, &[])
    }).await
}

#[tauri::command]
pub async fn invidious_get_instances(
    http_client: State<'_, SharedHttpClient>,
) -> Result<serde_json::Value, String> {
    http_client.get_json(INSTANCES_API).await
}

#[tauri::command]
pub async fn invidious_test_instance(
    http_client: State<'_, SharedHttpClient>,
    instance_url: String,
) -> Result<bool, String> {
    let url = format!("{}/api/v1/stats", instance_url);
    match http_client.get_json(&url).await {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

#[tauri::command]
pub async fn invidious_get_dash_manifest(
    http_client: State<'_, SharedHttpClient>,
    video_id: String,
) -> Result<String, String> {
    let mut last_error = String::new();

    for instance in FALLBACK_INSTANCES {
        let url = format!("{}/api/manifest/dash/id/{}?local=true", instance, video_id);

        let response = http_client
            .client()
            .get(&url)
            .send()
            .await;

        match response {
            Ok(resp) if resp.status().is_success() => {
                match resp.text().await {
                    Ok(text) => return Ok(text),
                    Err(e) => {
                        last_error = format!("Failed to read DASH manifest: {}", e);
                        continue;
                    }
                }
            }
            Ok(resp) => {
                last_error = format!("DASH manifest request failed with status: {}", resp.status());
                continue;
            }
            Err(e) => {
                last_error = format!("Request failed: {}", e);
                continue;
            }
        }
    }

    Err(format!("All instances failed: {}", last_error))
}

#[tauri::command]
pub async fn invidious_get_popular(
    http_client: State<'_, SharedHttpClient>,
) -> Result<serde_json::Value, String> {
    try_instances(&http_client, |instance| {
        build_api_url(instance, "popular", "", &[])
    }).await
}

#[tauri::command]
pub async fn invidious_get_channel_videos(
    http_client: State<'_, SharedHttpClient>,
    channel_id: String,
) -> Result<serde_json::Value, String> {
    try_instances(&http_client, |instance| {
        build_api_url(instance, "channels", &channel_id, &[
            ("videos", ""),
        ])
    }).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_api_url_without_params() {
        let url = build_api_url("https://example.com", "videos", "abc123", &[]);
        assert_eq!(url, "https://example.com/api/v1/videos/abc123");
    }

    #[test]
    fn test_build_api_url_with_params() {
        let url = build_api_url("https://example.com", "search", "", &[
            ("q", "hello world"),
            ("page", "1"),
        ]);
        assert_eq!(url, "https://example.com/api/v1/search?q=hello%20world&page=1");
    }

    #[test]
    fn test_build_api_url_with_empty_id() {
        let url = build_api_url("https://example.com", "trending", "", &[
            ("type", "news"),
        ]);
        assert_eq!(url, "https://example.com/api/v1/trending?type=news");
    }

    #[test]
    fn test_fallback_instances_not_empty() {
        assert!(!FALLBACK_INSTANCES.is_empty());
        for instance in FALLBACK_INSTANCES {
            assert!(instance.starts_with("https://"));
        }
    }
}

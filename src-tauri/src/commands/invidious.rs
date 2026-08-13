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
pub async fn invidious_get_dash_url(
    http_client: State<'_, SharedHttpClient>,
    video_id: String,
) -> Result<String, String> {
    let video_info = try_instances(&http_client, |instance| {
        build_api_url(instance, "videos", &video_id, &[])
    }).await?;

    if let Some(dash_url) = video_info.get("dashUrl").and_then(|v| v.as_str()) {
        if !dash_url.is_empty() {
            return Ok(dash_url.to_string());
        }
    }

    Err(format!("No DASH URL available for video {}", video_id))
}

#[tauri::command]
pub async fn invidious_get_format_streams(
    http_client: State<'_, SharedHttpClient>,
    video_id: String,
) -> Result<Vec<serde_json::Value>, String> {
    let video_info = try_instances(&http_client, |instance| {
        build_api_url(instance, "videos", &video_id, &[])
    }).await?;

    if let Some(formats) = video_info.get("formatStreams").and_then(|v| v.as_array()) {
        return Ok(formats.clone());
    }

    Err(format!("No format streams available for video {}", video_id))
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

        // Route through the hardened client to apply Invidious auth headers.
        match http_client.get_text(&url).await {
            Ok(text) => return Ok(text),
            Err(e) => {
                tracing::warn!("DASH manifest from {} failed: {}", instance, e);
                last_error = e;
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

#[tauri::command]
pub async fn invidious_resolve_url(
    http_client: State<'_, SharedHttpClient>,
    url: String,
) -> Result<serde_json::Value, String> {
    try_instances(&http_client, |instance| {
        build_api_url(instance, "resolveurl", "", &[
            ("url", &url),
        ])
    }).await
}

#[tauri::command]
pub async fn invidious_get_channel_tabs(
    http_client: State<'_, SharedHttpClient>,
    channel_id: String,
) -> Result<serde_json::Value, String> {
    try_instances(&http_client, |instance| {
        build_api_url(instance, "channels", &channel_id, &[
            ("tabs", ""),
        ])
    }).await
}

#[tauri::command]
pub async fn invidious_get_channel_shorts(
    http_client: State<'_, SharedHttpClient>,
    channel_id: String,
) -> Result<serde_json::Value, String> {
    try_instances(&http_client, |instance| {
        build_api_url(instance, "channels", &channel_id, &[
            ("shorts", ""),
        ])
    }).await
}

#[tauri::command]
pub async fn invidious_get_channel_live(
    http_client: State<'_, SharedHttpClient>,
    channel_id: String,
) -> Result<serde_json::Value, String> {
    try_instances(&http_client, |instance| {
        build_api_url(instance, "channels", &channel_id, &[
            ("live", ""),
        ])
    }).await
}

#[tauri::command]
pub async fn invidious_get_channel_playlists(
    http_client: State<'_, SharedHttpClient>,
    channel_id: String,
) -> Result<serde_json::Value, String> {
    try_instances(&http_client, |instance| {
        build_api_url(instance, "channels", &channel_id, &[
            ("playlists", ""),
        ])
    }).await
}

#[tauri::command]
pub async fn invidious_get_channel_releases(
    http_client: State<'_, SharedHttpClient>,
    channel_id: String,
) -> Result<serde_json::Value, String> {
    try_instances(&http_client, |instance| {
        build_api_url(instance, "channels", &channel_id, &[
            ("releases", ""),
        ])
    }).await
}

#[tauri::command]
pub async fn invidious_get_channel_podcasts(
    http_client: State<'_, SharedHttpClient>,
    channel_id: String,
) -> Result<serde_json::Value, String> {
    try_instances(&http_client, |instance| {
        build_api_url(instance, "channels", &channel_id, &[
            ("podcasts", ""),
        ])
    }).await
}

#[tauri::command]
pub async fn invidious_get_channel_courses(
    http_client: State<'_, SharedHttpClient>,
    channel_id: String,
) -> Result<serde_json::Value, String> {
    try_instances(&http_client, |instance| {
        build_api_url(instance, "channels", &channel_id, &[
            ("courses", ""),
        ])
    }).await
}

#[tauri::command]
pub async fn invidious_search_channel(
    http_client: State<'_, SharedHttpClient>,
    channel_id: String,
    query: String,
) -> Result<serde_json::Value, String> {
    try_instances(&http_client, |instance| {
        build_api_url(instance, "channels", &channel_id, &[
            ("search", ""),
            ("q", &query),
        ])
    }).await
}

#[tauri::command]
pub async fn invidious_get_comment_replies(
    http_client: State<'_, SharedHttpClient>,
    video_id: String,
    comment_id: String,
) -> Result<serde_json::Value, String> {
    try_instances(&http_client, |instance| {
        build_api_url(instance, "comments", &video_id, &[
            ("replyToken", &comment_id),
        ])
    }).await
}

#[tauri::command]
pub async fn invidious_get_search_suggestions(
    http_client: State<'_, SharedHttpClient>,
    query: String,
) -> Result<serde_json::Value, String> {
    try_instances(&http_client, |instance| {
        build_api_url(instance, "search", "suggestions", &[
            ("q", &query),
        ])
    }).await
}

#[tauri::command]
pub async fn invidious_search_with_filters(
    http_client: State<'_, SharedHttpClient>,
    query: String,
    search_params: Option<serde_json::Value>,
) -> Result<serde_json::Value, String> {
    let mut owned_params: Vec<(String, String)> = vec![
        ("q".to_string(), query),
        ("page".to_string(), "1".to_string()),
    ];

    // Extract optional filter parameters
    if let Some(sp) = search_params {
        if let Some(duration) = sp.get("duration").and_then(|v| v.as_str()) {
            if !duration.is_empty() {
                owned_params.push(("duration".to_string(), duration.to_string()));
            }
        }
        if let Some(sort) = sp.get("sort").and_then(|v| v.as_str()) {
            if !sort.is_empty() {
                owned_params.push(("sort".to_string(), sort.to_string()));
            }
        }
        if let Some(date) = sp.get("date").and_then(|v| v.as_str()) {
            if !date.is_empty() {
                owned_params.push(("date".to_string(), date.to_string()));
            }
        }
        if let Some(typ) = sp.get("type").and_then(|v| v.as_str()) {
            if !typ.is_empty() {
                owned_params.push(("type".to_string(), typ.to_string()));
            }
        }
        if let Some(features) = sp.get("features").and_then(|v| v.as_array()) {
            for feature in features {
                if let Some(f) = feature.as_str() {
                    owned_params.push(("features".to_string(), f.to_string()));
                }
            }
        }
    }

    // Convert to string references for build_api_url
    let static_params: Vec<(&str, &str)> = owned_params.iter()
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .collect();

    try_instances(&http_client, |instance| {
        build_api_url(instance, "search", "", &static_params)
    }).await
}

#[tauri::command]
pub async fn invidious_get_community_posts(
    http_client: State<'_, SharedHttpClient>,
    channel_id: String,
) -> Result<serde_json::Value, String> {
    try_instances(&http_client, |instance| {
        build_api_url(instance, "channels", &channel_id, &[
            ("community", ""),
        ])
    }).await
}

#[tauri::command]
pub async fn invidious_get_community_post(
    http_client: State<'_, SharedHttpClient>,
    channel_id: String,
    post_id: String,
) -> Result<serde_json::Value, String> {
    try_instances(&http_client, |instance| {
        build_api_url(instance, "channels", &channel_id, &[
            ("community", &post_id),
        ])
    }).await
}

#[tauri::command]
pub async fn invidious_get_community_post_comments(
    http_client: State<'_, SharedHttpClient>,
    channel_id: String,
    post_id: String,
) -> Result<serde_json::Value, String> {
    try_instances(&http_client, |instance| {
        build_api_url(instance, "channels", &channel_id, &[
            ("community", &format!("{}?comments=1", post_id)),
        ])
    }).await
}

#[tauri::command]
pub async fn invidious_get_community_post_comment_replies(
    http_client: State<'_, SharedHttpClient>,
    channel_id: String,
    post_id: String,
    comment_id: String,
) -> Result<serde_json::Value, String> {
    try_instances(&http_client, |instance| {
        build_api_url(instance, "channels", &channel_id, &[
            ("community", &format!("{}?comment={}", post_id, comment_id)),
        ])
    }).await
}

#[tauri::command]
pub async fn invidious_get_hashtag(
    http_client: State<'_, SharedHttpClient>,
    hashtag: String,
) -> Result<serde_json::Value, String> {
    let clean_tag = hashtag.trim_start_matches('#').to_string();
    try_instances(&http_client, |instance| {
        build_api_url(instance, "hashtag", &clean_tag, &[
            ("page", "1"),
        ])
    }).await
}

#[tauri::command]
pub async fn invidious_fetch(
    http_client: State<'_, SharedHttpClient>,
    path: String,
) -> Result<serde_json::Value, String> {
    try_instances(&http_client, |instance| {
        format!("{}/{}", instance, path.trim_start_matches('/'))
    }).await
}

#[tauri::command]
pub async fn invidious_get_instances_list(
    http_client: State<'_, SharedHttpClient>,
) -> Result<Vec<serde_json::Value>, String> {
    let response = http_client.get_json(INSTANCES_API).await?;
    let instances = response.as_array().ok_or("Invalid instances response")?;

    let result: Vec<serde_json::Value> = instances
        .iter()
        .filter_map(|instance| {
            let uri = instance.get(0)?.as_str()?;
            let info = instance.get(1)?;
            let api = info.get("api")?.as_bool()?;
            let cors = info.get("cors")?.as_bool()?;
            let instance_type = info.get("type")?.as_str()?;

            if api && cors && instance_type == "https" && !uri.contains(".onion") && !uri.contains(".i2p") {
                Some(serde_json::json!({
                    "url": uri,
                    "name": info.get("name").and_then(|n| n.as_str()).unwrap_or(uri),
                    "health": info.get("health").and_then(|h| h.as_f64()).unwrap_or(0.0) as i64,
                    "cors": cors,
                    "api": api
                }))
            } else {
                None
            }
        })
        .collect();

    if result.is_empty() {
        Err("No suitable instances found".to_string())
    } else {
        Ok(result)
    }
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

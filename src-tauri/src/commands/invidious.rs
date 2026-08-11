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

fn get_instance_url() -> String {
    FALLBACK_INSTANCES[0].to_string()
}

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
                last_error = e;
                continue;
            }
        }
    }

    Err(format!("All Invidious instances failed: {}", last_error))
}

#[tauri::command]
pub async fn invidious_get_video(
    http_client: State<'_, SharedHttpClient>,
    video_id: String,
) -> Result<serde_json::Value, String> {
    try_instances(&http_client, |instance| {
        format!("{}/api/v1/videos/{}", instance, video_id)
    }).await
}

#[tauri::command]
pub async fn invidious_search(
    http_client: State<'_, SharedHttpClient>,
    query: String,
) -> Result<serde_json::Value, String> {
    try_instances(&http_client, |instance| {
        format!(
            "{}/api/v1/search?q={}&page=1",
            instance,
            urlencoding::encode(&query)
        )
    }).await
}

#[tauri::command]
pub async fn invidious_get_trending(
    http_client: State<'_, SharedHttpClient>,
) -> Result<serde_json::Value, String> {
    try_instances(&http_client, |instance| {
        format!("{}/api/v1/trending?type=news", instance)
    }).await
}

#[tauri::command]
pub async fn invidious_get_channel(
    http_client: State<'_, SharedHttpClient>,
    channel_id: String,
) -> Result<serde_json::Value, String> {
    try_instances(&http_client, |instance| {
        format!("{}/api/v1/channels/{}", instance, channel_id)
    }).await
}

#[tauri::command]
pub async fn invidious_get_playlist(
    http_client: State<'_, SharedHttpClient>,
    playlist_id: String,
) -> Result<serde_json::Value, String> {
    try_instances(&http_client, |instance| {
        format!("{}/api/v1/playlists/{}", instance, playlist_id)
    }).await
}

#[tauri::command]
pub async fn invidious_get_comments(
    http_client: State<'_, SharedHttpClient>,
    video_id: String,
) -> Result<serde_json::Value, String> {
    try_instances(&http_client, |instance| {
        format!("{}/api/v1/comments/{}", instance, video_id)
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
    let instances = FALLBACK_INSTANCES;
    let mut last_error = String::new();

    for instance in instances {
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

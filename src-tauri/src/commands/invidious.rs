use tauri::State;

use crate::http_client::SharedHttpClient;

const INSTANCES_API: &str = "https://api.invidious.io/instances.json";

fn get_instance_url() -> String {
    // TODO: Load from settings store
    "https://inv.nadeko.net".to_string()
}

#[tauri::command]
pub async fn invidious_get_video(
    http_client: State<'_, SharedHttpClient>,
    video_id: String,
) -> Result<serde_json::Value, String> {
    let instance = get_instance_url();
    let url = format!("{}/api/v1/videos/{}", instance, video_id);
    http_client.get_json(&url).await
}

#[tauri::command]
pub async fn invidious_search(
    http_client: State<'_, SharedHttpClient>,
    query: String,
) -> Result<serde_json::Value, String> {
    let instance = get_instance_url();
    let url = format!(
        "{}/api/v1/search?q={}&page=1",
        instance,
        urlencoding::encode(&query)
    );
    http_client.get_json(&url).await
}

#[tauri::command]
pub async fn invidious_get_trending(
    http_client: State<'_, SharedHttpClient>,
) -> Result<serde_json::Value, String> {
    let instance = get_instance_url();
    let url = format!("{}/api/v1/trending?type=news", instance);
    tracing::info!("Fetching trending: {}", url);
    http_client.get_json(&url).await
}

#[tauri::command]
pub async fn invidious_get_channel(
    http_client: State<'_, SharedHttpClient>,
    channel_id: String,
) -> Result<serde_json::Value, String> {
    let instance = get_instance_url();
    let url = format!("{}/api/v1/channels/{}", instance, channel_id);
    http_client.get_json(&url).await
}

#[tauri::command]
pub async fn invidious_get_playlist(
    http_client: State<'_, SharedHttpClient>,
    playlist_id: String,
) -> Result<serde_json::Value, String> {
    let instance = get_instance_url();
    let url = format!("{}/api/v1/playlists/{}", instance, playlist_id);
    http_client.get_json(&url).await
}

#[tauri::command]
pub async fn invidious_get_comments(
    http_client: State<'_, SharedHttpClient>,
    video_id: String,
) -> Result<serde_json::Value, String> {
    let instance = get_instance_url();
    let url = format!("{}/api/v1/comments/{}", instance, video_id);
    http_client.get_json(&url).await
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
    let instance = get_instance_url();
    let url = format!("{}/api/manifest/dash/id/{}?local=true", instance, video_id);
    
    let response = http_client
        .client()
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch DASH manifest: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("DASH manifest request failed with status: {}", response.status()));
    }

    let text = response.text().await
        .map_err(|e| format!("Failed to read DASH manifest: {}", e))?;

    Ok(text)
}

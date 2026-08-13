use std::collections::HashMap;
use std::sync::Mutex;

use tauri::{AppHandle, Manager, State};
use tokio::sync::oneshot;

use crate::extractor::ExtractionMethod;

/// Pending request correlation: req_id → response sender.
pub struct PendingExtractions(Mutex<HashMap<String, oneshot::Sender<serde_json::Value>>>);

impl PendingExtractions {
    pub fn new() -> Self {
        Self(Mutex::new(HashMap::new()))
    }
}

/// Extract YouTube data via the hidden youtubei.js webview.
///
/// This command:
/// 1. Generates a unique request ID.
/// 2. Registers a oneshot channel to receive the result.
/// 3. Dispatches the method + params to the extractor hidden webview via `eval()`.
/// 4. Awaits the result (sent back via the `extraction_result` command).
///
/// The extractor webview runs youtubei.js (Innertube) and returns parsed,
/// flat JSON — Rust never touches YouTube's protobuf schemas directly.
#[tauri::command]
pub async fn extract(
    app: AppHandle,
    state: State<'_, PendingExtractions>,
    method: String,
    params: serde_json::Value,
) -> Result<serde_json::Value, String> {
    // Validate method upfront
    let valid_methods = [
        "getVideoInfo",
        "search",
        "getChannel",
        "getChannelVideos",
        "getChannelShorts",
        "getChannelLive",
        "getChannelCommunity",
        "getComments",
        "getCommentReplies",
        "getTrending",
        "getPlaylist",
        "getHashtag",
        "getCommunityPost",
        "getSearchSuggestions",
        "generatePoToken",
    ];
    if !valid_methods.contains(&method.as_str()) {
        return Err(format!("Unknown extraction method: {}", method));
    }

    let req_id = uuid::Uuid::new_v4().to_string();
    let (tx, rx) = oneshot::channel();

    // Register pending request
    {
        let mut pending = state
            .0
            .lock()
            .map_err(|e| format!("Failed to lock pending extractions: {}", e))?;
        pending.insert(req_id.clone(), tx);
    }

    // Get the extractor hidden webview
    let extractor = app
        .get_webview_window("extractor")
        .ok_or("Extractor webview not running. It may not have been initialized.")?;

    // Build the JS call with safe JSON encoding
    let params_json = serde_json::to_string(&params)
        .map_err(|e| format!("Failed to serialize params: {}", e))?;
    let method_json = serde_json::to_string(&method)
        .map_err(|e| format!("Failed to serialize method: {}", e))?;

    let js = format!(
        r#"
        (async () => {{
            try {{
                if (!window.__slytube_run) {{
                    throw new Error('Extractor bridge not initialized — youtubei.js not loaded');
                }}
                await window.__slytube_run({req_id:?}, {method_json}, {params_json});
            }} catch (err) {{
                // Send error back via invoke
                const {{ invoke }} = window.__TAURI_INTERNALS__ ? 
                    window.__TAURI_INTERNALS__ : 
                    (await import('@tauri-apps/api/core'));
                await invoke('extraction_result', {{
                    reqId: {req_id:?},
                    result: {{ error: err?.message || String(err) }}
                }});
            }}
        }})();
        "#,
    );

    extractor
        .eval(&js)
        .map_err(|e| format!("Failed to dispatch extraction to webview: {}", e))?;

    // Await the result (with timeout via try_recv pattern would be better,
    // but oneshot + async gives us natural cancellation)
    match rx.await {
        Ok(result) => {
            // Check if the JS side returned an error
            if let Some(error) = result.get("error").and_then(|e| e.as_str()) {
                Err(error.to_string())
            } else {
                Ok(result)
            }
        }
        Err(_) => {
            // Channel dropped — clean up pending
            let mut pending = state.0.lock().ok();
            if let Some(mut p) = pending {
                p.remove(&req_id);
            }
            Err("Extraction cancelled or extractor webview disconnected".to_string())
        }
    }
}

/// Callback command invoked BY the extractor webview to deliver results.
///
/// The extractor JS calls `invoke('extraction_result', { reqId, result })`
/// when youtubei.js finishes processing. This command correlates the reqId
/// back to the pending oneshot channel.
#[tauri::command]
pub fn extraction_result(
    state: State<'_, PendingExtractions>,
    req_id: String,
    result: serde_json::Value,
) -> Result<(), String> {
    let mut pending = state
        .0
        .lock()
        .map_err(|e| format!("Failed to lock pending extractions: {}", e))?;

    if let Some(tx) = pending.remove(&req_id) {
        let _ = tx.send(result);
        Ok(())
    } else {
        Err(format!("No pending extraction found for req_id: {}", req_id))
    }
}

/// Check whether the extractor webview is ready (youtubei.js loaded).
#[tauri::command]
pub async fn extractor_ready(app: AppHandle) -> Result<bool, String> {
    let extractor = app
        .get_webview_window("extractor")
        .ok_or("Extractor webview not running")?;

    // We'll set a flag from JS when youtubei.js is loaded
    // For now, just check the window exists
    Ok(true)
}

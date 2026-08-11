use std::sync::mpsc;
use std::time::Duration;


use tauri::{AppHandle, Listener, WebviewWindowBuilder};


use crate::potoken::PoTokenState;

/// BotGuard script content bundled with the app at compile time.
const BOTGUARD_SCRIPT: &str = include_str!("../../binaries/botGuardScript.js");

/// Maximum time to wait for PoToken generation before timing out.
const GENERATION_TIMEOUT: Duration = Duration::from_secs(60);

/// Generate a PoToken (Proof of Origin Token) for YouTube playback.
///
/// This command creates a hidden webview window to execute the BotGuard VM
/// challenge and mint a fresh PoToken bound to the given `video_id` and
/// Innertube `context`. The webview is destroyed immediately after the
/// token is produced (or on error/timeout).
///
/// # Arguments
/// * `video_id` - YouTube video identifier (e.g. `dQw4w9WgXcQ`).
/// * `context` - Innertube session context as a JSON string.
/// * `proxy_url` - Optional SOCKS/HTTP proxy URL forwarded to the VM.
#[tauri::command]
pub async fn generate_po_token(
    app_handle: AppHandle,
    video_id: String,
    context: String,
    proxy_url: Option<String>,
    state: tauri::State<'_, PoTokenState>,
) -> Result<String, String> {
    // Increment generation count.
    {
        let mut count = state
            .generation_count
            .lock()
            .map_err(|e| format!("Failed to lock generation_count: {}", e))?;
        *count += 1;
    }

    // Unique label for this generation window so we can route the result event.
    let window_label = format!("potoken-gen-{}", uuid::Uuid::new_v4());

    // Channel used by the JS side (via emit) to send the result back to Rust.
    let (tx, rx) = mpsc::channel::<Result<String, String>>();

    // Build the hidden webview that hosts the BotGuard VM.
    let window = WebviewWindowBuilder::new(
        &app_handle,
        &window_label,
        tauri::WebviewUrl::App("about:blank".into()),
    )
    .title("PoToken Generator")
    .visible(false)
    .inner_size(800.0, 600.0)
    .initialization_script(BOTGUARD_SCRIPT)
    .build()
    .map_err(|e| format!("Failed to create BotGuard webview window: {}", e))?;

    // Wrap sender in Option so we can take() it exactly once.
    let tx = std::sync::Arc::new(std::sync::Mutex::new(Some(tx)));

    // Listen for the one-shot result event emitted by the webview JS.
    // We use a unique event name per generation to avoid cross-talk.
    let result_event = format!("potoken:result:{}", window_label);
    let tx_for_result = tx.clone();

    app_handle.listen_any(&result_event, move |event| {
        let payload = event.payload().to_string();
        let guard = tx_for_result.lock();
        if let Ok(mut opt) = guard {
            if let Some(sender) = opt.take() {
                let _ = sender.send(Ok(payload));
            }
        }
    });

    // Also listen for error events from the JS side.
    let error_event = format!("potoken:error:{}", window_label);
    let tx_for_err = tx.clone();

    app_handle.listen_any(&error_event, move |event| {
        let payload = event.payload().to_string();
        let guard = tx_for_err.lock();
        if let Ok(mut opt) = guard {
            if let Some(sender) = opt.take() {
                let _ = sender.send(Err(payload));
            }
        }
    });

    // Build the JavaScript that drives the BotGuard VM and emits the result.
    let execution_js = build_execution_script(&video_id, &context, &proxy_url, &window_label);

    // Kick off execution inside the hidden webview.
    window
        .eval(&execution_js)
        .map_err(|e| format!("Failed to inject BotGuard execution script: {}", e))?;

    // Block (but still async) until we get a result or hit the timeout.
    match rx.recv_timeout(GENERATION_TIMEOUT) {
        Ok(Ok(token)) => {
            // Success — store last result and clean up webview.
            {
                let mut last = state
                    .last_result
                    .lock()
                    .map_err(|e| format!("Failed to lock last_result: {}", e))?;
                *last = Some(token.clone());
            }
            let _ = window.close();
            Ok(token)
        }
        Ok(Err(js_err)) => {
            // JS-side error reported via potoken:error event.
            let _ = window.close();
            Err(js_err)
        }
        Err(mpsc::RecvTimeoutError::Timeout) => {
            let _ = window.close();
            Err(format!(
                "PoToken generation timed out after {} seconds",
                GENERATION_TIMEOUT.as_secs()
            ))
        }
        Err(mpsc::RecvTimeoutError::Disconnected) => {
            let _ = window.close();
            Err("PoToken generation channel disconnected unexpectedly".to_string())
        }
    }
}

/// Build the JavaScript that runs inside the hidden webview to drive the
/// BotGuard VM challenge and mint a PoToken.
///
/// The bundled `BOTGUARD_SCRIPT` (injected via `initialization_script`)
/// is expected to expose a default export (async function) that takes
/// `(videoId, context)` and returns the minted token string.
fn build_execution_script(
    video_id: &str,
    context: &str,
    proxy_url: &Option<String>,
    window_label: &str,
) -> String {
    // Escape values for safe embedding in JS string literals.
    let video_id_json = serde_json::to_string(video_id).unwrap_or_default();
    let context_value: serde_json::Value =
        serde_json::from_str(context).unwrap_or(serde_json::Value::Null);
    let context_json =
        serde_json::to_string(&context_value).unwrap_or_else(|_| "null".to_string());
    let proxy_json = serde_json::to_string(proxy_url).unwrap_or_else(|_| "null".to_string());
    let result_event_json =
        serde_json::to_string(&format!("potoken:result:{}", window_label)).unwrap_or_default();
    let error_event_json =
        serde_json::to_string(&format!("potoken:error:{}", window_label)).unwrap_or_default();

    format!(
        r#"
(async () => {{
  const videoId = {video_id_json};
  const context = {context_json};
  const proxyUrl = {proxy_json};
  const resultEvent = {result_event_json};
  const errorEvent = {error_event_json};

  try {{
    // Proxy configuration: the webview itself does not natively support
    // per-request proxies. We expose the proxy URL to the BotGuard VM so
    // it can route fetches through it if the VM supports it.
    if (proxyUrl) {{
      globalThis.__proxyUrl = proxyUrl;
      console.info('[PoToken] Proxy configured:', proxyUrl);
    }}

    // The BOTGUARD_SCRIPT sets up `globalThis.__mintPoToken` after
    // bootstrapping. Wait briefly for the VM to finish initializing.
    const waitForVm = (timeoutMs) => new Promise((resolve, reject) => {{
      const start = Date.now();
      const check = () => {{
        if (globalThis.__mintPoToken) return resolve();
        if (Date.now() - start > timeoutMs) return reject(new Error('BotGuard VM bootstrap timeout'));
        setTimeout(check, 100);
      }};
      check();
    }});

    await waitForVm(10_000);

    // Execute the BotGuard challenge and mint the PoToken.
    const token = await globalThis.__mintPoToken(videoId, context);

    if (typeof token !== 'string' || !token.length) {{
      throw new Error('BotGuard VM returned empty or invalid token');
    }}

    // Emit success event back to Rust via Tauri IPC.
    if (window.__TAURI_INTERNALS__?.event?.emit) {{
      window.__TAURI_INTERNALS__.event.emit(resultEvent, token);
    }} else {{
      throw new Error('Tauri IPC bridge not available');
    }}
  }} catch (err) {{
    const message = err?.message || String(err);
    console.error('[PoToken] Generation failed:', message);
    if (window.__TAURI_INTERNALS__?.event?.emit) {{
      window.__TAURI_INTERNALS__.event.emit(errorEvent, message);
    }}
  }}
}})();
"#,
        video_id_json = video_id_json,
        context_json = context_json,
        proxy_json = proxy_json,
        result_event_json = result_event_json,
        error_event_json = error_event_json,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verify that the execution script embeds the video_id and context safely.
    #[test]
    fn test_build_execution_script_contains_params() {
        let script = build_execution_script(
            "dQw4w9WgXcQ",
            r#"{"client":{"clientName":"WEB","clientVersion":"2.20240101"}}"#,
            &None,
            "test-label",
        );

        assert!(script.contains("dQw4w9WgXcQ"));
        assert!(script.contains("WEB"));
        assert!(script.contains("test-label"));
        assert!(script.contains("__mintPoToken"));
        assert!(script.contains("potoken:result:test-label"));
        assert!(script.contains("potoken:error:test-label"));
    }

    /// Verify that proxy URL is propagated into the script.
    #[test]
    fn test_build_execution_script_with_proxy() {
        let script = build_execution_script(
            "abc123",
            "{}",
            &Some("socks5://127.0.0.1:9050".to_string()),
            "lbl",
        );

        assert!(script.contains("socks5://127.0.0.1:9050"));
        assert!(script.contains("__proxyUrl"));
    }

    /// Verify that special characters in video_id are JSON-escaped.
    #[test]
    fn test_build_execution_script_escapes_quotes() {
        let script = build_execution_script(r#"foo"bar"#, "{}", &None, "lbl");

        // The double-quote should be escaped in the JS string.
        assert!(script.contains(r#"foo\"bar"#) || script.contains("foo\\\"bar"));
    }
}

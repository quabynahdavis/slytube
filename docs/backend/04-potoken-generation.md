# 04 - PoToken Generation

> **Domain:** `backend`
> **Status:** Design specification (implementation target for `src-tauri/src/potoken`)
> **Related:** [03-yt-dlp-sidecar.md](03-yt-dlp-sidecar.md), [06-network-proxy.md](06-network-proxy.md)

---

## 1. Background

YouTube's **Proof-of-Origin Token** (PoToken, `poToken`) is an attestation produced by BotGuard — an obfuscated JavaScript VM that YouTube serves and expects to be executed in a real browser environment. Requests for streaming URLs that lack a valid PoToken are increasingly answered with `403`, throttled to unusable bitrates, or rejected with *"Sign in to confirm you're not a bot"*.

Generating one requires:

1. A JavaScript runtime with a **DOM**, `window`, `navigator`, `document`, and working timers.
2. The ability to fetch and execute YouTube's BotGuard VM (`/js/bg-<hash>.js`) at runtime.
3. A **binding identifier** (the "context"): the visitor data for a session token, or the video ID for a content token.

A headless HTTP client cannot do this. Node-based projects use `jsdom` plus a shim; SlyTube instead uses what it already has — **a real Chromium/WebKit webview** — by opening a hidden `WebviewWindow`.

> **Shared infrastructure:** The hidden-webview pattern used here is also shared with the **extraction layer** (`src-tauri/src/extractor/`), which runs youtubei.js (Innertube) in a persistent hidden window for all InnerTube data extraction (search, channels, videos, comments). See ADR 007 for the extraction strategy decision.

### 1.1 Token types

| Type | Binding (`context`) | Used for | Lifetime |
|---|---|---|---|
| **Session token** | `visitorData` | All requests from this session; player config | ~6–12 h |
| **Content token** | `videoId` | Streaming URLs for one specific video | ~6 h |

SlyTube caches both: one session token per `visitorData`, and content tokens keyed by `video_id`.

---

## 2. Architecture: Hidden WebviewWindow

```
┌─────────────────────────────────────────────────────────────────────┐
│                          Rust backend                               │
│                                                                     │
│  generate_po_token(video_id, context, proxy_url)                    │
│        │                                                            │
│        │ 1. check cache ──────────────► hit? return                 │
│        │ 2. acquire generation mutex (serialise all requests)       │
│        │ 3. build hidden WebviewWindow  label = "potoken-<uuid>"    │
│        │        visible(false) · skip_taskbar · 1x1 · no devtools   │
│        │        url = potoken://generate                            │
│        ▼                                                            │
│  ┌───────────────────────────────────────────────────────────┐      │
│  │  custom protocol handler  potoken://                      │      │
│  │    /generate      → minimal HTML shell                    │      │
│  │    /botguard.js   → bundled botGuardScript.js             │      │
│  │    /config.json   → { videoId, context, proxied }         │      │
│  └───────────────────────────────────────────────────────────┘      │
│        │                                                            │
│        │ 4. webview runs BotGuard, POSTs result back                │
│        │    via  invoke('potoken_submit', ...)                      │
│        ▼                                                            │
│  oneshot::Receiver<Result<PoTokenResult>>  (30 s timeout)           │
│        │                                                            │
│        │ 5. destroy window · clear session data · release mutex     │
│        ▼                                                            │
│  cache + return { poToken, visitorData, expiresAt }                 │
└─────────────────────────────────────────────────────────────────────┘
```

### 2.1 Why hidden rather than headless

| Option | Verdict |
|---|---|
| `jsdom` in a JS runtime | Requires bundling Node/Deno; BotGuard actively probes for jsdom fingerprints |
| Headless Chrome via CDP | Ships a second ~150 MB browser; detected as headless |
| Bundled JS engine (`boa`, `quickjs`) | No DOM; BotGuard fails immediately |
| **Hidden WebviewWindow** | Zero extra binaries, genuine browser fingerprint, already in-process |

The trade-off is that the webview is a *real* window object with an event loop, so it must be created and destroyed carefully to avoid leaking processes.

### 2.2 Window construction

```rust
// src-tauri/src/potoken/window.rs
use tauri::{AppHandle, WebviewUrl, WebviewWindowBuilder};

pub async fn spawn_generator_window(
    app: &AppHandle,
    label: &str,
    proxy_url: Option<&str>,
) -> Result<tauri::WebviewWindow, AppError> {
    let mut builder = WebviewWindowBuilder::new(
        app,
        label,
        WebviewUrl::CustomProtocol("potoken://localhost/generate".parse().unwrap()),
    )
    .title("")
    .inner_size(1.0, 1.0)
    .position(-10_000.0, -10_000.0)   // off-screen as well as hidden
    .visible(false)
    .focused(false)
    .skip_taskbar(true)
    .decorations(false)
    .resizable(false)
    .always_on_top(false)
    // A stable, current desktop UA. A mismatched or webview-flavoured UA
    // is one of the strongest bot signals BotGuard looks at.
    .user_agent(DESKTOP_USER_AGENT)
    // Isolated storage per generation attempt.
    .data_directory(potoken_data_dir(app, label)?)
    .incognito(true);

    // Per-window proxy is supported on Tauri 2.x desktop backends.
    if let Some(p) = proxy_url {
        builder = builder.proxy_url(p.parse().map_err(|_| AppError::Invalid("bad proxy url".into()))?);
    }

    builder.build().map_err(|e| AppError::Other(e.to_string()))
}

pub const DESKTOP_USER_AGENT: &str =
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 \
     (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36";
```

Key flags:

- `visible(false)` **and** an off-screen position — some window managers briefly flash a window that is only later hidden.
- `incognito(true)` plus a per-attempt `data_directory` gives a clean storage partition.
- No devtools: the window is never inspectable in release builds.

### 2.3 Capability isolation

The generator window gets its own capability that exposes exactly one command:

```json
// src-tauri/capabilities/potoken.json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "potoken",
  "description": "Minimal surface for the hidden PoToken generator window",
  "windows": ["potoken-*"],
  "permissions": [
    "core:event:default"
  ]
}
```

`potoken_submit` is registered but guarded in Rust by checking the calling window label, so even if the capability drifts, a normal app window cannot spoof a token result:

```rust
#[tauri::command]
pub async fn potoken_submit(
    window: tauri::Window,
    state: State<'_, AppState>,
    payload: SubmitPayload,
) -> Result<(), AppError> {
    let label = window.label().to_string();
    if !label.starts_with("potoken-") {
        return Err(AppError::Invalid("unauthorised caller".into()));
    }
    state.potoken.resolve(&label, payload).await
}
```

---

## 3. Custom Protocol (`potoken://`)

A custom protocol is used rather than `WebviewUrl::App` for three reasons: the generator assets must not be reachable from the main app origin, the origin must look like an ordinary secure origin to BotGuard, and per-request config injection is trivial.

```rust
// src-tauri/src/potoken/protocol.rs
use tauri::http::{Request, Response, StatusCode};

pub fn register(builder: tauri::Builder<tauri::Wry>) -> tauri::Builder<tauri::Wry> {
    builder.register_asynchronous_uri_scheme_protocol("potoken", move |ctx, request, responder| {
        let app = ctx.app_handle().clone();
        let label = ctx.webview_label().to_string();

        tauri::async_runtime::spawn(async move {
            let response = handle(&app, &label, request).await
                .unwrap_or_else(|e| {
                    Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body(e.to_string().into_bytes())
                        .unwrap()
                });
            responder.respond(response);
        });
    })
}

async fn handle(
    app: &tauri::AppHandle,
    label: &str,
    request: Request<Vec<u8>>,
) -> Result<Response<Vec<u8>>, AppError> {
    // Only the hidden generator windows may use this scheme.
    if !label.starts_with("potoken-") {
        return Ok(Response::builder().status(StatusCode::FORBIDDEN).body(Vec::new()).unwrap());
    }

    let path = request.uri().path();

    match path {
        "/generate" => Ok(html_response(GENERATOR_HTML)),

        "/botguard.js" => Ok(js_response(
            include_str!("../../assets/potoken/botGuardScript.js")
        )),

        "/config.json" => {
            let state: tauri::State<AppState> = app.state();
            let cfg = state.potoken.pending_config(label).await
                .ok_or_else(|| AppError::NotFound("no pending generation".into()))?;
            Ok(json_response(&cfg))
        }

        _ => Ok(Response::builder().status(StatusCode::NOT_FOUND).body(Vec::new()).unwrap()),
    }
}

fn html_response(body: &str) -> Response<Vec<u8>> {
    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/html; charset=utf-8")
        // Allow YouTube's BotGuard VM to be fetched and run; nothing else.
        .header(
            "Content-Security-Policy",
            "default-src 'none'; \
             script-src 'self' 'unsafe-eval' https://www.youtube.com https://www.google.com; \
             connect-src https://www.youtube.com https://www.google.com; \
             img-src data:; style-src 'unsafe-inline'",
        )
        .body(body.as_bytes().to_vec())
        .unwrap()
}
```

> `'unsafe-eval'` is unavoidable: BotGuard's VM is a self-interpreting bytecode machine that constructs functions at runtime. Containing it is precisely why this runs in a throwaway origin with no access to app data.

### 3.1 Generator shell

```html
<!-- GENERATOR_HTML -->
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8" />
  <title>&nbsp;</title>
</head>
<body>
  <script type="module">
    const invoke = window.__TAURI__.core.invoke;

    async function main() {
      const cfg = await (await fetch('potoken://localhost/config.json')).json();
      const { generatePoToken } = await import('potoken://localhost/botguard.js');

      const timer = setTimeout(() => {
        invoke('potoken_submit', { payload: { ok: false, error: 'script timeout' } });
      }, 25_000);

      try {
        const result = await generatePoToken({
          videoId: cfg.videoId ?? null,
          context: cfg.context,
          visitorData: cfg.visitorData ?? null,
        });
        clearTimeout(timer);
        await invoke('potoken_submit', { payload: { ok: true, ...result } });
      } catch (err) {
        clearTimeout(timer);
        await invoke('potoken_submit', {
          payload: { ok: false, error: String(err?.message ?? err) },
        });
      }
    }

    main();
  </script>
</body>
</html>
```

---

## 4. `botGuardScript.js`

Bundled at `src-tauri/assets/potoken/botGuardScript.js` and served only over `potoken://`.

### 4.1 Responsibilities

1. Fetch YouTube's challenge descriptor (`/youtubei/v1/att/get`) to obtain the BotGuard program.
2. Load and instantiate the VM (`window.trayride` / `globalThis.bg` callback surface).
3. Run the challenge to obtain the integrity token.
4. Mint a token bound to the supplied context via the `WebPoSignalOutput` minter.

### 4.2 Shape

```js
// src-tauri/assets/potoken/botGuardScript.js
const REQUEST_KEY = 'O43z0dpjhgX20SCx4KAo';   // public YouTube web constant

async function fetchChallenge() {
  const res = await fetch('https://www.youtube.com/youtubei/v1/att/get?prettyPrint=false', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'X-Goog-Api-Key': 'AIzaSyAO_FJ2SlqU8Q4STEHLGCilw_Y9_11qcW8',
    },
    body: JSON.stringify({
      engagementType: 'ENGAGEMENT_TYPE_UNBOUND',
      context: {
        client: { clientName: 'WEB', clientVersion: '2.20240726.00.00' },
      },
    }),
  });
  if (!res.ok) throw new Error(`challenge fetch failed: ${res.status}`);
  const data = await res.json();
  const raw = data.bgChallenge ?? JSON.parse(atob(data.challenge));
  return raw;
}

function loadVm(scriptSrc) {
  return new Promise((resolve, reject) => {
    const el = document.createElement('script');
    el.src = scriptSrc;
    el.onload = () => resolve();
    el.onerror = () => reject(new Error('failed to load BotGuard VM'));
    document.head.appendChild(el);
  });
}

async function runChallenge(challenge) {
  const [scriptSrc] = challenge.interpreterJavascript
    ? [null]
    : [challenge.interpreterUrl?.privateDoNotAccessOrElseTrustedResourceUrlWrappedValue];

  if (challenge.interpreterJavascript?.privateDoNotAccessOrElseSafeScriptWrappedValue) {
    // Inline interpreter: evaluate in this throwaway origin.
    new Function(
      challenge.interpreterJavascript.privateDoNotAccessOrElseSafeScriptWrappedValue
    )();
  } else if (scriptSrc) {
    await loadVm(scriptSrc.startsWith('//') ? `https:${scriptSrc}` : scriptSrc);
  } else {
    throw new Error('no interpreter in challenge');
  }

  const vm = globalThis[challenge.globalName];
  if (!vm) throw new Error('BotGuard VM did not register');

  return new Promise((resolve, reject) => {
    vm.a(challenge.program, (fn) => {
      fn().then((handles) => resolve(handles)).catch(reject);
    }, true, undefined, () => {});
  });
}

async function mintIntegrityToken(botguardResponse) {
  const res = await fetch(
    'https://www.youtube.com/api/jnn/v1/GenerateIT?prettyPrint=false',
    {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json+protobuf',
        'X-Goog-Api-Key': 'AIzaSyDyT5W0Jh49F30Pqqtyfdf7pDLFKLJoAnw',
      },
      body: JSON.stringify([REQUEST_KEY, botguardResponse]),
    }
  );
  if (!res.ok) throw new Error(`integrity token failed: ${res.status}`);
  const [token, ttlSeconds] = await res.json();
  return { token, ttlSeconds: ttlSeconds ?? 21_600 };
}

/**
 * @param {{ videoId: string|null, context: string, visitorData: string|null }} opts
 * @returns {Promise<{ poToken: string, visitorData: string|null, ttlSeconds: number }>}
 */
export async function generatePoToken({ videoId, context, visitorData }) {
  const challenge = await fetchChallenge();
  const handles = await runChallenge(challenge);
  const { token: integrityToken, ttlSeconds } = await mintIntegrityToken(handles.botguardResponse);

  // Bind the token to the requested identifier.
  const identifier = context === 'content' ? videoId : visitorData;
  if (!identifier) throw new Error(`missing identifier for context '${context}'`);

  const minter = await handles.integrityTokenMinter(integrityToken);
  const bytes = await minter(identifier);
  const poToken = btoa(String.fromCharCode(...bytes))
    .replace(/\+/g, '-')
    .replace(/\//g, '_');

  return { poToken, visitorData, ttlSeconds };
}
```

> **Maintenance reality.** YouTube changes the challenge endpoints, request keys, and VM entry points without notice. This file is the single highest-churn asset in the codebase. It is versioned (`BOTGUARD_SCRIPT_VERSION`) and reported in diagnostics so failures can be correlated with a known-bad revision, and the app must degrade gracefully — never hard-fail playback — when it stops working.

---

## 5. Command: `generate_po_token`

### 5.1 Signature

```rust
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PoTokenRequest {
    pub video_id: Option<String>,
    pub context: PoTokenContext,        // 'session' | 'content'
    pub proxy_url: Option<String>,
    pub visitor_data: Option<String>,
    #[serde(default)]
    pub force_refresh: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum PoTokenContext { Session, Content }

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PoTokenResult {
    pub po_token: String,
    pub visitor_data: Option<String>,
    pub context: PoTokenContext,
    pub video_id: Option<String>,
    pub generated_at: i64,
    pub expires_at: i64,
    pub from_cache: bool,
}
```

### 5.2 Implementation

```rust
#[tauri::command]
pub async fn generate_po_token(
    app: AppHandle,
    state: State<'_, AppState>,
    video_id: Option<String>,
    context: PoTokenContext,
    proxy_url: Option<String>,
) -> Result<PoTokenResult, AppError> {
    let req = PoTokenRequest { video_id, context, proxy_url, visitor_data: None, force_refresh: false };

    if context == PoTokenContext::Content && req.video_id.is_none() {
        return Err(AppError::Invalid("content tokens require a videoId".into()));
    }

    let cache_key = req.cache_key();

    // 1. Cache
    if !req.force_refresh {
        if let Some(hit) = state.potoken.cache_get(&cache_key).await {
            return Ok(PoTokenResult { from_cache: true, ..hit });
        }
    }

    // 2. Coalesce: only one generation at a time, process-wide.
    //    BotGuard rate-limits aggressively and parallel windows are wasteful.
    let _permit = state.potoken.generation_lock.lock().await;

    // Re-check: another caller may have populated the cache while we waited.
    if let Some(hit) = state.potoken.cache_get(&cache_key).await {
        return Ok(PoTokenResult { from_cache: true, ..hit });
    }

    let label = format!("potoken-{}", uuid::Uuid::new_v4().simple());
    let (tx, rx) = tokio::sync::oneshot::channel();
    state.potoken.register_pending(&label, &req, tx).await;

    // 3. Spawn the hidden window.
    let window = match spawn_generator_window(&app, &label, req.proxy_url.as_deref()).await {
        Ok(w) => w,
        Err(e) => {
            state.potoken.clear_pending(&label).await;
            return Err(e);
        }
    };

    // 4. Await the result with a hard timeout.
    let outcome = tokio::time::timeout(std::time::Duration::from_secs(30), rx).await;

    // 5. Always tear down, on every path.
    cleanup_session(&app, &window, &label).await;
    state.potoken.clear_pending(&label).await;

    let payload = match outcome {
        Err(_)          => return Err(AppError::Other("PoToken generation timed out".into())),
        Ok(Err(_))      => return Err(AppError::Other("PoToken channel closed".into())),
        Ok(Ok(payload)) => payload,
    };

    if !payload.ok {
        return Err(AppError::Other(payload.error.unwrap_or_else(|| "unknown BotGuard error".into())));
    }

    let now = now_ms();
    let ttl = payload.ttl_seconds.unwrap_or(21_600).clamp(60, 43_200);
    let result = PoTokenResult {
        po_token: payload.po_token.ok_or_else(|| AppError::Other("empty token".into()))?,
        visitor_data: payload.visitor_data,
        context: req.context,
        video_id: req.video_id.clone(),
        generated_at: now,
        // Refresh 5 minutes early to avoid using a token that expires mid-stream.
        expires_at: now + (ttl * 1000) - 300_000,
        from_cache: false,
    };

    state.potoken.cache_put(&cache_key, &result).await;
    Ok(result)
}
```

### 5.3 Behaviour summary

| Aspect | Value |
|---|---|
| Concurrency | Serialised by a process-wide `Mutex`; concurrent callers coalesce onto one generation |
| Timeout | 30 s outer (Rust), 25 s inner (JS) so the JS error message usually wins |
| Retry | Two retries with 2 s / 8 s backoff, only for transient classes (`network`, `timeout`) |
| Cache key | `session:<visitorData\|default>` or `content:<videoId>` |
| Early refresh | `expires_at` is TTL minus 5 min |
| Failure mode | Returns `Err`; callers proceed without a token rather than blocking playback |

### 5.4 Consumers

- **Player** — attaches `poToken` + `visitorData` to `youtubei` player requests.
- **yt-dlp** — passed through as `--extractor-args "youtube:po_token=web.gvs+<TOKEN>"` when a download fails with `bot_check` (see [03-yt-dlp-sidecar.md](03-yt-dlp-sidecar.md#54-error-classification)).

```rust
pub fn potoken_extractor_arg(t: &PoTokenResult) -> Vec<String> {
    vec![
        "--extractor-args".into(),
        format!("youtube:player_client=web,po_token=web.gvs+{}", t.po_token),
    ]
}
```

---

## 6. Session Cleanup

BotGuard fingerprints storage. A second generation from a window that inherited cookies, IndexedDB, or a service worker from the first is materially more likely to be rejected — and long-lived hidden webviews leak memory. **Every generation gets a fresh window and a fresh storage partition.**

```rust
async fn cleanup_session(app: &AppHandle, window: &tauri::WebviewWindow, label: &str) {
    // 1. Best-effort in-page wipe before teardown.
    let _ = window.eval(
        r#"
        try {
          localStorage.clear();
          sessionStorage.clear();
          if (window.indexedDB?.databases) {
            indexedDB.databases().then(dbs =>
              dbs.forEach(d => d.name && indexedDB.deleteDatabase(d.name)));
          }
          if (navigator.serviceWorker) {
            navigator.serviceWorker.getRegistrations()
              .then(rs => rs.forEach(r => r.unregister()));
          }
          if (window.caches) {
            caches.keys().then(ks => ks.forEach(k => caches.delete(k)));
          }
        } catch (_) {}
        "#,
    );

    // 2. Give the wipe a moment, then destroy the window.
    tokio::time::sleep(std::time::Duration::from_millis(150)).await;
    let _ = window.destroy();

    // 3. Remove the per-attempt data directory from disk.
    if let Ok(dir) = potoken_data_dir(app, label) {
        let _ = tokio::fs::remove_dir_all(&dir).await;
    }

    tracing::debug!(%label, "potoken session cleaned up");
}
```

### 6.1 Data directory

```rust
fn potoken_data_dir(app: &AppHandle, label: &str) -> Result<std::path::PathBuf, AppError> {
    let base = app.path().app_cache_dir()?.join("potoken-sessions");
    std::fs::create_dir_all(&base)?;
    Ok(base.join(label))
}
```

### 6.2 Orphan sweeping

A crash between window creation and cleanup leaves a directory behind. On startup:

```rust
pub async fn sweep_orphan_sessions(app: &AppHandle) {
    let Ok(base) = app.path().app_cache_dir().map(|d| d.join("potoken-sessions")) else { return };
    let Ok(mut entries) = tokio::fs::read_dir(&base).await else { return };

    while let Ok(Some(entry)) = entries.next_entry().await {
        // No generator windows exist at startup, so every directory is stale.
        let _ = tokio::fs::remove_dir_all(entry.path()).await;
    }
}
```

Also registered: an app-exit hook that destroys any window whose label starts with `potoken-`.

### 6.3 Cache invalidation

| Trigger | Effect |
|---|---|
| `expires_at` reached | Entry dropped lazily on next read |
| Proxy settings change | **Entire cache cleared** — tokens are IP-bound |
| `visitorData` rotation | Session entry cleared |
| Playback `403` despite a cached token | That entry evicted, one forced regeneration |
| App restart | Cache is in-memory only; nothing persists |

Tokens are deliberately **never written to disk**. They are short-lived credentials, and persisting them adds exfiltration risk for no meaningful benefit.

---

## 7. Proxy Support

The PoToken must be minted from the **same egress IP** that will later fetch the streams. Minting through the direct connection and streaming through a proxy produces a token YouTube rejects.

### 7.1 Propagation

```
settings.proxy ──┬──► reqwest client        (API/metadata)
                 ├──► yt-dlp --proxy        (downloads)
                 └──► WebviewWindowBuilder::proxy_url  (PoToken generation)
```

```rust
#[tauri::command]
pub async fn generate_po_token_auto(
    app: AppHandle,
    state: State<'_, AppState>,
    video_id: Option<String>,
    context: PoTokenContext,
) -> Result<PoTokenResult, AppError> {
    // Resolve the effective proxy from settings when the caller passes none.
    let proxy_url = state.proxy.current_url().await;
    generate_po_token(app, state, video_id, context, proxy_url).await
}
```

### 7.2 Supported schemes

| Scheme | Webview support | Notes |
|---|---|---|
| `http://` | Yes | Most portable |
| `https://` | Yes | |
| `socks5://` | Yes | Local DNS resolution |
| `socks5h://` | Platform-dependent | Remote DNS; preferred for Tor. Falls back to `socks5` with a warning where unsupported |

Proxy credentials are **not** placed in the `proxy_url` passed to the webview where avoidable, since some backends log the full URL. Where the platform offers no credential callback, the app warns that authenticated proxies may leak credentials into platform logs.

### 7.3 Failure handling

```rust
match generate_po_token(app.clone(), state.clone(), video_id.clone(), ctx, proxy.clone()).await {
    Ok(t) => Ok(t),
    Err(e) if is_proxy_error(&e) && settings.allow_direct_fallback => {
        tracing::warn!(?e, "potoken via proxy failed; falling back to direct");
        // Only permitted when the user has explicitly allowed leaking direct
        // connections; off by default, and never in 'enhanced' privacy mode.
        generate_po_token(app, state, video_id, ctx, None).await
    }
    Err(e) => Err(e),
}
```

The fallback defaults to **off**. Silently bypassing a proxy is a privacy failure, so the behaviour is opt-in and surfaced in the UI with an explicit warning.

---

## 8. Diagnostics

`potoken_diagnostics()` returns, for the Settings → Troubleshooting panel:

```rust
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PoTokenDiagnostics {
    pub script_version: &'static str,
    pub cache_entries: usize,
    pub last_success_at: Option<i64>,
    pub last_failure_at: Option<i64>,
    pub last_error: Option<String>,
    pub total_generated: u64,
    pub total_failed: u64,
    pub average_duration_ms: Option<u64>,
    pub proxy_in_use: bool,
}
```

Tracing spans wrap each generation (`potoken.generate` with `context`, `video_id`, `duration_ms`, `outcome`). Token values are **never** logged — only their length and a 6-character prefix, which is enough to correlate a cached token with a failing request without writing a credential to disk.

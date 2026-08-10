# 06 - Network & Proxy

> **Domain:** `backend`
> **Status:** Design specification (implementation target for `src-tauri/src/net`)
> **Related:** [03-yt-dlp-sidecar.md](03-yt-dlp-sidecar.md), [04-potoken-generation.md](04-potoken-generation.md)

---

## 1. Overview

All outbound HTTP originates in **Rust**, never in the webview. The renderer has no direct network access to YouTube or Invidious; it calls a fetch-wrapper command that proxies the request through `reqwest`.

```
┌────────────────────────────────────────────────────────────────────┐
│  Renderer (Vue)                                                    │
│    ytFetch(path, opts) ──► invoke('net_fetch', …)                  │
└──────────────────────────────┬─────────────────────────────────────┘
                               │
┌──────────────────────────────▼─────────────────────────────────────┐
│  Rust net layer                                                    │
│    ┌────────────────────────────────────────────────────────────┐  │
│    │ ClientRegistry                                             │  │
│    │   default (global proxy)  ·  per-proxy pool  ·  no-proxy   │  │
│    └────────────────────────────────────────────────────────────┘  │
│    header injection · host allow-list · size caps · retries        │
└──────┬─────────────────────┬───────────────────────┬───────────────┘
       ▼                     ▼                       ▼
   YouTube API          Invidious API             image CDNs
                                                (Rust image cache)
```

Why route everything through Rust:

- **One proxy switch.** The webview's own `fetch` bypasses any proxy the app configures, silently leaking the real IP.
- **Header control.** YouTube's InnerTube API needs headers (`Origin`, `X-Goog-*`, `User-Agent`) that the browser refuses to let JS set.
- **No CORS.** `reqwest` is not subject to the same-origin policy.
- **Auditability.** Every request passes one choke point that can be logged, capped, and rate-limited.

---

## 2. reqwest Client Configuration

```toml
# src-tauri/Cargo.toml
[dependencies]
reqwest = { version = "0.12", default-features = false, features = [
    "rustls-tls",
    "json",
    "gzip", "brotli", "deflate",
    "stream",
    "socks",
    "cookies",
] }
url  = "2"
bytes = "1"
tokio-util = { version = "0.7", features = ["io"] }
```

`rustls-tls` rather than native-TLS: consistent behaviour across platforms and no OpenSSL build dependency on Linux CI.

### 2.1 Builder

```rust
// src-tauri/src/net/client.rs
use reqwest::{Client, Proxy};
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct ProxyConfig {
    pub protocol: String,          // 'http' | 'https' | 'socks5' | 'socks5h'
    pub host: String,
    pub port: u16,
    pub username: Option<String>,
    pub password: Option<String>,
    pub bypass: Vec<String>,       // hosts / CIDRs to reach directly
}

impl ProxyConfig {
    pub fn to_url(&self) -> String {
        format!("{}://{}:{}", self.protocol, self.host, self.port)
    }
}

pub fn build_client(proxy: Option<&ProxyConfig>) -> Result<Client, AppError> {
    let mut b = Client::builder()
        .user_agent(DESKTOP_USER_AGENT)
        .timeout(Duration::from_secs(30))
        .connect_timeout(Duration::from_secs(10))
        .pool_idle_timeout(Duration::from_secs(90))
        .pool_max_idle_per_host(6)
        .redirect(reqwest::redirect::Policy::limited(5))
        .https_only(false)          // some Invidious instances are plain HTTP on LAN
        .gzip(true).brotli(true).deflate(true)
        .cookie_store(true)
        .tcp_keepalive(Duration::from_secs(60))
        .tcp_nodelay(true);

    if let Some(p) = proxy {
        let mut proxy_obj = Proxy::all(p.to_url())
            .map_err(|e| AppError::Network(format!("invalid proxy: {e}")))?;

        if let (Some(u), Some(pw)) = (&p.username, &p.password) {
            proxy_obj = proxy_obj.basic_auth(u, pw);
        }

        if !p.bypass.is_empty() {
            let bypass = p.bypass.clone();
            proxy_obj = proxy_obj.no_proxy(
                reqwest::NoProxy::from_string(&bypass.join(",")).unwrap_or_else(no_proxy_default),
            );
        }

        b = b.proxy(proxy_obj);
    } else {
        // Explicitly ignore ambient HTTP_PROXY/ALL_PROXY env vars: the app's
        // own setting is the single source of truth for egress.
        b = b.no_proxy();
    }

    b.build().map_err(|e| AppError::Network(e.to_string()))
}

pub const DESKTOP_USER_AGENT: &str =
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 \
     (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36";
```

### 2.2 Client registry

Building a `Client` per request destroys connection pooling. Clients are cached by proxy configuration and rebuilt only when settings change.

```rust
pub struct ClientRegistry {
    default: RwLock<Client>,                        // follows global settings
    direct:  Client,                                // never proxied
    per_proxy: RwLock<HashMap<ProxyConfig, Client>>,
}

impl ClientRegistry {
    pub async fn get(&self, override_proxy: Option<&ProxyConfig>) -> Result<Client, AppError> {
        match override_proxy {
            None => Ok(self.default.read().await.clone()),   // Client is Arc-backed; cheap
            Some(p) => {
                if let Some(c) = self.per_proxy.read().await.get(p) {
                    return Ok(c.clone());
                }
                let c = build_client(Some(p))?;
                self.per_proxy.write().await.insert(p.clone(), c.clone());
                Ok(c)
            }
        }
    }

    /// Called when the user saves new proxy settings.
    pub async fn reconfigure(&self, proxy: Option<&ProxyConfig>) -> Result<(), AppError> {
        *self.default.write().await = build_client(proxy)?;
        self.per_proxy.write().await.clear();
        Ok(())
    }
}
```

The registry is capped at 8 entries with LRU eviction so a script cannot exhaust memory by cycling proxies.

---

## 3. Per-Request Proxy

Some flows legitimately need a different egress than the global setting: testing a candidate proxy, comparing Invidious instances, or an explicit "open this once directly" action.

```rust
#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RequestProxy {
    /// 'global' (default) | 'direct' | 'custom'
    pub mode: Option<String>,
    pub config: Option<ProxyConfig>,
}

pub async fn resolve_client(
    state: &AppState,
    rp: Option<&RequestProxy>,
) -> Result<Client, AppError> {
    match rp.and_then(|r| r.mode.as_deref()).unwrap_or("global") {
        "global" => state.clients.get(None).await,
        "direct" => {
            // Refuse to silently bypass the proxy in enhanced privacy mode.
            if state.privacy_mode().await == PrivacyMode::Enhanced {
                return Err(AppError::Invalid(
                    "direct connections are disabled in enhanced privacy mode".into(),
                ));
            }
            Ok(state.clients.direct.clone())
        }
        "custom" => {
            let cfg = rp.and_then(|r| r.config.as_ref())
                .ok_or_else(|| AppError::Invalid("custom proxy requires a config".into()))?;
            state.clients.get(Some(cfg)).await
        }
        other => Err(AppError::Invalid(format!("unknown proxy mode: {other}"))),
    }
}
```

### 3.1 Consistency requirement

Proxy choice must be **uniform across a playback session**. YouTube binds stream URLs and PoTokens to the requesting IP, so a metadata call through the proxy followed by a media fetch made directly yields `403`. Consequences:

- Changing the proxy clears the PoToken cache ([04](04-potoken-generation.md#63-cache-invalidation)) and the playback-info cache ([03](03-yt-dlp-sidecar.md#45-ytdlp_get_playback_info)).
- yt-dlp always receives the same `--proxy` the metadata request used.
- The PoToken webview is built with the same `proxy_url`.

```rust
pub async fn apply_proxy_change(app: &AppHandle, state: &AppState, cfg: Option<ProxyConfig>) -> Result<(), AppError> {
    state.clients.reconfigure(cfg.as_ref()).await?;
    state.potoken.clear_cache().await;
    state.playback_cache.clear().await;
    state.image_cache.invalidate_pending().await;
    let _ = app.emit("network:proxy-changed", cfg.as_ref().map(|c| c.to_url()));
    Ok(())
}
```

### 3.2 `network_test_proxy`

```rust
#[tauri::command]
pub async fn test_proxy(req: ProxyTestRequest) -> Result<ProxyTestResult, AppError> {
    let cfg = ProxyConfig {
        protocol: req.protocol, host: req.host, port: req.port,
        username: req.username, password: req.password, bypass: vec![],
    };

    // Throwaway client: never touches the registry or saved settings.
    let client = build_client(Some(&cfg))?;
    let url = req.test_url.unwrap_or_else(|| "https://www.youtube.com/generate_204".into());
    let timeout = Duration::from_millis(req.timeout_ms.unwrap_or(10_000).min(30_000));

    let start = std::time::Instant::now();
    let result = tokio::time::timeout(timeout, client.get(&url).send()).await;
    let latency_ms = start.elapsed().as_millis();

    Ok(match result {
        Ok(Ok(resp)) => ProxyTestResult {
            ok: resp.status().is_success() || resp.status().as_u16() == 204,
            status: Some(resp.status().as_u16()),
            latency_ms, resolved_ip: None, error: None,
        },
        Ok(Err(e))  => ProxyTestResult { ok: false, status: None, latency_ms,
                                         resolved_ip: None, error: Some(sanitize_err(&e)) },
        Err(_)      => ProxyTestResult { ok: false, status: None, latency_ms,
                                         resolved_ip: None, error: Some("timed out".into()) },
    })
}
```

`sanitize_err` strips credentials before the error string ever reaches the UI or the log.

### 3.3 IP leak check

An optional follow-up compares the egress IP with and without the proxy:

```rust
pub async fn check_leak(state: &AppState) -> Result<LeakReport, AppError> {
    let direct  = fetch_ip(&state.clients.direct).await.ok();
    let proxied = fetch_ip(&state.clients.get(None).await?).await.ok();
    Ok(LeakReport {
        leaking: matches!((&direct, &proxied), (Some(a), Some(b)) if a == b),
        proxied_ip: proxied,
    })
}
```

The direct IP is **never** returned to the frontend or logged — only the boolean and the proxied IP.

---

## 4. Renderer Fetch Wrapper

### 4.1 Command

```rust
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FetchRequest {
    pub target: FetchTarget,          // 'youtube' | 'youtubei' | 'invidious' | 'generic'
    pub method: Option<String>,       // default GET
    pub path: String,                 // path+query, or absolute URL when target='generic'
    pub body: Option<serde_json::Value>,
    pub headers: Option<HashMap<String, String>>,
    pub proxy: Option<RequestProxy>,
    pub timeout_ms: Option<u64>,
    pub instance: Option<String>,     // Invidious instance override
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FetchResponse {
    pub status: u16,
    pub ok: bool,
    pub headers: HashMap<String, String>,
    pub body: serde_json::Value,      // parsed JSON, or { "text": "..." }
    pub final_url: String,
    pub duration_ms: u64,
}

#[tauri::command]
pub async fn net_fetch(
    state: State<'_, AppState>,
    req: FetchRequest,
) -> Result<FetchResponse, AppError> {
    let url = resolve_url(&state, &req).await?;
    enforce_host_allowlist(&url, req.target)?;

    let client = resolve_client(&state, req.proxy.as_ref()).await?;
    let method = reqwest::Method::from_bytes(
        req.method.as_deref().unwrap_or("GET").as_bytes(),
    ).map_err(|_| AppError::Invalid("bad method".into()))?;

    let mut rb = client.request(method, url.clone())
        .headers(target_headers(req.target, &url, &state).await?)
        .timeout(Duration::from_millis(req.timeout_ms.unwrap_or(30_000).min(120_000)));

    // Caller headers are merged last but cannot override protected names.
    if let Some(h) = &req.headers {
        for (k, v) in h {
            if is_protected_header(k) { continue; }
            rb = rb.header(k, v);
        }
    }
    if let Some(body) = &req.body {
        rb = rb.json(body);
    }

    let start = std::time::Instant::now();
    let resp = rb.send().await.map_err(|e| AppError::Network(sanitize_err(&e)))?;

    let status = resp.status();
    let final_url = resp.url().to_string();
    let headers = collect_safe_headers(resp.headers());

    // Cap the body: a hostile or misbehaving instance must not OOM the app.
    let bytes = read_capped(resp, 16 * 1024 * 1024).await?;
    let body = serde_json::from_slice(&bytes)
        .unwrap_or_else(|_| json!({ "text": String::from_utf8_lossy(&bytes) }));

    Ok(FetchResponse {
        status: status.as_u16(),
        ok: status.is_success(),
        headers, body, final_url,
        duration_ms: start.elapsed().as_millis() as u64,
    })
}
```

### 4.2 Target-specific headers

```rust
async fn target_headers(
    target: FetchTarget,
    url: &Url,
    state: &AppState,
) -> Result<HeaderMap, AppError> {
    let mut h = HeaderMap::new();
    h.insert(ACCEPT_LANGUAGE, HeaderValue::from_static("en-US,en;q=0.9"));

    match target {
        FetchTarget::Youtubei => {
            // InnerTube requires a browser-shaped request.
            h.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
            h.insert(ORIGIN,  HeaderValue::from_static("https://www.youtube.com"));
            h.insert(REFERER, HeaderValue::from_static("https://www.youtube.com/"));
            h.insert("X-YouTube-Client-Name",    HeaderValue::from_static("1"));
            h.insert("X-YouTube-Client-Version", HeaderValue::from_static("2.20240726.00.00"));
            h.insert("X-Goog-Api-Format-Version", HeaderValue::from_static("1"));
            h.insert("Sec-Fetch-Mode", HeaderValue::from_static("same-origin"));
            h.insert("Sec-Fetch-Site", HeaderValue::from_static("same-origin"));

            if let Some(vd) = state.session.visitor_data().await {
                h.insert("X-Goog-Visitor-Id", HeaderValue::from_str(&vd)
                    .map_err(|_| AppError::Invalid("bad visitor data".into()))?);
            }
            // Consent cookie avoids the EU interstitial.
            h.insert(COOKIE, HeaderValue::from_static("SOCS=CAI; CONSENT=YES+cb"));
        }

        FetchTarget::Youtube => {
            h.insert(ACCEPT, HeaderValue::from_static(
                "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8"));
            h.insert(REFERER, HeaderValue::from_static("https://www.youtube.com/"));
            h.insert(COOKIE,  HeaderValue::from_static("SOCS=CAI; CONSENT=YES+cb"));
            h.insert("Sec-Fetch-Dest", HeaderValue::from_static("document"));
        }

        FetchTarget::Invidious => {
            h.insert(ACCEPT, HeaderValue::from_static("application/json"));
            // Do NOT forward a YouTube UA or cookies to third-party instances.
            h.insert(USER_AGENT, HeaderValue::from_static("SlyTube/0.1 (+https://github.com/slytube)"));
        }

        FetchTarget::Generic => {
            h.insert(ACCEPT, HeaderValue::from_static("*/*"));
        }
    }

    let _ = url;
    Ok(h)
}
```

The Invidious case matters: forwarding YouTube-specific headers and cookies to an arbitrary community-run instance would hand it a session fingerprint it has no business seeing.

### 4.3 Host allow-list

```rust
const YT_HOSTS: &[&str] = &[
    "www.youtube.com", "youtube.com", "m.youtube.com",
    "youtubei.googleapis.com", "www.google.com",
    "i.ytimg.com", "yt3.ggpht.com", "yt4.ggpht.com",
];

fn enforce_host_allowlist(url: &Url, target: FetchTarget) -> Result<(), AppError> {
    let host = url.host_str().ok_or_else(|| AppError::Invalid("no host".into()))?;

    match target {
        FetchTarget::Youtube | FetchTarget::Youtubei => {
            if !YT_HOSTS.contains(&host) {
                return Err(AppError::Invalid(format!("host not permitted: {host}")));
            }
        }
        FetchTarget::Invidious => { /* validated against the configured instance list */ }
        FetchTarget::Generic => {
            // SSRF guard: block loopback, link-local, and private ranges.
            if is_internal_host(host) {
                return Err(AppError::Invalid("internal addresses are not permitted".into()));
            }
        }
    }

    if !matches!(url.scheme(), "http" | "https") {
        return Err(AppError::Invalid("only http(s) is supported".into()));
    }
    Ok(())
}
```

### 4.4 TypeScript wrapper

```ts
// src/lib/net.ts
import { invoke } from '@tauri-apps/api/core'

type Target = 'youtube' | 'youtubei' | 'invidious' | 'generic'

interface FetchOpts {
  method?: 'GET' | 'POST'
  body?: unknown
  headers?: Record<string, string>
  proxy?: { mode?: 'global' | 'direct' | 'custom'; config?: ProxyConfig }
  timeoutMs?: number
  instance?: string
}

export async function netFetch<T>(target: Target, path: string, opts: FetchOpts = {}): Promise<T> {
  const res = await invoke<FetchResponse>('net_fetch', { req: { target, path, ...opts } })
  if (!res.ok) throw new HttpError(res.status, res.finalUrl, res.body)
  return res.body as T
}

export const yt = {
  player:  (videoId: string, poToken?: string) =>
    netFetch<PlayerResponse>('youtubei', '/youtubei/v1/player?prettyPrint=false', {
      method: 'POST',
      body: {
        videoId,
        context: { client: { clientName: 'WEB', clientVersion: '2.20240726.00.00' } },
        ...(poToken
          ? { serviceIntegrityDimensions: { poToken } }
          : {}),
      },
    }),

  search: (query: string) =>
    netFetch<SearchResponse>('youtubei', '/youtubei/v1/search?prettyPrint=false', {
      method: 'POST',
      body: { query, context: { client: { clientName: 'WEB', clientVersion: '2.20240726.00.00' } } },
    }),
}

export const invidious = {
  videos: (instance: string, id: string) =>
    netFetch<InvVideo>('invidious', `/api/v1/videos/${id}`, { instance }),
}
```

### 4.5 Retry policy

```rust
pub async fn with_retry<F, Fut, T>(mut op: F, max: u32) -> Result<T, AppError>
where F: FnMut(u32) -> Fut, Fut: Future<Output = Result<T, AppError>> {
    let mut attempt = 0;
    loop {
        match op(attempt).await {
            Ok(v) => return Ok(v),
            Err(e) if attempt < max && is_retryable(&e) => {
                // 500ms, 1s, 2s, 4s + jitter
                let base = 500u64 << attempt;
                let jitter = rand::random::<u64>() % 250;
                tokio::time::sleep(Duration::from_millis(base + jitter)).await;
                attempt += 1;
            }
            Err(e) => return Err(e),
        }
    }
}
```

Retryable: connect/timeout errors, `429`, `502`, `503`, `504`. Never retryable: `4xx` other than `429` (retrying an `403` bot-check just burns quota — the correct response is to regenerate a PoToken).

### 4.6 Rate limiting

A token bucket per host (10 requests/second, burst 20) sits in front of the client. Invidious instances additionally get a 4-concurrent-request cap, since many are volunteer-run on modest hardware and aggressive clients get IP-banned.

---

## 5. Image Cache

Thumbnails dominate request volume — a subscription feed can reference 200+ images.

### 5.1 Options considered

| Approach | Pros | Cons | Verdict |
|---|---|---|---|
| Plain `<img src="https://i.ytimg.com/…">` | Zero code | Bypasses the proxy entirely, leaks IP, no offline | **Rejected** |
| ServiceWorker cache | Standard API, transparent to Vue | SW `fetch` cannot use the Rust proxy; needs a secure origin; opaque cross-origin responses can't be inspected; eviction is opaque | Rejected as primary |
| **Rust custom protocol + disk cache** | Honours the proxy, works offline, precise eviction, no CORS | Custom protocol plumbing | **Chosen** |

The decisive factor is the proxy: a ServiceWorker's `fetch` goes through the webview's network stack, which does not respect the app's proxy configuration. Every thumbnail would leak the user's real IP.

### 5.2 Protocol handler

```rust
// src-tauri/src/net/image_protocol.rs
// Usage in markup:  <img src="slyimg://localhost/?u=<url-encoded>&w=480">

pub fn register(builder: tauri::Builder<tauri::Wry>) -> tauri::Builder<tauri::Wry> {
    builder.register_asynchronous_uri_scheme_protocol("slyimg", |ctx, request, responder| {
        let app = ctx.app_handle().clone();
        tauri::async_runtime::spawn(async move {
            let response = serve_image(&app, request).await.unwrap_or_else(|_| {
                Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .header("Content-Type", "image/svg+xml")
                    .body(PLACEHOLDER_SVG.as_bytes().to_vec())
                    .unwrap()
            });
            responder.respond(response);
        });
    })
}

async fn serve_image(
    app: &AppHandle,
    request: Request<Vec<u8>>,
) -> Result<Response<Vec<u8>>, AppError> {
    let state: tauri::State<AppState> = app.state();

    let query: HashMap<_, _> = Url::parse(&request.uri().to_string())
        .map_err(|_| AppError::Invalid("bad uri".into()))?
        .query_pairs().into_owned().collect();

    let src = query.get("u").ok_or_else(|| AppError::Invalid("missing u".into()))?;
    let url = Url::parse(src).map_err(|_| AppError::Invalid("bad image url".into()))?;

    // Only known image CDNs — this protocol must never become an open proxy.
    const IMG_HOSTS: &[&str] = &[
        "i.ytimg.com", "img.youtube.com",
        "yt3.ggpht.com", "yt4.ggpht.com", "yt3.googleusercontent.com",
        "lh3.googleusercontent.com",
    ];
    let host = url.host_str().unwrap_or_default();
    let allowed = IMG_HOSTS.contains(&host)
        || state.invidious.is_known_instance(host).await;
    if !allowed {
        return Err(AppError::Invalid("image host not permitted".into()));
    }

    let key = blake3::hash(src.as_bytes()).to_hex().to_string();

    // 1. Memory (LRU, 64 MiB)
    if let Some(hit) = state.image_cache.mem_get(&key).await {
        return Ok(image_response(hit.bytes, &hit.mime, "MEM"));
    }

    // 2. Disk
    if let Some(hit) = state.image_cache.disk_get(&key).await? {
        state.image_cache.mem_put(&key, &hit).await;
        return Ok(image_response(hit.bytes, &hit.mime, "DISK"));
    }

    // 3. Network — through the app's proxied client
    let client = state.clients.get(None).await?;
    let resp = client.get(url.clone())
        .header(REFERER, "https://www.youtube.com/")
        .timeout(Duration::from_secs(15))
        .send().await
        .map_err(|e| AppError::Network(sanitize_err(&e)))?
        .error_for_status()
        .map_err(|e| AppError::Network(sanitize_err(&e)))?;

    let mime = resp.headers().get(CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("image/jpeg")
        .to_string();

    if !mime.starts_with("image/") {
        return Err(AppError::Invalid("not an image".into()));
    }

    let bytes = read_capped(resp, 8 * 1024 * 1024).await?;
    state.image_cache.put(&key, &bytes, &mime).await?;

    Ok(image_response(bytes, &mime, "MISS"))
}

fn image_response(bytes: Vec<u8>, mime: &str, source: &str) -> Response<Vec<u8>> {
    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", mime)
        .header("Cache-Control", "public, max-age=604800, immutable")
        .header("X-Cache", source)
        .header("Access-Control-Allow-Origin", "*")
        .body(bytes)
        .unwrap()
}
```

### 5.3 Cache layout & eviction

```
<app_cache>/images/
├── index.db                 # sqlite: key, url_host, mime, bytes, created_at, last_hit_at, hits
├── 0a/0a3f…bin
├── 1b/1b7c…bin
└── …                        # two-char shard prefix keeps directories small
```

| Tier | Size | Eviction |
|---|---|---|
| Memory | 64 MiB | LRU |
| Disk | 512 MiB (configurable 128 MiB – 4 GiB) | LRU by `last_hit_at`, swept when over 90% capacity |
| TTL | 30 days for thumbnails, 7 days for avatars | Lazy on read + weekly sweep |

Sharding by the first two hex characters avoids the tens-of-thousands-of-files-in-one-directory pathology that makes `readdir` crawl on ext4 and NTFS alike.

### 5.4 Frontend usage

```ts
// src/lib/img.ts
export function cachedImage(url: string | undefined | null): string {
  if (!url) return '/placeholder.svg'
  if (url.startsWith('data:') || url.startsWith('slyimg://')) return url
  return `slyimg://localhost/?u=${encodeURIComponent(url)}`
}
```

```vue
<img :src="cachedImage(video.thumbnail)" loading="lazy" decoding="async" />
```

Native `loading="lazy"` handles viewport deferral, so no IntersectionObserver is needed; the protocol handler only ever sees images actually scrolled into view.

### 5.5 Prefetch

After a feed loads, up to 20 above-the-fold thumbnails are warmed concurrently (bounded by a semaphore of 6) so scrolling doesn't stutter. Prefetch is skipped on metered connections and disabled entirely in `enhanced` privacy mode, where speculative fetching would reveal content the user never actually viewed.

---

## 6. CORS Handling

### 6.1 Why it mostly disappears

`reqwest` is a plain HTTP client with no origin, so the same-origin policy simply does not apply to any request made in Rust. Routing every YouTube/Invidious call through `net_fetch` removes CORS from the picture for API traffic entirely.

CORS remains relevant in exactly three places:

1. Custom protocol responses (`slyimg://`, `potoken://`) consumed by the webview.
2. Media elements loading stream URLs directly.
3. The dev server during `vite dev`.

### 6.2 Custom protocol responses

Handlers set `Access-Control-Allow-Origin: *`. On Windows the WebView2 backend also requires the app origin (`http://tauri.localhost`) to be listed in `dangerousRemoteDomainIpcAccess`-adjacent config for protocol streaming to work — worth noting because the failure mode is a silent broken image rather than a console error.

```rust
fn cors_headers(builder: http::response::Builder) -> http::response::Builder {
    builder
        .header("Access-Control-Allow-Origin", "*")
        .header("Access-Control-Allow-Methods", "GET, OPTIONS")
        .header("Access-Control-Allow-Headers", "Range, Content-Type")
        .header("Access-Control-Expose-Headers", "Content-Length, Content-Range, Accept-Ranges")
}
```

`Access-Control-Expose-Headers` with `Content-Range` is required for `<video>` seeking to work over a custom protocol.

### 6.3 Media streams

`googlevideo.com` URLs return permissive CORS headers, so `<video src>` works directly. Two caveats:

- Those URLs are **IP-bound**. When a proxy is active, the webview fetching them directly would come from the wrong IP and get `403`. In proxied mode media is therefore streamed through a Rust range-proxying protocol (`slymedia://`) rather than assigned to `<video src>` directly.
- Range requests must be forwarded verbatim, and `206 Partial Content` with `Content-Range` returned, or seeking breaks.

```rust
async fn serve_media(app: &AppHandle, request: Request<Vec<u8>>) -> Result<Response<Vec<u8>>, AppError> {
    let range = request.headers().get(RANGE).and_then(|v| v.to_str().ok()).map(str::to_owned);
    let client = /* proxied client */;

    let mut rb = client.get(upstream_url);
    if let Some(r) = &range {
        rb = rb.header(RANGE, r);
    }
    let resp = rb.send().await.map_err(|e| AppError::Network(sanitize_err(&e)))?;

    let status = resp.status();          // 206 when a range was requested
    let content_range = resp.headers().get(CONTENT_RANGE).cloned();
    let body = resp.bytes().await.map_err(|e| AppError::Network(sanitize_err(&e)))?;

    let mut b = Response::builder()
        .status(status)
        .header("Accept-Ranges", "bytes")
        .header("Content-Type", "video/mp4");
    if let Some(cr) = content_range {
        b = b.header(CONTENT_RANGE, cr);
    }
    Ok(cors_headers(b).body(body.to_vec()).unwrap())
}
```

### 6.4 CSP

```json
// tauri.conf.json
{
  "app": {
    "security": {
      "csp": "default-src 'self'; \
              img-src 'self' slyimg: asset: data: blob:; \
              media-src 'self' slymedia: blob: https://*.googlevideo.com; \
              script-src 'self'; \
              style-src 'self' 'unsafe-inline'; \
              connect-src 'self' ipc: http://ipc.localhost; \
              frame-src 'none'; object-src 'none'",
      "assetProtocol": { "enable": true, "scope": ["$APPCACHE/images/**"] }
    }
  }
}
```

`connect-src` deliberately omits every remote origin: the renderer has no business making direct network calls, and the CSP enforces that architectural rule rather than relying on code review.

### 6.5 Dev-server note

Under `vite dev` the app is served from `http://localhost:1420`. Custom protocols still work, but the CSP is relaxed to allow HMR websockets. The relaxation is gated on `#[cfg(debug_assertions)]` so it can never ship in a release build.

---

## 7. Command & Event Index

| Command | Purpose |
|---|---|
| `net_fetch` | Proxied HTTP for YouTube / Invidious / generic targets |
| `network_test_proxy` | Validate a candidate proxy without saving it |
| `network_resolve_favicon` | Fetch and cache an instance favicon |
| `net_check_leak` | Compare direct vs. proxied egress IP |
| `net_clear_image_cache` | Purge memory + disk image tiers |
| `net_cache_stats` | Entry counts, byte sizes, hit ratio |

| Event | Payload |
|---|---|
| `network:proxy-changed` | `string \| null` (proxy URL, credentials stripped) |
| `network:offline` | `{ since: number }` |
| `network:online` | `{ downtimeMs: number }` |
| `network:rate-limited` | `{ host: string, retryAfterMs: number }` |

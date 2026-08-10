# Phase 03 — PoToken Generator ⚠️ HIGHEST RISK

| Field | Value |
|-------|-------|
| **Timeline** | Week 3 – Week 4 |
| **Duration** | 10 working days |
| **Risk Level** | 🔴 **Critical — highest-risk phase of the migration** |
| **Blocks** | Phase 04 (potoken commands), Phase 06 (player views) |
| **Depends On** | Phase 01 (capabilities, resources), Phase 02 (`potoken_cache` table) |

---

## 0. Why This Is The Highest-Risk Phase

The Electron implementation (`src/main/poTokenGenerator.js`, 219 LOC) relies on **Puppeteer driving a headless Chromium** — a full, controllable browser with CDP access. Tauri has **no Puppeteer, no CDP, and no Chromium** on macOS/Linux (WKWebView / WebKitGTK) and WebView2 on Windows. The entire execution substrate changes.

| Dimension | Electron (current) | Tauri (target) |
|-----------|--------------------|----------------|
| Engine | Bundled Chromium (uniform) | WebView2 / WKWebView / WebKitGTK (**3 different engines**) |
| Control | Puppeteer + CDP | `WebviewWindow::eval()` + `emit`/`listen` only |
| Result retrieval | `page.evaluate()` returns a value | Fire-and-forget eval → must post back over the event bus |
| Isolation | Separate browser context | Separate `WebviewWindow` + scoped capability |
| Detection surface | Known-good UA/fingerprint | Engine-dependent fingerprint; WebKit may fail BotGuard VM |

**Consequence:** we must design for *graceful degradation*, not guaranteed success. A three-tier fallback chain is mandatory.

---

## 1. Goals

1. Generate a valid **PoToken (Proof-of-Origin Token)** natively inside a hidden Tauri `WebviewWindow`, with no Puppeteer/Chromium dependency.
2. Establish a secure, minimal-capability custom protocol to serve the BotGuard harness locally (never from a remote origin).
3. Port `botGuardScript.js` (BotGuard VM bootstrap + integrity-token → PoToken minting) to the harness page.
4. Expose `get_potoken` / `clear_potoken_cache` commands backed by the SQLite cache with TTL.
5. Guarantee deterministic session cleanup — no leaked windows, timers, memory, or zombie webviews.
6. Ship a documented fallback chain so a PoToken failure degrades playback quality instead of breaking the app.

---

## 2. Background: What a PoToken Is

```
visitorData / visitor_id  ──┐
                            ├──► BotGuard challenge (obfuscated VM program from YouTube)
BotGuard VM (JS, from YT) ──┘
                │
                ▼
        integrityToken (webPoSignalOutput)
                │
                ▼
   mintCallback(identifier) ──► poToken   (bound to visitorData or videoId)
```

Two binding modes:

| Mode | Identifier | Used for |
|------|-----------|----------|
| **Session-bound** (`web.player`) | `visitorData` | Player requests, whole session — cache aggressively |
| **Content-bound** (`web.gvs`) | `videoId` | GoogleVideo stream URLs / downloads — per-video |

Both are minted from the same integrity token, so **one BotGuard session can mint many PoTokens**. This is the key optimization: amortize the expensive VM bootstrap.

---

## 3. Tasks

### 3.1 Hidden `WebviewWindow` Creation (Day 1–3)

```rust
// src-tauri/src/services/potoken/window.rs
use tauri::{AppHandle, WebviewUrl, WebviewWindowBuilder, Manager};

pub const POTOKEN_LABEL: &str = "potoken-generator";

pub fn spawn_hidden_webview(app: &AppHandle) -> Result<tauri::WebviewWindow, AppError> {
    if let Some(existing) = app.get_webview_window(POTOKEN_LABEL) {
        return Ok(existing); // reuse warm session
    }

    let win = WebviewWindowBuilder::new(
            app,
            POTOKEN_LABEL,
            WebviewUrl::CustomProtocol("potoken://harness/index.html".parse()?),
        )
        .title("")
        .visible(false)
        .decorations(false)
        .skip_taskbar(true)
        .focused(false)
        .resizable(false)
        .inner_size(1280.0, 720.0)   // realistic viewport — 0x0 trips heuristics
        .user_agent(REALISTIC_UA)
        .incognito(true)             // ephemeral storage partition
        .initialization_script(include_str!("../../../resources/potoken/bootstrap.js"))
        .build()?;

    Ok(win)
}

const REALISTIC_UA: &str =
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 \
     (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36";
```

**Design rules**

- [ ] **Never** create the window at app startup — spawn lazily on first `get_potoken` miss.
- [ ] Viewport must be non-degenerate (1280×720). Zero-size or 1×1 windows are a known bot signal.
- [ ] `visible(false)` + `skip_taskbar(true)` + `focused(false)` — must never steal focus or appear in Alt-Tab.
- [ ] `incognito(true)` isolates cookies/storage from the main window.
- [ ] Use a **realistic desktop Chrome UA** on all platforms; WebKit's default UA is a strong fingerprint.
- [ ] Window is labelled `potoken-generator` so `capabilities/potoken.json` (Phase 01) applies.
- [ ] Platform quirk matrix must be validated:

| Platform | Engine | Known quirk | Handling |
|----------|--------|-------------|----------|
| Windows | WebView2 (Chromium) | Best compatibility | Reference implementation |
| macOS | WKWebView | Stricter JIT/eval policy; `eval()` on cross-origin blocked | Harness must be same-origin via custom protocol |
| Linux | WebKitGTK 4.1 | Slowest VM exec; occasional `Function` constructor limits | Longer timeout (45 s); early fallback |

### 3.2 Custom Protocol Setup (Day 3–4)

The BotGuard harness must be served from a **stable, same-origin, local** URL so `fetch`, `eval`, and module loading behave consistently and CSP is enforceable.

```rust
// lib.rs
builder.register_asynchronous_uri_scheme_protocol("potoken", move |_app, req, responder| {
    let path = req.uri().path().trim_start_matches('/');
    let body: Option<(&[u8], &str)> = match path {
        "harness/index.html" => Some((HARNESS_HTML, "text/html; charset=utf-8")),
        "harness/botguard.js" => Some((BOTGUARD_JS, "text/javascript; charset=utf-8")),
        "harness/runner.js"   => Some((RUNNER_JS,   "text/javascript; charset=utf-8")),
        _ => None,
    };

    let resp = match body {
        Some((bytes, mime)) => http::Response::builder()
            .status(200)
            .header("Content-Type", mime)
            .header("Cache-Control", "no-store")
            .header(
                "Content-Security-Policy",
                "default-src 'none'; \
                 script-src 'self' potoken://harness https://www.google.com https://www.youtube.com; \
                 connect-src https://www.youtube.com https://youtubei.googleapis.com https://jnn-pa.googleapis.com; \
                 img-src data:; style-src 'unsafe-inline'",
            )
            .body(bytes.to_vec()),
        None => http::Response::builder().status(404).body(Vec::new()),
    };
    responder.respond(resp.unwrap());
});

static HARNESS_HTML: &[u8] = include_bytes!("../resources/potoken/index.html");
static BOTGUARD_JS: &[u8]  = include_bytes!("../resources/potoken/botGuardScript.js");
static RUNNER_JS: &[u8]    = include_bytes!("../resources/potoken/runner.js");
```

**Checklist**

- [ ] Assets embedded via `include_bytes!` — no filesystem reads, no tampering surface.
- [ ] Scheme registered **before** any window using it is built.
- [ ] Windows maps custom schemes to `http://potoken.localhost/...`; harness code must not hardcode the scheme.
- [ ] `connect-src` allowlists exactly the three Google endpoints BotGuard needs; nothing else.
- [ ] Add `"potoken"` to `app.security.dangerousRemoteDomainIpcAccess`? — **No.** IPC to the harness goes through events only, never raw command access.

### 3.3 `botGuardScript.js` Integration (Day 4–7)

**Harness page** (`resources/potoken/index.html`):

```html
<!doctype html>
<html><head><meta charset="utf-8"><title></title></head>
<body>
  <script src="botguard.js"></script>
  <script src="runner.js" type="module"></script>
</body></html>
```

**Runner** (`resources/potoken/runner.js`) — orchestrates the three-step mint:

```js
import { emit, listen } from './tauri-event-shim.js';

const REQUEST_KEY = 'O43z0dpjhgX20SCx4KAo';   // YouTube's public BotGuard request key

async function createSession(requestKey) {
  // 1. Fetch the BotGuard challenge program
  const res = await fetch('https://jnn-pa.googleapis.com/$rpc/google.internal.waa.v1.Waa/Create', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json+protobuf', 'x-goog-api-key': API_KEY, 'x-user-agent': UA },
    body: JSON.stringify([requestKey]),
  });
  const challenge = parseChallenge(await res.json());

  // 2. Bootstrap the BotGuard VM (botGuardScript.js defines globalThis.trayride / vm loader)
  const vm = await loadBotGuardVm(challenge);         // evaluates the obfuscated program
  const botguardResponse = await vm.snapshot({ webPoSignalOutput: (out) => (signalOutput = out) });

  // 3. Exchange the BotGuard response for an integrity token
  const it = await fetch('https://jnn-pa.googleapis.com/$rpc/google.internal.waa.v1.Waa/GenerateIT', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json+protobuf', 'x-goog-api-key': API_KEY },
    body: JSON.stringify([requestKey, botguardResponse]),
  });
  const [integrityToken, ttlSeconds] = await it.json();
  return { integrityToken, ttlSeconds, signalOutput };
}

async function mint(identifier, session) {
  const mintFn = await session.signalOutput[0](
    Uint8Array.from(atob(session.integrityToken), c => c.charCodeAt(0))
  );
  const raw = await mintFn(new TextEncoder().encode(identifier));
  return base64UrlEncode(raw);
}

// Rust drives the harness via events
listen('potoken:generate', async ({ payload }) => {
  const { requestId, identifier, mode } = payload;   // mode: 'session' | 'content'
  try {
    session ??= await createSession(REQUEST_KEY);
    const token = await mint(identifier, session);
    emit('potoken:result', { requestId, ok: true, token, ttl: session.ttlSeconds, mode });
  } catch (e) {
    emit('potoken:result', { requestId, ok: false, error: String(e?.message ?? e), stage: currentStage });
  }
});

emit('potoken:ready', { engine: navigator.userAgent });
```

**Porting checklist**

- [ ] Vendor `botGuardScript.js` from the OpenTubeX source **verbatim**; record its upstream SHA-256 in `docs/decisions`.
- [ ] Remove all Puppeteer/Node-isms (`window.__puppeteer*`, `page.exposeFunction` shims) — replace with the Tauri event shim.
- [ ] Replace `page.evaluate(() => …)` return-value plumbing with `emit('potoken:result', …)`.
- [ ] Instrument each stage (`fetch_challenge` → `vm_bootstrap` → `snapshot` → `generate_it` → `mint`) and report `stage` on failure — essential for diagnosing engine-specific breakage.
- [ ] Never log token material at `info`; redact to first 8 chars in debug logs.
- [ ] Guard against harness hangs: `Promise.race` with an internal 30 s timeout that emits a structured failure.

### 3.4 Tauri Command Implementation (Day 7–8)

```rust
#[derive(Serialize)]
pub struct PoTokenResponse {
    pub token: String,
    pub cached: bool,
    pub expires_at: i64,
    pub mode: PoTokenMode,   // Session | Content
    pub source: PoTokenSource, // Webview | Cache | Fallback
}

#[tauri::command]
pub async fn get_potoken(
    app: AppHandle,
    video_id: Option<String>,     // None → session-bound token
    force_refresh: bool,
    state: State<'_, AppState>,
) -> Result<PoTokenResponse, AppError> {
    let identifier = match &video_id {
        Some(v) => v.clone(),
        None => state.visitor_data().await?,
    };

    // 1) Cache
    if !force_refresh {
        if let Some(hit) = PoTokenRepo(&state.pool).get_valid(&identifier).await? {
            return Ok(hit.into_response(true));
        }
    }

    // 2) Single-flight: coalesce concurrent requests for the same identifier
    let permit = state.potoken.single_flight(&identifier).await;

    // 3) Generate with retries + timeout
    let result = state.potoken.generate(&app, &identifier, mode).await;

    match result {
        Ok(tok) => {
            PoTokenRepo(&state.pool).put(&identifier, &tok).await?;
            drop(permit);
            Ok(tok.into_response(false))
        }
        Err(e) => {
            drop(permit);
            state.potoken.record_failure(&e).await;
            Err(AppError::PoToken(e.to_string()))
        }
    }
}

#[tauri::command]
pub async fn clear_potoken_cache(state: State<'_, AppState>) -> Result<u32, AppError> {
    PoTokenRepo(&state.pool).prune_all().await
}

#[tauri::command]
pub async fn potoken_diagnostics(state: State<'_, AppState>) -> Result<PoTokenDiagnostics, AppError> {
    // engine string, last success/failure, failure stage histogram, cache hit rate,
    // circuit-breaker state — surfaced in Settings → Advanced (Phase 06)
}
```

**Generation orchestration**

```rust
async fn generate(&self, app: &AppHandle, identifier: &str, mode: PoTokenMode)
    -> Result<PoToken, PoTokenError>
{
    const ATTEMPTS: u32 = 3;
    for attempt in 1..=ATTEMPTS {
        let win = spawn_hidden_webview(app)?;
        self.await_ready(&win, Duration::from_secs(15)).await?;

        let request_id = Uuid::new_v4().to_string();
        let (tx, rx) = oneshot::channel();
        self.pending.insert(request_id.clone(), tx);

        win.emit("potoken:generate", json!({
            "requestId": request_id, "identifier": identifier, "mode": mode
        }))?;

        match timeout(self.timeout_for_platform(), rx).await {
            Ok(Ok(Ok(tok))) => return Ok(tok),
            Ok(Ok(Err(e)))  => log::warn!("potoken attempt {attempt} failed at {}: {}", e.stage, e.message),
            Ok(Err(_)) | Err(_) => log::warn!("potoken attempt {attempt} timed out"),
        }

        self.pending.remove(&request_id);
        self.destroy_session(app).await;                       // cold restart on failure
        sleep(Duration::from_millis(750 * 2u64.pow(attempt))).await; // backoff
    }
    Err(PoTokenError::exhausted(ATTEMPTS))
}
```

**Fallback chain (mandatory)**

| Tier | Strategy | When |
|------|----------|------|
| 1 | Hidden webview generation | Default |
| 2 | Cached session token reuse (mint content tokens offline from a live integrity token) | Webview failed but session still valid |
| 3 | External provider (`bgutil-ytdlp-pot-provider` HTTP endpoint), opt-in | User-configured in Settings |
| 4 | Degrade: Invidious API / `--extractor-args player-client=ios,tv` paths without PoToken | All above failed |

- [ ] A **circuit breaker** opens after 5 consecutive failures within 10 min; skips straight to tier 4 for 30 min and surfaces a non-blocking UI notice.
- [ ] Tier 4 must keep playback and downloads *working* at reduced format availability — never a hard error.

### 3.5 Session Cleanup (Day 8–9)

Leaks here are severe: an orphaned webview holds a full engine process (~80–150 MB).

```rust
pub struct PoTokenSession {
    window_label: String,
    created_at: Instant,
    integrity_expires_at: Instant,
    mint_count: u32,
}

impl PoTokenService {
    pub async fn destroy_session(&self, app: &AppHandle) {
        if let Some(w) = app.get_webview_window(POTOKEN_LABEL) {
            let _ = w.eval("window.__potokenTeardown && window.__potokenTeardown()");
            tokio::time::sleep(Duration::from_millis(50)).await;
            let _ = w.close();               // graceful
            tokio::time::sleep(Duration::from_millis(250)).await;
            if app.get_webview_window(POTOKEN_LABEL).is_some() {
                let _ = w.destroy();          // forced
            }
        }
        self.pending.clear();
        *self.session.write().await = None;
    }
}
```

**Cleanup triggers**

| Trigger | Action |
|---------|--------|
| Integrity token TTL expiry (typically ~12 h) | Destroy + lazy re-create on next miss |
| Idle > 10 min with no mint requests | Destroy (free memory) |
| 3 consecutive generation failures | Destroy + backoff |
| Main window `CloseRequested` / app exit | Destroy synchronously before pool close |
| `RunEvent::Exit` | Final sweep: any window labelled `potoken-*` destroyed |
| Explicit `clear_potoken_cache` | Destroy + purge SQLite cache |
| System suspend/resume (where observable) | Destroy — post-resume VMs are unreliable |

**Verification requirements**

- [ ] `assert!(app.webview_windows().len() == 1)` after any generation cycle completes.
- [ ] `potoken:teardown` handler in the harness clears all timers, aborts in-flight `fetch`es via `AbortController`, and nulls VM references.
- [ ] Memory profile: 50 sequential generations must not grow RSS by more than 25 MB (validated in Phase 08).
- [ ] No `potoken-generator` window may ever be visible, focusable, or present in the taskbar/Dock — automated assertion in E2E tests.

### 3.6 Caching Strategy (Day 9)

`potoken_cache` (from Phase 02, migration `004`):

```sql
CREATE TABLE potoken_cache (
    identifier   TEXT PRIMARY KEY,      -- videoId | visitorData
    mode         TEXT NOT NULL CHECK (mode IN ('session','content')),
    token        TEXT NOT NULL,
    visitor_data TEXT,
    created_at   INTEGER NOT NULL DEFAULT (strftime('%s','now')),
    expires_at   INTEGER NOT NULL,
    hit_count    INTEGER NOT NULL DEFAULT 0,
    engine       TEXT                    -- webview engine string, for diagnostics
);
CREATE INDEX idx_potoken_expiry ON potoken_cache(expires_at);
```

| Mode | TTL | Rationale |
|------|-----|-----------|
| Session (`web.player`) | min(integrity TTL, 6 h) | Reused across all playback |
| Content (`web.gvs`) | 2 h | Bound to a stream URL's lifetime |

- [ ] Prune expired rows on startup and hourly.
- [ ] Cap table at 500 rows (LRU by `hit_count`, then `created_at`).
- [ ] Cache **misses must not stampede**: single-flight per identifier (see 3.4).

### 3.7 Test Harness & Diagnostics (Day 10)

- [ ] `cargo test --features potoken-live` — network-gated live generation test (excluded from default CI).
- [ ] Dev-only "PoToken Lab" panel: shows engine, last stage, timings per stage, cache table, manual force-refresh.
- [ ] Structured metrics logged per attempt: `{ engine, stage, duration_ms, outcome }`.
- [ ] Nightly canary workflow on all 3 OSes that generates a token and opens an issue on failure (early warning for YouTube-side changes).

---

## 4. Deliverables

| ID | Deliverable | Acceptance Criteria |
|----|-------------|---------------------|
| D3.1 | Hidden webview lifecycle module | Lazily spawned, never visible, reused while warm |
| D3.2 | `potoken://` custom protocol | Serves 3 embedded assets with strict CSP; works on all 3 engines |
| D3.3 | Ported `botGuardScript.js` harness | Mints a valid PoToken on Windows + macOS; documented status on Linux |
| D3.4 | `get_potoken`, `clear_potoken_cache`, `potoken_diagnostics` | Typed, cached, single-flighted, retried |
| D3.5 | 4-tier fallback chain + circuit breaker | App remains functional with PoToken fully disabled |
| D3.6 | Deterministic cleanup | 0 leaked windows; ≤25 MB RSS growth over 50 cycles |
| D3.7 | Diagnostics + canary CI | Stage-level failure attribution available in-app |
| D3.8 | ADR in `docs/decisions` | Records engine limitations, vendored script hash, fallback policy |

---

## 5. Dependencies

**Inbound**

| From | Needs |
|------|-------|
| Phase 01 | `capabilities/potoken.json`, resource embedding, CSP posture |
| Phase 02 | `potoken_cache` table + `PoTokenRepo`, `AppError::PoToken` |

**Outbound**

| Phase | Consumes |
|-------|----------|
| 04 | `get_potoken` for `get_video_details` and `start_download(use_potoken)` |
| 06 | Diagnostics panel in Settings; degraded-mode UI banner |
| 08 | Memory-leak assertions, cross-platform matrix results |

**External / uncontrollable:** YouTube BotGuard program changes, `jnn-pa.googleapis.com` API shape, per-engine JS capabilities.

---

## 6. Risks

| ID | Risk | Prob. | Impact | Mitigation |
|----|------|-------|--------|------------|
| R3.1 | **BotGuard VM fails to execute under WKWebView/WebKitGTK** | **High** | **Critical** | Same-origin custom protocol (avoids eval restrictions); per-platform timeouts; tier-3 external provider; tier-4 degrade. Spike this on day 1–2 before committing to the design |
| R3.2 | YouTube changes BotGuard program/endpoints, breaking the harness | High | Critical | Nightly canary CI; vendored script pinned + hashed; hot-swappable harness via updater; documented manual override |
| R3.3 | Hidden webview flashes visible on Linux/Windows | Medium | Medium | Build with `visible(false)` from the start (never show-then-hide); verify with automated screenshot diff |
| R3.4 | Webview leak → memory exhaustion | Medium | High | Explicit destroy on 7 triggers; idle reaper; RSS regression test |
| R3.5 | YouTube detects the automated fingerprint and hard-blocks | Medium | High | Realistic UA + viewport; no CDP artifacts; jittered timing; incognito partition; monitor error-rate telemetry (opt-in) |
| R3.6 | Concurrent requests stampede the VM bootstrap | Medium | Medium | Single-flight keyed by identifier; global mutex on session creation |
| R3.7 | Token leakage into logs/telemetry/crash dumps | Low | High | Redact in `Debug` impls; `#[serde(skip)]` on log paths; never persist to `store` plugin |
| R3.8 | Custom protocol URL differs on Windows (`http://potoken.localhost`) | High | Low | Never hardcode the origin; derive from `location.origin` |
| R3.9 | Phase overruns and blocks Phase 04 | Medium | High | **Timebox to 10 days.** If tier-1 is not working on ≥2 platforms by day 7, ship tiers 2–4 and file tier-1 as a follow-up; Phase 04 proceeds against the command contract, not the implementation |
| R3.10 | Legal/ToS concerns around BotGuard execution | Low | Medium | Document intent (interoperability, no ad-circumvention claims) in ADR; keep feature user-toggleable and default-on-demand |

**Contingency plan (if tier 1 proves infeasible):** ship with tiers 2–4 only. The app remains fully usable; age-restricted and some high-bitrate formats become unavailable, matching the behavior of other Invidious-backed clients. This must be an explicit, documented product decision — not a silent regression.

---

## 7. Estimated Duration

| Task | Days | Notes |
|------|------|-------|
| 3.0 Feasibility spike (WKWebView + WebKitGTK) | 1.5 | **Gate: go/no-go on tier 1** |
| 3.1 Hidden webview lifecycle | 1.5 | |
| 3.2 Custom protocol | 1.0 | |
| 3.3 botGuardScript port | 2.5 | Highest uncertainty |
| 3.4 Commands + retries + fallback | 1.5 | |
| 3.5 Session cleanup | 1.0 | |
| 3.6 Caching | 0.5 | |
| 3.7 Diagnostics + canary | 0.5 | |
| **Total** | **10.0** | 2 weeks @ 1 dev, **+50 % buffer recommended** |

---

## 8. Exit Criteria

- [ ] `get_potoken` returns a valid token on **at least Windows and macOS**; Linux status explicitly documented (pass or fallback).
- [ ] Token verified end-to-end: a `web.gvs` token yields a working GoogleVideo stream URL; a `web.player` token yields a successful `/youtubei/v1/player` response.
- [ ] Cache hit path returns in <5 ms; cold generation completes in <20 s (Win/macOS) / <45 s (Linux).
- [ ] 50-cycle soak: 0 leaked windows, RSS growth ≤25 MB.
- [ ] Disabling PoToken entirely (tier 4) keeps playback + downloads functional.
- [ ] ADR merged in `docs/decisions` recording engine matrix, vendored hash, and fallback policy.

---

## 9. References

- [Architecture — PoToken Generation Flow](../architecture/03-data-flow.md#potoken-generation-flow)
- [Architecture — Webview Control](../architecture/01-electron-vs-tauri.md)
- [Backend — PoToken Commands](../backend/02-tauri-commands.md#potoken-commands)
- Previous: [Phase 02 — Database & yt-dlp](02-database-yt-dlp.md) · Next: [Phase 04 — Backend Commands](04-backend-commands.md)

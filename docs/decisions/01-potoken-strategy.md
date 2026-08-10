# ADR 001: PoToken Generation Strategy

| Field | Value |
|-------|-------|
| **Status** | Accepted |
| **Date** | 2026-08-09 |
| **Deciders** | Migration Team |
| **Supersedes** | — |
| **Related** | [02-invidious-location.md](02-invidious-location.md), [05-migration-approach.md](05-migration-approach.md) |

---

## Context

YouTube's InnerTube API increasingly requires a **Proof of Origin Token (PoToken)** to serve
playable stream URLs. Without a valid PoToken, requests are downgraded, throttled, or rejected
outright with `403` / "content unavailable" responses on a growing share of videos.

PoToken is produced by **botGuard** — an obfuscated, self-modifying JavaScript VM shipped by
Google. It cannot be reimplemented statically; it must be **executed** in an environment that
presents a browser-like surface (DOM, `navigator`, `window`, timers, WebGL/canvas fingerprint
targets). It also must run in an **isolated session** so that:

- The visitor data / session identity used to mint the token matches the identity used to
  request the streams.
- botGuard's globals never leak into (or get polluted by) the main application renderer.
- The generator can be torn down and re-created on token expiry (tokens are short-lived,
  typically minutes to hours) without disturbing app state.

**Current (Electron) implementation:** `src/main/poTokenGenerator.js` (~219 lines) spawns a
headless Chromium instance via Puppeteer, loads a minimal page, evaluates the botGuard bundle,
and returns the minted token to the main process over IPC.

Tauri v2 has **no Node.js runtime and no Puppeteer**. The bundled Chromium that Electron relies
on does not exist — Tauri uses the OS system webview (WebView2 on Windows, WKWebView on macOS,
WebKitGTK on Linux). Therefore the PoToken subsystem must be redesigned from first principles.

### Constraints

| Constraint | Detail |
|-----------|--------|
| No Node.js | Puppeteer / `child_process` Chromium control is unavailable |
| No bundled browser | Shipping a second Chromium would defeat the ~10 MB binary goal |
| Session isolation | Token identity must be bound to a dedicated, disposable session |
| Cross-platform | Must work on WebView2, WKWebView and WebKitGTK |
| Obfuscated payload | botGuard is minified/obfuscated and changes without notice |

---

## Options Considered

### Option A — Hidden Tauri Webview

Create a dedicated `WebviewWindow` with `.visible(false)`, served from a custom protocol
(`potoken://`), which loads a minimal HTML shell plus the botGuard-js bundle. Rust drives it via
`webview.eval()` and receives the minted token back through a Tauri command or event.

```rust
// src-tauri/src/potoken/generator.rs (sketch)
let webview = WebviewWindowBuilder::new(
        &app,
        "potoken-generator",
        WebviewUrl::App("potoken/index.html".into()),
    )
    .visible(false)
    .decorations(false)
    .skip_taskbar(true)
    .inner_size(400.0, 300.0)
    .user_agent(CHROME_UA)
    .build()?;

webview.eval(include_str!("../../potoken/bootstrap.js"))?;
// bootstrap.js -> runs botGuard -> invoke('potoken_ready', { token, visitor_data })
```

| Pros | Cons |
|------|------|
| Uses the OS webview already shipped — zero extra binary weight | System webview engines differ (Blink vs WebKit) — fingerprint variance |
| Real DOM/`navigator`/canvas surface — botGuard runs unmodified | Requires window lifecycle management and cleanup discipline |
| Closest 1:1 mapping to the existing Electron design | Hidden windows can be flagged by some window managers / a11y tooling |
| Tauri 2 first-class support for `eval()`, custom protocols, per-window UA | Debugging a headless window is awkward without devtools builds |
| Session can be destroyed and recreated cheaply on expiry | Small startup cost per generation cycle (~200–500 ms) |

### Option B — WASM Port of botGuard

Compile or reimplement the botGuard VM to WebAssembly and execute it inside a Rust WASM runtime
(`wasmtime` / `wasmer`) with a shimmed DOM.

| Pros | Cons |
|------|------|
| No webview dependency; fully deterministic, testable in CI | botGuard is obfuscated JS, **not** compilable to WASM without a full JS engine |
| Runs entirely in the Rust process, no window lifecycle | Would require embedding a JS engine (`boa`, `quickjs`) **plus** a fake DOM |
| Fast once warm | The DOM shim is the actual product — hundreds of APIs, all fingerprintable |
| | Google changes botGuard frequently; every change may break the shim |
| | High probability of detection: a shimmed environment is trivially distinguishable |
| | Highest effort of all options by a wide margin |

### Option C — Keep PoToken Generation in the Renderer

Run botGuard inside the main application webview (the same window the user sees), reusing the
existing renderer-side helpers.

| Pros | Cons |
|------|------|
| Zero new infrastructure — smallest diff | **No session isolation** — botGuard globals pollute app scope |
| Reuses existing JS as-is | Token identity is tangled with the user-facing session and cookies |
| No IPC round trip | botGuard is self-modifying; it can and does patch `window` prototypes |
| | Cannot be torn down/recreated without reloading the whole app |
| | Blocks the UI thread during generation (janky playback start) |
| | A botGuard crash takes the entire app down |

---

## Decision

**Adopt Option A — Hidden Tauri Webview.**

PoToken generation will be implemented as a dedicated, invisible `WebviewWindow` managed
entirely from Rust, served over a custom `potoken://` protocol, with an explicit lifecycle
(create → warm → mint → cache → destroy).

### Target Shape

```
┌──────────────────────────────────────────────────────────────┐
│ Rust: PoTokenService (src-tauri/src/potoken/)                │
│  ├── generator.rs   create/destroy hidden WebviewWindow      │
│  ├── protocol.rs    register `potoken://` custom protocol    │
│  ├── cache.rs       token + visitor_data w/ TTL & refresh    │
│  └── commands.rs    #[tauri::command] potoken_ready(...)     │
└───────────────┬──────────────────────────────────────────────┘
                │ eval() bootstrap.js        ▲ invoke('potoken_ready')
                ▼                            │
┌──────────────────────────────────────────────────────────────┐
│ Hidden WebviewWindow "potoken-generator" (visible = false)   │
│  potoken://index.html  +  botGuard-js bundle                 │
└──────────────────────────────────────────────────────────────┘
```

---

## Rationale

1. **Closest to the current architecture.** The Electron implementation already executes
   botGuard in a separate browser context driven by the privileged process. A hidden webview is
   the direct Tauri analogue, so the JS payload and the control flow port with minimal change.
   This keeps the riskiest, least-understood component of the migration as close to a known-good
   design as possible.

2. **Tauri 2 supports the required primitives natively.**
   - `WebviewWindowBuilder::visible(false)` — genuinely offscreen, no user-visible artefact.
   - `webview.eval(js)` — inject the bootstrap and botGuard bundle from Rust.
   - `register_uri_scheme_protocol` — serve the shell over `potoken://` so the payload is
     embedded in the binary rather than fetched from disk or the network.
   - Per-window `user_agent()` — the generator can present a stable, realistic UA independent
     of the main window.
   - `WebviewWindow::destroy()` — deterministic teardown of the session.

3. **Real browser surface, zero extra bytes.** botGuard's whole purpose is to detect synthetic
   environments. The OS webview is a real browser engine that we are already shipping against;
   it gives us the highest-fidelity execution surface at literally no additional binary cost.
   Option B inverts this: maximum effort for the lowest-fidelity surface.

4. **Isolation is a hard requirement, not a nice-to-have.** botGuard mutates global prototypes
   and installs timers. Option C would let that bleed into the Vue application, producing
   heisenbugs that are effectively undebuggable. A separate webview gives us a clean, disposable
   blast radius.

5. **Failure containment.** If the generator hangs or crashes, Rust can kill the window, fall
   back to the Invidious path (see ADR 002), and retry with backoff — all without the user
   noticing anything beyond a slower first play.

---

## Implications

### Must build

- [ ] `src-tauri/src/potoken/` module: generator, protocol, cache, commands.
- [ ] Hidden webview lifecycle manager with **create → mint → destroy** semantics and a hard
      timeout (recommend 15 s) after which the window is force-destroyed.
- [ ] Custom `potoken://` URI scheme registered at app setup, serving an embedded minimal HTML
      shell plus the botGuard bundle (`include_bytes!`) — nothing loaded from the network.
- [ ] `potoken_ready` command for the webview → Rust handoff, guarded so only the generator
      window's label may call it.
- [ ] Token cache with TTL, proactive refresh before expiry, and single-flight de-duplication so
      concurrent playback requests do not spawn N generators.
- [ ] Capability file scoped to the `potoken-generator` window granting **only** the
      `potoken_ready` command — it must not inherit main-window permissions.

### Operational consequences

| Area | Consequence |
|------|-------------|
| **Session cleanup** | The generator window **must** be destroyed after each mint cycle (or on TTL expiry). Leaked windows accumulate memory and, worse, stale session identity. Add a shutdown hook on app exit. |
| **Cold start latency** | First playback pays ~200–500 ms for window creation + botGuard execution. Mitigate by warming the generator during app startup, after the main window paints. |
| **Cross-platform variance** | WebKitGTK and WKWebView produce different fingerprints than WebView2. Requires explicit testing on all three; expect per-platform UA tuning. |
| **Fragility to upstream change** | botGuard is updated without notice. Treat the bundle as a vendored, version-pinned asset with a documented refresh procedure. |
| **Fallback path required** | Every PoToken failure must degrade gracefully to the Invidious API path rather than surfacing an error. This couples ADR 001 to ADR 002. |
| **Testing** | Cannot be unit-tested meaningfully. Needs an integration test that boots the app, mints a token, and asserts a real stream URL resolves. |
| **Security review** | `eval()` of a third-party obfuscated bundle inside a webview is a real attack surface. It is acceptable **only** because the window is isolated, has no filesystem/shell capabilities, and loads no remote code. |

### Rejected-option debt

Option B (WASM) is not permanently off the table but is deferred indefinitely. Should Google
begin actively detecting and blocking system webviews, revisit — but the realistic escalation
path is UA/fingerprint tuning within Option A, not a rewrite to Option B.

---

## References

- Electron baseline: `src/main/poTokenGenerator.js` (219 lines)
- [Tauri v2 — Webview windows](https://v2.tauri.app/reference/javascript/api/namespacewebviewwindow/)
- [Tauri v2 — Custom URI schemes](https://v2.tauri.app/develop/)
- [../architecture/01-electron-vs-tauri.md](../architecture/01-electron-vs-tauri.md) §"PoToken Generator"

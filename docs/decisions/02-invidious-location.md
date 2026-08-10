# ADR 002: Location of the Invidious API Client

| Field | Value |
|-------|-------|
| **Status** | Accepted |
| **Date** | 2026-08-09 |
| **Deciders** | Migration Team |
| **Supersedes** | — |
| **Related** | [01-potoken-strategy.md](01-potoken-strategy.md), [05-migration-approach.md](05-migration-approach.md) |

---

## Context

Slytube resolves video metadata and stream URLs through **two independent paths**:

1. **Local / InnerTube** — `youtubei.js` talking directly to YouTube's private API
   (`src/renderer/helpers/api/local.js`, ~2573 lines).
2. **Invidious** — a community-run proxy API
   (`src/renderer/helpers/api/invidious.js`, ~1009 lines).

Both currently live in the **renderer**. The Invidious client is not a thin HTTP wrapper; it
carries a meaningful amount of behaviour:

| Responsibility | Detail |
|---------------|--------|
| **Instance selection** | User-configurable base URL, plus health/latency awareness |
| **Auth headers** | Optional token / cookie auth for private or rate-limited instances |
| **Proxy rewriting** | Rewrites `videoplayback`, thumbnail, and avatar URLs to route through the chosen instance so the client never contacts `googlevideo.com` directly |
| **Response normalisation** | Maps Invidious JSON into the same internal shape the Watch view consumes from `local.js` |
| **Fallback orchestration** | The Watch view attempts local → Invidious (or the reverse, per settings) and merges partial results |

The critical detail is the **fallback logic is tightly coupled into the Watch view**. It is not a
clean "try A, else B" at the API boundary — the view inspects which fields came back populated,
degrades format selection, swaps the player source, and reconciles caption/storyboard
availability across the two providers.

The migration question: does this client move to Rust alongside the other backend services, or
stay in the renderer?

---

## Options Considered

### Option A — Keep the Invidious client in the Renderer

Port `invidious.js` to TypeScript largely as-is, keep it in `src/lib/api/`, and let it issue
requests via `fetch` from the webview.

| Pros | Cons |
|------|------|
| Smallest possible diff — the highest-risk view (Watch) is barely touched | HTTP requests originate from the webview, subject to CORS and CSP |
| Fallback logic stays adjacent to the code that consumes it | Auth tokens live in renderer memory rather than Rust-owned state |
| `youtubei.js` interop is preserved verbatim (same runtime, same objects) | No Rust-side caching/coalescing of Invidious responses |
| Zero serialisation cost across the IPC boundary for large metadata payloads | Cannot reuse Rust's connection pool / retry middleware |
| Instance switching is a reactive store update, no backend round trip | Proxy rewriting happens client-side; slightly more work per render |

### Option B — Move the Invidious client to Rust

Reimplement as a `reqwest`-based service in `src-tauri/src/services/invidious.rs`, exposed via
`#[tauri::command]`.

| Pros | Cons |
|------|------|
| No CORS/CSP constraints — Rust makes arbitrary HTTP requests | **Splits the fallback logic across the IPC boundary** |
| Auth headers and instance credentials never enter the webview | `youtubei.js` stays in JS, so the two providers now live in different runtimes |
| Central retry/backoff/timeout policy shared with other services | Every Watch-view fallback decision becomes an async `invoke` round trip |
| Response caching and request coalescing in one place | Large metadata payloads pay serde serialisation both ways |
| Typed models via `serde` | Requires reimplementing ~1000 lines of normalisation in Rust up front |
| | Highest risk of subtle behavioural drift in the app's most-used screen |

---

## Decision

**Adopt Option A — keep the Invidious client in the Renderer.**

`invidious.js` is ported to TypeScript under `src/lib/api/invidious.ts`, retaining its current
responsibilities: instance selection, auth header injection, proxy URL rewriting, and response
normalisation. The Watch view's dual-provider fallback logic is preserved in place.

This is explicitly scoped as a **migration-time** decision, not a permanent architectural
boundary. See *Revisit Criteria* below.

---

## Rationale

1. **The fallback logic is tightly coupled in the Watch view.** Splitting the two providers
   across a process boundary would mean the Watch view orchestrates a JS-resident
   `youtubei.js` and a Rust-resident Invidious client, interleaving synchronous object access
   with async `invoke` calls. That is strictly worse than the status quo and would have to be
   untangled *during* the migration — exactly when we have the least ability to verify
   correctness.

2. **Minimises rewrite risk on the highest-traffic surface.** Watch is the screen users spend
   ~95% of their time on and the one with the most edge cases (age-gated, live, premiere,
   region-blocked, members-only, DRM). Every one of those paths currently threads through the
   local↔Invidious fallback. Keeping it in one runtime means we can port it mechanically and
   diff behaviour, rather than redesigning it.

3. **`youtubei.js` fallback stays intact.** We are not porting `youtubei.js` to Rust in this
   migration (no mature equivalent exists, and PoToken integration per ADR 001 is already
   webview-resident). Given the local path must remain in JS, co-locating the Invidious path in
   JS keeps both halves of the fallback in the same runtime with the same object shapes.

4. **Invidious is genuinely a fetch-based client.** Unlike yt-dlp (process spawning), the
   database (filesystem), or sync crypto (CPU-bound), Invidious is plain HTTP + JSON mapping.
   There is no capability the renderer lacks. Moving it to Rust buys architectural tidiness, not
   function.

5. **Proportionality.** ADR 005 commits to a Big Bang migration. That budget is best spent on
   the components that *must* move (yt-dlp, DB, crypto, PoToken). Rewriting a working HTTP
   client is discretionary work that competes with mandatory work.

---

## Implications

### Renderer responsibilities

- [ ] `src/lib/api/invidious.ts` — ported client with typed responses.
- [ ] Instance configuration surfaced through a Pinia store, persisted via a settings command to
      Rust (settings persistence stays in Rust; the *value* is read by the renderer).
- [ ] **Auth handling in the renderer.** Instance tokens are held in renderer memory and
      attached as request headers. They are read once from Rust-owned settings on demand and
      should not be mirrored into `localStorage`.
- [ ] **Proxy rewriting in the renderer.** `videoplayback`, thumbnail, and avatar URLs are
      rewritten to the active instance before being handed to the `<video>` element or `<img>`
      tags.
- [ ] `youtubei.js` remains a renderer dependency; the local↔Invidious fallback in the Watch
      view is preserved without restructuring.

### Configuration consequences

| Area | Consequence |
|------|-------------|
| **CSP** | `tauri.conf.json` CSP must permit `connect-src`, `img-src`, and `media-src` to user-configured Invidious instances. Since instances are dynamic, this needs a permissive-but-scoped policy (e.g. `https:` for these directives) — document the tradeoff. |
| **CORS** | Public Invidious instances generally send permissive CORS headers, but not universally. Misbehaving instances will fail in the webview where they would have succeeded from Rust. Surface a clear, actionable error. |
| **Secrets exposure** | Instance auth tokens are visible to anything running in the renderer. Acceptable because they are low-value, user-supplied, per-instance credentials — **not** acceptable for the sync keys, which is precisely why ADR 003 moves those to Rust. |
| **No backend caching** | Invidious responses are not cached by Rust. If caching becomes necessary, implement it in the renderer store first before reconsidering Option B. |
| **Network diagnostics** | Rust cannot log or instrument Invidious traffic. Any request-level telemetry must be added renderer-side. |

### Revisit criteria

Reopen this decision if **any** of the following becomes true:

1. CORS failures against common instances become a recurring support burden.
2. Instance auth expands to hold high-value credentials (OAuth, account-linked tokens).
3. The Watch view's fallback logic is refactored into a standalone, testable service — at which
   point the coupling argument in §Rationale-1 no longer holds.
4. A viable Rust InnerTube client emerges, making it sensible to move *both* providers together.

Moving one provider without the other is the outcome to avoid.

---

## References

- Electron baseline: `src/renderer/helpers/api/invidious.js` (1009 lines),
  `src/renderer/helpers/api/local.js` (2573 lines)
- [Invidious API documentation](https://docs.invidious.io/api/)
- [../architecture/01-electron-vs-tauri.md](../architecture/01-electron-vs-tauri.md) §"API Helpers"
- [../architecture/03-data-flow.md](../architecture/03-data-flow.md)

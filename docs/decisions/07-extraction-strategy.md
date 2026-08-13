# 07 - Extraction Strategy

> **Domain:** `decisions`
> **Status:** Accepted
> **Date:** 2026-08-13
> **Related:** [04-potoken-generation.md](../backend/04-potoken-generation.md), [02-tauri-commands.md](../backend/02-tauri-commands.md)

---

## 1. Context

SlyTube needs to extract data from YouTube: search results, video info, channels, comments, playlists, and feeds. The upstream OpenTubeX codebase relies on **youtubei.js** (Innertube), a JavaScript library that constructs InnerTube protobuf requests, parses responses into typed node trees, and handles signature deciphering and DASH manifest generation.

Two extraction paths were prototyped in early Slytube:

1. **Direct InnerTube HTTP in Rust** (`src-tauri/src/commands/youtube.rs`) — hand-rolled request construction and JSON parsing in Rust
2. **youtubei.js in a hidden webview** — embed the JS library and run it in a hidden `WebviewWindow`, returning parsed JSON to Rust

The direct-Rust path is unsustainable: YouTube has hundreds of renderer types, the protobuf schemas change frequently, and the parsing logic is tightly coupled to youtubei.js's typed node tree. Maintaining a parallel Rust reimplementation perpetually lags behind YouTube's changes.

## 2. Options Considered

| Option | Description | Verdict |
|---|---|---|
| **A. Hidden webview + youtubei.js** | Run Innertube in a hidden `WebviewWindow`; Rust dispatches requests via `eval()` and receives parsed JSON via `invoke()` callback | **Chosen** |
| B. Direct InnerTube HTTP in Rust | Hand-roll protobuf construction and JSON parsing | Rejected — high maintenance, always out of date |
| C. Embed a JS engine (deno_core/quick-js) | Run youtubei.js in an embedded Rust JS runtime | Rejected — youtubei.js expects browser globals (DOM, fetch); heavy polyfill burden |
| D. Call youtubei.js from the renderer | Run Innertube in the main app webview alongside Vue | Rejected — couples extraction to UI state; harder to isolate and test |

## 3. Decision

**Option A** — a persistent hidden `WebviewWindow` labeled `"extractor"` runs youtubei.js. Rust dispatches extraction requests with a unique request ID, the webview processes them through Innertube and returns flat JSON, and Rust correlates responses back via oneshot channels.

The **Invidious API** (already implemented in `src-tauri/src/commands/invidious.rs`) remains as the fallback path when the extractor fails.

## 4. Rationale

- **youtubei.js is the reference implementation.** It tracks YouTube's schema changes across hundreds of contributors. Reimplementing its parser in Rust is a multi-year effort that breaks on every YouTube update.
- **The hidden webview reuses Tauri's existing infrastructure.** We already create hidden webviews for PoToken generation (ADR 001). The extractor uses the same pattern — no new architectural primitive.
- **Flat JSON over the bridge.** Rust never touches YouTube's protobuf schemas. The JS side returns plain `{ type: 'video', data: { ... } }` objects. Rust only handles typed domain models (`Video`, `Channel`, etc.).
- **Request-ID correlation.** Each `extract()` call generates a UUID, registers a `tokio::sync::oneshot` sender, and the JS callback invokes `extraction_result` with the same UUID. This gives us async/await ergonomics in Rust without polling or shared mutable state.
- **Graceful degradation.** Every extraction call tries the extractor first, then falls back to Invidious. The user never sees a hard failure from the extraction layer.

## 5. Implications

- The extractor webview is a **long-lived hidden window** (not created/destroyed per request like PoToken). This amortizes Innertube session creation cost.
- `botGuardScript.js` is a placeholder until the real BotGuard VM is vendored from upstream OpenTubeX. Until then, age-restricted or bot-checked content falls back to Invidious.
- The direct InnerTube HTTP commands in `src-tauri/src/commands/youtube.rs` are **deprecated** but not deleted — they serve as a reference and can be used as a tertiary fallback if needed.
- Vite is configured for multi-page builds: `index.html` (main app) + `extractor.html` (hidden webview), both output to `dist/`.

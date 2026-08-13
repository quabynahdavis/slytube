# 07 - Extractor Bridge

> **Domain:** `backend`
> **Status:** Implemented (src-tauri/src/extractor/ + src/extractor/)
> **Related:** [04-potoken-generation.md](04-potoken-generation.md), [02-tauri-commands.md](02-tauri-commands.md)

---

## 1. Overview

The **extractor** is a persistent hidden `WebviewWindow` that runs youtubei.js (Innertube) for all InnerTube data extraction. Rust dispatches requests via `eval()` and receives parsed JSON back through `invoke()` callbacks, correlated by request ID.

This module implements ADR 007 (hidden webview youtubei.js pivot) and replaces the unsustainable direct-Rust-InnerTube HTTP approach.

## 2. Architecture

```
Frontend (Vue)
  ↓ invoke('extract', { method: 'getVideoInfo', params: { videoId: '...' } })
Rust (extractor::commands::extract)
  ↓ Generate UUID req_id, register oneshot sender
  ↓ eval() → window.__slytube_run(reqId, method, params)
Hidden Webview (extractor.html + main.ts)
  ↓ youtubei.js Innertube processes request
  ↓ Parse response to flat JSON
  ↓ invoke('extraction_result', { reqId, result })
Rust (extractor::commands::extraction_result)
  ↓ Look up oneshot sender by reqId, send result
  ↓ extract() returns deserialized JSON to frontend
```

### 2.1 Why a persistent window

Unlike PoToken generation (which creates/destroys a window per request), the extractor is a **long-lived** hidden window. This:
- Amortizes Innertube session creation (one session reuse across all requests)
- Avoids the overhead of spawning/teardown per extraction
- Keeps the implementation simpler (no per-request lifecycle management)

The trade-off is that the webview process stays resident for the app's lifetime. Memory overhead is ~50-100 MB (typical for a WebKit/WebView2 instance).

## 3. Rust API

### 3.1 Commands

| Command | Direction | Signature | Purpose |
|---------|-----------|-----------|---------|
| `extract` | invoke | `(method: String, params: Value) -> Result<Value, String>` | Dispatch extraction request |
| `extraction_result` | invoke | `(req_id: String, result: Value) -> Result<(), String>` | Callback from JS with result |
| `extractor_ready` | invoke | `() -> Result<bool, String>` | Health check |

### 3.2 State

```rust
pub struct PendingExtractions(Mutex<HashMap<String, oneshot::Sender<serde_json::Value>>>);
```

Managed via `app.manage()` in `lib.rs`. Maps request IDs to oneshot senders.

### 3.3 Method validation

The `extract` command validates the method against an allowlist before dispatching. Valid methods:

```
getVideoInfo, search, getChannel, getChannelVideos, getChannelShorts,
getChannelLive, getChannelCommunity, getChannelPlaylists, getComments,
getCommentReplies, getTrending, getPlaylist, getHashtag, getCommunityPost,
getSearchSuggestions, generatePoToken
```

## 4. JavaScript Bridge (src/extractor/main.ts)

### 4.1 Entry point

`extractor.html` is a minimal HTML shell that loads `main.ts` as an ES module. Built by Vite as a separate Rollup entry (see `vite.config.ts`), output to `dist/extractor.html`.

### 4.2 Innertube session

```typescript
let innertube: Innertube | null = null

async function getInnertube(): Promise<Innertube> {
  if (!innertube) {
    innertube = await Innertube.create({
      enable_session_cache: false,
      generate_session_locally: true,
      retrieve_innertube_config: true,
      user_agent: DESKTOP_USER_AGENT,
    })
  }
  return innertube
}
```

Session is created lazily on first request and reused for the app lifetime.

### 4.3 Request handling

`window.__slytube_run(reqId, method, params)` is the global entry point called by Rust via `eval()`. It dispatches on `method` to the appropriate handler.

### 4.4 Result delivery

```typescript
async function deliverResult(reqId: string, result: unknown): Promise<void> {
  await invoke('extraction_result', {
    reqId,
    result: { data: result },
  })
}

async function deliverError(reqId: string, error: string): Promise<void> {
  await invoke('extraction_result', {
    reqId,
    result: { error },
  })
}
```

## 5. Method Reference

### 5.1 getVideoInfo

Returns: `VideoInfo` with id, title, author, description, thumbnail, viewCount, likeCount, lengthSeconds, isLive, isUpcoming, chapters, captions, related.

Uses `innertube.getInfo(videoId, client)`.

### 5.2 search

Parameters: `query`, `upload_date`, `type`, `duration`, `prioritize`, `features`.

Returns: `Array<{ type: string, data: unknown }>` — mixed video/channel/playlist items.

Uses `innertube.search(query, filters)`.

### 5.3 getChannel

Parameters: `channelId`, `includeHomeShelves`.

Returns: `ChannelInfo` with id, name, description, avatar, banner, subscriberCount, tabs, videos, shelves.

Uses `innertube.getChannel(channelId)`. Tab detection via boolean getters (`has_home`, `has_videos`, etc.). Home shelves read from `current_tab.content.contents`.

### 5.4 Channel sub-tabs

| Method | youtubei.js call | Returns |
|--------|-----------------|---------|
| getChannelVideos | `channel.getVideos()` | `{ videos: VideoInfo[], continuation: bool }` |
| getChannelShorts | `channel.getShorts()` | `{ videos: VideoInfo[], continuation: bool }` |
| getChannelLive | `channel.getLiveStreams()` | `{ videos: VideoInfo[], continuation: bool }` |
| getChannelCommunity | `channel.getCommunity()` | `{ posts: unknown[] }` |
| getChannelPlaylists | `channel.getPlaylists()` | `{ playlists: PlaylistInfo[] }` |

### 5.5 getComments / getCommentReplies

Parameters: `videoId`, `sort_by` (TOP_COMMENTS | NEWEST_FIRST), `commentId` (for replies).

Uses `innertube.getComments(videoId, sortBy, commentId)`.

### 5.6 getTrending

Parameters: `tab` (default | music | gaming | movies | sports).

Uses `innertube.actions.execute('/browse', { browseId: 'FEtrending', params })` with per-tab protobuf params.

### 5.7 getPlaylist

Parameters: `playlistId`.

Uses `innertube.getPlaylist(playlistId)`.

### 5.8 getHashtag

Parameters: `hashtag`.

Uses `innertube.getHashtag(hashtag)`.

### 5.9 getCommunityPost

Parameters: `postId`, `channelId`.

Uses `innertube.getPost(postId, channelId)`.

### 5.10 getSearchSuggestions

Parameters: `query`.

Uses `innertube.getSearchSuggestions(query)`.

## 6. Parsers

The extractor includes JS ports of OpenTubeX's response parsers:

| Parser | Purpose | Handles |
|--------|---------|---------|
| `parseListItem` | Master dispatcher for search results | Video, GridVideo, Movie, Channel, GridChannel, Playlist, GridPlaylist, ReelItem, ShortsLockupView, LockupView, HashtagTile, Post, GameCard |
| `parseLockupView` | LockupView sub-type discriminator | ALBUM, PLAYLIST, PODCAST, SHORT, VIDEO (via `content_type`) |
| `parseVideo` | Video node → flat VideoInfo | Video, GridVideo, Movie nodes |
| `parseChannelHomeTab` | Channel home shelf detection | Shelf, ReelShelf, HorizontalCardList, RichShelf |
| `parseCommunityPost` | Community post → flat object | Post, BackstagePost |
| `parseComment` | Comment node → flat object | Comment nodes |

All parsers filter members-only content at parse time (return null), keeping the output clean.

## 7. Error Handling

- JS-side errors are caught in `window.__slytube_run` and delivered via `deliverError`
- The `extract` command checks for `{ error }` in the returned JSON
- Frontend `api/index.ts` catches extractor errors and falls back to Invidious
- Timeout is handled by the oneshot channel (drops on webview disconnect)

## 8. File Layout

```
src/extractor/
  main.ts          — Bridge entry point, Innertube session, method handlers, parsers
extractor.html     — Minimal HTML shell (Vite entry point)

src-tauri/src/extractor/
  mod.rs           — Module re-exports
  models.rs        — ExtractionRequest, ExtractionResult, ExtractionMethod
  commands.rs      — extract(), extraction_result(), extractor_ready()

vite.config.ts     — Multi-page Rollup: main + extractor entries
src-tauri/src/lib.rs — Creates hidden webview, manages PendingExtractions state
```

## 9. Invidious Fallback

When the extractor fails (youtubei.js error, webview not ready, etc.), the frontend falls back to the Invidious API via `commands/invidious.rs`. This two-layer approach ensures:

- **Primary**: youtubei.js — full feature set, handles all YouTube layouts
- **Fallback**: Invidious — simpler but works without a JS runtime

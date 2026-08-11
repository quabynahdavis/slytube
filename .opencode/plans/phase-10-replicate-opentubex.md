# Phase 10: Replicate OpenTubeX Video Fetching Exactly

## Problem Analysis

**Why videos aren't showing:**
1. **youtubei.js fails** - Tauri webview enforces CORS, blocking YouTube API calls from browser
2. **Invidious CORS** - Some Invidious instances don't send proper CORS headers
3. **No request interception** - Unlike Electron's `webRequest`, Tauri doesn't let us modify headers or proxy requests from the renderer

**How OpenTubeX solves this:**
- Electron `webSecurity: false` → no CORS at all
- `session.defaultSession.webRequest.onBeforeSendHeaders` → adds YouTube headers to ALL requests
- Custom `imagecache://` protocol → proxies images through Electron's `net.request` (no CORS)
- `getProxyUrl()` → rewrites media URLs to go through Invidious

## Solution Architecture

Since Tauri enforces CORS in the webview, we need to make API calls from the **Rust side** (no CORS) and return results to the frontend.

### Approach: Tauri Commands for All YouTube/Invidious Data

```
Frontend (Vue) → invoke("get_video_info", { videoId }) → Rust (reqwest, no CORS) → YouTube/Invidious → JSON response → Frontend
```

---

## Implementation Plan

### Task 1: Create Tauri HTTP Client Module

**File:** `src-tauri/src/http_client.rs`

- Create a `reqwest::Client` configured with:
  - YouTube-compatible headers (User-Agent, Referer, Origin)
  - SOCKS5 proxy support (for Tor/proxy users)
  - Timeout configuration
- Singleton client managed as Tauri state

### Task 2: Create YouTube InnerTube Commands

**File:** `src-tauri/src/commands/youtube.rs`

Commands:
- `get_video_info(video_id: String) -> Result<serde_json::Value, String>`
  - Makes POST to `https://www.youtube.com/youtubei/v1/player` with proper context
  - Returns raw InnerTube response
  
- `get_video_formats(video_id: String) -> Result<serde_json::Value, String>`
  - Gets streaming data (formats, DASH manifest URLs)
  
- `search_videos(query: String) -> Result<serde_json::Value, String>`
  - POST to `https://www.youtube.com/youtubei/v1/search`
  
- `get_channel_info(channel_id: String) -> Result<serde_json::Value, String>`
  - POST to `https://www.youtube.com/youtubei/v1/browse`
  
- `get_trending() -> Result<serde_json::Value, String>`
  - POST to `https://www.youtube.com/youtubei/v1/browse?browseId=FEtrending`

### Task 3: Create Invidious API Commands

**File:** `src-tauri/src/commands/invidious.rs`

Commands:
- `invidious_get_video(video_id: String) -> Result<serde_json::Value, String>`
  - GET `{instance}/api/v1/videos/{video_id}`
  
- `invidious_search(query: String) -> Result<serde_json::Value, String>`
  - GET `{instance}/api/v1/search?q={query}`
  
- `invidious_get_channel(channel_id: String) -> Result<serde_json::Value, String>`
  - GET `{instance}/api/v1/channels/{channel_id}`
  
- `invidious_get_trending() -> Result<serde_json::Value, String>`
  - GET `{instance}/api/v1/trending`
  
- `invidious_get_playlist(playlist_id: String) -> Result<serde_json::Value, String>`
  - GET `{instance}/api/v1/playlists/{playlist_id}`
  
- `invidious_get_comments(video_id: String) -> Result<serde_json::Value, String>`
  - GET `{instance}/api/v1/comments/{video_id}`
  
- `invidious_get_instances() -> Result<serde_json::Value, String>`
  - GET `https://api.invidious.io/instances.json`
  - Filters for CORS-enabled, API-working instances

### Task 4: Create DASH Manifest Generation

**File:** `src-tauri/src/commands/manifest.rs`

- `generate_dash_manifest(video_id: String) -> Result<String, String>`
  - Gets video info from InnerTube
  - Deciphers format URLs (n/sig deciphering)
  - Generates DASH XML manifest
  - Returns data URI: `data:application/dash+xml;charset=UTF-8,...`

### Task 5: Update Frontend API Layer

**File:** `src/api/index.ts`

Replace all `fetch()` calls with `invoke()` calls:

```typescript
import { invoke } from '@tauri-apps/api/core'

export async function getVideo(videoId: string): Promise<Video> {
  const result = await invoke('get_video_info', { videoId })
  return mapYouTubeResponse(result)
}

export async function search(query: string): Promise<Video[]> {
  const result = await invoke('search_videos', { query })
  return mapSearchResponse(result)
}
// ... etc
```

### Task 6: Update Cargo.toml Dependencies

Add to `src-tauri/Cargo.toml`:
```toml
reqwest = { version = "0.12", features = ["json", "socks"] }
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

### Task 7: Register Commands in lib.rs

```rust
.invoke_handler(tauri::generate_handler![
  // ... existing commands
  youtube::get_video_info,
  youtube::search_videos,
  youtube::get_channel_info,
  youtube::get_trending,
  invidious::invidious_get_video,
  invidious::invidious_search,
  invidious::invidious_get_channel,
  invidious::invidious_get_trending,
  invidious::invidious_get_playlist,
  invidious::invidious_get_comments,
  invidious::invidious_get_instances,
])
```

---

## Detailed Implementation

### YouTube InnerTube Request Format

```rust
// POST https://www.youtube.com/youtubei/v1/player
let body = serde_json::json!({
    "context": {
        "client": {
            "clientName": "WEB",
            "clientVersion": "2.20240101.01.00",
            "hl": "en",
            "gl": "US",
            "userAgent": "Mozilla/5.0 ...",
        }
    },
    "videoId": video_id,
    "playbackContext": {
        "contentPlaybackContext": {
            "html5Preference": "HTML5_PREF_WANTS"
        }
    },
    "racyCheckOk": true,
    "contentCheckOk": true,
});

let response = client
    .post("https://www.youtube.com/youtubei/v1/player")
    .header("Referer", "https://www.youtube.com/")
    .header("Origin", "https://www.youtube.com")
    .header("Content-Type", "application/json")
    .json(&body)
    .send()
    .await?;
```

### Invidious Instance Selection

1. Fetch list from `https://api.invidious.io/instances.json`
2. Filter: `cors == true && api == true && score > 0`
3. Test health: GET `/api/v1/stats` on each
4. Cache working instances, rotate on failure

### Fallback Chain

```
1. Try YouTube InnerTube (via Rust HTTP client)
   ↓ fails
2. Try Invidious API (via Rust HTTP client)
   ↓ fails  
3. Try next Invidious instance
   ↓ all fail
4. Show error to user
```

---

## Files to Create/Modify

| File | Action |
|------|--------|
| `src-tauri/src/http_client.rs` | Create - HTTP client with proxy support |
| `src-tauri/src/commands/youtube.rs` | Create - InnerTube API commands |
| `src-tauri/src/commands/invidious.rs` | Create - Invidious API commands |
| `src-tauri/src/commands/mod.rs` | Update - register new modules |
| `src-tauri/src/lib.rs` | Update - register new commands |
| `src-tauri/Cargo.toml` | Update - add reqwest dependency |
| `src/api/index.ts` | Modify - use invoke instead of fetch |
| `src/composables/useInnertube.ts` | Simplify - just call Tauri commands |

---

## Success Criteria

- [ ] YouTube trending videos display with thumbnails
- [ ] Search returns real YouTube results
- [ ] Video page loads real video info (title, author, views, description)
- [ ] Channel page loads real channel data
- [ ] Comments load for videos
- [ ] No CORS errors in console
- [ ] Fallback works when one source fails

---

## Risks & Mitigations

| Risk | Mitigation |
|----------|------------|
| YouTube rate limiting/blocking | Rotate User-Agents, add delay between requests |
| n/sig deciphering complexity | Start with Invidious only, add InnerTube later |
| Invidious instances down | Cache multiple instances, auto-rotate |
| DASH manifest generation | Use Invidious DASH endpoint initially |

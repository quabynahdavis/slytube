# API Integration

## Overview

Slytube keeps the **entire YouTube data layer in the renderer**. `youtubei.js`,
the Invidious client, and the SponsorBlock client all run inside the Tauri
webview exactly as they did inside the Electron renderer. Only two things cross
into Rust:

1. **PoToken generation** — requires a controlled webview and a native lifecycle
2. **HTTP requests that need forbidden headers or must bypass CORS** — handled
   by a Rust `reqwest` proxy command

This is a deliberate reversal of the earlier "port `local.js` to Rust" idea in
[../architecture/01-electron-vs-tauri.md](../architecture/01-electron-vs-tauri.md) §6.

**Why the renderer keeps the API layer:**

| Factor | Rust port | Keep in renderer |
|--------|-----------|------------------|
| `local.js` size | 2573 lines to re-implement | 0 lines changed |
| `invidious.js` size | 1009 lines to re-implement | 0 lines changed |
| YouTube protocol churn | Re-port on every InnerTube change | `bun update youtubei.js` |
| Upstream parity with FreeTube | Diverges immediately | Patches apply cleanly |
| Player integration | Serialise streams across IPC | Direct object handoff to Shaka |
| Risk | High | Low |

The cost is that the renderer must be able to make requests with headers the
webview normally forbids. §2 solves that.

---

## 1. Architecture

```
┌──────────────────────── Tauri Webview (renderer) ─────────────────────────┐
│                                                                            │
│  Vue views ──► Pinia stores ──► src/api/*  (data layer)                    │
│                                    │                                       │
│        ┌───────────────────────────┼───────────────────────────┐           │
│        ▼                           ▼                           ▼           │
│  ┌───────────┐            ┌────────────────┐          ┌──────────────┐    │
│  │youtubei.js│            │Invidious client│          │ SponsorBlock │    │
│  │ (Innertube)│           │  (fetch)       │          │   client     │    │
│  └─────┬─────┘            └────────┬───────┘          └──────┬───────┘    │
│        │                           │                         │            │
│        └──────────► src/api/fetch-wrapper.ts ◄───────────────┘            │
│                     (header injection · retry · routing)                   │
│                              │                    │                        │
│                     direct fetch()         invoke('proxy_fetch')           │
└──────────────────────────────┼────────────────────┼────────────────────────┘
                               │                    │
                               ▼                    ▼
                        youtube.com /        ┌──────────────────┐
                        invidious host       │ Rust: reqwest    │
                                             │ + hidden webview │
                                             │   (PoToken)      │
                                             └──────────────────┘
```

### 1.1 Directory layout

```
src/api/
├── fetch-wrapper.ts        # header-injecting fetch + proxy routing
├── innertube.ts            # youtubei.js singleton + session management
├── local.ts                # getVideoInformationLocal, channel, search, …
├── invidious.ts            # getVideoInformationInvidious, instance handling
├── fallback.ts             # local → invidious orchestration
├── sponsorblock.ts         # segment fetch + hashing + categories
├── potoken.ts              # invoke wrapper + cache for PoToken
└── types.ts                # normalised cross-backend models
```

---

## 2. Custom Fetch Wrapper

### 2.1 The problem

InnerTube rejects requests that lack the right context headers. Three of them
are on the [Forbidden header name](https://developer.mozilla.org/en-US/docs/Glossary/Forbidden_header_name)
list, so `fetch()` in a webview silently strips them:

| Header | Purpose | Settable from JS? |
|--------|---------|-------------------|
| `Referer` | Must be `https://www.youtube.com/` | ❌ forbidden |
| `Origin` | Must be `https://www.youtube.com` | ❌ forbidden |
| `X-Youtube-Bootstrap-Logged-In` | `false` for anonymous sessions | ✅ allowed |
| `X-Youtube-Client-Name` / `-Version` | Client identification | ✅ allowed |
| `X-Goog-Visitor-Id` | Visitor session continuity | ✅ allowed |
| `User-Agent` | Client fingerprint | ❌ forbidden |
| `Cookie` | Consent / visitor cookies | ❌ forbidden |

Electron solved this with `session.webRequest.onBeforeSendHeaders`. Tauri has no
such hook for the webview, so requests needing forbidden headers are routed
through a Rust command.

### 2.2 Routing rule

```
Request needs a forbidden header?
   ├── no  → native fetch() with allowed headers only   (fast path)
   └── yes → invoke('proxy_fetch', …) via reqwest       (proxy path)
```

In practice: **InnerTube → proxy path**, **Invidious / SponsorBlock / thumbnails
→ fast path**. Invidious and SponsorBlock are CORS-permissive and require no
forbidden headers.

### 2.3 Implementation

```typescript
// src/api/fetch-wrapper.ts
import { invoke } from '@tauri-apps/api/core'

const YOUTUBE_ORIGIN = 'https://www.youtube.com'

/** Headers required by InnerTube; several are forbidden in the webview. */
export interface InnertubeHeaderContext {
  visitorData?: string
  clientName: string      // 'WEB' | 'ANDROID' | 'IOS' | 'TVHTML5_SIMPLY_EMBEDDED_PLAYER'
  clientVersion: string
  userAgent: string
  loggedIn: boolean
}

const FORBIDDEN = new Set([
  'referer', 'origin', 'user-agent', 'cookie',
  'accept-encoding', 'host', 'connection'
])

function needsProxy(headers: Record<string, string>): boolean {
  return Object.keys(headers).some(h => FORBIDDEN.has(h.toLowerCase()))
}

export function buildInnertubeHeaders(
  ctx: InnertubeHeaderContext
): Record<string, string> {
  const headers: Record<string, string> = {
    'Content-Type': 'application/json',
    'Accept-Language': 'en-US,en;q=0.9',

    // forbidden — proxy path only
    'Referer': `${YOUTUBE_ORIGIN}/`,
    'Origin': YOUTUBE_ORIGIN,
    'User-Agent': ctx.userAgent,

    // allowed
    'X-Youtube-Bootstrap-Logged-In': String(ctx.loggedIn),
    'X-Youtube-Client-Name': ctx.clientName,
    'X-Youtube-Client-Version': ctx.clientVersion,
    'X-Goog-Api-Format-Version': '2'
  }
  if (ctx.visitorData) headers['X-Goog-Visitor-Id'] = ctx.visitorData
  return headers
}

export interface ProxyRequest {
  url: string
  method: string
  headers: Record<string, string>
  body: string | null
}

export interface ProxyResponse {
  status: number
  statusText: string
  headers: Record<string, string>
  body: string
}

/**
 * Drop-in `fetch` replacement handed to youtubei.js.
 * Routes through Rust when forbidden headers are present.
 */
export async function tauriFetch(
  input: RequestInfo | URL,
  init: RequestInit = {}
): Promise<Response> {
  const url = input instanceof Request ? input.url : input.toString()
  const method = (input instanceof Request ? input.method : init.method) ?? 'GET'

  const headers: Record<string, string> = {}
  new Headers(
    input instanceof Request ? input.headers : init.headers
  ).forEach((value, key) => { headers[key] = value })

  let body: string | null = null
  if (input instanceof Request) {
    body = input.body ? await input.clone().text() : null
  } else if (init.body != null) {
    body = typeof init.body === 'string' ? init.body : JSON.stringify(init.body)
  }

  // Fast path — no forbidden headers, let the webview handle it.
  if (!needsProxy(headers)) {
    return fetch(input, init)
  }

  // Proxy path — reqwest in Rust, full header control.
  const res = await invoke<ProxyResponse>('proxy_fetch', {
    request: { url, method, headers, body } satisfies ProxyRequest
  })

  return new Response(res.body, {
    status: res.status,
    statusText: res.statusText,
    headers: new Headers(res.headers)
  })
}
```

### 2.4 Rust side

```rust
// src-tauri/src/commands/proxy.rs
use std::collections::HashMap;
use reqwest::{Client, Method, header::{HeaderMap, HeaderName, HeaderValue}};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProxyRequest {
    pub url: String,
    pub method: String,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProxyResponse {
    pub status: u16,
    pub status_text: String,
    pub headers: HashMap<String, String>,
    pub body: String,
}

#[tauri::command]
pub async fn proxy_fetch(
    request: ProxyRequest,
    state: tauri::State<'_, AppState>,
) -> Result<ProxyResponse, AppError> {
    let client: &Client = &state.http_client;   // pooled, cookie-store enabled

    let mut headers = HeaderMap::new();
    for (k, v) in &request.headers {
        if let (Ok(name), Ok(value)) =
            (HeaderName::from_bytes(k.as_bytes()), HeaderValue::from_str(v))
        {
            headers.insert(name, value);   // reqwest permits forbidden names
        }
    }

    let method = Method::from_bytes(request.method.as_bytes())
        .map_err(|_| AppError::bad_request("invalid HTTP method"))?;

    let mut builder = client.request(method, &request.url).headers(headers);
    if let Some(body) = request.body {
        builder = builder.body(body);
    }

    let resp = builder.send().await.map_err(AppError::from)?;
    let status = resp.status();

    let resp_headers = resp.headers().iter()
        .filter_map(|(k, v)| v.to_str().ok().map(|s| (k.to_string(), s.to_string())))
        .collect();

    Ok(ProxyResponse {
        status: status.as_u16(),
        status_text: status.canonical_reason().unwrap_or("").to_string(),
        headers: resp_headers,
        body: resp.text().await.map_err(AppError::from)?,
    })
}
```

The shared `reqwest::Client` is built once with a cookie store, so YouTube
consent and visitor cookies persist across requests:

```rust
let http_client = Client::builder()
    .cookie_store(true)
    .gzip(true).brotli(true)
    .timeout(Duration::from_secs(30))
    .build()?;
```

### 2.5 Capability permissions

```jsonc
// src-tauri/capabilities/default.json
{
  "identifier": "default",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "opener:default",
    { "identifier": "http:default",
      "allow": [
        { "url": "https://*.youtube.com/*" },
        { "url": "https://*.googlevideo.com/*" },
        { "url": "https://*.ytimg.com/*" },
        { "url": "https://sponsor.ajay.app/*" },
        { "url": "https://api.invidious.io/*" }
      ] }
  ]
}
```

Invidious instances are user-configurable, so their requests go through
`proxy_fetch` (validated against the stored instance list) rather than a static
allowlist.

---

## 3. youtubei.js (Innertube) in the Webview

`youtubei.js` runs unmodified. It only needs its `fetch` option replaced and,
for protected content, a PoToken.

```typescript
// src/api/innertube.ts
import { Innertube, ClientType, Platform, UniversalCache } from 'youtubei.js'
import { tauriFetch, buildInnertubeHeaders } from './fetch-wrapper'
import { getPoToken } from './potoken'
import { useSettingsStore } from '@/stores/settings'

let instance: Innertube | null = null
let creating: Promise<Innertube> | null = null

const USER_AGENT =
  'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 ' +
  '(KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36'

export async function getInnertube(): Promise<Innertube> {
  if (instance) return instance
  if (creating) return creating

  creating = (async () => {
    const settings = useSettingsStore()
    const { token, visitorData } = await getPoToken()

    instance = await Innertube.create({
      lang: settings.values.locale ?? 'en',
      location: settings.values.region ?? 'US',
      client_type: ClientType.WEB,
      generate_session_locally: true,
      retrieve_player: true,
      enable_session_cache: true,
      cache: new UniversalCache(false),

      po_token: token,
      visitor_data: visitorData,

      // All Innertube traffic goes through the wrapper.
      fetch: (input, init) => {
        const merged: RequestInit = {
          ...init,
          headers: {
            ...buildInnertubeHeaders({
              visitorData,
              clientName: 'WEB',
              clientVersion: '2.20250101.00.00',
              userAgent: USER_AGENT,
              loggedIn: false
            }),
            ...(init?.headers as Record<string, string> | undefined)
          }
        }
        return tauriFetch(input, merged)
      }
    })
    return instance
  })()

  return creating
}

/** Force a fresh session — call after PoToken expiry or a 403 storm. */
export function resetInnertube(): void {
  instance = null
  creating = null
}
```

### 3.1 `getVideoInformationLocal`

```typescript
// src/api/local.ts
import { getInnertube, resetInnertube } from './innertube'
import { normaliseLocalVideo } from './types'
import type { VideoInformation } from './types'

export async function getVideoInformationLocal(
  videoId: string,
  options: { attempt?: number } = {}
): Promise<VideoInformation> {
  const attempt = options.attempt ?? 0
  const yt = await getInnertube()

  try {
    const info = await yt.getInfo(videoId)

    if (info.playability_status?.status === 'LOGIN_REQUIRED') {
      throw new ApiError('AGE_RESTRICTED', info.playability_status.reason ?? '')
    }
    if (info.playability_status?.status === 'UNPLAYABLE') {
      throw new ApiError('UNPLAYABLE', info.playability_status.reason ?? '')
    }

    return normaliseLocalVideo(info)
  } catch (err) {
    // A stale PoToken/session presents as 403 — refresh once and retry.
    if (isSessionError(err) && attempt === 0) {
      resetInnertube()
      return getVideoInformationLocal(videoId, { attempt: 1 })
    }
    throw err
  }
}
```

Everything else — `getLocalChannel`, `getLocalSearchResults`,
`getLocalPlaylist`, `getLocalComments`, `getLocalTrending` — ports over with the
same shape: call `youtubei.js`, normalise, throw a typed `ApiError`.

---

## 4. Invidious Client

Kept **verbatim** from the existing renderer implementation. It is plain
`fetch`, CORS-friendly, and needs no forbidden headers.

```typescript
// src/api/invidious.ts
import { useInvidiousStore } from '@/stores/invidious'
import { useSettingsStore } from '@/stores/settings'
import { normaliseInvidiousVideo } from './types'
import type { VideoInformation } from './types'

interface InvidiousRequestOptions {
  resource: string                     // 'videos' | 'channels' | 'search' | …
  id?: string
  params?: Record<string, string | number | boolean>
  signal?: AbortSignal
}

export async function invidiousRequest<T>(
  opts: InvidiousRequestOptions
): Promise<T> {
  const invidious = useInvidiousStore()
  const settings = useSettingsStore()

  const base = invidious.currentInstanceUrl
    ?? settings.values.currentInvidiousInstance

  const url = new URL(
    `${base}/api/v1/${opts.resource}${opts.id ? `/${opts.id}` : ''}`
  )
  for (const [k, v] of Object.entries(opts.params ?? {})) {
    url.searchParams.set(k, String(v))
  }

  const res = await fetch(url, {
    method: 'GET',
    headers: { Accept: 'application/json' },
    signal: opts.signal
  })

  if (!res.ok) {
    throw new ApiError(
      res.status === 429 ? 'RATE_LIMITED' : 'INVIDIOUS_ERROR',
      `${base} responded ${res.status}`
    )
  }
  return res.json() as Promise<T>
}

export async function getVideoInformationInvidious(
  videoId: string
): Promise<VideoInformation> {
  const raw = await invidiousRequest<InvidiousVideoResponse>({
    resource: 'videos',
    id: videoId
  })
  return normaliseInvidiousVideo(raw)
}
```

### 4.1 Instance health & rotation

The `invidious` store keeps the instance list and demotes failing hosts:

```typescript
async function requestWithRotation<T>(opts: InvidiousRequestOptions): Promise<T> {
  const invidious = useInvidiousStore()
  const candidates = invidious.healthyInstances

  let lastError: unknown
  for (const host of candidates.slice(0, 3)) {   // at most 3 hosts per call
    try {
      return await invidiousRequest<T>({ ...opts, /* host override */ })
    } catch (err) {
      lastError = err
      invidious.reportFailure(host)
    }
  }
  throw lastError
}
```

Rotation is **opt-in per call site**. Video playback uses it; search does not
(rotating mid-typing produces inconsistent results).

### 4.2 Normalisation contract

Both backends must produce the identical `VideoInformation` shape, or the Watch
view needs backend-aware branching. This is the single most important invariant
in the data layer.

```typescript
// src/api/types.ts
export interface VideoInformation {
  id: string
  title: string
  description: string
  descriptionRuns: DescriptionRun[]     // timestamps/links preserved
  channel: { id: string; name: string; thumbnailUrl: string | null; subscriberText: string | null }
  durationSeconds: number
  viewCount: number | null
  likeCount: number | null
  publishedAt: number | null
  isLive: boolean
  isUpcoming: boolean
  isFamilyFriendly: boolean
  keywords: string[]
  chapters: Chapter[]
  formats: VideoFormat[]
  dashManifestUrl: string | null
  hlsManifestUrl: string | null
  storyboards: Storyboard[]
  captions: CaptionTrack[]
  recommendations: Video[]
  /** Which backend produced this object — for diagnostics only. */
  source: 'local' | 'invidious'
}
```

`source` is for logging and the stats-for-nerds panel. **No view may branch on
it for rendering.**

---

## 5. Fallback Chain

### 5.1 Policy

| `backendPreference` | `backendFallback` | Order |
|---------------------|-------------------|-------|
| `local` | `true` | local → invidious |
| `local` | `false` | local only |
| `invidious` | `true` | invidious → local |
| `invidious` | `false` | invidious only |

### 5.2 Which errors trigger a fallback

| Condition | Fall back? | Rationale |
|-----------|-----------|-----------|
| Network failure / timeout | ✅ | Transient or host-specific |
| HTTP 5xx | ✅ | Backend fault |
| HTTP 429 | ✅ | Rate limited on this backend |
| HTTP 403 after PoToken refresh | ✅ | Local session unrecoverable |
| `AGE_RESTRICTED` | ✅ | Invidious sometimes succeeds |
| Parse/shape error | ✅ | InnerTube schema drift |
| `UNPLAYABLE` (private/removed/geo) | ❌ | Both backends will fail identically |
| `VIDEO_NOT_FOUND` | ❌ | Definitive |
| `AbortError` (user navigated away) | ❌ | Intentional cancellation |

### 5.3 Implementation

```typescript
// src/api/fallback.ts
import { getVideoInformationLocal } from './local'
import { getVideoInformationInvidious } from './invidious'
import { useSettingsStore } from '@/stores/settings'
import { useUtilsStore } from '@/stores/utils'
import type { VideoInformation } from './types'

const TERMINAL: ReadonlySet<ApiErrorCode> = new Set([
  'UNPLAYABLE', 'VIDEO_NOT_FOUND', 'ABORTED', 'PRIVATE', 'GEO_BLOCKED'
])

function shouldFallback(err: unknown): boolean {
  if (err instanceof ApiError) return !TERMINAL.has(err.code)
  return true   // unknown/network errors are always retryable elsewhere
}

export async function getVideoInformation(
  videoId: string
): Promise<VideoInformation> {
  const settings = useSettingsStore()
  const utils = useUtilsStore()

  const primary = settings.values.backendPreference === 'invidious'
    ? getVideoInformationInvidious
    : getVideoInformationLocal

  const secondary = settings.values.backendPreference === 'invidious'
    ? getVideoInformationLocal
    : getVideoInformationInvidious

  try {
    return await primary(videoId)
  } catch (primaryError) {
    if (!settings.values.backendFallback || !shouldFallback(primaryError)) {
      throw primaryError
    }

    console.warn('[api] primary backend failed, falling back', primaryError)

    try {
      const result = await secondary(videoId)
      utils.showToast(
        `Loaded via ${result.source === 'local' ? 'local API' : 'Invidious'} fallback`,
        { type: 'info', duration: 3000 }
      )
      return result
    } catch (secondaryError) {
      // Surface the primary error — it reflects the user's chosen backend.
      throw new ApiError(
        'ALL_BACKENDS_FAILED',
        'Both backends failed to load this video.',
        { primaryError, secondaryError }
      )
    }
  }
}
```

The same `withFallback` shape wraps channels, search, playlists, and comments.
Extract it once as a higher-order function rather than duplicating per resource:

```typescript
export function withFallback<A extends unknown[], R extends { source: Backend }>(
  local: (...args: A) => Promise<R>,
  invidious: (...args: A) => Promise<R>
): (...args: A) => Promise<R> { /* … */ }

export const getChannel  = withFallback(getLocalChannel,  getInvidiousChannel)
export const getSearch   = withFallback(getLocalSearch,   getInvidiousSearch)
export const getPlaylist = withFallback(getLocalPlaylist, getInvidiousPlaylist)
```

---

## 6. SponsorBlock Client

Runs in the renderer against `https://sponsor.ajay.app`. No proxying needed.

### 6.1 Privacy-preserving lookup

Never send a full video ID. SHA-256 the ID and send the first 4 hex characters;
the API returns all videos in that prefix bucket and the client filters locally.

```typescript
// src/api/sponsorblock.ts
import { useSettingsStore } from '@/stores/settings'

export type SponsorBlockCategory =
  | 'sponsor' | 'selfpromo' | 'interaction' | 'intro' | 'outro'
  | 'preview' | 'music_offtopic' | 'filler'

export type CategoryBehaviour = 'skip' | 'showInSeekBar' | 'doNothing'

export interface SponsorBlockSegment {
  uuid: string
  category: SponsorBlockCategory
  actionType: 'skip' | 'mute' | 'full' | 'poi'
  segment: [number, number]        // seconds
  videoDuration: number
  locked: number
  votes: number
}

async function sha256Hex(input: string): Promise<string> {
  const bytes = new TextEncoder().encode(input)
  const digest = await crypto.subtle.digest('SHA-256', bytes)
  return Array.from(new Uint8Array(digest))
    .map(b => b.toString(16).padStart(2, '0'))
    .join('')
}

export async function getSponsorBlockSegments(
  videoId: string
): Promise<SponsorBlockSegment[]> {
  const settings = useSettingsStore()
  if (!settings.values.useSponsorBlock) return []

  const categories = enabledCategories(settings)
  if (categories.length === 0) return []

  const prefix = (await sha256Hex(videoId)).slice(0, 4)
  const base = settings.values.sponsorBlockUrl || 'https://sponsor.ajay.app'

  const url = new URL(`${base}/api/skipSegments/${prefix}`)
  url.searchParams.set('categories', JSON.stringify(categories))
  url.searchParams.set('actionTypes', JSON.stringify(['skip', 'mute', 'poi']))

  const res = await fetch(url, { headers: { Accept: 'application/json' } })
  if (res.status === 404) return []          // no segments — not an error
  if (!res.ok) throw new ApiError('SPONSORBLOCK_ERROR', `HTTP ${res.status}`)

  const buckets = await res.json() as Array<{
    videoID: string
    segments: SponsorBlockSegment[]
  }>

  return buckets.find(b => b.videoID === videoId)?.segments ?? []
}
```

### 6.2 Skip execution

Skipping lives in the player component, driven by `timeupdate`. Segments are
sorted once and tracked by index so lookup stays O(1) per tick.

```typescript
// inside ft-shaka-video-player
function onTimeUpdate(currentTime: number): void {
  for (const seg of activeSegments.value) {
    const [start, end] = seg.segment
    if (currentTime < start || currentTime >= end) continue
    if (skippedUuids.has(seg.uuid)) continue

    const behaviour = categoryBehaviour(seg.category)
    if (behaviour === 'skip') {
      skippedUuids.add(seg.uuid)
      video.currentTime = end
      if (settings.values.sponsorBlockShowSkippedToast) {
        utils.showToast(`Skipped ${categoryLabel(seg.category)}`, {
          type: 'info',
          duration: 2500,
          action: { label: 'Undo', handler: () => { video.currentTime = start } }
        })
      }
    } else if (seg.actionType === 'mute') {
      video.muted = true
      muteUntil.value = end
    }
  }
}
```

Seek-bar markers render as absolutely-positioned divs over the Shaka progress
bar, coloured per category.

### 6.3 Behaviour matrix

Each category is independently configurable in Settings → SponsorBlock:

| Category | Default | Colour |
|----------|---------|--------|
| `sponsor` | skip | `#00d400` |
| `selfpromo` | showInSeekBar | `#ffff00` |
| `interaction` | showInSeekBar | `#cc00ff` |
| `intro` | showInSeekBar | `#00ffff` |
| `outro` | showInSeekBar | `#0202ed` |
| `preview` | showInSeekBar | `#008fd6` |
| `music_offtopic` | showInSeekBar | `#ff9900` |
| `filler` | doNothing | `#7300ff` |

### 6.4 Caching

Segments are cached in-memory per video for the session (`Map<videoId, Segment[]>`),
capped at 200 entries with LRU eviction. Not persisted — segments change
upstream and staleness causes wrong skips.

---

## 7. PoToken via Tauri Command

### 7.1 Why Rust owns this

PoToken (Proof of Origin Token) requires executing YouTube's BotGuard VM in a
real browser environment. The main webview cannot do it: BotGuard needs a clean
`youtube.com` origin with an untouched global scope, and running it in the app's
webview both pollutes that scope and risks detection.

Tauri creates a **hidden `WebviewWindow`** on `youtube.com`, runs the challenge
there, and returns the token. See
[../backend/02-tauri-commands.md](../backend/02-tauri-commands.md) §PoToken.

### 7.2 Renderer interface

```typescript
// src/api/potoken.ts
import { invoke } from '@tauri-apps/api/core'

export interface PoTokenResponse {
  token: string
  visitorData: string
  expiresAt: number      // epoch ms
  cached: boolean
}

let cached: PoTokenResponse | null = null
let inflight: Promise<PoTokenResponse> | null = null

const SAFETY_MARGIN_MS = 5 * 60_000   // refresh 5 min before expiry

function isFresh(t: PoTokenResponse | null): t is PoTokenResponse {
  return t !== null && Date.now() < t.expiresAt - SAFETY_MARGIN_MS
}

export async function getPoToken(
  options: { force?: boolean } = {}
): Promise<PoTokenResponse> {
  if (!options.force && isFresh(cached)) return cached
  if (inflight) return inflight            // de-duplicate concurrent callers

  inflight = (async () => {
    try {
      const result = await invoke<PoTokenResponse>('get_potoken', {
        force: options.force ?? false
      })
      cached = result
      return result
    } finally {
      inflight = null
    }
  })()

  return inflight
}

export async function invalidatePoToken(): Promise<void> {
  cached = null
  await invoke('clear_potoken_cache')
}
```

### 7.3 Lifecycle

```
App start
   └─► getPoToken()               ← blocks first Innertube creation
          ├─ cached & fresh  → return immediately (~1 ms)
          └─ stale/absent    → Rust spawns hidden webview (~2–5 s)

Playback 403
   └─► invalidatePoToken() → resetInnertube() → retry once
          └─ still failing  → fall back to Invidious

Every 45 min (token TTL ≈ 6 h, refreshed early)
   └─► background getPoToken() refresh, non-blocking
```

### 7.4 Failure handling

PoToken generation **must never block the app**. If it fails:

```typescript
export async function getPoTokenSafe(): Promise<PoTokenResponse | null> {
  try {
    return await getPoToken()
  } catch (err) {
    console.error('[potoken] generation failed', err)
    useUtilsStore().showToast(
      'Could not verify with YouTube — some videos may need the Invidious backend.',
      { type: 'warning', duration: 6000 }
    )
    return null
  }
}
```

`Innertube.create` then proceeds without `po_token`. Most videos still play;
the ones that don't fall through to Invidious via §5.

### 7.5 Startup sequence

```typescript
// App.vue onMounted
await settingsStore.load()                  // backend prefs first

// Non-blocking — the UI renders while the token generates.
void getPoTokenSafe().then(() => {
  if (settingsStore.values.backendPreference === 'local') {
    void getInnertube()                     // warm the session
  }
})

isBootstrapped.value = true
```

---

## 8. Error Taxonomy

```typescript
// src/api/types.ts
export type ApiErrorCode =
  | 'NETWORK'              // fetch/DNS/timeout
  | 'RATE_LIMITED'         // 429
  | 'AGE_RESTRICTED'       // login required
  | 'UNPLAYABLE'           // removed / private / terminated
  | 'PRIVATE'
  | 'GEO_BLOCKED'
  | 'VIDEO_NOT_FOUND'
  | 'PARSE'                // InnerTube schema drift
  | 'POTOKEN'              // token generation failed
  | 'INVIDIOUS_ERROR'      // instance-side failure
  | 'SPONSORBLOCK_ERROR'
  | 'ALL_BACKENDS_FAILED'
  | 'ABORTED'
  | 'UNKNOWN'

export class ApiError extends Error {
  constructor(
    public readonly code: ApiErrorCode,
    message: string,
    public readonly cause?: unknown
  ) {
    super(message)
    this.name = 'ApiError'
  }
}
```

| Code | User-facing message | Recovery |
|------|--------------------|----------|
| `NETWORK` | "Check your connection." | Retry button |
| `RATE_LIMITED` | "Too many requests — try again shortly." | Auto-retry w/ backoff |
| `AGE_RESTRICTED` | "This video is age-restricted." | Auto-fallback to Invidious |
| `UNPLAYABLE` / `PRIVATE` | "This video is unavailable." | None — terminal |
| `GEO_BLOCKED` | "Unavailable in your region." | Suggest proxy/instance change |
| `PARSE` | "Couldn't read YouTube's response." | Fallback + "report" link |
| `POTOKEN` | "Verification failed." | Retry, or switch backend |
| `ALL_BACKENDS_FAILED` | "Both backends failed." | Retry + open settings |

---

## 9. Testing Requirements

| Area | Test |
|------|------|
| Fetch wrapper | Forbidden headers route to `proxy_fetch`; allowed-only headers use native `fetch` |
| Fetch wrapper | `ProxyResponse` → `Response` conversion preserves status/headers/body |
| Fallback | Each terminal code does **not** fall back; each retryable code does |
| Fallback | `backendFallback: false` never calls the secondary |
| Normalisation | Local and Invidious fixtures produce structurally identical `VideoInformation` |
| SponsorBlock | Hash prefix bucket filtering returns only the requested video |
| SponsorBlock | Each `CategoryBehaviour` produces the correct player action |
| PoToken | Concurrent `getPoToken()` calls yield one `invoke` |
| PoToken | Expiry margin triggers regeneration |
| PoToken | Generation failure degrades gracefully, does not throw to the view |

Fixtures live in `src/api/__fixtures__/` and are captured from real responses
with IDs redacted.

---

## References

- [youtubei.js documentation](https://ytjs.dev/)
- [Invidious API docs](https://docs.invidious.io/api/)
- [SponsorBlock API](https://wiki.sponsor.ajay.app/w/API_Docs)
- [MDN — Forbidden header name](https://developer.mozilla.org/en-US/docs/Glossary/Forbidden_header_name)
- [../backend/02-tauri-commands.md](../backend/02-tauri-commands.md) — `proxy_fetch`, `get_potoken`
- [../architecture/03-data-flow.md](../architecture/03-data-flow.md) — IPC patterns
- [01-store-migration.md](01-store-migration.md) — Stores consuming this layer
- [03-view-migration-order.md](03-view-migration-order.md) — Watch view integration

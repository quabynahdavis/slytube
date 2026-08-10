# 02 — Component Mapping: OpenTubeX (Electron) → Slytube (Tauri v2)

> **Status:** Authoritative reference
> **Source:** `/home/davisville/Contributions/opentubex/src/**`
> **Target:** `/home/davisville/Contributions/slytube/{src,src-tauri}/**`
> **Related:** [`01-electron-vs-tauri.md`](01-electron-vs-tauri.md) · [`03-data-flow.md`](03-data-flow.md) · [`../backend/01-database-schema.md`](../backend/01-database-schema.md) · [`../backend/02-tauri-commands.md`](../backend/02-tauri-commands.md)

---

## 0. Master Mapping Table

| OpenTubeX source | Lines | Slytube destination | Strategy |
|---|---:|---|---|
| `src/main/index.js` | 4,558 | `src-tauri/src/{lib.rs,commands/,state/,window/,net/,menu/}` | Decompose into ~10 Rust modules |
| `src/main/tabs/TabManager.js` | 3,046 | `src-tauri/src/tabs/` + `src/stores/tabs.ts` | Redesign (no `WebContentsView` parity) |
| `src/main/ytDlp.js` | 1,375 | `src-tauri/src/commands/downloads.rs` + `sidecar/` | Port + sidecar binary |
| `src/preload/interface.js` | 1,023 | **deleted** → `src/lib/ipc.ts` | Eliminate; direct `invoke` |
| `src/preload/main.js` | 4 | **deleted** | Eliminate |
| `src/constants.js` (`IpcChannels`) | 627 | `src-tauri/src/events.rs` + `src/lib/events.ts` | 110 channels → commands + event names |
| `src/datastores/**` (NeDB) | 1,128 | `src-tauri/src/db/` (`sqlx` + SQLite) | Replace engine, keep semantics |
| `src/main/poTokenGenerator.js` | 219 | `src-tauri/src/potoken/` + hidden `WebviewWindow` | Redesign |
| `src/botGuardScript.js` | ~90 | `src-tauri/resources/botguard.js` | Reuse verbatim, new injection host |
| `src/main/externalPlayer.js` | 209 | `src-tauri/src/commands/external_player.rs` | Direct port |
| `src/main/favicon.js` | 112 | `src-tauri/src/net/favicon.rs` | Direct port |
| `src/main/ImageCache.js` | 73 | `src-tauri/src/net/image_cache.rs` (`moka`) | Direct port + disk tier |
| `src/renderer/store/modules/*.js` | 5,582 | `src/stores/*.ts` (Pinia) | 14 modules → 14 stores |
| `src/renderer/store/index.js` | 100 | `src/stores/plugins/sync-on-change.ts` | Vuex plugin → Pinia plugin |
| `src/renderer/helpers/api/local.js` | 2,573 | `src/services/youtube/` (webview, Phase 1) | Keep in webview |
| `src/renderer/helpers/api/invidious.js` | 1,009 | `src-tauri/src/services/invidious.rs` | Move to Rust (Phase 2) |
| `src/renderer/helpers/sync-server*.js` | 1,569 | `src-tauri/src/sync/` | Move to Rust crypto |
| `src/renderer/tabs/*.js` | 1,262 | `src/services/tabs/` | Port alongside tab redesign |
| `src/renderer/components/**` | 118 files | `src/components/**` (shadcn-vue + reka-ui) | Rebuild on primitives |
| `src/renderer/views/**` | 17 areas | `src/views/**` | Port with store rewiring |

---

## 1. Main Process → Rust Command Modules

### 1.1 What is inside `src/main/index.js`

4,558 lines, 44 imports, **52 `ipcMain` registrations** across **50 distinct channels** (`handle` = request/response, `on` = fire-and-forget):

```
handle: CHOOSE_IP_BLOCK_RECOVERY_SCRIPT, CONTEXT_MENU_EXECUTE, CONTEXT_MENU_OPEN,
        DB_HISTORY, DB_PLAYLISTS, DB_PROFILES, DB_SEARCH_HISTORY, DB_SETTINGS,
        DB_SUBSCRIPTION_CACHE, DB_WATCH_STATS, EXECUTE_IP_BLOCK_RECOVERY_SCRIPT,
        GENERATE_PO_TOKEN, GET_NAVIGATION_HISTORY, GET_REPLACE_HTTP_CACHE,
        GET_SYSTEM_LOCALE, IS_WAYLAND_PLATFORM, PLAYER_CACHE_GET, PLAYER_CACHE_SET,
        RESOLVE_FAVICON, START_IP_BLOCK_RECOVERY_SCRIPT,
        SUBSCRIPTION_AUTO_REFRESH_{ACQUIRE,GET_STATE,RELEASE},
        WAIT_FOR_IP_BLOCK_RECOVERY_SCRIPT, WRITE_TO_DEFAULT_FOLDER,
        YT_DLP_{CHOOSE_DOWNLOAD_FOLDER,CHOOSE_EXECUTABLE,CLEAR_DOWNLOADS,DOWNLOAD,
                DOWNLOAD_BINARY,GET_INFO,GET_PLAYBACK_INFO,LIST_DOWNLOADS,
                OPEN_DOWNLOAD,REMOVE_DOWNLOAD}
on:     APP_READY, CHOOSE_DEFAULT_FOLDER, CREATE_NEW_TAB, CREATE_NEW_WINDOW,
        DISABLE_PROXY, ENABLE_PROXY, OPEN_IN_EXTERNAL_PLAYER,
        SET_INVIDIOUS_AUTHORIZATION, SET_WINDOW_TITLE, SHOW_TOAST,
        START_POWER_SAVE_BLOCKER, STOP_POWER_SAVE_BLOCKER,
        SUBSCRIPTION_AUTO_REFRESH_{CANCEL,SET_PROGRESS}, YT_DLP_CANCEL_DOWNLOAD
```

The remaining 60 channels are registered elsewhere — chiefly `setupTabsIPC()` in `TabManager.js`.

### 1.2 Target module tree

```
src-tauri/src/
├── main.rs                     # thin: slytube_lib::run()
├── lib.rs                      # Builder, plugins, .manage(), generate_handler!
├── error.rs                    # AppError → serde-serialisable ApiError
├── events.rs                   # event-name constants (mirrors src/lib/events.ts)
├── state/
│   ├── mod.rs                  # AppState { db, http, downloads, potoken, sync, subs }
│   └── subscriptions.rs        # auto-refresh lock  ← SUBSCRIPTION_AUTO_REFRESH_* (7)
├── db/                         # ← src/datastores/**            (§3)
├── commands/
│   ├── app.rs                  # GET_SYSTEM_LOCALE, IS_WAYLAND_PLATFORM,
│   │                           #   RELAUNCH_REQUEST, APP_READY, runtimeVersions
│   ├── settings.rs             # ← DB_SETTINGS      (7 DBActions)
│   ├── history.rs              # ← DB_HISTORY       (GENERAL + 4 HISTORY actions)
│   ├── watch_stats.rs          # ← DB_WATCH_STATS   (GENERAL + 4)
│   ├── profiles.rs             # ← DB_PROFILES      (GENERAL + 2)
│   ├── playlists.rs            # ← DB_PLAYLISTS     (GENERAL + 5)
│   ├── search_history.rs       # ← DB_SEARCH_HISTORY
│   ├── subscription_cache.rs   # ← DB_SUBSCRIPTION_CACHE (GENERAL + 5)
│   ├── downloads.rs            # ← src/main/ytDlp.js            (§4)
│   ├── potoken.rs              # ← src/main/poTokenGenerator.js (§5)
│   ├── sync.rs                 # ← helpers/sync-server*.js      (§7)
│   ├── proxy.rs                # ENABLE_PROXY, DISABLE_PROXY,
│   │                           #   SET_INVIDIOUS_AUTHORIZATION
│   ├── fs.rs                   # CHOOSE_DEFAULT_FOLDER, WRITE_TO_DEFAULT_FOLDER,
│   │                           #   *_IP_BLOCK_RECOVERY_SCRIPT (4)
│   ├── external_player.rs      # ← src/main/externalPlayer.js
│   ├── player_cache.rs         # PLAYER_CACHE_GET / PLAYER_CACHE_SET
│   ├── tabs.rs                 # ← TabManager.js                (§8)
│   └── window.rs               # CREATE_NEW_WINDOW, SET_WINDOW_TITLE,
│                               #   POWER_SAVE_BLOCKER, setZoomFactor
├── net/
│   ├── client.rs               # shared reqwest::Client + proxy + header policy
│   ├── favicon.rs              # ← src/main/favicon.js  (RESOLVE_FAVICON)
│   └── image_cache.rs          # ← src/main/ImageCache.js
├── services/
│   └── invidious.rs            # ← helpers/api/invidious.js (Phase 2)
├── sync/
│   ├── crypto.rs               # PBKDF2-SHA256 + AES-GCM-256 envelope
│   └── client.rs               # sync-server REST client
├── potoken/
│   ├── mod.rs
│   └── webview.rs              # hidden WebviewWindow lifecycle
└── sidecar/
    └── ytdlp.rs                # binary resolution, version probe, spawn
```

### 1.3 Global state: module-level `let` → `tauri::State`

OpenTubeX keeps mutable state as module globals:

```js
// src/main/ytDlp.js:110-118
let downloadCounter = 0
const activeDownloads = new Map()   // Map<number, { child, cancelled }>
const downloadRecords = new Map()   // Map<number, YtDlpDownloadStatus>

// src/main/poTokenGenerator.js:11, 117-120
let queueGuardian = Promise.resolve()
let theSession
let cachedScript
```

```rust
// src-tauri/src/state/mod.rs
pub struct AppState {
    pub db:        SqlitePool,
    pub http:      RwLock<reqwest::Client>,          // rebuilt when proxy changes
    pub downloads: Mutex<DownloadRegistry>,          // ← activeDownloads + downloadRecords
    pub dl_seq:    AtomicU64,                        // ← downloadCounter
    pub potoken:   Mutex<PoTokenState>,              // ← queueGuardian + cachedScript
    pub subs:      Mutex<SubscriptionRefreshState>,  // ← SUBSCRIPTION_AUTO_REFRESH_*
    pub player_cache: DashMap<String, PlayerCacheEntry>,
}
```

`queueGuardian`'s promise-chain serialisation (`poTokenGenerator.js:13-31`) becomes a `tokio::sync::Mutex` held across the await — same semantics, enforced by the type system.

### 1.4 Signature conventions

```rust
// Request/response — was ipcMain.handle
#[tauri::command]
pub async fn get_settings(state: State<'_, AppState>) -> Result<Settings, ApiError>

// Fire-and-forget — was ipcMain.on; now returns Result so failures are observable
#[tauri::command]
pub async fn set_window_title(win: Window, title: String, tab_id: Option<String>)
    -> Result<(), ApiError>

// Streaming — was webContents.send in a loop
#[tauri::command]
pub async fn start_download(
    state: State<'_, AppState>,
    payload: DownloadRequest,
    on_progress: Channel<DownloadStatus>,
) -> Result<u64, ApiError>
```

---

## 2. Preload → Removed

### 2.1 What disappears

```js
// src/preload/main.js — the whole file
import { contextBridge } from 'electron/renderer'
import api from './interface.js'
contextBridge.exposeInMainWorld('ftElectron', api)
```

`src/preload/interface.js` exposes **70 members**: 48 `ipcRenderer.invoke`, 39 `ipcRenderer.send`, 25 `ipcRenderer.on`, 12 `removeListener`, plus manual listener registries (`currentUpdateSearchInputTextListener`, `ytDlpBinaryDownloadProgressListeners`) and static metadata (`isFlatpak`, `runtimeVersions`).

### 2.2 Complete member mapping

| `window.ftElectron.*` | IPC channel | Slytube replacement |
|---|---|---|
| `isFlatpak` | — (`process.env.FLATPAK_ID`) | `get_app_info().is_flatpak` |
| `runtimeVersions` | — (`process.versions`) | `get_app_info().versions` (tauri/webview/rust) |
| `setWindowTitle` | `SET_WINDOW_TITLE` | `invoke('set_window_title')` |
| `getSystemLocale` | `GET_SYSTEM_LOCALE` | `invoke('get_system_locale')` |
| `isWaylandPlatform` | `IS_WAYLAND_PLATFORM` | `invoke('is_wayland_platform')` |
| `openInNewWindow` | `CREATE_NEW_WINDOW` | `new WebviewWindow(...)` or `invoke('create_window')` |
| `enableProxy` / `disableProxy` | `ENABLE_PROXY` / `DISABLE_PROXY` | `invoke('set_proxy' \| 'clear_proxy')` |
| `setInvidiousAuthorization` / `clearInvidiousAuthorization` | `SET_INVIDIOUS_AUTHORIZATION` | `invoke('set_invidious_authorization')` |
| `startPowerSaveBlocker` / `stopPowerSaveBlocker` | `*_POWER_SAVE_BLOCKER` | `invoke('set_power_save_blocker')` |
| `getReplaceHttpCache` / `toggleReplaceHttpCache` | `GET_/TOGGLE_REPLACE_HTTP_CACHE` | `invoke('get_/toggle_http_cache_replacement')` |
| `requestPiP` | `TABS_REQUEST_PICTURE_IN_PICTURE` | Web API in webview + `invoke('tabs_request_pip')` |
| `requestFullscreen` | `TABS_REQUEST_FULLSCREEN` | `getCurrentWindow().setFullscreen()` |
| `handleWindowMinimizedState` | `WINDOW_MINIMIZED_STATE` | `getCurrentWindow().onResized()` / window events |
| `playerCacheGet` / `playerCacheSet` | `PLAYER_CACHE_GET/SET` | `invoke('player_cache_get' \| 'player_cache_set')` |
| `generatePoToken` | `GENERATE_PO_TOKEN` | `invoke('get_potoken')` — see §5 |
| `chooseDefaultFolder` | `CHOOSE_DEFAULT_FOLDER` | `tauri-plugin-dialog` `open({ directory: true })` |
| `chooseIpBlockRecoveryScript` | `CHOOSE_IP_BLOCK_RECOVERY_SCRIPT` | `dialog.open()` + `invoke('set_recovery_script')` |
| `writeToDefaultFolder` | `WRITE_TO_DEFAULT_FOLDER` | `invoke('write_to_default_folder')` (scoped `fs`) |
| `startIpBlockRecoveryScript` / `execute…` / `waitFor…` | 3 channels | `invoke('*_ip_block_recovery_script')` |
| `relaunch` | `RELAUNCH_REQUEST` | `tauri::process::restart()` via `invoke('relaunch')` |
| `openInExternalPlayer` / `handleOpenInExternalPlayerResult` | 2 channels | `invoke('open_in_external_player')` → `Result` (no result channel needed) |
| `ytDlpDownload` | `YT_DLP_DOWNLOAD` | `invoke('start_download', { onProgress: channel })` |
| `ytDlpCancelDownload` | `YT_DLP_CANCEL_DOWNLOAD` | `invoke('cancel_download')` |
| `ytDlpOpenDownload` / `ytDlpRemoveDownload` | 2 channels | `invoke('open_download' \| 'delete_download')` |
| `ytDlpListDownloads` / `ytDlpClearDownloads` | 2 channels | `invoke('get_downloads' \| 'clear_downloads')` |
| `handleYtDlpDownloadStatus` | `YT_DLP_DOWNLOAD_STATUS` | `Channel<DownloadStatus>` + `downloads://status` broadcast |
| `handleYtDlpDownloadsRemoved` | `YT_DLP_DOWNLOADS_REMOVED` | `downloads://removed` event |
| `ytDlpChooseDownloadFolder` / `ytDlpChooseExecutable` | 2 channels | `tauri-plugin-dialog` |
| `ytDlpGetInfo` / `ytDlpGetPlaybackInfo` | 2 channels | `invoke('yt_dlp_get_info' \| 'yt_dlp_get_playback_info')` |
| `ytDlpDownloadBinary` | `YT_DLP_DOWNLOAD_BINARY` | `invoke('download_ytdlp_binary', { onProgress })` |
| `setYtDlpBinaryDownloadProgressListener` / `add…` | `YT_DLP_BINARY_DOWNLOAD_PROGRESS` | Folded into the `Channel` above — registry deleted |
| `setZoomFactor` | — (`webFrame.setZoomFactor`) | CSS `zoom` / `document.documentElement.style` in webview |
| `getNavigationHistory` | `GET_NAVIGATION_HISTORY` | Pinia tabs store (client-side history model) |
| `resolveFavicon` | `RESOLVE_FAVICON` | `invoke('resolve_favicon')` |
| `dbSettings` … `dbSubscriptionCache` (7) | `DB_*` | ~40 typed commands — see §3 |
| `handleChangeView` | `CHANGE_VIEW` | `app://change-view` event |
| `handleOpenUrl` | `OPEN_URL` | `app://open-url` event (deep links) |
| `showToastOnAllTabs` / `handleShowToast` | `SHOW_TOAST` | `ui://toast` broadcast |
| `subscriptionAutoRefresh` (namespace) | 6 channels | `subscriptions::*` commands + `subs://state-changed` |
| `subscriptionFeeds` | `SUBSCRIPTION_FEED_REQUEST_RELOAD` | `subs://feed-reload` event |
| `contextMenu` | `CONTEXT_MENU_OPEN/EXECUTE` | `tauri::menu` native menus |
| `handleUpdateSearchInputText` | `UPDATE_SEARCH_INPUT_TEXT` | `ui://search-input` event |
| `handleSync*` (7) | `SYNC_*` | `sync://{collection}-changed` events |
| `tabs` (namespace) | 45 channels | `tabs::*` commands + `tabs://*` events — see §8 |

### 2.3 The thin replacement

Rather than scattering raw `invoke` calls, Slytube keeps one typed façade — the same ergonomic benefit as the preload, without the security cost:

```ts
// src/lib/ipc.ts
import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import type { Settings, DownloadStatus, TabState } from './types'

export const ipc = {
  settings: {
    getAll: () => invoke<Settings>('get_settings'),
    upsert: (key: string, value: unknown) => invoke<void>('settings_upsert', { key, value }),
  },
  downloads: {
    list:   () => invoke<DownloadStatus[]>('get_downloads'),
    cancel: (id: number) => invoke<void>('cancel_download', { id }),
  },
  potoken: {
    get: (videoId: string, context: unknown) =>
      invoke<string>('get_potoken', { videoId, context }),
  },
} as const

export const on = {
  tabsStateUpdated: (cb: (s: TabState) => void): Promise<UnlistenFn> =>
    listen<TabState>('tabs://state-updated', e => cb(e.payload)),
}
```

Key difference from the preload: `src/lib/ipc.ts` is **ordinary frontend code with no privilege**. It cannot widen the attack surface — only `generate_handler!` and `capabilities/*.json` can.

---

## 3. Datastores: NeDB → SQLite via `sqlx`

### 3.1 Current shape

```js
// src/datastores/index.js:28-44
function createDatastore(name) {
  return new Datastore({ filename: dbPath(name), autoload: !process.env.IS_ELECTRON_MAIN,
                         corruptAlertThreshold: 1 })
}
export const settings = createDatastore('settings')
export const profiles = createDatastore('profiles')
export const playlists = createDatastore('playlists')
export const history = createDatastore('history')
export const watchStats = createDatastore('watch-stats')
export const searchHistory = createDatastore('search-history')
export const subscriptionCache = createDatastore('subscription-cache')
export const tabSession = createDatastore('tab-session')
```

Eight append-only JSON-lines files in `app.getPath('userData')`. Handlers are three-tier: `base.js` (775 L, real implementations), `electron.js` (273 L, renderer→IPC proxies), `web.js` (27 L, browser build), selected by the webpack alias `DB_HANDLERS_ELECTRON_RENDERER_OR_WEB` (`handlers/index.js`).

### 3.2 The `DBActions` surface

Every `DB_*` call carries a numeric action. Full enumeration from `src/constants.js`:

| Group | Actions |
|---|---|
| `GENERAL` (all stores) | `CREATE(0)`, `FIND(1)`, `UPSERT(2)`, `DELETE(3)`, `DELETE_MULTIPLE(4)`, `DELETE_ALL(5)`, `OVERWRITE(6)` |
| `HISTORY` | `UPDATE_WATCH_PROGRESS(20)`, `UPDATE_PLAYLIST(21)`, `DELETE_OLDER_THAN(22)`, `APPLY_SYNC_CHANGES(23)` |
| `WATCH_STATS` | `ADD_WATCH_TIME(20)`, `MIGRATE_HISTORY(21)`, `GET_HISTORICAL_ADJUSTMENT(22)`, `ADJUST_HISTORICAL_WATCH_TIME(23)` |
| `PROFILES` | `ADD_CHANNEL(20)`, `REMOVE_CHANNEL(21)` |
| `PLAYLISTS` | `UPSERT_VIDEO(20)`, `UPSERT_VIDEOS(21)`, `DELETE_VIDEO_ID(22)`, `DELETE_VIDEO_IDS(23)`, `DELETE_ALL_VIDEOS(24)` |
| `SUBSCRIPTION_CACHE` | `UPDATE_VIDEOS_BY_CHANNEL(20)`, `UPDATE_LIVE_STREAMS_BY_CHANNEL(21)`, `UPDATE_SHORTS_BY_CHANNEL(22)`, `UPDATE_SHORTS_WITH_CHANNEL_PAGE_SHORTS_BY_CHANNEL(23)`, `UPDATE_COMMUNITY_POSTS_BY_CHANNEL(24)` |

**27 leaf actions across 6 groups**, all funnelled through 7 channels.

### 3.3 Mapping

| NeDB store | SQLite table(s) | Rust module | Commands |
|---|---|---|---|
| `settings.db` | `settings(key TEXT PK, value TEXT)` | `db/settings.rs` | `get_settings`, `settings_upsert`, `settings_delete`, `settings_overwrite` |
| `profiles.db` | `profiles`, `profile_channels` | `db/profiles.rs` | + `profiles_add_channel`, `profiles_remove_channel` |
| `playlists.db` | `playlists`, `playlist_videos` | `db/playlists.rs` | + `playlists_upsert_video(s)`, `playlists_delete_video_id(s)`, `playlists_delete_all_videos` |
| `history.db` | `history` | `db/history.rs` | + `history_update_watch_progress`, `history_update_playlist`, `history_delete_older_than`, `history_apply_sync_changes` |
| `watch-stats.db` | `watch_stats` | `db/watch_stats.rs` | + `watch_stats_add_time`, `watch_stats_migrate_history`, `watch_stats_get_historical_adjustment`, `watch_stats_adjust_historical` |
| `search-history.db` | `search_history` | `db/search_history.rs` | GENERAL only |
| `subscription-cache.db` | `subscription_cache` (+ per-kind columns) | `db/subscription_cache.rs` | + 5 `UPDATE_*_BY_CHANNEL` |
| `tab-session.db` | `tab_sessions` | `db/tabs.rs` | `tabs_get_sync_sessions`, `tabs_apply_sync_sessions` |

Normalisation decisions (embedded arrays like `playlist.videos` and `profile.subscriptions` → child tables) are specified in [`../backend/01-database-schema.md`](../backend/01-database-schema.md).

### 3.4 What improves

- **`toPlain()` deleted.** `handlers/electron.js:12` does a full `JSON.parse(JSON.stringify(...))` on every payload solely to strip Vue proxies before structured clone. Tauri's `serde` boundary needs no such shim.
- **Typed payloads.** `dbPlaylists(23, x)` becomes `playlists_delete_video_ids(playlist_id, video_ids)` — checked at compile time.
- **Real queries.** `history.js` (280 L) and `subscription-cache.js` (491 L) currently filter whole collections in JS; these become indexed SQL.
- **Transactions.** `UPSERT_VIDEOS` (bulk) becomes one transaction instead of N append-log writes.
- **Corruption handling.** NeDB's `corruptAlertThreshold: 1` workaround is replaced by SQLite's WAL + integrity guarantees.

### 3.5 One-shot import

A read-only importer parses each `.db` JSON-lines file, replays it with last-write-wins per `_id`, writes into SQLite inside a transaction, then records `schema_meta.imported_from_nedb = 1`. Original `.db` files are copied to `userData/nedb-backup/` and never mutated.

---

## 4. `ytDlp.js` → Sidecar Binary + Commands

### 4.1 What must be ported (1,375 lines)

| Concern | Source anchor | Rust target |
|---|---|---|
| Binary resolution (managed vs. user path) | `resolveExecutable` `:242` | `sidecar/ytdlp.rs::resolve()` |
| Managed binary directory | `getManagedBinariesDirectory` `:223` | `app.path().app_local_data_dir()` |
| Release channels | `YT_DLP_RELEASE_REPOSITORIES` `:100` (`stable`/`nightly`/`master`) | Same constants |
| Version probes | `getYtDlpVersion` `:290`, `getFfmpegVersion` `:309` | `Command::new_sidecar(..).args(["--version"])` |
| Binary download + progress | `downloadFile` `:344`, `downloadManagedYtDlp` `:520`, `downloadManagedFfmpeg` `:567` | `reqwest` stream + `Channel<BinaryProgress>` |
| Validators (checksum/etag) | `readDownloadValidators` `:405`, `writeDownloadValidators` `:424` | `sidecar/validators.rs` |
| Zip extraction (ffmpeg) | `extractZipEntry` `:440`, `installBinary` `:499` | `zip` crate |
| Proxy injection | `getProxyUrl` `:260`, `pushProxyArgument` `:277` | Read from `AppState.http` proxy config |
| Info probe + dedupe | `getInfoProbeKey` `:647`, `handleYtDlpGetInfo` `:665` | `commands/downloads.rs` + `DashMap` in-flight map |
| Abort signals | `takeGetInfoAbortSignal` `:24` | `tokio_util::sync::CancellationToken` |
| Playback info | `handleYtDlpGetPlaybackInfo` `:864`, `mapPlaybackFormat` `:832` | Same JSON mapping |
| Argument builder | `handleYtDlpDownload` `:951+`, `splitArguments` `:934` | `downloads/args.rs` |
| Spawn + stream | `spawn(executable, args, { windowsHide: true })` `:1134` | `Command::spawn()` → `CommandEvent::{Stdout,Stderr,Terminated}` |
| Progress parsing | `PROGRESS_REGEX`, `DESTINATION_REGEX`, `MERGER_REGEX`, `FINAL_PATH_PREFIX` `:105-108` | `once_cell::Lazy<Regex>` |
| Throttling | `sendStatus()` 500 ms gate `:1163-1173` | `tokio::time::Interval` (gate can be relaxed) |
| Record persistence | `loadDownloadRecords` `:132`, `saveDownloadRecords` `:151` (last 200, non-active) | `downloads` SQLite table |
| Cancel | `handleYtDlpCancelDownload` `:1280` | `CancellationToken` + `child.kill()` |
| Graceful shutdown | `shutdownYtDlpDownloads` `:167` | Tauri `RunEvent::ExitRequested` handler |

### 4.2 Status payload — preserved shape

The `YtDlpDownloadStatus` object built at `ytDlp.js:1140-1157` is the contract with the Downloads UI. Keep it field-for-field:

```rust
#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DownloadStatus {
    pub id: u64,
    pub video_id: String,
    pub playlist_id: String,      // truncated to 128 chars, as today
    pub playlist_key: String,     // truncated to 255
    pub title: String,            // truncated to 255
    pub thumbnail: String,        // truncated to 2048
    pub status: DownloadState,    // downloading | processing | completed | failed | cancelled
    pub percent: f32,
    pub speed: Option<String>,
    pub eta: Option<String>,
    pub destination: Option<String>,
    pub destinations: Vec<String>,
    pub error_message: Option<String>,
}
```

### 4.3 Sidecar declaration

```jsonc
// src-tauri/tauri.conf.json
{
  "bundle": {
    "externalBin": ["binaries/yt-dlp", "binaries/ffmpeg"]
  }
}
```
```jsonc
// src-tauri/capabilities/main.json
{ "identifier": "shell:allow-execute",
  "allow": [{ "name": "binaries/yt-dlp",  "sidecar": true, "args": true },
            { "name": "binaries/ffmpeg", "sidecar": true, "args": true }] }
```

Tauri resolves `binaries/yt-dlp-x86_64-unknown-linux-gnu` etc. by target triple at build time.

### 4.4 Broadcast replaced

```js
// src/main/ytDlp.js:120 — manual fan-out with an origin check
function broadcastToRenderers(channel, payload) {
  for (const browserWindow of BrowserWindow.getAllWindows()) {
    if (!browserWindow.webContents.isDestroyed() &&
        isOpenTubeXUrl(browserWindow.webContents.getURL())) {
      browserWindow.webContents.send(channel, payload)
    }
  }
}
```

Replaced by two mechanisms: a per-invocation `Channel<DownloadStatus>` for the initiating view, and `app.emit("downloads://status", …)` for other windows. The `isOpenTubeXUrl` guard becomes unnecessary — capabilities already restrict which windows may listen.

### 4.5 Open decision

macOS notarisation requires bundled binaries to be signed. OpenTubeX's runtime "managed download" flow (`downloadManagedYtDlp`) produces unsigned executables in app data. Options: (a) bundle-only, drop runtime updates; (b) bundle + allow runtime updates outside the app bundle, accepting a Gatekeeper prompt; (c) platform-conditional. **Must be decided before Phase 3** — record in `docs/decisions/`.

---

## 5. `poTokenGenerator.js` → Hidden Webview + Command

### 5.1 Current implementation

219 lines, three cooperating parts:

1. **Serialisation queue** (`:11-31`) — `enqueueAsyncFunction` chains promises so only one generation runs at a time, and a `cleanupSession` task is enqueued after each.
2. **Session hardening** (`sharedInit`, `:122-190`) — `session.fromPartition('potoken', { cache: false })`; permission check/request handlers return `false`; UA copied from the default session; `onBeforeSendHeaders` sets `Referer`/`Origin`/`Sec-Fetch-*`/`X-Youtube-Bootstrap-Logged-In` for `youtubei` and script requests; `onHeadersReceived` injects permissive CORS; `onBeforeRequest` cancels `cspReport` and `ping`. Then the botGuard bundle is read and its ESM export line is rewritten to an immediate call: `.replace(/export{(\w+) as default};/, ';$1(FT_PARAMS)')`.
3. **Execution** (`internalGeneratePotoken`, `:150-205`) — optional proxy; `WebContentsView` with `sandbox: true, contextIsolation: true, offscreen: true, backgroundThrottling: false, v8CacheOptions: 'none'`; window-open denied; audio muted; bounds 1920×1080; **CDP `Emulation.setDeviceMetricsOverride`** (1920×1080, dSF 1, `landscapePrimary`); load a `data:` URL with `baseURLForDataURL: 'https://www.youtube.com/'`; `executeJavaScript(script)`; close in `finally`. Then `closeAllConnections()` + `clearData()`.

The script itself (`src/botGuardScript.js`) uses `bgutils-js` (`BotGuardClient`, `WebPoMinter`, `buildURL`, `GOOG_API_KEY`), POSTs to `https://www.youtube.com/youtubei/v1/att/get` with `X-Goog-Visitor-Id` / `X-Youtube-Client-Version` / `X-Youtube-Client-Name` from the passed `context`, and mints the token.

Consumer: `helpers/api/local.js:477` → `window.ftElectron.generatePoToken(...)`, result used at `:619` and appended as `pot=` (`:749`) or `/pot/<token>` (`:783`).

### 5.2 Tauri design

```rust
// src-tauri/src/potoken/webview.rs
pub async fn generate(app: &AppHandle, video_id: &str, context: &str)
    -> Result<String, ApiError>
{
    let _guard = app.state::<AppState>().potoken.lock().await;   // ← queueGuardian

    let win = WebviewWindowBuilder::new(app, "potoken",
            WebviewUrl::App("potoken.html".into()))
        .visible(false)
        .incognito(true)                 // ← session.fromPartition(cache: false)
        .inner_size(1920.0, 1080.0)
        .focused(false)
        .skip_taskbar(true)
        .initialization_script(EMULATION_SHIM)   // ← replaces CDP Emulation
        .build()?;

    let script = BOTGUARD_JS.replace("FT_PARAMS",
                    &format!("\"{video_id}\",{context}"));
    let token = eval_with_result(&win, &script).await?;
    win.close()?;                        // ← finally { webContents.close() }
    clear_potoken_data(app).await?;      // ← closeAllConnections + clearData
    Ok(token)
}
```

### 5.3 Parity gaps and mitigations

| Electron mechanism | Tauri status | Mitigation |
|---|---|---|
| `session.fromPartition('potoken', {cache:false})` | Partial | `incognito: true` + explicit data clear after each run |
| `setPermissionCheckHandler(() => false)` | No direct API | Empty capability set (`capabilities/potoken.json` with `"permissions": []`) + CSP |
| `offscreen: true` | Not available | `visible: false` window positioned off-screen |
| `webRequest.onBeforeSendHeaders` | Not available in webview | Proxy `youtubei` requests through a Rust-side local endpoint, or override `window.fetch` in the init script to attach headers |
| `Emulation.setDeviceMetricsOverride` | **No equivalent** | `initialization_script` shim defining `screen.width/height/availWidth/availHeight`, `devicePixelRatio`, `window.outerWidth/Height`, `screen.orientation` |
| `executeJavaScript` returning a value | Not native | `eval()` + result posted back via a dedicated `potoken://result` event, awaited with a timeout |
| `baseURLForDataURL: youtube.com` | Not available | Serve `potoken.html` from a Tauri asset URL and set `Referer`/`Origin` on the intercepted fetches |
| Promise-chain queue | — | `tokio::sync::Mutex` |

**This is the highest-risk item in the migration.** botGuard actively probes for emulation inconsistencies; a JS-level shim is detectable in ways a CDP-level override is not. If the spike fails, fall back to a Node sidecar running the existing Electron/Playwright-style flow, invoked from Rust.

---

## 6. Vuex (14 modules) → Pinia

### 6.1 Module-by-module

| Vuex module | Lines | Pinia store | Backing commands / events |
|---|---:|---|---|
| `settings.js` | 1,112 | `useSettingsStore` | `get_settings`, `settings_upsert`; `sync://settings-changed` |
| `utils.js` | 958 | Split: `useUiStore`, `useAppStore`, `src/lib/*` helpers | Mixed — see §6.3 |
| `sync-server.js` | 737 | `useSyncStore` | `sync::*` commands, `sync://*` events |
| `playlists.js` | 649 | `usePlaylistsStore` | `playlists_*` (GENERAL + 5) |
| `tabs.js` | 532 | `useTabsStore` | `tabs::*` + `tabs://state-updated` |
| `subscription-cache.js` | 491 | `useSubscriptionCacheStore` | `subscription_cache_*` (GENERAL + 5) |
| `profiles.js` | 333 | `useProfilesStore` | `profiles_*` (GENERAL + 2) |
| `history.js` | 280 | `useHistoryStore` | `history_*` (GENERAL + 4) |
| `invidious.js` | 126 | `useInvidiousStore` | `fetch_invidious_instances`, `set_invidious_instance` |
| `search-history.js` | 119 | `useSearchHistoryStore` | `search_history_*` |
| `watch-stats.js` | 114 | `useWatchStatsStore` | `watch_stats_*` (GENERAL + 4) |
| `watch-queue.js` | 56 | `useWatchQueueStore` | Client-only |
| `downloads.js` | 41 | `useDownloadsStore` | `downloads::*` + `Channel` |
| `player.js` | 34 | `usePlayerStore` | Client-only |
| `store/index.js` | 100 | `plugins/sync-on-change.ts` | Pinia plugin |

### 6.2 Pattern translation

`downloads.js` is the clearest example. Today:

```js
// src/renderer/store/modules/downloads.js
const state = { ytDlpDownloads: {} }          // "replace with a Map after the Pinia migration"
const getters = { getYtDlpDownloads: (state) => state.ytDlpDownloads }
const mutations = {
  upsertYtDlpDownload(state, download) { state.ytDlpDownloads[download.id] = download },
  removeYtDlpDownload(state, id) { delete state.ytDlpDownloads[id] },
  clearFinishedYtDlpDownloads(state) { /* filter by status */ },
}
```

The file's own TODO is honoured:

```ts
// src/stores/downloads.ts
export const useDownloadsStore = defineStore('downloads', () => {
  const items = ref(new Map<number, DownloadStatus>())          // ← the TODO
  const active = computed(() => [...items.value.values()]
    .filter(d => d.status === 'downloading' || d.status === 'processing'))

  function upsert(d: DownloadStatus) { items.value.set(d.id, d) }
  function remove(id: number)        { items.value.delete(id) }
  function clearFinished() {
    for (const [id, d] of items.value)
      if (d.status !== 'downloading' && d.status !== 'processing') items.value.delete(id)
  }
  return { items, active, upsert, remove, clearFinished }
})
```

Mapping rules: `state` → `ref`/`reactive`; `getters` → `computed`; `mutations` + `actions` → plain functions (Pinia has no mutation/action split); `rootState` access → import the other store directly; `dispatch`/`commit` strings → direct typed calls.

### 6.3 `utils.js` (958 lines) must be split

It currently mixes toast display, clipboard, locale, external-link handling, `showToastOnAllTabs` IPC, formatting helpers, and more. Target: `useUiStore` (toasts, prompts), `useAppStore` (locale, versions, platform), and pure functions in `src/lib/format.ts` / `src/lib/url.ts`. Pure helpers should not live in a store at all.

### 6.4 The sync plugin

`store/index.js`'s `syncOnLocalChanges` (100 lines) uses `store.subscribe` (mutations) and `store.subscribeAction` (before/after) with a revision map, decoding setting names from mutation types (`setFoo` → `foo`, `updateFoo` → `foo`) and consulting `SYNC_MUTATION_REASONS` / `SYNC_ACTION_REASONS`.

Pinia makes this simpler and typed: `store.$onAction(({ name, after }) => …)` plus `store.$subscribe`. The string-decoding heuristics are replaced by an explicit registry:

```ts
// src/stores/plugins/sync-on-change.ts
const SYNC_REASONS: Record<string, SyncReason> = {
  'settings:upsert': 'settings',
  'history:upsert':  'history',
  'playlists:upsertVideo': 'playlists',
  // …
}
```

This removes an entire class of bug: today, renaming a mutation silently disables its sync trigger.

---

## 7. Sync Engine → Rust

| OpenTubeX source | Lines | Rust target |
|---|---:|---|
| `store/modules/sync-server.js` | 737 | `commands/sync.rs` + `useSyncStore` (UI state only) |
| `helpers/sync-server.js` | 1,040 | `sync/client.rs` (REST) |
| `helpers/sync-server-privacy.js` | 363 | `sync/crypto.rs` |
| `helpers/sync-server-errors.js` | 71 | `error.rs` variants |
| `helpers/sync-server-scheduling.js` | 95 | `stores/plugins/sync-on-change.ts` |

**Envelope format is a hard contract** (`sync-server-privacy.js`):

- AAD: `"OpenTubeX encrypted sync v1"` (`:5`)
- KDF: PBKDF2, SHA-256, fixed `PBKDF2_ITERATIONS`, 16-byte salt (`:55-73`, `:140`)
- Cipher: AES-GCM-256, 12-byte IV (`:166`)
- Optional gzip (`compression.name === GZIP_COMPRESSION`)
- Envelope validated on parse: `version`, `kdf.name/hash/iterations`, `cipher.name` (`:76-95`)
- Legacy path: `decryptLegacySyncDocument` (`:155`) + `LEGACY_ENCRYPTED_COLLECTIONS` (`sync-server.js:34`)

Rust crates: `pbkdf2` + `sha2`, `aes-gcm`, `flate2`, `base64`. Round-trip fixture tests in both directions are mandatory.

REST surface (`helpers/sync-server.js:86`, `apiRequest`): `/subscriptions/`, `/subscriptions/bulk`, `/subscriptions/groups/`, `/playlists/`, `/watch_history/`, `/watch_history/bulk`, `/channel_playback_speeds/` with `PUT`/`POST`/`PATCH`/`DELETE`. Ported verbatim to `reqwest`.

Retry/debounce constants to preserve: `EVENT_SYNC_DEBOUNCE_MS = 1500`, `ENCRYPTED_SYNC_RETRIES = 3` (`sync-server.js:32-33`).

---

## 8. Tabs → `WebviewWindow` + Pinia

### 8.1 Scale

| Source | Lines |
|---|---:|
| `src/main/tabs/TabManager.js` | 3,046 |
| `src/main/tabs/tabPreviewGeometry.js` | 147 |
| `src/main/tabs/tabPreviewCache.js` | 135 |
| `src/main/tabs/TabSessionStore.js` | 95 |
| `src/main/tabs/TabRendererBridge.js` | 76 |
| `src/main/tabs/tabOrder.js` | 32 |
| `src/renderer/tabs/TabNavigationService.js` | 792 |
| `src/renderer/tabs/TabMediaCoordinator.js` | 200 |
| `src/renderer/tabs/TabRuntimeRegistry.js` | 93 |
| `src/renderer/tabs/TabContext.js` | 76 |
| `src/renderer/tabs/TabLifecycleService.js` | 54 |
| `src/renderer/tabs/tabPreview.js` / `tabPageIcon.js` | 47 |
| `src/renderer/store/modules/tabs.js` | 532 |
| **Total** | **~5,325** |

Plus 45 IPC channels.

### 8.2 Two candidate designs

**A — Single window, virtual tabs (recommended for Phase 2).** All tabs are Vue route states inside one webview; `useTabsStore` owns `tabs[]`, `activeTabId`, per-tab history (`MAX_LOGICAL_HISTORY_ENTRIES = 100`), pinned/colour/selection state — most of which `store/modules/tabs.js` already models client-side. Rust persists sessions to `tab_sessions` and provides preview capture.

*Pros:* no multi-webview quirks; keeps the existing renderer tab services largely intact; lowest platform risk.
*Cons:* no process isolation between tabs; a heavy page can jank the whole UI.

**B — Multi-webview.** One `Webview` per tab inside a parent window (Tauri v2 multi-webview, currently unstable on some platforms).

*Pros:* closest to `WebContentsView` semantics; true isolation.
*Cons:* platform maturity risk; per-webview capability management; preview capture and geometry must be re-solved on three engines.

### 8.3 Channel mapping (representative)

| IPC channel | Slytube |
|---|---|
| `TABS_GET_STATE` | `invoke('tabs_get_state')` |
| `TABS_CREATE` / `CLOSE` / `ACTIVATE` / `DUPLICATE` / `MOVE` / `REORDER` | `tabs_*` commands (store-local in design A) |
| `TABS_SET_PINNED` / `SET_COLOR` / `SET_SELECTED` / `SET_LOADING` | Pinia mutations, persisted via `tabs_persist_session` |
| `TABS_CAPTURE_PREVIEW` / `GET_CACHED_PREVIEWS` / `SET_PREVIEW_CAPTURE_PAUSED` / `REQUEST_PREVIEW_REFRESH` | `tabs_capture_preview` → raw bytes over `Channel`; `moka` cache in Rust |
| `TABS_STATE_UPDATED` | `tabs://state-updated` broadcast |
| `TABS_GET_SYNC_SESSIONS` / `APPLY_SYNC_SESSIONS` | `tabs_get_sync_sessions` / `tabs_apply_sync_sessions` |
| `TABS_RESTORE_CLOSED` / `RELOAD` / `REQUEST_RELOAD` | Store + router |
| `TABS_UPDATE_ROUTE` / `NAV_HISTORY` / `TITLE` / `AVATAR` | Store-local (design A) |
| `TABS_REQUEST_FULLSCREEN` / `EXIT_FULLSCREEN` / `REQUEST_PICTURE_IN_PICTURE` | `getCurrentWindow().setFullscreen()` / Web PiP API |
| `CONTEXT_MENU_OPEN` / `EXECUTE` | `tauri::menu` native context menus |
| `RESOLVE_FAVICON` | `invoke('resolve_favicon')` (`net/favicon.rs`) |
| `CREATE_NEW_TAB` / `CREATE_NEW_WINDOW` | Store action / `WebviewWindow` |
| `WINDOW_MINIMIZED_STATE` | Tauri window events |

---

## 9. Components → shadcn-vue Primitives

### 9.1 Inventory

**118 `.vue` files** under `src/renderer/components/`, plus 17 view areas under `src/renderer/views/` (`Watch/Watch.vue` alone is 970 lines with a companion `Watch.js`).

Slytube's stack is already scaffolded: `shadcn-vue` ^2.8.2 on `reka-ui` ^2.10.1, Tailwind v4 (`@tailwindcss/vite`), `class-variance-authority`, `clsx` + `tailwind-merge` (`src/lib/utils.ts`), `@hugeicons/vue` for icons. `components.json` is present.

### 9.2 Mapping by family

| OpenTubeX family | Examples | shadcn-vue / reka-ui target |
|---|---|---|
| Buttons | `FtButton`, `FtIconButton`, `FtToggleSwitch` | `Button`, `Toggle`, `Switch` |
| Inputs | `FtInput`, `FtInputTags`, `FtCheckboxList`, `FtSelect`, `FtSlider` | `Input`, `TagsInput`, `Checkbox`, `Select`, `Slider` |
| Overlays | `FtPrompt`, `FtCreatePlaylistPrompt`, `FtPlaylistAddVideoPrompt`, `FtKeyboardShortcutPrompt`, `FtCollaboratorsPrompt` | `Dialog`, `AlertDialog` |
| Menus | `FtContextMenu`, `FtAddToPlaylistDropdown`, `FtIconPackSwitcher` | `DropdownMenu`, `ContextMenu` |
| Toasts | `FtToast`, `useTabToast` | `Sonner` / `Toast` |
| Layout | `ft-card`, `ft-flex-box`, `FtAutoGrid`, `FtElementList` | `Card` + Tailwind grid/flex utilities |
| Lists | `FtListVideo`, `FtListVideoLazy`, `FtListVideoNumbered`, `FtListChannel`, `FtListPlaylist`, `FtListHashtag`, `FtListLazyWrapper` | Custom `Video*` components on `Card` + **virtualised** list |
| Loading | `FtLoader`, `FtEmbeddedProgress`, `FtAutoLoadNextPageWrapper` | `Skeleton`, `Progress` |
| Notices | `FtNotificationBanner`, `FtAgeRestricted`, `FtPaidPromotionBadge`, `FtNewContentDot` | `Alert`, `Badge` |
| Settings panels | `DataSettings`, `DistractionSettings`, `DownloadSettings`, `ExperimentalSettings`, `ExternalPlayerSettings`, `ExternalSoftwareSettings`, `CaptionSettings`, `ChannelSettings`, `ContextMenuSearchSettings` | `Tabs` + `Accordion` + `Card` + form primitives |
| Channel | `ChannelAbout`, `ChannelDetails`, `ChannelHome` | Composed views on `Tabs`, `Avatar` |
| Comments | `CommentSection`, `FtCommunityPost`, `FtCommunityPoll` | Composed on `Card`, `Collapsible` |
| Icons | `FtIcon`, `customIcons.js`, `fontawesome-minimal.js` | `@hugeicons/vue` (already a dependency) |
| Scrolling | `AutoScrollWrapper`, `overlayScrollbars.js` | `ScrollArea` |

### 9.3 Rules for the port

1. **Do not port CSS.** `Ft*` components carry bespoke SCSS (`scss-partials/`, `themes.css`). Rebuild with Tailwind v4 tokens and CVA variants.
2. **Virtualise every long list.** Webview engines are slower than bundled Chromium; the subscriptions feed, history, and search results must not render full collections.
3. **Preserve accessibility.** reka-ui supplies focus management and ARIA that the hand-rolled `Ft*` components implement inconsistently.
4. **Theme via CSS variables.** OpenTubeX ships many themes (`LIGHT_BASE_THEMES` / `DARK_BASE_THEMES` in `constants.js` — nord, dracula, catppuccin, everforest, gruvbox, solarized, …). Map each to a shadcn CSS-variable palette rather than per-component overrides.
5. **Keyboard shortcuts stay data-driven.** `DefaultKeyboardShortcuts` + `applyKeyboardShortcutOverrides` / `getConfiguredKeyboardShortcuts` / `isKeyboardShortcutEditable` port as TypeScript, with `getElectronAccelerator` replaced by a Tauri accelerator formatter for global shortcuts.

---

## 10. Node Modules → Rust Crates

| Node (OpenTubeX) | Used for | Rust crate |
|---|---|---|
| `child_process` | yt-dlp / ffmpeg / external player | `tauri_plugin_shell` (`Command`) |
| `fs` / `fs/promises` | Datastores, records, binaries | `std::fs`, `tokio::fs`, `tauri_plugin_fs` |
| `path` | Path building | `std::path` + `app.path()` |
| `zlib` (`brotliDecompress`) | Compressed payloads | `brotli`, `flate2` |
| `crypto` / `crypto.subtle` | Sync envelope | `aes-gcm`, `pbkdf2`, `sha2`, `rand` |
| `@seald-io/nedb` | 8 datastores | `sqlx` (SQLite) |
| `js-yaml` (`loadYaml`, `index.js:10`) | Config parsing | `serde_yaml` |
| `electron.net` / `fetch` | HTTP | `reqwest` |
| `electron.session` | Proxy, headers, cookies | `reqwest` client config + webview settings |
| `electron.dialog` | File pickers | `tauri_plugin_dialog` |
| `electron.shell` | Open path/URL | `tauri_plugin_opener` |
| `electron.nativeImage` | Preview/favicon images | `image` crate |
| `electron.powerSaveBlocker` | Keep-awake | `tauri_plugin_prevent_default` / platform APIs |
| `electron.nativeTheme` | Dark-mode detection | `Window::theme()` + `on_theme_changed` |
| `electron-updater` | Updates | `tauri_plugin_updater` |
| `adm-zip` (`extractZipEntry`) | ffmpeg archives | `zip` |

---

## 11. Migration Sequence

| Phase | Scope | Source lines addressed | Gating risk |
|---|---|---:|---|
| 1 — Foundation | SQLite schema, `sqlx` repos, settings commands, `useSettingsStore`, NeDB importer | ~1,900 | None |
| 2 — Spikes (parallel) | PoToken hidden webview; tab design A vs. B | ~5,500 | **Both are go/no-go gates** |
| 3 — Downloads | yt-dlp sidecar, `Channel` progress, Downloads view | ~1,400 | Sidecar signing decision |
| 4 — API layer | Invidious → Rust; `youtubei.js` stays in webview | ~3,600 | Header/proxy parity |
| 5 — Sync | Rust crypto, REST client, Pinia plugin | ~2,300 | Envelope compatibility |
| 6 — UI | 118 components + 17 views onto shadcn-vue; remaining Vuex modules | ~10,000+ | Webview rendering performance |

Total in-scope source: roughly **25,000 lines** of OpenTubeX across main, preload, datastores, stores, helpers, and components.

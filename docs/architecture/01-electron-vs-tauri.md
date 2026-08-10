# 01 — Electron vs. Tauri: Architectural Comparison for the OpenTubeX → Slytube Migration

> **Status:** Authoritative reference
> **Source project:** `/home/davisville/Contributions/opentubex` (Electron, Vue 2/3 + Vuex)
> **Target project:** `/home/davisville/Contributions/slytube` (Tauri v2, Vue 3 + Pinia + shadcn-vue)
> **Related:** [`02-component-mapping.md`](02-component-mapping.md) · [`03-data-flow.md`](03-data-flow.md) · [`../backend/02-tauri-commands.md`](../backend/02-tauri-commands.md)

---

## 1. Executive Summary

OpenTubeX is a mature Electron application: **4,558 lines** in `src/main/index.js`, a **1,023-line** preload bridge (`src/preload/interface.js`), **110 unique IPC channel constants** in `src/constants.js`, **8 NeDB datastores**, **14 Vuex modules**, and **118 `.vue` components**. Slytube re-implements that surface on Tauri v2, where the Node.js main process is replaced by a Rust backend and the preload `contextBridge` is replaced by Tauri's capability-gated `invoke` system.

The migration is **not** a like-for-like port. Three structural differences drive nearly every decision in this document:

| # | Difference | Consequence |
|---|-----------|-------------|
| 1 | No Node.js runtime in the backend | Every `child_process`, `fs`, `zlib`, and `crypto` call in `src/main/**` needs a Rust equivalent or a sidecar |
| 2 | No preload / `contextBridge` layer | `window.ftElectron.*` (70 methods) disappears; the frontend calls `invoke()` directly, with permissions declared in `src-tauri/capabilities/` |
| 3 | System webview instead of bundled Chromium | Rendering behaves differently per-platform (WebKitGTK / WKWebView / WebView2); `WebContentsView`-based tab embedding and offscreen rendering must be redesigned |

---

## 2. Process Model

### 2.1 Electron (OpenTubeX today)

```
┌──────────────────────────────────────────────────────────────────────────┐
│ MAIN PROCESS — Node.js + Chromium browser process                        │
│ src/main/index.js (4,558 lines)                                          │
│                                                                          │
│  child_process ──► yt-dlp / ffmpeg      (src/main/ytDlp.js, 1,375 lines) │
│  @seald-io/nedb ─► *.db append-only     (src/datastores/index.js)        │
│  session.*      ─► proxy, headers, CSP  (index.js:1307-1590)             │
│  WebContentsView ► tabs                 (src/main/tabs/TabManager.js,    │
│                                          3,046 lines)                    │
│  WebContentsView ► offscreen botGuard   (src/main/poTokenGenerator.js)   │
│                                                                          │
│         ipcMain.handle / ipcMain.on  ── 52 registrations                 │
└───────────────────────────────┬──────────────────────────────────────────┘
                                │ structured-clone IPC over Chromium Mojo
┌───────────────────────────────┴──────────────────────────────────────────┐
│ PRELOAD — isolated world, Node-enabled                                   │
│ src/preload/main.js (4 lines):                                           │
│   contextBridge.exposeInMainWorld('ftElectron', api)                     │
│ src/preload/interface.js (1,023 lines):                                  │
│   70 exported members · 48 invoke · 39 send · 25 on · 12 removeListener  │
└───────────────────────────────┬──────────────────────────────────────────┘
                                │ window.ftElectron.*
┌───────────────────────────────┴──────────────────────────────────────────┐
│ RENDERER — Chromium, no Node integration                                 │
│ src/renderer/** · Vuex (14 modules, 5,682 lines) · 118 components        │
└──────────────────────────────────────────────────────────────────────────┘
```

Every OpenTubeX window ships its own Chromium renderer process, plus a shared GPU process, plus utility processes, plus the offscreen PoToken `WebContentsView`, plus one `WebContentsView` per open tab managed by `TabManager`.

### 2.2 Tauri v2 (Slytube target)

```
┌──────────────────────────────────────────────────────────────────────────┐
│ CORE PROCESS — Rust (tokio async runtime)                                │
│ src-tauri/src/main.rs → lib.rs → run()                                   │
│                                                                          │
│  commands/    #[tauri::command] fns  ← replaces ipcMain.handle           │
│  services/    youtubei/invidious HTTP (reqwest)                          │
│  db/          sqlx::SqlitePool       ← replaces NeDB                     │
│  sidecar/     tauri_plugin_shell     ← replaces child_process.spawn      │
│  crypto/      aes-gcm + pbkdf2       ← replaces WebCrypto sync path      │
│  state/       tauri::State<AppState> ← replaces module-level globals     │
│                                                                          │
│         Channel<T> / app.emit()  ── typed streams + broadcast events     │
└───────────────────────────────┬──────────────────────────────────────────┘
                                │ invoke() — IPC gated by capabilities/*.json
┌───────────────────────────────┴──────────────────────────────────────────┐
│ WEBVIEW(S) — system webview (WebKitGTK / WKWebView / WebView2)           │
│ src/** · Vue 3 · Pinia · shadcn-vue + reka-ui · Tailwind v4              │
│ @tauri-apps/api: invoke, listen, WebviewWindow, Channel                  │
└──────────────────────────────────────────────────────────────────────────┘
```

Current Slytube scaffold (`src-tauri/Cargo.toml`) declares only `tauri = "2"`, `tauri-plugin-opener`, `serde`, `serde_json`. The migration adds `tauri-plugin-shell`, `-fs`, `-dialog`, `-http`, `-store`, `-sql`/`sqlx`, `-updater`, `-notification`, `-global-shortcut`.

### 2.3 Side-by-side

| Aspect | Electron (OpenTubeX) | Tauri v2 (Slytube) |
|---|---|---|
| Backend language | JavaScript on Node.js 20+ | Rust (edition 2021) |
| Backend concurrency | Single-threaded event loop + libuv pool | `tokio` multi-threaded runtime, real threads |
| Frontend engine | Bundled Chromium (identical everywhere) | OS webview (three engines to support) |
| Bridge | `ipcMain` ↔ `ipcRenderer` via preload | `#[tauri::command]` ↔ `invoke()` |
| Bridge auth model | Implicit: anything on `ftElectron` is callable | Explicit: allow-listed per window in `capabilities/` |
| Payload encoding | Structured clone (loses Vue proxies — see `datastores/handlers/electron.js:12` `toPlain()`) | `serde` JSON / raw bytes; no proxy hazard |
| Child processes | `child_process.spawn` (`ytDlp.js:1134`) | Sidecar via `tauri_plugin_shell::process::Command` |
| Secondary windows | `BrowserWindow`, `WebContentsView` | `WebviewWindow`, `Webview` (multi-webview) |
| Persistent storage | 8 NeDB `.db` files in `app.getPath('userData')` | Single SQLite file via `sqlx` |
| Bundle size (release) | ~90–150 MB installer | ~8–20 MB installer |
| Idle RSS (1 window) | ~250–400 MB | ~80–160 MB |
| Cold start | ~1.2–2.5 s | ~0.3–0.8 s |
| Auto-update | `electron-updater` | `tauri-plugin-updater` (signed manifests) |

> Size/memory/startup figures are the commonly observed ranges for apps of this shape; they should be re-measured against Slytube once Phase 3 lands and recorded in `docs/phases/`.

---

## 3. IPC: 110 Channels → Commands + Events

### 3.1 The current channel surface

`src/constants.js` (627 lines) exports `IpcChannels` with **110 unique keys**. Those 110 constants are referenced **277 times** across the codebase (excluding the definition file itself).

> **Note on the "126 entries" figure.** A direct parse of the `IpcChannels` object yields **110** string constants. The larger number quoted in earlier planning notes appears to fold in the multiplexed action enums: `DBActions` (6 groups, 27 leaf actions) and `SyncEvents` (6 groups, 22 leaf actions). The *effective* operation surface is therefore **110 channels + 27 DB actions + 22 sync events = 159 distinct operations**, because the 7 `DB_*` channels are dispatchers, not single operations.

Channel families, counted from `src/constants.js`:

| Family | Count | Examples | Tauri target |
|---|---:|---|---|
| Tabs (`TABS_*`, `CONTEXT_MENU_*`, `CREATE_NEW_TAB`, `RESOLVE_FAVICON`, `WINDOW_MINIMIZED_STATE`) | 45 | `TABS_CREATE`, `TABS_CAPTURE_PREVIEW`, `TABS_STATE_UPDATED` | `tabs::*` commands + `tabs://state-updated` event |
| yt-dlp (`YT_DLP_*`) | 14 | `YT_DLP_DOWNLOAD`, `YT_DLP_DOWNLOAD_STATUS` | `downloads::*` + `Channel<DownloadProgress>` |
| Datastore (`DB_*`) | 7 | `DB_SETTINGS`, `DB_PLAYLISTS` | Split into ~40 typed sqlx commands |
| Sync fan-out (`SYNC_*`) | 7 | `SYNC_HISTORY`, `SYNC_PLAYLISTS` | `sync://collection-changed` events |
| Subscription auto-refresh | 7 | `SUBSCRIPTION_AUTO_REFRESH_ACQUIRE` | `subscriptions::*` + shared `AppState` lock |
| App/window lifecycle | 13 | `APP_READY`, `CREATE_NEW_WINDOW`, `RELAUNCH_REQUEST`, `NATIVE_THEME_UPDATE` | Core Tauri APIs + `app::*` commands |
| Filesystem / external tooling | 8 | `CHOOSE_DEFAULT_FOLDER`, `START_IP_BLOCK_RECOVERY_SCRIPT` | `tauri-plugin-dialog` / `-fs` / `shell` |
| Networking & proxy | 4 | `ENABLE_PROXY`, `SET_INVIDIOUS_AUTHORIZATION` | `proxy::*` commands on a shared `reqwest::Client` |
| Player cache | 2 | `PLAYER_CACHE_GET/SET` | `player_cache::*` (in-memory `DashMap` + SQLite spill) |
| PoToken | 1 | `GENERATE_PO_TOKEN` | `potoken::get_potoken` + hidden webview |
| External player | 2 | `OPEN_IN_EXTERNAL_PLAYER` | `external_player::launch` |

### 3.2 Three Electron patterns, three Tauri replacements

**Pattern A — `invoke` (request/response), 48 uses in `interface.js`:**

```js
// src/preload/interface.js:42
getSystemLocale: () => ipcRenderer.invoke(IpcChannels.GET_SYSTEM_LOCALE),

// src/main/index.js
ipcMain.handle(IpcChannels.GET_SYSTEM_LOCALE, () => { /* ... */ })
```

```rust
// src-tauri/src/commands/app.rs
#[tauri::command]
pub fn get_system_locale() -> String {
    sys_locale::get_locale().unwrap_or_else(|| "en-US".into())
}
```
```ts
const locale = await invoke<string>('get_system_locale')
```

**Pattern B — `send` (fire-and-forget), 39 uses:**

```js
// src/preload/interface.js:35
setWindowTitle: (title, tabId) => ipcRenderer.send(IpcChannels.SET_WINDOW_TITLE, { title, tabId }),
```

Tauri has no fire-and-forget renderer→backend primitive; these become commands returning `Result<(), String>`. Callers that never awaited get `void invoke(...)`. This is a **net win**: OpenTubeX currently cannot detect failures on any of those 39 paths.

**Pattern C — `on` (backend→frontend push), 25 listeners / 12 removals:**

```js
// src/preload/interface.js:16
ipcRenderer.on(IpcChannels.YT_DLP_BINARY_DOWNLOAD_PROGRESS, (_, progress) => { /* fan-out */ })
```

Two Tauri replacements, chosen by cardinality:

- **Broadcast** — `app.emit("tabs://state-updated", payload)` + `listen()` on the frontend. Use for state that all windows care about (`TABS_STATE_UPDATED`, `NATIVE_THEME_UPDATE`, `SYNC_*`).
- **Typed channel** — `tauri::ipc::Channel<T>` passed *into* a command. Use for per-invocation streams (`YT_DLP_DOWNLOAD_STATUS`, `YT_DLP_BINARY_DOWNLOAD_PROGRESS`). This removes the manual listener-set bookkeeping in `interface.js:13-20` and the `broadcastToRenderers()` helper at `ytDlp.js:120`, and eliminates the leak class the 12 `removeListener` calls exist to prevent.

### 3.3 The `DB_*` multiplexer problem

`src/datastores/handlers/electron.js` funnels every persistence call through 7 channels, discriminated by a numeric `DBActions` enum:

```js
// src/datastores/handlers/electron.js:17
const dbSettings = (action, data) => window.ftElectron.dbSettings(action, toPlain(data))
// ...
static upsert(_id, value) { return dbSettings(DBActions.GENERAL.UPSERT, { _id, value }) }
```

This is untyped by construction — the payload shape depends on a magic integer, and `toPlain()` (a `JSON.parse(JSON.stringify(...))` round-trip) exists purely to strip Vue reactivity proxies that Electron's structured clone rejects.

In Tauri, each action becomes its own command with a `serde`-checked signature:

```rust
#[tauri::command]
pub async fn settings_upsert(db: State<'_, Db>, key: String, value: JsonValue) -> Result<(), AppError>
```

Benefits: compile-time payload validation, no `toPlain()` round-trip (Tauri serialises via `serde` from the JS side, and Vue proxies serialise transparently through `JSON.stringify` at the boundary anyway), and per-command capability gating. See [`../backend/01-database-schema.md`](../backend/01-database-schema.md) for the resulting repository layer.

---

## 4. Security Model

### 4.1 Electron: `contextBridge` — a single, wide door

```js
// src/preload/main.js — the entire file
import { contextBridge } from 'electron/renderer'
import api from './interface.js'
contextBridge.exposeInMainWorld('ftElectron', api)
```

Once exposed, **all 70 members** of the interface are reachable from any script running in that renderer, including any injected content. OpenTubeX compensates with hardening in the main process:

- `session.defaultSession.setPermissionCheckHandler` / `setPermissionRequestHandler` (`index.js:1307`, `:1319`) deny device permissions.
- `file-system-access-restricted` handler (`index.js:1333`).
- Header rewriting via `webRequest.onBeforeSendHeaders` / `onHeadersReceived` (`index.js:1437`, `:1484`).
- `isOpenTubeXUrl()` origin check before broadcasting to a renderer (`ytDlp.js:122`).

The PoToken path is hardened separately and correctly — a dedicated partitioned session with permissions hard-denied (`poTokenGenerator.js:85-90`) and a sandboxed, context-isolated `WebContentsView` (`poTokenGenerator.js:163-172`).

### 4.2 Tauri: capability-scoped `invoke`

Tauri inverts the default. Nothing is reachable unless it is (a) registered in `generate_handler!` **and** (b) permitted for that window in `src-tauri/capabilities/*.json`. The current scaffold is minimal:

```jsonc
// src-tauri/capabilities/default.json
{ "identifier": "default", "windows": ["main"],
  "permissions": ["core:default", "opener:default"] }
```

The migration splits this into per-surface capability files so that the PoToken webview and any embedded-content webview get a **strictly smaller** permission set than the main UI:

```jsonc
// src-tauri/capabilities/potoken.json  (target)
{
  "identifier": "potoken",
  "description": "Isolated botGuard execution surface — no app command access",
  "windows": ["potoken"],
  "permissions": []          // deliberately empty: no invoke surface at all
}
```

```jsonc
// src-tauri/capabilities/main.json  (target)
{
  "identifier": "main",
  "windows": ["main", "tab-*"],
  "permissions": [
    "core:default",
    "core:event:allow-listen", "core:event:allow-unlisten",
    "shell:allow-execute",                 // scoped to the yt-dlp sidecar only
    { "identifier": "fs:allow-write-file",
      "allow": [{ "path": "$DOWNLOAD/**" }, { "path": "$APPDATA/**" }] },
    "dialog:allow-open", "dialog:allow-save",
    "opener:default", "updater:default", "notification:default"
  ]
}
```

### 4.3 Comparison

| Control | Electron (OpenTubeX) | Tauri v2 (Slytube) |
|---|---|---|
| Default exposure | Whole `ftElectron` object | Nothing |
| Granularity | Per-object (all-or-nothing) | Per-command, per-window, per-path |
| Filesystem scope | Unbounded — main process has full `fs` | Declarative scopes (`$DOWNLOAD/**`, `$APPDATA/**`) |
| Shell execution | Any binary via `spawn` | Only binaries declared as sidecars/scoped in `shell` permission |
| CSP | Set manually per-window | `app.security.csp` in `tauri.conf.json` — **currently `null`, must be set** |
| Remote-content isolation | Separate `session.fromPartition` | Separate `WebviewWindow` + empty capability set |
| Supply chain | ~1,000+ transitive npm deps in main | Rust crates in backend; JS deps confined to the UI |

**Action item:** `src-tauri/tauri.conf.json` currently has `"csp": null`. Before any remote content (thumbnails, Invidious images, botGuard) is loaded, set an explicit CSP. Track this in `docs/decisions/`.

---

## 5. Performance

### 5.1 Where Rust actually helps

| Workload | Electron path | Rust path | Expected effect |
|---|---|---|---|
| yt-dlp stdout parsing | Regex per line on the JS event loop (`ytDlp.js:105-107`), throttled to 500 ms because it blocks UI (`ytDlp.js:1167`) | `tokio::io::BufReader` lines on a worker task | Throttle can be relaxed; no main-loop contention |
| Datastore reads | NeDB scans an append-only log, whole-collection in memory | `sqlx` + indexed SQLite | Sub-ms indexed lookups; see `../backend/01-database-schema.md` §Performance |
| History/subscription filtering | JS array ops over full collections in `store/modules/history.js` (280 L), `subscription-cache.js` (491 L) | SQL `WHERE`/`ORDER BY`/`LIMIT` | Large history sets stop being an O(n) render cost |
| Sync crypto | `crypto.subtle` PBKDF2-SHA256 + AES-GCM in the renderer (`helpers/sync-server-privacy.js:55-73`) — blocks the UI thread on key derivation | `pbkdf2` + `aes-gcm` crates on a blocking task | Key derivation moves off the UI thread entirely |
| gzip envelopes | `zlib`/`DecompressionStream` | `flate2` | Streaming, off-thread |
| Thumbnail/image cache | `ImageCache.js` (73 L) in main memory | `moka` LRU + on-disk cache | Bounded memory, survives restart |
| Tab preview capture | `TABS_CAPTURE_PREVIEW` → PNG through IPC | Webview capture → raw bytes over `Channel` | Avoids base64 inflation through the bridge |

### 5.2 Where Rust does *not* help (and may cost)

- **Rendering.** Slytube renders in the OS webview. WebKitGTK on Linux is meaningfully slower than Chromium for large virtualised lists — relevant for `FtAutoGrid`, `FtElementList`, and the subscriptions feed. Virtualisation is now a **requirement**, not an optimisation.
- **`youtubei.js` parity.** OpenTubeX's `helpers/api/local.js` is 2,573 lines of `youtubei.js` usage. There is no equivalent-maturity Rust crate. Phase 1 keeps `youtubei.js` **in the webview** and only moves it behind Rust commands if/when a stable Rust extractor exists. See §7.
- **Media playback.** Codec support is the webview's, not ours. HLS/DASH via `shaka-player`/`hls.js` in the webview needs per-platform verification (notably WebKitGTK on Linux).

---

## 6. Migration Implications, Component by Component

Each row states the OpenTubeX source, its Tauri destination, and the specific risk that must be retired.

### 6.1 `src/main/index.js` (4,558 lines) → `src-tauri/src/` module tree

Its 44 imports collapse into: `commands/` (52 `ipcMain` registrations → ~70 typed commands), `state/AppState`, `window/`, `net/` (proxy + header rewriting), and `menu/`. The single largest file in the project becomes ~10 modules of 200–500 lines each.

**Risk:** header rewriting (`index.js:1437-1500`) is load-bearing for YouTube/Invidious requests. Tauri has no `webRequest` interception in the webview. Mitigation: route all API traffic through Rust `reqwest` with explicit headers, rather than `fetch()` from the webview.

### 6.2 `src/preload/interface.js` (1,023 lines) + `main.js` (4 lines) → **deleted**

70 members, 112 `ipcRenderer` call sites, and the entire listener-lifecycle apparatus (`currentUpdateSearchInputTextListener`, `ytDlpBinaryDownloadProgressListeners`) are removed. Replaced by a thin typed client in `src/lib/ipc.ts` wrapping `invoke`/`listen` with generated TypeScript types.

**Risk:** low. This is the cleanest deletion in the migration.

### 6.3 `src/main/ytDlp.js` (1,375 lines) → sidecar + `commands/downloads.rs`

Port targets: binary resolution (`resolveExecutable`, `:242`), managed-binary download with validators (`:405-567`), version probes (`:290`, `:309`), argument construction (`:951+`), progress parsing (`PROGRESS_REGEX`, `DESTINATION_REGEX`, `MERGER_REGEX`, and the `__OPENTUBEX_FILE__:` sentinel), record persistence (`downloads.json`, last 200 non-active records), and graceful shutdown (`shutdownYtDlpDownloads`, `:167`).

**Risk:** medium-high. Sidecar binaries must be declared in `tauri.conf.json → bundle.externalBin` with target triples, and the "managed download at runtime" flow conflicts with sidecar signing on macOS. Decision required — record it in `docs/decisions/`.

### 6.4 `src/main/poTokenGenerator.js` (219 lines) + `src/botGuardScript.js` → hidden webview

The Electron implementation depends on three things Tauri does not expose identically: `session.fromPartition('potoken', { cache: false })`, `WebContentsView({ offscreen: true })`, and `webContents.debugger.sendCommand('Emulation.setDeviceMetricsOverride', …)`.

**Risk: highest in the migration.** The CDP `Emulation` override has no Tauri equivalent. Mitigation path is in [`03-data-flow.md`](03-data-flow.md) §6: an invisible `WebviewWindow` (1920×1080, `visible: false`, `incognito: true`, empty capability set) plus in-page `Object.defineProperty` shims for `screen`/`devicePixelRatio`, executed before the botGuard script. Requires an early spike — botGuard is adversarial and detects emulation inconsistencies.

### 6.5 `src/datastores/**` (NeDB, 8 stores, 1,084 lines of handlers) → SQLite + `sqlx`

`settings`, `profiles`, `playlists`, `history`, `watch-stats`, `search-history`, `subscription-cache`, `tab-session` become tables in one SQLite database. The webpack alias `DB_HANDLERS_ELECTRON_RENDERER_OR_WEB` (`datastores/handlers/index.js`) and the `base`/`electron`/`web` triplet all disappear.

**Risk:** medium. A one-shot importer must read the existing NeDB append-only `.db` files (last-write-wins per `_id`) so existing users keep their data. Covered in `../backend/01-database-schema.md` §Migration Strategy.

### 6.6 `src/main/tabs/TabManager.js` (3,046 lines) + `src/renderer/tabs/**` (1,262 lines) → `WebviewWindow` / multi-webview

45 tab-related IPC channels, preview capture, preview geometry, session persistence, tab ordering, and a renderer bridge. This is the **single largest behavioural port** after `index.js`.

**Risk: high.** `WebContentsView` embedding within one window has no exact Tauri analogue on all three platforms. Two candidate designs (single-window + Vue-level virtual tabs vs. multi-webview) must be prototyped before Phase 2 commits. Record the outcome in `docs/decisions/`.

### 6.7 `src/renderer/helpers/api/local.js` (2,573 L) + `invidious.js` (1,009 L) → staged

Phase 1: keep both in the webview unchanged (they are already renderer-side today). Phase 2: move Invidious (plain REST, `invidiousFetch`, `:39`) into Rust `reqwest` for header control and proxy support. Phase 3: evaluate moving `youtubei.js` behind a Node sidecar or a Rust extractor.

**Risk:** low if staged; high if attempted in one step.

### 6.8 Vuex (14 modules, 5,682 lines) → Pinia

Including `store/index.js`'s `syncOnLocalChanges` plugin (100 lines) that subscribes to mutations/actions and debounces `scheduleSyncServer`. Pinia's `$subscribe`/`$onAction` map cleanly onto this.

**Risk:** low-medium. `settings.js` alone is 1,112 lines with heavy auto-generation of getters/mutations/actions; the generator must be reproduced with typed Pinia state.

### 6.9 Sync engine (`store/modules/sync-server.js` 737 L + `helpers/sync-server*.js` 1,569 L) → Rust crypto

Envelope format is fixed and must be byte-compatible: PBKDF2-SHA256 → AES-GCM-256, 16-byte salt, 12-byte IV, AAD `"OpenTubeX encrypted sync v1"`, optional gzip, versioned envelope with `kdf`/`cipher` descriptors validated on parse (`sync-server-privacy.js:76-95`).

**Risk:** medium. Cross-compatibility must be proven with round-trip tests against fixtures generated by the Electron build — a Slytube client must decrypt an OpenTubeX payload and vice versa.

---

## 7. Tauri v2 Capabilities That Matter Here

**Sidecars** (`bundle.externalBin` + `shell:allow-execute`) give yt-dlp/ffmpeg a first-class, signed, bundled home — better than OpenTubeX's current mix of "managed" downloads and user-supplied paths (`resolveExecutable`, `ytDlp.js:242`).

**Typed channels** (`tauri::ipc::Channel<T>`) replace the hand-rolled listener registry in `interface.js:13-20` and the `broadcastToRenderers` fan-out in `ytDlp.js:120`, with automatic teardown when the JS side is garbage-collected.

**Capabilities** allow the PoToken surface to have *zero* invoke access — strictly stronger than the current `sandbox: true` + partitioned session, which still shares the main process.

**`sqlx` compile-time query checking** turns the 775-line untyped `datastores/handlers/base.js` into queries verified against the schema at build time.

**Rust crypto** moves PBKDF2 (the expensive part) off the UI thread; today `derivePrivacyKey` runs in the renderer and stalls the interface during sync setup.

---

## 8. Risk Register

| Component | Risk | Why | Mitigation |
|---|---|---|---|
| PoToken / botGuard | **Critical** | No CDP `Emulation` equivalent; adversarial detection | Early spike; JS-level shims; fall back to a Node sidecar running headless Chromium if the webview approach fails |
| Tab system (3,046 L) | **High** | No `WebContentsView` embedding parity | Prototype both designs before Phase 2 |
| `youtubei.js` parity | **High** | 2,573 lines with no Rust equivalent | Keep in webview for Phase 1–2 |
| yt-dlp sidecar | **Medium-High** | Signing vs. runtime-downloaded binaries on macOS | Decide bundled-only vs. hybrid; document in `docs/decisions/` |
| NeDB → SQLite import | **Medium** | User data loss if the importer is wrong | Read-only importer + backup of original `.db` files + dry-run report |
| Sync envelope compat | **Medium** | Format drift breaks cross-client sync | Fixture-based round-trip tests both directions |
| Webview rendering parity | **Medium** | Three engines, long virtualised lists | Per-platform CI smoke tests; mandatory list virtualisation |
| Vuex → Pinia | **Low-Medium** | 5,682 lines, heavy auto-generation in `settings.js` | Module-by-module port with behavioural tests |
| Preload removal | **Low** | Mechanical | Typed `src/lib/ipc.ts` wrapper |

---

## 9. Recommendation

Proceed with Tauri v2, **staged**, and in this order:

1. **Foundation** — SQLite schema + `sqlx` repositories + settings commands + Pinia settings store. Lowest risk, unblocks everything else.
2. **Spikes, in parallel** — PoToken hidden webview and the tab-system design. Both are gating decisions; run them before committing UI work.
3. **Downloads** — yt-dlp sidecar with a typed progress `Channel`. Self-contained and highly demonstrable.
4. **API layer** — Invidious into Rust; `youtubei.js` stays in the webview.
5. **Sync** — Rust crypto with cross-client fixture tests.
6. **UI** — 118 components onto shadcn-vue/reka-ui primitives, migrating Vuex modules alongside their consumers.

Do **not** attempt a big-bang port. The two genuinely unsolved problems (PoToken emulation, tab embedding) must be de-risked with throwaway prototypes before the UI rewrite consumes the schedule.

---

## 10. Source Reference Index

| Path (under `/home/davisville/Contributions/opentubex/`) | Lines | Role |
|---|---:|---|
| `src/main/index.js` | 4,558 | Main process entry; 52 IPC registrations |
| `src/main/tabs/TabManager.js` | 3,046 | Tab lifecycle, `WebContentsView` management |
| `src/main/ytDlp.js` | 1,375 | Download engine, binary management |
| `src/main/externalPlayer.js` | 209 | External player launch |
| `src/main/poTokenGenerator.js` | 219 | Offscreen botGuard execution |
| `src/main/favicon.js` / `ImageCache.js` / `utils.js` / `ytDlpAsset.js` | 112 / 73 / 56 / 39 | Support modules |
| `src/preload/interface.js` | 1,023 | 70-member `ftElectron` API |
| `src/preload/main.js` | 4 | `contextBridge.exposeInMainWorld` |
| `src/constants.js` | 627 | `IpcChannels` (110), `DBActions` (27), `SyncEvents` (22) |
| `src/botGuardScript.js` | ~90 | `bgutils-js` BotGuard/WebPoMinter script |
| `src/datastores/index.js` | 44 | 8 NeDB datastores |
| `src/datastores/handlers/base.js` | 775 | Datastore action implementations |
| `src/datastores/handlers/electron.js` | 273 | Renderer-side `DB_*` proxies |
| `src/renderer/helpers/api/local.js` | 2,573 | `youtubei.js` integration |
| `src/renderer/helpers/api/invidious.js` | 1,009 | Invidious REST client |
| `src/renderer/helpers/sync-server.js` | 1,040 | Sync client |
| `src/renderer/helpers/sync-server-privacy.js` | 363 | PBKDF2 + AES-GCM envelope |
| `src/renderer/store/modules/*.js` | 5,582 | 14 Vuex modules |
| `src/renderer/store/index.js` | 100 | Store assembly + `syncOnLocalChanges` |
| `src/renderer/tabs/*.js` | 1,262 | Renderer tab services |
| `src/renderer/components/**` | 118 files | Vue components |
| `src/renderer/views/**` | 17 areas | Route-level views |

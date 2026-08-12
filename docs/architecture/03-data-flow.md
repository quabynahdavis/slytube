# 03 — Data Flow Architecture

> **Status:** Authoritative reference
> **Source:** `/home/davisville/Contributions/opentubex/src/**`
> **Target:** `/home/davisville/Contributions/slytube/{src,src-tauri}/**`
> **Related:** [`01-electron-vs-tauri.md`](01-electron-vs-tauri.md) · [`02-component-mapping.md`](02-component-mapping.md) · [`../backend/01-database-schema.md`](../backend/01-database-schema.md) · [`../backend/02-tauri-commands.md`](../backend/02-tauri-commands.md)

---

## 0. Conventions

Six flows are documented. Each has: the OpenTubeX baseline (with source anchors), the Slytube design, a sequence diagram, error handling, and a parity checklist.

**Three transport primitives** are used throughout:

| Primitive | Direction | Use when | Replaces |
|---|---|---|---|
| `invoke()` → `#[tauri::command]` | UI → Rust → UI | Request/response | `ipcRenderer.invoke` (48 sites) and `ipcRenderer.send` (39 sites) |
| `Channel<T>` | Rust → UI, per-invocation | Progress for one operation | `webContents.send` in a loop |
| `app.emit()` / `listen()` | Rust → all UI | Global state change | `broadcastToRenderers` (`ytDlp.js:120`) |

**Event namespace** (mirrored in `src-tauri/src/events.rs` and `src/lib/events.ts`):

```
settings://changed          downloads://status        downloads://removed
tabs://state-updated        tabs://active-changed     sync://{collection}-changed
sync://status               potoken://result          ui://toast
app://change-view           app://open-url            subs://state-changed
```

**Universal invariant.** Rust is the single writer to SQLite. Pinia is a read-through cache. The UI never writes to persistent storage directly — it invokes a command and applies the returned/emitted state. This is stricter than OpenTubeX, where the renderer both mutated Vuex and fired `DB_*` IPC, allowing the two to diverge.

---

## 1. Settings Flow

### 1.1 Baseline

`store/modules/settings.js` is **1,112 lines**, most of it auto-generating getters/mutations/actions from a `state` object plus hand-written `customState` entries. Persistence goes through `DBSettingHandlers` (`datastores/handlers/index.js` → `electron.js`):

```js
// src/datastores/handlers/electron.js:17, :26-31
const dbSettings = (action, data) => window.ftElectron.dbSettings(action, toPlain(data))
class Settings {
  static find()             { return dbSettings(DBActions.GENERAL.FIND) }
  static upsert(_id, value) { return dbSettings(DBActions.GENERAL.UPSERT, { _id, value }) }
}
```

→ `ipcMain.handle(IpcChannels.DB_SETTINGS)` → `baseHandlers.settings.*` → NeDB `settings.db`.

Side effects live in the store: `loadLocale` (i18n), `setReducedMotionPreference`, `setAnimationSpeed`, `applyKeyboardShortcutOverrides`, theme application, and a one-shot migration flag `ytDlpPlaybackEngineDefaultMigration`.

Every settings write also feeds the sync scheduler via `store/index.js`'s mutation-name heuristic (`setFoo` → `foo`, checked against `isSettingSyncable`).

### 1.2 Slytube flow

```
┌── STARTUP ────────────────────────────────────────────────────────────────┐
│ main.ts                                                                   │
│   └─ useSettingsStore().hydrate()                                         │
│        └─ invoke('get_settings')                                          │
│             └─ commands/settings.rs::get_settings(State<AppState>)        │
│                  └─ db/settings.rs: SELECT key, value FROM settings       │
│                       └─ merge over DEFAULT_SETTINGS (Rust-side source    │
│                          of truth) → Settings struct                      │
│             ◄─ Settings (serde JSON, camelCase)                           │
│        └─ store.$patch(settings)                                          │
│        └─ applySideEffects()  → i18n locale, theme vars,                  │
│                                  reduced-motion, animation speed          │
└───────────────────────────────────────────────────────────────────────────┘

┌── WRITE ──────────────────────────────────────────────────────────────────┐
│ SettingsView.vue  ──v-model──►  useSettingsStore().update('theme','nord')  │
│   1. optimistic:  state.theme = 'nord'         (UI updates immediately)   │
│   2. invoke('settings_upsert', { key:'theme', value:'nord' })             │
│        └─ UPSERT INTO settings … ON CONFLICT(key) DO UPDATE               │
│        └─ if syncable → app.emit('sync://settings-changed', { key })      │
│   3a. Ok  → done                                                          │
│   3b. Err → rollback to previous value + ui://toast(error)                │
└───────────────────────────────────────────────────────────────────────────┘

┌── EXTERNAL CHANGE (sync pull / other window) ─────────────────────────────┐
│ Rust writes settings → app.emit('settings://changed', ChangedKeys)        │
│   └─ every window: listen('settings://changed') → store.$patch(delta)     │
│        └─ applySideEffects() for affected keys only                       │
└───────────────────────────────────────────────────────────────────────────┘
```

### 1.3 Store shape

```ts
// src/stores/settings.ts
export const useSettingsStore = defineStore('settings', () => {
  const state = reactive<Settings>({ ...DEFAULTS })
  const hydrated = ref(false)

  async function hydrate() {
    Object.assign(state, await ipc.settings.getAll())
    hydrated.value = true
    applySideEffects(state)
    unlisten = await listen<Partial<Settings>>('settings://changed', e => {
      Object.assign(state, e.payload)
      applySideEffects(e.payload)
    })
  }

  async function update<K extends keyof Settings>(key: K, value: Settings[K]) {
    const previous = state[key]
    state[key] = value                                   // optimistic
    try { await ipc.settings.upsert(key as string, value) }
    catch (err) { state[key] = previous; toast.error(String(err)); throw err }
  }

  return { ...toRefs(state), hydrated, hydrate, update }
})
```

### 1.4 Design decisions

- **Defaults move to Rust.** OpenTubeX defines defaults in the Vuex `state` object, so a fresh install has no persisted row until first write. Slytube defines `DEFAULT_SETTINGS` in Rust and merges DB rows over it — the backend can then answer settings questions (e.g. proxy config for yt-dlp) without asking the UI, which `ytDlp.js:260` currently has to do awkwardly.
- **Optimistic with rollback.** OpenTubeX writes to Vuex and fires IPC without awaiting; a failed write leaves the UI lying. Slytube rolls back.
- **Typed keys.** `settings_upsert(key: String, value: JsonValue)` for the generic path; hot/structured settings (proxy, download dir, yt-dlp source) get dedicated typed commands so Rust can validate and react.
- **Explicit sync registry.** No mutation-name string decoding (see [`02-component-mapping.md`](02-component-mapping.md) §6.4).

### 1.5 Parity checklist

- [ ] All settings keys from `store/modules/settings.js` `state` + `customState` represented
- [ ] `applyKeyboardShortcutOverrides` / `sanitizeKeyboardShortcutOverrides` ported
- [ ] Locale loading (`loadLocale`, `activeLocales.json`) triggered on locale change
- [ ] `DEFAULT_THUMBNAIL_SIZE`, `DEFAULT_FIXED_TAB_WIDTH`, `DEFAULT_SEARCH_ENGINES_SETTING` preserved
- [ ] `helpers/settings-migrations.js` equivalents run once, flagged in `schema_meta`
- [ ] `isSettingSyncable` / `isSettingSyncEnabled` semantics preserved

---

## 2. Video Information Flow

### 2.1 Baseline

`views/Watch/Watch.js` picks a backend per settings and falls back between them:

```js
// src/renderer/views/Watch/Watch.js
:1317  await this.getVideoInformationLocal(loadGeneration)      // youtubei.js
:1320  this.getVideoInformationInvidious(loadGeneration)        // Invidious REST
:2311  this.getVideoInformationInvidious(loadGeneration)        // fallback local → invidious
:2543  this.getVideoInformationLocal(loadGeneration)            // fallback invidious → local
```

`loadGeneration` (`videoLoadGeneration`) is a monotonically increasing counter used to discard results from superseded loads — essential when the user navigates quickly.

- **Local path:** `helpers/api/local.js` (2,573 lines) — `youtubei.js`, PoToken acquisition at `:477`, signature deciphering at `:745`, `pot=` query param at `:749`, `/pot/<token>` path segment at `:783`.
- **Invidious path:** `helpers/api/invidious.js` (1,009 lines) — `invidiousFetch` (`:39`), `invidiousGetVideoInformation` (`:422`), instance selection from `store/modules/invidious.js` (`randomArrayItem` over `invidiousInstancesList`, refreshed from `https://api.invidious.io/instances.json` or the bundled `static/invidious-instances.json`).

### 2.2 Slytube flow (phased)

`youtubei.js` has no mature Rust equivalent, so it **stays in the webview** for Phase 1–2. Invidious is plain REST and moves to Rust for header control, proxy support, and caching.

```
┌── PHASE 1–2 ──────────────────────────────────────────────────────────────┐
│ WatchView.vue  onMounted / route change                                   │
│   └─ useWatchStore().load(videoId)                                        │
│        gen = ++generation                     ← loadGeneration equivalent │
│        backend = settings.backendPreference                               │
│                                                                           │
│   ┌─ LOCAL (webview) ───────────────────────────────────────────────────┐ │
│   │ services/youtube/local.ts  (youtubei.js)                            │ │
│   │   1. innertube.getInfo(videoId)                                     │ │
│   │   2. needs PoToken?  → invoke('get_potoken', { videoId, context })  │ │
│   │                        ────────────────► §6                         │ │
│   │   3. decipher formats, append pot= / /pot/<token>                   │ │
│   └─────────────────────────────────────────────────────────────────────┘ │
│   ┌─ INVIDIOUS (Rust) ──────────────────────────────────────────────────┐ │
│   │ invoke('invidious_get_video', { videoId })                          │ │
│   │   services/invidious.rs                                             │ │
│   │     • instance from AppState (health-scored, not purely random)     │ │
│   │     • shared reqwest::Client (proxy + Authorization applied)        │ │
│   │     • 60 s in-memory response cache (moka)                          │ │
│   │     • normalise → VideoInfo (same shape as the local path)          │ │
│   └─────────────────────────────────────────────────────────────────────┘ │
│                                                                           │
│   if (gen !== generation) return          ← discard superseded result     │
│   store.video = normalised                                                │
│   on failure → try the other backend once (mirrors :2311 / :2543)         │
│                                                                           │
│ WatchView.vue renders from useWatchStore()                                │
│   VideoPlayer ← store.formats                                             │
│   Description ← store.video.description                                   │
│   Comments    ← lazy, separate flow (watchComments.js)                    │
│   Related     ← store.video.related                                       │
└───────────────────────────────────────────────────────────────────────────┘
```

### 2.3 Normalisation contract

Both backends must produce one `VideoInfo` type. OpenTubeX normalises ad-hoc at each call site; Slytube defines it once in TypeScript (Phase 1) and, once Invidious is in Rust, in `serde` structs that generate the same shape.

```ts
export interface VideoInfo {
  videoId: string; title: string; description: string
  author: { id: string; name: string; thumbnail?: string; subscriberText?: string }
  lengthSeconds: number; viewCount: number; published: number
  isLive: boolean; isUpcoming: boolean; isPremiere: boolean
  formats: Format[]; captions: Caption[]; storyboards?: Storyboard[]
  chapters?: Chapter[]; related: VideoTeaser[]
  source: 'local' | 'invidious'          // provenance, for diagnostics
}
```

### 2.4 Instance selection improvement

`store/modules/invidious.js` (`:72`) picks an instance with `randomArrayItem` on every call — so a dead instance is re-picked repeatedly. Rust keeps a health score per instance (consecutive failures, last-success latency) in `AppState` and prefers healthy ones, falling back to the bundled `static/invidious-instances.json` when the remote list is unreachable (as `:60` does today).

### 2.5 History recording

Watch.vue records history on successful video load:

```ts
// Watch.vue — called inside load() after getVideo succeeds
await historyStore.addToHistory({
  videoId: video.value.id,
  title: video.value.title,
  author: video.value.author,
  authorId: video.value.authorId,
  lengthSeconds: video.value.lengthSeconds,
  timeWatched: new Date().toISOString(),  // ISO-8601 string
  watchProgress: 0,
  isWatched: true,
  isLive: video.value.isLive,
})
```

Navigation between videos uses Vue's `watch(videoId, (newId, oldId) => { if (newId !== oldId) load() })` — when the route's videoId query/param changes, the component reloads.

`historyStore` (`src/stores/history.ts`) is a Pinia store that:
1. Updates `historyCacheSorted` / `historyCacheById` optimistically.
2. Calls `invoke('db_history_upsert', { entry })` to persist.

> **Note:** Unlike OpenTubeX, the like/dislike buttons have been replaced with **Add to playlist**, **Download**, and **Watch later** actions.

### 2.6 Parity checklist

- [ ] `loadGeneration` cancellation semantics preserved (all four call sites)
- [ ] Bidirectional backend fallback preserved
- [ ] PoToken applied both as `pot=` query and `/pot/<token>` path
- [ ] `paidPromotionDurationMs`, `isPremiere`, `clientInfo` fields carried through (`local.js:555`, `:619`)
- [ ] `youtubeImageUrlToInvidious` (`invidious.js:586`) applied when the Invidious backend is active
- [ ] Age-restricted / unavailable states surface as typed errors, not thrown strings

---

## 3. Download Flow

### 3.1 Baseline

`src/main/ytDlp.js` (1,375 lines) spawns yt-dlp and streams parsed progress to every renderer:

```js
:1134  const child = spawn(executable, args, { windowsHide: true })
:1140  const status = { id, videoId, playlistId, playlistKey, title, thumbnail,
                        status:'downloading', percent:0, speed:null, eta:null,
                        destination:null, destinations:[], errorMessage:null }
:1163  function sendStatus(force=false) {           // 500 ms throttle
         if (!force && Date.now() - lastSent < 500) return
         broadcastToRenderers(IpcChannels.YT_DLP_DOWNLOAD_STATUS, { ...status })
       }
:1185  handleStdoutLine(line)                       // FINAL_PATH_PREFIX, PROGRESS_REGEX,
                                                    // DESTINATION_REGEX, MERGER_REGEX
```

Records persist to `userData/downloads.json` (last 200 non-active, `:151`). The renderer side is a 41-line Vuex module whose only job is `upsertYtDlpDownload` / `removeYtDlpDownload` / `clearFinishedYtDlpDownloads`.

### 3.2 Slytube flow (Phase 1 implementation)

```
┌── START ──────────────────────────────────────────────────────────────────┐
│ Watch.vue / DownloadButton → useDownloads().startDownload(args)           │
│   const id = await invoke('yt_dlp_download', { args: downloadArgs })       │
└───────────────────────────────────────────────────────────────────────────┘
                                    │
┌── RUST: yt_dlp/commands.rs::yt_dlp_download ──────────────────────────────┐
│ 1. resolve executable      ← get_binary_path (binaries/ dir or sidecar)   │
│ 2. build argv              ← YtDlpDownloadArgs → mode, quality, format,   │
│      chapters, SponsorBlock, subtitles, thumbnail, metadata, custom args  │
│ 3. download_id = state.download_counter.lock().await += 1                 │
│ 4. INSERT INTO download_records (id, video_id, title, status, percent,    │
│                                  destination, created_at)                 │
│      VALUES (…, 'pending', 0.0, '', …)                                    │
│ 5. spawn via tokio::process::Command (not Tauri sidecar in Phase 1)       │
│ 6. state.active_downloads.lock().await.insert(id, child)                  │
│ 7. tauri::async_runtime::spawn(monitor_download(id, …))                   │
│ 8. Ok(download_id)  → returns u64, not a Channel                          │
└───────────────────────────────────────────────────────────────────────────┘
                                    │
┌── PROGRESS FAN-OUT ───────────────────────────────────────────────────────┐
│ monitor_download reads stdout via tokio::io::BufReader:                    │
│   progress line  → app.emit("yt-dlp-progress", {id,percent,speed,eta})    │
│                    + UPDATE download_records SET percent, status          │
│   destination l. → app.emit("yt-dlp-destination", {id, destination})      │
│                    + UPDATE download_records SET destination, title       │
│   exit code 0    → app.emit("yt-dlp-complete", {id})                      │
│                    + UPDATE download_records SET status='completed'       │
│   non-zero exit  → app.emit("yt-dlp-error", {id, error})                  │
│                    + UPDATE download_records SET status='failed'          │
│ cancel           → app.emit("yt-dlp-cancelled", id)                       │
└───────────────────────────────────────────────────────────────────────────┘
                                    │
┌── UI ─────────────────────────────────────────────────────────────────────┐
│ useDownloads() composable (src/composables/useData.ts):                   │
│   downloads: Ref<DownloadStatus[]>                                        │
│   loadDownloads() → invoke('yt_dlp_list') → Vec<DownloadStatus>           │
│   listen('yt-dlp-*') → patch in-memory array by id                       │
│ DownloadsView.vue renders from useDownloads()                             │
└───────────────────────────────────────────────────────────────────────────┘
```

### 3.3 Parsing rules — carry over verbatim

```rust
static PROGRESS: Lazy<Regex> = Lazy::new(|| Regex::new(
    r"^\[download\]\s+(\d+(?:\.\d+)?)%(?:.*?\bat\s+(\S+))?(?:.*?\bETA\s+(\S+))?").unwrap());
static DESTINATION: Lazy<Regex> = Lazy::new(|| Regex::new(
    r"^\[(?:download|ExtractAudio)\] Destination: (.+)$").unwrap());
static MERGER: Lazy<Regex> = Lazy::new(|| Regex::new(
    r#"^\[Merger\] Merging formats into "(.+)"$"#).unwrap());
const FINAL_PATH_PREFIX: &str = "__OPENTUBEX_FILE__:";   // rename with care —
                                                          // it is emitted by a
                                                          // --print template
```

`destinations` accumulates unique paths (a merge produces several); `destination` holds the final one.

### 3.4 Binary acquisition sub-flow

```
useSettingsStore.ytDlpSource = 'managed' and binary missing
  → invoke('download_ytdlp_binary', { binary:'yt-dlp', onProgress: channel })
      channel := Channel<BinaryProgress { received, total, phase }>
      Rust: pick repo from { stable: yt-dlp/yt-dlp,
                             nightly: yt-dlp/yt-dlp-nightly-builds,
                             master:  yt-dlp/yt-dlp-master-builds }
            stream download (reqwest) → validators (readDownloadValidators:405 /
            writeDownloadValidators:424) → unzip if ffmpeg (extractZipEntry:440)
            → install + chmod +x (installBinary:499)
  → replaces YT_DLP_BINARY_DOWNLOAD_PROGRESS and the manual listener Set
    at interface.js:13-20
```

### 3.5 Cancellation and shutdown

`cancel_download(id)` sets the entry's `CancellationToken`, kills the child, marks `status = cancelled`, emits once. On `RunEvent::ExitRequested`, the equivalent of `shutdownYtDlpDownloads` (`:167`) awaits child termination with a timeout before allowing exit, so records are flushed — but records are already in SQLite, so the `flushYtDlpDownloadRecords` promise-queue (`:163`) is no longer needed.

### 3.6 Improvements over the baseline

| Baseline | Slytube |
|---|---|
| 500 ms throttle to protect the JS event loop | Parsing off-thread; throttle becomes a UI-smoothing choice (~100–250 ms) |
| Broadcast to all windows + `isOpenTubeXUrl` guard | Per-invocation `Channel` + capability-gated broadcast |
| `downloads.json`, last 200, rewritten wholesale | SQLite rows, indexed, queryable, unbounded with retention policy |
| Manual listener `Set` + `removeListener` | `Channel` cleaned up on GC |
| Cancel via `Map<id, {child, cancelled}>` flag | `CancellationToken` + typed registry |

### 3.7 Parity checklist

- [ ] All five states: `downloading`, `processing`, `completed`, `failed`, `cancelled`
- [ ] Field truncations preserved (playlistId 128, playlistKey 255, title 255, thumbnail 2048)
- [ ] Playlist / multi-video / single-video URL construction
- [ ] `--download-sections` with `TIME_REGEX` validation
- [ ] `getInfoProbeKey` in-flight dedupe (`:647`) and abort signals (`:24`)
- [ ] `handleYtDlpGetPlaybackInfo` (`:864`) + `mapPlaybackFormat` (`:832`) for the playback-engine path
- [ ] ffmpeg resolution and version probe alongside yt-dlp

---

## 4. Sync Flow

### 4.1 Baseline

Three layers, all in the renderer:

- `store/modules/sync-server.js` (737 L) — orchestration; `EVENT_SYNC_DEBOUNCE_MS = 1500`, `ENCRYPTED_SYNC_RETRIES = 3`, `activeSyncClients` set, `withSyncLock`, `runSync` (`:103`), `LEGACY_ENCRYPTED_COLLECTIONS`.
- `helpers/sync-server.js` (1,040 L) — REST client; `apiRequest` (`:86`) against `/subscriptions/`, `/subscriptions/bulk`, `/subscriptions/groups/`, `/playlists/`, `/watch_history/`, `/watch_history/bulk`, `/channel_playback_speeds/`.
- `helpers/sync-server-privacy.js` (363 L) — the crypto envelope, all via `crypto.subtle` **on the UI thread**.

Envelope, exactly as implemented:

```
AAD        "OpenTubeX encrypted sync v1"                  (:5)
KDF        PBKDF2 / SHA-256 / PBKDF2_ITERATIONS / 16-byte salt   (:55-73, :140)
Cipher     AES-GCM-256 / 12-byte IV                       (:166, :182)
Compress   optional gzip (envelope.compression.name)      (:24-29)
Validation version + kdf.{name,hash,iterations} + cipher.name  (:76-95)
Legacy     decryptLegacySyncDocument                      (:155)
```

Scheduling is driven by the Vuex plugin in `store/index.js`, which infers a "reason" from mutation/action names and calls `dispatch('scheduleSyncServer', reason)`.

### 4.2 Slytube flow

```
┌── TRIGGER ────────────────────────────────────────────────────────────────┐
│ Any store mutation registered in SYNC_REASONS                             │
│   → useSyncStore().schedule(reason)      debounce 1500 ms                 │
│   → invoke('trigger_sync', { reason })                                    │
│ Also: interval timer, app resume, manual "Sync now"                       │
└───────────────────────────────────────────────────────────────────────────┘
                                   │
┌── RUST: sync/mod.rs::run_sync ────────────────────────────────────────────┐
│  guard = state.sync.lock().await                ← withSyncLock (:96)      │
│  emit sync://status { state:'running' }                                   │
│                                                                           │
│  PUSH                                                                     │
│   1. collect dirty rows per collection (sqlx, WHERE dirty = 1)            │
│   2. serde_json::to_vec                                                   │
│   3. gzip (flate2) if above threshold                                     │
│   4. crypto.rs::encrypt:                                                  │
│        salt   = 16 random bytes  (or stored salt)                         │
│        key    = pbkdf2_hmac::<Sha256>(passphrase, salt, ITERATIONS)       │
│        iv     = 12 random bytes                                           │
│        ct     = Aes256Gcm::encrypt(iv, plaintext + AAD)                   │
│        → envelope { version, kdf{…}, cipher{…}, compression?, salt,       │
│                     iv, data }                                            │
│   5. sync/client.rs → PUT/POST/PATCH the endpoint for that collection     │
│      retry ×3 on transient failure       ← ENCRYPTED_SYNC_RETRIES         │
│                                                                           │
│  PULL                                                                     │
│   6. GET remote snapshots                                                 │
│   7. parse + validate envelope (reject on any field mismatch)             │
│   8. decrypt → gunzip → serde_json                                        │
│   9. merge (last-write-wins on updated_at; tombstones honoured)           │
│  10. write inside ONE sqlx transaction, clear dirty flags                 │
│  11. per changed collection: app.emit('sync://{collection}-changed')      │
│  12. emit sync://status { state:'idle', lastSync, counts }                │
└───────────────────────────────────────────────────────────────────────────┘
                                   │
┌── UI ─────────────────────────────────────────────────────────────────────┐
│ listen('sync://history-changed')   → useHistoryStore().refresh()          │
│ listen('sync://playlists-changed') → usePlaylistsStore().refresh()        │
│ listen('sync://settings-changed')  → useSettingsStore() patch             │
│ listen('sync://status')            → useSyncStore().status (badge/spinner)│
└───────────────────────────────────────────────────────────────────────────┘
```

The seven `SYNC_*` IPC channels (`SYNC_SETTINGS`, `SYNC_HISTORY`, `SYNC_WATCH_STATS`, `SYNC_SEARCH_HISTORY`, `SYNC_PROFILES`, `SYNC_PLAYLISTS`, `SYNC_SUBSCRIPTION_CACHE`) map one-to-one onto `sync://{collection}-changed` events.

### 4.3 Crypto module

```rust
// src-tauri/src/sync/crypto.rs
const AAD: &[u8] = b"OpenTubeX encrypted sync v1";
const PBKDF2_ITERATIONS: u32 = /* must equal helpers/sync-server-privacy.js */;

pub fn derive_key(passphrase: &str, salt: &[u8; 16]) -> [u8; 32] {
    let mut key = [0u8; 32];
    pbkdf2_hmac::<Sha256>(passphrase.as_bytes(), salt, PBKDF2_ITERATIONS, &mut key);
    key
}

pub fn encrypt(plaintext: &[u8], key: &[u8; 32]) -> Result<Envelope, CryptoError> {
    let iv: [u8; 12] = rand::random();
    let ct = Aes256Gcm::new(key.into())
        .encrypt(Nonce::from_slice(&iv), Payload { msg: plaintext, aad: AAD })?;
    Ok(Envelope::v1(iv, ct))
}
```

**Compatibility is non-negotiable.** Required tests:

1. Rust encrypts → JS (`decryptSyncDocument`) decrypts.
2. JS encrypts → Rust decrypts.
3. Legacy fixtures decrypt via the `decryptLegacySyncDocument` path.
4. Every envelope-validation rejection in `parseEnvelope` (`:76-95`) has a matching Rust rejection.
5. Gzip and non-gzip payloads both round-trip.

### 4.4 Why this moves to Rust

- `derivePrivacyKey` (PBKDF2 with high iteration count) currently blocks the renderer's main thread, visibly stalling the UI during sync setup. In Rust it runs on a blocking task.
- The sync client can run without a focused window.
- Multi-window consistency: today each window runs its own sync client (`activeSyncClients`, `:44`); Rust holds one lock for the whole app.

### 4.5 Parity checklist

- [ ] `EVENT_SYNC_DEBOUNCE_MS = 1500`, `ENCRYPTED_SYNC_RETRIES = 3`
- [ ] `allowDataLoss` path (`runSync(context, { allowDataLoss })`, `:103`)
- [ ] All REST endpoints and verbs from `helpers/sync-server.js`
- [ ] Bulk endpoints (`/subscriptions/bulk`, `/watch_history/bulk`)
- [ ] Cancellation (`cancelActiveSyncClients`, `:58`)
- [ ] Typed errors from `helpers/sync-server-errors.js`
- [ ] Tab-session sync (`TABS_GET_SYNC_SESSIONS` / `TABS_APPLY_SYNC_SESSIONS`)

---

## 5. PoToken Flow

### 5.1 Baseline

```
local.js:477   contentPoToken = await window.ftElectron.generatePoToken(videoId, context)
   → interface.js:151  ipcRenderer.invoke(GENERATE_PO_TOKEN, videoId, context)
   → index.js:2734     ipcMain.handle(...) → generatePoToken(videoId, context, proxyUrl)
   → poTokenGenerator.js:50
        enqueueAsyncFunction(internalGeneratePotoken, …)   // serialised queue
        enqueueAsyncFunction(cleanupSession)               // after each run
        first call → sharedInit():
          session.fromPartition('potoken', { cache:false })
          permission check/request handlers → false
          UA copied from default session
          onBeforeSendHeaders: Referer/Origin/Sec-Fetch-*/X-Youtube-Bootstrap-Logged-In
          onHeadersReceived:  Access-Control-Allow-{Origin,Methods}: *
          onBeforeRequest:    cancel cspReport + ping
          read botGuardScript.js, rewrite `export{X as default};` → `;X(FT_PARAMS)`
        internalGeneratePotoken():
          setProxy if configured
          WebContentsView { sandbox, contextIsolation, offscreen,
                            backgroundThrottling:false, v8CacheOptions:'none' }
          windowOpenHandler → deny;  audio muted;  bounds 1920×1080
          debugger.attach() → Emulation.setDeviceMetricsOverride(1920×1080, dSF 1,
                              landscapePrimary)
          loadURL('data:text/html,…', { baseURLForDataURL: 'https://www.youtube.com/' })
          executeJavaScript(script.replace('FT_PARAMS', `"${videoId}",${context}`))
        finally: webContents.close()
        cleanupSession(): closeAllConnections() + clearData()
   ← token
local.js:619   poToken: contentPoToken
local.js:749   urlObject.searchParams.set('pot', poToken)        // DASH/query form
local.js:783   decipheredUrlObject.pathname += `/pot/${encodeURIComponent(poToken)}`
```

`botGuardScript.js` POSTs to `https://www.youtube.com/youtubei/v1/att/get` with `X-Goog-Visitor-Id`, `X-Youtube-Client-Version`, `X-Youtube-Client-Name: 1` from `context`, then uses `BotGuardClient` + `WebPoMinter` from `bgutils-js`.

### 5.2 Slytube flow

```
┌── REQUEST ────────────────────────────────────────────────────────────────┐
│ services/youtube/local.ts (webview)                                       │
│   const pot = await invoke<string>('get_potoken', { videoId, context })   │
└───────────────────────────────────────────────────────────────────────────┘
                                   │
┌── RUST: potoken/mod.rs ───────────────────────────────────────────────────┐
│ 1. cache lookup — key (videoId, visitorData), TTL ~6 h                    │
│      hit → return immediately (NEW: OpenTubeX regenerates every time)     │
│ 2. guard = state.potoken.lock().await     ← queueGuardian equivalent      │
│ 3. build hidden webview:                                                  │
│      WebviewWindowBuilder::new(app, "potoken", App("potoken.html"))       │
│        .visible(false).incognito(true).focused(false).skip_taskbar(true)  │
│        .inner_size(1920.0, 1080.0)                                        │
│        .initialization_script(EMULATION_SHIM)   ← replaces CDP Emulation  │
│      capabilities/potoken.json → "permissions": []   (no invoke surface)  │
│ 4. script = BOTGUARD_JS.replace("FT_PARAMS", &format!("\"{vid}\",{ctx}")) │
│ 5. win.eval(&wrap_with_result_post(script))                               │
│      wrapper posts the result back:                                       │
│        Promise.resolve(run()).then(t => __TAURI__.event.emit(             │
│          'potoken://result', { ok:true, token:t }))                       │
│        .catch(e => …{ ok:false, error:String(e) })                        │
│ 6. await once::<PoTokenResult>("potoken://result") with a 30 s timeout    │
│ 7. win.close()                            ← finally { close() }           │
│ 8. clear webview data                     ← closeAllConnections+clearData │
│ 9. cache + return                                                         │
└───────────────────────────────────────────────────────────────────────────┘
                                   │
┌── CONSUME ────────────────────────────────────────────────────────────────┐
│ local.ts: attach as `pot=` (query) and `/pot/<token>` (path), per format  │
└───────────────────────────────────────────────────────────────────────────┘
```

Note that `potoken://result` is emitted **from** the isolated webview, which means it needs `core:event:allow-emit` — the single, deliberate exception to its otherwise-empty capability set. Scope it to that one event.

### 5.3 The emulation shim

The CDP `Emulation.setDeviceMetricsOverride` call (`poTokenGenerator.js:190-204`) has no Tauri equivalent. It is approximated by an `initialization_script` that runs before any page script:

```js
// EMULATION_SHIM — src-tauri/src/potoken/shim.js
(() => {
  const def = (obj, prop, value) =>
    Object.defineProperty(obj, prop, { get: () => value, configurable: false })
  def(window.screen, 'width', 1920);       def(window.screen, 'height', 1080)
  def(window.screen, 'availWidth', 1920);  def(window.screen, 'availHeight', 1080)
  def(window.screen, 'colorDepth', 24);    def(window.screen, 'pixelDepth', 24)
  def(window, 'devicePixelRatio', 1)
  def(window, 'outerWidth', 1920);         def(window, 'outerHeight', 1080)
  def(window, 'innerWidth', 1920);         def(window, 'innerHeight', 1080)
  Object.defineProperty(window.screen, 'orientation', {
    get: () => ({ type: 'landscape-primary', angle: 0 }), configurable: false,
  })
})()
```

**This is the weakest point in the migration.** A JS-level shim is observable in ways a CDP-level override is not (property descriptors, prototype identity, cross-frame consistency). Header injection is also unavailable — the `onBeforeSendHeaders` rules must be reproduced by overriding `window.fetch` inside the isolated page, which is likewise more detectable.

### 5.4 Fallback ladder

1. **Preferred** — hidden `WebviewWindow` + shim, as above.
2. **Fallback A** — Rust performs the `att/get` request via `reqwest` (correct headers, undetectable) and the webview only executes the BotGuard VM challenge. Reduces the emulated surface.
3. **Fallback B** — a Node sidecar bundling `bgutils-js` and a headless browser, invoked from Rust. Heaviest, but reuses a known-good execution environment.

Run the spike **before** committing to the Watch view (see [`01-electron-vs-tauri.md`](01-electron-vs-tauri.md) §9).

### 5.5 Parity checklist

- [ ] Only one generation at a time (mutex)
- [ ] Per-run data clearing
- [ ] Proxy honoured when configured
- [ ] Script rewrite (`export{X as default};` → `;X(FT_PARAMS)`) preserved
- [ ] `context` injected as raw JSON, not a string literal
- [ ] Timeout + typed error; the Watch flow degrades gracefully (`local.js:484` logs and continues)
- [ ] Isolated webview never gains access to app commands or user data

---

## 6. Tab Flow

### 6.1 Baseline

`TabManager.js` (3,046 lines) owns tabs in the main process using `WebContentsView`, coordinating with renderer services (`TabNavigationService.js` 792 L, `TabMediaCoordinator.js` 200 L, `TabRuntimeRegistry.js` 93 L, `TabContext.js` 76 L, `TabLifecycleService.js` 54 L) and a 532-line Vuex module. 45 IPC channels.

The Vuex module already models most tab state client-side:

```js
// src/renderer/store/modules/tabs.js:9-21
const state = {
  tabs: [], activeTabId: null, selectedTabIds: [], presentedTabId: null,
  mainPresentedTabId: null, selectionRevision: 0, transitionRevision: 0,
  transitionTargetTabId: null, containerIds: [], tabBarScrollPosition: 0,
  currentWatchTimestamps: {}
}
// MAX_LOGICAL_HISTORY_ENTRIES = 100, NAV_HISTORY_DISPLAY_LIMIT = 15
```

Per-tab navigation history with a windowed display list is computed in `getTabHistoryState` (`:35-67`).

### 6.2 Slytube flow (design A: single window, virtual tabs)

```
┌── CREATE ─────────────────────────────────────────────────────────────────┐
│ TabBar.vue "+"  → useTabsStore().create({ route:'/', title:'New Tab' })    │
│   1. id = crypto.randomUUID(); tabs.push({ id, route, history:[route],    │
│                                            historyIndex:0, pinned:false, … })│
│   2. activeTabId = id                                                     │
│   3. router.push(route)                                                   │
│   4. debounced invoke('tabs_persist_session', { tabs, activeTabId })      │
│        └─ UPSERT INTO tab_sessions   ← replaces TabSessionStore.js        │
└───────────────────────────────────────────────────────────────────────────┘

┌── ACTIVATE ───────────────────────────────────────────────────────────────┐
│ useTabsStore().activate(id)                                               │
│   activeTabId = id → router.replace(tab.route)                            │
│   keep-alive preserves each tab's component tree                          │
│   app.emit → 'tabs://active-changed' for other windows                    │
└───────────────────────────────────────────────────────────────────────────┘

┌── NAVIGATE ───────────────────────────────────────────────────────────────┐
│ router.afterEach → useTabsStore().recordNavigation(activeTabId, route)     │
│   push onto tab.history (cap 100)  ← MAX_LOGICAL_HISTORY_ENTRIES           │
│   getTabHistoryState() windowing preserved (limit 15, half-window 7)      │
│   ← replaces TABS_UPDATE_ROUTE / TABS_UPDATE_NAV_HISTORY /                │
│     TABS_UPDATE_TITLE / TABS_GO_HISTORY                                   │
└───────────────────────────────────────────────────────────────────────────┘

┌── PREVIEW ────────────────────────────────────────────────────────────────┐
│ hover a tab → useTabsStore().requestPreview(id)                           │
│   invoke('tabs_capture_preview', { tabId })                               │
│     Rust: webview.capture() → downscale (image crate) → moka LRU          │
│           ← replaces tabPreviewCache.js + tabPreviewGeometry.js           │
│   ← Vec<u8> (raw) or blob URL; never base64 through the bridge            │
└───────────────────────────────────────────────────────────────────────────┘

┌── RESTORE (startup) ──────────────────────────────────────────────────────┐
│ invoke('tabs_get_state') → SELECT FROM tab_sessions                       │
│   ← replaces loadAllTabSessions / clearAllTabSessions (TabSessionStore.js)│
└───────────────────────────────────────────────────────────────────────────┘

┌── NEW WINDOW ─────────────────────────────────────────────────────────────┐
│ new WebviewWindow(`win-${n}`, { url:'/', title:'Slytube' })                │
│   ← replaces CREATE_NEW_WINDOW; each window owns an independent tabs store│
│   cross-window coordination via tabs://* broadcasts                       │
└───────────────────────────────────────────────────────────────────────────┘
```

### 6.3 Where Rust participates

| Concern | Owner | Reason |
|---|---|---|
| Tab list, order, active/selected, pinned, colour | Pinia | Pure UI state; already client-side today |
| Per-tab navigation history | Pinia | Logical, not webview, history |
| Session persistence | Rust (`tab_sessions`) | Must survive crashes; syncable |
| Preview capture + cache | Rust | Needs webview capture and image processing |
| Favicon resolution | Rust (`net/favicon.rs`) | Network + cache; `RESOLVE_FAVICON` |
| Native context menus | Rust (`tauri::menu`) | `CONTEXT_MENU_OPEN` / `EXECUTE` |
| Fullscreen / PiP | Mixed | Window API in Rust, media API in webview |
| Cross-window broadcast | Rust | `app.emit` |

### 6.4 Media coordination

`TabMediaCoordinator.js` (200 L) ensures only one tab plays audio and pauses others on tab switch. In design A this is purely client-side: the store tracks `playbackState` per tab (`TABS_SET_PLAYBACK_STATE`) and pauses non-active players on `activate()`. `currentWatchTimestamps` (already in the Vuex state) carries resume positions.

### 6.5 Parity checklist

- [ ] All 45 `TABS_*`/context-menu/favicon channels accounted for
- [ ] `MAX_LOGICAL_HISTORY_ENTRIES = 100`, `NAV_HISTORY_DISPLAY_LIMIT = 15` and the windowing algorithm
- [ ] Restore-closed-tab stack (`TABS_RESTORE_CLOSED`)
- [ ] Pinned tabs ordered first; `tabOrder.js` semantics
- [ ] Multi-select (`selectedTabIds`, `selectionRevision`)
- [ ] Preview pause/refresh (`SET_PREVIEW_CAPTURE_PAUSED`, `REQUEST_PREVIEW_REFRESH`)
- [ ] Tab-bar scroll position persisted
- [ ] Session sync round-trip
- [ ] Keyboard shortcuts: new/close/reload/next/prev/restore/switch-1-9/toggle-orientation

---

## 7. Cross-Cutting Concerns

### 7.1 Error propagation

```rust
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("database error: {0}")]  Database(#[from] sqlx::Error),
    #[error("network error: {0}")]   Network(#[from] reqwest::Error),
    #[error("sidecar error: {0}")]   Sidecar(String),
    #[error("crypto error: {0}")]    Crypto(String),
    #[error("potoken error: {0}")]   PoToken(String),
    #[error("not found: {0}")]       NotFound(String),
    #[error("invalid input: {0}")]   Validation(String),
}
```

Serialised as `{ code, message, details? }` (see `../backend/02-tauri-commands.md` §Error Handling). The frontend maps `code` to a localised string; raw messages are never shown to users.

This is a real improvement: the 39 `ipcRenderer.send` call sites in OpenTubeX have **no error path at all**.

### 7.2 Startup sequence

```
1. Rust: run migrations (sqlx::migrate!) — includes the one-shot NeDB import
2. Rust: build AppState (pool, http client, registries)
3. Rust: register commands + create the main window
4. UI:   main.ts → createPinia() → useSettingsStore().hydrate()
5. UI:   applySideEffects (locale, theme, motion, animation speed)
6. UI:   useTabsStore().restore()  → invoke('tabs_get_state')
7. UI:   register global listeners (settings://, sync://, downloads://, tabs://)
8. UI:   mount app
9. Rust: post-mount — check yt-dlp binary, schedule the first sync
```

`APP_READY` disappears; Tauri's window lifecycle plus explicit hydration covers it.

### 7.3 Multi-window consistency

| State | Scope | Mechanism |
|---|---|---|
| Settings | Global | SQLite + `settings://changed` |
| Downloads | Global | SQLite + `downloads://status` |
| History / playlists / profiles | Global | SQLite + `sync://*-changed` |
| Tabs | Per-window | Local store + `tabs://*` for cross-window ops |
| Player state | Per-window | Local only |
| Sync status | Global | `sync://status` |

### 7.4 Performance guardrails

1. **No large payloads through `invoke`.** Previews and binaries use raw bytes or file paths, never base64 JSON.
2. **Paginate everything.** History and subscription feeds use SQL `LIMIT`/`OFFSET`, not full-collection loads (contrast `history.js` 280 L and `subscription-cache.js` 491 L doing JS-side filtering today).
3. **Debounce writes.** Settings 300 ms, tab sessions 1 s, sync 1500 ms.
4. **Throttle event emission.** Download progress ~100–250 ms per download.
5. **Cache with bounds.** `moka` for player cache, images, previews, Invidious responses — all with explicit capacity and TTL, replacing the unbounded `ImageCache.js`.
6. **Virtualise long lists.** Mandatory on WebKitGTK.

### 7.5 Flow summary

| Flow | Read path | Write path | Push mechanism |
|---|---|---|---|
| Settings | SQLite → `get_settings` → Pinia | Optimistic → `settings_upsert` → SQLite | `settings://changed` |
| Video info | youtubei.js (webview) / Invidious (Rust) → Pinia | — | — |
| Downloads | SQLite → `yt_dlp_list` → composable ref | `yt_dlp_download` → `tokio::process::Command` | `yt-dlp-*` events |
| History | SQLite → `db_history_find_all` → Pinia | `db_history_upsert` → SQLite | — |
| Sync | Server → Rust decrypt → SQLite → Pinia | Pinia dirty → Rust encrypt → server | `sync://*-changed` |
| PoToken | Cache / hidden webview | — | `potoken://result` (internal) |
| Tabs | SQLite → `tabs_get_state` → Pinia | Pinia → `tabs_persist_session` | `tabs://state-updated` |

# 02 - Tauri Commands Reference

> **Domain:** `backend`
> **Status:** Design specification (implementation target for `src-tauri/src/commands`)
> **Related:** [01-database-schema.md](01-database-schema.md), [03-yt-dlp-sidecar.md](03-yt-dlp-sidecar.md), [05-sync-encryption.md](05-sync-encryption.md)

---

## 1. Conventions

### 1.1 Naming

Commands are namespaced by module using a `<module>_<action>` snake_case name on the Rust side. The frontend calls them through a typed façade so call sites read as `db.settings.find()` rather than raw strings.

```
db/settings   → db_settings_find, db_settings_upsert, ...
db/profiles   → db_profiles_find, db_profiles_create, ...
sync          → sync_start, sync_status, ...
network       → network_test_proxy, network_resolve_favicon
window        → window_create, window_close, ...
shortcuts     → shortcuts_register, shortcuts_unregister
```

### 1.2 Serialisation contract

All db model structs in `src-tauri/src/db/models.rs` derive `Serialize`/`Deserialize` with `#[serde(rename_all = "camelCase")]`, so Rust `time_watched` becomes TS `timeWatched`. Tauri passes command *arguments* as camelCase automatically; return types opt in explicitly via the derive.

> **Phase 1 status:** `Setting`, `Playlist`, `PlaylistVideo`, `HistoryEntry`, `WatchStat`, `SearchEntry`, `SubscriptionCacheEntry`, `TabSession`, `DownloadRecord`, and `SyncState` all carry the attribute. `Profile` and `ProfileSubscription` still use default serde field names and will be updated when the profiles feature is wired up. The frontend `DbPlaylist` interface in `playlists.ts` already uses camelCase (`createdAt`), matching the Rust model without a translation layer.

### 1.3 Error model

```rust
// src-tauri/src/error.rs
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("database error: {0}")]      Database(#[from] sqlx::Error),
    #[error("io error: {0}")]            Io(#[from] std::io::Error),
    #[error("network error: {0}")]       Network(String),
    #[error("sidecar error: {0}")]       Sidecar(String),
    #[error("crypto error: {0}")]        Crypto(String),
    #[error("not found: {0}")]           NotFound(String),
    #[error("invalid input: {0}")]       Invalid(String),
    #[error("operation cancelled")]      Cancelled,
    #[error("{0}")]                      Other(String),
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorPayload {
    pub kind: &'static str,   // 'database' | 'network' | ... — stable, matchable
    pub message: String,
}

impl serde::Serialize for AppError {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        ErrorPayload { kind: self.kind(), message: self.to_string() }.serialize(s)
    }
}
```

All commands return `Result<T, AppError>`; the frontend `invoke` wrapper rethrows a typed `SlyTubeError` carrying `kind`.

### 1.4 Async and state

Every DB-touching command is `async` and takes `State<'_, AppState>`. Commands must never block the async runtime — CPU-heavy work (crypto, large JSON) goes through `tauri::async_runtime::spawn_blocking`.

### 1.5 Registration

```rust
// src-tauri/src/lib.rs
tauri::Builder::default()
    .plugin(tauri_plugin_shell::init())
    .plugin(tauri_plugin_fs::init())
    .plugin(tauri_plugin_dialog::init())
    .plugin(tauri_plugin_opener::init())
    .plugin(tauri_plugin_notification::init())
    .plugin(tauri_plugin_global_shortcut::Builder::new().build())
    .manage(AppState::new(pool))
    .invoke_handler(tauri::generate_handler![
        // db/settings
        commands::db::settings::find,
        commands::db::settings::upsert,
        commands::db::settings::find_one,
        commands::db::settings::update_bounds,
        // db/profiles
        commands::db::profiles::find,
        commands::db::profiles::create,
        commands::db::profiles::upsert,
        commands::db::profiles::delete,
        commands::db::profiles::delete_all,
        commands::db::profiles::persist_sync,
        // db/playlists
        commands::db::playlists::find,
        commands::db::playlists::create,
        commands::db::playlists::upsert,
        commands::db::playlists::delete,
        commands::db::playlists::delete_all,
        commands::db::playlists::upsert_video_by_playlist_name,
        commands::db::playlists::upsert_videos_by_playlist_id,
        commands::db::playlists::delete_video_id_by_playlist_id,
        commands::db::playlists::delete_video_ids_by_playlist_id,
        commands::db::playlists::delete_all_videos_by_playlist_id,
        commands::db::playlists::reorder_videos,
        commands::db::playlists::persist_sync,
        // db/history
        commands::db::history::find,
        commands::db::history::upsert,
        commands::db::history::overwrite,
        commands::db::history::update_watch_progress,
        commands::db::history::delete,
        commands::db::history::delete_all,
        commands::db::history::delete_older_than,
        commands::db::history::apply_sync,
        // db/watch_stats
        commands::db::watch_stats::find,
        commands::db::watch_stats::add_watch_time,
        commands::db::watch_stats::delete_all,
        commands::db::watch_stats::persist_sync,
        // db/search_history
        commands::db::search_history::find,
        commands::db::search_history::upsert,
        commands::db::search_history::delete,
        commands::db::search_history::delete_all,
        // db/subscription_cache
        commands::db::subscription_cache::find,
        commands::db::subscription_cache::find_one,
        commands::db::subscription_cache::upsert,
        commands::db::subscription_cache::delete,
        commands::db::subscription_cache::delete_all,
        commands::db::subscription_cache::persist_sync,
        // db/tab_sessions
        commands::db::tab_sessions::save,
        commands::db::tab_sessions::load,
        commands::db::tab_sessions::clear,
        // sync
        commands::sync::start,
        commands::sync::status,
        commands::sync::cancel,
        commands::sync::get_snapshot,
        // network
        commands::network::test_proxy,
        commands::network::resolve_favicon,
        // window
        commands::window::create,
        commands::window::close,
        commands::window::minimize,
        commands::window::tray,
        // shortcuts
        commands::shortcuts::register,
        commands::shortcuts::unregister,
    ])
```

### 1.6 Frontend façade

```ts
// src/lib/ipc.ts
import { invoke } from '@tauri-apps/api/core'

export class SlyTubeError extends Error {
  constructor(public kind: string, message: string) { super(message) }
}

async function call<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  try {
    return await invoke<T>(cmd, args)
  } catch (e: any) {
    throw new SlyTubeError(e?.kind ?? 'other', e?.message ?? String(e))
  }
}

export const db = {
  settings: {
    find:         ()                        => call<Setting[]>('db_settings_find'),
    findOne:      (key: string)             => call<Setting | null>('db_settings_find_one', { key }),
    upsert:       (key: string, value: unknown) => call<void>('db_settings_upsert', { key, value }),
    updateBounds: (bounds: WindowBounds)    => call<void>('db_settings_update_bounds', { bounds }),
  },
  // ...
}
```

---

## 2. Module: `db/settings`

Backing table: [`settings`](01-database-schema.md#41-settings).

### 2.1 `db_settings_find`

Returns every setting row, decoded from its `value_type` discriminator.

```rust
#[derive(Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Setting {
    pub key: String,
    pub value: serde_json::Value,
    pub updated_at: i64,
}

#[tauri::command]
pub async fn find(state: State<'_, AppState>) -> Result<Vec<Setting>, AppError> {
    Ok(db::settings::find_all(&state.pool).await?)
}
```

| Field | Type | Notes |
|---|---|---|
| `key` | `string` | Unique setting identifier |
| `value` | `unknown` | Decoded JSON value |
| `updatedAt` | `number` | Epoch ms |

### 2.2 `db_settings_find_one`

```rust
#[tauri::command]
pub async fn find_one(
    state: State<'_, AppState>,
    key: String,
) -> Result<Option<Setting>, AppError> {
    Ok(db::settings::find_one(&state.pool, &key).await?)
}
```

Returns `null` rather than erroring when the key is absent — callers fall back to the default table in `src/lib/defaults.ts`.

### 2.3 `db_settings_upsert`

```rust
#[tauri::command]
pub async fn upsert(
    state: State<'_, AppState>,
    app: AppHandle,
    key: String,
    value: serde_json::Value,
) -> Result<(), AppError> {
    if key.is_empty() || key.len() > 128 {
        return Err(AppError::Invalid("setting key length".into()));
    }
    db::settings::upsert(&state.pool, &key, &value).await?;
    // Broadcast so every window's store stays coherent.
    app.emit("settings:changed", SettingChanged { key, value })?;
    Ok(())
}
```

The emitted `settings:changed` event is what keeps multi-window state in sync; no window ever reads another window's memory.

### 2.4 `db_settings_update_bounds`

Dedicated command because window geometry is written far more often than any other setting.

```rust
#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WindowBounds {
    pub x: i32, pub y: i32,
    pub width: u32, pub height: u32,
    pub maximized: bool,
    #[serde(default)] pub display: u32,
}

#[tauri::command]
pub async fn update_bounds(
    state: State<'_, AppState>,
    bounds: WindowBounds,
) -> Result<(), AppError> {
    // Reject nonsense from a mid-drag resize event.
    if bounds.width < 320 || bounds.height < 240 {
        return Err(AppError::Invalid("bounds below minimum".into()));
    }
    state.bounds_debounce.submit(bounds).await;  // coalesced, flushed every 500 ms
    Ok(())
}
```

The debouncer holds the latest bounds and writes at most twice per second, plus one forced flush on `WindowEvent::CloseRequested`.

---

## 3. Module: `db/profiles`

Backing tables: [`profiles`](01-database-schema.md#42-profiles), `profile_subscriptions`.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Profile {
    pub id: String,
    pub name: String,
    pub bg_color: String,
    pub text_color: String,
    pub sort_order: i64,
    pub subscriptions: Vec<ProfileSubscription>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileSubscription {
    pub id: String,          // channel id
    pub name: Option<String>,
    pub thumbnail: Option<String>,
}
```

| Command | Signature | Behaviour |
|---|---|---|
| `db_profiles_find` | `() -> Profile[]` | All non-deleted profiles ordered by `sort_order`, each with its subscriptions eagerly joined |
| `db_profiles_create` | `(profile: Profile) -> Profile` | Generates a UUIDv4 id if empty; rejects duplicate names |
| `db_profiles_upsert` | `(profile: Profile) -> Profile` | Full replace of the profile row **and** its subscription set inside one transaction |
| `db_profiles_delete` | `(id: string) -> void` | Soft delete (`deleted = 1`); refuses `allChannels` |
| `db_profiles_delete_all` | `() -> void` | Tombstones everything except `allChannels` |
| `db_profiles_persist_sync` | `(profiles: Profile[]) -> void` | Applies a decrypted sync collection; last-write-wins on `updated_at` |

```rust
#[tauri::command]
pub async fn upsert(state: State<'_, AppState>, profile: Profile) -> Result<Profile, AppError> {
    let mut tx = state.pool.begin().await?;

    sqlx::query(
        r#"INSERT INTO profiles (id,name,bg_color,text_color,sort_order,created_at,updated_at,deleted)
           VALUES (?1,?2,?3,?4,?5,?6,?7,0)
           ON CONFLICT(id) DO UPDATE SET
             name=excluded.name, bg_color=excluded.bg_color,
             text_color=excluded.text_color, sort_order=excluded.sort_order,
             updated_at=excluded.updated_at, deleted=0"#,
    )
    .bind(&profile.id).bind(&profile.name).bind(&profile.bg_color)
    .bind(&profile.text_color).bind(profile.sort_order)
    .bind(profile.created_at).bind(profile.updated_at)
    .execute(&mut *tx).await?;

    // Subscription set is authoritative: replace wholesale.
    sqlx::query("DELETE FROM profile_subscriptions WHERE profile_id = ?1")
        .bind(&profile.id).execute(&mut *tx).await?;

    for sub in &profile.subscriptions {
        sqlx::query(
            "INSERT INTO profile_subscriptions (profile_id,channel_id,name,thumbnail,added_at)
             VALUES (?1,?2,?3,?4,?5)",
        )
        .bind(&profile.id).bind(&sub.id).bind(&sub.name)
        .bind(&sub.thumbnail).bind(profile.updated_at)
        .execute(&mut *tx).await?;
    }

    tx.commit().await?;
    Ok(profile)
}
```

**Sync semantics.** `persist_sync` never deletes local profiles absent from the remote snapshot unless the snapshot carries an explicit tombstone. This prevents a stale device from wiping newer profiles.

---

## 4. Module: `db/playlists`

Backing tables: [`playlists`](01-database-schema.md#43-playlists--playlist_videos), `playlist_videos`.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Playlist {
    pub id: String,
    pub playlist_name: String,
    pub description: String,
    pub protected: bool,
    pub kind: String,
    pub videos: Vec<PlaylistVideo>,
    pub created_at: i64,
    pub last_updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaylistVideo {
    pub playlist_item_id: String,
    pub video_id: String,
    pub title: String,
    pub author: Option<String>,
    pub author_id: Option<String>,
    pub length_seconds: Option<i64>,
    pub published: Option<i64>,
    pub time_added: i64,
}
```

### 4.1 CRUD

| Command | Signature | Behaviour |
|---|---|---|
| `db_playlists_find` | `(opts?: { includeVideos?: boolean }) -> Playlist[]` | `includeVideos: false` returns headers only (uses the denormalised `video_count`) — the default for the sidebar |
| `db_playlists_create` | `(playlist: Playlist) -> Playlist` | Unique live-name check via `idx_playlists_name_live` |
| `db_playlists_upsert` | `(playlist: Playlist) -> Playlist` | Header fields only; video array ignored (use the video ops) |
| `db_playlists_delete` | `(id: string) -> void` | Refuses `protected = 1` playlists |
| `db_playlists_delete_all` | `() -> void` | Drops user playlists, empties protected ones |

### 4.2 Video operations

| Command | Signature | Behaviour |
|---|---|---|
| `db_playlists_upsert_video_by_playlist_name` | `(playlistName: string, video: PlaylistVideo) -> void` | Convenience path for "Add to Favorites"; creates the playlist if missing |
| `db_playlists_upsert_videos_by_playlist_id` | `(playlistId: string, videos: PlaylistVideo[]) -> void` | Batch append/update in one transaction; positions continue from current max |
| `db_playlists_delete_video_id_by_playlist_id` | `(playlistId: string, videoId: string, playlistItemId?: string) -> void` | `playlistItemId` disambiguates duplicates; without it removes the newest match |
| `db_playlists_delete_video_ids_by_playlist_id` | `(playlistId: string, videoIds: string[]) -> void` | Bulk removal, single transaction |
| `db_playlists_delete_all_videos_by_playlist_id` | `(playlistId: string) -> void` | Empties without deleting the playlist |
| `db_playlists_reorder_videos` | `(playlistId: string, orderedItemIds: string[]) -> void` | Rewrites `position` for the given ids; validates the set matches exactly |

```rust
#[tauri::command]
pub async fn upsert_videos_by_playlist_id(
    state: State<'_, AppState>,
    playlist_id: String,
    videos: Vec<PlaylistVideo>,
) -> Result<(), AppError> {
    let mut tx = state.pool.begin().await?;

    let mut next: i64 = sqlx::query_scalar(
        "SELECT COALESCE(MAX(position), -1) + 1 FROM playlist_videos WHERE playlist_id = ?1",
    )
    .bind(&playlist_id)
    .fetch_one(&mut *tx)
    .await?;

    for v in &videos {
        let item_id = if v.playlist_item_id.is_empty() {
            uuid::Uuid::new_v4().to_string()
        } else {
            v.playlist_item_id.clone()
        };

        sqlx::query(
            r#"INSERT INTO playlist_videos
                 (id,playlist_id,video_id,title,author,author_id,length_secs,published,position,time_added)
               VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)
               ON CONFLICT(id) DO UPDATE SET
                 title=excluded.title, author=excluded.author,
                 author_id=excluded.author_id, length_secs=excluded.length_secs"#,
        )
        .bind(&item_id).bind(&playlist_id).bind(&v.video_id).bind(&v.title)
        .bind(&v.author).bind(&v.author_id).bind(v.length_seconds)
        .bind(v.published).bind(next).bind(v.time_added)
        .execute(&mut *tx).await?;

        next += 1;
    }

    sqlx::query("UPDATE playlists SET last_updated_at = ?1 WHERE id = ?2")
        .bind(now_ms()).bind(&playlist_id)
        .execute(&mut *tx).await?;

    tx.commit().await?;
    Ok(())
}
```

### 4.3 `db_playlists_persist_sync`

`(playlists: Playlist[]) -> void`. Merge rules:

1. Match by playlist `id`; if absent locally, insert whole.
2. If present, compare `last_updated_at` — newer wins for header fields.
3. Video sets are merged by union on `playlist_item_id`, then re-positioned by `time_added`. This is deliberately non-destructive: a device that has been offline never loses additions made elsewhere.
4. Protected playlists (`favorites`) merge contents but never adopt a remote `name`.

---

## 5. Module: `db/history`

Backing table: [`history`](01-database-schema.md#44-history).

| Command | Signature | Behaviour |
|---|---|---|
| `db_history_find_all` | `(limit?: number) -> HistoryEntry[]` | Newest first; default limit 100 |
| `db_history_upsert` | `(entry: HistoryEntry) -> void` | `ON CONFLICT(video_id) DO UPDATE` |
| `db_history_delete` | `(videoId: string) -> void` | Removes one entry |
| `db_history_clear` | `() -> void` | Clears the table |

**HistoryEntry model** (`src-tauri/src/db/models.rs`):

```rust
#[derive(Debug, Clone, FromRow, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryEntry {
    pub video_id: String,
    pub title: String,
    pub author: String,
    pub author_id: String,
    pub length_seconds: Option<i64>,
    pub watch_progress: Option<f64>,
    pub time_watched: String,          // ISO-8601 timestamp (UTC)
    pub is_watched: bool,
    pub is_live: bool,
}
```

Key differences from the original design spec:
- `time_watched` is a `String` (ISO-8601), not epoch milliseconds — matches JS `Date.toISOString()` directly.
- `watch_progress` and `length_seconds` are `Option<T>`, not defaulted scalars.
- No `paused`, `type`, `last_viewed_playlist_id`, or `synced_at` columns yet (follow-up migration when sync lands).

The frontend interface in `src/stores/history.ts` mirrors this exactly:

```ts
export interface HistoryEntry {
  videoId: string
  title: string
  author: string
  authorId: string
  lengthSeconds: number | null
  timeWatched: string             // ISO timestamp
  watchProgress: number | null
  isWatched: boolean
  isLive: boolean
  lastViewedPlaylistId?: string   // UI-only, not yet persisted
  lastViewedPlaylistType?: string
  lastViewedPlaylistItemId?: string
}
```

> **Phase 1 note:** `update_watch_progress`, `overwrite`, `delete_older_than`, and `apply_sync` commands are not yet implemented. `HistoryRepository::delete_older_than(days)` and `clear_all()` exist in the repository layer but lack Tauri command wrappers. These will land when the sync feature is built.

---

## 6. Module: `db/watch_stats`

Backing table: [`watch_stats`](01-database-schema.md#45-watch_stats).

| Command | Signature | Behaviour |
|---|---|---|
| `db_watch_stats_find` | `(bucketType?: 'day' \| 'channel' \| 'total', from?: number, to?: number) -> WatchStat[]` | Filters by bucket type and `updated_at` range |
| `db_watch_stats_add_watch_time` | `(videoId: string, channelId?: string, seconds: number) -> void` | Increments day + channel + total buckets atomically |
| `db_watch_stats_delete_all` | `() -> void` | Resets all statistics |
| `db_watch_stats_persist_sync` | `(stats: WatchStat[]) -> void` | Additive merge — takes `MAX(seconds)` per bucket, never sums (avoids double counting when two devices sync the same session) |

```rust
#[tauri::command]
pub async fn add_watch_time(
    state: State<'_, AppState>,
    video_id: String,
    channel_id: Option<String>,
    seconds: i64,
) -> Result<(), AppError> {
    if !(0..=86_400).contains(&seconds) {
        return Err(AppError::Invalid("seconds out of range".into()));
    }

    let day = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let mut tx = state.pool.begin().await?;

    bump(&mut tx, "day", &day, seconds).await?;
    if let Some(cid) = &channel_id {
        bump(&mut tx, "channel", cid, seconds).await?;
    }
    bump(&mut tx, "total", "all", seconds).await?;

    tracing::debug!(%video_id, seconds, "watch time recorded");
    tx.commit().await?;
    Ok(())
}

async fn bump(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    bucket_type: &str,
    bucket_key: &str,
    seconds: i64,
) -> Result<(), AppError> {
    sqlx::query(
        r#"INSERT INTO watch_stats (id,bucket_type,bucket_key,seconds,video_count,updated_at)
           VALUES (?1,?2,?3,?4,1,?5)
           ON CONFLICT(bucket_type,bucket_key) DO UPDATE SET
             seconds     = seconds + excluded.seconds,
             video_count = video_count + 1,
             updated_at  = excluded.updated_at"#,
    )
    .bind(format!("{bucket_type}:{bucket_key}"))
    .bind(bucket_type).bind(bucket_key).bind(seconds).bind(now_ms())
    .execute(&mut **tx).await?;
    Ok(())
}
```

The frontend calls `add_watch_time` on a 30-second heartbeat and once on pause/unmount, never per second.

---

## 7. Module: `db/search_history`

| Command | Signature | Behaviour |
|---|---|---|
| `db_search_history_find` | `(prefix?: string, limit?: number) -> SearchEntry[]` | Ordered by `last_used_at DESC`, default limit 20 |
| `db_search_history_upsert` | `(query: string) -> void` | Normalises (trim + lowercase) and increments `hit_count` on conflict |
| `db_search_history_delete` | `(id: string) -> void` | Removes a single suggestion |
| `db_search_history_delete_all` | `() -> void` | Clears suggestions |

Entries are capped at 500 rows; the upsert trims the oldest overflow in the same transaction.

---

## 8. Module: `db/subscription_cache`

Backing table: [`subscription_cache`](01-database-schema.md#47-subscription_cache).

| Command | Signature | Behaviour |
|---|---|---|
| `db_subscription_cache_find` | `(channelIds?: string[]) -> CacheEntry[]` | Omitting `channelIds` returns all; JSON feed columns are parsed before return |
| `db_subscription_cache_find_one` | `(channelId: string) -> CacheEntry \| null` | Single lookup |
| `db_subscription_cache_upsert` | `(entry: CacheEntry) -> void` | Replaces feeds and stamps `fetched_at`/`expires_at` |
| `db_subscription_cache_delete` | `(channelId: string) -> void` | Evicts one channel |
| `db_subscription_cache_delete_all` | `() -> void` | Full cache purge |
| `db_subscription_cache_persist_sync` | `(entries: CacheEntry[]) -> void` | Only accepts remote entries whose `fetched_at` is newer than local |

```rust
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CacheEntry {
    pub channel_id: String,
    pub name: Option<String>,
    pub thumbnail: Option<String>,
    pub videos: Vec<serde_json::Value>,
    pub shorts: Vec<serde_json::Value>,
    pub live: Vec<serde_json::Value>,
    pub community: Vec<serde_json::Value>,
    pub fetched_at: i64,
    pub expires_at: Option<i64>,
}
```

Cache TTL defaults to 6 hours. Because the payload is a cache, `persist_sync` is best-effort: failures are logged and swallowed rather than failing the whole sync run.

---

## 9. Module: `db/tab_sessions`

| Command | Signature | Behaviour |
|---|---|---|
| `db_tab_sessions_save` | `(windowLabel: string, tabs: Tab[], activeTab: number) -> void` | Serialises the tab array; debounced by the caller at 1 s |
| `db_tab_sessions_load` | `(windowLabel: string) -> TabSession \| null` | Read on window setup; `null` means "open the default home tab" |
| `db_tab_sessions_clear` | `(windowLabel?: string) -> void` | Omitting the label clears every window's session |

```rust
#[tauri::command]
pub async fn save(
    state: State<'_, AppState>,
    window_label: String,
    tabs: Vec<Tab>,
    active_tab: i64,
) -> Result<(), AppError> {
    if tabs.len() > 100 {
        return Err(AppError::Invalid("too many tabs".into()));
    }
    let json = serde_json::to_string(&tabs).map_err(|e| AppError::Other(e.to_string()))?;
    sqlx::query(
        "INSERT INTO tab_sessions (id,tabs,active_tab,saved_at) VALUES (?1,?2,?3,?4)
         ON CONFLICT(id) DO UPDATE SET tabs=excluded.tabs,
                                       active_tab=excluded.active_tab,
                                       saved_at=excluded.saved_at",
    )
    .bind(&window_label).bind(json).bind(active_tab).bind(now_ms())
    .execute(&state.pool).await?;
    Ok(())
}
```

---

## 10. Module: `sync`

Full protocol in [05-sync-encryption.md](05-sync-encryption.md). This section covers the command surface only.

```rust
#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SyncStatus {
    pub state: SyncState,          // idle | preparing | uploading | downloading | applying | error
    pub progress: f32,             // 0.0..=1.0
    pub current_collection: Option<String>,
    pub last_success_at: Option<i64>,
    pub last_error: Option<String>,
    pub pending_collections: Vec<String>,
}
```

| Command | Signature | Behaviour |
|---|---|---|
| `sync_start` | `(opts: { direction: 'push' \| 'pull' \| 'both', collections?: string[], force?: boolean }) -> string` | Returns a `runId`. Rejects with `Invalid` if a run is already active unless `force` |
| `sync_status` | `() -> SyncStatus` | Cheap poll of the in-memory status; the UI mostly listens to `sync:progress` events instead |
| `sync_cancel` | `(runId?: string) -> void` | Trips the run's `CancellationToken`; in-flight collection is rolled back |
| `sync_get_snapshot` | `(collections?: string[]) -> Snapshot` | Builds the plaintext snapshot from local tables — used for export/diagnostics and as the input to encryption |

```rust
#[tauri::command]
pub async fn start(
    app: AppHandle,
    state: State<'_, AppState>,
    opts: SyncOptions,
) -> Result<String, AppError> {
    let mut guard = state.sync.lock().await;
    if guard.is_running() && !opts.force {
        return Err(AppError::Invalid("sync already running".into()));
    }

    let run_id = uuid::Uuid::new_v4().to_string();
    let token = tokio_util::sync::CancellationToken::new();
    guard.begin(run_id.clone(), token.clone());
    drop(guard);

    let pool = state.pool.clone();
    let sync = state.sync.clone();
    tauri::async_runtime::spawn(async move {
        let result = sync::run(&app, &pool, opts, token).await;
        let mut g = sync.lock().await;
        match result {
            Ok(_)  => g.finish_ok(),
            Err(e) => { g.finish_err(&e.to_string()); let _ = app.emit("sync:error", e.to_string()); }
        }
    });

    Ok(run_id)
}
```

**Events emitted**

| Event | Payload | When |
|---|---|---|
| `sync:progress` | `SyncStatus` | On every collection transition and every 5% of upload/download |
| `sync:collection-applied` | `{ collection, added, updated, removed }` | After each collection commits |
| `sync:complete` | `{ runId, durationMs }` | Successful finish |
| `sync:error` | `{ kind, message }` | Any failure |

---

## 11. Module: `network`

Details in [06-network-proxy.md](06-network-proxy.md).

### 11.1 `network_test_proxy`

```rust
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProxyTestRequest {
    pub protocol: String,             // 'http' | 'https' | 'socks5' | 'socks5h'
    pub host: String,
    pub port: u16,
    pub username: Option<String>,
    pub password: Option<String>,
    pub test_url: Option<String>,     // default https://www.youtube.com/generate_204
    pub timeout_ms: Option<u64>,      // default 10_000, capped 30_000
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProxyTestResult {
    pub ok: bool,
    pub status: Option<u16>,
    pub latency_ms: u128,
    pub resolved_ip: Option<String>,
    pub error: Option<String>,
}
```

Builds a throwaway `reqwest::Client` with the given proxy, issues a `GET`, and measures wall time. Never mutates saved settings — the UI persists only after a successful test.

### 11.2 `network_resolve_favicon`

```rust
#[tauri::command]
pub async fn resolve_favicon(
    state: State<'_, AppState>,
    site_url: String,
) -> Result<Option<String>, AppError> { /* ... */ }
```

Resolution order: `/favicon.ico` → `<link rel="icon">` from the parsed HTML head → Google S2 fallback (only when the "allow third-party favicon service" setting is on). Results are cached on disk under `<app_cache>/favicons/<sha256(host)>.png` for 30 days and returned as an `asset://` URL. Fetches honour the global proxy and a 5 s timeout, and responses over 512 KiB are rejected.

---

## 12. Module: `window`

| Command | Signature | Behaviour |
|---|---|---|
| `window_create` | `(opts: WindowOptions) -> string` | Builds a `WebviewWindow`, returns the label |
| `window_close` | `(label?: string) -> void` | Defaults to the calling window; honours "close to tray" |
| `window_minimize` | `(label?: string) -> void` | Minimises, or hides to tray when configured |
| `window_tray` | `(action: TrayAction) -> void` | Show / hide / toggle / flash the tray-backed main window |

```rust
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowOptions {
    pub label: Option<String>,
    pub url: Option<String>,          // app route; external URLs are rejected
    pub title: Option<String>,
    pub width: Option<f64>,
    pub height: Option<f64>,
    pub resizable: Option<bool>,
    pub always_on_top: Option<bool>,
    pub decorations: Option<bool>,
}

#[tauri::command]
pub async fn create(app: AppHandle, opts: WindowOptions) -> Result<String, AppError> {
    let label = opts.label.unwrap_or_else(|| format!("w-{}", uuid::Uuid::new_v4().simple()));

    // Only in-app routes may be opened programmatically.
    let route = opts.url.unwrap_or_else(|| "index.html".into());
    if route.contains("://") {
        return Err(AppError::Invalid("external URLs are not permitted".into()));
    }

    tauri::WebviewWindowBuilder::new(&app, &label, tauri::WebviewUrl::App(route.into()))
        .title(opts.title.unwrap_or_else(|| "SlyTube".into()))
        .inner_size(opts.width.unwrap_or(1200.0), opts.height.unwrap_or(800.0))
        .min_inner_size(900.0, 600.0)
        .resizable(opts.resizable.unwrap_or(true))
        .always_on_top(opts.always_on_top.unwrap_or(false))
        .decorations(opts.decorations.unwrap_or(true))
        .build()
        .map_err(|e| AppError::Other(e.to_string()))?;

    Ok(label)
}
```

`TrayAction` is `'show' | 'hide' | 'toggle' | 'flash'`. Tray behaviour is driven by the `minimizeToTray` / `closeToTray` settings read through `db/settings`, so the command layer never hard-codes policy.

---

## 13. Module: `shortcuts`

Backed by `tauri-plugin-global-shortcut`.

```rust
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShortcutBinding {
    pub id: String,          // 'playPause', 'nextTrack', 'toggleWindow', ...
    pub accelerator: String, // 'CmdOrCtrl+Shift+P'
}

#[tauri::command]
pub async fn register(
    app: AppHandle,
    state: State<'_, AppState>,
    binding: ShortcutBinding,
) -> Result<(), AppError> {
    use tauri_plugin_global_shortcut::GlobalShortcutExt;

    let shortcut: tauri_plugin_global_shortcut::Shortcut = binding
        .accelerator
        .parse()
        .map_err(|_| AppError::Invalid(format!("bad accelerator: {}", binding.accelerator)))?;

    // Replace any previous binding for this action.
    let mut map = state.shortcuts.lock().await;
    if let Some(old) = map.remove(&binding.id) {
        let _ = app.global_shortcut().unregister(old);
    }

    let id = binding.id.clone();
    let handle = app.clone();
    app.global_shortcut()
        .on_shortcut(shortcut, move |_, _, event| {
            if event.state == tauri_plugin_global_shortcut::ShortcutState::Pressed {
                let _ = handle.emit("shortcut:triggered", &id);
            }
        })
        .map_err(|e| AppError::Other(e.to_string()))?;

    map.insert(binding.id, shortcut);
    Ok(())
}

#[tauri::command]
pub async fn unregister(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
) -> Result<(), AppError> {
    use tauri_plugin_global_shortcut::GlobalShortcutExt;
    if let Some(sc) = state.shortcuts.lock().await.remove(&id) {
        app.global_shortcut().unregister(sc).map_err(|e| AppError::Other(e.to_string()))?;
    }
    Ok(())
}
```

Registration failures (accelerator already claimed by the OS or another app) surface as `Invalid` so the settings UI can mark the row red without aborting the whole batch. All bindings are re-registered on startup from the `shortcuts` setting key.

---

## 14. Capability Permissions

Commands are only reachable from windows granted the capability:

```json
// src-tauri/capabilities/default.json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "default",
  "description": "Main application window capabilities",
  "windows": ["main", "w-*"],
  "permissions": [
    "core:default",
    "core:window:allow-start-dragging",
    "opener:default",
    "shell:allow-execute",
    "global-shortcut:default",
    "notification:default"
  ]
}
```

The hidden PoToken window (see [04-potoken-generation.md](04-potoken-generation.md)) uses a **separate, minimal capability** that grants no `db_*` commands at all.

---

## 15. Command Index

| Module | Commands |
|---|---|
| `db/settings` | `find_all`, `find_one`, `upsert` |
| `db/profiles` | `find_all`, `find_one`, `create`, `update`, `delete`, `get_subscriptions`, `add_subscription`, `remove_subscription` |
| `db/playlists` | `find_all`, `find_one`, `create`, `update`, `delete`, `get_videos`, `add_video`, `remove_video` |
| `db/history` | `find_all`, `find_one`, `upsert`, `delete`, `clear` |
| `db/watch_stats` | `add`, `get_total` |
| `db/search_history` | `find_all`, `add`, `clear` |
| `db/subscription_cache` | `find_one`, `upsert`, `get_all` |
| `db/tab_sessions` | `save`, `get_latest`, `clear` |
| `sync` | `test_connection`, `get_state`, `save_state`, `start`, `cancel` |
| `yt/invidious` | `get_video`, `search`, `get_trending`, `get_channel`, `get_playlist`, `get_comments`, `get_instances`, `test_instance`, `get_dash_manifest`, `get_dash_url`, `get_format_streams`, `get_popular`, `get_channel_videos`, `resolve_url`, `get_channel_tabs`, `get_channel_shorts`, `get_channel_live`, `get_channel_playlists`, `get_channel_releases`, `get_channel_podcasts`, `get_channel_courses`, `search_channel`, `get_comment_replies`, `get_search_suggestions`, `search_with_filters`, `get_community_posts`, `get_community_post`, `get_community_post_comments`, `get_community_post_comment_replies`, `get_hashtag` |
| `yt/youtube` | `get_video_info`, `search_videos`, `get_trending`, `get_channel_info`, `get_channel_videos`, `get_comments`, `get_search_suggestions`, `get_playlist_info`, `get_community_posts`, `get_hashtag` |
| `downloads` | `get_info`, `get_playback_info`, `download`, `cancel`, `list` |
| `potoken` | `generate_po_token` |
| `system` | `show_main_window`, `hide_main_window`, `toggle_window`, `get_version`, `check_for_updates`, `open_external`, `center_window`, `set_fullscreen`, `get_window_size` |

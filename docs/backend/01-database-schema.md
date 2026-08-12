# 01 - Database Schema (NeDB → SQLite / sqlx)

> **Domain:** `backend`
> **Status:** Design specification (implementation target for `src-tauri/src/db`)
> **Related:** [02-tauri-commands.md](02-tauri-commands.md), [05-sync-encryption.md](05-sync-encryption.md)

---

## 1. Overview

The legacy Electron application persisted all state in **NeDB** — a set of append-only, line-delimited JSON (`.db`) files stored in the Electron `userData` directory. The Tauri rewrite replaces this with a single **SQLite** database accessed through **sqlx** with compile-time verified queries.

| Concern | NeDB (legacy) | SQLite + sqlx (target) |
|---|---|---|
| Storage | 8 separate append-only JSON files | 1 `slytube.db` file (+ WAL/SHM) |
| Query engine | In-memory JS scan | SQL with B-tree indexes |
| Concurrency | Single JS thread, file locks | WAL mode, connection pool |
| Compaction | Manual `persistence.compactDatafile()` | `VACUUM` / autovacuum |
| Type safety | None (arbitrary JSON) | `sqlx::query!` compile-time checks |
| Corruption risk | Truncated last line on crash | ACID transactions |
| Cross-process | Unsafe | Safe via WAL |

### 1.1 Legacy NeDB file inventory

```
<userData>/
├── settings.db             → settings
├── profiles.db             → profiles
├── playlists.db            → playlists (+ embedded videos array)
├── history.db              → history
├── search-history.db       → search_history
├── subscription-cache.db   → subscription_cache
├── watch-stats.db          → watch_stats  (may not exist in older versions)
└── tab-sessions.db         → tab_sessions (may not exist in older versions)
```

Each line of a NeDB file is one JSON document. The *last* occurrence of a given `_id` wins; a line containing `{"$$deleted":true,"_id":"..."}` marks a tombstone. Any migration reader **must** replay the file top-to-bottom to reconstruct final state.

---

## 2. Migration Strategy

### 2.1 Phases

```
┌────────────────────────────────────────────────────────────────────┐
│ PHASE 0  Detect       Does <appdata>/slytube.db exist?             │
│                       Does <legacy userData>/settings.db exist?    │
├────────────────────────────────────────────────────────────────────┤
│ PHASE 1  Backup       Copy every *.db into  backups/nedb-<ts>/     │
├────────────────────────────────────────────────────────────────────┤
│ PHASE 2  Schema       sqlx::migrate!() applies 001..00N            │
├────────────────────────────────────────────────────────────────────┤
│ PHASE 3  Import       Stream each NeDB file → typed struct → batch │
│                       INSERT inside ONE transaction per collection │
├────────────────────────────────────────────────────────────────────┤
│ PHASE 4  Verify       Row counts vs. reconstructed doc counts      │
├────────────────────────────────────────────────────────────────────┤
│ PHASE 5  Seal         Write migration_state row; rename legacy dir │
│                       to  <userData>.migrated  (never delete)      │
└────────────────────────────────────────────────────────────────────┘
```

Migration is **idempotent** and **all-or-nothing per collection**. If Phase 3 fails for `playlists`, the transaction rolls back, the `migration_state` row for `playlists` stays `pending`, and the app retries on next launch. A collection that fails three times is marked `failed` and surfaced in the UI with a "Import legacy data" retry action.

### 2.2 Migration state table

```sql
CREATE TABLE migration_state (
    collection      TEXT PRIMARY KEY,   -- 'settings', 'profiles', ...
    status          TEXT NOT NULL,      -- 'pending' | 'done' | 'failed' | 'skipped'
    source_docs     INTEGER,            -- documents reconstructed from NeDB
    imported_rows   INTEGER,            -- rows actually written
    attempts        INTEGER NOT NULL DEFAULT 0,
    error           TEXT,
    completed_at    INTEGER
);
```

### 2.3 NeDB reader

```rust
// src-tauri/src/db/migrate_nedb.rs
use std::collections::HashMap;
use std::path::Path;
use tokio::io::{AsyncBufReadExt, BufReader};
use serde_json::Value;

/// Replays an append-only NeDB datafile and returns the final document set.
/// Tolerates a truncated trailing line (common after a hard crash).
pub async fn read_nedb(path: &Path) -> anyhow::Result<Vec<Value>> {
    if !path.exists() {
        return Ok(Vec::new());
    }

    let file = tokio::fs::File::open(path).await?;
    let mut lines = BufReader::new(file).lines();

    // Insertion order preserved so exports stay stable.
    let mut docs: HashMap<String, Value> = HashMap::new();
    let mut order: Vec<String> = Vec::new();
    let mut skipped = 0usize;

    while let Some(line) = lines.next_line().await? {
        if line.trim().is_empty() {
            continue;
        }
        let doc: Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(_) => { skipped += 1; continue; } // truncated / corrupt line
        };

        let Some(id) = doc.get("_id").and_then(Value::as_str) else {
            skipped += 1;
            continue;
        };
        let id = id.to_string();

        if doc.get("$$deleted").and_then(Value::as_bool).unwrap_or(false) {
            docs.remove(&id);
            order.retain(|k| k != &id);
            continue;
        }
        // $$indexCreated / $$indexRemoved are metadata lines, not documents.
        if doc.get("$$indexCreated").is_some() || doc.get("$$indexRemoved").is_some() {
            continue;
        }

        if !docs.contains_key(&id) {
            order.push(id.clone());
        }
        docs.insert(id, doc);
    }

    if skipped > 0 {
        tracing::warn!(?path, skipped, "skipped malformed NeDB lines");
    }

    Ok(order.into_iter().filter_map(|k| docs.remove(&k)).collect())
}
```

### 2.4 Orchestrator

```rust
// src-tauri/src/db/migrate_nedb.rs (cont.)
const COLLECTIONS: &[(&str, &str)] = &[
    ("settings",           "settings.db"),
    ("profiles",           "profiles.db"),
    ("playlists",          "playlists.db"),
    ("history",            "history.db"),
    ("search_history",     "search-history.db"),
    ("subscription_cache", "subscription-cache.db"),
    ("watch_stats",        "watch-stats.db"),
    ("tab_sessions",       "tab-sessions.db"),
];

pub async fn migrate_all(pool: &SqlitePool, legacy_dir: &Path) -> anyhow::Result<MigrationReport> {
    let mut report = MigrationReport::default();

    backup_legacy_dir(legacy_dir).await?;   // PHASE 1

    for (collection, filename) in COLLECTIONS {
        if is_done(pool, collection).await? {
            report.skipped.push((*collection).into());
            continue;
        }

        let docs = read_nedb(&legacy_dir.join(filename)).await?;
        if docs.is_empty() {
            mark(pool, collection, "skipped", 0, 0, None).await?;
            continue;
        }

        let mut tx = pool.begin().await?;                       // PHASE 3
        let result = match *collection {
            "settings"           => import_settings(&mut tx, &docs).await,
            "profiles"           => import_profiles(&mut tx, &docs).await,
            "playlists"          => import_playlists(&mut tx, &docs).await,
            "history"            => import_history(&mut tx, &docs).await,
            "search_history"     => import_search_history(&mut tx, &docs).await,
            "subscription_cache" => import_subscription_cache(&mut tx, &docs).await,
            "watch_stats"        => import_watch_stats(&mut tx, &docs).await,
            "tab_sessions"       => import_tab_sessions(&mut tx, &docs).await,
            _ => Ok(0),
        };

        match result {
            Ok(rows) => {
                tx.commit().await?;
                mark(pool, collection, "done", docs.len(), rows, None).await?;
                report.imported.push(((*collection).into(), rows));
            }
            Err(e) => {
                tx.rollback().await?;
                mark(pool, collection, "failed", docs.len(), 0, Some(&e.to_string())).await?;
                report.failed.push(((*collection).into(), e.to_string()));
            }
        }
    }

    Ok(report)
}
```

### 2.5 Field mapping notes

| Legacy quirk | Handling |
|---|---|
| `_id` is a 16-char NeDB random string | Preserved verbatim as the SQLite `id` so sync IDs stay stable |
| Dates stored as JS epoch **milliseconds** | Kept as milliseconds (`INTEGER`) — never silently divided by 1000 |
| `playlists.videos` is a nested array | Flattened into `playlist_videos` with `position` derived from array index |
| Booleans stored as `true`/`false` JSON | Mapped to `INTEGER 0/1` with `NOT NULL DEFAULT` |
| Settings values are heterogeneous | Stored as JSON text + a `value_type` discriminator |
| Duplicate `videoId` inside one playlist | Legacy allowed it; kept via surrogate `playlist_videos.id` PK |

---

## 3. sqlx Setup

### 3.1 Cargo dependencies

```toml
# src-tauri/Cargo.toml
[dependencies]
sqlx = { version = "0.8", default-features = false, features = [
    "runtime-tokio",
    "tls-rustls",
    "sqlite",
    "macros",
    "migrate",
    "json",
    "chrono",
] }
tokio      = { version = "1", features = ["fs", "io-util", "macros", "rt-multi-thread"] }
serde      = { version = "1", features = ["derive"] }
serde_json = "1"
anyhow     = "1"
thiserror  = "2"
tracing    = "0.1"
chrono     = { version = "0.4", features = ["serde"] }
uuid       = { version = "1", features = ["v4", "serde"] }
```

### 3.2 Offline compile-time verification

```bash
# Generate .sqlx/ metadata so CI builds without a live database
cargo sqlx prepare --workspace -- --all-targets
```

```bash
# .env  (development only — committed .sqlx/ is what CI uses)
DATABASE_URL=sqlite://./dev-data/slytube.db?mode=rwc
```

Add `.sqlx/` to version control; add `dev-data/` to `.gitignore`.

### 3.3 Pool initialisation

```rust
// src-tauri/src/db/mod.rs
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions, SqliteJournalMode, SqliteSynchronous};
use sqlx::SqlitePool;
use std::{path::Path, str::FromStr, time::Duration};

pub async fn init_db(app_data_dir: &Path) -> Result<SqlitePool, sqlx::Error> {
    tokio::fs::create_dir_all(app_data_dir).await.ok();
    let db_path = app_data_dir.join("slytube.db");

    let opts = SqliteConnectOptions::from_str(&format!("sqlite://{}", db_path.display()))?
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .synchronous(SqliteSynchronous::Normal)
        .foreign_keys(true)
        .busy_timeout(Duration::from_secs(10))
        .pragma("cache_size", "-32768")     // 32 MiB page cache
        .pragma("temp_store", "MEMORY")
        .pragma("mmap_size", "268435456")   // 256 MiB
        .pragma("wal_autocheckpoint", "1000");

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .min_connections(1)
        .acquire_timeout(Duration::from_secs(15))
        .idle_timeout(Duration::from_secs(300))
        .connect_with(opts)
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(pool)
}
```

> **WAL + `max_connections`**: SQLite permits many concurrent readers but only one writer. Five connections is a pragmatic ceiling; write-heavy commands (history upsert, sync apply) must use short transactions to avoid `SQLITE_BUSY` despite the 10 s busy timeout.

### 3.4 Managed state

```rust
// src-tauri/src/state.rs
pub struct AppState {
    pub pool: SqlitePool,
}

// lib.rs
tauri::Builder::default()
    .setup(|app| {
        let dir = app.path().app_data_dir()?;
        let pool = tauri::async_runtime::block_on(db::init_db(&dir))?;
        tauri::async_runtime::block_on(db::migrate_nedb::migrate_all(&pool, &legacy_dir(app)?)).ok();
        app.manage(AppState { pool });
        Ok(())
    })
```

---

## 4. Table Schemas

All timestamps are **Unix epoch milliseconds** stored as `INTEGER`, matching the legacy JS `Date.now()` semantics. Booleans are `INTEGER` `0`/`1`.

### 4.1 `settings`

Key/value store with a type discriminator. One row per setting key.

```sql
CREATE TABLE settings (
    key         TEXT    PRIMARY KEY,
    value       TEXT    NOT NULL,          -- JSON-encoded scalar or structure
    value_type  TEXT    NOT NULL           -- 'string'|'number'|'boolean'|'object'|'array'|'null'
        CHECK (value_type IN ('string','number','boolean','object','array','null')),
    updated_at  INTEGER NOT NULL DEFAULT (CAST(strftime('%s','now') AS INTEGER) * 1000)
) STRICT;
```

Window bounds are persisted under the reserved key `bounds`:

```json
{ "x": 120, "y": 80, "width": 1280, "height": 800, "maximized": false, "display": 0 }
```

`update_bounds` writes this single key with a debounce (see [02-tauri-commands.md](02-tauri-commands.md#21-dbsettings)) rather than one row per dimension.

### 4.2 `profiles`

```sql
CREATE TABLE profiles (
    id             TEXT    PRIMARY KEY,          -- legacy _id or 'allChannels'
    name           TEXT    NOT NULL,
    bg_color       TEXT    NOT NULL DEFAULT '#000000',
    text_color     TEXT    NOT NULL DEFAULT '#FFFFFF',
    sort_order     INTEGER NOT NULL DEFAULT 0,
    created_at     INTEGER NOT NULL,
    updated_at     INTEGER NOT NULL,
    synced_at      INTEGER,
    deleted        INTEGER NOT NULL DEFAULT 0 CHECK (deleted IN (0,1))
) STRICT;

CREATE TABLE profile_subscriptions (
    profile_id  TEXT NOT NULL REFERENCES profiles(id) ON DELETE CASCADE,
    channel_id  TEXT NOT NULL,
    name        TEXT,
    thumbnail   TEXT,
    added_at    INTEGER NOT NULL,
    PRIMARY KEY (profile_id, channel_id)
) STRICT, WITHOUT ROWID;

CREATE INDEX idx_profile_subs_channel ON profile_subscriptions(channel_id);
CREATE INDEX idx_profiles_sort        ON profiles(sort_order) WHERE deleted = 0;
```

The `allChannels` profile is seeded by migration `001` and can never be deleted (enforced in the command layer, not the schema, so sync payloads remain portable).

### 4.3 `playlists` + `playlist_videos`

```sql
CREATE TABLE playlists (
    id            TEXT    PRIMARY KEY,
    name          TEXT    NOT NULL,
    description   TEXT    NOT NULL DEFAULT '',
    protected     INTEGER NOT NULL DEFAULT 0 CHECK (protected IN (0,1)),
    kind          TEXT    NOT NULL DEFAULT 'user'   -- 'user'|'favorites'|'watch_later'
        CHECK (kind IN ('user','favorites','watch_later')),
    video_count   INTEGER NOT NULL DEFAULT 0,       -- denormalised, trigger-maintained
    created_at    INTEGER NOT NULL,
    updated_at    INTEGER NOT NULL,
    last_played_at INTEGER,
    synced_at     INTEGER,
    deleted       INTEGER NOT NULL DEFAULT 0 CHECK (deleted IN (0,1))
) STRICT;

CREATE UNIQUE INDEX idx_playlists_name_live ON playlists(name) WHERE deleted = 0;

CREATE TABLE playlist_videos (
    id           TEXT    PRIMARY KEY,             -- playlistItemId (stable across reorders)
    playlist_id  TEXT    NOT NULL REFERENCES playlists(id) ON DELETE CASCADE,
    video_id     TEXT    NOT NULL,
    title        TEXT    NOT NULL DEFAULT '',
    author       TEXT,
    author_id    TEXT,
    length_secs  INTEGER,
    published    INTEGER,
    position     INTEGER NOT NULL,
    time_added   INTEGER NOT NULL
) STRICT;

CREATE INDEX idx_pv_playlist_pos ON playlist_videos(playlist_id, position);
CREATE INDEX idx_pv_video        ON playlist_videos(video_id);
```

Counter-maintaining triggers:

```sql
CREATE TRIGGER trg_pv_insert AFTER INSERT ON playlist_videos BEGIN
    UPDATE playlists SET video_count = video_count + 1 WHERE id = NEW.playlist_id;
END;

CREATE TRIGGER trg_pv_delete AFTER DELETE ON playlist_videos BEGIN
    UPDATE playlists SET video_count = video_count - 1 WHERE id = OLD.playlist_id;
END;
```

### 4.4 `history`

```sql
CREATE TABLE history (
    video_id       TEXT    PRIMARY KEY,
    title          TEXT    NOT NULL DEFAULT '',
    author         TEXT,
    author_id      TEXT,
    length_secs    INTEGER,
    published      INTEGER,
    watch_progress REAL    NOT NULL DEFAULT 0,     -- seconds
    time_watched   INTEGER NOT NULL,               -- epoch ms of last view
    is_live        INTEGER NOT NULL DEFAULT 0 CHECK (is_live IN (0,1)),
    paused         INTEGER NOT NULL DEFAULT 0 CHECK (paused IN (0,1)),
    last_viewed_playlist_id TEXT,
    type           TEXT    NOT NULL DEFAULT 'video',
    synced_at      INTEGER
) STRICT;

CREATE INDEX idx_history_time    ON history(time_watched DESC);
CREATE INDEX idx_history_author  ON history(author_id);
CREATE INDEX idx_history_title   ON history(title COLLATE NOCASE);
```

`video_id` as the primary key gives `upsert` (`ON CONFLICT(video_id) DO UPDATE`) for free and guarantees the one-row-per-video invariant the legacy NeDB store only maintained by convention.

### 4.5 `watch_stats`

```sql
CREATE TABLE watch_stats (
    id           TEXT    PRIMARY KEY,             -- 'YYYY-MM-DD' bucket or channel key
    bucket_type  TEXT    NOT NULL                 -- 'day' | 'channel' | 'total'
        CHECK (bucket_type IN ('day','channel','total')),
    bucket_key   TEXT    NOT NULL,                -- '2026-08-09' | channelId | 'all'
    seconds      INTEGER NOT NULL DEFAULT 0,
    video_count  INTEGER NOT NULL DEFAULT 0,
    updated_at   INTEGER NOT NULL,
    synced_at    INTEGER
) STRICT;

CREATE UNIQUE INDEX idx_ws_bucket ON watch_stats(bucket_type, bucket_key);
CREATE INDEX        idx_ws_updated ON watch_stats(updated_at DESC);
```

`add_watch_time(video_id, channel_id, seconds)` performs three upserts in one transaction: the `day` bucket, the `channel` bucket, and the `total` bucket.

### 4.6 `search_history`

```sql
CREATE TABLE search_history (
    id           TEXT    PRIMARY KEY,             -- lowercase(query) hash or legacy _id
    query        TEXT    NOT NULL,
    normalized   TEXT    NOT NULL,                -- trim + lowercase, used for dedupe
    hit_count    INTEGER NOT NULL DEFAULT 1,
    last_used_at INTEGER NOT NULL,
    created_at   INTEGER NOT NULL
) STRICT;

CREATE UNIQUE INDEX idx_sh_normalized ON search_history(normalized);
CREATE INDEX        idx_sh_recent     ON search_history(last_used_at DESC);
```

Search history is deliberately **excluded from sync** in `enhanced` privacy mode (see [05-sync-encryption.md](05-sync-encryption.md)).

### 4.7 `subscription_cache`

```sql
CREATE TABLE subscription_cache (
    channel_id   TEXT    PRIMARY KEY,
    name         TEXT,
    thumbnail    TEXT,
    videos       TEXT    NOT NULL DEFAULT '[]',   -- JSON array of video summaries
    shorts       TEXT    NOT NULL DEFAULT '[]',
    live         TEXT    NOT NULL DEFAULT '[]',
    community    TEXT    NOT NULL DEFAULT '[]',
    fetched_at   INTEGER NOT NULL,
    expires_at   INTEGER,
    error_count  INTEGER NOT NULL DEFAULT 0,
    synced_at    INTEGER
) STRICT;

CREATE INDEX idx_sc_fetched ON subscription_cache(fetched_at DESC);
CREATE INDEX idx_sc_expires ON subscription_cache(expires_at);
```

The four feed arrays stay as JSON blobs: they are always read and written whole, are bounded (~60 entries per channel), and normalising them would create a hot write path for no query benefit.

### 4.8 `download_records`

Tracks yt-dlp download history and active downloads. Records are inserted at spawn time and updated on progress, destination discovery, completion, or failure.

```sql
CREATE TABLE IF NOT EXISTS download_records (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    video_id     TEXT    NOT NULL,
    title        TEXT    NOT NULL,
    status       TEXT    NOT NULL DEFAULT 'pending',
    percent      REAL    NOT NULL DEFAULT 0.0,
    destination  TEXT    NOT NULL DEFAULT '',
    created_at   TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_download_records_status      ON download_records(status);
CREATE INDEX IF NOT EXISTS idx_download_records_created_at  ON download_records(created_at DESC);
```

> **Note:** This is a simplified schema compared to the design spec in [`03-yt-dlp-sidecar.md`](03-yt-dlp-sidecar.md). The initial implementation persists only the fields needed for listing and basic progress tracking. Columns for `error_message`, `output_path`, `started_at`, `completed_at`, and `args_json` can be added in a follow-up migration when the richer tracking is required.

### 4.9 `tab_sessions`

```sql
CREATE TABLE tab_sessions (
    id          TEXT    PRIMARY KEY,              -- window label, e.g. 'main'
    tabs        TEXT    NOT NULL DEFAULT '[]',    -- JSON array of tab descriptors
    active_tab  INTEGER NOT NULL DEFAULT 0,
    saved_at    INTEGER NOT NULL
) STRICT;
```

Tab descriptor shape:

```json
{
  "id": "t-8f2c",
  "route": "/watch/dQw4w9WgXcQ",
  "title": "Video title",
  "scroll": 420,
  "pinned": false
}
```

Tab sessions are **window-local and never synced**; they are cleared on clean shutdown unless "restore tabs on start" is enabled.

---

## 5. Indexes Summary

| Table | Index | Purpose |
|---|---|---|
| `profiles` | `idx_profiles_sort` (partial) | Sidebar ordering, excludes tombstones |
| `profile_subscriptions` | `idx_profile_subs_channel` | "Which profiles contain this channel?" |
| `playlists` | `idx_playlists_name_live` (unique, partial) | Enforce unique live names, allow re-use after delete |
| `playlist_videos` | `idx_pv_playlist_pos` | Ordered playlist render, reorder writes |
| `playlist_videos` | `idx_pv_video` | "Which playlists contain this video?" badge |
| `history` | `idx_history_time` | Default history feed (newest first) |
| `history` | `idx_history_title` (NOCASE) | In-app history search |
| `watch_stats` | `idx_ws_bucket` (unique) | Upsert target for `add_watch_time` |
| `search_history` | `idx_sh_normalized` (unique) | Dedupe on insert |
| `search_history` | `idx_sh_recent` | Suggestion dropdown |
| `subscription_cache` | `idx_sc_expires` | Background refresh sweep |

**Sizing note:** indexes are intentionally sparse. `history` is the largest table in practice (10k–100k rows for heavy users); three indexes on it is the measured sweet spot before insert cost dominates.

---

## 6. Foreign Keys & Referential Integrity

```
profiles (id) ──1:N──> profile_subscriptions (profile_id)   ON DELETE CASCADE
playlists (id) ─1:N──> playlist_videos     (playlist_id)    ON DELETE CASCADE
```

Deliberately **not** foreign-keyed:

- `history.video_id` → no `videos` table exists; history is self-contained metadata.
- `playlist_videos.video_id` → playlists must survive a video becoming unavailable.
- `profile_subscriptions.channel_id` → `subscription_cache` is a *cache*; purging it must not delete subscriptions.

`PRAGMA foreign_keys = ON` is set per connection via `SqliteConnectOptions::foreign_keys(true)` (it is **not** persistent and must be re-applied on every connection — a common SQLite footgun).

Soft deletes (`deleted = 0/1`) exist on `profiles` and `playlists` because the sync protocol needs tombstones to propagate deletions across devices. A background job hard-deletes tombstones older than 90 days.

---

## 7. Migration Files

```
src-tauri/migrations/
├── 001_initial_schema.sql
├── 002_seed_defaults.sql
├── 003_sync_columns.sql
├── 004_watch_stats_buckets.sql
└── 005_search_history_dedupe.sql
```

Rules:

1. Migrations are **append-only**. Never edit a shipped file — `sqlx` stores a checksum in `_sqlx_migrations` and refuses to run on mismatch.
2. Every migration is a single statement stream; sqlx wraps it in a transaction for SQLite.
3. Destructive changes use the 12-step SQLite table rebuild (`CREATE new → INSERT SELECT → DROP old → ALTER RENAME`).
4. Column additions use `ALTER TABLE ... ADD COLUMN ... NOT NULL DEFAULT ...` (SQLite requires a non-null default).

### 7.1 `002_seed_defaults.sql`

```sql
INSERT OR IGNORE INTO profiles (id, name, bg_color, text_color, sort_order, created_at, updated_at)
VALUES ('allChannels', 'All Channels', '#000000', '#FFFFFF', 0,
        CAST(strftime('%s','now') AS INTEGER) * 1000,
        CAST(strftime('%s','now') AS INTEGER) * 1000);

INSERT OR IGNORE INTO playlists (id, name, description, protected, kind, created_at, updated_at)
VALUES ('favorites', 'Favorites', 'Your favourite videos', 1, 'favorites',
        CAST(strftime('%s','now') AS INTEGER) * 1000,
        CAST(strftime('%s','now') AS INTEGER) * 1000);
```

---

## 8. Repository Layer

One module per table under `src-tauri/src/db/`, each exposing plain `async fn` taking `&SqlitePool`. Tauri commands are thin wrappers (see [02-tauri-commands.md](02-tauri-commands.md)).

```rust
// src-tauri/src/db/history.rs
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct HistoryEntry {
    pub video_id: String,
    pub title: String,
    pub author: Option<String>,
    pub author_id: Option<String>,
    pub length_secs: Option<i64>,
    pub published: Option<i64>,
    pub watch_progress: f64,
    pub time_watched: i64,
    pub is_live: bool,
    pub paused: bool,
    pub last_viewed_playlist_id: Option<String>,
    pub r#type: String,
}

pub async fn find(pool: &SqlitePool, limit: i64, offset: i64) -> sqlx::Result<Vec<HistoryEntry>> {
    sqlx::query_as::<_, HistoryEntry>(
        r#"SELECT video_id, title, author, author_id, length_secs, published,
                  watch_progress, time_watched, is_live, paused,
                  last_viewed_playlist_id, type
           FROM history
           ORDER BY time_watched DESC
           LIMIT ?1 OFFSET ?2"#,
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
}

pub async fn upsert(pool: &SqlitePool, e: &HistoryEntry) -> sqlx::Result<()> {
    sqlx::query(
        r#"INSERT INTO history (
               video_id, title, author, author_id, length_secs, published,
               watch_progress, time_watched, is_live, paused,
               last_viewed_playlist_id, type
           ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12)
           ON CONFLICT(video_id) DO UPDATE SET
               title                   = excluded.title,
               author                  = excluded.author,
               author_id               = excluded.author_id,
               length_secs             = excluded.length_secs,
               published               = excluded.published,
               watch_progress          = excluded.watch_progress,
               time_watched            = excluded.time_watched,
               is_live                 = excluded.is_live,
               paused                  = excluded.paused,
               last_viewed_playlist_id = excluded.last_viewed_playlist_id,
               type                    = excluded.type"#,
    )
    .bind(&e.video_id).bind(&e.title).bind(&e.author).bind(&e.author_id)
    .bind(e.length_secs).bind(e.published).bind(e.watch_progress)
    .bind(e.time_watched).bind(e.is_live).bind(e.paused)
    .bind(&e.last_viewed_playlist_id).bind(&e.r#type)
    .execute(pool)
    .await
    .map(|_| ())
}

pub async fn delete_older_than(pool: &SqlitePool, cutoff_ms: i64) -> sqlx::Result<u64> {
    sqlx::query("DELETE FROM history WHERE time_watched < ?1")
        .bind(cutoff_ms)
        .execute(pool)
        .await
        .map(|r| r.rows_affected())
}
```

### 8.1 Bulk writes

Any operation touching more than ~20 rows (sync apply, playlist reorder, NeDB import) **must** run inside an explicit transaction. Without it, SQLite fsyncs per statement and throughput collapses from ~50k rows/s to ~200 rows/s.

```rust
let mut tx = pool.begin().await?;
for (pos, v) in videos.iter().enumerate() {
    sqlx::query("UPDATE playlist_videos SET position = ?1 WHERE id = ?2")
        .bind(pos as i64).bind(&v.id)
        .execute(&mut *tx).await?;
}
tx.commit().await?;
```

---

## 9. Entity Relationship Diagram

```
┌──────────────────┐        ┌──────────────────────────┐
│    profiles      │1      N│  profile_subscriptions   │
│ id (PK)          ├────────┤ profile_id (FK, CASCADE) │
│ name             │        │ channel_id (PK part)     │
│ bg/text_color    │        │ name, thumbnail          │
│ deleted          │        └────────────┬─────────────┘
└──────────────────┘                     ┆ (logical, no FK)
                                         ┆
┌──────────────────┐        ┌────────────▼─────────────┐
│    playlists     │1      N│   subscription_cache     │
│ id (PK)          ├──┐     │ channel_id (PK)          │
│ name (uniq live) │  │     │ videos/shorts/live JSON  │
│ kind, protected  │  │     │ fetched_at, expires_at   │
│ video_count      │  │     └──────────────────────────┘
└──────────────────┘  │
                      │     ┌──────────────────────────┐
                      └────►│    playlist_videos       │
                            │ id (PK)                  │
                            │ playlist_id (FK, CASCADE)│
                            │ video_id, position       │
                            └──────────────────────────┘

┌──────────────┐  ┌──────────────┐  ┌────────────────┐  ┌──────────────┐
│   history    │  │ watch_stats  │  │ search_history │  │ tab_sessions │
│ video_id PK  │  │ id PK        │  │ id PK          │  │ id PK        │
│ watch_progr. │  │ bucket_type  │  │ normalized uniq│  │ tabs JSON    │
│ time_watched │  │ seconds      │  │ hit_count      │  │ active_tab   │
└──────────────┘  └──────────────┘  └────────────────┘  └──────────────┘
        (standalone — no FKs, sync-tracked via synced_at)

┌──────────────┐  ┌──────────────────┐
│   settings   │  │ migration_state  │
│ key PK       │  │ collection PK    │
│ value/type   │  │ status, attempts │
└──────────────┘  └──────────────────┘
```

---

## 10. Performance & Maintenance

| Practice | Rationale |
|---|---|
| WAL journal | Readers never block the writer; essential for a UI that queries while syncing |
| `synchronous = NORMAL` | Safe under WAL; ~5× faster commits than `FULL` |
| Batch in transactions | Amortises fsync cost |
| Partial indexes | Tombstoned rows excluded → smaller B-trees |
| `STRICT` tables | Rejects type-mismatched writes at the engine level |
| Pagination everywhere | `find` commands take `limit`/`offset`; no unbounded `SELECT *` |
| `PRAGMA optimize` on close | Refreshes stat tables for the query planner |
| Weekly `VACUUM` | Reclaims space after history pruning; run when idle |

```rust
pub async fn shutdown(pool: &SqlitePool) {
    let _ = sqlx::query("PRAGMA optimize").execute(pool).await;
    let _ = sqlx::query("PRAGMA wal_checkpoint(TRUNCATE)").execute(pool).await;
    pool.close().await;
}
```

## 11. Backup

```rust
#[tauri::command]
pub async fn backup_database(state: tauri::State<'_, AppState>, dest: String) -> Result<String, AppError> {
    // VACUUM INTO produces a consistent, compacted copy without stopping writers.
    sqlx::query("VACUUM INTO ?1").bind(&dest).execute(&state.pool).await?;
    Ok(dest)
}
```

Backups are written to `<app_data>/backups/slytube-<YYYYMMDD-HHMMSS>.db`, retaining the newest 5 plus the pre-migration NeDB archive, which is never rotated away.

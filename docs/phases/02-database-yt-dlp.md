# Phase 02 — Database & yt-dlp Integration

| Field | Value |
|-------|-------|
| **Timeline** | Week 2 – Week 3 |
| **Duration** | 10 working days |
| **Risk Level** | 🟡 Medium |
| **Blocks** | Phases 03, 04, 05, 06 |
| **Depends On** | Phase 01 (sidecar pipeline, fs scope, plugin registry) |

---

## Status

**Status:** Complete ✅
**Completed:** 2026-08-10
**Notes:** All deliverables met. sqlx/SQLite with WAL mode, 6 migration files, repository layer (8 modules), NeDB importer with rollback, YtDlpService with concurrent downloads, progress event stream operational.

---

## 1. Goals

1. Stand up the persistence layer: `sqlx` + SQLite with compile-time-verified queries and versioned migrations.
2. Implement the full relational schema defined in [Database Schema](../backend/01-database-schema.md) and expose it through a typed repository layer.
3. Build a one-shot, idempotent, reversible **NeDB → SQLite** migration that imports existing OpenTubeX user data with zero loss.
4. Replace Electron's `child_process.spawn()` yt-dlp manager (1375 LOC) with a Rust sidecar service that supports concurrent, cancellable, resumable downloads.
5. Emit structured, throttled progress events consumable by the frontend download store.

**Non-goals:** UI for downloads (Phase 06), PoToken-gated formats (Phase 03), sync tables population (Phase 05 — schema only here).

---

## 2. Prerequisites

- Phase 01 exit criteria met (`health_check` returns a yt-dlp version on all 3 OSes).
- `sqlx-cli` installed: `cargo install sqlx-cli --no-default-features --features sqlite`.
- A representative OpenTubeX profile directory for migration testing (anonymized), containing `settings.db`, `history.db`, `playlists.db`, `profiles.db`, `subscription-cache.db`, `search-history.db`.

---

## 3. Tasks

### 3.1 sqlx Bootstrap (Day 1)

```bash
cargo add sqlx --features runtime-tokio,sqlite,migrate,chrono,json,macros \
  --no-default-features --manifest-path src-tauri/Cargo.toml
cargo add chrono uuid --manifest-path src-tauri/Cargo.toml
```

**Connection lifecycle** (`src-tauri/src/db/mod.rs`):

```rust
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions, SqliteJournalMode, SqliteSynchronous};
use std::{path::Path, str::FromStr, time::Duration};

pub async fn init_db(app_data_dir: &Path) -> Result<sqlx::SqlitePool, AppError> {
    std::fs::create_dir_all(app_data_dir)?;
    let db_path = app_data_dir.join("slytube.db");

    let opts = SqliteConnectOptions::from_str(&format!("sqlite://{}", db_path.display()))?
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .synchronous(SqliteSynchronous::Normal)
        .busy_timeout(Duration::from_secs(10))
        .foreign_keys(true)
        .pragma("cache_size", "-32768")     // 32 MB
        .pragma("temp_store", "MEMORY")
        .pragma("mmap_size", "268435456");  // 256 MB

    let pool = SqlitePoolOptions::new()
        .max_connections(8)
        .min_connections(1)
        .acquire_timeout(Duration::from_secs(15))
        .connect_with(opts)
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;
    Ok(pool)
}
```

- [ ] Add `.env` with `DATABASE_URL=sqlite://./dev.db` for `sqlx::query!` macro expansion.
- [ ] Commit `.sqlx/` offline query cache (`cargo sqlx prepare`) so CI builds without a live DB.
- [ ] Register the pool in `AppState` (`.manage(...)` in `lib.rs` setup hook).
- [ ] Add graceful shutdown: `pool.close().await` on `RunEvent::Exit`.

### 3.2 Migrations & Table Schemas (Day 2–4)

Migration files live in `src-tauri/migrations/`, applied in lexical order.

| File | Purpose | Key Tables |
|------|---------|------------|
| `001_initial_schema.sql` | Core domain | `settings`, `channels`, `videos`, `playlists`, `playlist_items` |
| `002_history_and_downloads.sql` | User activity | `watch_history`, `search_history`, `subscriptions`, `downloads`, `download_queue` |
| `003_sync_tables.sql` | Sync scaffolding (populated in Phase 05) | `sync_devices`, `sync_state`, `sync_conflicts`, `sync_log` |
| `004_potoken_cache.sql` | PoToken cache (consumed in Phase 03) | `potoken_cache` |
| `005_performance_indexes.sql` | Query tuning + FTS | indexes, `videos_fts` virtual table |
| `006_migration_bookkeeping.sql` | NeDB import audit trail | `legacy_migration_log` |

**Representative DDL — downloads:**

```sql
CREATE TABLE downloads (
    id                  TEXT PRIMARY KEY,             -- UUID v4
    video_id            TEXT NOT NULL,
    channel_id          TEXT,
    title               TEXT NOT NULL,
    status              TEXT NOT NULL CHECK (status IN
                          ('pending','downloading','paused','processing',
                           'completed','failed','cancelled')),
    quality             TEXT NOT NULL,
    format              TEXT NOT NULL,
    output_path         TEXT,
    part_path           TEXT,
    downloaded_bytes    INTEGER NOT NULL DEFAULT 0,
    total_bytes         INTEGER,
    progress            REAL    NOT NULL DEFAULT 0.0, -- 0.0 .. 1.0
    speed_bps           INTEGER NOT NULL DEFAULT 0,
    eta_seconds         INTEGER,
    fragment_index      INTEGER,
    fragment_count      INTEGER,
    retry_count         INTEGER NOT NULL DEFAULT 0,
    error_code          TEXT,
    error_message       TEXT,
    used_potoken        INTEGER NOT NULL DEFAULT 0,
    ytdlp_args          TEXT,                          -- JSON array, for reproducibility
    created_at          INTEGER NOT NULL DEFAULT (strftime('%s','now')),
    started_at          INTEGER,
    completed_at        INTEGER,
    updated_at          INTEGER NOT NULL DEFAULT (strftime('%s','now')),
    FOREIGN KEY (video_id)   REFERENCES videos(id)   ON DELETE CASCADE,
    FOREIGN KEY (channel_id) REFERENCES channels(id) ON DELETE SET NULL
);

CREATE INDEX idx_downloads_status     ON downloads(status, created_at DESC);
CREATE INDEX idx_downloads_video      ON downloads(video_id);
CREATE INDEX idx_downloads_active     ON downloads(status) WHERE status IN ('downloading','pending','paused');

CREATE TRIGGER trg_downloads_touch AFTER UPDATE ON downloads
BEGIN
    UPDATE downloads SET updated_at = strftime('%s','now') WHERE id = NEW.id;
END;
```

**Full-text search:**

```sql
CREATE VIRTUAL TABLE videos_fts USING fts5(
    id UNINDEXED, title, description, channel_title,
    content='videos', content_rowid='rowid', tokenize='unicode61 remove_diacritics 2'
);

CREATE TRIGGER trg_videos_fts_ai AFTER INSERT ON videos BEGIN
    INSERT INTO videos_fts(rowid, id, title, description) VALUES (new.rowid, new.id, new.title, new.description);
END;
CREATE TRIGGER trg_videos_fts_ad AFTER DELETE ON videos BEGIN
    INSERT INTO videos_fts(videos_fts, rowid, id, title, description) VALUES ('delete', old.rowid, old.id, old.title, old.description);
END;
CREATE TRIGGER trg_videos_fts_au AFTER UPDATE ON videos BEGIN
    INSERT INTO videos_fts(videos_fts, rowid, id, title, description) VALUES ('delete', old.rowid, old.id, old.title, old.description);
    INSERT INTO videos_fts(rowid, id, title, description) VALUES (new.rowid, new.id, new.title, new.description);
END;
```

**Checklist**

- [ ] Every migration is forward-only and idempotent under `IF NOT EXISTS` where safe.
- [ ] Every `status`/enum column carries a `CHECK` constraint.
- [ ] Every table has `created_at`/`updated_at` (unix seconds) and, where syncable, `synced_at` + `deleted_at` (soft delete tombstones for Phase 05).
- [ ] `cargo sqlx migrate run` + `cargo sqlx prepare --check` pass in CI.

### 3.3 Repository Layer (Day 4–5)

One module per aggregate under `src-tauri/src/db/repo/`:

```
repo/
├── mod.rs
├── settings.rs      // typed get/set with JSON value coercion
├── video.rs         // upsert, get, search (FTS), bulk_upsert
├── channel.rs
├── playlist.rs      // + playlist_items ordering
├── history.rs       // watch_history, search_history
├── subscription.rs
├── download.rs      // CRUD + status transitions + active-set queries
└── potoken.rs       // cache get/set/prune (Phase 03 consumer)
```

Pattern:

```rust
pub struct DownloadRepo<'a>(pub &'a SqlitePool);

impl<'a> DownloadRepo<'a> {
    pub async fn create(&self, req: NewDownload) -> Result<Download, AppError> {
        let id = uuid::Uuid::new_v4().to_string();
        let rec = sqlx::query_as!(
            Download,
            r#"INSERT INTO downloads (id, video_id, title, status, quality, format, ytdlp_args)
               VALUES (?1, ?2, ?3, 'pending', ?4, ?5, ?6)
               RETURNING id, video_id, title, status, quality, format,
                         downloaded_bytes, total_bytes, progress, created_at"#,
            id, req.video_id, req.title, req.quality, req.format, req.args_json
        )
        .fetch_one(self.0)
        .await?;
        Ok(rec)
    }

    pub async fn transition(&self, id: &str, to: DownloadStatus) -> Result<(), AppError> { /* ... */ }
    pub async fn active(&self) -> Result<Vec<Download>, AppError> { /* ... */ }
    pub async fn prune_completed(&self, older_than: i64) -> Result<u64, AppError> { /* ... */ }
}
```

- [ ] All write paths that touch >1 table use explicit transactions (`pool.begin()`).
- [ ] Bulk inserts batched at 500 rows/statement to avoid SQLite's 999-parameter limit.
- [ ] Every repo has a `#[cfg(test)]` module using an in-memory pool (`sqlite::memory:`).

### 3.4 NeDB → SQLite Migration Script (Day 5–7)

OpenTubeX stores NeDB append-only JSONL files. The importer must be **idempotent**, **resumable**, and **non-destructive**.

**Source mapping:**

| NeDB file | Target table(s) | Notes |
|-----------|-----------------|-------|
| `settings.db` | `settings` | Key remap + type coercion; unknown keys → `legacy_migration_log` |
| `history.db` | `videos`, `channels`, `watch_history` | Denormalized doc → 3 tables |
| `playlists.db` | `playlists`, `playlist_items`, `videos` | Preserve item order via `position` |
| `profiles.db` | `settings` (`profiles` JSON blob) + `subscriptions` | Multi-profile flattened to active profile + archive |
| `subscription-cache.db` | `channels` | Cache — safe to skip; imported for offline continuity |
| `search-history.db` | `search_history` | Dedup on `(query, lower(query))` |

**NeDB parsing rules:**
1. File is newline-delimited JSON; **later records supersede earlier ones by `_id`**.
2. A record containing `{"$$deleted": true}` tombstones its `_id`.
3. Records with `{"$$indexCreated": ...}` are index metadata → ignore.
4. Malformed trailing line (crash during write) → discard with a warning, do not abort.

```rust
pub fn parse_nedb(path: &Path) -> Result<Vec<serde_json::Value>, AppError> {
    let mut live: IndexMap<String, serde_json::Value> = IndexMap::new();
    let file = std::fs::File::open(path)?;
    for (lineno, line) in std::io::BufReader::new(file).lines().enumerate() {
        let Ok(line) = line else { continue };
        if line.trim().is_empty() { continue; }
        let doc: serde_json::Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(e) => { log::warn!("nedb {path:?}:{lineno} skipped: {e}"); continue; }
        };
        if doc.get("$$indexCreated").is_some() || doc.get("$$indexRemoved").is_some() { continue; }
        let Some(id) = doc.get("_id").and_then(|v| v.as_str()).map(String::from) else { continue };
        if doc.get("$$deleted").and_then(|v| v.as_bool()) == Some(true) { live.shift_remove(&id); }
        else { live.insert(id, doc); }
    }
    Ok(live.into_values().collect())
}
```

**Orchestration (`src-tauri/src/migration/legacy.rs`):**

```rust
#[derive(Serialize)]
pub struct MigrationReport {
    pub source_dir: String,
    pub started_at: i64,
    pub finished_at: i64,
    pub tables: Vec<TableReport>,   // name, read, inserted, updated, skipped, errors
    pub backup_path: String,
    pub dry_run: bool,
    pub success: bool,
}

#[tauri::command]
pub async fn migrate_legacy_data(
    source_dir: Option<String>,   // None → auto-detect OpenTubeX profile dir
    dry_run: bool,
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<MigrationReport, AppError> { /* ... */ }
```

**Safety protocol**

- [ ] **Auto-detect** legacy dir per OS: Linux `~/.config/OpenTubeX`, macOS `~/Library/Application Support/OpenTubeX`, Windows `%APPDATA%\OpenTubeX`.
- [ ] **Never mutate** the source directory — read-only file handles.
- [ ] Snapshot `slytube.db` to `slytube.db.pre-legacy-<ts>` before writing.
- [ ] Run the entire import inside **one transaction**; roll back on any hard error.
- [ ] `dry_run: true` produces the identical report without committing.
- [ ] Record a row in `legacy_migration_log` per source file with a SHA-256 of the file so re-runs are no-ops.
- [ ] Emit `legacy-migration-progress` events (`{ file, processed, total, phase }`) for the Phase 06 wizard UI.
- [ ] Provide `rollback_legacy_migration` restoring the pre-migration snapshot.

**Field-level coercions to verify**

| Legacy shape | Target | Rule |
|--------------|--------|------|
| `timeWatched` (ms epoch) | `watched_at` (s epoch) | integer divide by 1000 |
| `lengthSeconds` (string) | `duration` (INTEGER) | parse, default 0 |
| `viewCount` (`"1.2M views"`) | `view_count` (INTEGER) | suffix expansion, null on failure |
| `author` / `authorId` | `channels.title` / `channels.id` | upsert channel first (FK order) |
| `videos: [...]` in playlists | `playlist_items` | preserve array index as `position` |
| Boolean-as-string `"true"` | INTEGER 0/1 | strict coercion, log ambiguities |

### 3.5 yt-dlp Sidecar Service (Day 7–9)

Replaces `src/main/ytDlp.js`.

**Service shape** (`src-tauri/src/services/ytdlp/mod.rs`):

```rust
pub struct YtDlpService {
    app: AppHandle,
    pool: SqlitePool,
    jobs: Arc<DashMap<String, JobHandle>>,   // download_id → handle
    semaphore: Arc<Semaphore>,               // max concurrent downloads (setting)
}

struct JobHandle {
    child: Arc<Mutex<Option<CommandChild>>>,
    cancel: CancellationToken,
    paused: Arc<AtomicBool>,
}
```

**Argument builder** — a single canonical place that composes flags:

```rust
fn build_args(req: &DownloadRequest, cfg: &AppConfig) -> Vec<String> {
    let mut a = vec![
        "--newline".into(),
        "--progress".into(),
        "--progress-template".into(),
        // Machine-readable single-line JSON per tick
        r#"download:{"k":"p","id":"%(info.id)s","db":%(progress.downloaded_bytes)d,"tb":%(progress.total_bytes,progress.total_bytes_estimate)d,"sp":%(progress.speed)d,"eta":%(progress.eta)d,"fi":%(progress.fragment_index)d,"fc":%(progress.fragment_count)d}"#.into(),
        "--no-colors".into(),
        "--no-playlist".into(),
        "--no-warnings".into(),
        "--ignore-config".into(),
        "--restrict-filenames".into(),
        "--retries".into(), "10".into(),
        "--fragment-retries".into(), "10".into(),
        "--continue".into(),                       // resume .part files
        "--concurrent-fragments".into(), cfg.frag_concurrency.to_string(),
        "--ffmpeg-location".into(), cfg.ffmpeg_path.clone(),
        "-o".into(), cfg.output_template.clone(),
        "-f".into(), format_selector(&req.quality, &req.format),
    ];
    if let Some(p) = &cfg.proxy_url { a.extend(["--proxy".into(), p.clone()]); }
    if let Some(rl) = cfg.rate_limit { a.extend(["--limit-rate".into(), rl.clone()]); }
    if let Some(po) = &req.potoken {
        a.extend(["--extractor-args".into(),
                  format!("youtube:player-client=web,default;po_token=web.gvs+{po}")]);
    }
    if req.embed_metadata { a.extend(["--embed-metadata".into(), "--embed-thumbnail".into()]); }
    if req.write_subs { a.extend(["--write-subs".into(), "--sub-langs".into(), cfg.sub_langs.clone()]); }
    a.push(format!("https://www.youtube.com/watch?v={}", req.video_id));
    a
}
```

**Format selectors:**

| Quality | Selector |
|---------|----------|
| `best` | `bv*+ba/b` |
| `2160p` | `bv*[height<=2160]+ba/b[height<=2160]` |
| `1080p` | `bv*[height<=1080]+ba/b[height<=1080]` |
| `720p` | `bv*[height<=720]+ba/b[height<=720]` |
| `audio` (mp3) | `ba/b` + `-x --audio-format mp3 --audio-quality 0` |

**Spawn & stream:**

```rust
let (mut rx, child) = app.shell()
    .sidecar("yt-dlp")?
    .args(args)
    .spawn()?;

while let Some(ev) = rx.recv().await {
    match ev {
        CommandEvent::Stdout(line)   => handle_stdout(&line).await?,
        CommandEvent::Stderr(line)   => handle_stderr(&line).await?,
        CommandEvent::Terminated(p)  => { finalize(p.code).await?; break; }
        CommandEvent::Error(e)       => { fail(e).await?; break; }
        _ => {}
    }
}
```

**Commands delivered in this phase** (full catalogue in [Tauri Commands](../backend/02-tauri-commands.md)):

| Command | Behavior |
|---------|----------|
| `start_download` | Insert row → acquire semaphore → spawn → return `download_id` |
| `pause_download` | SIGSTOP-equivalent: kill child, keep `.part`, status → `paused` |
| `resume_download` | Re-spawn with `--continue`; byte offset recovered from `.part` |
| `cancel_download` | Kill + optional `.part` cleanup; status → `cancelled` |
| `retry_download` | Reset `retry_count` bounds and re-enqueue |
| `get_downloads` | Paginated query with status filter |
| `get_download_progress` | Point-in-time read from in-memory job map, fallback to DB |
| `probe_video_formats` | `--dump-single-json --no-download` → available formats |
| `get_ytdlp_version` / `update_ytdlp` | Version report; `-U` self-update where writable |

**Lifecycle rules**

- [ ] Concurrency capped by a `Semaphore` sized from settings (default 3).
- [ ] On app exit, all children are killed and in-flight rows transition `downloading → paused`.
- [ ] On app start, orphaned `downloading` rows are reconciled to `paused` (crash recovery).
- [ ] Every spawn is wrapped in a `CancellationToken::run_until_cancelled`.
- [ ] Windows: spawn with `CREATE_NO_WINDOW` so no console flashes.

### 3.6 Progress Events (Day 9–10)

**Event contract:**

| Event | Payload | Frequency |
|-------|---------|-----------|
| `download-progress` | `DownloadProgress` | throttled — max 4 Hz per job |
| `download-status` | `{ download_id, status, error_code?, error_message? }` | on transition |
| `download-completed` | `{ download_id, output_path, total_bytes, duration_ms }` | once |
| `download-failed` | `{ download_id, error_code, error_message, retryable }` | once |
| `ytdlp-log` | `{ download_id, level, line }` | dev builds only |
| `legacy-migration-progress` | `{ file, processed, total, phase }` | per 100 records |

```rust
#[derive(Clone, Serialize)]
pub struct DownloadProgress {
    pub download_id: String,
    pub video_id: String,
    pub status: DownloadStatus,
    pub progress: f64,             // 0.0 ..= 1.0
    pub downloaded_bytes: u64,
    pub total_bytes: u64,
    pub speed_bytes_per_sec: u64,
    pub eta_seconds: u64,
    pub current_fragment: u32,
    pub total_fragments: u32,
    pub phase: DownloadPhase,      // Video | Audio | Merging | PostProcessing
}
```

**Throttling & persistence policy**

- [ ] Emit to the frontend at ≤250 ms intervals per job (coalesce intermediate ticks).
- [ ] Persist to SQLite at ≤5 s intervals or on phase change — never per tick (avoids WAL thrash).
- [ ] Always emit a final 100 % tick before `download-completed`.
- [ ] Use `app.emit_to(EventTarget::labeled("main"), ...)` so the hidden PoToken window is not spammed.
- [ ] Classify stderr into `error_code`s: `NETWORK`, `GEO_BLOCKED`, `AGE_RESTRICTED`, `PRIVATE`, `REMOVED`, `FORMAT_UNAVAILABLE`, `DISK_FULL`, `FFMPEG`, `UNKNOWN`; mark `retryable` accordingly.

---

## 4. Deliverables

| ID | Deliverable | Acceptance Criteria |
|----|-------------|---------------------|
| D2.1 | sqlx bootstrap + pool in `AppState` | WAL enabled; FKs on; pool closes cleanly on exit |
| D2.2 | 6 migration files | `sqlx migrate run` idempotent; `sqlx prepare --check` green in CI |
| D2.3 | Repository layer (8 modules) | 100 % of tables reachable; ≥80 % unit-test coverage on repos |
| D2.4 | NeDB importer + rollback | Real profile imports with 0 data loss; dry-run report matches actual run |
| D2.5 | `YtDlpService` | 3 concurrent downloads; pause/resume across app restart |
| D2.6 | 9 download commands | All registered, typed, documented |
| D2.7 | Progress event stream | ≤4 Hz UI events; verified against a 1 GB download |
| D2.8 | Integration test suite | `cargo test --features integration` covers import + download happy/sad paths |

---

## 5. Dependencies

**Inbound**

| From | Needs |
|------|-------|
| Phase 01 | Sidecar binaries + `shell:allow-execute` scope; `fs` scope for output dirs; app data dir |

**Outbound**

| Phase | Consumes |
|-------|----------|
| 03 | `potoken_cache` table + repo |
| 04 | Pool, repositories, `AppError`, event conventions |
| 05 | `sync_*` tables, soft-delete tombstones, `updated_at` semantics |
| 06 | `download-progress` event contract, download command signatures |
| 08 | Migration verification fixtures, performance baselines |

**External:** yt-dlp CLI stability (`--progress-template` fields), ffmpeg availability for muxing.

---

## 6. Risks

| ID | Risk | Prob. | Impact | Mitigation |
|----|------|-------|--------|------------|
| R2.1 | `sqlx::query!` macros fail in CI without a live DB | High | Medium | Commit `.sqlx/` offline cache; `SQLX_OFFLINE=true` in CI |
| R2.2 | NeDB import silently drops/corrupts user data | Medium | **Critical** | Dry-run mode, pre-import snapshot, per-file SHA log, rollback command, golden-fixture tests in Phase 08 |
| R2.3 | Legacy `_id` collisions across NeDB files | Low | High | Namespace legacy IDs (`legacy:<file>:<id>`) in `legacy_migration_log` |
| R2.4 | `SQLITE_BUSY` under concurrent download writes | Medium | Medium | WAL + `busy_timeout(10s)` + batched progress persistence + single writer task |
| R2.5 | yt-dlp changes progress-template field names | Medium | High | Parse defensively (missing field → 0); pin sidecar version; nightly canary CI job |
| R2.6 | Pause/resume corrupts `.part` files | Medium | High | Always `--continue`; validate `.part` size before resume; on mismatch restart fragment |
| R2.7 | Zombie yt-dlp processes after crash | Medium | Medium | Track PIDs in `download_queue`; reap orphans on startup |
| R2.8 | ffmpeg merge fails on exotic codecs | Low | Medium | Fall back to `--remux-video mp4`; surface `FFMPEG` error code |
| R2.9 | Windows path length > 260 chars | Medium | Medium | `--restrict-filenames` + truncate title to 120 chars + `\\?\` prefix |
| R2.10 | Progress event flood freezes the UI | Medium | Medium | 250 ms coalescing; batch payloads; verified in Phase 08 profiling |

---

## 7. Estimated Duration

| Task | Days |
|------|------|
| 3.1 sqlx bootstrap | 1.0 |
| 3.2 Migrations & schemas | 2.5 |
| 3.3 Repository layer | 1.5 |
| 3.4 NeDB importer | 2.5 |
| 3.5 yt-dlp sidecar service | 2.0 |
| 3.6 Progress events | 0.5 |
| **Total** | **10.0** (2 weeks @ 1 dev) |

---

## 8. Exit Criteria

- [ ] Fresh install creates `slytube.db` and applies all migrations without error on 3 OSes.
- [ ] A real OpenTubeX profile imports with a report showing 0 errors; spot-check of 20 records matches source.
- [ ] `rollback_legacy_migration` restores byte-identical pre-import state.
- [ ] 3 concurrent 1080p downloads complete; one paused mid-flight resumes correctly after an app restart.
- [ ] Cancel leaves no orphan process and no stray `.part` when requested.
- [ ] `cargo clippy -- -D warnings` and `cargo test` green.

---

## 9. References

- [Backend — Database Schema](../backend/01-database-schema.md)
- [Backend — Tauri Commands](../backend/02-tauri-commands.md)
- [Architecture — Data Flow](../architecture/03-data-flow.md)
- Previous: [Phase 01 — Foundation](01-foundation.md) · Next: [Phase 03 — PoToken Generator](03-potoken-generator.md)

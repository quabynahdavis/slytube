# ADR 004: Database Access Layer

| Field | Value |
|-------|-------|
| **Status** | Accepted |
| **Date** | 2026-08-09 |
| **Deciders** | Migration Team |
| **Supersedes** | — |
| **Related** | [03-sync-encryption.md](03-sync-encryption.md), [05-migration-approach.md](05-migration-approach.md) |

---

## Context

The Electron application persists user data with **NeDB** — an append-only, JSON-per-line
document store — behind a handler abstraction (`src/datastores/handlers/base.js`, ~775 lines,
plus per-collection handlers). Collections cover settings, subscriptions, playlists, history,
profiles, and search history.

NeDB's limitations have become structural problems:

| Problem | Effect |
|---------|--------|
| Whole-file rewrite on compaction | Multi-hundred-ms stalls once history grows past a few thousand entries |
| Entire dataset held in memory | Memory footprint scales linearly with history size |
| No real indexes | Filters degrade to full scans |
| No transactions | Multi-collection writes (e.g. sync apply) can tear |
| No schema | Field drift across app versions with no migration story |
| Unmaintained upstream | Security and correctness fixes are not forthcoming |

The migration to Tauri is the natural point to move to **SQLite**: embedded, transactional,
indexed, ubiquitous, and well-supported in Rust. SQLite is settled; the open question is
**which Rust access layer** to use.

### Evaluation criteria

1. Compile-time confidence in query correctness (NeDB gave us none).
2. First-class async — the Tauri backend is Tokio-based and must not block the runtime.
3. Built-in, versioned migrations, including the one-time NeDB → SQLite import.
4. Ability to express the JSON-column hybrid model (some documents stay semi-structured).
5. Reasonable build times and CI ergonomics.

---

## Options Considered

### Option A — `sqlx`

Async, compile-time-verified SQL toolkit. Queries are written as SQL; the `query!` macros verify
them against a live database (or a cached offline schema) at build time.

```rust
// Verified at compile time: table/column existence, types, nullability, arity
let row = sqlx::query!(
    "SELECT id, title, channel_id, duration FROM videos WHERE id = ?",
    video_id
)
.fetch_one(&pool)
.await?;
// row.title is &str, row.duration is i64 — inferred from the schema
```

| Pros | Cons |
|------|------|
| **Compile-time checked queries** — schema drift becomes a build error | Macros need a reachable DB or a checked-in offline cache |
| Async-native, designed for Tokio | `cargo sqlx prepare` must be re-run when SQL or schema changes |
| **Migrations built in** (`sqlx::migrate!`, embedded in the binary) | Raw SQL — no ORM ergonomics for complex object graphs |
| Plain SQL — no DSL to learn or fight | Macro expansion adds some compile time |
| Strong `serde_json` support for hybrid JSON columns | Dynamic query construction is clumsier than with an ORM |
| Connection pooling out of the box | |

### Option B — `rusqlite`

Thin, mature, synchronous binding to the SQLite C API.

| Pros | Cons |
|------|------|
| Minimal, extremely stable, low dependency count | **Synchronous** — every call must be wrapped in `spawn_blocking` |
| Complete SQLite surface (extensions, hooks, VFS, backup API) | **No compile-time query verification** — typos are runtime errors |
| Fast compiles | Migrations require an external crate or hand-rolled versioning |
| No build-time database needed | Manual row→struct mapping for every query |
| Easy bundled-SQLite builds | Easy to accidentally block the Tokio runtime |

### Option C — `sea-orm`

Full async ORM built on top of `sqlx`.

| Pros | Cons |
|------|------|
| Entity models, relations, and an ActiveModel write API | Heavy dependency tree; noticeably slower compiles |
| Async, multi-backend | Adds a DSL abstraction over SQL we would frequently escape from |
| Codegen from an existing schema | Compile-time guarantees are weaker than `sqlx`'s macros |
| Migration framework included | ORM semantics are a poor fit for hybrid JSON-column documents |
| | Overkill: our access patterns are simple CRUD + a few aggregates |

---

## Decision

**Adopt Option A — `sqlx` with SQLite.**

```toml
# src-tauri/Cargo.toml
sqlx = { version = "0.8", features = [
    "runtime-tokio",
    "sqlite",
    "macros",
    "migrate",
    "json",
    "chrono",
] }
```

### Layout

```
src-tauri/
├── migrations/
│   ├── 0001_initial_schema.sql
│   ├── 0002_indexes.sql
│   └── ...
├── .sqlx/                  # offline query metadata (committed)
└── src/db/
    ├── mod.rs              # pool construction, migrate-on-start
    ├── models.rs           # FromRow structs
    ├── queries/            # one module per collection
    └── import_nedb.rs      # one-time NeDB → SQLite migration
```

Migrations are embedded and applied at startup:

```rust
let pool = SqlitePoolOptions::new()
    .max_connections(8)
    .connect_with(
        SqliteConnectOptions::new()
            .filename(db_path)
            .create_if_missing(true)
            .journal_mode(SqliteJournalMode::Wal)
            .foreign_keys(true),
    )
    .await?;

sqlx::migrate!("./migrations").run(&pool).await?;
```

---

## Rationale

1. **Compile-time checked queries.** This is the decisive factor. We are migrating *away* from a
   schemaless store precisely because field drift went undetected until runtime. `sqlx::query!`
   inverts that: rename a column and every affected query fails the build. Neither `rusqlite`
   (no verification) nor `sea-orm` (DSL-level typing only, no SQL verification) offers this.

2. **Async by design.** The Tauri backend is Tokio-based and concurrently serves downloads, sync,
   and PoToken work. `sqlx` participates in that runtime natively. `rusqlite` would require
   disciplined `spawn_blocking` at every call site — a correctness hazard that is easy to get
   wrong and hard to detect in review.

3. **Migrations built in.** `sqlx::migrate!` embeds versioned SQL migrations into the binary and
   applies them transactionally on startup, with a tracked version table. This directly serves
   the one-time NeDB import and every future schema change — no extra crate, no bespoke
   versioning code. `rusqlite` has no answer here.

4. **Plain SQL suits the workload.** Access patterns are CRUD plus a handful of aggregates and
   joins. An ORM's object-graph machinery is unused weight, and it actively obstructs the hybrid
   model where some columns hold `serde_json::Value`. Writing SQL directly is both simpler and
   more transparent.

5. **Proportionate cost.** `sea-orm` sits on `sqlx` anyway; adopting it means paying for `sqlx`
   plus an abstraction we would routinely bypass, at the cost of longer compiles and weaker
   guarantees.

---

## Implications

### Build-time database requirement

The `query!` family verifies SQL against a real schema at compile time. This means **the build
needs schema access** — the single most important operational consequence of this decision.

Two modes:

| Mode | Mechanism | When |
|------|-----------|------|
| **Live** | `DATABASE_URL=sqlite://dev.db` set at build time | Local development |
| **Offline** | `.sqlx/` metadata directory, generated by `cargo sqlx prepare` | CI, release builds, contributors |

**Offline mode is mandatory for CI and for any contributor who has not provisioned a dev
database.** Requirements:

- [ ] `.sqlx/` is **committed to version control**.
- [ ] Any change to SQL or schema requires re-running:
      ```bash
      cargo sqlx prepare --workspace -- --all-targets
      ```
- [ ] CI enforces freshness:
      ```bash
      cargo sqlx prepare --check --workspace
      ```
      A stale cache fails the build rather than silently falling back.
- [ ] `sqlx-cli` is documented as a required dev tool:
      ```bash
      cargo install sqlx-cli --no-default-features --features sqlite
      ```
- [ ] A `make db-reset` / script recreates `dev.db` from `migrations/` for onboarding.

### Schema and data migration

- [ ] Design a relational schema with JSON columns only where documents are genuinely
      semi-structured (e.g. cached metadata blobs); everything queried or filtered gets a real
      column and an index.
- [ ] `import_nedb.rs` performs a **one-time, idempotent** import: read each NeDB `.db` file,
      parse line-delimited JSON, apply NeDB's last-write-wins semantics, insert in a single
      transaction, then mark completion in a `meta` row.
- [ ] Back up the original NeDB files before import; never delete them in the same release.
- [ ] Import must be resumable and must not run twice.

### Runtime consequences

| Area | Consequence |
|------|-------------|
| **Sync → async** | Every former `better-sqlite3`/NeDB synchronous call becomes `.await`. Call chains up to the Tauri commands are async throughout. |
| **WAL mode** | Enable WAL for concurrent readers alongside the sync writer. Requires handling `SQLITE_BUSY` with a busy timeout. |
| **Transactions** | Multi-collection operations (notably sync apply) run in a single transaction — a capability NeDB never had. Use it. |
| **Pool sizing** | SQLite tolerates one writer. Size the pool for readers and funnel writes through a single path to avoid lock contention. |
| **Compile time** | Macro expansion measurably increases build time. Acceptable; monitor and consider `query_as` with hand-written structs for the hottest paths if it regresses. |
| **Dynamic queries** | Anything not expressible as static SQL (e.g. user-built filters) uses `QueryBuilder`, which forfeits compile-time checking. Keep these few, isolated, and directly tested. |

---

## References

- Electron baseline: `src/datastores/handlers/base.js` (775 lines) + per-collection handlers
- [`sqlx` documentation](https://docs.rs/sqlx/) · [offline mode](https://github.com/launchbadge/sqlx/blob/main/sqlx-cli/README.md#enable-building-in-offline-mode-with-query)
- [SQLite WAL mode](https://www.sqlite.org/wal.html)
- [../backend/01-database-schema.md](../backend/01-database-schema.md)

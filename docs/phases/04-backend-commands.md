# Phase 04 — Backend Commands

| Field | Value |
|-------|-------|
| **Timeline** | Week 4 – Week 6 |
| **Duration** | 15 working days |
| **Risk Level** | 🟡 Medium |
| **Blocks** | Phase 06 (frontend needs the full API surface) |
| **Depends On** | Phase 02 (pool, repos), Phase 03 (potoken service) |

---

## 1. Goals

1. Implement the **complete Tauri command surface** — every IPC handler from Electron's `src/main/index.js` (4558 LOC) has a typed Rust equivalent.
2. Establish uniform command ergonomics: one `AppError`, one `PaginatedResponse<T>`, consistent camelCase↔snake_case mapping, consistent event naming.
3. Deliver database CRUD for all seven aggregates (settings, videos, channels, playlists, history, subscriptions, downloads).
4. Provide sync command *stubs* with final signatures so Phase 05 can fill logic without breaking Phase 06 integration.
5. Implement network/proxy layer (HTTP/SOCKS5/Tor) and Invidious instance management with health probing.
6. Implement window/tray command primitives that Phase 07 configures.
7. Auto-generate TypeScript bindings so the frontend cannot drift from the Rust contract.

---

## 2. Command Inventory

Target: **~78 commands** across 9 domains.

| Domain | Count | Status after this phase |
|--------|-------|-------------------------|
| Settings | 7 | Complete |
| Video / Channel / Playlist | 18 | Complete |
| History & Subscriptions | 11 | Complete |
| Downloads | 9 | Wired from Phase 02 |
| PoToken | 3 | Wired from Phase 03 |
| Sync | 8 | **Signatures + stubs** (Phase 05 fills) |
| Network / Proxy | 8 | Complete |
| Window / Tray | 9 | Primitives complete (Phase 07 configures) |
| System | 5 | Complete |

---

## 3. Tasks

### 3.1 Command Infrastructure (Day 1–2)

**Module layout:**

```
src-tauri/src/
├── lib.rs                  // builder, plugin + handler registration
├── state.rs                // AppState
├── error.rs                // AppError + ApiError serialization
├── commands/
│   ├── mod.rs
│   ├── types.rs            // PaginatedResponse, common DTOs
│   ├── settings.rs
│   ├── video.rs
│   ├── channel.rs
│   ├── playlist.rs
│   ├── history.rs
│   ├── subscription.rs
│   ├── download.rs
│   ├── potoken.rs
│   ├── sync.rs
│   ├── proxy.rs
│   ├── window.rs
│   └── system.rs
└── services/               // business logic, called by commands
```

**`AppState`:**

```rust
pub struct AppState {
    pub pool: SqlitePool,
    pub http: RwLock<reqwest::Client>,        // rebuilt when proxy changes
    pub config: RwLock<AppConfig>,            // hot settings cache
    pub ytdlp: Arc<YtDlpService>,
    pub potoken: Arc<PoTokenService>,
    pub sync: Arc<SyncService>,
    pub invidious: Arc<InvidiousService>,
    pub shortcuts: RwLock<HashMap<String, String>>,
}
```

**Error contract** — `AppError` serializes to a stable `ApiError { code, message, details }`:

```rust
impl serde::Serialize for AppError {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        ApiError { code: self.code().into(), message: self.to_string(), details: self.details() }
            .serialize(s)
    }
}
```

**Conventions (enforced in review):**

- [ ] Every command is `async` and returns `Result<T, AppError>`.
- [ ] Every command taking user input validates before touching the DB (`validator` crate).
- [ ] Parameter naming: Rust `snake_case`; frontend passes `camelCase` — enable via `#[tauri::command(rename_all = "camelCase")]` consistently across **all** commands.
- [ ] Long-running commands (>500 ms) emit progress events rather than blocking.
- [ ] No command performs UI work; all UI effects go through events.
- [ ] Every command has a rustdoc block with an example `invoke` call.

### 3.2 TypeScript Binding Generation (Day 2)

```bash
cargo add specta specta-typescript tauri-specta --features derive --manifest-path src-tauri/Cargo.toml
```

```rust
#[cfg(debug_assertions)]
tauri_specta::Builder::<tauri::Wry>::new()
    .commands(tauri_specta::collect_commands![/* all commands */])
    .events(tauri_specta::collect_events![DownloadProgress, SyncStatusChanged, /* ... */])
    .export(specta_typescript::Typescript::default(), "../src/lib/bindings.ts")
    .expect("failed to export bindings");
```

- [ ] `src/lib/bindings.ts` is generated, committed, and CI-verified as up to date (`git diff --exit-code`).
- [ ] Frontend **never** hand-writes `invoke` payload types.

### 3.3 Settings Commands (Day 2–3)

| Command | Signature summary |
|---------|-------------------|
| `get_settings` | `() -> Settings` |
| `save_settings` | `(settings: Settings) -> ()` |
| `patch_setting` | `(key: String, value: JsonValue) -> Settings` |
| `reset_settings` | `() -> Settings` |
| `export_settings` | `() -> String` (path) |
| `import_settings` | `(file_path: String) -> Settings` |
| `get_default_download_dir` | `() -> String` |

- [ ] Settings are stored as typed key-value rows; `Settings` is assembled/validated in Rust.
- [ ] Any mutation that affects the HTTP client (proxy) or the sidecar (rate limit) triggers a service reconfigure.
- [ ] `settings-changed` event emitted on every write so all windows stay coherent.
- [ ] Import validates schema version and refuses downgrades with a clear error.

### 3.4 Database CRUD Commands (Day 3–7)

#### Videos & Channels

| Command | Notes |
|---------|-------|
| `search_videos` | Local FTS or remote Invidious, `use_local` flag, paginated |
| `get_video` / `get_video_details` | Details path may consume a PoToken |
| `get_related_videos` | |
| `upsert_video` / `bulk_upsert_videos` | Batched at 500 rows |
| `delete_video` | Soft delete (tombstone for sync) |
| `get_channel` / `get_channel_videos` | |
| `refresh_channel` | Re-fetch metadata |

#### Playlists

| Command | Notes |
|---------|-------|
| `get_playlists` / `get_playlist` / `get_playlist_videos` | |
| `create_playlist` / `update_playlist` / `delete_playlist` | |
| `add_to_playlist` / `remove_from_playlist` | |
| `reorder_playlist_items` | Single transaction rewriting `position` |
| `duplicate_playlist` / `export_playlist` / `import_playlist` | JSON + `.m3u` |

#### History & Subscriptions

| Command | Notes |
|---------|-------|
| `get_watch_history` | Paginated + date-range filter |
| `record_watch_progress` | Debounced writes (≥5 s) from the player |
| `remove_from_history` / `clear_watch_history` | |
| `get_search_history` / `add_search_history` / `clear_search_history` | Dedup + cap 500 |
| `get_subscriptions` / `subscribe_channel` / `unsubscribe_channel` | |
| `get_subscription_feed` | Merged, deduped, paginated feed with cache-age policy |
| `refresh_subscription_feed` | Concurrent fetch (bounded to 6 in-flight), emits `feed-refresh-progress` |

**Cross-cutting requirements**

- [ ] Every list command returns `PaginatedResponse<T>` — never an unbounded `Vec`.
- [ ] `page_size` clamped to ≤100.
- [ ] Multi-table writes wrapped in transactions.
- [ ] Every mutating command bumps `updated_at` and marks the row dirty for sync.
- [ ] Deletes are soft (tombstones) for syncable entities; hard-delete only via `purge_deleted`.

### 3.5 Sync Commands — Signatures & Stubs (Day 7–8)

Implemented as compiling stubs returning `AppError::Sync("not_implemented")` where logic is pending, but with **final** signatures and DTOs so Phase 06 can integrate immediately.

| Command | Phase 05 fills |
|---------|----------------|
| `get_sync_status` | ✅ real (reads local state) |
| `enable_sync` | stub → key derivation + device registration |
| `disable_sync` | ✅ real (clears local state) |
| `trigger_sync` | stub → snapshot protocol |
| `get_sync_devices` / `remove_sync_device` | stub |
| `resolve_conflict` | stub |
| `set_privacy_mode` | ✅ real (persists mode) |

- [ ] DTOs (`SyncStatus`, `SyncResult`, `SyncDevice`, `SyncConflict`, `PrivacyMode`) finalized here — Phase 05 must not change them.
- [ ] `sync-status-changed` and `sync-progress` events defined and emitted by the stub scheduler.

### 3.6 Network & Proxy Commands (Day 8–11)

**Client factory** — one place builds every outbound client:

```rust
pub fn build_client(cfg: &ProxyConfig, ua: &str) -> Result<reqwest::Client, AppError> {
    let mut b = reqwest::Client::builder()
        .user_agent(ua)
        .timeout(Duration::from_secs(30))
        .connect_timeout(Duration::from_secs(10))
        .pool_max_idle_per_host(6)
        .redirect(reqwest::redirect::Policy::limited(5));

    if cfg.enabled {
        let url = match cfg.kind {
            ProxyKind::Http   => format!("http://{}:{}", cfg.host, cfg.port),
            ProxyKind::Socks5 => format!("socks5h://{}:{}", cfg.host, cfg.port), // remote DNS
            ProxyKind::Tor    => "socks5h://127.0.0.1:9050".into(),
        };
        let mut p = reqwest::Proxy::all(url)?;
        if let (Some(u), Some(pw)) = (&cfg.username, &cfg.password) {
            p = p.basic_auth(u, pw);
        }
        b = b.proxy(p);
    } else {
        b = b.no_proxy();
    }
    Ok(b.build()?)
}
```

| Command | Notes |
|---------|-------|
| `get_proxy_config` / `set_proxy_config` | Rebuilds the shared client + updates yt-dlp args |
| `test_proxy` | Latency + egress IP + DNS-leak indicator |
| `get_proxy_status` | |
| `fetch_invidious_instances` | From `api.invidious.io`, cached 6 h |
| `probe_invidious_instance` | Health, latency, API/video capability |
| `set_invidious_instance` / `get_invidious_instance` | |
| `auto_select_invidious_instance` | Probe top N concurrently, pick lowest latency healthy |

- [ ] SOCKS5 must use `socks5h` (remote DNS) — prevents DNS leaks.
- [ ] Proxy credentials stored via OS keychain, never in plaintext settings.
- [ ] Instance failover: on 3 consecutive failures, auto-rotate and emit `invidious-instance-changed`.
- [ ] All outbound requests carry a consistent UA and honor `Retry-After`.
- [ ] Rate limiting: token bucket per host (default 10 req/s) to avoid instance bans.

### 3.7 Window & Tray Commands (Day 11–13)

Primitives only — Phase 07 wires the actual tray menu, shortcuts, and OS integration.

| Command | Notes |
|---------|-------|
| `show_main_window` / `hide_main_window` / `toggle_main_window` | |
| `minimize_to_tray` | Honors `minimizeToTray` setting |
| `set_always_on_top` | For mini-player |
| `open_mini_player` / `close_mini_player` | Second `WebviewWindow`, 480×270, always-on-top |
| `save_window_state` / `restore_window_state` | Geometry via `store` plugin; multi-monitor safe |
| `set_tray_tooltip` / `set_tray_icon_state` | `idle` / `downloading` / `error` |
| `set_progress_bar` | Taskbar/Dock progress (Windows + macOS) |
| `request_attention` | Flash taskbar on download completion |

- [ ] Window geometry restore validates the saved rect intersects a currently-connected monitor; otherwise centers on primary.
- [ ] `CloseRequested` on `main` respects `closeToTray`: prevent-close + hide, else full exit.
- [ ] Mini-player shares Pinia state with `main` via Tauri events (no duplicate DB reads).

### 3.8 System Commands (Day 13)

| Command | Notes |
|---------|-------|
| `get_app_info` | name, version, tauri/rust version, platform, arch, build type |
| `open_external` | Via `opener`; validates scheme is `https`/`http`/`mailto` |
| `show_in_folder` | Platform-specific reveal (`explorer /select`, `open -R`, `dbus FileManager1`) |
| `get_disk_usage` | Free space on the download volume; blocks downloads under 500 MB |
| `get_logs` / `open_log_dir` | Diagnostics support |

### 3.9 Registration, Testing & Docs (Day 14–15)

- [ ] All ~78 commands registered in `generate_handler!` and mirrored in `bindings.ts`.
- [ ] Integration tests using `tauri::test::mock_builder()` with an in-memory DB per test.
- [ ] Contract tests: for each command, assert the serialized error shape on the failure path.
- [ ] Update [Backend — Tauri Commands](../backend/02-tauri-commands.md) so it exactly matches the implemented surface (no drift).
- [ ] Benchmark: p95 latency per command class recorded for Phase 08.

| Command class | p95 budget |
|---------------|-----------|
| Settings read | < 5 ms |
| Local DB list (page 20) | < 20 ms |
| FTS search | < 50 ms |
| Remote Invidious call | < 1500 ms |
| PoToken (cached) | < 5 ms |

---

## 4. Deliverables

| ID | Deliverable | Acceptance Criteria |
|----|-------------|---------------------|
| D4.1 | Command infrastructure | Single `AppError`; `rename_all = "camelCase"` everywhere; `AppState` managed |
| D4.2 | Generated `bindings.ts` | CI fails if stale |
| D4.3 | Settings (7) | Round-trip export/import; `settings-changed` event |
| D4.4 | Database CRUD (29) | All aggregates covered; pagination enforced; soft deletes |
| D4.5 | Sync stubs (8) | Final DTOs; compiles; Phase 06 can integrate |
| D4.6 | Network/proxy (8) | HTTP/SOCKS5/Tor verified; no DNS leak; instance failover |
| D4.7 | Window/tray (9) | Multi-monitor safe restore; mini-player functional |
| D4.8 | System (5) | Reveal-in-folder verified on 3 OSes |
| D4.9 | Test suite | ≥70 % coverage on `commands/`; error-shape contract tests |
| D4.10 | Updated command reference doc | Zero drift vs implementation |

---

## 5. Dependencies

**Inbound**

| From | Needs |
|------|-------|
| Phase 02 | Pool, repositories, download service, event conventions |
| Phase 03 | `PoTokenService` for `get_video_details` / `start_download` |
| Phase 01 | Plugins (`dialog`, `opener`, `store`, `http`, `clipboard-manager`), capabilities |

**Outbound**

| Phase | Consumes |
|-------|----------|
| 05 | Sync DTOs + stubs to fill |
| 06 | Entire command surface + `bindings.ts` |
| 07 | Window/tray primitives |
| 08 | Latency budgets, contract tests |

**External:** Invidious instance availability; `reqwest` SOCKS feature; OS keychain APIs.

---

## 6. Risks

| ID | Risk | Prob. | Impact | Mitigation |
|----|------|-------|--------|------------|
| R4.1 | Inconsistent camelCase/snake_case causes silent `null` params | High | High | Mandate `rename_all = "camelCase"`; generated bindings; lint rule in review checklist |
| R4.2 | Command surface drifts from docs and frontend | High | Medium | `tauri-specta` generation + CI staleness check; doc updated in the same PR |
| R4.3 | Scope creep — 78 commands in 3 weeks | High | High | Freeze the inventory at day 2; anything new goes to a backlog doc |
| R4.4 | Blocking work on the async runtime stalls IPC | Medium | High | `spawn_blocking` for CPU-bound work; no `std::fs` in async paths |
| R4.5 | Proxy misconfiguration leaks real IP | Medium | **High** | `socks5h` only; `test_proxy` asserts egress IP differs; DNS-leak check; deny direct fallback when proxy enabled |
| R4.6 | Invidious instances rate-limit or die mid-session | High | Medium | Health probing, auto-rotation, token-bucket throttle, local cache first |
| R4.7 | Credentials stored in plaintext settings | Medium | High | OS keychain via `keyring` crate; settings store only a reference |
| R4.8 | Window state restore off-screen on monitor change | Medium | Low | Validate rect against connected monitors |
| R4.9 | Sync stubs harden into permanent API mistakes | Medium | Medium | Review DTOs with Phase 05 owner before day 8 sign-off |
| R4.10 | `AppError` variants leak internal paths/SQL to the UI | Medium | Medium | `message` is user-safe; internals only in `details` under debug builds |

---

## 7. Estimated Duration

| Task | Days |
|------|------|
| 3.1 Infrastructure | 2.0 |
| 3.2 TS bindings | 0.5 |
| 3.3 Settings | 1.0 |
| 3.4 Database CRUD | 4.0 |
| 3.5 Sync stubs | 1.0 |
| 3.6 Network/proxy | 3.0 |
| 3.7 Window/tray | 2.0 |
| 3.8 System | 0.5 |
| 3.9 Registration/tests/docs | 1.0 |
| **Total** | **15.0** (3 weeks @ 1 dev) |

> Weeks 5–6 overlap with Phase 05. Recommended split: developer A finishes 3.6–3.9 while developer B starts Phase 05 against the frozen sync DTOs.

---

## 8. Exit Criteria

- [ ] All ~78 commands registered, typed, and reachable from the frontend.
- [ ] `bindings.ts` regenerates with no diff in CI.
- [ ] Every command class meets its p95 latency budget on a 10k-video seeded database.
- [ ] Proxy test passes for HTTP, SOCKS5, and Tor with verified IP change and no DNS leak.
- [ ] `cargo clippy -- -D warnings`, `cargo test` green on 3 OSes.
- [ ] [Backend — Tauri Commands](../backend/02-tauri-commands.md) matches implementation exactly.

---

## 9. References

- [Backend — Tauri Commands](../backend/02-tauri-commands.md)
- [Backend — Database Schema](../backend/01-database-schema.md)
- [Architecture — IPC Patterns](../architecture/03-data-flow.md#ipc-communication-patterns)
- Previous: [Phase 03 — PoToken Generator](03-potoken-generator.md) · Next: [Phase 05 — Sync & Encryption](05-sync-encryption.md)

# ADR 005: Electron → Tauri Migration Approach

| Field | Value |
|-------|-------|
| **Status** | Accepted |
| **Date** | 2026-08-09 |
| **Deciders** | Migration Team |
| **Supersedes** | — |
| **Related** | [01-potoken-strategy.md](01-potoken-strategy.md), [03-sync-encryption.md](03-sync-encryption.md), [04-database-choice.md](04-database-choice.md), [06-theme-strategy.md](06-theme-strategy.md) |

---

## Context

Slytube is a rewrite of OpenTubeX from **Electron** to **Tauri v2**. The motivation is
established in [../architecture/01-electron-vs-tauri.md](../architecture/01-electron-vs-tauri.md):
~200 MB → ~10 MB binary, ~70% memory reduction, ~5× faster startup, capability-based security.

The codebase splits roughly as:

| Layer | Size | Portability |
|-------|------|-------------|
| Main process (`src/main/index.js`) | 4558 lines | **Must be rewritten** in Rust |
| Preload (`src/preload/interface.js`) | 1023 lines | **Eliminated** — no equivalent |
| yt-dlp manager (`src/main/ytDlp.js`) | 1375 lines | Rewritten as a sidecar driver |
| Datastores | ~775 lines + handlers | Rewritten on `sqlx` (ADR 004) |
| Renderer (Vue 3 SPA, stores, views, helpers) | The bulk of the app | **Largely portable** |

The critical structural observation: **Slytube is renderer-heavy**. The Vue application —
views, components, stores, API helpers — represents most of the code and most of the product
value, and it carries over with mechanical changes (`window.api.x()` → `invoke('x')`,
`ipcRenderer.on` → `listen`). What must be rewritten is the privileged layer, and that layer has
**no incremental path**: Electron's main process and Tauri's Rust backend cannot coexist in one
running application.

The question is how to sequence the work.

---

## Options Considered

### Option A — Phased (parallel run)

Ship Electron and Tauri side by side. Migrate subsystems one at a time, keeping both builds
releasable throughout; users stay on Electron until Tauri reaches parity.

| Pros | Cons |
|------|------|
| Each subsystem can be validated in production before the next | **Requires maintaining two backends simultaneously** |
| Rollback is trivial — keep shipping Electron | Every feature must be written twice for the duration |
| Lower per-step risk | The datastore cannot be split: NeDB and SQLite would need bidirectional sync |
| Team learns Rust gradually | Bug fixes land twice; the two builds drift |
| | The renderer must abstract over *both* IPC mechanisms — a compatibility shim that is pure throwaway work |
| | Realistically the longest calendar time and the highest total effort |

The fatal detail: there is no meaningful "half-migrated" state. Running Tauri at all requires the
Rust backend to already serve settings, database, windows, and IPC. Phasing gets you a
compatibility layer, not a partial migration.

### Option B — Big Bang

Build the Tauri application to feature parity on a clean tree, migrate user data once, and cut
over in a single release. Incomplete subsystems ship behind feature flags rather than blocking
the cutover.

| Pros | Cons |
|------|------|
| One backend, one IPC surface, no compatibility shim | **All-or-nothing** — no partial value until cutover |
| Clean architecture, unconstrained by Electron's shape | Long period without a shippable Tauri build |
| Renderer ports mechanically in one coherent pass | Integration risk concentrates at the end |
| No double-implementation of features | Data migration must be correct on the first attempt |
| Legacy code (App.css, themes.css, preload) is simply dropped | Requires disciplined scope control to avoid drift |
| Matches how the code actually decomposes | |

### Option C — Hybrid

Tauri shell hosting the existing renderer largely unchanged, with a Node.js sidecar retaining the
Electron main-process logic; migrate Node → Rust incrementally behind the sidecar boundary.

| Pros | Cons |
|------|------|
| Reuses main-process JS immediately | **Bundles a Node runtime** — destroys the binary-size and memory rationale |
| Incremental Rust adoption | Adds a process-boundary IPC layer that is itself throwaway |
| Renderer barely changes | Sidecar lifecycle, crash recovery, and port management are new failure modes |
| | Node sidecars complicate code signing and notarisation on macOS |
| | Delivers Tauri's costs without Tauri's benefits |
| | Strong tendency to become permanent |

---

## Decision

**Adopt Option B — Big Bang.**

Slytube is built as a new Tauri v2 application. The Rust backend is written from scratch; the
Vue renderer is ported in a single coherent pass; user data is migrated once at first launch of
the new build. Subsystems that are not ready at cutover ship **disabled behind feature flags**
rather than delaying the release.

### Cutover sequence

| Phase | Scope | Gate |
|-------|-------|------|
| 1 | Tauri scaffold, config, window management, settings, SQLite (ADR 004) | App launches, settings persist |
| 2 | Video API — Invidious stays in renderer (ADR 002), `youtubei.js` retained | Search + metadata work |
| 3 | yt-dlp sidecar, download manager, progress events | Downloads complete on all 3 OSes |
| 4 | PoToken hidden webview (ADR 001) | Playback succeeds on gated videos |
| 5 | Sync engine in Rust (ADR 003) | Golden-vector + interop tests pass |
| 6 | Full renderer IPC migration — all stores on `invoke`/`listen` | No `window.api` references remain |
| 7 | Multi-window, tray, updater, signing, notarisation | Release candidate |

Phases are a **work-ordering device, not incremental releases**. There is exactly one cutover.

---

## Rationale

1. **Cleaner architecture.** Building on a clean tree lets the Rust backend be designed around
   Tauri's model — capability-scoped permissions, typed commands, event streams, async
   everywhere — instead of transliterating 4558 lines of Electron main-process code. Every
   other ADR in this directory (Rust crypto, `sqlx`, hidden-webview PoToken, shadcn defaults)
   depends on that freedom. Under Option A or C, each would have to be compatible with the
   Electron design it is meant to replace.

2. **A renderer-heavy app suits a rewrite.** Because the majority of the code lives in the
   renderer and ports mechanically, the "big bang" is far smaller in practice than it sounds.
   The genuinely new work is the Rust backend — and that has to be written in full regardless of
   approach. Phasing does not reduce that work; it adds a compatibility layer on top of it.

3. **No viable partial state exists.** The database, the IPC surface, and window management are
   all-or-nothing. NeDB and SQLite cannot both be authoritative. `window.api` and `invoke`
   cannot both be the contract without a shim. Option A's central promise — incremental
   production validation — is not actually available at the level that matters.

4. **Feature flags absorb the schedule risk.** The standard objection to a big bang is that any
   unfinished subsystem blocks the entire release. Feature flags dissolve this: sync, or
   multi-window, or the updater can ship dark and be enabled in a follow-up release. The cutover
   is gated on *core* parity — browse, watch, download, persist — not on *total* parity.

5. **Hybrid defeats the purpose.** A Node sidecar reintroduces the runtime whose removal is the
   entire justification for migrating. It converts a one-time rewrite into permanent
   architectural debt, and in practice such sidecars are never removed.

---

## Implications

### All-or-nothing delivery

- [ ] No Tauri build ships to users until core parity is reached — expect an extended
      internal-only period.
- [ ] Progress must be tracked against explicit, testable parity criteria per phase, not vibes.
      Define the gate for each phase before starting it.
- [ ] The Electron build is **frozen to critical fixes only** during the migration. Feature work
      on Electron directly increases the parity target and must be refused.
- [ ] Integration risk concentrates at the end. Mitigate by keeping the app *launchable* from
      Phase 1 onward and exercising the full stack continuously, even with stubbed subsystems.
- [ ] Rollback is "keep using the Electron release." That option must remain viable — do not
      remove Electron distribution channels until the Tauri build has soaked.

### One-time data migration

This is the single highest-consequence operation in the project. It runs once, on the user's
real data, without supervision.

- [ ] **Back up first.** Copy the entire NeDB directory to a timestamped backup before touching
      anything. Never delete originals in the release that migrates them.
- [ ] **Idempotent and resumable.** A crash mid-import must not corrupt or duplicate. Record
      completion in a `meta` row and check it on every startup.
- [ ] **Transactional.** Import runs inside a single SQLite transaction per collection (ADR 004).
- [ ] **Validated.** Assert post-import row counts against source document counts; log
      per-collection discrepancies and surface a clear failure state rather than proceeding.
- [ ] **Legacy sync data.** Records encrypted by the Electron client must remain decryptable —
      see ADR 003 §"Legacy decryption for migration".
- [ ] **Tested against real corpora.** Collect anonymised NeDB datasets of varying size and age
      (including pre-release schema variants) and run the importer against all of them in CI.
- [ ] **Recoverable.** Document a manual recovery path: restore the backup, reinstall Electron.

### Scope discipline

| Rule | Rationale |
|------|-----------|
| No new features during migration | Parity is a moving target otherwise |
| Legacy styling is dropped, not ported | See ADR 006 — porting 66K lines of CSS would dominate the schedule |
| Feature flags default **off** for anything not parity-gated | Unfinished code must not affect the cutover |
| Every deferred subsystem gets a tracked follow-up | Flags must not become permanent dead code |

### Team consequences

- Rust proficiency is required across the backend team from the start; there is no gradual
  ramp as Option A would have provided. Budget for pairing and review time on the crypto
  (ADR 003) and database (ADR 004) work specifically.
- The renderer port is broad but shallow — parallelisable across contributors once the command
  surface is defined. Define and freeze the `invoke` contract early so renderer work can proceed
  against typed stubs.

---

## References

- [../architecture/01-electron-vs-tauri.md](../architecture/01-electron-vs-tauri.md) — full comparison and phase ordering
- [../architecture/02-component-mapping.md](../architecture/02-component-mapping.md)
- [Tauri v2 — Migrating from Electron](https://v2.tauri.app/start/migrate/from-electron/)
- [../phases/OVERVIEW.md](../phases/OVERVIEW.md)

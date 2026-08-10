# Phases Overview

## Scope
This domain covers project phase documentation including milestones, deliverables, timelines, sprint plans, risk registers, and phase-specific requirements for the SlyTube migration (OpenTubeX/Electron → Tauri v2).

Each phase document follows a fixed structure: **Goals → Tasks → Deliverables → Dependencies → Risks → Estimated Duration → Exit Criteria**.

## File Index
- [OVERVIEW.md](OVERVIEW.md) - This file
- [CHANGELOG.md](CHANGELOG.md) - Revision history for phases docs
- [01-foundation.md](01-foundation.md) - Tauri setup, config hardening, plugins, sidecars, shadcn-vue verification
- [02-database-yt-dlp.md](02-database-yt-dlp.md) - sqlx/SQLite, migrations, NeDB import, yt-dlp sidecar, progress events
- [03-potoken-generator.md](03-potoken-generator.md) - Hidden webview, custom protocol, botGuard port, session cleanup ⚠️ highest risk
- [04-backend-commands.md](04-backend-commands.md) - Full Tauri command surface: CRUD, sync, network/proxy, window/tray
- [05-sync-encryption.md](05-sync-encryption.md) - Rust crypto, snapshot protocol, legacy decryption, privacy modes
- [06-frontend-migration.md](06-frontend-migration.md) - Vuex→Pinia, shadcn-vue components, 16 views, API integration
- [07-system-integration.md](07-system-integration.md) - Shortcuts, tray, menus, protocol handler, file associations, updater
- [08-testing-polish.md](08-testing-polish.md) - Vitest, Playwright, migration verification, profiling, platform testing

## Phase Timeline

| Phase | Weeks | Duration | Risk |
|-------|-------|----------|------|
| 01 Foundation | 1–2 | 10 d | 🟢 Low |
| 02 Database & yt-dlp | 2–3 | 10 d | 🟡 Medium |
| 03 PoToken Generator | 3–4 | 10 d | 🔴 **Critical** |
| 04 Backend Commands | 4–6 | 15 d | 🟡 Medium |
| 05 Sync & Encryption | 5–6 | 10 d | 🟠 Medium-High |
| 06 Frontend Migration | 6–14 | 45 d | 🟡 Medium (high volume) |
| 07 System Integration | 13–14 | 10 d | 🟠 Medium-High |
| 08 Testing & Polish | 14–15 | 10 d | 🟠 Medium-High |

**Total: 15 weeks.** Phases 04/05 and 06/07 overlap; Phase 06 assumes two frontend developers from week 8.

## Dependency Graph

```
01 Foundation
 ├─► 02 Database & yt-dlp ──┬─► 04 Backend Commands ──┬─► 06 Frontend Migration ──┐
 │                          │                          │                          │
 ├─► 03 PoToken ────────────┘                          │                          ├─► 08 Testing & Polish
 │                                                     │                          │
 └────────────────────────► 05 Sync & Encryption ──────┘   07 System Integration ─┘
```

## Critical Path
`01 → 02 → 04 → 06 → 08`. Phase 03 is off the critical path by design (fallback chain), but its failure degrades functionality. Phase 06 is the longest and most likely to slip.

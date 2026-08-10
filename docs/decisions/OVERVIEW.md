# Decisions Overview

## Scope
This domain covers Architectural Decision Records (ADRs) and technical decision documentation including context, alternatives considered, decisions made, and consequences for the Slytube project.

Each ADR follows a consistent structure:
**Context → Options Considered → Decision → Rationale → Implications**

## File Index
- [OVERVIEW.md](OVERVIEW.md) - This file
- [CHANGELOG.md](CHANGELOG.md) - Revision history for decisions docs
- [01-potoken-strategy.md](01-potoken-strategy.md) - PoToken generation via hidden Tauri webview
- [02-invidious-location.md](02-invidious-location.md) - Invidious API client stays in the renderer
- [03-sync-encryption.md](03-sync-encryption.md) - E2E sync crypto ported to Rust
- [04-database-choice.md](04-database-choice.md) - SQLite access layer via `sqlx`
- [05-migration-approach.md](05-migration-approach.md) - Big Bang Electron → Tauri migration
- [06-theme-strategy.md](06-theme-strategy.md) - Adopt the shadcn-vue default (New York) theme

## Decision Register

| ADR | Title | Decision | Status |
|-----|-------|----------|--------|
| 001 | PoToken Strategy | Hidden Tauri Webview (Option A) | Accepted |
| 002 | Invidious Location | Keep in Renderer (Option A) | Accepted |
| 003 | Sync Encryption | Port to Rust (Option A) | Accepted |
| 004 | Database Choice | `sqlx` (Option A) | Accepted |
| 005 | Migration Approach | Big Bang (Option B) | Accepted |
| 006 | Theme Strategy | shadcn-vue default / New York (Option A) | Accepted |

## Cross-Cutting Notes
- ADR 005 (Big Bang) is the umbrella decision; ADRs 001, 003, 004, and 006 all depend on the
  freedom a clean-tree rewrite provides.
- ADRs 002 and 003 make deliberately opposite calls on renderer-vs-Rust placement. The
  distinguishing factor is the value of the secrets involved and the coupling of the surrounding
  logic — see the rationale sections of each.
- ADR 001 depends on ADR 002 for its fallback path: PoToken failures degrade to the Invidious
  route rather than surfacing an error.

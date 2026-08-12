# Changelog

| Date | Version | Description | Author |
|------|---------|-------------|--------|
| 2026-08-09 | 1.0.0 | Initial creation | Migration Team |
| 2026-08-10 | 1.1.0 | Added `01-store-migration.md` — Vuex → Pinia strategy, 14 module mapping, setup vs options decision, TypeScript conventions, commit/dispatch replacement rules, migration waves | Migration Team |
| 2026-08-10 | 1.1.0 | Added `02-shadcn-components.md` — 18 `Ft*` primitive replacements, 9 kept domain components, New York theme + CSS variable token set, per-component usage patterns | Migration Team |
| 2026-08-10 | 1.1.0 | Added `03-view-migration-order.md` — priority order for 16 views, dependency graph, 51 IED effort breakdown, 7-week 3-engineer schedule, milestones | Migration Team |
| 2026-08-10 | 1.1.0 | Added `04-api-integration.md` — youtubei.js in webview, `proxy_fetch` header wrapper, Invidious client, local→Invidious fallback chain, SponsorBlock client, PoToken lifecycle | Migration Team |
| 2026-08-10 | 1.1.0 | Updated `OVERVIEW.md` — file index, reading order, key decisions table, related domains | Migration Team |
| 2026-08-12 | 1.2.0 | Navigation restructure — YouTube-like sidebar with Shorts, Posts, Subscriptions in Library; "For You" curated feed; Phosphor icons; settings sidebar layout | SlyTube Team |
| 2026-08-12 | 1.2.0 | Database integration — subscriptions, history, playlists wired to Tauri SQLite backend | SlyTube Team |
| 2026-08-12 | 1.2.0 | UI improvements — player shortcuts, progressive disclosure, sync status, optimistic updates, command palette, scroll animations, breadcrumbs | SlyTube Team |
| 2026-08-12 | 1.3.0 | Updated `01-store-migration.md` — noted serde `camelCase` status per model (most done; `Profile` pending); clarified that frontend `DbPlaylist` already matches Rust model | Docs Update |
| 2026-08-12 | 1.4.0 | Updated `04-api-integration.md` — added §4.2 Channel tabs (Shorts/Live/Playlists/Community data sources and APIs); added §4.3 Comments (`useComments` composable → `invidiousGetComments`); updated §4.4 Normalisation contract | Docs Update |

## Pending Actions

| Item | Owner | Blocking |
|------|-------|----------|
| Change `components.json` `style` from `reka-nova` → `new-york` and `baseColor` from `mist` → `neutral` | Frontend | First `shadcn-vue add` — style is baked into generated files |
| Record any manual edits to `src/components/ui/*` here | Frontend | Prevents silent reverts on regeneration |

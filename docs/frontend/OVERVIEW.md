# Frontend Overview

## Scope
This domain covers frontend implementation including UI components, state management, routing, styling, build configuration, API integration, and user experience design for the Slytube project.

The frontend is **Vue 3 + TypeScript + Pinia + shadcn-vue (Reka UI + Tailwind v4)** running inside a Tauri v2 webview. It is migrated from the legacy Electron renderer (Vue 2/3 + Vuex + hand-rolled `Ft*` components + SCSS).

## File Index
- [OVERVIEW.md](OVERVIEW.md) - This file
- [CHANGELOG.md](CHANGELOG.md) - Revision history for frontend docs
- [01-store-migration.md](01-store-migration.md) - Vuex → Pinia migration: 14 module mapping, setup syntax, TypeScript conventions, replacing commit/dispatch
- [02-shadcn-components.md](02-shadcn-components.md) - `Ft*` → shadcn-vue primitive mapping, kept domain components, New York theme configuration
- [03-view-migration-order.md](03-view-migration-order.md) - Priority order for 16 views, dependency graph, effort estimates, schedule
- [04-api-integration.md](04-api-integration.md) - youtubei.js in webview, custom fetch wrapper, Invidious client, fallback chain, SponsorBlock, PoToken

## Reading Order

New contributors should read in numeric order — each document builds on the previous:

1. **01** establishes the state layer every component and view binds to.
2. **02** establishes the component vocabulary every view is built from.
3. **03** sequences the work using 01 and 02 as prerequisites.
4. **04** defines the data layer the Watch/Channel/Search views consume.

## Key Decisions

| Decision | Choice | Document |
|----------|--------|----------|
| State management | Pinia, setup syntax only (no options syntax) | 01 |
| Store count | 14 stores, 1:1 with legacy Vuex modules | 01 |
| Settings typing | Single typed record + generic `updateSetting<K>` | 01 |
| Component library | shadcn-vue, **New York** style, `neutral` base | 02 |
| Icons | Hugeicons (`@hugeicons/vue`) | 02 |
| Theming | CSS variables on `[data-theme]`, 5 themes | 02 |
| Kept components | 9 domain-specific (player, list cards, nav shell) | 02 |
| First view | Settings (unlocks all form primitives) | 03 |
| Highest-risk view | Watch (Shaka in webview, 3 platforms) | 03 |
| YouTube API location | Renderer, not Rust (`youtubei.js` unchanged) | 04 |
| Forbidden headers | Rust `proxy_fetch` command via `reqwest` | 04 |
| Backend fallback | local → Invidious, error-code gated | 04 |
| PoToken | Rust hidden webview via `get_potoken` | 04 |

## Related Domains
- [../architecture/](../architecture/) - Electron → Tauri component mapping and data flow
- [../backend/](../backend/) - Tauri command signatures and database schema
- [../phases/](../phases/) - Milestone and phase planning

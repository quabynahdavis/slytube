# Architecture Overview

## Scope

This domain covers system architecture, high-level design patterns, component interactions, data-flow diagrams, and infrastructure decisions for the **Slytube** project — the Tauri v2 migration of the Electron-based **OpenTubeX** application.

All documents in this domain are grounded in the actual OpenTubeX source tree at `/home/davisville/Contributions/opentubex/src/**` and the Slytube scaffold at `/home/davisville/Contributions/slytube/{src,src-tauri}/**`. Line counts, file paths, and channel/action inventories are measured, not estimated.

## File Index

| File | Purpose |
|---|---|
| [OVERVIEW.md](OVERVIEW.md) | This file — domain scope and index |
| [CHANGELOG.md](CHANGELOG.md) | Revision history for architecture docs |
| [01-electron-vs-tauri.md](01-electron-vs-tauri.md) | Process model, IPC, security, and performance comparison; per-component migration implications and risk register |
| [02-component-mapping.md](02-component-mapping.md) | Exhaustive OpenTubeX → Slytube mapping: main process, preload, datastores, yt-dlp, PoToken, Vuex → Pinia, components → shadcn-vue |
| [03-data-flow.md](03-data-flow.md) | Six end-to-end data flows: settings, video info, downloads, sync, PoToken, tabs — with sequence diagrams and parity checklists |

## Key Measured Facts

| Metric | Value | Source |
|---|---:|---|
| Main process | 4,558 lines | `src/main/index.js` |
| Tab manager | 3,046 lines | `src/main/tabs/TabManager.js` |
| yt-dlp engine | 1,375 lines | `src/main/ytDlp.js` |
| Preload bridge | 1,023 lines, 70 members | `src/preload/interface.js` |
| PoToken generator | 219 lines | `src/main/poTokenGenerator.js` |
| IPC channels | 110 unique (277 references) | `src/constants.js` → `IpcChannels` |
| DB actions | 27 leaf actions, 6 groups | `src/constants.js` → `DBActions` |
| Sync events | 22 leaf events, 6 groups | `src/constants.js` → `SyncEvents` |
| `ipcMain` registrations | 52 across 50 channels | `src/main/index.js` |
| NeDB datastores | 8 | `src/datastores/index.js` |
| Vuex modules | 14 (5,582 lines) | `src/renderer/store/modules/` |
| Vue components | 118 files | `src/renderer/components/` |

## Reading Order

1. **[01-electron-vs-tauri.md](01-electron-vs-tauri.md)** — why the migration is shaped the way it is, and what the gating risks are.
2. **[02-component-mapping.md](02-component-mapping.md)** — what moves where, file by file.
3. **[03-data-flow.md](03-data-flow.md)** — how the pieces talk at runtime.

## Related Domains

- [`../backend/`](../backend/) — SQLite schema (`sqlx`) and the Tauri command reference
- [`../frontend/`](../frontend/) — Vue 3, Pinia, and shadcn-vue implementation detail
- [`../phases/`](../phases/) — phased migration plan and milestones
- [`../decisions/`](../decisions/) — architectural decision records

## Open Decisions Raised Here

These are flagged in the documents above and must be resolved in [`../decisions/`](../decisions/):

1. **PoToken emulation strategy** — hidden webview + JS shim vs. Node sidecar fallback (highest risk).
2. **Tab architecture** — single-window virtual tabs (design A) vs. multi-webview (design B).
3. **yt-dlp sidecar distribution** — bundled-only vs. runtime-managed binaries, given macOS signing.
4. **Content Security Policy** — `tauri.conf.json` currently ships `"csp": null`; an explicit policy is required before loading remote content.

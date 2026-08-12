# Changelog

| Date | Version | Description | Author |
|------|---------|-------------|--------|
| 2026-08-10 | 1.1.0 | Rewrote `01-electron-vs-tauri.md`, `02-component-mapping.md`, and `03-data-flow.md` against measured OpenTubeX source facts (110 IPC channels, 27 DBActions, 22 SyncEvents, 52 `ipcMain` registrations, 8 NeDB stores, 14 Vuex modules, 118 components). Added per-component risk register, exhaustive 70-member preload mapping, `DBActions` enumeration, yt-dlp port table with source anchors, PoToken parity-gap analysis with fallback ladder, tab-system design options, and six annotated data flows with parity checklists. Expanded `OVERVIEW.md` with a measured-facts table, reading order, and open decisions. | Migration Team |
| 2026-08-09 | 1.0.0 | Initial creation | Migration Team |
| 2026-08-12 | 1.2.0 | Updated `03-data-flow.md` — rewrote the download flow (§3.2) to match the Phase 1 implementation: `yt_dlp_download` returns `u64`, progress via `yt-dlp-*` events, `tokio::process::Command` instead of Tauri sidecar, `download_records` table; added §2.5 documenting Watch.vue history recording on load; updated flow summary table | Docs Update |
| 2026-08-12 | 1.3.0 | Updated `02-component-mapping.md` — added TabBar.vue to component mapping as removed (orphaned, imported by nothing); noted `Channel` type now has optional `playlists` and `relatedPlaylists` fields | Docs Update |

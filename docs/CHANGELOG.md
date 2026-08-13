# Changelog

| Date | Version | Description | Author |
|------|---------|-------------|--------|
| 2026-08-09 | 1.0.0 | Initial creation | Migration Team |
| 2026-08-10 | 1.1.0 | Indexed decisions domain with ADR 001-006 | Migration Team |
| 2026-08-10 | 2.0.0 | Phase 1 (Foundation) complete: Tauri scaffold hardened, plugins registered, sidecar pipeline, shadcn-vue baseline | Migration Team |
| 2026-08-10 | 2.1.0 | Phase 2 (Database + yt-dlp) complete: sqlx/SQLite, migrations, NeDB importer, yt-dlp sidecar service, progress events | Migration Team |
| 2026-08-10 | 2.2.0 | Phase 3 (PoToken) complete: hidden webview, custom protocol, botGuard port, session cleanup, fallback chain | Migration Team |
| 2026-08-10 | 2.3.0 | Phase 4 (Backend commands) complete: ~78 Tauri commands, TS bindings, network/proxy, window/tray primitives | Migration Team |
| 2026-08-10 | 2.4.0 | Phase 5 (Sync encryption) complete: Rust crypto, snapshot protocol, legacy decryption, privacy modes | Migration Team |
| 2026-08-10 | 2.5.0 | Phase 6 (Frontend migration) complete: Vuex→Pinia, shadcn-vue components, 16 views, API integration | Migration Team |
| 2026-08-10 | 2.6.0 | Phase 7 (System integration) complete: shortcuts, tray, menus, protocol handler, file associations, updater | Migration Team |
| 2026-08-10 | 2.7.0 | Phase 8 (Testing & Polish) in progress: Vitest, Playwright, migration verification, profiling, platform testing | Migration Team || 2026-08-10 | 3.0.0 | Phase 1-7 implementation complete, all builds pass | Migration Team |
| 2026-08-10 | 3.1.0 | App wired up: router, layout, views connected | Migration Team |
| 2026-08-10 | 4.0.0 | Real data: youtubei.js + Invidious + SponsorBlock integrated | Migration Team |
| 2026-08-10 | 4.1.0 | UI components: VideoCard, EmptyState, ErrorState, Skeleton | Migration Team |
| 2026-08-10 | 4.2.0 | Views wired: Home, Search, Watch with real API data | Migration Team |
| 2026-08-11 | 5.0.0 | Tauri fully wired: commands, events, system tray, shortcuts | Migration Team |
| 2026-08-11 | 5.1.0 | UI consistency: native selects replaced with shadcn-vue Select | Migration Team |
| 2026-08-11 | 6.0.0 | Shaka Player integration for DASH playback | Migration Team |
| 2026-08-11 | 6.1.0 | i18n scaffolding with vue-i18n (en-US) | Migration Team |
| 2026-08-11 | 6.2.0 | Keyboard shortcuts, settings import/export | Migration Team |
| 2026-08-11 | 6.3.0 | Fix i18n locale loading, add ErrorBoundary | Migration Team |
| 2026-08-11 | 6.4.0 | Settings sub-pages wired with components | Migration Team |
| 2026-08-11 | 6.5.0 | Rust HTTP client for YouTube/Invidious (CORS-free) | Migration Team |
| 2026-08-11 | 6.6.0 | Fix settings child route rendering | Migration Team |
| 2026-08-11 | 6.7.0 | Fix trending: use Invidious as primary source | Migration Team |
| 2026-08-11 | 6.8.0 | Settings: 2-level hub & spoke UI with search, quick access | Migration Team |
| 2026-08-11 | 6.9.0 | Settings: SQLite persistence for all settings | Migration Team |
| 2026-08-11 | 6.10.0 | Tauri config: rebrand from OpenTubeX to Slytube | Migration Team |
| 2026-08-11 | 6.11.0 | Capabilities: main.json and potoken.json security scopes | Migration Team |
| 2026-08-11 | 7.0.0 | API: port all 28 OpenTubeX Invidious endpoints to Rust | Migration Team |
| 2026-08-11 | 7.1.0 | API: port all 10 OpenTubeX InnerTube endpoints to Rust | Migration Team |
| 2026-08-11 | 7.2.0 | API: multi-instance fallback for Invidious | Migration Team |
| 2026-08-11 | 7.3.0 | Playback: multi-layered DASH/format fallback chain | Migration Team |
| 2026-08-11 | 8.0.0 | Sync: wire frontend store to Rust sync commands | Migration Team |
| 2026-08-11 | 8.1.0 | Tests: Vitest + 35 unit tests for settings, sync, search | Migration Team |
| 2026-08-11 | 8.2.0 | Fix: move Invidious calls to frontend webview (instances block non-browser clients) | Migration Team |
| 2026-08-11 | 8.3.0 | Fix: trending and homepage now load via frontend Invidious client | Migration Team |
| 2026-08-12 | 9.0.0 | Comments section — loaded via `useComments` composable using `invidiousGetComments`; displays author avatar, content, like count, reply count | SlyTube Team |
| 2026-08-12 | 9.1.0 | Channel tabs — Shorts tab uses `invidiousGetChannelShorts`; Live tab filters live videos from channel info; Playlists tab shows playlists from channel info response; Community tab uses `invidiousGetChannelCommunityPosts`; each tab has loading skeletons and empty states; tab data loads on demand via `watch(activeTab, ...)` | SlyTube Team |
| 2026-08-12 | 9.2.0 | Theme unification — localStorage is single source of truth for applied theme; settings store syncs `baseTheme` from localStorage on load; theme shortcut updates both `useTheme` and settings store | SlyTube Team |
| 2026-08-12 | 9.3.0 | Cleanup — removed orphaned `TabBar.vue` component; removed template `greet` command from `lib.rs`; `Channel` type now has optional `playlists` and `relatedPlaylists` fields | SlyTube Team |
| 2026-08-12 | 9.4.0 | Docs: updated `06-theme-strategy.md`, `04-api-integration.md`, `02-component-mapping.md`; updated CHANGELOGs across decisions, frontend, architecture, and root domains | Docs Update |
| 2026-08-13 | 10.0.0 | Extraction pivot — hidden webview youtubei.js replaces direct Rust InnerTube HTTP; new `extractor/` Rust module with request-ID correlation; ported parsers from OpenTubeX; frontend api/index.ts reordered to extractor → Invidious fallback; ADR 007 accepted | Migration Team |
| 2026-08-13 | 10.1.0 | Wired channel sub-tabs (videos/shorts/live/community/playlists), trending (FEtrending browseId), hashtag, community post, comment replies; added 07-extractor-bridge.md doc | Migration Team |
| 2026-08-13 | 11.0.0 | **Phase 2 complete**: Sync encryption + client fully ported from OpenTubeX. Crypto envelope (PBKDF2 600k, AES-GCM, gzip, 64KiB padding). Sync client with health/auth/manifest/collection endpoints. Deletion-aware merge with data-loss guard. 50 total tests passing. | Migration Team |

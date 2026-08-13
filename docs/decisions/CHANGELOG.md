# Changelog

| Date | Version | Description | Author |
|------|---------|-------------|--------|
| 2026-08-09 | 1.0.0 | Initial creation | Migration Team |
| 2026-08-10 | 1.1.0 | Added ADR 001 (PoToken strategy), 002 (Invidious location), 003 (sync encryption), 004 (database choice), 005 (migration approach), 006 (theme strategy); expanded OVERVIEW with decision register | Migration Team |
| 2026-08-10 | 1.2.0 | All 6 ADRs marked as accepted following implementation validation across Phases 01–07 | Migration Team |
| 2026-08-12 | 1.3.0 | Updated `06-theme-strategy.md` — added localStorage as single source of truth for applied theme; settings store syncs `baseTheme` from localStorage on load; theme shortcut updates both `useTheme` and settings store | Docs Update |
| 2026-08-13 | 1.4.0 | Added ADR 007 (extraction strategy) — hidden webview youtubei.js pivot; replaced direct InnerTube HTTP in Rust with extractor bridge pattern | Migration Team |

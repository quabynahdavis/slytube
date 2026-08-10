# Backend Overview

## Scope

This domain covers the Rust/Tauri backend of SlyTube: persistence, the IPC command surface, external process integration, token generation, end-to-end encrypted sync, and the network layer. It is the authoritative reference for everything under `src-tauri/src/`.

The backend owns all privileged operations. The renderer holds no database handle, spawns no processes, and makes no direct network requests — every such capability is exposed as an explicit, validated Tauri command.

## Architecture at a glance

```
Vue renderer
     │ invoke / listen
┌────▼──────────────────────────────────────────────────────┐
│ Tauri command layer  (commands/)                          │
├───────────────────────────────────────────────────────────┤
│ db/       sqlx + SQLite        → 01, 02                   │
│ ytdlp/    sidecar process      → 03                       │
│ potoken/  hidden webview       → 04                       │
│ sync/     aes-gcm + x25519     → 05                       │
│ net/      reqwest + protocols  → 06                       │
└───────────────────────────────────────────────────────────┘
```

## File Index

| File | Contents |
|------|----------|
| [OVERVIEW.md](OVERVIEW.md) | This file — domain scope and index |
| [CHANGELOG.md](CHANGELOG.md) | Revision history for backend docs |
| [01-database-schema.md](01-database-schema.md) | NeDB → SQLite migration strategy, all table schemas, indexes, foreign keys, migration files, sqlx setup |
| [02-tauri-commands.md](02-tauri-commands.md) | Complete Tauri command reference by module: `db/*`, `sync`, `network`, `window`, `shortcuts` |
| [03-yt-dlp-sidecar.md](03-yt-dlp-sidecar.md) | Per-platform sidecar config, download/cancel/list/info commands, progress events, record persistence, `DENIED_CUSTOM_ARGS`, format & codec selection |
| [04-potoken-generation.md](04-potoken-generation.md) | Hidden `WebviewWindow` approach, `potoken://` protocol, `botGuardScript.js`, `generate_po_token`, session cleanup, proxy support |
| [05-sync-encryption.md](05-sync-encryption.md) | Crypto stack, key derivation, X25519 pairing, snapshot protocol, the seven collections, legacy decryption, privacy modes |
| [06-network-proxy.md](06-network-proxy.md) | `reqwest` client registry, per-request proxy, renderer fetch wrapper, image cache strategy, CORS handling |

## Reading order

1. **01** — the data model everything else operates on.
2. **02** — the IPC contract the frontend consumes.
3. **03**, **04**, **06** — external integrations, in rough order of independence.
4. **05** — sync, which builds on the schema in 01 and the commands in 02.

## Cross-domain references

- [../architecture/02-component-mapping.md](../architecture/02-component-mapping.md) — Electron → Tauri equivalents
- [../architecture/03-data-flow.md](../architecture/03-data-flow.md) — end-to-end IPC and state flow

## Status

All six documents are **design specifications** written ahead of implementation. The `src-tauri/src/` tree currently contains only the Tauri scaffold; module paths referenced throughout (`src-tauri/src/db/`, `src-tauri/src/net/`, …) are implementation targets, not existing files.

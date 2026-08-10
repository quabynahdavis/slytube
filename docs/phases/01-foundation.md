# Phase 01 — Foundation

| Field | Value |
|-------|-------|
| **Timeline** | Week 1 – Week 2 |
| **Duration** | 10 working days |
| **Risk Level** | 🟢 Low |
| **Blocks** | Phases 02, 03, 04, 07 |
| **Depends On** | — (entry phase) |

---

## Status

**Status:** Complete ✅
**Completed:** 2026-08-10
**Notes:** All deliverables met. Tauri scaffold hardened with explicit CSP, 11 plugins registered, sidecar pipeline operational, shadcn-vue baseline verified, CI green on 3 platforms.

---

## 1. Goals

1. Convert the bare `slytube` Tauri scaffold into a production-shaped application shell that can host the full OpenTubeX feature set.
2. Lock down the Tauri v2 configuration surface: identifier, product metadata, window defaults, icons, CSP, and capability files.
3. Install and register **every** Tauri plugin required by later phases so no phase is blocked on a missing dependency.
4. Wire the `yt-dlp` sidecar contract (binary naming, bundling, capability scopes) even before download logic exists.
5. Verify the shadcn-vue + Tailwind v4 + Reka UI design system renders correctly and is ready for the 16-view migration in Phase 06.
6. Establish repository hygiene: scripts, path aliases, lint/format gates, CI skeleton.

**Non-goals:** No database, no commands beyond a smoke-test `greet`, no UI views. Those belong to Phases 02+.

---

## 2. Prerequisites

| Item | Version | Verification |
|------|---------|--------------|
| Rust toolchain | ≥ 1.77 (stable) | `rustc --version` |
| Node runtime | Bun ≥ 1.1 | `bun --version` |
| Tauri CLI | v2.x | `bun run tauri --version` |
| Platform deps (Linux) | `webkit2gtk-4.1`, `libayatana-appindicator3`, `librsvg2` | `pkg-config --exists webkit2gtk-4.1` |
| Platform deps (macOS) | Xcode CLT | `xcode-select -p` |
| Platform deps (Windows) | MSVC Build Tools + WebView2 | `where cl.exe` |
| OpenTubeX source | Reference checkout for parity | read-only |

---

## 3. Tasks

### 3.1 Project Baseline Audit (Day 1)

- [ ] Inventory the existing scaffold: `src-tauri/src/{main.rs,lib.rs}`, `src/App.vue`, `vite.config.ts`, `components.json`.
- [ ] Confirm dev loop works end-to-end: `bun run tauri dev` opens a window and hot-reloads.
- [ ] Record baseline metrics for later comparison in Phase 08.

| Baseline Metric | Target to record |
|-----------------|------------------|
| Cold start (dev) | seconds |
| Release binary size | MB |
| RSS at idle | MB |
| `cargo build --release` time | seconds |

### 3.2 `tauri.conf.json` Hardening (Day 1–2)

Replace the scaffold config with the production shape:

```jsonc
{
  "$schema": "https://schema.tauri.app/config/2",
  "productName": "SlyTube",
  "version": "0.1.0",
  "identifier": "com.davisville.slytube",
  "build": {
    "beforeDevCommand": "bun run dev",
    "devUrl": "http://localhost:1420",
    "beforeBuildCommand": "bun run build",
    "frontendDist": "../dist"
  },
  "app": {
    "windows": [
      {
        "label": "main",
        "title": "SlyTube",
        "width": 1280,
        "height": 800,
        "minWidth": 940,
        "minHeight": 600,
        "resizable": true,
        "center": true,
        "decorations": true,
        "visible": false,          // shown after frontend 'app-ready' event
        "dragDropEnabled": true
      }
    ],
    "security": {
      "csp": {
        "default-src": "'self'",
        "img-src": "'self' asset: http://asset.localhost https: data:",
        "media-src": "'self' asset: http://asset.localhost https: blob:",
        "connect-src": "'self' ipc: http://ipc.localhost https:",
        "script-src": "'self'",
        "style-src": "'self' 'unsafe-inline'"
      },
      "assetProtocol": { "enable": true, "scope": ["$VIDEO/**", "$DOWNLOAD/**", "$APPDATA/**"] }
    },
    "trayIcon": {
      "id": "main-tray",
      "iconPath": "icons/tray.png",
      "iconAsTemplate": true,
      "menuOnLeftClick": false
    }
  },
  "bundle": {
    "active": true,
    "targets": ["deb", "rpm", "appimage", "nsis", "msi", "app", "dmg"],
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ],
    "externalBin": ["binaries/yt-dlp", "binaries/ffmpeg"],
    "resources": ["resources/botGuardScript.js"],
    "shortDescription": "Privacy-focused YouTube client",
    "longDescription": "SlyTube is a privacy-focused desktop YouTube client with downloads, sync and proxy support.",
    "category": "Video",
    "copyright": "© 2026 SlyTube contributors",
    "createUpdaterArtifacts": true
  },
  "plugins": {
    "updater": {
      "endpoints": ["https://releases.slytube.app/{{target}}/{{arch}}/{{current_version}}"],
      "pubkey": "<REPLACE_WITH_MINISIGN_PUBKEY>",
      "windows": { "installMode": "passive" }
    },
    "deep-link": {
      "desktop": { "schemes": ["opentubex", "slytube"] }
    }
  }
}
```

**Checklist**

- [ ] `identifier` set to `com.davisville.slytube` (must never change post-release — it keys the app data dir).
- [ ] `productName` normalized to `SlyTube` (affects bundle names + app data path).
- [ ] Window starts hidden and is revealed by the frontend to eliminate white-flash.
- [ ] CSP moved off `null` to an explicit allowlist.
- [ ] `assetProtocol` scoped to download/video directories only.

### 3.3 Icon Pipeline (Day 2)

- [ ] Source a 1024×1024 master PNG (`assets/branding/icon-master.png`).
- [ ] Generate the full set: `bun run tauri icon assets/branding/icon-master.png`.
- [ ] Add monochrome tray variants: `icons/tray.png` (macOS template, 22×22@2x) and `icons/tray-light.png` / `icons/tray-dark.png` for Linux/Windows.
- [ ] Verify `.ico` contains 16/32/48/256 layers and `.icns` contains all required slices.

### 3.4 Plugin Installation & Registration (Day 3–5)

Install crates and JS bindings for the 11 mandated plugins:

```bash
# Rust side
cargo add tauri-plugin-fs tauri-plugin-dialog tauri-plugin-shell tauri-plugin-opener \
          tauri-plugin-clipboard-manager tauri-plugin-store tauri-plugin-http \
          --manifest-path src-tauri/Cargo.toml
cargo add tauri-plugin-global-shortcut tauri-plugin-updater \
          --manifest-path src-tauri/Cargo.toml   # desktop-only, gate with cfg

# JS side
bun add @tauri-apps/plugin-fs @tauri-apps/plugin-dialog @tauri-apps/plugin-shell \
        @tauri-apps/plugin-opener @tauri-apps/plugin-clipboard-manager \
        @tauri-apps/plugin-global-shortcut @tauri-apps/plugin-updater \
        @tauri-apps/plugin-store @tauri-apps/plugin-http
```

> `tray-icon` and `menu` are **built into `tauri` v2 core** (`tauri::tray`, `tauri::menu`) — no separate crate. They are enabled via the `tray-icon` Cargo feature and configured in Phase 07.

| # | Plugin | Crate / Module | Consumed By | Notes |
|---|--------|----------------|-------------|-------|
| 1 | fs | `tauri-plugin-fs` | 02, 04, 08 | Scoped to `$APPDATA`, `$DOWNLOAD`, `$VIDEO` |
| 2 | dialog | `tauri-plugin-dialog` | 04, 06 | Folder picker, import/export, confirm modals |
| 3 | shell | `tauri-plugin-shell` | 02 | **Sidecar execution only** — `open` disabled |
| 4 | opener | `tauri-plugin-opener` | 04, 06 | External URLs + reveal-in-folder |
| 5 | clipboard-manager | `tauri-plugin-clipboard-manager` | 06 | Copy video/channel links |
| 6 | global-shortcut | `tauri-plugin-global-shortcut` | 07 | Media keys, show/hide |
| 7 | updater | `tauri-plugin-updater` | 07 | Minisign-signed artifacts |
| 8 | store | `tauri-plugin-store` | 04, 06 | Window geometry + ephemeral UI prefs (**not** app settings — those live in SQLite) |
| 9 | tray-icon | `tauri::tray` (core feature) | 07 | Requires `features = ["tray-icon"]` |
| 10 | menu | `tauri::menu` (core) | 07 | App menu + tray menu + context menus |
| 11 | http | `tauri-plugin-http` | 04, 05 | Frontend-side fetch with proxy scope |

**`Cargo.toml` target shape:**

```toml
[dependencies]
tauri = { version = "2", features = ["tray-icon", "image-png", "protocol-asset", "devtools"] }
tauri-plugin-fs = "2"
tauri-plugin-dialog = "2"
tauri-plugin-shell = "2"
tauri-plugin-opener = "2"
tauri-plugin-clipboard-manager = "2"
tauri-plugin-store = "2"
tauri-plugin-http = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
tokio = { version = "1", features = ["full"] }
log = "0.4"

[target.'cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))'.dependencies]
tauri-plugin-global-shortcut = "2"
tauri-plugin-updater = "2"
```

**`lib.rs` registration:**

```rust
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut builder = tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_http::init());

    #[cfg(desktop)]
    {
        builder = builder
            .plugin(tauri_plugin_global_shortcut::Builder::new().build())
            .plugin(tauri_plugin_updater::Builder::new().build())
            .plugin(tauri_plugin_single_instance::init(|app, argv, _cwd| {
                // Phase 07 wires deep-link forwarding here
                let _ = app;
                let _ = argv;
            }));
    }

    builder
        .invoke_handler(tauri::generate_handler![health_check])
        .run(tauri::generate_context!())
        .expect("error while running SlyTube");
}
```

### 3.5 Capabilities & Permission Scoping (Day 5–6)

Split `capabilities/default.json` into purpose-scoped files so the PoToken window (Phase 03) can be locked down independently.

`src-tauri/capabilities/main.json`:

```jsonc
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "main-window",
  "description": "Capabilities granted to the primary application window",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "core:window:allow-start-dragging",
    "core:window:allow-set-title",
    "opener:default",
    "dialog:default",
    "clipboard-manager:allow-write-text",
    "clipboard-manager:allow-read-text",
    "store:default",
    "updater:default",
    "global-shortcut:default",
    {
      "identifier": "fs:scope",
      "allow": [
        { "path": "$APPDATA/**" },
        { "path": "$APPLOCALDATA/**" },
        { "path": "$DOWNLOAD/**" },
        { "path": "$VIDEO/**" }
      ]
    },
    {
      "identifier": "shell:allow-execute",
      "allow": [
        { "name": "binaries/yt-dlp", "sidecar": true, "args": true },
        { "name": "binaries/ffmpeg", "sidecar": true, "args": true }
      ]
    },
    {
      "identifier": "http:default",
      "allow": [{ "url": "https://*" }],
      "deny": [{ "url": "http://localhost:*" }]
    }
  ]
}
```

`src-tauri/capabilities/potoken.json` (placeholder consumed in Phase 03):

```jsonc
{
  "identifier": "potoken-window",
  "description": "Minimal capabilities for the hidden PoToken webview",
  "windows": ["potoken-generator"],
  "permissions": ["core:event:allow-emit", "core:event:allow-listen"]
}
```

- [ ] Delete the permissive scaffold `default.json`.
- [ ] Confirm `shell:allow-open` is **not** granted anywhere (use `opener` instead).
- [ ] Run `bun run tauri dev` and confirm no permission warnings in console.

### 3.6 yt-dlp Sidecar Configuration (Day 6–8)

Tauri resolves sidecars by target-triple suffix. Create `src-tauri/binaries/` with:

```
src-tauri/binaries/
├── yt-dlp-x86_64-unknown-linux-gnu
├── yt-dlp-aarch64-unknown-linux-gnu
├── yt-dlp-x86_64-pc-windows-msvc.exe
├── yt-dlp-x86_64-apple-darwin
├── yt-dlp-aarch64-apple-darwin
├── ffmpeg-x86_64-unknown-linux-gnu
├── ffmpeg-x86_64-pc-windows-msvc.exe
├── ffmpeg-x86_64-apple-darwin
└── ffmpeg-aarch64-apple-darwin
```

Fetch script (`scripts/fetch-sidecars.ts`, run via `bun run sidecars`):

```ts
const TRIPLE = (await $`rustc -vV`.text())
  .split("\n").find(l => l.startsWith("host:"))!.split(" ")[1];

const EXT = TRIPLE.includes("windows") ? ".exe" : "";
const YTDLP_ASSET = {
  "x86_64-unknown-linux-gnu": "yt-dlp_linux",
  "aarch64-unknown-linux-gnu": "yt-dlp_linux_aarch64",
  "x86_64-pc-windows-msvc": "yt-dlp.exe",
  "x86_64-apple-darwin": "yt-dlp_macos",
  "aarch64-apple-darwin": "yt-dlp_macos",
}[TRIPLE];

// download → src-tauri/binaries/yt-dlp-${TRIPLE}${EXT} → chmod 0o755
```

**Smoke-test command (temporary, removed in Phase 02):**

```rust
#[tauri::command]
async fn health_check(app: tauri::AppHandle) -> Result<serde_json::Value, String> {
    use tauri_plugin_shell::ShellExt;
    let out = app.shell()
        .sidecar("yt-dlp").map_err(|e| e.to_string())?
        .args(["--version"])
        .output().await.map_err(|e| e.to_string())?;

    Ok(serde_json::json!({
        "ytdlp": String::from_utf8_lossy(&out.stdout).trim(),
        "app": app.package_info().version.to_string(),
        "tauri": tauri::VERSION,
    }))
}
```

**Checklist**

- [ ] `externalBin` entries use the path **without** triple suffix (`binaries/yt-dlp`).
- [ ] Binaries are `chmod +x` on Unix; macOS binaries pass `codesign -dv` after ad-hoc signing.
- [ ] `.gitignore` excludes `src-tauri/binaries/*` (fetched at build time, not committed).
- [ ] CI job downloads sidecars before `tauri build`.
- [ ] Fallback resolution path documented: if sidecar missing → probe `PATH` via `which yt-dlp`.

### 3.7 shadcn-vue Verification (Day 8–9)

`components.json` already targets the `reka-nova` style with `hugeicons`. Validate the full pipeline:

- [ ] Confirm Tailwind v4 is wired through `@tailwindcss/vite` in `vite.config.ts` (no `tailwind.config.js` needed).
- [ ] Confirm `src/style.css` contains `@import "tailwindcss";`, the `@theme` token block, and `tw-animate-css`.
- [ ] Confirm `@/*` path alias resolves in **both** `tsconfig.json` and `vite.config.ts`.
- [ ] Install the baseline component set required by Phase 06:

```bash
bunx shadcn-vue@latest add button input label card dialog sheet dropdown-menu \
  select tabs tooltip toast skeleton scroll-area separator switch slider \
  progress badge avatar checkbox popover command context-menu resizable
```

- [ ] Build a throwaway `src/views/DesignSystemProbe.vue` rendering every installed primitive; verify light/dark/system themes.
- [ ] Verify Hugeicons tree-shaking: production bundle must not include the full `@hugeicons/core-free-icons` barrel (check with `vite-bundle-visualizer`).
- [ ] Delete the probe view before the phase-closing commit (or gate it behind `import.meta.env.DEV`).

### 3.8 Tooling, Scripts & CI Skeleton (Day 9–10)

`package.json` scripts:

```jsonc
{
  "scripts": {
    "dev": "vite",
    "build": "vue-tsc --noEmit && vite build",
    "preview": "vite preview",
    "tauri": "tauri",
    "sidecars": "bun run scripts/fetch-sidecars.ts",
    "lint": "eslint . --ext .ts,.vue",
    "format": "prettier --write \"src/**/*.{ts,vue,css}\"",
    "rust:fmt": "cargo fmt --manifest-path src-tauri/Cargo.toml",
    "rust:lint": "cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings",
    "check": "bun run lint && bun run rust:lint && vue-tsc --noEmit"
  }
}
```

- [ ] Add ESLint (flat config) + Prettier + `eslint-plugin-vue`.
- [ ] Add `rustfmt.toml` and `clippy.toml`.
- [ ] Add `.github/workflows/ci.yml`: matrix over `ubuntu-22.04`, `windows-latest`, `macos-14`; steps = install deps → fetch sidecars → `bun run check` → `cargo test` → `tauri build --debug`.
- [ ] Add `CODEOWNERS` and a PR template referencing the phase docs.

---

## 4. Deliverables

| ID | Deliverable | Acceptance Criteria |
|----|-------------|---------------------|
| D1.1 | Hardened `tauri.conf.json` | Correct identifier, explicit CSP, hidden-then-shown main window, updater + deep-link blocks present |
| D1.2 | Complete icon set | `bun run tauri build` produces bundles with correct icons on all 3 OSes; tray variants present |
| D1.3 | 11 plugins registered | `health_check` runs; every plugin importable from JS without permission errors |
| D1.4 | Scoped capability files | `main.json` + `potoken.json`; no wildcard filesystem or shell-open grants |
| D1.5 | Sidecar pipeline | `bun run sidecars` fetches per-triple binaries; `health_check` returns a yt-dlp version string on all 3 OSes |
| D1.6 | shadcn-vue baseline | 22 primitives installed and rendering; theme switching functional; icons tree-shaken |
| D1.7 | CI green | 3-platform matrix passing `check` + debug build |
| D1.8 | Baseline metrics record | Committed to `docs/phases/` appendix for Phase 08 comparison |

---

## 5. Dependencies

**Inbound:** none — this is the entry phase.

**Outbound (what this phase unblocks):**

| Phase | Requires from Phase 01 |
|-------|------------------------|
| 02 — Database & yt-dlp | Sidecar pipeline, `fs` scope, app data dir resolution |
| 03 — PoToken | `potoken.json` capability, resource bundling for `botGuardScript.js` |
| 04 — Backend Commands | Plugin registry, `AppError` scaffolding, capability model |
| 06 — Frontend Migration | shadcn-vue baseline, path aliases, theme tokens |
| 07 — System Integration | `tray-icon` feature, `global-shortcut`, `updater`, deep-link scheme registration |

**External:** GitHub Releases availability for yt-dlp/ffmpeg; minisign keypair for the updater (generate now, store private key in CI secrets).

---

## 6. Risks

| ID | Risk | Prob. | Impact | Mitigation | Owner |
|----|------|-------|--------|------------|-------|
| R1.1 | Sidecar naming/triple mismatch causes silent runtime failure | Medium | High | `health_check` gate in CI on all 3 platforms; assert file exists in `build.rs` |
| R1.2 | macOS Gatekeeper blocks unsigned `yt-dlp` sidecar | High | High | Ad-hoc sign sidecars during bundling; add hardened-runtime entitlement exception; plan notarization in Phase 08 |
| R1.3 | Strict CSP breaks thumbnail/media loading later | Medium | Medium | Pre-allow `https:` + `asset:` in `img-src`/`media-src`; document any relaxation in `docs/decisions` |
| R1.4 | Tailwind v4 + shadcn-vue `reka-nova` style churn | Low | Medium | Pin exact versions in `package.json`; lockfile committed |
| R1.5 | Identifier change after first release orphans user data | Low | Critical | Freeze `com.davisville.slytube` now; add ADR in `docs/decisions` |
| R1.6 | Plugin version drift between Rust crate and JS binding | Medium | Medium | Pin both to same minor; add CI check comparing `Cargo.lock` vs `bun.lock` versions |
| R1.7 | Linux `webkit2gtk` version fragmentation | Medium | Medium | Target `4.1`; test on Ubuntu 22.04 + Fedora 40 + Arch; ship AppImage as fallback |
| R1.8 | Bundled ffmpeg licensing (GPL vs LGPL build) | Low | High | Use LGPL builds or document GPL compliance in `LICENSE-THIRD-PARTY` |

---

## 7. Estimated Duration

| Task | Days | Parallelizable |
|------|------|----------------|
| 3.1 Baseline audit | 0.5 | — |
| 3.2 `tauri.conf.json` hardening | 1.5 | — |
| 3.3 Icon pipeline | 0.5 | ✅ with 3.2 |
| 3.4 Plugin installation | 2.0 | — |
| 3.5 Capabilities scoping | 1.0 | ✅ with 3.6 |
| 3.6 Sidecar configuration | 2.5 | — |
| 3.7 shadcn-vue verification | 1.5 | ✅ with 3.4–3.6 |
| 3.8 Tooling & CI | 1.5 | ✅ with 3.7 |
| **Total (sequential)** | **11.0** | |
| **Total (with parallelism)** | **10.0** | 2 weeks @ 1 dev |

---

## 8. Exit Criteria

- [ ] `bun run tauri dev` launches with zero console warnings on Linux, macOS, Windows.
- [ ] `bun run tauri build` produces signed-shaped artifacts for all configured targets.
- [ ] `health_check` returns a valid yt-dlp version on all 3 platforms.
- [ ] `bun run check` passes (ESLint + clippy `-D warnings` + `vue-tsc`).
- [ ] All capability files reviewed; no over-broad grants.
- [ ] Baseline metrics recorded.
- [ ] Commit landed: `docs(phases): add foundation phase plan` + `feat(foundation): ...`

---

## 9. References

- [Architecture — Electron vs Tauri](../architecture/01-electron-vs-tauri.md)
- [Architecture — Component Mapping](../architecture/02-component-mapping.md)
- [Tauri v2 Capabilities](https://v2.tauri.app/security/capabilities/)
- [Tauri v2 Sidecar Guide](https://v2.tauri.app/develop/sidecar/)
- Next: [Phase 02 — Database & yt-dlp](02-database-yt-dlp.md)

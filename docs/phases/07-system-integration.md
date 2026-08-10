# Phase 07 — System Integration

| Field | Value |
|-------|-------|
| **Timeline** | Week 13 – Week 14 |
| **Duration** | 10 working days |
| **Risk Level** | 🟠 Medium-High (platform-divergent, packaging-coupled) |
| **Blocks** | Phase 08 (platform testing targets) |
| **Depends On** | Phase 01 (plugins, bundle config), Phase 04 (window/tray commands), Phase 06 (views to route into) |

---

## Status

**Status:** Complete ✅
**Completed:** 2026-08-10
**Notes:** All deliverables met. Global shortcuts with conflict detection, system tray with dynamic state, native menus per platform, `opentubex://` protocol handler with strict validation, file associations, signed auto-updater.

---

## 1. Goals

1. Make SlyTube behave like a **native desktop application** on Windows, macOS, and Linux — not a web page in a frame.
2. Register global shortcuts (media keys + app control) with conflict detection and user rebinding.
3. Ship a functional system tray with live download state and quick actions.
4. Provide native application menus with correct per-platform conventions.
5. Register the `opentubex://` (and `slytube://`) protocol handler for deep links, including cold-start and single-instance forwarding.
6. Register file associations for playlist/backup formats with correct open-with behavior.
7. Deliver a signed, staged, user-controllable auto-updater.

**Platform parity is not uniformity.** Each OS gets its idiomatic behavior; this document specifies the differences explicitly.

---

## 2. Tasks

### 2.1 Global Shortcuts (Day 1–2)

```rust
use tauri_plugin_global_shortcut::{Code, Modifiers, Shortcut, ShortcutState, GlobalShortcutExt};

pub fn register_defaults(app: &AppHandle) -> Result<(), AppError> {
    let bindings = load_bindings(app)?;   // user overrides ∪ defaults
    let gs = app.global_shortcut();

    for (action, accel) in bindings {
        let sc: Shortcut = accel.parse()?;
        if gs.is_registered(sc) { record_conflict(&action, &accel); continue; }
        gs.on_shortcut(sc, move |app, _sc, event| {
            if event.state() == ShortcutState::Pressed {
                dispatch_action(app, action);
            }
        })?;
    }
    Ok(())
}
```

**Default bindings:**

| Action | Windows / Linux | macOS | Scope |
|--------|-----------------|-------|-------|
| Play / Pause | `MediaPlayPause` | `MediaPlayPause` | Global |
| Next track | `MediaTrackNext` | `MediaTrackNext` | Global |
| Previous track | `MediaTrackPrevious` | `MediaTrackPrevious` | Global |
| Stop | `MediaStop` | `MediaStop` | Global |
| Show / hide window | `Ctrl+Alt+S` | `Cmd+Alt+S` | Global |
| Quick search (palette) | `Ctrl+Alt+F` | `Cmd+Alt+F` | Global |
| Toggle mini-player | `Ctrl+Alt+M` | `Cmd+Alt+M` | Global |
| Seek ±10 s, volume, fullscreen, `Ctrl+K` palette | — | — | **In-app only** (not global) |

**Requirements**

- [ ] Global registration is **opt-in per action**; media keys default on, others default off. Never squat on common system shortcuts.
- [ ] Conflict handling: registration failure is caught, logged, surfaced in Settings → Shortcuts as "unavailable (in use by another app)" — never a silent no-op.
- [ ] Rebinding UI: capture keystroke, validate, re-register atomically (unregister old → register new → rollback on failure).
- [ ] `unregister_all()` on app exit and before any bulk re-registration.
- [ ] Commands: `get_shortcuts`, `set_shortcut`, `reset_shortcuts`, `test_shortcut`.
- [ ] Platform notes: macOS may require Accessibility/Input-Monitoring permission for some keys — detect and prompt with a deep link to System Settings. Linux/Wayland global shortcuts are compositor-dependent; degrade gracefully and inform the user.
- [ ] Media keys should also participate in OS "now playing" where cheap: MPRIS on Linux (`zbus`), SMTC on Windows, `MPNowPlayingInfoCenter` on macOS. Mark as stretch; MPRIS first (highest value/lowest cost).

### 2.2 System Tray (Day 2–4)

```rust
use tauri::{tray::{TrayIconBuilder, TrayIconEvent, MouseButton, MouseButtonState}, menu::{Menu, MenuItem, PredefinedMenuItem, Submenu}};

pub fn build_tray(app: &AppHandle) -> Result<(), AppError> {
    let show     = MenuItem::with_id(app, "show",  "Show SlyTube", true, None::<&str>)?;
    let playpause= MenuItem::with_id(app, "pp",    "Play / Pause", true, None::<&str>)?;
    let downloads= MenuItem::with_id(app, "dl",    "Downloads…",   true, None::<&str>)?;
    let pause_all= MenuItem::with_id(app, "pall",  "Pause all downloads", true, None::<&str>)?;
    let settings = MenuItem::with_id(app, "set",   "Settings…",    true, None::<&str>)?;
    let quit     = MenuItem::with_id(app, "quit",  "Quit",         true, None::<&str>)?;

    let menu = Menu::with_items(app, &[
        &show, &PredefinedMenuItem::separator(app)?,
        &playpause, &PredefinedMenuItem::separator(app)?,
        &downloads, &pause_all, &PredefinedMenuItem::separator(app)?,
        &settings, &PredefinedMenuItem::separator(app)?,
        &quit,
    ])?;

    TrayIconBuilder::with_id("main-tray")
        .icon(tray_icon_for(TrayState::Idle))
        .icon_as_template(true)                 // macOS auto light/dark
        .tooltip("SlyTube")
        .menu(&menu)
        .show_menu_on_left_click(false)         // left-click toggles window
        .on_menu_event(handle_tray_menu)
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = event {
                toggle_main_window(tray.app_handle());
            }
        })
        .build(app)?;
    Ok(())
}
```

**Dynamic state**

| State | Icon | Tooltip |
|-------|------|---------|
| Idle | `tray.png` | `SlyTube` |
| Downloading | `tray-active.png` | `SlyTube — 3 downloads (47%)` |
| Error | `tray-error.png` | `SlyTube — 1 download failed` |
| Paused | `tray-paused.png` | `SlyTube — downloads paused` |

- [ ] Tooltip updates throttled to ≤1 Hz, driven by the download service.
- [ ] Menu item labels update dynamically (`Play` ↔ `Pause`, download count).
- [ ] `minimizeToTray` / `closeToTray` settings honored; when both off, the tray can be disabled entirely (`showTrayIcon` setting).
- [ ] Platform behavior:
  - **Windows** — left-click toggles, right-click menu, balloon notifications on completion.
  - **macOS** — template icon (auto light/dark), left-click opens the menu (platform convention), no Dock hiding unless `LSUIElement` is opted into.
  - **Linux** — requires `libayatana-appindicator3`; menu-only interaction (click events are unreliable on many DEs). Detect absence and degrade with a one-time notice.
- [ ] Never let the app become unquittable: `Quit` always performs a true exit (kills sidecars, closes pool, destroys PoToken window).

### 2.3 Native Menus (Day 4–5)

```
SlyTube (macOS app menu)     File            Edit         View              Playback        Window     Help
├ About SlyTube              ├ New Playlist  ├ Undo       ├ Reload          ├ Play/Pause    ├ Minimize ├ Documentation
├ Check for Updates…         ├ Open File…    ├ Redo       ├ Toggle Sidebar  ├ Next          ├ Zoom     ├ Keyboard Shortcuts
├ Settings…      ⌘,          ├ Import…       ├ Cut/Copy/  ├ Toggle Fullscr. ├ Previous      ├ Mini     ├ Report an Issue
├ Services                   ├ Export…       │  Paste/All ├ Zoom In/Out     ├ Volume Up/Dn  │  Player  ├ View Logs
├ Hide / Hide Others         ├ Close Window  ├ Find  ⌘F   ├ Actual Size     ├ Speed ▸       └ Bring    └ About (Win/Linux)
└ Quit           ⌘Q          └ Quit (W/L)    └ Preferences└ Developer Tools └ Subtitles ▸      All Front
```

- [ ] macOS: real app menu with `PredefinedMenuItem::{about, services, hide, hide_others, show_all, quit, separator}`; `Settings` under the app menu with `⌘,`.
- [ ] Windows/Linux: menu bar attached to the window (or fully hidden behind a hamburger if a custom title bar is used — pick one and be consistent).
- [ ] Menu events forwarded to the frontend as `menu-action` events; the frontend owns navigation.
- [ ] Enable/disable items reactively (e.g., `Play/Pause` disabled with no active media; `Export…` disabled with no playlist selected).
- [ ] Context menus (`ContextMenu` primitive, Phase 06) for video/playlist rows are frontend-owned — the native menu must not duplicate them.
- [ ] Developer Tools item only in debug builds.

### 2.4 Protocol Handler — `opentubex://` (Day 5–7)

```jsonc
// tauri.conf.json
"plugins": {
  "deep-link": { "desktop": { "schemes": ["opentubex", "slytube"] } }
}
```

**URL grammar:**

| URL | Action |
|-----|--------|
| `opentubex://watch/dQw4w9WgXcQ?t=42` | Open WatchView, seek to 42 s |
| `opentubex://channel/UCxxxx` | Open ChannelView |
| `opentubex://playlist/PLxxxx` | Open PlaylistView |
| `opentubex://search?q=rust+tutorial` | Open SearchView with the query |
| `opentubex://download/dQw4w9WgXcQ?quality=1080p` | Enqueue a download (confirmation prompt) |
| `opentubex://settings/network` | Open Settings at a section |
| `opentubex://sync/pair?code=123456` | Sync device pairing |

```rust
#[cfg(desktop)]
{
    use tauri_plugin_deep_link::DeepLinkExt;
    app.deep_link().register_all()?;                    // runtime registration (dev)
    app.deep_link().on_open_url(|event| {
        for url in event.urls() { route_deep_link(url); }
    });
}
```

**Requirements**

- [ ] **Single instance** (`tauri-plugin-single-instance`) is mandatory: a second launch forwards `argv` to the running instance instead of starting a second app.
- [ ] **Cold start:** URLs present in `argv` at launch are queued and dispatched only after the frontend emits `app-ready` — otherwise the route is lost.
- [ ] **Validation is security-critical.** Parse strictly:
  - Allowlist the scheme and the action segment.
  - Video IDs must match `^[A-Za-z0-9_-]{11}$`; channel IDs `^UC[A-Za-z0-9_-]{22}$`; playlist IDs `^(PL|UU|LL|RD)[A-Za-z0-9_-]{10,}$`.
  - Reject anything else with a logged, user-visible "invalid link" toast.
  - **Never** interpolate URL content into shell arguments, SQL, or `eval`.
- [ ] Destructive or costly actions (`download`) require explicit user confirmation — a link must never silently start writing files.
- [ ] Registration per platform:
  - **Windows** — registry keys written by the NSIS/MSI installer (`HKCU\Software\Classes\opentubex`).
  - **macOS** — `CFBundleURLTypes` in `Info.plist` via bundle config.
  - **Linux** — `.desktop` file with `MimeType=x-scheme-handler/opentubex;` + `update-desktop-database`.
- [ ] Verify handoff from a browser on all 3 OSes with the app (a) running and (b) closed.

### 2.5 File Associations (Day 7–8)

```jsonc
"bundle": {
  "fileAssociations": [
    { "ext": ["sltplaylist"], "name": "SlyTube Playlist", "description": "SlyTube playlist file",
      "role": "Editor", "mimeType": "application/x-slytube-playlist" },
    { "ext": ["sltbackup"], "name": "SlyTube Backup", "description": "SlyTube encrypted backup",
      "role": "Editor", "mimeType": "application/x-slytube-backup" }
  ]
}
```

| Extension | Content | Open behavior |
|-----------|---------|---------------|
| `.sltplaylist` | JSON playlist export | Import wizard → preview → confirm |
| `.sltbackup` | Encrypted settings/sync backup (Phase 05 envelope) | Restore wizard → password prompt → preview → confirm |
| `.m3u` / `.m3u8` | Standard playlist | **Opt-in only** — never claim by default (would hijack media players) |

- [ ] Opened file paths arrive via `argv` (Windows/Linux) or the `RunEvent::Opened` / `application:openFiles:` path (macOS). Both must be handled.
- [ ] Same cold-start queueing rule as deep links.
- [ ] Validate file magic/schema before parsing; cap file size (10 MB) to avoid memory abuse.
- [ ] Import is always previewed and confirmed — never applied automatically.
- [ ] Provide `Settings → Advanced → Register file associations` for manual (re)registration on Linux where installers vary.

### 2.6 Auto-Updater (Day 8–10)

```jsonc
"plugins": {
  "updater": {
    "endpoints": ["https://releases.slytube.app/{{target}}/{{arch}}/{{current_version}}"],
    "pubkey": "<minisign public key>",
    "windows": { "installMode": "passive" }
  }
}
```

```rust
#[tauri::command]
pub async fn check_updates(app: AppHandle) -> Result<UpdateInfo, AppError> {
    let update = app.updater()?.check().await?;
    Ok(match update {
        Some(u) => UpdateInfo { available: true, current_version: app.package_info().version.to_string(),
                                latest_version: u.version, release_notes: u.body.unwrap_or_default(),
                                release_date: u.date.map(|d| d.to_string()).unwrap_or_default() },
        None => UpdateInfo { available: false, .. },
    })
}

#[tauri::command]
pub async fn install_update(app: AppHandle) -> Result<(), AppError> {
    if let Some(u) = app.updater()?.check().await? {
        let mut downloaded = 0u64;
        u.download_and_install(
            |chunk, total| { downloaded += chunk as u64;
                let _ = app.emit("update-progress", json!({ "downloaded": downloaded, "total": total })); },
            || { let _ = app.emit("update-ready", ()); },
        ).await?;
        app.restart();
    }
    Ok(())
}
```

**Requirements**

- [ ] **Signing keys:** minisign keypair generated (`tauri signer generate`); private key + password stored only in CI secrets; public key committed. Losing the private key breaks updates for all users — document a key-rotation contingency.
- [ ] Release artifacts (`.AppImage.tar.gz`, `.msi.zip`/`.nsis.zip`, `.app.tar.gz`) plus `latest.json` published by CI.
- [ ] Check schedule: on startup (delayed 30 s) + every 6 h + manual from Settings/menu.
- [ ] User control: `autoCheckUpdates`, `autoDownloadUpdates`, `autoInstallOnQuit`. Never force-install without consent; never restart mid-download.
- [ ] UI: non-modal banner → release notes dialog → progress → "Restart to update".
- [ ] **Pre-update safety:** back up `slytube.db` before installing; refuse to update while downloads are active unless the user pauses them.
- [ ] **Rollback path:** if the app crashes twice within 60 s of an update, offer safe mode and a link to the previous release.
- [ ] Platform notes:
  - **macOS** — updates require a properly signed & notarized bundle; unsigned builds will fail silently under Gatekeeper.
  - **Windows** — `installMode: "passive"` shows progress without prompts; app must exit cleanly first.
  - **Linux** — only AppImage is self-updatable. `.deb`/`.rpm`/Flatpak users must be told to update via their package manager (detect install source and hide the updater UI accordingly).

### 2.7 Notifications & Startup Behavior (Day 10)

- [ ] Native notifications (`tauri-plugin-notification`) for: download complete, download failed, update available, sync conflict. Permission requested on first use; respect a `notificationsEnabled` setting.
- [ ] Clicking a notification focuses the app and routes to the relevant view.
- [ ] Launch-at-login (`tauri-plugin-autostart`), optional, default off; supports "start minimized to tray".
- [ ] Power/idle awareness: pause the PoToken idle reaper and sync scheduler while suspended; resume cleanly.
- [ ] Single-instance focus: launching a second copy raises and focuses the existing window.

---

## 3. Deliverables

| ID | Deliverable | Acceptance Criteria |
|----|-------------|---------------------|
| D7.1 | Global shortcuts + rebinding UI | Media keys work with the app unfocused on 3 OSes; conflicts surfaced, never silent |
| D7.2 | System tray | Live state icon/tooltip; quick actions functional; graceful Linux degradation |
| D7.3 | Native menus | macOS app menu correct; Win/Linux menu bar correct; items reactive |
| D7.4 | `opentubex://` handler | All 7 URL forms route correctly, warm and cold start, with strict validation |
| D7.5 | File associations | `.sltplaylist` / `.sltbackup` open via double-click on 3 OSes with preview-confirm |
| D7.6 | Auto-updater | Signed end-to-end update from v0.1.0 → v0.1.1 verified on Win + macOS + AppImage |
| D7.7 | Notifications + autostart | Actionable notifications; launch-at-login toggle works |
| D7.8 | Platform behavior matrix | Documented differences and degradations in this doc's appendix |

---

## 4. Dependencies

**Inbound**

| From | Needs |
|------|-------|
| Phase 01 | `tray-icon` feature, `global-shortcut`/`updater` plugins, bundle + deep-link config, tray icon assets |
| Phase 04 | Window/tray command primitives, `check_updates` / `install_update` scaffolding |
| Phase 06 | Routes and views to navigate into; Settings sections for shortcuts/updates |
| Phase 02 | Download state for tray/tooltip/notifications |

**Outbound**

| Phase | Consumes |
|-------|----------|
| 08 | Platform test matrix, updater verification, deep-link E2E scenarios |

**External:** code-signing certificate (Windows EV/OV), Apple Developer ID + notarization, release hosting for `latest.json` and artifacts.

---

## 5. Risks

| ID | Risk | Prob. | Impact | Mitigation |
|----|------|-------|--------|------------|
| R7.1 | macOS notarization not ready → updater and Gatekeeper both fail | **High** | **High** | Start the Apple Developer enrollment in Phase 01; test notarization on a throwaway build by week 10; budget 2 days for signing issues |
| R7.2 | Windows SmartScreen flags unsigned installer | High | High | Obtain an OV/EV certificate; sign installer + binaries; build reputation early with signed pre-releases |
| R7.3 | Linux tray missing (no appindicator) | High | Medium | Detect at startup, disable tray features, show a one-time notice with install instructions; never crash |
| R7.4 | Wayland blocks global shortcuts | High | Medium | Detect session type; document limitation; offer in-app shortcuts as the fallback |
| R7.5 | Deep-link injection (malicious URL) | Medium | **High** | Strict regex validation, allowlisted actions, confirmation for destructive actions, no shell/SQL interpolation, fuzz tests |
| R7.6 | Cold-start deep links dropped | High | Medium | Queue until `app-ready`; explicit E2E test with the app closed |
| R7.7 | Second instance spawns instead of forwarding | Medium | Medium | `single-instance` plugin registered **first**; test with file-open and URL-open paths |
| R7.8 | Updater bricks installs (bad signature / partial download) | Low | **Critical** | Signature verification is non-negotiable; staged rollout (5 % → 25 % → 100 %); pre-update DB backup; documented manual recovery |
| R7.9 | Lost minisign private key | Low | Critical | Key escrow in a password manager + offline backup; rotation runbook in `docs/decisions` |
| R7.10 | File association hijacking user complaints (`.m3u`) | Medium | Low | Never claim `.m3u` by default; opt-in only |
| R7.11 | App becomes unquittable via tray/close-to-tray combination | Low | Medium | `Quit` always hard-exits; E2E test asserting process termination |

---

## 6. Estimated Duration

| Task | Days |
|------|------|
| 2.1 Global shortcuts | 2.0 |
| 2.2 System tray | 2.0 |
| 2.3 Native menus | 1.0 |
| 2.4 Protocol handler | 2.0 |
| 2.5 File associations | 1.0 |
| 2.6 Auto-updater | 1.5 |
| 2.7 Notifications & startup | 0.5 |
| **Total** | **10.0** (2 weeks @ 1 dev) |

> Runs concurrently with the tail of Phase 06 (weeks 13–14). Requires a backend-capable developer separate from the frontend view work.

---

## 7. Platform Behavior Matrix (Appendix)

| Feature | Windows | macOS | Linux |
|---------|---------|-------|-------|
| Tray left-click | Toggle window | Opens menu | Menu only (DE-dependent) |
| Tray icon theming | Static PNG | Template (auto) | Light/dark variants |
| Global media keys | ✅ | ✅ (may need Accessibility permission) | ✅ X11 / ⚠️ Wayland |
| Now-playing integration | SMTC (stretch) | MPNowPlayingInfoCenter (stretch) | MPRIS (planned) |
| Native menu location | Window menu bar | System app menu | Window menu bar |
| Deep-link registration | Installer registry keys | `Info.plist` | `.desktop` + `update-desktop-database` |
| Auto-update | MSI/NSIS passive | Requires notarization | AppImage only |
| Launch at login | Registry Run key | LaunchAgent | `.desktop` in autostart |
| Taskbar progress | ✅ | ✅ (Dock) | ⚠️ Unity/KDE only |

---

## 8. Exit Criteria

- [ ] Media keys control playback with the app unfocused on Windows and macOS; Linux status documented.
- [ ] Tray shows live download progress and all quick actions work on 3 OSes (or degrade cleanly).
- [ ] All 7 deep-link forms verified from a browser, app running and app closed.
- [ ] Double-clicking `.sltplaylist` and `.sltbackup` opens SlyTube with the import wizard.
- [ ] A signed update from v0.1.0 → v0.1.1 installs successfully on Windows, macOS (notarized), and AppImage.
- [ ] Deep-link fuzz tests produce no crashes and no unvalidated actions.
- [ ] `Quit` terminates all processes (app, sidecars, PoToken webview) — verified with a process monitor.

---

## 9. References

- [Architecture — Menu & Tray](../architecture/02-component-mapping.md)
- [Backend — System Commands](../backend/02-tauri-commands.md#system-commands)
- [Tauri v2 Deep Linking](https://v2.tauri.app/plugin/deep-linking/)
- [Tauri v2 Updater](https://v2.tauri.app/plugin/updater/)
- Previous: [Phase 06 — Frontend Migration](06-frontend-migration.md) · Next: [Phase 08 — Testing & Polish](08-testing-polish.md)

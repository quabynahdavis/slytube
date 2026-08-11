# Phase 10: Critical Fixes & Polish

## Status: PLANNED

## Overview
Fix remaining stubs/placeholders and polish the app for a functional release. The app currently runs with real YouTube data but has several critical pieces that are still placeholders.

## Current State
- ✅ Tauri app running via `bun run tauri dev`
- ✅ Real YouTube data via youtubei.js + Invidious
- ✅ SQLite database with all tables
- ✅ 17 views with routing
- ✅ shadcn-vue components
- ✅ Dark/light theme toggle
- ✅ Keyboard shortcuts
- ✅ System tray
- ⚠️ Video player is placeholder (needs Shaka Player)
- ⚠️ PoToken is stub (needs hidden webview)
- ⚠️ Sync server has no actual server communication
- ❌ Tab system not working (will be removed)

## Tasks

### Task 1: Remove Tab System
**Priority: High** | **Est: 30 min**

Remove tab functionality for this release. Simplify to single-view navigation.

Actions:
- Remove TabBar component from AppLayout.vue
- Remove tab-related state from stores
- Remove useVerticalTabBar references
- Remove tab-related settings from Settings.vue
- Clean up tab-related keyboard shortcuts

Files:
- src/components/layout/AppLayout.vue
- src/components/layout/TopNav.vue
- src/views/Settings.vue

### Task 2: Shaka Player Integration
**Priority: Critical** | **Est: 3-4 hours**

Real video playback is core to the app.

Actions:
1. Install shaka-player: `bun add shaka-player`
2. Create src/components/player/ShakaPlayer.vue
3. Create src/components/player/PlayerControls.vue
4. Create src/components/player/PlayerSeekbar.vue
5. Create src/composables/usePlayer.ts
6. Implement manifest generation:
   - Local: Extract DASH from youtubei.js streaming_data
   - Invidious: Use /api/manifest/dash/id/{videoId}
7. SponsorBlock segment overlay on seekbar
8. Update Watch.vue to use ShakaPlayer
9. Wire keyboard shortcuts (space, arrows, f, m)

Files:
- src/components/player/ShakaPlayer.vue (new)
- src/components/player/PlayerControls.vue (new)
- src/components/player/PlayerSeekbar.vue (new)
- src/composables/usePlayer.ts (new)
- src/views/Watch.vue (update)

### Task 3: PoToken Hidden Webview Stub
**Priority: Medium** | **Est: 1 hour**

Set up infrastructure for future botGuard integration.

Actions:
1. Create src-tauri/src/potoken/webview.rs
2. Create hidden WebviewWindow with custom partition
3. Implement generate_po_token command (returns placeholder)
4. Frontend: Call generate_po_token before local requests
5. Cache tokens per video ID
6. Fall back to Invidious when PoToken fails

Files:
- src-tauri/src/potoken/webview.rs (new)
- src/composables/useInnertube.ts (update to include PoToken)

### Task 4: Sync Server - Skip
**Priority: Low** | **Est: 15 min**

Mark sync as "Coming Soon" in UI.

Actions:
1. Add "Coming Soon" badge to sync settings
2. Disable sync start button
3. Add tooltip explaining sync requires a server endpoint
4. Keep all crypto code in src-tauri/src/sync/ for future

Files:
- src/views/Settings.vue

### Task 5: Polish & Rough Edges
**Priority: Medium** | **Est: 2-3 hours**

Make the app feel complete and professional.

#### 5.1 Error Handling
- Add ErrorBoundary.vue component
- Consistent error messages with retry buttons
- Network error detection (offline mode)

#### 5.2 Empty States
- Proper empty states for all data views
- Call-to-action buttons

#### 5.3 Loading States
- Skeleton loaders for all data-fetching views
- Smooth transitions

#### 5.4 i18n
- Install vue-i18n
- Add src/i18n/index.ts
- Add en-US locale file
- Replace hardcoded strings with translation keys

#### 5.5 Keyboard Shortcuts
- / Focus search
- Space Play/Pause
- ←/→ Seek -5s/+5s
- ↑/↓ Volume up/down
- f Fullscreen
- m Mute toggle
- t Toggle theme
- j/k Scroll down/up
- Esc Close dialogs

#### 5.6 Settings
- Make all settings functional
- Add import/export
- Add reset to defaults

#### 5.7 About Page
- Show real app version
- Add keyboard shortcuts reference
- Add repository links

## Execution Order
1. Remove tabs (simplifies everything)
2. Shaka Player (core feature)
3. PoToken stub (infrastructure)
4. Sync skip (UI notes)
5. Polish (error handling, i18n, shortcuts)

## Future Releases (Not This One)
- Custom video player (replace Shaka)
- PoToken botGuard WASM port
- Sync server communication
- Tab system (re-add properly)
- PiP mode, Mini player, Shorts UI

## Definition of Done
- [ ] Tabs removed, single-view navigation works
- [ ] Shaka Player plays real YouTube videos with DASH
- [ ] Quality selector works
- [ ] SponsorBlock segments show on seekbar
- [ ] PoToken infrastructure in place (placeholder token)
- [ ] Sync marked as coming soon
- [ ] All views have proper error/empty/loading states
- [ ] i18n scaffolding in place
- [ ] All keyboard shortcuts work
- [ ] Settings fully functional
- [ ] About page complete

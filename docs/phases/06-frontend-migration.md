# Phase 06 — Frontend Migration

| Field | Value |
|-------|-------|
| **Timeline** | Week 6 – Week 14 |
| **Duration** | 45 working days (9 weeks) |
| **Risk Level** | 🟡 Medium (low technical risk, **high volume risk**) |
| **Blocks** | Phase 07 (UI hooks), Phase 08 (test targets) |
| **Depends On** | Phase 01 (design system), Phase 04 (command surface), Phase 05 (sync DTOs) |

---

## 1. Goals

1. Migrate state management from **Vuex (14 modules)** to **Pinia** with typed setup stores.
2. Replace every bespoke Vue component with **shadcn-vue / Reka UI** primitives on Tailwind v4, adopting the `reka-nova` style and Hugeicons.
3. Migrate all **16 views** in dependency-and-value order, each reaching feature parity with OpenTubeX.
4. Replace every `window.api.*` Electron bridge call with typed `invoke()` / `listen()` from generated bindings.
5. Establish routing, layout shell, theming, i18n, accessibility, and virtualization standards.
6. Keep the app runnable at every step — no long-lived broken `main`.

**Volume reality:** this is the largest phase (~40 % of total effort). Treat it as 4 sub-milestones with independent demos.

---

## 2. Sub-Milestones

| Milestone | Weeks | Content | Demo gate |
|-----------|-------|---------|-----------|
| **M6.1 Foundation** | 6–7 | Router, layout shell, theme, i18n, API client, Pinia core stores | App shell navigable with placeholder views |
| **M6.2 Core Views** | 8–10 | Views 1–6 (Watch, Home, Search, Channel, Playlist, Subscriptions) | End-to-end: search → watch → subscribe |
| **M6.3 Library & Downloads** | 11–12 | Views 7–11 (Downloads, History, Library, Playlists mgmt, Trending) | Full library management + live downloads |
| **M6.4 Settings & Auxiliary** | 13–14 | Views 12–16 (Settings, Sync, Proxy/Privacy, About, Mini-player) + polish | Feature-complete UI |

---

## 3. Tasks

### 3.1 Frontend Foundation (Week 6–7)

**Directory structure:**

```
src/
├── main.ts
├── App.vue
├── router/          index.ts, guards.ts
├── stores/          (Pinia)
├── views/           (16 views)
├── components/
│   ├── ui/          (shadcn-vue generated — do not hand-edit)
│   ├── layout/      AppShell, Sidebar, TitleBar, CommandPalette
│   ├── video/       VideoCard, VideoGrid, VideoList, VideoPlayer, PlayerControls
│   ├── channel/     ChannelCard, ChannelHeader
│   ├── playlist/    PlaylistCard, PlaylistItemRow
│   ├── download/    DownloadRow, DownloadProgress, QualityPicker
│   └── common/      EmptyState, ErrorState, LoadingSkeleton, InfiniteScroll, ConfirmDialog
├── composables/     useInvoke, useTauriEvent, useInfiniteScroll, useTheme, useVirtualList, useShortcuts
├── lib/             bindings.ts (generated), api/, utils.ts, format.ts
├── locales/         en.json, ...
└── style.css
```

**Router (16 routes + guards):**

```ts
const routes = [
  { path: '/',                 name: 'home',           component: () => import('@/views/HomeView.vue') },
  { path: '/watch/:id',        name: 'watch',          component: () => import('@/views/WatchView.vue'), props: true },
  { path: '/search',           name: 'search',         component: () => import('@/views/SearchView.vue') },
  { path: '/channel/:id/:tab?',name: 'channel',        component: () => import('@/views/ChannelView.vue'), props: true },
  { path: '/playlist/:id',     name: 'playlist',       component: () => import('@/views/PlaylistView.vue'), props: true },
  { path: '/subscriptions',    name: 'subscriptions',  component: () => import('@/views/SubscriptionsView.vue') },
  { path: '/downloads',        name: 'downloads',      component: () => import('@/views/DownloadsView.vue') },
  { path: '/history',          name: 'history',        component: () => import('@/views/HistoryView.vue') },
  { path: '/library',          name: 'library',        component: () => import('@/views/LibraryView.vue') },
  { path: '/playlists',        name: 'playlists',      component: () => import('@/views/PlaylistsView.vue') },
  { path: '/trending',         name: 'trending',       component: () => import('@/views/TrendingView.vue') },
  { path: '/settings/:section?', name: 'settings',     component: () => import('@/views/SettingsView.vue'), props: true },
  { path: '/sync',             name: 'sync',           component: () => import('@/views/SyncView.vue') },
  { path: '/privacy',          name: 'privacy',        component: () => import('@/views/PrivacyView.vue') },
  { path: '/about',            name: 'about',          component: () => import('@/views/AboutView.vue') },
  { path: '/mini',             name: 'mini-player',    component: () => import('@/views/MiniPlayerView.vue'), meta: { layout: 'bare' } },
];
```

- [ ] `createWebHashHistory()` — safest under Tauri's custom protocol origins.
- [ ] Scroll-position restoration per route (critical for infinite feeds).
- [ ] Every route lazy-loaded; measure chunk sizes.

**Core composables:**

```ts
// useInvoke — typed, cancellable, with loading/error state
export function useInvoke<T>(fn: () => Promise<T>, opts?: { immediate?: boolean; onError?: (e: ApiError) => void }) {
  const data = ref<T>(); const error = ref<ApiError>(); const loading = ref(false);
  let seq = 0;
  async function execute() {
    const my = ++seq; loading.value = true; error.value = undefined;
    try { const r = await fn(); if (my === seq) data.value = r; }
    catch (e) { if (my === seq) { error.value = e as ApiError; opts?.onError?.(e as ApiError); } }
    finally { if (my === seq) loading.value = false; }
  }
  if (opts?.immediate !== false) onMounted(execute);
  return { data, error, loading, execute };
}

// useTauriEvent — auto-unlisten on unmount (prevents the #1 leak class)
export function useTauriEvent<T>(name: string, handler: (p: T) => void) {
  let un: UnlistenFn | undefined;
  onMounted(async () => { un = await listen<T>(name, e => handler(e.payload)); });
  onUnmounted(() => un?.());
}
```

- [ ] **Rule:** no component calls `listen()` directly — always via `useTauriEvent`.
- [ ] **Rule:** no component calls `invoke()` with a string literal — always via `lib/api/*` wrappers over generated bindings.

**Layout shell:**

- [ ] `AppShell.vue`: resizable sidebar (`resizable` primitive), top bar with search + navigation, content outlet, global toaster, update banner, offline banner.
- [ ] Custom title bar (optional per platform) using `data-tauri-drag-region`.
- [ ] `CommandPalette.vue` (`command` primitive) — ⌘K/Ctrl+K global search and actions.
- [ ] Theme: `useTheme()` with `light | dark | system`, synced to the `settings` store and OS via `window.matchMedia`.
- [ ] i18n via `vue-i18n`; extract all OpenTubeX strings; English complete, scaffolding for others.

### 3.2 Store Migration: Vuex → Pinia (Week 6–8, parallel with views)

14 Vuex modules → 13 Pinia stores (two merged).

| # | Vuex module | Pinia store | Backing commands | Priority |
|---|-------------|-------------|------------------|----------|
| 1 | `settings` | `useSettingsStore` | `get_settings`, `save_settings`, `patch_setting` | P0 |
| 2 | `utils` | `useAppStore` | `get_app_info`, `get_disk_usage` | P0 |
| 3 | `player` | `usePlayerStore` | (client-side) + `record_watch_progress` | P0 |
| 4 | `videos` | `useVideoStore` | `search_videos`, `get_video_details`, `get_related_videos` | P0 |
| 5 | `subscriptions` | `useSubscriptionStore` | `get_subscriptions`, `subscribe_channel`, `get_subscription_feed` | P1 |
| 6 | `playlists` | `usePlaylistStore` | playlist CRUD + reorder | P1 |
| 7 | `history` | `useHistoryStore` | watch/search history commands | P1 |
| 8 | `downloads` | `useDownloadStore` | download commands + `download-progress` | P1 |
| 9 | `channels` | `useChannelStore` | channel commands | P1 |
| 10 | `search` | merged into `useVideoStore` | — | P1 |
| 11 | `invidious` | `useInstanceStore` | instance commands | P2 |
| 12 | `proxy` | `useProxyStore` | proxy commands | P2 |
| 13 | `sync` | `useSyncStore` | sync commands + sync events | P2 |
| 14 | `profiles` | `useProfileStore` | profile-scoped settings | P2 |

**Migration pattern (Vuex options → Pinia setup):**

```ts
// Before (Vuex)
const state = { downloads: [], activeCount: 0 };
const actions = {
  async startDownload({ commit }, payload) {
    const id = await window.api.startDownload(payload);
    commit('addDownload', { id, ...payload });
  },
};

// After (Pinia setup store)
export const useDownloadStore = defineStore('downloads', () => {
  const items = ref<Download[]>([]);
  const byId = computed(() => new Map(items.value.map(d => [d.id, d])));
  const active = computed(() => items.value.filter(d => d.status === 'downloading' || d.status === 'pending'));

  async function start(req: StartDownloadRequest) {
    const id = await api.downloads.start(req);
    items.value.unshift({ ...optimistic(req), id });
    return id;
  }

  function applyProgress(p: DownloadProgress) {
    const d = byId.value.get(p.downloadId);
    if (d) Object.assign(d, p);
  }

  return { items, active, start, applyProgress };
});
```

**Store rules**

- [ ] Setup-style stores only (better TS inference, tree-shaking).
- [ ] Stores own **all** `invoke` calls; components never call the API directly.
- [ ] Event subscriptions live in a single `stores/eventBridge.ts` initialized once in `main.ts` — not per store, not per component.
- [ ] Hydration on startup: `settings` → `app` → everything else lazily on first view mount.
- [ ] Optimistic updates with rollback on error for all mutations (subscribe, playlist edits, download actions).
- [ ] Normalize entities (`Map<id, T>`) for videos/channels to avoid duplicate objects across views.
- [ ] No `localStorage` for domain data — SQLite is the source of truth; `store` plugin only for UI ephemera.

### 3.3 Component Migration: custom → shadcn-vue (Week 7–13, continuous)

| OpenTubeX component | Replacement | Notes |
|---------------------|-------------|-------|
| `ft-button`, `ft-icon-button` | `Button` + Hugeicons | variants: default/secondary/ghost/destructive |
| `ft-input`, `ft-search-input` | `Input` + `Command` | palette-backed suggestions |
| `ft-select`, `ft-drop-down` | `Select`, `DropdownMenu` | |
| `ft-toggle-switch` | `Switch` | |
| `ft-slider` | `Slider` | volume, playback rate, seek |
| `ft-prompt`, `ft-modal` | `Dialog`, `AlertDialog` | focus trap for free |
| `ft-toast` | `Toast` / `Sonner` | queue + variants |
| `ft-loader` | `Skeleton` + `Progress` | skeletons must match final layout to avoid CLS |
| `ft-card` | `Card` | |
| `ft-list-video`, `ft-video-card` | `VideoCard` composed of `Card`+`AspectRatio`+`Badge` | 3 densities: grid / list / compact |
| `ft-channel-bubble` | `Avatar` | |
| `ft-tooltip` | `Tooltip` | |
| `ft-sidebar` | `Sheet` (mobile) + `resizable` panel (desktop) | |
| `ft-tabs` | `Tabs` | channel tabs, settings sections |
| Context menus | `ContextMenu` | right-click on video/playlist rows |
| `ft-video-player` (video.js) | **Custom `VideoPlayer.vue`** wrapping native `<video>` + shadcn controls | See below |

**Player decision:** drop `video.js`. Build a thin `VideoPlayer.vue` over the native element with a custom control bar composed of `Slider`, `Button`, `DropdownMenu`, `Tooltip`. Rationale: smaller bundle, full styling control, no theme fights. Requirements: DASH/HLS via `shaka-player` or `hls.js` only where adaptive streams demand it; keyboard shortcuts; PiP; captions; playback rate; sponsor-segment skip hooks; resume-from-position.

**Component standards**

- [ ] `<script setup lang="ts">` everywhere; props typed via interfaces; `defineEmits` typed.
- [ ] Every list >50 items uses virtualization (`useVirtualList` / `@tanstack/vue-virtual`).
- [ ] Every async surface has three states: skeleton, empty, error — via `LoadingSkeleton` / `EmptyState` / `ErrorState`.
- [ ] Images lazy-loaded with explicit `width`/`height` and blurhash-or-skeleton placeholder (zero CLS).
- [ ] Accessibility: keyboard reachable, visible focus ring, `aria-label` on icon-only buttons, `prefers-reduced-motion` honored. Reka UI provides most of this — do not defeat it.
- [ ] No component exceeds ~250 lines; extract sub-components.

### 3.4 View Migration — 16 Views in Priority Order (Week 8–14)

| # | View | Priority | Est. days | Key dependencies | Notes |
|---|------|----------|-----------|------------------|-------|
| 1 | **WatchView** | P0 | 5.0 | player store, `get_video_details`, PoToken | Largest single view: player, description, comments, related, chapters, captions, quality picker |
| 2 | **HomeView** | P0 | 2.0 | video store, subscription feed | Sections: continue watching, subscriptions, trending; infinite scroll |
| 3 | **SearchView** | P0 | 2.5 | `search_videos`, filters | Filters (type/date/duration/sort), suggestions, result virtualization |
| 4 | **ChannelView** | P0 | 3.0 | channel store | Tabs: videos / shorts / live / playlists / about; subscribe button |
| 5 | **PlaylistView** | P0 | 2.5 | playlist store | Ordered list, drag-reorder, play-all, shuffle |
| 6 | **SubscriptionsView** | P0 | 2.5 | subscription store | Merged feed, per-channel filter, refresh progress, mark-as-seen |
| 7 | **DownloadsView** | P1 | 3.0 | download store + progress events | Active/completed/failed tabs, live progress, pause/resume/cancel/retry, open folder |
| 8 | **HistoryView** | P1 | 2.0 | history store | Date grouping, search within history, per-item and bulk delete |
| 9 | **LibraryView** | P1 | 1.5 | multiple stores | Hub: playlists, downloads, history, subscriptions summary cards |
| 10 | **PlaylistsView** | P1 | 2.0 | playlist store | Grid, create/rename/delete, import/export, duplicate |
| 11 | **TrendingView** | P1 | 1.5 | video store | Region + category tabs |
| 12 | **SettingsView** | P1 | 5.0 | all stores | Sections: general, appearance, playback, downloads, network, privacy, sync, advanced, about. Largest surface after Watch |
| 13 | **SyncView** | P2 | 3.0 | sync store (Phase 05) | Setup wizard, device list + approval codes, conflict resolver, recovery kit, legacy import |
| 14 | **PrivacyView** | P2 | 2.0 | sync/settings stores | Privacy mode selector with impact preview, data report, purge actions |
| 15 | **AboutView** | P2 | 0.5 | app store | Version, licenses, credits, update check, log access |
| 16 | **MiniPlayerView** | P2 | 1.5 | player store | Separate window, compact controls, always-on-top |
| | **Subtotal** | | **39.5** | | |

**Per-view definition of done**

- [ ] Feature parity checklist versus the OpenTubeX view (documented per view in the PR description).
- [ ] Loading / empty / error states implemented.
- [ ] Keyboard navigable; screen-reader labels present.
- [ ] No `any` in the view's TypeScript; `vue-tsc` clean.
- [ ] Virtualized where lists can exceed 50 items.
- [ ] Component test (Vitest + Testing Library) covering the primary interaction.
- [ ] Atomic commit `feat(frontend): migrate <View>`.

**WatchView breakdown (largest):**

| Sub-task | Days |
|----------|------|
| Player shell + controls + keyboard map | 1.5 |
| Quality/format selection + PoToken-gated streams | 1.0 |
| Description, chapters, metadata, actions (like/share/download/add-to-playlist) | 1.0 |
| Comments (lazy, paginated, replies) | 1.0 |
| Related videos sidebar + autoplay | 0.5 |

### 3.5 API Integration Layer (Week 6, then continuous)

```
src/lib/api/
├── index.ts        // barrel
├── client.ts       // invoke wrapper: error normalization, timing, dev logging
├── settings.ts
├── videos.ts
├── channels.ts
├── playlists.ts
├── history.ts
├── subscriptions.ts
├── downloads.ts
├── potoken.ts
├── sync.ts
├── proxy.ts
├── window.ts
└── system.ts
```

```ts
// client.ts
export async function call<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  const t0 = performance.now();
  try {
    return await invoke<T>(cmd, args);
  } catch (raw) {
    throw normalizeError(raw, cmd);   // → ApiError { code, message, details }
  } finally {
    if (import.meta.env.DEV) console.debug(`[ipc] ${cmd} ${(performance.now() - t0).toFixed(1)}ms`);
  }
}
```

**Electron → Tauri bridge mapping:**

| Electron | Tauri |
|----------|-------|
| `window.api.invoke('x', a)` | `api.domain.x(a)` → `invoke('x', a)` |
| `ipcRenderer.on('evt', cb)` | `useTauriEvent('evt', cb)` |
| `ipcRenderer.send(...)` fire-and-forget | `invoke` with `Result<(), _>` (awaited) |
| `window.api.openPath(p)` | `api.system.showInFolder(p)` |
| `require('electron').shell.openExternal` | `api.system.openExternal(url)` |
| `navigator.clipboard` fallbacks | `@tauri-apps/plugin-clipboard-manager` |

- [ ] Global error interceptor maps `ApiError.code` → user-facing toast copy (i18n keys).
- [ ] Retry-with-backoff wrapper for `NETWORK`-coded failures (max 3).
- [ ] Request de-duplication for identical in-flight reads (e.g., the same video requested by two components).
- [ ] Delete the Electron `preload`/`window.api` shim entirely once the last view migrates — **verified by a lint rule banning `window.api`**.

### 3.6 Performance & Polish (Week 13–14)

- [ ] Route-level code splitting verified; initial chunk < 250 KB gzipped.
- [ ] Virtualize all long lists; measure with 5000-item fixtures.
- [ ] Image pipeline: lazy `loading="lazy"`, `decoding="async"`, thumbnail proxying honored under Strict privacy.
- [ ] Memoize expensive computeds; avoid deep watchers on large arrays; `shallowRef` for large collections.
- [ ] Transitions honoring `prefers-reduced-motion`.
- [ ] Bundle analysis: no duplicate Hugeicons barrels, no accidental `moment`/`lodash` full imports.
- [ ] Targets: TTI < 1.5 s cold; route switch < 100 ms; 60 fps scroll on a 1000-item grid.

---

## 4. Deliverables

| ID | Deliverable | Acceptance Criteria |
|----|-------------|---------------------|
| D6.1 | Frontend foundation | Router + shell + theme + i18n + composables; app navigable |
| D6.2 | 13 Pinia stores | All Vuex modules retired; no `window.api` references remain |
| D6.3 | Component library | All `ft-*` components replaced; shadcn-vue primitives only |
| D6.4 | 16 migrated views | Each meets its per-view DoD and parity checklist |
| D6.5 | API layer | Typed wrappers over generated bindings; global error handling |
| D6.6 | Event bridge | Single initialization point; zero listener leaks (asserted in tests) |
| D6.7 | Perf budget met | TTI < 1.5 s; initial chunk < 250 KB gz; 60 fps large lists |
| D6.8 | A11y baseline | Keyboard-complete; axe scan with no critical violations |

---

## 5. Dependencies

**Inbound**

| From | Needs |
|------|-------|
| Phase 01 | shadcn-vue baseline, Tailwind v4 tokens, path aliases |
| Phase 04 | Full command surface + `bindings.ts` (**hard blocker** for M6.2 onward) |
| Phase 05 | Sync DTOs + real sync behavior (blocks views 13–14 only) |
| Phase 03 | PoToken for WatchView format selection (degradable) |

**Outbound**

| Phase | Consumes |
|-------|----------|
| 07 | Views to attach shortcuts, tray actions, deep-link routes, update UI |
| 08 | Components/views as unit + E2E test targets |

**Internal ordering:** M6.1 must complete before M6.2. Views 1–6 before 7–11 (shared components mature first). Views 13–14 gated on Phase 05.

---

## 6. Risks

| ID | Risk | Prob. | Impact | Mitigation |
|----|------|-------|--------|------------|
| R6.1 | 9-week duration overruns and compresses Phases 07–08 | **High** | High | 4 sub-milestones with demo gates; weekly burn-down of the 16-view checklist; descope P2 views (13–16) to a post-1.0 release if week 12 tracking is red |
| R6.2 | Player rewrite (dropping video.js) underestimated | Medium | High | Timebox to 1.5 days for the shell; if adaptive streaming proves hard, fall back to `shaka-player` with custom skin |
| R6.3 | Feature-parity gaps discovered late | High | Medium | Write the per-view parity checklist **before** starting each view, derived from the OpenTubeX source |
| R6.4 | Event listener leaks on route change | Medium | Medium | Mandatory `useTauriEvent`; lint rule banning raw `listen(`; leak assertion in E2E |
| R6.5 | Store hydration races cause flicker/stale UI | Medium | Medium | Deterministic hydration order; suspense boundaries; skeletons matching final layout |
| R6.6 | shadcn-vue/Reka upgrades break generated components | Medium | Medium | Pin versions; treat `components/ui` as vendored (regenerate deliberately, review diffs) |
| R6.7 | Bundle bloat from icons/locales | Medium | Low | Per-icon imports; lazy locale chunks; CI bundle-size budget check |
| R6.8 | Inconsistent UX across views (different devs/weeks) | Medium | Medium | Component standards section is normative; design review at each milestone gate |
| R6.9 | Backend contract changes mid-phase | Medium | High | Phase 04 contract frozen at its exit; any change requires regenerating bindings + a migration note |
| R6.10 | Accessibility regressions | Medium | Medium | axe-core in CI on key views; keyboard-only walkthrough per milestone |

---

## 7. Estimated Duration

| Block | Days |
|-------|------|
| 3.1 Foundation | 6.0 |
| 3.2 Store migration | 6.0 (partly parallel with views) |
| 3.3 Component migration | continuous (absorbed into view estimates) |
| 3.4 View migration (16) | 39.5 |
| 3.5 API layer | 3.0 |
| 3.6 Performance & polish | 4.0 |
| **Raw total** | **58.5** |
| **With 2 devs in parallel from week 8** | **≈45.0** (9 weeks) |

> Single-developer execution will not fit 9 weeks. Plan for **2 frontend developers** from M6.2 onward, or descope views 13–16.

---

## 8. Exit Criteria

- [ ] All 16 views migrated and passing their parity checklists.
- [ ] Zero references to Vuex or `window.api` in `src/` (CI-enforced grep).
- [ ] `vue-tsc --noEmit` clean; ESLint clean; no `any` in stores or views.
- [ ] Performance budgets met on a mid-tier machine.
- [ ] axe scan: no critical/serious violations on the 6 P0 views.
- [ ] Every view has at least one Vitest component test.
- [ ] Demo of the full user journey: launch → search → watch → download → playlist → sync → settings.

---

## 9. References

- [Architecture — Store Migration Pattern](../architecture/02-component-mapping.md)
- [Frontend Overview](../frontend/OVERVIEW.md)
- [Backend — Tauri Commands](../backend/02-tauri-commands.md)
- Previous: [Phase 05 — Sync & Encryption](05-sync-encryption.md) · Next: [Phase 07 — System Integration](07-system-integration.md)

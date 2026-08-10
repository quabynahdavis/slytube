# Store Migration: Vuex → Pinia

## Overview

The legacy renderer uses **Vuex 4** with 14 namespaced modules registered in
`src/renderer/store/index.js`. Slytube replaces this entirely with **Pinia**
using the **setup (composition) syntax** and full **TypeScript** typing.

This document defines:

1. The 1:1 module → store mapping
2. The chosen authoring syntax (setup vs options) and why
3. The typing conventions every store must follow
4. The mechanical rules for replacing `commit` / `dispatch` / `mapGetters`

> **Scope note:** State *shape* is preserved wherever possible so that view
> migration (see [03-view-migration-order.md](03-view-migration-order.md)) is a
> rename exercise rather than a redesign. Persistence moves from
> `electron-store` / NeDB handlers to Tauri commands
> (see [../backend/02-tauri-commands.md](../backend/02-tauri-commands.md)).

---

## 1. Module Mapping

All stores live in `src/stores/` and are named `<domain>.ts`. The Pinia store id
is the **kebab-case** module name; the exported composable is
`use<PascalCase>Store`.

| # | Vuex Module | Pinia File | Store Id | Composable | Persistence Backend | Complexity |
|---|-------------|-----------|----------|------------|---------------------|------------|
| 1 | `settings.js` | `stores/settings.ts` | `settings` | `useSettingsStore` | `get_settings` / `save_settings` | **XL** |
| 2 | `history.js` | `stores/history.ts` | `history` | `useHistoryStore` | SQLite via `history_*` commands | L |
| 3 | `playlists.js` | `stores/playlists.ts` | `playlists` | `usePlaylistsStore` | SQLite via `playlist_*` commands | L |
| 4 | `profiles.js` | `stores/profiles.ts` | `profiles` | `useProfilesStore` | SQLite via `profile_*` commands | L |
| 5 | `subscription-cache.js` | `stores/subscription-cache.ts` | `subscription-cache` | `useSubscriptionCacheStore` | In-memory + SQLite TTL cache | M |
| 6 | `sync-server.js` | `stores/sync-server.ts` | `sync-server` | `useSyncServerStore` | Rust crypto + `sync_*` commands | M |
| 7 | `tabs.js` | `stores/tabs.ts` | `tabs` | `useTabsStore` | Session only (no persistence) | S |
| 8 | `watch-queue.js` | `stores/watch-queue.ts` | `watch-queue` | `useWatchQueueStore` | Session only | S |
| 9 | `watch-stats.js` | `stores/watch-stats.ts` | `watch-stats` | `useWatchStatsStore` | SQLite aggregate queries | S |
| 10 | `downloads.js` | `stores/downloads.ts` | `downloads` | `useDownloadsStore` | `download_*` commands + events | M |
| 11 | `invidious.js` | `stores/invidious.ts` | `invidious` | `useInvidiousStore` | Instance list cache | S |
| 12 | `player.js` | `stores/player.ts` | `player` | `usePlayerStore` | Settings-backed prefs | M |
| 13 | `search-history.js` | `stores/search-history.ts` | `search-history` | `useSearchHistoryStore` | SQLite via `search_history_*` | S |
| 14 | `utils.js` | `stores/utils.ts` | `utils` | `useUtilsStore` | None (ephemeral UI/app state) | M |

### 1.1 Per-Module Responsibilities

| Store | Owns | Key Actions | Notable Getters |
|-------|------|-------------|-----------------|
| `settings` | ~250 user preferences, theme, region, backend prefs | `load`, `updateSetting`, `resetToDefaults`, `exportSettings`, `importSettings` | `currentTheme`, `backendPreference`, `defaultQuality` |
| `history` | Watch history entries, watch progress | `load`, `upsertEntry`, `updateWatchProgress`, `removeEntry`, `clearAll` | `historyCacheById`, `sortedHistory` |
| `playlists` | User playlists + playlist items | `load`, `createPlaylist`, `addVideo`, `removeVideo`, `renamePlaylist`, `deletePlaylist` | `favourites`, `playlistById` |
| `profiles` | Subscription profiles, active profile | `load`, `createProfile`, `updateProfile`, `deleteProfile`, `setActiveProfile` | `activeProfile`, `activeSubscriptions`, `profileById` |
| `subscription-cache` | Per-channel cached videos / shorts / live / community | `getCached`, `setCached`, `invalidateChannel`, `clearAll` | `cacheForChannel`, `isStale` |
| `sync-server` | Sync session, device keys, sync status | `login`, `logout`, `push`, `pull`, `rotateKeys` | `isLinked`, `lastSyncedAt`, `syncState` |
| `tabs` | Open in-app tabs, active tab index | `openTab`, `closeTab`, `activateTab`, `reorderTabs` | `activeTab`, `tabCount` |
| `watch-queue` | Up-next queue, shuffle/repeat mode | `enqueue`, `dequeue`, `next`, `previous`, `shuffle`, `clear` | `currentItem`, `hasNext`, `queueLength` |
| `watch-stats` | Aggregated watch time, top channels | `refresh`, `recordWatchSession` | `totalWatchTime`, `topChannels`, `dailyBuckets` |
| `downloads` | Download jobs, progress, errors | `load`, `start`, `pause`, `resume`, `cancel`, `retry`, `remove` | `activeDownloads`, `completedDownloads`, `progressById` |
| `invidious` | Instance list, current instance, health | `fetchInstances`, `setInstance`, `pingInstance` | `currentInstanceUrl`, `healthyInstances` |
| `player` | Volume, rate, quality, captions, autoplay | `setVolume`, `setPlaybackRate`, `setQuality`, `toggleCaptions` | `effectiveQuality`, `isMuted` |
| `search-history` | Recent search terms | `load`, `addTerm`, `removeTerm`, `clear` | `recentTerms`, `suggestionsFor` |
| `utils` | Toasts, progress bar, sidebar state, app-wide flags | `showToast`, `setProgressBar`, `toggleSideNav`, `setOutlines` | `isSideNavOpen`, `showProgressBar` |

---

## 2. Setup Syntax vs Options Syntax

### 2.1 Decision

> **All Slytube stores use the setup (composition) syntax.**

The options syntax (`state` / `getters` / `actions`) mirrors Vuex most closely
and is tempting for a fast port, but it degrades in exactly the areas that
matter for this codebase.

| Concern | Options Syntax | Setup Syntax | Winner |
|---------|----------------|--------------|--------|
| Type inference for nested state | Requires explicit `state: (): State =>` and often breaks on recursive types | `ref<T>()` infers exactly | Setup |
| Getters referencing other getters | `this` typing frequently fails; needs manual annotations | Plain `computed` composition | Setup |
| Composables inside stores (`useEventListener`, i18n, router) | Not usable inside `state()` | First-class | Setup |
| Tauri `listen()` lifecycle (unlisten handles) | Awkward — must stash in state | Natural closure variable | Setup |
| Tree-shaking / code splitting | Whole object retained | Only used refs retained | Setup |
| Private (non-exported) internals | Impossible — all state public | Simply don't return it | Setup |
| Familiarity to Vuex authors | Higher | Lower | Options |

The deciding factors are **Tauri event listeners** (`downloads`, `sync-server`)
and **private internals** (cache maps in `subscription-cache` that must never be
mutated from views).

### 2.2 Side-by-side

**Vuex module (before) — `store/modules/player.js`:**

```javascript
const state = {
  volume: 1,
  playbackRate: 1,
  quality: 'auto',
  muted: false
}

const getters = {
  getVolume: (state) => state.volume,
  getEffectiveQuality: (state, getters, rootState) => {
    return state.quality === 'auto'
      ? rootState.settings.defaultQuality
      : state.quality
  }
}

const mutations = {
  setVolume(state, value) { state.volume = value },
  setMuted(state, value) { state.muted = value }
}

const actions = {
  updateVolume({ commit, dispatch }, value) {
    commit('setVolume', value)
    dispatch('settings/updateSetting', { key: 'volume', value }, { root: true })
  }
}

export default { state, getters, mutations, actions, namespaced: true }
```

**Pinia setup store (after) — `src/stores/player.ts`:**

```typescript
import { computed, ref } from 'vue'
import { defineStore } from 'pinia'
import { useSettingsStore } from '@/stores/settings'
import type { VideoQuality } from '@/types/player'

export const usePlayerStore = defineStore('player', () => {
  const settings = useSettingsStore()

  // ---- state ----
  const volume = ref<number>(1)
  const playbackRate = ref<number>(1)
  const quality = ref<VideoQuality>('auto')
  const muted = ref<boolean>(false)

  // ---- getters ----
  const effectiveQuality = computed<VideoQuality>(() =>
    quality.value === 'auto' ? settings.defaultQuality : quality.value
  )
  const isMuted = computed<boolean>(() => muted.value || volume.value === 0)

  // ---- actions ----
  async function setVolume(value: number): Promise<void> {
    volume.value = Math.min(1, Math.max(0, value))
    await settings.updateSetting('volume', volume.value)
  }

  function setPlaybackRate(rate: number): void {
    playbackRate.value = rate
  }

  function setQuality(next: VideoQuality): void {
    quality.value = next
  }

  function toggleMuted(): void {
    muted.value = !muted.value
  }

  return {
    volume, playbackRate, quality, muted,
    effectiveQuality, isMuted,
    setVolume, setPlaybackRate, setQuality, toggleMuted
  }
})
```

### 2.3 Mandatory store file layout

Every store file follows this ordering. Reviewers reject PRs that deviate.

```typescript
export const useXStore = defineStore('x', () => {
  // 1. Dependencies  — other stores, composables, router
  // 2. State         — ref() / reactive() / shallowRef()
  // 3. Private state — NOT returned (prefix with _ for clarity)
  // 4. Getters       — computed()
  // 5. Actions       — function declarations (never arrow consts)
  // 6. Lifecycle     — Tauri listen() wiring, watchers
  // 7. return { ... } — explicit public surface
})
```

Rules:

- `function foo() {}` not `const foo = () => {}` — hoisting lets actions call
  each other in any order and produces cleaner devtools names.
- Never return private refs. If a view needs read access, expose a `computed`.
- `shallowRef` for large immutable payloads (API responses, cached feeds) to
  avoid deep reactivity cost.
- One store per file. No barrel re-exports of stores (breaks HMR).

---

## 3. TypeScript Typing Conventions

### 3.1 Type location

| Kind | Location |
|------|----------|
| Domain models shared with Rust | `src/types/models.ts` (mirrors `serde` structs) |
| Store-local state interfaces | Co-located in the store file |
| Tauri command payloads/responses | `src/types/commands.ts` |
| Tauri event payloads | `src/types/events.ts` |

Rust ⇄ TS parity is enforced by `ts-rs` exports where practical; otherwise the
type is hand-written and annotated with the Rust struct name:

```typescript
/** Mirrors `src-tauri/src/models/video.rs::Video` */
export interface Video {
  id: string
  title: string
  channelId: string | null
  channelName: string | null
  durationSeconds: number
  publishedAt: number | null
  thumbnails: Thumbnail[]
  isLive: boolean
  isUpcoming: boolean
}
```

> **Casing:** Rust uses `snake_case`; all `serde` structs carry
> `#[serde(rename_all = "camelCase")]` so the frontend never converts casing
> manually. Command *arguments* are the exception — Tauri converts the JS
> `camelCase` argument object to Rust `snake_case` parameters automatically.

### 3.2 Typing state

```typescript
// Primitives: rely on inference, annotate only when the literal is too narrow.
const isLoading = ref(false)                  // boolean — inferred, fine
const quality = ref<VideoQuality>('auto')     // annotate — otherwise `string`

// Collections: always annotate the element type.
const items = ref<Download[]>([])
const byId = ref<Map<string, Download>>(new Map())

// Nullable: be explicit, never use `undefined` for "not loaded".
const activeProfile = ref<Profile | null>(null)

// Large API payloads: shallowRef to skip deep proxying.
const feed = shallowRef<SubscriptionFeed | null>(null)

// Records keyed by id: use Record<> for JSON-serialisable, Map for hot paths.
const progressById = ref<Record<string, DownloadProgress>>({})
```

### 3.3 Typing async actions

Every action that crosses the IPC boundary returns `Promise<T>` and handles
`AppError` explicitly. Never let a rejected `invoke` escape into a view.

```typescript
import { invoke } from '@tauri-apps/api/core'
import type { AppError } from '@/types/commands'
import { isAppError } from '@/lib/errors'

async function load(): Promise<void> {
  isLoading.value = true
  error.value = null
  try {
    items.value = await invoke<Download[]>('get_downloads')
  } catch (e: unknown) {
    error.value = isAppError(e) ? e : { code: 'UNKNOWN', message: String(e) }
  } finally {
    isLoading.value = false
  }
}
```

`src/lib/errors.ts`:

```typescript
import type { AppError } from '@/types/commands'

export function isAppError(value: unknown): value is AppError {
  return (
    typeof value === 'object' &&
    value !== null &&
    'code' in value &&
    'message' in value
  )
}
```

### 3.4 Typed settings — the `settings` store

`settings.js` is the largest module (~250 keys, auto-generated mutations). The
Vuex version generated `setX`/`updateX` pairs at runtime, which is untypable.
Pinia replaces this with a **single typed record plus a generic updater**.

```typescript
// src/types/settings.ts
export interface Settings {
  // appearance
  baseTheme: 'system' | 'light' | 'dark' | 'black' | 'dracula'
  mainColor: string
  secColor: string
  uiScale: number
  disableSmoothScrolling: boolean

  // playback
  defaultQuality: VideoQuality
  defaultVolume: number
  defaultPlaybackRate: number
  autoplayVideos: boolean
  autoplayPlaylists: boolean
  enterFullscreenOnDisplayVideo: boolean

  // backend
  backendPreference: 'local' | 'invidious'
  backendFallback: boolean
  currentInvidiousInstance: string

  // privacy
  rememberHistory: boolean
  saveWatchedProgress: boolean
  saveVideoHistoryWithLastViewedPlaylist: boolean

  // sponsorblock
  useSponsorBlock: boolean
  sponsorBlockUrl: string
  sponsorBlockSponsor: SponsorBlockCategoryPreference
  // ... remaining keys
}

export type SettingKey = keyof Settings
export type SettingValue<K extends SettingKey> = Settings[K]
```

```typescript
// src/stores/settings.ts
export const useSettingsStore = defineStore('settings', () => {
  const values = ref<Settings>({ ...DEFAULT_SETTINGS })
  const isLoaded = ref(false)

  // Generic, fully type-safe updater — replaces ~250 Vuex mutations.
  async function updateSetting<K extends SettingKey>(
    key: K,
    value: SettingValue<K>
  ): Promise<void> {
    values.value[key] = value
    await invoke('save_setting', { key, value })
  }

  async function load(): Promise<void> {
    values.value = await invoke<Settings>('get_settings')
    isLoaded.value = true
  }

  async function resetToDefaults(): Promise<void> {
    values.value = await invoke<Settings>('reset_settings')
  }

  // Hot-path getters get named computeds; everything else reads `values`.
  const baseTheme = computed(() => values.value.baseTheme)
  const backendPreference = computed(() => values.value.backendPreference)
  const defaultQuality = computed(() => values.value.defaultQuality)
  const useSponsorBlock = computed(() => values.value.useSponsorBlock)

  return {
    values, isLoaded,
    baseTheme, backendPreference, defaultQuality, useSponsorBlock,
    load, updateSetting, resetToDefaults
  }
})
```

Usage in a settings form is a one-liner with full autocompletion and a compile
error on type mismatch:

```vue
<script setup lang="ts">
const settings = useSettingsStore()
// ✅ ok
settings.updateSetting('defaultVolume', 0.8)
// ❌ TS2345: Argument of type 'string' is not assignable to type 'number'
settings.updateSetting('defaultVolume', 'loud')
</script>
```

### 3.5 Typing Tauri event listeners

Stores that receive backend pushes own their listener lifecycle:

```typescript
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import type { DownloadProgressEvent } from '@/types/events'

export const useDownloadsStore = defineStore('downloads', () => {
  const items = ref<Download[]>([])
  const progressById = ref<Record<string, DownloadProgress>>({})

  let _unlisten: UnlistenFn[] = []          // private — not returned

  async function initListeners(): Promise<void> {
    if (_unlisten.length) return
    _unlisten.push(
      await listen<DownloadProgressEvent>('download-progress', ({ payload }) => {
        progressById.value[payload.downloadId] = payload.progress
      }),
      await listen<DownloadCompleteEvent>('download-complete', ({ payload }) => {
        const item = items.value.find(i => i.id === payload.downloadId)
        if (item) item.status = 'completed'
        delete progressById.value[payload.downloadId]
      })
    )
  }

  function disposeListeners(): void {
    _unlisten.forEach(fn => fn())
    _unlisten = []
  }

  return { items, progressById, initListeners, disposeListeners }
})
```

`initListeners()` is called once from `App.vue` `onMounted`, never per-view.

### 3.6 `strict` mode requirements

`tsconfig.json` must keep these on. Stores are the highest-value place to catch
errors:

```jsonc
{
  "compilerOptions": {
    "strict": true,
    "noUncheckedIndexedAccess": true,   // forces null-checks on byId[key]
    "exactOptionalPropertyTypes": true,
    "noImplicitOverride": true,
    "verbatimModuleSyntax": true        // `import type` everywhere
  }
}
```

`noUncheckedIndexedAccess` in particular changes store code:

```typescript
// ❌ Download | undefined is not assignable to Download
const d: Download = itemsById.value[id]
// ✅
const d = itemsById.value[id]
if (!d) return
```

---

## 4. Replacing `commit` / `dispatch` with Direct Calls

### 4.1 Translation table

| Vuex | Pinia |
|------|-------|
| `this.$store.state.player.volume` | `playerStore.volume` |
| `this.$store.getters['player/getVolume']` | `playerStore.volume` (getter collapsed) |
| `this.$store.commit('player/setVolume', v)` | `playerStore.setVolume(v)` |
| `this.$store.dispatch('player/updateVolume', v)` | `await playerStore.setVolume(v)` |
| `commit('setX', v)` (inside module) | `x.value = v` |
| `dispatch('other/act', p, { root: true })` | `useOtherStore().act(p)` |
| `mapState('player', ['volume'])` | `const { volume } = storeToRefs(playerStore)` |
| `mapGetters('player', ['effectiveQuality'])` | `const { effectiveQuality } = storeToRefs(playerStore)` |
| `mapActions('player', ['setVolume'])` | `const { setVolume } = playerStore` |
| `mapMutations(...)` | *(deleted — mutations do not exist)* |
| `store.subscribe((mutation, state) => ...)` | `store.$subscribe((mutation, state) => ...)` |
| `store.subscribeAction(...)` | `store.$onAction(...)` |
| `store.replaceState(s)` | `store.$patch(s)` |
| `store.registerModule(...)` | *(not needed — stores register on first use)* |

### 4.2 The mutation/action collapse

Vuex forced a two-step: an action does async work, then commits a mutation.
Pinia has **no mutations** — an action mutates state directly. Every
`mutations` block collapses into its calling action.

**Before (3 concepts, 4 indirections):**

```javascript
// history.js
const mutations = {
  setHistory(state, records) { state.historyCache = records },
  addToHistory(state, record) { state.historyCache.unshift(record) }
}

const actions = {
  async grabHistory({ commit }) {
    const records = await DBHistoryHandlers.find()
    commit('setHistory', records)
  },
  async updateHistory({ commit }, record) {
    await DBHistoryHandlers.upsert(record)
    commit('addToHistory', record)
  }
}
```

**After (1 concept):**

```typescript
export const useHistoryStore = defineStore('history', () => {
  const records = ref<HistoryEntry[]>([])

  async function load(): Promise<void> {
    records.value = await invoke<HistoryEntry[]>('get_history')
  }

  async function upsertEntry(entry: HistoryEntry): Promise<void> {
    await invoke('upsert_history_entry', { entry })
    const idx = records.value.findIndex(r => r.videoId === entry.videoId)
    if (idx >= 0) records.value.splice(idx, 1)
    records.value.unshift(entry)
  }

  return { records, load, upsertEntry }
})
```

### 4.3 Cross-store calls replace root dispatch

Vuex root-namespaced dispatch becomes a plain import + call. Pinia resolves the
store lazily, so circular imports between store *modules* are safe as long as
the `use*Store()` call happens **inside** the setup function or action body.

```typescript
// ✅ safe — resolved at store-instantiation time
export const useWatchQueueStore = defineStore('watch-queue', () => {
  const history = useHistoryStore()      // top of setup: fine
  ...
})

// ✅ also safe — resolved at call time (use this if A↔B are mutually dependent)
async function next(): Promise<void> {
  const history = useHistoryStore()      // inside action: always fine
  await history.upsertEntry(current.value)
}
```

> **Rule:** if two stores import each other, at least one side must resolve the
> other **inside an action**, never at setup top-level.

### 4.4 Destructuring in views

Destructuring a store object breaks reactivity. Use `storeToRefs` for
state/getters; destructure actions freely.

```vue
<script setup lang="ts">
import { storeToRefs } from 'pinia'
import { usePlayerStore } from '@/stores/player'

const playerStore = usePlayerStore()

// ✅ reactive
const { volume, effectiveQuality } = storeToRefs(playerStore)
// ✅ actions are plain functions — safe to destructure
const { setVolume, toggleMuted } = playerStore
// ❌ loses reactivity — becomes a snapshot number
const { volume: brokenVolume } = playerStore
</script>
```

### 4.5 Batch updates

Replace multi-commit sequences with a single `$patch` to trigger one reactivity
flush and one devtools entry:

```typescript
// Instead of three separate assignments
store.$patch({ isLoading: false, items: fetched, lastFetchedAt: Date.now() })

// Function form for arrays/maps (avoids replacing the ref)
store.$patch((state) => {
  state.items.push(...newItems)
  state.lastFetchedAt = Date.now()
})
```

### 4.6 Store setup in `main.ts`

```typescript
import { createApp } from 'vue'
import { createPinia } from 'pinia'
import App from './App.vue'
import router from './router'

const app = createApp(App)
const pinia = createPinia()

app.use(pinia)
app.use(router)
app.mount('#app')
```

Bootstrapping order in `App.vue` matters — `settings` and `profiles` must
resolve before any view renders:

```typescript
onMounted(async () => {
  await settingsStore.load()      // themes, backend preference
  await profilesStore.load()      // active profile drives subscriptions
  await Promise.all([
    historyStore.load(),
    playlistsStore.load(),
    searchHistoryStore.load(),
    downloadsStore.load()
  ])
  await downloadsStore.initListeners()
  await syncServerStore.initListeners()
  isBootstrapped.value = true
})
```

---

## 5. Migration Order & Checklist

Stores are ported in dependency order — leaves first, so nothing imports an
unported store.

| Wave | Stores | Rationale |
|------|--------|-----------|
| 1 | `utils`, `tabs`, `search-history`, `invidious` | No dependencies; unblocks shell UI |
| 2 | `settings` | Depended on by nearly everything |
| 3 | `profiles`, `player` | Depend on `settings` |
| 4 | `history`, `playlists`, `watch-stats`, `watch-queue` | Depend on `settings` + `profiles` |
| 5 | `subscription-cache`, `downloads` | Depend on `profiles`, emit/consume events |
| 6 | `sync-server` | Depends on all persisted stores |

Per-store definition of done:

- [ ] `src/stores/<name>.ts` created with setup syntax
- [ ] All state typed; no `any`, no implicit `undefined`
- [ ] All mutations collapsed into actions
- [ ] All persistence routed through `invoke`, not direct DB/`window.api`
- [ ] Event listeners (if any) own `init`/`dispose` and are wired in `App.vue`
- [ ] Cross-store deps resolved inside actions where circular
- [ ] Consuming views use `storeToRefs`
- [ ] Old Vuex module deleted (no dual-write period)
- [ ] `vue-tsc --noEmit` passes

---

## 6. Anti-Patterns

| Anti-pattern | Why it fails | Do instead |
|--------------|--------------|------------|
| Keeping a `mutations`-like `setX` per field | Reintroduces Vuex boilerplate | Mutate refs directly in actions |
| `defineStore('x', { state, getters, actions })` | Options syntax — banned | Setup syntax |
| `const { items } = useStore()` | Reactivity lost | `storeToRefs` |
| Calling `useXStore()` at module top-level (outside setup) | Pinia not yet installed → runtime error | Call inside setup/action |
| Storing derived data in state | Drifts out of sync | `computed` |
| `any` on an `invoke` result | Silently breaks on Rust schema change | `invoke<T>(...)` with a mirrored type |
| Registering `listen()` in a view | Duplicate listeners per navigation | Store-owned `initListeners()` |
| Deep-reactive 500-item feed arrays | Proxy overhead on every render | `shallowRef` + explicit replacement |

---

## References

- [Pinia — Setup Stores](https://pinia.vuejs.org/core-concepts/#Setup-Stores)
- [Pinia — Migrating from Vuex](https://pinia.vuejs.org/cookbook/migration-vuex.html)
- [../architecture/02-component-mapping.md](../architecture/02-component-mapping.md) — Electron → Tauri mapping
- [../architecture/03-data-flow.md](../architecture/03-data-flow.md) — IPC and event patterns
- [../backend/02-tauri-commands.md](../backend/02-tauri-commands.md) — Command signatures
- [02-shadcn-components.md](02-shadcn-components.md) — Component layer
- [03-view-migration-order.md](03-view-migration-order.md) — View porting sequence

# View Migration Order

## Overview

Slytube ports **16 views** from the legacy renderer. This document defines the
order, the dependency graph that produces it, and the effort estimate per view.

Ordering principle:

> **Migrate by dependency-unlock value, not by size.** A view is scheduled early
> if porting it forces the creation of shared infrastructure (components,
> stores, composables) that later views reuse.

That is why `Settings` — the largest view — goes first: it exercises every form
primitive and the entire `settings` store, which nearly all other views read.

Effort is expressed in **ideal engineer-days (IED)** for one engineer already
familiar with the codebase. Multiply by ~1.4 for calendar time.

---

## 1. Migration Order Summary

| # | Priority | View | Route | Effort (IED) | Unlocks |
|---|----------|------|-------|--------------|---------|
| 1 | P0 | **Settings** | `/settings` | 8 | All form primitives, `settings` store, theming |
| 2 | P0 | **Watch** | `/watch/:id` | 10 | Player, `player`/`watch-queue`/`history` stores, API fallback chain |
| 3 | P1 | **Subscriptions** | `/subscriptions` | 5 | Feed tabs, `subscription-cache`, `profiles`, infinite scroll |
| 4 | P1 | **Channel** | `/channel/:id` | 4 | Channel tabs, subscribe flow, `FtListChannel` |
| 5 | P1 | **Playlist** | `/playlist/:id` | 4 | Playlist playback, reorder, `FtListPlaylist` |
| 6 | P1 | **Search** | `/search/:query` | 3 | Search filters, `search-history`, `Combobox` |
| 7 | P1 | **History** | `/history` | 2.5 | `history` store UI, bulk actions |
| 8 | P1 | **Downloads** | `/downloads` | 3 | `downloads` store, Tauri progress events, `Progress` |
| 9 | P2 | **ProfileSettings** | `/settings/profile/:id` | 2.5 | `profiles` CRUD, colour picker |
| 10 | P2 | **About** | `/about` | 0.5 | Static content |
| 11 | P2 | **Stats** | `/settings/stats` | 1.5 | `watch-stats` aggregates, charts |
| 12 | P2 | **Trending** | `/trending` | 1.5 | Region tabs, reuses feed grid |
| 13 | P2 | **Popular** | `/popular` | 1 | Invidious-only feed |
| 14 | P2 | **Hashtag** | `/hashtag/:tag` | 1 | Hashtag feed |
| 15 | P2 | **UserPlaylists** | `/userplaylists` | 2 | Playlist CRUD grid, favourites |
| 16 | P2 | **Post** | `/post/:id` | 1.5 | Community post rendering, polls |
| | | **Total** | | **51** | ≈ 10 calendar weeks solo, ≈ 4 weeks for 3 engineers |

---

## 2. Dependency Graph

```
                       ┌──────────────────┐
                       │  App Shell       │  TopNav · SideNav · TabBar
                       │  (pre-req)       │  router · theme · Toaster
                       └────────┬─────────┘
                                │
                    ┌───────────▼───────────┐
                    │  1. Settings          │  settings store
                    │  (all form UI)        │  Button/Input/Select/Switch/
                    └───────────┬───────────┘  Slider/RadioGroup/Tabs/Dialog
                                │
        ┌───────────────────────┼───────────────────────────┐
        │                       │                           │
┌───────▼────────┐   ┌──────────▼─────────┐    ┌────────────▼──────────┐
│ 2. Watch       │   │ 9. ProfileSettings │    │ 11. Stats             │
│ player・queue  │   │ profiles CRUD      │    │ watch-stats           │
│ history・SB    │   └──────────┬─────────┘    └───────────────────────┘
└───────┬────────┘              │
        │                       │ activeProfile
        │      ┌────────────────▼──────────────┐
        │      │ 3. Subscriptions              │  subscription-cache
        │      │ feed tabs (Videos/Shorts/     │  profiles
        │      │ Live/Community)               │  → FeedGrid, InfiniteScroll
        │      └──────┬───────────────┬────────┘
        │             │               │
        │   ┌─────────▼──────┐  ┌─────▼───────────┐
        │   │ 4. Channel     │  │ 16. Post        │
        │   │ (tabs, sub btn)│  │ (community)     │
        │   └─────────┬──────┘  └─────────────────┘
        │             │
        │   ┌─────────▼──────────────────────────────┐
        │   │ 12. Trending · 13. Popular · 14. Hashtag│  (all reuse FeedGrid)
        │   └─────────────────────────────────────────┘
        │
┌───────▼──────────┐   ┌──────────────────┐
│ 5. Playlist      │◄──┤ 15. UserPlaylists│
│ (playback order) │   │ (CRUD grid)      │
└──────────────────┘   └──────────────────┘

┌────────────────┐  ┌────────────────┐  ┌────────────┐
│ 6. Search      │  │ 7. History     │  │ 8.Downloads│
│ (needs FeedGrid│  │ (needs         │  │ (needs     │
│  + Combobox)   │  │  FtListVideo)  │  │  events)   │
└────────────────┘  └────────────────┘  └────────────┘

┌────────────┐
│ 10. About  │  no dependencies — can be done any time as a warm-up
└────────────┘
```

### 2.1 Hard dependencies

| View | Blocked by | Reason |
|------|-----------|--------|
| Everything | App Shell | Router, theme attribute, `TopNav`/`SideNav`, `Toaster` mount point |
| Watch | Settings | Reads quality, autoplay, SponsorBlock, proxy, backend prefs |
| Subscriptions | Settings, ProfileSettings*, Channel* | Needs `activeProfile`; `*` soft — can stub with default profile |
| Channel | Subscriptions | Shares `FeedGrid`, subscribe button, channel tab layout |
| Playlist | Watch | Playlist playback hands off to the player + `watch-queue` |
| UserPlaylists | Playlist | Navigates into `Playlist`; shares `FtListPlaylist` |
| Search | Subscriptions | Reuses `FeedGrid` + result-type filters |
| History | Watch | Entries are written by the player; needs `FtListVideo` watch-progress bar |
| Trending / Popular / Hashtag | Subscriptions | Pure `FeedGrid` consumers |
| Post | Subscriptions (Community tab) | Shares post renderer |
| Stats | Settings | Rendered as a settings sub-route |
| ProfileSettings | Settings | Rendered as a settings sub-route |

### 2.2 Shared artefacts produced along the way

Building these once, at the point marked, prevents rework:

| Artefact | Produced in | Consumed by |
|----------|-------------|-------------|
| `SettingsSection`, `SettingRow` | Settings | ProfileSettings, Stats |
| `useSettingBinding()` composable | Settings | ProfileSettings |
| `FeedGrid` (virtualised card grid) | Subscriptions | Channel, Search, Trending, Popular, Hashtag, UserPlaylists, History |
| `useInfiniteScroll()` composable | Subscriptions | all feed views |
| `useApiFallback()` (local → Invidious) | Watch | Channel, Search, Playlist, Trending |
| `VideoMenuItems` (shared menu) | Watch | every view rendering `FtListVideo` |
| `EmptyState` component | History | Downloads, UserPlaylists, Search, Subscriptions |
| `ErrorState` + retry | Watch | all data views |

> **Do not** build `FeedGrid` speculatively before Subscriptions. Its virtualisation
> requirements are only knowable once the subscription feed's real payload sizes
> (500–2000 items) are in hand.

---

## 3. Per-View Detail

### 3.1 Settings — P0, 8 IED

The largest view: ~14 sections, ~250 controls.

| Sub-section | Controls | Notes |
|-------------|----------|-------|
| General | 18 | Region, locale, landing page, list layout |
| Theme | 12 | Base theme, main/secondary colour, UI scale, expand side nav |
| Player | 24 | Quality, volume, rate, autoplay, PiP, screenshot |
| Privacy | 14 | History toggles, clear actions (AlertDialog) |
| Subscription | 8 | Feed fetching, unseen badge |
| Distraction Free | 16 | Hide comments/live chat/related/shorts |
| Data | 10 | Import/export subscriptions, history, playlists (file dialogs) |
| Proxy | 8 | Protocol, host, port, test button |
| SponsorBlock | 12 | Per-category behaviour selects |
| Download | 10 | Folder picker, quality, format, concurrency |
| Parental Control | 6 | — |
| External Player | 8 | Executable path, args |
| Experimental | 6 | — |
| Password / Lock | 4 | — |

**Effort breakdown**

| Task | IED |
|------|-----|
| `settings` store port + `Settings` type (250 keys) | 2.0 |
| `SettingsSection` / `SettingRow` / `useSettingBinding` | 1.0 |
| 14 section components | 3.5 |
| Import/export flows (Tauri dialog + fs) | 0.75 |
| Search-within-settings + deep links | 0.5 |
| Theme switching + verification across 5 themes | 0.25 |

**Risks:** the 250-key `Settings` interface must match the Rust struct exactly —
a mismatch surfaces as silent `undefined`. Generate from `ts-rs` if possible.

**Definition of done:** every control round-trips to SQLite and survives an app
restart; changing theme/UI-scale applies without reload.

---

### 3.2 Watch — P0, 10 IED

Highest-risk view. Combines the player, queue, comments, and recommendations.

| Concern | Detail |
|---------|--------|
| Player | `ft-shaka-video-player` — DASH/HLS manifests, adaptive quality, audio-track selection, captions, PiP, fullscreen, keyboard shortcuts, stats-for-nerds |
| Data | `getVideoInformationLocal` → fallback `getVideoInformationInvidious` (see [04-api-integration.md](04-api-integration.md)) |
| SponsorBlock | Segment fetch, skip execution, category behaviour, skip notice |
| Queue | Up-next, autoplay, playlist context, shuffle/repeat |
| History | Write entry on play; persist watch progress on `timeupdate` (throttled) |
| Comments | Lazy-loaded, paginated, sort by top/new, replies |
| Recommendations | Sidebar list, hidden when distraction-free |
| Description | Timestamp links, expandable, chapter list |
| Live | Live chat panel, DVR seeking |

**Effort breakdown**

| Task | IED |
|------|-----|
| `ft-shaka-video-player` port (Tailwind + TS + Pinia) | 3.5 |
| Video info fetch + fallback chain + error states | 1.5 |
| SponsorBlock client + skip logic | 1.0 |
| `watch-queue` + autoplay + playlist context | 1.0 |
| History write + progress persistence | 0.5 |
| Comments panel | 1.0 |
| Recommendations sidebar | 0.5 |
| Description / chapters / timestamps | 0.5 |
| Live chat | 0.5 |

**Risks:** Shaka in the Tauri webview differs per platform (WKWebView on macOS,
WebView2 on Windows, WebKitGTK on Linux). Codec support and MSE behaviour must
be verified on all three **before** the rest of the view is built. Budget a
0.5 IED spike in week 1.

**Definition of done:** 1080p DASH playback with audio-track switching, captions,
working SponsorBlock skips, and resumable progress on all three platforms.

---

### 3.3 Subscriptions — P1, 5 IED

Four tabs over the active profile's channels.

| Tab | Source | Notes |
|-----|--------|-------|
| Videos | per-channel feed merge | Default; largest payload |
| Shorts | per-channel shorts | Hideable via settings |
| Live | per-channel live/upcoming | — |
| Community | per-channel posts | Links to `Post` view |

Key work: parallel per-channel fetch with concurrency limit, merge + sort by
publish date, `subscription-cache` TTL, "N new since last visit" badge, and
virtualised rendering (a 200-channel profile yields 2000+ items).

**Effort breakdown**

| Task | IED |
|------|-----|
| `FeedGrid` + virtualisation | 1.5 |
| `useInfiniteScroll` composable | 0.5 |
| Parallel fetch + concurrency limiter + partial-failure UI | 1.25 |
| `subscription-cache` store + TTL invalidation | 0.75 |
| Tabs + per-tab scroll retention | 0.5 |
| Unseen-count badge, refresh, empty state | 0.5 |

**Risks:** partial failures are normal (one channel 403s). The UI must render
successful channels and surface failures non-blockingly (toast + retry), never
fail the whole feed.

---

### 3.4 Channel — P1, 4 IED

Tabs: Videos, Shorts, Live, Playlists, Community, Channels, About, Search.
Includes banner, avatar, sub count, subscribe/unsubscribe with optimistic
update, and per-tab sort (newest/oldest/popular).

| Task | IED |
|------|-----|
| Header (banner, `Avatar`, subscribe `Button`, stats) | 1.0 |
| 8 tabs wired to `FeedGrid` / playlist grid | 1.75 |
| In-channel search | 0.5 |
| About tab (links, description, joined date) | 0.5 |
| Optimistic subscribe + profile integration | 0.25 |

---

### 3.5 Playlist — P1, 4 IED

Handles both remote (YouTube/Invidious) and local user playlists — the branch
that makes this larger than it looks.

| Task | IED |
|------|-----|
| Header (stacked thumbnail, title, owner, count, duration) | 0.5 |
| Item list + pagination for 1000+ item playlists | 1.0 |
| Play-all / shuffle → `watch-queue` handoff | 0.75 |
| Local playlist edit: reorder (drag), remove, rename | 1.25 |
| Sort modes + remove-watched action | 0.5 |

**Risks:** drag-reorder plus virtualisation conflict. Mitigation — disable
virtualisation below 200 items (the overwhelming majority of local playlists)
and disable reorder above it.

---

### 3.6 Search — P1, 3 IED

| Task | IED |
|------|-----|
| Results list (video/channel/playlist/shorts mixed) | 0.75 |
| Filter panel (sort, date, type, duration, features) | 1.0 |
| Search suggestions `Combobox` in `TopNav` | 0.75 |
| `search-history` integration + clear | 0.5 |

---

### 3.7 History — P1, 2.5 IED

| Task | IED |
|------|-----|
| List with search/filter | 0.75 |
| Bulk select + delete + `AlertDialog` clear-all | 0.75 |
| Infinite scroll over SQLite pagination | 0.5 |
| `EmptyState` component (reused later) | 0.5 |

---

### 3.8 Downloads — P1, 3 IED

The only view driven primarily by **backend push events**.

| Task | IED |
|------|-----|
| `downloads` store + Tauri event listeners | 1.0 |
| Job rows: `Progress`, speed, ETA, size | 0.75 |
| Controls: pause/resume/cancel/retry/remove | 0.75 |
| Completed tab: open file, open folder, delete from disk | 0.5 |

**Risks:** progress events arrive at high frequency (yt-dlp emits per-chunk).
Throttle to ~4 Hz in the store, not in the view.

---

### 3.9 ProfileSettings — P2, 2.5 IED

| Task | IED |
|------|-----|
| Profile list + create/edit/delete `Dialog`s | 1.0 |
| Colour picker + auto-generated initials avatar | 0.5 |
| Channel assignment (move/copy between profiles) | 0.75 |
| Active-profile switch wiring to `TopNav` | 0.25 |

---

### 3.10 About — P2, 0.5 IED

Version (from Tauri), licences, links, credits, "check for updates" button.
Zero dependencies — a good warm-up task for onboarding an engineer.

---

### 3.11 Stats — P2, 1.5 IED

| Task | IED |
|------|-----|
| `watch-stats` aggregate queries + store | 0.5 |
| Summary cards (total time, videos, channels) | 0.25 |
| Daily/weekly chart | 0.5 |
| Top channels list + reset action | 0.25 |

Charting: prefer a small SVG-based chart written in-house or `unovis`. Do not
pull in a heavy chart library for one view.

---

### 3.12–3.14 Trending / Popular / Hashtag — P2, 1.5 + 1 + 1 IED

Thin `FeedGrid` consumers.

| View | Distinct work |
|------|---------------|
| Trending | Region selector + category tabs (Default/Music/Gaming/Movies) |
| Popular | Invidious-only; needs a clear "requires Invidious backend" empty state |
| Hashtag | Route param → feed; hashtag links from descriptions/comments |

---

### 3.15 UserPlaylists — P2, 2 IED

| Task | IED |
|------|-----|
| Playlist grid + create `Dialog` | 0.75 |
| Favourites pinning, sort, search | 0.5 |
| Bulk delete, import/export | 0.75 |

---

### 3.16 Post — P2, 1.5 IED

Community post detail: text, images, polls, attached video, comments. Shares
the comments panel from Watch.

---

## 4. Scheduling

Assumes 3 engineers. **Week 0** is the shared prerequisite — nobody starts a
view until the shell, router, and design tokens exist.

| Week | Engineer A | Engineer B | Engineer C |
|------|-----------|-----------|-----------|
| 0 | App shell, router, theme tokens, `components.json` → New York, primitive install | Shaka-in-webview spike (all 3 platforms) | `settings` store + `Settings` type |
| 1 | Settings sections 1–7 | Watch: player port | Settings sections 8–14 |
| 2 | Settings: import/export, search | Watch: fetch + fallback + SponsorBlock | About, ProfileSettings |
| 3 | Subscriptions: `FeedGrid` + virtualisation | Watch: queue, history, comments | Downloads |
| 4 | Subscriptions: fetch, cache, tabs | Watch: recommendations, description, live | History |
| 5 | Channel | Playlist | Search |
| 6 | Trending, Popular, Hashtag | UserPlaylists | Stats, Post |
| 7 | Cross-view polish, empty/error states, keyboard nav | Perf pass (virtualisation, memoisation) | A11y audit, theme QA |

**Milestones**

| Milestone | End of week | Criterion |
|-----------|-------------|-----------|
| M1 — Configurable shell | 2 | App launches, all settings persist, theme switching works |
| M2 — Playback | 4 | A video plays end-to-end with history + SponsorBlock |
| M3 — Content complete | 6 | All 16 views reachable and functional |
| M4 — Release candidate | 7 | Perf + a11y + theme QA clear |

---

## 5. Per-View Definition of Done

- [ ] Route registered with a typed `meta` (title, requires-backend)
- [ ] All Vuex references removed; uses Pinia + `storeToRefs`
- [ ] No `Ft*` primitive remains (only the kept domain components)
- [ ] Loading state uses `Skeleton`/`Spinner` per the policy in
      [02-shadcn-components.md](02-shadcn-components.md)
- [ ] Error state renders `ErrorState` with a working retry
- [ ] Empty state renders `EmptyState`
- [ ] Keyboard navigable end-to-end; focus visible throughout
- [ ] Verified in light / dark / black themes
- [ ] `vue-tsc --noEmit` clean
- [ ] Committed atomically as `feat(view): migrate <View>`

---

## References

- [01-store-migration.md](01-store-migration.md) — Store wave order maps onto view order
- [02-shadcn-components.md](02-shadcn-components.md) — Component inventory per view
- [04-api-integration.md](04-api-integration.md) — Data layer for Watch/Channel/Search
- [../architecture/03-data-flow.md](../architecture/03-data-flow.md) — IPC/event patterns

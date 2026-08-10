# Component Migration: FreeTube `Ft*` → shadcn-vue

## Overview

The legacy renderer ships ~60 hand-rolled `Ft*` components backed by SCSS
modules and CSS custom properties. Slytube replaces the **generic primitives**
with [shadcn-vue](https://www.shadcn-vue.com/) (Reka UI + Tailwind v4) and
**keeps the domain-specific components**, rewritten to Tailwind + `<script setup lang="ts">`.

Guiding rule:

> **If the component encodes a UI pattern that already exists in Reka UI,
> replace it. If it encodes YouTube domain knowledge, keep it.**

Replacing primitives buys accessibility (focus traps, roving tabindex, ARIA,
`aria-live`), keyboard support, RTL, and portalling for free — all of which the
hand-rolled components implement partially or not at all.

---

## 1. Theme & Configuration

### 1.1 Target style: **New York**

| Setting | Value | Notes |
|---------|-------|-------|
| `style` | `new-york` | shadcn-vue default; tighter spacing, smaller radii, `size-*` utilities, lucide-style stroke weights |
| `typescript` | `true` | All generated components are `.vue` + `<script setup lang="ts">` |
| `tailwind.css` | `src/style.css` | Tailwind v4 — config lives in CSS via `@theme`, not `tailwind.config.js` |
| `tailwind.cssVariables` | `true` | Required for FreeTube's multi-theme support (light / dark / black / dracula / catppuccin) |
| `tailwind.baseColor` | `neutral` | Neutral greys read best behind video thumbnails; avoids colour cast |
| `iconLibrary` | `hugeicons` | Already installed (`@hugeicons/vue`, `@hugeicons/core-free-icons`) |
| `aliases.ui` | `@/components/ui` | Generated primitives — treated as vendored source |
| `aliases.components` | `@/components` | Hand-written/domain components |

**Target `components.json`:**

```jsonc
{
  "$schema": "https://shadcn-vue.com/schema.json",
  "style": "new-york",
  "typescript": true,
  "tailwind": {
    "config": "",
    "css": "src/style.css",
    "baseColor": "neutral",
    "cssVariables": true,
    "prefix": ""
  },
  "iconLibrary": "hugeicons",
  "aliases": {
    "components": "@/components",
    "utils": "@/lib/utils",
    "ui": "@/components/ui",
    "lib": "@/lib",
    "composables": "@/composables"
  }
}
```

> ⚠️ **Current repo state:** `components.json` is scaffolded with
> `"style": "reka-nova"` and `"baseColor": "mist"`. This must be changed to
> `new-york` / `neutral` **before** the first `shadcn-vue add`, because the
> style is baked into every generated file and cannot be switched by editing
> the config afterwards. If primitives were already generated, delete
> `src/components/ui/` and regenerate.

### 1.2 Theme tokens

FreeTube themes map onto shadcn CSS variables. Each theme is a `[data-theme]`
block in `src/style.css`; no component references a raw colour.

```css
/* src/style.css */
@import "tailwindcss";
@import "tw-animate-css";

@custom-variant dark (&:where([data-theme=dark], [data-theme=dark] *));

:root,
[data-theme="light"] {
  --background: oklch(1 0 0);
  --foreground: oklch(0.145 0 0);
  --card: oklch(1 0 0);
  --card-foreground: oklch(0.145 0 0);
  --popover: oklch(1 0 0);
  --popover-foreground: oklch(0.145 0 0);
  --primary: oklch(0.577 0.245 27.325);   /* FreeTube red accent */
  --primary-foreground: oklch(0.985 0 0);
  --secondary: oklch(0.97 0 0);
  --muted: oklch(0.97 0 0);
  --muted-foreground: oklch(0.556 0 0);
  --accent: oklch(0.97 0 0);
  --destructive: oklch(0.577 0.245 27.325);
  --border: oklch(0.922 0 0);
  --input: oklch(0.922 0 0);
  --ring: oklch(0.708 0 0);
  --radius: 0.5rem;
}

[data-theme="dark"] {
  --background: oklch(0.145 0 0);
  --foreground: oklch(0.985 0 0);
  --card: oklch(0.205 0 0);
  --popover: oklch(0.205 0 0);
  --muted: oklch(0.269 0 0);
  --muted-foreground: oklch(0.708 0 0);
  --border: oklch(1 0 0 / 10%);
  --input: oklch(1 0 0 / 15%);
}

/* True-black OLED theme */
[data-theme="black"] {
  --background: oklch(0 0 0);
  --card: oklch(0.12 0 0);
  --popover: oklch(0.12 0 0);
  --border: oklch(1 0 0 / 8%);
}

[data-theme="dracula"] {
  --background: oklch(0.24 0.03 285);
  --foreground: oklch(0.95 0.01 280);
  --primary: oklch(0.78 0.14 320);
  --card: oklch(0.29 0.03 285);
}

@theme inline {
  --color-background: var(--background);
  --color-foreground: var(--foreground);
  --color-card: var(--card);
  --color-primary: var(--primary);
  --color-muted: var(--muted);
  --color-border: var(--border);
  --radius-lg: var(--radius);
  --radius-md: calc(var(--radius) - 2px);
  --radius-sm: calc(var(--radius) - 4px);
}
```

The `settings` store drives the theme by setting one attribute:

```typescript
watchEffect(() => {
  document.documentElement.dataset.theme = settings.baseTheme === 'system'
    ? (prefersDark.value ? 'dark' : 'light')
    : settings.baseTheme
})
```

`--ui-scale` from settings maps to the root font size, so all `rem`-based
shadcn spacing scales together.

---

## 2. Primitive Replacement Map

All entries below are generated with `npx shadcn-vue@latest add <name>` and land
in `src/components/ui/<name>/`.

| Legacy component | shadcn-vue replacement | Reka UI primitive | Install |
|------------------|------------------------|-------------------|---------|
| `FtButton` | **Button** | — (styled `<button>` + `Primitive`) | `add button` |
| `FtInput` | **Input** | — (styled `<input>`) | `add input` |
| `FtSelect` | **Select** | `SelectRoot` | `add select` |
| `FtCheckbox` | **Checkbox** | `CheckboxRoot` | `add checkbox` |
| `FtRadio` | **RadioGroup** | `RadioGroupRoot` | `add radio-group` |
| `FtToggle` | **Switch** | `SwitchRoot` | `add switch` |
| `FtSlider` | **Slider** | `SliderRoot` | `add slider` |
| `FtTooltip` | **Tooltip** | `TooltipRoot` | `add tooltip` |
| `FtToast` | **Toast** (`vue-sonner`) | — | `add sonner` |
| `FtPrompt` | **Dialog** / **AlertDialog** | `DialogRoot` / `AlertDialogRoot` | `add dialog alert-dialog` |
| `FtContextMenu` | **ContextMenu** | `ContextMenuRoot` | `add context-menu` |
| `FtTabs` | **Tabs** | `TabsRoot` | `add tabs` |
| `FtSidebar` | **Sheet** + **NavigationMenu** | `DialogRoot` (Sheet) / `NavigationMenuRoot` | `add sheet navigation-menu` |
| `FtAvatar` | **Avatar** | `AvatarRoot` | `add avatar` |
| `FtBadge` | **Badge** | — (styled `<span>`) | `add badge` |
| `FtSkeleton` | **Skeleton** | — | `add skeleton` |
| `FtSpinner` | **Spinner** | — | `add spinner` |
| `FtProgress` | **Progress** | `ProgressRoot` | `add progress` |

Bulk install:

```bash
npx shadcn-vue@latest add button input select checkbox radio-group switch \
  slider tooltip sonner dialog alert-dialog context-menu tabs sheet \
  navigation-menu avatar badge skeleton spinner progress
```

### 2.1 Button

| `FtButton` prop | Button equivalent |
|-----------------|-------------------|
| `label` | default slot |
| `textColor` / `backgroundColor` | `variant` |
| `theme="primary"` | `variant="default"` |
| `theme="secondary"` | `variant="secondary"` |
| `theme="destructive"` | `variant="destructive"` |
| `theme="text-only"` | `variant="ghost"` |
| `theme="link"` | `variant="link"` |
| `disabled` | `disabled` |
| `@click` | `@click` |
| `icon` | icon component in slot + `size="icon"` |

```vue
<!-- before -->
<ft-button label="Save" theme="primary" :disabled="saving" @click="save" />

<!-- after -->
<Button :disabled="saving" @click="save">Save</Button>
<Button variant="ghost" size="icon" aria-label="Refresh" @click="refresh">
  <HugeiconsIcon :icon="RefreshIcon" class="size-4" />
</Button>
```

Use `as-child` when the button must render a router link — this preserves
styling without nesting interactive elements:

```vue
<Button as-child variant="link">
  <RouterLink :to="`/channel/${channelId}`">{{ channelName }}</RouterLink>
</Button>
```

### 2.2 Input

`FtInput` bundled a label, clear button, and search-suggestion dropdown. That is
three responsibilities — split them:

- text field → `Input`
- label → `Label` (`add label`)
- suggestions → `Combobox` (`add combobox`) inside `SearchBar`

```vue
<div class="grid gap-2">
  <Label for="instance">Invidious instance</Label>
  <Input id="instance" v-model="instanceUrl" placeholder="https://yewtu.be" />
  <p class="text-muted-foreground text-sm">Used when backend is Invidious.</p>
</div>
```

`v-model` works directly — shadcn-vue `Input` uses `defineModel<string>()`.

### 2.3 Select

Reka's `Select` is portalled and typeahead-capable; the legacy `<select>` wrapper
was neither.

```vue
<Select v-model="quality">
  <SelectTrigger class="w-48">
    <SelectValue placeholder="Select quality" />
  </SelectTrigger>
  <SelectContent>
    <SelectGroup>
      <SelectLabel>Video quality</SelectLabel>
      <SelectItem v-for="q in qualities" :key="q.value" :value="q.value">
        {{ q.label }}
      </SelectItem>
    </SelectGroup>
  </SelectContent>
</Select>
```

> `SelectItem` values must be strings. For numeric settings (e.g. volume steps)
> convert at the store boundary, not in the template.

### 2.4 Checkbox / Switch / RadioGroup

FreeTube used `FtToggle` for both booleans and two-state choices. Split by
semantics:

- **Switch** — immediate-effect boolean setting (`Autoplay videos`)
- **Checkbox** — deferred/batched boolean, or multi-select list item
- **RadioGroup** — mutually exclusive choice of 3+ options

```vue
<!-- Switch: applies immediately -->
<div class="flex items-center justify-between py-3">
  <div class="space-y-0.5">
    <Label for="autoplay">Autoplay videos</Label>
    <p class="text-muted-foreground text-sm">Play the next video automatically.</p>
  </div>
  <Switch
    id="autoplay"
    :model-value="settings.values.autoplayVideos"
    @update:model-value="v => settings.updateSetting('autoplayVideos', v)"
  />
</div>

<!-- RadioGroup: backend preference -->
<RadioGroup v-model="backend" class="gap-3">
  <div class="flex items-center gap-2">
    <RadioGroupItem id="b-local" value="local" />
    <Label for="b-local">Local API</Label>
  </div>
  <div class="flex items-center gap-2">
    <RadioGroupItem id="b-invidious" value="invidious" />
    <Label for="b-invidious">Invidious</Label>
  </div>
</RadioGroup>
```

### 2.5 Slider

Reka `Slider` is array-based (supports ranges). Single-value settings must
wrap/unwrap:

```vue
<script setup lang="ts">
const volume = computed<number[]>({
  get: () => [player.volume * 100],
  set: ([v]) => player.setVolume((v ?? 0) / 100)
})
</script>

<template>
  <Slider v-model="volume" :min="0" :max="100" :step="1" class="w-40" />
</template>
```

> The player scrubber does **not** use this component — see
> §3 `ft-shaka-video-player`.

### 2.6 Tooltip

`TooltipProvider` must wrap the app once in `App.vue`; per-tooltip providers
break the shared open/close delay grouping.

```vue
<!-- App.vue -->
<TooltipProvider :delay-duration="300" :skip-delay-duration="150">
  <RouterView />
</TooltipProvider>
```

```vue
<Tooltip>
  <TooltipTrigger as-child>
    <Button variant="ghost" size="icon" aria-label="Add to queue">
      <HugeiconsIcon :icon="QueueIcon" class="size-4" />
    </Button>
  </TooltipTrigger>
  <TooltipContent>Add to watch queue</TooltipContent>
</Tooltip>
```

### 2.7 Toast (Sonner)

The `utils` store's `showToast` action becomes a thin wrapper so call sites
don't import the toast library directly:

```typescript
// src/stores/utils.ts
import { toast } from 'vue-sonner'

function showToast(message: string, options: ToastOptions = {}): void {
  const fn = options.type === 'error' ? toast.error
    : options.type === 'success' ? toast.success
    : toast
  fn(message, {
    duration: options.duration ?? 4000,
    action: options.action
      ? { label: options.action.label, onClick: options.action.handler }
      : undefined
  })
}
```

```vue
<!-- App.vue -->
<Toaster position="bottom-right" rich-colors close-button />
```

### 2.8 Dialog vs AlertDialog

| Use case | Component |
|----------|-----------|
| Non-destructive, dismissible (Add to playlist, Share, Create playlist) | `Dialog` |
| Destructive/blocking confirmation (Delete playlist, Clear history, Remove download) | `AlertDialog` |

`AlertDialog` traps focus, disables outside-click dismissal, and requires an
explicit action — correct for irreversible operations.

```vue
<AlertDialog v-model:open="confirmClear">
  <AlertDialogContent>
    <AlertDialogHeader>
      <AlertDialogTitle>Clear watch history?</AlertDialogTitle>
      <AlertDialogDescription>
        This removes {{ history.records.length }} entries. This cannot be undone.
      </AlertDialogDescription>
    </AlertDialogHeader>
    <AlertDialogFooter>
      <AlertDialogCancel>Cancel</AlertDialogCancel>
      <AlertDialogAction variant="destructive" @click="history.clearAll">
        Clear history
      </AlertDialogAction>
    </AlertDialogFooter>
  </AlertDialogContent>
</AlertDialog>
```

### 2.9 ContextMenu

Right-click menus on video/channel/playlist cards. Note the distinction:

- **ContextMenu** — right-click trigger (video card background)
- **DropdownMenu** — left-click trigger (the ⋮ button on a card)

Both are usually needed on the same card; share the item list via a small
`<VideoMenuItems>` component to avoid divergence.

```vue
<ContextMenu>
  <ContextMenuTrigger as-child>
    <FtListVideo :video="video" />
  </ContextMenuTrigger>
  <ContextMenuContent class="w-56">
    <ContextMenuItem @select="queue.enqueue(video)">Add to queue</ContextMenuItem>
    <ContextMenuSub>
      <ContextMenuSubTrigger>Add to playlist</ContextMenuSubTrigger>
      <ContextMenuSubContent>
        <ContextMenuItem
          v-for="p in playlists.userPlaylists" :key="p.id"
          @select="playlists.addVideo(p.id, video)"
        >{{ p.name }}</ContextMenuItem>
      </ContextMenuSubContent>
    </ContextMenuSub>
    <ContextMenuSeparator />
    <ContextMenuItem @select="copyLink(video)">Copy link</ContextMenuItem>
    <ContextMenuItem variant="destructive" @select="history.removeEntry(video.id)">
      Remove from history
    </ContextMenuItem>
  </ContextMenuContent>
</ContextMenu>
```

### 2.10 Tabs

Used by Subscriptions (Videos / Shorts / Live / Community), Channel, and
Settings section navigation. Bind `v-model` to the route query so tab state
survives navigation and reload:

```vue
<script setup lang="ts">
const route = useRoute()
const router = useRouter()
const tab = computed({
  get: () => (route.query.tab as string) ?? 'videos',
  set: (v) => router.replace({ query: { ...route.query, tab: v } })
})
</script>

<template>
  <Tabs v-model="tab">
    <TabsList>
      <TabsTrigger value="videos">Videos</TabsTrigger>
      <TabsTrigger value="shorts">Shorts</TabsTrigger>
      <TabsTrigger value="live">Live</TabsTrigger>
      <TabsTrigger value="community">Community</TabsTrigger>
    </TabsList>
    <TabsContent value="videos"><SubscriptionVideos /></TabsContent>
    <TabsContent value="shorts"><SubscriptionShorts /></TabsContent>
    <TabsContent value="live"><SubscriptionLive /></TabsContent>
    <TabsContent value="community"><SubscriptionCommunity /></TabsContent>
  </Tabs>
</template>
```

> `TabsContent` unmounts inactive panels by default. For expensive feeds that
> should retain scroll position, render the panel content inside `<KeepAlive>`.

### 2.11 Sidebar: Sheet + NavigationMenu

The legacy `FtSidebar` was one component handling both the desktop rail and the
mobile drawer. Split by breakpoint:

- **Desktop (≥ md):** persistent `<SideNav>` using `NavigationMenu` for
  keyboard-navigable sections + `Separator` between groups.
- **Mobile (< md):** `Sheet` with `side="left"`, triggered from `TopNav`,
  rendering the **same** `<SideNavContent>` child.

```vue
<!-- SideNav.vue -->
<template>
  <!-- desktop rail -->
  <aside class="hidden md:flex md:w-60 md:shrink-0 md:flex-col border-r">
    <SideNavContent />
  </aside>

  <!-- mobile drawer -->
  <Sheet v-model:open="utils.isSideNavOpen">
    <SheetContent side="left" class="w-64 p-0">
      <SheetHeader class="sr-only">
        <SheetTitle>Navigation</SheetTitle>
      </SheetHeader>
      <SideNavContent @navigate="utils.isSideNavOpen = false" />
    </SheetContent>
  </Sheet>
</template>
```

`SideNavContent` uses `NavigationMenu` for the primary links and plain
`RouterLink` rows for the dynamic profile/playlist lists.

### 2.12 Avatar / Badge / Skeleton / Spinner / Progress

```vue
<!-- Avatar: channel thumbnails with initials fallback -->
<Avatar class="size-10">
  <AvatarImage :src="channel.thumbnailUrl" :alt="channel.name" loading="lazy" />
  <AvatarFallback>{{ initials(channel.name) }}</AvatarFallback>
</Avatar>

<!-- Badge: LIVE / 4K / Members-only / New -->
<Badge variant="destructive" v-if="video.isLive">LIVE</Badge>
<Badge variant="secondary" v-if="video.isUpcoming">Upcoming</Badge>
<Badge variant="outline">{{ video.qualityLabel }}</Badge>

<!-- Skeleton: feed loading placeholder -->
<div v-for="n in 12" :key="n" class="space-y-2">
  <Skeleton class="aspect-video w-full rounded-md" />
  <Skeleton class="h-4 w-3/4" />
  <Skeleton class="h-3 w-1/2" />
</div>

<!-- Spinner: inline/button-level async -->
<Button :disabled="saving">
  <Spinner v-if="saving" class="size-4" />
  {{ saving ? 'Saving…' : 'Save' }}
</Button>

<!-- Progress: downloads -->
<Progress :model-value="progress.percent" class="h-1.5" />
```

**Skeleton vs Spinner policy:**

| Situation | Use |
|-----------|-----|
| Initial load of a list/grid with known layout | `Skeleton` |
| Loading more items (infinite scroll) | `Spinner` at list foot |
| Button-scoped async action | `Spinner` inside `Button` |
| Determinate long-running task (download, import) | `Progress` |
| Route-level navigation | thin top `Progress` bar driven by `utils` store |

---

## 3. Components Kept (Domain-Specific)

These encode YouTube/FreeTube behaviour with no Reka equivalent. They are
**rewritten** (SCSS → Tailwind, Options API → `<script setup lang="ts">`, Vuex →
Pinia) but **not replaced**.

| Component | Location | Responsibility | Rewrite notes |
|-----------|----------|----------------|---------------|
| `ft-card` | `components/ft-card.vue` | Generic elevated surface used across settings & lists | Thin wrapper over shadcn `Card`; keep the name to avoid a mass rename, re-export `Card` internals |
| `ft-flex-box` | `components/ft-flex-box.vue` | Responsive wrapping flex/grid container for card lists | Replace SCSS with Tailwind `grid` + container queries; keep the `:wrap`/`:justify` props |
| `ft-shaka-video-player` | `components/ft-shaka-video-player/` | Shaka Player wrapper: DASH/HLS, quality, captions, SponsorBlock skips, keyboard shortcuts, PiP, stats-for-nerds | **Highest-risk component.** Do not attempt to rebuild controls from shadcn primitives — Shaka owns the control surface. Only restyle the custom overlay layer |
| `TabBar` | `components/TabBar.vue` | In-app tab strip (open videos/channels), drag-to-reorder, close buttons | Keep custom; Reka `Tabs` cannot express closable/reorderable dynamic tabs. May use `useSortable` |
| `SideNav` | `components/SideNav.vue` | Primary navigation shell | Recomposed over `Sheet` + `NavigationMenu` (see §2.11) but remains a project component |
| `TopNav` | `components/TopNav.vue` | Search bar, back/forward, profile switcher, window controls | Composed from `Input`/`Combobox`/`DropdownMenu`/`Avatar`; keeps Tauri window-control wiring |
| `FtListVideo` | `components/FtListVideo.vue` | Video card: thumbnail, duration pill, watch-progress bar, title, channel, views, age, watched state, menu | Core list primitive — must support `list` and `grid` layouts |
| `FtListChannel` | `components/FtListChannel.vue` | Channel card: avatar, name, handle, sub count, subscribe button | Uses `Avatar` + `Button` internally |
| `FtListPlaylist` | `components/FtListPlaylist.vue` | Playlist card: stacked thumbnail, item count, owner | Uses `Badge` for count overlay |

### 3.1 `ft-card` as a compatibility wrapper

```vue
<!-- components/ft-card.vue -->
<script setup lang="ts">
import { Card, CardContent } from '@/components/ui/card'
import { cn } from '@/lib/utils'

withDefaults(defineProps<{ class?: string; padded?: boolean }>(), {
  padded: true
})
</script>

<template>
  <Card :class="cn('overflow-hidden', $props.class)">
    <CardContent :class="padded ? 'p-4' : 'p-0'">
      <slot />
    </CardContent>
  </Card>
</template>
```

This lets hundreds of existing `<ft-card>` usages migrate without edits while
still landing on shadcn styling.

### 3.2 `FtListVideo` composition

```vue
<script setup lang="ts">
import type { Video } from '@/types/models'

const props = defineProps<{
  video: Video
  layout?: 'grid' | 'list'
  showChannel?: boolean
}>()
</script>

<template>
  <article
    :class="cn(
      'group relative gap-3',
      layout === 'list' ? 'flex' : 'flex flex-col'
    )"
  >
    <RouterLink :to="`/watch/${video.id}`" class="relative block shrink-0">
      <img
        :src="thumbnailUrl" :alt="video.title" loading="lazy"
        :class="cn('aspect-video rounded-md object-cover',
                   layout === 'list' ? 'w-40' : 'w-full')"
      />
      <Badge v-if="video.isLive" variant="destructive"
             class="absolute bottom-1 right-1">LIVE</Badge>
      <span v-else
            class="absolute bottom-1 right-1 rounded bg-black/80 px-1
                   text-xs font-medium text-white">
        {{ formatDuration(video.durationSeconds) }}
      </span>
      <Progress v-if="watchProgress > 0" :model-value="watchProgress"
                class="absolute inset-x-0 bottom-0 h-1 rounded-none" />
    </RouterLink>

    <div class="min-w-0 flex-1 space-y-1">
      <h3 class="line-clamp-2 text-sm font-medium leading-snug">{{ video.title }}</h3>
      <RouterLink v-if="showChannel" :to="`/channel/${video.channelId}`"
                  class="text-muted-foreground hover:text-foreground block truncate text-xs">
        {{ video.channelName }}
      </RouterLink>
      <p class="text-muted-foreground text-xs">
        {{ formatViews(video.viewCount) }} · {{ formatAge(video.publishedAt) }}
      </p>
    </div>

    <DropdownMenu>
      <DropdownMenuTrigger as-child>
        <Button variant="ghost" size="icon" aria-label="Video options"
                class="opacity-0 transition group-hover:opacity-100
                       focus-visible:opacity-100">
          <HugeiconsIcon :icon="MoreVerticalIcon" class="size-4" />
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end"><VideoMenuItems :video="video" /></DropdownMenuContent>
    </DropdownMenu>
  </article>
</template>
```

---

## 4. Conventions

### 4.1 Ownership of `src/components/ui/`

Generated primitives are **vendored source, not a dependency**. Editing them is
allowed and expected, but:

- Record every edit in [CHANGELOG.md](CHANGELOG.md) so re-running
  `shadcn-vue add` doesn't silently revert it.
- Prefer extending via `cva` variants over forking a component.
- Never add domain logic (stores, API calls) inside `ui/` — it must stay
  app-agnostic.

Adding a variant, the sanctioned way:

```typescript
// src/components/ui/badge/index.ts
export const badgeVariants = cva(base, {
  variants: {
    variant: {
      default: '…',
      secondary: '…',
      destructive: '…',
      outline: '…',
      // project-specific additions — documented in CHANGELOG
      live: 'border-transparent bg-red-600 text-white',
      members: 'border-transparent bg-emerald-600 text-white'
    }
  }
})
```

### 4.2 Class merging

Always merge incoming classes with `cn()` so callers can override:

```typescript
// src/lib/utils.ts (already present)
import { type ClassValue, clsx } from 'clsx'
import { twMerge } from 'tailwind-merge'
export function cn(...inputs: ClassValue[]) { return twMerge(clsx(inputs)) }
```

### 4.3 Icons

`HugeiconsIcon` takes an icon object, not a name string. Import icons
individually — never `import * as icons` (kills tree-shaking):

```vue
<script setup lang="ts">
import { HugeiconsIcon } from '@hugeicons/vue'
import { Search01Icon, Settings02Icon } from '@hugeicons/core-free-icons'
</script>
<template>
  <HugeiconsIcon :icon="Search01Icon" class="size-4" :stroke-width="1.5" />
</template>
```

New York style expects `size-4` (16px) for inline icons and `size-5` for nav.

### 4.4 Accessibility baseline

Non-negotiable for every migrated component:

- Icon-only `Button` requires `aria-label`.
- Every `Input`/`Select`/`Switch` pairs with a `Label` via `for`/`id`.
- `DialogContent` and `SheetContent` require a `DialogTitle`/`SheetTitle`
  (use `class="sr-only"` if visually hidden) — Reka warns otherwise.
- Loading regions use `aria-busy="true"`; toasts are `aria-live` via Sonner.
- Focus rings are never removed; use `focus-visible:ring-ring/50` from the
  New York defaults.

### 4.5 Migration checklist per component

- [ ] Legacy `Ft*` file deleted (no parallel implementations)
- [ ] All usages updated (`rg '<ft-<name>' src/` returns nothing)
- [ ] Props mapped; removed props documented in this file
- [ ] SCSS module deleted; styling is Tailwind-only
- [ ] Dark / black / dracula themes visually verified
- [ ] Keyboard path verified (Tab, Shift+Tab, Enter, Esc, arrows)
- [ ] `vue-tsc --noEmit` passes

---

## 5. Removed Without Replacement

| Legacy component | Reason |
|------------------|--------|
| `FtIconButton` | `Button variant="ghost" size="icon"` |
| `FtLoader` | Split into `Skeleton` / `Spinner` |
| `FtFlexBox` variants (`ft-auto-grid`) | Tailwind `grid-cols-*` + container queries |
| `FtNotificationBanner` | Sonner toast with `action` |
| `FtToggleSwitch` | Duplicate of `FtToggle` → `Switch` |
| `FtSelectWithLabel` | `Label` + `Select` composition |
| `FtRefreshWidget` | `Button` + `Spinner` composition |
| Custom SCSS theme mixins | CSS variables in `style.css` |

---

## References

- [shadcn-vue — Installation (Vite)](https://www.shadcn-vue.com/docs/installation/vite)
- [shadcn-vue — Theming](https://www.shadcn-vue.com/docs/theming)
- [Reka UI — Components](https://reka-ui.com/docs/components/accordion)
- [Tailwind CSS v4 — Theme variables](https://tailwindcss.com/docs/theme)
- [01-store-migration.md](01-store-migration.md) — State layer these components bind to
- [03-view-migration-order.md](03-view-migration-order.md) — Where each component is first needed

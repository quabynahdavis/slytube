<script setup lang="ts">
import { computed } from 'vue'
import { useRoute } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { cn } from '@/lib/utils'
import { useSettingsStore } from '@/stores/settings'
import {
  PhHouse as House,
  PhTrendUp as TrendUp,
  PhWifiHigh as WifiHigh,
  PhClockCounterClockwise as ClockCounterClockwise,
  PhPlaylist as Playlist,
  PhDownloadSimple as DownloadSimple,
  PhClock as Clock,
  PhGear as Gear,
  PhInfo as Info,
} from '@phosphor-icons/vue'

const { t } = useI18n()
const route = useRoute()
const settingsStore = useSettingsStore()

interface NavItem {
  name: string
  icon: typeof House
  route: string
}

const mainNavItems = computed<NavItem[]>(() => [
  { name: t('nav.home'), icon: House, route: '/' },
  { name: t('nav.trending'), icon: TrendUp, route: '/trending' },
  { name: t('nav.subscriptions'), icon: WifiHigh, route: '/subscriptions' },
])

const libraryNavItems = computed<NavItem[]>(() => [
  { name: t('nav.history'), icon: ClockCounterClockwise, route: '/history' },
  { name: t('nav.playlists'), icon: Playlist, route: '/playlists' },
  { name: t('nav.downloads'), icon: DownloadSimple, route: '/downloads' },
  { name: t('nav.watchLater'), icon: Clock, route: '/playlist/watch-later' },
])

const bottomNavItems = computed<NavItem[]>(() => [
  { name: t('nav.settings'), icon: Gear, route: '/settings' },
  { name: t('nav.about'), icon: Info, route: '/about' },
])

const isExpanded = computed(() => settingsStore.expandSideBar)
</script>

<template>
  <aside
    :class="cn(
      'flex flex-col h-full bg-card border-r border-border transition-all duration-200 z-30',
      isExpanded ? 'w-56' : 'w-16'
    )"
  >
    <!-- Main Navigation -->
    <nav class="flex-1 overflow-y-auto py-2">
      <ul class="space-y-1 px-2">
        <li v-for="item in mainNavItems" :key="item.route">
          <router-link
            :to="item.route"
            :class="cn(
              'flex items-center gap-3 rounded-lg px-3 py-2 text-sm font-medium text-muted-foreground transition-colors hover:bg-accent hover:text-accent-foreground',
              route.path === item.route && 'bg-accent text-accent-foreground'
            )"
            :title="item.name"
          >
            <component :is="item.icon" :size="20" class="shrink-0" />
            <span v-if="isExpanded" class="truncate">{{ item.name }}</span>
          </router-link>
        </li>
      </ul>

      <div class="my-4 border-t border-border mx-2" />

      <!-- Library Section -->
      <div v-if="isExpanded" class="px-4 pb-2">
        <h3 class="text-xs font-semibold uppercase tracking-wider text-muted-foreground">
          {{ t('nav.library') }}
        </h3>
      </div>
      <ul class="space-y-1 px-2">
        <li v-for="item in libraryNavItems" :key="item.route">
          <router-link
            :to="item.route"
            :class="cn(
              'flex items-center gap-3 rounded-lg px-3 py-2 text-sm font-medium text-muted-foreground transition-colors hover:bg-accent hover:text-accent-foreground',
              route.path === item.route && 'bg-accent text-accent-foreground'
            )"
            :title="item.name"
          >
            <component :is="item.icon" :size="20" class="shrink-0" />
            <span v-if="isExpanded" class="truncate">{{ item.name }}</span>
          </router-link>
        </li>
      </ul>
    </nav>

    <!-- Bottom Navigation -->
    <nav class="border-t border-border py-2">
      <ul class="space-y-1 px-2">
        <li v-for="item in bottomNavItems" :key="item.route">
          <router-link
            :to="item.route"
            :class="cn(
              'flex items-center gap-3 rounded-lg px-3 py-2 text-sm font-medium text-muted-foreground transition-colors hover:bg-accent hover:text-accent-foreground',
              route.path === item.route && 'bg-accent text-accent-foreground'
            )"
            :title="item.name"
          >
            <component :is="item.icon" :size="20" class="shrink-0" />
            <span v-if="isExpanded" class="truncate">{{ item.name }}</span>
          </router-link>
        </li>
      </ul>
    </nav>
  </aside>
</template>

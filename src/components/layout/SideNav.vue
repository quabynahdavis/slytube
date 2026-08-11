<script setup lang="ts">
import { computed } from 'vue'
import { useRoute } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { cn } from '@/lib/utils'
import { useSettingsStore } from '@/stores/settings'

const { t } = useI18n()
const route = useRoute()
const settingsStore = useSettingsStore()

interface NavItem {
  name: string
  icon: string
  route: string
  badge?: number
}

const mainNavItems = computed<NavItem[]>(() => [
  { name: t('nav.home'), icon: 'home', route: '/' },
  { name: t('nav.trending'), icon: 'trending', route: '/trending' },
  { name: t('nav.subscriptions'), icon: 'subscriptions', route: '/subscriptions' },
])

const libraryNavItems = computed<NavItem[]>(() => [
  { name: t('nav.history'), icon: 'history', route: '/history' },
  { name: t('nav.playlists'), icon: 'playlists', route: '/playlists' },
  { name: t('nav.downloads'), icon: 'downloads', route: '/downloads' },
  { name: t('nav.watchLater'), icon: 'watch-later', route: '/playlist/watch-later' },
])

const bottomNavItems = computed<NavItem[]>(() => [
  { name: t('nav.settings'), icon: 'settings', route: '/settings' },
  { name: t('nav.about'), icon: 'about', route: '/about' },
])

const isExpanded = computed(() => settingsStore.expandSideBar)

function toggleSidebar() {
  settingsStore.updateSetting('expandSideBar', !settingsStore.expandSideBar)
}

function getIcon(icon: string): string {
  const icons: Record<string, string> = {
    home: 'M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-6 0a1 1 0 001-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 001 1m-6 0h6',
    trending: 'M13 7h8m0 0v8m0-8l-8 8-4-4-6 6',
    subscriptions: 'M19 11a7 7 0 01-7 7m0 0a7 7 0 01-7-7m7 7v4m0 0H8m4 0h4m-4-8a3 3 0 01-3-3V5a3 3 0 116 0v6a3 3 0 01-3 3z',
    history: 'M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z',
    playlists: 'M4 6h16M4 10h16M4 14h16M4 18h16',
    downloads: 'M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4',
    'watch-later': 'M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z',
    settings: 'M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.066 2.573c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.573 1.066c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.066-2.573c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z M15 12a3 3 0 11-6 0 3 3 0 016 0z',
    about: 'M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z',
  }
  return icons[icon] || icons.home
}
</script>

<template>
  <aside
    :class="cn(
      'flex flex-col h-full bg-card border-r border-border transition-all duration-200',
      isExpanded ? 'w-56' : 'w-16'
    )"
  >
    <!-- Hamburger / Toggle -->
    <div class="flex items-center justify-between p-3 border-b border-border">
      <button
        class="p-1.5 rounded-lg hover:bg-accent transition-colors"
        @click="toggleSidebar"
        :title="isExpanded ? 'Collapse sidebar' : 'Expand sidebar'"
      >
        <svg class="size-5 text-foreground" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
          <path stroke-linecap="round" stroke-linejoin="round" d="M4 6h16M4 12h16M4 18h16" />
        </svg>
      </button>
      <span v-if="isExpanded" class="text-sm font-semibold text-foreground">SlyTube</span>
    </div>

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
            <svg class="size-5 shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
              <path stroke-linecap="round" stroke-linejoin="round" :d="getIcon(item.icon)" />
            </svg>
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
            <svg class="size-5 shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
              <path stroke-linecap="round" stroke-linejoin="round" :d="getIcon(item.icon)" />
            </svg>
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
            <svg class="size-5 shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
              <path stroke-linecap="round" stroke-linejoin="round" :d="getIcon(item.icon)" />
            </svg>
            <span v-if="isExpanded" class="truncate">{{ item.name }}</span>
          </router-link>
        </li>
      </ul>
    </nav>
  </aside>
</template>

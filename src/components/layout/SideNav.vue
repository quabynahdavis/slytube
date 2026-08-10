<script setup lang="ts">
import { computed } from 'vue'
import { useRoute } from 'vue-router'
import { cn } from '@/lib/utils'
import { useSettingsStore } from '@/stores/settings'
import { useSubscriptionsStore } from '@/stores/subscriptions'

const route = useRoute()
const settingsStore = useSettingsStore()
const subscriptionsStore = useSubscriptionsStore()

interface NavItem {
  name: string
  icon: string
  route: string
  badge?: number
}

const mainNavItems = computed<NavItem[]>(() => [
  { name: 'Home', icon: 'home', route: '/' },
  { name: 'Trending', icon: 'trending', route: '/trending' },
  { name: 'Subscriptions', icon: 'subscriptions', route: '/subscriptions', badge: subscriptionsStore.getSubscriptionCacheReady ? undefined : 0 },
])

const libraryNavItems = computed<NavItem[]>(() => [
  { name: 'History', icon: 'history', route: '/history' },
  { name: 'Playlists', icon: 'playlists', route: '/playlists' },
  { name: 'Downloads', icon: 'downloads', route: '/downloads' },
  { name: 'Watch Later', icon: 'watch-later', route: '/playlist/watch-later' },
])

const bottomNavItems = computed<NavItem[]>(() => [
  { name: 'Settings', icon: 'settings', route: '/settings' },
  { name: 'About', icon: 'about', route: '/about' },
])

const isExpanded = computed(() => settingsStore.expandSideBar)
</script>

<template>
  <aside
    :class="cn(
      'flex flex-col h-full bg-card border-r border-border transition-all duration-200',
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
            <span class="shrink-0 size-5 flex items-center justify-center">
              <svg class="size-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <circle cx="12" cy="12" r="10" />
              </svg>
            </span>
            <span v-if="isExpanded" class="truncate">{{ item.name }}</span>
            <span
              v-if="isExpanded && item.badge"
              class="ml-auto flex size-5 items-center justify-center rounded-full bg-primary text-xs text-primary-foreground"
            >
              {{ item.badge }}
            </span>
          </router-link>
        </li>
      </ul>

      <div v-if="isExpanded" class="my-4 border-t border-border mx-4" />
      <div v-else class="my-4 border-t border-border mx-2" />

      <!-- Library Section -->
      <div v-if="isExpanded" class="px-4 pb-2">
        <h3 class="text-xs font-semibold uppercase tracking-wider text-muted-foreground">
          Library
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
            <span class="shrink-0 size-5 flex items-center justify-center">
              <svg class="size-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <rect x="3" y="3" width="18" height="18" rx="2" />
              </svg>
            </span>
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
            <span class="shrink-0 size-5 flex items-center justify-center">
              <svg class="size-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <path d="M12 20h9" />
                <path d="M16.5 3.5a2.121 2.121 0 0 1 3 3L7 19l-4 1 1-4L16.5 3.5z" />
              </svg>
            </span>
            <span v-if="isExpanded" class="truncate">{{ item.name }}</span>
          </router-link>
        </li>
      </ul>
    </nav>
  </aside>
</template>

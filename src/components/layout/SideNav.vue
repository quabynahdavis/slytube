<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { useRoute } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { cn } from '@/lib/utils'
import { useSettingsStore } from '@/stores/settings'
import { useSubscriptionsStore } from '@/stores/subscriptions'
import { getChannelInfo } from '@/api'
import {
  PhHouse,
  PhTrendUp,
  PhPlayCircle,
  PhNewspaper,
  PhWifiHigh,
  PhClockCounterClockwise,
  PhPlaylist,
  PhDownloadSimple,
  PhClock,
  PhGear,
  PhInfo,
  PhCaretRight,
} from '@phosphor-icons/vue'

const { t } = useI18n()
const route = useRoute()
const settingsStore = useSettingsStore()
const subscriptionsStore = useSubscriptionsStore()

const props = defineProps<{
  mode?: 'normal' | 'overlay'
}>()

const emit = defineEmits<{
  close: []
}>()

function handleNavigate() {
  if (props.mode === 'overlay') {
    emit('close')
  }
}

interface NavItem {
  name: string
  icon: typeof PhHouse
  route: string
}

interface ChannelNavItem {
  id: string
  name: string
  avatar: string
}

const mainNavItems = computed<NavItem[]>(() => [
  { name: t('nav.home'), icon: PhHouse, route: '/' },
  { name: t('nav.trending'), icon: PhTrendUp, route: '/trending' },
  { name: t('nav.shorts'), icon: PhPlayCircle, route: '/shorts' },
  { name: t('nav.posts'), icon: PhNewspaper, route: '/posts' },
])

const libraryNavItems = computed<NavItem[]>(() => [
  { name: t('nav.history'), icon: PhClockCounterClockwise, route: '/history' },
  { name: t('nav.playlists'), icon: PhPlaylist, route: '/playlists' },
  { name: t('nav.downloads'), icon: PhDownloadSimple, route: '/downloads' },
  { name: t('nav.watchLater'), icon: PhClock, route: '/playlist/watch-later' },
])

const bottomNavItems = computed<NavItem[]>(() => [
  { name: t('nav.settings'), icon: PhGear, route: '/settings' },
  { name: t('nav.about'), icon: PhInfo, route: '/about' },
])

const isExpanded = computed(() => settingsStore.expandSideBar)

const subscribedChannels = computed(() => subscriptionsStore.subscribedChannelIds)

const channelInfoMap = ref<Record<string, { name: string; avatar: string }>>({})

const subscribedChannelList = computed(() => {
  const channels: ChannelNavItem[] = []
  for (const channelId of Array.from(subscribedChannels.value).slice(0, 5)) {
    const info = channelInfoMap.value[channelId]
    channels.push({
      id: channelId,
      name: info?.name || channelId,
      avatar: info?.avatar || '',
    })
  }
  return channels
})

async function loadChannelInfo() {
  const channelIds = Array.from(subscribedChannels.value).slice(0, 5)
  for (const channelId of channelIds) {
    if (!channelInfoMap.value[channelId]) {
      try {
        const channel = await getChannelInfo(channelId)
        if (channel) {
          channelInfoMap.value[channelId] = {
            name: channel.name,
            avatar: channel.avatar,
          }
        }
      } catch {
        // Keep default if fetch fails
      }
    }
  }
}

onMounted(loadChannelInfo)
watch(subscribedChannels, loadChannelInfo)
</script>

<template>
  <aside
    :class="cn(
      'flex flex-col h-full bg-card border-r border-border transition-all duration-200 z-30',
      isExpanded ? 'w-56' : 'w-16'
    )"
  >
    <!-- Main Navigation -->
    <nav class="py-2">
      <ul class="space-y-0.5 px-2">
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
    </nav>

    <div class="my-2 border-t border-border mx-2" />

    <!-- Library Section -->
    <nav class="flex-1 overflow-y-auto py-2">
      <!-- Subscriptions Section -->
      <div v-if="isExpanded" class="px-4 pb-2">
        <h3 class="text-xs font-semibold uppercase tracking-wider text-muted-foreground">
          {{ t('nav.subscriptions') }}
        </h3>
      </div>
      <ul class="space-y-0.5 px-2">
        <!-- Subscriptions Link -->
        <li>
          <router-link
            to="/subscriptions"
            :class="cn(
              'flex items-center justify-between gap-3 rounded-lg px-3 py-2 text-sm font-medium text-muted-foreground transition-colors hover:bg-accent hover:text-accent-foreground',
              route.path === '/subscriptions' && 'bg-accent text-accent-foreground'
            )"
            :title="t('nav.subscriptions')"
          >
            <div class="flex items-center gap-3 min-w-0">
              <PhWifiHigh :size="20" class="shrink-0" />
              <span class="truncate">{{ t('nav.subscriptions') }}</span>
            </div>
            <PhCaretRight :size="14" v-if="isExpanded" class="truncate shrink-0 text-muted-foreground" />
          </router-link>
        </li>
        <!-- Channel List -->
        <li v-for="channel in subscribedChannelList" :key="channel.id">
          <router-link
            :to="`/channel/${channel.id}`"
            class="flex items-center gap-3 rounded-lg px-3 py-1.5 text-sm text-muted-foreground transition-colors hover:bg-accent hover:text-accent-foreground"
            :title="channel.name"
          >
            <div class="size-6 rounded-full bg-muted flex items-center justify-center shrink-0 overflow-hidden">
              <img
                v-if="channel.avatar"
                :src="channel.avatar"
                :alt="channel.name"
                class="w-full h-full object-cover"
              />
              <span v-else class="text-[10px] font-medium text-muted-foreground">{{ channel.name[0] || '?' }}</span>
            </div>
            <span v-if="isExpanded" class="truncate text-xs">{{ channel.name }}</span>
          </router-link>
        </li>
        <!-- Show All -->
        <li v-if="subscribedChannels.size > 5">
          <button
            class="flex items-center gap-3 rounded-lg px-3 py-1.5 text-xs text-muted-foreground transition-colors hover:bg-accent hover:text-accent-foreground w-full text-left"
          >
            <PhCaretRight :size="14" class="shrink-0" />
            <span v-if="isExpanded">{{ t('nav.showAll') }}</span>
          </button>
        </li>
      </ul>

      <div class="my-2 border-t border-border mx-2" />

      <!-- Library Section -->
      <div v-if="isExpanded" class="px-4 pb-2 pt-1">
        <h3 class="text-xs font-semibold uppercase tracking-wider text-muted-foreground">
          {{ t('nav.library') }}
        </h3>
      </div>
      <ul class="space-y-0.5 px-2">
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
      <ul class="space-y-0.5 px-2">
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

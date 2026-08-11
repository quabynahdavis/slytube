<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { useRoute } from 'vue-router'
import { cn } from '@/lib/utils'
import { useChannelLoader, useSubscriptions } from '../composables/useData'
import { getChannelInfo } from '../api'
import type { Video } from '../api/types'
import VideoCard from '../components/VideoCard.vue'
import ErrorState from '../components/ui/ErrorState.vue'

const route = useRoute()

const channelId = computed(() => route.params.id as string || '')
const { channel, error, load } = useChannelLoader()
const { loadSubscriptions, subscribe, unsubscribe, isSubscribed } = useSubscriptions()

const isLoading = ref(true)
const activeTab = ref('home')
const channelVideos = ref<Video[]>([])

const tabs = [
  { id: 'home', label: 'Home' },
  { id: 'videos', label: 'Videos' },
  { id: 'shorts', label: 'Shorts' },
  { id: 'live', label: 'Live' },
  { id: 'playlists', label: 'Playlists' },
  { id: 'community', label: 'Community' },
]

async function loadChannelData() {
  if (!channelId.value) return
  isLoading.value = true
  try {
    await load(channelId.value)
    await loadSubscriptions()
    // Fetch full channel info including videos
    try {
      const fullChannel = await getChannelInfo(channelId.value)
      channelVideos.value = fullChannel.videos || []
    } catch {
      // Channel already loaded by useChannelLoader
    }
  } finally {
    isLoading.value = false
  }
}

onMounted(loadChannelData)

watch(channelId, loadChannelData)

function formatSubscribers(count: number): string {
  if (count >= 1000000) return `${(count / 1000000).toFixed(1)}M subscribers`
  if (count >= 1000) return `${(count / 1000).toFixed(1)}K subscribers`
  return `${count} subscribers`
}

function toggleSubscription() {
  if (!channel.value) return
  if (isSubscribed(channel.value.id)) {
    unsubscribe(channel.value.id)
  } else {
    subscribe(channel.value.id)
  }
}
</script>

<template>
  <div class="min-h-screen">
    <!-- Loading State -->
    <div v-if="isLoading" class="animate-pulse">
      <div class="h-48 bg-muted" />
      <div class="container mx-auto max-w-7xl px-4 py-4">
        <div class="flex items-center gap-4">
          <div class="size-20 rounded-full bg-muted" />
          <div class="space-y-2">
            <div class="h-5 w-48 rounded bg-muted" />
            <div class="h-4 w-32 rounded bg-muted" />
          </div>
        </div>
      </div>
    </div>

    <!-- Error State -->
    <ErrorState v-else-if="error" :message="error" retryable @retry="loadChannelData" />

    <template v-else-if="channel">
      <!-- Channel Banner -->
      <div class="relative h-48 bg-gradient-to-r from-primary/20 to-primary/5">
        <img
          v-if="channel.banner"
          :src="channel.banner"
          :alt="channel.name"
          class="absolute inset-0 w-full h-full object-cover"
        />
        <div v-else class="absolute inset-0 flex items-center justify-center">
          <svg class="size-16 text-muted-foreground/30" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1">
            <rect x="2" y="2" width="20" height="20" rx="2.18" ry="2.18" />
            <line x1="7" y1="2" x2="7" y2="22" />
            <line x1="17" y1="2" x2="17" y2="22" />
            <line x1="2" y1="12" x2="22" y2="12" />
          </svg>
        </div>
      </div>

      <!-- Channel Info -->
      <div class="container mx-auto max-w-7xl px-4 py-4">
        <div class="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
          <div class="flex items-center gap-4">
            <img
              v-if="channel.avatar"
              :src="channel.avatar"
              :alt="channel.name"
              class="size-20 -mt-10 rounded-full border-4 border-background object-cover shrink-0"
            />
            <div v-else class="size-20 -mt-10 rounded-full border-4 border-background bg-muted flex items-center justify-center shrink-0">
              <svg class="size-10 text-muted-foreground" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2" />
                <circle cx="12" cy="7" r="4" />
              </svg>
            </div>
            <div>
              <h1 class="text-xl font-bold text-foreground">{{ channel.name }}</h1>
              <p class="text-sm text-muted-foreground">
                {{ formatSubscribers(channel.subscriberCount) }} &middot; {{ channel.videoCount || channelVideos.length }} videos
              </p>
              <p class="text-xs text-muted-foreground mt-1 line-clamp-1">{{ channel.description }}</p>
            </div>
          </div>
          <button
            :class="cn(
              'h-9 rounded-full px-6 text-sm font-medium transition-colors shrink-0',
              isSubscribed(channel.id)
                ? 'border border-border bg-muted text-foreground hover:bg-muted/80'
                : 'bg-primary text-primary-foreground hover:bg-primary/90'
            )"
            @click="toggleSubscription"
          >
            {{ isSubscribed(channel.id) ? 'Subscribed' : 'Subscribe' }}
          </button>
        </div>

        <!-- Channel Tabs -->
        <div class="mt-6 border-b border-border">
          <nav class="flex gap-6 overflow-x-auto">
            <button
              v-for="tab in tabs"
              :key="tab.id"
              :class="cn(
                'pb-3 text-sm font-medium border-b-2 transition-colors whitespace-nowrap',
                activeTab === tab.id
                  ? 'border-primary text-foreground'
                  : 'border-transparent text-muted-foreground hover:text-foreground'
              )"
              @click="activeTab = tab.id"
            >
              {{ tab.label }}
            </button>
          </nav>
        </div>

        <!-- Tab Content -->
        <div class="mt-6">
          <!-- Home / Videos Tab -->
          <div v-if="activeTab === 'home' || activeTab === 'videos'">
            <div v-if="channelVideos.length > 0" class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
              <VideoCard v-for="video in channelVideos" :key="video.id" :video="video" />
            </div>
            <div v-else class="rounded-lg border border-border bg-card p-8 text-center text-muted-foreground">
              <svg class="size-12 mx-auto mb-3 text-muted-foreground/50" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
                <rect x="2" y="2" width="20" height="20" rx="2.18" ry="2.18" />
                <line x1="7" y1="2" x2="7" y2="22" />
                <line x1="17" y1="2" x2="17" y2="22" />
                <line x1="2" y1="12" x2="22" y2="12" />
              </svg>
              <p class="text-sm">No videos available</p>
            </div>
          </div>

          <!-- Shorts Tab -->
          <div v-else-if="activeTab === 'shorts'">
            <div class="rounded-lg border border-border bg-card p-8 text-center text-muted-foreground">
              <svg class="size-12 mx-auto mb-3 text-muted-foreground/50" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
                <path d="M12 1a3 3 0 0 0-3 3v8a3 3 0 0 0 6 0V4a3 3 0 0 0-3-3z" />
                <path d="M19 10v2a7 7 0 0 1-14 0v-2" />
                <line x1="12" y1="19" x2="12" y2="23" />
                <line x1="8" y1="23" x2="16" y2="23" />
              </svg>
              <p class="text-sm">No shorts available</p>
            </div>
          </div>

          <!-- Live Tab -->
          <div v-else-if="activeTab === 'live'">
            <div class="rounded-lg border border-border bg-card p-8 text-center text-muted-foreground">
              <svg class="size-12 mx-auto mb-3 text-muted-foreground/50" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
                <path d="M12 1a3 3 0 0 0-3 3v8a3 3 0 0 0 6 0V4a3 3 0 0 0-3-3z" />
                <path d="M19 10v2a7 7 0 0 1-14 0v-2" />
                <line x1="12" y1="19" x2="12" y2="23" />
                <line x1="8" y1="23" x2="16" y2="23" />
              </svg>
              <p class="text-sm">No live streams available</p>
            </div>
          </div>

          <!-- Playlists Tab -->
          <div v-else-if="activeTab === 'playlists'">
            <div class="rounded-lg border border-border bg-card p-8 text-center text-muted-foreground">
              <svg class="size-12 mx-auto mb-3 text-muted-foreground/50" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
                <rect x="2" y="2" width="20" height="20" rx="2.18" ry="2.18" />
                <line x1="7" y1="2" x2="7" y2="22" />
                <line x1="17" y1="2" x2="17" y2="22" />
                <line x1="2" y1="12" x2="22" y2="12" />
              </svg>
              <p class="text-sm">No playlists available</p>
            </div>
          </div>

          <!-- Community Tab -->
          <div v-else-if="activeTab === 'community'">
            <div class="rounded-lg border border-border bg-card p-8 text-center text-muted-foreground">
              <svg class="size-12 mx-auto mb-3 text-muted-foreground/50" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
                <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z" />
              </svg>
              <p class="text-sm">No community posts yet</p>
            </div>
          </div>
        </div>
      </div>
    </template>
  </div>
</template>

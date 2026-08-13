<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { cn } from '@/lib/utils'
import { useSubscriptions } from '../composables/useData'
import { getChannelInfo } from '../api'
import type { Video } from '../api/types'
import VideoCard from '../components/VideoCard.vue'
import SkeletonGrid from '../components/ui/SkeletonGrid.vue'
import EmptyState from '../components/ui/EmptyState.vue'
import ErrorState from '../components/ui/ErrorState.vue'

const { subscriptions, loadSubscriptions } = useSubscriptions()

const isLoading = ref(true)
const isRefreshing = ref(false)
const activeTab = ref('videos')
const feedError = ref<string | null>(null)

interface ChannelVideo extends Video {
  channelId?: string
}

const feedVideos = ref<ChannelVideo[]>([])

const tabs = [
  { id: 'videos', label: 'Videos' },
  { id: 'shorts', label: 'Shorts' },
  { id: 'live', label: 'Live' },
  { id: 'posts', label: 'Posts' },
]

async function loadFeed() {
  isLoading.value = true
  feedError.value = null
  try {
    await loadSubscriptions()
    // Fetch videos from each subscribed channel
    const videoPromises = subscriptions.value.slice(0, 10).map(async (channelId) => {
      try {
        const channel = await getChannelInfo(channelId)
        return (channel.videos || []).map(v => ({ ...v, channelId }))
      } catch {
        return []
      }
    })
    const results = await Promise.allSettled(videoPromises)
    const allVideos: ChannelVideo[] = []
    for (const result of results) {
      if (result.status === 'fulfilled') {
        allVideos.push(...result.value)
      }
    }
    // Sort by published date (most recent first)
    feedVideos.value = allVideos.sort((a, b) => {
      const dateA = new Date(a.published).getTime()
      const dateB = new Date(b.published).getTime()
      if (isNaN(dateA) && isNaN(dateB)) return 0
      if (isNaN(dateA)) return 1
      if (isNaN(dateB)) return -1
      return dateB - dateA
    })
  } catch (e: any) {
    feedError.value = e.message || 'Failed to load subscription feed'
  } finally {
    isLoading.value = false
  }
}

async function refreshFeed() {
  isRefreshing.value = true
  try {
    await loadFeed()
  } finally {
    isRefreshing.value = false
  }
}

onMounted(loadFeed)
</script>

<template>
  <div class="container mx-auto max-w-7xl px-3 py-4">
    <!-- Header -->
    <div class="flex items-center justify-between mb-6">
      <div>
        <h1 class="text-2xl font-bold text-foreground">Subscriptions</h1>
        <p class="text-sm text-muted-foreground mt-1">Latest videos from your subscribed channels</p>
      </div>
      <button
        :disabled="isRefreshing"
        class="inline-flex items-center gap-1 rounded-md border border-border px-3 py-2 text-sm text-muted-foreground hover:bg-accent transition-colors disabled:opacity-50"
        @click="refreshFeed"
      >
        <svg
          :class="cn('size-4', isRefreshing && 'animate-spin')"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
        >
          <polyline points="23 4 23 10 17 10" />
          <path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10" />
        </svg>
        Refresh
      </button>
    </div>

    <!-- Feed Tabs -->
    <div class="border-b border-border mb-6">
      <nav class="flex gap-6">
        <button
          v-for="tab in tabs"
          :key="tab.id"
          :class="cn(
            'pb-3 text-sm font-medium border-b-2 transition-colors',
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

    <!-- Loading State -->
    <SkeletonGrid v-if="isLoading" :count="8" />

    <!-- Error State -->
    <ErrorState v-else-if="feedError" :message="feedError" retryable @retry="loadFeed" />

    <!-- Videos Grid -->
    <template v-else-if="activeTab === 'videos'">
      <EmptyState v-if="subscriptions.length === 0" title="No subscriptions yet" action="Browse Channels" @action="$router.push('/trending')">
        Find channels you love and subscribe to see their latest videos here.
      </EmptyState>
      <EmptyState v-else-if="feedVideos.length === 0" title="No videos from your subscriptions">
        Your subscribed channels haven't posted yet. Check back later or explore more channels.
      </EmptyState>
      <div v-else class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-2">
        <VideoCard
          v-for="video in feedVideos"
          :key="video.id"
          :video="video"
        />
      </div>
    </template>

    <!-- Shorts Grid -->
    <div v-else-if="activeTab === 'shorts'">
      <EmptyState v-if="!isLoading" title="No shorts from your subscriptions">
        Shorts from channels you subscribe to will show up here.
      </EmptyState>
    </div>

    <!-- Live Tab -->
    <div v-else-if="activeTab === 'live'">
      <EmptyState v-if="!isLoading" title="No live streams right now">
        Live streams from your subscribed channels will appear here when they go live.
      </EmptyState>
    </div>

    <!-- Posts Tab -->
    <div v-else-if="activeTab === 'posts'">
      <EmptyState v-if="!isLoading" title="No community posts">
        Community updates from your subscribed channels will appear here.
      </EmptyState>
    </div>
  </div>
</template>

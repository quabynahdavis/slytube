<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { cn } from '@/lib/utils'
import { useSubscriptionsStore } from '@/stores/subscriptions'
import { useProfilesStore } from '@/stores/profiles'

const subscriptionsStore = useSubscriptionsStore()
const profilesStore = useProfilesStore()

const isLoading = ref(true)
const activeTab = ref('videos')
const isRefreshing = ref(false)

const videos = ref<Array<{
  videoId: string
  title: string
  author: string
  authorId: string
  viewCount: number
  lengthSeconds: number
  published: string
  videoThumbnails: Array<{ url: string; width: number; height: number }>
}>>([])

const tabs = [
  { id: 'videos', label: 'Videos' },
  { id: 'shorts', label: 'Shorts' },
  { id: 'live', label: 'Live' },
  { id: 'posts', label: 'Posts' },
]

onMounted(async () => {
  isLoading.value = true
  try {
    await new Promise((resolve) => setTimeout(resolve, 600))
    videos.value = Array.from({ length: 20 }, (_, i) => ({
      videoId: `sub-video-${i}`,
      title: `Subscription Video ${i + 1} - New Upload`,
      author: `Subscribed Channel ${i + 1}`,
      authorId: `UC-sub-${i}`,
      viewCount: Math.floor(Math.random() * 1000000),
      lengthSeconds: Math.floor(Math.random() * 600) + 60,
      published: new Date(Date.now() - Math.random() * 7 * 24 * 60 * 60 * 1000).toISOString().split('T')[0],
      videoThumbnails: [{ url: '', width: 640, height: 360 }],
    }))
  } finally {
    isLoading.value = false
  }
})

async function refreshFeed() {
  isRefreshing.value = true
  subscriptionsStore.setSubscriptionFeedRefreshInProgress(true)
  try {
    await new Promise((resolve) => setTimeout(resolve, 1500))
    // Refresh logic placeholder
  } finally {
    isRefreshing.value = false
    subscriptionsStore.setSubscriptionFeedRefreshInProgress(false)
  }
}

function formatViews(views: number): string {
  if (views >= 1000000) return `${(views / 1000000).toFixed(1)}M views`
  if (views >= 1000) return `${(views / 1000).toFixed(1)}K views`
  return `${views} views`
}

function formatDuration(seconds: number): string {
  const h = Math.floor(seconds / 3600)
  const m = Math.floor((seconds % 3600) / 60)
  const s = seconds % 60
  if (h > 0) return `${h}:${m.toString().padStart(2, '0')}:${s.toString().padStart(2, '0')}`
  return `${m}:${s.toString().padStart(2, '0')}`
}

function timeAgo(dateStr: string): string {
  const diff = Date.now() - new Date(dateStr).getTime()
  const days = Math.floor(diff / (1000 * 60 * 60 * 24))
  if (days > 0) return `${days} days ago`
  const hours = Math.floor(diff / (1000 * 60 * 60))
  if (hours > 0) return `${hours} hours ago`
  return 'Just now'
}
</script>

<template>
  <div class="container mx-auto max-w-7xl px-4 py-6">
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
    <div v-if="isLoading" class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
      <div v-for="n in 8" :key="n" class="animate-pulse">
        <div class="aspect-video rounded-lg bg-muted" />
        <div class="mt-3 space-y-2">
          <div class="h-4 w-3/4 rounded bg-muted" />
          <div class="h-3 w-1/2 rounded bg-muted" />
        </div>
      </div>
    </div>

    <!-- Videos Grid -->
    <div v-else-if="activeTab === 'videos'">
      <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
        <router-link
          v-for="video in videos"
          :key="video.videoId"
          :to="`/watch?v=${video.videoId}`"
          class="group block"
        >
          <div class="relative aspect-video rounded-lg bg-muted overflow-hidden">
            <div class="absolute inset-0 flex items-center justify-center">
              <svg class="size-12 text-muted-foreground/50 group-hover:text-primary transition-colors" viewBox="0 0 24 24" fill="currentColor">
                <polygon points="5 3 19 12 5 21 5 3" />
              </svg>
            </div>
            <span class="absolute bottom-2 right-2 rounded bg-black/80 px-1.5 py-0.5 text-xs text-white font-medium">
              {{ formatDuration(video.lengthSeconds) }}
            </span>
          </div>
          <div class="mt-2">
            <h3 class="text-sm font-medium text-foreground line-clamp-2 group-hover:text-primary transition-colors">
              {{ video.title }}
            </h3>
            <p class="mt-1 text-xs text-muted-foreground hover:text-foreground">{{ video.author }}</p>
            <p class="text-xs text-muted-foreground">{{ formatViews(video.viewCount) }} &middot; {{ timeAgo(video.published) }}</p>
          </div>
        </router-link>
      </div>
    </div>

    <!-- Shorts Grid -->
    <div v-else-if="activeTab === 'shorts'">
      <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-3">
        <router-link
          v-for="n in 12"
          :key="n"
          :to="`/watch?v=short-${n}`"
          class="group block"
        >
          <div class="relative aspect-[9/16] rounded-lg bg-muted overflow-hidden">
            <div class="absolute inset-0 flex items-center justify-center">
              <svg class="size-10 text-muted-foreground/50 group-hover:text-primary transition-colors" viewBox="0 0 24 24" fill="currentColor">
                <polygon points="5 3 19 12 5 21 5 3" />
              </svg>
            </div>
          </div>
          <p class="mt-2 text-sm font-medium text-foreground line-clamp-2 group-hover:text-primary transition-colors">
            Short Video {{ n }}
          </p>
          <p class="text-xs text-muted-foreground">{{ formatViews(Math.floor(Math.random() * 1000000)) }}</p>
        </router-link>
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
        <p class="text-sm">No live streams from your subscriptions</p>
      </div>
    </div>

    <!-- Posts Tab -->
    <div v-else-if="activeTab === 'posts'">
      <div class="rounded-lg border border-border bg-card p-8 text-center text-muted-foreground">
        <svg class="size-12 mx-auto mb-3 text-muted-foreground/50" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
          <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z" />
        </svg>
        <p class="text-sm">No community posts from your subscriptions</p>
      </div>
    </div>
  </div>
</template>

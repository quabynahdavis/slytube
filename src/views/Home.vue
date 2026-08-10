<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { cn } from '@/lib/utils'
import { useSettingsStore } from '@/stores/settings'

const settingsStore = useSettingsStore()

const isLoading = ref(true)
const sortBy = ref('trending')
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

const sortOptions = [
  { value: 'trending', label: 'Trending' },
  { value: 'newest', label: 'Newest' },
  { value: 'popular', label: 'Popular' },
  { value: 'oldest', label: 'Oldest' },
]

const listType = computed(() => settingsStore.listType)

onMounted(async () => {
  isLoading.value = true
  try {
    await new Promise((resolve) => setTimeout(resolve, 600))
    videos.value = Array.from({ length: 24 }, (_, i) => ({
      videoId: `home-${i}`,
      title: `Trending Video ${i + 1} - Sample Title`,
      author: `Channel ${i + 1}`,
      authorId: `UC-channel-${i}`,
      viewCount: Math.floor(Math.random() * 5000000),
      lengthSeconds: Math.floor(Math.random() * 600) + 60,
      published: new Date(Date.now() - Math.random() * 30 * 24 * 60 * 60 * 1000).toISOString().split('T')[0],
      videoThumbnails: [{ url: '', width: 640, height: 360 }],
    }))
  } finally {
    isLoading.value = false
  }
})

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
  if (days > 30) return `${Math.floor(days / 30)} months ago`
  if (days > 7) return `${Math.floor(days / 7)} weeks ago`
  if (days > 0) return `${days} days ago`
  return 'Today'
}
</script>

<template>
  <div class="container mx-auto max-w-7xl px-4 py-6">
    <!-- Header -->
    <div class="mb-6">
      <h1 class="text-2xl font-bold text-foreground">Home</h1>
      <p class="text-sm text-muted-foreground mt-1">Trending videos for you</p>
    </div>

    <!-- Filter/Sort Bar -->
    <div class="flex flex-wrap items-center gap-3 mb-6">
      <div class="flex items-center gap-2">
        <label class="text-sm text-muted-foreground">Sort by:</label>
        <select
          v-model="sortBy"
          class="h-9 rounded-md border border-input bg-background px-3 text-sm"
        >
          <option v-for="opt in sortOptions" :key="opt.value" :value="opt.value">
            {{ opt.label }}
          </option>
        </select>
      </div>
      <div class="flex items-center gap-1 ml-auto">
        <button
          :class="cn(
            'inline-flex items-center justify-center size-8 rounded-md transition-colors',
            listType === 'grid' ? 'bg-accent text-accent-foreground' : 'text-muted-foreground hover:bg-accent'
          )"
          title="Grid view"
          @click="settingsStore.updateSetting('listType', 'grid')"
        >
          <svg class="size-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <rect x="3" y="3" width="7" height="7" />
            <rect x="14" y="3" width="7" height="7" />
            <rect x="3" y="14" width="7" height="7" />
            <rect x="14" y="14" width="7" height="7" />
          </svg>
        </button>
        <button
          :class="cn(
            'inline-flex items-center justify-center size-8 rounded-md transition-colors',
            listType === 'list' ? 'bg-accent text-accent-foreground' : 'text-muted-foreground hover:bg-accent'
          )"
          title="List view"
          @click="settingsStore.updateSetting('listType', 'list')"
        >
          <svg class="size-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="8" y1="6" x2="21" y2="6" />
            <line x1="8" y1="12" x2="21" y2="12" />
            <line x1="8" y1="18" x2="21" y2="18" />
            <line x1="3" y1="6" x2="3.01" y2="6" />
            <line x1="3" y1="12" x2="3.01" y2="12" />
            <line x1="3" y1="18" x2="3.01" y2="18" />
          </svg>
        </button>
      </div>
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

    <!-- Video Grid -->
    <div v-else>
      <div
        v-if="listType === 'grid'"
        class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4"
      >
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

      <!-- List View -->
      <div v-else class="space-y-4">
        <router-link
          v-for="video in videos"
          :key="video.videoId"
          :to="`/watch?v=${video.videoId}`"
          class="flex gap-4 group"
        >
          <div class="relative shrink-0 w-64 aspect-video rounded-lg bg-muted overflow-hidden">
            <div class="absolute inset-0 flex items-center justify-center">
              <svg class="size-10 text-muted-foreground/50 group-hover:text-primary transition-colors" viewBox="0 0 24 24" fill="currentColor">
                <polygon points="5 3 19 12 5 21 5 3" />
              </svg>
            </div>
            <span class="absolute bottom-2 right-2 rounded bg-black/80 px-1.5 py-0.5 text-xs text-white font-medium">
              {{ formatDuration(video.lengthSeconds) }}
            </span>
          </div>
          <div class="min-w-0 flex-1">
            <h3 class="text-base font-medium text-foreground line-clamp-2 group-hover:text-primary transition-colors">
              {{ video.title }}
            </h3>
            <p class="mt-1 text-sm text-muted-foreground">{{ formatViews(video.viewCount) }} &middot; {{ timeAgo(video.published) }}</p>
            <p class="mt-2 text-sm text-muted-foreground hover:text-foreground">{{ video.author }}</p>
          </div>
        </router-link>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { cn } from '@/lib/utils'
import { useSearchHistoryStore } from '@/stores/search-history'

const route = useRoute()
const router = useRouter()
const searchHistoryStore = useSearchHistoryStore()

const isLoading = ref(true)
const searchQuery = ref((route.query.q as string) || '')
const sortBy = ref('relevance')
const filterType = ref('all')
const filterDuration = ref('all')
const filterDate = ref('all')

const results = ref<Array<{
  videoId: string
  title: string
  author: string
  authorId: string
  description: string
  viewCount: number
  lengthSeconds: number
  published: string
  type: 'video' | 'channel' | 'playlist'
  videoThumbnails: Array<{ url: string; width: number; height: number }>
}>>([])

const sortOptions = [
  { value: 'relevance', label: 'Relevance' },
  { value: 'date', label: 'Upload date' },
  { value: 'views', label: 'View count' },
  { value: 'rating', label: 'Rating' },
]

const typeOptions = [
  { value: 'all', label: 'All' },
  { value: 'video', label: 'Video' },
  { value: 'channel', label: 'Channel' },
  { value: 'playlist', label: 'Playlist' },
]

const durationOptions = [
  { value: 'all', label: 'Any duration' },
  { value: 'short', label: 'Under 4 min' },
  { value: 'medium', label: '4-20 min' },
  { value: 'long', label: 'Over 20 min' },
]

const dateOptions = [
  { value: 'all', label: 'Any time' },
  { value: 'hour', label: 'Last hour' },
  { value: 'today', label: 'Today' },
  { value: 'week', label: 'This week' },
  { value: 'month', label: 'This month' },
  { value: 'year', label: 'This year' },
]

const showFilters = ref(false)

onMounted(async () => {
  if (searchQuery.value) {
    await performSearch()
  }
})

watch(() => route.query.q, async (newQuery) => {
  if (newQuery && typeof newQuery === 'string') {
    searchQuery.value = newQuery
    await performSearch()
  }
})

async function performSearch() {
  isLoading.value = true
  try {
    await new Promise((resolve) => setTimeout(resolve, 600))
    results.value = Array.from({ length: 20 }, (_, i) => ({
      videoId: `search-${i}`,
      title: `Search Result ${i + 1} for "${searchQuery.value}"`,
      author: `Channel ${i + 1}`,
      authorId: `UC-channel-${i}`,
      description: `This is a search result description for "${searchQuery.value}". It contains relevant information about the video content.`,
      viewCount: Math.floor(Math.random() * 5000000),
      lengthSeconds: Math.floor(Math.random() * 600) + 60,
      published: new Date(Date.now() - Math.random() * 365 * 24 * 60 * 60 * 1000).toISOString().split('T')[0],
      type: (['video', 'channel', 'playlist'] as const)[Math.floor(Math.random() * 3)],
      videoThumbnails: [{ url: '', width: 640, height: 360 }],
    }))
  } finally {
    isLoading.value = false
  }
}

function handleSearch() {
  if (!searchQuery.value.trim()) return
  searchHistoryStore.addSearchHistoryEntry({
    _id: searchQuery.value.trim(),
    timeWatched: Date.now(),
  })
  router.push({ path: '/search', query: { q: searchQuery.value.trim() } })
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
  if (days > 365) return `${Math.floor(days / 365)} years ago`
  if (days > 30) return `${Math.floor(days / 30)} months ago`
  if (days > 0) return `${days} days ago`
  return 'Today'
}
</script>

<template>
  <div class="container mx-auto max-w-7xl px-4 py-6">
    <!-- Search Bar -->
    <form @submit.prevent="handleSearch" class="mb-6">
      <div class="flex gap-2">
        <div class="relative flex-1">
          <input
            v-model="searchQuery"
            type="search"
            placeholder="Search videos, channels, playlists..."
            class="h-10 w-full rounded-lg border border-input bg-background px-4 pr-10 text-sm outline-none focus:border-primary focus:ring-1 focus:ring-primary"
          />
        </div>
        <button
          type="submit"
          class="h-10 rounded-lg bg-primary px-6 text-sm font-medium text-primary-foreground hover:bg-primary/90 transition-colors"
        >
          Search
        </button>
      </div>
    </form>

    <!-- Filter Toggle & Sort -->
    <div class="flex flex-wrap items-center gap-3 mb-4">
      <button
        :class="cn(
          'inline-flex items-center gap-1 rounded-md border border-border px-3 py-1.5 text-sm transition-colors',
          showFilters ? 'bg-accent text-accent-foreground' : 'text-muted-foreground hover:bg-accent'
        )"
        @click="showFilters = !showFilters"
      >
        <svg class="size-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <polygon points="22 3 2 3 10 12.46 10 19 14 21 14 12.46 22 3" />
        </svg>
        Filters
      </button>
      <select
        v-model="sortBy"
        class="h-8 rounded-md border border-input bg-background px-3 text-sm"
      >
        <option v-for="opt in sortOptions" :key="opt.value" :value="opt.value">
          {{ opt.label }}
        </option>
      </select>
    </div>

    <!-- Filter Panel -->
    <div v-if="showFilters" class="rounded-lg border border-border bg-card p-4 mb-6">
      <div class="grid grid-cols-1 sm:grid-cols-3 gap-4">
        <div>
          <label class="text-xs font-medium text-muted-foreground uppercase">Type</label>
          <select
            v-model="filterType"
            class="mt-1 h-9 w-full rounded-md border border-input bg-background px-3 text-sm"
          >
            <option v-for="opt in typeOptions" :key="opt.value" :value="opt.value">
              {{ opt.label }}
            </option>
          </select>
        </div>
        <div>
          <label class="text-xs font-medium text-muted-foreground uppercase">Duration</label>
          <select
            v-model="filterDuration"
            class="mt-1 h-9 w-full rounded-md border border-input bg-background px-3 text-sm"
          >
            <option v-for="opt in durationOptions" :key="opt.value" :value="opt.value">
              {{ opt.label }}
            </option>
          </select>
        </div>
        <div>
          <label class="text-xs font-medium text-muted-foreground uppercase">Upload Date</label>
          <select
            v-model="filterDate"
            class="mt-1 h-9 w-full rounded-md border border-input bg-background px-3 text-sm"
          >
            <option v-for="opt in dateOptions" :key="opt.value" :value="opt.value">
              {{ opt.label }}
            </option>
          </select>
        </div>
      </div>
    </div>

    <!-- Results Count -->
    <p v-if="!isLoading" class="text-sm text-muted-foreground mb-4">
      About {{ results.length }} results for "{{ searchQuery }}"
    </p>

    <!-- Loading State -->
    <div v-if="isLoading" class="space-y-4">
      <div v-for="n in 6" :key="n" class="flex gap-4 animate-pulse">
        <div class="w-64 aspect-video rounded-lg bg-muted shrink-0" />
        <div class="flex-1 space-y-2">
          <div class="h-4 w-3/4 rounded bg-muted" />
          <div class="h-3 w-1/2 rounded bg-muted" />
          <div class="h-3 w-full rounded bg-muted" />
        </div>
      </div>
    </div>

    <!-- Results -->
    <div v-else class="space-y-4">
      <div
        v-for="result in results"
        :key="result.videoId"
        class="flex gap-4 group"
      >
        <router-link
          :to="result.type === 'channel' ? `/channel/${result.authorId}` : `/watch?v=${result.videoId}`"
          class="relative shrink-0 w-64 aspect-video rounded-lg bg-muted overflow-hidden"
        >
          <div class="absolute inset-0 flex items-center justify-center">
            <svg class="size-10 text-muted-foreground/50 group-hover:text-primary transition-colors" viewBox="0 0 24 24" fill="currentColor">
              <polygon points="5 3 19 12 5 21 5 3" />
            </svg>
          </div>
          <span class="absolute bottom-2 right-2 rounded bg-black/80 px-1.5 py-0.5 text-xs text-white font-medium">
            {{ formatDuration(result.lengthSeconds) }}
          </span>
          <span
            v-if="result.type !== 'video'"
            class="absolute top-2 left-2 rounded bg-primary px-2 py-0.5 text-xs text-primary-foreground font-medium capitalize"
          >
            {{ result.type }}
          </span>
        </router-link>
        <div class="min-w-0 flex-1">
          <router-link
            :to="result.type === 'channel' ? `/channel/${result.authorId}` : `/watch?v=${result.videoId}`"
            class="text-base font-medium text-foreground line-clamp-2 group-hover:text-primary transition-colors"
          >
            {{ result.title }}
          </router-link>
          <p class="mt-1 text-sm text-muted-foreground">
            {{ formatViews(result.viewCount) }} &middot; {{ timeAgo(result.published) }}
          </p>
          <router-link
            :to="`/channel/${result.authorId}`"
            class="mt-2 flex items-center gap-2 text-sm text-muted-foreground hover:text-foreground"
          >
            <div class="size-6 rounded-full bg-muted" />
            <span>{{ result.author }}</span>
          </router-link>
          <p class="mt-2 text-sm text-muted-foreground line-clamp-2">{{ result.description }}</p>
        </div>
      </div>
    </div>
  </div>
</template>

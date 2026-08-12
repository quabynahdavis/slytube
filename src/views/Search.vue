<script setup lang="ts">
import { ref, onMounted, watch, computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { search } from '../api'
import type { Video } from '@/api/types'
import SkeletonGrid from '@/components/ui/SkeletonGrid.vue'
import EmptyState from '@/components/ui/EmptyState.vue'
import ErrorState from '@/components/ui/ErrorState.vue'
import { useSearchHistoryStore } from '@/stores/search-history'
import { getSearchSuggestions } from '@/composables/useInnertube'
import { PhPlayCircle, PhUser, PhMagnifyingGlass } from '@phosphor-icons/vue'

const route = useRoute()
const router = useRouter()
const searchHistoryStore = useSearchHistoryStore()

const query = ref((route.query.q as string) || '')
const results = ref<Video[]>([])
const loading = ref(false)
const error = ref<string | null>(null)

const sortBy = ref('relevance')
const filterType = ref('all')
const filterDuration = ref('all')
const filterDate = ref('all')

const suggestions = ref<string[]>([])
const isInputFocused = ref(false)
let debounceTimer: ReturnType<typeof setTimeout> | null = null

const shortsResults = computed(() => results.value.filter(r => r.isShort).slice(0, 6))
const regularResults = computed(() => results.value.filter(r => !r.isShort))

async function doSearch() {
  if (!query.value.trim()) return
  loading.value = true
  error.value = null
  suggestions.value = []
  router.replace({ query: { q: query.value } })
  try {
    results.value = await search(query.value, {
      sort: sortBy.value,
      type: filterType.value,
      duration: filterDuration.value,
      date: filterDate.value,
    })
    searchHistoryStore.addSearchHistoryEntry({
      _id: query.value.trim(),
      timeWatched: Date.now(),
    })
  } catch (e: any) {
    error.value = e.message || 'Search failed'
  } finally {
    loading.value = false
  }
}

function fetchSuggestions() {
  if (debounceTimer) clearTimeout(debounceTimer)
  if (!query.value.trim()) {
    suggestions.value = []
    return
  }
  debounceTimer = setTimeout(async () => {
    try {
      suggestions.value = await getSearchSuggestions(query.value)
    } catch {
      suggestions.value = []
    }
  }, 200)
}

function selectSuggestion(s: string) {
  query.value = s
  suggestions.value = []
  doSearch()
}

function clearSearch() {
  query.value = ''
  results.value = []
  suggestions.value = []
  router.replace({})
}

function handleBlur() {
  setTimeout(() => {
    isInputFocused.value = false
  }, 200)
}

watch(() => route.query.q, (q) => {
  if (q && q !== query.value) {
    query.value = q as string
    doSearch()
  }
})

watch([sortBy, filterType, filterDuration, filterDate], () => {
  if (query.value.trim() && results.value.length > 0) {
    doSearch()
  }
})

onMounted(() => {
  if (query.value) doSearch()
})

function formatViews(count: number): string {
  if (count >= 1_000_000) return `${(count / 1_000_000).toFixed(1)}M views`
  if (count >= 1_000) return `${(count / 1_000).toFixed(1)}K views`
  return `${count} views`
}

function timeAgo(published: string): string {
  if (!published) return ''
  if (published.includes('ago') || published.includes('yesterday')) {
    return published
  }
  const now = Date.now()
  const then = new Date(published).getTime()
  if (isNaN(then)) return published
  const diff = now - then
  const mins = Math.floor(diff / 60000)
  if (mins < 60) return `${mins}m ago`
  const hours = Math.floor(mins / 60)
  if (hours < 24) return `${hours}h ago`
  const days = Math.floor(hours / 24)
  if (days < 30) return `${days}d ago`
  const months = Math.floor(days / 30)
  if (months < 12) return `${months}mo ago`
  return `${Math.floor(months / 12)}y ago`
}
</script>

<template>
  <div class="p-4 max-w-5xl mx-auto">
    <!-- Search Bar -->
    <div class="relative mb-6">
      <form @submit.prevent="doSearch" class="flex gap-2">
        <div class="relative flex-1">
          <input
            v-model="query"
            type="text"
            placeholder="Search..."
            class="w-full px-4 py-2.5 rounded-lg border border-border bg-background text-foreground placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-primary"
            @input="fetchSuggestions"
            @focus="isInputFocused = true"
            @blur="handleBlur"
          />
          <button
            v-if="query"
            type="button"
            class="absolute right-3 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground"
            @click="clearSearch"
          >
            <svg class="size-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <line x1="18" y1="6" x2="6" y2="18" />
              <line x1="6" y1="6" x2="18" y2="18" />
            </svg>
          </button>
        </div>
        <button
          type="submit"
          class="px-6 py-2.5 bg-primary text-primary-foreground rounded-lg text-sm font-medium hover:bg-primary/90 transition-colors"
        >
          Search
        </button>
      </form>

      <!-- Suggestions Dropdown -->
      <div
        v-if="isInputFocused && suggestions.length > 0"
        class="absolute top-full left-0 right-16 mt-1 rounded-lg border border-border bg-popover shadow-xl z-50 max-h-64 overflow-y-auto"
      >
        <ul class="py-1">
          <li
            v-for="s in suggestions"
            :key="s"
            class="px-4 py-2 text-sm cursor-pointer hover:bg-accent flex items-center gap-2"
            @mousedown="selectSuggestion(s)"
          >
            <PhMagnifyingGlass :size="14" class="text-muted-foreground shrink-0" />
            {{ s }}
          </li>
        </ul>
      </div>
    </div>

    <!-- Search History (when no query) -->
    <div v-if="!query && !loading" class="mb-6">
      <h3 class="text-sm font-semibold text-foreground mb-3">Recent searches</h3>
      <div class="flex flex-wrap gap-2">
        <button
          v-for="entry in searchHistoryStore.getLatestSearchHistoryNames.slice(0, 8)"
          :key="entry"
          class="px-3 py-1.5 rounded-full bg-muted text-sm text-muted-foreground hover:bg-accent transition-colors"
          @click="query = entry; doSearch()"
        >
          {{ entry }}
        </button>
        <p v-if="searchHistoryStore.getLatestSearchHistoryNames.length === 0" class="text-sm text-muted-foreground">
          No recent searches
        </p>
      </div>
    </div>

    <!-- Filters -->
    <div v-if="results.length > 0 || loading" class="flex flex-wrap gap-3 mb-6 pb-4 border-b border-border">
      <select v-model="sortBy" class="h-8 px-2 rounded-md border border-input bg-background text-sm">
        <option value="relevance">Relevance</option>
        <option value="date">Upload date</option>
        <option value="views">View count</option>
        <option value="rating">Rating</option>
      </select>
      <select v-model="filterType" class="h-8 px-2 rounded-md border border-input bg-background text-sm">
        <option value="all">All</option>
        <option value="video">Videos</option>
        <option value="channel">Channels</option>
        <option value="playlist">Playlists</option>
      </select>
      <select v-model="filterDuration" class="h-8 px-2 rounded-md border border-input bg-background text-sm">
        <option value="all">Any duration</option>
        <option value="short">Under 4 min</option>
        <option value="medium">4-20 min</option>
        <option value="long">Over 20 min</option>
      </select>
      <select v-model="filterDate" class="h-8 px-2 rounded-md border border-input bg-background text-sm">
        <option value="all">Any time</option>
        <option value="hour">Last hour</option>
        <option value="today">Today</option>
        <option value="week">This week</option>
        <option value="month">This month</option>
        <option value="year">This year</option>
      </select>
    </div>

    <SkeletonGrid v-if="loading" :count="6" />
    <ErrorState v-else-if="error" :message="error" retryable @retry="doSearch" />
    <EmptyState v-else-if="query && results.length === 0" title="No results found">
      Try different keywords, check your spelling, or use fewer filters.
    </EmptyState>
    <EmptyState v-else-if="!query" title="Search YouTube" icon="search">
      Enter keywords above to find videos, channels, and playlists.
    </EmptyState>

    <div v-else class="space-y-6">
      <!-- Shorts Section -->
      <section v-if="shortsResults.length > 0">
        <div class="flex items-center gap-2 mb-4">
          <PhPlayCircle :size="20" class="text-primary" />
          <h2 class="text-lg font-semibold text-foreground">Shorts</h2>
        </div>
        <div class="flex gap-3 overflow-x-auto pb-4">
          <router-link
            v-for="short in shortsResults"
            :key="short.id"
            :to="`/watch?v=${short.id}`"
            class="shrink-0 w-40 group"
          >
            <div class="aspect-[9/16] rounded-xl overflow-hidden bg-muted mb-2">
              <img
                v-if="short.thumbnail"
                :src="short.thumbnail"
                :alt="short.title"
                class="w-full h-full object-cover group-hover:scale-105 transition-transform"
              />
            </div>
            <p class="text-xs text-foreground line-clamp-2 leading-tight">{{ short.title }}</p>
            <p class="text-[10px] text-muted-foreground mt-0.5">{{ formatViews(short.viewCount) }}</p>
          </router-link>
        </div>
      </section>

      <!-- Channels Section -->
      <section v-if="filterType === 'all' || filterType === 'channel'">
        <div class="flex items-center gap-2 mb-4">
          <PhUser :size="20" class="text-primary" />
          <h2 class="text-lg font-semibold text-foreground">Channels</h2>
        </div>
        <div class="space-y-3">
          <div
            v-for="video in regularResults.slice(0, 3)"
            :key="video.id"
            class="flex items-center gap-4 p-3 rounded-lg hover:bg-muted/50 transition-colors"
          >
            <router-link
              :to="`/channel/${video.authorId}`"
              class="size-16 rounded-full bg-muted flex items-center justify-center shrink-0 overflow-hidden"
            >
              <img
                v-if="video.authorAvatar"
                :src="video.authorAvatar"
                :alt="video.author"
                class="w-full h-full object-cover"
              />
              <PhUser v-else class="size-8 text-muted-foreground" />
            </router-link>
            <div class="flex-1 min-w-0">
              <router-link
                :to="`/channel/${video.authorId}`"
                class="text-sm font-medium text-foreground hover:text-primary"
              >
                {{ video.author }}
              </router-link>
              <p class="text-xs text-muted-foreground mt-0.5">Channel</p>
            </div>
            <button class="px-4 py-1.5 rounded-full bg-primary text-primary-foreground text-sm font-medium hover:bg-primary/90 transition-colors">
              Subscribe
            </button>
          </div>
        </div>
      </section>

      <!-- Videos List -->
      <section>
        <div v-if="shortsResults.length > 0 || filterType === 'all' || filterType === 'channel'" class="flex items-center gap-2 mb-4">
          <h2 class="text-lg font-semibold text-foreground">Videos</h2>
        </div>
        <div class="space-y-3">
          <router-link
            v-for="video in regularResults"
            :key="video.id"
            :to="`/watch?v=${video.id}`"
            class="flex gap-4 p-3 rounded-lg hover:bg-muted/50 transition-colors group"
          >
            <div class="relative w-64 shrink-0 aspect-video rounded-xl overflow-hidden bg-muted">
              <img
                v-if="video.thumbnail"
                :src="video.thumbnail"
                :alt="video.title"
                class="w-full h-full object-cover group-hover:scale-105 transition-transform"
              />
              <div v-if="video.lengthSeconds > 0" class="absolute bottom-1 right-1 bg-black/80 text-white text-[10px] px-1 py-0.5 rounded">
                {{ Math.floor(video.lengthSeconds / 60) }}:{{ (video.lengthSeconds % 60).toString().padStart(2, '0') }}
              </div>
            </div>
            <div class="flex-1 min-w-0">
              <h3 class="text-sm font-medium text-foreground line-clamp-2 group-hover:text-primary">
                {{ video.title }}
              </h3>
              <p class="text-xs text-muted-foreground mt-1">
                {{ formatViews(video.viewCount) }} • {{ timeAgo(video.published) }}
              </p>
              <div class="flex items-center gap-2 mt-2">
                <div class="size-6 rounded-full bg-muted flex items-center justify-center overflow-hidden">
                  <img
                    v-if="video.authorAvatar"
                    :src="video.authorAvatar"
                    :alt="video.author"
                    class="w-full h-full object-cover"
                  />
                  <span v-else class="text-[8px]">{{ video.author[0] }}</span>
                </div>
                <span class="text-xs text-muted-foreground hover:text-foreground">{{ video.author }}</span>
              </div>
              <p v-if="video.description" class="text-xs text-muted-foreground mt-2 line-clamp-2">
                {{ video.description }}
              </p>
            </div>
          </router-link>
        </div>
      </section>
    </div>
  </div>
</template>

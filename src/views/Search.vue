<script setup lang="ts">
import { ref, onMounted, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { search } from '../api'
import type { Video } from '../api/types'
import VideoCard from '../components/VideoCard.vue'
import SkeletonGrid from '../components/ui/SkeletonGrid.vue'
import EmptyState from '../components/ui/EmptyState.vue'
import ErrorState from '../components/ui/ErrorState.vue'

const route = useRoute()
const router = useRouter()

const query = ref((route.query.q as string) || '')
const results = ref<Video[]>([])
const loading = ref(false)
const error = ref<string | null>(null)

const sortBy = ref('relevance')
const filterType = ref('all')
const filterDuration = ref('all')
const filterDate = ref('all')

async function doSearch() {
  if (!query.value.trim()) return
  loading.value = true
  error.value = null
  router.replace({ query: { q: query.value } })
  try {
    results.value = await search(query.value)
  } catch (e: any) {
    error.value = e.message || 'Search failed'
  } finally {
    loading.value = false
  }
}

watch(() => route.query.q, (q) => {
  if (q && q !== query.value) {
    query.value = q as string
    doSearch()
  }
})

onMounted(() => {
  if (query.value) doSearch()
})
</script>

<template>
  <div class="p-6">
    <!-- Search Bar -->
    <form @submit.prevent="doSearch" class="flex gap-2 mb-6">
      <input
        v-model="query"
        type="text"
        placeholder="Search..."
        class="flex-1 px-4 py-2.5 rounded-lg border border-border bg-background text-foreground placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-primary"
      />
      <button
        type="submit"
        class="px-6 py-2.5 bg-primary text-primary-foreground rounded-lg text-sm font-medium hover:bg-primary/90 transition-colors"
      >
        Search
      </button>
    </form>

    <!-- Filters -->
    <div v-if="results.length > 0 || loading" class="flex gap-4 mb-6 pb-4 border-b border-border">
      <select v-model="sortBy" class="px-3 py-1.5 rounded-lg border border-border bg-background text-sm">
        <option value="relevance">Relevance</option>
        <option value="date">Upload date</option>
        <option value="views">View count</option>
        <option value="rating">Rating</option>
      </select>
      <select v-model="filterType" class="px-3 py-1.5 rounded-lg border border-border bg-background text-sm">
        <option value="all">All</option>
        <option value="video">Videos</option>
        <option value="channel">Channels</option>
        <option value="playlist">Playlists</option>
      </select>
      <select v-model="filterDuration" class="px-3 py-1.5 rounded-lg border border-border bg-background text-sm">
        <option value="all">Any duration</option>
        <option value="short">Under 4 min</option>
        <option value="medium">4-20 min</option>
        <option value="long">Over 20 min</option>
      </select>
      <select v-model="filterDate" class="px-3 py-1.5 rounded-lg border border-border bg-background text-sm">
        <option value="all">Any time</option>
        <option value="hour">Last hour</option>
        <option value="today">Today</option>
        <option value="week">This week</option>
        <option value="month">This month</option>
        <option value="year">This year</option>
      </select>
    </div>

    <SkeletonGrid v-if="loading" :count="12" />
    <ErrorState v-else-if="error" :message="error" retryable @retry="doSearch" />
    <EmptyState v-else-if="query && results.length === 0" title="No results found">
      Try different keywords or check your spelling.
    </EmptyState>
    <EmptyState v-else-if="!query" title="Search YouTube" icon="search">
      Enter a search query above to find videos.
    </EmptyState>
    <div v-else class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
      <VideoCard v-for="video in results" :key="video.id" :video="video" />
    </div>
  </div>
</template>

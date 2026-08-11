<script setup lang="ts">
import { ref, onMounted, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { search } from '../api'
import type { Video } from '../api/types'
import VideoCard from '../components/VideoCard.vue'
import SkeletonGrid from '../components/ui/SkeletonGrid.vue'
import EmptyState from '../components/ui/EmptyState.vue'
import ErrorState from '../components/ui/ErrorState.vue'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'

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
      <Select v-model="sortBy">
        <SelectTrigger class="w-[160px]">
          <SelectValue placeholder="Sort by..." />
        </SelectTrigger>
        <SelectContent>
          <SelectItem value="relevance">Relevance</SelectItem>
          <SelectItem value="date">Upload date</SelectItem>
          <SelectItem value="views">View count</SelectItem>
          <SelectItem value="rating">Rating</SelectItem>
        </SelectContent>
      </Select>
      <Select v-model="filterType">
        <SelectTrigger class="w-[140px]">
          <SelectValue placeholder="Type..." />
        </SelectTrigger>
        <SelectContent>
          <SelectItem value="all">All</SelectItem>
          <SelectItem value="video">Videos</SelectItem>
          <SelectItem value="channel">Channels</SelectItem>
          <SelectItem value="playlist">Playlists</SelectItem>
        </SelectContent>
      </Select>
      <Select v-model="filterDuration">
        <SelectTrigger class="w-[150px]">
          <SelectValue placeholder="Duration..." />
        </SelectTrigger>
        <SelectContent>
          <SelectItem value="all">Any duration</SelectItem>
          <SelectItem value="short">Under 4 min</SelectItem>
          <SelectItem value="medium">4-20 min</SelectItem>
          <SelectItem value="long">Over 20 min</SelectItem>
        </SelectContent>
      </Select>
      <Select v-model="filterDate">
        <SelectTrigger class="w-[140px]">
          <SelectValue placeholder="Date..." />
        </SelectTrigger>
        <SelectContent>
          <SelectItem value="all">Any time</SelectItem>
          <SelectItem value="hour">Last hour</SelectItem>
          <SelectItem value="today">Today</SelectItem>
          <SelectItem value="week">This week</SelectItem>
          <SelectItem value="month">This month</SelectItem>
          <SelectItem value="year">This year</SelectItem>
        </SelectContent>
      </Select>
    </div>

    <SkeletonGrid v-if="loading" :count="12" />
    <ErrorState v-else-if="error" :message="error" retryable @retry="doSearch" />
    <EmptyState v-else-if="query && results.length === 0" title="No results found">
      Try different keywords, check your spelling, or use fewer filters.
    </EmptyState>
    <EmptyState v-else-if="!query" title="Search YouTube" icon="search">
      Enter keywords above to find videos, channels, and playlists.
    </EmptyState>
    <div v-else class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
      <VideoCard
        v-for="video in results"
        :key="video.id"
        :video="video"
      />
    </div>
  </div>
</template>

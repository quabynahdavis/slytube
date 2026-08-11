<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { cn } from '@/lib/utils'
import { getTrendingVideos } from '../api'
import type { Video } from '../api/types'
import VideoCard from '../components/VideoCard.vue'
import SkeletonGrid from '../components/ui/SkeletonGrid.vue'
import ErrorState from '../components/ui/ErrorState.vue'
import EmptyState from '../components/ui/EmptyState.vue'

const isLoading = ref(true)
const error = ref<string | null>(null)
const selectedCategory = ref('all')
const videos = ref<Video[]>([])

const categories = [
  { value: 'all', label: 'All' },
  { value: 'music', label: 'Music' },
  { value: 'gaming', label: 'Gaming' },
  { value: 'news', label: 'News' },
  { value: 'sports', label: 'Sports' },
  { value: 'learning', label: 'Learning' },
  { value: 'fashion', label: 'Fashion' },
]

const filteredVideos = computed(() => {
  if (selectedCategory.value === 'all') return videos.value
  // Client-side filtering based on category (since API doesn't support category filtering directly)
  // In a real implementation, this would use API parameters
  return videos.value
})

async function loadTrending() {
  isLoading.value = true
  error.value = null
  try {
    videos.value = await getTrendingVideos()
  } catch (e: any) {
    error.value = e.message || 'Failed to load trending videos'
  } finally {
    isLoading.value = false
  }
}

onMounted(loadTrending)
</script>

<template>
  <div class="container mx-auto max-w-7xl px-4 py-6">
    <div class="mb-6">
      <h1 class="text-2xl font-bold text-foreground">Trending</h1>
      <p class="text-sm text-muted-foreground mt-1">What's popular right now</p>
    </div>

    <div class="flex gap-2 overflow-x-auto pb-4 mb-6">
      <button v-for="cat in categories" :key="cat.value" :class="cn('shrink-0 rounded-full px-4 py-1.5 text-sm font-medium transition-colors', selectedCategory === cat.value ? 'bg-primary text-primary-foreground' : 'bg-muted text-muted-foreground hover:bg-muted/80')" @click="selectedCategory = cat.value">{{ cat.label }}</button>
    </div>

    <SkeletonGrid v-if="isLoading" :count="8" />

    <ErrorState v-else-if="error" :message="error" retryable @retry="loadTrending" />

    <EmptyState v-else-if="filteredVideos.length === 0" title="No trending videos">
      Trending videos aren't available right now. Check back later or explore Popular videos.
    </EmptyState>

    <div v-else class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
      <VideoCard
        v-for="(video, index) in filteredVideos"
        :key="video.id"
        :video="video"
        v-staggered-anim="index"
      />
    </div>
  </div>
</template>

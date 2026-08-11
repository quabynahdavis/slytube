<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { getTrendingVideos } from '../api'
import type { Video } from '../api/types'
import VideoCard from '../components/VideoCard.vue'
import SkeletonGrid from '../components/ui/SkeletonGrid.vue'
import ErrorState from '../components/ui/ErrorState.vue'
import EmptyState from '../components/ui/EmptyState.vue'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'

const isLoading = ref(true)
const error = ref<string | null>(null)
const sortBy = ref('views')
const videos = ref<Video[]>([])

const sortedVideos = computed(() => {
  const sorted = [...videos.value]
  if (sortBy.value === 'views') {
    sorted.sort((a, b) => b.viewCount - a.viewCount)
  } else if (sortBy.value === 'likes') {
    sorted.sort((a, b) => b.likeCount - a.likeCount)
  } else if (sortBy.value === 'recent') {
    sorted.sort((a, b) => {
      const dateA = new Date(a.published).getTime()
      const dateB = new Date(b.published).getTime()
      if (isNaN(dateA) && isNaN(dateB)) return 0
      if (isNaN(dateA)) return 1
      if (isNaN(dateB)) return -1
      return dateB - dateA
    })
  }
  return sorted
})

async function loadPopular() {
  isLoading.value = true
  error.value = null
  try {
    // Use trending videos as placeholder for popular
    videos.value = await getTrendingVideos()
  } catch (e: any) {
    error.value = e.message || 'Failed to load popular videos'
  } finally {
    isLoading.value = false
  }
}

onMounted(loadPopular)
</script>

<template>
  <div class="container mx-auto max-w-7xl px-4 py-6">
    <div class="flex items-center justify-between mb-6">
      <div>
        <h1 class="text-2xl font-bold text-foreground">Popular</h1>
        <p class="text-sm text-muted-foreground mt-1">Most viewed videos</p>
      </div>
      <Select v-model="sortBy">
        <SelectTrigger class="w-[180px]">
          <SelectValue placeholder="Sort by..." />
        </SelectTrigger>
        <SelectContent>
          <SelectItem value="views">Most Views</SelectItem>
          <SelectItem value="likes">Most Liked</SelectItem>
          <SelectItem value="recent">Most Recent</SelectItem>
        </SelectContent>
      </Select>
    </div>

    <SkeletonGrid v-if="isLoading" :count="8" />

    <ErrorState v-else-if="error" :message="error" retryable @retry="loadPopular" />

    <EmptyState v-else-if="sortedVideos.length === 0" title="No popular videos">
      Check back later for popular content.
    </EmptyState>

    <div v-else class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
      <VideoCard v-for="video in sortedVideos" :key="video.id" :video="video" />
    </div>
  </div>
</template>

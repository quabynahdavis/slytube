<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { getTrendingVideos } from '../api'
import type { Video } from '../api/types'
import VideoCard from '../components/VideoCard.vue'
import SkeletonGrid from '../components/ui/SkeletonGrid.vue'
import EmptyState from '../components/ui/EmptyState.vue'
import ErrorState from '../components/ui/ErrorState.vue'

const videos = ref<Video[]>([])
const loading = ref(true)
const error = ref<string | null>(null)

async function load() {
  loading.value = true
  error.value = null
  try {
    videos.value = await getTrendingVideos()
  } catch (e: any) {
    error.value = e.message || 'Failed to load trending videos'
  } finally {
    loading.value = false
  }
}

onMounted(load)
</script>

<template>
  <div class="p-6">
    <div class="flex items-center justify-between mb-6">
      <h1 class="text-2xl font-bold text-foreground">Trending</h1>
      <div class="flex gap-2">
        <button class="px-3 py-1.5 text-sm rounded-lg bg-primary text-primary-foreground">Now</button>
        <button class="px-3 py-1.5 text-sm rounded-lg text-muted-foreground hover:bg-accent">Music</button>
        <button class="px-3 py-1.5 text-sm rounded-lg text-muted-foreground hover:bg-accent">Gaming</button>
        <button class="px-3 py-1.5 text-sm rounded-lg text-muted-foreground hover:bg-accent">Movies</button>
      </div>
    </div>

    <SkeletonGrid v-if="loading" :count="12" />
    <ErrorState v-else-if="error" :message="error" retryable @retry="load" />
    <EmptyState v-else-if="videos.length === 0" title="No trending videos" icon="trending">
      Check back later for trending content.
    </EmptyState>
    <div v-else class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
      <VideoCard v-for="video in videos" :key="video.id" :video="video" />
    </div>
  </div>
</template>

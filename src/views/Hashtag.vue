<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { useRoute } from 'vue-router'
import { search } from '../api'
import type { Video } from '../api/types'
import VideoCard from '../components/VideoCard.vue'
import SkeletonGrid from '../components/ui/SkeletonGrid.vue'
import ErrorState from '../components/ui/ErrorState.vue'
import EmptyState from '../components/ui/EmptyState.vue'

const route = useRoute()

const isLoading = ref(true)
const error = ref<string | null>(null)
const hashtag = computed(() => route.params.tag as string || route.query.tag as string || '')
const videos = ref<Video[]>([])

async function loadHashtagVideos() {
  if (!hashtag.value) return
  isLoading.value = true
  error.value = null
  try {
    videos.value = await search(`#${hashtag.value}`)
  } catch (e: any) {
    error.value = e.message || 'Failed to search for hashtag'
  } finally {
    isLoading.value = false
  }
}

onMounted(loadHashtagVideos)

watch(hashtag, loadHashtagVideos)
</script>

<template>
  <div class="container mx-auto max-w-7xl px-4 py-6">
    <div class="mb-6">
      <div class="flex items-center gap-3">
        <div class="size-12 rounded-full bg-primary/10 flex items-center justify-center">
          <svg class="size-6 text-primary" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="4" y1="9" x2="20" y2="9"/><line x1="4" y1="15" x2="20" y2="15"/><line x1="10" y1="3" x2="8" y2="21"/><line x1="16" y1="3" x2="14" y2="21"/></svg>
        </div>
        <div>
          <h1 class="text-2xl font-bold text-foreground">#{{ hashtag }}</h1>
          <p class="text-sm text-muted-foreground">{{ videos.length }} videos</p>
        </div>
      </div>
    </div>

    <SkeletonGrid v-if="isLoading" :count="8" />

    <ErrorState v-else-if="error" :message="error" retryable @retry="loadHashtagVideos" />

    <EmptyState v-else-if="videos.length === 0" title="No videos found">
      No videos found for #{{ hashtag }}
    </EmptyState>

    <div v-else class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
      <VideoCard v-for="video in videos" :key="video.id" :video="video" />
    </div>
  </div>
</template>

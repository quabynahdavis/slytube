<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { getTrendingVideos } from '../api'
import type { Video } from '../api/types'
import VideoCard from '../components/VideoCard.vue'
import SkeletonGrid from '../components/ui/SkeletonGrid.vue'
import EmptyState from '../components/ui/EmptyState.vue'
import ErrorState from '../components/ui/ErrorState.vue'

const { t } = useI18n()

const videos = ref<Video[]>([])
const loading = ref(true)
const error = ref<string | null>(null)

async function load() {
  loading.value = true
  error.value = null
  try {
    videos.value = await getTrendingVideos()
  } catch (e: any) {
    error.value = e.message || t('errors.failedToLoadTrending')
  } finally {
    loading.value = false
  }
}

onMounted(load)
</script>

<template>
  <div class="p-6">
    <div class="flex items-center justify-between mb-6">
      <h1 class="text-2xl font-bold text-foreground">{{ t('home.trending') }}</h1>
      <div class="flex gap-2">
        <button class="px-3 py-1.5 text-sm rounded-lg bg-primary text-primary-foreground">{{ t('home.now') }}</button>
        <button class="px-3 py-1.5 text-sm rounded-lg text-muted-foreground hover:bg-accent">{{ t('home.music') }}</button>
        <button class="px-3 py-1.5 text-sm rounded-lg text-muted-foreground hover:bg-accent">{{ t('home.gaming') }}</button>
        <button class="px-3 py-1.5 text-sm rounded-lg text-muted-foreground hover:bg-accent">{{ t('home.movies') }}</button>
      </div>
    </div>

    <SkeletonGrid v-if="loading" :count="12" />
    <ErrorState v-else-if="error" :message="error" retryable @retry="load" />
    <EmptyState v-else-if="videos.length === 0" :title="t('emptyStates.noTrendingVideos')" icon="trending">
      {{ t('emptyStates.noTrendingVideosDescription') }}
    </EmptyState>
    <div v-else class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
      <VideoCard
        v-for="(video, index) in videos"
        :key="video.id"
        :video="video"
        v-staggered-anim="index"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { getTrendingVideos, getSubscribedChannelsShorts } from '../api'
import { useSubscriptionsStore } from '@/stores/subscriptions'
import type { Video } from '../api/types'
import VideoCard from '../components/VideoCard.vue'
import SkeletonGrid from '../components/ui/SkeletonGrid.vue'
import EmptyState from '../components/ui/EmptyState.vue'
import ErrorState from '../components/ui/ErrorState.vue'

const { t } = useI18n()
const subscriptionsStore = useSubscriptionsStore()

const videos = ref<Video[]>([])
const loading = ref(true)
const error = ref<string | null>(null)

async function loadForYouFeed() {
  loading.value = true
  error.value = null
  try {
    await subscriptionsStore.loadSubscriptions()
    const channelIds = Array.from(subscriptionsStore.subscribedChannelIds)

    const [trendingVideos, subscriptionShorts] = await Promise.all([
      getTrendingVideos().catch(() => []),
      channelIds.length > 0 ? getSubscribedChannelsShorts(channelIds.slice(0, 3)).catch(() => []) : Promise.resolve([]),
    ])

    const allVideos = [...subscriptionShorts, ...trendingVideos]
    const seenIds = new Set<string>()
    const uniqueVideos: Video[] = []
    for (const video of allVideos) {
      if (video.id && !seenIds.has(video.id)) {
        seenIds.add(video.id)
        uniqueVideos.push(video)
      }
    }

    videos.value = uniqueVideos
  } catch (e: any) {
    error.value = e.message || t('errors.failedToLoad')
  } finally {
    loading.value = false
  }
}

onMounted(loadForYouFeed)
</script>

<template>
  <div class="p-4">
    <div class="mb-6">
      <h1 class="text-2xl font-bold text-foreground">{{ t('nav.home') }}</h1>
      <p class="text-sm text-muted-foreground mt-1">Personalized picks based on what you watch and subscribe to</p>
    </div>

    <SkeletonGrid v-if="loading" :count="12" />
    <ErrorState v-else-if="error" :message="error" retryable @retry="loadForYouFeed" />
    <EmptyState v-else-if="videos.length === 0" :title="t('emptyStates.noTrendingVideos')" icon="trending">
      Subscribe to channels and watch videos to get personalized recommendations.
    </EmptyState>
    <div v-else class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3">
      <VideoCard
        v-for="video in videos"
        :key="video.id"
        :video="video"
      />
    </div>
  </div>
</template>

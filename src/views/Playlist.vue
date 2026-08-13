<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRoute } from 'vue-router'
import { getPlaylistInfo } from '../api'
import type { Playlist, Video } from '../api/types'
import VideoCard from '../components/VideoCard.vue'
import ErrorState from '../components/ui/ErrorState.vue'
import EmptyState from '@/components/ui/EmptyState.vue'

const route = useRoute()

const playlistId = computed(() => route.params.id as string || '')
const isLoading = ref(true)
const error = ref<string | null>(null)
const playlist = ref<Playlist | null>(null)
const videos = ref<Video[]>([])

const isWatchLater = computed(() => playlistId.value === 'watch-later')

async function loadPlaylist() {
  if (!playlistId.value) return
  isLoading.value = true
  error.value = null
  try {
    const data = await getPlaylistInfo(playlistId.value)
    playlist.value = data
    videos.value = data.videos || []
  } catch (e: any) {
    error.value = e.message || 'Failed to load playlist'
  } finally {
    isLoading.value = false
  }
}

onMounted(loadPlaylist)
</script>

<template>
  <div class="p-6">
    <!-- Loading State -->
    <div v-if="isLoading" class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-1">
      <div v-for="n in 6" :key="n" class="aspect-video bg-muted rounded-xl animate-pulse" />
    </div>

    <!-- Error State -->
    <ErrorState v-else-if="error" :message="error" retryable @retry="loadPlaylist" />

    <!-- Empty State -->
    <EmptyState v-else-if="!playlist" title="Playlist not found">
      The requested playlist could not be loaded.
    </EmptyState>

    <template v-else>
      <!-- Playlist Header -->
      <div class="flex items-center justify-between mb-6">
        <div>
          <h1 class="text-2xl font-bold text-foreground">
            {{ isWatchLater ? 'Watch Later' : playlist.title }}
          </h1>
          <p class="text-sm text-muted-foreground mt-1">
            {{ videos.length }} videos
            <span v-if="!isWatchLater"> &middot; By {{ playlist.author }}</span>
          </p>
        </div>
        <button
          v-if="videos.length > 0"
          class="inline-flex items-center gap-2 px-4 py-2 bg-primary text-primary-foreground rounded-lg text-sm font-medium hover:bg-primary/90 transition-colors"
        >
          <svg class="size-4" viewBox="0 0 24 24" fill="currentColor">
            <polygon points="5 3 19 12 5 21 5 3" />
          </svg>
          Play All
        </button>
      </div>

      <!-- Video Grid -->
      <div v-if="videos.length > 0" class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-1">
        <VideoCard v-for="video in videos" :key="video.id" :video="video" />
      </div>

      <!-- Empty Playlist -->
      <EmptyState v-else title="No videos in this playlist">
        {{ isWatchLater ? 'Save videos to watch later by clicking the clock icon.' : 'Add videos to this playlist to watch them later.' }}
      </EmptyState>
    </template>
  </div>
</template>

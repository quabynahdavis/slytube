<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRoute } from 'vue-router'
import { getPlaylistInfo } from '../api'
import type { Playlist, Video } from '../api/types'
import VideoCard from '../components/VideoCard.vue'
import SkeletonGrid from '../components/ui/SkeletonGrid.vue'
import EmptyState from '@/components/ui/EmptyState.vue'
import ErrorState from '@/components/ui/ErrorState.vue'
import { PhPlayCircle, PhMagnifyingGlass } from '@phosphor-icons/vue'

const route = useRoute()

const playlistId = computed(() => route.params.id as string || '')
const isLoading = ref(true)
const error = ref<string | null>(null)
const playlist = ref<Playlist | null>(null)
const videos = ref<Video[]>([])
const searchQuery = ref('')
const activeTab = ref('videos')

const isWatchLater = computed(() => playlistId.value === 'watch-later')

const tabs = [
  { id: 'videos', label: 'Videos' },
  { id: 'shorts', label: 'Shorts' },
]

const filteredVideos = computed(() => {
  let result = videos.value

  if (activeTab.value === 'shorts') {
    result = result.filter(v => v.isShort)
  } else {
    result = result.filter(v => !v.isShort)
  }

  if (searchQuery.value.trim()) {
    const q = searchQuery.value.toLowerCase()
    result = result.filter(v =>
      v.title.toLowerCase().includes(q) ||
      v.author.toLowerCase().includes(q)
    )
  }

  return result
})

const videoCount = computed(() => videos.value.filter(v => !v.isShort).length)
const shortsCount = computed(() => videos.value.filter(v => v.isShort).length)

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

function clearSearch() {
  searchQuery.value = ''
}

onMounted(loadPlaylist)
</script>

<template>
  <div class="p-6">
    <!-- Loading State -->
    <SkeletonGrid v-if="isLoading" :count="6" />

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
          <svg class="size-4" viewBox="0.0 24 24" fill="currentColor">
            <polygon points="5 3 19 12 5 21 5 3" />
          </svg>
          Play All
        </button>
      </div>

      <!-- Tabs & Search -->
      <div class="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4 mb-6 pb-4 border-b border-border">
        <!-- Tabs -->
        <nav class="flex gap-4">
          <button
            v-for="tab in tabs"
            :key="tab.id"
            class="flex items-center gap-2 pb-2 text-sm font-medium border-b-2 transition-colors"
            :class="activeTab === tab.id
              ? 'border-primary text-foreground'
              : 'border-transparent text-muted-foreground hover:text-foreground'"
            @click="activeTab = tab.id"
          >
            <PhPlayCircle v-if="tab.id === 'shorts'" :size="16" />
            {{ tab.label }}
            <span class="text-xs text-muted-foreground">
              ({{ tab.id === 'shorts' ? shortsCount : videoCount }})
            </span>
          </button>
        </nav>

        <!-- Search -->
        <div class="relative w-full sm:w-64">
          <PhMagnifyingGlass :size="16" class="absolute left-3 top-1/2 -translate-y-1/2 text-muted-foreground" />
          <input
            v-model="searchQuery"
            type="text"
            placeholder="Search in playlist..."
            class="w-full h-9 pl-9 pr-9 rounded-lg border border-input bg-background text-sm outline-none focus:border-primary focus:ring-1 focus:ring-primary"
          />
          <button
            v-if="searchQuery"
            class="absolute right-3 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground"
            @click="clearSearch"
          >
            <svg class="size-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <line x1="18" y1="6" x2="6" y2="18" />
              <line x1="6" y1="6" x2="18" y2="18" />
            </svg>
          </button>
        </div>
      </div>

      <!-- Video Grid -->
      <div v-if="filteredVideos.length > 0" class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-1">
        <VideoCard v-for="video in filteredVideos" :key="video.id" :video="video" />
      </div>

      <!-- Empty for current tab/search -->
      <EmptyState v-else-if="searchQuery.trim()" title="No results found">
        No videos matching "{{ searchQuery }}" in {{ activeTab === 'shorts' ? 'shorts' : 'videos' }}.
      </EmptyState>
      <EmptyState v-else :title="`No ${activeTab === 'shorts' ? 'shorts' : 'videos'} in this playlist`">
        {{ isWatchLater ? `Save ${activeTab === 'shorts' ? 'shorts' : 'videos'} to watch later by clicking the clock icon.` : `Add ${activeTab === 'shorts' ? 'shorts' : 'videos'} to this playlist.` }}
      </EmptyState>
    </template>
  </div>
</template>

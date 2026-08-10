<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRoute } from 'vue-router'
import { cn } from '@/lib/utils'
import { usePlaylistsStore } from '@/stores/playlists'

const route = useRoute()
const playlistsStore = usePlaylistsStore()

const playlistId = computed(() => route.params.id as string || '')
const isLoading = ref(true)
const isReordering = ref(false)

const playlist = ref({
  _id: '',
  playlistName: '',
  description: '',
  videos: [] as Array<{
    videoId: string
    title: string
    author: string
    authorId: string
    lengthSeconds: number
    timeAdded: number
    playlistItemId: string
    videoThumbnails: Array<{ url: string; width: number; height: number }>
  }>,
  protected: false,
  createdAt: 0,
  lastUpdatedAt: 0,
  lastPlayedAt: 0,
})

onMounted(async () => {
  isLoading.value = true
  try {
    await new Promise((resolve) => setTimeout(resolve, 500))
    playlist.value = {
      _id: playlistId.value,
      playlistName: 'My Playlist',
      description: 'A collection of favorite videos',
      videos: Array.from({ length: 15 }, (_, i) => ({
        videoId: `playlist-video-${i}`,
        title: `Playlist Video ${i + 1}`,
        author: `Channel ${i + 1}`,
        authorId: `UC-channel-${i}`,
        lengthSeconds: Math.floor(Math.random() * 600) + 60,
        timeAdded: Date.now() - i * 86400000,
        playlistItemId: `pli-${i}`,
        videoThumbnails: [{ url: '', width: 320, height: 180 }],
      })),
      protected: false,
      createdAt: Date.now() - 30 * 86400000,
      lastUpdatedAt: Date.now(),
      lastPlayedAt: Date.now() - 86400000,
    }
  } finally {
    isLoading.value = false
  }
})

function formatDuration(seconds: number): string {
  const h = Math.floor(seconds / 3600)
  const m = Math.floor((seconds % 3600) / 60)
  const s = seconds % 60
  if (h > 0) return `${h}:${m.toString().padStart(2, '0')}:${s.toString().padStart(2, '0')}`
  return `${m}:${s.toString().padStart(2, '0')}`
}

function moveVideo(index: number, direction: 'up' | 'down') {
  const newIndex = direction === 'up' ? index - 1 : index + 1
  if (newIndex < 0 || newIndex >= playlist.value.videos.length) return
  const videos = playlist.value.videos
  const [item] = videos.splice(index, 1)
  videos.splice(newIndex, 0, item)
}

function removeFromPlaylist(videoId: string) {
  playlist.value.videos = playlist.value.videos.filter((v) => v.videoId !== videoId)
}
</script>

<template>
  <div class="container mx-auto max-w-5xl px-4 py-6">
    <!-- Loading State -->
    <div v-if="isLoading" class="animate-pulse">
      <div class="h-8 w-64 rounded bg-muted mb-2" />
      <div class="h-4 w-48 rounded bg-muted mb-6" />
      <div class="space-y-3">
        <div v-for="n in 5" :key="n" class="flex gap-3">
          <div class="w-32 aspect-video rounded bg-muted shrink-0" />
          <div class="flex-1 space-y-2">
            <div class="h-4 w-3/4 rounded bg-muted" />
            <div class="h-3 w-1/2 rounded bg-muted" />
          </div>
        </div>
      </div>
    </div>

    <template v-else>
      <!-- Playlist Header -->
      <div class="mb-6">
        <div class="flex items-start justify-between gap-4">
          <div>
            <h1 class="text-2xl font-bold text-foreground">{{ playlist.playlistName }}</h1>
            <p class="text-sm text-muted-foreground mt-1">{{ playlist.description }}</p>
            <p class="text-xs text-muted-foreground mt-2">
              {{ playlist.videos.length }} videos &middot; Last updated {{ new Date(playlist.lastUpdatedAt).toLocaleDateString() }}
            </p>
          </div>
          <div class="flex items-center gap-2 shrink-0">
            <button
              :class="cn(
                'inline-flex items-center gap-1 rounded-md border border-border px-3 py-2 text-sm transition-colors',
                isReordering ? 'bg-accent text-accent-foreground' : 'text-muted-foreground hover:bg-accent'
              )"
              @click="isReordering = !isReordering"
            >
              <svg class="size-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <line x1="8" y1="6" x2="21" y2="6" />
                <line x1="8" y1="12" x2="21" y2="12" />
                <line x1="8" y1="18" x2="21" y2="18" />
                <line x1="3" y1="6" x2="3.01" y2="6" />
                <line x1="3" y1="12" x2="3.01" y2="12" />
                <line x1="3" y1="18" x2="3.01" y2="18" />
              </svg>
              Reorder
            </button>
            <button class="inline-flex items-center gap-1 rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90 transition-colors">
              <svg class="size-4" viewBox="0 0 24 24" fill="currentColor">
                <polygon points="5 3 19 12 5 21 5 3" />
              </svg>
              Play All
            </button>
          </div>
        </div>
      </div>

      <!-- Video List -->
      <div class="space-y-2">
        <div
          v-for="(video, index) in playlist.videos"
          :key="video.videoId"
          :class="cn(
            'flex items-center gap-3 rounded-lg border border-border bg-card p-2 group transition-colors hover:bg-accent/50',
            isReordering && 'border-dashed'
          )"
        >
          <!-- Index / Reorder Controls -->
          <div class="w-8 text-center shrink-0">
            <template v-if="isReordering">
              <div class="flex flex-col gap-0.5">
                <button
                  :disabled="index === 0"
                  class="size-4 text-muted-foreground hover:text-foreground disabled:opacity-30"
                  @click="moveVideo(index, 'up')"
                >
                  <svg class="size-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <polyline points="18 15 12 9 6 15" />
                  </svg>
                </button>
                <button
                  :disabled="index === playlist.videos.length - 1"
                  class="size-4 text-muted-foreground hover:text-foreground disabled:opacity-30"
                  @click="moveVideo(index, 'down')"
                >
                  <svg class="size-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <polyline points="6 9 12 15 18 9" />
                  </svg>
                </button>
              </div>
            </template>
            <template v-else>
              <span class="text-sm text-muted-foreground">{{ index + 1 }}</span>
            </template>
          </div>

          <!-- Thumbnail -->
          <router-link
            :to="`/watch?v=${video.videoId}`"
            class="relative shrink-0 w-32 aspect-video rounded bg-muted overflow-hidden"
          >
            <div class="absolute inset-0 flex items-center justify-center">
              <svg class="size-8 text-muted-foreground/50 group-hover:text-primary transition-colors" viewBox="0 0 24 24" fill="currentColor">
                <polygon points="5 3 19 12 5 21 5 3" />
              </svg>
            </div>
            <span class="absolute bottom-1 right-1 rounded bg-black/80 px-1 text-xs text-white">
              {{ formatDuration(video.lengthSeconds) }}
            </span>
          </router-link>

          <!-- Video Info -->
          <div class="min-w-0 flex-1">
            <router-link
              :to="`/watch?v=${video.videoId}`"
              class="text-sm font-medium text-foreground line-clamp-1 hover:text-primary transition-colors"
            >
              {{ video.title }}
            </router-link>
            <p class="text-xs text-muted-foreground mt-0.5">{{ video.author }}</p>
          </div>

          <!-- Actions -->
          <div class="flex items-center gap-1 shrink-0 opacity-0 group-hover:opacity-100 transition-opacity">
            <button
              class="size-8 rounded-md text-muted-foreground hover:bg-accent hover:text-accent-foreground flex items-center justify-center"
              title="Remove from playlist"
              @click="removeFromPlaylist(video.videoId)"
            >
              <svg class="size-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <line x1="18" y1="6" x2="6" y2="18" />
                <line x1="6" y1="6" x2="18" y2="18" />
              </svg>
            </button>
          </div>
        </div>
      </div>
    </template>
  </div>
</template>

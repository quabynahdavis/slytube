<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRoute } from 'vue-router'
import { cn } from '@/lib/utils'
import { getPlaylistInfo } from '../api'
import type { Playlist, Video } from '../api/types'
import SkeletonGrid from '../components/ui/SkeletonGrid.vue'
import ErrorState from '../components/ui/ErrorState.vue'
import EmptyState from '../components/ui/EmptyState.vue'

const route = useRoute()

const playlistId = computed(() => route.params.id as string || '')
const isLoading = ref(true)
const error = ref<string | null>(null)
const playlist = ref<Playlist | null>(null)
const videos = ref<Video[]>([])
const isReordering = ref(false)

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

function formatDuration(seconds: number): string {
  const h = Math.floor(seconds / 3600)
  const m = Math.floor((seconds % 3600) / 60)
  const s = seconds % 60
  if (h > 0) return `${h}:${m.toString().padStart(2, '0')}:${s.toString().padStart(2, '0')}`
  return `${m}:${s.toString().padStart(2, '0')}`
}

function moveVideo(index: number, direction: 'up' | 'down') {
  const newIndex = direction === 'up' ? index - 1 : index + 1
  if (newIndex < 0 || newIndex >= videos.value.length) return
  const items = videos.value
  const [item] = items.splice(index, 1)
  items.splice(newIndex, 0, item)
}

function removeFromPlaylist(videoId: string) {
  videos.value = videos.value.filter((v) => v.id !== videoId)
}
</script>

<template>
  <div class="container mx-auto max-w-5xl px-4 py-6">
    <!-- Loading State -->
    <SkeletonGrid v-if="isLoading" :count="5" />

    <!-- Error State -->
    <ErrorState v-else-if="error" :message="error" retryable @retry="loadPlaylist" />

    <!-- Empty State -->
    <EmptyState v-else-if="!playlist" title="Playlist not found">
      The requested playlist could not be loaded.
    </EmptyState>

    <template v-else>
      <!-- Playlist Header -->
      <div class="mb-6">
        <div class="flex items-start justify-between gap-4">
          <div>
            <h1 class="text-2xl font-bold text-foreground">{{ playlist.title }}</h1>
            <p class="text-sm text-muted-foreground mt-1">{{ playlist.description }}</p>
            <p class="text-xs text-muted-foreground mt-2">
              {{ videos.length }} videos &middot; By {{ playlist.author }}
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
      <div v-if="videos.length > 0" class="space-y-2">
        <div
          v-for="(video, index) in videos"
          :key="video.id"
          :class="cn(
            'flex items-center gap-3 rounded-lg border border-border bg-card p-2 group transition-colors hover:bg-primary/8',
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
                  :disabled="index === videos.length - 1"
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
            :to="`/watch?v=${video.id}`"
            class="relative shrink-0 w-32 aspect-video rounded bg-muted overflow-hidden"
          >
            <img
              v-if="video.thumbnail"
              :src="video.thumbnail"
              :alt="video.title"
              class="absolute inset-0 w-full h-full object-cover"
            />
            <div v-else class="absolute inset-0 flex items-center justify-center">
              <svg class="size-8 text-muted-foreground/50 group-hover:text-primary transition-colors" viewBox="0 0 24 24" fill="currentColor">
                <polygon points="5 3 19 12 5 21 5 3" />
              </svg>
            </div>
            <span v-if="video.lengthSeconds > 0" class="absolute bottom-1 right-1 rounded bg-black/80 px-1 text-xs text-white">
              {{ formatDuration(video.lengthSeconds) }}
            </span>
          </router-link>

          <!-- Video Info -->
          <div class="min-w-0 flex-1">
            <router-link
              :to="`/watch?v=${video.id}`"
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
              @click="removeFromPlaylist(video.id)"
            >
              <svg class="size-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <line x1="18" y1="6" x2="6" y2="18" />
                <line x1="6" y1="6" x2="18" y2="18" />
              </svg>
            </button>
          </div>
        </div>
      </div>

      <!-- Empty Playlist -->
      <EmptyState v-else title="Empty playlist">
        Add videos to this playlist to watch them later.
      </EmptyState>
    </template>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRoute } from 'vue-router'
import { cn } from '@/lib/utils'
import { usePlayerStore } from '@/stores/player'
import { useHistoryStore } from '@/stores/history'
import { useWatchQueueStore } from '@/stores/watch-queue'

const route = useRoute()
const playerStore = usePlayerStore()
const historyStore = useHistoryStore()
const watchQueueStore = useWatchQueueStore()

const videoId = computed(() => route.query.v as string || route.params.id as string || '')
const isLoading = ref(true)
const showDownloadDialog = ref(false)
const showFullDescription = ref(false)

// Placeholder video data
const video = ref({
  title: 'Loading...',
  author: '',
  authorId: '',
  authorUrl: '',
  description: '',
  viewCount: 0,
  lengthSeconds: 0,
  uploadDate: '',
  likeCount: 0,
  videoThumbnails: [] as Array<{ url: string; width: number; height: number }>,
})

const relatedVideos = ref<Array<{
  videoId: string
  title: string
  author: string
  viewCount: number
  lengthSeconds: number
  videoThumbnails: Array<{ url: string; width: number; height: number }>
}>>([])

const comments = ref<unknown[]>([])
const downloadFormat = ref('video:best')
const downloadQuality = ref('720')

onMounted(async () => {
  isLoading.value = true
  try {
    // Placeholder: Fetch video data from API
    await new Promise((resolve) => setTimeout(resolve, 800))
    video.value = {
      title: 'Sample Video Title',
      author: 'Sample Channel',
      authorId: 'UCxxxxxxxx',
      authorUrl: '/channel/UCxxxxxxxx',
      description: 'This is a sample video description. It contains information about the video content and other relevant details that viewers might find useful.',
      viewCount: 1234567,
      lengthSeconds: 360,
      uploadDate: '2024-01-15',
      likeCount: 45000,
      videoThumbnails: [
        { url: '', width: 640, height: 360 },
      ],
    }
    relatedVideos.value = Array.from({ length: 12 }, (_, i) => ({
      videoId: `related-${i}`,
      title: `Related Video ${i + 1}`,
      author: `Channel ${i + 1}`,
      viewCount: Math.floor(Math.random() * 1000000),
      lengthSeconds: Math.floor(Math.random() * 600),
      videoThumbnails: [{ url: '', width: 320, height: 180 }],
    }))
  } finally {
    isLoading.value = false
  }
})

function formatViews(views: number): string {
  if (views >= 1000000) return `${(views / 1000000).toFixed(1)}M views`
  if (views >= 1000) return `${(views / 1000).toFixed(1)}K views`
  return `${views} views`
}

function formatDuration(seconds: number): string {
  const h = Math.floor(seconds / 3600)
  const m = Math.floor((seconds % 3600) / 60)
  const s = seconds % 60
  if (h > 0) return `${h}:${m.toString().padStart(2, '0')}:${s.toString().padStart(2, '0')}`
  return `${m}:${s.toString().padStart(2, '0')}`
}

function addToQueue() {
  watchQueueStore.addVideoToWatchQueue({
    videoId: videoId.value,
    title: video.value.title,
    author: video.value.author,
    authorId: video.value.authorId,
    lengthSeconds: video.value.lengthSeconds,
    videoThumbnails: video.value.videoThumbnails,
  })
}

function startDownload() {
  showDownloadDialog.value = false
  // Placeholder: Start download
}
</script>

<template>
  <div class="container mx-auto max-w-7xl px-4 py-4">
    <!-- Loading State -->
    <div v-if="isLoading" class="flex items-center justify-center h-96">
      <div class="flex flex-col items-center gap-3">
        <svg class="size-8 animate-spin text-primary" viewBox="0 0 24 24" fill="none">
          <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" />
          <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z" />
        </svg>
        <p class="text-sm text-muted-foreground">Loading video...</p>
      </div>
    </div>

    <div v-else class="flex flex-col lg:flex-row gap-6">
      <!-- Main Content -->
      <div class="flex-1 min-w-0">
        <!-- Video Player Placeholder -->
        <div class="relative w-full aspect-video rounded-lg bg-black overflow-hidden">
          <div class="absolute inset-0 flex items-center justify-center">
            <div class="text-center text-white/80">
              <svg class="size-16 mx-auto mb-3" viewBox="0 0 24 24" fill="currentColor">
                <polygon points="5 3 19 12 5 21 5 3" />
              </svg>
              <p class="text-sm">Video Player (shaka-player placeholder)</p>
              <p class="text-xs text-white/50 mt-1">Video ID: {{ videoId }}</p>
            </div>
          </div>
          <!-- Player Controls Overlay -->
          <div class="absolute bottom-0 left-0 right-0 bg-gradient-to-t from-black/80 to-transparent p-4">
            <div class="flex items-center gap-3">
              <button class="text-white hover:text-primary transition-colors">
                <svg class="size-6" viewBox="0 0 24 24" fill="currentColor">
                  <polygon points="5 3 19 12 5 21 5 3" />
                </svg>
              </button>
              <div class="flex-1 h-1 bg-white/30 rounded-full">
                <div class="h-full w-1/3 bg-primary rounded-full" />
              </div>
              <span class="text-xs text-white/80">2:00 / 6:00</span>
              <button class="text-white hover:text-primary transition-colors">
                <svg class="size-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5" />
                  <path d="M19.07 4.93a10 10 0 0 1 0 14.14M15.54 8.46a5 5 0 0 1 0 7.07" />
                </svg>
              </button>
              <button class="text-white hover:text-primary transition-colors">
                <svg class="size-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <rect x="2" y="2" width="20" height="20" rx="2.18" ry="2.18" />
                  <line x1="7" y1="2" x2="7" y2="22" />
                  <line x1="17" y1="2" x2="17" y2="22" />
                  <line x1="2" y1="12" x2="22" y2="12" />
                </svg>
              </button>
              <button class="text-white hover:text-primary transition-colors">
                <svg class="size-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z" />
                </svg>
              </button>
            </div>
          </div>
        </div>

        <!-- Video Info -->
        <div class="mt-4">
          <h1 class="text-xl font-semibold text-foreground">{{ video.title }}</h1>

          <!-- Author & Actions -->
          <div class="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-3 mt-3">
            <div class="flex items-center gap-3">
              <div class="size-10 rounded-full bg-muted flex items-center justify-center">
                <svg class="size-6 text-muted-foreground" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2" />
                  <circle cx="12" cy="7" r="4" />
                </svg>
              </div>
              <div>
                <router-link
                  :to="video.authorUrl"
                  class="text-sm font-medium text-foreground hover:text-primary"
                >
                  {{ video.author }}
                </router-link>
                <p class="text-xs text-muted-foreground">Subscribers</p>
              </div>
              <button class="ml-2 h-9 rounded-full bg-primary px-4 text-sm font-medium text-primary-foreground hover:bg-primary/90 transition-colors">
                Subscribe
              </button>
            </div>

            <div class="flex items-center gap-2">
              <!-- Like/Dislike -->
              <div class="flex items-center rounded-full border border-border overflow-hidden">
                <button class="flex items-center gap-1 px-3 py-2 text-sm text-foreground hover:bg-accent transition-colors">
                  <svg class="size-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <path d="M14 9V5a3 3 0 0 0-3-3l-4 9v11h11.28a2 2 0 0 0 2-1.7l1.38-9a2 2 0 0 0-2-2.3zM7 22H4a2 2 0 0 1-2-2v-7a2 2 0 0 1 2-2h3" />
                  </svg>
                  <span>{{ video.likeCount?.toLocaleString() }}</span>
                </button>
                <div class="w-px h-6 bg-border" />
                <button class="px-3 py-2 text-sm text-foreground hover:bg-accent transition-colors">
                  <svg class="size-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <path d="M10 15v4a3 3 0 0 0 3 3l4-9V2H5.72a2 2 0 0 0-2 1.7l-1.38 9a2 2 0 0 0 2 2.3zm7-13h2.67A2.31 2.31 0 0 1 22 4v7a2.31 2.31 0 0 1-2.33 2H17" />
                  </svg>
                </button>
              </div>

              <!-- Share -->
              <button class="flex items-center gap-1 rounded-full border border-border px-3 py-2 text-sm text-foreground hover:bg-accent transition-colors">
                <svg class="size-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <circle cx="18" cy="5" r="3" />
                  <circle cx="6" cy="12" r="3" />
                  <circle cx="18" cy="19" r="3" />
                  <line x1="8.59" y1="13.51" x2="15.42" y2="17.49" />
                  <line x1="15.41" y1="6.51" x2="8.59" y2="10.49" />
                </svg>
                <span class="hidden sm:inline">Share</span>
              </button>

              <!-- Download -->
              <button
                class="flex items-center gap-1 rounded-full border border-border px-3 py-2 text-sm text-foreground hover:bg-accent transition-colors"
                @click="showDownloadDialog = true"
              >
                <svg class="size-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
                  <polyline points="7 10 12 15 17 10" />
                  <line x1="12" y1="15" x2="12" y2="3" />
                </svg>
                <span class="hidden sm:inline">Download</span>
              </button>

              <!-- Add to Queue -->
              <button
                class="flex items-center gap-1 rounded-full border border-border px-3 py-2 text-sm text-foreground hover:bg-accent transition-colors"
                @click="addToQueue"
                title="Add to queue"
              >
                <svg class="size-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <line x1="12" y1="5" x2="12" y2="19" />
                  <line x1="5" y1="12" x2="19" y2="12" />
                </svg>
                <span class="hidden sm:inline">Queue</span>
              </button>
            </div>
          </div>

          <!-- Description -->
          <div class="mt-4 rounded-lg bg-muted/50 p-3">
            <div class="flex items-center gap-2 text-sm font-medium text-foreground">
              <span>{{ formatViews(video.viewCount) }}</span>
              <span class="text-muted-foreground">{{ video.uploadDate }}</span>
            </div>
            <p :class="cn('mt-2 text-sm text-muted-foreground whitespace-pre-wrap', !showFullDescription && 'line-clamp-3')">
              {{ video.description }}
            </p>
            <button
              class="mt-1 text-sm font-medium text-foreground hover:text-primary"
              @click="showFullDescription = !showFullDescription"
            >
              {{ showFullDescription ? 'Show less' : 'Show more' }}
            </button>
          </div>
        </div>

        <!-- Comments Section -->
        <div class="mt-6">
          <h2 class="text-lg font-semibold text-foreground mb-4">Comments</h2>
          <div class="rounded-lg border border-border bg-card p-6 text-center text-muted-foreground">
            <svg class="size-12 mx-auto mb-3 text-muted-foreground/50" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
              <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z" />
            </svg>
            <p class="text-sm">Comments section placeholder</p>
          </div>
        </div>
      </div>

      <!-- Related Videos Sidebar -->
      <aside class="lg:w-96 shrink-0">
        <h2 class="text-lg font-semibold text-foreground mb-3">Related Videos</h2>
        <div class="space-y-3">
          <div
            v-for="related in relatedVideos"
            :key="related.videoId"
            class="flex gap-3 group cursor-pointer"
          >
            <div class="relative shrink-0 w-40 aspect-video rounded-lg bg-muted overflow-hidden">
              <div class="absolute inset-0 flex items-center justify-center">
                <svg class="size-8 text-muted-foreground/50" viewBox="0 0 24 24" fill="currentColor">
                  <polygon points="5 3 19 12 5 21 5 3" />
                </svg>
              </div>
              <span class="absolute bottom-1 right-1 rounded bg-black/80 px-1 text-xs text-white">
                {{ formatDuration(related.lengthSeconds) }}
              </span>
            </div>
            <div class="min-w-0">
              <h3 class="text-sm font-medium text-foreground line-clamp-2 group-hover:text-primary transition-colors">
                {{ related.title }}
              </h3>
              <p class="mt-1 text-xs text-muted-foreground">{{ related.author }}</p>
              <p class="text-xs text-muted-foreground">{{ formatViews(related.viewCount) }}</p>
            </div>
          </div>
        </div>
      </aside>
    </div>

    <!-- Download Dialog -->
    <Teleport to="body">
      <div
        v-if="showDownloadDialog"
        class="fixed inset-0 z-50 flex items-center justify-center bg-black/50"
        @click.self="showDownloadDialog = false"
      >
        <div class="w-full max-w-md rounded-lg bg-card border border-border p-6 shadow-xl">
          <h3 class="text-lg font-semibold text-foreground mb-4">Download Video</h3>
          <div class="space-y-4">
            <div>
              <label class="text-sm font-medium text-foreground">Format</label>
              <select
                v-model="downloadFormat"
                class="mt-1 h-9 w-full rounded-md border border-input bg-background px-3 text-sm"
              >
                <option value="video:best">Best Video</option>
                <option value="video:720">720p Video</option>
                <option value="video:1080">1080p Video</option>
                <option value="audio:best">Audio Only</option>
              </select>
            </div>
            <div>
              <label class="text-sm font-medium text-foreground">Quality</label>
              <select
                v-model="downloadQuality"
                class="mt-1 h-9 w-full rounded-md border border-input bg-background px-3 text-sm"
              >
                <option value="auto">Auto</option>
                <option value="144">144p</option>
                <option value="360">360p</option>
                <option value="480">480p</option>
                <option value="720">720p</option>
                <option value="1080">1080p</option>
              </select>
            </div>
            <div class="flex justify-end gap-2 pt-2">
              <button
                class="h-9 rounded-md border border-input bg-background px-4 text-sm font-medium text-foreground hover:bg-accent transition-colors"
                @click="showDownloadDialog = false"
              >
                Cancel
              </button>
              <button
                class="h-9 rounded-md bg-primary px-4 text-sm font-medium text-primary-foreground hover:bg-primary/90 transition-colors"
                @click="startDownload"
              >
                Download
              </button>
            </div>
          </div>
        </div>
      </div>
    </Teleport>
  </div>
</template>

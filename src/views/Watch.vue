<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { PhHouse, PhCaretRight, PhPlus, PhDownload, PhClock } from '@phosphor-icons/vue'
import { useDownloads } from '../composables/useData'
import { useSubscriptionsStore } from '../stores/subscriptions'
import { usePlaylistsStore } from '../stores/playlists'
import type { Video } from '../api/types'
import type { SponsorBlockSegment } from '../api/sponsorblock'
import ErrorState from '../components/ui/ErrorState.vue'
import EmptyState from '../components/ui/EmptyState.vue'
import ShakaPlayer from '../components/player/ShakaPlayer.vue'

const route = useRoute()
const router = useRouter()
const videoId = computed(() => route.query.v as string || route.params.id as string || '')
const loading = ref(false)
const error = ref<string | null>(null)
const playerError = ref<string | null>(null)
const manifestUrl = ref<string>('')
const selectedFormatUrl = ref<string>('')

const subscriptionsStore = useSubscriptionsStore()
const playlistsStore = usePlaylistsStore()
const { startDownload } = useDownloads()

// Dummy data for UI development
const dummyComments = [
  { id: '1', author: 'TechReviewer', authorAvatar: '', content: 'This is an amazing video! Really well explained and the production quality is top notch.', published: '2 days ago', likeCount: 1240, replyCount: 45 },
  { id: '2', author: 'CodeMaster', authorAvatar: '', content: 'Great tutorial! I learned so much from this. Can you make more content like this?', published: '5 days ago', likeCount: 890, replyCount: 23 },
  { id: '3', author: 'DevEnthusiast', authorAvatar: '', content: 'The part about architecture patterns was incredibly helpful. Thanks for sharing your knowledge!', published: '1 week ago', likeCount: 567, replyCount: 12 },
  { id: '4', author: 'NewbieCoder', authorAvatar: '', content: 'Can someone explain the part at 5:30? I did not quite understand that concept.', published: '2 weeks ago', likeCount: 234, replyCount: 8 },
  { id: '5', author: 'SeniorDev', authorAvatar: '', content: 'Finally someone explaining this properly. Subscribed immediately!', published: '3 weeks ago', likeCount: 1890, replyCount: 67 },
]

const dummyRelated = [
  { id: 'abc123', title: 'Building Scalable Applications with Modern Architecture', author: 'TechChannel', authorId: 'UCtech', authorUrl: '/channel/UCtech', authorAvatar: '', description: '', thumbnail: '', viewCount: 1200000, likeCount: 45000, lengthSeconds: 1245, published: '1 month ago', isLive: false, isUpcoming: false, isShort: false, chapters: [], captions: [], related: [] },
  { id: 'def456', title: 'Understanding Design Patterns in 10 Minutes', author: 'CodeSimplified', authorId: 'UCcode', authorUrl: '/channel/UCcode', authorAvatar: '', description: '', thumbnail: '', viewCount: 890000, likeCount: 32000, lengthSeconds: 623, published: '2 months ago', isLive: false, isUpcoming: false, isShort: false, chapters: [], captions: [], related: [] },
  { id: 'ghi789', title: 'The Future of Web Development - What You Need to Know', author: 'WebDevPro', authorId: 'UCweb', authorUrl: '/channel/UCweb', authorAvatar: '', description: '', thumbnail: '', viewCount: 2100000, likeCount: 78000, lengthSeconds: 1834, published: '3 weeks ago', isLive: false, isUpcoming: false, isShort: false, chapters: [], captions: [], related: [] },
  { id: 'jkl012', title: 'Advanced TypeScript Tips and Tricks', author: 'TypeScriptMaster', authorId: 'UCts', authorUrl: '/channel/UCts', authorAvatar: '', description: '', thumbnail: '', viewCount: 560000, likeCount: 21000, lengthSeconds: 892, published: '1 month ago', isLive: false, isUpcoming: false, isShort: false, chapters: [], captions: [], related: [] },
  { id: 'mno345', title: 'Why You Should Start Using Rust in 2026', author: 'Rustacean', authorId: 'UCrust', authorUrl: '/channel/UCrust', authorAvatar: '', description: '', thumbnail: '', viewCount: 780000, likeCount: 29000, lengthSeconds: 1456, published: '2 months ago', isLive: false, isUpcoming: false, isShort: false, chapters: [], captions: [], related: [] },
]

const dummySegments = [
  { UUID: 'seg1', category: 'sponsor', segment: [45, 78] as [number, number], videoDuration: 600, actionType: 'skip' },
  { UUID: 'seg2', category: 'intro', segment: [0, 25] as [number, number], videoDuration: 600, actionType: 'skip' },
  { UUID: 'seg3', category: 'outro', segment: [540, 600] as [number, number], videoDuration: 600, actionType: 'skip' },
]

const video = ref<Video>({
  id: 'dQw4w9WgXcQ',
  title: 'Building a Modern YouTube Client with Vue 3 and Tauri - Complete Tutorial',
  author: 'TechChannel',
  authorId: 'UCtech123',
  authorUrl: '/channel/UCtech123',
  authorAvatar: '',
  description: `In this comprehensive tutorial, we build a full-featured YouTube client from scratch using Vue 3, TypeScript, and Tauri.

We cover:
- Setting up the Tauri project with Vue 3
- Integrating with the Invidious API for video data
- Implementing DASH video playback with Shaka Player
- Managing state with Pinia
- Building a responsive UI with Tailwind CSS
- Adding SponsorBlock integration
- Implementing subscription management
- Cross-device sync with a sync server

This is the first part of a multi-part series. Stay tuned for more!

Timestamps:
0:00 - Introduction
2:30 - Project Setup
8:15 - API Integration
15:40 - Video Player Setup
24:50 - UI Components
35:20 - State Management
42:10 - SponsorBlock Integration
50:00 - Conclusion`,
  thumbnail: '',
  viewCount: 1250000,
  likeCount: 45000,
  lengthSeconds: 3420,
  published: '2 weeks ago',
  isLive: false,
  isUpcoming: false,
  isShort: false,
  chapters: [],
  captions: [],
  related: dummyRelated,
})

const comments = dummyComments
const sponsorBlockSegments = dummySegments

async function load() {
  // Dummy load for UI development
  loading.value = false
  error.value = null
}

function formatViews(count: number): string {
  if (count >= 1_000_000) return `${(count / 1_000_000).toFixed(1)}M views`
  if (count >= 1_000) return `${(count / 1_000).toFixed(1)}K views`
  return `${count} views`
}

function formatDuration(seconds: number): string {
  const h = Math.floor(seconds / 3600)
  const m = Math.floor((seconds % 3600) / 60)
  const s = seconds % 60
  if (h > 0) return `${h}:${m.toString().padStart(2, '0')}:${s.toString().padStart(2, '0')}`
  return `${m}:${s.toString().padStart(2, '0')}`
}

const segments = computed(() => sponsorBlockSegments as SponsorBlockSegment[])

// Safely derive related videos, handling potential null/undefined at runtime
const relatedVideos = computed(() => video.value?.related?.slice(0, 10) || [])

// Subscription computed state for current video's channel
const isSubscribed = computed(() =>
  video.value ? subscriptionsStore.isSubscribed(video.value.authorId) : false
)
const isSubscribePending = computed(() =>
  video.value ? subscriptionsStore.isPending(video.value.authorId) : false
)

/**
 * Derives a category label from the referrer route.
 * Maps known route names to display labels, defaults to "Videos".
 */
const breadcrumbCategory = computed(() => {
  const ref = route.query.ref as string | undefined
  if (!ref) return 'Videos'
  const map: Record<string, string> = {
    trending: 'Trending',
    subscriptions: 'Subscriptions',
    history: 'History',
    search: 'Search Results',
    channel: 'Channel',
    hashtag: 'Hashtag',
    popular: 'Popular',
    playlists: 'Playlists',
    playlist: 'Playlist',
  }
  return map[ref] || 'Videos'
})

/** Route to navigate to when clicking the category breadcrumb. */
const breadcrumbCategoryRoute = computed(() => {
  const ref = route.query.ref as string | undefined
  if (!ref) return null
  // Map ref names that have corresponding routes
  const routeMap: Record<string, string> = {
    trending: '/trending',
    subscriptions: '/subscriptions',
    history: '/history',
    search: '/search',
    popular: '/popular',
    playlists: '/playlists',
  }
  return routeMap[ref] || null
})

/** Truncated video title for breadcrumb display (max 50 chars). */
const breadcrumbTitle = computed(() => {
  if (!video.value) return ''
  const title = video.value.title
  return title.length > 50 ? title.slice(0, 47) + '...' : title
})

/**
 * Handle subscribe button click with optimistic update.
 * Immediately toggles visual state, calls API in background,
 * and rolls back on failure.
 */
async function handleSubscribe() {
  if (!video.value) return
  const channelId = video.value.authorId
  const channelName = video.value.author

  const result = await subscriptionsStore.toggleSubscription(channelId, channelName)

  if (!result.success) {
    // Show user feedback on failure (could be a toast/notification)
    console.warn('Failed to update subscription')
  }
}

/**
 * Add video to quick bookmark playlist (Favorites)
 */
async function addToQuickBookmark() {
  if (!video.value) return
  const playlist = await playlistsStore.getQuickBookmarkPlaylist()
  if (playlist) {
    await playlistsStore.addToPlaylist(playlist._id, video.value.id)
  }
}

/**
 * Download the current video
 */
async function downloadVideo() {
  if (!video.value) return
  try {
    await startDownload({
      videoId: video.value.id,
      mode: 'video',
    })
  } catch (error) {
    console.error('Download failed:', error)
  }
}

/**
 * Save video to watch later playlist
 */
async function saveToWatchLater() {
  if (!video.value) return
  try {
    // Find or create "Watch later" playlist
    let watchLaterPlaylist = playlistsStore.playlists.find(
      (p) => p.playlistName === 'Watch later'
    )
    if (!watchLaterPlaylist) {
      watchLaterPlaylist = await playlistsStore.createPlaylist('Watch later', 'Watch later playlist')
    }
    if (watchLaterPlaylist) {
      await playlistsStore.addToPlaylist(watchLaterPlaylist._id, video.value.id)
    }
  } catch (error) {
    console.error('Save to watch later failed:', error)
  }
}

onMounted(() => {
  load()
})
</script>

<template>
  <div class="p-6">
    <template v-if="loading">
      <div class="aspect-video bg-muted rounded-xl mb-4 animate-pulse"></div>
      <div class="h-6 bg-muted rounded w-2/3 mb-2 animate-pulse"></div>
      <div class="h-4 bg-muted rounded w-1/3 animate-pulse"></div>
    </template>

    <ErrorState v-else-if="error" :message="error" retryable @retry="load" />

    <div v-else-if="video" class="grid grid-cols-1 lg:grid-cols-3 gap-6">
      <!-- Main Content -->
      <div class="lg:col-span-2 space-y-4">
        <!-- Shaka Player (DASH) -->
        <ShakaPlayer
          v-if="manifestUrl"
          :manifest-url="manifestUrl"
          :video-id="videoId"
          :title="video.title"
          :segments="segments"
          :chapters="video.chapters"
          @error="playerError = $event"
        />

        <!-- Direct Video Fallback -->
        <video
          v-else-if="selectedFormatUrl"
          :src="selectedFormatUrl"
          controls
          class="w-full aspect-video bg-black rounded-xl"
          @error="playerError = 'Failed to load video stream'"
        />

        <!-- Player Error Fallback -->
        <div v-if="playerError" class="bg-destructive/10 border border-destructive/20 rounded-xl p-4">
          <p class="text-sm text-destructive font-medium">Player Error: {{ playerError }}</p>
          <p class="text-xs text-muted-foreground mt-1">The video may be unavailable or require authentication.</p>
          <div v-if="selectedFormatUrl" class="mt-3">
            <a :href="selectedFormatUrl" target="_blank" class="text-sm text-primary hover:underline">
              Open stream in new tab
            </a>
          </div>
        </div>

        <!-- Breadcrumb Navigation -->
        <nav class="flex items-center gap-1.5 text-sm text-muted-foreground">
          <button
            class="flex items-center gap-1 hover:text-foreground transition-colors"
            @click="router.push('/')"
          >
            <PhHouse :size="14" weight="regular" />
            <span>Home</span>
          </button>
          <PhCaretRight :size="12" class="shrink-0" />
          <button
            v-if="breadcrumbCategoryRoute"
            class="hover:text-foreground transition-colors"
            @click="router.push(breadcrumbCategoryRoute)"
          >
            {{ breadcrumbCategory }}
          </button>
          <span v-else class="text-muted-foreground">{{ breadcrumbCategory }}</span>
          <PhCaretRight :size="12" class="shrink-0" />
          <span class="text-foreground truncate">{{ breadcrumbTitle }}</span>
        </nav>

        <!-- Video Info -->
        <div>
          <h1 class="text-xl font-bold text-foreground mb-2">{{ video.title }}</h1>
          <div class="flex items-center justify-between flex-wrap gap-4">
            <div class="flex items-center gap-3">
              <div class="size-10 rounded-full bg-primary/20 flex items-center justify-center">
                <span class="text-sm font-medium text-primary">{{ video.author[0] }}</span>
              </div>
              <div>
                <p class="text-sm font-medium text-foreground">{{ video.author }}</p>
                <p class="text-xs text-muted-foreground">{{ formatViews(video.viewCount) }}</p>
              </div>
              <button
                class="px-4 py-1.5 rounded-full text-sm font-medium transition-all duration-200 ease-in-out"
                :class="isSubscribed
                  ? 'bg-secondary text-secondary-foreground hover:bg-secondary/80'
                  : 'bg-primary text-primary-foreground hover:bg-primary/90'"
                :disabled="isSubscribePending"
                @click="handleSubscribe"
              >
                <span
                  class="inline-block transition-all duration-200"
                  :class="{ 'opacity-50': isSubscribePending }"
                >
                  {{ isSubscribePending ? '...' : isSubscribed ? 'Subscribed' : 'Subscribe' }}
                </span>
              </button>
            </div>
<div class="flex gap-2">
              <!-- Add to playlist (quick bookmark / Favorites) -->
              <button
                class="px-4 py-1.5 rounded-full text-sm flex items-center gap-2 transition-all duration-200 ease-in-out bg-secondary text-secondary-foreground hover:bg-secondary/80"
                @click="addToQuickBookmark"
              >
                <PhPlus class="size-4" />
                <span>Add to playlist</span>
              </button>
              <!-- Download -->
              <button
                class="px-4 py-1.5 rounded-full text-sm flex items-center gap-2 transition-all duration-200 ease-in-out bg-secondary text-secondary-foreground hover:bg-secondary/80"
                @click="downloadVideo"
              >
                <PhDownload class="size-4" />
                <span>Download</span>
              </button>
              <!-- Save to watch later -->
              <button
                class="px-4 py-1.5 rounded-full text-sm flex items-center gap-2 transition-all duration-200 ease-in-out bg-secondary text-secondary-foreground hover:bg-secondary/80"
                @click="saveToWatchLater"
              >
                <PhClock class="size-4" />
                <span>Watch later</span>
              </button>
              <!-- Share button (Coming Soon) -->
              <div class="relative group" title="Share — Coming Soon">
                <button
                  class="px-4 py-1.5 bg-secondary/60 text-secondary-foreground/50 rounded-full text-sm cursor-not-allowed"
                  disabled
                >
                  Share
                </button>
                <span class="absolute -top-6 left-1/2 -translate-x-1/2 px-1.5 py-0.5 bg-yellow-500/90 text-[9px] font-semibold text-black rounded opacity-0 group-hover:opacity-100 transition-opacity whitespace-nowrap pointer-events-none">
                  Soon
                </span>
              </div>
            </div>
          </div>
        </div>

        <!-- Description -->
        <div class="bg-card rounded-xl p-4">
          <p class="text-sm text-muted-foreground whitespace-pre-wrap">{{ video.description }}</p>
        </div>

        <!-- SponsorBlock Segments -->
        <div v-if="sponsorBlockSegments.length > 0" class="bg-card rounded-xl p-4">
          <h3 class="text-sm font-semibold text-foreground mb-3">SponsorBlock Segments</h3>
          <div class="space-y-2">
            <div
              v-for="seg in sponsorBlockSegments"
              :key="seg.UUID"
              class="flex items-center gap-3"
            >
              <div class="size-3 rounded" :style="{ backgroundColor: seg.category === 'sponsor' ? '#00d400' : seg.category === 'intro' ? '#00ffff' : '#0202ed' }"></div>
              <span class="text-sm text-foreground">{{ seg.category.charAt(0).toUpperCase() + seg.category.slice(1) }}</span>
              <span class="text-xs text-muted-foreground">{{ formatDuration(seg.segment[0]) }} - {{ formatDuration(seg.segment[1]) }}</span>
            </div>
          </div>
        </div>

        <!-- Comments -->
        <div class="bg-card rounded-xl p-4">
          <h3 class="text-sm font-semibold text-foreground mb-4">Comments ({{ comments.length }})</h3>
          <div v-if="comments.length > 0" class="space-y-4">
            <div v-for="comment in comments" :key="comment.id" class="flex gap-3">
              <img
                v-if="comment.authorAvatar"
                :src="comment.authorAvatar"
                :alt="comment.author"
                class="size-8 rounded-full shrink-0"
              />
              <div v-else class="size-8 rounded-full bg-primary/20 shrink-0 flex items-center justify-center">
                <span class="text-xs font-medium text-primary">{{ comment.author[0] }}</span>
              </div>
              <div class="flex-1 min-w-0">
                <div class="flex items-center gap-2">
                  <span class="text-sm font-medium text-foreground">{{ comment.author }}</span>
                  <span class="text-xs text-muted-foreground">{{ comment.published }}</span>
                </div>
                <p class="text-sm text-muted-foreground mt-1 whitespace-pre-wrap">{{ comment.content }}</p>
                <div class="flex items-center gap-3 mt-1">
                  <span class="text-xs text-muted-foreground">{{ comment.likeCount.toLocaleString() }} likes</span>
                  <span v-if="comment.replyCount > 0" class="text-xs text-muted-foreground">{{ comment.replyCount }} replies</span>
                </div>
              </div>
            </div>
          </div>
          <div v-else class="flex items-center gap-3 py-4 justify-center">
            <svg class="size-5 text-muted-foreground/50" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.5">
              <path stroke-linecap="round" stroke-linejoin="round" d="M7.5 8.25h9m-9 3H12m-9.75 1.51c0 1.6 1.123 2.994 2.707 3.227 1.129.166 2.27.293 3.423.379.35.026.67.21.865.501L12 21l2.755-4.133a1.14 1.14 0 01.865-.501 48.172 48.172 0 003.423-.379c1.584-.233 2.707-1.626 2.707-3.228V6.741c0-1.602-1.123-2.995-2.707-3.228A48.394 48.394 0 0012 3c-2.392 0-4.744.175-7.043.513C3.373 3.746 2.25 5.14 2.25 6.741v6.018z" />
            </svg>
            <span class="text-sm text-muted-foreground/60">No comments available</span>
          </div>
        </div>
      </div>

      <!-- Related Videos Sidebar -->
      <div class="space-y-4">
        <h3 class="text-sm font-semibold text-foreground">Related Videos</h3>
        <div v-if="relatedVideos.length > 0" class="space-y-3">
          <div
            v-for="rel in relatedVideos"
            :key="rel.id"
            class="flex gap-3 cursor-pointer group rounded-lg p-1.5 transition-colors hover:bg-primary/8"
          >
            <router-link :to="`/watch?v=${rel.id}`" class="relative w-40 aspect-video rounded-lg overflow-hidden bg-muted shrink-0">
              <img v-if="rel.thumbnail" :src="rel.thumbnail" :alt="rel.title" class="w-full h-full object-cover" />
            </router-link>
            <div class="flex-1 min-w-0">
              <router-link :to="`/watch?v=${rel.id}`" class="text-sm font-medium text-foreground line-clamp-2">{{ rel.title }}</router-link>
              <router-link :to="`/channel/${rel.authorId}`" class="text-xs text-muted-foreground mt-1 hover:text-foreground block">{{ rel.author }}</router-link>
              <p class="text-xs text-muted-foreground">{{ formatViews(rel.viewCount) }}</p>
            </div>
          </div>
        </div>
        <EmptyState v-else title="No related videos" />
      </div>
    </div>
  </div>
</template>

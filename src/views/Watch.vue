<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { PhHouse, PhCaretRight, PhPlus, PhDownload, PhClock } from '@phosphor-icons/vue'
import { getVideo, getVideoPlaybackInfo } from '../api'
import { useSponsorBlock, useDownloads, useComments } from '../composables/useData'
import { getInvidiousManifestUrl } from '../api/manifest'
import { useSubscriptionsStore } from '../stores/subscriptions'
import { useHistoryStore } from '../stores/history'
import { usePlaylistsStore } from '../stores/playlists'
import type { Video } from '../api/types'
import type { SponsorBlockSegment } from '../api/sponsorblock'
import ErrorState from '../components/ui/ErrorState.vue'
import EmptyState from '../components/ui/EmptyState.vue'
import ShakaPlayer from '../components/player/ShakaPlayer.vue'

const route = useRoute()
const router = useRouter()
const videoId = computed(() => route.query.v as string || route.params.id as string || '')
const video = ref<Video | null>(null)
const loading = ref(true)
const error = ref<string | null>(null)
const playerError = ref<string | null>(null)
const manifestUrl = ref<string>('')
const selectedFormatUrl = ref<string>('')

const subscriptionsStore = useSubscriptionsStore()
const historyStore = useHistoryStore()
const playlistsStore = usePlaylistsStore()
const { startDownload } = useDownloads()
const comments = useComments(videoId.value)
const sponsorBlock = useSponsorBlock(videoId.value)

async function load() {
  if (!videoId.value) {
    error.value = 'No video ID provided'
    loading.value = false
    return
  }
  loading.value = true
  error.value = null
  selectedFormatUrl.value = ''

  try {
    video.value = await getVideo(videoId.value)
    await sponsorBlock.load()
    comments.load()

    // Add to history when video loads successfully
    if (video.value) {
      await historyStore.addToHistory({
        videoId: video.value.id,
        title: video.value.title,
        author: video.value.author,
        authorId: video.value.authorId,
        lengthSeconds: video.value.lengthSeconds,
        timeWatched: new Date().toISOString(),
        watchProgress: 0,
        isWatched: true,
        isLive: video.value.isLive,
      })
    }

    const playbackInfo = await getVideoPlaybackInfo(videoId.value)

    if (playbackInfo.dashUrl) {
      manifestUrl.value = playbackInfo.dashUrl
    } else if (playbackInfo.manifestXml) {
      manifestUrl.value = `data:application/dash+xml;charset=UTF-8,${encodeURIComponent(playbackInfo.manifestXml)}`
    } else if (playbackInfo.formatStreams.length > 0) {
      const bestFormat = playbackInfo.formatStreams
        .filter((f: any) => f.qualityLabel)
        .sort((a: any, b: any) => {
          const aHeight = parseInt(a.qualityLabel) || 0
          const bHeight = parseInt(b.qualityLabel) || 0
          return bHeight - aHeight
        })[0]
      if (bestFormat?.url) {
        selectedFormatUrl.value = bestFormat.url
        manifestUrl.value = ''
      } else {
        manifestUrl.value = getInvidiousManifestUrl(videoId.value)
      }
    } else {
      manifestUrl.value = getInvidiousManifestUrl(videoId.value)
    }
  } catch (e: any) {
    error.value = e.message || 'Failed to load video'
  } finally {
    loading.value = false
  }
}

// Reload when videoId changes (navigation between videos)
watch(videoId, (newId, oldId) => {
  if (newId !== oldId) {
    load()
  }
})

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

const segments = computed(() => sponsorBlock.segments.value as SponsorBlockSegment[])

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
        <div v-if="sponsorBlock.segments.value.length > 0" class="bg-card rounded-xl p-4">
          <h3 class="text-sm font-semibold text-foreground mb-3">SponsorBlock Segments</h3>
          <div class="space-y-2">
            <div
              v-for="seg in sponsorBlock.segments.value"
              :key="seg.UUID"
              class="flex items-center gap-3"
            >
              <div class="size-3 rounded" :style="{ backgroundColor: sponsorBlock.getColor(seg.category) }"></div>
              <span class="text-sm text-foreground">{{ sponsorBlock.formatCategory(seg.category) }}</span>
              <span class="text-xs text-muted-foreground">{{ formatDuration(seg.segment[0]) }} - {{ formatDuration(seg.segment[1]) }}</span>
            </div>
          </div>
        </div>

        <!-- Comments -->
        <div class="bg-card rounded-xl p-4">
          <h3 class="text-sm font-semibold text-foreground mb-4">Comments ({{ comments.comments.value.length }})</h3>
          <div v-if="comments.loading.value" class="flex justify-center py-4">
            <div class="size-5 animate-spin rounded-full border-2 border-primary border-t-transparent"></div>
          </div>
          <div v-else-if="comments.comments.value.length > 0" class="space-y-4">
            <div v-for="comment in comments.comments.value" :key="comment.id" class="flex gap-3">
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
                  <span class="text-xs text-muted-foreground">{{ comment.likeCount }} likes</span>
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
        <div v-if="video!.related.length > 0" class="space-y-3">
          <div
            v-for="rel in video!.related.slice(0, 10)"
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
</template>

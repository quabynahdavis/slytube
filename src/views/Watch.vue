<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useDownloads } from '../composables/useData'
import { useSubscriptionsStore } from '../stores/subscriptions'
import { usePlaylistsStore } from '../stores/playlists'
import type { Video } from '../api/types'
import type { SponsorBlockSegment } from '../api/sponsorblock'
import EmptyState from '../components/ui/EmptyState.vue'
import ShakaPlayer from '../components/player/ShakaPlayer.vue'

const loading = ref(false)
const error = ref<string | null>(null)
const playerError = ref<string | null>(null)

const subscriptionsStore = useSubscriptionsStore()
const playlistsStore = usePlaylistsStore()
const { startDownload } = useDownloads()

// Test video ID - using the provided YouTube link
const testVideoId = '7zuv5KMQFj8'

// Invidious DASH manifest URL for the test video
const manifestUrl = `https://yewtu.be/api/manifest/dash/id/${testVideoId}?local=true`

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

const dummyChapters = [
  { title: 'Introduction', startSeconds: 0, thumbnail: '' },
  { title: 'Project Setup', startSeconds: 150, thumbnail: '' },
  { title: 'API Integration', startSeconds: 495, thumbnail: '' },
  { title: 'Video Player', startSeconds: 940, thumbnail: '' },
  { title: 'State Management', startSeconds: 2120, thumbnail: '' },
  { title: 'Conclusion', startSeconds: 3000, thumbnail: '' },
]

const dummyTranscript = [
  { start: 0, duration: 4, text: 'Welcome to this comprehensive tutorial on building a modern YouTube client.' },
  { start: 4, duration: 5, text: 'Today we will be using Vue 3, TypeScript, and Tauri to create a desktop application.' },
  { start: 9, duration: 4, text: 'Let us start by setting up our development environment.' },
  { start: 13, duration: 6, text: 'First, make sure you have Node.js version 18 or higher installed on your system.' },
  { start: 19, duration: 5, text: 'We will also need Rust and Cargo installed for the Tauri backend.' },
  { start: 24, duration: 4, text: 'Once everything is set up, we can create our project.' },
]

const video = ref<Video>({
  id: testVideoId,
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
  chapters: dummyChapters,
  captions: [],
  related: dummyRelated,
})

// Comments - not loaded automatically
const comments = ref<typeof dummyComments>([])
const commentsLoaded = ref(false)
const sponsorBlockSegments = dummySegments
const chapters = dummyChapters
const transcript = dummyTranscript

// Description expand/collapse
const showFullDescription = ref(false)
const descriptionExpanded = computed(() => showFullDescription.value || video.value.description.length < 200)

function toggleDescription() {
  showFullDescription.value = !showFullDescription.value
}

const truncatedDescription = computed(() => {
  if (descriptionExpanded.value) return video.value.description
  const lines = video.value.description.split('\n')
  return lines.slice(0, 3).join('\n')
})

// Transcript/Chapters/SponsorBlock panel visibility
const showInfoPanel = ref(false)
const activeTab = ref<'chapters' | 'transcript' | 'sponsorblock'>('chapters')

function toggleInfoPanel() {
  showInfoPanel.value = !showInfoPanel.value
}

function loadComments() {
  comments.value = dummyComments
  commentsLoaded.value = true
}

function formatDuration(seconds: number): string {
  const h = Math.floor(seconds / 3600)
  const m = Math.floor((seconds % 3600) / 60)
  const s = seconds % 60
  if (h > 0) return `${h}:${m.toString().padStart(2, '0')}:${s.toString().padStart(2, '0')}`
  return `${m}:${s.toString().padStart(2, '0')}`
}

function formatViews(count: number): string {
  if (count >= 1_000_000) return `${(count / 1_000_000).toFixed(1)}M views`
  if (count >= 1_000) return `${(count / 1_000).toFixed(1)}K views`
  return `${count} views`
}

const segments = computed(() => sponsorBlockSegments as SponsorBlockSegment[])

const isSubscribed = computed(() =>
  subscriptionsStore.isSubscribed(video.value.authorId)
)
const isSubscribePending = computed(() =>
  subscriptionsStore.isPending(video.value.authorId)
)

async function handleSubscribe() {
  const channelId = video.value.authorId
  const channelName = video.value.author
  const result = await subscriptionsStore.toggleSubscription(channelId, channelName)
  if (!result.success) {
    console.warn('Failed to update subscription')
  }
}

async function addToQuickBookmark() {
  try {
    let playlist = playlistsStore.playlists.find((p) => p.playlistName === 'Favorites')
    if (!playlist) {
      playlist = await playlistsStore.createPlaylist('Favorites', 'Quick bookmark playlist')
    }
    if (playlist) {
      await playlistsStore.addToPlaylist(playlist._id, video.value.id)
    }
  } catch {
    console.error('Failed to add to quick bookmark')
  }
}

async function downloadVideo() {
  try {
    await startDownload({ videoId: video.value.id, mode: 'video' })
  } catch (error) {
    console.error('Download failed:', error)
  }
}

async function saveToWatchLater() {
  try {
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
  loading.value = false
  error.value = null
})
</script>

<template>
  <div class="p-4">
    <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
      <!-- Main Content -->
      <div class="lg:col-span-2 space-y-4">
        <!-- Shaka Player -->
        <ShakaPlayer
          :manifest-url="manifestUrl"
          :video-id="testVideoId"
          :title="video.title"
          :segments="segments"
          :chapters="video.chapters"
          @error="playerError = $event"
        />

        <!-- Player Error Fallback -->
        <div v-if="playerError" class="bg-destructive/10 border border-destructive/20 rounded-xl p-4">
          <p class="text-sm text-destructive font-medium">Player Error: {{ playerError }}</p>
          <p class="text-xs text-muted-foreground mt-1">The video may be unavailable or require authentication.</p>
        </div>

        <!-- Video Info -->
        <div>
          <h1 class="text-xl font-bold text-foreground mb-2">{{ video.title }}</h1>
          <div class="flex items-center justify-between flex-wrap gap-4">
            <div class="flex items-center gap-3">
              <router-link :to="`/channel/${video.authorId}`" class="size-10 rounded-full bg-primary/20 flex items-center justify-center hover:ring-2 hover:ring-primary/30 transition-all">
                <span class="text-sm font-medium text-primary">{{ video.author[0] }}</span>
              </router-link>
              <div>
                <router-link :to="`/channel/${video.authorId}`" class="text-sm font-medium text-foreground hover:underline">{{ video.author }}</router-link>
                <p class="text-xs text-muted-foreground">{{ formatViews(video.viewCount) }} &middot; {{ video.published }}</p>
              </div>
              <button
                class="px-4 py-1.5 rounded-full text-sm font-medium transition-all duration-200"
                :class="isSubscribed
                  ? 'bg-secondary text-secondary-foreground hover:bg-secondary/80'
                  : 'bg-primary text-primary-foreground hover:bg-primary/90'"
                :disabled="isSubscribePending"
                @click="handleSubscribe"
              >
                {{ isSubscribePending ? '...' : isSubscribed ? 'Subscribed' : 'Subscribe' }}
              </button>
            </div>
            <div class="flex gap-2">
              <button
                class="px-4 py-1.5 rounded-full text-sm flex items-center gap-2 bg-secondary text-secondary-foreground hover:bg-secondary/80 transition-colors"
                @click="addToQuickBookmark"
              >
                <svg class="size-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M19 21l-7-5-7 5V5a2 2 0 012-2h10a2 2 0 012 2z" /></svg>
                Save
              </button>
              <button
                class="px-4 py-1.5 rounded-full text-sm flex items-center gap-2 bg-secondary text-secondary-foreground hover:bg-secondary/80 transition-colors"
                @click="downloadVideo"
              >
                <svg class="size-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4" /><polyline points="7 10 12 15 17 10" /><line x1="12" y1="15" x2="12" y2="3" /></svg>
                Download
              </button>
              <button
                class="px-4 py-1.5 rounded-full text-sm flex items-center gap-2 bg-secondary text-secondary-foreground hover:bg-secondary/80 transition-colors"
                @click="saveToWatchLater"
              >
                <svg class="size-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10" /><polyline points="12 6 12 12 16 14" /></svg>
                Watch Later
              </button>
              <button
                class="px-4 py-1.5 rounded-full text-sm flex items-center gap-2 transition-colors"
                :class="showInfoPanel ? 'bg-primary text-primary-foreground' : 'bg-secondary text-secondary-foreground hover:bg-secondary/80'"
                @click="toggleInfoPanel"
              >
                <svg class="size-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="8" y1="6" x2="21" y2="6" /><line x1="8" y1="12" x2="21" y2="12" /><line x1="8" y1="18" x2="21" y2="18" /><line x1="3" y1="6" x2="3.01" y2="6" /><line x1="3" y1="12" x2="3.01" y2="12" /><line x1="3" y1="18" x2="3.01" y2="18" /></svg>
                More
              </button>
            </div>
          </div>
        </div>

        <!-- Description with Show More -->
        <div class="bg-card rounded-xl p-4">
          <p class="text-sm text-muted-foreground whitespace-pre-wrap">{{ truncatedDescription }}</p>
          <button
            v-if="video.description.length >= 200"
            class="text-sm font-medium text-foreground mt-2 hover:underline"
            @click="toggleDescription"
          >
            {{ descriptionExpanded ? 'Show less' : 'Show more' }}
          </button>
        </div>

        <!-- Comments -->
        <div class="bg-card rounded-xl p-4">
          <div class="flex items-center justify-between mb-4">
            <h3 class="text-sm font-semibold text-foreground">Comments</h3>
            <button
              v-if="!commentsLoaded"
              class="text-sm text-primary hover:underline"
              @click="loadComments"
            >
              Load Comments
            </button>
          </div>
          <div v-if="commentsLoaded">
            <div v-if="comments.length > 0" class="space-y-4">
              <div v-for="comment in comments" :key="comment.id" class="flex gap-3">
                <div class="size-8 rounded-full bg-primary/20 shrink-0 flex items-center justify-center">
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
            <p v-else class="text-sm text-muted-foreground text-center py-4">No comments available</p>
          </div>
          <p v-else class="text-sm text-muted-foreground text-center py-4">Click "Load Comments" to view comments</p>
        </div>
      </div>

      <!-- Sidebar: Related Videos + Info Panel -->
      <div class="space-y-4">
        <!-- Transcript/Chapters/SponsorBlock Panel -->
        <div v-if="showInfoPanel" class="bg-card rounded-xl overflow-hidden">
          <div class="flex border-b border-border">
            <button
              class="flex-1 px-4 py-3 text-sm font-medium transition-colors"
              :class="activeTab === 'chapters' ? 'text-foreground border-b-2 border-primary' : 'text-muted-foreground hover:text-foreground'"
              @click="activeTab = 'chapters'"
            >
              Chapters
            </button>
            <button
              class="flex-1 px-4 py-3 text-sm font-medium transition-colors"
              :class="activeTab === 'transcript' ? 'text-foreground border-b-2 border-primary' : 'text-muted-foreground hover:text-foreground'"
              @click="activeTab = 'transcript'"
            >
              Transcript
            </button>
            <button
              class="flex-1 px-4 py-3 text-sm font-medium transition-colors"
              :class="activeTab === 'sponsorblock' ? 'text-foreground border-b-2 border-primary' : 'text-muted-foreground hover:text-foreground'"
              @click="activeTab = 'sponsorblock'"
            >
              SponsorBlock
            </button>
          </div>

          <div class="p-4 max-h-96 overflow-y-auto">
            <!-- Chapters Tab -->
            <div v-if="activeTab === 'chapters'">
              <div v-if="chapters.length > 0" class="space-y-2">
                <div
                  v-for="(chapter, idx) in chapters"
                  :key="idx"
                  class="flex items-center gap-3 p-2 rounded-lg hover:bg-accent/50 cursor-pointer transition-colors"
                >
                  <div class="size-12 rounded bg-muted flex items-center justify-center shrink-0">
                    <svg class="size-4 text-muted-foreground" viewBox="0 0 24 24" fill="currentColor"><polygon points="5 3 19 12 5 21 5 3" /></svg>
                  </div>
                  <div class="flex-1 min-w-0">
                    <p class="text-sm font-medium text-foreground">{{ chapter.title }}</p>
                    <p class="text-xs text-muted-foreground">{{ formatDuration(chapter.startSeconds) }}</p>
                  </div>
                </div>
              </div>
              <p v-else class="text-sm text-muted-foreground text-center py-4">No chapters available</p>
            </div>

            <!-- Transcript Tab -->
            <div v-else-if="activeTab === 'transcript'">
              <div v-if="transcript.length > 0" class="space-y-2">
                <div
                  v-for="(line, idx) in transcript"
                  :key="idx"
                  class="flex gap-3 p-2 rounded-lg hover:bg-accent/50 cursor-pointer transition-colors"
                >
                  <span class="text-xs text-primary font-mono shrink-0 w-12 text-right pt-0.5">{{ formatDuration(line.start) }}</span>
                  <p class="text-sm text-foreground">{{ line.text }}</p>
                </div>
              </div>
              <p v-else class="text-sm text-muted-foreground text-center py-4">No transcript available</p>
            </div>

            <!-- SponsorBlock Tab -->
            <div v-else-if="activeTab === 'sponsorblock'">
              <div v-if="sponsorBlockSegments.length > 0" class="space-y-2">
                <div
                  v-for="seg in sponsorBlockSegments"
                  :key="seg.UUID"
                  class="flex items-center gap-3 p-2 rounded-lg hover:bg-accent/50"
                >
                  <div class="size-3 rounded" :style="{ backgroundColor: seg.category === 'sponsor' ? '#00d400' : seg.category === 'intro' ? '#00ffff' : '#0202ed' }" />
                  <span class="text-sm text-foreground flex-1">{{ seg.category.charAt(0).toUpperCase() + seg.category.slice(1) }}</span>
                  <span class="text-xs text-muted-foreground">{{ formatDuration(seg.segment[0]) }} - {{ formatDuration(seg.segment[1]) }}</span>
                </div>
              </div>
              <p v-else class="text-sm text-muted-foreground text-center py-4">No SponsorBlock segments</p>
            </div>
          </div>
        </div>

        <!-- Related Videos -->
        <div>
          <h3 class="text-sm font-semibold text-foreground mb-3">Related Videos</h3>
          <div v-if="video.related.length > 0" class="space-y-3">
            <div
              v-for="rel in video.related"
              :key="rel.id"
              class="flex gap-3 cursor-pointer group rounded-lg p-2 transition-colors hover:bg-primary/8"
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
  </div>
</template>

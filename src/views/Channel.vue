<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRoute } from 'vue-router'
import { cn } from '@/lib/utils'

const route = useRoute()

const channelId = computed(() => route.params.id as string || '')
const isLoading = ref(true)
const activeTab = ref('home')
const isSubscribed = ref(false)

const channel = ref({
  author: '',
  authorId: '',
  description: '',
  subscriberCount: 0,
  videoCount: 0,
  joinedDate: '',
  totalViews: 0,
  authorBanners: [] as Array<{ url: string; width: number; height: number }>,
  authorThumbnails: [] as Array<{ url: string; width: number; height: number }>,
  tabs: [] as string[],
  relatedChannels: [] as Array<{ authorId: string; author: string }>,
})

const videos = ref<Array<{
  videoId: string
  title: string
  viewCount: number
  lengthSeconds: number
  published: string
  videoThumbnails: Array<{ url: string; width: number; height: number }>
}>>([])

const tabs = [
  { id: 'home', label: 'Home' },
  { id: 'videos', label: 'Videos' },
  { id: 'shorts', label: 'Shorts' },
  { id: 'live', label: 'Live' },
  { id: 'playlists', label: 'Playlists' },
  { id: 'community', label: 'Community' },
]

onMounted(async () => {
  isLoading.value = true
  try {
    await new Promise((resolve) => setTimeout(resolve, 700))
    channel.value = {
      author: 'Sample Channel',
      authorId: channelId.value,
      description: 'This is a sample channel description. The channel creates content about various topics including technology, entertainment, and education.',
      subscriberCount: 1250000,
      videoCount: 342,
      joinedDate: '2018-03-15',
      totalViews: 125000000,
      authorBanners: [{ url: '', width: 1920, height: 320 }],
      authorThumbnails: [{ url: '', width: 200, height: 200 }],
      tabs: ['home', 'videos', 'shorts', 'live', 'playlists', 'community'],
      relatedChannels: [],
    }
    videos.value = Array.from({ length: 16 }, (_, i) => ({
      videoId: `channel-video-${i}`,
      title: `Channel Video ${i + 1} - Sample Title`,
      viewCount: Math.floor(Math.random() * 2000000),
      lengthSeconds: Math.floor(Math.random() * 600) + 60,
      published: new Date(Date.now() - Math.random() * 180 * 24 * 60 * 60 * 1000).toISOString().split('T')[0],
      videoThumbnails: [{ url: '', width: 640, height: 360 }],
    }))
  } finally {
    isLoading.value = false
  }
})

function formatSubscribers(count: number): string {
  if (count >= 1000000) return `${(count / 1000000).toFixed(1)}M subscribers`
  if (count >= 1000) return `${(count / 1000).toFixed(1)}K subscribers`
  return `${count} subscribers`
}

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

function timeAgo(dateStr: string): string {
  const diff = Date.now() - new Date(dateStr).getTime()
  const days = Math.floor(diff / (1000 * 60 * 60 * 24))
  if (days > 30) return `${Math.floor(days / 30)} months ago`
  if (days > 0) return `${days} days ago`
  return 'Today'
}

function toggleSubscription() {
  isSubscribed.value = !isSubscribed.value
}
</script>

<template>
  <div class="min-h-screen">
    <!-- Loading State -->
    <div v-if="isLoading" class="animate-pulse">
      <div class="h-48 bg-muted" />
      <div class="container mx-auto max-w-7xl px-4 py-4">
        <div class="flex items-center gap-4">
          <div class="size-20 rounded-full bg-muted" />
          <div class="space-y-2">
            <div class="h-5 w-48 rounded bg-muted" />
            <div class="h-4 w-32 rounded bg-muted" />
          </div>
        </div>
      </div>
    </div>

    <template v-else>
      <!-- Channel Banner -->
      <div class="relative h-48 bg-gradient-to-r from-primary/20 to-primary/5">
        <div class="absolute inset-0 flex items-center justify-center">
          <svg class="size-16 text-muted-foreground/30" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1">
            <rect x="2" y="2" width="20" height="20" rx="2.18" ry="2.18" />
            <line x1="7" y1="2" x2="7" y2="22" />
            <line x1="17" y1="2" x2="17" y2="22" />
            <line x1="2" y1="12" x2="22" y2="12" />
          </svg>
        </div>
      </div>

      <!-- Channel Info -->
      <div class="container mx-auto max-w-7xl px-4 py-4">
        <div class="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
          <div class="flex items-center gap-4">
            <div class="size-20 -mt-10 rounded-full border-4 border-background bg-muted flex items-center justify-center shrink-0">
              <svg class="size-10 text-muted-foreground" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2" />
                <circle cx="12" cy="7" r="4" />
              </svg>
            </div>
            <div>
              <h1 class="text-xl font-bold text-foreground">{{ channel.author }}</h1>
              <p class="text-sm text-muted-foreground">
                {{ formatSubscribers(channel.subscriberCount) }} &middot; {{ channel.videoCount }} videos
              </p>
              <p class="text-xs text-muted-foreground mt-1 line-clamp-1">{{ channel.description }}</p>
            </div>
          </div>
          <button
            :class="cn(
              'h-9 rounded-full px-6 text-sm font-medium transition-colors shrink-0',
              isSubscribed
                ? 'border border-border bg-muted text-foreground hover:bg-muted/80'
                : 'bg-primary text-primary-foreground hover:bg-primary/90'
            )"
            @click="toggleSubscription"
          >
            {{ isSubscribed ? 'Subscribed' : 'Subscribe' }}
          </button>
        </div>

        <!-- Channel Tabs -->
        <div class="mt-6 border-b border-border">
          <nav class="flex gap-6 overflow-x-auto">
            <button
              v-for="tab in tabs"
              :key="tab.id"
              :class="cn(
                'pb-3 text-sm font-medium border-b-2 transition-colors whitespace-nowrap',
                activeTab === tab.id
                  ? 'border-primary text-foreground'
                  : 'border-transparent text-muted-foreground hover:text-foreground'
              )"
              @click="activeTab = tab.id"
            >
              {{ tab.label }}
            </button>
          </nav>
        </div>

        <!-- Tab Content -->
        <div class="mt-6">
          <!-- Home / Videos Tab -->
          <div v-if="activeTab === 'home' || activeTab === 'videos'">
            <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
              <router-link
                v-for="video in videos"
                :key="video.videoId"
                :to="`/watch?v=${video.videoId}`"
                class="group block"
              >
                <div class="relative aspect-video rounded-lg bg-muted overflow-hidden">
                  <div class="absolute inset-0 flex items-center justify-center">
                    <svg class="size-12 text-muted-foreground/50 group-hover:text-primary transition-colors" viewBox="0 0 24 24" fill="currentColor">
                      <polygon points="5 3 19 12 5 21 5 3" />
                    </svg>
                  </div>
                  <span class="absolute bottom-2 right-2 rounded bg-black/80 px-1.5 py-0.5 text-xs text-white font-medium">
                    {{ formatDuration(video.lengthSeconds) }}
                  </span>
                </div>
                <div class="mt-2">
                  <h3 class="text-sm font-medium text-foreground line-clamp-2 group-hover:text-primary transition-colors">
                    {{ video.title }}
                  </h3>
                  <p class="mt-1 text-xs text-muted-foreground">{{ formatViews(video.viewCount) }} &middot; {{ timeAgo(video.published) }}</p>
                </div>
              </router-link>
            </div>
          </div>

          <!-- Shorts Tab -->
          <div v-else-if="activeTab === 'shorts'">
            <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-3">
              <router-link
                v-for="n in 12"
                :key="n"
                :to="`/watch?v=short-${n}`"
                class="group block"
              >
                <div class="relative aspect-[9/16] rounded-lg bg-muted overflow-hidden">
                  <div class="absolute inset-0 flex items-center justify-center">
                    <svg class="size-10 text-muted-foreground/50 group-hover:text-primary transition-colors" viewBox="0 0 24 24" fill="currentColor">
                      <polygon points="5 3 19 12 5 21 5 3" />
                    </svg>
                  </div>
                </div>
                <p class="mt-2 text-sm font-medium text-foreground line-clamp-2 group-hover:text-primary transition-colors">
                  Short Video {{ n }}
                </p>
                <p class="text-xs text-muted-foreground">{{ formatViews(Math.floor(Math.random() * 1000000)) }}</p>
              </router-link>
            </div>
          </div>

          <!-- Live Tab -->
          <div v-else-if="activeTab === 'live'">
            <div class="rounded-lg border border-border bg-card p-8 text-center text-muted-foreground">
              <svg class="size-12 mx-auto mb-3 text-muted-foreground/50" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
                <path d="M12 1a3 3 0 0 0-3 3v8a3 3 0 0 0 6 0V4a3 3 0 0 0-3-3z" />
                <path d="M19 10v2a7 7 0 0 1-14 0v-2" />
                <line x1="12" y1="19" x2="12" y2="23" />
                <line x1="8" y1="23" x2="16" y2="23" />
              </svg>
              <p class="text-sm">No live streams available</p>
            </div>
          </div>

          <!-- Playlists Tab -->
          <div v-else-if="activeTab === 'playlists'">
            <div class="rounded-lg border border-border bg-card p-8 text-center text-muted-foreground">
              <svg class="size-12 mx-auto mb-3 text-muted-foreground/50" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
                <rect x="2" y="2" width="20" height="20" rx="2.18" ry="2.18" />
                <line x1="7" y1="2" x2="7" y2="22" />
                <line x1="17" y1="2" x2="17" y2="22" />
                <line x1="2" y1="12" x2="22" y2="12" />
              </svg>
              <p class="text-sm">No playlists available</p>
            </div>
          </div>

          <!-- Community Tab -->
          <div v-else-if="activeTab === 'community'">
            <div class="rounded-lg border border-border bg-card p-8 text-center text-muted-foreground">
              <svg class="size-12 mx-auto mb-3 text-muted-foreground/50" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
                <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z" />
              </svg>
              <p class="text-sm">No community posts yet</p>
            </div>
          </div>
        </div>
      </div>
    </template>
  </div>
</template>

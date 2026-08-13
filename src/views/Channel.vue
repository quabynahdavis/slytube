<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue'
import { useRoute } from 'vue-router'
import { cn } from '@/lib/utils'
import { PhPlayCircle, PhList, PhHeart, PhShare, PhSparkle, PhMagnifyingGlass } from '@phosphor-icons/vue'
import { useThumbnailColor } from '@/composables/useThumbnailColor'
import { useSubscriptionsStore } from '@/stores/subscriptions'
import { getChannelInfo, getChannelShorts, getChannelCommunityPosts } from '@/api'
import type { Channel as ChannelType, Video } from '@/api/types'
import EmptyState from '@/components/ui/EmptyState.vue'

const route = useRoute()
const { getColor } = useThumbnailColor()
const subscriptionsStore = useSubscriptionsStore()

const channelId = computed(() => route.params.id as string || '')
const activeTab = ref('home')
const searchQuery = ref('')

const tabs = [
  { id: 'home', label: 'Home' },
  { id: 'videos', label: 'Videos' },
  { id: 'shorts', label: 'Shorts' },
  { id: 'playlists', label: 'Playlists' },
  { id: 'community', label: 'Community' },
]

const isLoading = ref(true)
const tabLoading = ref(false)
const error = ref<string | null>(null)

const channel = ref<ChannelType | null>(null)
const channelVideos = ref<Video[]>([])
const shorts = ref<Video[]>([])
const playlists = ref<any[]>([])
const communityPosts = ref<any[]>([])

const isSubscribed = computed(() =>
  channel.value ? subscriptionsStore.isSubscribed(channel.value.id) : false
)

const filteredVideos = computed(() => {
  if (!searchQuery.value.trim()) return channelVideos.value
  const q = searchQuery.value.toLowerCase()
  return channelVideos.value.filter(v =>
    v.title.toLowerCase().includes(q) ||
    v.author.toLowerCase().includes(q)
  )
})

const videoCount = computed(() => channelVideos.value.filter(v => !v.isShort).length)
const shortsCount = computed(() => channelVideos.value.filter(v => v.isShort).length)

async function loadChannelData() {
  if (!channelId.value) return
  isLoading.value = true
  error.value = null
  try {
    await subscriptionsStore.loadSubscriptions()
    const data = await getChannelInfo(channelId.value)
    channel.value = data
    channelVideos.value = data.videos || []
  } catch (e: any) {
    error.value = e.message || 'Failed to load channel'
  } finally {
    isLoading.value = false
  }
}

async function loadTabData(tab: string) {
  if (!channelId.value) return
  if (tab === 'shorts' && shorts.value.length === 0) {
    tabLoading.value = true
    try {
      shorts.value = await getChannelShorts(channelId.value)
    } catch {
      shorts.value = []
    } finally {
      tabLoading.value = false
    }
  } else if (tab === 'community' && communityPosts.value.length === 0) {
    tabLoading.value = true
    try {
      communityPosts.value = await getChannelCommunityPosts(channelId.value)
    } catch {
      communityPosts.value = []
    } finally {
      tabLoading.value = false
    }
  }
}

async function toggleSubscription() {
  if (!channel.value) return
  if (isSubscribed.value) {
    await subscriptionsStore.unsubscribeFromChannel(channel.value.id)
  } else {
    await subscriptionsStore.subscribeToChannel(channel.value.id)
  }
}

watch(channelId, loadChannelData)
watch(activeTab, (tab) => loadTabData(tab))

onMounted(loadChannelData)

function formatSubscribers(count: number): string {
  if (count >= 1000000) return `${(count / 1000000).toFixed(1)}M`
  if (count >= 1000) return `${(count / 1000).toFixed(1)}K`
  return `${count}`
}

function formatViews(count: number): string {
  if (count >= 1000000) return `${(count / 1000000).toFixed(1)}M views`
  if (count >= 1000) return `${(count / 1000).toFixed(1)}K views`
  return `${count} views`
}

function formatDuration(seconds: number): string {
  const m = Math.floor(seconds / 60)
  const s = seconds % 60
  return `${m}:${s.toString().padStart(2, '0')}`
}

function timeAgo(published: string): string {
  if (!published) return ''
  if (published.includes('ago') || published.includes('yesterday')) {
    return published
  }
  const now = Date.now()
  const then = new Date(published).getTime()
  if (isNaN(then)) return published
  const diff = now - then
  const mins = Math.floor(diff / 60000)
  if (mins < 60) return `${mins}m ago`
  const hours = Math.floor(mins / 60)
  if (hours < 24) return `${hours}h ago`
  const days = Math.floor(hours / 24)
  if (days < 30) return `${days}d ago`
  const months = Math.floor(days / 30)
  if (months < 12) return `${months}mo ago`
  return `${Math.floor(months / 12)}y ago`
}
</script>

<template>
  <div class="min-h-screen">
    <!-- Loading -->
    <div v-if="isLoading" class="animate-pulse">
      <div class="h-48 bg-muted" />
      <div class="max-w-7xl mx-auto px-4 py-4">
        <div class="flex items-center gap-4">
          <div class="size-20 -mt-10 rounded-full bg-muted" />
          <div class="space-y-2">
            <div class="h-5 w-48 rounded bg-muted" />
            <div class="h-4 w-32 rounded bg-muted" />
          </div>
        </div>
      </div>
    </div>

    <!-- Error -->
    <div v-else-if="error" class="flex items-center justify-center h-64">
      <div class="text-center max-w-md">
        <div class="size-16 mx-auto mb-4 rounded-full bg-destructive/10 flex items-center justify-center">
          <svg class="size-8 text-destructive" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="12" cy="12" r="10"/>
            <line x1="12" y1="8" x2="12" y2="12"/>
            <line x1="12" y1="16" x2="12.01" y2="16"/>
          </svg>
        </div>
        <p class="text-destructive font-medium">Failed to load channel</p>
        <p class="text-sm text-muted-foreground mt-1">{{ error }}</p>
        <button class="mt-4 px-4 py-2 bg-primary text-primary-foreground rounded-lg text-sm font-medium hover:bg-primary/90 transition-colors" @click="loadChannelData">
          Retry
        </button>
      </div>
    </div>

    <template v-else-if="channel">
      <!-- Banner -->
      <div class="relative h-40 sm:h-48 md:h-56 bg-gradient-to-r from-blue-600/30 to-purple-600/30">
        <img v-if="channel.banner" :src="channel.banner" :alt="channel.name" class="absolute inset-0 w-full h-full object-cover" />
        <div v-else class="absolute inset-0 flex items-center justify-center">
          <span class="text-6xl opacity-20">📺</span>
        </div>
      </div>

      <!-- Channel Info Bar -->
      <div class="border-b border-border">
        <div class="max-w-7xl mx-auto px-4 py-4">
          <div class="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
            <div class="flex items-center gap-4">
              <div class="size-20 -mt-10 rounded-full border-4 border-background bg-gradient-to-br from-primary to-purple-500 flex items-center justify-center text-2xl text-white font-bold shrink-0 overflow-hidden">
                <img v-if="channel.avatar" :src="channel.avatar" :alt="channel.name" class="w-full h-full object-cover" />
                <span v-else>{{ channel.name[0] }}</span>
              </div>
              <div>
                <h1 class="text-xl font-bold text-foreground">
                  {{ channel.name === 'Unknown' ? 'Channel unavailable' : channel.name }}
                </h1>
                <p v-if="channel.name !== 'Unknown'" class="text-sm text-muted-foreground">
                  {{ formatSubscribers(channel.subscriberCount) }} subscribers • {{ channel.videoCount.toLocaleString() }} videos
                </p>
                <p v-else class="text-sm text-muted-foreground">
                  Channel data could not be loaded
                </p>
                <p v-if="channel.name !== 'Unknown'" class="text-xs text-muted-foreground mt-1 line-clamp-1 max-w-md">{{ channel.description }}</p>
              </div>
            </div>
            <div class="flex items-center gap-2">
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
              <button class="size-9 rounded-full bg-muted flex items-center justify-center text-muted-foreground hover:bg-muted/80 transition-colors">
                <PhHeart :size="18" />
              </button>
              <button class="size-9 rounded-full bg-muted flex items-center justify-center text-muted-foreground hover:bg-muted/80 transition-colors">
                <PhShare :size="18" />
              </button>
            </div>
          </div>
        </div>
      </div>

      <!-- Tabs & Search -->
      <div class="sticky top-0 z-30 bg-background border-b border-border">
        <div class="max-w-7xl mx-auto px-4">
          <div class="flex items-center justify-between gap-4">
            <nav class="flex gap-6 overflow-x-auto">
              <button
                v-for="tab in tabs"
                :key="tab.id"
                :class="cn(
                  'py-3 text-sm font-medium border-b-2 transition-colors whitespace-nowrap',
                  activeTab === tab.id
                    ? 'border-primary text-foreground'
                    : 'border-transparent text-muted-foreground hover:text-foreground'
                )"
                @click="activeTab = tab.id"
              >
                {{ tab.label }}
                <span v-if="tab.id === 'videos'" class="text-xs text-muted-foreground ml-1">({{ videoCount }})</span>
                <span v-if="tab.id === 'shorts'" class="text-xs text-muted-foreground ml-1">({{ shortsCount }})</span>
              </button>
            </nav>
            <div v-if="activeTab === 'home' || activeTab === 'videos'" class="relative w-48 shrink-0">
              <PhMagnifyingGlass :size="14" class="absolute left-2 top-1/2 -translate-y-1/2 text-muted-foreground" />
              <input
                v-model="searchQuery"
                type="text"
                placeholder="Search"
                class="w-full h-8 pl-7 pr-3 rounded-md border border-input bg-background text-xs outline-none focus:border-primary"
              />
            </div>
          </div>
        </div>
      </div>

      <!-- Tab Content -->
      <div class="max-w-7xl mx-auto px-4 py-6">

        <!-- ==================== HOME TAB ==================== -->
        <div v-if="activeTab === 'home'" class="space-y-8">

          <!-- Featured Video (first video) -->
          <section v-if="filteredVideos.length > 0">
            <div class="flex items-center gap-2 mb-4">
              <PhSparkle :size="18" class="text-primary" />
              <h2 class="text-base font-semibold text-foreground">Featured</h2>
            </div>
            <router-link
              :to="`/watch?v=${filteredVideos[0].id}`"
              class="flex flex-col md:flex-row gap-4 group p-3 rounded-xl hover:bg-muted/50 transition-colors"
            >
              <div class="relative w-full md:w-80 shrink-0 aspect-video rounded-xl overflow-hidden bg-muted">
                <img v-if="filteredVideos[0].thumbnail" :src="filteredVideos[0].thumbnail" :alt="filteredVideos[0].title" class="w-full h-full object-cover" />
                <div v-else class="absolute inset-0 flex items-center justify-center text-4xl opacity-30">📌</div>
                <div v-if="filteredVideos[0].lengthSeconds > 0" class="absolute bottom-1 right-1 bg-black/80 text-white text-[10px] px-1 py-0.5 rounded">
                  {{ formatDuration(filteredVideos[0].lengthSeconds) }}
                </div>
              </div>
              <div class="flex-1 min-w-0">
                <h3 class="text-base font-medium text-foreground group-hover:text-primary">{{ filteredVideos[0].title }}</h3>
                <p class="text-sm text-muted-foreground mt-1">{{ formatViews(filteredVideos[0].viewCount) }} • {{ timeAgo(filteredVideos[0].published) }}</p>
                <p class="text-xs text-muted-foreground mt-2 line-clamp-2">{{ filteredVideos[0].description }}</p>
              </div>
            </router-link>
          </section>

          <!-- Recent Videos -->
          <section v-if="filteredVideos.length > 1">
            <div class="flex items-center justify-between mb-4">
              <h2 class="text-base font-semibold text-foreground">Videos</h2>
              <button class="text-sm text-primary hover:underline" @click="activeTab = 'videos'">See all</button>
            </div>
            <div class="space-y-3">
              <router-link
                v-for="video in filteredVideos.slice(1, 5)"
                :key="video.id"
                :to="`/watch?v=${video.id}`"
                class="flex gap-4 p-3 rounded-xl hover:bg-muted/50 transition-colors group"
              >
                <div class="relative w-40 shrink-0 aspect-video rounded-xl overflow-hidden bg-muted">
                  <img v-if="video.thumbnail" :src="video.thumbnail" :alt="video.title" class="w-full h-full object-cover" />
                  <div v-else class="absolute inset-0 flex items-center justify-center text-3xl opacity-30">🎬</div>
                  <div v-if="video.lengthSeconds > 0" class="absolute bottom-1 right-1 bg-black/80 text-white text-[10px] px-1 py-0.5 rounded">
                    {{ formatDuration(video.lengthSeconds) }}
                  </div>
                </div>
                <div class="flex-1 min-w-0">
                  <h3 class="text-sm font-medium text-foreground line-clamp-2 group-hover:text-primary">{{ video.title }}</h3>
                  <p class="text-xs text-muted-foreground mt-1">{{ formatViews(video.viewCount) }} • {{ timeAgo(video.published) }}</p>
                </div>
              </router-link>
            </div>
          </section>

          <!-- Shorts -->
          <section v-if="shorts.length > 0 || tabLoading">
            <div class="flex items-center gap-2 mb-4">
              <PhPlayCircle :size="18" class="text-primary" />
              <h2 class="text-base font-semibold text-foreground">Shorts</h2>
            </div>
            <div v-if="tabLoading" class="flex gap-3">
              <div v-for="n in 4" :key="n" class="w-36 aspect-[9/16] bg-muted rounded-xl animate-pulse" />
            </div>
            <div v-else class="flex gap-3 overflow-x-auto pb-4">
              <router-link
                v-for="short in shorts.slice(0, 6)"
                :key="short.id"
                :to="`/watch?v=${short.id}`"
                class="shrink-0 w-36 group"
              >
                <div class="aspect-[9/16] rounded-xl overflow-hidden bg-muted mb-2 relative">
                  <img v-if="short.thumbnail" :src="short.thumbnail" :alt="short.title" class="w-full h-full object-cover" />
                  <div v-else class="absolute inset-0 flex items-center justify-center text-2xl opacity-30">📱</div>
                  <div v-if="short.lengthSeconds > 0" class="absolute bottom-1 right-1 bg-black/75 text-white text-[10px] px-1 rounded">
                    {{ short.lengthSeconds }}s
                  </div>
                </div>
                <p class="text-xs font-medium text-foreground line-clamp-2 group-hover:text-primary">{{ short.title }}</p>
              </router-link>
            </div>
          </section>

          <!-- Empty State -->
          <EmptyState v-if="filteredVideos.length === 0 && !isLoading" title="No videos found">
            This channel hasn't uploaded any videos yet.
          </EmptyState>
        </div>

        <!-- ==================== VIDEOS TAB ==================== -->
        <div v-else-if="activeTab === 'videos'">
          <div v-if="filteredVideos.length > 0" class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-1">
            <router-link
              v-for="video in filteredVideos"
              :key="video.id"
              :to="`/watch?v=${video.id}`"
              class="group"
            >
              <div class="aspect-video rounded-xl overflow-hidden bg-muted mb-2 relative">
                <img v-if="video.thumbnail" :src="video.thumbnail" :alt="video.title" class="w-full h-full object-cover group-hover:scale-105 transition-transform" />
                <div v-else class="absolute inset-0 flex items-center justify-center text-4xl opacity-30">🎬</div>
                <div v-if="video.lengthSeconds > 0" class="absolute bottom-1 right-1 bg-black/80 text-white text-[10px] px-1 py-0.5 rounded font-medium">
                  <span class="flex items-center gap-1">
                    <svg class="size-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><polyline points="12 6 12 12 16 14"/></svg>
                    {{ formatDuration(video.lengthSeconds) }}
                  </span>
                </div>
              </div>
              <h3 class="text-sm font-medium text-foreground line-clamp-2 group-hover:text-primary">{{ video.title }}</h3>
              <p class="text-xs text-muted-foreground mt-1">{{ formatViews(video.viewCount) }} • {{ timeAgo(video.published) }}</p>
            </router-link>
          </div>
          <EmptyState v-else title="No videos found">
            This channel hasn't uploaded any videos yet.
          </EmptyState>
        </div>

        <!-- ==================== SHORTS TAB ==================== -->
        <div v-else-if="activeTab === 'shorts'">
          <div v-if="tabLoading" class="flex gap-3">
            <div v-for="n in 6" :key="n" class="w-40 aspect-[9/16] bg-muted rounded-xl animate-pulse" />
          </div>
          <div v-else-if="shorts.length > 0" class="flex gap-3 overflow-x-auto pb-4">
            <router-link
              v-for="short in shorts"
              :key="short.id"
              :to="`/watch?v=${short.id}`"
              class="shrink-0 w-40 group"
            >
              <div class="aspect-[9/16] rounded-xl overflow-hidden bg-muted mb-2 relative">
                <img v-if="short.thumbnail" :src="short.thumbnail" :alt="short.title" class="w-full h-full object-cover" />
                <div v-else class="absolute inset-0 flex items-center justify-center text-3xl opacity-30">📱</div>
                <div v-if="short.lengthSeconds > 0" class="absolute bottom-1 right-1 bg-black/75 text-white text-[10px] px-1 rounded">
                  {{ short.lengthSeconds }}s
                </div>
              </div>
              <p class="text-xs font-medium text-foreground line-clamp-2 group-hover:text-primary">{{ short.title }}</p>
              <p class="text-[10px] text-muted-foreground">{{ formatViews(short.viewCount) }}</p>
            </router-link>
          </div>
          <EmptyState v-else title="No shorts found">
            This channel hasn't uploaded any shorts yet.
          </EmptyState>
        </div>

        <!-- ==================== PLAYLISTS TAB ==================== -->
        <div v-else-if="activeTab === 'playlists'">
          <div v-if="playlists.length > 0" class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
            <div
              v-for="playlist in playlists"
              :key="playlist.id"
              class="cursor-pointer group"
            >
              <div class="aspect-video rounded-xl overflow-hidden bg-muted relative">
                <img v-if="playlist.thumbnail || playlist.playlistThumbnail" :src="playlist.thumbnail || playlist.playlistThumbnail" :alt="playlist.title" class="w-full h-full object-cover" />
                <div v-else class="absolute inset-0 flex items-center justify-center text-3xl opacity-30">📋</div>
                <div class="absolute inset-0 bg-black/40 flex items-center justify-center">
                  <span class="text-white text-sm font-medium flex items-center gap-1">
                    <PhList :size="16" />
                    {{ playlist.videoCount || 0 }} videos
                  </span>
                </div>
              </div>
              <p class="text-sm font-medium text-foreground mt-1 line-clamp-2 group-hover:text-primary">{{ playlist.title }}</p>
            </div>
          </div>
          <EmptyState v-else title="No playlists found">
            This channel hasn't created any playlists yet.
          </EmptyState>
        </div>

        <!-- ==================== COMMUNITY TAB ==================== -->
        <div v-else-if="activeTab === 'community'">
          <div v-if="tabLoading" class="space-y-4">
            <div v-for="n in 3" :key="n" class="rounded-xl border border-border bg-card p-4 animate-pulse">
              <div class="h-12 bg-muted rounded" />
            </div>
          </div>
          <div v-else-if="communityPosts.length > 0" class="max-w-3xl mx-auto space-y-4">
            <div
              v-for="post in communityPosts"
              :key="post.commentId || post.id"
              class="rounded-xl border border-border bg-card p-4"
            >
              <div class="flex items-center gap-3 mb-3">
                <div class="size-8 rounded-full bg-gradient-to-br from-primary to-purple-500 flex items-center justify-center text-xs text-white font-bold overflow-hidden">
                  <img v-if="post.authorThumbnails?.[0]?.url" :src="post.authorThumbnails[0].url" :alt="post.author" class="w-full h-full object-cover" />
                  <span v-else>{{ channel.name[0] }}</span>
                </div>
                <div>
                  <span class="text-sm font-medium text-foreground">{{ post.author || channel.name }}</span>
                  <p class="text-xs text-muted-foreground">{{ timeAgo((post.published || post.publishedText) as string) }}</p>
                </div>
              </div>
              <p class="text-sm text-foreground whitespace-pre-wrap leading-relaxed">{{ post.content || post.commentText || '' }}</p>
              <div class="flex items-center gap-4 mt-3 pt-3 border-t border-border">
                <button class="flex items-center gap-1.5 text-sm text-muted-foreground hover:text-foreground transition-colors">
                  <PhHeart :size="16" />
                  {{ post.likeCount || 0 }}
                </button>
                <button class="flex items-center gap-1.5 text-sm text-muted-foreground hover:text-foreground transition-colors">
                  <PhPlayCircle :size="16" />
                  {{ post.replyCount || post.commentCount || 0 }}
                </button>
              </div>
            </div>
          </div>
          <EmptyState v-else title="No community posts">
            This channel hasn't posted any community updates yet.
          </EmptyState>
        </div>
      </div>
    </template>
  </div>
</template>

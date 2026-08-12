<script setup lang="ts">
import { ref } from 'vue'
import { cn } from '@/lib/utils'
import { PhPlayCircle, PhList, PhHeart, PhShare } from '@phosphor-icons/vue'

const activeTab = ref('home')

const tabs = [
  { id: 'home', label: 'Home' },
  { id: 'videos', label: 'Videos' },
  { id: 'shorts', label: 'Shorts' },
  { id: 'playlists', label: 'Playlists' },
  { id: 'community', label: 'Community' },
]

const channel = ref({
  id: 'UCXuqSBlHAE6Xw-yeJA0Tunw',
  name: 'Linus Tech Tips',
  avatar: '',
  banner: '',
  subscriberCount: 15600000,
  videoCount: 6842,
  description: 'We make entertaining videos about technology, including product reviews, build guides, and more. Subscribe for new videos every week!',
  isSubscribed: false,
})

const dummyVideos = Array.from({ length: 12 }, (_, i) => ({
  id: `video-${i}`,
  title: `Amazing Tech Review ${i + 1}: You Won't Believe What We Found!`,
  thumbnail: '',
  author: channel.value.name,
  authorId: channel.value.id,
  viewCount: Math.floor(Math.random() * 5000000) + 100000,
  published: `${Math.floor(Math.random() * 11) + 1} days ago`,
  lengthSeconds: Math.floor(Math.random() * 600) + 60,
  description: 'This is a sample video description. In a real implementation, this would come from the API.',
}))

const dummyShorts = Array.from({ length: 6 }, (_, i) => ({
  id: `short-${i}`,
  title: `Quick Tech Tip #${i + 1}`,
  thumbnail: '',
  author: channel.value.name,
  authorId: channel.value.id,
  viewCount: Math.floor(Math.random() * 1000000) + 50000,
  lengthSeconds: Math.floor(Math.random() * 50) + 10,
}))

const dummyPlaylists = Array.from({ length: 4 }, (_, i) => ({
  id: `playlist-${i}`,
  title: `Tech Series ${i + 1}: Complete Guide`,
  thumbnail: '',
  videoCount: Math.floor(Math.random() * 20) + 5,
}))

const dummyPosts = Array.from({ length: 3 }, (_, i) => ({
  id: `post-${i}`,
  content: `Hey everyone! We're working on something exciting for next week. Stay tuned! 🎉`,
  published: `${i + 1} days ago`,
  likeCount: Math.floor(Math.random() * 5000) + 100,
  commentCount: Math.floor(Math.random() * 500) + 10,
}))

function formatSubscribers(count: number): string {
  if (count >= 1000000) return `${(count / 1000000).toFixed(1)}M subscribers`
  if (count >= 1000) return `${(count / 1000).toFixed(1)}K subscribers`
  return `${count} subscribers`
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
</script>

<template>
  <div class="min-h-screen">
    <!-- Banner -->
    <div class="relative h-40 sm:h-48 md:h-56 bg-gradient-to-r from-blue-600/30 to-purple-600/30">
      <div class="absolute inset-0 flex items-center justify-center">
        <span class="text-6xl opacity-20">📺</span>
      </div>
    </div>

    <!-- Channel Info Bar -->
    <div class="border-b border-border">
      <div class="max-w-7xl mx-auto px-4 py-4">
        <div class="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
          <div class="flex items-center gap-4">
            <!-- Avatar -->
            <div class="size-20 -mt-10 rounded-full border-4 border-background bg-gradient-to-br from-primary to-purple-500 flex items-center justify-center text-2xl text-white font-bold shrink-0">
              {{ channel.name[0] }}
            </div>
            <div>
              <h1 class="text-xl font-bold text-foreground">{{ channel.name }}</h1>
              <p class="text-sm text-muted-foreground">
                {{ formatSubscribers(channel.subscriberCount) }} • {{ channel.videoCount.toLocaleString() }} videos
              </p>
              <p class="text-xs text-muted-foreground mt-1 line-clamp-1 max-w-md">{{ channel.description }}</p>
            </div>
          </div>
          <div class="flex items-center gap-2">
            <button
              :class="cn(
                'h-9 rounded-full px-6 text-sm font-medium transition-colors shrink-0',
                channel.isSubscribed
                  ? 'border border-border bg-muted text-foreground hover:bg-muted/80'
                  : 'bg-primary text-primary-foreground hover:bg-primary/90'
              )"
              @click="channel.isSubscribed = !channel.isSubscribed"
            >
              {{ channel.isSubscribed ? 'Subscribed' : 'Subscribe' }}
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

    <!-- Tabs -->
    <div class="sticky top-0 z-30 bg-background border-b border-border">
      <div class="max-w-7xl mx-auto px-4">
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
          </button>
        </nav>
      </div>
    </div>

    <!-- Tab Content -->
    <div class="max-w-7xl mx-auto px-4 py-6">

      <!-- Home / Videos Tab -->
      <div v-if="activeTab === 'home' || activeTab === 'videos'">
        <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
          <router-link
            v-for="video in dummyVideos"
            :key="video.id"
            :to="`/watch?v=${video.id}`"
            class="group"
          >
            <div class="aspect-video rounded-xl overflow-hidden bg-muted mb-2 relative">
              <div class="absolute inset-0 flex items-center justify-center text-4xl opacity-30">🎬</div>
              <div class="absolute bottom-1 right-1 bg-black/80 text-white text-[10px] px-1 py-0.5 rounded">
                {{ formatDuration(video.lengthSeconds) }}
              </div>
            </div>
            <h3 class="text-sm font-medium text-foreground line-clamp-2 group-hover:text-primary">{{ video.title }}</h3>
            <p class="text-xs text-muted-foreground mt-1">{{ formatViews(video.viewCount) }} • {{ video.published }}</p>
          </router-link>
        </div>
      </div>

      <!-- Shorts Tab -->
      <div v-else-if="activeTab === 'shorts'">
        <div class="flex gap-3 overflow-x-auto pb-4">
          <router-link
            v-for="short in dummyShorts"
            :key="short.id"
            :to="`/watch?v=${short.id}`"
            class="shrink-0 w-40 group"
          >
            <div class="aspect-[9/16] rounded-xl overflow-hidden bg-muted mb-2 relative">
              <div class="absolute inset-0 flex items-center justify-center text-3xl opacity-30">📱</div>
              <div class="absolute bottom-1 right-1 bg-black/75 text-white text-[10px] px-1 rounded">
                {{ short.lengthSeconds }}s
              </div>
            </div>
            <p class="text-xs font-medium text-foreground line-clamp-2 group-hover:text-primary">{{ short.title }}</p>
            <p class="text-[10px] text-muted-foreground">{{ formatViews(short.viewCount) }}</p>
          </router-link>
        </div>
      </div>

      <!-- Playlists Tab -->
      <div v-else-if="activeTab === 'playlists'">
        <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
          <div
            v-for="playlist in dummyPlaylists"
            :key="playlist.id"
            class="cursor-pointer group"
          >
            <div class="aspect-video rounded-xl overflow-hidden bg-muted relative">
              <div class="absolute inset-0 flex items-center justify-center text-4xl opacity-30">📋</div>
              <div class="absolute inset-0 bg-black/40 flex items-center justify-center">
                <span class="text-white text-sm font-medium flex items-center gap-1">
                  <PhList :size="16" />
                  {{ playlist.videoCount }} videos
                </span>
              </div>
            </div>
            <p class="text-sm font-medium text-foreground mt-1 line-clamp-2 group-hover:text-primary">{{ playlist.title }}</p>
          </div>
        </div>
      </div>

      <!-- Community Tab -->
      <div v-else-if="activeTab === 'community'">
        <div class="max-w-2xl space-y-4">
          <div
            v-for="post in dummyPosts"
            :key="post.id"
            class="rounded-xl border border-border bg-card p-4"
          >
            <div class="flex items-center gap-3 mb-3">
              <div class="size-8 rounded-full bg-gradient-to-br from-primary to-purple-500 flex items-center justify-center text-xs text-white font-bold">
                {{ channel.name[0] }}
              </div>
              <div>
                <span class="text-sm font-medium text-foreground">{{ channel.name }}</span>
                <p class="text-xs text-muted-foreground">{{ post.published }}</p>
              </div>
            </div>
            <p class="text-sm text-foreground whitespace-pre-wrap leading-relaxed">{{ post.content }}</p>
            <div class="flex items-center gap-4 mt-3 pt-3 border-t border-border">
              <button class="flex items-center gap-1.5 text-sm text-muted-foreground hover:text-foreground transition-colors">
                <PhHeart :size="16" />
                {{ post.likeCount.toLocaleString() }}
              </button>
              <button class="flex items-center gap-1.5 text-sm text-muted-foreground hover:text-foreground transition-colors">
                <PhPlayCircle :size="16" />
                {{ post.commentCount }} comments
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

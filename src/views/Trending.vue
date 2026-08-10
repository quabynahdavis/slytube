<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { cn } from '@/lib/utils'

const isLoading = ref(true)
const selectedCategory = ref('all')
const videos = ref<Array<{
  videoId: string; title: string; author: string; authorId: string; viewCount: number; lengthSeconds: number; published: string; videoThumbnails: Array<{ url: string; width: number; height: number }>
}>>([])

const categories = [
  { value: 'all', label: 'All' },
  { value: 'music', label: 'Music' },
  { value: 'gaming', label: 'Gaming' },
  { value: 'news', label: 'News' },
  { value: 'sports', label: 'Sports' },
  { value: 'learning', label: 'Learning' },
  { value: 'fashion', label: 'Fashion' },
]

onMounted(async () => {
  isLoading.value = true
  try {
    await new Promise((r) => setTimeout(r, 600))
    videos.value = Array.from({ length: 24 }, (_, i) => ({
      videoId: `trending-${i}`, title: `Trending Video ${i + 1} - Popular Content`,
      author: `Creator ${i + 1}`, authorId: `UC-creator-${i}`,
      viewCount: Math.floor(Math.random() * 10000000), lengthSeconds: Math.floor(Math.random() * 600) + 60,
      published: new Date(Date.now() - Math.random() * 7 * 86400000).toISOString().split('T')[0],
      videoThumbnails: [{ url: '', width: 640, height: 360 }],
    }))
  } finally {
    isLoading.value = false
  }
})

function formatViews(v: number): string { return v >= 1e6 ? `${(v / 1e6).toFixed(1)}M` : v >= 1e3 ? `${(v / 1e3).toFixed(1)}K` : `${v}` }
function formatDuration(s: number): string { const h = Math.floor(s / 3600), m = Math.floor((s % 3600) / 60), sec = s % 60; return h > 0 ? `${h}:${m.toString().padStart(2, '0')}:${sec.toString().padStart(2, '0')}` : `${m}:${sec.toString().padStart(2, '0')}` }
function timeAgo(d: string): string { const days = Math.floor((Date.now() - new Date(d).getTime()) / 86400000); return days > 0 ? `${days}d ago` : 'Today' }
</script>

<template>
  <div class="container mx-auto max-w-7xl px-4 py-6">
    <div class="mb-6">
      <h1 class="text-2xl font-bold text-foreground">Trending</h1>
      <p class="text-sm text-muted-foreground mt-1">What's popular right now</p>
    </div>

    <div class="flex gap-2 overflow-x-auto pb-4 mb-6">
      <button v-for="cat in categories" :key="cat.value" :class="cn('shrink-0 rounded-full px-4 py-1.5 text-sm font-medium transition-colors', selectedCategory === cat.value ? 'bg-primary text-primary-foreground' : 'bg-muted text-muted-foreground hover:bg-muted/80')" @click="selectedCategory = cat.value">{{ cat.label }}</button>
    </div>

    <div v-if="isLoading" class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
      <div v-for="n in 8" :key="n" class="animate-pulse"><div class="aspect-video rounded-lg bg-muted"/><div class="mt-3 space-y-2"><div class="h-4 w-3/4 rounded bg-muted"/><div class="h-3 w-1/2 rounded bg-muted"/></div></div>
    </div>

    <div v-else class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
      <router-link v-for="(video, idx) in videos" :key="video.videoId" :to="`/watch?v=${video.videoId}`" class="group block">
        <div class="relative aspect-video rounded-lg bg-muted overflow-hidden">
          <div class="absolute top-2 left-2 rounded bg-primary px-2 py-0.5 text-xs font-bold text-primary-foreground">#{{ idx + 1 }}</div>
          <div class="absolute inset-0 flex items-center justify-center"><svg class="size-12 text-muted-foreground/50 group-hover:text-primary transition-colors" viewBox="0 0 24 24" fill="currentColor"><polygon points="5 3 19 12 5 21 5 3"/></svg></div>
          <span class="absolute bottom-2 right-2 rounded bg-black/80 px-1.5 py-0.5 text-xs text-white font-medium">{{ formatDuration(video.lengthSeconds) }}</span>
        </div>
        <div class="mt-2"><h3 class="text-sm font-medium text-foreground line-clamp-2 group-hover:text-primary transition-colors">{{ video.title }}</h3><p class="mt-1 text-xs text-muted-foreground">{{ video.author }}</p><p class="text-xs text-muted-foreground">{{ formatViews(video.viewCount) }} views &middot; {{ timeAgo(video.published) }}</p></div>
      </router-link>
    </div>
  </div>
</template>

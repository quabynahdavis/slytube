<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRoute } from 'vue-router'

const route = useRoute()

const isLoading = ref(true)
const hashtag = computed(() => route.params.tag as string || route.query.tag as string || '')

const videos = ref<Array<{
  videoId: string; title: string; author: string; authorId: string; viewCount: number; lengthSeconds: number; published: string; videoThumbnails: Array<{ url: string; width: number; height: number }>
}>>([])

onMounted(async () => {
  isLoading.value = true
  try {
    await new Promise((r) => setTimeout(r, 500))
    videos.value = Array.from({ length: 16 }, (_, i) => ({
      videoId: `hashtag-${i}`, title: `Video about #${hashtag.value} - Content ${i + 1}`,
      author: `Creator ${i + 1}`, authorId: `UC-creator-${i}`,
      viewCount: Math.floor(Math.random() * 2000000), lengthSeconds: Math.floor(Math.random() * 600) + 60,
      published: new Date(Date.now() - Math.random() * 30 * 86400000).toISOString().split('T')[0],
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
      <div class="flex items-center gap-3">
        <div class="size-12 rounded-full bg-primary/10 flex items-center justify-center">
          <svg class="size-6 text-primary" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="4" y1="9" x2="20" y2="9"/><line x1="4" y1="15" x2="20" y2="15"/><line x1="10" y1="3" x2="8" y2="21"/><line x1="16" y1="3" x2="14" y2="21"/></svg>
        </div>
        <div>
          <h1 class="text-2xl font-bold text-foreground">#{{ hashtag }}</h1>
          <p class="text-sm text-muted-foreground">{{ videos.length }} videos</p>
        </div>
      </div>
    </div>

    <div v-if="isLoading" class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
      <div v-for="n in 8" :key="n" class="animate-pulse"><div class="aspect-video rounded-lg bg-muted"/><div class="mt-3 space-y-2"><div class="h-4 w-3/4 rounded bg-muted"/><div class="h-3 w-1/2 rounded bg-muted"/></div></div>
    </div>

    <div v-else-if="videos.length === 0" class="rounded-lg border border-border bg-card p-12 text-center">
      <svg class="size-16 mx-auto mb-4 text-muted-foreground/50" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><line x1="4" y1="9" x2="20" y2="9"/><line x1="4" y1="15" x2="20" y2="15"/><line x1="10" y1="3" x2="8" y2="21"/><line x1="16" y1="3" x2="14" y2="21"/></svg>
      <h3 class="text-lg font-medium text-foreground">No videos found</h3>
      <p class="text-sm text-muted-foreground mt-1">No videos found for #{{ hashtag }}</p>
    </div>

    <div v-else class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
      <router-link v-for="video in videos" :key="video.videoId" :to="`/watch?v=${video.videoId}`" class="group block">
        <div class="relative aspect-video rounded-lg bg-muted overflow-hidden">
          <div class="absolute inset-0 flex items-center justify-center"><svg class="size-12 text-muted-foreground/50 group-hover:text-primary transition-colors" viewBox="0 0 24 24" fill="currentColor"><polygon points="5 3 19 12 5 21 5 3"/></svg></div>
          <span class="absolute bottom-2 right-2 rounded bg-black/80 px-1.5 py-0.5 text-xs text-white font-medium">{{ formatDuration(video.lengthSeconds) }}</span>
        </div>
        <div class="mt-2"><h3 class="text-sm font-medium text-foreground line-clamp-2 group-hover:text-primary transition-colors">{{ video.title }}</h3><p class="mt-1 text-xs text-muted-foreground">{{ video.author }}</p><p class="text-xs text-muted-foreground">{{ formatViews(video.viewCount) }} views &middot; {{ timeAgo(video.published) }}</p></div>
      </router-link>
    </div>
  </div>
</template>

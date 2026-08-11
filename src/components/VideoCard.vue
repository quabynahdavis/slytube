<script setup lang="ts">
import type { Video } from '../api/types'

defineProps<{
  video: Video
}>()

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

function timeAgo(published: string): string {
  if (!published) return ''
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
  <router-link :to="`/watch?v=${video.id}`" class="block group">
    <div class="relative aspect-video rounded-xl overflow-hidden bg-muted mb-2">
      <img
        v-if="video.thumbnail"
        :src="video.thumbnail"
        :alt="video.title"
        class="w-full h-full object-cover transition-transform group-hover:scale-105"
      />
      <div v-else class="w-full h-full flex items-center justify-center">
        <svg class="size-8 text-muted-foreground" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.5">
          <path stroke-linecap="round" stroke-linejoin="round" d="M15.75 10.5l4.72-4.72a.75.75 0 011.28.53v11.38a.75.75 0 01-1.28.53l-4.72-4.72M4.5 18.75h9a2.25 2.25 0 002.25-2.25v-9a2.25 2.25 0 00-2.25-2.25h-9A2.25 2.25 0 002.25 7.5v9a2.25 2.25 0 002.25 2.25z" />
        </svg>
      </div>
      <!-- Duration Badge -->
      <div v-if="video.lengthSeconds > 0" class="absolute bottom-2 right-2 bg-black/80 text-white text-xs px-1.5 py-0.5 rounded">
        {{ formatDuration(video.lengthSeconds) }}
      </div>
      <!-- Live Badge -->
      <div v-if="video.isLive" class="absolute bottom-2 right-2 bg-red-600 text-white text-xs px-2 py-0.5 rounded font-medium">
        LIVE
      </div>
      <!-- Upcoming Badge -->
      <div v-if="video.isUpcoming" class="absolute bottom-2 right-2 bg-orange-500 text-white text-xs px-2 py-0.5 rounded font-medium">
        UPCOMING
      </div>
    </div>
    <div class="flex gap-3">
      <div class="size-9 rounded-full bg-primary/20 flex items-center justify-center shrink-0">
        <span class="text-xs font-medium text-primary">{{ video.author?.[0] || '?' }}</span>
      </div>
      <div class="flex-1 min-w-0">
        <h3 class="text-sm font-medium text-foreground line-clamp-2 group-hover:text-primary leading-snug">
          {{ video.title }}
        </h3>
        <p class="text-xs text-muted-foreground mt-1 hover:text-foreground">{{ video.author }}</p>
        <p class="text-xs text-muted-foreground">
          {{ formatViews(video.viewCount) }} &middot; {{ timeAgo(video.published) }}
        </p>
      </div>
    </div>
  </router-link>
</template>

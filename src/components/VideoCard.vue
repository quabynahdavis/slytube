<script setup lang="ts">
import type { Video } from '../api/types'
import { useWatchQueueStore } from '../stores/watch-queue'
import { useToast } from '../composables/useToast'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
  DropdownMenuSeparator,
} from '../components/ui/dropdown-menu'

const props = defineProps<{
  video: Video
}>()

const watchQueueStore = useWatchQueueStore()
const toast = useToast()

function addToWatchLater() {
  watchQueueStore.addVideoToWatchQueue({
    videoId: props.video.id,
    title: props.video.title,
    author: props.video.author,
    authorId: props.video.authorId,
    lengthSeconds: props.video.lengthSeconds,
    videoThumbnails: props.video.thumbnail
      ? [{ url: props.video.thumbnail, width: 320, height: 180 }]
      : [],
  })
  toast.success(`Added to queue: ${props.video.title}`)
}

function addToQueue() {
  watchQueueStore.addVideoToWatchQueue({
    videoId: props.video.id,
    title: props.video.title,
    author: props.video.author,
    authorId: props.video.authorId,
    lengthSeconds: props.video.lengthSeconds,
    videoThumbnails: props.video.thumbnail
      ? [{ url: props.video.thumbnail, width: 320, height: 180 }]
      : [],
  }, true)
  toast.success(`Playing next: ${props.video.title}`)
}

function copyToClipboard(text: string, label: string) {
  navigator.clipboard.writeText(text)
  toast.success(`${label} copied`)
}

function copyYoutubeLink() {
  copyToClipboard(`https://www.youtube.com/watch?v=${props.video.id}`, 'YouTube link')
}

function copyEmbedLink() {
  copyToClipboard(`https://www.youtube.com/embed/${props.video.id}`, 'Embed link')
}

function copyInvidiousLink() {
  copyToClipboard(`https://yewtu.be/watch?v=${props.video.id}`, 'Invidious link')
}

function openInYoutube() {
  window.open(`https://www.youtube.com/watch?v=${props.video.id}`, '_blank')
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
  <div
    class="group relative rounded-xl transition-colors duration-200 hover:bg-muted/50 p-3"
  >
    <!-- Video Thumbnail -->
    <router-link :to="`/watch?v=${video.id}`" class="block relative">
      <div class="relative aspect-video rounded-xl overflow-hidden bg-muted mb-3">
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
        <div  class="absolute bottom-2 right-2 flex items-center gap-1 bg-black/80 text-white text-xs px-1.5 py-0.5 rounded font-medium">
          <svg class="size-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="12" cy="12" r="10" />
            <polyline points="12 6 12 12 16 14" />
          </svg>
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
    </router-link>

    <!-- Metadata Row -->
    <div class="flex gap-3 mt-2 p-3 relative">
      <!-- Creator Avatar - Links to Channel -->
      <router-link
        :to="`/channel/${video.authorId}`"
        class="size-9 rounded-full overflow-hidden shrink-0 bg-muted ring-2 ring-transparent hover:ring-primary/30 transition-all mt-0.5"
        @click.stop
      >
        <img
          v-if="video.authorAvatar"
          :src="video.authorAvatar"
          :alt="video.author"
          class="w-full h-full object-cover"
        />
        <div v-else class="w-full h-full bg-primary/20 flex items-center justify-center">
          <span class="text-xs font-medium text-primary">{{ video.author?.[0] || '?' }}</span>
        </div>
      </router-link>

      <!-- Title and Metadata - Links to Video -->
      <router-link :to="`/watch?v=${video.id}`" class="flex-1 min-w-0">
        <h3 class="text-base font-medium text-foreground line-clamp-2 leading-snug">
          {{ video.title }}
        </h3>
        <p class="text-xs text-muted-foreground mt-1 hover:text-foreground">{{ video.author }}</p>
        <p class="text-xs text-muted-foreground">
          {{ formatViews(video.viewCount) }} &middot; {{ timeAgo(video.published) }}
        </p>
      </router-link>

      <!-- Ellipsis Dropdown -->
      <DropdownMenu>
        <DropdownMenuTrigger as-child>
          <button
            class="size-8 flex items-center justify-center rounded-full hover:bg-accent transition-all shrink-0"
            @click.stop
          >
            <svg class="size-5 text-foreground" viewBox="0 0 24 24" fill="currentColor">
              <circle cx="12" cy="5" r="2" />
              <circle cx="12" cy="12" r="2" />
              <circle cx="12" cy="19" r="2" />
            </svg>
          </button>
        </DropdownMenuTrigger>
        <DropdownMenuContent align="end" class="w-52">
          <DropdownMenuItem @click="addToQueue" class="gap-2">
            <svg class="size-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <polygon points="5 3 19 12 5 21 5 3" />
            </svg>
            Play Next
          </DropdownMenuItem>
          <DropdownMenuItem @click="addToWatchLater" class="gap-2">
            <svg class="size-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <circle cx="12" cy="12" r="10" />
              <polyline points="12 6 12 12 16 14" />
            </svg>
            Add to Queue
          </DropdownMenuItem>
          <DropdownMenuItem class="gap-2">
            <svg class="size-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M19 21l-7-5-7 5V5a2 2 0 012-2h10a2 2 0 012 2z" />
            </svg>
            Save to Playlist
          </DropdownMenuItem>
          <DropdownMenuSeparator />
          <DropdownMenuItem @click="copyYoutubeLink" class="gap-2">
            <svg class="size-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M10 13a5 5 0 007.54.54l3-3a5 5 0 00-7.07-7.07l-1.72 1.71" />
              <path d="M14 11a5 5 0 00-7.54-.54l-3 3a5 5 0 007.07 7.07l1.71-1.71" />
            </svg>
            Copy YouTube Link
          </DropdownMenuItem>
          <DropdownMenuItem @click="copyEmbedLink" class="gap-2">
            <svg class="size-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <polyline points="16 18 22 12 16 6" />
              <polyline points="8 6 2 12 8 18" />
            </svg>
            Copy Embed Link
          </DropdownMenuItem>
          <DropdownMenuItem @click="copyInvidiousLink" class="gap-2">
            <svg class="size-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M10 13a5 5 0 007.54.54l3-3a5 5 0 00-7.07-7.07l-1.72 1.71" />
              <path d="M14 11a5 5 0 00-7.54-.54l-3 3a5 5 0 007.07 7.07l1.71-1.71" />
            </svg>
            Copy Invidious Link
          </DropdownMenuItem>
          <DropdownMenuItem @click="openInYoutube" class="gap-2">
            <svg class="size-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M18 13v6a2 2 0 01-2 2H5a2 2 0 01-2-2V8a2 2 0 012-2h6" />
              <polyline points="15 3 21 3 21 9" />
              <line x1="10" y1="14" x2="21" y2="3" />
            </svg>
            Open in YouTube
          </DropdownMenuItem>
          <DropdownMenuItem class="gap-2 text-destructive">
            <svg class="size-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M18.36 6.64a9 9 0 11-12.73 0" />
              <line x1="12" y1="2" x2="12" y2="12" />
            </svg>
            Don't Recommend This Channel
          </DropdownMenuItem>
          <DropdownMenuSeparator />
          <DropdownMenuItem class="gap-2">
            <svg class="size-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <polyline points="20 6 9 17 4 12" />
            </svg>
            Mark as Watched
          </DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>
    </div>
  </div>
</template>

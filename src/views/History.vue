<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { cn } from '@/lib/utils'
import { useHistoryStore } from '@/stores/history'

const historyStore = useHistoryStore()

const isLoading = ref(true)
const showClearConfirm = ref(false)
const searchQuery = ref('')

const allHistory = computed(() => historyStore.getHistoryCacheSorted)

const filteredHistory = computed(() => {
  if (!searchQuery.value.trim()) return allHistory.value
  const q = searchQuery.value.toLowerCase()
  return allHistory.value.filter(
    (entry) =>
      entry.title.toLowerCase().includes(q) ||
      entry.author.toLowerCase().includes(q)
  )
})

onMounted(async () => {
  isLoading.value = true
  try {
    await new Promise((resolve) => setTimeout(resolve, 500))
    // Placeholder: Load history from store
    const sampleEntries = Array.from({ length: 20 }, (_, i) => ({
      videoId: `history-${i}`,
      title: `Watched Video ${i + 1}`,
      author: `Channel ${i + 1}`,
      authorId: `UC-channel-${i}`,
      authorUrl: `/channel/UC-channel-${i}`,
      description: '',
      viewCount: Math.floor(Math.random() * 5000000),
      lengthSeconds: Math.floor(Math.random() * 600) + 60,
      timeWatched: Date.now() - i * 86400000,
      watchProgress: Math.random(),
      isWatched: Math.random() > 0.5,
      type: 'video',
      videoThumbnails: [{ url: '', width: 320, height: 180 }],
    }))
    sampleEntries.forEach((entry) => historyStore.addToHistory(entry))
  } finally {
    isLoading.value = false
  }
})

function formatDuration(seconds: number): string {
  const h = Math.floor(seconds / 3600)
  const m = Math.floor((seconds % 3600) / 60)
  const s = seconds % 60
  if (h > 0) return `${h}:${m.toString().padStart(2, '0')}:${s.toString().padStart(2, '0')}`
  return `${m}:${s.toString().padStart(2, '0')}`
}

function formatDate(timestamp: number): string {
  return new Date(timestamp).toLocaleDateString(undefined, {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
  })
}

function removeEntry(videoId: string) {
  historyStore.removeFromHistory(videoId)
}

function clearAllHistory() {
  historyStore.clearHistory()
  showClearConfirm.value = false
}
</script>

<template>
  <div class="container mx-auto max-w-5xl px-4 py-6">
    <!-- Header -->
    <div class="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4 mb-6">
      <div>
        <h1 class="text-2xl font-bold text-foreground">Watch History</h1>
        <p class="text-sm text-muted-foreground mt-1">{{ allHistory.length }} videos watched</p>
      </div>
      <div class="flex items-center gap-2">
        <div class="relative">
          <input
            v-model="searchQuery"
            type="search"
            placeholder="Search history..."
            class="h-9 w-48 rounded-md border border-input bg-background pl-8 pr-3 text-sm outline-none focus:border-primary focus:ring-1 focus:ring-primary"
          />
          <svg class="absolute left-2.5 top-2.5 size-4 text-muted-foreground" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="11" cy="11" r="8" />
            <line x1="21" y1="21" x2="16.65" y2="16.65" />
          </svg>
        </div>
        <button
          class="inline-flex items-center gap-1 rounded-md border border-destructive/50 bg-destructive/10 px-3 py-2 text-sm text-destructive hover:bg-destructive/20 transition-colors"
          @click="showClearConfirm = true"
        >
          <svg class="size-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <polyline points="3 6 5 6 21 6" />
            <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" />
          </svg>
          Clear History
        </button>
      </div>
    </div>

    <!-- Loading State -->
    <div v-if="isLoading" class="space-y-3">
      <div v-for="n in 6" :key="n" class="flex gap-3 animate-pulse">
        <div class="w-48 aspect-video rounded-lg bg-muted shrink-0" />
        <div class="flex-1 space-y-2">
          <div class="h-4 w-3/4 rounded bg-muted" />
          <div class="h-3 w-1/2 rounded bg-muted" />
          <div class="h-3 w-1/4 rounded bg-muted" />
        </div>
      </div>
    </div>

    <!-- Empty State -->
    <div v-else-if="filteredHistory.length === 0" class="rounded-lg border border-border bg-card p-12 text-center">
      <svg class="size-16 mx-auto mb-4 text-muted-foreground/50" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
        <circle cx="12" cy="12" r="10" />
        <polyline points="12 6 12 12 16 14" />
      </svg>
      <h3 class="text-lg font-medium text-foreground">No watch history</h3>
      <p class="text-sm text-muted-foreground mt-1">
        {{ searchQuery ? 'No results found for your search' : 'Videos you watch will appear here' }}
      </p>
    </div>

    <!-- History List -->
    <div v-else class="space-y-3">
      <div
        v-for="entry in filteredHistory"
        :key="entry.videoId"
        class="flex gap-3 rounded-lg border border-border bg-card p-2 group transition-colors hover:bg-accent/50"
      >
        <router-link
          :to="`/watch?v=${entry.videoId}`"
          class="relative shrink-0 w-48 aspect-video rounded bg-muted overflow-hidden"
        >
          <div class="absolute inset-0 flex items-center justify-center">
            <svg class="size-10 text-muted-foreground/50 group-hover:text-primary transition-colors" viewBox="0 0 24 24" fill="currentColor">
              <polygon points="5 3 19 12 5 21 5 3" />
            </svg>
          </div>
          <span class="absolute bottom-1 right-1 rounded bg-black/80 px-1 text-xs text-white">
            {{ formatDuration(entry.lengthSeconds) }}
          </span>
          <!-- Progress Bar -->
          <div class="absolute bottom-0 left-0 right-0 h-1 bg-muted-foreground/30">
            <div
              class="h-full bg-primary"
              :style="{ width: `${entry.watchProgress * 100}%` }"
            />
          </div>
        </router-link>
        <div class="min-w-0 flex-1">
          <router-link
            :to="`/watch?v=${entry.videoId}`"
            class="text-sm font-medium text-foreground line-clamp-2 hover:text-primary transition-colors"
          >
            {{ entry.title }}
          </router-link>
          <p class="mt-1 text-xs text-muted-foreground">{{ entry.author }}</p>
          <p class="text-xs text-muted-foreground">
            {{ entry.viewCount.toLocaleString() }} views &middot; Watched {{ formatDate(entry.timeWatched) }}
          </p>
        </div>
        <div class="shrink-0 opacity-0 group-hover:opacity-100 transition-opacity">
          <button
            class="size-8 rounded-md text-muted-foreground hover:bg-accent hover:text-accent-foreground flex items-center justify-center"
            title="Remove from history"
            @click="removeEntry(entry.videoId)"
          >
            <svg class="size-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <line x1="18" y1="6" x2="6" y2="18" />
              <line x1="6" y1="6" x2="18" y2="18" />
            </svg>
          </button>
        </div>
      </div>
    </div>

    <!-- Clear History Confirmation Dialog -->
    <Teleport to="body">
      <div
        v-if="showClearConfirm"
        class="fixed inset-0 z-50 flex items-center justify-center bg-black/50"
        @click.self="showClearConfirm = false"
      >
        <div class="w-full max-w-sm rounded-lg bg-card border border-border p-6 shadow-xl">
          <h3 class="text-lg font-semibold text-foreground">Clear Watch History</h3>
          <p class="text-sm text-muted-foreground mt-2">
            This will permanently delete all your watch history. This action cannot be undone.
          </p>
          <div class="flex justify-end gap-2 mt-6">
            <button
              class="h-9 rounded-md border border-input bg-background px-4 text-sm font-medium text-foreground hover:bg-accent transition-colors"
              @click="showClearConfirm = false"
            >
              Cancel
            </button>
            <button
              class="h-9 rounded-md bg-destructive px-4 text-sm font-medium text-destructive-foreground hover:bg-destructive/90 transition-colors"
              @click="clearAllHistory"
            >
              Clear All
            </button>
          </div>
        </div>
      </div>
    </Teleport>
  </div>
</template>

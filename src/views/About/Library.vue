<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const dbStats = ref<any>(null)
const loading = ref(true)

onMounted(async () => {
  try {
    dbStats.value = await invoke<any>('db_get_stats')
  } catch {
    dbStats.value = null
  }
  loading.value = false
})

const statItems = [
  { key: 'subscriptions', label: 'Subscriptions', icon: 'M19 11a7 7 0 01-7 7m0 0a7 7 0 01-7-7m7 7v4m0 0H8m4 0h4m-4-8a3 3 0 01-3-3V5a3 3 0 116 0v6a3 3 0 01-3 3z' },
  { key: 'playlists', label: 'Playlists', icon: 'M4 6h16M4 10h16M4 14h16M4 18h16' },
  { key: 'history', label: 'History', icon: 'M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z' },
  { key: 'downloads', label: 'Downloads', icon: 'M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4' },
  { key: 'search_history', label: 'Searches', icon: 'M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z' },
]
</script>

<template>
  <div class="space-y-6">
    <div class="rounded-xl border border-border bg-card p-6">
      <h2 class="text-lg font-semibold text-foreground mb-4">Your Library</h2>

      <div v-if="loading" class="flex items-center gap-2 text-sm text-muted-foreground">
        <svg class="size-4 animate-spin" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M21 12a9 9 0 11-6.219-8.56" />
        </svg>
        Loading...
      </div>

      <div v-else-if="dbStats" class="grid grid-cols-2 sm:grid-cols-3 gap-4">
        <div
          v-for="item in statItems"
          :key="item.key"
          class="rounded-xl bg-gradient-to-br from-primary/5 to-primary/10 p-4 text-center"
        >
          <div class="size-10 mx-auto mb-2 rounded-full bg-primary/10 flex items-center justify-center">
            <svg class="size-5 text-primary" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
              <path :d="item.icon" />
            </svg>
          </div>
          <p class="text-2xl font-bold text-foreground">{{ dbStats[item.key] }}</p>
          <p class="text-xs text-muted-foreground">{{ item.label }}</p>
        </div>
      </div>

      <div v-else class="text-sm text-muted-foreground">
        Database statistics unavailable.
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const systemInfo = ref<any>(null)
const loading = ref(true)

onMounted(async () => {
  try {
    systemInfo.value = await invoke<any>('get_system_info')
  } catch {
    systemInfo.value = { os: 'Unknown', arch: 'Unknown' }
  }
  loading.value = false
})
</script>

<template>
  <div class="space-y-6">
    <div class="rounded-xl border border-border bg-card p-6">
      <h2 class="text-lg font-semibold text-foreground mb-4">System Information</h2>

      <div v-if="loading" class="flex items-center gap-2 text-sm text-muted-foreground">
        <svg class="size-4 animate-spin" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M21 12a9 9 0 11-6.219-8.56" />
        </svg>
        Loading...
      </div>

      <div v-else-if="systemInfo" class="grid grid-cols-2 gap-4">
        <div class="rounded-lg bg-background/50 p-4">
          <p class="text-xs text-muted-foreground mb-1">Operating System</p>
          <p class="text-sm font-medium text-foreground capitalize">{{ systemInfo.os }} {{ systemInfo.family }}</p>
        </div>
        <div class="rounded-lg bg-background/50 p-4">
          <p class="text-xs text-muted-foreground mb-1">Architecture</p>
          <p class="text-sm font-medium text-foreground">{{ systemInfo.arch }}</p>
        </div>
        <div class="rounded-lg bg-background/50 p-4">
          <p class="text-xs text-muted-foreground mb-1">Tauri Version</p>
          <p class="text-sm font-medium text-foreground">{{ systemInfo.tauri_version }}</p>
        </div>
        <div class="rounded-lg bg-background/50 p-4">
          <p class="text-xs text-muted-foreground mb-1">Platform</p>
          <p class="text-sm font-medium text-foreground">{{ systemInfo.family === 'unix' ? 'Unix-like' : systemInfo.family }}</p>
        </div>
      </div>

      <div v-else class="text-sm text-muted-foreground">
        Unable to load system information.
      </div>
    </div>
  </div>
</template>

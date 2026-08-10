<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { cn } from '@/lib/utils'
import { useDownloadsStore } from '@/stores/downloads'

const downloadsStore = useDownloadsStore()

const isLoading = ref(true)
const activeTab = ref<'active' | 'completed'>('active')
const showDownloadForm = ref(false)
const newDownload = ref({ url: '', format: 'video:best', quality: '720', outputPath: '' })

const activeDownloads = computed(() => downloadsStore.getActiveDownloads)
const completedDownloads = computed(() => downloadsStore.getCompletedDownloads)

onMounted(async () => {
  isLoading.value = true
  try {
    await new Promise((r) => setTimeout(r, 500))
    const samples = [
      { id: 'dl-1', status: 'downloading' as const, progress: 45, title: 'Sample Video 1', url: 'https://youtube.com/watch?v=s1', outputPath: '~/Downloads', format: 'video:best', quality: '1080', fileSize: 250000000, downloadedBytes: 112500000, speed: 5000000, eta: 25, createdAt: Date.now() - 60000, updatedAt: Date.now() },
      { id: 'dl-2', status: 'processing' as const, progress: 100, title: 'Sample Video 2', url: 'https://youtube.com/watch?v=s2', outputPath: '~/Downloads', format: 'audio:best', quality: 'auto', createdAt: Date.now() - 120000, updatedAt: Date.now() },
      { id: 'dl-3', status: 'completed' as const, progress: 100, title: 'Sample Video 3', url: 'https://youtube.com/watch?v=s3', outputPath: '~/Downloads', format: 'video:720', quality: '720', fileSize: 180000000, createdAt: Date.now() - 3600000, updatedAt: Date.now() - 3000000 },
    ]
    samples.forEach((dl) => downloadsStore.addDownload(dl))
  } finally {
    isLoading.value = false
  }
})

function formatBytes(bytes: number): string {
  if (bytes >= 1e9) return `${(bytes / 1e9).toFixed(1)} GB`
  if (bytes >= 1e6) return `${(bytes / 1e6).toFixed(1)} MB`
  if (bytes >= 1e3) return `${(bytes / 1e3).toFixed(1)} KB`
  return `${bytes} B`
}

function startNewDownload() {
  if (!newDownload.value.url.trim()) return
  downloadsStore.addDownload({
    id: `dl-${Date.now()}`, status: 'queued', progress: 0, title: 'New Download',
    url: newDownload.value.url.trim(), outputPath: newDownload.value.outputPath || '~/Downloads',
    format: newDownload.value.format, quality: newDownload.value.quality,
    createdAt: Date.now(), updatedAt: Date.now(),
  })
  showDownloadForm.value = false
  newDownload.value = { url: '', format: 'video:best', quality: '720', outputPath: '' }
}
</script>

<template>
  <div class="container mx-auto max-w-5xl px-4 py-6">
    <div class="flex items-center justify-between mb-6">
      <div>
        <h1 class="text-2xl font-bold text-foreground">Downloads</h1>
        <p class="text-sm text-muted-foreground mt-1">Manage your video downloads</p>
      </div>
      <button class="inline-flex items-center gap-1 rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90 transition-colors" @click="showDownloadForm = true">
        <svg class="size-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/></svg>
        New Download
      </button>
    </div>

    <div class="flex items-center justify-between border-b border-border mb-6">
      <nav class="flex gap-6">
        <button :class="cn('pb-3 text-sm font-medium border-b-2 transition-colors', activeTab === 'active' ? 'border-primary text-foreground' : 'border-transparent text-muted-foreground hover:text-foreground')" @click="activeTab = 'active'">Active ({{ activeDownloads.length }})</button>
        <button :class="cn('pb-3 text-sm font-medium border-b-2 transition-colors', activeTab === 'completed' ? 'border-primary text-foreground' : 'border-transparent text-muted-foreground hover:text-foreground')" @click="activeTab = 'completed'">Completed ({{ completedDownloads.length }})</button>
      </nav>
      <button v-if="activeTab === 'completed' && completedDownloads.length > 0" class="pb-3 text-sm text-muted-foreground hover:text-foreground transition-colors" @click="downloadsStore.clearCompleted()">Clear Completed</button>
    </div>

    <div v-if="isLoading" class="space-y-3">
      <div v-for="n in 3" :key="n" class="animate-pulse rounded-lg border border-border p-4"><div class="h-4 w-3/4 rounded bg-muted mb-3"/><div class="h-2 w-full rounded bg-muted"/></div>
    </div>

    <div v-else-if="activeTab === 'active'">
      <div v-if="activeDownloads.length === 0" class="rounded-lg border border-border bg-card p-12 text-center">
        <svg class="size-16 mx-auto mb-4 text-muted-foreground/50" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="7 10 12 15 17 10"/><line x1="12" y1="15" x2="12" y2="3"/></svg>
        <h3 class="text-lg font-medium text-foreground">No active downloads</h3>
        <p class="text-sm text-muted-foreground mt-1">Start a new download to see it here</p>
      </div>
      <div v-else class="space-y-3">
        <div v-for="dl in activeDownloads" :key="dl.id" class="rounded-lg border border-border bg-card p-4">
          <div class="flex items-start justify-between gap-4">
            <div class="min-w-0 flex-1">
              <h3 class="text-sm font-medium text-foreground truncate">{{ dl.title }}</h3>
              <p class="text-xs text-muted-foreground mt-0.5">{{ dl.format }} &middot; {{ dl.quality }}</p>
            </div>
            <div class="flex items-center gap-2 shrink-0">
              <span :class="cn('rounded-full px-2 py-0.5 text-xs font-medium capitalize', dl.status === 'downloading' && 'bg-blue-500/10 text-blue-500', dl.status === 'processing' && 'bg-yellow-500/10 text-yellow-500', dl.status === 'queued' && 'bg-muted text-muted-foreground')">{{ dl.status }}</span>
              <button class="size-7 rounded-md text-muted-foreground hover:bg-accent flex items-center justify-center" @click="downloadsStore.removeDownload(dl.id)"><svg class="size-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg></button>
            </div>
          </div>
          <div class="mt-3">
            <div class="flex items-center justify-between text-xs text-muted-foreground mb-1"><span>{{ dl.progress }}%</span><span v-if="dl.speed">{{ formatBytes(dl.speed) }}/s</span><span v-if="dl.eta">ETA: {{ Math.floor(dl.eta / 60) }}:{{ (dl.eta % 60).toString().padStart(2, '0') }}</span></div>
            <div class="h-2 w-full rounded-full bg-muted overflow-hidden"><div :class="cn('h-full rounded-full transition-all', dl.status === 'downloading' && 'bg-blue-500', dl.status === 'processing' && 'bg-yellow-500 animate-pulse')" :style="{ width: `${dl.progress}%` }"/></div>
            <div v-if="dl.fileSize" class="mt-1 text-xs text-muted-foreground">{{ formatBytes(dl.downloadedBytes || 0) }} / {{ formatBytes(dl.fileSize) }}</div>
          </div>
        </div>
      </div>
    </div>

    <div v-else>
      <div v-if="completedDownloads.length === 0" class="rounded-lg border border-border bg-card p-12 text-center">
        <svg class="size-16 mx-auto mb-4 text-muted-foreground/50" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"/><polyline points="22 4 12 14.01 9 11.01"/></svg>
        <h3 class="text-lg font-medium text-foreground">No completed downloads</h3>
      </div>
      <div v-else class="space-y-2">
        <div v-for="dl in completedDownloads" :key="dl.id" class="flex items-center gap-3 rounded-lg border border-border bg-card p-3 group">
          <div class="size-8 rounded-full bg-green-500/10 flex items-center justify-center shrink-0"><svg class="size-4 text-green-500" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="20 6 9 17 4 12"/></svg></div>
          <div class="min-w-0 flex-1"><h3 class="text-sm font-medium text-foreground truncate">{{ dl.title }}</h3><p class="text-xs text-muted-foreground">{{ dl.format }} &middot; {{ formatBytes(dl.fileSize || 0) }}</p></div>
          <button class="size-7 rounded-md text-muted-foreground hover:bg-accent flex items-center justify-center opacity-0 group-hover:opacity-100 transition-opacity" @click="downloadsStore.removeDownload(dl.id)"><svg class="size-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="3 6 5 6 21 6"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/></svg></button>
        </div>
      </div>
    </div>

    <Teleport to="body">
      <div v-if="showDownloadForm" class="fixed inset-0 z-50 flex items-center justify-center bg-black/50" @click.self="showDownloadForm = false">
        <div class="w-full max-w-md rounded-lg bg-card border border-border p-6 shadow-xl">
          <h3 class="text-lg font-semibold text-foreground mb-4">New Download</h3>
          <div class="space-y-4">
            <div><label class="text-sm font-medium text-foreground">Video URL</label><input v-model="newDownload.url" type="url" placeholder="https://youtube.com/watch?v=..." class="mt-1 h-9 w-full rounded-md border border-input bg-background px-3 text-sm outline-none focus:border-primary focus:ring-1 focus:ring-primary"/></div>
            <div><label class="text-sm font-medium text-foreground">Format</label><select v-model="newDownload.format" class="mt-1 h-9 w-full rounded-md border border-input bg-background px-3 text-sm"><option value="video:best">Best Video</option><option value="video:720">720p Video</option><option value="video:1080">1080p Video</option><option value="audio:best">Audio Only</option></select></div>
            <div><label class="text-sm font-medium text-foreground">Quality</label><select v-model="newDownload.quality" class="mt-1 h-9 w-full rounded-md border border-input bg-background px-3 text-sm"><option value="auto">Auto</option><option value="360">360p</option><option value="720">720p</option><option value="1080">1080p</option></select></div>
            <div class="flex justify-end gap-2 pt-2">
              <button class="h-9 rounded-md border border-input bg-background px-4 text-sm font-medium text-foreground hover:bg-accent transition-colors" @click="showDownloadForm = false">Cancel</button>
              <button class="h-9 rounded-md bg-primary px-4 text-sm font-medium text-primary-foreground hover:bg-primary/90 transition-colors" @click="startNewDownload">Download</button>
            </div>
          </div>
        </div>
      </div>
    </Teleport>
  </div>
</template>

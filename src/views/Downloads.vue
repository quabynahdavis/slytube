<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { cn } from '@/lib/utils'
import { useDownloads } from '../composables/useData'
import SkeletonGrid from '../components/ui/SkeletonGrid.vue'
import EmptyState from '../components/ui/EmptyState.vue'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'

const { downloads, loadDownloads, startDownload, cancelDownload } = useDownloads()

const isLoading = ref(true)
const activeTab = ref<'active' | 'completed'>('active')
const showDownloadForm = ref(false)
const downloadUrl = ref('')
const downloadFormat = ref('video:best')
const isStarting = ref(false)
const startError = ref<string | null>(null)

const activeDownloads = computed(() =>
  downloads.value.filter((d: any) => d.status === 'downloading' || d.status === 'processing' || d.status === 'queued')
)

const completedDownloads = computed(() =>
  downloads.value.filter((d: any) => d.status === 'completed' || d.status === 'error')
)

onMounted(async () => {
  isLoading.value = true
  try {
    await loadDownloads()
  } finally {
    isLoading.value = false
  }
})

async function handleStartDownload() {
  if (!downloadUrl.value.trim()) return
  isStarting.value = true
  startError.value = null
  try {
    await startDownload({
      url: downloadUrl.value.trim(),
      format: downloadFormat.value,
    })
    downloadUrl.value = ''
    downloadFormat.value = 'video:best'
    showDownloadForm.value = false
    await loadDownloads()
  } catch (e: any) {
    startError.value = e.message || 'Failed to start download'
  } finally {
    isStarting.value = false
  }
}

async function handleCancelDownload(id: number) {
  try {
    await cancelDownload(id)
    await loadDownloads()
  } catch {
    // Silently handle cancel errors
  }
}

function clearCompleted() {
  downloads.value = downloads.value.filter((d: any) => d.status !== 'completed' && d.status !== 'error')
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
      <button v-if="activeTab === 'completed' && completedDownloads.length > 0" class="pb-3 text-sm text-muted-foreground hover:text-foreground transition-colors" @click="clearCompleted">Clear Completed</button>
    </div>

    <SkeletonGrid v-if="isLoading" :count="3" />

    <!-- Active Downloads -->
    <div v-else-if="activeTab === 'active'">
      <EmptyState v-if="activeDownloads.length === 0" title="No active downloads" action="Start Download" @action="showDownloadForm = true">
        Paste a video URL above and click "New Download" to get started.
      </EmptyState>
      <div v-else class="space-y-3">
        <div v-for="dl in activeDownloads" :key="dl.id" class="rounded-lg border border-border bg-card p-4">
          <div class="flex items-start justify-between gap-4">
            <div class="min-w-0 flex-1">
              <h3 class="text-sm font-medium text-foreground truncate">{{ dl.title || dl.videoId || 'Downloading...' }}</h3>
              <p class="text-xs text-muted-foreground mt-0.5">{{ dl.destination || dl.status }}</p>
            </div>
            <div class="flex items-center gap-2 shrink-0">
              <span :class="cn('rounded-full px-2 py-0.5 text-xs font-medium capitalize', dl.status === 'downloading' && 'bg-blue-500/10 text-blue-500', dl.status === 'processing' && 'bg-yellow-500/10 text-yellow-500', dl.status === 'queued' && 'bg-muted text-muted-foreground')">{{ dl.status }}</span>
              <button class="size-7 rounded-md text-muted-foreground hover:bg-accent flex items-center justify-center" @click="handleCancelDownload(dl.id)"><svg class="size-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg></button>
            </div>
          </div>
          <div class="mt-3">
            <div class="flex items-center justify-between text-xs text-muted-foreground mb-1"><span>{{ Math.round(dl.percent || 0) }}%</span><span v-if="dl.speed">{{ dl.speed }}</span><span v-if="dl.eta">ETA: {{ dl.eta }}</span></div>
            <div class="h-2 w-full rounded-full bg-muted overflow-hidden"><div :class="cn('h-full rounded-full transition-all', dl.status === 'downloading' && 'bg-blue-500', dl.status === 'processing' && 'bg-yellow-500 animate-pulse')" :style="{ width: `${dl.percent || 0}%` }"/></div>
            <div v-if="dl.errorMessage" class="mt-1 text-xs text-destructive">{{ dl.errorMessage }}</div>
          </div>
        </div>
      </div>
    </div>

    <!-- Completed Downloads -->
    <div v-else>
      <EmptyState v-if="completedDownloads.length === 0" title="No completed downloads">
        Videos you've downloaded will appear here for offline viewing.
      </EmptyState>
      <div v-else class="space-y-2">
        <div v-for="dl in completedDownloads" :key="dl.id" class="flex items-center gap-3 rounded-lg border border-border bg-card p-3 group">
          <div class="size-8 rounded-full bg-green-500/10 flex items-center justify-center shrink-0"><svg class="size-4 text-green-500" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="20 6 9 17 4 12"/></svg></div>
          <div class="min-w-0 flex-1"><h3 class="text-sm font-medium text-foreground truncate">{{ dl.title || 'Downloaded Video' }}</h3><p class="text-xs text-muted-foreground">{{ dl.destination || dl.videoId }}</p></div>
          <button class="size-7 rounded-md text-muted-foreground hover:bg-accent flex items-center justify-center opacity-0 group-hover:opacity-100 transition-opacity" @click="downloads = downloads.filter((d: any) => d.id !== dl.id)"><svg class="size-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="3 6 5 6 21 6"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/></svg></button>
        </div>
      </div>
    </div>

    <!-- Download Form Dialog -->
    <Teleport to="body">
      <div v-if="showDownloadForm" class="fixed inset-0 z-50 flex items-center justify-center bg-black/50" @click.self="showDownloadForm = false">
        <div class="w-full max-w-md rounded-lg bg-card border border-border p-6 shadow-xl">
          <h3 class="text-lg font-semibold text-foreground mb-4">New Download</h3>
          <div class="space-y-4">
            <div>
              <label class="text-sm font-medium text-foreground">Video URL</label>
              <input
                v-model="downloadUrl"
                type="url"
                placeholder="https://youtube.com/watch?v=..."
                class="mt-1 h-9 w-full rounded-md border border-input bg-background px-3 text-sm outline-none focus:border-primary focus:ring-1 focus:ring-primary"
              />
            </div>
            <div>
              <label class="text-sm font-medium text-foreground">Format</label>
              <Select v-model="downloadFormat" class="mt-1">
                <SelectTrigger class="w-full">
                  <SelectValue placeholder="Select format..." />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="video:best">Best Video</SelectItem>
                  <SelectItem value="video:720">720p Video</SelectItem>
                  <SelectItem value="video:1080">1080p Video</SelectItem>
                  <SelectItem value="audio:best">Audio Only</SelectItem>
                </SelectContent>
              </Select>
            </div>
            <div v-if="startError" class="rounded-md bg-destructive/10 p-3 text-sm text-destructive">
              {{ startError }}
            </div>
            <div class="flex justify-end gap-2 pt-2">
              <button class="h-9 rounded-md border border-input bg-background px-4 text-sm font-medium text-foreground hover:bg-accent transition-colors" @click="showDownloadForm = false">Cancel</button>
              <button :disabled="isStarting || !downloadUrl.trim()" class="h-9 rounded-md bg-primary px-4 text-sm font-medium text-primary-foreground hover:bg-primary/90 transition-colors disabled:opacity-50" @click="handleStartDownload">
                {{ isStarting ? 'Starting...' : 'Download' }}
              </button>
            </div>
          </div>
        </div>
      </div>
    </Teleport>
  </div>
</template>

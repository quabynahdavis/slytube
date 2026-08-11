<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { usePlaylists } from '../composables/useData'
import SkeletonGrid from '../components/ui/SkeletonGrid.vue'
import EmptyState from '../components/ui/EmptyState.vue'

const { playlists, loadPlaylists, createPlaylist, deletePlaylist } = usePlaylists()

const isLoading = ref(true)
const showCreateDialog = ref(false)
const newPlaylistName = ref('')
const newPlaylistDescription = ref('')

async function load() {
  isLoading.value = true
  try {
    await loadPlaylists()
  } finally {
    isLoading.value = false
  }
}

onMounted(load)

async function handleCreatePlaylist() {
  if (!newPlaylistName.value.trim()) return
  await createPlaylist(newPlaylistName.value.trim(), newPlaylistDescription.value.trim())
  newPlaylistName.value = ''
  newPlaylistDescription.value = ''
  showCreateDialog.value = false
}

async function handleDeletePlaylist(id: string) {
  await deletePlaylist(id)
}
</script>

<template>
  <div class="container mx-auto max-w-5xl px-4 py-6">
    <div class="flex items-center justify-between mb-6">
      <div>
        <h1 class="text-2xl font-bold text-foreground">Playlists</h1>
        <p class="text-sm text-muted-foreground mt-1">{{ playlists.length }} playlists</p>
      </div>
      <button class="inline-flex items-center gap-1 rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90 transition-colors" @click="showCreateDialog = true">
        <svg class="size-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/></svg>
        New Playlist
      </button>
    </div>

    <SkeletonGrid v-if="isLoading" :count="6" :columns="3" />

    <EmptyState v-else-if="playlists.length === 0" title="No playlists yet" action="Create Playlist" @action="showCreateDialog = true">
      Create playlists to organize and save your favorite videos.
    </EmptyState>

    <div v-else class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
      <div v-for="playlist in playlists" :key="(playlist as any).id || (playlist as any)._id" class="group relative rounded-lg border border-border bg-card overflow-hidden">
        <router-link :to="`/playlist/${(playlist as any).id || (playlist as any)._id}`" class="block">
          <div class="relative aspect-video bg-muted">
            <div class="absolute inset-0 flex items-center justify-center"><svg class="size-12 text-muted-foreground/50" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><polygon points="5 3 19 12 5 21 5 3"/></svg></div>
            <div class="absolute bottom-2 right-2 rounded bg-black/80 px-2 py-1 text-xs text-white font-medium">{{ (playlist as any).videoCount || (playlist as any).videos?.length || 0 }} videos</div>
          </div>
          <div class="p-3">
            <h3 class="text-sm font-medium text-foreground truncate group-hover:text-primary transition-colors">{{ (playlist as any).name || (playlist as any).playlistName }}</h3>
            <p class="text-xs text-muted-foreground mt-0.5 line-clamp-1">{{ (playlist as any).description || 'No description' }}</p>
          </div>
        </router-link>
        <button class="absolute top-2 right-2 size-7 rounded-md bg-black/50 text-white opacity-0 group-hover:opacity-100 flex items-center justify-center transition-opacity" @click="handleDeletePlaylist((playlist as any).id || (playlist as any)._id)">
          <svg class="size-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="3 6 5 6 21 6"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/></svg>
        </button>
      </div>
    </div>

    <!-- Create Dialog -->
    <Teleport to="body">
      <div v-if="showCreateDialog" class="fixed inset-0 z-50 flex items-center justify-center bg-black/50" @click.self="showCreateDialog = false">
        <div class="w-full max-w-md rounded-lg bg-card border border-border p-6 shadow-xl">
          <h3 class="text-lg font-semibold text-foreground mb-4">Create Playlist</h3>
          <div class="space-y-4">
            <div>
              <label class="text-sm font-medium text-foreground">Name</label>
              <input v-model="newPlaylistName" type="text" placeholder="Playlist name" class="mt-1 h-9 w-full rounded-md border border-input bg-background px-3 text-sm outline-none focus:border-primary focus:ring-1 focus:ring-primary"/>
            </div>
            <div>
              <label class="text-sm font-medium text-foreground">Description</label>
              <textarea v-model="newPlaylistDescription" placeholder="Optional description" rows="3" class="mt-1 w-full rounded-md border border-input bg-background px-3 py-2 text-sm outline-none focus:border-primary focus:ring-1 focus:ring-primary resize-none"/>
            </div>
            <div class="flex justify-end gap-2">
              <button class="h-9 rounded-md border border-input bg-background px-4 text-sm font-medium text-foreground hover:bg-accent transition-colors" @click="showCreateDialog = false">Cancel</button>
              <button class="h-9 rounded-md bg-primary px-4 text-sm font-medium text-primary-foreground hover:bg-primary/90 transition-colors" @click="handleCreatePlaylist">Create</button>
            </div>
          </div>
        </div>
      </div>
    </Teleport>
  </div>
</template>

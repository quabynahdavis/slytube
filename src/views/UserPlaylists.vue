<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { usePlaylistsStore } from '@/stores/playlists'

const playlistsStore = usePlaylistsStore()

const isLoading = ref(true)
const showCreateDialog = ref(false)
const newPlaylistName = ref('')
const newPlaylistDescription = ref('')

const playlists = computed(() => playlistsStore.getAllPlaylists)

onMounted(async () => {
  isLoading.value = true
  try {
    await new Promise((r) => setTimeout(r, 500))
    // Sample playlists
    const samples = [
      { name: 'Watch Later', description: 'Videos to watch later', protected: true },
      { name: 'Favorites', description: 'My favorite videos', protected: true },
      { name: 'Music', description: 'Music videos and songs', protected: false },
      { name: 'Tutorials', description: 'Educational content', protected: false },
    ]
    samples.forEach((p) => playlistsStore.createPlaylist(p.name, p.description))
  } finally {
    isLoading.value = false
  }
})

function createPlaylist() {
  if (!newPlaylistName.value.trim()) return
  playlistsStore.createPlaylist(newPlaylistName.value.trim(), newPlaylistDescription.value.trim())
  newPlaylistName.value = ''
  newPlaylistDescription.value = ''
  showCreateDialog.value = false
}

function deletePlaylist(id: string) {
  playlistsStore.deletePlaylist(id)
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

    <div v-if="isLoading" class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
      <div v-for="n in 6" :key="n" class="animate-pulse"><div class="aspect-video rounded-lg bg-muted"/><div class="mt-3 space-y-2"><div class="h-4 w-3/4 rounded bg-muted"/><div class="h-3 w-1/2 rounded bg-muted"/></div></div>
    </div>

    <div v-else-if="playlists.length === 0" class="rounded-lg border border-border bg-card p-12 text-center">
      <svg class="size-16 mx-auto mb-4 text-muted-foreground/50" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><rect x="2" y="2" width="20" height="20" rx="2.18"/><line x1="7" y1="2" x2="7" y2="22"/><line x1="17" y1="2" x2="17" y2="22"/><line x1="2" y1="12" x2="22" y2="12"/></svg>
      <h3 class="text-lg font-medium text-foreground">No playlists yet</h3>
      <p class="text-sm text-muted-foreground mt-1">Create a playlist to organize your videos</p>
    </div>

    <div v-else class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
      <div v-for="playlist in playlists" :key="playlist._id" class="group relative rounded-lg border border-border bg-card overflow-hidden">
        <router-link :to="`/playlist/${playlist._id}`" class="block">
          <div class="relative aspect-video bg-muted">
            <div class="absolute inset-0 flex items-center justify-center"><svg class="size-12 text-muted-foreground/50" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><polygon points="5 3 19 12 5 21 5 3"/></svg></div>
            <div class="absolute bottom-2 right-2 rounded bg-black/80 px-2 py-1 text-xs text-white font-medium">{{ playlist.videos.length }} videos</div>
          </div>
          <div class="p-3">
            <h3 class="text-sm font-medium text-foreground truncate group-hover:text-primary transition-colors">{{ playlist.playlistName }}</h3>
            <p class="text-xs text-muted-foreground mt-0.5 line-clamp-1">{{ playlist.description }}</p>
          </div>
        </router-link>
        <button v-if="!playlist.protected" class="absolute top-2 right-2 size-7 rounded-md bg-black/50 text-white opacity-0 group-hover:opacity-100 flex items-center justify-center transition-opacity" @click="deletePlaylist(playlist._id)">
          <svg class="size-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="3 6 5 6 21 6"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/></svg>
        </button>
      </div>
    </div>

    <Teleport to="body">
      <div v-if="showCreateDialog" class="fixed inset-0 z-50 flex items-center justify-center bg-black/50" @click.self="showCreateDialog = false">
        <div class="w-full max-w-md rounded-lg bg-card border border-border p-6 shadow-xl">
          <h3 class="text-lg font-semibold text-foreground mb-4">Create Playlist</h3>
          <div class="space-y-4">
            <div><label class="text-sm font-medium text-foreground">Name</label><input v-model="newPlaylistName" type="text" placeholder="Playlist name" class="mt-1 h-9 w-full rounded-md border border-input bg-background px-3 text-sm outline-none focus:border-primary focus:ring-1 focus:ring-primary"/></div>
            <div><label class="text-sm font-medium text-foreground">Description</label><textarea v-model="newPlaylistDescription" placeholder="Optional description" rows="3" class="mt-1 w-full rounded-md border border-input bg-background px-3 py-2 text-sm outline-none focus:border-primary focus:ring-1 focus:ring-primary resize-none"/></div>
            <div class="flex justify-end gap-2">
              <button class="h-9 rounded-md border border-input bg-background px-4 text-sm font-medium text-foreground hover:bg-accent transition-colors" @click="showCreateDialog = false">Cancel</button>
              <button class="h-9 rounded-md bg-primary px-4 text-sm font-medium text-primary-foreground hover:bg-primary/90 transition-colors" @click="createPlaylist">Create</button>
            </div>
          </div>
        </div>
      </div>
    </Teleport>
  </div>
</template>

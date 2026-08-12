import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'

export interface PlaylistVideo {
  videoId: string
  title: string
  author: string
  authorId: string
  lengthSeconds: number
  timeAdded: number
  playlistItemId: string
  type: string
  videoThumbnails: Array<{ url: string; width: number; height: number }>
}

export interface Playlist {
  _id: string
  playlistName: string
  description: string
  videos: PlaylistVideo[]
  protected: boolean
  createdAt: number
  lastUpdatedAt: number
  lastPlayedAt?: number
}

export interface PlaylistsState {
  playlistsReady: boolean
  playlists: Playlist[]
  playlistVideoCounts: Map<string, number>
}

interface DbPlaylist {
  id: string
  profile_id: string
  name: string
  description: string | null
  created_at: string
}

const DEFAULT_PROFILE_ID = 'default'

function mapDbPlaylistToPlaylist(db: DbPlaylist): Playlist {
  const createdAt = new Date(db.created_at).getTime()
  return {
    _id: db.id,
    playlistName: db.name,
    description: db.description ?? '',
    videos: [],
    protected: db.name === 'Favorites',
    createdAt,
    lastUpdatedAt: createdAt,
  }
}

export const usePlaylistsStore = defineStore('playlists', {
  state: (): PlaylistsState => ({
    playlistsReady: false,
    playlists: [],
    playlistVideoCounts: new Map(),
  }),

  getters: {
    getPlaylistsReady: (state) => state.playlistsReady,
    getAllPlaylists: (state) => state.playlists,
    getPlaylistVideoCounts: (state) => state.playlistVideoCounts,

    getPlaylist: (state) => (playlistId: string) => {
      return state.playlists.find((playlist) => playlist._id === playlistId)
    },

    getQuickBookmarkPlaylist: (state) => {
      return state.playlists.find((p) => p.protected) ?? state.playlists.find((p) => p.playlistName === 'Favorites')
    },
  },

  actions: {
    async loadPlaylists() {
      try {
        const playlists = await invoke<DbPlaylist[]>('db_playlists_find_all', {
          profileId: DEFAULT_PROFILE_ID,
        })
        this.playlists = playlists.map(mapDbPlaylistToPlaylist)
        this.playlistsReady = true
      } catch {
        // Database unavailable, keep existing state
      }
    },

    async createPlaylist(name: string, description: string) {
      const trimmedName = name.trim()
      const now = new Date().toISOString()
      const id = `ft-playlist--${Date.now()}-${Math.floor(Math.random() * 10000)}`

      const dbPlaylist: DbPlaylist = {
        id,
        profile_id: DEFAULT_PROFILE_ID,
        name: trimmedName,
        description: description.trim() || null,
        created_at: now,
      }

      try {
        await invoke('db_playlists_create', { playlist: dbPlaylist })
      } catch {
        // Database unavailable, fall through to in-memory only
      }

      const playlist = mapDbPlaylistToPlaylist(dbPlaylist)
      this.playlists.push(playlist)
      return playlist
    },

    async deletePlaylist(id: string) {
      const index = this.playlists.findIndex((p) => p._id === id)
      if (index !== -1 && !this.playlists[index].protected) {
        try {
          await invoke('db_playlists_delete', { id })
        } catch {
          // Database unavailable, fall through to in-memory only
        }
        this.playlists.splice(index, 1)
      }
    },

    async addToPlaylist(playlistId: string, videoId: string) {
      const playlist = this.playlists.find((p) => p._id === playlistId)
      if (!playlist) return

      const position = playlist.videos.length

      try {
        await invoke('db_playlists_add_video', { playlistId, videoId, position })
      } catch {
        // Database unavailable, fall through to in-memory only
      }

      playlist.lastUpdatedAt = Date.now()
    },

    async removeFromPlaylist(playlistId: string, videoId: string) {
      const playlist = this.playlists.find((p) => p._id === playlistId)
      if (!playlist) return

      try {
        await invoke('db_playlists_remove_video', { playlistId, videoId })
      } catch {
        // Database unavailable, fall through to in-memory only
      }

      playlist.videos = playlist.videos.filter((v) => v.videoId !== videoId)
      playlist.lastUpdatedAt = Date.now()
    },

    async getQuickBookmarkPlaylist(): Promise<Playlist | undefined> {
      const existing = this.playlists.find((p) => p.protected) ?? this.playlists.find((p) => p.playlistName === 'Favorites')
      if (existing) return existing

      return this.createPlaylist('Favorites', 'Quick bookmark playlist')
    },

    setPlaylistsReady(ready: boolean) {
      this.playlistsReady = ready
    },
  },
})

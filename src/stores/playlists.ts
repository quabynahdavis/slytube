import { defineStore } from 'pinia'

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

    getQuickBookmarkPlaylist: () => {
      // Placeholder - would reference settings store
      return undefined
    },
  },

  actions: {
    loadPlaylists() {
      // Placeholder for DB integration
    },

    createPlaylist(name: string, description: string) {
      const now = Date.now()
      const playlist: Playlist = {
        _id: `ft-playlist--${now}-${Math.floor(Math.random() * 10000)}`,
        playlistName: name.trim(),
        description: description.trim(),
        videos: [],
        protected: false,
        createdAt: now,
        lastUpdatedAt: now,
      }
      this.playlists.push(playlist)
      return playlist
    },

    deletePlaylist(id: string) {
      const index = this.playlists.findIndex((p) => p._id === id)
      if (index !== -1 && !this.playlists[index].protected) {
        this.playlists.splice(index, 1)
      }
    },

    addToPlaylist(playlistId: string, video: PlaylistVideo) {
      const playlist = this.playlists.find((p) => p._id === playlistId)
      if (playlist) {
        playlist.videos.push(video)
        playlist.lastUpdatedAt = Date.now()
      }
    },

    removeFromPlaylist(playlistId: string, videoId: string) {
      const playlist = this.playlists.find((p) => p._id === playlistId)
      if (playlist) {
        playlist.videos = playlist.videos.filter((v) => v.videoId !== videoId)
        playlist.lastUpdatedAt = Date.now()
      }
    },

    setPlaylistsReady(ready: boolean) {
      this.playlistsReady = ready
    },
  },
})

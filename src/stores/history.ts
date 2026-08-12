import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'

export interface HistoryEntry {
  videoId: string
  title: string
  author: string
  authorId: string
  authorUrl: string
  description: string
  viewCount: number
  lengthSeconds: number
  timeWatched: number
  watchProgress: number
  isWatched: boolean
  lastViewedPlaylistId?: string
  lastViewedPlaylistType?: string
  lastViewedPlaylistItemId?: string
  type: string
  videoThumbnails: Array<{ url: string; width: number; height: number }>
}

export interface HistoryState {
  historyCacheSorted: HistoryEntry[]
  historyCacheById: Record<string, HistoryEntry>
}

export const useHistoryStore = defineStore('history', {
  state: (): HistoryState => ({
    historyCacheSorted: [],
    historyCacheById: {},
  }),

  getters: {
    getHistoryCacheSorted: (state) => state.historyCacheSorted,
    getHistoryCacheById: (state) => state.historyCacheById,

    getHistoryByAuthor: (state) => (authorId: string) => {
      return state.historyCacheSorted.filter((entry) => entry.authorId === authorId)
    },

    getWatchProgress: (state) => (videoId: string) => {
      return state.historyCacheById[videoId]?.watchProgress ?? 0
    },
  },

  actions: {
    async loadHistory() {
      try {
        const entries = await invoke<HistoryEntry[]>('db_history_find_all', { limit: 100 })
        this.historyCacheSorted = entries
        this.historyCacheById = {}
        for (const entry of entries) {
          this.historyCacheById[entry.videoId] = entry
        }
      } catch {
        // Database unavailable, keep in-memory state
      }
    },

    async addToHistory(entry: HistoryEntry) {
      const i = this.historyCacheSorted.findIndex(
        (currentRecord) => entry.videoId === currentRecord.videoId
      )

      if (i !== -1) {
        const currentRecord = this.historyCacheSorted[i]

        if (entry.timeWatched === currentRecord.timeWatched) {
          this.historyCacheSorted.splice(i, 1, entry)
          this.historyCacheById[entry.videoId] = entry
          return
        }

        this.historyCacheSorted.splice(i, 1)
      }

      this.historyCacheSorted.unshift(entry)
      this.historyCacheById[entry.videoId] = entry

      try {
        await invoke('db_history_upsert', { entry })
      } catch {
        // Database unavailable, change is in-memory only
      }
    },

    async removeFromHistory(videoId: string) {
      for (let i = 0; i < this.historyCacheSorted.length; i++) {
        if (this.historyCacheSorted[i].videoId === videoId) {
          this.historyCacheSorted.splice(i, 1)
          break
        }
      }
      delete this.historyCacheById[videoId]

      try {
        await invoke('db_history_delete', { videoId })
      } catch {
        // Database unavailable, change is in-memory only
      }
    },

    async clearHistory() {
      this.historyCacheSorted = []
      this.historyCacheById = {}

      try {
        await invoke('db_history_clear')
      } catch {
        // Database unavailable, change is in-memory only
      }
    },

    updateWatchProgress(videoId: string, progress: number) {
      const record = this.historyCacheById[videoId]
      if (record) {
        record.watchProgress = progress
      }
    },

    updateLastViewedPlaylist(
      videoId: string,
      lastViewedPlaylistId: string,
      lastViewedPlaylistType?: string,
      lastViewedPlaylistItemId?: string
    ) {
      const record = this.historyCacheById[videoId]
      if (record) {
        record.lastViewedPlaylistId = lastViewedPlaylistId
        record.lastViewedPlaylistType = lastViewedPlaylistType
        record.lastViewedPlaylistItemId = lastViewedPlaylistItemId
      }
    },
  },
})

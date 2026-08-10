import { defineStore } from 'pinia'

export interface SyncServerState {
  syncServerStatus: 'idle' | 'syncing' | 'success' | 'error'
  syncServerProgress: { stage: string; percentage: number } | null
  syncServerError: string
  syncServerLastResult: Record<string, number> | null
  syncServerHistorySupported: boolean | null
  syncServerPlaybackSpeedsSupported: boolean | null
  syncServerSessionExpired: boolean
}

export const useSyncStore = defineStore('sync', {
  state: (): SyncServerState => ({
    syncServerStatus: 'idle',
    syncServerProgress: null,
    syncServerError: '',
    syncServerLastResult: null,
    syncServerHistorySupported: null,
    syncServerPlaybackSpeedsSupported: null,
    syncServerSessionExpired: false,
  }),

  getters: {
    getSyncServerStatus: (state) => state.syncServerStatus,
    getSyncServerProgress: (state) => state.syncServerProgress,
    getSyncServerError: (state) => state.syncServerError,
    getSyncServerLastResult: (state) => state.syncServerLastResult,
    getSyncServerHistorySupported: (state) => state.syncServerHistorySupported,
    getSyncServerPlaybackSpeedsSupported: (state) => state.syncServerPlaybackSpeedsSupported,
  },

  actions: {
    setSyncServerStatus(status: SyncServerState['syncServerStatus']) {
      this.syncServerStatus = status
    },

    setSyncServerProgress(progress: SyncServerState['syncServerProgress']) {
      this.syncServerProgress = progress
    },

    setSyncServerError(error: string) {
      this.syncServerError = error
    },

    setSyncServerLastResult(result: Record<string, number> | null) {
      this.syncServerLastResult = result
    },

    setSyncServerHistorySupported(supported: boolean) {
      this.syncServerHistorySupported = supported
    },

    setSyncServerPlaybackSpeedsSupported(supported: boolean) {
      this.syncServerPlaybackSpeedsSupported = supported
    },

    setSyncServerSessionExpired(expired: boolean) {
      this.syncServerSessionExpired = expired
    },

    async authenticateSyncServer(_options: {
      mode: 'login' | 'register'
      serverUrl: string
      username: string
      password: string
      privacyPassphrase?: string
    }) {
      // Placeholder for sync server authentication
    },

    async disconnectSyncServer() {
      this.syncServerError = ''
      this.syncServerLastResult = null
      this.syncServerHistorySupported = null
      this.syncServerPlaybackSpeedsSupported = null
      this.syncServerProgress = null
      this.syncServerStatus = 'idle'
      this.syncServerSessionExpired = false
    },

    async syncWithSyncServer() {
      // Placeholder for sync server sync
    },

    async expireSyncServerSession() {
      this.syncServerProgress = null
      this.syncServerError = 'Session expired. Please sign in again.'
      this.syncServerSessionExpired = true
      this.syncServerStatus = 'error'
    },
  },
})

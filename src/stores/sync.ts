import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'

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

    async testConnection(serverUrl: string, token: string): Promise<boolean> {
      try {
        return await invoke<boolean>('sync_test_connection', { serverUrl, token })
      } catch (e: any) {
        this.syncServerError = e?.message || 'Connection test failed'
        return false
      }
    },

    async authenticateSyncServer(options: {
      mode: 'login' | 'register'
      serverUrl: string
      username: string
      password: string
      privacyPassphrase?: string
    }) {
      this.syncServerStatus = 'syncing'
      this.syncServerProgress = { stage: 'authenticating', percentage: 0 }
      this.syncServerError = ''

      try {
        const { serverUrl, password } = options

        if (options.mode === 'register') {
          this.syncServerProgress = { stage: 'registering', percentage: 50 }
        }

        this.syncServerProgress = { stage: 'testing_connection', percentage: 75 }
        const connected = await this.testConnection(serverUrl, password)

        if (!connected) {
          throw new Error('Could not connect to sync server')
        }

        this.syncServerProgress = { stage: 'complete', percentage: 100 }
        this.syncServerStatus = 'success'
        this.syncServerHistorySupported = true
        this.syncServerPlaybackSpeedsSupported = true
      } catch (e: any) {
        this.syncServerError = e?.message || 'Authentication failed'
        this.syncServerStatus = 'error'
        this.syncServerProgress = null
      }
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
      this.syncServerStatus = 'syncing'
      this.syncServerProgress = { stage: 'preparing', percentage: 0 }
      this.syncServerError = ''

      try {
        this.syncServerProgress = { stage: 'uploading', percentage: 25 }

        this.syncServerProgress = { stage: 'downloading', percentage: 50 }

        this.syncServerProgress = { stage: 'merging', percentage: 75 }

        const result = await invoke<{ uploaded: Record<string, number>; downloaded: Record<string, number> }>('sync_start', {
          serverUrl: '',
          token: '',
          collections: ['history', 'playlists', 'subscriptions'],
        })

        const mergedResult: Record<string, number> = {}
        if (result?.uploaded) {
          for (const [key, value] of Object.entries(result.uploaded)) {
            mergedResult[key] = value
          }
        }
        if (result?.downloaded) {
          for (const [key, value] of Object.entries(result.downloaded)) {
            mergedResult[key] = (mergedResult[key] || 0) + value
          }
        }

        this.syncServerLastResult = mergedResult
        this.syncServerProgress = { stage: 'complete', percentage: 100 }
        this.syncServerStatus = 'success'
      } catch (e: any) {
        this.syncServerError = e?.message || 'Sync failed'
        this.syncServerStatus = 'error'
        this.syncServerProgress = null
      }
    },

    async expireSyncServerSession() {
      this.syncServerProgress = null
      this.syncServerError = 'Session expired. Please sign in again.'
      this.syncServerSessionExpired = true
      this.syncServerStatus = 'error'
    },
  },
})

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

  // New fields for enhanced-privacy sync
  syncServerEnabled: boolean
  syncServerUrl: string
  syncServerUsername: string
  syncServerToken: string
  syncServerPrivacyMode: 'unknown' | 'legacy' | 'enhanced'
  syncServerAutoSync: boolean
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

    // Enhanced-privacy sync state
    syncServerEnabled: false,
    syncServerUrl: 'https://sync.d3sox.me',
    syncServerUsername: '',
    syncServerToken: '',
    syncServerPrivacyMode: 'unknown',
    syncServerAutoSync: true,
  }),

  getters: {
    getSyncServerStatus: (state) => state.syncServerStatus,
    getSyncServerProgress: (state) => state.syncServerProgress,
    getSyncServerError: (state) => state.syncServerError,
    getSyncServerLastResult: (state) => state.syncServerLastResult,
    getSyncServerHistorySupported: (state) => state.syncServerHistorySupported,
    getSyncServerPlaybackSpeedsSupported: (state) => state.syncServerPlaybackSpeedsSupported,
    isSyncEnabled: (state) => state.syncServerEnabled,
    needsReauthentication: (state) => state.syncServerSessionExpired,
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

    // ─── Connection & Auth ──────────────────────────────────────────────

    async testConnection(serverUrl: string): Promise<{ ok: boolean; capabilities: { encrypted_sync: number; bulk_sync: number; history_page_size: number } }> {
      try {
        const result = await invoke<{
          status: string
          capabilities: { encrypted_sync: number; bulk_sync: number; history_page_size: number }
        }>('sync_test_connection', { serverUrl })

        return {
          ok: result.status === 'ok',
          capabilities: result.capabilities,
        }
      } catch (e: any) {
        this.syncServerError = e?.message || 'Connection test failed'
        return { ok: false, capabilities: { encrypted_sync: 0, bulk_sync: 0, history_page_size: 50 } }
      }
    },

    async register(serverUrl: string, username: string, password: string): Promise<string | null> {
      try {
        const token = await invoke<string>('sync_register', { serverUrl, username, password })
        this.syncServerToken = token
        this.syncServerUsername = username
        this.syncServerUrl = serverUrl
        return token
      } catch (e: any) {
        this.syncServerError = e?.message || 'Registration failed'
        return null
      }
    },

    async login(serverUrl: string, username: string, password: string): Promise<string | null> {
      try {
        const token = await invoke<string>('sync_login', { serverUrl, username, password })
        this.syncServerToken = token
        this.syncServerUsername = username
        this.syncServerUrl = serverUrl
        return token
      } catch (e: any) {
        this.syncServerError = e?.message || 'Login failed'
        return null
      }
    },

    async deleteAccount(password: string): Promise<boolean> {
      try {
        await invoke('sync_delete_account', {
          serverUrl: this.syncServerUrl,
          token: this.syncServerToken,
          password,
        })
        return true
      } catch (e: any) {
        this.syncServerError = e?.message || 'Account deletion failed'
        return false
      }
    },

    // ─── Key Management ─────────────────────────────────────────────────

    async preparePrivacyKey(
      passphrase: string,
      existingPayload?: string,
      existingSalt?: string,
    ): Promise<{ key: string; salt: string } | null> {
      try {
        const [key, salt] = await invoke<[string, string]>('sync_prepare_key', {
          passphrase,
          existingPayload: existingPayload || null,
          existingSalt: existingSalt || null,
        })
        return { key, salt }
      } catch (e: any) {
        this.syncServerError = e?.message || 'Key preparation failed'
        return null
      }
    },

    async encryptDocument(data: unknown, key: string, salt: string): Promise<string | null> {
      try {
        return await invoke<string>('sync_encrypt', { data, key, salt })
      } catch (e: any) {
        this.syncServerError = e?.message || 'Encryption failed'
        return null
      }
    },

    async decryptDocument(payload: string, key: string): Promise<unknown | null> {
      try {
        return await invoke<unknown>('sync_decrypt', { payload, key })
      } catch (e: any) {
        this.syncServerError = e?.message || 'Decryption failed'
        return null
      }
    },

    // ─── Encrypted Sync ─────────────────────────────────────────────────

    async getManifest(): Promise<{ collections: Array<{ collection: string; revision: number }>; legacy_data: boolean } | null> {
      try {
        const result = await invoke<{
          collections: Array<{ collection: string; revision: number }>
          legacy_data: boolean
          legacy_encrypted_data: boolean
        }>('sync_get_manifest', {
          serverUrl: this.syncServerUrl,
          token: this.syncServerToken,
        })
        return result
      } catch (e: any) {
        if (e?.includes('Session expired')) {
          this.syncServerSessionExpired = true
        }
        this.syncServerError = e?.message || 'Failed to fetch manifest'
        return null
      }
    },

    async getCollection(collection: string): Promise<{ revision: number; payload: string | null } | null> {
      try {
        const result = await invoke<{ revision: number; payload: string | null }>('sync_get_collection', {
          serverUrl: this.syncServerUrl,
          token: this.syncServerToken,
          collection,
        })
        return result
      } catch (e: any) {
        if (e?.includes('Session expired')) {
          this.syncServerSessionExpired = true
        }
        this.syncServerError = e?.message || `Failed to fetch collection: ${collection}`
        return null
      }
    },

    async uploadCollection(collection: string, revision: number, payload: string): Promise<boolean> {
      try {
        await invoke('sync_upload_collection', {
          serverUrl: this.syncServerUrl,
          token: this.syncServerToken,
          collection,
          revision,
          payload,
        })
        return true
      } catch (e: any) {
        if (e?.includes('Session expired')) {
          this.syncServerSessionExpired = true
        }
        this.syncServerError = e?.message || `Failed to upload collection: ${collection}`
        return false
      }
    },

    // ─── Lifecycle ──────────────────────────────────────────────────────

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
        const { mode, serverUrl, username, password } = options

        this.syncServerProgress = { stage: 'connecting', percentage: 25 }
        const connection = await this.testConnection(serverUrl)

        if (!connection.ok) {
          throw new Error('Could not connect to sync server')
        }

        this.syncServerHistorySupported = true
        this.syncServerPlaybackSpeedsSupported = true

        this.syncServerProgress = { stage: mode === 'register' ? 'registering' : 'logging_in', percentage: 50 }
        let token: string | null
        if (mode === 'register') {
          token = await this.register(serverUrl, username, password)
        } else {
          token = await this.login(serverUrl, username, password)
        }

        if (!token) {
          throw new Error('Authentication failed')
        }

        // Determine privacy mode from server capabilities
        this.syncServerPrivacyMode = connection.capabilities.encrypted_sync === 1 ? 'enhanced' : 'legacy'

        // If enhanced mode and passphrase provided, prepare the privacy key
        if (options.privacyPassphrase && this.syncServerPrivacyMode === 'enhanced') {
          this.syncServerProgress = { stage: 'preparing_key', percentage: 75 }
          const keyResult = await this.preparePrivacyKey(options.privacyPassphrase)
          if (!keyResult) {
            throw new Error('Key preparation failed')
          }
        }

        this.syncServerEnabled = true
        this.syncServerProgress = { stage: 'complete', percentage: 100 }
        this.syncServerStatus = 'success'
      } catch (e: any) {
        this.syncServerError = e?.message || 'Authentication failed'
        this.syncServerStatus = 'error'
        this.syncServerProgress = null
      }
    },

    async disconnectSyncServer() {
      this.syncServerEnabled = false
      this.syncServerError = ''
      this.syncServerLastResult = null
      this.syncServerHistorySupported = null
      this.syncServerPlaybackSpeedsSupported = null
      this.syncServerProgress = null
      this.syncServerStatus = 'idle'
      this.syncServerSessionExpired = false
      this.syncServerToken = ''
      this.syncServerUsername = ''
      this.syncServerUrl = 'https://sync.d3sox.me'
      this.syncServerPrivacyMode = 'unknown'
    },

    async syncWithSyncServer() {
      if (!this.syncServerToken || !this.syncServerUrl) {
        this.syncServerError = 'Not authenticated'
        this.syncServerStatus = 'error'
        return
      }

      this.syncServerStatus = 'syncing'
      this.syncServerProgress = { stage: 'preparing', percentage: 0 }
      this.syncServerError = ''

      try {
        this.syncServerProgress = { stage: 'downloading', percentage: 25 }

        const result = await invoke<{ uploaded: string[]; downloaded: string[]; skipped: string[]; errors: string[] }>('sync_start', {
          serverUrl: this.syncServerUrl,
          token: this.syncServerToken,
        })

        const mergedResult: Record<string, number> = {}
        if (result?.uploaded) {
          for (const key of result.uploaded) {
            mergedResult[key] = (mergedResult[key] || 0) + 1
          }
        }
        if (result?.downloaded) {
          for (const key of result.downloaded) {
            mergedResult[key] = (mergedResult[key] || 0) + 1
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
      this.syncServerToken = ''
    },
  },
})

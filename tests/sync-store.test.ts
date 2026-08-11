import { describe, it, expect, beforeEach } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { useSyncStore } from '@/stores/sync'

describe('Sync Store', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  it('initializes with idle status', () => {
    const store = useSyncStore()
    expect(store.syncServerStatus).toBe('idle')
  })

  it('can set sync status', () => {
    const store = useSyncStore()
    store.setSyncServerStatus('syncing')
    expect(store.syncServerStatus).toBe('syncing')
  })

  it('can set sync progress', () => {
    const store = useSyncStore()
    store.setSyncServerProgress({ stage: 'uploading', percentage: 50 })
    expect(store.syncServerProgress).toEqual({ stage: 'uploading', percentage: 50 })
  })

  it('can set sync error', () => {
    const store = useSyncStore()
    store.setSyncServerError('Connection failed')
    expect(store.syncServerError).toBe('Connection failed')
  })

  it('can disconnect sync server', () => {
    const store = useSyncStore()
    store.setSyncServerStatus('success')
    store.setSyncServerError('some error')
    store.disconnectSyncServer()
    expect(store.syncServerStatus).toBe('idle')
    expect(store.syncServerError).toBe('')
    expect(store.syncServerHistorySupported).toBeNull()
  })

  it('can expire sync session', () => {
    const store = useSyncStore()
    store.expireSyncServerSession()
    expect(store.syncServerStatus).toBe('error')
    expect(store.syncServerSessionExpired).toBe(true)
    expect(store.syncServerError).toContain('Session expired')
  })

  it('can set last sync result', () => {
    const store = useSyncStore()
    store.setSyncServerLastResult({ history: 10, playlists: 5 })
    expect(store.syncServerLastResult).toEqual({ history: 10, playlists: 5 })
  })
})

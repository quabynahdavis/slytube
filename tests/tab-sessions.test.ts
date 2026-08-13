import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'

// Mock the Tauri invoke function
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}))

import { invoke } from '@tauri-apps/api/core'
import { useTabsStore } from '@/stores/tabs'

describe('Tab Session Persistence', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
  })

  afterEach(() => {
    vi.restoreAllMocks()
  })

  it('serializes tab session to JSON with correct schema', async () => {
    const store = useTabsStore()

    // Create a tab with some state
    store.createTab({
      name: 'watch',
      path: '/watch',
      params: {},
      query: { v: 'abc123' },
      hash: '',
      fullPath: '/watch?v=abc123',
    })
    const tabId = store.activeTabId!
    store.setTabContentTitle(tabId, 'Test Video Title')
    store.setTabPinned(tabId, true)
    store.setTabColor(tabId, '#ff0000')

    // Save triggers invoke with serialized JSON
    await store.saveTabs()

    expect(invoke).toHaveBeenCalledWith('db_tab_sessions_save', expect.objectContaining({
      data: expect.any(String),
    }))

    // Extract the serialized data
    const callArgs = vi.mocked(invoke).mock.calls[0][1] as { data: string }
    const parsed = JSON.parse(callArgs.data)

    expect(parsed).toHaveProperty('tabs')
    expect(parsed).toHaveProperty('activeTabId')
    expect(parsed).toHaveProperty('updatedAt')
    expect(parsed.tabs).toHaveLength(1)
    expect(parsed.tabs[0]).toEqual({
      id: tabId,
      url: '/watch?v=abc123',
      title: 'Test Video Title',
      isPinned: true,
      color: '#ff0000',
    })
    expect(parsed.activeTabId).toBe(tabId)
    expect(typeof parsed.updatedAt).toBe('number')
  })

  it('restores tabs from a serialized session', async () => {
    const mockInvoke = vi.mocked(invoke)
    mockInvoke.mockResolvedValue({
      data: JSON.stringify({
        tabs: [
          {
            id: 'tab-1',
            url: '/watch?v=xyz789',
            title: 'Restored Video',
            isPinned: true,
            color: '#00ff00',
          },
          {
            id: 'tab-2',
            url: '/feed/subscriptions',
            title: 'Subscriptions',
            isPinned: false,
            color: null,
          },
        ],
        activeTabId: 'tab-2',
        updatedAt: 1234567890,
      }),
    })

    const store = useTabsStore()
    await store.restoreTabs()

    expect(store.tabs).toHaveLength(2)
    expect(store.activeTabId).toBe('tab-2')
    expect(store.tabs[0].id).toBe('tab-1')
    expect(store.tabs[0].route.fullPath).toBe('/watch?v=xyz789')
    expect(store.tabs[0].contentTitle).toBe('Restored Video')
    expect(store.tabs[0].isPinned).toBe(true)
    expect(store.tabs[0].color).toBe('#00ff00')
    expect(store.tabs[1].id).toBe('tab-2')
    expect(store.tabs[1].route.fullPath).toBe('/feed/subscriptions')
  })

  it('handles gracefully when no saved session exists', async () => {
    const mockInvoke = vi.mocked(invoke)
    mockInvoke.mockResolvedValue(null)

    const store = useTabsStore()
    store.createTab({
      name: null,
      path: '/',
      params: {},
      query: {},
      hash: '',
      fullPath: '/',
    })

    await store.restoreTabs()

    // Should keep existing tabs since there's nothing to restore
    expect(store.tabs).toHaveLength(1)
    expect(store.activeTabId).not.toBeNull()
  })

  it('handles gracefully when invoke throws', async () => {
    const mockInvoke = vi.mocked(invoke)
    mockInvoke.mockRejectedValue(new Error('Database unavailable'))

    const store = useTabsStore()
    store.createTab({
      name: null,
      path: '/',
      params: {},
      query: {},
      hash: '',
      fullPath: '/',
    })

    // Should not throw
    await expect(store.restoreTabs()).resolves.toBeUndefined()

    // Tabs should remain unchanged
    expect(store.tabs).toHaveLength(1)
  })

  it('does not throw on malformed JSON during restore', async () => {
    const mockInvoke = vi.mocked(invoke)
    mockInvoke.mockResolvedValue({
      data: 'not valid json {{{',
    })

    const store = useTabsStore()
    store.createTab({
      name: null,
      path: '/',
      params: {},
      query: {},
      hash: '',
      fullPath: '/',
    })

    // Should not throw, just skip restoration
    await expect(store.restoreTabs()).resolves.toBeUndefined()

    // Original tab should remain
    expect(store.tabs).toHaveLength(1)
  })

  it('falls back to first tab when activeTabId is not in restored tabs', async () => {
    const mockInvoke = vi.mocked(invoke)
    mockInvoke.mockResolvedValue({
      data: JSON.stringify({
        tabs: [
          {
            id: 'tab-1',
            url: '/watch?v=abc',
            title: 'Video',
            isPinned: false,
            color: null,
          },
        ],
        activeTabId: 'nonexistent-tab',
        updatedAt: 1234567890,
      }),
    })

    const store = useTabsStore()
    await store.restoreTabs()

    // Should fall back to first tab since 'nonexistent-tab' doesn't exist
    expect(store.activeTabId).toBe('tab-1')
  })

  it('skips restoration when session has empty tabs array', async () => {
    const mockInvoke = vi.mocked(invoke)
    mockInvoke.mockResolvedValue({
      data: JSON.stringify({
        tabs: [],
        activeTabId: null,
        updatedAt: 1234567890,
      }),
    })

    const store = useTabsStore()
    store.createTab({
      name: null,
      path: '/',
      params: {},
      query: {},
      hash: '',
      fullPath: '/',
    })

    await store.restoreTabs()

    // Should keep existing state since there are no tabs to restore
    expect(store.tabs).toHaveLength(1)
  })

  it('falls back to route fullPath when contentTitle is empty', async () => {
    const store = useTabsStore()

    store.createTab({
      name: 'watch',
      path: '/watch',
      params: {},
      query: { v: 'abc123' },
      hash: '',
      fullPath: '/watch?v=abc123',
    })
    const tabId = store.activeTabId!
    // Don't set contentTitle — leave it empty

    await store.saveTabs()

    const callArgs = vi.mocked(invoke).mock.calls[0][1] as { data: string }
    const parsed = JSON.parse(callArgs.data)

    // Title should fall back to fullPath when contentTitle is empty
    expect(parsed.tabs[0].title).toBe('/watch?v=abc123')
  })
})

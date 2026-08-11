import { describe, it, expect, beforeEach } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { useSettingsStore } from '@/stores/settings'

describe('Settings Store', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  it('has default pinned quick access items', () => {
    const store = useSettingsStore()
    expect(store.pinnedQuickAccess).toContain('baseTheme')
    expect(store.pinnedQuickAccess).toContain('enableNotifications')
    expect(store.pinnedQuickAccess).toContain('rememberHistory')
    expect(store.pinnedQuickAccess).toContain('autoplayVideos')
  })

  it('can pin a setting to quick access', () => {
    const store = useSettingsStore()
    store.pinToQuickAccess('useSponsorBlock')
    expect(store.pinnedQuickAccess).toContain('useSponsorBlock')
  })

  it('can unpin a setting from quick access', () => {
    const store = useSettingsStore()
    store.unpinFromQuickAccess('baseTheme')
    expect(store.pinnedQuickAccess).not.toContain('baseTheme')
  })

  it('can toggle pinned state', () => {
    const store = useSettingsStore()
    const initialState = store.isPinned('useSponsorBlock')
    store.togglePinned('useSponsorBlock')
    expect(store.isPinned('useSponsorBlock')).toBe(!initialState)
  })

  it('can update a setting value', () => {
    const store = useSettingsStore()
    store.updateSetting('autoplayVideos', false)
    expect(store.autoplayVideos).toBe(false)
  })

  it('can update string settings', () => {
    const store = useSettingsStore()
    store.updateSetting('defaultQuality', '1080')
    expect(store.defaultQuality).toBe('1080')
  })

  it('can import settings', () => {
    const store = useSettingsStore()
    store.importSettings({ defaultQuality: '480', autoplayVideos: false })
    expect(store.defaultQuality).toBe('480')
    expect(store.autoplayVideos).toBe(false)
  })

  it('can export transferable settings', () => {
    const store = useSettingsStore()
    const exported = store.exportSettings()
    expect(exported).toHaveProperty('defaultQuality')
    expect(exported).toHaveProperty('autoplayVideos')
  })

  it('excludes non-transferable settings from export', () => {
    const store = useSettingsStore()
    const exported = store.exportSettings()
    expect(exported).not.toHaveProperty('useProxy')
    expect(exported).not.toHaveProperty('proxyHostname')
    expect(exported).not.toHaveProperty('syncServerToken')
  })

  it('can reset a setting to default', () => {
    const store = useSettingsStore()
    store.updateSetting('defaultQuality', '1080')
    store.resetSettingToDefault('defaultQuality')
    expect(store.defaultQuality).toBe('720')
  })
})

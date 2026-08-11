import { describe, it, expect, beforeEach } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { useSettingsSearch } from '@/composables/useSettingsSearch'

beforeEach(() => {
  setActivePinia(createPinia())
})

describe('useSettingsSearch', () => {
  it('returns empty results for empty query', () => {
    const { searchResults, setSearchQuery } = useSettingsSearch()
    setSearchQuery('')
    expect(searchResults.value).toHaveLength(0)
  })

  it('returns empty results for single character query', () => {
    const { searchResults, setSearchQuery } = useSettingsSearch()
    setSearchQuery('a')
    expect(searchResults.value).toHaveLength(0)
  })

  it('finds theme settings when searching "theme"', () => {
    const { searchResults, setSearchQuery } = useSettingsSearch()
    setSearchQuery('theme')
    expect(searchResults.value.length).toBeGreaterThan(0)
    const hasThemeResult = searchResults.value.some(
      r => r.item.key === 'baseTheme'
    )
    expect(hasThemeResult).toBe(true)
  })

  it('finds theme settings via synonym "dark mode"', () => {
    const { searchResults, setSearchQuery } = useSettingsSearch()
    setSearchQuery('dark mode')
    expect(searchResults.value.length).toBeGreaterThan(0)
  })

  it('finds quality settings when searching "quality"', () => {
    const { searchResults, setSearchQuery } = useSettingsSearch()
    setSearchQuery('quality')
    const hasQualityResult = searchResults.value.some(
      r => r.item.key === 'defaultQuality'
    )
    expect(hasQualityResult).toBe(true)
  })

  it('finds history settings when searching "privacy"', () => {
    const { searchResults, setSearchQuery } = useSettingsSearch()
    setSearchQuery('privacy')
    expect(searchResults.value.length).toBeGreaterThan(0)
  })

  it('finds proxy settings when searching "proxy"', () => {
    const { searchResults, setSearchQuery } = useSettingsSearch()
    setSearchQuery('proxy')
    const hasProxyResult = searchResults.value.some(
      r => r.item.key === 'useProxy' || r.item.key === 'proxyVideos'
    )
    expect(hasProxyResult).toBe(true)
  })

  it('returns results sorted by score', () => {
    const { searchResults, setSearchQuery } = useSettingsSearch()
    setSearchQuery('video')
    if (searchResults.value.length > 1) {
      for (let i = 0; i < searchResults.value.length - 1; i++) {
        expect(searchResults.value[i].score).toBeGreaterThanOrEqual(
          searchResults.value[i + 1].score
        )
      }
    }
  })

  it('limits results to 20 items', () => {
    const { searchResults, setSearchQuery } = useSettingsSearch()
    setSearchQuery('a')
    setSearchQuery('settings')
    expect(searchResults.value.length).toBeLessThanOrEqual(20)
  })
})

import { computed, ref } from 'vue'
import { useSettingsStore } from '@/stores/settings'
import { settingsConfig, type SettingsCategory, type SettingItem } from '../views/Settings/config'

export interface SearchResult {
  category: SettingsCategory
  section: { id: string; label: string }
  item: SettingItem
  score: number
  matchedOn: 'label' | 'description' | 'synonym' | 'value'
}

const synonymMap: Record<string, string[]> = {
  theme: ['dark mode', 'night', 'appearance', 'color', 'look', 'style', 'skin', 'light'],
  notifications: ['alerts', 'push', 'email', 'bell', 'notify', 'messages'],
  privacy: ['security', 'data', 'tracking', 'permissions', 'history', 'private'],
  playback: ['autoplay', 'video', 'player', 'watch', 'stream', 'play'],
  quality: ['resolution', 'hd', '1080', '720', '480', '4k', '2k', 'video'],
  volume: ['audio', 'sound', 'loud', 'mute', 'loudness'],
  profile: ['identity', 'user', 'account', 'who', 'name'],
  sync: ['cloud', 'backup', 'cross-device', 'server'],
  downloads: ['save', 'offline', 'yt-dlp', 'format', 'path'],
  region: ['language', 'locale', 'country', 'location', 'timezone'],
  performance: ['speed', 'smooth', 'scrolling', 'animation', 'lag'],
  backend: ['api', 'invidious', 'server', 'source', 'provider'],
  sponsorblock: ['sponsor', 'skip', 'segment', 'ads'],
  history: ['watch', 'tracking', 'record', 'log'],
  search: ['queries', 'suggestions', 'autocomplete', 'find'],
  proxy: ['socks', 'vpn', 'tunnel', 'network', 'tor'],
  landing: ['startup', 'home', 'start', 'page', 'default'],
  sidebar: ['navigation', 'menu', 'expand', 'labels', 'hide'],
  appearance: ['theme', 'dark', 'light', 'color', 'look'],
  accessibility: ['motion', 'reduce', 'scale', 'zoom', 'ui'],
  subscription: ['channels', 'feed', 'fetch', 'update', 'subscribe'],
  player: ['video', 'playback', 'quality', 'speed', 'volume'],
  data: ['export', 'import', 'delete', 'clear', 'reset', 'backup'],
}

export function useSettingsSearch() {
  const settingsStore = useSettingsStore()
  const searchQuery = ref('')

  const allItems = computed(() => {
    const items: { category: SettingsCategory; section: { id: string; label: string }; item: SettingItem }[] = []
    for (const category of settingsConfig) {
      for (const section of category.sections) {
        for (const item of section.items) {
          items.push({ category, section: { id: section.id, label: section.label }, item })
        }
      }
    }
    return items
  })

  function normalizeText(text: string): string {
    return text.toLowerCase().replace(/[^a-z0-9\s]/g, '').trim()
  }

  function getSearchableText(item: SettingItem): string[] {
    const texts: string[] = []
    texts.push(normalizeText(item.label))
    texts.push(normalizeText(item.description))
    for (const syn of item.synonyms) {
      texts.push(normalizeText(syn))
    }
    return texts
  }

  function findSynonyms(query: string): string[] {
    const normalized = normalizeText(query)
    const synonyms: string[] = [normalized]
    for (const [key, values] of Object.entries(synonymMap)) {
      if (key === normalized || values.includes(normalized)) {
        synonyms.push(key, ...values)
      }
    }
    return [...new Set(synonyms)]
  }

  function getValueDisplay(key: string): string {
    const value = (settingsStore as any)[key]
    if (value === undefined || value === null) return ''
    if (typeof value === 'boolean') return value ? 'on' : 'off'
    return String(value)
  }

  function calculateScore(text: string, query: string, synonyms: string[]): { score: number; matchedOn: SearchResult['matchedOn'] } {
    const normalizedText = normalizeText(text)
    const normalizedQuery = normalizeText(query)

    if (normalizedText === normalizedQuery) return { score: 100, matchedOn: 'label' }
    if (normalizedText.startsWith(normalizedQuery)) return { score: 80, matchedOn: 'label' }
    if (normalizedText.includes(normalizedQuery)) return { score: 60, matchedOn: 'label' }

    for (const syn of synonyms) {
      if (normalizedText.includes(syn)) return { score: 40, matchedOn: 'synonym' }
    }

    return { score: 0, matchedOn: 'label' }
  }

  const searchResults = computed<SearchResult[]>(() => {
    const query = searchQuery.value.trim()
    if (!query || query.length < 2) return []

    const querySynonyms = findSynonyms(query)
    const results: SearchResult[] = []

    for (const { category, section, item } of allItems.value) {
      let bestScore = 0
      let bestMatch: SearchResult['matchedOn'] = 'label'

      const searchableTexts = getSearchableText(item)
      for (const text of searchableTexts) {
        for (const q of [query, ...querySynonyms]) {
          const { score, matchedOn } = calculateScore(text, q, querySynonyms)
          if (score > bestScore) {
            bestScore = score
            bestMatch = matchedOn
          }
        }
      }

      const valueDisplay = getValueDisplay(item.key)
      if (valueDisplay) {
        for (const q of [query, ...querySynonyms]) {
          if (normalizeText(valueDisplay).includes(normalizeText(q))) {
            const valueScore = 30
            if (valueScore > bestScore) {
              bestScore = valueScore
              bestMatch = 'value'
            }
          }
        }
      }

      if (bestScore > 0) {
        results.push({
          category,
          section,
          item,
          score: bestScore,
          matchedOn: bestMatch,
        })
      }
    }

    return results.sort((a, b) => b.score - a.score).slice(0, 20)
  })

  function setSearchQuery(query: string) {
    searchQuery.value = query
  }

  return {
    searchQuery,
    searchResults,
    setSearchQuery,
  }
}

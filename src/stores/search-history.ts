import { defineStore } from 'pinia'

export interface SearchHistoryEntry {
  _id: string
  timeWatched: number
  [key: string]: unknown
}

export interface SearchHistoryState {
  searchHistoryEntries: SearchHistoryEntry[]
}

const MIXED_SEARCH_HISTORY_ENTRIES_DISPLAY_LIMIT = 10

export const useSearchHistoryStore = defineStore('search-history', {
  state: (): SearchHistoryState => ({
    searchHistoryEntries: [],
  }),

  getters: {
    getSearchHistoryEntries: (state) => state.searchHistoryEntries,

    getLatestSearchHistoryNames: (state) => {
      return state.searchHistoryEntries.map((entry) => entry._id)
    },

    getLatestMatchingSearchHistoryNames: (state) => (id: string) => {
      const matches: string[] = []
      let counter = 0

      for (const entry of state.searchHistoryEntries) {
        if (entry._id.startsWith(id)) {
          matches.push(entry._id)
          counter++
          if (counter === MIXED_SEARCH_HISTORY_ENTRIES_DISPLAY_LIMIT) {
            break
          }
        }
      }

      return matches.sort((a, b) => a.length - b.length)
    },

    getSearchHistoryEntryWithId: (state) => (id: string) => {
      return state.searchHistoryEntries.find((p) => p._id === id)
    },
  },

  actions: {
    loadSearchHistory() {
      // Placeholder for DB integration
    },

    addSearchHistoryEntry(entry: SearchHistoryEntry) {
      this.searchHistoryEntries = this.searchHistoryEntries.filter(
        (p) => p._id !== entry._id
      )
      this.searchHistoryEntries.unshift(entry)
    },

    removeSearchHistoryEntry(_id: string) {
      this.searchHistoryEntries = this.searchHistoryEntries.filter(
        (entry) => entry._id !== _id
      )
    },

    clearSearchHistory() {
      this.searchHistoryEntries = []
    },

    setSearchHistoryEntries(entries: SearchHistoryEntry[]) {
      this.searchHistoryEntries = entries
    },
  },
})

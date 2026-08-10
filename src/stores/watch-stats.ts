import { defineStore } from 'pinia'

export interface WatchStatsState {
  watchSecondsByDate: Record<string, number>
  hasHistoricalWatchTimeEstimate: boolean
  historicalWatchTimePlaybackSpeed: number | null
  watchStatsResetVersion: number
}

export const useWatchStatsStore = defineStore('watch-stats', {
  state: (): WatchStatsState => ({
    watchSecondsByDate: {},
    hasHistoricalWatchTimeEstimate: false,
    historicalWatchTimePlaybackSpeed: null,
    watchStatsResetVersion: 0,
  }),

  getters: {
    getWatchSecondsByDate: (state) => state.watchSecondsByDate,
    getHasHistoricalWatchTimeEstimate: (state) => state.hasHistoricalWatchTimeEstimate,
    getHistoricalWatchTimePlaybackSpeed: (state) => state.historicalWatchTimePlaybackSpeed,
    getWatchStatsResetVersion: (state) => state.watchStatsResetVersion,
  },

  actions: {
    loadWatchStats() {
      // Placeholder for DB integration
    },

    recordWatchTime(date: string, seconds: number) {
      if (!/^\d{4}-\d{2}-\d{2}$/.test(date) || !Number.isFinite(seconds) || seconds <= 0) {
        return
      }
      this.watchSecondsByDate[date] = (this.watchSecondsByDate[date] ?? 0) + seconds
    },

    setWatchStats(records: Array<{ date: string; seconds: number }>) {
      this.watchSecondsByDate = Object.fromEntries(
        records.map(({ date, seconds }) => [date, seconds])
      )
    },

    setHasHistoricalWatchTimeEstimate(value: boolean) {
      this.hasHistoricalWatchTimeEstimate = value
    },

    setHistoricalWatchTimePlaybackSpeed(value: number | null) {
      this.historicalWatchTimePlaybackSpeed = value
    },

    resetWatchStats() {
      this.watchSecondsByDate = {}
      this.hasHistoricalWatchTimeEstimate = false
      this.historicalWatchTimePlaybackSpeed = null
      this.watchStatsResetVersion++
    },
  },
})

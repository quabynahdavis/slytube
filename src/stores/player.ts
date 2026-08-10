import { defineStore } from 'pinia'

export interface PlayerState {
  cachedPlayerLocales: Record<string, unknown>
  volume: number
  playbackRate: number
  quality: string
  autoplay: boolean
}

export const usePlayerStore = defineStore('player', {
  state: (): PlayerState => ({
    cachedPlayerLocales: {},
    volume: 1,
    playbackRate: 1,
    quality: '720',
    autoplay: true,
  }),

  getters: {
    getVolume: (state) => state.volume,
    getPlaybackRate: (state) => state.playbackRate,
    getQuality: (state) => state.quality,
    getAutoplay: (state) => state.autoplay,
  },

  actions: {
    setVolume(v: number) {
      this.volume = Math.max(0, Math.min(1, v))
    },

    setPlaybackRate(r: number) {
      this.playbackRate = r
    },

    setQuality(q: string) {
      this.quality = q
    },

    setAutoplay(enabled: boolean) {
      this.autoplay = enabled
    },

    cachePlayerLocale(locale: string, data: unknown) {
      this.cachedPlayerLocales[locale] = data
    },
  },
})

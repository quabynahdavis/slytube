import { defineStore } from 'pinia'

export interface WatchQueueItem {
  videoId: string
  title: string
  author: string
  authorId: string
  lengthSeconds: number
  videoThumbnails: Array<{ url: string; width: number; height: number }>
  queueItemId: number
}

export interface WatchQueueState {
  items: WatchQueueItem[]
}

let nextQueueItemId = 1

export const useWatchQueueStore = defineStore('watch-queue', {
  state: (): WatchQueueState => ({
    items: [],
  }),

  getters: {
    getWatchQueue: (state) => state.items,
    getWatchQueueLength: (state) => state.items.length,
    getNextQueuedVideo: (state) => state.items[0] ?? null,
  },

  actions: {
    addVideoToWatchQueue(video: Omit<WatchQueueItem, 'queueItemId'>, playNext = false) {
      const item: WatchQueueItem = {
        ...video,
        queueItemId: nextQueueItemId++,
      }

      if (playNext) {
        this.items.unshift(item)
      } else {
        this.items.push(item)
      }
    },

    removeVideoFromWatchQueue(queueItemId: number) {
      const index = this.items.findIndex((item) => item.queueItemId === queueItemId)
      if (index !== -1) {
        this.items.splice(index, 1)
      }
    },

    moveVideoInWatchQueue(queueItemId: number, offset: number) {
      const index = this.items.findIndex((item) => item.queueItemId === queueItemId)
      const targetIndex = index + offset
      if (index === -1 || targetIndex < 0 || targetIndex >= this.items.length) {
        return
      }

      const [item] = this.items.splice(index, 1)
      this.items.splice(targetIndex, 0, item)
    },

    clearWatchQueue() {
      this.items.splice(0)
    },
  },
})

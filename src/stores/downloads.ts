import { defineStore } from 'pinia'

export interface Download {
  id: string
  status: 'downloading' | 'processing' | 'completed' | 'error' | 'queued'
  progress: number
  title: string
  url: string
  outputPath: string
  format: string
  quality: string
  fileSize?: number
  downloadedBytes?: number
  speed?: number
  eta?: number
  errorMessage?: string
  createdAt: number
  updatedAt: number
}

export interface DownloadsState {
  downloads: Record<string, Download>
}

export const useDownloadsStore = defineStore('downloads', {
  state: (): DownloadsState => ({
    downloads: {},
  }),

  getters: {
    getDownloads: (state) => state.downloads,

    getActiveDownloads: (state) => {
      return Object.values(state.downloads).filter(
        (d) => d.status === 'downloading' || d.status === 'processing'
      )
    },

    getCompletedDownloads: (state) => {
      return Object.values(state.downloads).filter((d) => d.status === 'completed')
    },
  },

  actions: {
    loadDownloads() {
      // Placeholder for DB integration
    },

    addDownload(download: Download) {
      this.downloads[download.id] = download
    },

    updateDownload(id: string, status: Partial<Download>) {
      if (this.downloads[id]) {
        Object.assign(this.downloads[id], status, { updatedAt: Date.now() })
      }
    },

    removeDownload(id: string) {
      delete this.downloads[id]
    },

    clearCompleted() {
      for (const [id, download] of Object.entries(this.downloads)) {
        if (download.status !== 'downloading' && download.status !== 'processing') {
          delete this.downloads[id]
        }
      }
    },
  },
})

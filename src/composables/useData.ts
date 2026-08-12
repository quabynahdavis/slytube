import { ref, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import type { Video, Channel } from '../api/types'
import { getCommentsInfo } from '../api'
import * as api from '../api'
import { getSegments, formatCategory } from '../api/sponsorblock'

export function useVideoLoader() {
  const video = ref<Video | null>(null)
  const loading = ref(false)
  const error = ref<string | null>(null)

  async function load(videoId: string) {
    loading.value = true
    error.value = null
    try {
      video.value = await api.getVideo(videoId)
    } catch (e: any) {
      error.value = e.message || 'Failed to load video'
      video.value = null
    } finally {
      loading.value = false
    }
  }

  return { video, loading, error, load }
}

export function useChannelLoader() {
  const channel = ref<Channel | null>(null)
  const loading = ref(false)
  const error = ref<string | null>(null)

  async function load(channelId: string) {
    loading.value = true
    error.value = null
    try {
      channel.value = await api.getChannelInfo(channelId)
    } catch (e: any) {
      error.value = e.message || 'Failed to load channel'
      channel.value = null
    } finally {
      loading.value = false
    }
  }

  return { channel, loading, error, load }
}

export function useSearch() {
  const results = ref<Video[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)

  async function search(query: string) {
    if (!query.trim()) return
    loading.value = true
    error.value = null
    try {
      results.value = await api.search(query)
    } catch (e: any) {
      error.value = e.message || 'Search failed'
      results.value = []
    } finally {
      loading.value = false
    }
  }

  return { results, loading, error, search }
}

export function useComments(videoId: string) {
  const comments = ref<any[]>([])
  const loading = ref(false)

  async function load() {
    if (!videoId) return
    loading.value = true
    try {
      comments.value = await getCommentsInfo(videoId)
    } catch {
      comments.value = []
    } finally {
      loading.value = false
    }
  }

  return { comments, loading, load }
}

export function useSponsorBlock(videoId: string) {
  const segments = ref<Awaited<ReturnType<typeof getSegments>>>([])
  const loading = ref(false)

  async function load() {
    if (!videoId) return
    loading.value = true
    try {
      segments.value = await getSegments(videoId)
    } catch {
      segments.value = []
    } finally {
      loading.value = false
    }
  }

  function getColor(category: string): string {
    const colors: Record<string, string> = {
      sponsor: '#00d400',
      intro: '#00ffff',
      outro: '#0202ed',
      selfpromo: '#ffff00',
      interaction: '#cc00ff',
      music_offtopic: '#ff9900',
      preview: '#008fd6',
      filler: '#7300ff',
      highlight: '#ff0000',
    }
    return colors[category] || '#ffffff'
  }

  return { segments, load, getColor, formatCategory }
}

interface DownloadStatus {
  id: number
  videoId: string
  title: string
  status: string
  percent: number
  speed: string | null
  eta: string | null
  destination: string | null
  errorMessage: string | null
}

export function useDownloads() {
  const downloads = ref<DownloadStatus[]>([])
  const loading = ref(false)
  let unlistenProgress: (() => void) | null = null
  let unlistenDestination: (() => void) | null = null
  let unlistenComplete: (() => void) | null = null
  let unlistenError: (() => void) | null = null
  let unlistenCancelled: (() => void) | null = null

  async function loadDownloads() {
    loading.value = true
    try {
      downloads.value = await invoke('yt_dlp_list')
    } catch {
      downloads.value = []
    } finally {
      loading.value = false
    }
    await setupListeners()
  }

  async function setupListeners() {
    // Clean up existing listeners
    cleanupListeners()

    unlistenProgress = await listen('yt-dlp-progress', (event: any) => {
      const { id, percent, speed, eta } = event.payload
      const idx = downloads.value.findIndex(d => d.id === id)
      if (idx !== -1) {
        downloads.value[idx].percent = percent
        downloads.value[idx].speed = speed
        downloads.value[idx].eta = eta
        downloads.value[idx].status = 'downloading'
      }
    })

    unlistenDestination = await listen('yt-dlp-destination', (event: any) => {
      const { id, destination } = event.payload
      const idx = downloads.value.findIndex(d => d.id === id)
      if (idx !== -1) {
        downloads.value[idx].destination = destination
        downloads.value[idx].title = destination.split('/').pop()?.split('\\').pop() || destination
      }
    })

    unlistenComplete = await listen('yt-dlp-complete', (event: any) => {
      const { id } = event.payload
      const idx = downloads.value.findIndex(d => d.id === id)
      if (idx !== -1) {
        downloads.value[idx].status = 'completed'
        downloads.value[idx].percent = 100
      }
    })

    unlistenError = await listen('yt-dlp-error', (event: any) => {
      const { id, error } = event.payload
      const idx = downloads.value.findIndex(d => d.id === id)
      if (idx !== -1) {
        downloads.value[idx].status = 'failed'
        downloads.value[idx].errorMessage = error
      }
    })

    unlistenCancelled = await listen('yt-dlp-cancelled', (event: any) => {
      const id = event.payload
      const idx = downloads.value.findIndex(d => d.id === id)
      if (idx !== -1) {
        downloads.value[idx].status = 'cancelled'
      }
    })
  }

  function cleanupListeners() {
    if (unlistenProgress) { unlistenProgress(); unlistenProgress = null }
    if (unlistenDestination) { unlistenDestination(); unlistenDestination = null }
    if (unlistenComplete) { unlistenComplete(); unlistenComplete = null }
    if (unlistenError) { unlistenError(); unlistenError = null }
    if (unlistenCancelled) { unlistenCancelled(); unlistenCancelled = null }
  }

  onUnmounted(cleanupListeners)

  async function startDownload(args: any) {
    try {
      // Map legacy {url, format} to new {videoId, mode} format
      let downloadArgs = args
      if (args.url && !args.videoId) {
        // Extract videoId from YouTube URL
        const urlMatch = args.url.match(/[?&]v=([^&]+)/) || args.url.match(/youtu\.be\/([^?&]+)/)
        downloadArgs = {
          videoId: urlMatch ? urlMatch[1] : args.url,
          mode: args.format === 'audio' ? 'audio' : 'video',
          quality: args.quality,
        }
      }
      const id = await invoke('yt_dlp_download', { args: downloadArgs })
      return id
    } catch (e: any) {
      throw new Error(e)
    }
  }

  async function cancelDownload(id: number) {
    try {
      await invoke('yt_dlp_cancel', { id })
    } catch (e: any) {
      throw new Error(e)
    }
  }

  async function getInfo(url: string) {
    try {
      return await invoke('yt_dlp_get_info', { url })
    } catch (e: any) {
      throw new Error(e)
    }
  }

  return { downloads, loading, loadDownloads, startDownload, cancelDownload, getInfo }
}

export function useHistory() {
  const history = ref<any[]>([])
  const loading = ref(false)

  async function loadHistory() {
    loading.value = true
    try {
      history.value = await invoke('db_history_find_all', { limit: 100 })
    } catch {
      history.value = []
    } finally {
      loading.value = false
    }
  }

  async function addToHistory(entry: any) {
    try {
      await invoke('db_history_upsert', { entry })
      await loadHistory()
    } catch {}
  }

  async function clearHistory() {
    try {
      await invoke('db_history_clear')
      history.value = []
    } catch {}
  }

  async function removeFromHistory(videoId: string) {
    try {
      await invoke('db_history_delete', { videoId })
      await loadHistory()
    } catch {}
  }

  return { history, loading, loadHistory, addToHistory, clearHistory, removeFromHistory }
}

export function usePlaylists() {
  const playlists = ref<any[]>([])
  const loading = ref(false)

  async function loadPlaylists() {
    loading.value = true
    try {
      playlists.value = await invoke('db_playlists_find_all', { profileId: 'default' })
    } catch {
      playlists.value = []
    } finally {
      loading.value = false
    }
  }

  async function createPlaylist(name: string, description = '') {
    try {
      const id = crypto.randomUUID()
      await invoke('db_playlists_create', { id, profileId: 'default', name, description })
      await loadPlaylists()
      return id
    } catch { return null }
  }

  async function deletePlaylist(id: string) {
    try {
      await invoke('db_playlists_delete', { id })
      await loadPlaylists()
    } catch {}
  }

  async function addToPlaylist(playlistId: string, videoId: string) {
    try {
      await invoke('db_playlists_add_video', { playlistId, videoId, position: 0 })
    } catch {}
  }

  async function removeFromPlaylist(playlistId: string, videoId: string) {
    try {
      await invoke('db_playlists_remove_video', { playlistId, videoId })
    } catch {}
  }

  return { playlists, loading, loadPlaylists, createPlaylist, deletePlaylist, addToPlaylist, removeFromPlaylist }
}

export function useSubscriptions() {
  const subscriptions = ref<string[]>([])
  const loading = ref(false)

  async function loadSubscriptions() {
    loading.value = true
    try {
      const result = await invoke('db_profiles_get_subscriptions', { profileId: 'default' })
      subscriptions.value = result as string[]
    } catch {
      subscriptions.value = []
    } finally {
      loading.value = false
    }
  }

  async function subscribe(channelId: string) {
    try {
      await invoke('db_profiles_add_subscription', { profileId: 'default', channelId })
      await loadSubscriptions()
    } catch {}
  }

  async function unsubscribe(channelId: string) {
    try {
      await invoke('db_profiles_remove_subscription', { profileId: 'default', channelId })
      await loadSubscriptions()
    } catch {}
  }

  function isSubscribed(channelId: string): boolean {
    return subscriptions.value.includes(channelId)
  }

  return { subscriptions, loading, loadSubscriptions, subscribe, unsubscribe, isSubscribed }
}

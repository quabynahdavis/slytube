import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { Video, Channel } from '../api/types'
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

export function useDownloads() {
  const downloads = ref<any[]>([])
  const loading = ref(false)

  async function loadDownloads() {
    loading.value = true
    try {
      downloads.value = await invoke('yt_dlp_list')
    } catch {
      downloads.value = []
    } finally {
      loading.value = false
    }
  }

  async function startDownload(args: any) {
    try {
      const id = await invoke('yt_dlp_download', { args })
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
      await invoke('db_history_upsert', entry)
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

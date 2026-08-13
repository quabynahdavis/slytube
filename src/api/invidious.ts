import { invoke } from '@tauri-apps/api/core'

// Invidious instance management
let currentInstance: { url: string } | null = null

export async function loadInstances(): Promise<void> {
  try {
    const instances = await invoke<string[]>('invidious_get_instances_list')
    if (instances && instances.length > 0) {
      currentInstance = { url: instances[0] }
    }
  } catch {
    console.log('[Invidious] Using default instance')
  }
}

export function getThumbnailUrl(videoId: string, quality = 'maxresdefault'): string {
  if (currentInstance) {
    return `${currentInstance.url}/vi/${videoId}/${quality}.jpg`
  }
  return `https://i.ytimg.com/vi/${videoId}/${quality}.jpg`
}

/**
 * Returns the current Invidious instance, or a fallback if none loaded.
 */
export function getCurrentInstance(): { url: string } {
  return currentInstance || { url: 'https://inv.nadeko.net' }
}

/**
 * Returns an image URL directly (no caching layer).
 * The image cache protocol handler was removed — using direct YouTube URLs.
 */
export function cacheImageUrl(url: string): string {
  if (!url) return ''
  return url
}

/**
 * Proxies an image URL through Invidious if available,
 * otherwise returns the original YouTube image URL.
 */
export function proxyImageUrl(url: string, videoId?: string): string {
  if (!url) {
    if (videoId) return getThumbnailUrl(videoId, 'hqdefault')
    return ''
  }

  if (url.includes('googleusercontent.com') || url.includes('ytimg.com')) {
    return url
  }

  const videoIdFromUrl = url.match(/\/vi\/([^\/]+)\//)?.[1]
  const id = videoId || videoIdFromUrl
  if (id) {
    return getThumbnailUrl(id, 'hqdefault')
  }

  return url
}

/**
 * Returns an author avatar URL as-is without video thumbnail transformation.
 * Use this for channel/author avatars to prevent them from being turned
 * into video thumbnail URLs.
 */
export function proxyAvatarUrl(url: string): string {
  if (!url) return ''
  return url
}

// Invidious API calls
export async function invidiousGetVideo(videoId: string): Promise<any> {
  return await invoke('invidious_get_video', { videoId })
}

export async function invidiousSearch(query: string, page = 1, filters?: any): Promise<any[]> {
  return await invoke<any[]>('invidious_search', { query, page, ...filters })
}

export async function invidiousGetPopular(): Promise<any[]> {
  return await invoke<any[]>('invidious_get_popular')
}

export async function invidiousGetChannel(channelId: string): Promise<any> {
  return await invoke('invidious_get_channel', { channelId })
}

export async function invidiousGetPlaylist(playlistId: string): Promise<any> {
  return await invoke('invidious_get_playlist', { playlistId })
}

export async function invidiousGetComments(videoId: string): Promise<any> {
  return await invoke('invidious_get_comments', { videoId })
}

export async function invidiousGetDashManifest(videoId: string): Promise<string | null> {
  return await invoke<string | null>('invidious_get_dash_manifest', { videoId })
}

export async function invidiousGetDashUrl(videoId: string): Promise<string | null> {
  return await invoke<string | null>('invidious_get_dash_url', { videoId })
}

export async function invidiousGetChannelShorts(channelId: string): Promise<any> {
  return await invoke('invidious_get_channel_shorts', { channelId })
}

export async function invidiousGetChannelCommunityPosts(channelId: string): Promise<any> {
  return await invoke('invidious_get_community_posts', { channelId })
}

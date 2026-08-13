import { invoke } from '@tauri-apps/api/core'

export interface InvidiousInstance {
  url: string
  name: string
  health: number
  cors: boolean
  api: boolean
}

const FALLBACK_INSTANCES: InvidiousInstance[] = [
  { url: 'https://inv.nadeko.net', name: 'Nadeko', health: 0, cors: false, api: true },
  { url: 'https://invidious.nerdvpn.de', name: 'NerdVPN', health: 0, cors: true, api: true },
  { url: 'https://yewtu.be', name: 'Yewtu', health: 0, cors: false, api: true },
  { url: 'https://invidious.private.coffee', name: 'Private Coffee', health: 0, cors: true, api: true },
  { url: 'https://invidious.jing.rocks', name: 'Jing', health: 0, cors: true, api: true },
]

let currentInstance: InvidiousInstance = FALLBACK_INSTANCES[0]
let instancesLoaded = false

export function getCurrentInstance(): InvidiousInstance {
  return currentInstance
}

export function getCurrentInstanceUrl(): string {
  return currentInstance.url
}

export async function loadInstances(): Promise<void> {
  if (instancesLoaded) return
  instancesLoaded = true

  try {
    const instances = await invoke<InvidiousInstance[]>('invidious_get_instances_list')
    if (instances && instances.length > 0) {
      currentInstance = instances[0]
      console.log(`[Invidious] Using instance: ${currentInstance.url}`)
    }
  } catch {
    console.log('[Invidious] Using default instance:', currentInstance.url)
  }
}

export function proxyImageUrl(url: string, videoId?: string): string {
  if (!url) {
    if (videoId) return `https://i.ytimg.com/vi/${videoId}/hqdefault.jpg`
    return ''
  }

  if (url.includes('googleusercontent.com') || url.includes('ytimg.com')) {
    return url
  }

  const videoIdFromUrl = url.match(/\/vi\/([^\/]+)\//)?.[1]
  const id = videoId || videoIdFromUrl
  if (id) {
    return `https://i.ytimg.com/vi/${id}/hqdefault.jpg`
  }

  return url
}

/**
 * Wraps a YouTube image URL in the custom `imgcache://` scheme so it is
 * fetched and cached by the Rust backend instead of directly by the webview.
 *
 * The original URL is URI-encoded and appended to the scheme:
 *   imgcache://<encodeURIComponent(originalUrl)>
 *
 * Returns an empty string when the input is empty.
 */
export function cacheImageUrl(url: string): string {
  if (!url) return ''
  return `imgcache://${encodeURIComponent(url)}`
}

export function getThumbnailUrl(videoId: string, quality = 'maxresdefault'): string {
  return `${currentInstance.url}/vi/${videoId}/${quality}.jpg`
}

async function invidiousFetch<T>(path: string): Promise<T> {
  return await invoke<T>('invidious_fetch', { path })
}

export async function invidiousGetVideo(videoId: string): Promise<any> {
  return invidiousFetch(`/api/v1/videos/${videoId}`)
}

export async function invidiousSearch(
  query: string,
  page = 1,
  filters?: {
    sort?: string
    type?: string
    duration?: string
    date?: string
  }
): Promise<any> {
  const params = new URLSearchParams()
  params.set('q', query)
  params.set('page', String(page))
  if (filters?.sort && filters.sort !== 'relevance') params.set('sort', filters.sort)
  if (filters?.type && filters.type !== 'all') params.set('type', filters.type)
  if (filters?.duration && filters.duration !== 'all') params.set('duration', filters.duration)
  if (filters?.date && filters.date !== 'all') params.set('date', filters.date)
  return invidiousFetch(`/api/v1/search?${params.toString()}`)
}

export async function invidiousGetTrending(type = 'default'): Promise<any> {
  return invidiousFetch(`/api/v1/trending?type=${type}`)
}

export async function invidiousGetPopular(): Promise<any> {
  return invidiousFetch('/api/v1/popular')
}

export async function invidiousGetChannel(channelId: string): Promise<any> {
  return invidiousFetch(`/api/v1/channels/${channelId}`)
}

export async function invidiousGetPlaylist(playlistId: string): Promise<any> {
  return invidiousFetch(`/api/v1/playlists/${playlistId}`)
}

export async function invidiousGetComments(videoId: string, continuation?: string): Promise<any> {
  const params = continuation ? `?continuation=${continuation}` : ''
  return invidiousFetch(`/api/v1/comments/${videoId}${params}`)
}

export async function invidiousGetCommentReplies(videoId: string, replyToken: string): Promise<any> {
  return invidiousFetch(`/api/v1/comments/${videoId}?continuation=${replyToken}`)
}

export async function invidiousGetSearchSuggestions(query: string): Promise<any> {
  return invidiousFetch(`/api/v1/search/suggestions?q=${encodeURIComponent(query)}`)
}

export async function invidiousGetDashManifest(videoId: string, local = true): Promise<string> {
  return invidiousFetch(`/api/manifest/dash/id/${videoId}?local=${local}`)
}

export async function invidiousGetChannelShorts(channelId: string): Promise<any> {
  return invidiousFetch(`/api/v1/channels/${channelId}/shorts`)
}

export async function invidiousGetChannelCommunityPosts(channelId: string): Promise<any> {
  return invidiousFetch(`/api/v1/channels/${channelId}/community`)
}

export async function invidiousGetChannelInfo(channelId: string): Promise<any> {
  return invidiousFetch(`/api/v1/channels/${channelId}`)
}

export async function invidiousGetDashUrl(videoId: string): Promise<string> {
  const info = await invidiousGetVideo(videoId)
  return info.dashUrl || ''
}

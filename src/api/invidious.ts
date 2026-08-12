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
    if (videoId) return getThumbnailUrl(videoId, 'hqdefault')
    return ''
  }
  const instance = getCurrentInstanceUrl()

  if (url.includes(instance)) {
    return url
  }

  const videoIdFromUrl = url.match(/\/vi\/([^\/]+)\//)?.[1]
  const id = videoId || videoIdFromUrl
  if (id) {
    return getThumbnailUrl(id, 'hqdefault')
  }

  return url
    .replace('https://i.ytimg.com', instance)
    .replace('https://i1.ytimg.com', instance)
    .replace('https://i2.ytimg.com', instance)
    .replace('https://i3.ytimg.com', instance)
    .replace('https://i4.ytimg.com', instance)
    .replace('https://yt3.ggpht.com', `${instance}/ggpht`)
    .replace('https://yt3.googleusercontent.com', `${instance}/ggpht`)
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

export async function invidiousSearch(query: string, page = 1): Promise<any> {
  return invidiousFetch(`/api/v1/search?q=${encodeURIComponent(query)}&page=${page}`)
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

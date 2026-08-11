export interface InvidiousInstance {
  url: string
  name: string
  health: number
  cors: boolean
  api: boolean
}

const FALLBACK_INSTANCES = [
  'https://invidious.nerdvpn.de',
  'https://inv.nadeko.net',
  'https://invidious.jing.rocks',
  'https://invidious.nerdvpn.de',
  'https://yewtu.be',
  'https://invidious.fdn.fr',
  'https://invidious.private.coffee',
  'https://iv.delnet.org',
]

let currentInstance = FALLBACK_INSTANCES[0]

export function getCurrentInstance(): string {
  return currentInstance
}

export function setInstance(url: string) {
  currentInstance = url
}

export async function testInstance(url: string): Promise<boolean> {
  try {
    const response = await fetch(`${url}/api/v1/stats`, { signal: AbortSignal.timeout(5000) })
    return response.ok
  } catch {
    return false
  }
}

export async function getHealthyInstance(): Promise<string> {
  for (const url of FALLBACK_INSTANCES) {
    if (await testInstance(url)) {
      currentInstance = url
      return url
    }
  }
  return currentInstance
}

function buildUrl(resource: string, id: string, params: Record<string, string> = {}): string {
  const query = new URLSearchParams(params).toString()
  return `${currentInstance}/api/v1/${resource}/${id}${query ? '?' + query : ''}`
}

export async function invidiousFetch<T>(url: string): Promise<T> {
  const response = await fetch(url)
  if (!response.ok) throw new Error(`Invidious error: ${response.status}`)
  return response.json()
}

export async function getVideoInfoInvidious(videoId: string) {
  return await invidiousFetch(buildUrl('videos', videoId))
}

export async function getChannelInvidious(channelId: string) {
  return await invidiousFetch(buildUrl('channels', channelId))
}

export async function searchInvidious(query: string, page = 1) {
  return await invidiousFetch(buildUrl('search', '', { q: query, page: String(page) }))
}

export async function getPlaylistInvidious(playlistId: string) {
  return await invidiousFetch(buildUrl('playlists', playlistId))
}

export async function getCommentsInvidious(videoId: string, continuation?: string) {
  const params: Record<string, string> = {}
  if (continuation) params.continuation = continuation
  return await invidiousFetch(buildUrl('comments', videoId, params))
}

export async function getTrendingInvidious(type = 'news') {
  return await invidiousFetch(buildUrl('trending', '', { type }))
}

export function getProxyUrl(originalUrl: string): string {
  const url = new URL(originalUrl)
  if (!url.searchParams.has('host')) {
    url.searchParams.set('host', url.hostname)
  }
  return originalUrl.replace(url.origin, currentInstance)
}

export function getThumbnailUrl(videoId: string, quality = 'maxresdefault'): string {
  return `${currentInstance}/vi/${videoId}/${quality}.jpg`
}

export function getDashManifestUrl(videoId: string): string {
  return `${currentInstance}/api/manifest/dash/id/${videoId}`
}

export function getStoryboardUrl(videoId: string): string {
  return `${currentInstance}/api/v1/storyboards/${videoId}?height=90`
}

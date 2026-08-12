let tauriFetch: any = null

try {
  const http = await import('@tauri-apps/plugin-http')
  tauriFetch = http.fetch
  console.log('[Invidious] Tauri HTTP client loaded')
} catch (e) {
  console.warn('[Invidious] Tauri HTTP not available, using standard fetch:', e)
  tauriFetch = null
}

export function getFetch(): typeof fetch {
  return (tauriFetch || fetch) as any
}

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
let instancesList: InvidiousInstance[] = [...FALLBACK_INSTANCES]

export function getCurrentInstance(): InvidiousInstance {
  return currentInstance
}

export function getCurrentInstanceUrl(): string {
  return currentInstance.url
}

export async function loadInstances(): Promise<void> {
  console.log('[Invidious] Loading instances...')
  try {
    const response = await getFetch()('https://api.invidious.io/instances.json', {
      method: 'GET',
    } as any)
    if (response.ok) {
      const data: any[] = await response.json()
      const instances = data
        .filter(([, info]: any) => info.api === true && info.cors === true && info.type === 'https')
        .map(([url, info]: any) => ({
          url,
          name: info.name || url,
          health: info.health || 0,
          cors: info.cors,
          api: info.api,
        }))
        .sort((a: InvidiousInstance, b: InvidiousInstance) => b.health - a.health)

      if (instances.length > 0) {
        instancesList = instances
        currentInstance = instances[0]
        console.log(`[Invidious] Loaded ${instances.length} instances, using ${currentInstance.url}`)
      }
    }
  } catch (err) {
    console.warn('[Invidious] Failed to load instances, using fallbacks:', err)
  }

  for (const instance of FALLBACK_INSTANCES) {
    console.log(`[Invidious] Testing fallback instance: ${instance.url}`)
    if (await testInstance(instance.url)) {
      currentInstance = instance
      console.log(`[Invidious] Using fallback instance: ${instance.url}`)
      break
    }
  }
}

export async function testInstance(url: string): Promise<boolean> {
  try {
    const response = await getFetch()(`${url}/api/v1/stats`, {
      method: 'GET',
    } as any)
    return response.ok
  } catch {
    return false
  }
}

export function proxyImageUrl(url: string): string {
  if (!url) return ''
  const instance = getCurrentInstanceUrl()
  // Rewrite YouTube image URLs through Invidious proxy
  return url
    .replace('https://i.ytimg.com', `${instance}`)
    .replace('https://i1.ytimg.com', `${instance}`)
    .replace('https://i2.ytimg.com', `${instance}`)
    .replace('https://i3.ytimg.com', `${instance}`)
    .replace('https://i4.ytimg.com', `${instance}`)
    .replace('https://yt3.ggpht.com', `${instance}/ggpht`)
    .replace('https://yt3.googleusercontent.com', `${instance}/ggpht`)
}

async function invidiousFetch<T>(path: string): Promise<T> {
  const instances = instancesList.length > 0 ? instancesList : FALLBACK_INSTANCES
  const useFetch = getFetch()

  for (const instance of instances) {
    try {
      const fullUrl = `${instance.url}${path}`
      const response = await useFetch(fullUrl, {
        method: 'GET',
        headers: {
          Accept: 'application/json',
        },
      } as any)

      if (response.ok) {
        if (instance.url !== currentInstance.url) {
          currentInstance = instance
        }
        return await response.json() as T
      }
    } catch (e) {
      console.warn(`Invidious instance ${instance.url} failed:`, e)
      continue
    }
  }

  throw new Error(`All Invidious instances failed for ${path}`)
}

function buildPath(resource: string, id: string, params?: Record<string, string>): string {
  const basePath = id ? `/api/v1/${resource}/${id}` : `/api/v1/${resource}`
  if (!params) return basePath

  const searchParams = new URLSearchParams(params)
  const query = searchParams.toString()
  return query ? `${basePath}?${query}` : basePath
}

export async function invidiousGetVideo(videoId: string): Promise<any> {
  return invidiousFetch(buildPath('videos', videoId))
}

export async function invidiousSearch(query: string, page = 1): Promise<any> {
  return invidiousFetch(buildPath('search', '', { q: query, page: String(page) }))
}

export async function invidiousGetTrending(type = 'default'): Promise<any> {
  return invidiousFetch(buildPath('trending', '', { type }))
}

export async function invidiousGetPopular(): Promise<any> {
  return invidiousFetch(buildPath('popular', ''))
}

export async function invidiousGetChannel(channelId: string): Promise<any> {
  return invidiousFetch(buildPath('channels', channelId))
}

export async function invidiousGetPlaylist(playlistId: string): Promise<any> {
  return invidiousFetch(buildPath('playlists', playlistId))
}

export async function invidiousGetComments(videoId: string, continuation?: string): Promise<any> {
  const params: Record<string, string> = {}
  if (continuation) params.continuation = continuation
  return invidiousFetch(buildPath('comments', videoId, params))
}

export async function invidiousGetCommentReplies(videoId: string, replyToken: string): Promise<any> {
  return invidiousFetch(buildPath('comments', videoId, { continuation: replyToken }))
}

export async function invidiousGetSearchSuggestions(query: string): Promise<any> {
  return invidiousFetch(buildPath('search/suggestions', '', { q: query }))
}

export async function invidiousSearchWithFilters(
  query: string,
  page = 1,
  filters?: {
    sortBy?: string
    date?: string
    duration?: string
    type?: string
    features?: string[]
  }
): Promise<any> {
  const params: Record<string, string> = { q: query, page: String(page) }
  if (filters?.sortBy) params.sort_by = filters.sortBy
  if (filters?.date) params.date = filters.date
  if (filters?.duration) params.duration = filters.duration
  if (filters?.type) params.type = filters.type
  if (filters?.features?.length) params.features = filters.features.join(',')

  return invidiousFetch(buildPath('search', '', params))
}

export async function invidiousGetHashtag(hashtag: string, page = 1): Promise<any> {
  return invidiousFetch(buildPath('hashtag', hashtag, { page: String(page) }))
}

export async function invidiousResolveUrl(url: string): Promise<any> {
  return invidiousFetch(buildPath('resolveurl', '', { url }))
}

export async function invidiousGetDashManifest(videoId: string, local = true): Promise<string> {
  const path = `/api/manifest/dash/id/${videoId}?local=${local}`

  for (const instance of instancesList.length > 0 ? instancesList : FALLBACK_INSTANCES) {
    try {
      const fullUrl = `${instance.url}${path}`
      const response = await getFetch()(fullUrl, {
        method: 'GET',
      })
      if (response.ok) {
        return await response.text()
      }
    } catch {
      continue
    }
  }

  throw new Error(`Failed to fetch DASH manifest for ${videoId}`)
}

export async function invidiousGetDashUrl(videoId: string): Promise<string> {
  const info = await invidiousGetVideo(videoId)
  return info.dashUrl || ''
}

export function getProxyUrl(originalUrl: string): string {
  const url = new URL(originalUrl)
  if (!url.searchParams.has('host')) {
    url.searchParams.set('host', url.hostname)
  }
  return originalUrl.replace(url.origin, currentInstance.url)
}

export function getThumbnailUrl(videoId: string, quality = 'maxresdefault'): string {
  return `${currentInstance.url}/vi/${videoId}/${quality}.jpg`
}

export { instancesList, currentInstance }

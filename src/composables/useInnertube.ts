import { Innertube, UniversalCache } from 'youtubei.js'

let searchSession: Awaited<ReturnType<typeof Innertube.create>> | null = null

export async function createInnertube(withPlayer = false) {
  return await Innertube.create({
    enable_session_cache: false,
    retrieve_innertube_config: true,
    user_agent: navigator.userAgent,
    retrieve_player: withPlayer,
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    fetch: (input: any, init?: any) => fetch(input, init),
    cache: withPlayer ? new UniversalCache(false) : undefined,
    generate_session_locally: true,
  })
}

export async function getSearchSuggestions(query: string): Promise<string[]> {
  if (!searchSession) searchSession = await createInnertube()
  try {
    return await searchSession.getSearchSuggestions(query)
  } catch {
    return []
  }
}

export function clearSearchSession() {
  searchSession = null
}

export async function getVideoInfo(videoId: string) {
  const yt = await createInnertube(true)
  return await yt.getInfo(videoId)
}

export async function getChannel(channelId: string) {
  const yt = await createInnertube()
  return await yt.getChannel(channelId)
}

export async function searchVideos(query: string) {
  const yt = await createInnertube()
  return await yt.search(query)
}

export async function getPlaylist(playlistId: string) {
  const yt = await createInnertube()
  return await yt.getPlaylist(playlistId)
}

export async function getComments(videoId: string) {
  const yt = await createInnertube()
  return await yt.getComments(videoId)
}

export async function getTrending() {
  const yt = await createInnertube()
  // Try multiple approaches for trending
  try {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    return await (yt as any).getTrending()
  } catch {
    try {
      // Fall back to browsing the trending page
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      return await (yt as any).browse({ browseId: 'FEtrending' })
    } catch {
      // Return empty result if trending not available
      return { videos: [], contents: [] }
    }
  }
}

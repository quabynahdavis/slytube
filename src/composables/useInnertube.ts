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

export async function getComments(videoId: string) {
  const yt = await createInnertube()
  return await yt.getComments(videoId)
}

export async function getTrending() {
  return { videos: [], contents: [] } as any
}

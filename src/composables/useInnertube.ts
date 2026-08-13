import { invoke } from '@tauri-apps/api/core'
import { Innertube, UniversalCache } from 'youtubei.js'

// ─── Direct youtubei.js (kept for non-extractor use cases) ────────────────────

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
  // Use extractor (hidden webview) for suggestions
  try {
    const result = await invoke('extract', {
      method: 'getSearchSuggestions',
      params: { query },
    })
    return (result as string[]) || []
  } catch {
    // Fallback: local Innertube session
    if (!searchSession) searchSession = await createInnertube()
    try {
      return await searchSession.getSearchSuggestions(query)
    } catch {
      return []
    }
  }
}

export function clearSearchSession() {
  searchSession = null
}

export async function getComments(videoId: string) {
  // Use extractor (hidden webview) for comments
  try {
    const result = await invoke('extract', {
      method: 'getComments',
      params: { videoId },
    })
    return { comments: (result as any[]) || [] }
  } catch {
    // Fallback: local Innertube session
    const yt = await createInnertube()
    return await yt.getComments(videoId)
  }
}

export async function getTrending() {
  // Use extractor (hidden webview) for trending
  try {
    const result = await invoke('extract', {
      method: 'getTrending',
      params: {},
    })
    return { videos: (result as any[]) || [], contents: [] }
  } catch {
    return { videos: [], contents: [] }
  }
}

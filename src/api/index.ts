import { invoke } from '@tauri-apps/api/core'
import type { Video, Channel, Playlist, Comment } from './types'
import {
  invidiousGetVideo,
  invidiousSearch,
  invidiousGetPopular,
  invidiousGetChannel,
  invidiousGetPlaylist,
  invidiousGetComments,
  invidiousGetDashManifest,
  invidiousGetDashUrl,
  invidiousGetChannelShorts,
  invidiousGetChannelCommunityPosts,
  proxyImageUrl,
  proxyAvatarUrl,
  cacheImageUrl,
  getThumbnailUrl,
} from './invidious'

// ─── Extraction via hidden webview (youtubei.js) ────────────────────────────

/**
 * Extract YouTube data via the hidden webview running youtubei.js.
 * This is the PRIMARY extraction path. Falls back to Invidious on failure.
 */
async function extract(method: string, params: Record<string, unknown>): Promise<unknown> {
  return await invoke('extract', { method, params })
}

// ─── Mapping helpers ─────────────────────────────────────────────────────────

function getBestThumbnail(thumbnails: any[] | undefined): string {
  if (!thumbnails || thumbnails.length === 0) return ''
  const sorted = [...thumbnails].sort((a, b) => (b.width || 0) - (a.width || 0))
  return sorted[0]?.url || ''
}

function getAuthorAvatar(thumbnails: any[] | undefined): string {
  if (!thumbnails || thumbnails.length === 0) return ''
  const sorted = [...thumbnails].sort((a, b) => (b.width || 0) - (a.width || 0))
  return sorted[0]?.url || ''
}

function formatPublished(timestamp: number): string {
  if (!timestamp) return ''
  const now = Math.floor(Date.now() / 1000)
  const diff = now - timestamp
  const days = Math.floor(diff / 86400)
  if (days < 1) return 'Today'
  if (days === 1) return 'Yesterday'
  if (days < 30) return `${days} days ago`
  const months = Math.floor(days / 30)
  if (months < 12) return `${months} month${months > 1 ? 's' : ''} ago`
  const years = Math.floor(months / 12)
  return `${years} year${years > 1 ? 's' : ''} ago`
}

function mapExtractedVideo(v: any): Video {
  if (!v) return { id: '', title: 'Unknown', author: 'Unknown', authorId: '', authorUrl: '', authorAvatar: '', description: '', thumbnail: '', viewCount: 0, likeCount: 0, lengthSeconds: 0, published: '', isLive: false, isUpcoming: false, isShort: false, chapters: [], captions: [], related: [] }

  const videoId = v.id || ''

  const authorAvatarUrl = v.authorAvatar ||
    v.authorThumbnail ||
    (v.author?.thumbnails?.[0]?.url) ||
    (v.authorThumbnails?.[0]?.url) ||
    ''

  return {
    id: videoId,
    title: v.title || 'Unknown',
    author: v.author?.name || v.author || 'Unknown',
    authorId: v.authorId || v.author?.channelId || '',
    authorUrl: `/channel/${v.authorId || ''}`,
    authorAvatar: cacheImageUrl(proxyAvatarUrl(authorAvatarUrl)),
    description: v.description || '',
    thumbnail: cacheImageUrl(proxyImageUrl(v.thumbnail) || getThumbnailUrl(videoId, 'hqdefault')),
    viewCount: Number(v.viewCount) || parseInt(String(v.viewCountText || '').replace(/[^0-9]/g, '')) || 0,
    likeCount: Number(v.likeCount) || 0,
    lengthSeconds: Number(v.lengthSeconds) || 0,
    published: v.published || '',
    isLive: v.liveNow || v.isLive || false,
    isUpcoming: v.isUpcoming || false,
    isShort: v.isShort || false,
    chapters: v.chapters || [],
    captions: v.captions || [],
    related: (v.related || []).map(mapExtractedVideo),
  }
}

function mapExtractedChannel(c: any): Channel {
  if (!c) return { id: '', name: 'Unknown', description: '', avatar: '', banner: '', subscriberCount: 0, videoCount: 0, tabs: [], videos: [], relatedChannels: [] }

  return {
    id: c.id || '',
    name: c.name || 'Unknown',
    description: c.description || '',
    avatar: cacheImageUrl(c.avatar || ''),
    banner: cacheImageUrl(c.banner || ''),
    subscriberCount: c.subscriberCount || 0,
    videoCount: c.videoCount || 0,
    tabs: c.tabs || ['home', 'videos', 'playlists', 'community'],
    videos: (c.videos || []).map(mapExtractedVideo),
    relatedChannels: (c.relatedChannels || []).map(mapExtractedChannel),
    playlists: c.playlists || [],
    relatedPlaylists: c.relatedPlaylists || [],
  }
}

function mapExtractedPlaylist(p: any): Playlist {
  if (!p) return { id: '', title: 'Unknown', description: '', author: '', authorId: '', videoCount: 0, videos: [] }

  return {
    id: p.id || '',
    title: p.title || 'Unknown',
    description: p.description || '',
    author: p.author || '',
    authorId: p.authorId || '',
    videoCount: p.videoCount || 0,
    videos: (p.videos || []).map(mapExtractedVideo),
  }
}

function mapExtractedComment(c: any): Comment {
  if (!c) return { id: '', author: 'Unknown', authorId: '', authorAvatar: '', content: '', likeCount: 0, published: '', replies: [], replyCount: 0 }

  return {
    id: c.id || '',
    author: c.author || 'Unknown',
    authorId: c.authorId || '',
    authorAvatar: cacheImageUrl(c.authorAvatar || ''),
    content: c.content || '',
    likeCount: c.likeCount || 0,
    published: c.published || '',
    replies: (c.replies || []).map(mapExtractedComment),
    replyCount: c.replyCount || 0,
  }
}

// ─── Invidious fallback mappers ──────────────────────────────────────────────

function mapInvidiousVideo(v: any): Video {
  const videoId = v.videoId || ''
  const authorId = v.authorId || ''

  const rawThumbnail = getBestThumbnail(v.videoThumbnails)
  const rawAuthorAvatar = getAuthorAvatar(v.authorThumbnails)

  return {
    id: videoId,
    title: v.title || 'Unknown',
    author: v.author || 'Unknown',
    authorId,
    authorUrl: `/channel/${authorId}`,
    authorAvatar: cacheImageUrl(proxyAvatarUrl(rawAuthorAvatar)),
    description: v.description || '',
    thumbnail: cacheImageUrl(proxyImageUrl(rawThumbnail, videoId)),
    viewCount: Number(v.viewCount) || parseInt(String(v.viewCountText || '').replace(/[^0-9]/g, '')) || 0,
    likeCount: Number(v.likeCount) || 0,
    lengthSeconds: Number(v.lengthSeconds) || 0,
    published: v.publishedText || formatPublished(v.published),
    isLive: v.liveNow || false,
    isUpcoming: v.isUpcoming || false,
    isShort: v.isShort || false,
    chapters: [],
    captions: [],
    related: (v.recommendedVideos || []).map(mapInvidiousVideo),
  }
}

function mapInvidiousChannel(c: any): Channel {
  return {
    id: c.authorId || '',
    name: c.author || 'Unknown',
    description: c.description || '',
    avatar: cacheImageUrl(c.authorThumbnails?.[0]?.url || ''),
    banner: cacheImageUrl(c.authorBanners?.[0]?.url || ''),
    subscriberCount: c.subCount || 0,
    videoCount: c.videoCount || 0,
    tabs: c.tabs || ['home', 'videos', 'playlists', 'community'],
    videos: (c.latestVideos || []).map(mapInvidiousVideo),
    relatedChannels: (c.relatedChannels || []).map(mapInvidiousChannel),
  }
}

function mapInvidiousPlaylist(p: any): Playlist {
  return {
    id: p.playlistId || '',
    title: p.title || 'Unknown',
    description: p.description || '',
    author: p.author || 'Unknown',
    authorId: p.authorId || '',
    videoCount: p.videoCount || 0,
    videos: (p.videos || []).map((v: any) => mapInvidiousVideo({ ...v, videoId: v.videoId || v.id })),
  }
}

function mapInvidiousComment(c: any): Comment {
  return {
    id: c.commentId || '',
    author: c.author || 'Unknown',
    authorId: c.authorId || '',
    authorAvatar: cacheImageUrl(c.authorThumbnails?.[0]?.url || ''),
    content: c.content || '',
    likeCount: Number(c.likeCount) || 0,
    published: c.publishedText || formatPublished(c.published),
    replies: (c.replies?.comments || []).map(mapInvidiousComment),
    replyCount: c.replyCount || 0,
  }
}

// ─── Public API ──────────────────────────────────────────────────────────────

export async function getVideo(videoId: string): Promise<Video> {
  // Primary: extractor (youtubei.js via hidden webview)
  try {
    const result = await extract('getVideoInfo', { videoId })
    return mapExtractedVideo(result)
  } catch (e) {
    console.warn('[API] Extractor failed for getVideo, falling back to Invidious:', e)
  }

  // Fallback: Invidious
  try {
    const result = await invidiousGetVideo(videoId)
    return mapInvidiousVideo(result)
  } catch (e) {
    console.error('[API] All extraction methods failed for getVideo:', e)
    throw new Error(`Failed to load video: ${videoId}`)
  }
}

export async function getVideoPlaybackInfo(videoId: string): Promise<{
  dashUrl: string | null
  formatStreams: any[]
  manifestXml: string | null
}> {
  let dashUrl: string | null = null
  let formatStreams: any[] = []
  let manifestXml: string | null = null

  try {
    dashUrl = await invidiousGetDashUrl(videoId)
  } catch {
    // dashUrl not available
  }

  if (!dashUrl) {
    try {
      const info = await invidiousGetVideo(videoId)
      if (info?.dashUrl) {
        dashUrl = info.dashUrl
      }
      if (info?.formatStreams) {
        formatStreams = info.formatStreams
      }
    } catch {
      // ignore
    }
  }

  if (!dashUrl && formatStreams.length === 0) {
    try {
      manifestXml = await invidiousGetDashManifest(videoId)
    } catch {
      // manifest fetch failed
    }
  }

  return { dashUrl, formatStreams, manifestXml }
}

export async function search(
  query: string,
  filters?: {
    sort?: string
    type?: string
    duration?: string
    date?: string
  }
): Promise<Video[]> {
  // Primary: extractor (youtubei.js via hidden webview)
  try {
    // Map frontend filter names to youtubei.js SearchFilters
    const ytFilters: Record<string, unknown> = {}
    if (filters?.type) ytFilters.type = filters.type
    if (filters?.duration) ytFilters.duration = filters.duration
    if (filters?.date) ytFilters.upload_date = filters.date
    if (filters?.sort) ytFilters.prioritize = filters.sort

    const result = await extract('search', { query, ...ytFilters })
    const items = (result as any[]) || []
    const videos: Video[] = []

    for (const item of items) {
      if (item?.type === 'video' && item.data) {
        videos.push(mapExtractedVideo(item.data))
      }
    }
    return videos
  } catch (e) {
    console.warn('[API] Extractor failed for search, falling back to Invidious:', e)
  }

  // Fallback: Invidious
  try {
    const result = await invidiousSearch(query, 1, filters)
    return (result || []).filter((i: any) => i.type === 'video').map(mapInvidiousVideo)
  } catch (e) {
    console.error('[API] All extraction methods failed for search:', e)
    return []
  }
}

export async function getTrendingVideos(): Promise<Video[]> {
  // Primary: extractor (youtubei.js via hidden webview)
  try {
    const result = await extract('getTrending', {})
    const videos = (result as any[]) || []
    return videos.map(mapExtractedVideo)
  } catch (e) {
    console.warn('[API] Extractor failed for trending, falling back to Invidious:', e)
  }

  // Fallback: Invidious popular
  try {
    const result = await invidiousGetPopular()
    const data = Array.isArray(result) ? result : []
    return data
      .filter((i: any) => i.type === 'video' || i.type === 'shortVideo')
      .map(mapInvidiousVideo)
  } catch (e) {
    console.error('[API] All extraction methods failed for trending:', e)
    return []
  }
}

export async function getPopularVideos(): Promise<Video[]> {
  try {
    const result = await invidiousGetPopular()
    const data = Array.isArray(result) ? result : []
    return data.filter((i: any) => i.type === 'video' || i.type === 'shortVideo').map(mapInvidiousVideo)
  } catch {
    return []
  }
}

export async function getChannelInfo(channelId: string): Promise<Channel> {
  // Primary: extractor (youtubei.js via hidden webview)
  try {
    const result = await extract('getChannel', { channelId, includeHomeShelves: true })
    return mapExtractedChannel(result)
  } catch (e) {
    console.warn('[API] Extractor failed for getChannel, falling back to Invidious:', e)
  }

  // Fallback: Invidious
  try {
    const result = await invidiousGetChannel(channelId)
    return mapInvidiousChannel(result)
  } catch (e) {
    console.error('[API] All extraction methods failed for getChannel:', e)
    return { id: channelId, name: 'Unknown', description: '', avatar: '', banner: '', subscriberCount: 0, videoCount: 0, tabs: [], videos: [], relatedChannels: [] }
  }
}

export async function getPlaylistInfo(playlistId: string): Promise<Playlist> {
  // Primary: extractor (youtubei.js via hidden webview)
  try {
    const result = await extract('getPlaylist', { playlistId })
    return mapExtractedPlaylist(result)
  } catch (e) {
    console.warn('[API] Extractor failed for getPlaylist, falling back to Invidious:', e)
  }

  // Fallback: Invidious
  try {
    const result = await invidiousGetPlaylist(playlistId)
    return mapInvidiousPlaylist(result)
  } catch {
    return { id: playlistId, title: 'Playlist unavailable', description: '', author: '', authorId: '', videoCount: 0, videos: [] }
  }
}

export async function getCommentsInfo(videoId: string): Promise<Comment[]> {
  // Primary: extractor (youtubei.js via hidden webview)
  try {
    const result = await extract('getComments', { videoId })
    const comments = (result as any[]) || []
    return comments.map(mapExtractedComment)
  } catch (e) {
    console.warn('[API] Extractor failed for getComments, falling back to Invidious:', e)
  }

  // Fallback: Invidious
  try {
    const result = await invidiousGetComments(videoId)
    return (result.comments || []).map(mapInvidiousComment)
  } catch {
    return []
  }
}

export async function getChannelShorts(channelId: string): Promise<Video[]> {
  // Primary: extractor (youtubei.js via hidden webview)
  try {
    const result = await extract('getChannelShorts', { channelId })
    const shorts = (result as any)?.videos || []
    return shorts.map(mapExtractedVideo)
  } catch (e) {
    console.warn('[API] Extractor failed for getChannelShorts, falling back to Invidious:', e)
  }

  // Fallback: Invidious
  try {
    const result = await invidiousGetChannelShorts(channelId)
    const shorts = result.videos || result.shorts || []
    return shorts.map((s: any) => mapInvidiousVideo({ ...s, videoId: s.videoId || s.id, isShort: true }))
  } catch {
    return []
  }
}

export async function getChannelLiveVideos(channelId: string): Promise<Video[]> {
  // Primary: extractor (youtubei.js via hidden webview)
  try {
    const result = await extract('getChannelLive', { channelId })
    const live = (result as any)?.videos || []
    return live.map(mapExtractedVideo)
  } catch (e) {
    console.warn('[API] Extractor failed for getChannelLive:', e)
  }
  return []
}

export async function getChannelCommunityPosts(channelId: string): Promise<any[]> {
  // Primary: extractor (youtubei.js via hidden webview)
  try {
    const result = await extract('getChannelCommunity', { channelId })
    const posts = (result as any)?.posts || []
    return posts
  } catch (e) {
    console.warn('[API] Extractor failed for getChannelCommunity, falling back to Invidious:', e)
  }

  // Fallback: Invidious
  try {
    const result = await invidiousGetChannelCommunityPosts(channelId)
    return result.comments || result.posts || []
  } catch {
    return []
  }
}

export async function getChannelPlaylists(channelId: string): Promise<Playlist[]> {
  // Primary: extractor (youtubei.js via hidden webview)
  try {
    const result = await extract('getChannelPlaylists', { channelId })
    const playlists = (result as any)?.playlists || []
    return playlists.map((p: any) => mapExtractedPlaylist(p))
  } catch (e) {
    console.warn('[API] Extractor failed for getChannelPlaylists:', e)
  }
  return []
}

export async function getCommentReplies(videoId: string, commentId: string): Promise<Comment[]> {
  // Primary: extractor (youtubei.js via hidden webview)
  try {
    const result = await extract('getCommentReplies', { videoId, commentId })
    const replies = (result as any[]) || []
    return replies.map(mapExtractedComment)
  } catch (e) {
    console.warn('[API] Extractor failed for getCommentReplies:', e)
  }
  return []
}

export async function getHashtagVideos(hashtag: string): Promise<Video[]> {
  // Primary: extractor (youtubei.js via hidden webview)
  try {
    const result = await extract('getHashtag', { hashtag })
    const videos = (result as any[]) || []
    return videos.map(mapExtractedVideo)
  } catch (e) {
    console.warn('[API] Extractor failed for getHashtag:', e)
  }
  return []
}

export async function getSubscribedChannelsShorts(channelIds: string[]): Promise<Video[]> {
  const allShorts: Video[] = []
  const promises = channelIds.slice(0, 5).map(async (channelId) => {
    try {
      const shorts = await getChannelShorts(channelId)
      return shorts.slice(0, 5)
    } catch {
      return []
    }
  })
  const results = await Promise.allSettled(promises)
  for (const result of results) {
    if (result.status === 'fulfilled') {
      allShorts.push(...result.value)
    }
  }
  return allShorts
}

export async function getSubscribedChannelsPosts(channelIds: string[]): Promise<any[]> {
  const allPosts: any[] = []
  const promises = channelIds.slice(0, 5).map(async (channelId) => {
    try {
      const posts = await getChannelCommunityPosts(channelId)
      return posts.slice(0, 3)
    } catch {
      return []
    }
  })
  const results = await Promise.allSettled(promises)
  for (const result of results) {
    if (result.status === 'fulfilled') {
      allPosts.push(...result.value)
    }
  }
  return allPosts
}

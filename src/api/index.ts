import { invoke } from '@tauri-apps/api/core'
import type { Video, Channel, Playlist, Comment } from './types'

// ============================================================================
// Utility Functions
// ============================================================================

function getBestThumbnail(thumbnails: any[] | undefined): string {
  if (!thumbnails || thumbnails.length === 0) return ''
  const best = thumbnails.sort((a: any, b: any) => (b.width || 0) - (a.width || 0))[0]
  return best?.url || ''
}

function parseViewCount(text: string | undefined): number {
  if (!text) return 0
  const cleaned = text.replace(/[^0-9.KMkB]/g, '')
  const multiplier = cleaned.includes('K') ? 1000 : cleaned.includes('M') ? 1000000 : cleaned.includes('B') ? 1000000000 : 1
  const num = parseFloat(cleaned.replace(/[KMkB]/g, '')) || 0
  return Math.floor(num * multiplier)
}

// ============================================================================
// Invidious Mappers
// ============================================================================

function mapInvidiousVideo(v: any): Video {
  return {
    id: v.videoId || '',
    title: v.title || 'Unknown',
    author: v.author || 'Unknown',
    authorId: v.authorId || '',
    authorUrl: `/channel/${v.authorId || ''}`,
    description: v.description || '',
    thumbnail: v.videoThumbnails?.[0]?.url || '',
    viewCount: v.viewCount || 0,
    likeCount: v.likeCount || 0,
    lengthSeconds: v.lengthSeconds || 0,
    published: v.publishedText || '',
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
    avatar: c.authorThumbnails?.[0]?.url || '',
    banner: '',
    subscriberCount: c.subCount || 0,
    videoCount: c.videoCount || 0,
    tabs: c.tabs || ['home', 'videos', 'playlists', 'community'],
    videos: (c.latestVideos || []).map(mapInvidiousVideo),
    relatedChannels: [],
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
    authorAvatar: c.authorThumbnails?.[0]?.url || '',
    content: c.content || '',
    likeCount: c.likeCount || 0,
    published: c.publishedText || '',
    replies: (c.replies?.comments || []).map(mapInvidiousComment),
    replyCount: c.replyCount || 0,
  }
}

// ============================================================================
// InnerTube / YouTube Mappers
// ============================================================================

function mapYouTubeResponse(result: any): Video {
  const details = result.videoDetails || {}
  const author = details.author || {}
  const thumbnail = details.thumbnail?.thumbnails

  return {
    id: details.videoId || '',
    title: details.title || 'Unknown',
    author: typeof author === 'string' ? author : (author.name || 'Unknown'),
    authorId: details.channelId || '',
    authorUrl: `/channel/${details.channelId || ''}`,
    description: details.shortDescription || '',
    thumbnail: getBestThumbnail(thumbnail),
    viewCount: parseInt(details.viewCount || '0'),
    likeCount: parseInt(result.likes || '0'),
    lengthSeconds: parseInt(details.lengthSeconds || '0'),
    published: '',
    isLive: details.isLive || false,
    isUpcoming: false,
    isShort: false,
    chapters: [],
    captions: [],
    related: [],
  }
}

function mapYouTubeSearchResults(result: any): Video[] {
  const contents = result.contents?.twoColumnSearchResultsRenderer?.primaryContents
    ?.sectionListRenderer?.contents || []

  const videos: Video[] = []
  for (const section of contents) {
    const items = section.itemSectionRenderer?.contents || []
    for (const item of items) {
      if (item.videoRenderer) {
        const vr = item.videoRenderer
        videos.push({
          id: vr.videoId || '',
          title: vr.title?.runs?.[0]?.text || vr.title?.simpleText || 'Unknown',
          author: vr.ownerText?.runs?.[0]?.text || 'Unknown',
          authorId: vr.ownerText?.runs?.[0]?.navigationEndpoint?.browseEndpoint?.browseId || '',
          authorUrl: '',
          description: vr.descriptionSnippet?.runs?.[0]?.text || '',
          thumbnail: getBestThumbnail(vr.thumbnail?.thumbnails),
          viewCount: parseViewCount(vr.viewCountText?.simpleText),
          likeCount: 0,
          lengthSeconds: 0,
          published: vr.publishedTimeText?.simpleText || '',
          isLive: false,
          isUpcoming: false,
          isShort: false,
          chapters: [],
          captions: [],
          related: [],
        })
      }
    }
  }
  return videos
}

function mapYouTubeChannel(result: any): Channel {
  const metadata = result.metadata?.channelMetadataRenderer
  const headerData = result.header?.c4TabbedHeaderRenderer

  // Extract tabs from the channel header
  const tabs: string[] = []
  if (result.contents?.twoColumnBrowseResultsRenderer?.tabs) {
    for (const tab of result.contents.twoColumnBrowseResultsRenderer.tabs) {
      if (tab.tabRenderer) {
        const title = tab.tabRenderer.title || ''
        const titleLower = title.toLowerCase()
        if (titleLower.includes('home')) tabs.push('home')
        else if (titleLower.includes('video')) tabs.push('videos')
        else if (titleLower.includes('short')) tabs.push('shorts')
        else if (titleLower.includes('live') || titleLower.includes('stream')) tabs.push('live')
        else if (titleLower.includes('playlist')) tabs.push('playlists')
        else if (titleLower.includes('community')) tabs.push('community')
        else if (titleLower.includes('channel')) tabs.push('channels')
        else if (titleLower.includes('about')) tabs.push('about')
      }
    }
  }

  return {
    id: metadata?.externalId || '',
    name: metadata?.title || 'Unknown',
    description: metadata?.description || '',
    avatar: getBestThumbnail(metadata?.avatar?.thumbnails),
    banner: getBestThumbnail(headerData?.banner?.thumbnails),
    subscriberCount: parseViewCount(headerData?.subscriberCountText?.simpleText),
    videoCount: 0,
    tabs: tabs.length > 0 ? tabs : ['home', 'videos', 'playlists', 'community'],
    videos: [],
    relatedChannels: [],
  }
}

// ============================================================================
// Core Video & Playback
// ============================================================================

export async function getVideo(videoId: string): Promise<Video> {
  try {
    const result = await invoke('get_video_info', { videoId })
    return mapYouTubeResponse(result as any)
  } catch {
    const result = await invoke('invidious_get_video', { videoId })
    return mapInvidiousVideo(result as any)
  }
}

export async function getVideoWithPoToken(videoId: string, potoken: string): Promise<Video> {
  try {
    const result = await invoke('get_video_info', { videoId, potoken })
    return mapYouTubeResponse(result as any)
  } catch {
    const result = await invoke('invidious_get_video', { videoId })
    return mapInvidiousVideo(result as any)
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
    dashUrl = await invoke<string>('invidious_get_dash_url', { videoId })
  } catch {
    // dashUrl not available
  }

  if (!dashUrl) {
    try {
      const info = await invoke<any>('invidious_get_video', { videoId })
      if (info?.dashUrl) {
        dashUrl = info.dashUrl
      }
    } catch {
      // ignore
    }
  }

  try {
    formatStreams = await invoke<any[]>('invidious_get_format_streams', { videoId })
  } catch {
    // format streams not available
  }

  if (!dashUrl && formatStreams.length === 0) {
    try {
      manifestXml = await invoke<string>('invidious_get_dash_manifest', { videoId })
    } catch {
      // manifest fetch failed
    }
  }

  return { dashUrl, formatStreams, manifestXml }
}

// ============================================================================
// Search
// ============================================================================

export interface SearchFilters {
  duration?: 'short' | 'medium' | 'long'
  sort?: 'relevance' | 'rating' | 'date' | 'views'
  date?: 'hour' | 'today' | 'week' | 'month' | 'year'
  type?: 'video' | 'channel' | 'playlist' | 'movie' | 'all'
  features?: string[] // 'hd', 'subtitles', 'creative-commons', '3d', 'live', 'purchased', '4k', '360', 'location', 'hdr', 'vr180'
  params?: string // Base64-encoded protobuf params for direct API usage
}

export async function search(query: string, filters?: SearchFilters): Promise<Video[]> {
  try {
    const result = await invoke('search_videos', {
      query,
      filters: filters ? {
        params: filters.params,
      } : null,
    })
    return mapYouTubeSearchResults(result as any)
  } catch {
    const result = await invoke('invidious_search_with_filters', {
      query,
      searchParams: filters ? {
        duration: filters.duration,
        sort: filters.sort,
        date: filters.date,
        type: filters.type,
        features: filters.features,
      } : null,
    })
    return (result as any[] || []).filter((i: any) => i.type === 'video').map(mapInvidiousVideo)
  }
}

export async function searchWithContinuation(query: string, continuationToken: string): Promise<{
  videos: Video[]
  continuation: string | null
}> {
  try {
    const result = await invoke('search_videos', { query, continuation: continuationToken })
    return {
      videos: mapYouTubeSearchResults(result as any),
      continuation: (result as any).continuationContents?.itemSectionContinuation?.continuations?.[0]?.nextContinuationData?.continuation || null,
    }
  } catch {
    return { videos: [], continuation: null }
  }
}

export async function getSearchSuggestions(query: string): Promise<string[]> {
  try {
    const result = await invoke<string[]>('get_search_suggestions', { query })
    // Response format: [query, [suggestions], ...]
    if (Array.isArray(result) && result.length > 1 && Array.isArray(result[1])) {
      return result[1].map((s: any) => s[0] || '').filter(Boolean)
    }
    return []
  } catch {
    try {
      const result = await invoke<{ suggestions: string[] }>('invidious_get_search_suggestions', { query })
      return result.suggestions || []
    } catch {
      return []
    }
  }
}

// ============================================================================
// Trending & Popular
// ============================================================================

export type TrendingCategory = 'all' | 'music' | 'gaming' | 'movies'

export async function getTrendingVideos(category?: TrendingCategory): Promise<Video[]> {
  try {
    const result = await invoke('get_trending', { category: category === 'all' ? null : category })
    const videos = parseYouTubeFeedVideos(result as any)
    return videos
  } catch {
    try {
      const result = await invoke('invidious_get_trending')
      const data = (result as any[]) || []
      return data.filter((i: any) => i.type === 'video').map(mapInvidiousVideo)
    } catch {
      return []
    }
  }
}

function parseYouTubeFeedVideos(result: any): Video[] {
  const videos: Video[] = []
  const contents = result.contents?.twoColumnBrowseResultsRenderer?.tabs?.[0]?.tabRenderer?.content
    ?.richGridRenderer?.contents || result.contents?.sectionListRenderer?.contents || []

  for (const section of contents) {
    // Rich grid items
    if (section.richItemRenderer?.content?.videoRenderer) {
      const vr = section.richItemRenderer.content.videoRenderer
      videos.push({
        id: vr.videoId || '',
        title: vr.title?.runs?.[0]?.text || vr.title?.simpleText || 'Unknown',
        author: vr.ownerText?.runs?.[0]?.text || 'Unknown',
        authorId: vr.ownerText?.runs?.[0]?.navigationEndpoint?.browseEndpoint?.browseId || '',
        authorUrl: '',
        description: '',
        thumbnail: getBestThumbnail(vr.thumbnail?.thumbnails),
        viewCount: parseViewCount(vr.viewCountText?.simpleText),
        likeCount: 0,
        lengthSeconds: 0,
        published: vr.publishedTimeText?.simpleText || '',
        isLive: false,
        isUpcoming: false,
        isShort: false,
        chapters: [],
        captions: [],
        related: [],
      })
    }
    // Section items (Invidious-style fallback)
    if (section.itemSectionRenderer?.contents) {
      for (const item of section.itemSectionRenderer.contents) {
        if (item.videoRenderer) {
          const vr = item.videoRenderer
          videos.push({
            id: vr.videoId || '',
            title: vr.title?.runs?.[0]?.text || vr.title?.simpleText || 'Unknown',
            author: vr.ownerText?.runs?.[0]?.text || 'Unknown',
            authorId: vr.ownerText?.runs?.[0]?.navigationEndpoint?.browseEndpoint?.browseId || '',
            authorUrl: '',
            description: '',
            thumbnail: getBestThumbnail(vr.thumbnail?.thumbnails),
            viewCount: parseViewCount(vr.viewCountText?.simpleText),
            likeCount: 0,
            lengthSeconds: 0,
            published: vr.publishedTimeText?.simpleText || '',
            isLive: false,
            isUpcoming: false,
            isShort: false,
            chapters: [],
            captions: [],
            related: [],
          })
        }
      }
    }
  }
  return videos
}

export async function getPopularVideos(): Promise<Video[]> {
  try {
    const result = await invoke('invidious_get_popular')
    const data = (result as any[]) || []
    return data.filter((i: any) => i.type === 'video' || i.type === 'shortVideo').map(mapInvidiousVideo)
  } catch {
    return []
  }
}

// ============================================================================
// Channel
// ============================================================================

export async function getChannelInfo(channelId: string): Promise<Channel> {
  try {
    const result = await invoke('get_channel_info', { channelId })
    return mapYouTubeChannel(result as any)
  } catch {
    const result = await invoke('invidious_get_channel', { channelId })
    return mapInvidiousChannel(result as any)
  }
}

export async function getChannelInfoWithTab(channelId: string, tab?: string): Promise<Channel> {
  try {
    const result = await invoke('get_channel_info', { channelId, tab })
    const channel = mapYouTubeChannel(result as any)
    // Also extract videos from the tab content if available
    channel.videos = parseYouTubeFeedVideos(result as any)
    return channel
  } catch {
    const result = await invoke('invidious_get_channel', { channelId })
    return mapInvidiousChannel(result as any)
  }
}

export async function getChannelVideos(channelId: string, continuation?: string): Promise<{
  videos: Video[]
  continuation: string | null
}> {
  try {
    const result = await invoke('get_channel_videos', { channelId, continuation })
    return {
      videos: parseYouTubeFeedVideos(result as any),
      continuation: (result as any).onResponseReceivedEndpoints?.[0]?.appendContinuationItemsAction?.continuationItems?.[0]?.continuationItemRenderer?.continuationEndpoint?.continuationCommand?.token || null,
    }
  } catch {
    const result = await invoke('invidious_get_channel_videos', { channelId })
    return {
      videos: ((result as any).videos || []).map(mapInvidiousVideo),
      continuation: null,
    }
  }
}

export async function getChannelShorts(channelId: string): Promise<Video[]> {
  try {
    const result = await invoke('get_channel_info', { channelId, tab: 'shorts' })
    return parseYouTubeFeedVideos(result as any).map(v => ({ ...v, isShort: true }))
  } catch {
    try {
      const result = await invoke('invidious_get_channel_shorts', { channelId })
      return ((result as any).videos || []).map(mapInvidiousVideo)
    } catch {
      return []
    }
  }
}

export async function getChannelLive(channelId: string): Promise<Video[]> {
  try {
    const result = await invoke('get_channel_info', { channelId, tab: 'live' })
    return parseYouTubeFeedVideos(result as any).map(v => ({ ...v, isLive: true }))
  } catch {
    try {
      const result = await invoke('invidious_get_channel_live', { channelId })
      return ((result as any).videos || []).map(mapInvidiousVideo)
    } catch {
      return []
    }
  }
}

export async function getChannelPlaylists(channelId: string): Promise<Playlist[]> {
  try {
    const result = await invoke('invidious_get_channel_playlists', { channelId })
    return ((result as any).playlists || []).map(mapInvidiousPlaylist)
  } catch {
    return []
  }
}

export async function getChannelReleases(channelId: string): Promise<Playlist[]> {
  try {
    const result = await invoke('invidious_get_channel_releases', { channelId })
    return ((result as any).playlists || []).map(mapInvidiousPlaylist)
  } catch {
    return []
  }
}

export async function getChannelPodcasts(channelId: string): Promise<Playlist[]> {
  try {
    const result = await invoke('invidious_get_channel_podcasts', { channelId })
    return ((result as any).playlists || []).map(mapInvidiousPlaylist)
  } catch {
    return []
  }
}

export async function getChannelCourses(channelId: string): Promise<Playlist[]> {
  try {
    const result = await invoke('invidious_get_channel_courses', { channelId })
    return ((result as any).courses || []).map(mapInvidiousPlaylist)
  } catch {
    return []
  }
}

export async function searchChannel(channelId: string, query: string): Promise<Video[]> {
  try {
    const result = await invoke('invidious_search_channel', { channelId, query })
    return ((result as any).videos || []).map(mapInvidiousVideo)
  } catch {
    return []
  }
}

export async function getChannelTabs(channelId: string): Promise<string[]> {
  try {
    const result = await invoke<any>('invidious_get_channel_tabs', { channelId })
    return result.tabs || ['home', 'videos', 'playlists', 'community']
  } catch {
    try {
      const channel = await getChannelInfo(channelId)
      return channel.tabs
    } catch {
      return ['home', 'videos', 'playlists', 'community']
    }
  }
}

// ============================================================================
// URL Resolution
// ============================================================================

export async function resolveUrl(url: string): Promise<{
  type: 'video' | 'channel' | 'playlist' | 'hashtag' | 'unknown'
  id: string
} | null> {
  try {
    const result = await invoke<any>('invidious_resolve_url', { url })
    if (result.pageType === 'video') {
      return { type: 'video', id: result.videoId || result.ucid || '' }
    }
    if (result.pageType === 'channel') {
      return { type: 'channel', id: result.ucid || result.browseId || '' }
    }
    if (result.pageType === 'playlist') {
      return { type: 'playlist', id: result.playlistId || '' }
    }
    return { type: 'unknown', id: '' }
  } catch {
    // Fallback: try to parse the URL directly
    return parseUrlFallback(url)
  }
}

function parseUrlFallback(url: string): { type: 'video' | 'channel' | 'playlist' | 'hashtag' | 'unknown'; id: string } | null {
  try {
    const urlObj = new URL(url)
    const hostname = urlObj.hostname

    if (!hostname.includes('youtube.com') && !hostname.includes('youtu.be')) {
      return null
    }

    // youtu.be short links
    if (hostname === 'youtu.be') {
      const videoId = urlObj.pathname.slice(1)
      if (videoId) return { type: 'video', id: videoId }
    }

    // Watch URLs
    if (urlObj.pathname === '/watch') {
      const videoId = urlObj.searchParams.get('v')
      if (videoId) return { type: 'video', id: videoId }
      const playlistId = urlObj.searchParams.get('list')
      if (playlistId) return { type: 'playlist', id: playlistId }
    }

    // Channel URLs
    if (urlObj.pathname.startsWith('/channel/')) {
      const channelId = urlObj.pathname.split('/')[2]
      if (channelId) return { type: 'channel', id: channelId }
    }

    // Handle URLs (@username)
    if (urlObj.pathname.startsWith('/@')) {
      return { type: 'channel', id: urlObj.pathname }
    }

    // Playlist URLs
    if (urlObj.pathname === '/playlist') {
      const playlistId = urlObj.searchParams.get('list')
      if (playlistId) return { type: 'playlist', id: playlistId }
    }

    // Hashtag URLs
    if (urlObj.pathname === '/hashtag') {
      const tag = urlObj.pathname.split('/').pop()
      if (tag) return { type: 'hashtag', id: tag }
    }

    return { type: 'unknown', id: '' }
  } catch {
    return null
  }
}

// ============================================================================
// Comments
// ============================================================================

export async function getCommentsInfo(videoId: string): Promise<Comment[]> {
  try {
    const result = await invoke('invidious_get_comments', { videoId })
    return ((result as any).comments || []).map(mapInvidiousComment)
  } catch {
    return []
  }
}

export async function getCommentsWithPagination(videoId: string, continuation?: string): Promise<{
  comments: Comment[]
  continuation: string | null
}> {
  try {
    const result = await invoke<any>('get_comments', { videoId, continuation })
    const comments: Comment[] = []
    const endpoints = result.onResponseReceivedEndpoints || []
    for (const endpoint of endpoints) {
      if (endpoint.reloadContinuationItemsCommand?.continuationItems) {
        for (const item of endpoint.reloadContinuationItemsCommand.continuationItems) {
          if (item.commentThreadRenderer) {
            const comment = item.commentThreadRenderer.comment?.commentRenderer
            if (comment) {
              comments.push({
                id: comment.commentId || '',
                author: comment.authorText?.simpleText || 'Unknown',
                authorId: comment.authorEndpoint?.browseEndpoint?.browseId || '',
                authorAvatar: getBestThumbnail(comment.authorThumbnail?.thumbnails),
                content: comment.contentText?.runs?.map((r: any) => r.text).join('') || '',
                likeCount: parseViewCount(comment.voteCount?.simpleText),
                published: comment.publishedTimeText?.runs?.[0]?.text || '',
                replies: [],
                replyCount: parseInt(comment.replyCount?.simpleText || '0'),
              })
            }
          }
        }
      }
    }
    return {
      comments,
      continuation: result.continuationContents?.commentContinuation?.continuations?.[0]?.nextContinuationData?.continuation || null,
    }
  } catch {
    const comments = await getCommentsInfo(videoId)
    return { comments, continuation: null }
  }
}

export async function getCommentReplies(videoId: string, commentId: string): Promise<Comment[]> {
  try {
    const result = await invoke('invidious_get_comment_replies', { videoId, commentId })
    return ((result as any).comments || []).map(mapInvidiousComment)
  } catch {
    return []
  }
}

// ============================================================================
// Playlist
// ============================================================================

export async function getPlaylistInfo(playlistId: string): Promise<Playlist> {
  try {
    const result = await invoke<any>('get_playlist_info', { playlistId })
    const videos = parseYouTubeFeedVideos(result)
    return {
      id: playlistId,
      title: result.metadata?.playlistMetadataRenderer?.title || result.header?.playlistHeaderRenderer?.title?.simpleText || 'Unknown',
      description: result.metadata?.playlistMetadataRenderer?.description || '',
      author: result.header?.playlistHeaderRenderer?.ownerText?.runs?.[0]?.text || '',
      authorId: result.header?.playlistHeaderRenderer?.ownerText?.runs?.[0]?.navigationEndpoint?.browseEndpoint?.browseId || '',
      videoCount: videos.length,
      videos,
    }
  } catch {
    try {
      const result = await invoke<any>('invidious_get_playlist', { playlistId })
      return mapInvidiousPlaylist(result)
    } catch {
      return { id: playlistId, title: 'Playlist unavailable', description: '', author: '', authorId: '', videoCount: 0, videos: [] }
    }
  }
}

export async function getPlaylistContinuation(playlistId: string, continuation: string): Promise<{
  videos: Video[]
  continuation: string | null
}> {
  try {
    const result = await invoke('get_playlist_info', { playlistId, continuation })
    return {
      videos: parseYouTubeFeedVideos(result as any),
      continuation: (result as any).continuationContents?.continuation || null,
    }
  } catch {
    return { videos: [], continuation: null }
  }
}

// ============================================================================
// Community Posts
// ============================================================================

export interface CommunityPost {
  id: string
  author: string
  authorId: string
  authorAvatar: string
  content: string
  likeCount: number
  commentCount: number
  published: string
  imageUrls: string[]
  poll?: { question: string; choices: string[] }
}

export async function getCommunityPosts(channelId: string, continuation?: string): Promise<{
  posts: CommunityPost[]
  continuation: string | null
}> {
  try {
    const result = await invoke<any>('get_community_posts', { channelId, continuation })
    const posts: CommunityPost[] = []
    const contents = result.contents?.twoColumnBrowseResultsRenderer?.tabs?.[3]?.tabRenderer?.content
      ?.sectionListRenderer?.contents || []

    for (const section of contents) {
      if (section.itemSectionRenderer?.contents) {
        for (const item of section.itemSectionRenderer.contents) {
          if (item.backstagePostThreadRenderer?.post?.backstagePostRenderer) {
            const post = item.backstagePostThreadRenderer.post.backstagePostRenderer
            posts.push({
              id: post.postId || '',
              author: post.authorText?.simpleText || 'Unknown',
              authorId: post.authorEndpoint?.browseEndpoint?.browseId || '',
              authorAvatar: getBestThumbnail(post.authorThumbnail?.thumbnails),
              content: post.contentText?.runs?.map((r: any) => r.text).join('') || '',
              likeCount: parseViewCount(post.voteCount?.simpleText),
              commentCount: parseInt(post.actionButtons?.commentActionButtonsRenderer?.replyButton?.buttonRenderer?.text?.simpleText || '0'),
              published: post.publishedTimeText?.runs?.[0]?.text || '',
              imageUrls: (post.backstageAttachment?.backstageImageRenderer?.image?.thumbnails || []).map((t: any) => t.url),
            })
          }
        }
      }
    }

    return {
      posts,
      continuation: result.continuationContents?.sectionListContinuation?.continuations?.[0]?.nextContinuationData?.continuation || null,
    }
  } catch {
    try {
      const result = await invoke<any>('invidious_get_community_posts', { channelId })
      const posts = ((result as any).comments || []).map((p: any): CommunityPost => ({
        id: p.postId || p.commentId || '',
        author: p.author || 'Unknown',
        authorId: p.authorId || '',
        authorAvatar: p.authorThumbnails?.[0]?.url || '',
        content: p.content || '',
        likeCount: p.likeCount || 0,
        commentCount: p.commentCount || 0,
        published: p.publishedText || '',
        imageUrls: [],
      }))
      return { posts, continuation: null }
    } catch {
      return { posts: [], continuation: null }
    }
  }
}

export async function getCommunityPost(channelId: string, postId: string): Promise<CommunityPost | null> {
  try {
    const result = await invoke<any>('invidious_get_community_post', { channelId, postId })
    const post = result.post || result
    return {
      id: post.postId || post.commentId || postId,
      author: post.author || 'Unknown',
      authorId: post.authorId || '',
      authorAvatar: post.authorThumbnails?.[0]?.url || '',
      content: post.content || '',
      likeCount: post.likeCount || 0,
      commentCount: post.commentCount || 0,
      published: post.publishedText || '',
      imageUrls: [],
    }
  } catch {
    return null
  }
}

export async function getCommunityPostComments(channelId: string, postId: string): Promise<Comment[]> {
  try {
    const result = await invoke<any>('invidious_get_community_post_comments', { channelId, postId })
    return ((result as any).comments || []).map(mapInvidiousComment)
  } catch {
    return []
  }
}

export async function getCommunityPostCommentReplies(channelId: string, postId: string, commentId: string): Promise<Comment[]> {
  try {
    const result = await invoke<any>('invidious_get_community_post_comment_replies', { channelId, postId, commentId })
    return ((result as any).comments || []).map(mapInvidiousComment)
  } catch {
    return []
  }
}

// ============================================================================
// Hashtag
// ============================================================================

export async function getHashtagVideos(hashtag: string): Promise<Video[]> {
  try {
    const result = await invoke<any>('get_hashtag', { hashtag })
    return parseYouTubeFeedVideos(result)
  } catch {
    try {
      const result = await invoke<any>('invidious_get_hashtag', { hashtag })
      return ((result as any).videos || []).map(mapInvidiousVideo)
    } catch {
      return []
    }
  }
}

// ============================================================================
// Invidious Instance Management
// ============================================================================

export async function getInvidiousInstances(): Promise<any[]> {
  try {
    const result = await invoke('invidious_get_instances')
    return (result as any) || []
  } catch {
    return []
  }
}

export async function testInvidiousInstance(instanceUrl: string): Promise<boolean> {
  try {
    return await invoke<boolean>('invidious_test_instance', { instanceUrl })
  } catch {
    return false
  }
}

import { invoke } from '@tauri-apps/api/core'
import type { Video, Channel, Playlist, Comment } from './types'
import {
  invidiousGetVideo,
  invidiousSearch,
  invidiousGetTrending,
  invidiousGetPopular,
  invidiousGetChannel,
  invidiousGetPlaylist,
  invidiousGetComments,
  invidiousGetDashManifest,
  invidiousGetDashUrl,
} from './invidious'

function getBestThumbnail(thumbnails: any[] | undefined): string {
  if (!thumbnails || thumbnails.length === 0) return ''
  const best = thumbnails.sort((a: any, b: any) => (b.width || 0) - (a.width || 0))[0]
  return best?.url || ''
}

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
    banner: c.authorBanners?.[0]?.url || '',
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
    authorAvatar: c.authorThumbnails?.[0]?.url || '',
    content: c.content || '',
    likeCount: c.likeCount || 0,
    published: c.publishedText || '',
    replies: (c.replies?.comments || []).map(mapInvidiousComment),
    replyCount: c.replyCount || 0,
  }
}

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
          viewCount: parseInt(vr.viewCountText?.simpleText?.replace(/[^0-9]/g, '') || '0'),
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

  return {
    id: metadata?.externalId || '',
    name: metadata?.title || 'Unknown',
    description: metadata?.description || '',
    avatar: getBestThumbnail(metadata?.avatar?.thumbnails),
    banner: getBestThumbnail(headerData?.banner?.thumbnails),
    subscriberCount: parseInt(headerData?.subscriberCountText?.simpleText?.replace(/[^0-9]/g, '') || '0'),
    videoCount: 0,
    tabs: ['home', 'videos', 'playlists', 'community'],
    videos: [],
    relatedChannels: [],
  }
}

export async function getVideo(videoId: string): Promise<Video> {
  try {
    const result = await invoke('get_video_info', { videoId })
    return mapYouTubeResponse(result as any)
  } catch {
    const result = await invidiousGetVideo(videoId)
    return mapInvidiousVideo(result)
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

export async function search(query: string): Promise<Video[]> {
  try {
    const result = await invoke('search_videos', { query })
    return mapYouTubeSearchResults(result as any)
  } catch {
    const result = await invidiousSearch(query)
    return (result || []).filter((i: any) => i.type === 'video').map(mapInvidiousVideo)
  }
}

export async function getTrendingVideos(): Promise<Video[]> {
  try {
    const result = await invidiousGetTrending('default')
    const data = Array.isArray(result) ? result : []
    return data.filter((i: any) => i.type === 'video' || i.type === 'shortVideo').map(mapInvidiousVideo)
  } catch {
    try {
      const result = await invidiousGetPopular()
      const data = Array.isArray(result) ? result : []
      return data.filter((i: any) => i.type === 'video' || i.type === 'shortVideo').map(mapInvidiousVideo)
    } catch {
      return []
    }
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
  try {
    const result = await invoke('get_channel_info', { channelId })
    return mapYouTubeChannel(result as any)
  } catch {
    const result = await invidiousGetChannel(channelId)
    return mapInvidiousChannel(result)
  }
}

export async function getPlaylistInfo(playlistId: string): Promise<Playlist> {
  try {
    const result = await invidiousGetPlaylist(playlistId)
    return mapInvidiousPlaylist(result)
  } catch {
    return { id: playlistId, title: 'Playlist unavailable', description: '', author: '', authorId: '', videoCount: 0, videos: [] }
  }
}

export async function getCommentsInfo(videoId: string): Promise<Comment[]> {
  try {
    const result = await invidiousGetComments(videoId)
    return (result.comments || []).map(mapInvidiousComment)
  } catch {
    return []
  }
}

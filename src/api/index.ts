import { getVideoInfo, getChannel, searchVideos as searchLocal, getPlaylist, getComments, getTrending } from '../composables/useInnertube'
import * as inv from './invidious'
import type { Video, Channel, Playlist, Comment } from './types'

// eslint-disable-next-line @typescript-eslint/no-explicit-any
function getBestThumbnail(thumbnails: any[] | undefined): string {
  if (!thumbnails || thumbnails.length === 0) return ''
  const best = thumbnails.sort((a: any, b: any) => (b.width || 0) - (a.width || 0))[0]
  return best?.url || ''
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
function mapYouTubeVideo(v: any): Video {
  const author = v.author || v.channel || {}
  return {
    id: v.id || '',
    title: v.title?.text || v.title || 'Unknown',
    author: author.name || author.author || 'Unknown',
    authorId: author.id || author.channelId || '',
    authorUrl: author.channel_url || author.url || '',
    description: v.description || '',
    thumbnail: getBestThumbnail(v.thumbnails),
    viewCount: v.view_count || v.views || 0,
    likeCount: v.like_count || v.likes || 0,
    lengthSeconds: v.duration?.seconds || v.length_seconds || 0,
    published: v.published?.text || v.published || '',
    isLive: v.is_live || v.isLive || false,
    isUpcoming: v.is_upcoming || v.isUpcoming || false,
    isShort: v.is_short || v.isShort || false,
    chapters: (v.chapters || []).map((c: any) => ({
      title: c.title?.text || c.title || '',
      startSeconds: c.start_seconds || 0,
      thumbnail: getBestThumbnail(c.thumbnails),
    })),
    captions: [],
    related: [],
  }
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
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
    related: [],
  }
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
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

// eslint-disable-next-line @typescript-eslint/no-explicit-any
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

// eslint-disable-next-line @typescript-eslint/no-explicit-any
function mapYouTubeComment(c: any): Comment {
  const author = c.author || {}
  return {
    id: c.comment_id || c.id || '',
    author: author.name || 'Unknown',
    authorId: author.id || '',
    authorAvatar: getBestThumbnail(author.thumbnails),
    content: c.content?.text || c.content || '',
    likeCount: c.like_count || c.likes || 0,
    published: c.published?.text || c.published || '',
    replies: [],
    replyCount: c.reply_count || c.replyCount || 0,
  }
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
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

export async function getVideo(videoId: string): Promise<Video> {
  try {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const info: any = await getVideoInfo(videoId)
    const details = info.basic_info
    const author = details.channel

    return {
      id: details.id || videoId,
      title: details.title || 'Unknown',
      author: author?.name || 'Unknown',
      authorId: author?.id || '',
      authorUrl: author?.channel_url || '',
      description: details.short_description || '',
      thumbnail: getBestThumbnail(details.thumbnail),
      viewCount: details.view_count || 0,
      likeCount: details.like_count || 0,
      lengthSeconds: details.duration || 0,
      published: '',
      isLive: details.is_live || false,
      isUpcoming: details.is_upcoming || false,
      isShort: false,
      chapters: [],
      captions: [],
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      related: (info.related_videos || info.related || []).map(mapYouTubeVideo),
    }
  } catch {
    const data = await inv.getVideoInfoInvidious(videoId)
    return mapInvidiousVideo(data)
  }
}

export async function getChannelInfo(channelId: string): Promise<Channel> {
  try {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const channel: any = await getChannel(channelId)
    const metadata = channel.metadata

    return {
      id: metadata.external_id || channelId,
      name: metadata.title || 'Unknown',
      description: metadata.description || '',
      avatar: getBestThumbnail(metadata.avatar),
      banner: getBestThumbnail(metadata.banner),
      subscriberCount: metadata.subscriber_count || 0,
      videoCount: 0,
      tabs: channel.tabs || [],
      videos: [],
      relatedChannels: [],
    }
  } catch {
    const data = await inv.getChannelInvidious(channelId)
    return mapInvidiousChannel(data)
  }
}

export async function search(query: string): Promise<Video[]> {
  try {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const results: any = await searchLocal(query)
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    return (results.results || []).map(mapYouTubeVideo)
  } catch {
    const data = await inv.searchInvidious(query)
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    return ((data as any[]) || []).filter((i: any) => i.type === "video").map(mapInvidiousVideo)
  }
}

export async function getPlaylistInfo(playlistId: string): Promise<Playlist> {
  try {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const pl: any = await getPlaylist(playlistId)
    return {
      id: pl.id || playlistId,
      title: pl.title?.text || pl.title || 'Unknown',
      description: pl.description || '',
      author: pl.author?.name || 'Unknown',
      authorId: pl.author?.id || '',
      videoCount: pl.video_count || pl.videos?.length || 0,
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      videos: (pl.videos || []).map(mapYouTubeVideo),
    }
  } catch {
    const data = await inv.getPlaylistInvidious(playlistId)
    return mapInvidiousPlaylist(data)
  }
}

export async function getCommentsInfo(videoId: string): Promise<Comment[]> {
  try {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const comments: any = await getComments(videoId)
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    return (comments.contents || []).map(mapYouTubeComment)
  } catch {
    const data = await inv.getCommentsInvidious(videoId) as any
    return (data.comments || []).map(mapInvidiousComment)
  }
}

export async function getTrendingVideos(): Promise<Video[]> {
  try {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const trending: any = await getTrending()
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    return (trending.videos || trending.contents || []).map(mapYouTubeVideo)
  } catch {
    const data = await inv.getTrendingInvidious()
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    return ((data as any[]) || []).filter((i: any) => i.type === "video").map(mapInvidiousVideo)
  }
}

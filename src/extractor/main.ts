import { invoke } from '@tauri-apps/api/core'
import { Innertube } from 'youtubei.js'

// ─── Types ───────────────────────────────────────────────────────────────────

interface VideoInfo {
  id: string
  title: string
  author: string
  authorId: string
  authorUrl: string
  authorAvatar: string
  description: string
  thumbnail: string
  viewCount: number
  likeCount: number
  lengthSeconds: number
  published: string
  isLive: boolean
  isUpcoming: boolean
  isShort: boolean
  chapters: Array<{ title: string; startSeconds: number; thumbnail: string }>
  captions: Array<{ languageCode: string; name: string; url: string }>
  related: VideoInfo[]
}

interface ChannelInfo {
  id: string
  name: string
  description: string
  avatar: string
  banner: string
  subscriberCount: number
  videoCount: number
  tabs: string[]
  videos: VideoInfo[]
  relatedChannels: ChannelInfo[]
  shelves: Array<{ title: string; content: unknown[] }>
}

interface PlaylistInfo {
  id: string
  title: string
  description: string
  author: string
  authorId: string
  videoCount: number
  videos: VideoInfo[]
}

interface CommentInfo {
  id: string
  author: string
  authorId: string
  authorAvatar: string
  content: string
  likeCount: number
  published: string
  replies: CommentInfo[]
  replyCount: number
}

// ─── Innertube session ───────────────────────────────────────────────────────

let innertube: Awaited<ReturnType<typeof Innertube.create>> | null = null

async function getInnertube(): Promise<Awaited<ReturnType<typeof Innertube.create>>> {
  if (!innertube) {
    innertube = await Innertube.create({
      enable_session_cache: false,
      generate_session_locally: true,
      retrieve_innertube_config: true,
      user_agent:
        'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36',
    })
    console.info('[Extractor] Innertube session created')
  }
  return innertube
}

// ─── Result delivery ─────────────────────────────────────────────────────────

async function deliverResult(reqId: string, result: unknown): Promise<void> {
  await invoke('extraction_result', {
    reqId,
    result: { data: result },
  })
}

async function deliverError(reqId: string, error: string): Promise<void> {
  await invoke('extraction_result', {
    reqId,
    result: { error },
  })
}

// ─── Utility helpers ─────────────────────────────────────────────────────────

function getBestThumbnail(thumbnails: Array<{ url: string; width?: number }> | undefined): string {
  if (!thumbnails || thumbnails.length === 0) return ''
  const sorted = [...thumbnails].sort((a, b) => (b.width || 0) - (a.width || 0))
  return sorted[0]?.url || ''
}

function parseViewCount(text: string | undefined): number {
  if (!text) return 0
  const cleaned = text.replace(/[^0-9]/g, '')
  return parseInt(cleaned, 10) || 0
}

// ─── Video parser ────────────────────────────────────────────────────────────

function parseVideo(video: any): VideoInfo | null {
  if (!video) return null

  const videoId = video.videoId || video.id || ''
  if (!videoId) return null

  const author = video.author
  const authorName = typeof author === 'string' ? author : (author?.name || author?.title?.text || 'Unknown')
  const authorId = author?.id || author?.channelId || video.channelId || ''

  const thumbnails = video.thumbnail?.thumbnails || video.videoThumbnails
  const thumbnail = getBestThumbnail(thumbnails) || `https://i.ytimg.com/vi/${videoId}/hqdefault.jpg`

  const viewCountText = video.viewCount?.text || video.viewCountText || video.shortViewCount?.text
  const lengthText = video.duration?.text || video.lengthText?.simpleText || video.lengthText?.accessibility?.accessibility_data?.label

  let lengthSeconds = 0
  if (video.duration?.seconds) {
    lengthSeconds = video.duration.seconds
  } else if (lengthText) {
    const parts = lengthText.split(':').map(Number)
    if (!parts.some(isNaN)) {
      lengthSeconds = parts.reduce((acc: number, part: number) => acc * 60 + part, 0)
    }
  }

  return {
    id: videoId,
    title: video.title?.text || video.title?.runs?.[0]?.text || 'Unknown',
    author: authorName,
    authorId,
    authorUrl: `/channel/${authorId}`,
    authorAvatar: '',
    description: video.descriptionSnippet?.runs?.[0]?.text || video.description || '',
    thumbnail,
    viewCount: parseViewCount(viewCountText),
    likeCount: parseViewCount(video.likeCount),
    lengthSeconds,
    published: video.publishedTimeText?.simpleText || video.published?.text || '',
    isLive: video.isLive || (video.duration?.seconds === 0 && !!video.isLive) || false,
    isUpcoming: !!video.isUpcoming,
    isShort: false,
    chapters: [],
    captions: [],
    related: [],
  }
}

// ─── Master item dispatcher (equivalent to OpenTubeX parseListItem) ───────────

function parseListItem(item: any): unknown | null {
  if (!item) return null

  switch (item.type) {
    case 'Video':
    case 'GridVideo':
    case 'Movie':
    case 'GridMovie':
    case 'VideoCard':
      return { type: 'video', data: parseVideo(item) }

    case 'Channel':
    case 'GridChannel': {
      const channelId = item.channelId || item.id || ''
      return {
        type: 'channel',
        data: {
          id: channelId,
          name: item.name?.text || item.author?.name || 'Unknown',
          description: item.descriptionSnippet?.runs?.[0]?.text || item.description || '',
          avatar: getBestThumbnail(item.author?.thumbnails || item.thumbnail?.thumbnails),
          banner: '',
          subscriberCount: parseViewCount(item.subscriberCountText?.text || item.subscriber_count?.text),
          videoCount: parseViewCount(item.videoCountText?.text || item.video_count?.text),
          tabs: [],
          videos: [],
          relatedChannels: [],
        },
      }
    }

    case 'Playlist':
    case 'GridPlaylist':
    case 'CompactStation': {
      const playlistId = item.playlistId || item.id || ''
      return {
        type: 'playlist',
        data: {
          id: playlistId,
          title: item.title?.text || item.title?.runs?.[0]?.text || 'Unknown',
          description: item.description || '',
          author: item.author?.name || item.subtitle?.runs?.[0]?.text || '',
          authorId: item.author?.id || '',
          videoCount: parseViewCount(item.videoCountText?.text),
          videos: [],
        },
      }
    }

    case 'ReelItem':
    case 'ShortsLockupView': {
      const video = parseVideo({
        ...item,
        videoId: item.videoId || item.onTap?.payload?.videoId,
        duration: { seconds: 0 },
      })
      if (video) video.isShort = true
      return { type: 'video', data: video }
    }

    case 'LockupView': {
      const content_type = item.content_type
      if (content_type === 'ALBUM' || content_type === 'PLAYLIST' || content_type === 'PODCAST') {
        const playlistId = item.content_id || ''
        return {
          type: 'playlist',
          data: {
            id: playlistId,
            title: item.metadata?.title?.text || 'Unknown',
            description: '',
            author: '',
            authorId: '',
            videoCount: 0,
            videos: [],
          },
        }
      }
      if (content_type === 'SHORT' || content_type === 'VIDEO') {
        return { type: 'video', data: parseVideo(item.metadata || {}) }
      }
      console.warn(`[Extractor] Unknown LockupView content_type: ${content_type}`)
      return null
    }

    case 'HashtagTile':
      return {
        type: 'hashtag',
        data: {
          name: item.hashtag?.text || item.title?.text || '',
          videoCount: parseViewCount(item.hashtagVideoCount?.text),
          channelCount: parseViewCount(item.hashtagChannelCount?.text),
        },
      }

    case 'Post':
    case 'BackstagePost':
      return { type: 'community', data: parseCommunityPost(item) }

    case 'GameCard':
      return {
        type: 'channel',
        data: {
          id: item.game?.id || '',
          name: item.game?.title?.text || item.title?.text || 'Unknown',
          description: '',
          avatar: getBestThumbnail(item.game?.boxArt?.thumbnails || item.thumbnail?.thumbnails),
          banner: '',
          subscriberCount: 0,
          videoCount: 0,
          tabs: [],
          videos: [],
          relatedChannels: [],
        },
      }

    default:
      console.warn(`[Extractor] Unknown search result type: ${item.type}`)
      return null
  }
}

// ─── Community post parser ────────────────────────────────────────────────────

function parseCommunityPost(post: any): unknown {
  const content = post.content?.runs?.map((r: any) => r.text).join('') || post.content?.simpleText || ''

  let attachment: unknown = null
  if (post.backstageAttachment) {
    const a = post.backstageAttachment
    if (a.backstageImage) {
      attachment = { type: 'image', url: getBestThumbnail(a.backstageImage.image?.thumbnails) }
    } else if (a.poll) {
      attachment = { type: 'poll', question: a.poll.question, choices: a.poll.choices }
    }
  }

  return {
    id: post.postId || post.id || '',
    author: post.author?.[0]?.name || post.authorText?.runs?.[0]?.text || 'Unknown',
    authorId: post.author?.[0]?.id || post.author?.id || '',
    authorAvatar: getBestThumbnail(post.author?.[0]?.thumbnails || post.authorThumbnail?.thumbnails),
    content,
    likeCount: parseViewCount(post.voteCount?.text),
    published: post.publishedTimeText?.text || '',
    attachment,
    postId: post.postId || post.id || '',
  }
}

// ─── Method handlers ──────────────────────────────────────────────────────────

async function handleVideoInfo(params: any): Promise<VideoInfo> {
  const yt = await getInnertube()
  const videoId = params.videoId

  if (!videoId) throw new Error('Missing videoId')

  const info = (await yt.getInfo(videoId, (params.client || 'WEB') as any)) as any
  const basicInfo = info.basic_info
  const details = info.videoDetails

  return {
    id: videoId,
    title: basicInfo?.title || details?.title || 'Unknown',
    author: basicInfo?.author || details?.author || 'Unknown',
    authorId: basicInfo?.channel_id || details?.channelId || '',
    authorUrl: `/channel/${basicInfo?.channel_id || details?.channelId || ''}`,
    authorAvatar: '',
    description: basicInfo?.short_description || details?.shortDescription || '',
    thumbnail:
      getBestThumbnail(basicInfo?.thumbnail || details?.thumbnail?.thumbnails) ||
      `https://i.ytimg.com/vi/${videoId}/hqdefault.jpg`,
    viewCount: basicInfo?.view_count || parseInt(details?.viewCount || '0') || 0,
    likeCount: basicInfo?.like_count || parseInt(details?.likeCount || '0') || 0,
    lengthSeconds: basicInfo?.duration || parseInt(details?.lengthSeconds || '0') || 0,
    published: basicInfo?.publish_date || '',
    isLive: basicInfo?.is_live || details?.isLive || false,
    isUpcoming: !!basicInfo?.is_upcoming,
    isShort: false,
    chapters: (info.chapters || []).map((c: any) => ({
      title: c.title?.text || c.title || '',
      startSeconds: c.start_seconds || 0,
      thumbnail: getBestThumbnail(c.thumbnail),
    })),
    captions: (info.captions?.caption_tracks || []).map((c: any) => ({
      languageCode: c.language_code || c.languageCode || '',
      name: c.name?.simpleText || c.name?.text || '',
      url: c.base_url || c.baseUrl || '',
    })),
    related:
      (info.watch_next_feed || [])
        .map((item: any) => (parseListItem(item) as any)?.data as VideoInfo)
        .filter(Boolean),
  }
}

async function handleSearch(params: any): Promise<unknown[]> {
  const yt = await getInnertube()
  const query = params.query

  if (!query) throw new Error('Missing query')

  const search = await yt.search(query, {
    upload_date: params.upload_date,
    type: params.type,
    duration: params.duration,
    prioritize: params.prioritize,
    features: params.features,
  })

  const results: unknown[] = []
  for (const item of (search as any).results || []) {
    const parsed = parseListItem(item)
    if (parsed && (parsed as any).data) {
      results.push(parsed)
    }
  }

  return results
}

async function handleGetChannel(params: any): Promise<ChannelInfo> {
  const yt = await getInnertube()
  const channelId = params.channelId

  if (!channelId) throw new Error('Missing channelId')

  const channel = await yt.getChannel(channelId)
  const metadata = channel.metadata as any
  const header = channel.header as any

  const ch = channel as any

  // Determine available tabs
  const tabs: string[] = []
  if (ch.has_home || ch.home) tabs.push('home')
  if (ch.has_videos || ch.videos) tabs.push('videos')
  if (ch.has_shorts || ch.shorts || ch.has_short_videos || ch.shortVideos) tabs.push('shorts')
  if (ch.has_streams || ch.streams || ch.has_live_streams || ch.liveStreams) tabs.push('live')
  if (ch.has_playlists || ch.playlists) tabs.push('playlists')
  if (ch.has_community || ch.community) tabs.push('community')

  // Parse home tab shelves if requested
  const shelves: Array<{ title: string; content: unknown[] }> = []
  if (params.includeHomeShelves && ch.home) {
    const homeContent = ch.home.current_tab?.content
    if (homeContent?.contents) {
      for (const section of homeContent.contents) {
        const firstChild = section.content?.contents?.[0] || section.content
        if (firstChild?.type === 'Shelf') {
          shelves.push({
            title: firstChild.title?.text || 'Videos',
            content: (firstChild.content?.items || [])
              .map((item: unknown) => parseListItem(item))
              .filter(Boolean),
          })
        } else if (firstChild?.type === 'ReelShelf') {
          shelves.push({
            title: firstChild.title?.text || 'Shorts',
            content: (firstChild.items || [])
              .map((item: unknown) => parseListItem(item))
              .filter(Boolean),
          })
        } else if (firstChild?.type === 'HorizontalCardList') {
          shelves.push({
            title: firstChild.header?.title?.text || 'Channels',
            content: (firstChild.cards || [])
              .map((item: unknown) => parseListItem(item))
              .filter(Boolean),
          })
        } else if (firstChild?.type === 'RichShelf') {
          shelves.push({
            title: firstChild.title?.text || 'Playlists',
            content: (firstChild.contents || [])
              .map((item: any) => parseListItem(item?.content || item))
              .filter(Boolean),
          })
        }
      }
    }
  }

  return {
    id: metadata?.external_id || channelId,
    name: metadata?.title || header?.title?.text || 'Unknown',
    description: metadata?.description || '',
    avatar: getBestThumbnail(metadata?.avatar?.thumbnails || header?.author?.thumbnails),
    banner: getBestThumbnail(header?.banner?.thumbnails),
    subscriberCount: parseViewCount(header?.subscriber_count?.text || metadata?.subscriber_count),
    videoCount: 0,
    tabs,
    videos:
      (ch.videos || []).map((v: unknown) => parseVideo(v)).filter(Boolean),
    relatedChannels: [],
    shelves,
  }
}

async function handleGetComments(params: any): Promise<CommentInfo[]> {
  const yt = await getInnertube()
  const videoId = params.videoId

  if (!videoId) throw new Error('Missing videoId')

  const comments = await yt.getComments(videoId, params.sort_by)

  return ((comments as any).contents || [])
    .map((thread: any) => {
      const comment = thread.comment
      if (!comment) return null

      const replies: CommentInfo[] = (thread.replies || [])
        .map((reply: unknown) => parseComment(reply))
        .filter(Boolean) as CommentInfo[]

      return {
        id: comment.comment_id || '',
        author: comment.author?.name || 'Unknown',
        authorId: comment.author?.id || '',
        authorAvatar: getBestThumbnail(comment.author?.thumbnails),
        content:
          comment.content?.runs?.map((r: any) => r.text).join('') ||
          comment.content?.simpleText ||
          '',
        likeCount: parseViewCount(comment.vote_count?.text),
        published: comment.published_time?.text || '',
        replies,
        replyCount: replies.length,
      }
    })
    .filter(Boolean) as CommentInfo[]
}

function parseComment(comment: any): CommentInfo | null {
  if (!comment) return null
  return {
    id: comment.comment_id || comment.id || '',
    author: comment.author?.name || 'Unknown',
    authorId: comment.author?.id || '',
    authorAvatar: getBestThumbnail(comment.author?.thumbnails),
    content:
      comment.content?.runs?.map((r: any) => r.text).join('') ||
      comment.content?.simpleText ||
      '',
    likeCount: parseViewCount(comment.vote_count?.text),
    published: comment.published_time?.text || '',
    replies: [],
    replyCount: 0,
  }
}

async function handleGetTrending(_params: any): Promise<VideoInfo[]> {
  const yt = await getInnertube()

  // Use getHomeFeed which returns trending-like content
  const feed = await yt.getHomeFeed()

  const videos: VideoInfo[] = []
  for (const section of (feed as any).contents || []) {
    for (const item of section.contents || []) {
      const parsed = parseListItem(item)
      if (parsed && (parsed as any).type === 'video' && (parsed as any).data) {
        videos.push((parsed as any).data)
      }
    }
  }

  return videos
}

async function handleGetPlaylist(params: any): Promise<PlaylistInfo> {
  const yt = await getInnertube()
  const playlistId = params.playlistId

  if (!playlistId) throw new Error('Missing playlistId')

  const playlist = await yt.getPlaylist(playlistId)
  const p = playlist as any

  return {
    id: playlistId,
    title: p.info?.title?.text || 'Unknown',
    description: p.info?.description?.text || p.info?.short_description || '',
    author: p.info?.author?.name || '',
    authorId: p.info?.author?.id || '',
    videoCount: p.video_count || 0,
    videos:
      (p.videos || []).map((v: unknown) => parseVideo(v)).filter(Boolean),
  }
}

async function handleGetSearchSuggestions(params: any): Promise<string[]> {
  const yt = await getInnertube()
  const query = params.query

  if (!query) return []

  return await yt.getSearchSuggestions(query)
}

// ─── Main bridge ─────────────────────────────────────────────────────────────

;(window as any).__slytube_run = async (reqId: string, method: string, params: any) => {
  try {
    let result: unknown

    switch (method) {
      case 'getVideoInfo':
        result = await handleVideoInfo(params)
        break
      case 'search':
        result = await handleSearch(params)
        break
      case 'getChannel':
        result = await handleGetChannel(params)
        break
      case 'getChannelVideos': {
        const yt = await getInnertube()
        await yt.actions.execute('/browse', {
          browseId: params.channelId,
          params: 'EgZ2aWRlb3PyBgQKAjoA',
        })
        result = { videos: [] }
        break
      }
      case 'getComments':
        result = await handleGetComments(params)
        break
      case 'getTrending':
        result = await handleGetTrending(params)
        break
      case 'getPlaylist':
        result = await handleGetPlaylist(params)
        break
      case 'getSearchSuggestions':
        result = await handleGetSearchSuggestions(params)
        break
      default:
        throw new Error(`Unknown method: ${method}`)
    }

    await deliverResult(reqId, result)
  } catch (err: any) {
    console.error(`[Extractor] Error in ${method}:`, err)
    await deliverError(reqId, err?.message || String(err))
  }
}

// Signal ready
const statusEl = document.getElementById('status')
if (statusEl) statusEl.textContent = 'youtubei.js ready'

console.info('[Extractor] Bridge initialized — youtubei.js loaded')

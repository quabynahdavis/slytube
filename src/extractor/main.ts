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
        // LockupView.metadata contains video metadata in a different shape than Video nodes.
        // Map it to the structure parseVideo expects.
        const metadata = item.metadata || {}
        const videoData = {
          videoId: item.content_id || metadata.videoId || '',
          title: metadata.title?.text || metadata.title?.runs?.[0]?.text || 'Unknown',
          author: {
            name: metadata.metadata?.metadata?.metadata_rows?.[0]?.metadata_parts?.[0]?.text || '',
          },
          thumbnail: metadata.content_image?.primary_thumbnail?.image || metadata.thumbnail,
          viewCount: metadata.metadata?.metadata?.metadata_rows?.flatMap((r: any) => r.metadata_parts || []).find((p: any) => /views?/i.test(p.text)),
          duration: metadata.thumbnail_overlay?.thumbnail_overlay_time_status_text,
        }
        const video = parseVideo(videoData)
        if (video && content_type === 'SHORT') video.isShort = true
        return { type: 'video', data: video }
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

  // Determine available tabs using youtubei.js boolean getters
  const tabs: string[] = []
  if (ch.has_home) tabs.push('home')
  if (ch.has_videos) tabs.push('videos')
  if (ch.has_shorts) tabs.push('shorts')
  if (ch.has_live_streams) tabs.push('live')
  if (ch.has_playlists) tabs.push('playlists')
  if (ch.has_community) tabs.push('community')
  if (ch.has_releases) tabs.push('releases')

  // Parse home tab shelves if requested.
  // The initial getChannel() response includes the home tab content in current_tab.
  const shelves: Array<{ title: string; content: unknown[] }> = []
  if (params.includeHomeShelves) {
    const homeContent = ch.current_tab?.content as any
    const sections = homeContent?.contents || []
    for (const section of sections) {
      // Each section may be an ItemSection (first child is Shelf/ReelShelf/HorizontalCardList)
      // or a RichSection (content is RichShelf)
      const sectionContent = section.content || section
      const firstChild = sectionContent.contents?.[0] || sectionContent.content || sectionContent

      if (firstChild?.type === 'Shelf') {
        shelves.push({
          title: firstChild.title?.text || 'Videos',
          content: (firstChild.content?.items || firstChild.contents || [])
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
      } else if (sectionContent.content?.type === 'RichShelf') {
        shelves.push({
          title: sectionContent.content.title?.text || 'Playlists',
          content: (sectionContent.content.contents || [])
            .map((item: any) => parseListItem(item?.content || item))
            .filter(Boolean),
        })
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

async function handleGetTrending(params: any): Promise<VideoInfo[]> {
  const yt = await getInnertube()
  const tab = params?.tab || 'default'

  // Map tab names to YouTube browseIds
  const browseIds: Record<string, string> = {
    default: 'FEtrending',
    music: 'FEtrending',       // Music tab uses protobuf param
    gaming: 'FEtrending',     // Gaming tab uses protobuf param
    movies: 'FEtrending',     // Movies tab uses protobuf param
    sports: 'FEtrending',
  }
  const protobufParams: Record<string, string | undefined> = {
    default: undefined,
    music: '4gINGgtpbiAQBBoIdHJlbmRpbmcYmgMiBQgBGAQ%3D',
    gaming: '4gINGgtpbiAYBBoIdHJlbmRpbmcYmgMiBQgBGAQ%3D',
    movies: '4gIKGghmaWxtZGG4AQCSAwDyBgQKAjIA',
    sports: '4gIKGghzcG9ydHN0YWK4AQCSAwDyBgQKAjIA',
  }

  const browseId = browseIds[tab] || 'FEtrending'
  const protoParams = protobufParams[tab]

  // Fetch the trending browse response
  const response = await yt.actions.execute('/browse', {
    browseId,
    ...(protoParams ? { params: protoParams } : {}),
  })

  const videos: VideoInfo[] = []
  const contents = (response as any)?.data?.contents?.tabbedBrowseResultsRenderer?.tabs || []

  for (const tabData of contents) {
    const tabRenderer = tabData.tabRenderer
    if (!tabRenderer?.content) continue

    const sections = tabRenderer.content.sectionListRenderer?.contents || []
    for (const section of sections) {
      // Each section has itemSectionRenderer with video items
      const items = section?.itemSectionRenderer?.contents || []
      for (const item of items) {
        const parsed = parseListItem(item)
        if (parsed && (parsed as any).type === 'video' && (parsed as any).data) {
          videos.push((parsed as any).data)
        }
      }

      // Handle shelf renderers (e.g. "Trending" shelf within a tab)
      const shelf = section?.shelfRenderer
      if (shelf?.content) {
        const shelfItems = shelf.content.horizontalListRenderer?.items || shelf.content.expandedShelfContentsRenderer?.items || []
        for (const item of shelfItems) {
          const parsed = parseListItem(item)
          if (parsed && (parsed as any).type === 'video' && (parsed as any).data) {
            videos.push((parsed as any).data)
          }
        }
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

async function handleGetCommentReplies(params: any): Promise<CommentInfo[]> {
  const yt = await getInnertube()
  const videoId = params.videoId
  const commentId = params.commentId

  if (!videoId || !commentId) throw new Error('Missing videoId or commentId')

  const comments = await yt.getComments(videoId, params.sort_by, commentId)

  return ((comments as any).contents || [])
    .map((thread: any) => parseComment(thread.comment))
    .filter(Boolean) as CommentInfo[]
}

async function handleGetHashtag(params: any): Promise<VideoInfo[]> {
  const yt = await getInnertube()
  const hashtag = params.hashtag

  if (!hashtag) throw new Error('Missing hashtag')

  const hashtagFeed = await yt.getHashtag(hashtag)

  const videos: VideoInfo[] = []
  const sections = (hashtagFeed as any)?.contents?.contents || (hashtagFeed as any)?.contents || []
  for (const section of sections) {
    const items = section?.itemSectionRenderer?.contents || section?.contents || []
    for (const item of items) {
      const parsed = parseListItem(item)
      if (parsed && (parsed as any).type === 'video' && (parsed as any).data) {
        videos.push((parsed as any).data)
      }
    }
  }

  return videos
}

async function handleGetCommunityPost(params: any): Promise<unknown> {
  const yt = await getInnertube()
  const postId = params.postId
  const channelId = params.channelId

  if (!postId || !channelId) throw new Error('Missing postId or channelId')

  const post = await yt.getPost(postId, channelId)

  // Extract post data and comments from the feed
  const posts = (post as any)?.contents || []
  const result = []
  for (const section of posts) {
    const items = section?.itemSectionRenderer?.contents || []
    for (const item of items) {
      if (item?.backstagePostThreadRenderer) {
        const postData = item.backstagePostThreadRenderer.post
        if (postData) {
          result.push(parseCommunityPost(postData))
        }
      }
    }
  }

  return result[0] || null
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
        const videosChannel = await (await yt.getChannel(params.channelId)).getVideos()
        const videosTab = videosChannel as any
        result = {
          videos: (videosTab.videos || [])
            .map((v: unknown) => parseVideo(v))
            .filter(Boolean),
          continuation: videosTab.has_continuation || false,
        }
        break
      }
      case 'getChannelShorts': {
        const yt = await getInnertube()
        const shortsChannel = await (await yt.getChannel(params.channelId)).getShorts()
        const shortsTab = shortsChannel as any
        result = {
          videos: (shortsTab.videos || [])
            .map((v: unknown) => { const video = parseVideo(v); if (video) video.isShort = true; return video })
            .filter(Boolean),
          continuation: shortsTab.has_continuation || false,
        }
        break
      }
      case 'getChannelLive': {
        const yt = await getInnertube()
        const liveChannel = await (await yt.getChannel(params.channelId)).getLiveStreams()
        const liveTab = liveChannel as any
        result = {
          videos: (liveTab.videos || [])
            .map((v: unknown) => parseVideo(v))
            .filter(Boolean),
          continuation: liveTab.has_continuation || false,
        }
        break
      }
      case 'getChannelCommunity': {
        const yt = await getInnertube()
        const communityChannel = await (await yt.getChannel(params.channelId)).getCommunity()
        const communityTab = communityChannel as any
        result = {
          posts: (communityTab.posts || [])
            .map((p: unknown) => parseCommunityPost(p)),
        }
        break
      }
      case 'getChannelPlaylists': {
        const yt = await getInnertube()
        const playlistsChannel = await (await yt.getChannel(params.channelId)).getPlaylists()
        const playlistsTab = playlistsChannel as any
        result = {
          playlists: (playlistsTab.playlists || [])
            .map((p: any) => ({
              id: p.playlistId || p.id || '',
              title: p.title?.text || p.title?.runs?.[0]?.text || 'Unknown',
              description: p.description || '',
              author: p.author?.name || p.subtitle?.runs?.[0]?.text || '',
              authorId: p.author?.id || '',
              videoCount: parseViewCount(p.videoCountText?.text),
              videos: [],
            })),
        }
        break
      }
      case 'getComments':
        result = await handleGetComments(params)
        break
      case 'getCommentReplies':
        result = await handleGetCommentReplies(params)
        break
      case 'getHashtag':
        result = await handleGetHashtag(params)
        break
      case 'getCommunityPost':
        result = await handleGetCommunityPost(params)
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

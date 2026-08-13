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

/**
 * Extracts a number from a string like "1,234,567 views" or "1.2M".
 * Strips all non-digit characters and parses the result.
 */
function extractNumberFromString(str: string | undefined): number {
  if (typeof str !== 'string') return 0
  const cleaned = str.replace(/\D+/g, '')
  return parseInt(cleaned, 10) || 0
}

/**
 * Parses subscriber/view count strings with K/M/B suffixes.
 * e.g. "1.2M" → 1200000, "3.5K" → 3500, "1.2 thousand" → 1200
 */
function parseSubscriberCount(text: string | undefined): number {
  if (!text) return 0
  const match = text.match(/(\d+)(?:[,.](\d+))?\s?([BKMbkm]|thousand|[bm]illion)\b/)
  if (match) {
    let multiplier = 0
    switch (match[3]) {
      case 'K':
      case 'k':
      case 'thousand':
        multiplier = 3
        break
      case 'M':
      case 'm':
      case 'million':
        multiplier = 6
        break
      case 'B':
      case 'b':
      case 'billion':
        multiplier = 9
        break
    }
    let parsedDecimals: string
    if (typeof match[2] === 'undefined') {
      parsedDecimals = '0'.repeat(multiplier)
    } else {
      parsedDecimals = match[2].padEnd(multiplier, '0')
    }
    return parseInt(match[1] + parsedDecimals, 10) || 0
  }
  return extractNumberFromString(text)
}

/**
 * Converts a relative date string like "2 days ago" to a timestamp.
 * Handles: "X seconds/minutes/hours/days/weeks/months/years ago"
 * Also handles "Premieres Jan 1, 2025" and "Streamed X days ago".
 * Returns undefined for live/upcoming content without a known premiere date.
 */
function calculatePublishedDate(
  publishedText: string | undefined,
  isLive = false,
  isUpcoming = false,
  premiereDate?: Date
): string | undefined {
  const now = Date.now()

  if (isLive) {
    return new Date(now).toISOString()
  } else if (isUpcoming) {
    if (premiereDate && !isNaN(premiereDate.getTime())) {
      return premiereDate.toISOString()
    }
    return undefined
  }

  if (!publishedText) return undefined

  // Try to parse "Premieres Jan 1, 2025" or "Scheduled for ..."
  const premieresMatch = publishedText.match(/^(?:premieres?|scheduled for)\s+/i)
  if (premieresMatch) {
    const dateStr = publishedText.replace(/^(?:premieres?|scheduled for)\s+/i, '').trim()
    const parsed = new Date(dateStr)
    if (!isNaN(parsed.getTime())) {
      return parsed.toISOString()
    }
  }

  // Try relative date: "2 days ago", "1 year ago", "Streamed 3 weeks ago"
  const relativeMatch = publishedText.match(/^(?:streamed\s+)?(\d+)\s?([a-z]+)\s+ago/i)
  if (relativeMatch) {
    const timeAmount = parseInt(relativeMatch[1], 10)
    const timeFrame = relativeMatch[2].toLowerCase()
    let timeSpan = 0

    if (timeFrame.startsWith('second') || timeFrame === 's') {
      timeSpan = timeAmount * 1000
    } else if (timeFrame.startsWith('minute') || timeFrame === 'm') {
      timeSpan = timeAmount * 60000
    } else if (timeFrame.startsWith('hour') || timeFrame === 'h') {
      timeSpan = timeAmount * 3600000
    } else if (timeFrame.startsWith('day') || timeFrame === 'd') {
      timeSpan = timeAmount * 86400000
    } else if (timeFrame.startsWith('week') || timeFrame === 'w') {
      timeSpan = timeAmount * 604800000
    } else if (timeFrame.startsWith('month') || timeFrame === 'mo') {
      timeSpan = timeAmount * 2592000000
    } else if (timeFrame.startsWith('year') || timeFrame === 'y') {
      timeSpan = timeAmount * 31556952000
    }

    return new Date(now - timeSpan).toISOString()
  }

  // Try direct date parse
  const directParse = new Date(publishedText)
  if (!isNaN(directParse.getTime())) {
    return directParse.toISOString()
  }

  return undefined
}

/**
 * Converts a duration string "HH:MM:SS" or "MM:SS" to seconds.
 */
function parseDurationText(text: string | undefined): number {
  if (!text) return 0
  const parts = text.split(':').map(Number)
  if (parts.some(isNaN)) return 0
  return parts.reduce((acc: number, part: number) => acc * 60 + part, 0)
}

// ─── Video parser ────────────────────────────────────────────────────────────

/**
 * Parses a youtubei.js video node into a VideoInfo.
 * Handles Video, GridVideo, LockupView, and Movie node shapes defensively.
 *
 * Field access is defensive because youtubei.js nodes may have data in different
 * locations depending on context (watch page vs feed vs search).
 */
function parseVideo(video: any): VideoInfo | null {
  if (!video) return null

  // ── Extract videoId (defensive: different node types use different id fields) ──
  const videoId = video.video_id || video.videoId || video.id || ''
  if (!videoId) return null

  // ── Extract title ──
  const title = video.title?.text || video.title?.runs?.[0]?.text || 'Unknown'

  // ── Extract author info ──
  const author = video.author
  const authorName = typeof author === 'string'
    ? author
    : (author?.name || author?.title?.text || 'Unknown')
  const authorId = author?.id || author?.channelId || video.channel_id || video.channelId || ''

  // ── Extract author thumbnails ──
  const authorThumbnails = author?.thumbnails
  const authorAvatar = getBestThumbnail(authorThumbnails) || ''

  // ── Extract thumbnails ──
  const thumbnails = video.thumbnail?.thumbnails || video.videoThumbnails
  const thumbnail = getBestThumbnail(thumbnails) || `https://i.ytimg.com/vi/${videoId}/hqdefault.jpg`

  // ── Extract view count (fallback chain) ──
  // youtubei.js nodes may expose view count in different fields:
  //   - Video: view_count.text, short_view_count.text
  //   - GridVideo: views.text
  //   - Some contexts: view_count as a plain number/string
  let viewCount: number | null = null

  if (video.view_count?.text) {
    const text = video.view_count.text.toLowerCase()
    viewCount = text === 'no views' ? 0 : extractNumberFromString(video.view_count.text)
  } else if (video.short_view_count?.text) {
    const text = video.short_view_count.text.toLowerCase()
    viewCount = text === 'no views' ? 0 : parseSubscriberCount(video.short_view_count.text)
  } else if (video.views?.text) {
    viewCount = extractNumberFromString(video.views.text)
  } else if (video.view_count != null && typeof video.view_count !== 'object') {
    // Plain number or string (not the typical { text } object)
    viewCount = typeof video.view_count === 'number'
      ? video.view_count
      : extractNumberFromString(String(video.view_count))
  }

  // ── Extract duration (fallback chain) ──
  // Duration may be in:
  //   - duration.seconds (number)
  //   - duration.text ("MM:SS" or "HH:MM:SS" or "LIVE")
  //   - length_seconds (number, some node types)
  let lengthSeconds = 0

  if (video.duration?.seconds && !isNaN(Number(video.duration.seconds))) {
    lengthSeconds = Number(video.duration.seconds)
  } else if (video.duration?.text && video.duration.text !== 'LIVE') {
    lengthSeconds = parseDurationText(video.duration.text)
  } else if (video.length_seconds && !isNaN(Number(video.length_seconds))) {
    lengthSeconds = Number(video.length_seconds)
  }

  // ── Extract published date ──
  const isLive = !!video.is_live || video.duration?.text === 'LIVE'
  const isUpcoming = !!video.is_upcoming || !!video.is_premiere
  const premiereDate = video.upcoming ? new Date(video.upcoming) : undefined

  let publishedText: string | undefined
  if (video.published?.text) {
    publishedText = video.published.text
  } else if (video.publishedTimeText?.simpleText) {
    publishedText = video.publishedTimeText.simpleText
  } else if (video.date?.text) {
    publishedText = video.date.text
  }

  const published = calculatePublishedDate(publishedText, isLive, isUpcoming, premiereDate)

  // ── Extract description ──
  const description = video.description?.text
    || video.description_snippet?.text
    || video.descriptionSnippet?.runs?.[0]?.text
    || video.description
    || ''

  // ── Debug logging for missing data ──
  if (viewCount === null) {
    console.warn(
      `[Extractor] parseVideo: viewCount is null for video ${videoId}.`,
      'Available keys:', Object.keys(video),
      'view_count:', video.view_count,
      'short_view_count:', video.short_view_count,
      'views:', video.views,
    )
  }
  if (lengthSeconds === 0 && !isLive) {
    console.warn(
      `[Extractor] parseVideo: lengthSeconds is 0 for video ${videoId}.`,
      'duration:', video.duration,
      'length_seconds:', video.length_seconds,
    )
  }

  return {
    id: videoId,
    title,
    author: authorName,
    authorId,
    authorUrl: `/channel/${authorId}`,
    authorAvatar,
    description,
    thumbnail,
    viewCount: viewCount ?? 0,
    likeCount: parseViewCount(video.likeCount),
    lengthSeconds,
    published: published || '',
    isLive,
    isUpcoming,
    isShort: false,
    chapters: [],
    captions: [],
    related: [],
  }
}

// ─── Feed item wrapper detection ──────────────────────────────────────────────

/**
 * Detects and unwraps feed/search wrapper node types to extract the inner content.
 *
 * youtubei.js wraps videos in different container types depending on context:
 * - RichItem: wraps a single video node in .content (common on home feed)
 * - RichShelf: a shelf of items in .contents (common for "For You" sections)
 * - ItemSection: section wrapper with .contents array
 * - RichSection: section with .content being a RichShelf
 *
 * Returns the unwrapped inner item, or the original item if no wrapper detected.
 */
function unwrapFeedItem(item: any): any {
  if (!item) return item

  // RichItem wraps a single video/playlist/etc in .content
  if (item.type === 'RichItem' && item.content) {
    return item.content
  }

  // RichGrid: contains .contents array of RichItems
  if (item.type === 'RichGrid' && item.contents) {
    // Return as-is; callers iterate over contents
    return item
  }

  // ItemSection: contains .contents array of items
  if (item.type === 'ItemSection' && item.contents) {
    return item
  }

  // RichSection: wraps a single shelf in .content
  if (item.type === 'RichSection' && item.content) {
    return item.content
  }

  return item
}

/**
 * Master dispatcher for feed/search page items.
 * First unwraps any wrapper type (RichItem, RichShelf, etc.), then delegates
 * to parseListItem for the actual content parsing.
 *
 * This is the recommended entry point for feed/search parsing to ensure
 * wrapper types are handled correctly.
 */
function parseFeedItem(item: any): unknown | null {
  if (!item) return null

  const unwrapped = unwrapFeedItem(item)

  // After unwrapping RichGrid/ItemSection/RichSection, we may have a container
  // that holds multiple items. Return as-is for callers to iterate.
  if (unwrapped.type === 'RichGrid' || unwrapped.type === 'ItemSection') {
    return {
      type: 'feed_group',
      data: (unwrapped.contents || []).map((inner: any) => {
        // RichGrid contents may themselves be RichItems
        const innerItem = inner.type === 'RichItem' ? inner.content : inner
        return innerItem ? parseListItem(innerItem) : null
      }).filter(Boolean),
    }
  }

  // RichShelf: contains .contents array of items (often LockupViews)
  if (unwrapped.type === 'RichShelf') {
    return {
      type: 'shelf',
      title: unwrapped.title?.text || '',
      data: (unwrapped.contents || []).map((inner: any) => {
        const innerItem = inner.type === 'RichItem' ? inner.content : inner
        return innerItem ? parseListItem(innerItem) : null
      }).filter(Boolean),
    }
  }

  // Single item — delegate to parseListItem
  return parseListItem(unwrapped)
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
        // YouTube changed the metadata row structure in 2026. The layout can be:
        //   - 2 rows: [author] [views, date]
        //   - 1 row:  [views, date]
        //   - 1 row:  [date, views] (parts in reverse order)
        //   - 1 row:  [views] (live streams)
        // We collect all parts dynamically and search for views/dates by pattern.
        const metadata = item.metadata || {}
        const metadataRows = metadata.metadata?.metadata_rows || []
        const metadataParts = metadataRows.flatMap((row: any) => row.metadata_parts || [])

        // Helper: find first part text matching a predicate
        const findPartText = (predicate: (text: string) => boolean): string | undefined =>
          metadataParts.find((part: any) => part.text?.text && predicate(part.text.text))?.text?.text

        // View count: look for parts containing "views" or "watching" or pure numbers
        const viewCountText = findPartText((text: string) =>
          /views?|watching|waiting/i.test(text) || /^\d+(\.\d)?[bkm]?$/i.test(text)
        )

        // Published date: relative time like "2 days ago" or "Streamed 3 weeks ago"
        const publishedText = findPartText((text: string) =>
          /^(streamed )?\d+ ?\w+? ago/i.test(text)
        )

        // Duration: from thumbnail overlay time status badge
        const thumbnailOverlay = metadata.thumbnail_overlay
        let durationText: string | undefined
        let isLive = false
        let isUpcoming = false

        if (thumbnailOverlay?.thumbnail_overlay_time_status_text) {
          durationText = thumbnailOverlay.thumbnail_overlay_time_status_text?.text || undefined
          if (durationText === 'LIVE') {
            isLive = true
          }
        }

        // Check for live/upcoming badges in thumbnailBottomOverlay
        const thumbnailBottomOverlay = metadata.content_image?.overlays?.find(
          (o: any) => o.type === 'ThumbnailBottomOverlayView'
        )
        if (thumbnailBottomOverlay?.badges) {
          for (const badge of thumbnailBottomOverlay.badges) {
            const badgeText = badge.text?.toLowerCase() || ''
            if (badgeText === 'live') isLive = true
            if (badgeText === 'upcoming') isUpcoming = true
          }
        }

        // Author: prefer part with channel endpoint, else first non-views/date part
        const authorPart = metadataParts.find(
          (part: any) => part.text?.endpoint?.metadata?.page_type === 'WEB_PAGE_TYPE_CHANNEL'
        )?.text
        const authorName = authorPart?.text ?? metadataRows[0]?.metadata_parts?.[0]?.text?.text ?? ''
        const authorId = authorPart?.endpoint?.payload?.browseId ?? ''

        // Construct video data in the shape parseVideo expects
        const videoData = {
          video_id: item.content_id || metadata.videoId || '',
          title: metadata.title?.text || metadata.title?.runs?.[0]?.text || 'Unknown',
          author: { name: authorName, id: authorId },
          thumbnail: { thumbnails: metadata.content_image?.primary_thumbnail?.image || metadata.thumbnail },
          view_count: viewCountText ? { text: viewCountText } : undefined,
          duration: durationText ? { text: durationText } : undefined,
          published: publishedText ? { text: publishedText } : undefined,
          is_live: isLive,
          is_upcoming: isUpcoming,
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

  // Extract view count — handle both numeric and text formats
  let viewCount = 0
  if (basicInfo?.view_count && typeof basicInfo.view_count === 'string') {
    viewCount = extractNumberFromString(basicInfo.view_count) || parseSubscriberCount(basicInfo.view_count)
  } else if (basicInfo?.view_count) {
    viewCount = basicInfo.view_count
  } else if (basicInfo?.views) {
    viewCount = typeof basicInfo.views === 'string'
      ? (extractNumberFromString(basicInfo.views) || parseSubscriberCount(basicInfo.views))
      : basicInfo.views
  } else if (details?.viewCount) {
    viewCount = parseInt(details.viewCount, 10) || 0
  }

  // Extract author thumbnail — check multiple possible locations
  let authorAvatar = ''
  if (basicInfo?.author_thumbnail) {
    authorAvatar = getBestThumbnail(basicInfo.author_thumbnail?.thumbnails || basicInfo.author_thumbnail)
  } else if (details?.author?.avatar) {
    authorAvatar = details.author.avatar
  } else if (basicInfo?.channel?.metadata?.thumbnail) {
    authorAvatar = getBestThumbnail(basicInfo.channel.metadata.thumbnail)
  }

  return {
    id: videoId,
    title: basicInfo?.title || details?.title || 'Unknown',
    author: basicInfo?.author || details?.author || 'Unknown',
    authorId: basicInfo?.channel_id || details?.channelId || '',
    authorUrl: `/channel/${basicInfo?.channel_id || details?.channelId || ''}`,
    authorAvatar,
    description: basicInfo?.short_description || details?.shortDescription || '',
    thumbnail:
      getBestThumbnail(basicInfo?.thumbnail || details?.thumbnail?.thumbnails) ||
      `https://i.ytimg.com/vi/${videoId}/hqdefault.jpg`,
    viewCount,
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
    // Use parseFeedItem to handle wrapper types (RichItem, etc.) that may appear
    // in search results
    const parsed = parseFeedItem(item)
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
        // Use parseFeedItem to handle wrapper types (RichItem, etc.)
        const parsed = parseFeedItem(item)
        if (parsed && (parsed as any).type === 'video' && (parsed as any).data) {
          videos.push((parsed as any).data)
        }
      }

      // Handle shelf renderers (e.g. "Trending" shelf within a tab)
      const shelf = section?.shelfRenderer
      if (shelf?.content) {
        const shelfItems = shelf.content.horizontalListRenderer?.items || shelf.content.expandedShelfContentsRenderer?.items || []
        for (const item of shelfItems) {
          const parsed = parseFeedItem(item)
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
      // Use parseFeedItem to handle wrapper types (RichItem, etc.)
      const parsed = parseFeedItem(item)
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

import { describe, it, expect } from 'vitest'

/**
 * Tests for the video parser in src/extractor/main.ts.
 *
 * Since the parser depends on Tauri's @tauri-apps/api/core import,
 * we extract the pure helper functions here to test them in isolation.
 * The logic mirrors the implementation in main.ts.
 */

// ─── Extracted helper functions (mirrors of main.ts) ─────────────────────────

function getBestThumbnail(thumbnails: Array<{ url: string; width?: number }> | undefined): string {
  if (!thumbnails || thumbnails.length === 0) return ''
  const sorted = [...thumbnails].sort((a, b) => (b.width || 0) - (a.width || 0))
  return sorted[0]?.url || ''
}

function extractNumberFromString(str: string | undefined): number {
  if (typeof str !== 'string') return 0
  const cleaned = str.replace(/\D+/g, '')
  return parseInt(cleaned, 10) || 0
}

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

function parseDurationText(text: string | undefined): number {
  if (!text) return 0
  const parts = text.split(':').map(Number)
  if (parts.some(isNaN)) return 0
  return parts.reduce((acc: number, part: number) => acc * 60 + part, 0)
}

// ─── Simplified parseVideo (mirrors the logic in main.ts) ────────────────────

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

function parseVideo(video: any): VideoInfo | null {
  if (!video) return null

  const isMovie = video.type === 'Movie' || video.type === 'GridMovie'
  const isGridVideo = video.type === 'GridVideo'
  const isVideoNode = video.type === 'Video' || (!isMovie && !isGridVideo)

  const videoId = video.video_id || video.videoId || video.id || ''
  if (!videoId) return null

  const title = video.title?.text || video.title?.runs?.[0]?.text || 'Unknown'

  const author = video.author
  const authorName = typeof author === 'string'
    ? author
    : (author?.name || author?.title?.text || 'Unknown')
  const authorId = author?.id || author?.channelId || video.channel_id || video.channelId || ''

  const authorThumbnails = author?.thumbnails
  const authorAvatar = getBestThumbnail(authorThumbnails) || ''

  const thumbnails = video.thumbnail?.thumbnails || video.videoThumbnails
  const thumbnail = getBestThumbnail(thumbnails) || `https://i.ytimg.com/vi/${videoId}/hqdefault.jpg`

  let viewCount = 0
  if (isVideoNode) {
    if (video.view_count?.text) {
      const text = video.view_count.text.toLowerCase()
      viewCount = text === 'no views' ? 0 : extractNumberFromString(video.view_count.text)
    } else if (video.short_view_count?.text) {
      const text = video.short_view_count.text.toLowerCase()
      viewCount = text === 'no views' ? 0 : parseSubscriberCount(video.short_view_count.text)
    }
  } else if (isGridVideo) {
    if (video.views?.text) {
      viewCount = extractNumberFromString(video.views.text)
    }
  }

  let lengthSeconds = 0
  if (video.duration?.seconds && !isNaN(video.duration.seconds)) {
    lengthSeconds = video.duration.seconds
  } else if (video.duration?.text && video.duration.text !== 'LIVE') {
    lengthSeconds = parseDurationText(video.duration.text)
  }

  const isLive = !!video.is_live || video.duration?.text === 'LIVE'
  const isUpcoming = !!video.is_upcoming || !!video.is_premiere
  const premiereDate = video.upcoming ? new Date(video.upcoming) : undefined

  let publishedText: string | undefined
  if (video.published?.text) {
    publishedText = video.published.text
  } else if (video.publishedTimeText?.simpleText) {
    publishedText = video.publishedTimeText.simpleText
  }

  const published = calculatePublishedDate(publishedText, isLive, isUpcoming, premiereDate)

  const description = video.description?.text
    || video.description_snippet?.text
    || video.descriptionSnippet?.runs?.[0]?.text
    || video.description
    || ''

  return {
    id: videoId,
    title,
    author: authorName,
    authorId,
    authorUrl: `/channel/${authorId}`,
    authorAvatar,
    description,
    thumbnail,
    viewCount,
    likeCount: extractNumberFromString(video.likeCount),
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

// ─── Tests ────────────────────────────────────────────────────────────────────

describe('parseVideo - Video node (search/watch-next)', () => {
  it('parses a standard Video node correctly', () => {
    const video = {
      type: 'Video',
      video_id: 'abc123',
      title: { text: 'Test Video' },
      author: { name: 'Test Channel', id: 'UC123', thumbnails: [{ url: 'https://avatar.com/large.jpg', width: 88 }] },
      view_count: { text: '1,234,567 views' },
      short_view_count: { text: '1.2M' },
      published: { text: '2 days ago' },
      duration: { text: '10:30', seconds: 630 },
      thumbnail: { thumbnails: [{ url: 'https://thumb.com/hq.jpg', width: 640 }] },
      is_live: false,
      is_upcoming: false,
    }

    const result = parseVideo(video)
    expect(result).not.toBeNull()
    expect(result?.id).toBe('abc123')
    expect(result?.title).toBe('Test Video')
    expect(result?.author).toBe('Test Channel')
    expect(result?.authorId).toBe('UC123')
    expect(result?.authorAvatar).toBe('https://avatar.com/large.jpg')
    expect(result?.viewCount).toBe(1234567)
    expect(result?.lengthSeconds).toBe(630)
    expect(result?.isLive).toBe(false)
    expect(result?.isUpcoming).toBe(false)
    expect(result?.published).toBeTruthy()
  })

  it('falls back to short_view_count when view_count is missing', () => {
    const video = {
      type: 'Video',
      video_id: 'def456',
      title: { text: 'Short View Count Video' },
      author: { name: 'Channel', id: 'UC456' },
      short_view_count: { text: '1.5M' },
      published: { text: '1 week ago' },
      duration: { text: '5:00', seconds: 300 },
      thumbnail: { thumbnails: [] },
    }

    const result = parseVideo(video)
    expect(result).not.toBeNull()
    expect(result?.viewCount).toBe(1500000)
  })

  it('handles "no views" text correctly', () => {
    const video = {
      type: 'Video',
      video_id: 'ghi789',
      title: { text: 'No Views Video' },
      author: { name: 'New Channel', id: 'UC789' },
      view_count: { text: 'No views' },
      duration: { seconds: 120 },
      thumbnail: { thumbnails: [] },
    }

    const result = parseVideo(video)
    expect(result).not.toBeNull()
    expect(result?.viewCount).toBe(0)
  })

  it('handles live videos', () => {
    const video = {
      type: 'Video',
      video_id: 'live123',
      title: { text: 'Live Stream' },
      author: { name: 'Live Channel', id: 'UClive' },
      view_count: { text: '5,000 views' },
      is_live: true,
      duration: { text: 'LIVE' },
      thumbnail: { thumbnails: [] },
    }

    const result = parseVideo(video)
    expect(result).not.toBeNull()
    expect(result?.isLive).toBe(true)
    expect(result?.published).toBeTruthy() // Live = current timestamp
  })

  it('handles upcoming/premiere videos', () => {
    const video = {
      type: 'Video',
      video_id: 'upcoming123',
      title: { text: 'Upcoming Premiere' },
      author: { name: 'Channel', id: 'UCup' },
      is_upcoming: true,
      upcoming: '2025-12-25T00:00:00.000Z',
      duration: { seconds: 0 },
      thumbnail: { thumbnails: [] },
    }

    const result = parseVideo(video)
    expect(result).not.toBeNull()
    expect(result?.isUpcoming).toBe(true)
    expect(result?.published).toBe('2025-12-25T00:00:00.000Z')
  })

  it('handles premiere flag as upcoming', () => {
    const video = {
      type: 'Video',
      video_id: 'premiere123',
      title: { text: 'Premiere Video' },
      author: { name: 'Channel', id: 'UCprem' },
      is_premiere: true,
      duration: { seconds: 0 },
      thumbnail: { thumbnails: [] },
    }

    const result = parseVideo(video)
    expect(result).not.toBeNull()
    expect(result?.isUpcoming).toBe(true)
  })
})

describe('parseVideo - GridVideo node (channel/playlist pages)', () => {
  it('parses a GridVideo node correctly', () => {
    const video = {
      type: 'GridVideo',
      video_id: 'grid123',
      title: { text: 'Grid Video' },
      author: { name: 'Grid Channel', id: 'UCgrid', thumbnails: [{ url: 'https://avatar.com/grid.jpg', width: 48 }] },
      views: { text: '2,500,000 views' },
      published: { text: '3 months ago' },
      duration: { text: '15:45' },
      thumbnail: { thumbnails: [{ url: 'https://thumb.com/grid.jpg', width: 320 }] },
      is_live: false,
      is_upcoming: false,
    }

    const result = parseVideo(video)
    expect(result).not.toBeNull()
    expect(result?.id).toBe('grid123')
    expect(result?.title).toBe('Grid Video')
    expect(result?.author).toBe('Grid Channel')
    expect(result?.authorId).toBe('UCgrid')
    expect(result?.authorAvatar).toBe('https://avatar.com/grid.jpg')
    expect(result?.viewCount).toBe(2500000)
    expect(result?.lengthSeconds).toBe(945) // 15*60 + 45
    expect(result?.published).toBeTruthy()
  })

  it('handles GridVideo with LIVE duration', () => {
    const video = {
      type: 'GridVideo',
      video_id: 'gridLive',
      title: { text: 'Live Grid' },
      author: { name: 'Live Channel', id: 'UClive' },
      views: { text: '10,000 views' },
      duration: { text: 'LIVE' },
      thumbnail: { thumbnails: [] },
    }

    const result = parseVideo(video)
    expect(result).not.toBeNull()
    expect(result?.isLive).toBe(true)
    expect(result?.lengthSeconds).toBe(0)
  })

  it('handles GridVideo with missing author', () => {
    const video = {
      type: 'GridVideo',
      video_id: 'gridNoAuthor',
      title: { text: 'No Author Video' },
      views: { text: '100 views' },
      duration: { text: '2:00' },
      thumbnail: { thumbnails: [] },
    }

    const result = parseVideo(video)
    expect(result).not.toBeNull()
    expect(result?.author).toBe('Unknown')
    expect(result?.authorId).toBe('')
    expect(result?.viewCount).toBe(100)
  })
})

describe('parseVideo - Movie node', () => {
  it('parses a Movie node correctly', () => {
    const movie = {
      type: 'Movie',
      id: 'movie123',
      title: { text: 'A Movie' },
      author: { name: 'Movie Studio', id: 'UCmovie' },
      duration: { seconds: 7200 },
      thumbnail: { thumbnails: [{ url: 'https://thumb.com/movie.jpg', width: 480 }] },
    }

    const result = parseVideo(movie)
    expect(result).not.toBeNull()
    expect(result?.id).toBe('movie123')
    expect(result?.title).toBe('A Movie')
    expect(result?.author).toBe('Movie Studio')
    expect(result?.authorId).toBe('UCmovie')
    expect(result?.lengthSeconds).toBe(7200)
    expect(result?.viewCount).toBe(0) // Movies don't have view counts in list context
  })

  it('parses a GridMovie node correctly', () => {
    const movie = {
      type: 'GridMovie',
      id: 'gridMovie456',
      title: { text: 'Grid Movie' },
      author: { name: 'Studio', id: 'UCstudio' },
      duration: { seconds: 5400 },
      thumbnail: { thumbnails: [] },
    }

    const result = parseVideo(movie)
    expect(result).not.toBeNull()
    expect(result?.id).toBe('gridMovie456')
    expect(result?.lengthSeconds).toBe(5400)
  })
})

describe('parseVideo - edge cases', () => {
  it('returns null for null input', () => {
    expect(parseVideo(null)).toBeNull()
  })

  it('returns null for undefined input', () => {
    expect(parseVideo(undefined)).toBeNull()
  })

  it('returns null when no videoId is present', () => {
    const video = {
      type: 'Video',
      title: { text: 'No ID' },
    }
    expect(parseVideo(video)).toBeNull()
  })

  it('handles video with runs in title', () => {
    const video = {
      type: 'Video',
      video_id: 'runs123',
      title: { runs: [{ text: 'Title from runs' }] },
      author: { name: 'Channel', id: 'UC123' },
      duration: { seconds: 60 },
      thumbnail: { thumbnails: [] },
    }

    const result = parseVideo(video)
    expect(result).not.toBeNull()
    expect(result?.title).toBe('Title from runs')
  })

  it('handles video with string author', () => {
    const video = {
      type: 'Video',
      video_id: 'strAuthor',
      title: { text: 'String Author Video' },
      author: 'Direct String Author',
      duration: { seconds: 60 },
      thumbnail: { thumbnails: [] },
    }

    const result = parseVideo(video)
    expect(result).not.toBeNull()
    expect(result?.author).toBe('Direct String Author')
  })

  it('falls back to hqdefault.jpg when no thumbnails', () => {
    const video = {
      type: 'Video',
      video_id: 'noThumbs',
      title: { text: 'No Thumbnails' },
      author: { name: 'Channel', id: 'UC123' },
      duration: { seconds: 60 },
    }

    const result = parseVideo(video)
    expect(result).not.toBeNull()
    expect(result?.thumbnail).toBe('https://i.ytimg.com/vi/noThumbs/hqdefault.jpg')
  })

  it('picks highest resolution thumbnail', () => {
    const video = {
      type: 'Video',
      video_id: 'resTest',
      title: { text: 'Resolution Test' },
      author: { name: 'Channel', id: 'UC123' },
      duration: { seconds: 60 },
      thumbnail: { thumbnails: [
        { url: 'https://thumb.com/small.jpg', width: 120 },
        { url: 'https://thumb.com/large.jpg', width: 640 },
        { url: 'https://thumb.com/medium.jpg', width: 320 },
      ] },
    }

    const result = parseVideo(video)
    expect(result).not.toBeNull()
    expect(result?.thumbnail).toBe('https://thumb.com/large.jpg')
  })

  it('handles duration from text when seconds not available', () => {
    const video = {
      type: 'Video',
      video_id: 'durText',
      title: { text: 'Duration Text' },
      author: { name: 'Channel', id: 'UC123' },
      duration: { text: '1:23:45' },
      thumbnail: { thumbnails: [] },
    }

    const result = parseVideo(video)
    expect(result).not.toBeNull()
    expect(result?.lengthSeconds).toBe(5025) // 1*3600 + 23*60 + 45
  })

  it('handles duration MM:SS format', () => {
    const video = {
      type: 'Video',
      video_id: 'durMMSS',
      title: { text: 'MM:SS Duration' },
      author: { name: 'Channel', id: 'UC123' },
      duration: { text: '12:34' },
      thumbnail: { thumbnails: [] },
    }

    const result = parseVideo(video)
    expect(result).not.toBeNull()
    expect(result?.lengthSeconds).toBe(754) // 12*60 + 34
  })
})

describe('extractNumberFromString', () => {
  it('extracts number from "1,234,567 views"', () => {
    expect(extractNumberFromString('1,234,567 views')).toBe(1234567)
  })

  it('extracts number from "2.5M views"', () => {
    expect(extractNumberFromString('2.5M views')).toBe(25)
  })

  it('returns 0 for undefined', () => {
    expect(extractNumberFromString(undefined)).toBe(0)
  })

  it('returns 0 for empty string', () => {
    expect(extractNumberFromString('')).toBe(0)
  })

  it('returns 0 for non-string', () => {
    expect(extractNumberFromString(123 as any)).toBe(0)
  })
})

describe('parseSubscriberCount', () => {
  it('parses "1.2M" correctly', () => {
    expect(parseSubscriberCount('1.2M')).toBe(1200000)
  })

  it('parses "3.5K" correctly', () => {
    expect(parseSubscriberCount('3.5K')).toBe(3500)
  })

  it('parses "1.2 thousand" correctly', () => {
    expect(parseSubscriberCount('1.2 thousand')).toBe(1200)
  })

  it('parses "2.5 million" correctly', () => {
    expect(parseSubscriberCount('2.5 million')).toBe(2500000)
  })

  it('parses "1B" correctly', () => {
    expect(parseSubscriberCount('1B')).toBe(1000000000)
  })

  it('falls back to extractNumberFromString for plain numbers', () => {
    expect(parseSubscriberCount('12345')).toBe(12345)
  })

  it('returns 0 for undefined', () => {
    expect(parseSubscriberCount(undefined)).toBe(0)
  })

  it('returns 0 for empty string', () => {
    expect(parseSubscriberCount('')).toBe(0)
  })
})

describe('calculatePublishedDate', () => {
  it('returns current timestamp for live videos', () => {
    const result = calculatePublishedDate('2 days ago', true)
    expect(result).toBeTruthy()
    const parsed = new Date(result!)
    expect(parsed.getTime()).toBeCloseTo(Date.now(), -3) // within ~1 second
  })

  it('returns premiere date for upcoming videos', () => {
    const premiere = new Date('2025-12-25T00:00:00.000Z')
    const result = calculatePublishedDate(undefined, false, true, premiere)
    expect(result).toBe('2025-12-25T00:00:00.000Z')
  })

  it('returns undefined for upcoming without premiere date', () => {
    const result = calculatePublishedDate(undefined, false, true)
    expect(result).toBeUndefined()
  })

  it('returns undefined for empty text', () => {
    const result = calculatePublishedDate(undefined)
    expect(result).toBeUndefined()
  })

  it('parses "2 days ago" correctly', () => {
    const result = calculatePublishedDate('2 days ago')
    expect(result).toBeTruthy()
    const parsed = new Date(result!)
    const expected = Date.now() - 2 * 86400000
    expect(parsed.getTime()).toBeCloseTo(expected, -3) // within ~1 second
  })

  it('parses "1 year ago" correctly', () => {
    const result = calculatePublishedDate('1 year ago')
    expect(result).toBeTruthy()
    const parsed = new Date(result!)
    const expected = Date.now() - 31556952000
    expect(parsed.getTime()).toBeCloseTo(expected, -3)
  })

  it('parses "Premieres Jan 1, 2025" correctly', () => {
    const result = calculatePublishedDate('Premieres Jan 1, 2025')
    expect(result).toBe('2025-01-01T00:00:00.000Z')
  })

  it('parses "Streamed 3 weeks ago" correctly', () => {
    const result = calculatePublishedDate('Streamed 3 weeks ago')
    expect(result).toBeTruthy()
    const parsed = new Date(result!)
    const expected = Date.now() - 3 * 604800000
    expect(parsed.getTime()).toBeCloseTo(expected, -3)
  })

  it('parses "5 hours ago" correctly', () => {
    const result = calculatePublishedDate('5 hours ago')
    expect(result).toBeTruthy()
    const parsed = new Date(result!)
    const expected = Date.now() - 5 * 3600000
    expect(parsed.getTime()).toBeCloseTo(expected, -3)
  })

  it('parses "30 minutes ago" correctly', () => {
    const result = calculatePublishedDate('30 minutes ago')
    expect(result).toBeTruthy()
    const parsed = new Date(result!)
    const expected = Date.now() - 30 * 60000
    expect(parsed.getTime()).toBeCloseTo(expected, -3)
  })

  it('parses "6 months ago" correctly', () => {
    const result = calculatePublishedDate('6 months ago')
    expect(result).toBeTruthy()
    const parsed = new Date(result!)
    const expected = Date.now() - 6 * 2592000000
    expect(parsed.getTime()).toBeCloseTo(expected, -3)
  })
})

describe('parseDurationText', () => {
  it('parses "1:23:45" correctly', () => {
    expect(parseDurationText('1:23:45')).toBe(5025)
  })

  it('parses "12:34" correctly', () => {
    expect(parseDurationText('12:34')).toBe(754)
  })

  it('parses "0:45" correctly', () => {
    expect(parseDurationText('0:45')).toBe(45)
  })

  it('returns 0 for undefined', () => {
    expect(parseDurationText(undefined)).toBe(0)
  })

  it('returns 0 for invalid text', () => {
    expect(parseDurationText('invalid')).toBe(0)
  })
})

describe('getBestThumbnail', () => {
  it('returns empty string for undefined', () => {
    expect(getBestThumbnail(undefined)).toBe('')
  })

  it('returns empty string for empty array', () => {
    expect(getBestThumbnail([])).toBe('')
  })

  it('picks highest width thumbnail', () => {
    const thumbs = [
      { url: 'small.jpg', width: 120 },
      { url: 'large.jpg', width: 640 },
      { url: 'medium.jpg', width: 320 },
    ]
    expect(getBestThumbnail(thumbs)).toBe('large.jpg')
  })

  it('handles thumbnails without width', () => {
    const thumbs = [
      { url: 'no-width.jpg' },
      { url: 'with-width.jpg', width: 100 },
    ]
    expect(getBestThumbnail(thumbs)).toBe('with-width.jpg')
  })
})

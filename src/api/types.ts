export interface Video {
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
  chapters: Chapter[]
  captions: CaptionTrack[]
  related: Video[]
}

export interface Chapter {
  title: string
  startSeconds: number
  thumbnail: string
}

export interface CaptionTrack {
  languageCode: string
  name: string
  url: string
}

export interface Channel {
  id: string
  name: string
  description: string
  avatar: string
  banner: string
  subscriberCount: number
  videoCount: number
  tabs: string[]
  videos: Video[]
  relatedChannels: Channel[]
}

export interface Playlist {
  id: string
  title: string
  description: string
  author: string
  authorId: string
  videoCount: number
  videos: Video[]
}

export interface Comment {
  id: string
  author: string
  authorId: string
  authorAvatar: string
  content: string
  likeCount: number
  published: string
  replies: Comment[]
  replyCount: number
}

export interface SponsorBlockSegment {
  category: string
  segment: [number, number]
  UUID: string
  videoDuration: number
  actionType: string
}

export interface DownloadArgs {
  videoId: string
  mode: 'video' | 'audio' | 'custom'
  quality?: string
  format?: string
  subtitles?: boolean
  outputPath?: string
}

export interface DownloadStatus {
  id: number
  videoId: string
  title: string
  status: string
  progress: number
  speed: string
  eta: string
  outputPath: string
}

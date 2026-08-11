const SPONSORBLOCK_API = 'https://sponsor.ajay.app/api'

export interface SponsorBlockSegment {
  category: string
  segment: [number, number]
  UUID: string
  videoDuration: number
  actionType: string
  locked: number
  votes: number
  description: string
}

const CATEGORIES = [
  'sponsor', 'intro', 'outro', 'selfpromo', 'interaction',
  'music_offtopic', 'preview', 'filler', 'highlight',
]

export async function getSegments(videoId: string, categories = CATEGORIES): Promise<SponsorBlockSegment[]> {
  try {
    const catParam = JSON.stringify(categories)
    const response = await fetch(
      `${SPONSORBLOCK_API}/skipSegments?videoID=${videoId}&categories=${encodeURIComponent(catParam)}`,
      { signal: AbortSignal.timeout(5000) }
    )
    if (!response.ok) return []
    const data = await response.json()
    return data.map((s: any) => ({
      category: s.category,
      segment: s.segment,
      UUID: s.UUID,
      videoDuration: s.videoDuration,
      actionType: s.actionType,
      locked: s.locked,
      votes: s.votes,
      description: s.description || '',
    }))
  } catch {
    return []
  }
}

export async function getSegmentsByCategory(videoId: string): Promise<Record<string, SponsorBlockSegment[]>> {
  try {
    const catParam = JSON.stringify(CATEGORIES)
    const response = await fetch(
      `${SPONSORBLOCK_API}/skipSegments/${videoId}?categories=${encodeURIComponent(catParam)}`,
      { signal: AbortSignal.timeout(5000) }
    )
    if (!response.ok) return {}
    const data = await response.json()
    const grouped: Record<string, SponsorBlockSegment[]> = {}
    for (const seg of data) {
      const cat = seg.category
      if (!grouped[cat]) grouped[cat] = []
      grouped[cat].push({
        category: seg.category,
        segment: seg.segment,
        UUID: seg.UUID,
        videoDuration: seg.videoDuration,
        actionType: seg.actionType,
        locked: seg.locked,
        votes: seg.votes,
        description: seg.description || '',
      })
    }
    return grouped
  } catch {
    return {}
  }
}

export async function viewSegment(UUID: string): Promise<void> {
  try {
    await fetch(`${SPONSORBLOCK_API}/viewedVideoSponsorTime?UUID=${UUID}`)
  } catch {
    // Ignore errors
  }
}

export async function submitSegment(
  videoId: string,
  category: string,
  startTime: number,
  endTime: number
): Promise<boolean> {
  try {
    const response = await fetch(`${SPONSORBLOCK_API}/skipSegments`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        videoID: videoId,
        userID: 'opentubex-user',
        segments: [{ category, segment: [startTime, endTime] }],
      }),
    })
    return response.ok
  } catch {
    return false
  }
}

export function formatCategory(category: string): string {
  return category
    .replace(/_/g, ' ')
    .replace(/\b\w/g, (c) => c.toUpperCase())
}

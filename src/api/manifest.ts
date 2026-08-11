import { getCurrentInstance } from './invidious'

export function getLocalManifestUrl(dashManifestUrl: string, poToken?: string): string {
  let url = dashManifestUrl
  if (poToken) {
    url += (url.includes('?') ? '&' : '?') + 'pot=' + poToken
  }
  return url
}

export function getInvidiousManifestUrl(videoId: string): string {
  const instance = getCurrentInstance()
  return `${instance.url}/api/manifest/dash/id/${videoId}?local=true`
}

export function getInvidiousDashManifest(videoId: string, format: 'dash' | 'blob' = 'dash'): string {
  const instance = getCurrentInstance()
  return `${instance.url}/api/manifest/dash/id/${videoId}?format=${format}`
}

export function getLegacyStreamUrl(format: any, poToken?: string): string {
  let url = format.url || format.mp4_url || format.audio_url || ''
  if (poToken && url) {
    url += (url.includes('?') ? '&' : '?') + 'pot=' + poToken
  }
  return url
}

import { ref, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import shaka from 'shaka-player/dist/shaka-player.compiled'

export function usePlayer() {
  const videoRef = ref<HTMLVideoElement | null>(null)
  const player = ref<any>(null)
  const isPlaying = ref(false)
  const currentTime = ref(0)
  const duration = ref(0)
  const volume = ref(1)
  const isMuted = ref(false)
  const isFullscreen = ref(false)
  const qualities = ref<any[]>([])
  const currentQuality = ref<string>('auto')
  const playbackRate = ref(1)
  const error = ref<string | null>(null)
  const isLoading = ref(false)

  async function init(videoElement: HTMLVideoElement) {
    videoRef.value = videoElement

    shaka.polyfill.installAll()

    if (!shaka.Player.isBrowserSupported()) {
      error.value = 'Browser not supported for Shaka Player'
      return false
    }

    player.value = new shaka.Player(videoElement)

    player.value.addEventListener('error', (e: any) => {
      error.value = e.detail?.message || 'Playback error'
    })

    player.value.addEventListener('loading', () => {
      isLoading.value = true
    })

    player.value.addEventListener('loaded', () => {
      isLoading.value = false
      updateQualities()
    })

    const interval = setInterval(() => {
      if (videoElement) {
        currentTime.value = videoElement.currentTime
        duration.value = videoElement.duration || 0
        isPlaying.value = !videoElement.paused
      }
    }, 250)

    onUnmounted(() => clearInterval(interval))

    return true
  }

  async function generatePoToken(videoId: string): Promise<string | undefined> {
    try {
      const context = JSON.stringify({
        client: {
          clientName: 'WEB',
          clientVersion: '2.20240101.01.00',
          hl: 'en',
          gl: 'US',
        },
      })

      const token = await invoke<string>('generate_po_token', {
        videoId,
        context,
        proxyUrl: undefined,
      })
      return token
    } catch {
      return undefined
    }
  }

  async function loadManifest(manifestUri: string, poToken?: string, videoId?: string) {
    if (!player.value) return

    isLoading.value = true
    error.value = null

    try {
      player.value.configure({
        streaming: {
          bufferingGoal: 10,
          bufferBehind: 30,
          retryParameters: {
            maxAttempts: 3,
            baseDelay: 1000,
            backoffFactor: 2,
          },
        },
        manifest: {
          retryParameters: {
            maxAttempts: 3,
            baseDelay: 1000,
            backoffFactor: 2,
          },
        },
      })

      let url = manifestUri
      let token = poToken

      if (!token && videoId) {
        token = await generatePoToken(videoId)
      }

      if (token) {
        url += (url.includes('?') ? '&' : '?') + 'pot=' + token
      }

      await player.value.load(url)
      updateQualities()
    } catch (e: any) {
      error.value = e.message || 'Failed to load video'
    } finally {
      isLoading.value = false
    }
  }

  function updateQualities() {
    if (!player.value) return
    const tracks = player.value.getVariantTracks()
    qualities.value = tracks.filter((t: any) => t.type === 'variant')
  }

  function play() {
    videoRef.value?.play()
  }

  function pause() {
    videoRef.value?.pause()
  }

  function togglePlay() {
    if (isPlaying.value) pause()
    else play()
  }

  function seek(time: number) {
    if (videoRef.value) {
      videoRef.value.currentTime = time
    }
  }

  function seekRelative(seconds: number) {
    if (videoRef.value) {
      videoRef.value.currentTime = Math.max(0, Math.min(videoRef.value.duration, videoRef.value.currentTime + seconds))
    }
  }

  function setVolume(v: number) {
    volume.value = Math.max(0, Math.min(1, v))
    if (videoRef.value) {
      videoRef.value.volume = volume.value
    }
    isMuted.value = volume.value === 0
  }

  function toggleMute() {
    isMuted.value = !isMuted.value
    if (videoRef.value) {
      videoRef.value.muted = isMuted.value
    }
  }

  function setQuality(qualityId: string) {
    if (!player.value) return
    if (qualityId === 'auto') {
      player.value.configure({ abr: { enabled: true } })
    } else {
      player.value.configure({ abr: { enabled: false } })
      const track = qualities.value.find((t: any) => t.id.toString() === qualityId)
      if (track) {
        player.value.selectVariantTrack(track, true)
      }
    }
    currentQuality.value = qualityId
  }

  function setRate(rate: number) {
    playbackRate.value = rate
    if (videoRef.value) {
      videoRef.value.playbackRate = rate
    }
  }

  async function toggleFullscreen() {
    const container = videoRef.value?.parentElement
    if (!container) return
    if (!document.fullscreenElement) {
      await container.requestFullscreen()
      isFullscreen.value = true
    } else {
      await document.exitFullscreen()
      isFullscreen.value = false
    }
  }

  function destroy() {
    player.value?.destroy()
    player.value = null
  }

  onUnmounted(destroy)

  return {
    videoRef, isPlaying, currentTime, duration, volume, isMuted,
    isFullscreen, qualities, currentQuality, playbackRate, error, isLoading,
    init, loadManifest, generatePoToken, play, pause, togglePlay, seek, seekRelative,
    setVolume, toggleMute, setQuality, setRate, toggleFullscreen, destroy
  }
}

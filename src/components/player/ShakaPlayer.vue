<script setup lang="ts">
/**
 * ShakaPlayer.vue
 *
 * Ported from OpenTubeX's ft-shaka-video-player to work with Slytube's
 * Vue 3 Composition API + Tauri stack. Retains the core Shaka Player
 * configuration (buffering, manifest settings, quality selection, captions)
 * while dropping Electron- and FreeTube-specific machinery.
 */

import { ref, onMounted, onBeforeUnmount, watch } from 'vue'
import shaka from 'shaka-player/dist/shaka-player.compiled'
import PlayerControls from './PlayerControls.vue'
import PlayerSeekbar from './PlayerSeekbar.vue'
import { useKeyboardShortcuts } from '../../composables/useKeyboardShortcuts'

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

interface Segment {
  category: string
  segment: [number, number]
  UUID: string
}

interface Chapter {
  title: string
  startSeconds: number
}

interface CaptionTrack {
  url: string
  language: string
  label: string
  mimeType: string
  kind?: string
}

/** A variant track as returned by shaka.Player.getVariantTracks(). */
interface VariantTrack {
  id: number
  type: string
  active: boolean
  bandwidth: number
  width: number | null
  height: number | null
  codecs: string | null
}

// ---------------------------------------------------------------------------
// Props & Emits
// ---------------------------------------------------------------------------

const props = defineProps<{
  manifestUrl?: string
  poToken?: string
  videoId?: string
  title?: string
  segments?: Segment[]
  chapters?: Chapter[]
  captions?: CaptionTrack[]
  startTime?: number
  manifestMimeType?: string
}>()

const emit = defineEmits<{
  error: [message: string]
  loaded: []
  timeupdate: [currentTime: number]
  ended: []
  pause: []
  seeking: []
}>()

// ---------------------------------------------------------------------------
// Shaka Player state
// ---------------------------------------------------------------------------

/** The shaka.Player instance driving playback. */
let player: any = null

/** Whether Shaka Player has finished loading the manifest. */
const hasLoaded = ref(false)

/** Whether we are currently buffering. */
const isBuffering = ref(false)

// ---------------------------------------------------------------------------
// Refs for the DOM elements
// ---------------------------------------------------------------------------

const containerRef = ref<HTMLDivElement | null>(null)
const videoRef = ref<HTMLVideoElement | null>(null)

// ---------------------------------------------------------------------------
// Playback state (mirrored from the video element for the UI)
// ---------------------------------------------------------------------------

const isPlaying = ref(false)
const currentTime = ref(0)
const duration = ref(0)
const volume = ref(1)
const isMuted = ref(false)
const isFullscreen = ref(false)
const qualities = ref<VariantTrack[]>([])
const currentQuality = ref<string>('auto')
const playbackRate = ref(1)
const error = ref<string | null>(null)
const isLoading = ref(false)

// ---------------------------------------------------------------------------
// Controls visibility
// ---------------------------------------------------------------------------

const showControls = ref(true)
let controlsTimeout: ReturnType<typeof setTimeout> | null = null

function resetControlsTimeout() {
  showControls.value = true
  if (controlsTimeout) clearTimeout(controlsTimeout)
  controlsTimeout = setTimeout(() => {
    if (isPlaying.value) {
      showControls.value = false
    }
  }, 3000)
}

function onPlayerMouseMove() {
  resetControlsTimeout()
}

// ---------------------------------------------------------------------------
// Quality selection – ported from OpenTubeX's setDashQuality / getActiveVariantQuality
// ---------------------------------------------------------------------------

/**
 * Derive a human-readable quality label ("720p", "1080p", …) from a variant's
 * dimensions, matching OpenTubeX's getQualityFromDimensions behaviour.
 */
function getQualityFromDimensions(width: number, height: number): string {
  const resolution = height > width ? width : height
  return `${resolution}p`
}

/**
 * Returns the quality label (e.g. "720p") of the *active* variant track, or
 * "auto" when no variant is active yet.
 */
function getActiveVariantQuality(): string {
  if (!player) return 'auto'
  const tracks: VariantTrack[] = player.getVariantTracks()
  const active = tracks.find((t: VariantTrack) => t.active)
  if (!active || !active.height) return 'auto'
  return getQualityFromDimensions(active.width ?? 0, active.height)
}

/**
 * Switch to a specific quality. When `qualityId` is `"auto"` we hand control
 * back to Shaka's ABR; otherwise we pick the closest-matching variant.
 *
 * Simplified version of OpenTubeX's setDashQuality that omits the audio-bandwidth
 * and label-matching logic (specific to FreeTube's multi-audio handling).
 */
function setQuality(qualityId: string) {
  if (!player) return

  if (qualityId === 'auto') {
    player.configure({ abr: { enabled: true } })
    currentQuality.value = 'auto'
    return
  }

  // Disable ABR so the manual selection sticks.
  player.configure({ abr: { enabled: false } })

  const targetHeight = parseInt(qualityId.replace('p', ''), 10)
  if (Number.isNaN(targetHeight)) return

  const variants: VariantTrack[] = player.getVariantTracks()

  // First look for an exact resolution match.
  let matches = variants.filter((v: VariantTrack) => {
    if (!v.height) return false
    const res = v.height > (v.width ?? 0) ? v.width ?? 0 : v.height
    return res === targetHeight
  })

  // Fall back to variants *at or above* the requested quality.
  if (matches.length === 0) {
    matches = variants.filter((v: VariantTrack) => {
      if (!v.height) return false
      const res = v.height > (v.width ?? 0) ? v.width ?? 0 : v.height
      return res >= targetHeight
    })
  }

  // Prefer the highest resolution available among the matches.
  matches.sort((a: VariantTrack, b: VariantTrack) => {
    const resA = (a.height ?? 0) > (a.width ?? 0) ? a.width ?? 0 : a.height ?? 0
    const resB = (b.height ?? 0) > (b.width ?? 0) ? b.width ?? 0 : b.height ?? 0
    return resA - resB
  })

  if (matches.length > 0) {
    player.selectVariantTrack(matches[0], true)
  }

  currentQuality.value = qualityId
}

/**
 * Rebuild the `qualities` ref from the player's current variant tracks.
 */
function updateQualities() {
  if (!player) return
  const tracks: VariantTrack[] = player.getVariantTracks()
  // Deduplicate by height – YouTube advertises the same resolution in
  // several codecs, and we surface one entry per height.
  const seen = new Set<number>()
  qualities.value = tracks.filter((t: VariantTrack) => {
    const res = (t.height ?? 0) > (t.width ?? 0) ? t.width ?? 0 : t.height ?? 0
    if (seen.has(res)) return false
    seen.add(res)
    return true
  })
}

// ---------------------------------------------------------------------------
// Caption support – ported from OpenTubeX's handleLoaded
// ---------------------------------------------------------------------------

/**
 * Add externally-provided caption tracks to the player. Mirrors the
 * addTextTrackAsync loop in OpenTubeX's handleLoaded.
 */
async function addCaptionTracks() {
  if (!player || !props.captions || props.captions.length === 0) return

  for (const caption of props.captions) {
    try {
      await player.addTextTrackAsync(
        caption.url,
        caption.language,
        caption.kind || 'captions',
        caption.mimeType,
        undefined, // codec (only needed for container-wrapped captions)
        caption.label
      )
    } catch (err) {
      // Caption failures are non-fatal; log and continue.
      console.error('[ShakaPlayer] Failed to add caption track:', caption.language, err)
    }
  }
}

// ---------------------------------------------------------------------------
// Shaka Player configuration – ported from OpenTubeX's getPlayerConfig
// ---------------------------------------------------------------------------

function getPlayerConfig(): Record<string, any> {
  // Start from Shaka's default retry parameters so we get all required
  // fields (connectionTimeout, fuzzFactor, stallTimeout, …).
  const streamingRetry = shaka.net.NetworkingEngine.defaultRetryParameters()
  const manifestRetry = shaka.net.NetworkingEngine.defaultRetryParameters()

  return {
    // YouTube uses these values and they work well.
    streaming: {
      bufferingGoal: 180,
      rebufferingGoal: 0.02,
      bufferBehind: 300,
      retryParameters: {
        ...streamingRetry,
        maxAttempts: 3,
        baseDelay: 1000,
        backoffFactor: 2,
        timeout: 30_000,
      },
    },
    manifest: {
      retryParameters: {
        ...manifestRetry,
        maxAttempts: 3,
        baseDelay: 1000,
        backoffFactor: 2,
        timeout: 30_000,
      },
      // Makes captions work for live streams with no downside for VOD.
      segmentRelativeVttTiming: true,
    },
    abr: {
      enabled: true,
      restrictToElementSize: true,
    },
    // Prioritise variants predicted to play smoothly and power-efficiently.
    preferredDecodingAttributes: ['smooth', 'powerEfficient'],
  }
}

// ---------------------------------------------------------------------------
// Error handling – ported from OpenTubeX's handleError
// ---------------------------------------------------------------------------

const ErrorSeverity = shaka.util.Error?.Severity
const ErrorCategory = shaka.util.Error?.Category
const ErrorCode = shaka.util.Error?.Code

function handleError(err: any, context: string) {
  if (!err || typeof err !== 'object') {
    console.error(`[ShakaPlayer] Unknown error in ${context}:`, err)
    return
  }

  // Unwrap filter-wrapper errors.
  while (err && (err.code === ErrorCode?.REQUEST_FILTER_ERROR || err.code === ErrorCode?.RESPONSE_FILTER_ERROR)) {
    err = err.data?.[0]
  }

  // Recoverable network errors – let Shaka retry.
  if (err?.severity === ErrorSeverity?.RECOVERABLE && err?.category === ErrorCategory?.NETWORK) {
    console.warn(`[ShakaPlayer] Recoverable network error in ${context}:`, err.code, err.data)
    return
  }

  // Aborts / lifecycle interruptions that are not evidence of a real failure.
  if (
    err &&
    (err.code === ErrorCode?.OPERATION_ABORTED ||
      err.code === ErrorCode?.LOAD_INTERRUPTED ||
      err.code === ErrorCode?.OBJECT_DESTROYED ||
      err.code === ErrorCode?.CONTENT_NOT_LOADED ||
      err.code === ErrorCode?.PRELOAD_DESTROYED)
  ) {
    console.warn(`[ShakaPlayer] Ignoring abort/interruption (code ${err.code}) in ${context}`)
    return
  }

  // Text-related errors (captions, thumbnails) are non-fatal.
  if (err?.category === ErrorCategory?.TEXT) {
    console.warn(`[ShakaPlayer] Text-track error in ${context}:`, err)
    return
  }

  console.error(`[ShakaPlayer] Error in ${context}:`, err)
  const message = err?.message || err?.code?.toString() || 'Playback error'
  error.value = message
  emit('error', message)
}

// ---------------------------------------------------------------------------
// Manifest loading – ported from OpenTubeX's performFirstLoad
// ---------------------------------------------------------------------------

async function loadManifest(manifestUri: string, poToken?: string, videoId?: string) {
  if (!player) return

  isLoading.value = true
  error.value = null

  try {
    player.configure(getPlayerConfig())

    let url = manifestUri
    let token = poToken

    // If no explicit token was passed but we have a videoId, ask the Tauri
    // backend to generate one (mirrors the existing usePlayer composable).
    if (!token && videoId) {
      try {
        const { invoke } = await import('@tauri-apps/api/core')
        const context = JSON.stringify({
          client: {
            clientName: 'WEB',
            clientVersion: '2.20240101.01.00',
            hl: 'en',
            gl: 'US',
          },
        })
        token = await invoke<string>('generate_po_token', {
          videoId,
          context,
          proxyUrl: undefined,
        })
      } catch {
        // Token generation is best-effort; continue without it.
      }
    }

    if (token) {
      url += (url.includes('?') ? '&' : '?') + 'pot=' + token
    }

    await player.load(url, props.startTime ?? null, props.manifestMimeType ?? undefined)
    updateQualities()
  } catch (err: any) {
    handleError(err, 'loadManifest')
  } finally {
    isLoading.value = false
  }
}

// ---------------------------------------------------------------------------
// Live detection – mirrors OpenTubeX's isLive tracking
// ---------------------------------------------------------------------------

const isLive = ref(false)

// ---------------------------------------------------------------------------
// Player lifecycle – ported from OpenTubeX's onMounted / destroyPlayer
// ---------------------------------------------------------------------------

async function initializePlayer() {
  const videoElement = videoRef.value
  if (!videoElement) return

  // Install polyfills.
  shaka.polyfill.installAll()

  if (!shaka.Player.isBrowserSupported()) {
    const msg = 'Your browser is not supported for Shaka Player'
    error.value = msg
    emit('error', msg)
    return false
  }

  player = new shaka.Player(videoElement)

  // Attach the player to the media element *before* configuring.
  await player.attach(videoElement)

  // ---- Event listeners ---------------------------------------------------

  player.addEventListener('error', (event: any) => {
    handleError(event.detail, 'shaka error handler')
  })

  player.addEventListener('buffering', (event: any) => {
    isBuffering.value = event.detail?.buffering ?? false
  })

  player.addEventListener('loading', () => {
    hasLoaded.value = false
    isLoading.value = true
  })

  player.addEventListener('loaded', async () => {
    hasLoaded.value = true
    isLoading.value = false
    isLive.value = player.isLive()
    emit('loaded')

    // Add external caption tracks after the manifest is loaded.
    await addCaptionTracks()
  })

  // Track quality changes when the user has manual quality disabled.
  player.addEventListener('variantchanged', () => {
    if (player.getConfiguration().abr.enabled) return
    const quality = getActiveVariantQuality()
    if (quality !== 'auto') {
      currentQuality.value = quality
    }
  })

  // ---- Video element event listeners -------------------------------------

  videoElement.addEventListener('play', onPlay)
  videoElement.addEventListener('playing', onPlaying)
  videoElement.addEventListener('pause', onPause)
  videoElement.addEventListener('ended', onEnded)
  videoElement.addEventListener('seeking', onSeeking)
  videoElement.addEventListener('timeupdate', onTimeupdate)
  videoElement.addEventListener('volumechange', onVolumeChange)

  // Time tracking interval for the reactive refs.
  timeTrackingInterval = window.setInterval(() => {
    if (videoElement) {
      currentTime.value = videoElement.currentTime
      duration.value = videoElement.duration || 0
      isPlaying.value = !videoElement.paused
    }
  }, 250)

  // ---- Load the manifest -------------------------------------------------

  if (props.manifestUrl) {
    await loadManifest(props.manifestUrl, props.poToken, props.videoId)
  }

  return true
}

// ---------------------------------------------------------------------------
// Playback event handlers – ported from OpenTubeX's handlePlay / handlePlaying / …
// ---------------------------------------------------------------------------

function onPlay() {
  isPlaying.value = true
  isBuffering.value = false
}

function onPlaying() {
  isBuffering.value = false
}

function onPause() {
  isPlaying.value = false
  emit('pause')
}

function onEnded() {
  isPlaying.value = false
  emit('ended')
}

function onSeeking() {
  emit('seeking')
}

function onTimeupdate() {
  if (!videoRef.value) return
  currentTime.value = videoRef.value.currentTime
  emit('timeupdate', currentTime.value)
}

function onVolumeChange() {
  if (!videoRef.value) return
  volume.value = videoRef.value.muted ? 0 : videoRef.value.volume
  isMuted.value = videoRef.value.muted
}

// ---------------------------------------------------------------------------
// Playback controls – used by the UI
// ---------------------------------------------------------------------------

function togglePlay() {
  if (!videoRef.value) return
  if (videoRef.value.paused) {
    videoRef.value.play()
  } else {
    videoRef.value.pause()
  }
}

function seek(time: number) {
  if (videoRef.value) {
    videoRef.value.currentTime = time
  }
}

function seekRelative(seconds: number) {
  if (videoRef.value) {
    videoRef.value.currentTime = Math.max(
      0,
      Math.min(videoRef.value.duration, videoRef.value.currentTime + seconds)
    )
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

function setRate(rate: number) {
  playbackRate.value = rate
  if (videoRef.value) {
    videoRef.value.playbackRate = rate
  }
}

async function toggleFullscreen() {
  if (!containerRef.value) return
  if (!document.fullscreenElement) {
    await containerRef.value.requestFullscreen()
    isFullscreen.value = true
  } else {
    await document.exitFullscreen()
    isFullscreen.value = false
  }
}

// ---------------------------------------------------------------------------
// Time tracking interval
// ---------------------------------------------------------------------------

let timeTrackingInterval: number | undefined

// ---------------------------------------------------------------------------
// Watch for manifest URL changes (e.g. user loads a new video)
// ---------------------------------------------------------------------------

watch(
  () => props.manifestUrl,
  async (newUrl) => {
    if (newUrl && player) {
      await loadManifest(newUrl, props.poToken, props.videoId)
    }
  }
)

// ---------------------------------------------------------------------------
// Keyboard shortcuts (same as existing Slytube player)
// ---------------------------------------------------------------------------

const { register, unregister } = useKeyboardShortcuts()

function registerShortcuts() {
  register('space', () => { if (hasLoaded.value) togglePlay() })
  register('left', () => { if (hasLoaded.value) seekRelative(-5) })
  register('right', () => { if (hasLoaded.value) seekRelative(5) })
  register('up', () => { if (hasLoaded.value) setVolume(volume.value + 0.1) })
  register('down', () => { if (hasLoaded.value) setVolume(volume.value - 0.1) })
  register('f', () => { if (hasLoaded.value) toggleFullscreen() })
  register('m', () => { if (hasLoaded.value) toggleMute() })
}

function unregisterShortcuts() {
  unregister('space')
  unregister('left')
  unregister('right')
  unregister('up')
  unregister('down')
  unregister('f')
  unregister('m')
}

// ---------------------------------------------------------------------------
// Lifecycle hooks
// ---------------------------------------------------------------------------

onMounted(async () => {
  await initializePlayer()
  registerShortcuts()
})

onBeforeUnmount(async () => {
  // Clear intervals.
  if (timeTrackingInterval) {
    clearInterval(timeTrackingInterval)
    timeTrackingInterval = undefined
  }
  if (controlsTimeout) {
    clearTimeout(controlsTimeout)
    controlsTimeout = null
  }

  // Unregister keyboard shortcuts.
  unregisterShortcuts()

  // Destroy the Shaka Player instance.
  if (player) {
    try {
      await player.destroy()
    } catch (err) {
      console.error('[ShakaPlayer] Error during destroy:', err)
    }
    player = null
  }
})

// ---------------------------------------------------------------------------
// Expose imperative API for the parent (mirrors OpenTubeX's expose block)
// ---------------------------------------------------------------------------

defineExpose({
  togglePlay,
  seek,
  seekRelative,
  setVolume,
  toggleMute,
  setRate,
  toggleFullscreen,
  getCurrentTime: () => videoRef.value?.currentTime ?? 0,
  get isPlaying() { return isPlaying.value },
  get hasLoaded() { return hasLoaded.value },
})
</script>

<template>
  <div
    ref="containerRef"
    class="relative w-full aspect-video bg-black rounded-xl overflow-hidden group/player select-none"
    @mousemove="onPlayerMouseMove"
    @mouseleave="showControls = false"
  >
    <!-- Video Element – structure ported from OpenTubeX's ft-shaka-video-player.vue -->
    <video
      ref="videoRef"
      class="w-full h-full object-contain"
      preload="auto"
      crossorigin="anonymous"
      playsinline
    />

    <!-- Loading Spinner -->
    <div
      v-if="isLoading || isBuffering"
      class="absolute inset-0 flex items-center justify-center bg-black/40 pointer-events-none"
    >
      <div class="size-12 border-4 border-white/30 border-t-white rounded-full animate-spin" />
    </div>

    <!-- Error Display -->
    <div
      v-if="error"
      class="absolute inset-0 flex items-center justify-center bg-black/70 z-10"
    >
      <div class="text-center p-6">
        <svg
          class="size-12 text-red-400 mx-auto mb-3"
          fill="none"
          viewBox="0 0 24 24"
          stroke="currentColor"
          stroke-width="2"
        >
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L4.082 16.5c-.77.833.192 2.5 1.732 2.5z"
          />
        </svg>
        <p class="text-white text-sm font-medium">{{ error }}</p>
        <button
          class="mt-3 px-4 py-1.5 bg-white/10 hover:bg-white/20 text-white text-xs rounded-lg transition-colors"
          @click="loadManifest(props.manifestUrl!, props.poToken, props.videoId)"
        >
          Retry
        </button>
      </div>
    </div>

    <!-- Controls Overlay – only shown once playback has started -->
    <div
      v-if="hasLoaded"
      class="absolute bottom-0 left-0 right-0 bg-gradient-to-t from-black/80 to-transparent transition-opacity duration-300"
      :class="showControls || !isPlaying ? 'opacity-100' : 'opacity-0 pointer-events-none'"
    >
      <PlayerControls
        :is-playing="isPlaying"
        :current-time="currentTime"
        :duration="duration"
        :volume="volume"
        :is-muted="isMuted"
        :is-fullscreen="isFullscreen"
        :qualities="qualities.map((q) => ({ id: getQualityFromDimensions(q.width ?? 0, q.height ?? 0), height: (q.height ?? 0) > (q.width ?? 0) ? q.width ?? 0 : q.height ?? 0, bandwidth: q.bandwidth }))"
        :current-quality="currentQuality"
        :playback-rate="playbackRate"
        @toggle-play="togglePlay"
        @seek-relative="seekRelative"
        @set-volume="setVolume"
        @toggle-mute="toggleMute"
        @set-quality="setQuality"
        @set-rate="setRate"
        @toggle-fullscreen="toggleFullscreen"
      >
        <template #seekbar>
          <div class="px-3">
            <PlayerSeekbar
              :current-time="currentTime"
              :duration="duration"
              :segments="segments"
              :chapters="chapters"
              @seek="seek"
            />
          </div>
        </template>
      </PlayerControls>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from 'vue'
import { usePlayer } from '../../composables/usePlayer'
import PlayerControls from './PlayerControls.vue'
import PlayerSeekbar from './PlayerSeekbar.vue'

interface Segment {
  category: string
  segment: [number, number]
  UUID: string
}

interface Chapter {
  title: string
  startSeconds: number
}

const props = defineProps<{
  manifestUrl?: string
  poToken?: string
  title?: string
  segments?: Segment[]
  chapters?: Chapter[]
}>()

const emit = defineEmits<{
  ready: []
  error: [message: string]
}>()

const {
  videoRef, isPlaying, currentTime, duration, volume, isMuted,
  isFullscreen, qualities, currentQuality, playbackRate, error, isLoading,
  init, loadManifest, togglePlay, seek, seekRelative,
  setVolume, toggleMute, setQuality, setRate, toggleFullscreen
} = usePlayer()

const containerRef = ref<HTMLElement | null>(null)
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

function onPlayerClick() {
  togglePlay()
  resetControlsTimeout()
}

function onPlayerDoubleClick() {
  toggleFullscreen()
}

async function handleInit() {
  if (!videoRef.value) return
  const supported = await init(videoRef.value)
  if (supported && props.manifestUrl) {
    await loadManifest(props.manifestUrl, props.poToken)
    emit('ready')
  }
}

onMounted(handleInit)

watch(error, (newError) => {
  if (newError) {
    emit('error', newError)
  }
})

onUnmounted(() => {
  if (controlsTimeout) clearTimeout(controlsTimeout)
})
</script>

<template>
  <div
    ref="containerRef"
    class="relative w-full aspect-video bg-black rounded-xl overflow-hidden group/player select-none"
    @mousemove="onPlayerMouseMove"
    @mouseleave="showControls = false"
  >
    <!-- Video Element -->
    <video
      ref="videoRef"
      class="w-full h-full object-contain"
      @click="onPlayerClick"
      @dblclick="onPlayerDoubleClick"
    />
    
    <!-- Loading Spinner -->
    <div
      v-if="isLoading"
      class="absolute inset-0 flex items-center justify-center bg-black/40 pointer-events-none"
    >
      <div class="size-12 border-4 border-white/30 border-t-white rounded-full animate-spin" />
    </div>
    
    <!-- Error Display -->
    <div
      v-if="error"
      class="absolute inset-0 flex items-center justify-center bg-black/70"
    >
      <div class="text-center p-6">
        <svg class="size-12 text-red-400 mx-auto mb-3" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
          <path stroke-linecap="round" stroke-linejoin="round" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L4.082 16.5c-.77.833.192 2.5 1.732 2.5z" />
        </svg>
        <p class="text-white text-sm font-medium">{{ error }}</p>
        <button
          class="mt-3 px-4 py-1.5 bg-white/10 hover:bg-white/20 text-white text-xs rounded-lg transition-colors"
          @click="handleInit"
        >
          Retry
        </button>
      </div>
    </div>
    
    <!-- Controls Overlay -->
    <div
      class="absolute bottom-0 left-0 right-0 bg-gradient-to-t from-black/80 to-transparent transition-opacity duration-300"
      :class="showControls || !isPlaying ? 'opacity-100' : 'opacity-0'"
    >
      <PlayerControls
        :is-playing="isPlaying"
        :current-time="currentTime"
        :duration="duration"
        :volume="volume"
        :is-muted="isMuted"
        :is-fullscreen="isFullscreen"
        :qualities="qualities.map((q: any) => ({ id: q.id.toString(), height: q.height, bandwidth: q.bandwidth }))"
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

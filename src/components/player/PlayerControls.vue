<script setup lang="ts">
import { ref } from 'vue'

interface Quality {
  id: string
  height?: number
  bandwidth?: number
}

defineProps<{
  isPlaying: boolean
  currentTime: number
  duration: number
  volume: number
  isMuted: boolean
  isFullscreen: boolean
  qualities: Quality[]
  currentQuality: string
  playbackRate: number
}>()

const emit = defineEmits<{
  togglePlay: []
  seekRelative: [seconds: number]
  setVolume: [volume: number]
  toggleMute: []
  setQuality: [qualityId: string]
  setRate: [rate: number]
  toggleFullscreen: []
}>()

const showQualityMenu = ref(false)
const showRateMenu = ref(false)

const playbackRates = [0.25, 0.5, 0.75, 1, 1.25, 1.5, 1.75, 2]

function formatTime(seconds: number): string {
  const h = Math.floor(seconds / 3600)
  const m = Math.floor((seconds % 3600) / 60)
  const s = Math.floor(seconds % 60)
  if (h > 0) return `${h}:${m.toString().padStart(2, '0')}:${s.toString().padStart(2, '0')}`
  return `${m}:${s.toString().padStart(2, '0')}`
}

function getQualityLabel(q: Quality): string {
  if (q.height) {
    const label = `${q.height}p`
    if (q.height >= 2160) return `4K (${label})`
    if (q.height >= 1440) return `2K (${label})`
    return label
  }
  return `ID ${q.id}`
}
</script>

<template>
  <div class="flex flex-col w-full">
    <!-- Seekbar area slot -->
    <slot name="seekbar" />
    
    <!-- Controls row -->
    <div class="flex items-center justify-between px-3 py-2 gap-4">
      <!-- Left controls -->
      <div class="flex items-center gap-2">
        <!-- Play/Pause -->
        <button
          class="p-1.5 hover:bg-white/10 rounded-full transition-colors"
          @click="emit('togglePlay')"
        >
          <svg v-if="!isPlaying" class="size-5 text-white" fill="currentColor" viewBox="0 0 24 24">
            <path d="M8 5v14l11-7z" />
          </svg>
          <svg v-else class="size-5 text-white" fill="currentColor" viewBox="0 0 24 24">
            <path d="M6 19h4V5H6v14zm8-14v14h4V5h-4z" />
          </svg>
        </button>
        
        <!-- Skip backward -->
        <button
          class="p-1.5 hover:bg-white/10 rounded-full transition-colors"
          @click="emit('seekRelative', -10)"
        >
          <svg class="size-5 text-white" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
            <path stroke-linecap="round" stroke-linejoin="round" d="M12.066 11.2a1 1 0 000 1.6l5.334 4A1 1 0 0019 16V8a1 1 0 00-1.6-.8l-5.333 4zM4.066 11.2a1 1 0 000 1.6l5.334 4A1 1 0 0011 16V8a1 1 0 00-1.6-.8l-5.334 4z" />
          </svg>
        </button>
        
        <!-- Skip forward -->
        <button
          class="p-1.5 hover:bg-white/10 rounded-full transition-colors"
          @click="emit('seekRelative', 10)"
        >
          <svg class="size-5 text-white" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
            <path stroke-linecap="round" stroke-linejoin="round" d="M11.933 12.8a1 1 0 000-1.6L6.6 7.2A1 1 0 005 8v8a1 1 0 001.6.8l5.333-4zM19.933 12.8a1 1 0 000-1.6l-5.333-4A1 1 0 0013 8v8a1 1 0 001.6.8l5.333-4z" />
          </svg>
        </button>
        
        <!-- Volume -->
        <div class="flex items-center gap-1.5">
          <button
            class="p-1.5 hover:bg-white/10 rounded-full transition-colors"
            @click="emit('toggleMute')"
          >
            <svg v-if="isMuted || volume === 0" class="size-5 text-white" fill="currentColor" viewBox="0 0 24 24">
              <path d="M16.5 12c0-1.77-1.02-3.29-2.5-4.03v2.21l2.45 2.45c.03-.2.05-.41.05-.63zm2.5 0c0 .94-.2 1.82-.54 2.64l1.51 1.51C20.63 14.91 21 13.5 21 12c0-4.28-2.99-7.86-7-8.77v2.06c2.89.86 5 3.54 5 6.71zM4.27 3L3 4.27 7.73 9H3v6h4l5 5v-6.73l4.25 4.25c-.67.52-1.42.93-2.25 1.18v2.06c1.38-.31 2.63-.95 3.69-1.81L19.73 21 21 19.73l-9-9L4.27 3zM12 4L9.91 6.09 12 8.18V4z"/>
            </svg>
            <svg v-else-if="volume < 0.5" class="size-5 text-white" fill="currentColor" viewBox="0 0 24 24">
              <path d="M18.5 12c0-1.77-1.02-3.29-2.5-4.03v8.05c1.48-.73 2.5-2.25 2.5-4.02zM5 9v6h4l5 5V4L9 9H5z"/>
            </svg>
            <svg v-else class="size-5 text-white" fill="currentColor" viewBox="0 0 24 24">
              <path d="M3 9v6h4l5 5V4L7 9H3zm13.5 3c0-1.77-1.02-3.29-2.5-4.03v8.05c1.48-.73 2.5-2.25 2.5-4.02zM14 3.23v2.06c2.89.86 5 3.54 5 6.71s-2.11 5.85-5 6.71v2.06c4.01-.91 7-4.49 7-8.77s-2.99-7.86-7-8.77z"/>
            </svg>
          </button>
          <input
            type="range"
            min="0"
            max="1"
            step="0.01"
            :value="volume"
            class="w-16 h-1 accent-white cursor-pointer"
            @input="emit('setVolume', parseFloat(($event.target as HTMLInputElement).value))"
          />
        </div>
        
        <!-- Time display -->
        <span class="text-white/80 text-xs tabular-nums">
          {{ formatTime(currentTime) }} / {{ formatTime(duration) }}
        </span>
      </div>
      
      <!-- Right controls -->
      <div class="flex items-center gap-1">
        <!-- Playback speed -->
        <div class="relative">
          <button
            class="px-2 py-1 text-white text-xs font-medium hover:bg-white/10 rounded transition-colors"
            @click="showRateMenu = !showRateMenu"
          >
            {{ playbackRate }}x
          </button>
          <div
            v-if="showRateMenu"
            class="absolute bottom-full right-0 mb-2 bg-gray-900/95 backdrop-blur rounded-lg shadow-xl border border-white/10 py-1 min-w-[100px] z-50"
          >
            <button
              v-for="rate in playbackRates"
              :key="rate"
              class="w-full px-3 py-1.5 text-xs text-left hover:bg-white/10 transition-colors"
              :class="playbackRate === rate ? 'text-red-400 font-medium' : 'text-white'"
              @click="emit('setRate', rate); showRateMenu = false"
            >
              {{ rate }}x {{ rate === 1 ? '(Normal)' : '' }}
            </button>
          </div>
        </div>
        
        <!-- Quality selector -->
        <div class="relative">
          <button
            class="px-2 py-1 text-white text-xs font-medium hover:bg-white/10 rounded transition-colors"
            @click="showQualityMenu = !showQualityMenu"
          >
            {{ currentQuality === 'auto' ? 'Auto' : currentQuality }}
          </button>
          <div
            v-if="showQualityMenu"
            class="absolute bottom-full right-0 mb-2 bg-gray-900/95 backdrop-blur rounded-lg shadow-xl border border-white/10 py-1 min-w-[120px] z-50"
          >
            <button
              class="w-full px-3 py-1.5 text-xs text-left hover:bg-white/10 transition-colors"
              :class="currentQuality === 'auto' ? 'text-red-400 font-medium' : 'text-white'"
              @click="emit('setQuality', 'auto'); showQualityMenu = false"
            >
              Auto
            </button>
            <button
              v-for="q in qualities"
              :key="q.id"
              class="w-full px-3 py-1.5 text-xs text-left hover:bg-white/10 transition-colors"
              :class="currentQuality === q.id ? 'text-red-400 font-medium' : 'text-white'"
              @click="emit('setQuality', q.id); showQualityMenu = false"
            >
              {{ getQualityLabel(q) }}
            </button>
          </div>
        </div>
        
        <!-- PiP (Coming Soon) -->
        <div class="relative group" title="Picture in Picture — Coming Soon">
          <button
            class="p-1.5 rounded-full transition-colors opacity-40 cursor-not-allowed"
            disabled
          >
            <svg class="size-5 text-white" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
              <path stroke-linecap="round" stroke-linejoin="round" d="M4 4h16v16H4V4zm8 9h6v4h-6v-4z" />
            </svg>
          </button>
          <span class="absolute -top-7 left-1/2 -translate-x-1/2 px-1.5 py-0.5 bg-yellow-500/90 text-[9px] font-semibold text-black rounded opacity-0 group-hover:opacity-100 transition-opacity whitespace-nowrap pointer-events-none">
            Soon
          </span>
        </div>
        
        <!-- Fullscreen -->
        <button
          class="p-1.5 hover:bg-white/10 rounded-full transition-colors"
          @click="emit('toggleFullscreen')"
        >
          <svg v-if="!isFullscreen" class="size-5 text-white" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
            <path stroke-linecap="round" stroke-linejoin="round" d="M4 8V4h4M20 8V4h-4M4 16v4h4M20 16v4h-4" />
          </svg>
          <svg v-else class="size-5 text-white" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
            <path stroke-linecap="round" stroke-linejoin="round" d="M8 4v4H4M16 4v4h4M8 20v-4H4M16 20v-4h4" />
          </svg>
        </button>
      </div>
    </div>
  </div>
</template>

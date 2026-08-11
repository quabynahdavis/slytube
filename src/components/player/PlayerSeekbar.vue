<script setup lang="ts">
import { ref, computed } from 'vue'

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
  currentTime: number
  duration: number
  segments?: Segment[]
  chapters?: Chapter[]
}>()

const emit = defineEmits<{
  seek: [time: number]
}>()

const seekbarRef = ref<HTMLElement | null>(null)
const isDragging = ref(false)
const hoverPosition = ref(0)
const isHovering = ref(false)

const segmentColors: Record<string, string> = {
  sponsor: '#00d400',
  intro: '#00ffff',
  outro: '#0202ed',
  selfpromo: '#ffff00',
  interaction: '#cc00ff',
  music_offtopic: '#ff9900',
  preview: '#008fd6',
  filler: '#7300ff',
  highlight: '#ff0000',
}

function getSegmentColor(category: string): string {
  return segmentColors[category] || '#ffffff'
}

const progress = computed(() => {
  if (!props.duration) return 0
  return (props.currentTime / props.duration) * 100
})

const buffered = ref(0)

function updateBuffered() {
  // Placeholder - actual buffer tracking would need video element access
  buffered.value = Math.min(100, progress.value + 10)
}

updateBuffered()

function getTimeFromPosition(clientX: number): number {
  if (!seekbarRef.value || !props.duration) return 0
  const rect = seekbarRef.value.getBoundingClientRect()
  const ratio = Math.max(0, Math.min(1, (clientX - rect.left) / rect.width))
  return ratio * props.duration
}

function onMouseMove(e: MouseEvent) {
  if (!seekbarRef.value) return
  const rect = seekbarRef.value.getBoundingClientRect()
  hoverPosition.value = Math.max(0, Math.min(1, (e.clientX - rect.left) / rect.width))
  
  if (isDragging.value) {
    const time = getTimeFromPosition(e.clientX)
    emit('seek', time)
  }
}

function onMouseDown(e: MouseEvent) {
  isDragging.value = true
  const time = getTimeFromPosition(e.clientX)
  emit('seek', time)
  
  const onMouseUp = () => {
    isDragging.value = false
    document.removeEventListener('mouseup', onMouseUp)
  }
  document.addEventListener('mouseup', onMouseUp)
}

function formatTime(seconds: number): string {
  const h = Math.floor(seconds / 3600)
  const m = Math.floor((seconds % 3600) / 60)
  const s = Math.floor(seconds % 60)
  if (h > 0) return `${h}:${m.toString().padStart(2, '0')}:${s.toString().padStart(2, '0')}`
  return `${m}:${s.toString().padStart(2, '0')}`
}

const hoverTime = computed(() => hoverPosition.value * props.duration)
</script>

<template>
  <div
    ref="seekbarRef"
    class="relative h-1.5 w-full cursor-pointer group/seekbar"
    @mousedown="onMouseDown"
    @mousemove="onMouseMove"
    @mouseenter="isHovering = true"
    @mouseleave="isHovering = false"
  >
    <!-- Background track -->
    <div class="absolute inset-0 bg-white/20 rounded-full overflow-hidden">
      <!-- Buffered progress -->
      <div
        class="absolute h-full bg-white/30 rounded-full"
        :style="{ width: `${buffered}%` }"
      />
      <!-- Play progress -->
      <div
        class="absolute h-full bg-red-500 rounded-full"
        :style="{ width: `${progress}%` }"
      />
      
      <!-- SponsorBlock segments -->
      <div
        v-for="seg in segments"
        :key="seg.UUID"
        class="absolute h-full opacity-70"
        :style="{
          left: `${(seg.segment[0] / duration) * 100}%`,
          width: `${((seg.segment[1] - seg.segment[0]) / duration) * 100}%`,
          backgroundColor: getSegmentColor(seg.category),
        }"
      />
    </div>
    
    <!-- Chapter markers -->
    <div
      v-for="(chapter, idx) in chapters"
      :key="idx"
      class="absolute top-0 w-0.5 h-full bg-white/60 -translate-x-1/2"
      :style="{ left: `${(chapter.startSeconds / duration) * 100}%` }"
    />
    
    <!-- Hover tooltip -->
    <div
      v-if="isHovering"
      class="absolute -top-8 transform -translate-x-1/2 bg-black/80 text-white text-xs px-2 py-1 rounded pointer-events-none"
      :style="{ left: `${hoverPosition * 100}%` }"
    >
      {{ formatTime(hoverTime) }}
    </div>
    
    <!-- Thumb -->
    <div
      class="absolute top-1/2 -translate-y-1/2 -translate-x-1/2 w-3 h-3 bg-red-500 rounded-full opacity-0 group-hover/seekbar:opacity-100 transition-opacity"
      :style="{ left: `${progress}%` }"
    />
  </div>
</template>

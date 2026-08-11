<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { cn } from '@/lib/utils'
import { useHistory } from '../composables/useData'
import SkeletonGrid from '../components/ui/SkeletonGrid.vue'

const { history, loadHistory } = useHistory()

const isLoading = ref(true)
const totalWatchTime = ref(0)
const dailyAverage = ref(0)
const mostActiveDay = ref({ date: '', seconds: 0 })

interface DayStat {
  date: string
  seconds: number
}

const chartData = ref<DayStat[]>([])

onMounted(async () => {
  isLoading.value = true
  try {
    await loadHistory()
    // Generate chart data from history entries
    const dayMap = new Map<string, number>()
    for (const entry of history.value as any[]) {
      if (entry.timeWatched) {
        const date = new Date(entry.timeWatched).toISOString().split('T')[0]
        const current = dayMap.get(date) || 0
        dayMap.set(date, current + (entry.lengthSeconds || 0))
      }
    }
    // Fill in last 30 days
    const data: DayStat[] = []
    for (let i = 29; i >= 0; i--) {
      const date = new Date()
      date.setDate(date.getDate() - i)
      const dateStr = date.toISOString().split('T')[0]
      data.push({ date: dateStr, seconds: dayMap.get(dateStr) || 0 })
    }
    chartData.value = data
    totalWatchTime.value = data.reduce((sum, d) => sum + d.seconds, 0)
    dailyAverage.value = Math.floor(totalWatchTime.value / 30)
    mostActiveDay.value = data.reduce((max, d) => d.seconds > max.seconds ? d : max, data[0] || { date: '', seconds: 0 })
  } finally {
    isLoading.value = false
  }
})

function formatTime(seconds: number): string {
  const h = Math.floor(seconds / 3600)
  const m = Math.floor((seconds % 3600) / 60)
  if (h > 0) return `${h}h ${m}m`
  return `${m}m`
}

function formatDate(dateStr: string): string {
  if (!dateStr) return ''
  return new Date(dateStr).toLocaleDateString(undefined, { month: 'short', day: 'numeric' })
}

const maxSeconds = computed(() => Math.max(...chartData.value.map((d) => d.seconds), 1))

function resetStats() {
  chartData.value = chartData.value.map(d => ({ ...d, seconds: 0 }))
  totalWatchTime.value = 0
  dailyAverage.value = 0
  mostActiveDay.value = { date: '', seconds: 0 }
}
</script>

<template>
  <div class="container mx-auto max-w-5xl px-4 py-6">
    <div class="flex items-center justify-between mb-6">
      <div>
        <h1 class="text-2xl font-bold text-foreground">Watch Statistics</h1>
        <p class="text-sm text-muted-foreground mt-1">Your viewing habits and watch time</p>
      </div>
      <button class="inline-flex items-center gap-1 rounded-md border border-destructive/50 bg-destructive/10 px-3 py-2 text-sm text-destructive hover:bg-destructive/20 transition-colors" @click="resetStats">
        <svg class="size-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="3 6 5 6 21 6"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/></svg>
        Reset Stats
      </button>
    </div>

    <SkeletonGrid v-if="isLoading" :count="3" :columns="3" />

    <template v-else>
      <!-- Stats Cards -->
      <div class="grid grid-cols-1 sm:grid-cols-3 gap-4 mb-8">
        <div class="rounded-lg border border-border bg-card p-6">
          <p class="text-sm text-muted-foreground">Total Watch Time</p>
          <p class="text-2xl font-bold text-foreground mt-1">{{ formatTime(totalWatchTime) }}</p>
          <p class="text-xs text-muted-foreground mt-1">Last 30 days</p>
        </div>
        <div class="rounded-lg border border-border bg-card p-6">
          <p class="text-sm text-muted-foreground">Daily Average</p>
          <p class="text-2xl font-bold text-foreground mt-1">{{ formatTime(dailyAverage) }}</p>
          <p class="text-xs text-muted-foreground mt-1">Per day</p>
        </div>
        <div class="rounded-lg border border-border bg-card p-6">
          <p class="text-sm text-muted-foreground">Most Active Day</p>
          <p class="text-2xl font-bold text-foreground mt-1">{{ formatTime(mostActiveDay.seconds) }}</p>
          <p class="text-xs text-muted-foreground mt-1">{{ formatDate(mostActiveDay.date) }}</p>
        </div>
      </div>

      <!-- Chart -->
      <div class="rounded-lg border border-border bg-card p-6">
        <h2 class="text-lg font-semibold text-foreground mb-4">Daily Watch Time</h2>
        <div class="flex items-end gap-1 h-48">
          <div v-for="day in chartData" :key="day.date" class="flex-1 flex flex-col items-center gap-1 group">
            <div class="relative w-full">
              <div class="absolute -top-8 left-1/2 -translate-x-1/2 rounded bg-popover px-2 py-1 text-xs text-foreground shadow-lg opacity-0 group-hover:opacity-100 transition-opacity whitespace-nowrap z-10">
                {{ formatTime(day.seconds) }} on {{ formatDate(day.date) }}
              </div>
              <div class="w-full rounded-t bg-primary/80 hover:bg-primary transition-colors cursor-pointer" :style="{ height: `${(day.seconds / maxSeconds) * 100}%`, minHeight: day.seconds > 0 ? '4px' : '0' }"/>
            </div>
          </div>
        </div>
        <div class="flex justify-between mt-2 text-xs text-muted-foreground">
          <span>{{ formatDate(chartData[0]?.date || '') }}</span>
          <span>{{ formatDate(chartData[chartData.length - 1]?.date || '') }}</span>
        </div>
      </div>

      <!-- Settings -->
      <div class="rounded-lg border border-border bg-card p-6 mt-6">
        <h2 class="text-lg font-semibold text-foreground mb-4">Statistics Settings</h2>
        <div class="space-y-4">
          <div class="flex items-center justify-between">
            <div><p class="text-sm font-medium text-foreground">Enable Watch Stats</p><p class="text-xs text-muted-foreground">Track your viewing time</p></div>
            <button :class="cn('relative inline-flex h-6 w-11 items-center rounded-full transition-colors', 'bg-primary')">
              <span :class="cn('inline-block size-4 rounded-full bg-white transition-transform', 'translate-x-6')"/>
            </button>
          </div>
        </div>
      </div>
    </template>
  </div>
</template>

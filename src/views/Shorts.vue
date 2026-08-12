<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useSubscriptionsStore } from '@/stores/subscriptions'
import { getSubscribedChannelsShorts } from '@/api'
import type { Video } from '@/api/types'
import SkeletonGrid from '@/components/ui/SkeletonGrid.vue'
import EmptyState from '@/components/ui/EmptyState.vue'
import ErrorState from '@/components/ui/ErrorState.vue'
import { PhHeart, PhChatCircle, PhShare, PhCaretUp, PhCaretDown } from '@phosphor-icons/vue'

const router = useRouter()
const subscriptionsStore = useSubscriptionsStore()

const shorts = ref<Video[]>([])
const loading = ref(true)
const error = ref<string | null>(null)
const currentIndex = ref(0)

async function loadShorts() {
  loading.value = true
  error.value = null
  try {
    await subscriptionsStore.loadSubscriptions()
    const channelIds = Array.from(subscriptionsStore.subscribedChannelIds)
    if (channelIds.length === 0) {
      shorts.value = []
      loading.value = false
      return
    }
    shorts.value = await getSubscribedChannelsShorts(channelIds)
  } catch (e: any) {
    error.value = e.message || 'Failed to load shorts'
  } finally {
    loading.value = false
  }
}

function nextShort() {
  if (currentIndex.value < shorts.value.length - 1) {
    currentIndex.value++
  }
}

function prevShort() {
  if (currentIndex.value > 0) {
    currentIndex.value--
  }
}

function goToWatch(videoId: string) {
  router.push(`/watch?v=${videoId}`)
}

onMounted(loadShorts)
</script>

<template>
  <div class="h-full flex flex-col bg-background">
    <!-- Header -->
    <div class="px-6 py-4 border-b border-border">
      <h1 class="text-2xl font-bold text-foreground">Shorts</h1>
      <p class="text-sm text-muted-foreground mt-1">Short videos from channels you follow</p>
    </div>

    <div class="flex-1 flex items-center justify-center">
    <SkeletonGrid v-if="loading" :count="1" :columns="1" />

    <ErrorState v-else-if="error" :message="error" retryable @retry="loadShorts" />

    <EmptyState v-else-if="shorts.length === 0" title="No shorts from your subscriptions">
      Shorts from channels you subscribe to will show up here.
    </EmptyState>

    <div v-else class="relative w-full max-w-md mx-auto h-full flex flex-col">
      <!-- Shorts Navigation -->
      <div class="absolute top-4 left-4 z-20 flex gap-2">
        <button
          :disabled="currentIndex === 0"
          class="size-10 rounded-full bg-black/50 text-white flex items-center justify-center disabled:opacity-30 hover:bg-black/70 transition-colors"
          @click="prevShort"
        >
          <PhCaretUp :size="20" />
        </button>
        <button
          :disabled="currentIndex >= shorts.length - 1"
          class="size-10 rounded-full bg-black/50 text-white flex items-center justify-center disabled:opacity-30 hover:bg-black/70 transition-colors"
          @click="nextShort"
        >
          <PhCaretDown :size="20" />
        </button>
      </div>

      <!-- Short Video Card -->
      <div
        v-for="(short, index) in shorts"
        v-show="index === currentIndex"
        :key="short.id"
        class="relative flex-1 flex items-center justify-center"
      >
        <div class="relative w-full max-w-sm aspect-[9/16] rounded-2xl overflow-hidden bg-black cursor-pointer" @click="goToWatch(short.id)">
          <img
            :src="short.thumbnail"
            :alt="short.title"
            class="w-full h-full object-cover"
          />
          <!-- Gradient Overlay -->
          <div class="absolute inset-0 bg-gradient-to-t from-black/60 via-transparent to-transparent" />
          <!-- Video Info -->
          <div class="absolute bottom-4 left-4 right-4 text-white">
            <h3 class="text-sm font-medium line-clamp-2">{{ short.title }}</h3>
            <p class="text-xs mt-1 opacity-80">{{ short.author }}</p>
          </div>
        </div>

        <!-- Action Buttons -->
        <div class="absolute right-4 bottom-20 flex flex-col gap-4">
          <button class="flex flex-col items-center gap-1 text-white">
            <div class="size-12 rounded-full bg-black/50 flex items-center justify-center hover:bg-black/70 transition-colors">
              <PhHeart :size="24" />
            </div>
            <span class="text-xs">Like</span>
          </button>
          <button class="flex flex-col items-center gap-1 text-white">
            <div class="size-12 rounded-full bg-black/50 flex items-center justify-center hover:bg-black/70 transition-colors">
              <PhChatCircle :size="24" />
            </div>
            <span class="text-xs">Comment</span>
          </button>
          <button class="flex flex-col items-center gap-1 text-white">
            <div class="size-12 rounded-full bg-black/50 flex items-center justify-center hover:bg-black/70 transition-colors">
              <PhShare :size="24" />
            </div>
            <span class="text-xs">Share</span>
          </button>
        </div>
      </div>

      <!-- Progress Indicator -->
      <div class="absolute top-4 right-4 text-white text-sm bg-black/50 px-2 py-1 rounded">
        {{ currentIndex + 1 }} / {{ shorts.length }}
      </div>
    </div>
    </div>
  </div>
</template>

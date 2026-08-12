<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useSubscriptionsStore } from '@/stores/subscriptions'
import { getSubscribedChannelsPosts } from '@/api'
import SkeletonGrid from '@/components/ui/SkeletonGrid.vue'
import EmptyState from '@/components/ui/EmptyState.vue'
import ErrorState from '@/components/ui/ErrorState.vue'
import { PhHeart, PhChatCircle, PhShare } from '@phosphor-icons/vue'

const subscriptionsStore = useSubscriptionsStore()

const posts = ref<any[]>([])
const loading = ref(true)
const error = ref<string | null>(null)

async function loadPosts() {
  loading.value = true
  error.value = null
  try {
    await subscriptionsStore.loadSubscriptions()
    const channelIds = Array.from(subscriptionsStore.subscribedChannelIds)
    if (channelIds.length === 0) {
      posts.value = []
      loading.value = false
      return
    }
    posts.value = await getSubscribedChannelsPosts(channelIds)
  } catch (e: any) {
    error.value = e.message || 'Failed to load posts'
  } finally {
    loading.value = false
  }
}

function formatDate(dateInput: string | number): string {
  if (!dateInput) return ''
  const timestamp = typeof dateInput === 'number' ? dateInput * 1000 : new Date(dateInput).getTime()
  if (isNaN(timestamp)) return ''
  const now = Date.now()
  const diff = now - timestamp
  const mins = Math.floor(diff / 60000)
  if (mins < 60) return `${mins}m ago`
  const hours = Math.floor(mins / 60)
  if (hours < 24) return `${hours}h ago`
  const days = Math.floor(hours / 24)
  if (days < 7) return `${days}d ago`
  return new Date(timestamp).toLocaleDateString(undefined, { year: 'numeric', month: 'short', day: 'numeric' })
}

onMounted(loadPosts)
</script>

<template>
  <div class="max-w-3xl mx-auto px-4 py-6">
    <div class="mb-6">
      <h1 class="text-2xl font-bold text-foreground">Posts</h1>
      <p class="text-sm text-muted-foreground mt-1">Community updates from channels you follow</p>
    </div>

    <SkeletonGrid v-if="loading" :count="3" :columns="1" />

    <ErrorState v-else-if="error" :message="error" retryable @retry="loadPosts" />

    <EmptyState v-else-if="posts.length === 0" title="No community posts">
      Community updates from your subscribed channels will appear here.
    </EmptyState>

    <div v-else class="space-y-4">
      <article
        v-for="post in posts"
        :key="post.commentId || post.id"
        class="rounded-xl border border-border bg-card p-4"
      >
        <!-- Post Header -->
        <div class="flex items-center gap-3 mb-3">
          <div class="size-10 rounded-full bg-muted flex items-center justify-center shrink-0 overflow-hidden">
            <img
              v-if="post.authorThumbnails?.[0]?.url"
              :src="post.authorThumbnails[0].url"
              :alt="post.author"
              class="w-full h-full object-cover"
            />
            <span v-else class="text-sm font-medium text-muted-foreground">{{ post.author?.[0] || '?' }}</span>
          </div>
          <div>
            <span class="text-sm font-medium text-foreground">
              {{ post.author || 'Unknown' }}
            </span>
            <p class="text-xs text-muted-foreground">{{ formatDate((post.published || post.publishedText) as string) }}</p>
          </div>
        </div>

        <!-- Post Content -->
        <div class="mb-3">
          <p class="text-sm text-foreground whitespace-pre-wrap leading-relaxed">
            {{ post.content || post.commentText || post.text || '' }}
          </p>
        </div>

        <!-- Post Image (if any) -->
        <div v-if="post.attachment || post.image" class="mb-3 rounded-lg overflow-hidden">
          <img
            :src="post.attachment || post.image"
            alt="Post attachment"
            class="w-full max-h-96 object-cover"
          />
        </div>

        <!-- Post Actions -->
        <div class="flex items-center gap-4 pt-3 border-t border-border">
          <button class="flex items-center gap-1.5 text-sm text-muted-foreground hover:text-foreground transition-colors">
            <PhHeart :size="18" />
            <span>{{ post.likeCount || 0 }}</span>
          </button>
          <button class="flex items-center gap-1.5 text-sm text-muted-foreground hover:text-foreground transition-colors">
            <PhChatCircle :size="18" />
            <span>{{ post.replyCount || post.commentCount || 0 }}</span>
          </button>
          <button class="flex items-center gap-1.5 text-sm text-muted-foreground hover:text-foreground transition-colors">
            <PhShare :size="18" />
          </button>
        </div>
      </article>
    </div>
  </div>
</template>

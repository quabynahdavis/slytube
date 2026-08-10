<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRoute } from 'vue-router'
import { cn } from '@/lib/utils'

const route = useRoute()

const isLoading = ref(true)
const postId = computed(() => route.params.id as string || '')

const post = ref({
  author: '',
  authorId: '',
  content: '',
  published: '',
  likeCount: 0,
  commentCount: 0,
  authorThumbnails: [] as Array<{ url: string; width: number; height: number }>,
})

const comments = ref<Array<{
  id: string; author: string; content: string; published: string; likeCount: number
}>>([])

onMounted(async () => {
  isLoading.value = true
  try {
    await new Promise((r) => setTimeout(r, 500))
    post.value = {
      author: 'Sample Channel', authorId: 'UC-sample',
      content: 'This is a community post from a channel. It can contain text, images, polls, and other types of content that creators share with their audience.',
      published: new Date(Date.now() - 86400000).toISOString(),
      likeCount: 1234, commentCount: 56,
      authorThumbnails: [{ url: '', width: 80, height: 80 }],
    }
    comments.value = Array.from({ length: 8 }, (_, i) => ({
      id: `comment-${i}`, author: `User ${i + 1}`,
      content: `This is comment ${i + 1} on the community post.`,
      published: new Date(Date.now() - Math.random() * 86400000).toISOString(),
      likeCount: Math.floor(Math.random() * 100),
    }))
  } finally {
    isLoading.value = false
  }
})

function formatDate(dateStr: string): string {
  return new Date(dateStr).toLocaleDateString(undefined, { year: 'numeric', month: 'short', day: 'numeric' })
}
</script>

<template>
  <div class="container mx-auto max-w-3xl px-4 py-6">
    <div v-if="isLoading" class="animate-pulse space-y-4">
      <div class="flex items-center gap-3"><div class="size-10 rounded-full bg-muted"/><div class="space-y-2"><div class="h-4 w-32 rounded bg-muted"/><div class="h-3 w-24 rounded bg-muted"/></div></div>
      <div class="space-y-2"><div class="h-4 w-full rounded bg-muted"/><div class="h-4 w-3/4 rounded bg-muted"/></div>
    </div>

    <template v-else>
      <!-- Post Header -->
      <div class="flex items-center gap-3 mb-4">
        <router-link :to="`/channel/${post.authorId}`" class="size-10 rounded-full bg-muted flex items-center justify-center shrink-0">
          <svg class="size-6 text-muted-foreground" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"/><circle cx="12" cy="7" r="4"/></svg>
        </router-link>
        <div>
          <router-link :to="`/channel/${post.authorId}`" class="text-sm font-medium text-foreground hover:text-primary">{{ post.author }}</router-link>
          <p class="text-xs text-muted-foreground">{{ formatDate(post.published) }}</p>
        </div>
      </div>

      <!-- Post Content -->
      <div class="rounded-lg border border-border bg-card p-6 mb-6">
        <p class="text-sm text-foreground whitespace-pre-wrap leading-relaxed">{{ post.content }}</p>
        <div class="flex items-center gap-4 mt-4 pt-4 border-t border-border">
          <button class="flex items-center gap-1 text-sm text-muted-foreground hover:text-foreground transition-colors">
            <svg class="size-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M14 9V5a3 3 0 0 0-3-3l-4 9v11h11.28a2 2 0 0 0 2-1.7l1.38-9a2 2 0 0 0-2-2.3zM7 22H4a2 2 0 0 1-2-2v-7a2 2 0 0 1 2-2h3"/></svg>
            {{ post.likeCount.toLocaleString() }}
          </button>
          <span class="flex items-center gap-1 text-sm text-muted-foreground">
            <svg class="size-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/></svg>
            {{ post.commentCount }} comments
          </span>
        </div>
      </div>

      <!-- Comments -->
      <div>
        <h2 class="text-lg font-semibold text-foreground mb-4">Comments ({{ comments.length }})</h2>
        <div class="space-y-4">
          <div v-for="comment in comments" :key="comment.id" class="flex gap-3">
            <div class="size-8 rounded-full bg-muted flex items-center justify-center shrink-0">
              <svg class="size-4 text-muted-foreground" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"/><circle cx="12" cy="7" r="4"/></svg>
            </div>
            <div class="flex-1 min-w-0">
              <div class="flex items-center gap-2">
                <span class="text-sm font-medium text-foreground">{{ comment.author }}</span>
                <span class="text-xs text-muted-foreground">{{ formatDate(comment.published) }}</span>
              </div>
              <p class="text-sm text-muted-foreground mt-1">{{ comment.content }}</p>
              <div class="flex items-center gap-2 mt-1">
                <button class="flex items-center gap-1 text-xs text-muted-foreground hover:text-foreground transition-colors">
                  <svg class="size-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M14 9V5a3 3 0 0 0-3-3l-4 9v11h11.28a2 2 0 0 0 2-1.7l1.38-9a2 2 0 0 0-2-2.3zM7 22H4a2 2 0 0 1-2-2v-7a2 2 0 0 1 2-2h3"/></svg>
                  {{ comment.likeCount }}
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>
    </template>
  </div>
</template>

import { defineStore } from 'pinia'

export interface SubscriptionCacheEntry {
  videos: unknown[] | null
  timestamp: Date
}

export interface SubscriptionCacheState {
  videoCache: Record<string, SubscriptionCacheEntry>
  liveCache: Record<string, SubscriptionCacheEntry>
  shortsCache: Record<string, SubscriptionCacheEntry>
  postsCache: Record<string, { posts: unknown[] | null; timestamp: Date }>
  subscriptionCacheReady: boolean
  subscriptionFeedRefreshInProgress: boolean
  subscriptionFeedRefreshTab: string | null
  subscriptionFeedRefreshProgress: number
  subscriptionFeedLastRefreshTimestamp: Date | null
  subscriptionFeedNextAutoRefreshTimestamp: Date | null
  subscriptionShortsLastRefreshTimestamp: Date | null
  subscriptionShortsNextAutoRefreshTimestamp: Date | null
  subscriptionLiveLastRefreshTimestamp: Date | null
  subscriptionLiveNextAutoRefreshTimestamp: Date | null
  subscriptionPostsLastRefreshTimestamp: Date | null
  subscriptionPostsNextAutoRefreshTimestamp: Date | null
  subscribedChannelIds: Set<string>
  pendingSubscriptions: Set<string>
}

export const useSubscriptionsStore = defineStore('subscriptions', {
  state: (): SubscriptionCacheState => ({
    videoCache: {},
    liveCache: {},
    shortsCache: {},
    postsCache: {},
    subscriptionCacheReady: false,
    subscriptionFeedRefreshInProgress: false,
    subscriptionFeedRefreshTab: null,
    subscriptionFeedRefreshProgress: 0,
    subscriptionFeedLastRefreshTimestamp: null,
    subscriptionFeedNextAutoRefreshTimestamp: null,
    subscriptionShortsLastRefreshTimestamp: null,
    subscriptionShortsNextAutoRefreshTimestamp: null,
    subscriptionLiveLastRefreshTimestamp: null,
    subscriptionLiveNextAutoRefreshTimestamp: null,
    subscriptionPostsLastRefreshTimestamp: null,
    subscriptionPostsNextAutoRefreshTimestamp: null,
    subscribedChannelIds: new Set(),
    pendingSubscriptions: new Set(),
  }),

  getters: {
    getSubscriptionCacheReady: (state) => state.subscriptionCacheReady,
    getSubscriptionFeedRefreshInProgress: (state) => state.subscriptionFeedRefreshInProgress,
    getSubscriptionFeedRefreshTab: (state) => state.subscriptionFeedRefreshTab,
    getSubscriptionFeedRefreshProgress: (state) => state.subscriptionFeedRefreshProgress,
    getSubscriptionFeedLastRefreshTimestamp: (state) => state.subscriptionFeedLastRefreshTimestamp,
    getSubscriptionFeedNextAutoRefreshTimestamp: (state) => state.subscriptionFeedNextAutoRefreshTimestamp,
    getSubscriptionShortsLastRefreshTimestamp: (state) => state.subscriptionShortsLastRefreshTimestamp,
    getSubscriptionShortsNextAutoRefreshTimestamp: (state) => state.subscriptionShortsNextAutoRefreshTimestamp,
    getSubscriptionLiveLastRefreshTimestamp: (state) => state.subscriptionLiveLastRefreshTimestamp,
    getSubscriptionLiveNextAutoRefreshTimestamp: (state) => state.subscriptionLiveNextAutoRefreshTimestamp,
    getSubscriptionPostsLastRefreshTimestamp: (state) => state.subscriptionPostsLastRefreshTimestamp,
    getSubscriptionPostsNextAutoRefreshTimestamp: (state) => state.subscriptionPostsNextAutoRefreshTimestamp,
    getVideoCache: (state) => state.videoCache,
    getShortsCache: (state) => state.shortsCache,
    getLiveCache: (state) => state.liveCache,
    getPostsCache: (state) => state.postsCache,
    isSubscribed: (state) => (channelId: string) => state.subscribedChannelIds.has(channelId),
    isPending: (state) => (channelId: string) => state.pendingSubscriptions.has(channelId),
  },

  actions: {
    loadSubscriptions() {
      // Placeholder for DB integration
    },

    updateVideoCacheByChannel(channelId: string, entries: unknown[], timestamp: Date = new Date()) {
      const existingObject = this.videoCache[channelId]
      const newObject = existingObject ?? { videos: null, timestamp: new Date() }
      if (entries != null) newObject.videos = entries
      newObject.timestamp = timestamp
      this.videoCache[channelId] = newObject
    },

    updateShortsCacheByChannel(channelId: string, entries: unknown[], timestamp: Date = new Date()) {
      const existingObject = this.shortsCache[channelId]
      const newObject = existingObject ?? { videos: null, timestamp: new Date() }
      if (entries != null) newObject.videos = entries
      newObject.timestamp = timestamp
      this.shortsCache[channelId] = newObject
    },

    updateLiveCacheByChannel(channelId: string, entries: unknown[], timestamp: Date = new Date()) {
      const existingObject = this.liveCache[channelId]
      const newObject = existingObject ?? { videos: null, timestamp: new Date() }
      if (entries != null) newObject.videos = entries
      newObject.timestamp = timestamp
      this.liveCache[channelId] = newObject
    },

    updatePostsCacheByChannel(channelId: string, entries: unknown[], timestamp: Date = new Date()) {
      const existingObject = this.postsCache[channelId]
      const newObject = existingObject ?? { posts: null, timestamp: new Date() }
      if (entries != null) newObject.posts = entries
      newObject.timestamp = timestamp
      this.postsCache[channelId] = newObject
    },

    clearCaches() {
      this.videoCache = {}
      this.shortsCache = {}
      this.liveCache = {}
      this.postsCache = {}
    },

    clearCachesForManyChannels(channelIds: string[]) {
      for (const channelId of channelIds) {
        this.videoCache[channelId] = { videos: null, timestamp: new Date() }
        this.liveCache[channelId] = { videos: null, timestamp: new Date() }
        this.shortsCache[channelId] = { videos: null, timestamp: new Date() }
        this.postsCache[channelId] = { posts: null, timestamp: new Date() }
      }
    },

    setCaches(videos: Record<string, SubscriptionCacheEntry>, liveStreams: Record<string, SubscriptionCacheEntry>, shorts: Record<string, SubscriptionCacheEntry>, communityPosts: Record<string, { posts: unknown[] | null; timestamp: Date }>) {
      this.videoCache = videos
      this.liveCache = liveStreams
      this.shortsCache = shorts
      this.postsCache = communityPosts
    },

    setSubscriptionCacheReady(ready: boolean) {
      this.subscriptionCacheReady = ready
    },

    setSubscriptionFeedRefreshInProgress(inProgress: boolean) {
      this.subscriptionFeedRefreshInProgress = inProgress
    },

    setSubscribed(channelId: string, subscribed: boolean) {
      if (subscribed) {
        this.subscribedChannelIds.add(channelId)
      } else {
        this.subscribedChannelIds.delete(channelId)
      }
    },

    setPending(channelId: string, pending: boolean) {
      if (pending) {
        this.pendingSubscriptions.add(channelId)
      } else {
        this.pendingSubscriptions.delete(channelId)
      }
    },

    /**
     * Optimistically toggle subscription status.
     * Immediately updates UI state, then calls the API.
     * Rolls back on failure.
     */
    async toggleSubscription(channelId: string, channelName: string): Promise<{ success: boolean; subscribed: boolean }> {
      const wasSubscribed = this.subscribedChannelIds.has(channelId)
      const newState = !wasSubscribed

      // Optimistic update: toggle UI immediately
      this.setSubscribed(channelId, newState)
      this.setPending(channelId, true)

      try {
        // Simulate API call - replace with real API when backend is ready
        await this.performSubscriptionApiCall(channelId, channelName, newState)
        return { success: true, subscribed: newState }
      } catch (error) {
        // Rollback on failure
        this.setSubscribed(channelId, wasSubscribed)
        console.error('Subscription toggle failed:', error)
        return { success: false, subscribed: wasSubscribed }
      } finally {
        this.setPending(channelId, false)
      }
    },

    /**
     * Placeholder for the actual subscription API call.
     * Replace with real implementation (e.g., invoke('subscribe_channel') or Invidious API).
     */
    async performSubscriptionApiCall(_channelId: string, _channelName: string, _subscribe: boolean): Promise<void> {
      // Simulate network delay
      await new Promise((resolve) => setTimeout(resolve, 300))
      // TODO: Replace with real API call:
      // await invoke('subscribe_channel', { channelId, channelName, subscribe })
    },
  },
})

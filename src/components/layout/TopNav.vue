<script setup lang="ts">
import { ref, watch, computed } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { useSettingsStore } from '@/stores/settings'
import { useSyncStore } from '@/stores/sync'
import { useSearchHistoryStore } from '@/stores/search-history'
import { getSearchSuggestions } from '@/composables/useInnertube'

const { t } = useI18n()
const router = useRouter()
const settingsStore = useSettingsStore()
const syncStore = useSyncStore()
const searchHistoryStore = useSearchHistoryStore()

const searchQuery = ref('')
const isSearchFocused = ref(false)
const suggestions = ref<string[]>([])
const isLoadingSuggestions = ref(false)

let debounceTimer: ReturnType<typeof setTimeout> | null = null

function handleSearch() {
  if (!searchQuery.value.trim()) return
  searchHistoryStore.addSearchHistoryEntry({
    _id: searchQuery.value.trim(),
    timeWatched: Date.now(),
  })
  router.push({ path: '/search', query: { q: searchQuery.value.trim() } })
}

function handleSearchFocus() {
  isSearchFocused.value = true
  if (searchQuery.value.trim()) {
    fetchSuggestions(searchQuery.value.trim())
  }
}

function handleSearchBlur() {
  setTimeout(() => {
    isSearchFocused.value = false
  }, 200)
}

function selectSearchSuggestion(query: string) {
  searchQuery.value = query
  handleSearch()
}

function fetchSuggestions(query: string) {
  if (debounceTimer) clearTimeout(debounceTimer)
  if (!query.trim()) {
    suggestions.value = []
    return
  }
  isLoadingSuggestions.value = true
  debounceTimer = setTimeout(async () => {
    try {
      const results = await getSearchSuggestions(query)
      suggestions.value = results.slice(0, 8)
    } catch {
      suggestions.value = []
    } finally {
      isLoadingSuggestions.value = false
    }
  }, 200)
}

watch(searchQuery, (newVal) => {
  if (isSearchFocused.value) {
    fetchSuggestions(newVal)
  }
})

const recentSearches = () => searchHistoryStore.getLatestSearchHistoryNames.slice(0, 5)

const showDropdown = () => isSearchFocused.value && (suggestions.value.length > 0 || recentSearches().length > 0 || isLoadingSuggestions.value)

const syncStatusTooltip = computed(() => {
  if (!settingsStore.syncServerEnabled) return ''
  const status = syncStore.syncServerStatus
  if (status === 'syncing') return 'Syncing...'
  if (status === 'error') return syncStore.syncServerError || 'Sync error'
  if (status === 'success') {
    const lastSync = settingsStore.syncServerLastSyncAt
    if (lastSync) {
      const date = new Date(lastSync)
      return `Last synced: ${date.toLocaleString()}`
    }
    return 'Synced'
  }
  return 'Sync idle'
})

function toggleSidebar() {
  settingsStore.updateSetting('expandSideBar', !settingsStore.expandSideBar)
}

defineExpose({
  focusSearch: () => {
    const input = document.querySelector('input[type="search"]') as HTMLInputElement | null
    input?.focus()
  }
})
</script>

<template>
  <header class="sticky top-0 z-40 flex h-14 items-center gap-4 border-b border-border bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60 px-4">
    <!-- Hamburger + Logo -->
    <div class="flex items-center gap-2">
      <button
        class="inline-flex items-center justify-center size-9 rounded-full text-muted-foreground hover:bg-accent hover:text-accent-foreground transition-colors"
        @click="toggleSidebar"
        :title="settingsStore.expandSideBar ? 'Collapse sidebar' : 'Expand sidebar'"
      >
        <svg class="size-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <line x1="3" y1="12" x2="21" y2="12" />
          <line x1="3" y1="6" x2="21" y2="6" />
          <line x1="3" y1="18" x2="21" y2="18" />
        </svg>
      </button>
      <router-link to="/" class="flex items-center gap-2 font-semibold text-foreground">
        <svg class="size-6 text-primary" viewBox="0 0 24 24" fill="currentColor">
          <path d="M23.498 6.186a3.016 3.016 0 0 0-2.122-2.136C19.505 3.545 12 3.545 12 3.545s-7.505 0-9.377.505A3.017 3.017 0 0 0 .502 6.186C0 8.07 0 12 0 12s0 3.93.502 5.814a3.016 3.016 0 0 0 2.122 2.136c1.871.505 9.376.505 9.376.505s7.505 0 9.377-.505a3.015 3.015 0 0 0 2.122-2.136C24 15.93 24 12 24 12s0-3.93-.502-5.814zM9.545 15.568V8.432L15.818 12l-6.273 3.568z" />
        </svg>
        <span v-if="!settingsStore.hideHeaderLogo" class="hidden sm:inline">Slytube</span>
      </router-link>
    </div>

    <!-- Search Bar -->
    <div class="flex flex-1 items-center justify-center">
      <div class="relative w-full max-w-xl">
        <form @submit.prevent="handleSearch" class="flex">
          <div class="relative flex-1">
            <input
              v-model="searchQuery"
              type="search"
              :placeholder="t('search.placeholder')"
              class="h-10 w-full rounded-l-full border border-input bg-background px-4 pr-10 text-sm outline-none focus:border-primary focus:ring-1 focus:ring-primary"
              @focus="handleSearchFocus"
              @blur="handleSearchBlur"
            />
            <!-- Search Suggestions Dropdown -->
            <div
              v-if="showDropdown()"
              class="absolute top-full left-0 right-0 mt-1 rounded-md border border-border bg-popover shadow-lg z-50"
            >
              <ul class="py-1">
                <!-- Loading indicator -->
                <li v-if="isLoadingSuggestions" class="flex items-center gap-2 px-4 py-2 text-sm text-muted-foreground">
                  <svg class="size-4 animate-spin" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <path d="M21 12a9 9 0 11-6.219-8.56" />
                  </svg>
                  <span>{{ t('search.loadingSuggestions') }}</span>
                </li>
                <!-- Live search suggestions -->
                <template v-else-if="suggestions.length > 0">
                  <li
                    v-for="suggestion in suggestions"
                    :key="suggestion"
                    class="flex items-center gap-2 px-4 py-2 text-sm cursor-pointer hover:bg-accent"
                    @mousedown="selectSearchSuggestion(suggestion)"
                  >
                    <svg class="size-4 text-muted-foreground" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                      <circle cx="11" cy="11" r="8" />
                      <line x1="21" y1="21" x2="16.65" y2="16.65" />
                    </svg>
                    <span>{{ suggestion }}</span>
                  </li>
                </template>
                <!-- Recent searches fallback -->
                <template v-else>
                  <li
                    v-for="suggestion in recentSearches()"
                    :key="suggestion"
                    class="flex items-center gap-2 px-4 py-2 text-sm cursor-pointer hover:bg-accent"
                    @mousedown="selectSearchSuggestion(suggestion)"
                  >
                    <svg class="size-4 text-muted-foreground" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                      <circle cx="12" cy="12" r="10" />
                      <polyline points="12 6 12 12 16 14" />
                    </svg>
                    <span>{{ suggestion }}</span>
                  </li>
                </template>
              </ul>
            </div>
          </div>
          <button
            type="submit"
            class="h-10 px-5 rounded-r-full border border-l-0 border-input bg-muted text-muted-foreground hover:bg-accent hover:text-accent-foreground transition-colors"
          >
            <svg class="size-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <circle cx="11" cy="11" r="8" />
              <line x1="21" y1="21" x2="16.65" y2="16.65" />
            </svg>
          </button>
        </form>
      </div>
    </div>

    <!-- Right Actions -->
    <div class="flex items-center gap-2">
      <!-- Sync Status Indicator -->
      <div
        v-if="settingsStore.syncServerEnabled"
        class="inline-flex items-center justify-center size-9 rounded-full text-muted-foreground relative"
        :title="syncStatusTooltip"
      >
        <svg v-if="syncStore.syncServerStatus === 'success'" class="size-5 text-green-500" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M22 11.08V12a10 10 0 11-5.93-9.14" />
          <polyline points="22 4 12 14.01 9 11.01" />
        </svg>
        <svg v-else-if="syncStore.syncServerStatus === 'error'" class="size-5 text-red-500" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <circle cx="12" cy="12" r="10" />
          <line x1="15" y1="9" x2="9" y2="15" />
          <line x1="9" y1="9" x2="15" y2="15" />
        </svg>
        <svg v-else-if="syncStore.syncServerStatus === 'syncing'" class="size-5 text-yellow-500 animate-pulse" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <polyline points="23 4 23 10 17 10" />
          <path d="M20.49 15a9 9 0 11-2.12-9.36L23 10" />
        </svg>
        <svg v-else class="size-5 text-muted-foreground/60" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M22 11.08V12a10 10 0 11-5.93-9.14" />
          <polyline points="22 4 12 14.01 9 11.01" />
        </svg>
      </div>
      <button
        class="inline-flex items-center justify-center size-9 rounded-full text-muted-foreground hover:bg-accent hover:text-accent-foreground transition-colors"
        :title="t('nav.downloads')"
        @click="router.push('/downloads')"
      >
        <svg class="size-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4" />
          <polyline points="7 10 12 15 17 10" />
          <line x1="12" y1="15" x2="12" y2="3" />
        </svg>
      </button>
      <button
        class="inline-flex items-center justify-center size-9 rounded-full text-muted-foreground hover:bg-accent hover:text-accent-foreground transition-colors"
        :title="t('nav.settings')"
        @click="router.push('/settings')"
      >
        <svg class="size-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <circle cx="12" cy="12" r="3" />
          <path d="M19.4 15a1.65 1.65 0 00.33 1.82l.06.06a2 2 0 010 2.83 2 2 0 01-2.83 0l-.06-.06a1.65 1.65 0 00-1.82-.33 1.65 1.65 0 00-1 1.51V21a2 2 0 01-2 2 2 2 0 01-2-2v-.09A1.65 1.65 0 009 19.4a1.65 1.65 0 00-1.82.33l-.06.06a2 2 0 01-2.83 0 2 2 0 010-2.83l.06-.06A1.65 1.65 0 004.68 15a1.65 1.65 0 00-1.51-1H3a2 2 0 01-2-2 2 2 0 012-2h.09A1.65 1.65 0 004.6 9a1.65 1.65 0 00-.33-1.82l-.06-.06a2 2 0 010-2.83 2 2 0 012.83 0l.06.06A1.65 1.65 0 009 4.68a1.65 1.65 0 001-1.51V3a2 2 0 012-2 2 2 0 012 2v.09a1.65 1.65 0 001 1.51 1.65 1.65 0 001.82-.33l.06-.06a2 2 0 012.83 0 2 2 0 010 2.83l-.06.06A1.65 1.65 0 0019.4 9a1.65 1.65 0 001.51 1H21a2 2 0 012 2 2 2 0 01-2 2h-.09a1.65 1.65 0 00-1.51 1z" />
        </svg>
      </button>
    </div>
  </header>
</template>

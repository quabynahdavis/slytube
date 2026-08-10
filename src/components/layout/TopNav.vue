<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { useSettingsStore } from '@/stores/settings'
import { useSearchHistoryStore } from '@/stores/search-history'

const router = useRouter()
const settingsStore = useSettingsStore()
const searchHistoryStore = useSearchHistoryStore()

const searchQuery = ref('')
const isSearchFocused = ref(false)

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

const recentSearches = () => searchHistoryStore.getLatestSearchHistoryNames.slice(0, 5)
</script>

<template>
  <header class="sticky top-0 z-40 flex h-14 items-center gap-4 border-b border-border bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60 px-4">
    <!-- Logo / Brand -->
    <div class="flex items-center gap-2">
      <router-link to="/" class="flex items-center gap-2 font-semibold text-foreground">
        <svg class="size-6 text-primary" viewBox="0 0 24 24" fill="currentColor">
          <path d="M23.498 6.186a3.016 3.016 0 0 0-2.122-2.136C19.505 3.545 12 3.545 12 3.545s-7.505 0-9.377.505A3.017 3.017 0 0 0 .502 6.186C0 8.07 0 12 0 12s0 3.93.502 5.814a3.016 3.016 0 0 0 2.122 2.136c1.871.505 9.376.505 9.376.505s7.505 0 9.377-.505a3.015 3.015 0 0 0 2.122-2.136C24 15.93 24 12 24 12s0-3.93-.502-5.814zM9.545 15.568V8.432L15.818 12l-6.273 3.568z" />
        </svg>
        <span v-if="!settingsStore.hideHeaderLogo" class="hidden sm:inline">SlyTube</span>
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
              placeholder="Search"
              class="h-10 w-full rounded-l-full border border-input bg-background px-4 pr-10 text-sm outline-none focus:border-primary focus:ring-1 focus:ring-primary"
              @focus="handleSearchFocus"
              @blur="handleSearchBlur"
            />
            <!-- Search Suggestions Dropdown -->
            <div
              v-if="isSearchFocused && recentSearches().length > 0"
              class="absolute top-full left-0 right-0 mt-1 rounded-md border border-border bg-popover shadow-lg z-50"
            >
              <ul class="py-1">
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
      <button
        class="inline-flex items-center justify-center size-9 rounded-full text-muted-foreground hover:bg-accent hover:text-accent-foreground transition-colors"
        title="Downloads"
        @click="router.push('/downloads')"
      >
        <svg class="size-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
          <polyline points="7 10 12 15 17 10" />
          <line x1="12" y1="15" x2="12" y2="3" />
        </svg>
      </button>
      <button
        class="inline-flex items-center justify-center size-9 rounded-full text-muted-foreground hover:bg-accent hover:text-accent-foreground transition-colors"
        title="Settings"
        @click="router.push('/settings')"
      >
        <svg class="size-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <circle cx="12" cy="12" r="3" />
          <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z" />
        </svg>
      </button>
    </div>
  </header>
</template>

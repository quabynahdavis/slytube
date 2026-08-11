<script setup lang="ts">
import { ref, watch, computed } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { useSettingsStore } from '@/stores/settings'
import { useSyncStore } from '@/stores/sync'
import { useSearchHistoryStore } from '@/stores/search-history'
import { getSearchSuggestions } from '@/composables/useInnertube'
import {
  PhList as List,
  PhDownloadSimple as DownloadSimple,
  PhGear as Gear,
  PhMagnifyingGlass as MagnifyingGlass,
  PhClockCounterClockwise as ClockCounterClockwise,
  PhSpinner as Spinner,
  PhCloud as Cloud,
  PhCloudCheck as CloudCheck,
  PhCloudX as CloudX,
  PhCloudSlash as CloudSlash,
} from '@phosphor-icons/vue'

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
        <List :size="20" />
      </button>
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
                  <Spinner :size="16" class="animate-spin" />
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
                    <MagnifyingGlass :size="16" class="text-muted-foreground" />
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
                    <ClockCounterClockwise :size="16" class="text-muted-foreground" />
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
            <MagnifyingGlass :size="20" />
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
        <CloudCheck v-if="syncStore.syncServerStatus === 'success'" :size="20" class="text-green-500" />
        <CloudX v-else-if="syncStore.syncServerStatus === 'error'" :size="20" class="text-red-500" />
        <Cloud v-else-if="syncStore.syncServerStatus === 'syncing'" :size="20" class="text-yellow-500 animate-pulse" />
        <Cloud v-else :size="20" class="text-muted-foreground/60" />
        <!-- Status dot -->
        <span
          class="absolute top-1 right-1 size-2 rounded-full"
          :class="{
            'bg-green-500': syncStore.syncServerStatus === 'success',
            'bg-red-500': syncStore.syncServerStatus === 'error',
            'bg-yellow-500 animate-ping': syncStore.syncServerStatus === 'syncing',
            'bg-muted-foreground/40': syncStore.syncServerStatus === 'idle',
          }"
        />
      </div>
      <button
        class="inline-flex items-center justify-center size-9 rounded-full text-muted-foreground hover:bg-accent hover:text-accent-foreground transition-colors"
        :title="t('nav.downloads')"
        @click="router.push('/downloads')"
      >
        <DownloadSimple :size="20" />
      </button>
      <button
        class="inline-flex items-center justify-center size-9 rounded-full text-muted-foreground hover:bg-accent hover:text-accent-foreground transition-colors"
        :title="t('nav.settings')"
        @click="router.push('/settings')"
      >
        <Gear :size="20" />
      </button>
    </div>
  </header>
</template>

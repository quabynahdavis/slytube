<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { useSettingsSearch } from '@/composables/useSettingsSearch'

const { t } = useI18n()
const router = useRouter()
const { searchQuery, searchResults, setSearchQuery } = useSettingsSearch()

const inputRef = ref<HTMLInputElement | null>(null)
const showDropdown = ref(false)
const selectedIndex = ref(-1)

function handleInput(e: Event) {
  const value = (e.target as HTMLInputElement).value
  setSearchQuery(value)
  showDropdown.value = value.length >= 2
  selectedIndex.value = -1
}

function handleFocus() {
  if (searchQuery.value.length >= 2) {
    showDropdown.value = true
  }
}

function handleBlur() {
  setTimeout(() => {
    showDropdown.value = false
  }, 200)
}

async function navigateToResult(result: typeof searchResults.value[0]) {
  showDropdown.value = false
  setSearchQuery('')

  if (result.category.route === '/settings') {
    const el = document.getElementById(`setting-${result.item.key}`)
    if (el) {
      el.scrollIntoView({ behavior: 'smooth', block: 'center' })
      el.classList.add('ring-2', 'ring-primary', 'ring-offset-2')
      setTimeout(() => el.classList.remove('ring-2', 'ring-primary', 'ring-offset-2'), 2000)
    }
    return
  }

  await router.push(`${result.category.route}#setting-${result.item.key}`)

  await new Promise(resolve => setTimeout(resolve, 100))
  const el = document.getElementById(`setting-${result.item.key}`)
  if (el) {
    el.scrollIntoView({ behavior: 'smooth', block: 'center' })
    el.classList.add('ring-2', 'ring-primary', 'ring-offset-2')
    setTimeout(() => el.classList.remove('ring-2', 'ring-primary', 'ring-offset-2'), 2000)
  }
}

function handleKeydown(e: KeyboardEvent) {
  if (!showDropdown.value || searchResults.value.length === 0) return

  if (e.key === 'ArrowDown') {
    e.preventDefault()
    selectedIndex.value = Math.min(selectedIndex.value + 1, searchResults.value.length - 1)
  } else if (e.key === 'ArrowUp') {
    e.preventDefault()
    selectedIndex.value = Math.max(selectedIndex.value - 1, 0)
  } else if (e.key === 'Enter' && selectedIndex.value >= 0) {
    e.preventDefault()
    navigateToResult(searchResults.value[selectedIndex.value])
  } else if (e.key === 'Escape') {
    showDropdown.value = false
  }
}

function clearSearch() {
  setSearchQuery('')
  showDropdown.value = false
  inputRef.value?.focus()
}
</script>

<template>
  <div class="relative">
    <div class="relative">
      <svg class="absolute left-3 top-1/2 -translate-y-1/2 size-4 text-muted-foreground" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <circle cx="11" cy="11" r="8" />
        <line x1="21" y1="21" x2="16.65" y2="16.65" />
      </svg>
      <input
        ref="inputRef"
        :value="searchQuery"
        type="text"
        :placeholder="t('settings.search.placeholder')"
        class="h-10 w-full rounded-lg border border-input bg-background pl-10 pr-10 text-sm outline-none focus:border-primary focus:ring-1 focus:ring-primary"
        @input="handleInput"
        @focus="handleFocus"
        @blur="handleBlur"
        @keydown="handleKeydown"
      />
      <button
        v-if="searchQuery"
        class="absolute right-3 top-1/2 -translate-y-1/2 size-4 text-muted-foreground hover:text-foreground"
        @click="clearSearch"
      >
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <line x1="18" y1="6" x2="6" y2="18" />
          <line x1="6" y1="6" x2="18" y2="18" />
        </svg>
      </button>
    </div>

    <!-- Search Results Dropdown -->
    <div
      v-if="showDropdown && searchResults.length > 0"
      class="absolute top-full left-0 right-0 mt-1 rounded-lg border border-border bg-popover shadow-xl z-50 max-h-80 overflow-y-auto"
    >
      <div class="px-3 py-2 border-b border-border">
        <p class="text-xs text-muted-foreground">{{ t('settings.search.results', { count: searchResults.length }) }}</p>
      </div>
      <ul class="py-1">
        <li
          v-for="(result, idx) in searchResults"
          :key="`${result.category.id}-${result.item.key}`"
          class="px-3 py-2 cursor-pointer transition-colors"
          :class="idx === selectedIndex ? 'bg-accent' : 'hover:bg-accent/50'"
          @mousedown="navigateToResult(result)"
        >
          <div class="flex items-center gap-2">
            <span class="text-xs font-medium text-primary shrink-0">{{ result.category.id }}</span>
            <span class="text-sm text-foreground truncate">{{ result.item.label }}</span>
          </div>
          <p class="text-xs text-muted-foreground mt-0.5">{{ result.item.description }}</p>
        </li>
      </ul>
    </div>

    <!-- No Results -->
    <div
      v-else-if="showDropdown && searchQuery.length >= 2 && searchResults.length === 0"
      class="absolute top-full left-0 right-0 mt-1 rounded-lg border border-border bg-popover shadow-xl z-50 p-4 text-center"
    >
      <p class="text-sm text-muted-foreground">{{ t('settings.search.noResults', { query: searchQuery }) }}</p>
    </div>
  </div>
</template>

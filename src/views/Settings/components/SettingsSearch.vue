<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { useSettingsSearch } from '@/composables/useSettingsSearch'
import { PhMagnifyingGlass, PhX } from '@phosphor-icons/vue'

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
  await router.push(result.category.route)
  await new Promise(resolve => setTimeout(resolve, 100))
  const el = document.getElementById(`setting-${result.item.key}`)
  if (el) {
    el.scrollIntoView({ behavior: 'smooth', block: 'center' })
    el.classList.add('ring-2', 'ring-primary', 'ring-offset-2', 'dark:ring-offset-background')
    setTimeout(() => el.classList.remove('ring-2', 'ring-primary', 'ring-offset-2', 'dark:ring-offset-background'), 2000)
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
      <PhMagnifyingGlass :size="16" class="absolute left-2.5 top-1/2 -translate-y-1/2 text-muted-foreground" />
      <input
        ref="inputRef"
        :value="searchQuery"
        type="text"
        :placeholder="t('settings.search.placeholder')"
        class="h-8 w-full rounded-md border border-input bg-background pl-8 pr-8 text-xs outline-none focus:border-primary focus:ring-1 focus:ring-primary"
        @input="handleInput"
        @focus="handleFocus"
        @blur="handleBlur"
        @keydown="handleKeydown"
      />
      <button
        v-if="searchQuery"
        class="absolute right-2.5 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground"
        @click="clearSearch"
      >
        <PhX :size="14" />
      </button>
    </div>

    <!-- Results Dropdown -->
    <div
      v-if="showDropdown && searchResults.length > 0"
      class="absolute top-full left-0 right-0 mt-1 rounded-md border border-border bg-popover shadow-xl z-50 max-h-72 overflow-y-auto"
    >
      <ul class="py-1">
        <li
          v-for="(result, idx) in searchResults"
          :key="`${result.category.id}-${result.item.key}`"
          class="px-3 py-2 cursor-pointer transition-colors"
          :class="idx === selectedIndex ? 'bg-accent' : 'hover:bg-accent/50'"
          @mousedown="navigateToResult(result)"
        >
          <div class="text-xs text-foreground">{{ result.item.label }}</div>
          <div class="text-[10px] text-muted-foreground">{{ result.category.id }} → {{ result.section.id }}</div>
        </li>
      </ul>
    </div>

    <!-- No Results -->
    <div
      v-else-if="showDropdown && searchQuery.length >= 2 && searchResults.length === 0"
      class="absolute top-full left-0 right-0 mt-1 rounded-md border border-border bg-popover shadow-xl z-50 p-3 text-center"
    >
      <p class="text-xs text-muted-foreground">{{ t('settings.search.noResults', { query: searchQuery }) }}</p>
    </div>
  </div>
</template>

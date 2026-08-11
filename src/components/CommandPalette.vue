<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import { useTheme } from '@/composables/useTheme'
import { useSettingsStore } from '@/stores/settings'
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from '@/components/ui/command'
import { HugeiconsIcon } from '@hugeicons/vue'
import {
  Home01Icon,
  TrendingUpDownIcon,
  VideoReplayIcon,
  HistoryIcon,
  Download01Icon,
  Settings02Icon,
  Sun01Icon,
  Moon01Icon,
  FileImportIcon,
  FileExportIcon,
  Search01Icon,
} from '@hugeicons/core-free-icons'

interface CommandItem {
  id: string
  label: string
  icon: any
  shortcut?: string
  group: 'navigation' | 'actions'
  action: () => void
}

const props = defineProps<{
  open: boolean
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
}>()

const router = useRouter()
const { theme, setTheme } = useTheme()
const settingsStore = useSettingsStore()

const search = ref('')
const selectedIndex = ref(0)

// Build commands list
const commands = computed<CommandItem[]>(() => {
  const items: CommandItem[] = [
    // Navigation
    {
      id: 'go-home',
      label: 'Go to Home',
      icon: Home01Icon,
      group: 'navigation',
      action: () => {
        router.push({ name: 'home' })
        close()
      },
    },
    {
      id: 'go-trending',
      label: 'Go to Trending',
      icon: TrendingUpDownIcon,
      group: 'navigation',
      action: () => {
        router.push({ name: 'trending' })
        close()
      },
    },
    {
      id: 'go-subscriptions',
      label: 'Go to Subscriptions',
      icon: VideoReplayIcon,
      group: 'navigation',
      action: () => {
        router.push({ name: 'subscriptions' })
        close()
      },
    },
    {
      id: 'go-history',
      label: 'Go to History',
      icon: HistoryIcon,
      group: 'navigation',
      action: () => {
        router.push({ name: 'history' })
        close()
      },
    },
    {
      id: 'go-downloads',
      label: 'Go to Downloads',
      icon: Download01Icon,
      group: 'navigation',
      action: () => {
        router.push({ name: 'downloads' })
        close()
      },
    },
    {
      id: 'go-settings',
      label: 'Go to Settings',
      icon: Settings02Icon,
      group: 'navigation',
      action: () => {
        router.push({ name: 'settings' })
        close()
      },
    },
    // Actions
    {
      id: 'toggle-theme',
      label: theme.value === 'dark' ? 'Switch to Light Mode' : 'Switch to Dark Mode',
      icon: theme.value === 'dark' ? Sun01Icon : Moon01Icon,
      group: 'actions',
      action: () => {
        setTheme(theme.value === 'dark' ? 'light' : 'dark')
        close()
      },
    },
    {
      id: 'import-settings',
      label: 'Import Settings',
      icon: FileImportIcon,
      group: 'actions',
      action: () => {
        const input = document.createElement('input')
        input.type = 'file'
        input.accept = '.json'
        input.onchange = async (e) => {
          const file = (e.target as HTMLInputElement).files?.[0]
          if (!file) return
          try {
            const text = await file.text()
            const imported = JSON.parse(text)
            await settingsStore.importSettings(imported)
          } catch {
            // Invalid JSON, silently fail
          }
        }
        input.click()
        close()
      },
    },
    {
      id: 'export-settings',
      label: 'Export Settings',
      icon: FileExportIcon,
      group: 'actions',
      action: () => {
        const settings = settingsStore.exportSettings()
        const blob = new Blob([JSON.stringify(settings, null, 2)], { type: 'application/json' })
        const url = URL.createObjectURL(blob)
        const a = document.createElement('a')
        a.href = url
        a.download = 'slytube-settings.json'
        a.click()
        URL.revokeObjectURL(url)
        close()
      },
    },
  ]
  return items
})

// Filter commands based on search
const filteredGroups = computed(() => {
  const query = search.value.toLowerCase().trim()
  const filtered = query
    ? commands.value.filter((cmd) => cmd.label.toLowerCase().includes(query))
    : commands.value

  const groups: Record<string, CommandItem[]> = {}
  for (const cmd of filtered) {
    if (!groups[cmd.group]) {
      groups[cmd.group] = []
    }
    groups[cmd.group].push(cmd)
  }
  return groups
})

// Flatten filtered items for keyboard navigation
const flatFilteredItems = computed(() => {
  const items: CommandItem[] = []
  for (const groupItems of Object.values(filteredGroups.value)) {
    items.push(...groupItems)
  }
  return items
})

watch(
  () => props.open,
  (isOpen) => {
    if (isOpen) {
      search.value = ''
      selectedIndex.value = 0
    }
  }
)

function close() {
  emit('update:open', false)
}

function handleKeyDown(e: KeyboardEvent) {
  if (e.key === 'Escape') {
    e.preventDefault()
    close()
  } else if (e.key === 'ArrowDown') {
    e.preventDefault()
    selectedIndex.value = Math.min(selectedIndex.value + 1, flatFilteredItems.value.length - 1)
  } else if (e.key === 'ArrowUp') {
    e.preventDefault()
    selectedIndex.value = Math.max(selectedIndex.value - 1, 0)
  } else if (e.key === 'Enter') {
    e.preventDefault()
    const item = flatFilteredItems.value[selectedIndex.value]
    if (item) {
      item.action()
    }
  }
}

function handleItemSelect(item: CommandItem) {
  item.action()
}
</script>

<template>
  <Teleport to="body">
    <!-- Backdrop -->
    <Transition name="fade">
      <div
        v-if="open"
        class="fixed inset-0 z-50 bg-black/60 backdrop-blur-sm"
        @click="close"
      />
    </Transition>

    <!-- Command Palette -->
    <Transition name="scale">
      <div
        v-if="open"
        class="fixed left-1/2 top-[20%] z-50 w-full max-w-xl -translate-x-1/2"
        @keydown="handleKeyDown"
      >
        <Command
          class="rounded-lg border bg-popover shadow-2xl"
        >
          <div class="flex items-center border-b px-3">
            <HugeiconsIcon :icon="Search01Icon" class="mr-2 h-4 w-4 shrink-0 opacity-50" />
            <CommandInput
              v-model="search"
              placeholder="Type a command or search..."
              class="flex h-11 w-full rounded-md bg-transparent py-3 text-sm outline-none placeholder:text-muted-foreground"
            />
          </div>

          <CommandList class="max-h-[300px] overflow-y-auto p-1">
            <CommandEmpty>No results found.</CommandEmpty>

            <CommandGroup
              v-for="(items, group) in filteredGroups"
              :key="group"
              :heading="group === 'navigation' ? 'Navigation' : 'Actions'"
            >
              <CommandItem
                v-for="item in items"
                :key="item.id"
                :value="item.label"
                :data-index="flatFilteredItems.indexOf(item)"
                :data-selected="flatFilteredItems.indexOf(item) === selectedIndex"
                :class="'flex items-center gap-2 px-2 py-2.5 text-sm cursor-pointer rounded-sm' + (flatFilteredItems.indexOf(item) === selectedIndex ? ' bg-accent text-accent-foreground' : '')"
                @click="handleItemSelect(item)"
                @mouseenter="selectedIndex = flatFilteredItems.indexOf(item)"
              >
                <HugeiconsIcon :icon="item.icon" class="h-4 w-4 shrink-0 opacity-70" />
                <span>{{ item.label }}</span>
              </CommandItem>
            </CommandGroup>
          </CommandList>

          <!-- Footer -->
          <div class="flex items-center justify-between border-t px-3 py-2 text-xs text-muted-foreground">
            <div class="flex items-center gap-3">
              <span class="flex items-center gap-1">
                <kbd class="rounded bg-muted px-1.5 py-0.5 font-mono text-[10px]">↑</kbd>
                <kbd class="rounded bg-muted px-1.5 py-0.5 font-mono text-[10px]">↓</kbd>
                Navigate
              </span>
              <span class="flex items-center gap-1">
                <kbd class="rounded bg-muted px-1.5 py-0.5 font-mono text-[10px]">↵</kbd>
                Select
              </span>
            </div>
            <span class="flex items-center gap-1">
              <kbd class="rounded bg-muted px-1.5 py-0.5 font-mono text-[10px]">esc</kbd>
              Close
            </span>
          </div>
        </Command>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.15s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}

.scale-enter-active,
.scale-leave-active {
  transition: all 0.15s ease;
}

.scale-enter-from,
.scale-leave-to {
  opacity: 0;
  transform: translateX(-50%) scale(0.95);
}
</style>

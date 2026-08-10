<script setup lang="ts">
import { computed } from 'vue'
import { useRouter } from 'vue-router'
import { cn } from '@/lib/utils'
import { useTabsStore } from '@/stores/tabs'
import { useSettingsStore } from '@/stores/settings'

const router = useRouter()
const tabsStore = useTabsStore()
const settingsStore = useSettingsStore()

const tabs = computed(() => tabsStore.getTabs)
const activeTabId = computed(() => tabsStore.getActiveTabId)

function handleTabClick(tabId: string) {
  tabsStore.activateTab(tabId)
  const tab = tabsStore.getTabById(tabId)
  if (tab) {
    router.push(tab.route)
  }
}

function handleTabClose(tabId: string, event: MouseEvent) {
  event.stopPropagation()
  tabsStore.closeTab(tabId)
}

function handleNewTab() {
  tabsStore.createTab({
    name: 'Home',
    path: '/',
    params: {},
    query: {},
    hash: '',
    fullPath: '/',
  })
  router.push('/')
}
</script>

<template>
  <div
    :class="cn(
      'flex items-center h-9 bg-muted/50 border-b border-border overflow-x-auto scrollbar-none',
      settingsStore.useVerticalTabBar ? 'flex-col h-full w-auto border-r border-b-0 overflow-y-auto' : ''
    )"
  >
    <div
      v-for="tab in tabs"
      :key="tab.id"
      :class="cn(
        'group flex items-center gap-1 px-3 h-full text-sm cursor-pointer border-r border-border/50 min-w-0 max-w-[200px] transition-colors',
        activeTabId === tab.id
          ? 'bg-background text-foreground border-b-2 border-b-primary'
          : 'text-muted-foreground hover:bg-muted',
        settingsStore.useVerticalTabBar && 'w-full max-w-none border-r-0 border-b border-border/50'
      )"
      @click="handleTabClick(tab.id)"
    >
      <span v-if="settingsStore.showTabIcons" class="shrink-0 size-4">
        <svg class="size-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M13 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V9z" />
          <polyline points="13 2 13 9 20 9" />
        </svg>
      </span>
      <span class="truncate text-xs">{{ tab.contentTitle || tab.route.path || 'New Tab' }}</span>
      <button
        v-if="!tab.isPinned"
        class="shrink-0 ml-auto size-4 rounded-sm opacity-0 group-hover:opacity-100 hover:bg-muted-foreground/20 flex items-center justify-center transition-opacity"
        @click="handleTabClose(tab.id, $event)"
      >
        <svg class="size-3" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <line x1="18" y1="6" x2="6" y2="18" />
          <line x1="6" y1="6" x2="18" y2="18" />
        </svg>
      </button>
    </div>

    <!-- New Tab Button -->
    <button
      :class="cn(
        'flex items-center justify-center size-9 shrink-0 text-muted-foreground hover:bg-muted hover:text-foreground transition-colors',
        settingsStore.useVerticalTabBar && 'w-full'
      )"
      title="New Tab"
      @click="handleNewTab"
    >
      <svg class="size-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <line x1="12" y1="5" x2="12" y2="19" />
        <line x1="5" y1="12" x2="19" y2="12" />
      </svg>
    </button>
  </div>
</template>

import { defineStore } from 'pinia'

export interface TabHistoryEntry {
  route: {
    name: string | null
    path: string
    params: Record<string, string>
    query: Record<string, string>
    hash: string
    fullPath: string
  }
  title: string
  titlePending: boolean
  scroll: { left: number; top: number }
}

export interface Tab {
  id: string
  route: {
    name: string | null
    path: string
    params: Record<string, string>
    query: Record<string, string>
    hash: string
    fullPath: string
  }
  history: TabHistoryEntry[]
  historyIndex: number
  contentTitle: string
  pendingReloadRoute: TabHistoryEntry['route'] | null
  refreshKey: number
  isLoading: boolean
  isPinned: boolean
  color: string | null
  url?: string
  title?: string
  loadState?: string
}

export interface TabsState {
  tabs: Tab[]
  activeTabId: string | null
  selectedTabIds: string[]
  presentedTabId: string | null
  mainPresentedTabId: string | null
  selectionRevision: number
  transitionRevision: number
  transitionTargetTabId: string | null
  containerIds: string[]
  tabBarScrollPosition: number
  currentWatchTimestamps: Record<string, number>
}

const MAX_LOGICAL_HISTORY_ENTRIES = 100
const NAV_HISTORY_DISPLAY_LIMIT = 15
const HALF_NAV_HISTORY_DISPLAY_LIMIT = Math.trunc(NAV_HISTORY_DISPLAY_LIMIT / 2)

export const useTabsStore = defineStore('tabs', {
  state: (): TabsState => ({
    tabs: [],
    activeTabId: null,
    selectedTabIds: [],
    presentedTabId: null,
    mainPresentedTabId: null,
    selectionRevision: 0,
    transitionRevision: 0,
    transitionTargetTabId: null,
    containerIds: [],
    tabBarScrollPosition: 0,
    currentWatchTimestamps: {},
  }),

  getters: {
    getTabs: (state) => state.tabs,
    getActiveTabId: (state) => state.activeTabId,
    getActiveTab: (state) => state.tabs.find((tab) => tab.id === state.activeTabId) ?? null,
    getSelectedTabIds: (state) => state.selectedTabIds,
    getPresentedTabId: (state) => state.presentedTabId,
    getPresentedTab: (state) => state.tabs.find((tab) => tab.id === state.presentedTabId) ?? null,
    getTabById: (state) => (tabId: string) => state.tabs.find((tab) => tab.id === tabId) ?? null,
    getTabCount: (state) => state.tabs.length,
    getTabContainerIds: (state) => state.containerIds,
    getTabBarScrollPosition: (state) => state.tabBarScrollPosition,
    getCurrentWatchTimestamp: (state) => state.currentWatchTimestamps[state.activeTabId ?? ''] ?? null,
    getWatchTimestamp: (state) => (tabId: string) => state.currentWatchTimestamps[tabId] ?? null,

    getTabHistoryState: (state) => (tabId: string) => {
      const tab = state.tabs.find((candidate) => candidate.id === tabId)
      if (!tab) {
        return { canGoBack: false, canGoForward: false, options: [] }
      }

      const historyLength = tab.history.length
      let end: number
      if (tab.historyIndex < HALF_NAV_HISTORY_DISPLAY_LIMIT) {
        end = Math.min(historyLength - 1, NAV_HISTORY_DISPLAY_LIMIT - 1)
      } else if (historyLength - tab.historyIndex < HALF_NAV_HISTORY_DISPLAY_LIMIT + 1) {
        end = historyLength - 1
      } else {
        end = tab.historyIndex + HALF_NAV_HISTORY_DISPLAY_LIMIT
      }

      const options: Array<{ label: string; value: number; active: boolean; icon?: unknown }> = []
      for (let index = end; index >= Math.max(0, end + 1 - NAV_HISTORY_DISPLAY_LIMIT); index--) {
        const entry = tab.history[index]
        options.push({
          label: entry.title || entry.route.fullPath,
          value: index - tab.historyIndex,
          active: index === tab.historyIndex,
        })
      }

      return {
        canGoBack: tab.historyIndex > 0,
        canGoForward: tab.historyIndex < historyLength - 1,
        options,
      }
    },
  },

  actions: {
    setTabsState(payload: Partial<TabsState> = {}) {
      const incomingTabs = payload.tabs ?? []
      const incomingIds = new Set(incomingTabs.map((tab) => tab.id))

      this.containerIds = this.containerIds.filter((tabId) => incomingIds.has(tabId))
      for (const tab of incomingTabs) {
        if (!this.containerIds.includes(tab.id)) {
          this.containerIds.push(tab.id)
        }
      }

      this.tabs = incomingTabs
      this.selectedTabIds = this.selectedTabIds.filter((tabId) => incomingIds.has(tabId))
      this.activeTabId = payload.activeTabId ?? null
      this.mainPresentedTabId = payload.presentedTabId ?? null
      this.selectionRevision = payload.selectionRevision ?? this.selectionRevision

      if (payload.tabBarScrollPosition != null) {
        this.tabBarScrollPosition = payload.tabBarScrollPosition
      }

      for (const tabId of Object.keys(this.currentWatchTimestamps)) {
        if (!incomingIds.has(tabId)) {
          delete this.currentWatchTimestamps[tabId]
        }
      }
    },

    createTab(route: Tab['route']) {
      const tab: Tab = {
        id: `tab--${Date.now()}-${Math.floor(Math.random() * 10000)}`,
        route,
        history: [{ route, title: route.fullPath, titlePending: false, scroll: { left: 0, top: 0 } }],
        historyIndex: 0,
        contentTitle: '',
        pendingReloadRoute: null,
        refreshKey: 0,
        isLoading: false,
        isPinned: false,
        color: null,
      }
      this.tabs.push(tab)
      this.activeTabId = tab.id
      return tab
    },

    closeTab(id: string) {
      const index = this.tabs.findIndex((tab) => tab.id === id)
      if (index !== -1) {
        this.tabs.splice(index, 1)
        if (this.activeTabId === id) {
          this.activeTabId = this.tabs[Math.min(index, this.tabs.length - 1)]?.id ?? null
        }
      }
    },

    activateTab(id: string) {
      this.activeTabId = id
    },

    moveTab(from: number, to: number) {
      if (from < 0 || from >= this.tabs.length || to < 0 || to >= this.tabs.length) {
        return
      }
      const [tab] = this.tabs.splice(from, 1)
      this.tabs.splice(to, 0, tab)
    },

    setTabPinned(id: string, pinned: boolean) {
      const tab = this.tabs.find((t) => t.id === id)
      if (tab) {
        tab.isPinned = pinned
      }
    },

    setTabColor(id: string, color: string) {
      const tab = this.tabs.find((t) => t.id === id)
      if (tab) {
        tab.color = color
      }
    },

    setTabNavigation(tabId: string, route: Tab['route'], history: TabHistoryEntry[], historyIndex: number) {
      const tab = this.tabs.find((candidate) => candidate.id === tabId)
      if (!tab) return

      tab.route = route
      tab.history = history.slice(-MAX_LOGICAL_HISTORY_ENTRIES)
      tab.historyIndex = Math.max(0, Math.min(historyIndex, tab.history.length - 1))
    },

    setTabContentTitle(tabId: string, title: string) {
      const tab = this.tabs.find((candidate) => candidate.id === tabId)
      if (!tab) return

      tab.contentTitle = title
      const entry = tab.history[tab.historyIndex]
      if (entry) {
        entry.titlePending = false
        const resolvedTitle = title || entry.route.fullPath
        if (resolvedTitle !== entry.route.fullPath || entry.title === entry.route.fullPath) {
          entry.title = resolvedTitle
        }
      }
    },

    setCurrentWatchTimestamp(tabId: string, value: number) {
      if (typeof value === 'number' && Number.isFinite(value) && value > 0) {
        this.currentWatchTimestamps[tabId] = value
      } else {
        delete this.currentWatchTimestamps[tabId]
      }
    },
  },
})

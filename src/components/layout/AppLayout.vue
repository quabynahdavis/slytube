<script setup lang="ts">
import { computed, ref } from 'vue'
import { useSettingsStore } from '@/stores/settings'
import { useRoute } from 'vue-router'
import { useTheme } from '@/composables/useTheme'
import { useKeyboardShortcuts } from '@/composables/useKeyboardShortcuts'
import SideNav from './SideNav.vue'
import TopNav from './TopNav.vue'
import ToastContainer from '@/components/ToastContainer.vue'
import ErrorBoundary from '@/components/error/ErrorBoundary.vue'
import CommandPalette from '@/components/CommandPalette.vue'

const settingsStore = useSettingsStore()
const route = useRoute()
const { theme, setTheme } = useTheme()
const { register } = useKeyboardShortcuts()

const isWatchPage = computed(() => route.path === '/watch')
const hideSideBarOnWatch = computed(() => settingsStore.hideSideBarOnWatchPages && isWatchPage.value)
const sidebarOverlayOpen = ref(false)

const topNavRef = ref<InstanceType<typeof TopNav> | null>(null)
const mainRef = ref<HTMLElement | null>(null)
const commandPaletteOpen = ref(false)

function toggleSidebar() {
  if (isWatchPage.value) {
    sidebarOverlayOpen.value = !sidebarOverlayOpen.value
  } else {
    settingsStore.updateSetting('expandSideBar', !settingsStore.expandSideBar)
  }
}

function closeSidebarOverlay() {
  sidebarOverlayOpen.value = false
}

// Register keyboard shortcuts
register('/', () => {
  topNavRef.value?.focusSearch()
})

register('t', () => {
  const current = theme.value
  const next = current === 'dark' ? 'light' : 'dark'
  setTheme(next)
  settingsStore.baseTheme = next
})

register('j', () => {
  if (mainRef.value) {
    mainRef.value.scrollBy({ top: 200, behavior: 'smooth' })
  }
})

register('k', () => {
  if (mainRef.value) {
    mainRef.value.scrollBy({ top: -200, behavior: 'smooth' })
  }
})

register('escape', () => {
  sidebarOverlayOpen.value = false
  window.dispatchEvent(new CustomEvent('close-dialogs'))
})

register('mod+k', () => {
  commandPaletteOpen.value = !commandPaletteOpen.value
})
</script>

<template>
  <div class="flex flex-col h-screen overflow-hidden bg-background">
    <!-- Top Navigation (full width, always) -->
    <TopNav ref="topNavRef" @toggle-sidebar="toggleSidebar" />

    <!-- Below Header: Sidebar + Content -->
    <div class="flex flex-1 overflow-hidden relative">
      <!-- Side Navigation: Normal mode (not watch page) -->
      <SideNav
        v-if="!hideSideBarOnWatch"
        mode="normal"
      />

      <!-- Page Content -->
      <main ref="mainRef" class="flex-1 overflow-y-auto">
        <ErrorBoundary>
        <router-view v-slot="{ Component }">
          <transition name="fade" mode="out-in">
            <component :is="Component" />
          </transition>
        </router-view>
        </ErrorBoundary>
      </main>

      <!-- Overlay Sidebar for Watch Page -->
      <Transition name="slide">
        <div
          v-if="isWatchPage && sidebarOverlayOpen"
          class="absolute inset-0 z-50 flex"
        >
          <!-- Backdrop -->
          <div
            class="absolute inset-0 bg-black/50"
            @click="closeSidebarOverlay"
          />
          <!-- Sidebar -->
          <SideNav
            mode="overlay"
            @close="closeSidebarOverlay"
          />
        </div>
      </Transition>
    </div>

    <!-- Toast Notifications -->
    <ToastContainer />

    <!-- Command Palette -->
    <CommandPalette v-model:open="commandPaletteOpen" />
  </div>
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

.slide-enter-active,
.slide-leave-active {
  transition: opacity 0.2s ease;
}
.slide-enter-from,
.slide-leave-to {
  opacity: 0;
}
</style>

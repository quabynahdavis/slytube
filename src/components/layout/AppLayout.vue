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

const settingsStore = useSettingsStore()
const route = useRoute()
const { theme, setTheme } = useTheme()
const { register } = useKeyboardShortcuts()

const hideSideBarOnWatch = computed(() => settingsStore.hideSideBarOnWatchPages && route.path === '/watch')

const topNavRef = ref<InstanceType<typeof TopNav> | null>(null)
const mainRef = ref<HTMLElement | null>(null)

// Register keyboard shortcuts
register('/', () => {
  topNavRef.value?.focusSearch()
})

register('t', () => {
  const current = theme.value
  const next = current === 'dark' ? 'light' : 'dark'
  setTheme(next)
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
  // Close any open dialogs by dispatching a global event
  window.dispatchEvent(new CustomEvent('close-dialogs'))
})
</script>

<template>
  <div class="flex h-screen overflow-hidden bg-background">
    <!-- Side Navigation -->
    <SideNav
      v-if="!hideSideBarOnWatch"
    />

    <!-- Main Content Area -->
    <div class="flex flex-1 flex-col overflow-hidden">
      <!-- Top Navigation -->
      <TopNav ref="topNavRef" />

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
    </div>

    <!-- Toast Notifications -->
    <ToastContainer />
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
</style>

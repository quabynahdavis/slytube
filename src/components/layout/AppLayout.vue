<script setup lang="ts">
import { computed } from 'vue'
import { cn } from '@/lib/utils'
import { useSettingsStore } from '@/stores/settings'
import SideNav from './SideNav.vue'
import TopNav from './TopNav.vue'
import TabBar from './TabBar.vue'

const settingsStore = useSettingsStore()

const isVerticalTabBar = computed(() => settingsStore.useVerticalTabBar)
const hideSideBarOnWatch = computed(() => settingsStore.hideSideBarOnWatchPages)
</script>

<template>
  <div class="flex h-screen overflow-hidden bg-background">
    <!-- Side Navigation -->
    <SideNav
      v-if="!hideSideBarOnWatch"
      :class="cn(
        isVerticalTabBar ? 'hidden' : ''
      )"
    />

    <!-- Main Content Area -->
    <div class="flex flex-1 flex-col overflow-hidden">
      <!-- Top Navigation -->
      <TopNav />

      <!-- Tab Bar -->
      <TabBar />

      <!-- Page Content -->
      <main class="flex-1 overflow-y-auto">
        <router-view v-slot="{ Component }">
          <transition name="fade" mode="out-in">
            <component :is="Component" />
          </transition>
        </router-view>
      </main>
    </div>
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

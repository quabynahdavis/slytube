<script setup lang="ts">
import { useRoute } from 'vue-router'

const route = useRoute()

const tabs = [
  { id: 'overview', label: 'Overview', icon: 'info' },
  { id: 'system', label: 'System', icon: 'monitor' },
  { id: 'library', label: 'Library', icon: 'library' },
  { id: 'shortcuts', label: 'Shortcuts', icon: 'keyboard' },
  { id: 'changelog', label: 'Changelog', icon: 'history' },
  { id: 'license', label: 'License', icon: 'document' },
]

function getIcon(icon: string): string {
  const icons: Record<string, string> = {
    info: 'M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z',
    monitor: 'M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z',
    library: 'M8 14v3m4-3v3m4-3v3M3 21h18M3 10h18M3 7l9-4 9 4M4 10h16v11H4V10z',
    keyboard: 'M12 18h.01M8 21h8a2 2 0 002-2V5a2 2 0 00-2-2H8a2 2 0 00-2 2v14a2 2 0 002 2z',
    history: 'M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z',
    document: 'M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z',
  }
  return icons[icon] || icons.info
}
</script>

<template>
  <div class="min-h-screen bg-background">
    <!-- Header -->
    <div class="border-b border-border bg-card">
      <div class="container mx-auto max-w-3xl px-4 py-6">
        <div class="flex items-center gap-4">
          <div class="size-12 rounded-xl bg-primary/10 flex items-center justify-center">
            <svg class="size-6 text-primary" viewBox="0 0 24 24" fill="currentColor">
              <path d="M23.498 6.186a3.016 3.016 0 0 0-2.122-2.136C19.505 3.545 12 3.545 12 3.545s-7.505 0-9.377.505A3.017 3.017 0 0 0 .502 6.186C0 8.07 0 12 0 12s0 3.93.502 5.814a3.016 3.016 0 0 0 2.122 2.136c1.871.505 9.376.505 9.376.505s7.505 0 9.377-.505a3.015 3.015 0 0 0 2.122-2.136C24 15.93 24 12 24 12s0-3.93-.502-5.814zM9.545 15.568V8.432L15.818 12l-6.273 3.568z" />
            </svg>
          </div>
          <div>
            <h1 class="text-xl font-bold text-foreground">About</h1>
            <p class="text-sm text-muted-foreground">Slytube v0.1.0</p>
          </div>
        </div>
      </div>

      <!-- Tabs -->
      <div class="container mx-auto max-w-3xl px-4">
        <nav class="flex gap-1 overflow-x-auto">
          <router-link
            v-for="tab in tabs"
            :key="tab.id"
            :to="`/about/${tab.id}`"
            class="flex items-center gap-2 px-4 py-3 text-sm font-medium border-b-2 transition-colors whitespace-nowrap"
            :class="route.name === `about-${tab.id}`
              ? 'border-primary text-primary'
              : 'border-transparent text-muted-foreground hover:text-foreground hover:border-border'"
          >
            <svg class="size-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path :d="getIcon(tab.icon)" />
            </svg>
            {{ tab.label }}
          </router-link>
        </nav>
      </div>
    </div>

    <!-- Content -->
    <div class="container mx-auto max-w-3xl px-4 py-6">
      <router-view />
    </div>
  </div>
</template>

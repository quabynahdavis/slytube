<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'

const appVersion = ref('0.1.0')

onMounted(async () => {
  try {
    appVersion.value = await invoke<string>('system_get_version')
  } catch {
    // use default
  }
})

const features = [
  'Ad-free video watching',
  'Subscription management without an account',
  'Video downloading support',
  'SponsorBlock integration',
  'Multi-profile support',
  'Cross-device sync support',
  'Keyboard shortcuts',
  'Dark mode support',
]
</script>

<template>
  <div class="space-y-6">
    <div class="rounded-xl border border-border bg-card p-6">
      <h2 class="text-lg font-semibold text-foreground mb-3">About Slytube</h2>
      <p class="text-sm text-muted-foreground leading-relaxed">
        SlyTube is a free and open-source YouTube client that puts your privacy first.
        It provides a clean, ad-free experience for watching YouTube content without tracking.
        Built with Vue 3 and Tauri for a fast, native desktop experience.
      </p>
    </div>

    <div class="rounded-xl border border-border bg-card p-6">
      <h2 class="text-lg font-semibold text-foreground mb-4">Features</h2>
      <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
        <div
          v-for="feature in features"
          :key="feature"
          class="flex items-center gap-3 rounded-lg bg-background/50 p-3"
        >
          <div class="size-8 rounded-lg bg-primary/10 flex items-center justify-center shrink-0">
            <svg class="size-4 text-primary" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <polyline points="20 6 9 17 4 12" />
            </svg>
          </div>
          <span class="text-sm text-foreground">{{ feature }}</span>
        </div>
      </div>
    </div>

    <div class="rounded-xl border border-border bg-card p-6">
      <h2 class="text-lg font-semibold text-foreground mb-2">Version</h2>
      <p class="text-sm text-muted-foreground">
        Current version: <span class="font-medium text-foreground">{{ appVersion }}</span>
      </p>
    </div>
  </div>
</template>

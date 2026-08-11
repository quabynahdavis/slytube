<script setup lang="ts">
import { ref } from 'vue'

const appVersion = ref('0.1.0')
const appEnvironment = ref('Development')

// Try to get version from import.meta.env (Vite environment)
try {
  const envVersion = import.meta.env.VITE_APP_VERSION
  if (envVersion) {
    appVersion.value = envVersion
  }
  const envMode = import.meta.env.MODE
  if (envMode) {
    appEnvironment.value = envMode.charAt(0).toUpperCase() + envMode.slice(1)
  }
} catch {
  // Fallback to defaults
}
</script>

<template>
  <div class="container mx-auto max-w-3xl px-4 py-12">
    <div class="text-center mb-8">
      <div class="size-20 mx-auto rounded-2xl bg-primary/10 flex items-center justify-center mb-4">
        <svg class="size-10 text-primary" viewBox="0 0 24 24" fill="currentColor">
          <path d="M23.498 6.186a3.016 3.016 0 0 0-2.122-2.136C19.505 3.545 12 3.545 12 3.545s-7.505 0-9.377.505A3.017 3.017 0 0 0 .502 6.186C0 8.07 0 12 0 12s0 3.93.502 5.814a3.016 3.016 0 0 0 2.122 2.136c1.871.505 9.376.505 9.376.505s7.505 0 9.377-.505a3.015 3.015 0 0 0 2.122-2.136C24 15.93 24 12 24 12s0-3.93-.502-5.814zM9.545 15.568V8.432L15.818 12l-6.273 3.568z" />
        </svg>
      </div>
      <h1 class="text-3xl font-bold text-foreground">SlyTube</h1>
      <p class="text-muted-foreground mt-2">A privacy-respecting YouTube client</p>
      <p class="text-xs text-muted-foreground mt-1">Version {{ appVersion }} ({{ appEnvironment }})</p>
    </div>

    <div class="space-y-4">
      <div class="rounded-lg border border-border bg-card p-6">
        <h2 class="text-lg font-semibold text-foreground mb-2">About SlyTube</h2>
        <p class="text-sm text-muted-foreground leading-relaxed">
          SlyTube is a free and open-source YouTube client that puts your privacy first.
          It provides a clean, ad-free experience for watching YouTube content without tracking.
        </p>
      </div>

      <div class="rounded-lg border border-border bg-card p-6">
        <h2 class="text-lg font-semibold text-foreground mb-3">Features</h2>
        <ul class="space-y-2">
          <li class="flex items-start gap-2 text-sm text-muted-foreground">
            <svg class="size-4 mt-0.5 text-green-500 shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="20 6 9 17 4 12"/></svg>
            Ad-free video watching
          </li>
          <li class="flex items-start gap-2 text-sm text-muted-foreground">
            <svg class="size-4 mt-0.5 text-green-500 shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="20 6 9 17 4 12"/></svg>
            Subscription management without an account
          </li>
          <li class="flex items-start gap-2 text-sm text-muted-foreground">
            <svg class="size-4 mt-0.5 text-green-500 shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="20 6 9 17 4 12"/></svg>
            Video downloading support
          </li>
          <li class="flex items-start gap-2 text-sm text-muted-foreground">
            <svg class="size-4 mt-0.5 text-green-500 shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="20 6 9 17 4 12"/></svg>
            SponsorBlock integration
          </li>
          <li class="flex items-start gap-2 text-sm text-muted-foreground">
            <svg class="size-4 mt-0.5 text-green-500 shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="20 6 9 17 4 12"/></svg>
            Multi-profile support
          </li>
          <li class="flex items-start gap-2 text-sm text-muted-foreground">
            <svg class="size-4 mt-0.5 text-green-500 shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="20 6 9 17 4 12"/></svg>
            Cross-device sync support
          </li>
        </ul>
      </div>

      <div class="rounded-lg border border-border bg-card p-6">
        <h2 class="text-lg font-semibold text-foreground mb-2">Technology</h2>
        <p class="text-sm text-muted-foreground leading-relaxed">
          Built with Vue 3, TypeScript, Tailwind CSS, Pinia, and Tauri.
          Uses Invidious and other privacy-respecting APIs for data.
        </p>
      </div>

      <div class="rounded-lg border border-border bg-card p-6">
        <h2 class="text-lg font-semibold text-foreground mb-2">License</h2>
        <p class="text-sm text-muted-foreground">
          SlyTube is free software licensed under the AGPL-3.0 License.
        </p>
      </div>
    </div>
  </div>
</template>

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

const shortcuts = [
  { key: '/', description: 'Focus search bar' },
  { key: 'T', description: 'Toggle theme' },
  { key: 'J', description: 'Scroll down' },
  { key: 'K', description: 'Scroll up' },
  { key: 'Space', description: 'Play / Pause' },
  { key: '←', description: 'Seek backward 5s' },
  { key: '→', description: 'Seek forward 5s' },
  { key: '↑', description: 'Volume up' },
  { key: '↓', description: 'Volume down' },
  { key: 'F', description: 'Toggle fullscreen' },
  { key: 'M', description: 'Toggle mute' },
  { key: 'Esc', description: 'Close dialog' },
]

const techStack = [
  { name: 'Vue 3', description: 'Progressive JavaScript framework' },
  { name: 'TypeScript', description: 'Type-safe JavaScript' },
  { name: 'Tailwind CSS', description: 'Utility-first CSS framework' },
  { name: 'Pinia', description: 'State management' },
  { name: 'Tauri', description: 'Desktop app framework' },
  { name: 'shadcn-vue', description: 'UI component library' },
  { name: 'youtubei.js', description: 'YouTube API client' },
]
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

      <!-- Keyboard Shortcuts -->
      <div class="rounded-lg border border-border bg-card p-6">
        <h2 class="text-lg font-semibold text-foreground mb-3">Keyboard Shortcuts</h2>
        <div class="overflow-x-auto">
          <table class="w-full text-sm">
            <thead>
              <tr class="border-b border-border">
                <th class="text-left py-2 pr-4 font-medium text-foreground">Key</th>
                <th class="text-left py-2 font-medium text-foreground">Action</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="shortcut in shortcuts" :key="shortcut.key" class="border-b border-border/50 last:border-0">
                <td class="py-2 pr-4">
                  <kbd class="inline-flex items-center justify-center min-w-[2rem] h-7 px-2 rounded-md border border-border bg-muted text-xs font-mono text-foreground">
                    {{ shortcut.key }}
                  </kbd>
                </td>
                <td class="py-2 text-muted-foreground">{{ shortcut.description }}</td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>

      <!-- Technology Stack -->
      <div class="rounded-lg border border-border bg-card p-6">
        <h2 class="text-lg font-semibold text-foreground mb-3">Technology Stack</h2>
        <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
          <div
            v-for="tech in techStack"
            :key="tech.name"
            class="flex items-center gap-3 p-3 rounded-lg border border-border/50 bg-background/50"
          >
            <div class="size-8 rounded-md bg-primary/10 flex items-center justify-center shrink-0">
              <span class="text-xs font-bold text-primary">{{ tech.name[0] }}</span>
            </div>
            <div>
              <p class="text-sm font-medium text-foreground">{{ tech.name }}</p>
              <p class="text-xs text-muted-foreground">{{ tech.description }}</p>
            </div>
          </div>
        </div>
      </div>

      <!-- License -->
      <div class="rounded-lg border border-border bg-card p-6">
        <h2 class="text-lg font-semibold text-foreground mb-2">License</h2>
        <p class="text-sm text-muted-foreground">
          SlyTube is free software licensed under the
          <a
            href="https://www.gnu.org/licenses/agpl-3.0.html"
            target="_blank"
            rel="noopener noreferrer"
            class="text-primary hover:underline"
          >AGPL-3.0 License</a>.
        </p>
      </div>

      <!-- GitHub Link -->
      <div class="rounded-lg border border-border bg-card p-6">
        <h2 class="text-lg font-semibold text-foreground mb-2">Source Code</h2>
        <p class="text-sm text-muted-foreground mb-3">
          SlyTube is open source. View the code, report issues, or contribute on GitHub.
        </p>
        <a
          href="https://github.com/officialpack/slytube"
          target="_blank"
          rel="noopener noreferrer"
          class="inline-flex items-center gap-2 px-4 py-2 bg-secondary text-secondary-rounded-lg text-sm font-medium rounded-lg hover:bg-secondary/80 transition-colors"
        >
          <svg class="size-4" viewBox="0 0 24 24" fill="currentColor">
            <path d="M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z"/>
          </svg>
          View on GitHub
        </a>
      </div>
    </div>
  </div>
</template>

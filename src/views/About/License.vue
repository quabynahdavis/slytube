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

const techStack = [
  { name: 'Vue 3', description: 'Progressive JavaScript framework', icon: 'V' },
  { name: 'TypeScript', description: 'Type-safe JavaScript', icon: 'T' },
  { name: 'Tailwind CSS', description: 'Utility-first CSS framework', icon: 'W' },
  { name: 'Pinia', description: 'State management', icon: 'P' },
  { name: 'Tauri', description: 'Desktop app framework', icon: 'T' },
  { name: 'shadcn-vue', description: 'UI component library', icon: 'S' },
  { name: 'youtubei.js', description: 'YouTube API client', icon: 'Y' },
]
</script>

<template>
  <div class="space-y-6">
    <div class="rounded-xl border border-border bg-card p-6">
      <h2 class="text-lg font-semibold text-foreground mb-4">Technology Stack</h2>
      <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
        <div
          v-for="tech in techStack"
          :key="tech.name"
          class="flex items-center gap-3 rounded-lg border border-border/50 bg-background/50 p-3 hover:bg-accent/50 transition-colors"
        >
          <div class="size-10 rounded-lg bg-primary/10 flex items-center justify-center shrink-0">
            <span class="text-sm font-bold text-primary">{{ tech.icon }}</span>
          </div>
          <div>
            <p class="text-sm font-medium text-foreground">{{ tech.name }}</p>
            <p class="text-xs text-muted-foreground">{{ tech.description }}</p>
          </div>
        </div>
      </div>
    </div>

    <div class="rounded-xl border border-border bg-card p-6">
      <h2 class="text-lg font-semibold text-foreground mb-2">License</h2>
      <p class="text-sm text-muted-foreground mb-4">
        SlyTube is free software licensed under the
        <a
          href="https://www.gnu.org/licenses/agpl-3.0.html"
          target="_blank"
          rel="noopener noreferrer"
          class="text-primary hover:underline"
        >AGPL-3.0 License</a>.
      </p>
      <p class="text-sm text-muted-foreground">
        Version <span class="font-medium text-foreground">{{ appVersion }}</span>
      </p>
    </div>

    <div class="rounded-xl border border-border bg-card p-6">
      <h2 class="text-lg font-semibold text-foreground mb-2">Source Code</h2>
      <p class="text-sm text-muted-foreground mb-3">
        SlyTube is open source. View the code, report issues, or contribute on GitHub.
      </p>
      <a
        href="https://github.com/quabynahdavis/slytube"
        target="_blank"
        rel="noopener noreferrer"
        class="inline-flex items-center gap-2 px-4 py-2 bg-secondary text-secondary-foreground rounded-lg text-sm font-medium hover:bg-secondary/80 transition-colors"
      >
        <svg class="size-4" viewBox="0 0 24 24" fill="currentColor">
          <path d="M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z" />
        </svg>
        View on GitHub
      </a>
    </div>
  </div>
</template>

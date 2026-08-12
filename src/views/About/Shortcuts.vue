<script setup lang="ts">
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
</script>

<template>
  <div class="space-y-6">
    <div class="rounded-xl border border-border bg-card p-6">
      <h2 class="text-lg font-semibold text-foreground mb-4">Keyboard Shortcuts</h2>
      <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
        <div
          v-for="shortcut in shortcuts"
          :key="shortcut.key"
          class="flex items-center justify-between rounded-lg bg-background/50 px-4 py-3"
        >
          <span class="text-sm text-muted-foreground">{{ shortcut.description }}</span>
          <kbd class="inline-flex items-center justify-center min-w-[2.5rem] h-8 px-2.5 rounded-lg border border-border bg-muted text-xs font-mono text-foreground shadow-sm">
            {{ shortcut.key }}
          </kbd>
        </div>
      </div>
    </div>
  </div>
</template>

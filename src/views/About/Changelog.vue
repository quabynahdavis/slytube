<script setup lang="ts">
const changelog = [
  {
    version: '0.1.0',
    date: '2026-08-11',
    changes: [
      'Initial release',
      'Browse trending and popular videos',
      'Search YouTube content',
      'Watch videos with DASH playback',
      'Manage subscriptions and playlists',
      'Download videos with yt-dlp',
      'SponsorBlock integration',
      'Cross-device sync via sync server',
      'Multi-profile support',
      'Keyboard shortcuts',
    ],
  },
]
</script>

<template>
  <div class="space-y-6">
    <div class="rounded-xl border border-border bg-card p-6">
      <h2 class="text-lg font-semibold text-foreground mb-4">Changelog</h2>
      <div class="space-y-6">
        <div
          v-for="release in changelog"
          :key="release.version"
          class="relative border-l-2 border-primary pl-5"
        >
          <div class="absolute -left-[9px] top-1 size-4 rounded-full bg-primary flex items-center justify-center">
            <svg class="size-2.5 text-primary-foreground" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3">
              <polyline points="20 6 9 17 4 12" />
            </svg>
          </div>
          <div class="flex items-center gap-2 mb-2">
            <span class="text-sm font-semibold text-foreground">v{{ release.version }}</span>
            <span class="text-xs text-muted-foreground">{{ release.date }}</span>
          </div>
          <ul class="space-y-1.5">
            <li
              v-for="change in release.changes"
              :key="change"
              class="text-sm text-muted-foreground flex items-start gap-2"
            >
              <span class="text-primary mt-0.5">•</span>
              {{ change }}
            </li>
          </ul>
        </div>
      </div>
    </div>
  </div>
</template>

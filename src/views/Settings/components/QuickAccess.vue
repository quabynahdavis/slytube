<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { useSettingsStore } from '@/stores/settings'
import { settingsConfig, type SettingItem } from '../config'

const { t } = useI18n()
const settingsStore = useSettingsStore()

interface QuickAccessItem {
  item: SettingItem
  category: { id: string; label: string }
  value: boolean
}

const quickAccessItems = computed<QuickAccessItem[]>(() => {
  const items: QuickAccessItem[] = []
  for (const category of settingsConfig) {
    for (const section of category.sections) {
      for (const item of section.items) {
        if (settingsStore.pinnedQuickAccess.includes(item.key) && item.type === 'toggle') {
          items.push({
            item,
            category: { id: category.id, label: category.label },
            value: (settingsStore as any)[item.key] ?? false,
          })
        }
      }
    }
  }
  return items
})

function toggleSetting(key: string, value: boolean) {
  settingsStore.updateSetting(key as any, value)
}

function unpin(key: string) {
  settingsStore.unpinFromQuickAccess(key)
}

const icons: Record<string, string> = {
  baseTheme: 'M21 12.79A9 9 0 1111.21 3 7 7 0 0021 12.79z',
  enableNotifications: 'M15 17h5l-1.405-1.405A2.032 2.032 0 0118 14.158V11a6.002 6.002 0 00-4-5.659V5a2 2 0 10-4 0v.341C7.67 6.165 6 8.388 6 11v3.159c0 .538-.214 1.055-.595 1.436L4 17h5m6 0v1a3 3 0 11-6 0v-1m6 0H9',
  rememberHistory: 'M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z',
  autoplayVideos: 'M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z',
}
</script>

<template>
  <div v-if="quickAccessItems.length > 0" class="space-y-3">
    <div class="flex items-center justify-between">
      <h2 class="text-sm font-semibold text-foreground">{{ t('settings.quickAccess.title') }}</h2>
      <span class="text-xs text-muted-foreground">{{ t('settings.common.pinHint') }}</span>
    </div>
    <div class="grid grid-cols-2 sm:grid-cols-4 gap-3">
      <div
        v-for="entry in quickAccessItems"
        :key="entry.item.key"
        class="relative flex flex-col items-center gap-2 rounded-xl border border-border bg-card p-4 hover:border-primary/50 transition-all group"
      >
        <button
          class="absolute top-2 right-2 size-5 rounded-full text-muted-foreground hover:text-destructive opacity-0 group-hover:opacity-100 transition-opacity"
          :title="t('settings.quickAccess.unpin')"
          @click="unpin(entry.item.key)"
        >
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="size-3.5">
            <line x1="18" y1="6" x2="6" y2="18" />
            <line x1="6" y1="6" x2="18" y2="18" />
          </svg>
        </button>
        <div class="size-8 rounded-full bg-primary/10 flex items-center justify-center">
          <svg class="size-4 text-primary" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
            <path :d="icons[entry.item.key] || icons.enableNotifications" />
          </svg>
        </div>
        <span class="text-xs font-medium text-foreground text-center leading-tight">{{ t(entry.item.label) }}</span>
        <button
          class="relative inline-flex h-5 w-9 shrink-0 cursor-pointer items-center rounded-full border-2 border-transparent transition-colors"
          :class="entry.value ? 'bg-primary' : 'bg-input'"
          @click="toggleSetting(entry.item.key, !entry.value)"
        >
          <span
            class="pointer-events-none block size-4 rounded-full bg-background shadow-lg ring-0 transition-transform"
            :class="entry.value ? 'translate-x-4' : 'translate-x-0'"
          />
        </button>
      </div>
    </div>
  </div>

  <!-- Empty State -->
  <div
    v-else
    class="rounded-xl border border-dashed border-border bg-muted/30 p-6 text-center"
  >
    <svg class="size-8 text-muted-foreground mx-auto mb-2" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
      <path d="M12 2l3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01L12 2z" />
    </svg>
    <p class="text-sm text-muted-foreground">{{ t('settings.quickAccess.empty') }}</p>
  </div>
</template>

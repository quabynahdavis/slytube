<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { useSettingsStore } from '@/stores/settings'
import { Switch } from '@/components/ui/switch'
import { DEFAULT_SETTINGS } from '@/stores/settings'

const { t } = useI18n()
const settingsStore = useSettingsStore()

const props = defineProps<{
  item: {
    key: string
    label: string
    description: string
    quickAccess?: boolean
  }
}>()

const isPinned = computed(() => settingsStore.isPinned(props.item.key))

const isChanged = computed(() => {
  if (!settingsStore.highlightChangedSettings) return false
  const currentValue = (settingsStore as any)[props.item.key]
  const defaultValue = (DEFAULT_SETTINGS as any)[props.item.key]
  return currentValue !== defaultValue
})

function handleToggle(value: boolean) {
  settingsStore.updateSetting(props.item.key as any, value)
}

function handlePin() {
  settingsStore.togglePinned(props.item.key)
}
</script>

<template>
  <div
    class="flex items-center justify-between gap-4 px-5 py-3.5 group transition-colors"
    :class="isChanged && 'bg-yellow-500/5 border-l-2 border-l-yellow-500'"
  >
    <div class="min-w-0 flex-1">
      <p class="text-sm font-medium text-foreground flex items-center gap-2">
        {{ t(item.label) }}
        <span
          v-if="isChanged"
          class="inline-flex items-center px-1.5 py-0.5 rounded text-[10px] font-medium bg-yellow-500/10 text-yellow-600 dark:text-yellow-400 shrink-0"
        >
          Modified
        </span>
      </p>
      <p class="text-xs text-muted-foreground mt-0.5">{{ t(item.description) }}</p>
    </div>
    <div class="flex items-center gap-1 shrink-0">
      <button
        class="size-7 rounded-md flex items-center justify-center text-muted-foreground hover:text-foreground hover:bg-accent transition-all opacity-0 group-hover:opacity-100"
        :class="isPinned && 'opacity-100 text-primary'"
        :title="isPinned ? t('settings.quickAccess.unpin') : t('settings.quickAccess.pin')"
        @click.stop="handlePin"
      >
        <svg class="size-3.5" viewBox="0 0 24 24" :fill="isPinned ? 'currentColor' : 'none'" stroke="currentColor" stroke-width="2">
          <path d="M12 2l3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01L12 2z" />
        </svg>
      </button>
      <Switch
        :checked="(settingsStore as any)[item.key]"
        @update:checked="handleToggle"
      />
    </div>
  </div>
</template>

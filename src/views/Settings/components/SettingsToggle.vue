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
  }
}>()

const isChanged = computed(() => {
  if (!settingsStore.highlightChangedSettings) return false
  const currentValue = (settingsStore as any)[props.item.key]
  const defaultValue = (DEFAULT_SETTINGS as any)[props.item.key]
  return currentValue !== defaultValue
})

function handleToggle(value: boolean) {
  settingsStore.updateSetting(props.item.key as any, value)
}
</script>

<template>
  <div class="flex items-center justify-between gap-4 px-4 py-3 group">
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
    <Switch
      :checked="(settingsStore as any)[item.key]"
      @update:checked="handleToggle"
      class="shrink-0"
    />
  </div>
</template>

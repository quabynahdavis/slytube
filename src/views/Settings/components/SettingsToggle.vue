<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import { useSettingsStore } from '@/stores/settings'
import { Switch } from '@/components/ui/switch'

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

function handleToggle(value: boolean) {
  settingsStore.updateSetting(props.item.key as any, value)
}
</script>

<template>
  <div class="flex items-center justify-between gap-4 px-5 py-3.5">
    <div class="min-w-0 flex-1">
      <p class="text-sm font-medium text-foreground">{{ t(item.label) }}</p>
      <p class="text-xs text-muted-foreground mt-0.5">{{ t(item.description) }}</p>
    </div>
    <Switch
      :checked="(settingsStore as any)[item.key]"
      @update:checked="handleToggle"
      class="shrink-0"
    />
  </div>
</template>

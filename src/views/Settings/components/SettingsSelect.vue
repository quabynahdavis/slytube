<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import { useSettingsStore } from '@/stores/settings'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'

const { t } = useI18n()
const settingsStore = useSettingsStore()

const props = defineProps<{
  item: {
    key: string
    label: string
    description: string
    options?: { value: string; label: string }[]
  }
}>()

function handleChange(value: string | null) {
  if (value !== null) {
    settingsStore.updateSetting(props.item.key as any, value)
  }
}
</script>

<template>
  <div class="flex items-center justify-between gap-4 px-5 py-3.5">
    <div class="min-w-0 flex-1">
      <p class="text-sm font-medium text-foreground">{{ t(item.label) }}</p>
      <p class="text-xs text-muted-foreground mt-0.5">{{ t(item.description) }}</p>
    </div>
    <Select :model-value="(settingsStore as any)[item.key]" @update:model-value="handleChange as any">
      <SelectTrigger class="w-[160px] shrink-0">
        <SelectValue />
      </SelectTrigger>
      <SelectContent>
        <SelectItem v-for="opt in item.options" :key="opt.value" :value="opt.value">
          {{ opt.label }}
        </SelectItem>
      </SelectContent>
    </Select>
  </div>
</template>

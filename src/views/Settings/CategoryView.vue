<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { settingsConfig, type SettingsCategory } from './config'
import type { SettingsState } from '@/stores/settings'
import SettingsSection from './components/SettingsSection.vue'
import SettingsToggle from './components/SettingsToggle.vue'
import SettingsSelect from './components/SettingsSelect.vue'
import SettingsAccordion from './components/SettingsAccordion.vue'
import SettingsLink from './components/SettingsLink.vue'
import { useSettingsStore } from '@/stores/settings'

const { t } = useI18n()
const settingsStore = useSettingsStore()

const props = defineProps<{
  categoryId: string
}>()

const category = computed(() => settingsConfig.find(c => c.id === props.categoryId)!)

function extractKeys(cat: SettingsCategory): (keyof SettingsState)[] {
  const keys: (keyof SettingsState)[] = []
  for (const section of cat.sections) {
    for (const item of section.items) {
      keys.push(item.key as keyof SettingsState)
      if (item.children) {
        for (const child of item.children) {
          keys.push(child.key as keyof SettingsState)
        }
      }
    }
  }
  return keys
}

const categoryKeys = computed(() => extractKeys(category.value))

const changedCount = computed(() =>
  categoryKeys.value.filter(key => settingsStore.isSettingChanged(key)).length
)

async function resetCategory() {
  await settingsStore.resetCategoryToDefaults(categoryKeys.value)
}
</script>

<template>
  <div class="max-w-3xl mx-auto px-6 py-8">
    <!-- Header -->
    <div class="flex items-center justify-between mb-8">
      <div>
        <h2 class="text-2xl font-bold text-foreground">{{ t(category.label) }}</h2>
        <p class="text-sm text-muted-foreground mt-1">{{ t(category.description) }}</p>
      </div>
      <button
        v-if="changedCount > 0"
        class="inline-flex items-center gap-2 px-3 py-1.5 rounded-md text-xs font-medium text-muted-foreground hover:text-foreground hover:bg-accent transition-colors"
        @click="resetCategory"
      >
        <svg class="size-3.5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M3 12a9 9 0 109-9 9.75 9.75 0 00-6.74 2.74L3 8" />
          <path d="M3 3v5h5" />
        </svg>
        {{ t('settings.resetCategory') }}
        <span class="text-yellow-500">({{ changedCount }})</span>
      </button>
    </div>

    <!-- Sections -->
    <div class="space-y-6">
      <SettingsSection
        v-for="section in category.sections"
        :key="section.id"
        :label="t(section.label)"
        :description="t(section.description)"
      >
        <template v-for="item in section.items" :key="item.key">
          <SettingsToggle v-if="item.type === 'toggle'" :item="item" />
          <SettingsSelect v-else-if="item.type === 'select'" :item="item" />
          <SettingsAccordion v-else-if="item.type === 'accordion'" :item="item" />
          <SettingsLink v-else-if="item.type === 'link' || item.crossLink" :item="item" />
          <div v-else-if="item.type === 'action'" class="px-4 py-3">
            <button
              class="inline-flex items-center gap-2 text-sm font-medium text-primary hover:underline"
              @click="item.key === 'exportData' || item.key === 'deleteData' ? null : null"
            >
              {{ t(item.label) }}
            </button>
            <p class="text-xs text-muted-foreground mt-0.5">{{ t(item.description) }}</p>
          </div>
        </template>
      </SettingsSection>
    </div>
  </div>
</template>

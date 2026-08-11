<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import { computed } from 'vue'
import { useRouter } from 'vue-router'
import { settingsConfig, type SettingsCategory } from '../config'
import type { SettingsState } from '@/stores/settings'
import SettingsHeader from './SettingsHeader.vue'
import SettingsSection from './SettingsSection.vue'
import SettingsToggle from './SettingsToggle.vue'
import SettingsSelect from './SettingsSelect.vue'
import SettingsAccordion from './SettingsAccordion.vue'
import SettingsLink from './SettingsLink.vue'

const { t } = useI18n()
const router = useRouter()

const props = defineProps<{
  categoryId: string
}>()

const category = computed(() => settingsConfig.find((c: typeof settingsConfig[number]) => c.id === props.categoryId)!)

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

function goBack() {
  router.push('/settings')
}
</script>

<template>
  <div class="min-h-screen bg-background">
    <SettingsHeader :category-label="t(category.label)" :category-keys="categoryKeys" @back="goBack" />

    <div class="max-w-3xl mx-auto px-4 py-6 space-y-4">
      <SettingsSection
        v-for="section in category.sections"
        :key="section.id"
        :label="t(section.label)"
        :description="t(section.description)"
      >
        <template v-for="item in section.items" :key="item.key">
          <div :id="`setting-${item.key}`" class="scroll-mt-20">
            <SettingsToggle
              v-if="item.type === 'toggle'"
              :item="item"
            />
            <SettingsSelect
              v-else-if="item.type === 'select'"
              :item="item"
            />
            <SettingsAccordion
              v-else-if="item.type === 'accordion'"
              :item="item"
            />
            <SettingsLink
              v-else-if="item.type === 'link' || item.crossLink"
              :item="item"
            />
            <div v-else-if="item.type === 'action'" class="px-5 py-3">
              <button
                class="inline-flex items-center gap-2 text-sm font-medium text-primary hover:bg-primary/5 px-3 py-1.5 rounded-md transition-colors"
                @click="item.key === 'exportData' ? router.push('/settings/account') : item.key === 'deleteData' ? router.push('/settings/privacy') : null"
              >
                <svg class="size-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                  <path d="M5 12h14M12 5l7 7-7 7" />
                </svg>
                {{ t(item.label) }}
              </button>
              <p class="text-xs text-muted-foreground mt-1 ml-1">{{ t(item.description) }}</p>
            </div>
          </div>
        </template>
      </SettingsSection>
    </div>
  </div>
</template>

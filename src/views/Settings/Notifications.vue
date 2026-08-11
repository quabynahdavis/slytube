<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import { computed } from 'vue'
import { useRouter } from 'vue-router'
import { settingsConfig } from './config'
import SettingsHeader from './components/SettingsHeader.vue'
import SettingsSection from './components/SettingsSection.vue'
import SettingsToggle from './components/SettingsToggle.vue'
import SettingsSelect from './components/SettingsSelect.vue'
import SettingsAccordion from './components/SettingsAccordion.vue'
import SettingsLink from './components/SettingsLink.vue'

const { t } = useI18n()
const router = useRouter()

const category = computed(() => settingsConfig.find(c => c.id === 'notifications')!)

function goBack() {
  router.push('/settings')
}
</script>

<template>
  <div class="min-h-screen bg-background">
    <SettingsHeader :category-label="t(category.label)" @back="goBack" />

    <div class="max-w-3xl mx-auto px-4 py-6 space-y-4">
      <SettingsSection
        v-for="section in category.sections"
        :key="section.id"
        :label="t(section.label)"
        :description="t(section.description)"
      >
        <template v-for="item in section.items" :key="item.key">
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
        </template>
      </SettingsSection>
    </div>
  </div>
</template>

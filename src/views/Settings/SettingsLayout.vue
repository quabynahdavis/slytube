<script setup lang="ts">
import { computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { useSettingsStore } from '@/stores/settings'
import { settingsConfig, type SettingsCategory } from './config'
import SettingsSearch from './components/SettingsSearch.vue'
import ImportExport from './components/ImportExport.vue'
import {
  PhUserCircle,
  PhPalette,
  PhBell,
  PhShield,
  PhPlay,
} from '@phosphor-icons/vue'

const { t } = useI18n()
const route = useRoute()
const router = useRouter()
const settingsStore = useSettingsStore()

type PhosphorIcon = typeof PhUserCircle

const categoryIcons: Record<string, PhosphorIcon> = {
  user: PhUserCircle,
  palette: PhPalette,
  bell: PhBell,
  shield: PhShield,
  play: PhPlay,
}

const categories = computed(() =>
  settingsConfig.map((cat: SettingsCategory) => ({
    ...cat,
    iconComponent: categoryIcons[cat.icon] || PhUserCircle,
  }))
)

const activeCategory = computed(() =>
  categories.value.find(c => route.path.includes(c.id))
)

const changedCount = computed(() => settingsStore.changedSettingsCount)

function navigateTo(category: SettingsCategory) {
  router.push(category.route)
}
</script>

<template>
  <div class="flex h-full">
    <!-- Sidebar -->
    <aside class="w-60 shrink-0 border-r border-border bg-card flex flex-col">
      <!-- Header -->
      <div class="p-4 border-b border-border">
        <h1 class="text-lg font-bold text-foreground">{{ t('settings.title') }}</h1>
      </div>

      <!-- Search -->
      <div class="p-3 border-b border-border">
        <SettingsSearch />
      </div>

      <!-- Navigation -->
      <nav class="flex-1 overflow-y-auto py-2">
        <ul class="space-y-0.5 px-2">
          <li v-for="cat in categories" :key="cat.id">
            <button
              class="w-full flex items-center gap-3 rounded-lg px-3 py-2.5 text-sm font-medium transition-colors text-left"
              :class="activeCategory?.id === cat.id
                ? 'bg-primary/10 text-primary'
                : 'text-muted-foreground hover:bg-accent hover:text-accent-foreground'"
              @click="navigateTo(cat)"
            >
              <component :is="cat.iconComponent" :size="18" weight="duotone" class="shrink-0" />
              <span class="truncate">{{ t(cat.label) }}</span>
            </button>
          </li>
        </ul>
      </nav>

      <!-- Footer Actions -->
      <div class="p-3 border-t border-border space-y-2">
        <div v-if="changedCount > 0" class="px-2 py-1.5 rounded-md bg-yellow-500/10 text-xs text-yellow-600 dark:text-yellow-400">
          {{ t('settings.changedCount', { count: changedCount }) }}
        </div>
        <ImportExport />
      </div>
    </aside>

    <!-- Content -->
    <div class="flex-1 overflow-y-auto">
      <router-view />
    </div>
  </div>
</template>

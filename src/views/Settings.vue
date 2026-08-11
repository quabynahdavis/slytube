<script setup lang="ts">
import { ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { cn } from '@/lib/utils'
import { useSettingsStore } from '@/stores/settings'
import { useTheme } from '../composables/useTheme'
import { useToast } from '../composables/useToast'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'

const { t } = useI18n()
const settingsStore = useSettingsStore()
const { theme: currentTheme, setTheme } = useTheme()
const toast = useToast()

const activeTab = ref('general')

const settingsTabs = [
  { id: 'general', labelKey: 'settings.tabs.general' },
  { id: 'player', labelKey: 'settings.tabs.player' },
  { id: 'downloads', labelKey: 'settings.tabs.downloads' },
  { id: 'subscription', labelKey: 'settings.tabs.subscription' },
  { id: 'privacy', labelKey: 'settings.tabs.privacy' },
  { id: 'performance', labelKey: 'settings.tabs.performance' },
  { id: 'advanced', labelKey: 'settings.tabs.advanced' },
]
const isLoading = ref(false)
const saveSuccess = ref(false)


// Import settings from JSON
function importSettings() {
  const input = document.createElement('input')
  input.type = 'file'
  input.accept = '.json'
  input.onchange = async (e) => {
    const file = (e.target as HTMLInputElement).files?.[0]
    if (!file) return
    try {
      const text = await file.text()
      const data = JSON.parse(text)
      settingsStore.importSettings(data)
      toast.success(t('settings.importSuccess'))
    } catch {
      toast.error(t('settings.importError'))
    }
  }
  input.click()
}

// Export settings to JSON
function exportSettings() {
  try {
    const settings = settingsStore.exportSettings()
    const blob = new Blob([JSON.stringify(settings, null, 2)], { type: 'application/json' })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = 'slytube-settings.json'
    a.click()
    URL.revokeObjectURL(url)
    toast.success(t('settings.exportSuccess'))
  } catch {
    toast.error(t('settings.exportError'))
  }
}

const expandedSections = ref<Set<string>>(new Set(['general']))

function toggleSection(section: string) {
  if (expandedSections.value.has(section)) {
    expandedSections.value.delete(section)
  } else {
    expandedSections.value.add(section)
  }
}

function isSectionExpanded(section: string): boolean {
  return expandedSections.value.has(section)
}

async function saveSettings() {
  isLoading.value = true
  saveSuccess.value = false
  try {
    settingsStore.loadSettings()
    saveSuccess.value = true
    setTimeout(() => { saveSuccess.value = false }, 3000)
  } finally {
    isLoading.value = false
  }
}

function resetAllSettings() {
  // Reset theme to system
  setTheme('system')
  // Reset settings store
  settingsStore.loadSettings()
}

// Sync theme changes to settings store
watch(currentTheme, (newTheme) => {
  settingsStore.updateSetting('baseTheme', newTheme)
})
</script>

<template>
  <div class="flex max-w-6xl mx-auto">
    <!-- Settings Sidebar -->
    <aside class="w-56 shrink-0 border-r border-border p-4 space-y-1">
      <button
        v-for="tab in settingsTabs"
        :key="tab.id"
        :class="[
          'w-full text-left px-3 py-2 rounded-lg text-sm font-medium transition-colors',
          activeTab === tab.id
            ? 'bg-primary text-primary-foreground'
            : 'text-muted-foreground hover:bg-accent hover:text-accent-foreground'
        ]"
        @click="activeTab = tab.id"
      >
        {{ t(tab.labelKey) }}
      </button>
    </aside>

    <!-- Settings Content -->
    <div class="flex-1 p-6">
      <!-- Header -->
      <div class="mb-6">
        <h1 class="text-2xl font-bold text-foreground">{{ t('settings.title') }}</h1>
        <p class="text-sm text-muted-foreground mt-1">{{ t('settings.description') }}</p>
        <div class="flex gap-2 mt-3">
          <button @click="importSettings()" class="px-3 py-1.5 text-sm bg-secondary text-secondary-foreground rounded-lg hover:bg-secondary/80">Import</button>
          <button @click="exportSettings()" class="px-3 py-1.5 text-sm bg-secondary text-secondary-foreground rounded-lg hover:bg-secondary/80">Export</button>
          <button @click="resetAllSettings()" class="px-3 py-1.5 text-sm bg-destructive/10 text-destructive rounded-lg hover:bg-destructive/20">Reset</button>
        </div>
      </div>

      <!-- General Section -->
      <section v-show="activeTab === 'general'" class="space-y-6">
        <div class="rounded-lg border border-border bg-card">
          <button class="flex w-full items-center justify-between p-4 text-left font-semibold text-foreground" @click="toggleSection('general-theme')">
            <span>{{ t('settings.theme.title') }}</span>
            <svg :class="cn('size-4 transition-transform', isSectionExpanded('general-theme') && 'rotate-180')" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
              <path stroke-linecap="round" stroke-linejoin="round" d="M19 9l-7 7-7-7" />
            </svg>
          </button>
          <div v-show="isSectionExpanded('general-theme')" class="border-t border-border p-4 space-y-4">
            <div class="flex items-center justify-between">
              <div>
                <p class="text-sm font-medium text-foreground">{{ t('settings.theme.baseTheme') }}</p>
                <p class="text-xs text-muted-foreground">{{ t('settings.theme.baseThemeDescription') }}</p>
              </div>
              <Select :model-value="currentTheme" @update:model-value="setTheme($event as any)">
                <SelectTrigger class="w-[180px]">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="system">{{ t('settings.theme.system') }}</SelectItem>
                  <SelectItem value="light">{{ t('settings.theme.light') }}</SelectItem>
                  <SelectItem value="dark">{{ t('settings.theme.dark') }}</SelectItem>
                </SelectContent>
              </Select>
            </div>
          </div>
        </div>
      </section>

      <!-- Player Section -->
      <section v-show="activeTab === 'player'" class="space-y-6">
        <div class="rounded-lg border border-border bg-card p-4">
          <p class="text-sm text-muted-foreground">Player settings will appear here.</p>
        </div>
      </section>

      <!-- Downloads Section -->
      <section v-show="activeTab === 'downloads'" class="space-y-6">
        <div class="rounded-lg border border-border bg-card p-4">
          <p class="text-sm text-muted-foreground">Download settings will appear here.</p>
        </div>
      </section>

      <!-- Subscription Section -->
      <section v-show="activeTab === 'subscription'" class="space-y-6">
        <div class="rounded-lg border border-border bg-card p-4">
          <p class="text-sm text-muted-foreground">Subscription settings will appear here.</p>
        </div>
      </section>

      <!-- Privacy Section -->
      <section v-show="activeTab === 'privacy'" class="space-y-6">
        <div class="rounded-lg border border-border bg-card p-4">
          <p class="text-sm text-muted-foreground">Privacy settings will appear here.</p>
        </div>
      </section>

      <!-- Performance Section -->
      <section v-show="activeTab === 'performance'" class="space-y-6">
        <div class="rounded-lg border border-border bg-card p-4">
          <p class="text-sm text-muted-foreground">Performance settings will appear here.</p>
        </div>
      </section>

      <!-- Advanced Section -->
      <section v-show="activeTab === 'advanced'" class="space-y-6">
        <div class="rounded-lg border border-border bg-card p-4">
          <p class="text-sm text-muted-foreground">Advanced settings will appear here. Sync server support is coming soon.</p>
        </div>
      </section>

      <!-- Action Buttons -->
      <div class="mt-6 flex items-center gap-3">
        <button class="inline-flex items-center justify-center h-9 rounded-md bg-primary px-4 text-sm font-medium text-primary-foreground shadow hover:bg-primary/90 transition-colors" @click="saveSettings">
          {{ t('settings.save') }}
        </button>
        <button class="inline-flex items-center justify-center h-9 rounded-md border border-input bg-background px-4 text-sm font-medium text-foreground shadow-sm hover:bg-accent hover:text-accent-foreground transition-colors" @click="resetAllSettings">
          {{ t('actions.reset') }}
        </button>
        <span v-if="saveSuccess" class="text-sm text-green-500">{{ t('settings.saved') }}</span>
      </div>
    </div>
  </div>
</template>

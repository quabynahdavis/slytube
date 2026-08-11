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
  <div class="container mx-auto max-w-5xl px-4 py-6">
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

    <!-- Settings Content -->
    <div class="space-y-6">
        <!-- General Section -->
        <section class="space-y-6">
          <div class="rounded-lg border border-border bg-card">
            <button
              class="flex w-full items-center justify-between p-4 text-left font-semibold text-foreground"
              @click="toggleSection('general-theme')"
            >
              <span>{{ t('settings.theme.title') }}</span>
              <svg
                :class="cn('size-4 transition-transform', isSectionExpanded('general-theme') && 'rotate-180')"
                viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"
              >
                <polyline points="6 9 12 15 18 9" />
              </svg>
            </button>
            <div v-show="isSectionExpanded('general-theme')" class="border-t border-border p-4 space-y-4">
              <div class="flex items-center justify-between">
                <div>
                  <p class="text-sm font-medium text-foreground">{{ t('settings.theme.baseTheme') }}</p>
                  <p class="text-xs text-muted-foreground">{{ t('settings.theme.baseThemeDescription') }}</p>
                </div>
                <div class="flex gap-2">
                  <button
                    :class="cn(
                      'h-9 rounded-md px-3 text-sm font-medium transition-colors',
                      currentTheme === 'system' ? 'bg-primary text-primary-foreground' : 'border border-border text-muted-foreground hover:bg-accent'
                    )"
                    @click="setTheme('system')"
                  >
                    {{ t('settings.theme.system') }}
                  </button>
                  <button
                    :class="cn(
                      'h-9 rounded-md px-3 text-sm font-medium transition-colors',
                      currentTheme === 'light' ? 'bg-primary text-primary-foreground' : 'border border-border text-muted-foreground hover:bg-accent'
                    )"
                    @click="setTheme('light')"
                  >
                    {{ t('settings.theme.light') }}
                  </button>
                  <button
                    :class="cn(
                      'h-9 rounded-md px-3 text-sm font-medium transition-colors',
                      currentTheme === 'dark' ? 'bg-primary text-primary-foreground' : 'border border-border text-muted-foreground hover:bg-accent'
                    )"
                    @click="setTheme('dark')"
                  >
                    {{ t('settings.theme.dark') }}
                  </button>
                </div>
              </div>
              <div class="flex items-center justify-between">
                <div>
                  <p class="text-sm font-medium text-foreground">{{ t('settings.sidebar.expand') }}</p>
                  <p class="text-xs text-muted-foreground">{{ t('settings.sidebar.expandDescription') }}</p>
                </div>
                <button
                  :class="cn(
                    'relative inline-flex h-6 w-11 items-center rounded-full transition-colors',
                    settingsStore.expandSideBar ? 'bg-primary' : 'bg-muted'
                  )"
                  @click="settingsStore.updateSetting('expandSideBar', !settingsStore.expandSideBar)"
                >
                  <span
                    :class="cn(
                      'inline-block size-4 rounded-full bg-white transition-transform',
                      settingsStore.expandSideBar ? 'translate-x-6' : 'translate-x-1'
                    )"
                  />
                </button>
              </div>
              <div class="flex items-center justify-between">
                <div>
                  <p class="text-sm font-medium text-foreground">{{ t('settings.landingPage.title') }}</p>
                  <p class="text-xs text-muted-foreground">{{ t('settings.landingPage.description') }}</p>
                </div>
                <Select v-model="settingsStore.landingPage">
                  <SelectTrigger class="w-[180px]">
                    <SelectValue placeholder="Select page..." />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="subscriptions">{{ t('nav.subscriptions') }}</SelectItem>
                    <SelectItem value="trending">{{ t('nav.trending') }}</SelectItem>
                    <SelectItem value="popular">{{ t('home.popular') }}</SelectItem>
                    <SelectItem value="search">{{ t('nav.search') }}</SelectItem>
                  </SelectContent>
                </Select>
              </div>
            </div>
          </div>

          <div class="rounded-lg border border-border bg-card">
            <button
              class="flex w-full items-center justify-between p-4 text-left font-semibold text-foreground"
              @click="toggleSection('general-region')"
            >
              <span>{{ t('settings.region.title') }}</span>
              <svg
                :class="cn('size-4 transition-transform', isSectionExpanded('general-region') && 'rotate-180')"
                viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"
              >
                <polyline points="6 9 12 15 18 9" />
              </svg>
            </button>
            <div v-show="isSectionExpanded('general-region')" class="border-t border-border p-4 space-y-4">
              <div class="flex items-center justify-between">
                <div>
                  <p class="text-sm font-medium text-foreground">{{ t('settings.region.region') }}</p>
                  <p class="text-xs text-muted-foreground">{{ t('settings.region.regionDescription') }}</p>
                </div>
                <Select v-model="settingsStore.region">
                  <SelectTrigger class="w-[180px]">
                    <SelectValue placeholder="Select region..." />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="US">United States</SelectItem>
                    <SelectItem value="GB">United Kingdom</SelectItem>
                    <SelectItem value="DE">Germany</SelectItem>
                    <SelectItem value="FR">France</SelectItem>
                    <SelectItem value="JP">Japan</SelectItem>
                  </SelectContent>
                </Select>
              </div>
              <div class="flex items-center justify-between">
                <div>
                  <p class="text-sm font-medium text-foreground">{{ t('settings.region.locale') }}</p>
                  <p class="text-xs text-muted-foreground">{{ t('settings.region.localeDescription') }}</p>
                </div>
                <Select v-model="settingsStore.currentLocale">
                  <SelectTrigger class="w-[180px]">
                    <SelectValue placeholder="Select language..." />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="system">System Default</SelectItem>
                    <SelectItem value="en">English</SelectItem>
                    <SelectItem value="es">Spanish</SelectItem>
                    <SelectItem value="fr">French</SelectItem>
                    <SelectItem value="de">German</SelectItem>
                    <SelectItem value="ja">Japanese</SelectItem>
                  </SelectContent>
                </Select>
              </div>
            </div>
          </div>
        </section>

        <!-- Player Section -->
        <section v-show="activeTab === 'player'" class="space-y-6">
          <div class="rounded-lg border border-border bg-card">
            <button
              class="flex w-full items-center justify-between p-4 text-left font-semibold text-foreground"
              @click="toggleSection('player-playback')"
            >
              <span>{{ t('settings.player.playback') }}</span>
              <svg
                :class="cn('size-4 transition-transform', isSectionExpanded('player-playback') && 'rotate-180')"
                viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"
              >
                <polyline points="6 9 12 15 18 9" />
              </svg>
            </button>
            <div v-show="isSectionExpanded('player-playback')" class="border-t border-border p-4 space-y-4">
              <div class="flex items-center justify-between">
                <div>
                  <p class="text-sm font-medium text-foreground">{{ t('settings.player.autoplay') }}</p>
                  <p class="text-xs text-muted-foreground">{{ t('settings.player.autoplayDescription') }}</p>
                </div>
                <button
                  :class="cn(
                    'relative inline-flex h-6 w-11 items-center rounded-full transition-colors',
                    settingsStore.autoplayVideos ? 'bg-primary' : 'bg-muted'
                  )"
                  @click="settingsStore.updateSetting('autoplayVideos', !settingsStore.autoplayVideos)"
                >
                  <span
                    :class="cn(
                      'inline-block size-4 rounded-full bg-white transition-transform',
                      settingsStore.autoplayVideos ? 'translate-x-6' : 'translate-x-1'
                    )"
                  />
                </button>
              </div>
              <div class="flex items-center justify-between">
                <div>
                  <p class="text-sm font-medium text-foreground">{{ t('settings.player.defaultQuality') }}</p>
                  <p class="text-xs text-muted-foreground">{{ t('settings.player.defaultQualityDescription') }}</p>
                </div>
                <Select v-model="settingsStore.defaultQuality">
                  <SelectTrigger class="w-[180px]">
                    <SelectValue placeholder="Select quality..." />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="auto">Auto</SelectItem>
                    <SelectItem value="144">144p</SelectItem>
                    <SelectItem value="240">240p</SelectItem>
                    <SelectItem value="360">360p</SelectItem>
                    <SelectItem value="480">480p</SelectItem>
                    <SelectItem value="720">720p</SelectItem>
                    <SelectItem value="1080">1080p</SelectItem>
                    <SelectItem value="1440">1440p</SelectItem>
                    <SelectItem value="2160">4K</SelectItem>
                  </SelectContent>
                </Select>
              </div>
              <div class="flex items-center justify-between">
                <div>
                  <p class="text-sm font-medium text-foreground">{{ t('settings.player.defaultPlaybackRate') }}</p>
                  <p class="text-xs text-muted-foreground">{{ t('settings.player.defaultPlaybackRateDescription') }}</p>
                </div>
                <Select v-model="settingsStore.defaultPlayback">
                  <SelectTrigger class="w-[180px]">
                    <SelectValue placeholder="Select speed..." />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem :value="0.25">0.25x</SelectItem>
                    <SelectItem :value="0.5">0.5x</SelectItem>
                    <SelectItem :value="0.75">0.75x</SelectItem>
                    <SelectItem :value="1">1x (Normal)</SelectItem>
                    <SelectItem :value="1.25">1.25x</SelectItem>
                    <SelectItem :value="1.5">1.5x</SelectItem>
                    <SelectItem :value="1.75">1.75x</SelectItem>
                    <SelectItem :value="2">2x</SelectItem>
                  </SelectContent>
                </Select>
              </div>
              <div class="flex items-center justify-between">
                <div>
                  <p class="text-sm font-medium text-foreground">{{ t('settings.player.defaultVolume') }}</p>
                  <p class="text-xs text-muted-foreground">{{ t('settings.player.defaultVolumeDescription') }}</p>
                </div>
                <input
                  type="range"
                  :value="settingsStore.defaultVolume"
                  min="0"
                  max="1"
                  step="0.05"
                  class="w-32"
                  @input="settingsStore.updateSetting('defaultVolume', Number(($event.target as HTMLInputElement).value))"
                />
              </div>
            </div>
          </div>

          <div class="rounded-lg border border-border bg-card">
            <button
              class="flex w-full items-center justify-between p-4 text-left font-semibold text-foreground"
              @click="toggleSection('player-sponsorblock')"
            >
              <span>{{ t('settings.sponsorblock.title') }}</span>
              <svg
                :class="cn('size-4 transition-transform', isSectionExpanded('player-sponsorblock') && 'rotate-180')"
                viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"
              >
                <polyline points="6 9 12 15 18 9" />
              </svg>
            </button>
            <div v-show="isSectionExpanded('player-sponsorblock')" class="border-t border-border p-4 space-y-4">
              <div class="flex items-center justify-between">
                <div>
                  <p class="text-sm font-medium text-foreground">{{ t('settings.sponsorblock.enable') }}</p>
                  <p class="text-xs text-muted-foreground">{{ t('settings.sponsorblock.enableDescription') }}</p>
                </div>
                <button
                  :class="cn(
                    'relative inline-flex h-6 w-11 items-center rounded-full transition-colors',
                    settingsStore.useSponsorBlock ? 'bg-primary' : 'bg-muted'
                  )"
                  @click="settingsStore.updateSetting('useSponsorBlock', !settingsStore.useSponsorBlock)"
                >
                  <span
                    :class="cn(
                      'inline-block size-4 rounded-full bg-white transition-transform',
                      settingsStore.useSponsorBlock ? 'translate-x-6' : 'translate-x-1'
                    )"
                  />
                </button>
              </div>
            </div>
          </div>
        </section>

        <!-- Downloads Section -->
        <section v-show="activeTab === 'downloads'" class="space-y-6">
          <div class="rounded-lg border border-border bg-card">
            <button
              class="flex w-full items-center justify-between p-4 text-left font-semibold text-foreground"
              @click="toggleSection('downloads-general')"
            >
              <span>{{ t('settings.downloads.title') }}</span>
              <svg
                :class="cn('size-4 transition-transform', isSectionExpanded('downloads-general') && 'rotate-180')"
                viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"
              >
                <polyline points="6 9 12 15 18 9" />
              </svg>
            </button>
            <div v-show="isSectionExpanded('downloads-general')" class="border-t border-border p-4 space-y-4">
              <div class="flex items-center justify-between">
                <div>
                  <p class="text-sm font-medium text-foreground">{{ t('settings.downloads.path') }}</p>
                  <p class="text-xs text-muted-foreground">{{ t('settings.downloads.pathDescription') }}</p>
                </div>
                <input
                  v-model="settingsStore.ytDlpDownloadFolderPath"
                  type="text"
                  placeholder="~/Downloads"
                  class="h-9 w-48 rounded-md border border-input bg-background px-3 text-sm"
                />
              </div>
              <div class="flex items-center justify-between">
                <div>
                  <p class="text-sm font-medium text-foreground">{{ t('settings.downloads.defaultFormat') }}</p>
                  <p class="text-xs text-muted-foreground">{{ t('settings.downloads.defaultFormatDescription') }}</p>
                </div>
                <Select v-model="settingsStore.ytDlpSelectedTemplate">
                  <SelectTrigger class="w-[180px]">
                    <SelectValue placeholder="Select format..." />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="video:best">{{ t('downloads.bestVideo') }}</SelectItem>
                    <SelectItem value="video:720">{{ t('downloads.video720') }}</SelectItem>
                    <SelectItem value="video:1080">{{ t('downloads.video1080') }}</SelectItem>
                    <SelectItem value="audio:best">{{ t('downloads.audioOnly') }}</SelectItem>
                  </SelectContent>
                </Select>
              </div>
            </div>
          </div>
        </section>

        <!-- Subscription Section -->
        <section v-show="activeTab === 'subscription'" class="space-y-6">
          <div class="rounded-lg border border-border bg-card">
            <button
              class="flex w-full items-center justify-between p-4 text-left font-semibold text-foreground"
              @click="toggleSection('subscription-general')"
            >
              <span>{{ t('settings.subscription.title') }}</span>
              <svg
                :class="cn('size-4 transition-transform', isSectionExpanded('subscription-general') && 'rotate-180')"
                viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"
              >
                <polyline points="6 9 12 15 18 9" />
              </svg>
            </button>
            <div v-show="isSectionExpanded('subscription-general')" class="border-t border-border p-4 space-y-4">
              <div class="flex items-center justify-between">
                <div>
                  <p class="text-sm font-medium text-foreground">{{ t('settings.subscription.fetchAutomatically') }}</p>
                  <p class="text-xs text-muted-foreground">{{ t('settings.subscription.fetchAutomaticallyDescription') }}</p>
                </div>
                <button
                  :class="cn(
                    'relative inline-flex h-6 w-11 items-center rounded-full transition-colors',
                    settingsStore.fetchSubscriptionsAutomatically ? 'bg-primary' : 'bg-muted'
                  )"
                  @click="settingsStore.updateSetting('fetchSubscriptionsAutomatically', !settingsStore.fetchSubscriptionsAutomatically)"
                >
                  <span
                    :class="cn(
                      'inline-block size-4 rounded-full bg-white transition-transform',
                      settingsStore.fetchSubscriptionsAutomatically ? 'translate-x-6' : 'translate-x-1'
                    )"
                  />
                </button>
              </div>
              <div class="flex items-center justify-between">
                <div>
                  <p class="text-sm font-medium text-foreground">{{ t('settings.subscription.hideWatched') }}</p>
                  <p class="text-xs text-muted-foreground">{{ t('settings.subscription.hideWatchedDescription') }}</p>
                </div>
                <button
                  :class="cn(
                    'relative inline-flex h-6 w-11 items-center rounded-full transition-colors',
                    settingsStore.hideWatchedSubs ? 'bg-primary' : 'bg-muted'
                  )"
                  @click="settingsStore.updateSetting('hideWatchedSubs', !settingsStore.hideWatchedSubs)"
                >
                  <span
                    :class="cn(
                      'inline-block size-4 rounded-full bg-white transition-transform',
                      settingsStore.hideWatchedSubs ? 'translate-x-6' : 'translate-x-1'
                    )"
                  />
                </button>
              </div>
            </div>
          </div>
        </section>

        <!-- Privacy Section -->
        <section v-show="activeTab === 'privacy'" class="space-y-6">
          <div class="rounded-lg border border-border bg-card">
            <button
              class="flex w-full items-center justify-between p-4 text-left font-semibold text-foreground"
              @click="toggleSection('privacy-history')"
            >
              <span>{{ t('settings.privacy.title') }}</span>
              <svg
                :class="cn('size-4 transition-transform', isSectionExpanded('privacy-history') && 'rotate-180')"
                viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"
              >
                <polyline points="6 9 12 15 18 9" />
              </svg>
            </button>
            <div v-show="isSectionExpanded('privacy-history')" class="border-t border-border p-4 space-y-4">
              <div class="flex items-center justify-between">
                <div>
                  <p class="text-sm font-medium text-foreground">{{ t('settings.privacy.rememberHistory') }}</p>
                  <p class="text-xs text-muted-foreground">{{ t('settings.privacy.rememberHistoryDescription') }}</p>
                </div>
                <button
                  :class="cn(
                    'relative inline-flex h-6 w-11 items-center rounded-full transition-colors',
                    settingsStore.rememberHistory ? 'bg-primary' : 'bg-muted'
                  )"
                  @click="settingsStore.updateSetting('rememberHistory', !settingsStore.rememberHistory)"
                >
                  <span
                    :class="cn(
                      'inline-block size-4 rounded-full bg-white transition-transform',
                      settingsStore.rememberHistory ? 'translate-x-6' : 'translate-x-1'
                    )"
                  />
                </button>
              </div>
              <div class="flex items-center justify-between">
                <div>
                  <p class="text-sm font-medium text-foreground">{{ t('settings.privacy.rememberSearchHistory') }}</p>
                  <p class="text-xs text-muted-foreground">{{ t('settings.privacy.rememberSearchHistoryDescription') }}</p>
                </div>
                <button
                  :class="cn(
                    'relative inline-flex h-6 w-11 items-center rounded-full transition-colors',
                    settingsStore.rememberSearchHistory ? 'bg-primary' : 'bg-muted'
                  )"
                  @click="settingsStore.updateSetting('rememberSearchHistory', !settingsStore.rememberSearchHistory)"
                >
                  <span
                    :class="cn(
                      'inline-block size-4 rounded-full bg-white transition-transform',
                      settingsStore.rememberSearchHistory ? 'translate-x-6' : 'translate-x-1'
                    )"
                  />
                </button>
              </div>
            </div>
          </div>
        </section>

        <!-- Performance Section -->
        <section v-show="activeTab === 'performance'" class="space-y-6">
          <div class="rounded-lg border border-border bg-card">
            <button
              class="flex w-full items-center justify-between p-4 text-left font-semibold text-foreground"
              @click="toggleSection('performance-general')"
            >
              <span>{{ t('settings.performance.title') }}</span>
              <svg
                :class="cn('size-4 transition-transform', isSectionExpanded('performance-general') && 'rotate-180')"
                viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"
              >
                <polyline points="6 9 12 15 18 9" />
              </svg>
            </button>
            <div v-show="isSectionExpanded('performance-general')" class="border-t border-border p-4 space-y-4">
              <div class="flex items-center justify-between">
                <div>
                  <p class="text-sm font-medium text-foreground">{{ t('settings.performance.disableSmoothScrolling') }}</p>
                  <p class="text-xs text-muted-foreground">{{ t('settings.performance.disableSmoothScrollingDescription') }}</p>
                </div>
                <button
                  :class="cn(
                    'relative inline-flex h-6 w-11 items-center rounded-full transition-colors',
                    settingsStore.disableSmoothScrolling ? 'bg-primary' : 'bg-muted'
                  )"
                  @click="settingsStore.updateSetting('disableSmoothScrolling', !settingsStore.disableSmoothScrolling)"
                >
                  <span
                    :class="cn(
                      'inline-block size-4 rounded-full bg-white transition-transform',
                      settingsStore.disableSmoothScrolling ? 'translate-x-6' : 'translate-x-1'
                    )"
                  />
                </button>
              </div>
              <div class="flex items-center justify-between">
                <div>
                  <p class="text-sm font-medium text-foreground">{{ t('settings.performance.ambientMode') }}</p>
                  <p class="text-xs text-muted-foreground">{{ t('settings.performance.ambientModeDescription') }}</p>
                </div>
                <button
                  :class="cn(
                    'relative inline-flex h-6 w-11 items-center rounded-full transition-colors',
                    settingsStore.ambientMode ? 'bg-primary' : 'bg-muted'
                  )"
                  @click="settingsStore.updateSetting('ambientMode', !settingsStore.ambientMode)"
                >
                  <span
                    :class="cn(
                      'inline-block size-4 rounded-full bg-white transition-transform',
                      settingsStore.ambientMode ? 'translate-x-6' : 'translate-x-1'
                    )"
                  />
                </button>
              </div>
            </div>
          </div>
        </section>

        <!-- Advanced Section -->
        <section v-show="activeTab === 'advanced'" class="space-y-6">
          <div class="rounded-lg border border-border bg-card">
            <button
              class="flex w-full items-center justify-between p-4 text-left font-semibold text-foreground"
              @click="toggleSection('advanced-backend')"
            >
              <span>{{ t('settings.advanced.backend') }}</span>
              <svg
                :class="cn('size-4 transition-transform', isSectionExpanded('advanced-backend') && 'rotate-180')"
                viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"
              >
                <polyline points="6 9 12 15 18 9" />
              </svg>
            </button>
            <div v-show="isSectionExpanded('advanced-backend')" class="border-t border-border p-4 space-y-4">
              <div class="flex items-center justify-between">
                <div>
                  <p class="text-sm font-medium text-foreground">{{ t('settings.advanced.backendPreference') }}</p>
                  <p class="text-xs text-muted-foreground">{{ t('settings.advanced.backendPreferenceDescription') }}</p>
                </div>
                <Select v-model="settingsStore.backendPreference">
                  <SelectTrigger class="w-[180px]">
                    <SelectValue placeholder="Select backend..." />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="invidious">Invidious</SelectItem>
                    <SelectItem value="local">Local API</SelectItem>
                    <SelectItem value="piped">Piped</SelectItem>
                  </SelectContent>
                </Select>
              </div>
              <div class="flex items-center justify-between">
                <div>
                  <p class="text-sm font-medium text-foreground">{{ t('settings.advanced.invidiousInstance') }}</p>
                  <p class="text-xs text-muted-foreground">{{ t('settings.advanced.invidiousInstanceDescription') }}</p>
                </div>
                <input
                  v-model="settingsStore.defaultInvidiousInstance"
                  type="text"
                  placeholder="https://invidious.example.com"
                  class="h-9 w-64 rounded-md border border-input bg-background px-3 text-sm"
                />
              </div>
              <div class="flex items-center justify-between">
                <div>
                  <p class="text-sm font-medium text-foreground">{{ t('settings.advanced.backendFallback') }}</p>
                  <p class="text-xs text-muted-foreground">{{ t('settings.advanced.backendFallbackDescription') }}</p>
                </div>
                <button
                  :class="cn(
                    'relative inline-flex h-6 w-11 items-center rounded-full transition-colors',
                    settingsStore.backendFallback ? 'bg-primary' : 'bg-muted'
                  )"
                  @click="settingsStore.updateSetting('backendFallback', !settingsStore.backendFallback)"
                >
                  <span
                    :class="cn(
                      'inline-block size-4 rounded-full bg-white transition-transform',
                      settingsStore.backendFallback ? 'translate-x-6' : 'translate-x-1'
                    )"
                  />
                </button>
              </div>
            </div>
          </div>

          <div class="rounded-lg border border-border bg-card">
            <button
              class="flex w-full items-center justify-between p-4 text-left font-semibold text-foreground"
              @click="toggleSection('advanced-sync')"
            >
              <span>{{ t('settings.sync.title') }}</span>
              <span class="inline-flex items-center rounded-full bg-yellow-500/10 px-2.5 py-0.5 text-xs font-medium text-yellow-600 dark:text-yellow-400">
                {{ t('settings.sync.comingSoon') }}
              </span>
              <svg
                :class="cn('size-4 transition-transform', isSectionExpanded('advanced-sync') && 'rotate-180')"
                viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"
              >
                <polyline points="6 9 12 15 18 9" />
              </svg>
            </button>
            <div v-show="isSectionExpanded('advanced-sync')" class="border-t border-border p-4 space-y-4">
              <!-- Coming Soon Notice -->
              <div class="flex items-center gap-2 rounded-md bg-yellow-500/10 p-3 text-sm text-yellow-600 dark:text-yellow-400">
                <svg class="size-4 shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                  <circle cx="12" cy="12" r="10" />
                  <line x1="12" y1="8" x2="12" y2="12" />
                  <line x1="12" y1="16" x2="12.01" y2="16" />
                </svg>
                <span>{{ t('settings.sync.comingSoonNote') }}</span>
              </div>
              <div class="flex items-center justify-between opacity-50">
                <div>
                  <p class="text-sm font-medium text-foreground">{{ t('settings.sync.enable') }}</p>
                  <p class="text-xs text-muted-foreground">{{ t('settings.sync.enableDescription') }}</p>
                </div>
                <button
                  :class="cn(
                    'relative inline-flex h-6 w-11 items-center rounded-full transition-colors',
                    settingsStore.syncServerEnabled ? 'bg-primary' : 'bg-muted'
                  )"
                  disabled
                  :title="t('settings.sync.comingSoonNote')"
                >
                  <span
                    :class="cn(
                      'inline-block size-4 rounded-full bg-white transition-transform',
                      settingsStore.syncServerEnabled ? 'translate-x-6' : 'translate-x-1'
                    )"
                  />
                </button>
              </div>
              <div class="flex items-center justify-between opacity-50">
                <div>
                  <p class="text-sm font-medium text-foreground">{{ t('settings.sync.serverUrl') }}</p>
                  <p class="text-xs text-muted-foreground">{{ t('settings.sync.serverUrlDescription') }}</p>
                </div>
                <input
                  v-model="settingsStore.syncServerUrl"
                  type="text"
                  placeholder="https://sync.example.com"
                  disabled
                  class="h-9 w-64 rounded-md border border-input bg-background px-3 text-sm disabled:opacity-50"
                />
              </div>
            </div>
          </div>
        </section>

        <!-- Action Buttons -->
        <div class="mt-6 flex items-center gap-3">
          <button
            :disabled="isLoading"
            class="inline-flex items-center justify-center h-9 rounded-md bg-primary px-4 text-sm font-medium text-primary-foreground shadow hover:bg-primary/90 disabled:opacity-50 transition-colors"
            @click="saveSettings"
          >
            <svg
              v-if="isLoading"
              class="mr-2 size-4 animate-spin"
              viewBox="0 0 24 24"
              fill="none"
            >
              <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" />
              <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z" />
            </svg>
            {{ isLoading ? t('settings.saving') : t('settings.save') }}
          </button>
          <button
            class="inline-flex items-center justify-center h-9 rounded-md border border-input bg-background px-4 text-sm font-medium text-foreground shadow-sm hover:bg-accent hover:text-accent-foreground transition-colors"
            @click="resetAllSettings"
          >
            {{ t('actions.reset') }}
          </button>
          <span v-if="saveSuccess" class="text-sm text-green-500">{{ t('settings.saved') }}</span>
        </div>
      </div>
    
  </div>
</template>

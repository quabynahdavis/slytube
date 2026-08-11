<script setup lang="ts">
import { ref, watch } from 'vue'
import { cn } from '@/lib/utils'
import { useSettingsStore } from '@/stores/settings'
import { useTheme } from '../composables/useTheme'

const settingsStore = useSettingsStore()
const { theme: currentTheme, setTheme } = useTheme()

const activeTab = ref('general')
const isLoading = ref(false)
const saveSuccess = ref(false)

const tabs = [
  { id: 'general', label: 'General', icon: 'settings' },
  { id: 'player', label: 'Player', icon: 'play' },
  { id: 'downloads', label: 'Downloads', icon: 'download' },
  { id: 'subscription', label: 'Subscription', icon: 'subscriptions' },
  { id: 'privacy', label: 'Privacy', icon: 'shield' },
  { id: 'performance', label: 'Performance', icon: 'gauge' },
  { id: 'advanced', label: 'Advanced', icon: 'code' },
]

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
      <h1 class="text-2xl font-bold text-foreground">Settings</h1>
      <p class="text-sm text-muted-foreground mt-1">Configure your SlyTube experience</p>
    </div>

    <!-- Settings Tabs -->
    <div class="flex flex-col md:flex-row gap-6">
      <!-- Tab Navigation -->
      <nav class="md:w-48 shrink-0">
        <ul class="flex md:flex-col gap-1 overflow-x-auto md:overflow-visible pb-2 md:pb-0">
          <li v-for="tab in tabs" :key="tab.id">
            <button
              :class="cn(
                'flex items-center gap-2 w-full rounded-lg px-3 py-2 text-sm font-medium transition-colors whitespace-nowrap',
                activeTab === tab.id
                  ? 'bg-primary text-primary-foreground'
                  : 'text-muted-foreground hover:bg-accent hover:text-accent-foreground'
              )"
              @click="activeTab = tab.id"
            >
              <svg class="size-4 shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <circle cx="12" cy="12" r="3" />
                <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z" />
              </svg>
              <span class="hidden md:inline">{{ tab.label }}</span>
            </button>
          </li>
        </ul>
      </nav>

      <!-- Settings Content -->
      <div class="flex-1 min-w-0">
        <!-- General Section -->
        <section v-show="activeTab === 'general'" class="space-y-6">
          <div class="rounded-lg border border-border bg-card">
            <button
              class="flex w-full items-center justify-between p-4 text-left font-semibold text-foreground"
              @click="toggleSection('general-theme')"
            >
              <span>Theme & Appearance</span>
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
                  <p class="text-sm font-medium text-foreground">Base Theme</p>
                  <p class="text-xs text-muted-foreground">Choose between light, dark, or system theme</p>
                </div>
                <div class="flex gap-2">
                  <button
                    :class="cn(
                      'h-9 rounded-md px-3 text-sm font-medium transition-colors',
                      currentTheme === 'system' ? 'bg-primary text-primary-foreground' : 'border border-border text-muted-foreground hover:bg-accent'
                    )"
                    @click="setTheme('system')"
                  >
                    System
                  </button>
                  <button
                    :class="cn(
                      'h-9 rounded-md px-3 text-sm font-medium transition-colors',
                      currentTheme === 'light' ? 'bg-primary text-primary-foreground' : 'border border-border text-muted-foreground hover:bg-accent'
                    )"
                    @click="setTheme('light')"
                  >
                    Light
                  </button>
                  <button
                    :class="cn(
                      'h-9 rounded-md px-3 text-sm font-medium transition-colors',
                      currentTheme === 'dark' ? 'bg-primary text-primary-foreground' : 'border border-border text-muted-foreground hover:bg-accent'
                    )"
                    @click="setTheme('dark')"
                  >
                    Dark
                  </button>
                </div>
              </div>
              <div class="flex items-center justify-between">
                <div>
                  <p class="text-sm font-medium text-foreground">Expand Sidebar</p>
                  <p class="text-xs text-muted-foreground">Show labels in the sidebar navigation</p>
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
                  <p class="text-sm font-medium text-foreground">Landing Page</p>
                  <p class="text-xs text-muted-foreground">Choose what page to show on startup</p>
                </div>
                <select
                  v-model="settingsStore.landingPage"
                  class="h-9 rounded-md border border-input bg-background px-3 text-sm"
                >
                  <option value="subscriptions">Subscriptions</option>
                  <option value="trending">Trending</option>
                  <option value="popular">Popular</option>
                  <option value="search">Search</option>
                </select>
              </div>
            </div>
          </div>

          <div class="rounded-lg border border-border bg-card">
            <button
              class="flex w-full items-center justify-between p-4 text-left font-semibold text-foreground"
              @click="toggleSection('general-region')"
            >
              <span>Region & Language</span>
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
                  <p class="text-sm font-medium text-foreground">Region</p>
                  <p class="text-xs text-muted-foreground">Content region preference</p>
                </div>
                <select
                  v-model="settingsStore.region"
                  class="h-9 rounded-md border border-input bg-background px-3 text-sm"
                >
                  <option value="US">United States</option>
                  <option value="GB">United Kingdom</option>
                  <option value="DE">Germany</option>
                  <option value="FR">France</option>
                  <option value="JP">Japan</option>
                </select>
              </div>
              <div class="flex items-center justify-between">
                <div>
                  <p class="text-sm font-medium text-foreground">Locale</p>
                  <p class="text-xs text-muted-foreground">Interface language</p>
                </div>
                <select
                  v-model="settingsStore.currentLocale"
                  class="h-9 rounded-md border border-input bg-background px-3 text-sm"
                >
                  <option value="system">System Default</option>
                  <option value="en">English</option>
                  <option value="es">Spanish</option>
                  <option value="fr">French</option>
                  <option value="de">German</option>
                  <option value="ja">Japanese</option>
                </select>
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
              <span>Playback</span>
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
                  <p class="text-sm font-medium text-foreground">Autoplay Videos</p>
                  <p class="text-xs text-muted-foreground">Automatically play videos when opened</p>
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
                  <p class="text-sm font-medium text-foreground">Default Quality</p>
                  <p class="text-xs text-muted-foreground">Preferred video quality</p>
                </div>
                <select
                  v-model="settingsStore.defaultQuality"
                  class="h-9 rounded-md border border-input bg-background px-3 text-sm"
                >
                  <option value="auto">Auto</option>
                  <option value="144">144p</option>
                  <option value="240">240p</option>
                  <option value="360">360p</option>
                  <option value="480">480p</option>
                  <option value="720">720p</option>
                  <option value="1080">1080p</option>
                  <option value="1440">1440p</option>
                  <option value="2160">4K</option>
                </select>
              </div>
              <div class="flex items-center justify-between">
                <div>
                  <p class="text-sm font-medium text-foreground">Default Playback Rate</p>
                  <p class="text-xs text-muted-foreground">Default video speed</p>
                </div>
                <select
                  v-model="settingsStore.defaultPlayback"
                  class="h-9 rounded-md border border-input bg-background px-3 text-sm"
                >
                  <option :value="0.25">0.25x</option>
                  <option :value="0.5">0.5x</option>
                  <option :value="0.75">0.75x</option>
                  <option :value="1">1x (Normal)</option>
                  <option :value="1.25">1.25x</option>
                  <option :value="1.5">1.5x</option>
                  <option :value="1.75">1.75x</option>
                  <option :value="2">2x</option>
                </select>
              </div>
              <div class="flex items-center justify-between">
                <div>
                  <p class="text-sm font-medium text-foreground">Default Volume</p>
                  <p class="text-xs text-muted-foreground">Default audio volume level</p>
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
              <span>SponsorBlock</span>
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
                  <p class="text-sm font-medium text-foreground">Enable SponsorBlock</p>
                  <p class="text-xs text-muted-foreground">Skip sponsored segments in videos</p>
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
              <span>Download Settings</span>
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
                  <p class="text-sm font-medium text-foreground">Download Path</p>
                  <p class="text-xs text-muted-foreground">Where downloaded files are saved</p>
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
                  <p class="text-sm font-medium text-foreground">Default Format</p>
                  <p class="text-xs text-muted-foreground">Preferred download format</p>
                </div>
                <select
                  v-model="settingsStore.ytDlpSelectedTemplate"
                  class="h-9 rounded-md border border-input bg-background px-3 text-sm"
                >
                  <option value="video:best">Best Video</option>
                  <option value="video:720">720p Video</option>
                  <option value="video:1080">1080p Video</option>
                  <option value="audio:best">Audio Only</option>
                </select>
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
              <span>Subscription Settings</span>
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
                  <p class="text-sm font-medium text-foreground">Fetch Automatically</p>
                  <p class="text-xs text-muted-foreground">Automatically fetch new subscription videos</p>
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
                  <p class="text-sm font-medium text-foreground">Hide Watched Videos</p>
                  <p class="text-xs text-muted-foreground">Hide videos you've already watched in subscriptions</p>
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
              <span>History & Privacy</span>
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
                  <p class="text-sm font-medium text-foreground">Remember History</p>
                  <p class="text-xs text-muted-foreground">Keep track of watched videos</p>
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
                  <p class="text-sm font-medium text-foreground">Remember Search History</p>
                  <p class="text-xs text-muted-foreground">Save search queries for suggestions</p>
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
              <span>Performance Settings</span>
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
                  <p class="text-sm font-medium text-foreground">Disable Smooth Scrolling</p>
                  <p class="text-xs text-muted-foreground">Improve scrolling performance</p>
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
                  <p class="text-sm font-medium text-foreground">Ambient Mode</p>
                  <p class="text-xs text-muted-foreground">Enable ambient lighting effect around videos</p>
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
              <span>Backend & API</span>
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
                  <p class="text-sm font-medium text-foreground">Backend Preference</p>
                  <p class="text-xs text-muted-foreground">Choose your preferred API backend</p>
                </div>
                <select
                  v-model="settingsStore.backendPreference"
                  class="h-9 rounded-md border border-input bg-background px-3 text-sm"
                >
                  <option value="invidious">Invidious</option>
                  <option value="local">Local API</option>
                  <option value="piped">Piped</option>
                </select>
              </div>
              <div class="flex items-center justify-between">
                <div>
                  <p class="text-sm font-medium text-foreground">Default Invidious Instance</p>
                  <p class="text-xs text-muted-foreground">Your preferred Invidious server</p>
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
                  <p class="text-sm font-medium text-foreground">Backend Fallback</p>
                  <p class="text-xs text-muted-foreground">Use fallback if primary backend fails</p>
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
              <span>Sync Server</span>
              <svg
                :class="cn('size-4 transition-transform', isSectionExpanded('advanced-sync') && 'rotate-180')"
                viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"
              >
                <polyline points="6 9 12 15 18 9" />
              </svg>
            </button>
            <div v-show="isSectionExpanded('advanced-sync')" class="border-t border-border p-4 space-y-4">
              <div class="flex items-center justify-between">
                <div>
                  <p class="text-sm font-medium text-foreground">Enable Sync Server</p>
                  <p class="text-xs text-muted-foreground">Sync data across devices</p>
                </div>
                <button
                  :class="cn(
                    'relative inline-flex h-6 w-11 items-center rounded-full transition-colors',
                    settingsStore.syncServerEnabled ? 'bg-primary' : 'bg-muted'
                  )"
                  @click="settingsStore.updateSetting('syncServerEnabled', !settingsStore.syncServerEnabled)"
                >
                  <span
                    :class="cn(
                      'inline-block size-4 rounded-full bg-white transition-transform',
                      settingsStore.syncServerEnabled ? 'translate-x-6' : 'translate-x-1'
                    )"
                  />
                </button>
              </div>
              <div class="flex items-center justify-between">
                <div>
                  <p class="text-sm font-medium text-foreground">Sync Server URL</p>
                  <p class="text-xs text-muted-foreground">Your sync server address</p>
                </div>
                <input
                  v-model="settingsStore.syncServerUrl"
                  type="text"
                  placeholder="https://sync.example.com"
                  class="h-9 w-64 rounded-md border border-input bg-background px-3 text-sm"
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
            {{ isLoading ? 'Saving...' : 'Save Settings' }}
          </button>
          <button
            class="inline-flex items-center justify-center h-9 rounded-md border border-input bg-background px-4 text-sm font-medium text-foreground shadow-sm hover:bg-accent hover:text-accent-foreground transition-colors"
            @click="resetAllSettings"
          >
            Reset to Defaults
          </button>
          <span v-if="saveSuccess" class="text-sm text-green-500">Settings saved!</span>
        </div>
      </div>
    </div>
  </div>
</template>

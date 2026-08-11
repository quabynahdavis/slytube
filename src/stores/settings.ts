import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'

export interface SponsorBlockCategory {
  color: string
  skip: string
}

export interface SettingsState {
  pinnedQuickAccess: string[]
  alwaysShowScrollbars: boolean
  autoOpenChapters: boolean
  autoplayPlaylists: boolean
  autoplayVideos: boolean
  autoPictureInPictureTriggers: string[]
  scrollMiniPlayerEnabled: boolean
  scrollMiniPlayerSavedRect: string
  avoidTranslation: string
  backendFallback: boolean
  backendPreference: string
  barColor: boolean
  checkForUpdates: boolean
  confirmCloseApp: boolean
  baseTheme: string
  mainColor: string
  secColor: string
  defaultAutoplayInterruptionIntervalHours: number
  defaultCaptionSettings: string
  enableCaptionTranslations: boolean
  preferredCaptionLocale: string
  defaultInterval: number
  defaultPlayback: number
  defaultPlaybackRate: number
  defaultProfile: string
  defaultQuality: string
  defaultSkipInterval: number
  seekIntervalMultiplyByPlaybackRate: boolean
  showPlaybackRateAdjustedTimestamp: boolean
  useCustomShortsPlayer: boolean
  loopShorts: boolean
  defaultViewingMode: string
  defaultVideoFormat: string
  disableSmoothScrolling: boolean
  disableChannelLinks: boolean
  displayVideoPlayButton: boolean
  ambientMode: boolean
  enableWatchStats: boolean
  statsWeekStartsOn: string
  enableSearchSuggestions: boolean
  contextMenuSearchEngines: string
  enableSubtitlesByDefault: boolean
  enterFullscreenOnDisplayRotate: boolean
  externalLinkHandling: string
  externalPlayer: string
  externalPlayerExecutable: string
  externalPlayerIgnoreWarnings: boolean
  externalPlayerIgnoreDefaultArgs: boolean
  externalPlayerCustomArgs: string
  showAddedExternalPlayerCustomArgs: boolean
  videoPlaybackEngine: string
  ytDlpSource: string
  ytDlpChannel: string
  ytDlpPath: string
  ytDlpFfmpegSource: string
  ytDlpFfmpegPath: string
  ytDlpDownloadFolderPath: string
  ytDlpDownloadTemplates: string
  ytDlpSelectedTemplate: string
  expandSideBar: boolean
  hideActiveSubscriptions: boolean
  hideChannelCommunity: boolean
  hideChannelHome: boolean
  hideChannelPlaylists: boolean
  hideChannelReleases: boolean
  hideChannelPodcasts: boolean
  hideChannelCourses: boolean
  hideChannelShorts: boolean
  hideChannelSubscriptions: boolean
  hideCommentLikes: boolean
  hideCommentPhotos: boolean
  hideComments: boolean
  hideEndScreenAnnotations: boolean
  hidePaidPromotion: boolean
  hideFeaturedChannels: boolean
  channelsHidden: string
  forbiddenTitles: string
  showAddedChannelsHidden: boolean
  showAddedForbiddenTitles: boolean
  hideVideoDescription: boolean
  hideLiveChat: boolean
  hideLiveStreams: boolean
  hideHeaderLogo: boolean
  hidePlaylists: boolean
  hidePopularVideos: boolean
  hideProfileSelectorInHeader: boolean
  hideRecommendedVideos: boolean
  hideSearchBar: boolean
  hideSideBarOnWatchPages: boolean
  hideSharingActions: boolean
  hideSubscriptionsVideos: boolean
  hideSubscriptionsShorts: boolean
  hideSubscriptionsLive: boolean
  hideSubscriptionsCommunity: boolean
  hideTrendingVideos: boolean
  hideUnsubscribeButton: boolean
  hideUpcomingPremieres: boolean
  hideVideoLikesAndDislikes: boolean
  hideVideoViews: boolean
  hideWatchedSubs: boolean
  hideUploader: boolean
  unsubscriptionPopupStatus: boolean
  hideLabelsSideBar: boolean
  hideChapters: boolean
  showDistractionFreeTitles: boolean
  showLiveChatTimestamps: boolean
  liveChatFilter: string
  landingPage: string
  newTabPosition: string
  tabCloseFocus: string
  startupBehavior: string
  showTabIcons: boolean
  useVerticalTabBar: boolean
  verticalTabBarWidth: number
  useFixedTabWidth: boolean
  fixedTabWidth: number
  listType: string
  maxVideoPlaybackRate: number
  onlyShowLatestFromChannel: boolean
  onlyShowLatestFromChannelNumber: number
  openDeepLinksInNewWindow: boolean
  playNextVideo: boolean
  playlistReverseStates: Record<string, boolean>
  proxyHostname: string
  proxyPort: string
  proxyUsername: string
  proxyPassword: string
  proxyProtocol: string
  videoIpBlockScriptPath: string
  proxyVideos: boolean
  region: string
  rememberHistory: boolean
  historyRetentionDays: string
  rememberSearchHistory: boolean
  rememberTabNavigationHistory: boolean
  watchedProgressSavingMode: string
  watchedPercentageThreshold: number
  saveVideoHistoryWithLastViewedPlaylist: boolean
  showFamilyFriendlyOnly: boolean
  sponsorBlockShowSkippedToast: boolean
  sponsorBlockSkippedToastDuration: number
  sponsorBlockEnableSubmission: boolean
  sponsorBlockUserId: string
  sponsorBlockGeneratedUserId: string
  sponsorBlockDraftSegmentsByVideoId: Record<string, unknown>
  sponsorBlockChannelWhitelist: string[]
  sponsorBlockUrl: string
  sponsorBlockSponsor: SponsorBlockCategory
  sponsorBlockSelfPromo: SponsorBlockCategory
  sponsorBlockInteraction: SponsorBlockCategory
  sponsorBlockIntro: SponsorBlockCategory
  sponsorBlockOutro: SponsorBlockCategory
  sponsorBlockRecap: SponsorBlockCategory
  sponsorBlockHook: SponsorBlockCategory
  sponsorBlockMusicOffTopic: SponsorBlockCategory
  sponsorBlockFiller: SponsorBlockCategory
  sponsorBlockHighlight: SponsorBlockCategory
  thumbnailPreference: string
  thumbnailSize: string
  uiRoundness: number
  animationSpeed: number
  showThumbnailSizeButtonInHeader: boolean
  showToastTimeoutIndicator: boolean
  toastPosition: string
  extraThumbnailAction: string
  blurThumbnails: boolean
  syncServerEnabled: boolean
  syncServerUrl: string
  syncServerUsername: string
  syncServerToken: string
  syncServerPrivacyMode: string
  syncServerPrivacyKey: string
  syncServerPrivacySalt: string
  syncServerAutoSync: boolean
  syncServerSyncSubscriptions: boolean
  syncServerSyncPlaylists: boolean
  syncServerSyncHistory: boolean
  syncServerSyncPlaybackSpeeds: boolean
  syncServerSyncProfiles: boolean
  syncServerSyncSessions: boolean
  syncServerSyncSettings: boolean
  syncServerSettingsExcluded: string[]
  syncServerLastSyncAt: number
  syncServerSnapshot: string
  useProxy: boolean
  userPlaylistSortOrder: string
  useRssFeeds: boolean
  useReturnYouTubeDislikes: boolean
  returnYouTubeDislikesUrl: string
  useSponsorBlock: boolean
  videoVolumeMouseScroll: boolean
  videoPlaybackRateMouseScroll: boolean
  videoSkipMouseScroll: boolean
  videoPlaybackRateInterval: number
  rememberVolume: boolean
  skipSilence: boolean
  showSkipSilenceButton: boolean
  holdToDoublePlaybackSpeed: boolean
  keyboardShortcuts: string
  rememberPlaybackSpeedPerChannel: boolean
  autoUpdateChannelPlaybackSpeeds: boolean
  channelPlaybackSpeeds: string
  useQuickPlaybackSpeedBar: boolean
  quickPlaybackSpeedBarOptions: string
  rememberVideoQualityPerChannel: boolean
  autoUpdateChannelVideoQualities: boolean
  channelVideoQualities: string
  rememberSubtitlesStatePerChannel: boolean
  autoUpdateChannelSubtitlesStates: boolean
  channelSubtitlesStates: string
  rememberVolumePerChannel: boolean
  autoUpdateChannelVolumes: boolean
  channelVolumes: string
  enableScreenshot: boolean
  screenshotMode: string
  screenshotFormat: string
  screenshotQuality: number
  screenshotFolderPath: string
  screenshotFilenamePattern: string
  settingsSectionSortEnabled: boolean
  highlightChangedSettings: boolean
  fetchSubscriptionsAutomatically: boolean
  showNewSubscriptionFeed: boolean
  showNewSubscriptionFeedIndicators: boolean
  subscriptionFeedAutoRefreshInterval: string
  subscriptionShortsAutoRefreshInterval: string
  subscriptionLiveAutoRefreshInterval: string
  subscriptionPostsAutoRefreshInterval: string
  showProgressBarToast: boolean
  settingsPassword: string
  useDeArrowTitles: boolean
  useDeArrowThumbnails: boolean
  deArrowThumbnailGeneratorUrl: string
  quickBookmarkTargetPlaylistId: string
  generalAutoLoadMorePaginatedItemsEnabled: boolean
  hideToTrayOnMinimize: boolean
  currentLocale: string
  reducedMotion: string
  defaultInvidiousInstance: string
  defaultVolume: number
  uiScale: number
  userPlaylistsSortBy: string
  userHistorySortBy: string
  enableNotifications: boolean
  enablePushNotifications: boolean
  enableEmailNotifications: boolean
}

const DEFAULT_SETTINGS: SettingsState = {
  pinnedQuickAccess: ['baseTheme', 'enableNotifications', 'rememberHistory', 'autoplayVideos'],
  alwaysShowScrollbars: false,
  autoOpenChapters: false,
  autoplayPlaylists: true,
  autoplayVideos: true,
  autoPictureInPictureTriggers: [],
  scrollMiniPlayerEnabled: true,
  scrollMiniPlayerSavedRect: '',
  avoidTranslation: 'disabled',
  backendFallback: false,
  backendPreference: 'invidious',
  barColor: false,
  checkForUpdates: true,
  confirmCloseApp: true,
  baseTheme: 'system',
  mainColor: 'Red',
  secColor: 'Blue',
  defaultAutoplayInterruptionIntervalHours: 3,
  defaultCaptionSettings: '{}',
  enableCaptionTranslations: false,
  preferredCaptionLocale: '',
  defaultInterval: 5,
  defaultPlayback: 1,
  defaultPlaybackRate: 1,
  defaultProfile: 'allChannels',
  defaultQuality: '720',
  defaultSkipInterval: 5,
  seekIntervalMultiplyByPlaybackRate: false,
  showPlaybackRateAdjustedTimestamp: false,
  useCustomShortsPlayer: true,
  loopShorts: true,
  defaultViewingMode: 'default',
  defaultVideoFormat: 'dash',
  disableSmoothScrolling: false,
  disableChannelLinks: false,
  displayVideoPlayButton: false,
  ambientMode: false,
  enableWatchStats: true,
  statsWeekStartsOn: '1',
  enableSearchSuggestions: true,
  contextMenuSearchEngines: '["YouTube","Invidious"]',
  enableSubtitlesByDefault: false,
  enterFullscreenOnDisplayRotate: false,
  externalLinkHandling: '',
  externalPlayer: '',
  externalPlayerExecutable: '',
  externalPlayerIgnoreWarnings: false,
  externalPlayerIgnoreDefaultArgs: false,
  externalPlayerCustomArgs: '[]',
  showAddedExternalPlayerCustomArgs: true,
  videoPlaybackEngine: 'yt-dlp',
  ytDlpSource: 'system',
  ytDlpChannel: 'stable',
  ytDlpPath: '',
  ytDlpFfmpegSource: 'system',
  ytDlpFfmpegPath: '',
  ytDlpDownloadFolderPath: '',
  ytDlpDownloadTemplates: '[]',
  ytDlpSelectedTemplate: 'video:best',
  expandSideBar: false,
  hideActiveSubscriptions: false,
  hideChannelCommunity: false,
  hideChannelHome: false,
  hideChannelPlaylists: false,
  hideChannelReleases: false,
  hideChannelPodcasts: false,
  hideChannelCourses: false,
  hideChannelShorts: false,
  hideChannelSubscriptions: false,
  hideCommentLikes: false,
  hideCommentPhotos: false,
  hideComments: false,
  hideEndScreenAnnotations: false,
  hidePaidPromotion: false,
  hideFeaturedChannels: false,
  channelsHidden: '[]',
  forbiddenTitles: '[]',
  showAddedChannelsHidden: true,
  showAddedForbiddenTitles: true,
  hideVideoDescription: false,
  hideLiveChat: false,
  hideLiveStreams: false,
  hideHeaderLogo: false,
  hidePlaylists: false,
  hidePopularVideos: false,
  hideProfileSelectorInHeader: false,
  hideRecommendedVideos: false,
  hideSearchBar: false,
  hideSideBarOnWatchPages: true,
  hideSharingActions: false,
  hideSubscriptionsVideos: false,
  hideSubscriptionsShorts: false,
  hideSubscriptionsLive: false,
  hideSubscriptionsCommunity: false,
  hideTrendingVideos: false,
  hideUnsubscribeButton: false,
  hideUpcomingPremieres: false,
  hideVideoLikesAndDislikes: false,
  hideVideoViews: false,
  hideWatchedSubs: false,
  hideUploader: false,
  unsubscriptionPopupStatus: false,
  hideLabelsSideBar: false,
  hideChapters: false,
  showDistractionFreeTitles: false,
  showLiveChatTimestamps: false,
  liveChatFilter: 'TOP_CHAT',
  landingPage: 'subscriptions',
  newTabPosition: 'afterCurrent',
  tabCloseFocus: 'previousTab',
  startupBehavior: 'loadLastActiveTab',
  showTabIcons: true,
  useVerticalTabBar: false,
  verticalTabBarWidth: 220,
  useFixedTabWidth: false,
  fixedTabWidth: 220,
  listType: 'grid',
  maxVideoPlaybackRate: 3,
  onlyShowLatestFromChannel: false,
  onlyShowLatestFromChannelNumber: 1,
  openDeepLinksInNewWindow: false,
  playNextVideo: false,
  playlistReverseStates: {},
  proxyHostname: '127.0.0.1',
  proxyPort: '9050',
  proxyUsername: '',
  proxyPassword: '',
  proxyProtocol: 'socks5',
  videoIpBlockScriptPath: '',
  proxyVideos: true,
  region: 'US',
  rememberHistory: true,
  historyRetentionDays: '',
  rememberSearchHistory: true,
  rememberTabNavigationHistory: false,
  watchedProgressSavingMode: 'auto',
  watchedPercentageThreshold: 0.9,
  saveVideoHistoryWithLastViewedPlaylist: true,
  showFamilyFriendlyOnly: false,
  sponsorBlockShowSkippedToast: true,
  sponsorBlockSkippedToastDuration: 6,
  sponsorBlockEnableSubmission: false,
  sponsorBlockUserId: '',
  sponsorBlockGeneratedUserId: '',
  sponsorBlockDraftSegmentsByVideoId: {},
  sponsorBlockChannelWhitelist: [],
  sponsorBlockUrl: 'https://sponsor.ajay.app',
  sponsorBlockSponsor: { color: 'Green', skip: 'autoSkip' },
  sponsorBlockSelfPromo: { color: 'Yellow', skip: 'promptToSkip' },
  sponsorBlockInteraction: { color: 'Pink', skip: 'promptToSkip' },
  sponsorBlockIntro: { color: 'Cyan', skip: 'promptToSkip' },
  sponsorBlockOutro: { color: 'Blue', skip: 'promptToSkip' },
  sponsorBlockRecap: { color: 'Indigo', skip: 'promptToSkip' },
  sponsorBlockHook: { color: 'Blue', skip: 'promptToSkip' },
  sponsorBlockMusicOffTopic: { color: 'Orange', skip: 'promptToSkip' },
  sponsorBlockFiller: { color: 'Purple', skip: 'promptToSkip' },
  sponsorBlockHighlight: { color: 'Red', skip: 'promptToSkip' },
  thumbnailPreference: '',
  thumbnailSize: '240',
  uiRoundness: 100,
  animationSpeed: 100,
  showThumbnailSizeButtonInHeader: true,
  showToastTimeoutIndicator: true,
  toastPosition: 'bottom-left',
  extraThumbnailAction: '',
  blurThumbnails: false,
  syncServerEnabled: false,
  syncServerUrl: 'https://sync.d3sox.me',
  syncServerUsername: '',
  syncServerToken: '',
  syncServerPrivacyMode: 'unknown',
  syncServerPrivacyKey: '',
  syncServerPrivacySalt: '',
  syncServerAutoSync: true,
  syncServerSyncSubscriptions: true,
  syncServerSyncPlaylists: true,
  syncServerSyncHistory: true,
  syncServerSyncPlaybackSpeeds: true,
  syncServerSyncProfiles: true,
  syncServerSyncSessions: true,
  syncServerSyncSettings: true,
  syncServerSettingsExcluded: [],
  syncServerLastSyncAt: 0,
  syncServerSnapshot: '{}',
  useProxy: false,
  userPlaylistSortOrder: 'date_added_descending',
  useRssFeeds: false,
  useReturnYouTubeDislikes: false,
  returnYouTubeDislikesUrl: 'https://ryd-proxy.kavin.rocks',
  useSponsorBlock: false,
  videoVolumeMouseScroll: false,
  videoPlaybackRateMouseScroll: false,
  videoSkipMouseScroll: false,
  videoPlaybackRateInterval: 0.25,
  rememberVolume: true,
  skipSilence: false,
  showSkipSilenceButton: false,
  holdToDoublePlaybackSpeed: true,
  keyboardShortcuts: '{}',
  rememberPlaybackSpeedPerChannel: false,
  autoUpdateChannelPlaybackSpeeds: false,
  channelPlaybackSpeeds: '{}',
  useQuickPlaybackSpeedBar: false,
  quickPlaybackSpeedBarOptions: '[{"speed":0.5,"name":""},{"speed":1,"name":""},{"speed":1.25,"name":""},{"speed":1.5,"name":""},{"speed":1.75,"name":""},{"speed":2,"name":""},{"speed":2.25,"name":""},{"speed":2.5,"name":""},{"speed":3,"name":""}]',
  rememberVideoQualityPerChannel: false,
  autoUpdateChannelVideoQualities: false,
  channelVideoQualities: '{}',
  rememberSubtitlesStatePerChannel: false,
  autoUpdateChannelSubtitlesStates: false,
  channelSubtitlesStates: '{}',
  rememberVolumePerChannel: false,
  autoUpdateChannelVolumes: false,
  channelVolumes: '{}',
  enableScreenshot: false,
  screenshotMode: 'prompt_folder',
  screenshotFormat: 'png',
  screenshotQuality: 95,
  screenshotFolderPath: '',
  screenshotFilenamePattern: '%Y%M%D-%H%N%S',
  settingsSectionSortEnabled: false,
  highlightChangedSettings: false,
  fetchSubscriptionsAutomatically: true,
  showNewSubscriptionFeed: true,
  showNewSubscriptionFeedIndicators: false,
  subscriptionFeedAutoRefreshInterval: '0',
  subscriptionShortsAutoRefreshInterval: '0',
  subscriptionLiveAutoRefreshInterval: '0',
  subscriptionPostsAutoRefreshInterval: '0',
  showProgressBarToast: true,
  settingsPassword: '',
  useDeArrowTitles: false,
  useDeArrowThumbnails: false,
  deArrowThumbnailGeneratorUrl: 'https://dearrow-thumb.ajay.app',
  quickBookmarkTargetPlaylistId: 'favorites',
  generalAutoLoadMorePaginatedItemsEnabled: false,
  hideToTrayOnMinimize: false,
  currentLocale: 'system',
  reducedMotion: 'system',
  defaultInvidiousInstance: '',
  defaultVolume: 1,
  uiScale: 100,
  userPlaylistsSortBy: 'latest_played_first',
  userHistorySortBy: 'latest_played_first',
  enableNotifications: true,
  enablePushNotifications: false,
  enableEmailNotifications: false,
}

export const useSettingsStore = defineStore('settings', {
  state: (): SettingsState => ({ ...DEFAULT_SETTINGS }),

  getters: {
    getChannelsHiddenParsed: (state) => {
      return JSON.parse(state.channelsHidden).map((ch: any) => {
        if (typeof ch === 'string') {
          return { name: ch, preferredName: '', icon: '' }
        }
        return ch
      })
    },

    getChannelsHiddenNames: (state) => {
      return new Set(
        (JSON.parse(state.channelsHidden) as Array<{ name: string }>)
          .map((ch) => typeof ch === 'string' ? ch : ch.name)
      )
    },

    getForbiddenTitlesParsed: (state) => {
      return JSON.parse(state.forbiddenTitles).map((title: string) => title.toLowerCase())
    },

    getTransferableSettings: (state) => {
      const NON_TRANSFERABLE_SETTINGS = new Set([
        'useProxy', 'proxyProtocol', 'proxyHostname', 'proxyPort', 'proxyUsername', 'proxyPassword',
        'externalPlayer', 'externalPlayerExecutable', 'externalPlayerIgnoreWarnings',
        'externalPlayerIgnoreDefaultArgs', 'externalPlayerCustomArgs', 'showAddedExternalPlayerCustomArgs',
        'videoPlaybackEngine', 'ytDlpSource', 'ytDlpChannel', 'ytDlpPath', 'ytDlpFfmpegSource', 'ytDlpFfmpegPath',
        'ytDlpDownloadFolderPath', 'disableSmoothScrolling', 'hideToTrayOnMinimize', 'settingsPassword',
        'screenshotFolderPath', 'syncServerEnabled', 'syncServerUrl', 'syncServerUsername', 'syncServerToken',
        'syncServerPrivacyMode', 'syncServerPrivacyKey', 'syncServerPrivacySalt', 'syncServerAutoSync',
        'syncServerSyncSubscriptions', 'syncServerSyncPlaylists', 'syncServerSyncHistory',
        'syncServerSyncPlaybackSpeeds', 'syncServerSyncProfiles', 'syncServerSyncSessions',
        'syncServerSyncSettings', 'syncServerSettingsExcluded', 'syncServerLastSyncAt', 'syncServerSnapshot',
        'backendFallback', 'backendPreference', 'proxyVideos',
      ])

      const transferableSettings: Partial<SettingsState> = {}
      for (const [key, value] of Object.entries(state)) {
        if (!NON_TRANSFERABLE_SETTINGS.has(key)) {
          ;(transferableSettings as any)[key] = value
        }
      }
      return transferableSettings
    },
  },

  actions: {
    async loadSettings() {
      try {
        const settings = await invoke<{ id: string; value: string }[]>('db_settings_find_all')
        for (const setting of settings) {
          if (setting.id in DEFAULT_SETTINGS) {
            const defaultValue = DEFAULT_SETTINGS[setting.id as keyof SettingsState]
            let parsed: unknown
            try {
              parsed = JSON.parse(setting.value)
            } catch {
              parsed = setting.value
            }
            if (typeof defaultValue === 'boolean') {
              ;(this as any)[setting.id] = parsed === true || parsed === 'true'
            } else if (typeof defaultValue === 'number') {
              ;(this as any)[setting.id] = Number(parsed) || defaultValue
            } else {
              ;(this as any)[setting.id] = parsed
            }
          }
        }
      } catch {
        // Database unavailable, use defaults
      }
    },

    async updateSetting<K extends keyof SettingsState>(key: K, value: SettingsState[K]) {
      ;(this as any)[key] = value
      try {
        const serialized = typeof value === 'object' ? JSON.stringify(value) : String(value)
        await invoke('db_settings_upsert', { id: String(key), value: serialized })
      } catch {
        // Database unavailable, change is in-memory only
      }
    },

    async importSettings(settings: Partial<SettingsState>) {
      Object.assign(this, settings)
      for (const [key, value] of Object.entries(settings)) {
        try {
          const serialized = typeof value === 'object' ? JSON.stringify(value) : String(value)
          await invoke('db_settings_upsert', { id: String(key), value: serialized })
        } catch {
          // Database unavailable
        }
      }
    },

    exportSettings() {
      return this.getTransferableSettings
    },

    resetSettingToDefault(settingKey: keyof SettingsState) {
      if (settingKey in DEFAULT_SETTINGS) {
        ;(this as any)[settingKey] = DEFAULT_SETTINGS[settingKey]
      }
    },

    pinToQuickAccess(settingKey: string) {
      if (!this.pinnedQuickAccess.includes(settingKey)) {
        this.pinnedQuickAccess.push(settingKey)
      }
    },

    unpinFromQuickAccess(settingKey: string) {
      this.pinnedQuickAccess = this.pinnedQuickAccess.filter(k => k !== settingKey)
    },

    togglePinned(settingKey: string) {
      if (this.pinnedQuickAccess.includes(settingKey)) {
        this.unpinFromQuickAccess(settingKey)
      } else {
        this.pinToQuickAccess(settingKey)
      }
    },

    isPinned(settingKey: string): boolean {
      return this.pinnedQuickAccess.includes(settingKey)
    },
  },
})

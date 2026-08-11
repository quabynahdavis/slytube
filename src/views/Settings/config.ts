export interface SettingItem {
  key: string
  type: 'toggle' | 'select' | 'accordion' | 'link' | 'action'
  label: string
  description: string
  synonyms: string[]
  quickAccess?: boolean
  options?: { value: string; label: string }[]
  children?: SettingItem[]
  crossLink?: { category: string; label: string }
}

export interface SettingsSection {
  id: string
  label: string
  description: string
  icon?: string
  items: SettingItem[]
}

export interface SettingsCategory {
  id: string
  icon: string
  label: string
  description: string
  route: string
  sections: SettingsSection[]
}

export const settingsConfig: SettingsCategory[] = [
  {
    id: 'account',
    icon: 'user',
    label: 'settings.categories.account',
    description: 'settings.categories.accountDesc',
    route: '/settings/account',
    sections: [
      {
        id: 'profile',
        label: 'settings.account.profile',
        description: 'settings.account.profileDesc',
        icon: 'profile',
        items: [
          {
            key: 'defaultProfile',
            type: 'select',
            label: 'settings.account.defaultProfile',
            description: 'settings.account.defaultProfileDesc',
            synonyms: ['profile', 'identity', 'default', 'who', 'channel'],
            options: [
              { value: 'allChannels', label: 'All Channels' },
            ],
          },
        ],
      },
      {
        id: 'sync',
        label: 'settings.account.sync',
        description: 'settings.account.syncDesc',
        icon: 'sync',
        items: [
          {
            key: 'syncServerEnabled',
            type: 'toggle',
            label: 'settings.sync.enable',
            description: 'settings.sync.enableDesc',
            synonyms: ['sync', 'cloud', 'backup', 'cross-device', 'server'],
          },
          {
            key: 'syncServerUrl',
            type: 'accordion',
            label: 'settings.sync.serverUrl',
            description: 'settings.sync.serverUrlDesc',
            synonyms: ['sync', 'server', 'url', 'address', 'endpoint'],
            children: [
              {
                key: 'syncServerToken',
                type: 'toggle',
                label: 'settings.sync.serverToken',
                description: 'settings.sync.serverTokenDesc',
                synonyms: ['token', 'auth', 'password', 'key'],
              },
              {
                key: 'syncServerAutoSync',
                type: 'toggle',
                label: 'settings.sync.autoSync',
                description: 'settings.sync.autoSyncDesc',
                synonyms: ['auto', 'automatic', 'sync', 'background'],
              },
            ],
          },
        ],
      },
      {
        id: 'security',
        label: 'settings.account.security',
        description: 'settings.account.securityDesc',
        icon: 'security',
        items: [
          {
            key: 'settingsPassword',
            type: 'action',
            label: 'settings.account.password',
            description: 'settings.account.passwordDesc',
            synonyms: ['password', 'lock', 'pin', 'security', 'protect'],
          },
        ],
      },
    ],
  },
  {
    id: 'appearance',
    icon: 'palette',
    label: 'settings.categories.appearance',
    description: 'settings.categories.appearanceDesc',
    route: '/settings/appearance',
    sections: [
      {
        id: 'theme',
        label: 'settings.appearance.theme',
        description: 'settings.appearance.themeDesc',
        icon: 'theme',
        items: [
          {
            key: 'baseTheme',
            type: 'select',
            label: 'settings.appearance.baseTheme',
            description: 'settings.appearance.baseThemeDesc',
            synonyms: ['theme', 'dark mode', 'night', 'appearance', 'color', 'look', 'style', 'skin', 'light'],
            quickAccess: true,
            options: [
              { value: 'system', label: 'System' },
              { value: 'light', label: 'Light' },
              { value: 'dark', label: 'Dark' },
            ],
          },
          {
            key: 'mainColor',
            type: 'select',
            label: 'settings.appearance.mainColor',
            description: 'settings.appearance.mainColorDesc',
            synonyms: ['color', 'accent', 'primary', 'highlight', 'theme color'],
            options: [
              { value: 'Red', label: 'Red' },
              { value: 'Blue', label: 'Blue' },
              { value: 'Green', label: 'Green' },
              { value: 'Purple', label: 'Purple' },
              { value: 'Orange', label: 'Orange' },
              { value: 'Pink', label: 'Pink' },
            ],
          },
          {
            key: 'secColor',
            type: 'select',
            label: 'settings.appearance.secColor',
            description: 'settings.appearance.secColorDesc',
            synonyms: ['secondary', 'color', 'accent', 'gradient'],
            options: [
              { value: 'Red', label: 'Red' },
              { value: 'Blue', label: 'Blue' },
              { value: 'Green', label: 'Green' },
              { value: 'Purple', label: 'Purple' },
              { value: 'Orange', label: 'Orange' },
              { value: 'Pink', label: 'Pink' },
            ],
          },
        ],
      },
      {
        id: 'language',
        label: 'settings.appearance.language',
        description: 'settings.appearance.languageDesc',
        icon: 'language',
        items: [
          {
            key: 'currentLocale',
            type: 'select',
            label: 'settings.appearance.locale',
            description: 'settings.appearance.localeDesc',
            synonyms: ['language', 'locale', 'interface', 'translate', 'i18n'],
            options: [
              { value: 'system', label: 'System' },
              { value: 'en-US', label: 'English (US)' },
            ],
          },
          {
            key: 'region',
            type: 'select',
            label: 'settings.appearance.region',
            description: 'settings.appearance.regionDesc',
            synonyms: ['region', 'country', 'location', 'content', 'geo'],
            options: [
              { value: 'US', label: 'United States' },
              { value: 'GB', label: 'United Kingdom' },
              { value: 'DE', label: 'Germany' },
              { value: 'FR', label: 'France' },
              { value: 'JP', label: 'Japan' },
              { value: 'KR', label: 'South Korea' },
              { value: 'IN', label: 'India' },
              { value: 'BR', label: 'Brazil' },
            ],
          },
        ],
      },
      {
        id: 'sidebar',
        label: 'settings.appearance.sidebar',
        description: 'settings.appearance.sidebarDesc',
        icon: 'sidebar',
        items: [
          {
            key: 'expandSideBar',
            type: 'toggle',
            label: 'settings.appearance.expandSideBar',
            description: 'settings.appearance.expandSideBarDesc',
            synonyms: ['sidebar', 'expand', 'labels', 'navigation', 'menu'],
          },
          {
            key: 'hideSideBarOnWatchPages',
            type: 'toggle',
            label: 'settings.appearance.hideSideBarOnWatch',
            description: 'settings.appearance.hideSideBarOnWatchDesc',
            synonyms: ['sidebar', 'hide', 'watch', 'fullscreen', 'video'],
          },
          {
            key: 'hideHeaderLogo',
            type: 'toggle',
            label: 'settings.appearance.hideHeaderLogo',
            description: 'settings.appearance.hideHeaderLogoDesc',
            synonyms: ['header', 'logo', 'brand', 'title', 'hide'],
          },
          {
            key: 'landingPage',
            type: 'select',
            label: 'settings.appearance.landingPage',
            description: 'settings.appearance.landingPageDesc',
            synonyms: ['landing', 'startup', 'home', 'start', 'page', 'default'],
            options: [
              { value: 'subscriptions', label: 'Subscriptions' },
              { value: 'trending', label: 'Trending' },
              { value: 'popular', label: 'Popular' },
              { value: 'search', label: 'Search' },
            ],
          },
        ],
      },
      {
        id: 'accessibility',
        label: 'settings.appearance.accessibility',
        description: 'settings.appearance.accessibilityDesc',
        icon: 'accessibility',
        items: [
          {
            key: 'reducedMotion',
            type: 'select',
            label: 'settings.appearance.reducedMotion',
            description: 'settings.appearance.reducedMotionDesc',
            synonyms: ['motion', 'animation', 'reduce', 'accessibility', 'movement'],
            options: [
              { value: 'system', label: 'System' },
              { value: 'reduce', label: 'Reduce' },
              { value: 'none', label: 'None' },
            ],
          },
          {
            key: 'uiScale',
            type: 'select',
            label: 'settings.appearance.uiScale',
            description: 'settings.appearance.uiScaleDesc',
            synonyms: ['scale', 'size', 'zoom', 'ui', 'interface', 'bigger', 'smaller'],
            options: [
              { value: '75', label: '75%' },
              { value: '100', label: '100%' },
              { value: '125', label: '125%' },
              { value: '150', label: '150%' },
            ],
          },
        ],
      },
      {
        id: 'performance',
        label: 'settings.appearance.performance',
        description: 'settings.appearance.performanceDesc',
        icon: 'performance',
        items: [
          {
            key: 'disableSmoothScrolling',
            type: 'toggle',
            label: 'settings.appearance.disableSmoothScrolling',
            description: 'settings.appearance.disableSmoothScrollingDesc',
            synonyms: ['scrolling', 'smooth', 'performance', 'speed', 'lag'],
          },
          {
            key: 'ambientMode',
            type: 'toggle',
            label: 'settings.appearance.ambientMode',
            description: 'settings.appearance.ambientModeDesc',
            synonyms: ['ambient', 'lighting', 'effect', 'glow', 'video'],
          },
          {
            key: 'animationSpeed',
            type: 'select',
            label: 'settings.appearance.animationSpeed',
            description: 'settings.appearance.animationSpeedDesc',
            synonyms: ['animation', 'speed', 'transition', 'motion', 'fast', 'slow'],
            options: [
              { value: '0', label: 'Instant' },
              { value: '50', label: 'Fast' },
              { value: '100', label: 'Normal' },
              { value: '200', label: 'Slow' },
            ],
          },
        ],
      },
    ],
  },
  {
    id: 'notifications',
    icon: 'bell',
    label: 'settings.categories.notifications',
    description: 'settings.categories.notificationsDesc',
    route: '/settings/notifications',
    sections: [
      {
        id: 'general',
        label: 'settings.notifications.general',
        description: 'settings.notifications.generalDesc',
        icon: 'general',
        items: [
          {
            key: 'enableNotifications',
            type: 'toggle',
            label: 'settings.notifications.enable',
            description: 'settings.notifications.enableDesc',
            synonyms: ['notifications', 'enable', 'push', 'alerts', 'notify', 'bell'],
            quickAccess: true,
          },
        ],
      },
      {
        id: 'push',
        label: 'settings.notifications.push',
        description: 'settings.notifications.pushDesc',
        icon: 'push',
        items: [
          {
            key: 'enablePushNotifications',
            type: 'toggle',
            label: 'settings.notifications.pushEnable',
            description: 'settings.notifications.pushEnableDesc',
            synonyms: ['push', 'notifications', 'device', 'mobile', 'background'],
          },
        ],
      },
      {
        id: 'email',
        label: 'settings.notifications.email',
        description: 'settings.notifications.emailDesc',
        icon: 'email',
        items: [
          {
            key: 'enableEmailNotifications',
            type: 'toggle',
            label: 'settings.notifications.emailEnable',
            description: 'settings.notifications.emailEnableDesc',
            synonyms: ['email', 'mail', 'notifications', 'digest'],
          },
        ],
      },
      {
        id: 'inApp',
        label: 'settings.notifications.inApp',
        description: 'settings.notifications.inAppDesc',
        icon: 'inApp',
        items: [
          {
            key: 'subscriptionUpdates',
            type: 'toggle',
            label: 'settings.notifications.subscriptionUpdates',
            description: 'settings.notifications.subscriptionUpdatesDesc',
            synonyms: ['subscription', 'updates', 'channels', 'new videos', 'feed'],
          },
          {
            key: 'showToastTimeoutIndicator',
            type: 'toggle',
            label: 'settings.appearance.showToastTimeoutIndicator',
            description: 'settings.appearance.showToastTimeoutIndicatorDesc',
            synonyms: ['toast', 'timeout', 'indicator', 'notification', 'progress'],
          },
          {
            key: 'toastPosition',
            type: 'select',
            label: 'settings.appearance.toastPosition',
            description: 'settings.appearance.toastPositionDesc',
            synonyms: ['toast', 'position', 'location', 'corner', 'notification'],
            options: [
              { value: 'top-right', label: 'Top Right' },
              { value: 'top-left', label: 'Top Left' },
              { value: 'bottom-right', label: 'Bottom Right' },
              { value: 'bottom-left', label: 'Bottom Left' },
              { value: 'top-center', label: 'Top Center' },
              { value: 'bottom-center', label: 'Bottom Center' },
            ],
          },
        ],
      },
    ],
  },
  {
    id: 'privacy',
    icon: 'shield',
    label: 'settings.categories.privacy',
    description: 'settings.categories.privacyDesc',
    route: '/settings/privacy',
    sections: [
      {
        id: 'history',
        label: 'settings.privacy.history',
        description: 'settings.privacy.historyDesc',
        icon: 'history',
        items: [
          {
            key: 'rememberHistory',
            type: 'toggle',
            label: 'settings.privacy.rememberHistory',
            description: 'settings.privacy.rememberHistoryDesc',
            synonyms: ['history', 'remember', 'watch', 'tracking', 'record'],
            quickAccess: true,
          },
          {
            key: 'historyRetentionDays',
            type: 'select',
            label: 'settings.privacy.historyRetention',
            description: 'settings.privacy.historyRetentionDesc',
            synonyms: ['history', 'retention', 'days', 'keep', 'duration', 'auto-delete'],
            options: [
              { value: '', label: 'Forever' },
              { value: '30', label: '30 days' },
              { value: '90', label: '90 days' },
              { value: '180', label: '180 days' },
              { value: '365', label: '1 year' },
            ],
          },
          {
            key: 'watchedProgressSavingMode',
            type: 'select',
            label: 'settings.privacy.watchedProgressSavingMode',
            description: 'settings.privacy.watchedProgressSavingModeDesc',
            synonyms: ['progress', 'watched', 'resume', 'save', 'timestamp'],
            options: [
              { value: 'auto', label: 'Automatic' },
              { value: 'manual', label: 'Manual' },
            ],
          },
        ],
      },
      {
        id: 'searchHistory',
        label: 'settings.privacy.searchHistory',
        description: 'settings.privacy.searchHistoryDesc',
        icon: 'search',
        items: [
          {
            key: 'rememberSearchHistory',
            type: 'toggle',
            label: 'settings.privacy.rememberSearchHistory',
            description: 'settings.privacy.rememberSearchHistoryDesc',
            synonyms: ['search', 'history', 'remember', 'queries', 'suggestions'],
          },
          {
            key: 'enableSearchSuggestions',
            type: 'toggle',
            label: 'settings.appearance.enableSearchSuggestions',
            description: 'settings.appearance.enableSearchSuggestionsDesc',
            synonyms: ['search', 'suggestions', 'autocomplete', 'search bar'],
          },
        ],
      },
      {
        id: 'proxy',
        label: 'settings.privacy.proxy',
        description: 'settings.privacy.proxyDesc',
        icon: 'proxy',
        items: [
          {
            key: 'useProxy',
            type: 'toggle',
            label: 'settings.privacy.useProxy',
            description: 'settings.privacy.useProxyDesc',
            synonyms: ['proxy', 'socks', 'vpn', 'tunnel', 'network', 'tor'],
          },
          {
            key: 'proxyVideos',
            type: 'toggle',
            label: 'settings.privacy.proxyVideos',
            description: 'settings.privacy.proxyVideosDesc',
            synonyms: ['proxy', 'video', 'stream', 'media', 'traffic'],
          },
        ],
      },
      {
        id: 'data',
        label: 'settings.privacy.data',
        description: 'settings.privacy.dataDesc',
        icon: 'data',
        items: [
          {
            key: 'exportData',
            type: 'action',
            label: 'settings.privacy.exportData',
            description: 'settings.privacy.exportDataDesc',
            synonyms: ['export', 'download', 'backup', 'data', 'json'],
            crossLink: { category: 'account', label: 'settings.common.goToSync' },
          },
          {
            key: 'deleteData',
            type: 'action',
            label: 'settings.privacy.deleteData',
            description: 'settings.privacy.deleteDataDesc',
            synonyms: ['delete', 'clear', 'remove', 'erase', 'purge', 'reset'],
          },
        ],
      },
    ],
  },
  {
    id: 'player',
    icon: 'play',
    label: 'settings.categories.player',
    description: 'settings.categories.playerDesc',
    route: '/settings/player',
    sections: [
      {
        id: 'playback',
        label: 'settings.player.playback',
        description: 'settings.player.playbackDesc',
        icon: 'playback',
        items: [
          {
            key: 'autoplayVideos',
            type: 'toggle',
            label: 'settings.player.autoplay',
            description: 'settings.player.autoplayDesc',
            synonyms: ['autoplay', 'auto', 'play', 'start', 'automatic', 'video'],
            quickAccess: true,
          },
          {
            key: 'defaultQuality',
            type: 'select',
            label: 'settings.player.defaultQuality',
            description: 'settings.player.defaultQualityDesc',
            synonyms: ['quality', 'resolution', 'hd', '1080', '720', '480', '4k', '2k', 'video'],
            options: [
              { value: '144', label: '144p' },
              { value: '240', label: '240p' },
              { value: '360', label: '360p' },
              { value: '480', label: '480p' },
              { value: '720', label: '720p HD' },
              { value: '1080', label: '1080p Full HD' },
              { value: '1440', label: '1440p 2K' },
              { value: '2160', label: '2160p 4K' },
            ],
          },
          {
            key: 'defaultPlaybackRate',
            type: 'select',
            label: 'settings.player.defaultPlaybackRate',
            description: 'settings.player.defaultPlaybackRateDesc',
            synonyms: ['speed', 'playback', 'rate', 'fast', 'slow', 'velocity'],
            options: [
              { value: '0.25', label: '0.25x' },
              { value: '0.5', label: '0.5x' },
              { value: '0.75', label: '0.75x' },
              { value: '1', label: '1x (Normal)' },
              { value: '1.25', label: '1.25x' },
              { value: '1.5', label: '1.5x' },
              { value: '1.75', label: '1.75x' },
              { value: '2', label: '2x' },
            ],
          },
          {
            key: 'defaultVolume',
            type: 'select',
            label: 'settings.player.defaultVolume',
            description: 'settings.player.defaultVolumeDesc',
            synonyms: ['volume', 'audio', 'sound', 'loud', 'quiet', 'loudness'],
            options: [
              { value: '0', label: 'Muted' },
              { value: '0.25', label: '25%' },
              { value: '0.5', label: '50%' },
              { value: '0.75', label: '75%' },
              { value: '1', label: '100%' },
            ],
          },
          {
            key: 'playNextVideo',
            type: 'toggle',
            label: 'settings.player.playNextVideo',
            description: 'settings.player.playNextVideoDesc',
            synonyms: ['next', 'queue', 'autoplay', 'continue', 'auto'],
          },
        ],
      },
      {
        id: 'sponsorblock',
        label: 'settings.player.sponsorblock',
        description: 'settings.player.sponsorblockDesc',
        icon: 'sponsorblock',
        items: [
          {
            key: 'useSponsorBlock',
            type: 'toggle',
            label: 'settings.player.useSponsorBlock',
            description: 'settings.player.useSponsorBlockDesc',
            synonyms: ['sponsorblock', 'sponsor', 'skip', 'segment', 'ads'],
          },
          {
            key: 'sponsorBlockUrl',
            type: 'select',
            label: 'settings.player.sponsorBlockUrl',
            description: 'settings.player.sponsorBlockUrlDesc',
            synonyms: ['sponsorblock', 'url', 'server', 'api', 'endpoint'],
            options: [
              { value: 'https://sponsor.ajay.app', label: 'sponsor.ajay.app' },
            ],
          },
          {
            key: 'sponsorBlockShowSkippedToast',
            type: 'toggle',
            label: 'settings.player.sponsorBlockShowSkippedToast',
            description: 'settings.player.sponsorBlockShowSkippedToastDesc',
            synonyms: ['sponsorblock', 'toast', 'notification', 'skip', 'message'],
          },
        ],
      },
      {
        id: 'downloads',
        label: 'settings.player.downloads',
        description: 'settings.player.downloadsDesc',
        icon: 'downloads',
        items: [
          {
            key: 'ytDlpDownloadFolderPath',
            type: 'action',
            label: 'settings.player.downloadPath',
            description: 'settings.player.downloadPathDesc',
            synonyms: ['download', 'path', 'folder', 'location', 'save', 'directory'],
          },
          {
            key: 'ytDlpSelectedTemplate',
            type: 'select',
            label: 'settings.player.downloadFormat',
            description: 'settings.player.downloadFormatDesc',
            synonyms: ['download', 'format', 'template', 'quality', 'format selection'],
            options: [
              { value: 'video:best', label: 'Best Video' },
              { value: 'video:1080', label: '1080p' },
              { value: 'video:720', label: '720p' },
              { value: 'audio:best', label: 'Audio Only' },
              { value: 'custom', label: 'Custom' },
            ],
          },
          {
            key: 'ytDlpPath',
            type: 'action',
            label: 'settings.player.ytDlpPath',
            description: 'settings.player.ytDlpPathDesc',
            synonyms: ['yt-dlp', 'path', 'binary', 'executable', 'location'],
          },
        ],
      },
      {
        id: 'subscriptions',
        label: 'settings.player.subscriptions',
        description: 'settings.player.subscriptionsDesc',
        icon: 'subscriptions',
        items: [
          {
            key: 'fetchSubscriptionsAutomatically',
            type: 'toggle',
            label: 'settings.player.fetchAutomatically',
            description: 'settings.player.fetchAutomaticallyDesc',
            synonyms: ['subscription', 'auto', 'fetch', 'refresh', 'update', 'background'],
          },
          {
            key: 'hideWatchedSubs',
            type: 'toggle',
            label: 'settings.player.hideWatched',
            description: 'settings.player.hideWatchedDesc',
            synonyms: ['subscription', 'hide', 'watched', 'videos', 'filter'],
          },
          {
            key: 'showNewSubscriptionFeed',
            type: 'toggle',
            label: 'settings.player.showNewSubscriptionFeed',
            description: 'settings.player.showNewSubscriptionFeedDesc',
            synonyms: ['subscription', 'new', 'feed', 'badge', 'indicator'],
          },
        ],
      },
      {
        id: 'backend',
        label: 'settings.player.backend',
        description: 'settings.player.backendDesc',
        icon: 'backend',
        items: [
          {
            key: 'backendPreference',
            type: 'select',
            label: 'settings.player.backendPreference',
            description: 'settings.player.backendPreferenceDesc',
            synonyms: ['backend', 'api', 'invidious', 'youtube', 'source', 'provider'],
            options: [
              { value: 'invidious', label: 'Invidious API' },
              { value: 'local', label: 'Local Extraction' },
            ],
          },
          {
            key: 'defaultInvidiousInstance',
            type: 'action',
            label: 'settings.player.invidiousInstance',
            description: 'settings.player.invidiousInstanceDesc',
            synonyms: ['invidious', 'instance', 'server', 'api', 'url', 'endpoint'],
          },
          {
            key: 'backendFallback',
            type: 'toggle',
            label: 'settings.player.backendFallback',
            description: 'settings.player.backendFallbackDesc',
            synonyms: ['fallback', 'backup', 'failover', 'alternative', 'redundancy'],
          },
        ],
      },
    ],
  },
]

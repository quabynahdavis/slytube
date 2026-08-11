export interface SettingItem {
  key: string
  type: 'toggle' | 'select' | 'accordion' | 'link' | 'action' | 'text'
  label: string
  description: string
  synonyms: string[]
  options?: { value: string; label: string }[]
  children?: SettingItem[]
  crossLink?: { category: string; label: string }
}

export interface SettingsSection {
  id: string
  label: string
  description: string
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
    id: 'general',
    icon: 'gear',
    label: 'settings.categories.general',
    description: 'settings.categories.generalDesc',
    route: '/settings/general',
    sections: [
      {
        id: 'startup',
        label: 'settings.general.startup',
        description: 'settings.general.startupDesc',
        items: [
          {
            key: 'landingPage',
            type: 'select',
            label: 'settings.general.landingPage',
            description: 'settings.general.landingPageDesc',
            synonyms: ['landing', 'startup', 'home', 'start', 'page', 'default'],
            options: [
              { value: 'subscriptions', label: 'Subscriptions' },
              { value: 'trending', label: 'Trending' },
              { value: 'popular', label: 'Popular' },
              { value: 'search', label: 'Search' },
            ],
          },
          {
            key: 'checkForUpdates',
            type: 'toggle',
            label: 'settings.general.autoUpdate',
            description: 'settings.general.autoUpdateDesc',
            synonyms: ['update', 'check', 'automatic', 'version'],
          },
        ],
      },
      {
        id: 'behavior',
        label: 'settings.general.behavior',
        description: 'settings.general.behaviorDesc',
        items: [
          {
            key: 'confirmCloseApp',
            type: 'toggle',
            label: 'settings.general.confirmClose',
            description: 'settings.general.confirmCloseDesc',
            synonyms: ['close', 'confirm', 'exit', 'quit'],
          },
          {
            key: 'hideToTrayOnMinimize',
            type: 'toggle',
            label: 'settings.general.minimizeTray',
            description: 'settings.general.minimizeTrayDesc',
            synonyms: ['tray', 'minimize', 'taskbar', 'background'],
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
        items: [
          {
            key: 'baseTheme',
            type: 'select',
            label: 'settings.appearance.baseTheme',
            description: 'settings.appearance.baseThemeDesc',
            synonyms: ['theme', 'dark mode', 'night', 'appearance', 'color', 'look', 'style', 'skin', 'light'],
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
        id: 'display',
        label: 'settings.appearance.display',
        description: 'settings.appearance.displayDesc',
        items: [
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
          {
            key: 'disableSmoothScrolling',
            type: 'toggle',
            label: 'settings.appearance.smoothScrolling',
            description: 'settings.appearance.smoothScrollingDesc',
            synonyms: ['scrolling', 'smooth', 'performance', 'speed', 'lag'],
          },
          {
            key: 'ambientMode',
            type: 'toggle',
            label: 'settings.appearance.ambientMode',
            description: 'settings.appearance.ambientModeDesc',
            synonyms: ['ambient', 'lighting', 'effect', 'glow', 'video'],
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
        items: [
          {
            key: 'autoplayVideos',
            type: 'toggle',
            label: 'settings.player.autoplay',
            description: 'settings.player.autoplayDesc',
            synonyms: ['autoplay', 'auto', 'play', 'start', 'automatic', 'video'],
          },
          {
            key: 'playNextVideo',
            type: 'toggle',
            label: 'settings.player.playNextVideo',
            description: 'settings.player.playNextVideoDesc',
            synonyms: ['next', 'queue', 'autoplay', 'continue', 'auto'],
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
            label: 'settings.player.defaultSpeed',
            description: 'settings.player.defaultSpeedDesc',
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
        ],
      },
      {
        id: 'features',
        label: 'settings.player.features',
        description: 'settings.player.featuresDesc',
        items: [
          {
            key: 'useSponsorBlock',
            type: 'toggle',
            label: 'settings.player.sponsorblock',
            description: 'settings.player.sponsorblockDesc',
            synonyms: ['sponsorblock', 'sponsor', 'skip', 'segment', 'ads'],
          },
          {
            key: 'sponsorBlockShowSkippedToast',
            type: 'toggle',
            label: 'settings.player.sponsorblockToast',
            description: 'settings.player.sponsorblockToastDesc',
            synonyms: ['sponsorblock', 'toast', 'notification', 'skip', 'message'],
          },
          {
            key: 'autoOpenChapters',
            type: 'toggle',
            label: 'settings.player.chapters',
            description: 'settings.player.chaptersDesc',
            synonyms: ['chapters', 'auto', 'open', 'sections'],
          },
          {
            key: 'scrollMiniPlayerEnabled',
            type: 'toggle',
            label: 'settings.player.miniPlayer',
            description: 'settings.player.miniPlayerDesc',
            synonyms: ['mini', 'player', 'scroll', 'floating', 'picture'],
          },
          {
            key: 'loopShorts',
            type: 'toggle',
            label: 'settings.player.loopShorts',
            description: 'settings.player.loopShortsDesc',
            synonyms: ['shorts', 'loop', 'repeat', 'replay'],
          },
        ],
      },
    ],
  },
  {
    id: 'downloads',
    icon: 'download',
    label: 'settings.categories.downloads',
    description: 'settings.categories.downloadsDesc',
    route: '/settings/downloads',
    sections: [
      {
        id: 'general',
        label: 'settings.downloads.general',
        description: 'settings.downloads.generalDesc',
        items: [
          {
            key: 'ytDlpSelectedTemplate',
            type: 'select',
            label: 'settings.downloads.format',
            description: 'settings.downloads.formatDesc',
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
            key: 'ytDlpDownloadFolderPath',
            type: 'action',
            label: 'settings.downloads.path',
            description: 'settings.downloads.pathDesc',
            synonyms: ['download', 'path', 'folder', 'location', 'save', 'directory'],
          },
          {
            key: 'ytDlpPath',
            type: 'action',
            label: 'settings.downloads.ytdlpPath',
            description: 'settings.downloads.ytdlpPathDesc',
            synonyms: ['yt-dlp', 'path', 'binary', 'executable', 'location'],
          },
        ],
      },
      {
        id: 'subtitles',
        label: 'settings.downloads.subtitles',
        description: 'settings.downloads.subtitlesDesc',
        items: [
          {
            key: 'enableSubtitlesByDefault',
            type: 'toggle',
            label: 'settings.downloads.enableSubtitles',
            description: 'settings.downloads.enableSubtitlesDesc',
            synonyms: ['subtitles', 'captions', 'cc', 'text'],
          },
          {
            key: 'preferredCaptionLocale',
            type: 'select',
            label: 'settings.downloads.subtitleLang',
            description: 'settings.downloads.subtitleLangDesc',
            synonyms: ['subtitle', 'language', 'locale', 'caption'],
            options: [
              { value: '', label: 'None' },
              { value: 'en', label: 'English' },
              { value: 'es', label: 'Spanish' },
              { value: 'fr', label: 'French' },
              { value: 'de', label: 'German' },
              { value: 'ja', label: 'Japanese' },
              { value: 'ko', label: 'Korean' },
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
        items: [
          {
            key: 'rememberHistory',
            type: 'toggle',
            label: 'settings.privacy.watchHistory',
            description: 'settings.privacy.watchHistoryDesc',
            synonyms: ['history', 'remember', 'watch', 'tracking', 'record'],
          },
          {
            key: 'historyRetentionDays',
            type: 'select',
            label: 'settings.privacy.retention',
            description: 'settings.privacy.retentionDesc',
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
            key: 'rememberSearchHistory',
            type: 'toggle',
            label: 'settings.privacy.searchHistory',
            description: 'settings.privacy.searchHistoryDesc',
            synonyms: ['search', 'history', 'remember', 'queries', 'suggestions'],
          },
          {
            key: 'enableSearchSuggestions',
            type: 'toggle',
            label: 'settings.privacy.searchSuggestions',
            description: 'settings.privacy.searchSuggestionsDesc',
            synonyms: ['search', 'suggestions', 'autocomplete', 'search bar'],
          },
        ],
      },
      {
        id: 'proxy',
        label: 'settings.privacy.proxy',
        description: 'settings.privacy.proxyDesc',
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
    ],
  },
  {
    id: 'sync',
    icon: 'cloud',
    label: 'settings.categories.sync',
    description: 'settings.categories.syncDesc',
    route: '/settings/sync',
    sections: [
      {
        id: 'server',
        label: 'settings.sync.server',
        description: 'settings.sync.serverDesc',
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
            type: 'text',
            label: 'settings.sync.serverUrl',
            description: 'settings.sync.serverUrlDesc',
            synonyms: ['sync', 'server', 'url', 'address', 'endpoint'],
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
    id: 'advanced',
    icon: 'code',
    label: 'settings.categories.advanced',
    description: 'settings.categories.advancedDesc',
    route: '/settings/advanced',
    sections: [
      {
        id: 'backend',
        label: 'settings.advanced.backend',
        description: 'settings.advanced.backendDesc',
        items: [
          {
            key: 'backendPreference',
            type: 'select',
            label: 'settings.advanced.backendPreference',
            description: 'settings.advanced.backendPreferenceDesc',
            synonyms: ['backend', 'api', 'invidious', 'youtube', 'source', 'provider'],
            options: [
              { value: 'invidious', label: 'Invidious API' },
              { value: 'local', label: 'Local Extraction' },
            ],
          },
          {
            key: 'defaultInvidiousInstance',
            type: 'action',
            label: 'settings.advanced.instance',
            description: 'settings.advanced.instanceDesc',
            synonyms: ['invidious', 'instance', 'server', 'api', 'url', 'endpoint'],
          },
          {
            key: 'backendFallback',
            type: 'toggle',
            label: 'settings.advanced.fallback',
            description: 'settings.advanced.fallbackDesc',
            synonyms: ['fallback', 'backup', 'failover', 'alternative', 'redundancy'],
          },
        ],
      },
      {
        id: 'stats',
        label: 'settings.advanced.stats',
        description: 'settings.advanced.statsDesc',
        items: [
          {
            key: 'enableWatchStats',
            type: 'toggle',
            label: 'settings.advanced.watchStats',
            description: 'settings.advanced.watchStatsDesc',
            synonyms: ['stats', 'statistics', 'watch', 'tracking', 'analytics'],
          },
        ],
      },
      {
        id: 'contentFilters',
        label: 'settings.advanced.contentFilters',
        description: 'settings.advanced.contentFiltersDesc',
        items: [
          {
            key: 'hideComments',
            type: 'toggle',
            label: 'settings.advanced.hideComments',
            description: 'settings.advanced.hideCommentsDesc',
            synonyms: ['comments', 'hide', 'remove'],
          },
          {
            key: 'hideLiveStreams',
            type: 'toggle',
            label: 'settings.advanced.hideLiveStreams',
            description: 'settings.advanced.hideLiveStreamsDesc',
            synonyms: ['live', 'streams', 'hide'],
          },
          {
            key: 'hideRecommendedVideos',
            type: 'toggle',
            label: 'settings.advanced.hideRecommended',
            description: 'settings.advanced.hideRecommendedDesc',
            synonyms: ['recommended', 'hide', 'suggestions'],
          },
          {
            key: 'hideTrendingVideos',
            type: 'toggle',
            label: 'settings.advanced.hideTrending',
            description: 'settings.advanced.hideTrendingDesc',
            synonyms: ['trending', 'hide'],
          },
          {
            key: 'hidePopularVideos',
            type: 'toggle',
            label: 'settings.advanced.hidePopular',
            description: 'settings.advanced.hidePopularDesc',
            synonyms: ['popular', 'hide'],
          },
        ],
      },
    ],
  },
]

import { createRouter, createWebHistory } from 'vue-router'

import Home from '@/views/Home.vue'
import Trending from '@/views/Trending.vue'
import Subscriptions from '@/views/Subscriptions.vue'
import Shorts from '@/views/Shorts.vue'
import Posts from '@/views/Posts.vue'
import History from '@/views/History.vue'
import UserPlaylists from '@/views/UserPlaylists.vue'
import Playlist from '@/views/Playlist.vue'
import Downloads from '@/views/Downloads.vue'
import Search from '@/views/Search.vue'
import Watch from '@/views/Watch.vue'
import Channel from '@/views/Channel.vue'
import Hashtag from '@/views/Hashtag.vue'
import Popular from '@/views/Popular.vue'
import Post from '@/views/Post.vue'
import Settings from '@/views/Settings/index.vue'
import SettingsCategory from '@/views/Settings/CategoryView.vue'
import ProfileSettings from '@/views/ProfileSettings.vue'
import Stats from '@/views/Stats.vue'
import About from '@/views/About.vue'

const router = createRouter({
  history: createWebHistory(),
  routes: [
    { path: '/', name: 'home', component: Home },
    { path: '/trending', name: 'trending', component: Trending },
    { path: '/subscriptions', name: 'subscriptions', component: Subscriptions },
    { path: '/shorts', name: 'shorts', component: Shorts },
    { path: '/posts', name: 'posts', component: Posts },
    { path: '/history', name: 'history', component: History },
    { path: '/playlists', name: 'playlists', component: UserPlaylists },
    { path: '/playlist/:id', name: 'playlist', component: Playlist },
    { path: '/playlist/watch-later', name: 'watch-later', component: Playlist },
    { path: '/downloads', name: 'downloads', component: Downloads },
    { path: '/search', name: 'search', component: Search },
    { path: '/watch', name: 'watch', component: Watch },
    { path: '/channel/:id', name: 'channel', component: Channel },
    { path: '/hashtag/:tag', name: 'hashtag', component: Hashtag },
    { path: '/popular', name: 'popular', component: Popular },
    { path: '/post/:id', name: 'post', component: Post },
    {
      path: '/settings',
      name: 'settings',
      component: Settings,
      children: [
        { path: '', name: 'settings-home', redirect: { name: 'settings-general' } },
        { path: 'general', name: 'settings-general', component: SettingsCategory, props: { categoryId: 'general' } },
        { path: 'appearance', name: 'settings-appearance', component: SettingsCategory, props: { categoryId: 'appearance' } },
        { path: 'player', name: 'settings-player', component: SettingsCategory, props: { categoryId: 'player' } },
        { path: 'downloads', name: 'settings-downloads', component: SettingsCategory, props: { categoryId: 'downloads' } },
        { path: 'privacy', name: 'settings-privacy', component: SettingsCategory, props: { categoryId: 'privacy' } },
        { path: 'sync', name: 'settings-sync', component: SettingsCategory, props: { categoryId: 'sync' } },
        { path: 'advanced', name: 'settings-advanced', component: SettingsCategory, props: { categoryId: 'advanced' } },
      ],
    },
    { path: '/profiles', name: 'profiles', component: ProfileSettings },
    { path: '/stats', name: 'stats', component: Stats },
    { path: '/about', name: 'about', component: About },
  ],
})

export default router

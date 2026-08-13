import { createApp } from 'vue'
import { pinia } from './stores'
import router from './router'
import App from './App.vue'
import './style.css'
import i18n from './i18n'
import { loadInstances } from './api/invidious'
import { scrollAnimationPlugin } from './composables/useScrollAnimation'
import { useTabsStore, startTabSessionPersistence } from './stores/tabs'

await loadInstances()

const app = createApp(App)
app.use(pinia)
app.use(router)
app.use(i18n)
app.use(scrollAnimationPlugin)

// Restore persisted tab session and start debounced persistence
const tabsStore = useTabsStore()
await tabsStore.restoreTabs()
startTabSessionPersistence()

app.mount('#app')

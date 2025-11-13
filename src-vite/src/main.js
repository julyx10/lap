import { createApp } from 'vue'
import { createI18n } from 'vue-i18n'
import { createPinia } from 'pinia'
import piniaPersistedState from 'pinia-plugin-persistedstate'
import { listen } from '@tauri-apps/api/event'
import { invoke } from '@tauri-apps/api/core'
import router from '@/common/router'
import App from '@/App.vue'
import { useConfigStore } from '@/stores/configStore'
import '@/assets/app.css'

// I18n
import en from '@/locales/en.json'
import ja from '@/locales/ja.json'
import zh from '@/locales/zh.json'

// Create the app instance
const app = createApp(App)

// Create Pinia store and use the persisted state plugin
const pinia = createPinia()
pinia.use(piniaPersistedState)
app.use(pinia) // Use Pinia
const config = useConfigStore() // Use the config store

// apply the current appearance setting
const theme = en.settings.general.appearance_options[config.settings.appearance] || 'light'; // Default to light if not set
document.documentElement.setAttribute('data-theme', theme);

// Create the I18n instance
const i18n = createI18n({
  legacy: false, // Disable legacy mode
  locale: config.settings.language, // Use language setting from config store
  fallbackLocale: "en",
  messages: {
    en,
    ja,
    zh
  },
})

// Set up global properties
app.config.globalProperties.$invoke = invoke

// Use the router and i18n
app.use(router)
app.use(i18n)

// Mount the app
app.mount('#app')
console.log('App mounted', app)

// Listen for events
listen('settings-settingsTabIndex-changed', (event) => {
  config.setSettingsTabIndex(event.payload)
})
listen('settings-appearance-changed', (event) => {
  config.setAppearance(event.payload)
})
listen('settings-language-changed', (event) => {
  config.setLanguage(event.payload)
})
listen('settings-showButtonText-changed', (event) => {
  config.setShowButtonText(event.payload)
})
listen('settings-showToolTip-changed', (event) => {
  config.setShowToolTip(event.payload)
})
listen('settings-showStatusBar-changed', (event) => {
  config.setShowStatusBar(event.payload)
})
listen('settings-debugMode-changed', (event) => {
  config.setDebugMode(event.payload)
})
listen('settings-gridSize-changed', (event) => {
  config.setGridSize(event.payload)
})
listen('settings-gridScaling-changed', (event) => {
  config.setGridScaling(event.payload)
})
listen('settings-gridLabelPrimary-changed', (event) => {
  config.setGridLabelPrimary(event.payload)
})
listen('settings-gridLabelSecondary-changed', (event) => {
  config.setGridLabelSecondary(event.payload)
})
listen('settings-filmStripViewPreviewPosition-changed', (event) => {
  config.setfilmStripViewPreviewPosition(event.payload)
})
listen('settings-mouseWheelMode-changed', (event) => {
  config.setMouseWheelMode(event.payload)
})
listen('settings-slideShowInterval-changed', (event) => {
  config.setSlideShowInterval(event.payload)
})
listen('settings-autoPlayVideo-changed', (event) => {
  config.setAutoPlayVideo(event.payload)
})
listen('settings-navigatorViewMode-changed', (event) => {
  config.setNavigatorViewMode(event.payload)
})
listen('settings-navigatorViewSize-changed', (event) => {
  config.setNavigatorViewSize(event.payload)
})
listen('settings-showComment-changed', (event) => {
  config.setShowComment(event.payload)
})
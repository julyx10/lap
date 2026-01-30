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
import zh from '@/locales/zh.json'
import es from '@/locales/es.json'
import fr from '@/locales/fr.json'
import de from '@/locales/de.json'
import ja from '@/locales/ja.json'
import ko from '@/locales/ko.json'
import ru from '@/locales/ru.json'
import pt from '@/locales/pt.json'

// Create the app instance
const app = createApp(App)

// Create Pinia store and use the persisted state plugin
const pinia = createPinia()
pinia.use(piniaPersistedState)
app.use(pinia) // Use Pinia
const config = useConfigStore() // Use the config store

// Create the I18n instance
const i18n = createI18n({
  legacy: false, // Disable legacy mode
  locale: config.settings.language, // Use language setting from config store
  fallbackLocale: "en",
  messages: {
    en,
    zh,
    es,
    fr,
    de,
    ja,
    ko,
    ru,
    pt
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
listen('settings-appearance-changed', (event) => {
  config.setAppearance(event.payload)
})
listen('settings-lightTheme-changed', (event) => {
  config.setLightTheme(event.payload)
})
listen('settings-darkTheme-changed', (event) => {
  config.setDarkTheme(event.payload)
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
listen('settings-settingsTabIndex-changed', (event) => {
  config.setSettingsTabIndex(event.payload)
})
listen('settings-gridSize-changed', (event) => {
  config.setGridSize(event.payload)
})
listen('settings-gridStyle-changed', (event) => {
  config.setGridStyle(event.payload)
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
  config.setFilmStripViewPreviewPosition(event.payload)
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
listen('settings-imageSearchThresholdIndex-changed', (event) => {
  config.setImageSearchThresholdIndex(event.payload)
})
listen('settings-imageSearchLimit-changed', (event) => {
  config.setImageSearchLimit(event.payload)
})
listen('settings-faceClusterThresholdIndex-changed', (event) => {
  config.setFaceClusterThresholdIndex(event.payload)
})
  
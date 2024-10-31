import { createApp } from 'vue'
import { createI18n } from 'vue-i18n'
import { createPinia } from 'pinia'
import piniaPersistedState from 'pinia-plugin-persistedstate'
import { listen } from '@tauri-apps/api/event'
import { invoke } from '@tauri-apps/api/core'
import router from '@/common/router'
import App from '@/App.vue'
import { useConfigStore } from '@/stores/configStore'
import '@/tailwind.css'

// I18n
import en from '@/locales/en.json'
import zh from '@/locales/zh.json'
import jp from '@/locales/jp.json'

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
  locale: config.language, // Use language setting from config store
  fallbackLocale: "en",
  messages: {
    en,
    zh,
    jp
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
listen('settings-language-changed', (event) => {
  console.log('Received event', event)
  config.setLanguage(event.payload)
})
listen('settings-showButtonText-changed', (event) => {
  console.log('Received event', event)
  config.setShowButtonText(event.payload)
})

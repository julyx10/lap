import { createApp } from 'vue'
import { createPinia } from 'pinia'
import piniaPersistedState from 'pinia-plugin-persistedstate';
import App from '@/App.vue'
import router from '@/common/router'
import { invoke } from '@tauri-apps/api/core'
import '@/tailwind.css'

// I18n
import { createI18n } from 'vue-i18n'
import en from '@/locales/en.json'
import zh from '@/locales/zh.json'
import jp from '@/locales/jp.json'

// I18n - create an i18n instance with the languages
// https://vue-i18n.intlify.dev/
const i18n = createI18n({
  legacy: false, // Disable legacy mode
  locale: "zh",  // Set the default language
  fallbackLocale: "en",
  messages: {
    en,
    zh,
    jp
  },
})

/// create app
const app = createApp(App)
const pinia = createPinia();
pinia.use(piniaPersistedState);

app.config.globalProperties.$invoke = invoke

app.use(pinia)
app.use(router)
app.use(i18n)

app.mount('#app')
console.log('App mounted', app)

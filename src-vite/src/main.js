import { createApp } from 'vue'
import App from './App.vue'
import router from '@/common/router'
import { invoke } from '@tauri-apps/api'
import './tailwind.css'

// Vuetify
import 'vuetify/styles'
import { createVuetify } from 'vuetify'

// Vuetify - Import all components and directives
// import * as components from 'vuetify/components'
// import * as directives from 'vuetify/directives'
// const vuetify = createVuetify({
//   components,
//   directives,
// })

// Vuetify - Only import the required components
import { VApp, VBtn } from 'vuetify/components'
const vuetify = createVuetify({
  components: {
    VApp,
    VBtn
  }
})


// I18n
import { createI18n } from 'vue-i18n'
import en from '@/locales/en.json'
import zh from '@/locales/zh.json'

// I18n - create an i18n instance with the languages
// https://vue-i18n.intlify.dev/
const i18n = createI18n({
  legacy: false, // Disable legacy mode
  locale: "en",  // Set the default language
  fallbackLocale: "en",
  messages: {
    en,
    zh
  },
})

/// create app
const app = createApp(App)
app.config.globalProperties.$invoke = invoke

app.use(router)
app.use(i18n)
app.use(vuetify)

app.mount('#app')
console.log('App mounted', app)

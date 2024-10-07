import { createApp } from 'vue'
import App from './App.vue'
import router from '@/common/router'
import { invoke } from '@tauri-apps/api'
import './tailwind.css'


// NOTE: Vuetify will add vertical scrollbars to all windows 2024-10-07

// Vuetify
// import 'vuetify/styles'
// import { createVuetify } from 'vuetify'

// Vuetify - Import all components and directives
// import * as components from 'vuetify/components'
// import * as directives from 'vuetify/directives'
// const vuetify = createVuetify({
//   components,
//   directives,
// })

// Vuetify - Only import the required components
// import { VApp, VBtn } from 'vuetify/components'
// const vuetify = createVuetify({
//   components: {
//     VApp,
//     VBtn
//   }
// })


// I18n
import { createI18n } from 'vue-i18n'
import en from '@/locales/en.json'
import zh from '@/locales/zh.json'
import { is, th } from 'date-fns/locale'

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

// Load config from localStorage
const savedConfig = JSON.parse(localStorage.getItem('appConfig')) || {
  appName: "jc-photo",
  theme: "dark",
  
  // Define the default language
  defaultLanguage: "en",
  
  // imageviewer
  imageViewer: {
    width: 800,
    height: 600,
    isFullscreen: false,
    showFileInfo: true,
  }
};

// Define global config and load saved data
app.config.globalProperties.$config = savedConfig;

app.config.globalProperties.$invoke = invoke

app.use(router)
app.use(i18n)
// app.use(vuetify)

app.mount('#app')
console.log('App mounted', app)

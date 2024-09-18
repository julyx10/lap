import { createApp } from 'vue'
import App from './App.vue'
import router from '@/common/router'
import { createI18n } from 'vue-i18n';
import { invoke } from '@tauri-apps/api';
import './style.css'

// import locales files
import en from '@/locales/en.json';
import zh from '@/locales/zh.json';

// create an i18n instance with the languages
// https://vue-i18n.intlify.dev/
const i18n = createI18n({
  legacy: false, // Disable legacy mode
  locale: "en",  // Set the default language
  fallbackLocale: "en",
  messages: {
    en,
    zh
  },
});
  
/// create app
const app = createApp(App);

// use the router
app.use(router);

// use the i18n instance
app.use(i18n);

app.config.globalProperties.$invoke = invoke;

app.mount('#app');
console.log('App mounted', app);

import { createApp } from 'vue'
import './style.css'
import App from './App.vue'
import { invoke } from '@tauri-apps/api';

const app = createApp(App);
app.config.globalProperties.$invoke = invoke;
app.mount('#app');

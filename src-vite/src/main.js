import { createApp } from 'vue'
import App from './App.vue'
import './style.css'
import { invoke } from '@tauri-apps/api';

const app = createApp(App);
app.config.globalProperties.$invoke = invoke;
app.mount('#app');

import { createApp } from 'vue'
import App from './App.vue'
import './style.css'
import { invoke } from '@tauri-apps/api';

const app = createApp(App);
app.config.globalProperties.$invoke = invoke;
app.mount('#app');

/**
 * splitter implementation
 */
const splitter = document.getElementById('splitter');
const leftPane = document.getElementById('leftPane');
const rightPane = document.getElementById('rightPane');

let isDragging = false;

splitter.addEventListener('mousedown', function(e) {
    isDragging = true;
    document.addEventListener('mousemove', onMouseMove);
    document.addEventListener('mouseup', onMouseUp);
});

function onMouseMove(e) {
    if (!isDragging) return;
    const newLeftWidth = e.clientX / window.innerWidth * 100;
    leftPane.style.width = `${newLeftWidth}%`;
    rightPane.style.width = `${100 - newLeftWidth}%`;
}

function onMouseUp() {
    isDragging = false;
    document.removeEventListener('mousemove', onMouseMove);
    document.removeEventListener('mouseup', onMouseUp);
}

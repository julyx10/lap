// src/stores/windowStore.js
import { defineStore } from 'pinia';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';

export const useWindowStore = defineStore('window', {
  state: () => ({
    position: { x: 0, y: 0 },
    size: { width: 800, height: 600 },
  }),
  actions: {
    async fetchWindowState() {
			const appWindow = getCurrentWebviewWindow()

      // Get current window position and size
      const { x, y } = await appWindow.outerPosition();
      const { width, height } = await appWindow.outerSize();
      this.position = { x, y };
      this.size = { width, height };
    },
    setPosition(x, y) {
      this.position = { x, y };
    },
    setSize(width, height) {
      this.size = { width, height };
    },
  },
});

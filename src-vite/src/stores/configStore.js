// stores/configStore.js
import { defineStore } from 'pinia';
import { ref } from 'vue';

export const useConfigStore = defineStore('configStore', {
  state: () => ({
    theme: 'dark',              // e.g., light or dark theme
    language: 'en',             // default language
    // notificationsEnabled: true, // notifications setting
    gridSize: 200,              // grid size in grid view, range 120-360
  }),
  actions: {
    setTheme(theme) {
      this.theme = theme;
    },
    setLanguage(language) {
      this.language = language;
    },
    // toggleNotifications() {
    //   this.notificationsEnabled = !this.notificationsEnabled;
    // },
    setGridSize(gridSize) {
      this.gridSize = gridSize;
    },
  },
  persist: {
    enabled: true,
    strategies: [
      {
        key: 'app-config',
        storage: localStorage,
      },
    ],
  },
});

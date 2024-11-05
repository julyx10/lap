// stores/configStore.js
import { ca } from 'date-fns/locale';
import { defineStore } from 'pinia';

export const useConfigStore = defineStore('configStore', {
  state: () => ({

    // settings
    language: 'en',             // default language
    showButtonText: false,      // show button text
    darkMode: true,             // light or dark theme

    // app
    toolbarIndex: 1,            // toolbar index
    leftPaneWidth: 300,         // left pane width

    // Content.vue
    gridSize: 200,              // grid size in grid view, range 120-360
    isFitWidth: false,          // fit width mode
    isFavorite: false,          // show favorite only
    sortingAsc: true,           // sorting order
    sortingType: 'name',        // sorting type
    showPreview: false,         // show preview
    previewPaneWidth: 300,      // preview pane width

    // albums
    albumId: null,              // selected album id
    folderId: null,             // selected folder id

    // calendar
    calendarIsMonthly: true,    // display monthly or daily calendar
    calendarSortingAsc: true,   // sorting order
    calendarYear: null,         // selected year (...2024)
    calendarMonth: null,        // selected month (1-12)
    calendarDate: null,         // selected date (1-31), -1 means selecting a month

    // location

    // people

    // cameras
    cameraMake: null,           // selected camera make
    cameraModel: null,          // selected camera model

    // ImageViewer.vue
    isFullscreen: false,   // full screen mode
  }),

  actions: {
    setLanguage(language) {
      this.language = language;
    },
    setShowButtonText(showButtonText) {
      this.showButtonText = showButtonText;
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

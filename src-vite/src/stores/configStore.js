// stores/configStore.js
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
    previewPosition: 'right',   // preview position: right, bottom
    previewPaneWidth: 30,       // preview pane width(20-80%)

    // albums
    albumId: null,              // selected album id
    albumName: null,            // selected album name
    albumPath: null,            // selected album path
    albumFolderId: null,        // selected folder id
    albumFolderName: null,      // selected folder name
    albumFolderPath: null,      // selected folder path

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

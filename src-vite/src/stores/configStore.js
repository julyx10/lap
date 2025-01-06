// stores/configStore.js
import { defineStore } from 'pinia';

export const useConfigStore = defineStore('configStore', {
  state: () => ({

    // settings - general
    language: 'en',             // default language
    showButtonText: false,      // show button text
    darkMode: true,             // light or dark theme
    debugMode: false,           // debug mode

    // settings - thumbnail
    thumbnailSize: 200,         // thumbnail size in thumbnail view, range 120-360
    thumbnailImageOption: 0,     // 0: Fit Entire Image, 1: Crop to Fill, 2: Stretch to Fill
    thumbnailPrimaryOption: 1,   // Name
    thumbnailSecondaryOption: 2, // Resolution

    // settings - image viewer
    mouseWheelMode: 0,          // 0: previous/next, 1: zoom in/out
    autoPlayInterval: 1,        // auto play interval in seconds

    // app
    toolbarIndex: 1,            // toolbar index
    leftPaneWidth: 300,         // left pane width

    // Content.vue
    sortingAsc: true,           // sorting order
    sortingType: 'name',        // sorting type
    showPreview: false,         // show preview
    previewPosition: 'right',   // preview position: right, bottom
    previewPaneWidth: 30,       // preview pane width(20-80%)

    // albums
    albumId: null,              // selected album id
    // albumName: null,            // selected album name
    // albumPath: null,            // selected album path
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
    isZoomFit: true,            // true: zoom to fit container; false: original size(scale = 1)
    isFullScreen: false,        // full screen mode
    isPinned: false,            // pinned mode
    showFileInfo: false,        // show file info
  }),

  actions: {
    // general settings
    setLanguage(language) {
      this.language = language;
    },
    setShowButtonText(showButtonText) {
      this.showButtonText = showButtonText;
    },
    
    // thumbnail settings
    setThumbnailSize(thumbnailSize) {
      this.thumbnailSize = thumbnailSize;
    },
    setThumbnailImageOption(thumbnailImageOption) {
      this.thumbnailImageOption = thumbnailImageOption;
    },
    setThumbnailPrimaryOption(thumbnailPrimaryOption) {
      this.thumbnailPrimaryOption = thumbnailPrimaryOption;
    },
    setThumbnailSecondaryOption(thumbnailSecondaryOption) {
      this.thumbnailSecondaryOption = thumbnailSecondaryOption;
    },

    // image viewer settings
    setMouseWheelMode(mouseWheelMode) {
      this.mouseWheelMode = mouseWheelMode;
    },
    setAutoPlayInterval(autoPlayInterval) {
      this.autoPlayInterval = autoPlayInterval;
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

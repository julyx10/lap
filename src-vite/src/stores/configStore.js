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
    thumbnailScalingOption: 0,  // 0: Fit Entire Image, 1: Crop to Fill, 2: Stretch to Fill
    thumbnailLabelPrimaryOption: 1,   // Name
    thumbnailLabelSecondaryOption: 2, // Resolution

    // settings - image viewer
    mouseWheelMode: 0,          // 0: previous/next, 1: zoom in/out
    autoPlayInterval: 1,        // auto play interval in seconds

    // app
    toolbarIndex: 1,            // toolbar index
    leftPaneWidth: 300,         // left pane width

    // Content.vue
    sortingType: 0,             // sorting type
    sortingDirection: 0,        // sorting direction(0: ascending, 1: descending)
    filterType: 0,              // filter type
    showPreview: false,         // show preview
    previewPosition: 'right',   // preview position: right, bottom
    previewPaneWidth: 30,       // preview pane width(20-80%)

    // move/copy to... destination folder
    destAlbumId: null,          // destination album id
    destFolderId: null,         // destination folder id
    destFolderPath: null,       // destination folder path

    // albums
    albumId: null,              // selected album id
    albumFolderId: null,        // selected folder id
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
    setThumbnailScalingOption(thumbnailScalingOption) {
      this.thumbnailScalingOption = thumbnailScalingOption;
    },
    setThumbnailLabelPrimaryOption(thumbnailLabelPrimaryOption) {
      this.thumbnailLabelPrimaryOption = thumbnailLabelPrimaryOption;
    },
    setThumbnailLabelSecondaryOption(thumbnailLabelSecondaryOption) {
      this.thumbnailLabelSecondaryOption = thumbnailLabelSecondaryOption;
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

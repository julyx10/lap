import { defineStore } from 'pinia';

export const useConfigStore = defineStore('configStore', {
  state: () => ({
    ////// Home.vue //////

    // toolbar index
    sidebarIndex: 1,            // toolbar index
    leftPaneWidth: 300,         // left pane width

    // album
    albumId: null,              // selected album id
    albumFolderId: null,        // selected folder id
    albumFolderPath: null,      // selected folder path

    // favorites
    favoriteAlbumId: null,      // favorite album id
    favoriteFolderId: null,     // favorite folder id
    favoriteFolderPath: null,   // favorite folder path

    // tags
    tagId: null,                // selected tag id

    // calendar
    calendarIsMonthly: true,    // display monthly or daily calendar
    calendarSortingAsc: true,   // sorting order
    calendarYear: null,         // selected year (...2024)
    calendarMonth: null,        // selected month (1-12)
    calendarDate: null,         // selected date (1-31), -1 means selecting a month

    // location
    locationId: null,           // selected location id
    
    // cameras
    cameraMake: null,           // selected camera make
    cameraModel: null,          // selected camera model

    // trash
    trashAlbumId: null,         // trash album id
    trashFolderId: null,        // trash folder id
    trashFolderPath: null,      // trash folder path

    ////// Content.vue //////

    // filter fileList
    searchText: '',             // filter file name
    searchFileType: 0,                // filter file type (0: all, 1: image, 2: video, 3: audio)
    sortType: 0,                // sort type
    sortOrder: 0,               // sort order(0: ascending, 1: descending)

    // preview
    showPreview: false,         // show preview
    previewPosition: 'right',   // preview position: right, bottom
    previewPaneWidth: 30,       // preview pane width(20-80%)

    // move/copy to... destination folder
    destAlbumId: null,          // destination album id
    destFolderId: null,         // destination folder id
    destFolderPath: null,       // destination folder path

    ////// ImageViewer.vue //////

    isZoomFit: true,            // true: zoom to fit container; false: original size(scale = 1)
    isFullScreen: false,        // full screen mode
    isPinned: true,             // pinned mode
    showFileInfo: false,        // show file info
    fileInfoPanelWidth: 20,     // file info panel width

    ////// Settings.vue //////

    settingsTabIndex: 0,        // settings tab index

    // settings - general
    appearance: 1,              // 0: light; 1: dark
    language: 'en',             // default language
    showButtonText: true,      // show button text
    showToolTip: true,          // show button tooltip
    showStatusBar: true,        // show status bar
    showComment: true,          // show comment
    debugMode: false,           // debug mode
    fileListPageSize: 200,      // number of file list per page

    // settings - thumbnail
    thumbnailImageSize: 256,    // thumbnail image size (small: 128, medium: 256, large: 512, extra large: 1024)
    thumbnailSize: 200,         // thumbnail size in thumbnail view, range 120-360
    thumbnailScalingOption: 0,  // 0: Fit Entire Image, 1: Crop to Fill, 2: Stretch to Fill
    thumbnailLabelPrimaryOption: 1,   // Name
    thumbnailLabelSecondaryOption: 2, // Dimension

    // settings - image viewer
    mouseWheelMode: 0,          // 0: previous/next, 1: zoom in/out
    autoPlayInterval: 1,        // auto play interval in seconds [1, 3, 5, 10, 30, 60]
  }),

  actions: {
    // general settings
    setAppearance(appearance) {
      this.appearance = appearance;
    },
    setLanguage(language) {
      this.language = language;
    },
    setShowButtonText(showButtonText) {
      this.showButtonText = showButtonText;
    },
    setShowToolTip(showToolTip) {
      this.showToolTip = showToolTip;
    },
    setShowStatusBar(showStatusBar) {
      this.showStatusBar = showStatusBar;
    },
    setShowComment(showComment) {
      this.showComment = showComment;
    },
    setDebugMode(debugMode) {
      this.debugMode = debugMode;
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
  persist: true
});

import { defineStore } from 'pinia';

export const useConfigStore = defineStore('configStore', {
  state: () => ({
    ////// Home.vue //////

    // toolbar index
    sidebarIndex: 1,            // toolbar index
    leftPaneWidth: 300,         // left pane width

    // album
    album: {
      id: null,
      folderId: null,
      folderPath: null,
    },

    // favorites
    favorite: {
      albumId: null,
      folderId: null,
      folderPath: null,
    },

    // tags
    tagId: null,                // selected tag id

    // calendar
    calendar: {
      isMonthly: true,    // display monthly or daily calendar
      sortingAsc: true,   // sorting order
      year: null,         // selected year (...2024)
      month: null,        // selected month (1-12)
      date: null,         // selected date (1-31), -1 means selecting a month
    },

    // cameras
    camera: {
      make: null,           // selected camera make
      model: null,          // selected camera model
    },

    // location
    location: {
      admin1: null,      // selected location admin1 (e.g. California)
      name: null,        // selected location name (e.g. San Francisco)
    },

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
    dest: {
      albumId: null,          // destination album id
      folderId: null,         // destination folder id
      folderPath: null,       // destination folder path
    },

    ////// ImageViewer.vue //////

    isZoomFit: true,            // true: zoom to fit container; false: original size(scale = 1)
    isFullScreen: false,        // full screen mode
    isPinned: true,             // pinned mode
    showFileInfo: false,        // show file info
    fileInfoPanelWidth: 20,     // file info panel width
    showMap: false,             // show map
    
    map: {
      zoom: 13,
      theme: 0,                 // 0: standard, 2: satellite
    },

    ////// ImageEditor.vue //////
    imageEditor: {
      cropShape: 0,             // image editor crop shape (0: Custom, 1: 1:1, 2: 1:2, 3: 2:3, 4: 3:4, 5: 9:16) 
      saveAs: 0,                // image editor save as (0: Overwrite existing file, 1: Save as new file)
      format: 0,                // image editor format (0: JPEG, 1: PNG, 2: WEBP)
      quality: 0,               // image editor quality (0: High, 1: Medium, 2: Low)
    },

    ////// Video.vue //////
    video: {
      muted: false,           // video muted
      volume: 1.0,            // video volume (0.0-1.0)
    },

    ////// Settings.vue //////

    settingsTabIndex: 0,        // settings tab index

    // settings - general
    appearance: 1,              // 0: light; 1: dark
    language: 'en',             // default language
    showHiddenAlbum: false,     // show hidden album
    showButtonText: true,       // show button text
    showToolTip: true,          // show button tooltip
    showStatusBar: true,        // show status bar
    debugMode: false,           // debug mode
    fileListPageSize: 200,      // number of file list per page
    
    // settings - grid view
    thumbnailSize: 512,         // thumbnail image size (small: 128, medium: 256, large: 512, extra large: 1024)
    grid: {
      size: 200,              // grid size, range 120-360
      scaling: 0,             // 0: Fit Entire Image, 1: Crop to Fill, 2: Stretch to Fill
      labelPrimary: 1,        // Primary label (1: Name)
      labelSecondary: 2,      // Secondary label (2: Dimension)
    },

    // settings - image viewer
    mouseWheelMode: 0,          // 0: previous/next, 1: zoom in/out
    slideShowInterval: 1,       // slide show interval in seconds [1, 3, 5, 10, 30, 60]
    autoPlayVideo: false,       // auto play video
    navigatorViewMode: 2,       // 0: Invisible, 1: Small thumbnail(120), 2: Standard thumbnail(200), 3: Large thumbnail(320)
    showComment: true,          // show comment
  }),

  actions: {
    // tab index
    setSettingsTabIndex(settingsTabIndex) {
      this.settingsTabIndex = settingsTabIndex;
    },

    // general settings
    setAppearance(appearance) {
      this.appearance = appearance;
    },
    setLanguage(language) {
      this.language = language;
    },
    setShowHiddenAlbum(showHiddenAlbum) {
      this.showHiddenAlbum = showHiddenAlbum;
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
    setDebugMode(debugMode) {
      this.debugMode = debugMode;
    },

    // video settings
    setVideoMuted(videoMuted) {
      this.video.muted = videoMuted;
    },
    setVideoVolume(videoVolume) {
      this.video.volume = videoVolume;
    },

    // grid view settings
    setGridSize(gridSize) {
      this.grid.size = gridSize;
    },
    setGridScaling(gridScaling) {
      this.grid.scaling = gridScaling;
    },
    setGridLabelPrimary(gridLabelPrimary) {
      this.grid.labelPrimary = gridLabelPrimary;
    },
    setGridLabelSecondary(gridLabelSecondary) {
      this.grid.labelSecondary = gridLabelSecondary;
    },
    // image viewer settings
    setMouseWheelMode(mouseWheelMode) {
      this.mouseWheelMode = mouseWheelMode;
    },
    setSlideShowInterval(slideShowInterval) {
      this.slideShowInterval = slideShowInterval;
    },
    setAutoPlayVideo(autoPlayVideo) {
      this.autoPlayVideo = autoPlayVideo;
    },
    setNavigatorViewMode(navigatorViewMode) {
      this.navigatorViewMode = navigatorViewMode;
    },
    setShowComment(showComment) {
      this.showComment = showComment;
    },
  },
  persist: true
});

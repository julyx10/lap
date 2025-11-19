import { defineStore } from 'pinia';

export const useConfigStore = defineStore('configStore', {
  state: () => ({
    home: {
      sidebarIndex: 1,            // toolbar index
      leftPaneWidth: 300,         // left pane width
    },

    content: {
      pageSize: 200,              // number of files per page
      layout: 0,                  // 0: grid view, 1: film strip view
      filmStripPaneHeight: 200,   // film strip pane height (px)
    },

    infoPanel: {
      show: false,               // show info panel
      tabIndex: 0,               // info panel tab index (0: file info, 1: preview)
      width: 30,                 // info panel width(20-80%)
    },

    search: {
      text: '',             // filter file name
      fileType: 0,          // filter file type (0: all, 1: image, 2: video)
      sortType: 0,          // sort type
      sortOrder: 0,         // sort order(0: ascending, 1: descending)
    },

    fileInfo: {
      showMap: true,          // show map
      mapTheme: 0,            // 0: standard, 2: satellite
    },

    // move/copy to... destination folder
    destFolder: {
      albumId: null,          // destination album id
      folderId: null,         // destination folder id
      folderPath: null,       // destination folder path
    },

    album: {
      id: null,
      folderId: null,
      folderPath: null,
    },

    favorite: {
      albumId: null,
      folderId: null,
      folderPath: null,
    },

    tag: {
      id: null,
    },

    calendar: {
      isMonthly: true,    // display monthly or daily calendar
      sortingAsc: true,   // sorting order
      year: null,         // selected year (...2024)
      month: null,        // selected month (1-12)
      date: null,         // selected date (1-31), -1 means selecting a month
    },

    camera: {
      make: null,           // selected camera make
      model: null,          // selected camera model
    },

    location: {
      admin1: null,      // selected location admin1 (e.g. California)
      name: null,        // selected location name (e.g. San Francisco)
    },

    imageViewer: {
      isZoomFit: true,            // true: zoom to fit container; false: original size(scale = 1)
      isLocked: false,            // true: locked mode; false: unlocked mode (image will not be updated)
      isFullScreen: false,        // full screen mode
      isPinned: true,             // pinned mode
    },

    imageEditor: {
      cropShape: 0,             // image editor crop shape (0: Custom, 1: 1:1, 2: 1:2, 3: 2:3, 4: 3:4, 5: 9:16) 
      saveAs: 0,                // image editor save as (0: Overwrite existing file, 1: Save as new file)
      format: 0,                // image editor format (0: JPEG, 1: PNG, 2: WEBP)
      quality: 0,               // image editor quality (0: High, 1: Medium, 2: Low)
    },

    video: {
      muted: false,           // video muted
      volume: 1.0,            // video volume (0.0-1.0)
    },

    settings: {
      tabIndex: 0,               // settings tab index (0: general, 1: grid view, 2: image viewer, 3: about)

      // general settings
      language: 'en',             // default language
      appearance: 1,              // appearance (0: light; 1: dark)
      lightTheme: 0,              // light theme color index
      darkTheme: 0,               // dark theme color index
      showButtonText: true,       // show button text
      showToolTip: true,          // show button tooltip
      showStatusBar: true,        // show status bar
      debugMode: false,           // debug mode

      // grid view settings
      thumbnailSize: 512,         // thumbnail image size (small: 128, medium: 256, large: 512, extra large: 1024)
      grid: {
        size: 200,              // grid size, range 120-360
        style: 0,               // 0: comfortable view, 1: compact view
        scaling: 0,             // 0: Fit Entire Image, 1: Crop to Fill, 2: Stretch to Fill
        labelPrimary: 1,        // Primary label (1: Name)
        labelSecondary: 2,      // Secondary label (2: Dimension)
      },
      filmStripView: {
        previewPosition: 0,      // 0: top display, 1: bottom display
      },

      // image viewer settings
      mouseWheelMode: 0,          // 0: previous/next, 1: zoom in/out
      slideShowInterval: 1,       // slide show interval in seconds [1, 3, 5, 10, 30, 60]
      autoPlayVideo: false,       // auto play video
      navigatorViewMode: 0,       // 0: Auto, 1: Always hide, 2: Always show
      navigatorViewSize: 200,     // navigator view size (120, 200, 320, 480)
      showComment: true,          // show comment
    },
  }),

  actions: {
    // tab index
    setSettingsTabIndex(settingsTabIndex) {
      this.settings.tabIndex = settingsTabIndex;
    },

    // general settings
    setAppearance(appearance) {
      this.settings.appearance = appearance;
    },
    setLightTheme(lightTheme) {
      this.settings.lightTheme = lightTheme;
    },
    setDarkTheme(darkTheme) {
      this.settings.darkTheme = darkTheme;
    },
    setLanguage(language) {
      this.settings.language = language;
    },
    setShowButtonText(showButtonText) {
      this.settings.showButtonText = showButtonText;
    },
    setShowToolTip(showToolTip) {
      this.settings.showToolTip = showToolTip;
    },
    setShowStatusBar(showStatusBar) {
      this.settings.showStatusBar = showStatusBar;
    },
    setDebugMode(debugMode) {
      this.settings.debugMode = debugMode;
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
      this.settings.grid.size = gridSize;
    },
    setGridStyle(gridStyle) {
      this.settings.grid.style = gridStyle;
    },
    setGridScaling(gridScaling) {
      this.settings.grid.scaling = gridScaling;
    },
    setGridLabelPrimary(gridLabelPrimary) {
      this.settings.grid.labelPrimary = gridLabelPrimary;
    },
    setGridLabelSecondary(gridLabelSecondary) {
      this.settings.grid.labelSecondary = gridLabelSecondary;
    },
    setfilmStripViewPreviewPosition(filmStripViewPreviewPosition) {
      this.settings.filmStripView.previewPosition = filmStripViewPreviewPosition;
    },
    // image viewer settings
    setMouseWheelMode(mouseWheelMode) {
      this.settings.mouseWheelMode = mouseWheelMode;
    },
    setSlideShowInterval(slideShowInterval) {
      this.settings.slideShowInterval = slideShowInterval;
    },
    setAutoPlayVideo(autoPlayVideo) {
      this.settings.autoPlayVideo = autoPlayVideo;
    },
    setNavigatorViewMode(navigatorViewMode) {
      this.settings.navigatorViewMode = navigatorViewMode;
    },
    setNavigatorViewSize(navigatorViewSize) {
      this.settings.navigatorViewSize = navigatorViewSize;
    },
    setShowComment(showComment) {
      this.settings.showComment = showComment;
    },
  },
  persist: true
});

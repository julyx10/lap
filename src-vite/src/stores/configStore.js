/**
 * Config Store - Global application configuration
 */
import { defineStore } from 'pinia';

export const useConfigStore = defineStore('configStore', {
  state: () => ({
    main: {
      showLeftPane: true,         // show left pane
      leftPaneWidth: 320,         // left pane width
      sidebarIndex: 0,            // toolbar index
      albumCount: 0,              // album count
      maxLibraryCount: 10,        // max library count
    },

    content: {
      showFilmStrip: false,       // false: grid view, true: film strip view
      filmStripPaneHeight: 160,   // film strip pane height (px)
    },

    leftPanel: {
      sortCount: false,           // false: default sort by name, true: sort by count
    },

    infoPanel: {
      show: false,               // show info panel
      width: 30,                 // info panel width(20-80%)
      showMap: true,             // show map
      mapTheme: 0,               // 0: standard, 2: satellite
    },

    search: {
      searchType: 0,           // 0: ai search, 1: similar image, 2: filename search
      maxSearchHistory: 20,    // max search history
      fileType: 0,              // filter file type (0: all, 1: image, 2: video)
      sortType: 0,              // sort type (default to time)
      sortOrder: 0,             // sort order(0: ascending, 1: descending)
    },

    calendar: {
      isMonthly: true,    // display monthly or daily calendar
      sortingAsc: false,   // sorting order
    },

    mediaViewer: {
      isZoomFit: true,      // true: zoom to fit container; false: original size(scale = 1)
      isPinned: true,       // pinned mode
    },

    video: {
      muted: false,           // video muted
      volume: 1.0,            // video volume (0.0-1.0)
    },

    imageEditor: {
      cropShape: 0,             // image editor crop shape (0: Custom, 1: 1:1, 2: 1:2, 3: 2:3, 4: 3:4, 5: 9:16) 
      saveAs: 0,                // image editor save as (0: Overwrite existing file, 1: Save as new file)
      format: 0,                // image editor format (0: JPEG, 1: PNG, 2: WEBP)
      quality: 0,               // jpeg quality (0: High, 1: Medium, 2: Low), [90, 80, 60]
    },

    settings: {
      tabIndex: 0,               // settings tab index (0: general, 1: grid view, 2: image view, 3: image search, 4: about)

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
        size: 160,              // grid size, range 120-360
        style: 0,               // 0: comfortable view, 1: compact view
        scaling: 0,             // 0: Fit Entire Image, 1: Crop to Fill, 2: Stretch to Fill
        labelPrimary: 1,        // Primary label (1: Name)
        labelSecondary: 2,      // Secondary label (2: Dimension)
      },
      
      // image view settings
      previewPosition: 0,         // 0: top display, 1: bottom display
      mouseWheelMode: 0,          // 0: previous/next, 1: zoom in/out
      slideShowInterval: 1,       // slide show interval in seconds [1, 3, 5, 10, 30, 60]
      navigatorViewMode: 0,       // 0: Auto, 1: Always hide, 2: Always show
      navigatorViewSize: 240,     // navigator view size (160, 240, 320, 400)
      autoPlayVideo: false,       // auto play video
      showComment: true,          // show comment

      // image search settings
      imageSearch: {
        thresholdIndex: 3,                 // image search threshold index (default is Low)
        limit: 1000,                       // image search limit
      },
      
      // face recognition settings
      face: {
        // image source
        imageSource: 0, // 0: original, 1: thumbnail
        
        // Cluster threshold index: 0=Very High, 1=High, 2=Medium, 3=Low
        clusterThresholdIndex: 2, // Default: Medium
      },
    },
  }),

  getters: {
    // Image search threshold values
    // [Very High, High, Medium, Low]
    imageSearchThresholds: () => [0.8, 0.6, 0.4, 0.25],
    
    // Cluster threshold values: cosine distance (lower = stricter, higher = looser)
    // [Very High, High, Medium, Low]
    faceClusterThresholds: () => [0.35, 0.45, 0.55, 0.65],
  },

  actions: {
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
    setSettingsTabIndex(tabIndex) {
      this.settings.tabIndex = tabIndex;
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

    // image view settings
    setFilmStripViewPreviewPosition(filmStripViewPreviewPosition) {
      this.settings.previewPosition = filmStripViewPreviewPosition;
    },
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
    
    // image search settings
    setImageSearchThresholdIndex(imageSearchThresholdIndex) {
      this.settings.imageSearch.thresholdIndex = imageSearchThresholdIndex;
    },
    setImageSearchLimit(imageSearchLimit) {
      this.settings.imageSearch.limit = imageSearchLimit;
    },

    // face recognition settings
    setFaceClusterThresholdIndex(index) {
      if (!this.settings.face) {
        this.settings.face = { clusterThresholdIndex: index };
      } else {
        this.settings.face.clusterThresholdIndex = index;
      }
    },
  },
  persist: true
});

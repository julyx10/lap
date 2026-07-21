/**
 * Config Store - Global application configuration
 */
import { defineStore } from 'pinia';
import { SIDEBAR } from '@/common/constants';

export const useConfigStore = defineStore('configStore', {
  state: () => ({
    main: {
      sidebarIndex: SIDEBAR.ALBUM, // toolbar index
      maxLibraryCount: 20,        // max library count
      selectionChunkSize: 200,    // virtual list fetch chunk size
    },

    content: {
      filmStripPaneHeight: 160,   // film strip pane height (px)
    },

    leftPanel: {
      show: true,                 // show left pane
      width: 320,                 // left pane width
    },

    collectionTray: {
      expanded: false,            // show collection tray in left pane
      height: 220,                // expanded tray height (px)
    },

    rightPanel: {
      show: false,                // show right panel
      width: 360,                 // panel width in px
      mode: 'info',               // right panel mode ('info' | 'dedup')
    },

    infoPanel: {
      showPreview: true,         // show preview thumbnail
      previewMode: 'thumbnail',  // preview section mode ('thumbnail' | 'histogram')
      previewScale: 1,           // preview thumbnail scale (1, 0.5, 0.25)
      histogramChannels: 15,     // histogram channel mask (L=1, R=2, G=4, B=8; 0=none, 15=all)
      showBasicInfo: true,       // show basic info
      showMetadata: true,        // show metadata
      showMap: true,             // show map
      mapTheme: 0,               // 0: standard, 2: satellite
    },

    search: {
      maxSearchHistory: 20,     // max search history
      fileType: 0,              // filter file type bitmask (0: all, 1: image, 2: video, 4: raw)
      sortType: 0,              // sort type (default to time)
      sortOrder: 0,             // sort order(0: ascending, 1: descending)
      groupBy: 0,               // result grouping: 0=none, 1=folder, 2=day, 3=month, 4=rating, 5=location, 6=camera, 7=lens, 8=year
    },

    calendar: {
      isMonthly: true,    // display monthly or daily calendar
    },

    camera: {
      isCamera: true,    // show cameras or lens
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
      tab: 'edit',               // image editor active tab ('edit' | 'adjust')
      custom: {
        brightness: 0,
        contrast: 0,
        saturation: 100,
        hue: 0,
        blur: 0,
        filter: '',
      },
      cropShape: 0,             // image editor crop shape (0: Custom, 1: 1:1, 2: 1:2, 3: 2:3, 4: 3:4, 5: 9:16) 
      saveAs: 0,                // image editor save as (0: Overwrite existing file, 1: Save as new file)
      format: 0,                // image editor format (0: JPEG, 1: PNG, 2: WEBP)
      quality: 0,               // jpeg quality (0: High, 1: Medium, 2: Low), [90, 80, 60]
    },

    imageViewer: {
      isSyncViewport: false,    // sync viewport
      isFullScreen: false,      // native fullscreen in image viewer window
    },

    libraryChangedVersion: 0,

    settings: {
      tabIndex: 0,               // settings tab index (0: general, 1: library, 2: browse, 3: viewer, 4: search, 5: shortcuts, 6: advanced, 7: about)

      // general settings
      language: 'en',             // default language
      appearance: 1,              // appearance (0: light; 1: dark)
      lightTheme: 0,              // light theme color index
      darkTheme: 0,               // dark theme color index
      scale: 1,                   // root font-size scale
      showToolTip: true,          // show button tooltip
      showStatusBar: true,        // show status bar
      autoCheckUpdates: true,      // automatically check for updates
      showCollections: false,      // show collection tray in left panel
      debugMode: false,           // debug mode

      // navigation settings
      folderSort: 0,              // folder_sort_options: 0=name asc, 1=name desc, 2=date asc(oldest first), 3=date desc(newest first)
      calendarSort: 0,            // 0=taken asc, 1=taken desc, 2=created asc, 3=created desc, 4=modified asc, 5=modified desc
      categorySort: 0,            // category_sort_options: 0=name asc, 1=name desc, 2=count asc, 3=count desc
      showSubfolderFiles: false,  // show subfolder files (in album folder view)
      
      // grid view settings
      thumbnailSize: 512,         // thumbnail image size (small: 128, medium: 256, large: 512, extra large: 1024)
      grid: {
        size: 160,               // grid size, range 120-360
        style: 0,                // 0: card view, 1: tile view, 2: justified view, 3: masonry view
        showFilmStrip: false,    // show filmstrip view
        scaling: 1,              // 0: Fit Entire Image, 1: Crop to Fill, 2: Stretch to Fill
        labelPrimary: 1,         // card view: primary label (1: Name)
        labelSecondary: 3,       // card view: secondary label (3: Dimension)
        thumbnailBadge: 0,       // thumbnail badge (0: empty, 1: file format, 2: ISO, 3: shutter, 4: aperture, 5: focal length, 6: exposure)
        previewPosition: 0,      // filmstrip view: preview position (0: top, 1: bottom, 2: left, 3: right)
      },
      
      // image view settings
      mouseWheelMode: 1,         // 0: previous/next, 1: zoom in/out (default)
      slideShowInterval: 1,      // slide show interval in seconds [1, 3, 5, 10, 30, 60]
      slideShowTransition: 0,    // 0: Slide, 1: Fade, 2: None
      navigatorViewMode: 0,      // 0: Auto, 1: Always show, 2: Always hide
      navigatorViewSize: 240,    // navigator view size (160, 240, 320, 400)
      viewBackground: 0,         // 0: default, 1: black, 2: dark gray, 3: medium gray, 4: light gray, 5: white
      autoPlayVideo: true,       // auto play video
      loopVideo: false,          // loop video (only effective when autoPlayVideo is off)
      // showComment: false,        // show comment
      externalImageAppPath: '',    // external image app path
      externalImageAppName: '',    // external image app display name
      externalVideoAppPath: '',    // external video app path
      externalVideoAppName: '',    // external video app display name

      // image search settings
      imageSearch: {
        model: 0,                  // 0: default English-only model, 1: multilingual model
        thresholdIndex: 3,         // image search threshold index (default is Low)
        limit: 1000,               // image search limit
      },
      
      // face recognition settings
      face: {
        enabled: false, // enable face recognition in image search
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
    setScale(scale) {
      this.settings.scale = scale;
    },
    setExternalImageAppPath(externalImageAppPath) {
      this.settings.externalImageAppPath = externalImageAppPath;
    },
    setExternalImageAppName(externalImageAppName) {
      this.settings.externalImageAppName = externalImageAppName;
    },
    setExternalVideoAppPath(externalVideoAppPath) {
      this.settings.externalVideoAppPath = externalVideoAppPath;
    },
    setExternalVideoAppName(externalVideoAppName) {
      this.settings.externalVideoAppName = externalVideoAppName;
    },
    setLanguage(language) {
      this.settings.language = language;
    },
    setShowToolTip(showToolTip) {
      this.settings.showToolTip = showToolTip;
    },
    setShowStatusBar(showStatusBar) {
      this.settings.showStatusBar = showStatusBar;
    },
    setAutoCheckUpdates(autoCheckUpdates) {
      this.settings.autoCheckUpdates = autoCheckUpdates;
    },
    setShowCollections(showCollections) {
      this.settings.showCollections = showCollections;
    },
    setDebugMode(debugMode) {
      this.settings.debugMode = debugMode;
    },
    setSettingsTabIndex(tabIndex) {
      this.settings.tabIndex = tabIndex;
    },
    setFolderSort(folderSort) {
      this.settings.folderSort = folderSort;
    },
    setCalendarSort(calendarSort) {
      this.settings.calendarSort = calendarSort;
    },
    setCategorySort(categorySort) {
      this.settings.categorySort = categorySort;
    },
    setShowSubfolderFiles(showSubfolderFiles) {
      this.settings.showSubfolderFiles = showSubfolderFiles;
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
    setGridThumbnailBadge(thumbnailBadge) {
      this.settings.grid.thumbnailBadge = thumbnailBadge;
    },
    setShowFilmStrip(showFilmStrip) {
      this.settings.grid.showFilmStrip = showFilmStrip;
    },

    // image view settings
    setFilmStripViewPreviewPosition(filmStripViewPreviewPosition) {
      this.settings.grid.previewPosition = filmStripViewPreviewPosition;
    },
    setMouseWheelMode(mouseWheelMode) {
      this.settings.mouseWheelMode = mouseWheelMode;
    },
    setSlideShowInterval(slideShowInterval) {
      this.settings.slideShowInterval = slideShowInterval;
    },
    setSlideShowTransition(slideShowTransition) {
      this.settings.slideShowTransition = slideShowTransition;
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
    setViewBackground(viewBackground) {
      this.settings.viewBackground = viewBackground;
    },
    cycleViewBackground() {
      this.settings.viewBackground = (Number(this.settings.viewBackground || 0) + 1) % 6;
    },
    // setShowComment(showComment) {
    //   this.settings.showComment = showComment;
    // },
    // image search settings
    setImageSearchThresholdIndex(imageSearchThresholdIndex) {
      this.settings.imageSearch.thresholdIndex = imageSearchThresholdIndex;
    },
    setImageSearchLimit(imageSearchLimit) {
      this.settings.imageSearch.limit = imageSearchLimit;
    },

    // face recognition settings
    setFaceEnabled(enabled) {
      if (!this.settings.face) {
        this.settings.face = { enabled, clusterThresholdIndex: 2 };
      } else {
        this.settings.face.enabled = enabled;
      }
    },
    setFaceClusterThresholdIndex(index) {
      if (!this.settings.face) {
        this.settings.face = { enabled: true, clusterThresholdIndex: index };
      } else {
        this.settings.face.clusterThresholdIndex = index;
      }
    },

    notifyLibrariesChanged() {
      this.libraryChangedVersion++;
    },

  },
  persist: true
});

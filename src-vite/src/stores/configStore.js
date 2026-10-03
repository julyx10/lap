/**
 * Config Store - Global application configuration
 */
import { defineStore } from 'pinia';
import { SIDEBAR, MAP_MARKER_SIZES } from '@/common/constants';

export const useConfigStore = defineStore('configStore', {
  state: () => ({
    main: {
      sidebarIndex: SIDEBAR.ALBUM, // toolbar index
      maxLibraryCount: 20,        // max library count
      maxCollectionCount: 100,    // max collection count
      selectionChunkSize: 200,    // virtual list fetch chunk size
    },

    content: {
      filmStripPaneHeight: 160,   // film strip pane height (px)
    },

    leftPanel: {
      show: false,                // left content panel expanded
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

    dedup: {
      activeTab: 'duplicates',   // active dedup tab ('duplicates' | 'similar')
      duplicateSetsHeight: 50,   // duplicate sets section height as a percentage
    },

    infoPanel: {
      showPreview: true,         // show preview thumbnail
      previewMode: 'thumbnail',  // preview section mode ('thumbnail' | 'histogram')
      previewScale: 1,           // preview thumbnail scale (1, 0.5, 0.25)
      histogramChannels: 15,     // histogram channel mask (L=1, R=2, G=4, B=8; 0=none, 15=all)
      showBasicInfo: true,       // show basic info
      showMetadata: true,        // show metadata
      showFujiRecipe: true,      // show fujifilm film recipe
      showMap: true,             // show map
      mapTheme: 0,               // 0: standard, 2: satellite
    },

    search: {
      maxSearchHistory: 20,     // max search history
      fileType: 0,              // filter file type bitmask (0: all, 1: image, 2: video, 4: raw)
      sortType: 0,              // sort type (default to time)
      sortOrder: 0,             // sort order(0: ascending, 1: descending)
      groupBy: 0,               // result grouping: 0=none, 1=folder, 2=day, 3=month, 4=rating, 5=location, 6=camera, 7=lens, 8=year, 9=file type, 10=culling
    },

    calendar: {
      view: 'years',      // years | months | days
    },

    camera: {
      isCamera: true,    // show cameras or lens
    },

    mediaViewer: {
      isFullScreen: false,  // remember preview fullscreen independently of ImageViewer
      isZoomFit: true,      // true: zoom to fit container; false: original size(scale = 1)
      isPinned: true,       // pinned mode
      pinnedPosition: 'top', // 'top' | 'bottom'
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
      tabIndex: 0,               // settings tab index (0: general, 1: browse, 2: grid, 3: viewer, 4: RAW, 5: search, 6: advanced, 7: shortcuts, 8: about)

      // general settings
      language: 'en',             // default language
      appearance: 1,              // appearance (0: light; 1: dark)
      lightTheme: 0,              // light theme color index
      darkTheme: 0,               // dark theme color index
      scale: 1,                   // root font-size scale
      showToolTip: true,          // show button tooltip
      showStatusBar: true,        // show status bar
      autoCheckUpdates: true,      // automatically check for updates
      debugMode: false,           // debug mode
      exiftoolPath: '',           // custom exiftool binary path

      // navigation settings
      folderSort: 0,              // folder_sort_options: 0=name asc, 1=name desc, 2=date asc(oldest first), 3=date desc(newest first)
      calendarSort: 0,            // 0=taken asc, 1=taken desc, 2=created asc, 3=created desc, 4=modified asc, 5=modified desc
      categorySort: 0,            // category_sort_options: 0=name asc, 1=name desc, 2=count asc, 3=count desc
      showSubfolderFiles: false,  // show subfolder files (in album folder view)

      // RAW display settings
      groupRawJpegPairs: false,   // group matching RAW and JPEG/HEIC files
      rawPairDisplaySource: 'jpeg', // jpeg | raw (used when pairs are grouped)
      rawPreviewSource: 'embedded', // embedded | rendered
      rawRenderBrightness: 'original', // original | brightened
      
      // grid view settings
      thumbnailSize: 512,         // gallery thumbnail quality: 256, 512, or 1024
      mapProvider: 'global',      // global | tianditu
      tiandituToken: '',
      mapMarkerSize: 64,          // map photo marker size in px
      grid: {
        sizePosition: 0,         // grid size slider position (0-1)
        style: 0,                // 0: card view, 1: tile view, 2: justified view, 3: masonry view
        viewMode: 'grid',        // grid | filmstrip | map
        scaling: 1,              // 0: Fit Entire Image, 1: Crop to Fill, 2: Stretch to Fill
        thumbnailCorners: 0,     // 0: Follow theme, 1: Square
        labelPrimary: 1,         // card view: primary label (1: Name)
        labelSecondary: 3,       // card view: secondary label (3: Dimension)
        thumbnailBadge: 0,       // thumbnail badge (0: empty, 1: file format, 2: ISO, 3: shutter, 4: aperture, 5: focal length, 6: exposure)
        previewPosition: 1,      // filmstrip view: preview position (0: top, 1: bottom, 2: left, 3: right)
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
      externalApps: {
        image: { defaultId: null, apps: [] },
        video: { defaultId: null, apps: [] },
      },

      // image search settings
      imageSearch: {
        model: 0,                  // 0: default English-only model, 1: multilingual model
        thresholdIndex: 2,         // image search threshold index (default is Standard)
      },

      // similar photos settings
      similarPhotos: {
        groupingThresholdIndex: 1, // default: Strict
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
    externalAppsFor: (state) => (kind) => state.settings.externalApps?.[kind]?.apps || [],
    defaultExternalApp: (state) => (kind) => {
      const group = state.settings.externalApps?.[kind];
      return group?.apps?.find((app) => app.id === group.defaultId) || group?.apps?.[0] || null;
    },
    // Image search threshold values: [Strict, Focused, Standard, Broad]
    imageSearchThresholds: () => [0.32, 0.29, 0.26, 0.255],

    // Fixed threshold for image-to-image "Find related photos" search.
    similarImageSearchThreshold: () => 0.7,

    // Fixed default threshold for Smart Tags.
    smartTagSearchThreshold: () => 0.25,

    // Similar photo grouping thresholds
    // [Very strict, Strict, Moderate, Relaxed]
    similarPhotoGroupingThresholds: () => [0.97, 0.93, 0.9, 0.85],
    
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
    setExternalApps(externalApps) {
      const normalizeGroup = (kind) => {
        const paths = new Set();
        const apps = [];
        for (const app of externalApps?.[kind]?.apps || []) {
          const path = String(app?.path || '').trim();
          if (!path || paths.has(path) || apps.length >= 5) continue;
          paths.add(path);
          apps.push({
            id: `${kind}:${path}`,
            name: String(app?.name || ''),
            path,
          });
        }
        const requestedDefaultId = String(externalApps?.[kind]?.defaultId || '');
        return {
          apps,
          defaultId: apps.some((app) => app.id === requestedDefaultId)
            ? requestedDefaultId
            : apps[0]?.id || null,
        };
      };
      this.settings.externalApps = {
        image: normalizeGroup('image'),
        video: normalizeGroup('video'),
      };
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
    setThumbnailSize(thumbnailSize) {
      this.settings.thumbnailSize = thumbnailSize;
    },
    setRawPairDisplaySource(source) {
      this.settings.rawPairDisplaySource = source === 'raw' ? 'raw' : 'jpeg';
    },
    setRawPreviewSource(source) {
      this.settings.rawPreviewSource = source === 'rendered' ? 'rendered' : 'embedded';
    },
    setRawRenderBrightness(brightness) {
      this.settings.rawRenderBrightness = brightness === 'brightened' ? 'brightened' : 'original';
    },
    setMapProvider(mapProvider) {
      this.settings.mapProvider = mapProvider === 'tianditu' ? 'tianditu' : 'global';
    },
    setTiandituToken(tiandituToken) {
      this.settings.tiandituToken = String(tiandituToken || '').trim();
    },
    setMapMarkerSize(mapMarkerSize) {
      const size = Number(mapMarkerSize);
      this.settings.mapMarkerSize = MAP_MARKER_SIZES.includes(size) ? size : 64;
    },
    setGridStyle(gridStyle) {
      this.settings.grid.style = gridStyle;
    },
    setGridScaling(gridScaling) {
      this.settings.grid.scaling = gridScaling;
    },
    setGridThumbnailCorners(thumbnailCorners) {
      this.settings.grid.thumbnailCorners = thumbnailCorners;
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
    setImageSearchModel(imageSearchModel) {
      this.settings.imageSearch.model = imageSearchModel;
    },
    setImageSearchThresholdIndex(imageSearchThresholdIndex) {
      this.settings.imageSearch.thresholdIndex = imageSearchThresholdIndex;
    },
    setSimilarPhotoGroupingThresholdIndex(index) {
      if (!this.settings.similarPhotos) this.settings.similarPhotos = { groupingThresholdIndex: 1 };
      this.settings.similarPhotos.groupingThresholdIndex = index;
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
  persist: {
    serializer: {
      serialize: JSON.stringify,
      deserialize: (value) => {
        const state = JSON.parse(value);
        const settings = state?.settings;
        if (settings) {
          settings.rawPreviewSource = settings.rawPreviewSource === 'rendered' ? 'rendered' : 'embedded';
          settings.rawRenderBrightness = settings.rawRenderBrightness === 'brightened' ? 'brightened' : 'original';
          settings.rawPairDisplaySource = settings.rawPairDisplaySource === 'raw' ? 'raw' : 'jpeg';
        }
        return state;
      },
    },
  }
});

// sidebar index enum
export const SIDEBAR = {
  LIBRARY: 0,
  ALBUM: 1,
  SMART_ALBUM: 2,
  SEARCH: 3,
  TAG: 4,
  CALENDAR: 5,
  PERSON: 6,
  LOCATION: 7,
  CAMERA: 8,
} as const;

export type Sidebar = (typeof SIDEBAR)[keyof typeof SIDEBAR];

// library panel item enum
export const LIB_ITEM = {
  ALL: 'all-files',
  FAV: 'favorites',
  TODAY: 'on-this-day',
  RATINGS: 'ratings',
  CULLING: 'culling',
  SUBJECTS: 'subjects',
} as const;

export type LibItem = (typeof LIB_ITEM)[keyof typeof LIB_ITEM];

// rating filter enum
export const RATE = {
  ALL: -2,
  NONE: -1,
  UNRATED: 0,
} as const;

export const CULLING = {
  PICK: 'pick',
  REJECT: 'reject',
  UNREVIEWED: 'unreviewed',
} as const;

// Thumbnail badge selection
export const THUMBNAIL_BADGE = {
  EMPTY: 0,
  FILE_FORMAT: 1,
  ISO: 2,
  SHUTTER_SPEED: 3,
  APERTURE: 4,
  FOCAL_LENGTH: 5,
  EXPOSURE: 6,
} as const;

export type ThumbnailBadge = (typeof THUMBNAIL_BADGE)[keyof typeof THUMBNAIL_BADGE];

// date sort enum
export const DATE_SORT = {
  TAKEN_DESC: 1,
} as const;

// group by enum
export const GROUP = {
  NONE: 0,
  FOLDER: 1,
  DAY: 2,
  MONTH: 3,
  RATING: 4,
  LOCATION: 5,
  CAMERA: 6,
  LENS: 7,
  YEAR: 8,
  FILE_TYPE: 9,
  CULLING: 10,
} as const;

export type Group = (typeof GROUP)[keyof typeof GROUP];

export const SIMILAR_SCAN = {
  PAGE_SIZE: 100,
  LARGE_RESULT_THRESHOLD: 10_000,
  THUMBNAIL_LIMIT: 19,
} as const;

// map photo marker size presets (px)
export const MAP_MARKER_SIZES: readonly number[] = [48, 64, 88, 120];

// Settings window tab ids — single source of truth. Use these names (never raw
// literals) wherever a settings tab is opened or matched, so reordering tabs
// only requires editing this map and the settingsTabs list in Settings.vue.
export const SETTINGS_TAB = {
  GENERAL: 0,
  BROWSE: 1, // deprecated: merged into General; value kept reserved so a persisted tabIndex=1 falls back safely (do not reuse)
  GRID: 2,
  IMAGE_VIEW: 3,
  RAW: 4,
  IMAGE_SEARCH: 5,
  ADVANCED: 6,
  SHORTCUTS: 7,
  ABOUT: 8,
  METADATA: 9,
} as const;

export type SettingsTab = (typeof SETTINGS_TAB)[keyof typeof SETTINGS_TAB];

// UI limits, dimensions and interaction timing
export const SELECTED_THUMBNAIL_LIMIT = 15;
export const RIGHT_PANEL_MIN_WIDTH = 160;
export const PANEL_ANIMATION_MS = 200;
export const MONTAGE_MAX_PHOTOS = 100;
export const LARGE_BATCH_CONFIRM_THRESHOLD = 1000;
export const EXTERNAL_OPEN_WARNING_THRESHOLD = 100;
export const SETTINGS_BASE_WIDTH = 640;
export const SETTINGS_BASE_HEIGHT = 640;
export const MONTAGE_BACKGROUND_PRESETS = ['#ffffff', '#f5f0e6', '#202020'];
// Export size in pixels, landscape; print formats use 300 dpi.
export const MONTAGE_PAGE_SIZES = {
  a4: [3508, 2480],
  photo: [1772, 1181],
  screen: [3840, 2160],
  square: [3000, 3000],
} as const;
export const GESTURE_LOCK_THRESHOLD = 10;
export const HORIZONTAL_NAV_THRESHOLD = 100;
export const MOUSE_DRAG_NAV_THRESHOLD = 120;
export const PLAYBACK_DEADLINE_MS = 30_000;
export const DIRECT_PLAYBACK_TIMEOUT_MS = 8000;
export const LOADING_OVERLAY_DELAY_MS = 500;
export const OVERLAY_CLICK_MOVE_TOLERANCE = 4;
export const MAX_EXTERNAL_APPS = 5;
export const DEDUP_THUMBNAIL_PAGE_SIZE = 100;
export const TOOLTIP_GAP = 4;
export const TOOLTIP_PADDING = 8;
export const TOOLTIP_AUTO_HIDE_MS = 3000;
export const GROUP_HEADER_HEIGHT = 40;
export const PERSON_PAGE_SIZE = 100;
export const MAP_DETAIL_ZOOM = 13;
export const MAP_DETAIL_LIMIT = 500;
export const MAP_MARKER_FADE_MS = 200;
// Cap single-point bounds to retain map context and supported raster detail.
export const MAP_SINGLE_POINT_FIT_ZOOM = 13;
export const MAX_TOASTS_PER_PLACEMENT = 3;

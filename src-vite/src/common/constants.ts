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
  MAP: 9,
} as const;

export type Sidebar = (typeof SIDEBAR)[keyof typeof SIDEBAR];

// library panel item enum
export const LIB_ITEM = {
  ALL: 'all-files',
  FAV: 'favorites',
  TODAY: 'on-this-day',
  RATINGS: 'ratings',
  SUBJECTS: 'subjects',
} as const;

export type LibItem = (typeof LIB_ITEM)[keyof typeof LIB_ITEM];

// rating filter enum
export const RATE = {
  ALL: -2,
  NONE: -1,
  UNRATED: 0,
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
} as const;

export type Group = (typeof GROUP)[keyof typeof GROUP];

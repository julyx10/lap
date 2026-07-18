import { format } from 'date-fns';
import { open as openDialog } from '@tauri-apps/plugin-dialog';
import { convertFileSrc } from '@tauri-apps/api/core';
import { useUIStore } from '@/stores/uiStore';

/// get the current operating system (mac, win, linux, or '')
export function getOS() {
  const userAgent = navigator.userAgent;

  if (userAgent.includes('Mac')) {
    return 'mac';
  } else if (userAgent.includes('Win')) {
    return 'win';
  } else if (userAgent.includes('Linux') || userAgent.includes('X11')) {
    return 'linux';
  } else {
    return '';
  }
}

export const isMac = getOS() === 'mac';
export const isWin = getOS() === 'win';
export const isLinux = getOS() === 'linux';
export const separator = isWin ? '\\' : '/';

// scale values for window size and font size
export const SCALE_VALUES = [0.8, 0.9, 1, 1.1, 1.2];

/// set the theme
export function setTheme(appearance: number, themeId: number) {
  const theme = appearance === 0 ? [
    "light",
    "cupcake",
    "bumblebee",
    "emerald",
    "corporate",
    "retro",
    "cyberpunk",
    "valentine",
    "garden",
    "lofi",
    "pastel",
    "fantasy",
    "wireframe",
    "cmyk",
    "autumn",
    "acid",
    "lemonade",
    "winter",
    "nord",
    "caramellatte",
    "silk"
  ][themeId] || 'light' : [
    "dark",
    "synthwave",
    "halloween",
    "forest",
    "aqua",
    "black",
    "luxury",
    "dracula",
    "business",
    "night",
    "coffee",
    "dim",
    "sunset",
    "abyss"
  ][themeId] || 'dark';

  document.documentElement.setAttribute('data-theme', theme);
}

/// get the select options for a dropdown list
export function getSelectOptions(options: string[]): { label: string, value: number }[] {
  const result = [];
  for (let i = 0; options && i < options.length; i++) {
    result.push({ label: options[i], value: i });
  }
  return result;
}

/// get the file extension
export function getFileExtension(fileName: string): string {
  return fileName.split('.').pop() || '';
}

/// get the seconds of slide show interval
export function getSlideShowInterval(interval: number): number {
  return [1, 3, 5, 10, 15, 30][interval] || 1;
}

/// get days elapsed since the timestamp
export function getDaysElapsed(timestamp: number): number {
  if (!timestamp) {
    return 0;
  }
  const currentTimestamp = Date.now() / 1000;
  const diff = currentTimestamp - timestamp;
  return Math.floor(diff / (60 * 60 * 24));
}

/// format timestamp to string
export function formatTimestamp(timestamp: number, formatStr: string): string {
  if (!timestamp || isNaN(timestamp)) return '';
  try {
    return format(new Date(timestamp * 1000), formatStr);
  } catch (e) {
    return '';
  }
}

/// format relative time string using i18n keys
export function formatRelativeTime(timestamp: number, t: (key: string, data?: any) => string): string {
  if (!timestamp || isNaN(timestamp)) return '';
  
  const now = Date.now() / 1000;
  const diff = now - timestamp;
  const absDiff = Math.abs(diff);

  if (absDiff < 10) {
    return t('format.relative_time.just_now');
  }
  if (absDiff < 60) {
    return t('format.relative_time.seconds', { count: Math.floor(absDiff) });
  }
  const minutes = Math.floor(absDiff / 60);
  if (minutes < 60) {
    return t('format.relative_time.minutes', { count: minutes });
  }
  const hours = Math.floor(minutes / 60);
  if (hours < 24) {
    return t('format.relative_time.hours', { count: hours });
  }
  const days = Math.floor(hours / 24);
  if (days < 7) {
    return t('format.relative_time.days', { count: days });
  }
  const weeks = Math.floor(days / 7);
  if (weeks < 4) {
    return t('format.relative_time.weeks', { count: weeks });
  }
  const months = Math.floor(days / 30);
  if (months < 12) {
    return t('format.relative_time.months', { count: months });
  }
  const years = Math.floor(days / 365);
  return t('format.relative_time.years', { count: years });
}

/// format date to string
export function formatDate(year: number, month: number, date: number, formatStr: string): string {
  try {
    return format(new Date(year, month - 1, date), formatStr);
  } catch (e) {
    return '';
  }
}

/// get the date range of a month
export function getCalendarDateRange(year: number, month: number, date: number) {
  let startDate = 0;
  let endDate = 0;

  if (month === -1) { // -1 means selecting a year
    startDate = new Date(year, 0, 1).getTime() / 1000;
    endDate = new Date(year + 1, 0, 1).getTime() / 1000;
  }
  else if (date === -1) { // -1 means selecting a month
    startDate = new Date(year, month - 1, 1).getTime() / 1000;
    endDate = new Date(year, month, 1).getTime() / 1000;
  }
  else {  // otherwise, get files by date
    startDate = new Date(year, month - 1, date).getTime() / 1000;
    endDate = new Date(year, month - 1, date + 1).getTime() / 1000;
  }
  return [startDate, endDate];
}

/// format file size to string
export function formatFileSize(bytes: number): string {
  if (bytes == null) return '';
  if (bytes === 0) return '0 KB';

  const sizes = ['KB', 'MB', 'GB', 'TB'];
  const i = Math.max(Math.floor(Math.log(bytes) / Math.log(1024)) - 1, 0);
  const fileSize = bytes / Math.pow(1024, i + 1);
  return i === 0 ? `${fileSize.toFixed(0)} ${sizes[i]}` : `${fileSize.toFixed(2)} ${sizes[i]}`;
}

/// format dimension text (width x height - pixel count)
export function formatDimensionText(
  width: number,
  height: number,
  showRatio: boolean = false
): string {
  if (!Number.isFinite(width) || !Number.isFinite(height) || width <= 0 || height <= 0) return '';

  const pixel = width * height;
  let pixelText: string;
  if (pixel >= 1_000_000) {
    pixelText = `(${(pixel / 1_000_000).toFixed(1)} MP)`;
  } else {
    pixelText = `(${pixel.toLocaleString()} px)`;
  }

  const ratioText = showRatio ? formatAspectRatio(width, height) : null;
  const ratioStr = ratioText ? ` · ${ratioText}` : '';

  return `${width} × ${height}${ratioStr} ${pixelText}`;
}

// ---- internal helpers ----

const COMMON_RATIOS: { w: number; h: number; label: string }[] = [
  { w: 1,  h: 1,  label: '1:1'   },
  { w: 2,  h: 1,  label: '2:1'   },
  { w: 1,  h: 2,  label: '1:2'   },
  { w: 4,  h: 3,  label: '4:3'   },
  { w: 3,  h: 4,  label: '3:4'   },
  { w: 3,  h: 2,  label: '3:2'   },
  { w: 2,  h: 3,  label: '2:3'   },
  { w: 16, h: 9,  label: '16:9'  },
  { w: 9,  h: 16, label: '9:16'  },
  { w: 16, h: 10, label: '16:10' },
  { w: 10, h: 16, label: '10:16' },
  { w: 5,  h: 4,  label: '5:4'   },
];

const FLOAT_EPSILON = 0.001;
const MAX_SIMPLIFIED_DENOMINATOR = 20;

function gcd(a: number, b: number): number {
  if (!Number.isFinite(a) || !Number.isFinite(b)) return 1;
  return b === 0 ? a : gcd(b, a % b);
}

function formatAspectRatio(width: number, height: number): string | null {
  const actualRatio = width / height;

  for (const { w, h, label } of COMMON_RATIOS) {
    if (Math.abs(actualRatio - w / h) < FLOAT_EPSILON) {
      return label;
    }
  }

  const divisor = gcd(width, height);
  const sw = width / divisor;
  const sh = height / divisor;
  if (sw <= MAX_SIMPLIFIED_DENOMINATOR && sh <= MAX_SIMPLIFIED_DENOMINATOR) {
    return `${sw}:${sh}`;
  }

  return `${actualRatio.toFixed(2)}:1`;
}

/// format duration to string
export function formatDuration(seconds: number): string {
  const hours = Math.floor(seconds / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);
  const secs = seconds % 60;

  if (hours > 0) {
    return `${hours}:${String(minutes).padStart(2, '0')}:${String(secs).padStart(2, '0')}`;
  } else {
    return `${minutes}:${String(secs).padStart(2, '0')}`;
  }
}

/// format camera 
export function formatCameraInfo(make: string, model: string): string {
  if (!make && !model) return "";

  if (!make) return model;
  if (!model) return make;

  if (model.toLowerCase().includes(make.toLowerCase())) {
    return model;
  }

  return `${make} ${model}`;
}

/// Format EXIF numeric values for display without reducing the precision stored in the database.
export function formatCaptureSettingValue(value: string | null | undefined): string {
  if (!value) return '';

  return String(value).replace(/-?\d+(?:\.\d+)?/g, (numberText) => {
    const number = Number(numberText);
    if (!Number.isFinite(number)) return numberText;
    return Number(number.toFixed(2)).toString();
  });
}

/// format capture settings to string
export function formatCaptureSettings(focal_length: string, exposure_time: string, f_number: string, iso_speed: string, exposure_bias: string): string {
  let result = '';
  result += focal_length ? formatCaptureSettingValue(focal_length) : '';
  result += exposure_time ? `, ${formatCaptureSettingValue(exposure_time)}` : '';
  result += f_number ? `, ${formatCaptureSettingValue(f_number)}` : '';
  result += iso_speed ? `, ISO ${formatCaptureSettingValue(iso_speed)}` : '';
  result += exposure_bias ? `, ${formatCaptureSettingValue(exposure_bias)}` : '';

  // remove the first ',' if it exists
  if (result[0] === ',' && result.length > 1) {
    result = result.substring(1);
  }

  return result;
}

/// get full path
export function getFullPath(path: string, name: string): string {
  return path + separator + name;
}

/// get file folder path
export function getFolderPath(filepath: string | null | undefined): string {
  if (!filepath) {
    return '';  // Return empty string for null/undefined filepath
  }
  const lastSlashIndex = filepath.lastIndexOf(separator);
  if (lastSlashIndex === -1) {
    return '';  // No folder part, return an empty string
  }
  return filepath.substring(0, lastSlashIndex);
}

/// get file folder name
export function getFolderName(path: string): string {
  const lastSlashIndex = path.lastIndexOf(separator);
  if (lastSlashIndex === -1) {
    return path;
  }
  return path.substring(lastSlashIndex + 1);
}

export function normalizePathForCompare(path: string): string {
  if (!path) return '';
  const unified = separator === '\\'
    ? path.replace(/\//g, '\\')
    : path.replace(/\\/g, '/');
  const trimmed = unified.replace(/[\\/]+$/, '');
  return separator === '\\' ? trimmed.toLowerCase() : trimmed;
}

export function isWithinRootPath(path: string, rootPath: string): boolean {
  const normalizedPath = normalizePathForCompare(path);
  const normalizedRoot = normalizePathForCompare(rootPath);
  if (!normalizedPath || !normalizedRoot) return false;
  return normalizedPath === normalizedRoot || normalizedPath.startsWith(`${normalizedRoot}${separator}`);
}

export function buildFolderBreadcrumbs(
  folderPath: string,
  rootPath = '',
  rootLabel = ''
): Array<{ label: string; path: string }> {
  if (!folderPath) return [];

  if (!rootPath || !isWithinRootPath(folderPath, rootPath)) {
    return [{ label: getFolderName(folderPath), path: folderPath }];
  }

  const normalizedRootPath = rootPath.replace(/[\\/]+$/, '');
  const items: Array<{ label: string; path: string }> = [
    { label: rootLabel || getFolderName(normalizedRootPath), path: normalizedRootPath }
  ];
  const relative = folderPath.slice(normalizedRootPath.length).split(separator).filter(Boolean);
  let currentPath = normalizedRootPath;
  for (const segment of relative) {
    currentPath = `${currentPath}${separator}${segment}`;
    items.push({ label: segment, path: currentPath });
  }
  return items;
}

export function formatFolderBreadcrumb(
  folderPath: string,
  rootPath = '',
  rootLabel = ''
): string {
  return buildFolderBreadcrumbs(folderPath, rootPath, rootLabel)
    .map(item => item.label)
    .join(' > ');
}

let _thumbLibraryId = 'default';
const THUMBNAIL_DATA_URL_CACHE_MAX_BYTES = 96 * 1024 * 1024;
const thumbnailDataUrlCache = new Map<string, { dataUrl: string; bytes: number }>();
const thumbnailDataUrlInflight = new Map<string, Promise<string>>();
let thumbnailDataUrlCacheBytes = 0;

function getThumbnailCacheKey(fileId: number, thumbnailSize = 0): string {
  return `${_thumbLibraryId}:${thumbnailSize}:${fileId}`;
}

function estimateDataUrlBytes(dataUrl: string): number {
  return dataUrl.length * 2;
}

function touchThumbnailCacheEntry(key: string, dataUrl: string) {
  const existing = thumbnailDataUrlCache.get(key);
  if (existing) {
    thumbnailDataUrlCacheBytes -= existing.bytes;
    thumbnailDataUrlCache.delete(key);
  }

  const bytes = estimateDataUrlBytes(dataUrl);
  thumbnailDataUrlCache.set(key, { dataUrl, bytes });
  thumbnailDataUrlCacheBytes += bytes;

  while (thumbnailDataUrlCacheBytes > THUMBNAIL_DATA_URL_CACHE_MAX_BYTES) {
    const oldestKey = thumbnailDataUrlCache.keys().next().value;
    if (!oldestKey) break;
    const oldest = thumbnailDataUrlCache.get(oldestKey);
    if (oldest) {
      thumbnailDataUrlCacheBytes -= oldest.bytes;
    }
    thumbnailDataUrlCache.delete(oldestKey);
  }
}

export function setThumbLibraryId(id: string) {
  const nextId = id || 'default';
  if (_thumbLibraryId !== nextId) {
    thumbnailDataUrlCache.clear();
    thumbnailDataUrlInflight.clear();
    thumbnailDataUrlCacheBytes = 0;
  }
  _thumbLibraryId = nextId;
}

export function getCachedThumbnailDataUrl(
  fileId: number | null | undefined,
  thumbnailSize = 0
): string {
  if (!fileId || fileId <= 0) return '';
  const key = getThumbnailCacheKey(fileId, thumbnailSize);
  const cached = thumbnailDataUrlCache.get(key);
  if (!cached) return '';
  touchThumbnailCacheEntry(key, cached.dataUrl);
  return cached.dataUrl;
}

export function setCachedThumbnailDataUrl(
  fileId: number | null | undefined,
  dataUrl: string,
  thumbnailSize = 0
) {
  if (!fileId || fileId <= 0 || !dataUrl.startsWith('data:image/')) return;
  touchThumbnailCacheEntry(getThumbnailCacheKey(fileId, thumbnailSize), dataUrl);
}

export function clearCachedThumbnailDataUrl(fileId: number | null | undefined, thumbnailSize = 0) {
  if (!fileId || fileId <= 0) return;
  const key = getThumbnailCacheKey(fileId, thumbnailSize);
  const existing = thumbnailDataUrlCache.get(key);
  if (existing) {
    thumbnailDataUrlCacheBytes -= existing.bytes;
  }
  thumbnailDataUrlCache.delete(key);
  thumbnailDataUrlInflight.delete(key);
}

export function getThumbnailDataUrlInflight(
  fileId: number | null | undefined,
  thumbnailSize = 0
): Promise<string> | undefined {
  if (!fileId || fileId <= 0) return undefined;
  return thumbnailDataUrlInflight.get(getThumbnailCacheKey(fileId, thumbnailSize));
}

export function setThumbnailDataUrlInflight(
  fileId: number | null | undefined,
  thumbnailSize: number,
  promise: Promise<string>
): Promise<string> {
  if (!fileId || fileId <= 0) return promise;
  const key = getThumbnailCacheKey(fileId, thumbnailSize);
  thumbnailDataUrlInflight.set(key, promise);
  promise.finally(() => {
    if (thumbnailDataUrlInflight.get(key) === promise) {
      thumbnailDataUrlInflight.delete(key);
    }
  });
  return promise;
}

export function getThumbUrl(
  fileId: number | null | undefined,
  bustCache = false,
  thumbnailSize = 0
): string {
  if (!fileId || fileId <= 0) return '';
  if (isWin && !bustCache) {
    const cached = getCachedThumbnailDataUrl(fileId, thumbnailSize);
    if (cached) return cached;
  }
  const scheme = isWin ? 'http://thumb.localhost' : 'thumb://localhost';
  const base = `${scheme}/${_thumbLibraryId}/${fileId}`;
  return bustCache ? `${base}?t=${Date.now()}` : base;
}

export function getPreviewUrl(
  fileId: number | null | undefined,
  filePath?: string | null,
  bustCache = false,
): string {
  if (!fileId || fileId <= 0) return '';
  const scheme = isWin ? 'http://preview.localhost' : 'preview://localhost';
  const base = `${scheme}/${_thumbLibraryId}/${fileId}`;

  if (bustCache) {
    return `${base}?t=${Date.now()}`;
  }

  if (filePath) {
    const uiStore = useUIStore();
    const version = uiStore.getFileVersion(filePath);
    if (version > 0) {
      return `${base}?v=${version}`;
    }
  }

  return base;
}

export function shouldUseBackendPreview(filePath = '', fileType = 0): boolean {
  if (!filePath) return false;
  if (Number(fileType) === 3) return true;

  const extension = getFileExtension(filePath).toLowerCase();
  if (isLinux && extension === 'avif') {
    return true;
  }
  return [
    'tif', 'tiff', 'jxl', 'heic', 'heif', 'hif',
    'exr', 'hdr', 'rgbe', 'psd', 'jp2', 'j2k', 'j2c', 'jpc', 'jpf', 'jpx',
    'dpx', 'dds', 'tga', 'qoi', 'fits', 'fit', 'fts'
  ].includes(extension);
}

export function getThumbnailDataUrl(
  thumb: { file_id?: number | null; error_code?: number | null; thumb_data_base64?: string | null } | null | undefined,
  placeholder = '',
  bustCache = false,
  thumbnailSize = 0,
  filePath?: string | null
): string {
  if (!thumb || thumb.file_id == null || thumb.file_id <= 0) {
    return placeholder;
  }
  if (thumb.error_code === 2) {
    if (filePath) return getAssetSrc(filePath);
    return getThumbUrl(thumb.file_id, bustCache, thumbnailSize) || placeholder;
  }
  if (thumb.error_code !== 0) {
    return placeholder;
  }
  if (thumb.thumb_data_base64) {
    const mimeType = thumb.thumb_data_base64.startsWith('iVBORw0KGgo') ? 'image/png' : 'image/jpeg';
    const dataUrl = `data:${mimeType};base64,${thumb.thumb_data_base64}`;
    setCachedThumbnailDataUrl(thumb.file_id, dataUrl, thumbnailSize);
    return dataUrl;
  }
  return getThumbUrl(thumb.file_id, bustCache, thumbnailSize) || placeholder;
}

export function getRelativePath(path: string, basePath: string): string {
  if (!path || !basePath || !isWithinRootPath(path, basePath)) {
    return '';
  }
  const normalizedBasePath = basePath.replace(/[\\/]+$/, '');
  const relativeParts = path.slice(normalizedBasePath.length).split(separator).filter(Boolean);
  return relativeParts.join(' > ');
}

/// extract the name and the extension from a file name
export function extractFileName(fileName: string): { name: string; ext: string } {
  const idx = fileName.lastIndexOf('.');
  return idx <= 0
    ? { name: fileName, ext: '' }
    : { name: fileName.slice(0, idx), ext: fileName.slice(idx + 1) };
}

/// combine the name and the extension to a file name
export function combineFileName(name: string, ext: string): string {
  return ext ? `${name}.${ext}` : name;
}

/// shorten a filename while preserving its extension
export function shortenFilename(fileName: string, maxLength = 16): string {
  if (!fileName) {
    return '';
  }
  if (fileName.length <= maxLength) {
    return fileName;
  }

  const extIndex = fileName.lastIndexOf('.');
  const hasExt = extIndex !== -1;

  if (!hasExt) {
    const keep = maxLength - 3;
    const front = Math.ceil(keep / 2);
    const back = Math.floor(keep / 2);
    return fileName.substring(0, front) + '...' + fileName.substring(fileName.length - back);
  }

  const name = fileName.substring(0, extIndex);
  const ext = fileName.substring(extIndex);

  const keep = maxLength - ext.length - 3;
  if (keep <= 0) {
    return fileName.substring(0, maxLength - 3) + '...';
  }

  const front = Math.ceil(keep / 2);
  const back = Math.floor(keep / 2);
  return name.substring(0, front) + '...' + name.substring(name.length - back) + ext;
}

// validate the file or folder name
export const isValidFileName = (name: string) => {
  const invalidChars = /[\\/:*?"<>|]/;
  return !invalidChars.test(name);
};

// Function to select a folder
export async function openFolderDialog(title?: string) {
  const selected = await openDialog({
    title: title || 'Select a folder',
    directory: true,  // Enables folder selection
    multiple: false,  // Allows selecting only one folder
  });

  if (selected) {
    console.log('Selected folder:', selected);
    return selected;
  } else {
    console.log('No folder selected.');
  }
  return null;
}

// compare two strings in different languages
export function localeComp(lang: string, str1: string, str2: string) {
  const localeMap = {
    'zh': 'zh-Hans-CN', // chinese
    'en': 'en-US',      // english
  };

  const locale = localeMap[lang as keyof typeof localeMap] || 'en-US';
  if (locale === 'en-US') {
    return str1.localeCompare(str2, undefined, { numeric: true });
  } else {
    return str1.localeCompare(str2, locale, { numeric: true });
  }
};

// scroll to the folder
export function scrollToFolder(folderId: number) {
  const folderElement = document.getElementById(`folder-${folderId}`);
  if (folderElement) {
    folderElement.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
  }
}

// get image file asset source url with version number
export function getAssetSrc(filePath: string): string {
  if (!filePath) {
    return '';
  }
  const uiStore = useUIStore();
  const version = uiStore.getFileVersion(filePath);
  const assetUrl = convertFileSrc(filePath);
  return version > 0 ? `${assetUrl}?v=${version}` : assetUrl;
}

// get country name from country code
export function getCountryName(cc: string, lang: string = 'en'): string {
  if (!cc) return '';
  try {
    const regionNames = new Intl.DisplayNames([lang], { type: 'region' });
    return regionNames.of(cc) || cc;
  } catch (e) {
    return cc;
  }
}

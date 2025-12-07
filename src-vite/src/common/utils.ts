import { format } from 'date-fns';
import { open as openDialog } from '@tauri-apps/plugin-dialog';
import { convertFileSrc } from '@tauri-apps/api/core';
import { useUIStore } from '@/stores/uiStore';

/// get the current operating system (mac, win, or '')
export function getOS() {
  const userAgent = navigator.userAgent;

  if (userAgent.includes('Mac')) {
    return 'mac';
  } else if (userAgent.includes('Win')) {
    return 'win';
  } else {
    return '';
  }
}

export const isMac = getOS() === 'mac';
export const isWin = getOS() === 'win';
export const separator = isWin ? '\\' : '/';

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
  if(!timestamp) {
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
export function formatDimensionText(width: number, height: number): string {
  if (width > 0 && height > 0) {
    const pixel = width * height;
    if (pixel > 1_000_000) {
      return `${width} x ${height} (${(pixel / 1_000_000).toFixed(1)} MP)`;
    } else if (pixel > 1_000) {
      return `${width} x ${height} (${(pixel / 1_000).toFixed(1)} KP)`;
    } else {
      return `${width} x ${height} (${pixel} P)`;
    }
  } else {
    return '';
  }
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

/// format capture settings to string
export function formatCaptureSettings(focal_length: string, exposure_time: string, f_number: string, iso_speed: string, exposure_bias: string): string {
  let result = '';
  result += focal_length ? `${focal_length}` : '';
  result += exposure_time ? `, ${exposure_time}` : '';
  result += f_number ? `, ${f_number}` : '';
  result += iso_speed ? `, ISO ${iso_speed}` : '';
  result += exposure_bias ? `, ${exposure_bias}` : '';

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

export function getRelativePath(path: string, basePath: string): string {
  return path.replace(basePath, '').split(separator).join(' > ');;
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
export async function openFolderDialog() {
  const selected = await openDialog({
    directory: true,  // Enables folder selection
    multiple: false,  // Allows selecting only one folder
  });

  if (selected ) {
    console.log('Selected folder:', selected );
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
    'ja': 'ja-JP',      // japanese
    'en': 'en-US',      // english
  };

  const locale = localeMap[lang as keyof typeof localeMap] || 'en-US';
  if (locale === 'en-US') {
    return str1.localeCompare(str2);
  } else {
    return str1.localeCompare(str2, locale);
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

import { format } from 'date-fns';
import { open as openDialog } from '@tauri-apps/plugin-dialog';
import { useConfigStore } from '@/stores/configStore';

/// config store
export const config = useConfigStore();

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
export function setTheme(themeId: number) {
  const htmlElement = document.documentElement;
  const theme = [
    'light', 'dark', 'purple'
  ][themeId];
  htmlElement.setAttribute('data-theme', theme);
}

/// get the seconds of auto play interval
export function getPlayInterval(): number {
  return [1, 3, 5, 10, 15, 30][config.autoPlayInterval] || 1;
}

/// format timestamp to string
export function formatTimestamp(timestamp: number, formatStr: string): string {
  return format(new Date(timestamp * 1000), formatStr);
}

/// format date to string
export function formatDate(year: number, month: number, date: number, formatStr: string): string {
  return format(new Date(year, month - 1, date), formatStr);
}

/// get the date range of a month
export function getCalendarDateRange(year, month, date) {
  let startDate = "";
  let endDate = "";

  if (month === -1) { // -1 means selecting a year
    // get the first and last days of the year.
    startDate = format(new Date(year, 0, 1), 'yyyy-MM-dd');
    endDate = format(new Date(year, 11, 31), 'yyyy-MM-dd');
  } 
  else if (date === -1) { // -1 means selecting a month
    // get the first and last days of the month.
    startDate = format(new Date(year, month - 1, 1), 'yyyy-MM-dd');
    endDate = format(new Date(year, month, 0), 'yyyy-MM-dd');
  } 
  else {  // otherwise, get files by date
    startDate = format(new Date(year, month - 1, date), 'yyyy-MM-dd');
  }
  return [startDate, endDate];
}

/// format file size to string
export function formatFileSize(bytes: number): string {
  const sizes = ['KB', 'MB', 'GB', 'TB'];
  if (bytes === 0) return '0 KB';
  const i = Math.max(Math.floor(Math.log(bytes) / Math.log(1024)) - 1, 0);
  const fileSize = bytes / Math.pow(1024, i + 1);
  return i === 0 ? `${fileSize.toFixed(0)} ${sizes[i]}` : `${fileSize.toFixed(2)} ${sizes[i]}`;
}

/// get full path
export function getFullPath(path: string, name: string): string {
  return path + separator + name;
}

/// get file folder path
export function getFolderPath(filepath: string): string {
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
  const lastDotIndex = fileName.lastIndexOf('.');
  if (lastDotIndex === -1) {
    return { name: fileName, ext: '' };
  }
  const name = fileName.substring(0, lastDotIndex);
  const ext = fileName.substring(lastDotIndex + 1);
  return { name, ext };
}

/// combine the name and the extension to a file name
export function combineFileName(name: string, ext: string): string {
  return ext ? `${name}.${ext}` : name;
}

/// shorten a filename while preserving its extension
export function shortenFilename(fileName: string, maxLength = 16): string {
  const extIndex = fileName.lastIndexOf('.');
    
  // If no extension is found, return the shortened filename
  if (extIndex === -1) {
      return fileName.length > maxLength ? fileName.substring(0, maxLength - 3) + '...' : fileName;
  }

  const name = fileName.substring(0, extIndex);
  const ext = fileName.substring(extIndex);

  // If the filename is within the limit, return it as is
  if (fileName.length <= maxLength) {
      return fileName;
  }

  // Shorten the name part and keep the extension
  const shortName = name.substring(0, maxLength - ext.length - 3) + '...';
  return shortName + ext;
}

// validate the file or folder name
export const isValidFileName = (name) => {
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
export function localeComp(lang, str1, str2) {
  const localeMap = {
    'zh': 'zh-Hans-CN', // chinese
    'ja': 'ja-JP',      // japanese
    'en': 'en-US',      // english
  };

  const locale = localeMap[lang] || 'en-US';
  if (locale === 'en-US') {
    return str1.localeCompare(str2);
  } else {
    return str1.localeCompare(str2, locale);
  }
};

// scroll to the folder
export function scrollToFolder(folderId) {
  const folderElement = document.getElementById(`folder-${folderId}`);
  if (folderElement) {
    folderElement.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
  }
}
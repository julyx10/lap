import { format } from 'date-fns';
import { platform } from '@tauri-apps/plugin-os';
import { mkdir } from '@tauri-apps/plugin-fs';
import { open } from '@tauri-apps/plugin-shell';
import { open as openDialog } from '@tauri-apps/plugin-dialog';

export const THUMBNAIL_SIZE = 320;    // thumbnail size
export const FILES_PAGE_SIZE = 1000;  // number of files per page

export const separator = platform() === 'windows' ? '\\' : '/';

/// format timestamp to string
export function formatTimestamp(timestamp: number, formatStr: string): string {
  return format(new Date(timestamp * 1000), formatStr);
}

/// format date to string
export function formatDate(year: number, month: number, date: number, formatStr: string): string {
  return format(new Date(year, month - 1, date), formatStr);
}

/// format file size to string
export function formatFileSize(bytes: number): string {
  const sizes = ['KB', 'MB', 'GB', 'TB'];
  if (bytes === 0) return '0 KB';
  const i = Math.max(Math.floor(Math.log(bytes) / Math.log(1024)) - 1, 0);
  const fileSize = bytes / Math.pow(1024, i + 1);
  return `${fileSize.toFixed(2)} ${sizes[i]}`;
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

// Function to create a folder
export async function createFolder(path, name) {
  try {
    const fullPath = getFullPath(path, name);
    await mkdir(fullPath, { recursive: true });
    console.log('Create folder:', fullPath);
    return fullPath;
  } catch (error) {
    console.error('Failed to create folder:', error);
  }
  return null;
}

// Function to open the shell folder
export async function openShellFolder(path) {
  try {
    await open(path); // Open the shell folder
    console.log('Open shell folder:', path);
    return path;
  } catch (error) {
    console.error('Failed to open shell folder:', error);
  }
  return null;
};


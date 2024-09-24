import { os } from '@tauri-apps/api';
import { format } from 'date-fns';

/// thumbnail size
export const THUMBNAIL_SIZE = 320;
export const separator = (await os.platform()) === 'win32' ? '\\' : '/';

/// format timestamp to string
export function formatTimestamp(timestamp: number): string {
  return format(new Date(timestamp * 1000), 'yyyy-MM-dd HH:mm');
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
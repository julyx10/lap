import { os } from '@tauri-apps/api';
import { format } from 'date-fns';

/// thumbnail size
export const THUMBNAIL_SIZE = 320;

// export const separator = (await os.platform()) === 'win32' ? '\\' : '/';
export let separator;
(async () => {
  const osPlatform = await os.platform();
  separator = osPlatform === 'win32' ? '\\' : '/';
})();

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


/// shorten a filename while preserving its extension
export function shortenFilename(filename: string): string {
  const maxLength = 16;
  const extIndex = filename.lastIndexOf('.');
    
  // If no extension is found, return the shortened filename
  if (extIndex === -1) {
      return filename.length > maxLength ? filename.substring(0, maxLength - 3) + '...' : filename;
  }

  const name = filename.substring(0, extIndex);
  const ext = filename.substring(extIndex);

  // If the filename is within the limit, return it as is
  if (filename.length <= maxLength) {
      return filename;
  }

  // Shorten the name part and keep the extension
  const shortName = name.substring(0, maxLength - ext.length - 3) + '...';
  return shortName + ext;
}

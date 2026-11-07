import { reactive } from 'vue';
import { libConfig } from '@/common/config';

// Library-scoped access state is independent of catalog membership and caches.
const albums = reactive(new Map<string, boolean>());
const folders = reactive(new Map<string, boolean>());
const paths = reactive(new Map<string, boolean>());
const key = (id: number | string) => `${libConfig._libraryId}:${id}`;
export function setAlbumAccessibility(id: number, available: boolean) {
  albums.set(key(id), available);
}
export function getAlbumAccessibility(id: number): boolean | undefined {
  return albums.get(key(id));
}
export function getUnavailableAlbumIds(): number[] {
  const prefix = `${libConfig._libraryId}:`;
  return [...albums].filter(([id, available]) => id.startsWith(prefix) && !available)
    .map(([id]) => Number(id.slice(prefix.length)));
}
export function setFileAccessibility(path: string, available: boolean) {
  if (available) paths.delete(key(path));
  else paths.set(key(path), false);
}
export function setFolderAccessibility(path: string, available: boolean) {
  if (available) folders.delete(key(path));
  else folders.set(key(path), false);
}
export function isFolderUnavailable(path: string): boolean {
  const target = key(path);
  return [...folders.keys()].some(folder => target === folder || target.startsWith(`${folder}/`) || target.startsWith(`${folder}\\`));
}
export function isOriginalUnavailable(file: any): boolean {
  if (!file) return false;
  const album = albums.get(key(Number(file.album_id || 0))) ?? file.album_accessible;
  return album === false || paths.get(key(String(file.file_path || ''))) === false || isFolderUnavailable(String(file.file_path || ''));
}
export function resetFileAccessibility(rootPath?: string) {
  const root = rootPath ? key(rootPath.replace(/[\\/]+$/, '')) : null;
  let changed = false;
  for (const states of [paths, folders]) {
    for (const path of states.keys()) {
      if (!root || path === root || path.startsWith(`${root}/`) || path.startsWith(`${root}\\`)) {
        states.delete(path);
        changed = true;
      }
    }
  }
  return changed;
}

export function resetLibraryAccessibility() {
  albums.clear();
  resetFileAccessibility();
}

// Clipboard copy validates its policy-selected paths in clickCopyImages.
// A RAW primary can be offline while its JPEG companion remains available.
const originalActions = new Set([
  'edit', 'print', 'rename', 'move-within-library', 'move-to-folder',
  'save-xmp', 'copy-to-folder', 'trash', 'reveal', 'refresh-file-info', 'set-desktop-wallpaper', 'create-montage',
]);
export function requiresOriginalAction(action: string): boolean {
  return originalActions.has(action) || action.startsWith('open-external-app');
}

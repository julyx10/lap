import { invoke } from '@tauri-apps/api/core';

// Share metadata work between the image overlay and status bar. Keep results
// briefly: an external companion may change without changing the JPEG version.
export function createFocusInfoLoader(request, { maxEntries = 32, ttl = 5000, now = Date.now } = {}) {
  const cache = new Map();
  return function load(path, version = 0, libraryId = '') {
    const key = JSON.stringify([libraryId, path, version || 0]);
    const cached = cache.get(key);
    if (cached && cached.expires > now()) {
      cache.delete(key);
      cache.set(key, cached);
      return cached.promise;
    }
    const entry = { expires: Infinity, promise: null };
    entry.promise = request('get_focus_info', { path }).then(result => {
      entry.expires = now() + ttl;
      return result;
    }, error => {
      if (cache.get(key) === entry) cache.delete(key);
      throw error;
    });
    cache.set(key, entry);
    while (cache.size > maxEntries) cache.delete(cache.keys().next().value);
    return entry.promise;
  };
}

export const loadFocusInfo = createFocusInfoLoader(invoke);

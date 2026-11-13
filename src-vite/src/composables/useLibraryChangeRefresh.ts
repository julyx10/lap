import { onBeforeUnmount, onMounted } from 'vue';
import { listen } from '@tauri-apps/api/event';

/**
 * Refresh sidebar data after catalog mutations. Debounces bursts so a single
 * delete/move/import does not trigger one query per emitted event.
 *
 * Only the active sidebar panel is mounted, so listening here keeps inactive
 * panels free of background queries.
 */
export function useLibraryChangeRefresh(refresh: () => void | Promise<void>, debounceMs = 50) {
  let disposed = false;
  let unlisten: (() => void) | null = null;
  let timer: ReturnType<typeof setTimeout> | null = null;

  const flush = () => {
    timer = null;
    if (disposed) return;
    try {
      void Promise.resolve(refresh()).catch(error => {
        console.error('Sidebar refresh failed:', error);
      });
    } catch (error) {
      console.error('Sidebar refresh failed:', error);
    }
  };

  onMounted(async () => {
    try {
      const stop = await listen('library-total-refreshed', () => {
        if (disposed) return;
        if (timer) clearTimeout(timer);
        timer = setTimeout(flush, debounceMs);
      });
      if (disposed) stop();
      else unlisten = stop;
    } catch (error) {
      console.error('Failed to listen for library changes:', error);
    }
  });

  onBeforeUnmount(() => {
    disposed = true;
    if (timer) clearTimeout(timer);
    unlisten?.();
    unlisten = null;
  });
}

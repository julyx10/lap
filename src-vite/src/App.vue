<template>
  <router-view />
</template>
 
<script setup>
import { onMounted, onUnmounted } from 'vue';
import { emit } from '@tauri-apps/api/event';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { useConfigStore } from '@/stores/configStore';
import { useLibraryStore } from '@/stores/libraryStore';
import { setTheme } from '@/common/utils';

const libConfig = useLibraryStore();

onMounted(async () => {
  const win = getCurrentWebviewWindow();
  if (win.label === 'main') {
    window.addEventListener('keydown', handleKeyDown);
  }

  const config = useConfigStore();
  setTheme(config.settings.appearance, 
    config.settings.appearance === 0 ? config.settings.lightTheme : config.settings.darkTheme);

  // Initialize library state from backend
  try {
    await libConfig.init();
  } catch (error) {
    console.error('[App] Library initialization failed:', error);
    // Continue anyway - user can retry from UI
  } finally {
    // Show window after everything is loaded (main window only)
    if (win.label === 'main') {
      await win.show();
    }
  }
});

onUnmounted(async () => {
  // Save state on app exit/reload
  libConfig.save();

  const win = getCurrentWebviewWindow();
  if (win.label === 'main') {
    window.removeEventListener('keydown', handleKeyDown);
  }
});

const handleKeyDown = (event) => {
  emit('global-keydown', {
    key: event.key,
    altKey: event.altKey,
    ctrlKey: event.ctrlKey,
    metaKey: event.metaKey,
    shiftKey: event.shiftKey,
  });
};

</script>

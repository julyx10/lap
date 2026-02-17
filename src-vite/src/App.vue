<template>
  <div v-if="!isReady" class="w-screen h-screen flex items-center justify-center bg-base-300">
    <span class="loading loading-spinner loading-lg text-primary"></span>
  </div>
  <router-view v-else />
</template>
 
<script setup>
import { ref, watch, onMounted, onUnmounted } from 'vue';
import { emit } from '@tauri-apps/api/event';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { useConfigStore } from '@/stores/configStore';
import { useLibraryStore } from '@/stores/libraryStore';
import { setTheme } from '@/common/utils';

const libConfig = useLibraryStore();
const isReady = ref(false);

// Auto-save library state when any config changes
watch(() => libConfig.$state, () => {
  if (libConfig._initialized) {
    libConfig.save();
  }
}, { deep: true });

onMounted(async () => {
  const win = getCurrentWebviewWindow();
  if (win.label === 'main') {
    window.addEventListener('keydown', handleKeyDown);
    if (import.meta.env.PROD) {
      window.addEventListener('contextmenu', handleContextMenu);
    }
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
    isReady.value = true;
    // Show window after everything is loaded (main window only)
    if (win.label === 'main') {
      await win.show();
    }
  }
});

onUnmounted(async () => {
  const win = getCurrentWebviewWindow();
  if (win.label === 'main') {
    window.removeEventListener('keydown', handleKeyDown);
    if (import.meta.env.PROD) {
      window.removeEventListener('contextmenu', handleContextMenu);
    }
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

const handleContextMenu = (e) => {
  e.preventDefault();
};

</script>

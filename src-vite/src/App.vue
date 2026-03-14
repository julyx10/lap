<template>
  <div v-if="!isReady" class="w-screen h-screen flex items-center justify-center bg-base-300">
    <span class="loading loading-spinner loading-lg text-primary app-loading-delayed"></span>
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
const config = useConfigStore();
const UI_SCALE_STEP = 0.1;
const UI_SCALE_MIN = 0.8;
const UI_SCALE_MAX = 1.2;

// Auto-save library state when any config changes
watch(() => libConfig.$state, () => {
  if (libConfig._initialized) {
    libConfig.save();
  }
}, { deep: true });

watch(
  () => Number(config.settings.uiScale || 1),
  (newScale) => {
    const win = getCurrentWebviewWindow();
    if (win.label === 'main') {
      void applyMainWindowScale(newScale);
    }
  }
);

onMounted(async () => {
  const win = getCurrentWebviewWindow();
  if (win.label === 'main') {
    window.addEventListener('keydown', handleKeyDown, { capture: true });
    void applyMainWindowScale(Number(config.settings.uiScale || 1));
    if (import.meta.env.PROD) {
      window.addEventListener('contextmenu', handleContextMenu);
    }
  }

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
    window.removeEventListener('keydown', handleKeyDown, { capture: true });
    if (import.meta.env.PROD) {
      window.removeEventListener('contextmenu', handleContextMenu);
    }
  }
});

const handleKeyDown = (event) => {
  if (handleMainWindowZoomShortcut(event)) {
    return;
  }

  emit('global-keydown', {
    key: event.key,
    altKey: event.altKey,
    ctrlKey: event.ctrlKey,
    metaKey: event.metaKey,
    shiftKey: event.shiftKey,
  });
};

function clampScale(value) {
  return Math.max(UI_SCALE_MIN, Math.min(UI_SCALE_MAX, Number(value.toFixed(2))));
}

function applyMainWindowScale(scale) {
  const clampedScale = clampScale(scale);
  document.documentElement.style.zoom = String(clampedScale);
}

function handleMainWindowZoomShortcut(event) {
  const win = getCurrentWebviewWindow();
  if (win.label !== 'main') return false;

  const isCmdOrCtrl = event.metaKey || event.ctrlKey;
  if (!isCmdOrCtrl || event.altKey) return false;

  const key = event.key;
  const code = event.code;
  const isZoomIn = key === '+' || key === '=' || code === 'NumpadAdd';
  const isZoomOut = key === '-' || key === '_' || code === 'NumpadSubtract';
  const isZoomReset = key === '0' || code === 'Numpad0';

  if (!isZoomIn && !isZoomOut && !isZoomReset) return false;

  event.preventDefault();
  event.stopPropagation();

  let nextScale = Number(config.settings.uiScale || 1);
  if (isZoomIn) {
    nextScale = clampScale(nextScale + UI_SCALE_STEP);
  } else if (isZoomOut) {
    nextScale = clampScale(nextScale - UI_SCALE_STEP);
  } else {
    nextScale = 1;
  }

  if (nextScale !== Number(config.settings.uiScale || 1)) {
    config.setUiScale(nextScale);
  }
  void applyMainWindowScale(nextScale);
  return true;
}

const handleContextMenu = (e) => {
  e.preventDefault();
};

</script>

<style scoped>
.app-loading-delayed {
  opacity: 0;
  animation: appLoadingShow 0s linear 0.5s forwards;
}

@keyframes appLoadingShow {
  to {
    opacity: 1;
  }
}
</style>

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
import { clearIndexRecoveryInfo } from '@/common/api';
import { setTheme, SCALE_VALUES } from '@/common/utils';

const libConfig = useLibraryStore();
const isReady = ref(false);
const config = useConfigStore();
let unlistenMainCloseRequested = null;
let isHandlingMainClose = false;

// Auto-save library state when any config changes
watch(() => libConfig.$state, () => {
  if (libConfig._initialized) {
    libConfig.save();
  }
}, { deep: true });

watch(
  () => Number(config.settings.scale || 1),
  (newScale) => {
    const win = getCurrentWebviewWindow();
    if (win.label === 'main') {
      applyMainWindowScale(newScale);
    }
  }
);

onMounted(async () => {
  const win = getCurrentWebviewWindow();
  if (win.label === 'main') {
    window.addEventListener('keydown', handleKeyDown, { capture: true });
    applyMainWindowScale(Number(config.settings.scale || 1));
    if (import.meta.env.PROD) {
      window.addEventListener('contextmenu', handleContextMenu);
    }
    if (typeof win.onCloseRequested === 'function') {
      unlistenMainCloseRequested = await win.onCloseRequested(async (event) => {
        if (isHandlingMainClose) return;
        isHandlingMainClose = true;
        event.preventDefault();

        try {
          if (libConfig._initialized) {
            // Mark scanning as paused so it won't auto-resume on restart
            if (libConfig.index.status === 1) {
              libConfig.index.status = 2;
            }
            // Normal close → clear recovery trace (crash leaves it intact)
            await clearIndexRecoveryInfo();
            await libConfig.save();
          }
        } finally {
          await win.destroy();
        }
      });
    } else {
      unlistenMainCloseRequested = await win.listen('tauri://close-requested', async () => {
        if (libConfig._initialized) {
          // Mark scanning as paused so it won't auto-resume on restart
          if (libConfig.index.status === 1) {
            libConfig.index.status = 2;
          }
          await clearIndexRecoveryInfo();
          await libConfig.save();
        }
      });
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
    document.documentElement.style.fontSize = '';
    if (import.meta.env.PROD) {
      window.removeEventListener('contextmenu', handleContextMenu);
    }
  }
  unlistenMainCloseRequested?.();
  unlistenMainCloseRequested = null;
});

const handleKeyDown = (event) => {
  if (handleMainWindowScaleShortcut(event)) {
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

function normalizeScale(value) {
  return SCALE_VALUES.find((item) => item === Number(value)) ?? 1;
}

function applyMainWindowScale(scale) {
  const normalizedScale = normalizeScale(scale);
  document.documentElement.style.fontSize = `${normalizedScale * 16}px`;
}

function handleMainWindowScaleShortcut(event) {
  const win = getCurrentWebviewWindow();
  if (win.label !== 'main') return false;

  const isCmdOrCtrl = event.metaKey || event.ctrlKey;
  if (!isCmdOrCtrl || event.altKey) return false;

  const key = event.key;
  const code = event.code;
  const isScaleUp = key === '+' || key === '=' || code === 'NumpadAdd';
  const isScaleDown = key === '-' || key === '_' || code === 'NumpadSubtract';
  const isScaleReset = key === '0' || code === 'Numpad0';

  if (!isScaleUp && !isScaleDown && !isScaleReset) return false;

  event.preventDefault();
  event.stopPropagation();

  const currentScale = normalizeScale(config.settings.scale || 1);
  const currentIndex = SCALE_VALUES.indexOf(currentScale);
  let nextScale = currentScale;

  if (isScaleReset) {
    nextScale = 1;
  } else if (isScaleUp) {
    nextScale = SCALE_VALUES[Math.min(currentIndex + 1, SCALE_VALUES.length - 1)];
  } else if (isScaleDown) {
    nextScale = SCALE_VALUES[Math.max(currentIndex - 1, 0)];
  }

  if (nextScale !== currentScale) {
    config.setScale(nextScale);
  }
  applyMainWindowScale(nextScale);
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

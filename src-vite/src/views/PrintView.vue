<template>
  <div class="print-view w-screen h-screen flex flex-col overflow-hidden bg-base-300 text-base-content/70">
    <TitleBar :titlebar="windowTitle" :resizable="false" viewName="Print" class="print-ui shrink-0 z-50" />

    <div class="print-ui flex-1 overflow-hidden px-3">
      <div class="h-full flex items-center justify-center">
        <div v-if="loadError" class="text-sm text-base-content/50">
          {{ $t('print_view.load_failed') }}
        </div>

        <div v-else class="w-full h-full flex items-center justify-center">
          <div class="relative w-full h-full rounded-box overflow-hidden border border-base-content/5 bg-base-300/30 shadow-sm cursor-default">
            <div
              v-if="isLoading"
              class="absolute inset-0 z-10 flex items-center justify-center bg-base-100/55 backdrop-blur-sm"
            >
              <span class="loading loading-dots text-primary"></span>
            </div>
            <div class="w-full h-full p-2 flex items-center justify-center">
              <img
                v-if="imageSrc"
                :src="imageSrc"
                class="block w-full h-full object-contain"
                @load="handleImageLoad"
                @error="handleImageError"
              />
            </div>
          </div>
        </div>
      </div>
    </div>

    <div class="print-ui shrink-0 mx-1 mb-1 px-2 py-2 flex items-center justify-end gap-2">
      <button
        class="px-4 py-1 rounded-box hover:bg-base-100 hover:text-base-content cursor-pointer transition-colors duration-200"
        @click="closeWindow"
      >
        {{ $t('image_viewer.toolbar.close') }}
      </button>
      <button
        :class="[
          'px-4 py-1 rounded-box transition-colors duration-200',
          !imageSrc || isLoading || loadError
            ? 'opacity-50 cursor-not-allowed'
            : 'hover:bg-primary hover:text-primary-content cursor-pointer'
        ]"
        :disabled="!imageSrc || isLoading || loadError"
        @click="openPrintDialog"
      >
        {{ $t('print_view.title') }}
      </button>
    </div>

    <div class="print-only hidden h-screen w-screen items-center justify-center bg-white">
      <img
        v-if="imageSrc"
        :src="imageSrc"
        class="block max-w-full max-h-full object-contain"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, onBeforeUnmount } from 'vue';
import { listen } from '@tauri-apps/api/event';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { useI18n } from 'vue-i18n';
import { config } from '@/common/config';
import { getAssetSrc, getFileExtension, getPreviewUrl, setTheme, SCALE_VALUES, shortenFilename } from '@/common/utils';
import TitleBar from '@/components/TitleBar.vue';

const { locale, t } = useI18n();
const appWindow = getCurrentWebviewWindow();

const imageSrc = ref('');
const filePath = ref('');
const fileId = ref(0);
const fileType = ref(1);
const isLoading = ref(false);
const loadError = ref(false);
let unlistenUpdatePrint: null | (() => void) = null;

const fileName = computed(() => {
  const value = filePath.value;
  if (!value) return '';
  const parts = value.split(/[\\/]/);
  return parts[parts.length - 1] || '';
});

const windowTitle = computed(() => {
  return fileName.value
    ? `${t('print_view.title')} - ${shortenFilename(fileName.value, 32)}`
    : t('print_view.title');
});

function normalizeScale(value: number) {
  return SCALE_VALUES.find((item) => item === Number(value)) ?? 1;
}

function applyWindowScale(scale: number) {
  const normalizedScale = normalizeScale(scale);
  document.documentElement.style.fontSize = `${normalizedScale * 16}px`;
}

function shouldUseBackendPreview(targetPath: string, targetFileType: number): boolean {
  if (targetFileType === 3) return true;
  const extension = getFileExtension(targetPath || '').toLowerCase();
  return extension === 'tif' || extension === 'tiff' || extension === 'jxl';
}

async function resolvePrintSource(targetPath: string, targetFileType: number, targetFileId: number) {
  isLoading.value = true;
  loadError.value = false;
  imageSrc.value = '';
  filePath.value = targetPath;
  fileId.value = targetFileId;
  fileType.value = targetFileType;

  if (!targetPath) {
    loadError.value = true;
    isLoading.value = false;
    return;
  }

  try {
    if (shouldUseBackendPreview(targetPath, targetFileType)) {
      const previewSrc = getPreviewUrl(targetFileId, targetPath);
      if (!previewSrc) {
        loadError.value = true;
        return;
      }
      imageSrc.value = previewSrc;
      return;
    }

    imageSrc.value = getAssetSrc(targetPath);
  } catch (error) {
    console.error('Failed to resolve print source:', error);
    loadError.value = true;
    isLoading.value = false;
  } finally {
    // Keep the loading state until the preview image is actually rendered.
  }
}

async function handlePrintPayload(payload: { fileId?: number; filePath?: string; fileType?: number }) {
  const targetFileId = Number(payload?.fileId || 0);
  const targetPath = String(payload?.filePath || '');
  const targetFileType = Number(payload?.fileType || 1);
  await resolvePrintSource(targetPath, targetFileType, targetFileId);
}

function openPrintDialog() {
  if (!imageSrc.value || isLoading.value || loadError.value) return;
  window.print();
}

function handleImageLoad() {
  loadError.value = false;
  isLoading.value = false;
}

function handleImageError() {
  isLoading.value = false;
  loadError.value = true;
}

function closeWindow() {
  void appWindow.close();
}

function updateWindowTitle() {
  void appWindow.setTitle(windowTitle.value);
}

function handleKeyDown(event: KeyboardEvent) {
  if (event.key === 'Escape') {
    event.preventDefault();
    closeWindow();
  } else if ((event.metaKey || event.ctrlKey) && !event.altKey && event.key.toLowerCase() === 'p') {
    event.preventDefault();
    openPrintDialog();
  }
}

onMounted(async () => {
  window.addEventListener('keydown', handleKeyDown, { capture: true });

  applyWindowScale(Number(config.settings.scale || 1));
  setTheme(
    config.settings.appearance,
    config.settings.appearance === 0 ? config.settings.lightTheme : config.settings.darkTheme
  );
  updateWindowTitle();

  unlistenUpdatePrint = await listen('update-print', async (event) => {
    await handlePrintPayload((event.payload || {}) as { fileId?: number; filePath?: string; fileType?: number });
  });

  const search = new URLSearchParams(window.location.search);
  await handlePrintPayload({
    fileId: Number(search.get('fileId') || 0),
    filePath: search.get('filePath') || '',
    fileType: Number(search.get('fileType') || 1),
  });

  await appWindow.show();
  await appWindow.setFocus();
});

onBeforeUnmount(() => {
  if (unlistenUpdatePrint) {
    unlistenUpdatePrint();
    unlistenUpdatePrint = null;
  }
  window.removeEventListener('keydown', handleKeyDown, { capture: true });
  document.documentElement.style.fontSize = '';
});

watch(() => config.settings.language, (newLanguage) => {
  locale.value = newLanguage;
  updateWindowTitle();
});

watch(windowTitle, () => {
  updateWindowTitle();
});

watch(() => config.settings.appearance, (newAppearance) => {
  setTheme(newAppearance, newAppearance === 0 ? config.settings.lightTheme : config.settings.darkTheme);
});

watch(() => config.settings.lightTheme, (newLightTheme) => {
  setTheme(config.settings.appearance, newLightTheme);
});

watch(() => config.settings.darkTheme, (newDarkTheme) => {
  setTheme(config.settings.appearance, newDarkTheme);
});

watch(() => Number(config.settings.scale || 1), (newScale) => {
  applyWindowScale(newScale);
});
</script>

<style scoped>
@media print {
  @page {
    margin: 0;
  }
  
  .print-view {
    width: auto !important;
    height: auto !important;
    display: block !important;
    overflow: visible !important;
    background: #fff !important;
    color: #000 !important;
  }

  .print-ui {
    display: none !important;
  }

  .print-only {
    display: flex !important;
    width: 100% !important;
    height: 100% !important;
  }

  .print-only img {
    width: auto !important;
    height: auto !important;
    max-width: 100% !important;
    max-height: 100% !important;
  }
}
</style>

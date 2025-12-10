<template>

  <div
    :class="[
      'relative w-screen h-screen flex flex-col overflow-hidden bg-base-300 text-base-content/70',
      config.imageViewer.isFullScreen ? 'fixed top-0 left-0 z-50' : '',
    ]"
  >
    <!-- title bar -->
    <TitleBar v-if="!config.imageViewer.isFullScreen"
      :titlebar="isWin ? `jc-photo ${localeMsg.image_viewer.title}${fileIndex >= 0 ? ` - ${fileIndex + 1}/${fileCount}` : ''}` : ''"
      viewName="ImageViewer"
    />

    <!-- Toolbar -->
    <div 
      :class="[
        'absolute left-1/2 z-40 bg-transparent transform -translate-x-1/2 select-none group',
      ]"
      data-tauri-drag-region
    >
      <div id="responsiveDiv"
        :class="[
          'px-4 h-12 space-x-2 rounded-box flex flex-row items-center justify-center bg-base-300',
          config.imageViewer.isFullScreen && !config.imageViewer.isPinned ? '-translate-y-8 opacity-0 group-hover:translate-y-2 group-hover:opacity-80 transition-transform duration-300 ease-in-out' : '',
          config.imageViewer.isFullScreen && config.imageViewer.isPinned ? 'opacity-80 translate-y-2 transition-transform duration-300 ease-in-out' : ''
        ]"
      >
        <TButton
          :icon="IconPrev"
          :disabled="fileIndex <= 0 || config.imageViewer.isLocked"
          :tooltip="$t('image_viewer.toolbar.prev') + ` (${fileIndex + 1}/${fileCount})`"
          @click="clickPrev()" 
        />
        <TButton
          :icon="IconLock"
          :disabled="fileIndex < 0"
          :selected="config.imageViewer.isLocked"
          :tooltip="config.imageViewer.isLocked ? $t('image_viewer.toolbar.unlock') : $t('image_viewer.toolbar.lock')"
          @click="toggleLock()"
        />
        <TButton
          :icon="IconNext"
          :disabled="fileIndex < 0 || fileIndex >= fileCount - 1 || config.imageViewer.isLocked"
          :tooltip="$t('image_viewer.toolbar.next') + ` (${fileIndex + 1}/${fileCount})`"
          @click="clickNext()" 
        />
        <TButton
          :icon="isSlideShow ? IconPause : IconPlay"
          :disabled="fileIndex < 0 || config.imageViewer.isLocked"
          :tooltip="(isSlideShow ? $t('image_viewer.toolbar.pause') : $t('image_viewer.toolbar.slide_show')) + ` (${getSlideShowInterval(config.settings.slideShowInterval)}s)`"
          @click="clickSlideShow()" 
        />
        <TButton
          :icon="IconZoomOut"
          :disabled="fileIndex < 0 || imageScale <= imageMinScale"
          :tooltip="$t('image_viewer.toolbar.zoom_out') + ` (${(imageScale * 100).toFixed(0)}%)`"
          @click="clickZoomOut()"
        />
        <TButton
          :icon="IconZoomIn"
          :disabled="fileIndex < 0 || imageScale >= imageMaxScale"
          :tooltip="$t('image_viewer.toolbar.zoom_in') + ` (${(imageScale * 100).toFixed(0)}%)`"
          @click="clickZoomIn()" 
        />
        <TButton
          :icon="!config.imageViewer.isZoomFit ? IconZoomFit : IconZoomActual"
          :disabled="fileIndex < 0"
          :tooltip="(!config.imageViewer.isZoomFit ? $t('image_viewer.toolbar.zoom_fit') : $t('image_viewer.toolbar.zoom_actual')) + ` (${(imageScale * 100).toFixed(0)}%)`"
          @click="toggleZoomFit()"
        />
        <TButton v-if="isWin"
          :icon="!config.imageViewer.isFullScreen ? IconFullScreen : IconRestoreScreen"
          :tooltip="!config.imageViewer.isFullScreen ? $t('image_viewer.toolbar.fullscreen') : $t('image_viewer.toolbar.exit_fullscreen')"
          @click="toggleFullScreen()"
        />

        <TButton v-if="config.imageViewer.isFullScreen"
          :icon="IconSeparator"
          :disabled="true"
        />

        <TButton v-if="config.imageViewer.isFullScreen"
          :icon="config.imageViewer.isPinned ? IconPin : IconUnPin"
          :disabled="fileIndex < 0"
          :tooltip="!config.imageViewer.isPinned ? $t('image_viewer.toolbar.pin') : $t('image_viewer.toolbar.unpin')"
          @click="config.imageViewer.isPinned = !config.imageViewer.isPinned"
        />
        <TButton v-if="config.imageViewer.isFullScreen"
          :icon="IconClose"
          :tooltip="$t('image_viewer.toolbar.close')"
          @click="appWindow.close()"
        />
      </div>
    </div>

    <!-- image container -->
    <div ref="viewerContainer" class="relative flex-1 flex justify-center items-center overflow-hidden select-none">

      <template v-if="fileIndex >= 0">
        <MediaViewer
          ref="mediaViewerRef"
          :file="fileInfo"
          :isZoomFit="config.imageViewer.isZoomFit"
          :showNavButton="!config.imageViewer.isLocked"
          :hasPrevious="fileIndex > 0"
          :hasNext="fileIndex < fileCount - 1"
          @prev="onPrev()"
          @next="onNext()"
          @dblclick="toggleZoomFit()"
        />

        <!-- comments -->
        <div v-if="config.settings.showComment && fileInfo?.comments?.length > 0" 
          class="absolute flex m-2 p-2 bottom-0 left-0 right-0 text-sm bg-base-100/30 rounded-box select-text" 
        >
          <IconComment class="t-icon-size-sm shrink-0 mr-2"></IconComment>
          {{ fileInfo?.comments }}
        </div>
      </template>

      <!-- no image selected -->
      <div v-else class="flex flex-col items-center justify-center w-full h-full text-base-content/30">
        <IconSearch class="w-8 h-8" />
        <span>{{ $t('tooltip.not_found.files') }}</span>
      </div>
    </div>

  </div>

</template>


<script setup lang="ts">

import { ref, watch, computed, onMounted, onUnmounted } from 'vue';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { emit, listen } from '@tauri-apps/api/event';
import { useI18n } from 'vue-i18n';
import { useUIStore } from '@/stores/uiStore';
import { config } from '@/common/config';
import { isWin, isMac, setTheme, getSlideShowInterval } from '@/common/utils';
import { getFileInfo } from '@/common/api';

import TitleBar from '@/components/TitleBar.vue';
import TButton from '@/components/TButton.vue';
import MediaViewer from '@/components/MediaViewer.vue';

import { 
  IconPrev,
  IconNext,
  IconPlay,
  IconPause,
  IconZoomIn,
  IconZoomOut,
  IconZoomFit,
  IconZoomActual,
  IconSearch,
  IconFullScreen,
  IconRestoreScreen,
  IconPin,
  IconUnPin,
  IconSeparator,
  IconClose,
  IconComment,
  IconLock,
 } from '@/common/icons';

/// i18n
const { locale, messages } = useI18n();
const localeMsg = computed(() => messages.value[locale.value]);
const uiStore = useUIStore();

const appWindow = getCurrentWebviewWindow()

// input parameters
const fileId = ref(0);       // File ID
const fileIndex = ref(0);       // Index of the current file
const fileCount = ref(0);       // Total number of files

const fileInfo = ref<any>(null);
const iconRotate = ref(0);      // icon rotation angle
const isTransitionDisabled = ref(true);

const mediaViewerRef = ref<any>(null); // media viewer reference

const isSlideShow = ref(false);     // Slide show state
let timer: NodeJS.Timeout | null = null;  // Timer for slide show

const imageScale = ref(1);          // Image scale
const imageMinScale = ref(0);       // Minimum image scale
const imageMaxScale = ref(10);      // Maximum image scale

let unlistenResize: () => void;
let unlistenImg: () => void;
let unlistenImage: () => void;
let unlistenGridView: () => void;

onMounted(async() => {
  window.addEventListener('keydown', handleKeyDown);
  unlistenResize = await appWindow.listen('tauri://resize', handleResize);     // macOS: Listen for full screen change

  const urlParams = new URLSearchParams(window.location.search);
  
  fileId.value    = Number(urlParams.get('fileId'));
  fileIndex.value = Number(urlParams.get('fileIndex'));
  fileCount.value = Number(urlParams.get('fileCount'));

  // Listen 
  unlistenImg = await listen('update-img', async (event) => {
    if(uiStore.inputStack.length > 0 || config.imageViewer.isLocked) {
      return;
    }
    
    fileId.value    = Number(event.payload.fileId);
    fileIndex.value = Number(event.payload.fileIndex);
    fileCount.value = Number(event.payload.fileCount);
  });

  unlistenImage = await listen('message-from-image', (event) => {
    const { message } = event.payload;
    console.log('ImageViewer.vue: message-from-image:', message);
    switch (message) {
      case 'scale':
        imageScale.value = event.payload.scale;
        imageMinScale.value = event.payload.minScale;
        imageMaxScale.value = event.payload.maxScale;
        break;
      default:
        break;
    }
  });

  unlistenGridView = await listen('message-from-content', (event) => {
    const { message } = event.payload;
    console.log('message-from-content:', message);
    switch (message) {
      case 'rotate':
        if (config.imageViewer.isLocked) {
          return;
        }
        if (mediaViewerRef.value) {
          mediaViewerRef.value.rotateRight();
        }
        iconRotate.value += 90;
        if (fileInfo.value) {
          fileInfo.value.rotate = (fileInfo.value.rotate || 0) + 90;
        }
        break;
      default:
        break;
    }
  });

  setTimeout(() => {
    isTransitionDisabled.value = false;
  }, 500);
});

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeyDown);
  
  // unlisten
  unlistenResize();
  unlistenImg();
  unlistenImage();
  unlistenGridView();
});

// Handle keyboard shortcuts
function handleKeyDown(event: KeyboardEvent) {
  if(uiStore.inputStack.length > 0) {
    return;
  }
  
  const key = event.key;
  if (keyActions[key]) {
    keyActions[key]();
  }
}

const keyActions = {
  ArrowLeft:  () => clickPrev(),
  ArrowRight: () => clickNext(),
  Home:       () => clickHome(),
  End:        () => clickEnd(),
  ArrowUp:    () => clickZoomIn(),
  ArrowDown:  () => clickZoomOut(),
  '=':        () => clickZoomIn(),
  '-':        () => clickZoomOut(),
  '0':        () => clickZoomActual(),
  ' ':        () => toggleZoomFit(),
  Escape:     () => closeWindow(),
};

// Handle resize event
const handleResize = async () => {
  if(isMac) {
    config.imageViewer.isFullScreen = await appWindow.isFullscreen();
    console.log('handleFullScreenChange:', config.imageViewer.isFullScreen);
  }
};

/// watch appearance
watch(() => config.settings.appearance, (newAppearance) => {
  setTheme(newAppearance, newAppearance === 0 ? config.settings.lightTheme : config.settings.darkTheme);
});

/// watch light theme
watch(() => config.settings.lightTheme, (newLightTheme) => {
  setTheme(config.settings.appearance, newLightTheme);
});

/// watch dark theme
watch(() => config.settings.darkTheme, (newDarkTheme) => {
  setTheme(config.settings.appearance, newDarkTheme);
});

// watch language
watch(() => config.settings.language, (newLanguage) => {
    console.log('Language changed to:', newLanguage);
    locale.value = newLanguage; // update locale based on config.settings.language
});

// watch full screen (win only)
watch(() => config.imageViewer.isFullScreen, async (newFullScreen) => {
  if(!isWin) return;
  await appWindow.setFullscreen(newFullScreen);
  await appWindow.setResizable(!newFullScreen);
  // await appWindow.setDecorations(false);
}); 

// watch file changed
watch(() => fileId.value, async () => {
  fileInfo.value = await getFileInfo(fileId.value);
  iconRotate.value = fileInfo.value.rotate || 0;
  console.log('fileInfo:', fileInfo.value);
});

// watch file index
watch(() => fileIndex.value, async (newIndex) => {
  if(newIndex === -1) {
    isSlideShow.value = false;
    iconRotate.value = 0; // reset rotation
  } 
});

watch(() => [isSlideShow.value, config.settings.slideShowInterval], ([newIsSlideShow, newInterval]) => {
  if(newIsSlideShow) {
    clearInterval(timer);
    timer = setInterval(() => {
      clickNext();
    }, getSlideShowInterval(newInterval) * 1000);
  } else {
    clearInterval(timer);
  }
});

function clickPrev() {
  if (config.imageViewer.isLocked) return;
  mediaViewerRef.value?.triggerPrev();
}

function onPrev() {
  emit('message-from-image-viewer', { message: 'prev' });
}

function clickNext() {
  if (config.imageViewer.isLocked) return;
  mediaViewerRef.value?.triggerNext();
}

function onNext() {
  if(isSlideShow.value && fileIndex.value === fileCount.value - 1) {
    emit('message-from-image-viewer', { message: 'home' });
  } else {
    emit('message-from-image-viewer', { message: 'next' });
  }
}

function clickHome() {
  if (config.imageViewer.isLocked) return;
  emit('message-from-image-viewer', { message: 'home' });
}

function clickEnd() {
  if (config.imageViewer.isLocked) return;
  emit('message-from-image-viewer', { message: 'end' });
}

function clickSlideShow() {
  if (config.imageViewer.isLocked) return;
  isSlideShow.value = !isSlideShow.value;
}

const clickZoomIn = () => {
  if(mediaViewerRef.value) {
    mediaViewerRef.value.zoomIn();
  }
};

const clickZoomOut = () => {
  if(mediaViewerRef.value) {
    mediaViewerRef.value.zoomOut();
  }
};

const clickZoomActual = () => {
  if(mediaViewerRef.value) {
    mediaViewerRef.value.zoomActual();
  }
};

const toggleZoomFit = () => {
  config.imageViewer.isZoomFit =!config.imageViewer.isZoomFit;
};

// toggle lock status(locked mode: image will not be updated)
const toggleLock = () => {
  config.imageViewer.isLocked = !config.imageViewer.isLocked;
}

const closeWindow = () => {
  if(config.imageViewer.isFullScreen) {
    config.imageViewer.isFullScreen = false;
    appWindow.setFocus();
  } else {
    appWindow.close();
  }
}

// Function to maximize the window and setup full screen
const toggleFullScreen = () => {
  config.imageViewer.isFullScreen = !config.imageViewer.isFullScreen;
}

</script>

<style scoped>
/* Disable text selection while dragging */
* {
  user-select: none;
}
 
@media (max-width: 600px) {
  #responsiveDiv {
    visibility: hidden;
  }
}
@media (min-width: 600px) {
  #responsiveDiv {
    visibility: visible;
  }
}
</style>

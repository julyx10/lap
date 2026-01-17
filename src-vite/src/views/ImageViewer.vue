<template>

  <div
    :class="[
      'relative w-screen h-screen flex flex-col overflow-hidden bg-base-300 text-base-content/70',
      isFullScreen ? 'fixed top-0 left-0 z-50' : '',
    ]"
  >
    <div ref="viewerContainer" class="relative flex-1 flex justify-center items-center bg-base-200 overflow-hidden select-none">

      <template v-if="fileIndex >= 0">
        <MediaViewer
          ref="mediaViewerRef"
          :mode="2"
          :isFullScreen="isFullScreen"
          :file="fileInfo"
          :hasPrevious="fileIndex > 0"
          :hasNext="fileIndex < fileCount - 1"
          :fileIndex="fileIndex"
          :fileCount="fileCount"
          :isSlideShow="isSlideShow"
          :imageScale="imageScale"
          :imageMinScale="imageMinScale"
          :imageMaxScale="imageMaxScale"
          :isZoomFit="isZoomFit"
          @prev="clickPrev()"
          @next="clickNext()"
          @toggle-slide-show="clickSlideShow()"
          @scale="clickScale"
          @update:isZoomFit="(val) => isZoomFit = val"
          @dblclick="toggleZoomFit()"
          @close="appWindow.close()"
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

import MediaViewer from '@/components/MediaViewer.vue';

import { 
  IconSearch,
  IconComment,
 } from '@/common/icons';

/// i18n
const { locale, messages } = useI18n();
const localeMsg = computed(() => messages.value[locale.value] as any);
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
const isFullScreen = ref(false);
const isZoomFit = ref(true);

const isSlideShow = ref(false);     // Slide show state
let timer: NodeJS.Timeout | null = null;  // Timer for slide show

const imageScale = ref(1);          // Image scale
const imageMinScale = ref(0);       // Minimum image scale
const imageMaxScale = ref(10);      // Maximum image scale

let unlistenImg: () => void;
let unlistenGridView: () => void;

onMounted(async() => {
  window.addEventListener('keydown', handleKeyDown);
  window.addEventListener('resize', handleResize);

  const urlParams = new URLSearchParams(window.location.search);
  
  fileId.value    = Number(urlParams.get('fileId'));
  fileIndex.value = Number(urlParams.get('fileIndex'));
  fileCount.value = Number(urlParams.get('fileCount'));

  // Listen 
  unlistenImg = await listen('update-img', async (event: any) => {
    if(uiStore.inputStack.length > 0) {
      return;
    }
    
    fileId.value    = Number(event.payload.fileId);
    fileIndex.value = Number(event.payload.fileIndex);
    fileCount.value = Number(event.payload.fileCount);
  });


  unlistenGridView = await listen('message-from-content', (event) => {
    const { message, fileId: targetFileId } = event.payload as any;
    console.log('message-from-content:', message, targetFileId);
    switch (message) {
      case 'rotate':
        if (targetFileId === fileId.value) {
          if (mediaViewerRef.value) {
            mediaViewerRef.value.rotateRight();
          }
          iconRotate.value += 90;
          if (fileInfo.value) {
            fileInfo.value.rotate = (fileInfo.value.rotate || 0) + 90;
          }
        }
        break;
      default:
        break;
    }
  });

  setTimeout(() => {
    isTransitionDisabled.value = false;
  }, 500);

  await handleResize();
  
  // Show window after mount (if it was created hidden)
  try {
    await appWindow.show();
  } catch (e) {
    // Window might already be visible, ignore error
  }
});

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeyDown);
  window.removeEventListener('resize', handleResize);
  
  // unlisten
  unlistenImg();
  unlistenGridView();
});

// Handle keyboard shortcuts
function handleKeyDown(event: KeyboardEvent) {
  if(uiStore.inputStack.length > 0) {
    return;
  }

  // Disable keyboard events during slideshow (except Escape)
  if (isSlideShow.value && event.key !== 'Escape') {
    return;
  }
  
  const key = event.key;
  if ((keyActions as any)[key]) {
    (keyActions as any)[key]();
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
    const checkFullScreen = async () => {
      isFullScreen.value = await appWindow.isFullscreen();
    };
    await checkFullScreen();
    setTimeout(checkFullScreen, 600); 
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

// watch full screen
watch(() => isFullScreen.value, async (newFullScreen) => {
  if(isWin) {
    await appWindow.setFullscreen(newFullScreen);
    await appWindow.setResizable(!newFullScreen);
    // await appWindow.setDecorations(false);
  } else if (isMac) {
      if (newFullScreen !== await appWindow.isFullscreen()) {
        await appWindow.setFullscreen(newFullScreen);
    }
  }
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
  if (fileIndex.value > 0) {
      emit('message-from-image-viewer', { message: 'request-file-at-index', index: fileIndex.value - 1 });
  } else {
     mediaViewerRef.value?.showMessage((localeMsg.value as any).tooltip.image_viewer.first_image);
  }
}

function clickNext() {
  // Fix loop for slideshow
  if (isSlideShow.value && fileIndex.value >= fileCount.value - 1) {
    emit('message-from-image-viewer', { message: 'request-file-at-index', index: 0 });
    return;
  }
  
  if (fileIndex.value < fileCount.value - 1) {
    emit('message-from-image-viewer', { message: 'request-file-at-index', index: fileIndex.value + 1 });
  } else {
    mediaViewerRef.value?.showMessage((localeMsg.value as any).tooltip.image_viewer.last_image);
  }
}

function clickHome() {
  emit('message-from-image-viewer', { message: 'request-file-at-index', index: 0 });
}

function clickEnd() {
  emit('message-from-image-viewer', { message: 'request-file-at-index', index: fileCount.value - 1 });
}

function clickSlideShow() {
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
  isZoomFit.value = !isZoomFit.value;
};

const closeWindow = () => {
  if(isFullScreen.value) {
    isFullScreen.value = false;
    appWindow.setFocus();
  } else {
    appWindow.close();
  }
}

const clickScale = (event: any) => {
  imageScale.value = event.scale;
  imageMinScale.value = event.minScale;
  imageMaxScale.value = event.maxScale;
};

</script>

<style scoped>
* {
  user-select: none;
}
</style>

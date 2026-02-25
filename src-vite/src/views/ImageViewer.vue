<template>

  <div
    :class="[
      'relative w-screen h-screen flex flex-col overflow-hidden bg-base-300 text-base-content/70',
      isFullScreen ? 'fixed top-0 left-0 z-50' : '',
    ]"
  >
    <div class="absolute right-1 top-1 z-95 flex items-center gap-1">
      <TButton
        :icon="IconLink"
        :selected="isSplit && isSyncViewport"
        :disabled="!isSplit"
        :tooltip="isSplit
          ? (isSyncViewport ? $t('image_viewer.toolbar.sync_viewport_off') : $t('image_viewer.toolbar.sync_viewport_on'))
          : $t('image_viewer.toolbar.sync_viewport_need_split')"
        @click="toggleSyncViewport()"
      />
      <TButton
        :icon="IconSplitOn"
        :selected="isSplit"
        :tooltip="isSplit ? $t('image_viewer.toolbar.split_off') : $t('image_viewer.toolbar.split_on')"
        @click="toggleSplit()"
      />
    </div>

    <div
      ref="viewerContainer"
      :class="[
        'relative flex-1 flex justify-center items-center overflow-hidden select-none',
        config.settings.showStatusBar ? 'pb-8' : '',
      ]"
    >
      <template v-if="!isSplit && fileIndex >= 0">
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
          @update:isZoomFit="(val) => handleZoomFitUpdate(val, 'left')"
          @media-dblclick="toggleZoomFit()"
          @close="appWindow.close()"
          @slideshow-next="handleSlideshowNext"
        />

        <!-- comments -->
        <div v-if="config.settings.showComment && fileInfo?.comments?.length > 0" 
          class="absolute flex m-2 p-2 bottom-0 left-0 right-0 text-sm bg-base-100/30 rounded-box select-text" 
        >
          <IconComment class="t-icon-size-sm shrink-0 mr-2"></IconComment>
          {{ fileInfo?.comments }}
        </div>
      </template>

      <template v-else-if="isSplit && fileIndex >= 0">
        <div class="w-full h-full flex">
          <div
            class="relative w-1/2 h-full border-r border-base-content/10"
            @mousedown="setActivePane('left')"
          >
            <IconDot
              v-if="activePane === 'left'"
              class="absolute right-2 top-2 z-90 t-icon-size-sm text-primary pointer-events-none"
            />
            <MediaViewer
              ref="mediaViewerRef"
              :mode="2"
              :isFullScreen="isFullScreen"
              :file="fileInfo"
              :hasPrevious="fileIndex > 0"
              :hasNext="fileIndex < fileCount - 1"
              :fileIndex="fileIndex"
              :fileCount="fileCount"
              :isSlideShow="false"
              :imageScale="imageScale"
              :imageMinScale="imageMinScale"
              :imageMaxScale="imageMaxScale"
              :isZoomFit="isZoomFit"
              @prev="clickPrev('left')"
              @next="clickNext('left')"
              @scale="clickScale($event, 'left')"
              @update:isZoomFit="(val) => handleZoomFitUpdate(val, 'left')"
              @media-dblclick="toggleZoomFit('left')"
              @viewport-change="handleViewportChange($event, 'left')"
              @close="appWindow.close()"
            />
            <div v-if="config.settings.showComment && fileInfo?.comments?.length > 0" 
              class="absolute flex m-2 p-2 bottom-0 left-0 right-0 text-sm bg-base-100/30 rounded-box select-text"
            >
              <IconComment class="t-icon-size-sm shrink-0 mr-2"></IconComment>
              {{ fileInfo?.comments }}
            </div>
          </div>

          <div
            class="relative w-1/2 h-full"
            @mousedown="setActivePane('right')"
          >
            <IconDot
              v-if="activePane === 'right'"
              class="absolute left-2 top-2 z-90 t-icon-size-sm text-primary pointer-events-none"
            />
            <MediaViewer
              ref="rightMediaViewerRef"
              :mode="2"
              :isFullScreen="isFullScreen"
              :file="rightFileInfo"
              :hasPrevious="rightFileIndex > 0"
              :hasNext="rightFileIndex < fileCount - 1"
              :fileIndex="rightFileIndex"
              :fileCount="fileCount"
              :isSlideShow="false"
              :imageScale="rightImageScale"
              :imageMinScale="rightImageMinScale"
              :imageMaxScale="rightImageMaxScale"
              :isZoomFit="rightIsZoomFit"
              @prev="clickPrev('right')"
              @next="clickNext('right')"
              @scale="clickScale($event, 'right')"
              @update:isZoomFit="(val) => handleZoomFitUpdate(val, 'right')"
              @media-dblclick="toggleZoomFit('right')"
              @viewport-change="handleViewportChange($event, 'right')"
              @close="appWindow.close()"
            />
            <div v-if="config.settings.showComment && rightFileInfo?.comments?.length > 0" 
              class="absolute flex m-2 p-2 bottom-0 left-0 right-0 text-sm bg-base-100/30 rounded-box select-text"
            >
              <IconComment class="t-icon-size-sm shrink-0 mr-2"></IconComment>
              {{ rightFileInfo?.comments }}
            </div>
          </div>
        </div>
      </template>

      <!-- no image selected -->
      <div v-else class="flex flex-col items-center justify-center w-full h-full text-base-content/30">
        <IconSearch class="w-8 h-8" />
        <span>{{ $t('tooltip.not_found.files') }}</span>
      </div>
    </div>

    <div
      v-if="config.settings.showStatusBar"
      class="absolute bottom-0 left-0 right-0 z-30 h-8 bg-base-300/80 backdrop-blur-md"
    >
      <template v-if="!isSplit">
        <StatusBar
          :selected-file="fileInfo"
          :selected-item-index="fileIndex"
          :total-file-count="fileCount"
          :total-file-size="fileInfo?.size || 0"
          :image-scale="imageScale"
          :show-scale="true"
          :is-embedded="true"
        />
      </template>
      <template v-else>
        <div class="h-8 flex">
          <div class="w-1/2 border-r border-base-content/10">
            <StatusBar
              :selected-file="fileInfo"
              :selected-item-index="fileIndex"
              :total-file-count="fileCount"
              :total-file-size="fileInfo?.size || 0"
              :image-scale="imageScale"
              :show-scale="true"
              :is-embedded="true"
            />
          </div>
          <div class="w-1/2">
            <StatusBar
              :selected-file="rightFileInfo"
              :selected-item-index="rightFileIndex"
              :total-file-count="fileCount"
              :total-file-size="rightFileInfo?.size || 0"
              :image-scale="rightImageScale"
              :show-scale="true"
              :is-embedded="true"
            />
          </div>
        </div>
      </template>
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
import TButton from '@/components/TButton.vue';
import StatusBar from '@/components/StatusBar.vue';

import { 
  IconSearch,
  IconComment,
  IconDot,
  IconLink,
  IconSplitOn,
  IconSplitOff,
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
const rightMediaViewerRef = ref<any>(null); // right media viewer reference (split mode)
const isFullScreen = ref(false);
const isZoomFit = ref(true);
const rightIsZoomFit = ref(true);
const isSplit = ref(false);
const activePane = ref<'left' | 'right'>('left');
const isSyncViewport = ref(false);
const syncingPane = ref<'left' | 'right' | ''>('');
const animateSyncOnce = ref(false);

const isSlideShow = ref(false);     // Slide show state
let timer: NodeJS.Timeout | null = null;  // Timer for slide show

const imageScale = ref(1);          // Image scale
const imageMinScale = ref(0);       // Minimum image scale
const imageMaxScale = ref(10);      // Maximum image scale
const rightImageScale = ref(1);     // Right image scale
const rightImageMinScale = ref(0);  // Right minimum scale
const rightImageMaxScale = ref(10); // Right maximum scale

const rightFileId = ref(0);         // Right file ID
const rightFileIndex = ref(-1);     // Right file index
const rightFileInfo = ref<any>(null);

let unlistenImg: () => void;
let unlistenGridView: () => void;

onMounted(async() => {
  window.addEventListener('keydown', handleKeyDown);
  window.addEventListener('resize', handleResize);

  const urlParams = new URLSearchParams(window.location.search);
  
  fileId.value    = Number(urlParams.get('fileId'));
  fileIndex.value = Number(urlParams.get('fileIndex'));
  fileCount.value = Number(urlParams.get('fileCount'));
  const initialRightFileId = Number(urlParams.get('rightFileId') || '0');
  const initialRightFileIndex = Number(urlParams.get('rightFileIndex') || '-1');
  const forceSplit = urlParams.get('forceSplit') === '1';

  isSplit.value = forceSplit ? true : !!config.imageViewer?.isSplit;
  isSyncViewport.value = isSplit.value ? !!config.imageViewer?.isSyncViewport : false;
  rightFileId.value = initialRightFileId > 0 ? initialRightFileId : 0;
  rightFileIndex.value = initialRightFileId > 0 ? initialRightFileIndex : -1;
  rightFileInfo.value = null;
  rightIsZoomFit.value = true;
  activePane.value = 'left';

  // Listen 
  unlistenImg = await listen('update-img', async (event: any) => {
    if(uiStore.inputStack.length > 0) {
      return;
    }

    const pane = event.payload?.pane === 'right' ? 'right' : 'left';
    if (typeof event.payload?.forceSplit === 'boolean') {
      isSplit.value = !!event.payload.forceSplit;
      if (!isSplit.value) {
        rightFileId.value = 0;
        rightFileIndex.value = -1;
        rightFileInfo.value = null;
        rightIsZoomFit.value = true;
      }
    }
    if (event.payload?.resetSplit) {
      isSplit.value = !!config.imageViewer?.isSplit;
      isSyncViewport.value = isSplit.value ? !!config.imageViewer?.isSyncViewport : false;
      if (!isSplit.value) {
        rightFileId.value = 0;
        rightFileIndex.value = -1;
        rightFileInfo.value = null;
        rightIsZoomFit.value = true;
      }
    }

    fileCount.value = Number(event.payload.fileCount);
    if (pane === 'right') {
      rightFileId.value = Number(event.payload.fileId);
      rightFileIndex.value = Number(event.payload.fileIndex);
    } else {
      fileId.value = Number(event.payload.fileId);
      fileIndex.value = Number(event.payload.fileIndex);
    }
  });


  unlistenGridView = await listen('message-from-content', (event) => {
    const { message, fileId: targetFileId } = event.payload as any;
    console.log('message-from-content:', message, targetFileId);
    switch (message) {
      case 'rotate':
        if (targetFileId === fileId.value) {
          mediaViewerRef.value?.rotateRight();
          iconRotate.value += 90;
          if (fileInfo.value) {
            fileInfo.value.rotate = (fileInfo.value.rotate || 0) + 90;
          }
        } else if (targetFileId === rightFileId.value) {
          rightMediaViewerRef.value?.rotateRight();
          if (rightFileInfo.value) {
            rightFileInfo.value.rotate = (rightFileInfo.value.rotate || 0) + 90;
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
  ArrowLeft:  () => clickPrev(getActivePane()),
  ArrowRight: () => clickNext(getActivePane()),
  Home:       () => clickHome(getActivePane()),
  End:        () => clickEnd(getActivePane()),
  ArrowUp:    () => clickZoomIn(getActivePane()),
  ArrowDown:  () => clickZoomOut(getActivePane()),
  '=':        () => clickZoomIn(getActivePane()),
  '-':        () => clickZoomOut(getActivePane()),
  '0':        () => clickZoomActual(getActivePane()),
  ' ':        () => toggleZoomFit(getActivePane()),
  Escape:     () => closeWindow(),
};

function getActivePane(): 'left' | 'right' {
  return isSplit.value ? activePane.value : 'left';
}

function setActivePane(pane: 'left' | 'right') {
  activePane.value = pane;
}

function getViewerRef(pane: 'left' | 'right') {
  return pane === 'right' ? rightMediaViewerRef.value : mediaViewerRef.value;
}

function getFileInfoByPane(pane: 'left' | 'right') {
  return pane === 'right' ? rightFileInfo.value : fileInfo.value;
}

function isImagePane(pane: 'left' | 'right') {
  return getFileInfoByPane(pane)?.file_type === 1;
}

function syncViewportFrom(pane: 'left' | 'right', animate = false) {
  if (!isSplit.value || !isSyncViewport.value) return;
  if (!isImagePane('left') || !isImagePane('right')) return;

  const sourceRef = getViewerRef(pane);
  const targetPane = pane === 'left' ? 'right' : 'left';
  const targetRef = getViewerRef(targetPane);
  const viewport = sourceRef?.getViewportState?.();
  if (!viewport) return;

  syncingPane.value = pane;
  targetRef?.applyViewportState?.(viewport, !animate);
  syncingPane.value = '';
}

function handleViewportChange(viewport: any, pane: 'left' | 'right') {
  if (!isSplit.value || !isSyncViewport.value) return;
  if (syncingPane.value) return;
  if (!isImagePane('left') || !isImagePane('right')) return;

  const targetPane = pane === 'left' ? 'right' : 'left';
  const shouldAnimate = animateSyncOnce.value;
  animateSyncOnce.value = false;
  syncingPane.value = pane;
  // Drag/wheel sync stays no-animation; zoom-fit sync can opt-in animation.
  getViewerRef(targetPane)?.applyViewportState?.(viewport, !shouldAnimate);
  syncingPane.value = '';
}

function getZoomFitByPane(pane: 'left' | 'right') {
  return pane === 'right' ? rightIsZoomFit.value : isZoomFit.value;
}

function setZoomFitByPane(pane: 'left' | 'right', val: boolean) {
  if (pane === 'right') {
    rightIsZoomFit.value = val;
    return;
  }
  isZoomFit.value = val;
}

function handleZoomFitUpdate(val: boolean, pane: 'left' | 'right') {
  setActivePane(pane);
  setZoomFitByPane(pane, val);
  if (isSplit.value && isSyncViewport.value && isImagePane('left') && isImagePane('right')) {
    animateSyncOnce.value = true;
  }
}

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

watch(() => rightFileId.value, async () => {
  if (rightFileId.value > 0) {
    rightFileInfo.value = await getFileInfo(rightFileId.value);
  } else {
    rightFileInfo.value = null;
  }
});

// watch file index
watch(() => fileIndex.value, async (newIndex) => {
  if(newIndex === -1) {
    isSlideShow.value = false;
    iconRotate.value = 0; // reset rotation
  } 
});

// Check if current file is a video
function isCurrentFileVideo() {
  return fileInfo.value?.file_type === 2;
}

// Schedule next slide based on file type
function scheduleNextSlide() {
  if (timer) {
    clearTimeout(timer);
    timer = null;
  }
  
  if (!isSlideShow.value) return;
  
  // If current file is video, don't set timer - video's ended event will trigger next
  if (isCurrentFileVideo()) {
    return;
  }
  
  // For images, use the configured interval
  const interval = getSlideShowInterval(config.settings.slideShowInterval) * 1000;
  timer = setTimeout(() => {
    clickNext();
    scheduleNextSlide();
  }, interval);
}

// Called when video ends in slideshow mode
function handleSlideshowNext() {
  if (isSlideShow.value) {
    clickNext();
    scheduleNextSlide();
  }
}

watch(() => [isSlideShow.value, config.settings.slideShowInterval], ([newIsSlideShow]) => {
  if(newIsSlideShow) {
    scheduleNextSlide();
  } else {
    if (timer) {
      clearTimeout(timer);
      timer = null;
    }
  }
});

function ensureRightPaneLoaded() {
  if (!isSplit.value) return;
  if (rightFileIndex.value >= 0 && rightFileId.value > 0) return;
  if (fileCount.value <= 0 || fileIndex.value < 0) return;

  const nextIndex = Math.min(fileIndex.value + 1, fileCount.value - 1);
  requestFileAtIndex(nextIndex, 'right');
}

watch(() => isSplit.value, (val) => {
  if (!config.imageViewer) {
    (config as any).imageViewer = { isSplit: false, isSyncViewport: false };
  }
  config.imageViewer.isSplit = val;
  if (!val) {
    isSyncViewport.value = false;
  } else {
    ensureRightPaneLoaded();
  }
});

watch(() => isSyncViewport.value, (val) => {
  if (!config.imageViewer) {
    (config as any).imageViewer = { isSplit: false, isSyncViewport: false };
  }
  config.imageViewer.isSyncViewport = val;
});

watch(() => [fileIndex.value, fileCount.value], () => {
  ensureRightPaneLoaded();
});

function requestFileAtIndex(index: number, pane: 'left' | 'right' = 'left') {
  emit('message-from-image-viewer', { message: 'request-file-at-index', index, pane });
}

function clickPrev(pane: 'left' | 'right' = 'left') {
  setActivePane(pane);
  const currentIndex = pane === 'right' ? rightFileIndex.value : fileIndex.value;
  const viewerRef = pane === 'right' ? rightMediaViewerRef.value : mediaViewerRef.value;
  if (currentIndex > 0) {
    requestFileAtIndex(currentIndex - 1, pane);
  } else {
    viewerRef?.showMessage((localeMsg.value as any).tooltip.image_viewer.first_image);
  }
}

function clickNext(pane: 'left' | 'right' = 'left') {
  setActivePane(pane);
  const currentIndex = pane === 'right' ? rightFileIndex.value : fileIndex.value;
  const viewerRef = pane === 'right' ? rightMediaViewerRef.value : mediaViewerRef.value;

  // Fix loop for slideshow
  if (pane === 'left' && isSlideShow.value && currentIndex >= fileCount.value - 1) {
    requestFileAtIndex(0, pane);
    return;
  }
  
  if (currentIndex < fileCount.value - 1) {
    requestFileAtIndex(currentIndex + 1, pane);
  } else {
    viewerRef?.showMessage((localeMsg.value as any).tooltip.image_viewer.last_image);
  }
}

function clickHome(pane: 'left' | 'right' = 'left') {
  setActivePane(pane);
  requestFileAtIndex(0, pane);
}

function clickEnd(pane: 'left' | 'right' = 'left') {
  setActivePane(pane);
  requestFileAtIndex(fileCount.value - 1, pane);
}

function clickSlideShow() {
  isSlideShow.value = !isSlideShow.value;
}

const clickZoomIn = (pane: 'left' | 'right' = 'left') => {
  setActivePane(pane);
  const viewerRef = pane === 'right' ? rightMediaViewerRef.value : mediaViewerRef.value;
  viewerRef?.zoomIn();
};

const clickZoomOut = (pane: 'left' | 'right' = 'left') => {
  setActivePane(pane);
  const viewerRef = pane === 'right' ? rightMediaViewerRef.value : mediaViewerRef.value;
  viewerRef?.zoomOut();
};

const clickZoomActual = (pane: 'left' | 'right' = 'left') => {
  setActivePane(pane);
  const viewerRef = pane === 'right' ? rightMediaViewerRef.value : mediaViewerRef.value;
  viewerRef?.zoomActual();
};

const toggleZoomFit = (pane: 'left' | 'right' = 'left') => {
  const current = getZoomFitByPane(pane);
  handleZoomFitUpdate(!current, pane);
};

const closeWindow = () => {
  if(isFullScreen.value) {
    isFullScreen.value = false;
    appWindow.setFocus();
  } else {
    appWindow.close();
  }
}

const clickScale = (event: any, pane: 'left' | 'right' = 'left') => {
  if (pane === 'right') {
    rightImageScale.value = event.scale;
    rightImageMinScale.value = event.minScale;
    rightImageMaxScale.value = event.maxScale;
    return;
  }

  imageScale.value = event.scale;
  imageMinScale.value = event.minScale;
  imageMaxScale.value = event.maxScale;
};

const toggleSplit = () => {
  const willEnable = !isSplit.value;
  isSplit.value = willEnable;
  activePane.value = 'left';
  if (!willEnable) {
    isSyncViewport.value = false;
  }

  if (willEnable) {
    if (isSlideShow.value) {
      isSlideShow.value = false;
    }
    rightIsZoomFit.value = true;
    rightImageScale.value = 1;
    rightImageMinScale.value = 0;
    rightImageMaxScale.value = 10;

    if (fileCount.value > 0) {
      const nextIndex = Math.min(fileIndex.value + 1, fileCount.value - 1);
      requestFileAtIndex(nextIndex, 'right');
    }
  }
};

const toggleSyncViewport = () => {
  if (!isSplit.value) return;
  isSyncViewport.value = !isSyncViewport.value;
  if (isSyncViewport.value) {
    syncViewportFrom(activePane.value);
  }
};

</script>

<style scoped>
* {
  user-select: none;
}
</style>

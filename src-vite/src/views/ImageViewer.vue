<template>

  <div
    :class="[
      'relative w-screen h-screen flex flex-col overflow-hidden bg-base-300 text-base-content/70',
      isFullScreen ? 'fixed top-0 left-0 z-50' : '',
    ]"
    @mousemove="handleRootMouseMove"
    @mouseleave="handleRootMouseLeave"
  >

    <div
      ref="viewerContainer"
      :class="[
        'relative flex-1 flex justify-center items-center overflow-hidden select-none',
        showEmbeddedStatusBar ? 'pb-8' : '',
      ]"
    >
      <template v-if="!isSplit && fileIndex >= 0">
        <MediaViewer
          ref="mediaViewerRef"
          :mode="2"
          :isFullScreen="isFullScreen"
          :file="fileInfo"
          :nextFilePath="nextFilePath"
          :hasPrevious="fileIndex > 0"
          :hasNext="fileIndex < fileCount - 1"
          :fileIndex="fileIndex"
          :fileCount="fileCount"
          :isSlideShow="isSlideShow"
          :canSlideShow="true"
          :slideShowIntervalIndex="slideShowIntervalIndex"
          :canInteract="true"
          :imageScale="imageScale"
          :imageMinScale="imageMinScale"
          :imageMaxScale="imageMaxScale"
          :isZoomFit="isZoomFit"
          :isSplit="isSplit"
          :isSyncViewport="isSyncViewport"
          :showWindowControls="true"
          @prev="clickPrev()"
          @next="clickNext()"
          @toggle-slide-show="clickSlideShow()"
          @update:slideShowIntervalIndex="slideShowIntervalIndex = $event"
          @item-action="handleItemAction"
          @scale="clickScale"
          @update:isZoomFit="(val) => handleZoomFitUpdate(val, 'left')"
          @media-dblclick="toggleZoomFit()"
          @toggle-full-screen="toggleNativeFullScreen"
          @close="closeWindow"
          @slideshow-next="handleSlideshowNext"
        />

        <!--
        <div v-if="config.settings.showComment && fileInfo?.comments?.length > 0" 
          class="absolute flex m-2 p-2 bottom-0 left-0 right-0 text-sm bg-base-100/30 rounded-box select-text" 
        >
          <IconComment class="t-icon-size-sm shrink-0 mr-2"></IconComment>
          {{ fileInfo?.comments }}
        </div>
        -->
      </template>

      <template v-else-if="isSplit && fileIndex >= 0">
        <div class="w-full h-full flex flex-col">
          <!-- Shared toolbar above both panes -->
          <MediaViewer
            :mode="2"
            :toolbarOnly="true"
            :showToolbar="true"
            :showWindowControls="true"
            :isFullScreen="isFullScreen"
            :file="activePane === 'left' ? fileInfo : rightFileInfo"
            :nextFilePath="activePane === 'left' ? nextFilePath : rightNextFilePath"
            :hasPrevious="activePane === 'left' ? fileIndex > 0 : rightFileIndex > 0"
            :hasNext="activePane === 'left' ? fileIndex < fileCount - 1 : rightFileIndex < fileCount - 1"
            :fileIndex="activePane === 'left' ? fileIndex : rightFileIndex"
            :fileCount="fileCount"
            :isSlideShow="false"
            :canSlideShow="false"
            :canInteract="true"
            :imageScale="activePane === 'left' ? imageScale : rightImageScale"
            :imageMinScale="activePane === 'left' ? imageMinScale : rightImageMinScale"
            :imageMaxScale="activePane === 'left' ? imageMaxScale : rightImageMaxScale"
            :isZoomFit="activePane === 'left' ? isZoomFit : rightIsZoomFit"
            :isSplit="isSplit"
            :isSyncViewport="isSyncViewport"
            :forceToolbarVisible="isFullScreen && splitToolbarVisible"
            @prev="clickPrev(activePane)"
            @next="clickNext(activePane)"
            @toggle-slide-show="clickSlideShow(activePane)"
            @item-action="handleItemAction"
            @scale="clickScale($event, activePane)"
            @update:isZoomFit="(val) => handleZoomFitUpdate(val, activePane)"
            @toggle-full-screen="toggleNativeFullScreen"
            @close="closeWindow"
            @slideshow-next="handleSlideshowNext"
          />

          <!-- Split Panes -->
          <div class="flex-1 flex min-h-0">
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
                :nextFilePath="nextFilePath"
                :hasPrevious="fileIndex > 0"
                :hasNext="fileIndex < fileCount - 1"
                :fileIndex="fileIndex"
                :fileCount="fileCount"
                :isSlideShow="false"
                :canSlideShow="false"
                :canInteract="activePane === 'left'"
                :showToolbar="false"
                :imageScale="imageScale"
                :imageMinScale="imageMinScale"
                :imageMaxScale="imageMaxScale"
                :isZoomFit="isZoomFit"
                @prev="clickPrev('left')"
                @next="clickNext('left')"
                @toggle-slide-show="clickSlideShow('left')"
                @item-action="handleItemAction"
                @scale="clickScale($event, 'left')"
                @update:isZoomFit="(val) => handleZoomFitUpdate(val, 'left')"
                @media-dblclick="toggleZoomFit('left')"
                @viewport-change="handleViewportChange($event, 'left')"
                @toggle-full-screen="toggleNativeFullScreen"
                @close="closeWindow"
                @slideshow-next="handleSlideshowNext"
              />
              <!--
              <div v-if="config.settings.showComment && fileInfo?.comments?.length > 0" 
                class="absolute flex m-2 p-2 bottom-0 left-0 right-0 text-sm bg-base-100/30 rounded-box select-text"
              >
                <IconComment class="t-icon-size-sm shrink-0 mr-2"></IconComment>
                {{ fileInfo?.comments }}
              </div>
              -->
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
                :nextFilePath="rightNextFilePath"
                :hasPrevious="rightFileIndex > 0"
                :hasNext="rightFileIndex < fileCount - 1"
                :fileIndex="rightFileIndex"
                :fileCount="fileCount"
                :isSlideShow="false"
                :canSlideShow="false"
                :canInteract="activePane === 'right'"
                :showToolbar="false"
                :imageScale="rightImageScale"
                :imageMinScale="rightImageMinScale"
                :imageMaxScale="rightImageMaxScale"
                :isZoomFit="rightIsZoomFit"
                @prev="clickPrev('right')"
                @next="clickNext('right')"
                @toggle-slide-show="clickSlideShow('right')"
                @item-action="handleItemAction"
                @scale="clickScale($event, 'right')"
                @update:isZoomFit="(val) => handleZoomFitUpdate(val, 'right')"
                @media-dblclick="toggleZoomFit('right')"
                @viewport-change="handleViewportChange($event, 'right')"
                @toggle-full-screen="toggleNativeFullScreen"
                @close="closeWindow"
                @slideshow-next="handleSlideshowNext"
              />
              <!--
              <div v-if="config.settings.showComment && rightFileInfo?.comments?.length > 0" 
                class="absolute flex m-2 p-2 bottom-0 left-0 right-0 text-sm bg-base-100/30 rounded-box select-text"
              >
                <IconComment class="t-icon-size-sm shrink-0 mr-2"></IconComment>
                {{ rightFileInfo?.comments }}
              </div>
              -->
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
      v-if="showEmbeddedStatusBar"
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

    <TaggingDialog
      v-if="showTaggingDialog"
      :fileIds="taggingFileIds"
      @ok="updateFileHasTags"
      @cancel="showTaggingDialog = false"
    />

    <MessageBox
      v-if="showCommentMsgbox"
      :title="$t('msgbox.edit_comment.title')"
      :showInput="true"
      :inputText="activeFileInfo?.comments ?? ''"
      :inputPlaceholder="$t('msgbox.edit_comment.placeholder')"
      :multiLine="true"
      :OkText="$t('msgbox.ok')"
      :cancelText="$t('msgbox.cancel')"
      @ok="onEditComment"
      @cancel="showCommentMsgbox = false"
    />

  </div>

</template>

<script setup lang="ts">

import { ref, watch, computed, onMounted, onUnmounted } from 'vue';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { emit, listen } from '@tauri-apps/api/event';
import { useI18n } from 'vue-i18n';
import { useUIStore } from '@/stores/uiStore';
import { config } from '@/common/config';
import { isWin, isMac, setTheme, getSlideShowInterval, SCALE_VALUES } from '@/common/utils';
import {
  editFileComment,
  getFileInfo,
  getTagsForFile,
  setFileFavorite,
  setFileRating,
  setFileRotate,
} from '@/common/api';

import MediaViewer from '@/components/MediaViewer.vue';
import MessageBox from '@/components/MessageBox.vue';
import TButton from '@/components/TButton.vue';
import StatusBar from '@/components/StatusBar.vue';
import TaggingDialog from '@/components/TaggingDialog.vue';

import { 
  IconSearch,
  IconComment,
  IconDot,
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
const nextFilePath = ref('');
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
const isCompareModeSession = ref(false);
const syncingPane = ref<'left' | 'right' | ''>('');
const animateSyncOnce = ref(false);
const splitToolbarVisible = ref(false);

const isSlideShow = ref(false);     // Slide show state
const slideShowIntervalIndex = ref(Number(config.settings.slideShowInterval ?? 0));
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
const rightNextFilePath = ref('');
const showTaggingDialog = ref(false);
const showCommentMsgbox = ref(false);
const taggingFileIds = ref<number[]>([]);

let unlistenImg: () => void;
let unlistenGridView: () => void;
let unlistenFilesDeleted: (() => void) | null = null;

const activeFileInfo = computed(() => {
  return isSplit.value && activePane.value === 'right' ? rightFileInfo.value : fileInfo.value;
});

const activeFileId = computed(() => {
  return isSplit.value && activePane.value === 'right' ? rightFileId.value : fileId.value;
});
const showEmbeddedStatusBar = computed(() => config.settings.showStatusBar && !isFullScreen.value);

function normalizeScale(value: number) {
  return SCALE_VALUES.find((item) => item === Number(value)) ?? 1;
}

function applyViewerScale(scale: number) {
  const normalizedScale = normalizeScale(scale);
  document.documentElement.style.fontSize = `${normalizedScale * 16}px`;
}

function handleRootMouseMove(event: MouseEvent) {
  if (!isFullScreen.value || !isSplit.value) {
    splitToolbarVisible.value = false;
    return;
  }
  const root = event.currentTarget as HTMLElement | null;
  if (!root) return;
  const rect = root.getBoundingClientRect();
  splitToolbarVisible.value = event.clientY - rect.top < 60;
}

function handleRootMouseLeave() {
  splitToolbarVisible.value = false;
}

onMounted(async() => {
  appWindow.setFocus();
  applyViewerScale(Number(config.settings.scale || 1));
  window.addEventListener('keydown', handleKeyDown);
  window.addEventListener('resize', handleResize);

  const urlParams = new URLSearchParams(window.location.search);
  
  fileId.value    = Number(urlParams.get('fileId'));
  fileIndex.value = Number(urlParams.get('fileIndex'));
  fileCount.value = Number(urlParams.get('fileCount'));
  nextFilePath.value = decodeURIComponent(urlParams.get('nextFilePath') || '');
  const initialRightFileId = Number(urlParams.get('rightFileId') || '0');
  const initialRightFileIndex = Number(urlParams.get('rightFileIndex') || '-1');
  rightNextFilePath.value = decodeURIComponent(urlParams.get('rightNextFilePath') || '');
  const forceSplit = urlParams.get('forceSplit') === '1';
  isCompareModeSession.value = urlParams.get('compareMode') === '1';

  isSplit.value = forceSplit ? true : !!config.imageViewer?.isSplit;
  if (isCompareModeSession.value) {
    isSplit.value = true;
    isSyncViewport.value = true;
  } else {
    isSyncViewport.value = isSplit.value ? !!config.imageViewer?.isSyncViewport : false;
  }
  rightFileId.value = initialRightFileId > 0 ? initialRightFileId : 0;
  rightFileIndex.value = initialRightFileId > 0 ? initialRightFileIndex : -1;
  rightFileInfo.value = null;
  rightIsZoomFit.value = true;
  activePane.value = 'left';
  isFullScreen.value = !!config.imageViewer?.isFullScreen;

  // Listen 
  unlistenImg = await listen('update-img', async (event: any) => {
    if(uiStore.inputStack.length > 0) {
      return;
    }

    const pane = event.payload?.pane === 'right' ? 'right' : 'left';
    if (typeof event.payload?.compareMode === 'boolean') {
      isCompareModeSession.value = !!event.payload.compareMode;
    }
    if (typeof event.payload?.forceSplit === 'boolean') {
      isSplit.value = !!event.payload.forceSplit;
      if (isSplit.value && typeof event.payload?.forceSyncViewport === 'boolean') {
        isSyncViewport.value = !!event.payload.forceSyncViewport;
      }
      if (!isSplit.value) {
        rightFileId.value = 0;
        rightFileIndex.value = -1;
        rightFileInfo.value = null;
        rightNextFilePath.value = '';
        rightIsZoomFit.value = true;
      }
    }
    if (event.payload?.resetSplit) {
      if (isCompareModeSession.value) {
        isSplit.value = true;
        isSyncViewport.value = true;
      } else {
        isSplit.value = !!config.imageViewer?.isSplit;
        isSyncViewport.value = isSplit.value ? !!config.imageViewer?.isSyncViewport : false;
      }
      if (!isSplit.value) {
        rightFileId.value = 0;
        rightFileIndex.value = -1;
        rightFileInfo.value = null;
        rightNextFilePath.value = '';
        rightIsZoomFit.value = true;
      }
    }

    fileCount.value = Number(event.payload.fileCount);
    if (pane === 'right') {
      rightFileId.value = Number(event.payload.fileId);
      rightFileIndex.value = Number(event.payload.fileIndex);
      rightNextFilePath.value = event.payload.nextFilePath || '';
    } else {
      fileId.value = Number(event.payload.fileId);
      fileIndex.value = Number(event.payload.fileIndex);
      nextFilePath.value = event.payload.nextFilePath || '';
    }
  });


  unlistenGridView = await listen('message-from-content', (event) => {
    const { message, fileId: targetFileId, changes } = event.payload as any;
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
      case 'update-file-meta':
        if (targetFileId === fileId.value && fileInfo.value) {
          Object.assign(fileInfo.value, changes || {});
        }
        if (targetFileId === rightFileId.value && rightFileInfo.value) {
          Object.assign(rightFileInfo.value, changes || {});
        }
        break;
      default:
        break;
    }
  });

  unlistenFilesDeleted = await listen('files-deleted', (event: any) => {
    const deletedIds = Array.isArray(event?.payload?.fileIds)
      ? event.payload.fileIds.map((id: any) => Number(id)).filter((id: number) => id > 0)
      : [];
    const nextCount = Number(event?.payload?.fileCount);
    if (!Number.isNaN(nextCount) && nextCount >= 0) {
      fileCount.value = nextCount;
    }

    if (fileCount.value <= 0) {
      fileId.value = 0;
      fileIndex.value = -1;
      nextFilePath.value = '';
      rightFileId.value = 0;
      rightFileIndex.value = -1;
      rightFileInfo.value = null;
      rightNextFilePath.value = '';
      return;
    }

    const leftDeleted = deletedIds.includes(fileId.value);
    const rightDeleted = deletedIds.includes(rightFileId.value);

    if (leftDeleted || fileIndex.value >= fileCount.value) {
      const targetIndex = Math.max(0, Math.min(fileIndex.value, fileCount.value - 1));
      requestFileAtIndex(targetIndex, 'left');
    }

    if (isSplit.value && (rightDeleted || rightFileIndex.value >= fileCount.value)) {
      const fallbackBase = rightFileIndex.value >= 0 ? rightFileIndex.value : (fileIndex.value + 1);
      const targetIndex = Math.max(0, Math.min(fallbackBase, fileCount.value - 1));
      requestFileAtIndex(targetIndex, 'right');
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
  document.documentElement.style.fontSize = '';
  clearSlideShowTimer();
  
  // unlisten
  unlistenImg();
  unlistenGridView();
  if (unlistenFilesDeleted) unlistenFilesDeleted();
});

// Handle keyboard shortcuts
function handleKeyDown(event: KeyboardEvent) {
  if(uiStore.inputStack.length > 0) {
    return;
  }

  // Disable keyboard events during slideshow except the toggle shortcut.
  if (isSlideShow.value && event.key !== 'Escape' && event.code !== 'KeyP') {
    return;
  }

  const ratingShortcut = Number.parseInt(event.key, 10);
  const hasModifier = event.metaKey || event.ctrlKey || event.altKey;

  if (!hasModifier && Number.isInteger(ratingShortcut) && ratingShortcut >= 0 && ratingShortcut <= 5) {
    event.preventDefault();
    void setCurrentFileRating(ratingShortcut, getActiveFilePane());
    return;
  }

  if (!hasModifier) {
    if (event.code === 'KeyP') {
      event.preventDefault();
      clickSlideShow(getActiveFilePane());
      return;
    }

    if (event.code === 'KeyF') {
      event.preventDefault();
      void toggleFavorite(getActiveFilePane());
      return;
    }

    if (event.code === 'KeyT') {
      event.preventDefault();
      clickTag(getActiveFilePane());
      return;
    }

    if (event.code === 'KeyC') {
      event.preventDefault();
      openCommentEditor(getActiveFilePane());
      return;
    }

    if (event.code === 'KeyR') {
      event.preventDefault();
      void clickRotate(getActiveFilePane());
      return;
    }
  }

  if (event.key === 'Tab' && isSplit.value) {
    event.preventDefault();
    setActivePane(activePane.value === 'left' ? 'right' : 'left');
    return;
  }

  const key = event.key;
  if ((keyActions as any)[key]) {
    event.preventDefault();
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

function haveMatchingSyncableMedia() {
  const leftType = fileInfo.value?.file_type;
  const rightType = rightFileInfo.value?.file_type;
  const isImageType = (t: number) => t === 1 || t === 3;
  if (isImageType(leftType) && isImageType(rightType)) return true;
  return leftType === 2 && rightType === 2;
}

function syncViewportFrom(pane: 'left' | 'right', animate = false) {
  if (!isSplit.value || !isSyncViewport.value) return;
  if (!haveMatchingSyncableMedia()) return;

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
  if (!haveMatchingSyncableMedia()) return;

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
  if (isSplit.value && isSyncViewport.value && haveMatchingSyncableMedia()) {
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

watch(() => Number(config.settings.scale || 1), (newScale) => {
  applyViewerScale(newScale);
});

// watch language
watch(() => config.settings.language, (newLanguage) => {
    console.log('Language changed to:', newLanguage);
    locale.value = newLanguage; // update locale based on config.settings.language
});

// watch full screen
watch(() => isFullScreen.value, async (newFullScreen) => {
  if (!config.imageViewer) {
    (config as any).imageViewer = { isSplit: false, isSyncViewport: false, isFullScreen: false };
  }
  config.imageViewer.isFullScreen = newFullScreen;

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
  if (isSlideShow.value) {
    scheduleNextSlide();
  }
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
    stopSlideShow();
    iconRotate.value = 0; // reset rotation
  } 
});

// Check if current file is a video
function isCurrentFileVideo() {
  return fileInfo.value?.file_type === 2;
}

function clearSlideShowTimer() {
  if (timer) {
    clearTimeout(timer);
    timer = null;
  }
}

function advanceSlideShow() {
  if (fileCount.value <= 0) return;

  if (fileIndex.value >= fileCount.value - 1) {
    requestFileAtIndex(0, 'left');
    return;
  }
  requestFileAtIndex(fileIndex.value + 1, 'left');
}

// Schedule next slide based on file type
function scheduleNextSlide() {
  clearSlideShowTimer();

  if (!isSlideShow.value) return;

  // If current file is video, don't set timer - video's ended event will trigger next
  if (isCurrentFileVideo()) {
    return;
  }

  const interval = getSlideShowInterval(slideShowIntervalIndex.value) * 1000;
  timer = setTimeout(() => {
    advanceSlideShow();
  }, interval);
}

function startSlideShow() {
  scheduleNextSlide();
}

function stopSlideShow() {
  isSlideShow.value = false;
  clearSlideShowTimer();
}

// Called when video ends in slideshow mode
function handleSlideshowNext() {
  if (isSlideShow.value) {
    advanceSlideShow();
  }
}

watch(() => slideShowIntervalIndex.value, () => {
  if (isSlideShow.value && !isCurrentFileVideo()) {
    scheduleNextSlide();
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
  if (isCompareModeSession.value) {
    if (!val) {
      isSyncViewport.value = false;
    } else {
      ensureRightPaneLoaded();
    }
    return;
  }
  if (!config.imageViewer) {
    (config as any).imageViewer = { isSplit: false, isSyncViewport: false, isFullScreen: false };
  }
  config.imageViewer.isSplit = val;
  if (!val) {
    isSyncViewport.value = false;
  } else {
    ensureRightPaneLoaded();
  }
});

watch(() => isSyncViewport.value, (val) => {
  if (isCompareModeSession.value) return;
  if (!config.imageViewer) {
    (config as any).imageViewer = { isSplit: false, isSyncViewport: false, isFullScreen: false };
  }
  config.imageViewer.isSyncViewport = val;
});

watch(() => [fileIndex.value, fileCount.value], () => {
  ensureRightPaneLoaded();
});

function requestFileAtIndex(index: number, pane: 'left' | 'right' = 'left') {
  emit('message-from-image-viewer', { message: 'request-file-at-index', index, pane });
}

function getFileInfoByPane(pane: 'left' | 'right' = 'left') {
  return pane === 'right' ? rightFileInfo.value : fileInfo.value;
}

function getFileIdByPane(pane: 'left' | 'right' = 'left') {
  return pane === 'right' ? rightFileId.value : fileId.value;
}

function getActiveFilePane() {
  return isSplit.value ? activePane.value : 'left';
}

function syncFileMetaToContent(targetFileId: number, changes: Record<string, any>) {
  emit('message-from-image-viewer', {
    message: 'update-file-meta',
    fileId: targetFileId,
    changes,
  });
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
  if (isSlideShow.value && currentIndex >= fileCount.value - 1) {
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

function clickSlideShow(pane: 'left' | 'right' = 'left') {
  setActivePane(pane);
  isSlideShow.value = !isSlideShow.value;
  if (isSlideShow.value) {
    startSlideShow();
  } else {
    stopSlideShow();
  }
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

const toggleNativeFullScreen = () => {
  isFullScreen.value = !isFullScreen.value;
};

const closeWindow = () => {
  appWindow.close();
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
      stopSlideShow();
    }
    rightIsZoomFit.value = true;
    rightImageScale.value = 1;
    rightImageMinScale.value = 0;
    rightImageMaxScale.value = 10;

    if (fileCount.value > 0) {
      const nextIndex = Math.min(fileIndex.value + 1, fileCount.value - 1);
      requestFileAtIndex(nextIndex, 'right');
    }
  } else {
    if (isSlideShow.value) {
      scheduleNextSlide();
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

const toggleFavorite = async (pane: 'left' | 'right' = 'left') => {
  const target = getFileInfoByPane(pane);
  const currentFileId = getFileIdByPane(pane);
  if (!target || currentFileId <= 0) return;

  target.is_favorite = !target.is_favorite;
  await setFileFavorite(currentFileId, target.is_favorite);
  syncFileMetaToContent(currentFileId, { is_favorite: target.is_favorite });
};

const setCurrentFileRating = async (rating: number, pane: 'left' | 'right' = 'left') => {
  const target = getFileInfoByPane(pane);
  const currentFileId = getFileIdByPane(pane);
  if (!target || currentFileId <= 0) return;

  const normalized = Number(target.rating || 0) === rating ? 0 : rating;
  target.rating = normalized;
  await setFileRating(currentFileId, normalized);
  syncFileMetaToContent(currentFileId, { rating: normalized });
};

const clickRotate = async (pane: 'left' | 'right' = 'left') => {
  const target = getFileInfoByPane(pane);
  const currentFileId = getFileIdByPane(pane);
  const viewerRef = getViewerRef(pane);
  if (!target || currentFileId <= 0) return;

  target.rotate = (Number(target.rotate) || 0) + 90;
  viewerRef?.rotateRight?.();
  await setFileRotate(currentFileId, target.rotate);
  syncFileMetaToContent(currentFileId, { rotate: target.rotate });
};

const clickTag = (pane: 'left' | 'right' = 'left') => {
  const currentFileId = getFileIdByPane(pane);
  if (currentFileId <= 0) return;

  setActivePane(pane);
  taggingFileIds.value = [currentFileId];
  showTaggingDialog.value = true;
};

const openCommentEditor = (pane: 'left' | 'right' = 'left') => {
  const currentFileId = getFileIdByPane(pane);
  if (currentFileId <= 0) return;

  setActivePane(pane);
  showCommentMsgbox.value = true;
};

const onEditComment = async (newComment: any) => {
  const target = activeFileInfo.value;
  const currentFileId = activeFileId.value;
  if (!target || currentFileId <= 0) return;

  const result = await editFileComment(currentFileId, newComment);
  if (result) {
    target.comments = newComment;
    showCommentMsgbox.value = false;
    syncFileMetaToContent(currentFileId, { comments: newComment });
  }
};

async function updateFileHasTags(fileIds: number[]) {
  if (!Array.isArray(fileIds) || fileIds.length === 0) {
    showTaggingDialog.value = false;
    return;
  }

  for (const taggedFileId of fileIds) {
    if (taggedFileId === fileId.value && fileInfo.value) {
      const tags = (await getTagsForFile(taggedFileId)) || [];
      fileInfo.value.has_tags = tags.length > 0;
      fileInfo.value.tags = tags;
      syncFileMetaToContent(taggedFileId, { has_tags: fileInfo.value.has_tags, tags });
    }

    if (taggedFileId === rightFileId.value && rightFileInfo.value) {
      const tags = (await getTagsForFile(taggedFileId)) || [];
      rightFileInfo.value.has_tags = tags.length > 0;
      rightFileInfo.value.tags = tags;
      syncFileMetaToContent(taggedFileId, { has_tags: rightFileInfo.value.has_tags, tags });
    }
  }

  showTaggingDialog.value = false;
}

const handleItemAction = async (payload: { action: string }) => {
  const pane = getActiveFilePane();

  switch (payload.action) {
    case 'favorite':
      await toggleFavorite(pane);
      break;
    case 'rotate':
      await clickRotate(pane);
      break;
    case 'tag':
      clickTag(pane);
      break;
    case 'comment':
      openCommentEditor(pane);
      break;
    case 'rating-0':
    case 'rating-1':
    case 'rating-2':
    case 'rating-3':
    case 'rating-4':
    case 'rating-5':
      await setCurrentFileRating(Number(payload.action.split('-')[1]), pane);
      break;
    case 'zoom-in':
      clickZoomIn(pane);
      break;
    case 'zoom-out':
      clickZoomOut(pane);
      break;
    case 'zoom-actual':
      clickZoomActual(pane);
      break;
    case 'toggle-split':
      toggleSplit();
      break;
    case 'toggle-sync-viewport':
      toggleSyncViewport();
      break;
    default:
      break;
  }
};

</script>

<style scoped>
* {
  user-select: none;
}
</style>

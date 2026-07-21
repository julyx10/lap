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
      <template v-if="splitCount === 1 && fileIndex >= 0">
        <MediaViewer
          ref="mediaViewerRef"
          :mode="2"
          :isFullScreen="isFullScreen"
          :file="fileInfo"
          :showThumbnailPlaceholder="true"
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
          :isSyncViewport="isSyncViewport"
          :showWindowControls="true"
          @prev="clickPrev()"
          @next="clickNext()"
          @toggle-slide-show="clickSlideShow()"
          @update:slideShowIntervalIndex="slideShowIntervalIndex = $event"
          @item-action="handleItemAction"
          @scale="clickScale"
          @update:isZoomFit="(val) => handleZoomFitUpdate(val, 'left')"
          @view-background-change="setViewerBackground"
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

      <template v-else-if="splitCount > 1 && fileIndex >= 0">
        <div class="w-full h-full flex flex-col">
          <!-- Shared toolbar above both panes -->
          <MediaViewer
            :mode="2"
            :toolbarOnly="true"
            :showToolbar="true"
            :showWindowControls="true"
            :isFullScreen="isFullScreen"
            :file="getFileInfoByPane(activePane)"
            :nextFilePath="getNextFilePathByPane(activePane)"
            :hasPrevious="getFileIndexByPane(activePane) > 0"
            :hasNext="getFileIndexByPane(activePane) < fileCount - 1"
            :fileIndex="getFileIndexByPane(activePane)"
            :fileCount="fileCount"
            :isSlideShow="false"
            :canSlideShow="false"
            :canInteract="true"
            :imageScale="getPaneScale(activePane).scale"
            :imageMinScale="getPaneScale(activePane).min"
            :imageMaxScale="getPaneScale(activePane).max"
            :isZoomFit="getZoomFitByPane(activePane)"
            :isSyncViewport="isSyncViewport"
            :showSyncViewportControl="isCompareModeSession"
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
          <div
            class="flex-1 grid min-h-0"
            :class="splitCount === 4 ? 'grid-cols-2 grid-rows-2' : 'grid-cols-2 grid-rows-1'"
          >
            <div
              v-for="pane in visiblePanes"
              :key="pane"
              class="relative min-w-0 min-h-0"
              :class="[paneBorderClass(pane), !isPaneAvailable(pane) ? 'pointer-events-none opacity-40' : '']"
              @mousedown="setActivePane(pane)"
            >
              <MediaViewer
                :ref="(el) => setPaneViewerRef(pane, el)"
                :mode="2"
                :isFullScreen="isFullScreen"
                :file="getFileInfoByPane(pane)"
                :showThumbnailPlaceholder="true"
                :nextFilePath="getNextFilePathByPane(pane)"
                :hasPrevious="getFileIndexByPane(pane) > 0"
                :hasNext="getFileIndexByPane(pane) < fileCount - 1"
                :fileIndex="getFileIndexByPane(pane)"
                :fileCount="fileCount"
                :isSlideShow="false"
                :canSlideShow="false"
                :canInteract="isPaneAvailable(pane) && activePane === pane"
                :isPlaybackActive="isPaneAvailable(pane) && activePane === pane"
                :showToolbar="false"
                :imageScale="getPaneScale(pane).scale"
                :imageMinScale="getPaneScale(pane).min"
                :imageMaxScale="getPaneScale(pane).max"
                :isZoomFit="getZoomFitByPane(pane)"
                @prev="clickPrev(pane)"
                @next="clickNext(pane)"
                @toggle-slide-show="clickSlideShow(pane)"
                @item-action="handleItemAction"
                @scale="clickScale($event, pane)"
                @update:isZoomFit="(val) => handleZoomFitUpdate(val, pane)"
                @view-background-change="setViewerBackground"
                @media-dblclick="toggleZoomFit(pane)"
                @viewport-change="handleViewportChange($event, pane)"
                @toggle-full-screen="toggleNativeFullScreen"
                @close="closeWindow"
                @slideshow-next="handleSlideshowNext"
              />
              <div
                v-if="isPaneAvailable(pane) && activePane === pane"
                class="absolute inset-0 z-90 border border-primary/70 pointer-events-none"
              />
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
      <StatusBar
        :selected-file="activeFileInfo"
        :selected-item-index="getFileIndexByPane(statusPane)"
        :total-file-count="fileCount"
        :total-file-size="activeFileInfo?.size || 0"
        :image-scale="getPaneScale(statusPane).display"
        :show-scale="true"
        :is-embedded="true"
      />
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

import { ref, watch, computed, onMounted, onUnmounted, reactive } from 'vue';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { emit, listen } from '@tauri-apps/api/event';
import { useI18n } from 'vue-i18n';
import { useUIStore } from '@/stores/uiStore';
import { config } from '@/common/config';
import { isWin, isMac, isLinux, setTheme, getSlideShowInterval, SCALE_VALUES } from '@/common/utils';
import { matchesShortcut, ShortcutActionId, ShortcutPlatform, VIEW_BACKGROUND_SHORTCUTS } from '@/common/shortcuts';
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
 } from '@/common/icons';

/// i18n
const { locale, messages } = useI18n();
const localeMsg = computed(() => messages.value[locale.value] as any);
const uiStore = useUIStore();

const appWindow = getCurrentWebviewWindow()
const shortcutPlatform: ShortcutPlatform = isMac ? 'mac' : (isLinux ? 'linux' : 'windows');

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
type Pane = 'left' | 'right' | 'bottomLeft' | 'bottomRight';
const allPanes: Pane[] = ['left', 'right', 'bottomLeft', 'bottomRight'];
const paneViewerRefs = new Map<Pane, any>();
const isFullScreen = ref(false);
const isZoomFit = ref(true);
const rightIsZoomFit = ref(true);
const splitCount = ref<1 | 2 | 4>(1);
const activePane = ref<Pane>('left');
const isSyncViewport = ref(false);
const isCompareModeSession = ref(false);
const compareFileCount = ref(0);
const syncingPane = ref<Pane | ''>('');
const animateSyncOnce = ref(false);
const splitToolbarVisible = ref(false);

const isSlideShow = ref(false);     // Slide show state
const slideShowIntervalIndex = ref(Number(config.settings.slideShowInterval ?? 0));
let timer: NodeJS.Timeout | null = null;  // Timer for slide show

const imageScale = ref(1);          // Image scale
const imageDisplayScale = ref(1);   // User-facing image scale
const imageMinScale = ref(0);       // Minimum image scale
const imageMaxScale = ref(10);      // Maximum image scale
const rightImageScale = ref(1);     // Right image scale
const rightImageDisplayScale = ref(1); // User-facing right image scale
const rightImageMinScale = ref(0);  // Right minimum scale
const rightImageMaxScale = ref(10); // Right maximum scale

const rightFileId = ref(0);         // Right file ID
const rightFileIndex = ref(-1);     // Right file index
const rightFileInfo = ref<any>(null);
const rightNextFilePath = ref('');
const extraPaneState = reactive<Record<'bottomLeft' | 'bottomRight', {
  fileId: number;
  fileIndex: number;
  fileInfo: any;
  nextFilePath: string;
  isZoomFit: boolean;
  scale: number;
  displayScale: number;
  minScale: number;
  maxScale: number;
}>>({
  bottomLeft: {
    fileId: 0, fileIndex: -1, fileInfo: null, nextFilePath: '',
    isZoomFit: true, scale: 1, displayScale: 1, minScale: 0, maxScale: 10,
  },
  bottomRight: {
    fileId: 0, fileIndex: -1, fileInfo: null, nextFilePath: '',
    isZoomFit: true, scale: 1, displayScale: 1, minScale: 0, maxScale: 10,
  },
});
const visiblePanes = computed<Pane[]>(() =>
  splitCount.value === 4 ? allPanes : ['left', 'right']
);
const showTaggingDialog = ref(false);
const showCommentMsgbox = ref(false);
const taggingFileIds = ref<number[]>([]);

let unlistenImg: () => void;
let unlistenGridView: () => void;
let unlistenFilesDeleted: (() => void) | null = null;

const activeFileInfo = computed(() => {
  return getFileInfoByPane(splitCount.value > 1 ? activePane.value : 'left');
});

const activeFileId = computed(() => {
  return getFileIdByPane(splitCount.value > 1 ? activePane.value : 'left');
});
const statusPane = computed<Pane>(() => splitCount.value > 1 ? activePane.value : 'left');
const showEmbeddedStatusBar = computed(() => config.settings.showStatusBar && !isFullScreen.value);

function normalizeScale(value: number) {
  return SCALE_VALUES.find((item) => item === Number(value)) ?? 1;
}

function normalizeSplitCount(value: unknown): 1 | 2 | 4 {
  const count = Number(value);
  return count === 2 || count === 4 ? count : 1;
}

function applyViewerScale(scale: number) {
  const normalizedScale = normalizeScale(scale);
  document.documentElement.style.fontSize = `${normalizedScale * 16}px`;
}

function handleRootMouseMove(event: MouseEvent) {
  if (!isFullScreen.value || splitCount.value === 1) {
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
  const forceSplitCount = normalizeSplitCount(urlParams.get('forceSplitCount'));
  isCompareModeSession.value = urlParams.get('compareMode') === '1';
  compareFileCount.value = isCompareModeSession.value ? Number(urlParams.get('compareFileCount') || '0') : 0;

  splitCount.value = 1;
  if (isCompareModeSession.value) {
    splitCount.value = forceSplitCount > 1 ? forceSplitCount : 2;
    isSyncViewport.value = !!config.imageViewer?.isSyncViewport;
  } else {
    isSyncViewport.value = false;
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

    const requestedPane = String(event.payload?.pane || 'left');
    const pane: Pane = allPanes.includes(requestedPane as Pane) ? requestedPane as Pane : 'left';
    if (typeof event.payload?.compareMode === 'boolean') {
      isCompareModeSession.value = !!event.payload.compareMode;
    }
    if (typeof event.payload?.compareFileCount === 'number') {
      compareFileCount.value = Number(event.payload.compareFileCount);
    } else if (!isCompareModeSession.value) {
      compareFileCount.value = 0;
    }
    const requestedSplitCount = normalizeSplitCount(event.payload?.forceSplitCount);
    if (requestedSplitCount > 1) {
      splitCount.value = requestedSplitCount;
    }
    if (event.payload?.resetSplit) {
      if (isCompareModeSession.value) clearSecondaryPanes();
      if (isCompareModeSession.value) {
        splitCount.value = requestedSplitCount > 1 ? requestedSplitCount : splitCount.value > 1 ? splitCount.value : 2;
        isSyncViewport.value = !!config.imageViewer?.isSyncViewport;
      } else {
        splitCount.value = 1;
        isSyncViewport.value = false;
      }
      if (splitCount.value === 1) {
        clearSecondaryPanes();
      }
    }

    fileCount.value = Number(event.payload.fileCount);
    if (pane === 'right') {
      rightFileId.value = Number(event.payload.fileId);
      rightFileIndex.value = Number(event.payload.fileIndex);
      rightNextFilePath.value = event.payload.nextFilePath || '';
      if (rightFileId.value <= 0) rightFileInfo.value = null;
    } else if (pane === 'bottomLeft' || pane === 'bottomRight') {
      extraPaneState[pane].fileId = Number(event.payload.fileId);
      extraPaneState[pane].fileIndex = Number(event.payload.fileIndex);
      extraPaneState[pane].nextFilePath = event.payload.nextFilePath || '';
      if (extraPaneState[pane].fileId <= 0) extraPaneState[pane].fileInfo = null;
    } else {
      fileId.value = Number(event.payload.fileId);
      fileIndex.value = Number(event.payload.fileIndex);
      nextFilePath.value = event.payload.nextFilePath || '';
    }
  });
  ensureFourPanesLoaded();


  unlistenGridView = await listen('message-from-content', (event) => {
    const { message, fileId: targetFileId, changes } = event.payload as any;
    console.log('message-from-content:', message, targetFileId);
    switch (message) {
      case 'rotate':
        for (const pane of allPanes) {
          if (targetFileId !== getFileIdByPane(pane)) continue;
          getViewerRef(pane)?.rotateRight();
          const target = getFileInfoByPane(pane);
          if (target) target.rotate = (target.rotate || 0) + 90;
        }
        if (targetFileId === fileId.value) {
          iconRotate.value += 90;
        }
        break;
      case 'update-file-meta':
        for (const pane of allPanes) {
          const target = getFileInfoByPane(pane);
          if (targetFileId === getFileIdByPane(pane) && target) {
            Object.assign(target, changes || {});
          }
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
    const comparisonNextCount = Number(event?.payload?.compareFileCount);
    // An external deletion arrives before Content has updated and rebroadcast
    // the comparison snapshot. Ignore that first event instead of applying the
    // full file list count to this isolated session.
    if (isCompareModeSession.value && Number.isNaN(comparisonNextCount)) return;
    if (isCompareModeSession.value) {
      compareFileCount.value = comparisonNextCount;
      fileCount.value = comparisonNextCount;
      fileId.value = 0;
      fileIndex.value = -1;
      fileInfo.value = null;
      nextFilePath.value = '';
      clearSecondaryPanes();
      activePane.value = 'left';

      // Compare always restarts with the first visible items in the updated
      // selection, rather than preserving stale per-pane positions.
      if (comparisonNextCount > 0) {
        requestFileAtIndex(0, 'left');
        if (comparisonNextCount > 1) requestFileAtIndex(1, 'right');
        if (splitCount.value === 4 && comparisonNextCount > 2) requestFileAtIndex(2, 'bottomLeft');
        if (splitCount.value === 4 && comparisonNextCount > 3) requestFileAtIndex(3, 'bottomRight');
      }
      return;
    }
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
      for (const pane of ['bottomLeft', 'bottomRight'] as const) {
        extraPaneState[pane].fileId = 0;
        extraPaneState[pane].fileIndex = -1;
        extraPaneState[pane].fileInfo = null;
        extraPaneState[pane].nextFilePath = '';
      }
      return;
    }

    const leftDeleted = deletedIds.includes(fileId.value);
    const rightDeleted = deletedIds.includes(rightFileId.value);

    if (leftDeleted || fileIndex.value >= fileCount.value) {
      const targetIndex = Math.max(0, Math.min(fileIndex.value, fileCount.value - 1));
      requestFileAtIndex(targetIndex, 'left');
    }

    if (splitCount.value > 1 && (rightDeleted || rightFileIndex.value >= fileCount.value)) {
      const fallbackBase = rightFileIndex.value >= 0 ? rightFileIndex.value : (fileIndex.value + 1);
      const targetIndex = Math.max(0, Math.min(fallbackBase, fileCount.value - 1));
      requestFileAtIndex(targetIndex, 'right');
    }

    for (const pane of ['bottomLeft', 'bottomRight'] as const) {
      const state = extraPaneState[pane];
      if (splitCount.value !== 4 || (!deletedIds.includes(state.fileId) && state.fileIndex < fileCount.value)) continue;
      const targetIndex = Math.max(0, Math.min(state.fileIndex, fileCount.value - 1));
      requestFileAtIndex(targetIndex, pane);
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

  if (event.key === 'Tab' && splitCount.value > 1 && !event.ctrlKey && !event.metaKey && !event.altKey) {
    event.preventDefault();
    const panes = getAvailablePanes();
    if (panes.length === 0) return;
    const currentIndex = panes.indexOf(activePane.value);
    const direction = event.shiftKey ? -1 : 1;
    const startIndex = currentIndex >= 0 ? currentIndex : 0;
    setActivePane(panes[(startIndex + direction + panes.length) % panes.length]);
    return;
  }

  // Disable keyboard events during slideshow except close and toggle slideshow.
  if (
    isSlideShow.value &&
    !matchesShortcut('view.close', event, shortcutPlatform) &&
    !matchesShortcut('view.cycleBackground', event, shortcutPlatform) &&
    getMatchedViewBackground(event) === null &&
    !matchesShortcut('slideshow.toggle', event, shortcutPlatform)
  ) {
    return;
  }

  const ratingShortcut = getMatchedRating(event);
  if (ratingShortcut !== null) {
    event.preventDefault();
    void setCurrentFileRating(ratingShortcut, getActiveFilePane());
    return;
  }

  if (matchesShortcut('slideshow.toggle', event, shortcutPlatform)) {
    event.preventDefault();
    clickSlideShow(getActiveFilePane());
    return;
  }

  if (matchesShortcut('view.cycleBackground', event, shortcutPlatform)) {
    event.preventDefault();
    config.cycleViewBackground();
    void emit('settings-viewBackground-changed', config.settings.viewBackground);
    return;
  }

  const viewBackground = getMatchedViewBackground(event);
  if (viewBackground !== null) {
    event.preventDefault();
    setViewerBackground(viewBackground);
    return;
  }

  if (matchesShortcut('meta.favorite', event, shortcutPlatform)) {
    event.preventDefault();
    void toggleFavorite(getActiveFilePane());
    return;
  }

  if (matchesShortcut('meta.tag', event, shortcutPlatform)) {
    event.preventDefault();
    clickTag(getActiveFilePane());
    return;
  }

  if (matchesShortcut('meta.comment', event, shortcutPlatform)) {
    event.preventDefault();
    openCommentEditor(getActiveFilePane());
    return;
  }

  if (matchesShortcut('meta.rotate', event, shortcutPlatform)) {
    event.preventDefault();
    void clickRotate(getActiveFilePane());
    return;
  }

  if (matchesShortcut('view.togglePane', event, shortcutPlatform) && splitCount.value > 1) {
    event.preventDefault();
    const panes = getAvailablePanes();
    if (panes.length === 0) return;
    const currentIndex = panes.indexOf(activePane.value);
    setActivePane(panes[((currentIndex >= 0 ? currentIndex : -1) + 1) % panes.length]);
    return;
  }

  const matchedAction = getMatchedViewAction(event);
  if (matchedAction) {
    event.preventDefault();
    viewActions[matchedAction]?.();
  }
}

const ratingActions: Array<{ actionId: ShortcutActionId; rating: number }> = [
  { actionId: 'meta.rating.clear', rating: 0 },
  { actionId: 'meta.rating.one', rating: 1 },
  { actionId: 'meta.rating.two', rating: 2 },
  { actionId: 'meta.rating.three', rating: 3 },
  { actionId: 'meta.rating.four', rating: 4 },
  { actionId: 'meta.rating.five', rating: 5 },
];

function getMatchedRating(event: KeyboardEvent) {
  const match = ratingActions.find(({ actionId }) => matchesShortcut(actionId, event, shortcutPlatform));
  return match ? match.rating : null;
}

function getMatchedViewBackground(event: KeyboardEvent): number | null {
  const match = VIEW_BACKGROUND_SHORTCUTS.find(({ actionId }) => matchesShortcut(actionId, event, shortcutPlatform));
  return match?.value ?? null;
}

const viewActions: Partial<Record<ShortcutActionId, () => void>> = {
  'view.previous': () => clickPrev(getActivePane()),
  'view.next': () => clickNext(getActivePane()),
  'view.first': () => clickHome(getActivePane()),
  'view.last': () => clickEnd(getActivePane()),
  'view.zoomIn': () => clickZoomIn(getActivePane()),
  'view.zoomOut': () => clickZoomOut(getActivePane()),
  'view.zoomInDirectional': () => clickZoomIn(getActivePane()),
  'view.zoomOutDirectional': () => clickZoomOut(getActivePane()),
  'view.zoomFit': () => toggleZoomFit(getActivePane()),
  'view.close': () => closeWindow(),
};

const viewActionOrder: ShortcutActionId[] = [
  'view.previous',
  'view.next',
  'view.first',
  'view.last',
  'view.zoomIn',
  'view.zoomOut',
  'view.zoomInDirectional',
  'view.zoomOutDirectional',
  'view.zoomFit',
  'view.close',
];

function getMatchedViewAction(event: KeyboardEvent) {
  if (isMac && event.metaKey && !event.ctrlKey && !event.altKey && !event.shiftKey) {
    if (event.key === 'ArrowUp') return 'view.first';
    if (event.key === 'ArrowDown') return 'view.last';
  }
  return viewActionOrder.find((actionId) => matchesShortcut(actionId, event, shortcutPlatform));
}

function getActivePane(): Pane {
  return splitCount.value > 1 ? activePane.value : 'left';
}

function setActivePane(pane: Pane) {
  if (!isPaneAvailable(pane)) return;
  activePane.value = pane;
}

function isPaneAvailable(pane: Pane) {
  return getFileIdByPane(pane) > 0;
}

function getAvailablePanes() {
  return visiblePanes.value.filter(isPaneAvailable);
}

function setPaneViewerRef(pane: Pane, viewer: any) {
  if (viewer) paneViewerRefs.set(pane, viewer);
  else paneViewerRefs.delete(pane);
  if (pane === 'left') mediaViewerRef.value = viewer;
  if (pane === 'right') rightMediaViewerRef.value = viewer;
}

function getViewerRef(pane: Pane) {
  return paneViewerRefs.get(pane)
    || (pane === 'right' ? rightMediaViewerRef.value : pane === 'left' ? mediaViewerRef.value : null);
}

function paneBorderClass(pane: Pane) {
  if (splitCount.value !== 4) return pane === 'left' ? 'border-r border-base-content/10' : '';
  return {
    left: 'border-r border-b border-base-content/10',
    right: 'border-b border-base-content/10',
    bottomLeft: 'border-r border-base-content/10',
    bottomRight: '',
  }[pane];
}

function syncViewportFrom(pane: Pane, animate = false) {
  if (splitCount.value === 1 || !isSyncViewport.value) return;

  const sourceRef = getViewerRef(pane);
  const viewport = sourceRef?.getViewportState?.();
  if (!viewport) return;

  syncingPane.value = pane;
  for (const targetPane of visiblePanes.value) {
    if (targetPane !== pane) {
      getViewerRef(targetPane)?.applyViewportState?.(viewport, !animate);
    }
  }
  syncingPane.value = '';
}

function handleViewportChange(viewport: any, pane: Pane) {
  if (splitCount.value === 1 || !isSyncViewport.value) return;
  if (syncingPane.value) return;

  const shouldAnimate = animateSyncOnce.value;
  animateSyncOnce.value = false;
  syncingPane.value = pane;
  // Drag/wheel sync stays no-animation; zoom-fit sync can opt-in animation.
  for (const targetPane of visiblePanes.value) {
    if (targetPane !== pane) {
      getViewerRef(targetPane)?.applyViewportState?.(viewport, !shouldAnimate);
    }
  }
  syncingPane.value = '';
}

function getZoomFitByPane(pane: Pane) {
  if (pane === 'right') return rightIsZoomFit.value;
  if (pane === 'bottomLeft' || pane === 'bottomRight') return extraPaneState[pane].isZoomFit;
  return isZoomFit.value;
}

function setZoomFitByPane(pane: Pane, val: boolean) {
  if (pane === 'right') {
    rightIsZoomFit.value = val;
    return;
  }
  if (pane === 'bottomLeft' || pane === 'bottomRight') {
    extraPaneState[pane].isZoomFit = val;
    return;
  }
  isZoomFit.value = val;
}

function handleZoomFitUpdate(val: boolean, pane: Pane) {
  setActivePane(pane);
  setZoomFitByPane(pane, val);
  if (splitCount.value > 1 && isSyncViewport.value) {
    animateSyncOnce.value = true;
  }
}

function setViewerBackground(value: number) {
  config.setViewBackground(value);
  void emit('settings-viewBackground-changed', config.settings.viewBackground);
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
    (config as any).imageViewer = { isSyncViewport: false, isFullScreen: false };
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

watch(() => isSyncViewport.value, (isSync) => {
  if (!isCompareModeSession.value) return;
  if (!config.imageViewer) {
    (config as any).imageViewer = { isSyncViewport: false, isFullScreen: false };
  }
  config.imageViewer.isSyncViewport = isSync;
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

for (const pane of ['bottomLeft', 'bottomRight'] as const) {
  watch(() => extraPaneState[pane].fileId, async (newFileId) => {
    extraPaneState[pane].fileInfo = newFileId > 0 ? await getFileInfo(newFileId) : null;
  });
}

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
  if (splitCount.value === 1) return;
  if (isCompareModeSession.value && compareFileCount.value < 2) {
    clearSecondaryPanes();
    return;
  }
  if (rightFileIndex.value >= 0 && rightFileId.value > 0) return;
  if (fileCount.value <= 0 || fileIndex.value < 0) return;

  const nextIndex = Math.min(fileIndex.value + 1, fileCount.value - 1);
  requestFileAtIndex(nextIndex, 'right');
}

function ensureFourPanesLoaded() {
  if (splitCount.value !== 4 || fileCount.value <= 0 || fileIndex.value < 0) return;
  ensureRightPaneLoaded();
  for (const [pane, offset] of [['bottomLeft', 2], ['bottomRight', 3]] as const) {
    if (isCompareModeSession.value && offset >= compareFileCount.value) continue;
    if (extraPaneState[pane].fileIndex >= 0 && extraPaneState[pane].fileId > 0) continue;
    requestFileAtIndex(Math.min(fileIndex.value + offset, fileCount.value - 1), pane);
  }
}

function clearSecondaryPanes() {
  rightFileId.value = 0;
  rightFileIndex.value = -1;
  rightFileInfo.value = null;
  rightNextFilePath.value = '';
  rightIsZoomFit.value = true;
  for (const pane of ['bottomLeft', 'bottomRight'] as const) {
    Object.assign(extraPaneState[pane], {
      fileId: 0, fileIndex: -1, fileInfo: null, nextFilePath: '',
      isZoomFit: true, scale: 1, displayScale: 1, minScale: 0, maxScale: 10,
    });
  }
}

watch(() => splitCount.value, (count) => {
  if (count > 1) {
    ensureRightPaneLoaded();
    ensureFourPanesLoaded();
  }
});

watch(() => [fileIndex.value, fileCount.value], () => {
  ensureRightPaneLoaded();
  ensureFourPanesLoaded();
});

function requestFileAtIndex(index: number, pane: Pane = 'left') {
  emit('message-from-image-viewer', { message: 'request-file-at-index', index, pane });
}

function getFileInfoByPane(pane: Pane = 'left') {
  if (pane === 'right') return rightFileInfo.value;
  if (pane === 'bottomLeft' || pane === 'bottomRight') return extraPaneState[pane].fileInfo;
  return fileInfo.value;
}

function getFileIdByPane(pane: Pane = 'left') {
  if (pane === 'right') return rightFileId.value;
  if (pane === 'bottomLeft' || pane === 'bottomRight') return extraPaneState[pane].fileId;
  return fileId.value;
}

function getFileIndexByPane(pane: Pane = 'left') {
  if (pane === 'right') return rightFileIndex.value;
  if (pane === 'bottomLeft' || pane === 'bottomRight') return extraPaneState[pane].fileIndex;
  return fileIndex.value;
}

function getNextFilePathByPane(pane: Pane = 'left') {
  if (pane === 'right') return rightNextFilePath.value;
  if (pane === 'bottomLeft' || pane === 'bottomRight') return extraPaneState[pane].nextFilePath;
  return nextFilePath.value;
}

function getPaneScale(pane: Pane) {
  if (pane === 'right') {
    return {
      scale: rightImageScale.value, display: rightImageDisplayScale.value,
      min: rightImageMinScale.value, max: rightImageMaxScale.value,
    };
  }
  if (pane === 'bottomLeft' || pane === 'bottomRight') {
    const state = extraPaneState[pane];
    return {
      scale: state.scale, display: state.displayScale,
      min: state.minScale, max: state.maxScale,
    };
  }
  return {
    scale: imageScale.value, display: imageDisplayScale.value,
    min: imageMinScale.value, max: imageMaxScale.value,
  };
}

function getActiveFilePane() {
  return splitCount.value > 1 ? activePane.value : 'left';
}

function syncFileMetaToContent(targetFileId: number, changes: Record<string, any>) {
  emit('message-from-image-viewer', {
    message: 'update-file-meta',
    fileId: targetFileId,
    changes,
  });
}

function applyFileMetaToPanes(targetFileId: number, changes: Record<string, any>) {
  for (const pane of allPanes) {
    const target = getFileInfoByPane(pane);
    if (targetFileId === getFileIdByPane(pane) && target) {
      Object.assign(target, changes);
    }
  }
}

function clickPrev(pane: Pane = 'left') {
  setActivePane(pane);
  const currentIndex = getFileIndexByPane(pane);
  const viewerRef = getViewerRef(pane);
  if (currentIndex > 0) {
    requestFileAtIndex(currentIndex - 1, pane);
  } else {
    viewerRef?.showMessage((localeMsg.value as any).tooltip.image_viewer.first_image);
  }
}

function clickNext(pane: Pane = 'left') {
  setActivePane(pane);
  const currentIndex = getFileIndexByPane(pane);
  const viewerRef = getViewerRef(pane);

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

function clickHome(pane: Pane = 'left') {
  setActivePane(pane);
  requestFileAtIndex(0, pane);
}

function clickEnd(pane: Pane = 'left') {
  setActivePane(pane);
  requestFileAtIndex(fileCount.value - 1, pane);
}

function clickSlideShow(pane: Pane = 'left') {
  setActivePane(pane);
  isSlideShow.value = !isSlideShow.value;
  if (isSlideShow.value) {
    startSlideShow();
  } else {
    stopSlideShow();
  }
}

const clickZoomIn = (pane: Pane = 'left') => {
  setActivePane(pane);
  getViewerRef(pane)?.zoomIn();
};

const clickZoomOut = (pane: Pane = 'left') => {
  setActivePane(pane);
  getViewerRef(pane)?.zoomOut();
};

const clickZoomActual = (pane: Pane = 'left') => {
  setActivePane(pane);
  getViewerRef(pane)?.zoomActual();
};

const toggleZoomFit = (pane: Pane = 'left') => {
  const current = getZoomFitByPane(pane);
  handleZoomFitUpdate(!current, pane);
};

const toggleNativeFullScreen = () => {
  isFullScreen.value = !isFullScreen.value;
};

const closeWindow = () => {
  appWindow.close();
}

const clickScale = (event: any, pane: Pane = 'left') => {
  if (pane === 'right') {
    rightImageScale.value = event.scale;
    rightImageDisplayScale.value = event.displayScale ?? event.scale;
    rightImageMinScale.value = event.minScale;
    rightImageMaxScale.value = event.maxScale;
    return;
  }
  if (pane === 'bottomLeft' || pane === 'bottomRight') {
    const state = extraPaneState[pane];
    state.scale = event.scale;
    state.displayScale = event.displayScale ?? event.scale;
    state.minScale = event.minScale;
    state.maxScale = event.maxScale;
    return;
  }

  imageScale.value = event.scale;
  imageDisplayScale.value = event.displayScale ?? event.scale;
  imageMinScale.value = event.minScale;
  imageMaxScale.value = event.maxScale;
};

const toggleSyncViewport = () => {
  if (splitCount.value === 1) return;
  isSyncViewport.value = !isSyncViewport.value;
  if (isSyncViewport.value) {
    syncViewportFrom(activePane.value);
  }
};

const toggleFavorite = async (pane: Pane = 'left') => {
  const target = getFileInfoByPane(pane);
  const currentFileId = getFileIdByPane(pane);
  if (!target || currentFileId <= 0) return;

  const isFavorite = !target.is_favorite;
  applyFileMetaToPanes(currentFileId, { is_favorite: isFavorite });
  await setFileFavorite(currentFileId, isFavorite);
  syncFileMetaToContent(currentFileId, { is_favorite: isFavorite });
};

const setCurrentFileRating = async (rating: number, pane: Pane = 'left') => {
  const target = getFileInfoByPane(pane);
  const currentFileId = getFileIdByPane(pane);
  if (!target || currentFileId <= 0) return;

  const normalized = Number(target.rating || 0) === rating ? 0 : rating;
  applyFileMetaToPanes(currentFileId, { rating: normalized });
  await setFileRating(currentFileId, normalized);
  syncFileMetaToContent(currentFileId, { rating: normalized });
};

const clickRotate = async (pane: Pane = 'left') => {
  const target = getFileInfoByPane(pane);
  const currentFileId = getFileIdByPane(pane);
  if (!target || currentFileId <= 0) return;

  const rotate = (Number(target.rotate) || 0) + 90;
  applyFileMetaToPanes(currentFileId, { rotate });
  for (const targetPane of allPanes) {
    if (getFileIdByPane(targetPane) === currentFileId) {
      getViewerRef(targetPane)?.rotateRight?.();
    }
  }
  await setFileRotate(currentFileId, rotate);
  syncFileMetaToContent(currentFileId, { rotate });
};

const clickTag = (pane: Pane = 'left') => {
  const currentFileId = getFileIdByPane(pane);
  if (currentFileId <= 0) return;

  setActivePane(pane);
  taggingFileIds.value = [currentFileId];
  showTaggingDialog.value = true;
};

const openCommentEditor = (pane: Pane = 'left') => {
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
    applyFileMetaToPanes(currentFileId, { comments: newComment });
    showCommentMsgbox.value = false;
    syncFileMetaToContent(currentFileId, { comments: newComment });
  }
};

async function updateFileHasTags(fileStates: Array<{ file_id: number; has_tags: boolean }>) {
  if (!Array.isArray(fileStates) || fileStates.length === 0) {
    showTaggingDialog.value = false;
    return;
  }

  for (const state of fileStates) {
    const taggedFileId = Number(state.file_id);
    if (taggedFileId <= 0) continue;
    const hasTags = Boolean(state.has_tags);
    const tags = hasTags ? ((await getTagsForFile(taggedFileId)) || []) : [];
    const changes = { has_tags: hasTags, tags };
    applyFileMetaToPanes(taggedFileId, changes);
    syncFileMetaToContent(taggedFileId, changes);
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

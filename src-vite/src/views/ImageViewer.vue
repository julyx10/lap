<template>

  <div
    :class="[
      'relative w-screen h-screen flex flex-col overflow-hidden bg-base-300 text-base-content/70',
      isFullScreen ? 'fixed top-0 left-0 z-50' : '',
    ]"
    @mousemove="handleRootMouseMove"
    @mouseleave="handleRootMouseLeave"
    @contextmenu.prevent="openViewerContextMenu"
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
          :showViewerMenu="true"
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
          @context-menu="openPaneContextMenu('left', $event)"
          @viewer-more="openViewerMoreMenu"
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
            :displayZoomFit="paneDisplayFit[activePane] ?? getZoomFitByPane(activePane)"
            :imageMinScale="getPaneScale(activePane).min"
            :imageMaxScale="getPaneScale(activePane).max"
            :isZoomFit="getZoomFitByPane(activePane)"
            :isSyncViewport="isSyncViewport"
            :showSyncViewportControl="isCompareModeSession"
            :forceToolbarVisible="isFullScreen && splitToolbarVisible"
            :showViewerMenu="true"
            @toggle-af-areas="getViewerRef(activePane)?.toggleAfAreas?.()"
            @prev="clickPrev(activePane)"
            @next="clickNext(activePane)"
            @toggle-slide-show="clickSlideShow(activePane)"
            @item-action="handleItemAction"
            @scale="clickScale($event, activePane)"
            @update:isZoomFit="(val) => handleZoomFitUpdate(val, activePane)"
            @zoom-percent="getViewerRef(activePane)?.zoomToPercent?.($event)"
            @zoom-fit="getViewerRef(activePane)?.zoomToFit?.()"
            @toggle-full-screen="toggleNativeFullScreen"
            @close="closeWindow"
            @slideshow-next="handleSlideshowNext"
            @viewer-more="openViewerMoreMenu"
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
                :forceToolbarVisible="isFullScreen && splitToolbarVisible"
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
                @activate="setActivePane(pane)"
                @toggle-full-screen="toggleNativeFullScreen"
                @close="closeWindow"
                @slideshow-next="handleSlideshowNext"
                @context-menu="openPaneContextMenu(pane, $event)"
              />
              <div
                v-if="isPaneAvailable(pane) && activePane === pane"
                class="absolute inset-0 z-90 border border-primary/70 pointer-events-none"
              />
            </div>
          </div>
        </div>
      </template>

      <!-- no image selected: keep the full MediaViewer/image container mounted -->
      <div v-else class="relative w-full h-full">
        <MediaViewer
          :mode="2"
          :showToolbar="true"
          :showWindowControls="true"
          :isFullScreen="isFullScreen"
          :file="null"
          :fileIndex="-1"
          :fileCount="0"
          :canInteract="false"
          :isZoomFit="true"
          :showViewerMenu="false"
          @toggle-full-screen="toggleNativeFullScreen"
          @close="closeWindow"
        />
        <div class="pointer-events-none absolute inset-0 flex flex-col items-center justify-center gap-1 pt-10 text-base-content/30">
          <IconSearch class="w-8 h-8" />
          <span>{{ $t('tooltip.not_found.files') }}</span>
        </div>
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
        :total-file-size="getMediaItemSize(activeFileInfo)"
        :is-embedded="true"
      />
    </div>

    <div class="hidden">
      <ContextMenu
        ref="viewerContextMenuRef"
        :menuItems="viewerContextMenuItems"
      >
        <template #trigger><span></span></template>
      </ContextMenu>
    </div>

    <PrintImage ref="printImageView" />

    <MoveTo
      ref="moveToRef"
      v-if="showMoveTo"
      :title="$t(moveMode === 'copy' ? 'msgbox.copy_to.title' : 'msgbox.move_to.title', { source: moveTarget?.name || '' })"
      :message="$t(moveMode === 'copy' ? 'msgbox.copy_to.content' : 'msgbox.move_to.content')"
      :OkText="$t(moveMode === 'copy' ? 'msgbox.copy_to.ok' : 'msgbox.move_to.ok')"
      :cancelText="$t('msgbox.cancel')"
      @ok="onMoveTo"
      @cancel="showMoveTo = false"
    />

    <TaggingDialog
      v-if="showTaggingDialog"
      position-key="edit-tag"
      :fileIds="taggingFileIds"
      :file-name="activeFileInfo?.name || ''"
      @ok="updateFileHasTags"
      @states-changed="syncTagStates"
      @cancel="showTaggingDialog = false"
    />

    <AddToCollectionDialog
      v-if="showAddToCollectionDialog"
      position-key="edit-collection"
      :fileIds="collectionFileIds"
      @applied="handleCollectionsAdded"
      @deleted="handleCollectionDeleted"
      @cancel="showAddToCollectionDialog = false"
    />

    <EditMetadataDialog v-if="showCommentMsgbox && metadataFiles.length" :files="metadataFiles" :focus-field="metadataFocus"
    @saved="onMetadataSaved" @cancel="showCommentMsgbox = false" />

    <FileConflictDialog
      v-if="fileConflictDialog.show"
      :name="fileConflictDialog.name"
      :destination="fileConflictDialog.destination"
      :showApplyAll="fileConflictDialog.showApplyAll"
      :allowReplace="fileConflictDialog.allowReplace"
      @resolve="resolveFileConflict"
    />

    <FileDeleteDialog
      v-if="showDeleteDialog"
      :plan="deletePlan"
      :permanently="deletePermanently"
      :confirmation-count="deleteConfirmationCount"
      :subtitle="deleteSubtitle"
      :trash-message="''"
      :preview-files="deletePlan?.files || []"
      :preview-item-count="deletePlan?.files.length || 0"
      :preview-limit="5"
      :thumbnails="deleteThumbnails"
      :raw-pair-delete-count="rawPairDeleteCount"
      :single-delete-count="singleDeleteCount"
      :raw-pair-delete-choices="rawPairDeleteChoices"
      :raw-pair-scope="rawPairScope"
      :is-deleting="isDeleting"
      @update:rawPairScope="rawPairScope = $event"
      @ok="confirmDelete"
      @cancel="showDeleteDialog = false"
    />

    <RenameFileDialog
      v-if="showRenameDialog"
      :name="renameName"
      :extension="renameExtension"
      :error-message="renameError"
      @ok="submitRename"
      @cancel="showRenameDialog = false"
      @reset="renameError = ''"
    />

  </div>

</template>

<script setup lang="ts">

import { getVisibleEditableMetadataFields } from '@/common/fileInfoFields';
import EditMetadataDialog from "@/components/EditMetadataDialog.vue";
import { isOriginalStorageUnavailable, isOriginalUnavailable, setAlbumAccessibility, resetFileAccessibility } from '@/common/availability';
import { ref, shallowRef, watch, computed, nextTick, onMounted, onUnmounted, reactive } from 'vue';
import { getCurrentWebviewWindow, WebviewWindow } from '@tauri-apps/api/webviewWindow';
import { ask, open as openDialog } from '@tauri-apps/plugin-dialog';
import { emit, listen } from '@tauri-apps/api/event';
import { useI18n } from 'vue-i18n';
import { useToast } from '@/common/toast';
import { useUIStore } from '@/stores/uiStore';
import { config, libConfig } from '@/common/config';
import { isWin, isMac, isLinux, extractFileName, getAssetSrc, getFolderName, getFullPath, getMediaItemSize, getPreviewUrl, getSlideShowInterval, getThumbnailDataUrl, isPreviewOnlyDocument, isWithinRootPath, normalizePathForCompare, shouldUseBackendPreview, SCALE_VALUES } from '@/common/utils';
import { matchesShortcut, ShortcutActionId, ShortcutPlatform, VIEW_BACKGROUND_SHORTCUTS } from '@/common/shortcuts';
import { createMetadataSyncTracker, getMatchingPaneFileInfo } from '@/common/metadataSync';
import { handleVideoPlaybackShortcut, handleVideoEscapeShortcut } from '@/common/video';
import {
  batchDeleteFiles,
  getFileCollections,
  addFileToDb,
  checkFileExists,
  getAllAlbums,
  copyFile,
  getFileInfo,
  getFileThumbById,
  isCloudOnlyFile,
  moveFile,
  moveFileOutsideLibrary,
  openFilesWithApp,
  selectFolder,
  saveXmpMetadata,
  getTagsForFile,
  refreshFileInfo,
  setAlbumCover,
  setDesktopWallpaper,
  setFileFavorite,
  setFileRating,
  setFileCullingFlag,
  setFileRotate,
} from '@/common/api';

import {
  buildDeleteRetryTargets, buildFileDeleteTargets, copyFilePathsForFiles, copyFilesToClipboard,
  countDeleteTargets, countRawPairTargets, countSingleTargets, isOriginalUnavailableActionError,
  rawPairCompanionFormat, renameSingleFile, resolveExternalAppTarget, revealFile, shouldConfirmDelete,
  type FileDeletePlan,
} from '@/common/fileActions';
import ContextMenu from '@/components/ContextMenu.vue';
import { useFileMenuItems } from '@/common/fileMenu';
import MediaViewer from '@/components/MediaViewer.vue';
import MessageBox from '@/components/MessageBox.vue';
import TButton from '@/components/TButton.vue';
import StatusBar from '@/components/StatusBar.vue';
import TaggingDialog from '@/components/TaggingDialog.vue';
import AddToCollectionDialog from '@/components/AddToCollectionDialog.vue';
import RenameFileDialog from '@/components/RenameFileDialog.vue';
import FileDeleteDialog from '@/components/FileDeleteDialog.vue';
import MoveTo from '@/components/MoveTo.vue';
import FileConflictDialog from '@/components/FileConflictDialog.vue';
import PrintImage from '@/components/PrintImage.vue';

import { 
  IconSearch,
  IconComment,
 } from '@/common/icons';

/// i18n
const { locale, messages, t } = useI18n();
const localeMsg = computed(() => messages.value[locale.value] as any);
const uiStore = useUIStore();
const toast = useToast();

const appWindow = getCurrentWebviewWindow()
const shortcutPlatform: ShortcutPlatform = isMac ? 'mac' : (isLinux ? 'linux' : 'windows');

// input parameters
const fileId = ref(0);       // File ID
const fileIndex = ref(0);       // Index of the current file
const fileCount = ref(0);       // Total number of files

const fileInfo = ref<any>(null);
const viewerContextMenuRef = ref<InstanceType<typeof ContextMenu> | null>(null);
const contextMenuFile = ref<any>(null);
const showRenameDialog = ref(false);
const renameTarget = ref<any>(null);
const renameName = ref('');
const renameExtension = ref('');
const renameError = ref('');
const showDeleteDialog = ref(false);
const deletePlan = shallowRef<FileDeletePlan | null>(null);
const deletePermanently = ref(false);
const rawPairScope = ref<'both' | 'raw' | 'companion'>('both');
// Viewer file info has no `thumbnail` URL (AFile doesn't carry one), so the
// delete dialog's previews are built from the thumb protocol directly.
const deleteThumbnails = ref<Record<number, string>>({});
const isDeleting = ref(false);
const showMoveTo = ref(false);
const moveTarget = ref<any>(null);
const moveMode = ref<'move' | 'copy'>('move');
const printImageView = ref<InstanceType<typeof PrintImage> | null>(null);
const moveToRef = ref<InstanceType<typeof MoveTo> | null>(null);
type FileConflictPolicy = 'skip' | 'keep_both' | 'replace';
const fileConflictDialog = ref({ show: false, name: '', destination: '', showApplyAll: false, allowReplace: true });
let fileConflictResolver: ((result: { policy: FileConflictPolicy; applyAll: boolean }) => void) | null = null;
const nextFilePath = ref('');
const iconRotate = ref(0);      // icon rotation angle
const isTransitionDisabled = ref(true);

const mediaViewerRef = ref<any>(null); // media viewer reference
const rightMediaViewerRef = ref<any>(null); // right media viewer reference (split mode)
type Pane = 'left' | 'right' | 'bottomLeft' | 'bottomRight';
const allPanes: Pane[] = ['left', 'right', 'bottomLeft', 'bottomRight'];
const cullingSync = createMetadataSyncTracker();
const ratingSync = createMetadataSyncTracker('rating');
const favoriteSync = createMetadataSyncTracker('is_favorite');
const paneViewerRefs = new Map<Pane, any>();
const isFullScreen = ref(false);
const wasMaximizedBeforeFullScreen = ref(false);
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
const imageMinScale = ref(0);       // Minimum image scale
const imageMaxScale = ref(10);      // Maximum image scale
const rightImageScale = ref(1);     // Right image scale
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
  minScale: number;
  maxScale: number;
}>>({
  bottomLeft: {
    fileId: 0, fileIndex: -1, fileInfo: null, nextFilePath: '',
    isZoomFit: true, scale: 1, minScale: 0, maxScale: 10,
  },
  bottomRight: {
    fileId: 0, fileIndex: -1, fileInfo: null, nextFilePath: '',
    isZoomFit: true, scale: 1, minScale: 0, maxScale: 10,
  },
});
const visiblePanes = computed<Pane[]>(() =>
  splitCount.value === 4 ? allPanes : ['left', 'right']
);
const showTaggingDialog = ref(false);
const showAddToCollectionDialog = ref(false);
const showCommentMsgbox = ref(false);
const metadataFiles = ref<any[]>([]);
const metadataFocus = 'comments';
const taggingFileIds = ref<number[]>([]);
const collectionFileIds = ref<number[]>([]);

let unlistenImg: () => void;
let unlistenGridView: () => void;
let unlistenAlbumAccessChanged: (() => void) | null = null;
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
  window.addEventListener('keydown', handleVideoSpaceKeyDown, true);
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

  unlistenAlbumAccessChanged = await listen('album-accessibility-changed', async (event: any) => {
    const { albumId, rootPath, available, libraryId } = event.payload || {};
    if (libraryId !== libConfig._libraryId) return;
    setAlbumAccessibility(albumId, available);
    resetFileAccessibility(rootPath);
    await Promise.all(getAvailablePanes().map(async pane => {
      const file = getFileInfoByPane(pane);
      if (!file || Number(file.album_id) !== Number(albumId)) return;
      const updated = await getFileInfo(file.id);
      if (libraryId !== libConfig._libraryId || getFileInfoByPane(pane)?.id !== file.id) return;
      if (pane === 'left') fileInfo.value = updated;
      else if (pane === 'right') rightFileInfo.value = updated;
      else extraPaneState[pane].fileInfo = updated;
    }));
  });

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
    const { message, fileId: targetFileId, changes, rotateDelta = 90 } = event.payload as any;
    console.log('message-from-content:', message, targetFileId);
    switch (message) {
      case 'rotate':
        for (const pane of allPanes) {
          if (targetFileId !== getFileIdByPane(pane)) continue;
          getViewerRef(pane)?.rotateView(rotateDelta);
          const target = getFileInfoByPane(pane);
          if (target) target.rotate = (target.rotate || 0) + rotateDelta;
        }
        if (targetFileId === fileId.value) {
          iconRotate.value += rotateDelta;
        }
        break;
      case 'file-saved': {
        const savedFile = (event.payload as any).file;
        const savedId = Number((event.payload as any).fileId || savedFile?.id || 0);
        if (savedId > 0 && savedFile) {
          applyFileMetaToPanes(savedId, savedFile);
          for (const pane of allPanes) {
            const paneFile = getFileInfoByPane(pane);
            if (!paneFile || Number(paneFile.id) !== savedId) continue;
            if (savedFile.file_path) uiStore.updateFileVersion(savedFile.file_path);
            const savedPath = paneFile.file_path;
            paneFile.file_path = '';
            void nextTick(() => { paneFile.file_path = savedPath; });
          }
        }
        break;
      }
      case 'update-file-meta': {
        const incomingCulling = changes?.culling_flag ?? changes?.cullingFlag;
        if (incomingCulling !== undefined) {
          cullingSync.recordExternalMetadata(targetFileId, Number(incomingCulling));
        }
        if (changes?.rating !== undefined) ratingSync.recordExternalMetadata(targetFileId, Number(changes.rating));
        if (changes?.is_favorite !== undefined) favoriteSync.recordExternalMetadata(targetFileId, Number(changes.is_favorite));
        applyFileMetaToPanes(targetFileId, changes || {});
        break;
      }
      default:
        break;
    }
  });

  unlistenFilesDeleted = await listen('files-deleted', (event: any) => {
    // Content re-broadcasts the settled list; ignore our own normal-mode
    // event so navigation never uses the pre-removal file count.
    if (event?.payload?.source === 'image-viewer' && !isCompareModeSession.value) return;
    const deletedIds = Array.isArray(event?.payload?.fileIds)
      ? event.payload.fileIds.map((id: any) => Number(id)).filter((id: number) => id > 0)
      : [];
    const updatedFiles = event?.payload?.updatedFiles;
    if (Array.isArray(updatedFiles)) {
      for (const updatedFile of updatedFiles) {
        if (updatedFile?.id) applyFileMetaToPanes(Number(updatedFile.id), updatedFile);
      }
    }
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

    const deletedIndices = Array.isArray(event?.payload?.deletedIndices)
      ? event.payload.deletedIndices.map((index: any) => Number(index)).filter((index: number) => index >= 0)
      : [];
    const shiftForIndex = (index: number) =>
      deletedIndices.reduce((shift, deletedIndex) => shift + (deletedIndex < index ? 1 : 0), 0);
    if (!isCompareModeSession.value) {
      if (fileIndex.value >= 0 && !deletedIds.includes(fileId.value)) {
        fileIndex.value = Math.max(0, fileIndex.value - shiftForIndex(fileIndex.value));
      }
      if (rightFileIndex.value >= 0 && !deletedIds.includes(rightFileId.value)) {
        rightFileIndex.value = Math.max(0, rightFileIndex.value - shiftForIndex(rightFileIndex.value));
      }
      for (const pane of ['bottomLeft', 'bottomRight'] as const) {
        const state = extraPaneState[pane];
        if (state.fileIndex >= 0 && !deletedIds.includes(state.fileId)) {
          state.fileIndex = Math.max(0, state.fileIndex - shiftForIndex(state.fileIndex));
        }
      }
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
  window.removeEventListener('keydown', handleVideoSpaceKeyDown, true);
  window.removeEventListener('resize', handleResize);
  document.documentElement.style.fontSize = '';
  clearSlideShowTimer();
  
  // unlisten
  unlistenImg();
  unlistenGridView();
  if (unlistenFilesDeleted) unlistenFilesDeleted();
  unlistenAlbumAccessChanged?.();
});

async function copyActiveFilePath() {
  try {
    const count = await copyFilePathsForFiles([getFileInfoByPane(getActivePane())], config.settings.groupRawJpegPairs);
    if (count > 0) toast.success(t('file_info.paths_copied', { count }));
  } catch (error) {
    console.error('Failed to copy file path:', error);
    toast.error(t('file_info.copy_path_failed'));
  }
}

async function copyActiveFiles(pane = getActiveFilePane()) {
  const file = getFileInfoByPane(pane);
  if (!file) return;
  try {
    const result = await copyFilesToClipboard([file], {
      groupRawJpegPairs: config.settings.groupRawJpegPairs,
    });
    if (result.offline) {
      toast.warning(t('offline.requires_original'));
      return;
    }
    if (result.copiedCount > 0) {
      toast.success(t('tooltip.copy_image.success', { count: result.copiedCount.toLocaleString() }));
      return;
    }
  } catch (error) {
    console.error('Failed to copy files to clipboard:', error);
  }
  toast.error(t('tooltip.copy_image.failed'));
}


const viewerContextMenuItems = useFileMenuItems(
  contextMenuFile,
  localeMsg,
  isMac,
  t,
  (action: string) => { void handleItemAction({ action }); },
  { hideViewInNewWindow: true },
);

async function openExternalApp(pane: Pane, appId?: string) {
  const file = getFileInfoByPane(pane);
  if (!file) return;
  if (isOriginalUnavailable(file)) {
    toast.warning(t('offline.requires_original'));
    return;
  }
  const target = resolveExternalAppTarget([file], appId, config);
  if (target.kind === 'empty' || target.kind === 'mixed' || target.kind === 'no-files') return;
  if (target.kind === 'no-app') {
    toast.warning(t('tooltip.open_external.no_app'));
    return;
  }
  try {
    await openFilesWithApp(target.paths, target.appPath);
  } catch (error) {
    toast.error(String(error).includes('File is online-only')
      ? t('offline.requires_original')
      : t('tooltip.open_external.failed'));
  }
}

const rawPairDeleteCount = computed(() => countRawPairTargets(deletePlan.value));
const singleDeleteCount = computed(() => countSingleTargets(deletePlan.value));
const deleteConfirmationCount = computed(() => countDeleteTargets(deletePlan.value, rawPairScope.value));
const deleteSubtitle = computed(() => t(deletePermanently.value
  ? 'msgbox.file_delete.permanent_summary' : 'msgbox.file_delete.trash_summary', deleteConfirmationCount.value));
const rawPairDeleteChoices = computed(() => {
  const format = rawPairCompanionFormat(deletePlan.value);
  return [
    { value: 'both', label: t('msgbox.raw_pair_delete.both') },
    { value: 'raw', label: t('msgbox.raw_pair_delete.raw') },
    { value: 'companion', label: format
      ? t('msgbox.raw_pair_delete.format', { format })
      : t('msgbox.raw_pair_delete.companion') },
  ];
});

function showActionError(error: unknown) {
  if (isOriginalUnavailableActionError(error)) {
    toast.warning(t('offline.requires_original'));
  } else {
    toast.error(String(error));
  }
}

function loadDeleteThumbnails(plan: FileDeletePlan) {
  for (const target of plan.files.slice(0, 5)) {
    void getFileThumbById(target.id, config.settings.thumbnailSize).then(thumb => {
      if (!thumb || deletePlan.value !== plan) return;
      const url = getThumbnailDataUrl(thumb, '', false, config.settings.thumbnailSize);
      if (url) deleteThumbnails.value[target.id] = url;
    });
  }
}

function requestDelete(pane: Pane, permanently = config.settings.fileDeleteBehavior === 'permanent') {
  const file = getFileInfoByPane(pane);
  if (!file) return;
  if (isOriginalUnavailable(file)) {
    toast.warning(t('offline.requires_original'));
    return;
  }
  const plan: FileDeletePlan = {
    libraryId: libConfig._libraryId,
    requestId: 0,
    files: buildFileDeleteTargets([file], Boolean(config.settings.groupRawJpegPairs)),
  };
  if (!plan.files.length) return;
  deletePlan.value = plan;
  deleteThumbnails.value = {};
  loadDeleteThumbnails(plan);
  deletePermanently.value = permanently;
  rawPairScope.value = 'both';
  if (shouldConfirmDelete(plan, permanently)) {
    showDeleteDialog.value = true;
    return;
  }
  void confirmDelete();
}

async function confirmDelete() {
  const plan = deletePlan.value;
  if (!plan || isDeleting.value) return;
  isDeleting.value = true;
  try {
    const result = await batchDeleteFiles(
      plan.files.map(file => ({ fileId: file.id, filePath: file.file_path })),
      deletePermanently.value,
      {
        libraryId: plan.libraryId,
        scope: rawPairScope.value,
        companions: plan.files.map(file => file.companion
          ? { fileId: file.companion.id, filePath: file.companion.file_path } : null),
      },
    );
    if (!result) {
      toast.error(t('msgbox.trash_failed.title'));
      return;
    }
    const deletedIds = Array.isArray(result?.deletedFileIds)
      ? result.deletedFileIds.map((id: any) => Number(id)).filter((id: number) => id > 0)
      : [];
    const failedIds = Array.isArray(result?.trashFailedFileIds)
      ? result.trashFailedFileIds.map((id: any) => Number(id)).filter((id: number) => id > 0)
      : [];
    if (deletedIds.length) {
      emit('files-deleted', {
        source: 'image-viewer',
        fileIds: deletedIds,
        fileCount: isCompareModeSession.value
          ? undefined
          : Math.max(0, fileCount.value - deletedIds.length),
      });
    }
    const failedCount = Math.max(0, Number(result.failedCount || 0));
    const retryFailures = failedIds.length;
    if (failedIds.length) {
      const retryFiles = buildDeleteRetryTargets(plan.files, failedIds);
      if (retryFiles.length) {
        deletePlan.value = { ...plan, files: retryFiles };
        loadDeleteThumbnails(deletePlan.value);
        deletePermanently.value = true;
        rawPairScope.value = 'both';
        showDeleteDialog.value = true;
        toast.error(t('msgbox.trash_failed.title'));
        const unhandledFailures = Math.max(0, failedCount - retryFailures);
        if (unhandledFailures > 0) toast.error(t('msgbox.raw_pair_delete.failed', unhandledFailures));
        return;
      }
    }
    if (failedCount > 0) {
      toast.error(t('msgbox.raw_pair_delete.failed', failedCount));
    }
    showDeleteDialog.value = false;
    deletePlan.value = null;
  } catch (error) {
    showActionError(error);
  } finally {
    isDeleting.value = false;
  }
}

function clickRename(pane: Pane) {
  const file = getFileInfoByPane(pane);
  if (!file) return;
  renameTarget.value = file;
  const parsed = extractFileName(file.name);
  renameName.value = parsed.name || '';
  renameExtension.value = parsed.ext || '';
  renameError.value = '';
  showRenameDialog.value = true;
}

async function submitRename(newName: string) {
  const target = renameTarget.value;
  if (!target) return;
  try {
    const result = await renameSingleFile(target, newName, renameExtension.value);
    if (!result) {
      renameError.value = t('msgbox.rename_file.error');
      return;
    }
    if (!result.unchanged) {
      // rename_file also renames the paired companion. Refresh the whole row
      // so live_photo_video_path / companion metadata do not stay stale.
      const refreshed = await getFileInfo(target.id).catch(() => null);
      const updatedFile = refreshed || {
        ...target,
        name: result.fileName,
        file_path: result.filePath,
      };
      Object.assign(target, updatedFile);
      applyFileMetaToPanes(Number(target.id), updatedFile);
      emit('message-from-image-viewer', {
        message: 'file-renamed',
        fileId: Number(target.id),
        name: result.fileName,
        filePath: result.filePath,
        file: updatedFile,
      });
    }
    showRenameDialog.value = false;
  } catch (error) {
    renameError.value = `${t('msgbox.rename_file.error')}: ${String(error)}`;
  }
}

async function openViewerMoreMenu(event: MouseEvent) {
  if (uiStore.inputStack.length > 0) return;
  const pane = getActiveFilePane();
  const file = getFileInfoByPane(pane);
  if (!file) return;
  const rect = (event.currentTarget as HTMLElement | null)?.getBoundingClientRect();
  contextMenuFile.value = file;
  await nextTick();
  await viewerContextMenuRef.value?.open(
    rect?.left ?? event.clientX,
    rect?.bottom ?? event.clientY,
  );
}

async function openPaneContextMenu(pane: Pane, event: MouseEvent) {
  setActivePane(pane);
  await openViewerContextMenu(event);
}

async function openViewerContextMenu(event: MouseEvent) {
  if (uiStore.inputStack.length > 0) return;
  const pane = getActivePane();
  const file = getFileInfoByPane(pane);
  if (!file) return;
  contextMenuFile.value = file;
  await nextTick();
  await viewerContextMenuRef.value?.open(event.clientX, event.clientY);
}

// Handle keyboard shortcuts
function handleVideoSpaceKeyDown(event: KeyboardEvent) {
  if (uiStore.inputStack.length > 0 || getFileInfoByPane(getActivePane())?.file_type !== 2) return;
  handleVideoEscapeShortcut(event, shortcutPlatform, () => closeWindow());
  if (event.defaultPrevented) return;
  handleVideoPlaybackShortcut(event, shortcutPlatform, () => getViewerRef(getActivePane())?.togglePlay?.());
}

function handleKeyDown(event: KeyboardEvent) {
  if (uiStore.isInputActive('MoveTo') && event.key === 'Escape') {
    event.preventDefault();
    showMoveTo.value = false;
    return;
  }
  if (uiStore.isInputActive('MoveTo') && event.key === 'Enter') {
    const target = event.target as HTMLElement | null;
    if (target?.closest('button, [role="button"], input, select, textarea')) return;
    event.preventDefault();
    moveToRef.value?.clickOk();
    return;
  }
  if(uiStore.inputStack.length > 0) {
    return;
  }

  const isEditingText = event.target instanceof HTMLElement
    && (event.target.isContentEditable || ['INPUT', 'TEXTAREA', 'SELECT'].includes(event.target.tagName));
  if (!isEditingText && matchesShortcut('file.rename', event, shortcutPlatform)) {
    event.preventDefault();
    clickRename(getActivePane());
    return;
  }
  if (!isEditingText && matchesShortcut('file.trash', event, shortcutPlatform)) {
    event.preventDefault();
    requestDelete(getActivePane(), config.settings.fileDeleteBehavior === 'permanent');
    return;
  }
  if (!isEditingText && matchesShortcut('file.deletePermanently', event, shortcutPlatform)) {
    event.preventDefault();
    requestDelete(getActivePane(), true);
    return;
  }
  if (!isEditingText && matchesShortcut('view.toggleFullscreen', event, shortcutPlatform)) {
    event.preventDefault();
    if (!event.repeat) toggleNativeFullScreen();
    return;
  }
  if (matchesShortcut('file.copyPath', event, shortcutPlatform) && !isEditingText) {
    event.preventDefault();
    if (!event.repeat) void copyActiveFilePath();
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

  // Keep only close and background shortcuts available during slideshow.
  if (
    isSlideShow.value &&
    !matchesShortcut('view.close', event, shortcutPlatform) &&
    !matchesShortcut('view.cycleBackground', event, shortcutPlatform) &&
    getMatchedViewBackground(event) === null
  ) {
    return;
  }

  if (matchesShortcut('file.copy', event, shortcutPlatform) && !isEditingText) {
    event.preventDefault();
    if (!event.repeat) void copyActiveFiles();
    return;
  }

  const ratingShortcut = getMatchedRating(event);
  if (ratingShortcut !== null) {
    event.preventDefault();
    if (!event.repeat) void setCurrentFileRating(ratingShortcut, getActiveFilePane(), event.shiftKey);
    return;
  }

  const cullingShortcut = getMatchedCulling(event);
  if (cullingShortcut !== null) {
    event.preventDefault();
    if (!event.repeat) void setCurrentFileCullingFlag(cullingShortcut, getActiveFilePane(), event.shiftKey);
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

  if (matchesShortcut('view.toggleAfAreas', event, shortcutPlatform)) {
    const target = event.target as HTMLElement | null;
    if (event.defaultPrevented || target?.isContentEditable || ['INPUT', 'TEXTAREA', 'SELECT'].includes(target?.tagName || '')) return;
    event.preventDefault();
    if (!event.repeat) getViewerRef(getActivePane())?.toggleAfAreas?.();
    return;
  }

  if (matchesShortcut('meta.favorite', event, shortcutPlatform)) {
    event.preventDefault();
    if (!event.repeat) void toggleFavorite(getActiveFilePane(), event.shiftKey);
    return;
  }

  if (matchesShortcut('meta.tag', event, shortcutPlatform)) {
    event.preventDefault();
    clickTag(getActiveFilePane());
    return;
  }

  if (matchesShortcut('meta.edit', event, shortcutPlatform)) {
    event.preventDefault();
    if (!event.repeat) openCommentEditor(getActiveFilePane());
    return;
  }

  if (matchesShortcut('meta.collection', event, shortcutPlatform)) {
    event.preventDefault();
    clickAddToCollection(getActiveFilePane());
    return;
  }

  if (!isEditingText && matchesShortcut('view.cycleRawPreview', event, shortcutPlatform)) {
    if (!event.repeat && getViewerRef(getActivePane())?.cycleRawPreview?.()) event.preventDefault();
    return;
  }

  if (matchesShortcut('meta.rotate', event, shortcutPlatform)
    || matchesShortcut('meta.rotateCounterclockwise', event, shortcutPlatform)) {
    event.preventDefault();
    void clickRotate(getActiveFilePane(), event.shiftKey ? -90 : 90);
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
const cullingActions: Array<{ actionId: ShortcutActionId; cullingFlag: number }> = [
  { actionId: 'meta.culling.pick', cullingFlag: 1 },
  { actionId: 'meta.culling.reject', cullingFlag: 2 },
  { actionId: 'meta.culling.unreviewed', cullingFlag: 0 },
];

function getMatchedRating(event: KeyboardEvent) {
  const match = ratingActions.find(({ actionId }) => matchesShortcut(actionId, event, shortcutPlatform));
  return match ? match.rating : null;
}

function getMatchedCulling(event: KeyboardEvent) {
  const match = cullingActions.find(({ actionId }) => matchesShortcut(actionId, event, shortcutPlatform));
  return match ? match.cullingFlag : null;
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

const paneDisplayFit = reactive<Partial<Record<Pane, boolean>>>({});
function handleViewportChange(viewport: any, pane: Pane) {
  paneDisplayFit[pane] = viewport.isZoomFit ?? false;
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
    const pauseWindowTransition = () => new Promise(resolve => setTimeout(resolve, 80));

    if (newFullScreen) {
      wasMaximizedBeforeFullScreen.value = await appWindow.isMaximized();
      await appWindow.setResizable(true);

      if (wasMaximizedBeforeFullScreen.value) {
        await appWindow.unmaximize();
        await pauseWindowTransition();
      }

      if (!(await appWindow.isFullscreen())) {
        await appWindow.setFullscreen(true);
      }
    } else {
      if (await appWindow.isFullscreen()) {
        await appWindow.setFullscreen(false);
        await pauseWindowTransition();
      }

      await appWindow.setResizable(true);

      if (wasMaximizedBeforeFullScreen.value) {
        await appWindow.maximize();
      }

      wasMaximizedBeforeFullScreen.value = false;
    }
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

async function loadPaneFileInfo(pane: Pane, targetId: number) {
  const requestId = cullingSync.beginFileInfoLoad(pane);
  ratingSync.beginFileInfoLoad(pane);
  favoriteSync.beginFileInfoLoad(pane);
  if (getFileInfoByPane(pane) && Number(getFileInfoByPane(pane).id) !== targetId) {
    setFileInfoByPane(pane, null);
  }
  const loaded = targetId > 0 ? await getFileInfo(targetId) : null;
  if (!cullingSync.isCurrentFileInfoLoad(pane, requestId, getFileIdByPane(pane), targetId)) {
    return;
  }
  const resolved = cullingSync.applyPendingMetadata(loaded, requestId);
  ratingSync.applyPendingMetadata(resolved, requestId);
  favoriteSync.applyPendingMetadata(resolved, requestId);
  setFileInfoByPane(pane, resolved);
  if (pane === 'left') {
    iconRotate.value = resolved?.rotate || 0;
    console.log('fileInfo:', resolved);
    if (isSlideShow.value) {
      scheduleNextSlide();
    }
  }
}

function setFileInfoByPane(pane: Pane, info: any) {
  if (pane === 'right') {
    rightFileInfo.value = info;
    return;
  }
  if (pane === 'bottomLeft' || pane === 'bottomRight') {
    extraPaneState[pane].fileInfo = info;
    return;
  }
  fileInfo.value = info;
}

// watch file changed
watch(() => fileId.value, async (newFileId) => {
  await loadPaneFileInfo('left', newFileId);
});

watch(() => rightFileId.value, async (newFileId) => {
  await loadPaneFileInfo('right', newFileId);
});

for (const pane of ['bottomLeft', 'bottomRight'] as const) {
  watch(() => extraPaneState[pane].fileId, async (newFileId) => {
    await loadPaneFileInfo(pane, newFileId);
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
  if (isCurrentFileVideo() && !isOriginalUnavailable(fileInfo.value)) {
    return;
  }

  const interval = getSlideShowInterval(slideShowIntervalIndex.value) * 1000;
  timer = setTimeout(() => {
    advanceSlideShow();
  }, interval);
}

watch(() => isOriginalUnavailable(fileInfo.value), () => {
  if (isSlideShow.value) scheduleNextSlide();
});

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
      isZoomFit: true, scale: 1, minScale: 0, maxScale: 10,
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
      scale: rightImageScale.value,
      min: rightImageMinScale.value, max: rightImageMaxScale.value,
    };
  }
  if (pane === 'bottomLeft' || pane === 'bottomRight') {
    const state = extraPaneState[pane];
    return {
      scale: state.scale,
      min: state.minScale, max: state.maxScale,
    };
  }
  return {
    scale: imageScale.value,
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
    const target = getMatchingPaneFileInfo(getFileInfoByPane(pane), targetFileId);
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
  getViewerRef(pane)?.toggleZoomFit?.();
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
    rightImageMinScale.value = event.minScale;
    rightImageMaxScale.value = event.maxScale;
    return;
  }
  if (pane === 'bottomLeft' || pane === 'bottomRight') {
    const state = extraPaneState[pane];
    state.scale = event.scale;
    state.minScale = event.minScale;
    state.maxScale = event.maxScale;
    return;
  }

  imageScale.value = event.scale;
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

function advanceAfterMetadataShortcut(fileId: number, pane: Pane) {
  if (getFileIdByPane(pane) === fileId && getActiveFilePane() === pane
      && getFileIndexByPane(pane) < fileCount.value - 1) {
    clickNext(pane);
  }
}

const toggleFavorite = (pane: Pane = 'left', advance = false) => {
  const target = getMatchingPaneFileInfo(getFileInfoByPane(pane), getFileIdByPane(pane));
  if (!target) return;
  return setCurrentMetadata('is_favorite', Number(!target.is_favorite), pane, advance);
};
const setCurrentFileRating = (rating: number, pane: Pane = 'left', advance = false) =>
  setCurrentMetadata('rating', rating, pane, advance);
const setCurrentFileCullingFlag = (flag: number, pane: Pane = 'left', advance = false) =>
  setCurrentMetadata('culling_flag', flag, pane, advance);

const metadataWriters = {
  culling_flag: { tracker: cullingSync, persist: setFileCullingFlag },
  rating: { tracker: ratingSync, persist: setFileRating },
  is_favorite: { tracker: favoriteSync, persist: (id: number, value: number) => setFileFavorite(id, Boolean(value)) },
};
async function setCurrentMetadata(field: keyof typeof metadataWriters, value: number, pane: Pane, advance: boolean) {
  const currentFileId = getFileIdByPane(pane);
  if (currentFileId <= 0) return;

  const target = getMatchingPaneFileInfo(getFileInfoByPane(pane), currentFileId);
  const knownConfirmed = target ? Number(target[field] ?? 0) : null;

  await metadataWriters[field].tracker.enqueueMetadataUpdate({
    fileId: currentFileId,
    value,
    knownConfirmed,
    pane,
    advance,
    applyOptimistic: (id, flag) => {
      applyFileMetaToPanes(id, { [field]: field === 'is_favorite' ? Boolean(flag) : flag });
    },
    persist: (id, flag) => metadataWriters[field].persist(id, flag),
    onCommit: (id, flag, settled, advanceInfo) => {
      if (settled) {
        applyFileMetaToPanes(id, { [field]: field === 'is_favorite' ? Boolean(flag) : flag });
      }
      syncFileMetaToContent(id, { [field]: field === 'is_favorite' ? Boolean(flag) : flag });
      if (field === 'culling_flag') void emit('culling-status-updated');
      if (advanceInfo?.advance) {
        advanceAfterMetadataShortcut(id, advanceInfo.pane);
      }
    },
    onError: (id, rollbackFlag) => {
      if (rollbackFlag !== null) {
        applyFileMetaToPanes(id, { [field]: field === 'is_favorite' ? Boolean(rollbackFlag) : rollbackFlag });
      }
      toast.error(t('file_info.metadata_save_failed'));
    },
  });
};

const clickRotate = async (pane: Pane = 'left', rotateDelta = 90) => {
  const target = getFileInfoByPane(pane);
  const currentFileId = getFileIdByPane(pane);
  if (!target || currentFileId <= 0) return;

  const rotate = (Number(target.rotate) || 0) + rotateDelta;
  applyFileMetaToPanes(currentFileId, { rotate });
  for (const targetPane of allPanes) {
    if (getFileIdByPane(targetPane) === currentFileId) {
      getViewerRef(targetPane)?.rotateView?.(rotateDelta);
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
  if (showCommentMsgbox.value || !getVisibleEditableMetadataFields(config.infoPanel.fields).length) return;
  const currentFileId = getFileIdByPane(pane);
  if (currentFileId <= 0) return;

  setActivePane(pane);
  metadataFiles.value = [activeFileInfo.value];
  showCommentMsgbox.value = true;
};

const clickAddToCollection = (pane: Pane = 'left') => {
  const currentFileId = getFileIdByPane(pane);
  if (currentFileId <= 0) return;
  setActivePane(pane);
  collectionFileIds.value = [currentFileId];
  showAddToCollectionDialog.value = true;
};

async function handleCollectionsAdded({ fileIds, results, changedCollectionIds = [], failed = 0 }: { fileIds: number[]; results: any[]; changedCollectionIds?: number[]; failed?: number }) {
  const addedFileIds = new Set(results.flatMap(result => result?.addedFileIds || []).map(Number));
  const changedFileIds = changedCollectionIds.length > 0 ? new Set(fileIds.map(Number)) : addedFileIds;
  if (changedFileIds.size === 0) {
    if (failed > 0) toast.error(t('collection.add_failed_toast', { count: failed }));
    showAddToCollectionDialog.value = false;
    return;
  }
  const updatedIds = new Set<number>();
  for (const pane of allPanes) {
    const currentFileId = getFileIdByPane(pane);
    if (!changedFileIds.has(currentFileId) || updatedIds.has(currentFileId)) continue;
    updatedIds.add(currentFileId);
    const collectionVersion = Number(getFileInfoByPane(pane)?.collectionVersion || 0) + 1;
    const hasCollections = addedFileIds.has(currentFileId)
      || Boolean((await getFileCollections(currentFileId))?.length);
    applyFileMetaToPanes(currentFileId, { has_collections: hasCollections, collectionVersion });
    syncFileMetaToContent(currentFileId, { has_collections: hasCollections, collectionVersion });
  }
  void emit('collection-files-dropped', { fileIds: [...changedFileIds] });
  if (failed > 0) toast.error(t('collection.add_failed_toast', { count: failed }));
  showAddToCollectionDialog.value = false;
}

async function handleCollectionDeleted(collectionId: number) {
  // Deleting a collection may have removed the current file from its last
  // collection, so re-check membership for the active panes. The dialog stays
  // open (the delete action doesn't close it).
  void collectionId;
  const updatedIds = new Set<number>();
  for (const pane of allPanes) {
    const currentFileId = getFileIdByPane(pane);
    if (currentFileId <= 0 || updatedIds.has(currentFileId)) continue;
    updatedIds.add(currentFileId);
    const collectionVersion = Number(getFileInfoByPane(pane)?.collectionVersion || 0) + 1;
    const hasCollections = Boolean((await getFileCollections(currentFileId))?.length);
    applyFileMetaToPanes(currentFileId, { has_collections: hasCollections, collectionVersion });
    syncFileMetaToContent(currentFileId, { has_collections: hasCollections, collectionVersion });
  }
}

function onMetadataSaved(files: any[], changes: Record<string, string>) {
  for (const file of files) {
    applyFileMetaToPanes(file.id, changes);
    syncFileMetaToContent(file.id, changes);
  }
  showCommentMsgbox.value = false;
}

async function updateFileHasTags(fileStates: Array<{ file_id: number; has_tags: boolean }>) {
  await syncTagStates(fileStates);
  showTaggingDialog.value = false;
  void emit("tags-changed");
}

async function syncTagStates(fileStates: Array<{ file_id: number; has_tags: boolean }>) {
  if (!Array.isArray(fileStates) || fileStates.length === 0) {
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
}

function openMoveDialog(pane: Pane, mode: 'move' | 'copy') {
  const file = getFileInfoByPane(pane);
  if (!file) return;
  moveTarget.value = file;
  moveMode.value = mode;
  showMoveTo.value = true;
}

function requestFileConflict(name: string, destination: string, allowReplace = true) {
  fileConflictDialog.value = { show: true, name, destination, showApplyAll: false, allowReplace };
  return new Promise<{ policy: FileConflictPolicy; applyAll: boolean }>(resolve => {
    fileConflictResolver = resolve;
  });
}

function resolveFileConflict(result: { policy: FileConflictPolicy; applyAll: boolean }) {
  fileConflictDialog.value.show = false;
  fileConflictResolver?.(result);
  fileConflictResolver = null;
}

async function selectSystemDestination(title: string): Promise<string | null> {
  const destination = await openDialog({ title, multiple: false, directory: true });
  return !destination || Array.isArray(destination) ? null : String(destination);
}

async function resolveLibraryDestination(destPath: string) {
  const albums = await getAllAlbums();
  const album = Array.isArray(albums) ? albums.find(item => isWithinRootPath(destPath, item.path)) : null;
  if (!album) return null;
  const folder = await selectFolder(album.id, destPath);
  return folder?.id ? { albumId: Number(album.id), folderId: Number(folder.id) } : null;
}

async function applyMovedFile(
  file: any,
  movedPath: string,
  fallback: { folderId?: number; albumId?: number },
) {
  const refreshed = await getFileInfo(file.id).catch(() => null);
  const updatedFile = refreshed || {
    ...file,
    file_path: movedPath,
    folder_id: fallback.folderId ?? file.folder_id,
    album_id: fallback.albumId ?? file.album_id,
  };
  Object.assign(file, updatedFile);
  applyFileMetaToPanes(Number(file.id), updatedFile);
  emit('message-from-image-viewer', {
    message: 'file-moved',
    fileId: file.id,
    path: movedPath,
    albumId: Number(updatedFile.album_id || 0),
    file: updatedFile,
  });
  return updatedFile;
}

async function transferFileToFolder(pane: Pane, mode: 'move' | 'copy') {
  const file = getFileInfoByPane(pane);
  if (!file) return;
  if (isOriginalUnavailable(file)) {
    toast.warning(t('offline.requires_original'));
    return;
  }
  const destPath = await selectSystemDestination(t(mode === 'copy' ? 'msgbox.copy_to_folder.title' : 'msgbox.move_to_folder.title'));
  if (!destPath) return;

  const destinationPath = getFullPath(destPath, file.name);
  if (normalizePathForCompare(file.file_path) === normalizePathForCompare(destinationPath)) {
    toast.info(t('msgbox.file_conflict.same_folder'));
    return;
  }

  const libraryDestination = await resolveLibraryDestination(destPath);
  if (mode === 'move' && !libraryDestination) {
    const confirmed = await ask(
      t('msgbox.move_to_folder.warning', { source: file.name, dest: destPath }),
      {
        title: t('msgbox.move_to_folder.confirm_title'),
        kind: 'warning',
        okLabel: t('msgbox.move_to_folder.ok'),
        cancelLabel: t('msgbox.cancel'),
      },
    );
    if (!confirmed) return;
  }

  let conflictPolicy: FileConflictPolicy = 'keep_both';
  if (await checkFileExists(destinationPath)) {
    const conflict = await requestFileConflict(file.name, destPath);
    if (conflict.policy === 'skip') return;
    conflictPolicy = conflict.policy;
  }

  const destLabel = getFolderName(destPath) || destPath;
  try {
    if (mode === 'move') {
      const movedPath = libraryDestination
        ? await moveFile(file.id, file.file_path, libraryDestination.folderId, destPath, conflictPolicy)
        : await moveFileOutsideLibrary(file.id, file.file_path, destPath, conflictPolicy);
      if (movedPath) {
        if (!libraryDestination) {
          // move_file_outside_library removes the database row. Drop the
          // viewer's stale file identity so follow-up metadata actions cannot
          // target a deleted record.
          emit('files-deleted', {
            source: 'image-viewer',
            fileIds: [file.id],
            fileCount: isCompareModeSession.value ? undefined : Math.max(0, fileCount.value - 1),
            albumId: Number(file.album_id || 0),
          });
        } else {
          await applyMovedFile(file, movedPath, {
            folderId: libraryDestination.folderId,
            albumId: libraryDestination.albumId,
          });
        }
        toast.success(t('msgbox.move_to_folder.success', { source: file.name, dest: destLabel }));
      } else {
        toast.error(t('msgbox.move_to_folder.error', { source: file.name, dest: destLabel }));
      }
    } else {
      const copiedPath = await copyFile(file.file_path, destPath, conflictPolicy);
      if (copiedPath) {
        if (libraryDestination) await addFileToDb(libraryDestination.folderId, copiedPath);
        emit('message-from-image-viewer', { message: 'file-copied', fileId: file.id, path: copiedPath, albumId: libraryDestination?.albumId });
        toast.success(t('msgbox.copy_to_folder.success', { source: file.name, dest: destLabel }));
      } else {
        toast.error(t('msgbox.copy_to_folder.error', { source: file.name, dest: destLabel }));
      }
    }
  } catch (error) {
    showActionError(error);
  }
}

async function onMoveTo() {
  const file = moveTarget.value;
  if (!file) return;
  if (isOriginalUnavailable(file)) {
    toast.warning(t('offline.requires_original'));
    return;
  }
  const destPath = String(libConfig.destFolder.folderPath || '');
  const destAlbumId = Number(libConfig.destFolder.albumId || 0);
  let destFolderId = Number(libConfig.destFolder.folderId || 0);
  if (!destPath) return;
  if (destFolderId <= 0 && destAlbumId > 0) {
    const resolved = await selectFolder(destAlbumId, destPath);
    destFolderId = Number(resolved?.id || 0);
  }
  const destinationPath = getFullPath(destPath, file.name);
  if (normalizePathForCompare(file.file_path) === normalizePathForCompare(destinationPath)) {
    toast.info(t('msgbox.file_conflict.same_folder'));
    showMoveTo.value = false;
    return;
  }
  let conflictPolicy: FileConflictPolicy = 'keep_both';
  if (await checkFileExists(destinationPath)) {
    const conflict = await requestFileConflict(file.name, destPath);
    if (conflict.policy === 'skip') return;
    conflictPolicy = conflict.policy;
  }
  try {
    if (moveMode.value === 'copy') {
      const copiedPath = await copyFile(file.file_path, destPath, conflictPolicy);
      if (copiedPath) {
        if (destFolderId > 0) await addFileToDb(destFolderId, copiedPath);
        emit('message-from-image-viewer', { message: 'file-copied', fileId: file.id, path: copiedPath, albumId: destAlbumId });
        toast.success(t('msgbox.copy_to.success', { source: file.name, dest: destPath }));
      } else {
        toast.error(t('msgbox.copy_to.error', { source: file.name, dest: destPath }));
      }
    } else {
      const movedPath = await moveFile(file.id, file.file_path, destFolderId, destPath, conflictPolicy);
      if (movedPath) {
        await applyMovedFile(file, movedPath, {
          folderId: destFolderId,
          albumId: destAlbumId,
        });
        toast.success(t('msgbox.move_to.success', { source: file.name, dest: destPath }));
      } else {
        toast.error(t('msgbox.move_to.error', { source: file.name, dest: destPath }));
      }
    }
    showMoveTo.value = false;
  } catch (error) {
    showActionError(error);
  }
}

async function editActiveImage(pane: Pane) {
  const file = getFileInfoByPane(pane);
  if (!file || ![1, 3].includes(Number(file.file_type))) return;
  if (isPreviewOnlyDocument(file.file_path || '')) return;
  if (isOriginalUnavailable(file)) {
    toast.warning(t('offline.requires_original'));
    return;
  }
  const fileId = Number(file.id || 0);
  if (fileId <= 0) return;

  // Register the save-as context in the main window before the editor can save.
  emit('message-from-image-viewer', { message: 'register-image-editor-context', fileId });
  const existing = await WebviewWindow.getByLabel('imageeditor');
  if (existing) {
    await existing.emit('update-file', { fileId });
    existing.show();
    existing.setFocus();
    return;
  }

  const editor = new WebviewWindow('imageeditor', {
    url: `/image-editor?fileId=${fileId}`,
    title: 'Image Editor',
    width: 1100,
    height: 700,
    minWidth: 800,
    minHeight: 500,
    resizable: true,
    maximizable: false,
    visible: false,
    transparent: true,
    decorations: isMac,
    ...(isMac && {
      titleBarStyle: 'overlay',
      hiddenTitle: true,
      minimizable: false,
    }),
  });
  editor.once('tauri://created', () => { void editor.show(); });
}

async function printActiveImage(pane: Pane) {
  const file = getFileInfoByPane(pane);
  if (!file?.file_path) return;
  if (isOriginalStorageUnavailable(file) || await isCloudOnlyFile(file.file_path)) {
    toast.warning(t('offline.requires_original'));
    return;
  }
  try {
    const fileType = Number(file.file_type || 1);
    const source = shouldUseBackendPreview(file.file_path, fileType)
      ? getPreviewUrl(Number(file.id), file.file_path, false, Number(file.modified_at || 0))
      : getAssetSrc(file.file_path, Number(file.modified_at || 0));
    await printImageView.value?.print(source);
  } catch (error) {
    console.error('Failed to prepare image for printing:', error);
  }
}

async function refreshActiveFileInfo(pane: Pane) {
  const file = getFileInfoByPane(pane);
  if (!file) return;
  try {
    const refreshed = await refreshFileInfo(libConfig._libraryId, file.id);
    const message = [refreshed?.warning, refreshed?.error].filter(Boolean).join('; ');
    if (refreshed?.file) {
      Object.assign(file, refreshed.file);
      applyFileMetaToPanes(Number(file.id), refreshed.file);
      emit('message-from-image-viewer', {
        message: 'file-refreshed',
        fileId: Number(file.id),
        file: refreshed.file,
      });
    }
    if (refreshed?.error) toast.error(message);
    else if (refreshed?.warning) toast.warning(message);
    else if (refreshed?.file) toast.success(t('tooltip.update_image.success'));
    else toast.error(t('tooltip.update_image.failed'));
  } catch (error) {
    toast.error(t('tooltip.update_image.failed'));
  }
}

async function setActiveAlbumCover(pane: Pane) {
  const file = getFileInfoByPane(pane);
  const albumId = Number(file?.album_id || libConfig.album.id || 0);
  if (!file || albumId <= 0) return;
  try {
    const result = await setAlbumCover(albumId, file.id);
    if (!result) throw new Error('Failed to set album cover');
    emit('album-cover-changed', { albumId, fileId: file.id });
    toast.success(t('tooltip.set_album_cover.success'));
  } catch (error) {
    toast.error(t('tooltip.set_album_cover.failed'));
  }
}

async function setActiveDesktopWallpaper(pane: Pane) {
  const file = getFileInfoByPane(pane);
  if (!file?.file_path) return;
  if (isOriginalUnavailable(file)) {
    toast.warning(t('offline.requires_original'));
    return;
  }
  const companionPath = file.media_subtype === 'raw_jpeg_pair'
    ? String(file.live_photo_video_path || '')
    : null;
  try {
    await setDesktopWallpaper(file.file_path, companionPath || null);
    toast.success(t('tooltip.set_desktop_wallpaper.success'));
  } catch (error) {
    toast.error(t('tooltip.set_desktop_wallpaper.failed'));
  }
}

async function saveActiveXmp(pane: Pane) {
  const file = getFileInfoByPane(pane);
  if (!file) return;
  try {
    const result = await saveXmpMetadata([file.id], libConfig._libraryId);
    if (result?.savedIds?.length) {
      toast.success(t('menu.file.xmp_save_success', { count: result.savedIds.length }));
    }
    if (result?.failures?.length) {
      toast.error(t('menu.file.xmp_save_failed', { count: result.failures.length }) + ': ' + result.failures[0].error);
    }
  } catch (error) {
    toast.error(String(error));
  }
}

function runInMainWindow(action: string) {
  const pane = getActiveFilePane();
  const file = getFileInfoByPane(pane);
  if (!file) return;
  emit('message-from-image-viewer', {
    message: 'run-file-action',
    action,
    fileId: Number(file.id),
  });
}

const handleItemAction = async (payload: { action: string }) => {
  const pane = getActiveFilePane();

  if (payload.action.startsWith('open-external-app:')) {
    await openExternalApp(pane, payload.action.slice('open-external-app:'.length));
    return;
  }

  switch (payload.action) {
    case 'copy':
      await copyActiveFiles(pane);
      break;
    case 'reveal':
      revealFile(getFileInfoByPane(pane));
      break;
    case 'rename':
      clickRename(pane);
      break;
    case 'open-external-app':
      await openExternalApp(pane);
      break;
    case 'trash':
      requestDelete(pane, config.settings.fileDeleteBehavior === 'permanent');
      break;
    case 'favorite':
      await toggleFavorite(pane);
      break;
    case 'rotate':
      await clickRotate(pane);
      break;
    case 'tag':
      clickTag(pane);
      break;
    case 'add-to-collection':
      clickAddToCollection(pane);
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
    case 'culling-pick':
      await setCurrentFileCullingFlag(1, pane);
      break;
    case 'culling-reject':
      await setCurrentFileCullingFlag(2, pane);
      break;
    case 'culling-unreviewed':
      await setCurrentFileCullingFlag(0, pane);
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
    case 'save-xmp':
      await saveActiveXmp(pane);
      break;
    case 'edit':
      await editActiveImage(pane);
      break;
    case 'print':
      await printActiveImage(pane);
      break;
    case 'refresh-file-info':
      await refreshActiveFileInfo(pane);
      break;
    case 'set-album-cover':
      await setActiveAlbumCover(pane);
      break;
    case 'set-desktop-wallpaper':
      await setActiveDesktopWallpaper(pane);
      break;
    case 'move-within-library':
      openMoveDialog(pane, 'move');
      break;
    case 'move-to-folder':
      await transferFileToFolder(pane, 'move');
      break;
    case 'copy-to-folder':
      await transferFileToFolder(pane, 'copy');
      break;
    default:
      runInMainWindow(payload.action);
      break;
  }
};

</script>

<style scoped>
* {
  user-select: none;
}
</style>

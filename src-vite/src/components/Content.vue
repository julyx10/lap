<template>

  <div class="relative flex-1 flex flex-col select-none"
    :class="{ 'opacity-50 pointer-events-none': uiStore.isInputActive('AlbumList-edit') }"
    @keydown="handleLocalKeyDown"
  >

    <!-- Loading overlay -->
    <transition name="fade">
      <div v-if="isProcessing" class="absolute inset-0 bg-base-100/50 flex items-center justify-center z-50 rounded-box">
        <span class="loading loading-dots text-primary"></span>
      </div>
    </transition>

    <!-- title bar -->
    <div v-if="!uiStore.isFullScreen"
      class="absolute top-0 left-0 right-0 px-2 h-12 flex flex-row flex-wrap items-center justify-between bg-base-300/80 backdrop-blur-md z-30"  
      data-tauri-drag-region
    >
      <!-- title -->
      <div class="mr-1 flex flex-row items-center gap-1 min-w-0 flex-1" data-tauri-drag-region>
        <component v-if="currentTitleIcon" 
          :is="currentTitleIcon" 
          class="t-icon-size-sm shrink-0"
          :class="{ 'cursor-pointer text-primary': tempViewMode !== 'none' }" 
          @click="handleTitleClick"
        />
        <div class="cursor-default overflow-hidden whitespace-pre text-ellipsis">
          <span 
            :class="{ 'cursor-pointer text-primary': tempViewMode !== 'none' }"
            data-tauri-drag-region 
            @click="handleTitleClick"
          >{{ contentTitle }}</span>
        </div>
        <TButton v-if="tempViewMode !== 'none'" 
          :icon="IconRestore" 
          :buttonSize="'medium'"
          :tooltip="$t('toolbar.tooltip.restore')"
          :selected="true"
          @click="exitTempViewMode" 
        />
      </div>

      <!-- toolbar -->
      <div class="flex items-center gap-2 shrink-0">

        <!-- select mode -->
        <div tabindex="-1"
          :class="[
            'px-2 py-1 h-8 flex flex-row items-center rounded-box border focus:outline-none text-sm shrink-0',
            fileList.length === 0 || showQuickView || isIndexing ? 'text-base-content/30' : 'border-base-content/30 hover:bg-base-100 hover:text-base-content cursor-pointer',
            selectMode ? 'border-primary' : ''
          ]"
          @click="handleSelectMode(true)"
        >
          <TButton v-if="selectMode"
            :icon="IconClose"
            :buttonSize="'small'"
            @click.stop="handleSelectMode(false)" 
          />
          <span class="px-1">{{ selectMode ? $t('toolbar.filter.select_count', { count: selectedCount.toLocaleString() }) : $t('toolbar.filter.select_mode') }}</span>
          <ContextMenu v-if="selectMode"
            :iconMenu="IconArrowDown"
            :menuItems="moreMenuItems"
            :smallIcon="true"
            @click.stop
          />
        </div>

        <!-- file type options -->
        <DropDownSelect
          :options="fileTypeOptions"
          :defaultIndex="config.search.fileType"
          :disabled="config.main.sidebarIndex === 3 || tempViewMode !== 'none' || isIndexing || showQuickView"
          :selected="config.search.fileType !== 0"
          @select="handleFileTypeSelect"
        />

        <!-- sort type options -->
        <DropDownSelect
          :options="sortOptions"
          :defaultIndex="config.search.sortType"
          :extendOptions="sortExtendOptions"
          :defaultExtendIndex="config.search.sortOrder"
          :disabled="config.main.sidebarIndex === 3 || tempViewMode !== 'none' || isIndexing || showQuickView"
          @select="handleSortTypeSelect"
        />

        <!-- select and layout section -->
        <div class="flex flex-row items-center">
          <IconSeparator class="t-icon-size-sm text-base-content/30" />
          
          <!-- refresh file list -->
          <!-- Bugfix: need to update album files when the album is updated -->
          <!-- <TButton
            :icon="IconRefresh"
            :tooltip="$t('toolbar.tooltip.refresh')"
            :disabled="isIndexing || showQuickView"
            @click="updateContent()"
          /> -->
          
          <!-- grid size slider -->
          <div class="flex flex-row items-center gap-2 px-2 shrink-0 group">
            <SliderInput v-model="config.settings.grid.size" 
              :min="120" :max="360" :step="1" label="" :slider_width="80" 
              :disabled="isIndexing || showQuickView" 
            />
          </div>

          <!-- grid styles cycle -->
          <TButton
            :icon="[IconCard, IconTile, IconJustified][config.settings.grid.style]"
            :tooltip="localeMsg.settings.gallery_view.style_options[config.settings.grid.style]"
            :disabled="isIndexing || showQuickView"
            @click="cycleGridStyle"
          />

          <!-- toggle filmstrip -->
          <TButton
            :icon="IconFilmstrip"
            :iconStyle="{ 
              transform: `rotate(${config.settings.grid.previewPosition === 1 ? 180 : 0}deg)`, 
              transition: 'transform 0.3s ease-in-out' 
            }" 
            :tooltip="localeMsg.settings.filmstrip_view.title"
            :selected="config.settings.grid.showFilmStrip"
            :disabled="isIndexing || showQuickView"
            @click="toggleFilmstripView"
          />

          <!-- toggle info panel -->
          <TButton
            :icon="config.infoPanel.show ? IconSideBarOn : IconSideBarOff"
            :tooltip="config.infoPanel.show ? $t('toolbar.tooltip.hide_info') : $t('toolbar.tooltip.show_info')"
            :selected="config.infoPanel.show"
            :disabled="isIndexing"
            @click="config.infoPanel.show ? checkUnsavedChanges(() => config.infoPanel.show = false) : config.infoPanel.show = true"
          />
        </div>
      </div>
    </div>

    <!-- progress bar -->
    <div v-if="fileList.length > 0 && showProgressBar" class="absolute top-11 left-0 right-0 z-50">
      <ProgressBar :percent="Number(((thumbCount / fileList.length) * 100).toFixed(0))" />
    </div>

    <!-- content view -->
    <div ref="contentViewDiv" class="relative flex-1 flex flex-row overflow-hidden">
      <div class="relative flex-1 flex flex-row overflow-hidden">
        <div ref="gridViewDiv" 
          :class="[
            'flex-1 flex',
            config.settings.grid.previewPosition === 0 ? 'flex-col-reverse' : 'flex-col',
            config.settings.grid.showFilmStrip ? (config.settings.showStatusBar ? 'mt-12 mb-8' : 'mt-12 mb-1') : ''
          ]"
        >
          <div class="relative" 
            :class="{ 'flex-1': !config.settings.grid.showFilmStrip }"
            :style="{ height: config.settings.grid.showFilmStrip ? itemSize + 'px' : '' }"
          >
            <!-- grid view -->
            <div ref="gridScrollContainerRef" class="absolute w-full h-full">
              <GridView ref="gridViewRef"
                :selected-item-index="selectedItemIndex"
                :fileList="fileList"
                :showFolderFiles="showFolderFiles"
                :selectMode="selectMode"
                :loading="isLoading"
                :layout-version="layoutVersion"
                @item-clicked="handleItemClicked"
                @item-dblclicked="handleItemDblClicked"
                @item-select-toggled="handleItemSelectToggled"
                @item-action="handleItemAction"
                @visible-range-update="handleVisibleRangeUpdate"
                @scroll="handleGridScroll"
                @layout-update="handleLayoutUpdate"
              />
              <!-- Navigation buttons -->
              <div v-if="config.settings.grid.showFilmStrip && fileList.length > 0" 
                class="absolute z-10 inset-1 flex items-center justify-between pointer-events-none"
              >
                <button 
                  :class="[
                    'p-2 rounded-full pointer-events-auto bg-base-100/30', 
                    selectedItemIndex > 0 ? 'text-base-content/70 hover:text-base-content hover:bg-base-100/70 cursor-pointer' : 'text-base-content/30'
                  ]"
                  @click="requestNavigate('prev')"
                  @dblclick.stop
                >
                  <IconLeft class="w-8 h-8" />
                </button>
                <button 
                  :class="[
                    'p-2 rounded-full pointer-events-auto bg-base-100/30', 
                    selectedItemIndex < fileList.length - 1 ? 'text-base-content/70 hover:text-base-content hover:bg-base-100/70 cursor-pointer' : 'text-base-content/30'
                  ]"
                  @click="requestNavigate('next')" 
                  @dblclick.stop
                >
                  <IconRight class="w-8 h-8" />
                </button> 
              </div>
            </div>
          </div>

          <div v-if="config.settings.grid.showFilmStrip" class="h-1"></div>

          <!-- film strip preview -->
          <div v-if="config.settings.grid.showFilmStrip" ref="previewDiv" 
            class="flex-1 bg-base-200 overflow-hidden"
          >
            <div v-if="selectedItemIndex >= 0 && selectedItemIndex < fileList.length"
              class="w-full h-full flex items-center justify-center"
            >
              <MediaViewer
                ref="filmStripMediaRef"
                :mode="1"
                :isFullScreen="false"
                :file="fileList[selectedItemIndex]"
                :hasPrevious="selectedItemIndex > 0"
                :hasNext="selectedItemIndex < fileList.length - 1"
                :fileIndex="selectedItemIndex"
                :fileCount="fileList.length"
                :isSlideShow="isSlideShow"
                :imageScale="imageScale"
                :imageMinScale="imageMinScale"
                :imageMaxScale="imageMaxScale"
                v-model:isZoomFit="filmStripZoomFit"
                @prev="performNavigate('prev')"
                @next="performNavigate('next')"
                @toggle-slide-show="toggleSlideShow"
                @scale="onScale"
                @item-action="handleItemAction"
                @slideshow-next="handleSlideshowNext"
              />
            </div>
          </div> <!-- film strip preview -->
        </div> <!-- grid view -->

        <!-- custom scrollbar -->
        <div v-if="!config.settings.grid.showFilmStrip && fileList.length > 0" 
          class="mt-12 shrink-0" 
          :class="[ config.settings.showStatusBar ? 'mb-8' : 'mb-1' ]"
        >
          <ScrollBar
            :total="totalFileCount"
            :pageSize="visibleItemCount"
            :modelValue="scrollPosition"
            :markers="timelineData"
            :selectedIndex="selectedItemIndex"
            @update:modelValue="handleScrollUpdate"
            @select-item="handleItemClicked"
          ></ScrollBar>
        </div>

        <!-- Quick View Overlay -->
        <div v-if="showQuickView && fileList[selectedItemIndex]" 
          class="absolute inset-0 z-60 flex items-center justify-center bg-base-200/80 backdrop-blur-md overflow-hidden"
          :class="[
            !uiStore.isFullScreen ? (config.settings.showStatusBar ? 'mt-12 mb-8': 'mt-12 mb-1') : ''
          ]"
        >
          <div class="relative w-full h-full flex items-center justify-center">
            <MediaViewer
              ref="quickViewMediaRef"
              :mode="0"
              :isFullScreen="uiStore.isFullScreen"
              :file="fileList[selectedItemIndex]"
              :hasPrevious="selectedItemIndex > 0"
              :hasNext="selectedItemIndex < fileList.length - 1"
              :fileIndex="selectedItemIndex"
              :fileCount="fileList.length"
              :isSlideShow="isSlideShow"
              :imageScale="imageScale"
              :imageMinScale="imageMinScale"
              :imageMaxScale="imageMaxScale"
              v-model:isZoomFit="quickViewZoomFit"
              @prev="performNavigate('prev')"
              @next="performNavigate('next')"
              @toggle-slide-show="toggleSlideShow"
              @scale="onScale"
              @item-action="handleItemAction"
              @toggle-full-screen="uiStore.isFullScreen = !uiStore.isFullScreen"
              @close="showQuickView = false; uiStore.isFullScreen = false; stopSlideShow()"
              @slideshow-next="handleSlideshowNext"
            />
          </div>
        </div>
      </div>

      <!-- info panel splitter -->
      <div v-if="config.infoPanel.show && !uiStore.isFullScreen"
        class="w-1 shrink-0 transition-colors mt-12"
        :class="{
          'mb-8': config.settings.showStatusBar,
          'mb-1': !config.settings.showStatusBar,
          'hover:bg-primary cursor-col-resize': config.infoPanel.show,
          'bg-primary': config.infoPanel.show && isDraggingInfoPanel,
        }" 
        @mousedown="startDraggingInfoPanelSplitter"
      ></div>

      <!-- info panel -->
      <transition
        enter-active-class="transition-all duration-200 ease-in-out overflow-hidden"
        leave-active-class="transition-all duration-200 ease-in-out overflow-hidden"
        enter-from-class="!w-0 opacity-0"
        enter-to-class="opacity-100"
        leave-from-class="opacity-100"
        leave-to-class="!w-0 opacity-0"
      >
        <div v-if="config.infoPanel.show && !uiStore.isFullScreen"
          :class="[ 'pt-12 pr-1', config.settings.showStatusBar ? 'pb-8' : 'pb-1' ]" 
          :style="{ width: config.infoPanel.width + '%' }">
          <FileInfo 
            ref="fileInfoRef"
            :fileInfo="fileList[selectedItemIndex]" 
            :multiSelect="selectMode"
            :selectedFiles="selectedFiles"
            @close="checkUnsavedChanges(() => config.infoPanel.show = false)" 
            @success="onImageEdited(true)"
            @failed="onImageEdited(false)"
            @deselect="(file: any) => file.isSelected = false"
            @favoriteAll="selectModeSetFavorites(true)"
            @unfavoriteAll="selectModeSetFavorites(false)"
            @tagAll="clickTag"
          />
        </div>
      </transition>

      <!-- Indexing Overlay -->
      <div v-if="isIndexing" 
        class="absolute inset-0 z-100 bg-base-200/80 backdrop-blur-md flex flex-col items-center justify-center gap-4 text-base-content/30"
        :class="[ config.settings.showStatusBar ? 'mt-12 mb-8': 'mt-12 mb-1' ]"
      >
        <IconUpdate class="mx-1 w-8 h-8 animate-spin"/>
        <span class="text-lg text-center">{{ libConfig.index.albumQueue[0] === libConfig.album.id
          ? $t('search.index.indexing', { album: libConfig.index.albumName, count: libConfig.index.indexed.toLocaleString(), total: libConfig.index.total.toLocaleString() }) 
          : $t('search.index.wait_index') 
        }}</span>
        <span class="text-sm text-center">{{ $t('search.index.description') }}</span>
        <button class="btn btn-primary" @click="cancelIndexing">
          <IconClose class="w-5 h-5" />
          {{ $t('search.index.cancel') }}
        </button>
      </div>
    </div>

    <StatusBar
      v-if="config.settings.showStatusBar"
      :file-list="fileList"
      :selected-item-index="selectedItemIndex"
      :total-file-count="totalFileCount"
      :total-file-size="totalFileSize"
      :select-mode="selectMode"
      :selected-count="selectedCount"
      :selected-size="selectedSize"
      :show-film-strip="config.settings.grid.showFilmStrip"
      :show-quick-view="showQuickView"
      :image-scale="imageScale"
    />
  </div>

  <!-- image editor -->
  <ImageEditor 
    v-if="showImageEditor"
    :fileInfo="fileList[selectedItemIndex]" 
    @success="onImageEdited(true)" 
    @failed="onImageEdited(false)"
    @cancel="showImageEditor = false"
  />

  <!-- rename -->
  <MessageBox
    v-if="showRenameMsgbox"
    :title="$t('msgbox.rename_file.title')"
    :showInput="true"
    :inputText="renamingFileName.name"
    :inputPlaceholder="$t('msgbox.rename_file.placeholder')"
    :inputExtension="renamingFileName.ext"
    :needValidateInput="true"
    :OkText="$t('msgbox.rename_file.ok')"
    :cancelText="$t('msgbox.cancel')"
    :errorMessage="errorMessage"
    @ok="onRenameFile"
    @cancel="showRenameMsgbox = false"
    @reset="errorMessage = ''"
  />

  <!-- move to -->
  <MoveTo
    v-if="showMoveTo"
    :title="`${$t('msgbox.move_to.title', { source: selectMode ? $t('toolbar.filter.select_count', { count: selectedCount.toLocaleString() }) : shortenFilename(fileList[selectedItemIndex].name, 32) })}`"
    :message="$t('msgbox.move_to.content')"
    :OkText="$t('msgbox.move_to.ok')" 
    :cancelText="$t('msgbox.cancel')"
    @ok="onMoveTo"
    @cancel="showMoveTo = false"
  />

  <!-- copy to -->
  <MoveTo
    v-if="showCopyTo"
    :title="`${$t('msgbox.copy_to.title', { source: selectMode ? $t('toolbar.filter.select_count', { count: selectedCount.toLocaleString() }) : shortenFilename(fileList[selectedItemIndex].name, 32) })}`"
    :message="$t('msgbox.copy_to.content')"
    :OkText="$t('msgbox.copy_to.ok')"
    :cancelText="$t('msgbox.cancel')"
    @ok="onCopyTo"
    @cancel="showCopyTo = false"
  />

  <!-- move to trash -->
  <MessageBox
    v-if="showTrashMsgbox"
    :title="selectMode ? $t('msgbox.trash_files.title') : $t('msgbox.trash_file.title')"
    :message="selectMode ? $t('msgbox.trash_files.content', { count: selectedCount.toLocaleString() }) : $t('msgbox.trash_file.content', { file: fileList[selectedItemIndex].name })"
    :OkText="selectMode ? $t('msgbox.trash_files.ok') : $t('msgbox.trash_file.ok')"
    :cancelText="$t('msgbox.cancel')"
    :warningOk="true"
    @ok="onTrashFile"
    @cancel="showTrashMsgbox = false"
  />

  <!-- tag -->
  <TaggingDialog 
    v-if="showTaggingDialog"
    :fileIds="fileIdsToTag"
    @ok="updateFileHasTags"
    @cancel="showTaggingDialog = false"
  />

  <!-- comment -->
  <MessageBox
    v-if="showCommentMsgbox"
    :title="$t('msgbox.edit_comment.title')"
    :showInput="true"
    :inputText="fileList[selectedItemIndex]?.comments ?? ''"
    :inputPlaceholder="$t('msgbox.edit_comment.placeholder')"
    :multiLine="true"
    :OkText="$t('msgbox.ok')"
    :cancelText="$t('msgbox.cancel')"
    @ok="onEditComment"
    @cancel="showCommentMsgbox = false"
  />

  <!-- Unsaved Changes Confirmation -->
  <MessageBox
    v-if="showUnsavedChangesMsgbox"
    :title="$t('msgbox.unsaved_changes.title') || 'Unsaved Changes'"
    :message="$t('msgbox.unsaved_changes.message') || 'You have unsaved changes. Do you want to save them?'"
    :OkText="$t('msgbox.image_editor.save') || 'Save'"
    :cancelText="$t('msgbox.cancel')"
    :thirdText="$t('msgbox.discard') || 'Discard'"
    :warningThird="true"
    @ok="confirmSave"
    @cancel="cancelDiscard"
    @third="confirmDiscard"
  />

  <ToolTip ref="toolTipRef" />

</template>

<script setup lang="ts">

import { ref, watch, computed, onMounted, onBeforeUnmount, nextTick } from 'vue';
import { emit as tauriEmit, listen } from '@tauri-apps/api/event';
import { WebviewWindow } from '@tauri-apps/api/webviewWindow';
import { useI18n } from 'vue-i18n';
import { useUIStore } from '@/stores/uiStore';
import { getAlbum, getQueryCountAndSum, getQueryTimeLine, getQueryFiles, getFolderFiles, getFolderThumbCount,
         copyImage, renameFile, moveFile, copyFile, deleteFile, deleteDbFile, editFileComment, getFileThumb, getFileInfo,
         setFileRotate, getFileHasTags, setFileFavorite, getTagsForFile, searchSimilarImages, generateEmbedding, 
         revealFolder, getTagName, indexAlbum, listenIndexProgress, listenIndexFinished, setAlbumCover,
         updateFileInfo, cancelIndexing as cancelIndexingApi, getFacesForFile, listenFaceIndexProgress } from '@/common/api';  
import { config, libConfig } from '@/common/config';
import { isWin, isMac, setTheme,
         formatFileSize, formatDate, getCalendarDateRange, getRelativePath, 
         extractFileName, combineFileName, getFolderPath, getFolderName, getSelectOptions, 
         shortenFilename, getSlideShowInterval } from '@/common/utils';

import DropDownSelect from '@/components/DropDownSelect.vue';
import ContextMenu from '@/components/ContextMenu.vue';
import ProgressBar from '@/components/ProgressBar.vue';
import GridView  from '@/components/GridView.vue';
import MediaViewer from '@/components/MediaViewer.vue';
import MessageBox from '@/components/MessageBox.vue';
import MoveTo from '@/components/MoveTo.vue';
import ToolTip from '@/components/ToolTip.vue';
import TButton from '@/components/TButton.vue';
import TaggingDialog from '@/components/TaggingDialog.vue';
import ImageEditor from '@/components/ImageEditor.vue';
import FileInfo from '@/components/FileInfo.vue';
import ScrollBar from '@/components/ScrollBar.vue';
import SliderInput from '@/components/SliderInput.vue';
import StatusBar from '@/components/StatusBar.vue';

import {
  IconPhotoAll,
  IconSideBarOn,
  IconSideBarOff,
  IconArrowDown,
  IconClose,
  IconCheckAll,
  IconCheckNone,
  IconFavorite,
  IconUnFavorite,
  IconFolderFavorite,
  IconMoveTo,
  IconFiles,
  IconFolderExpanded,
  IconChecked,
  IconTag,
  IconLocation,
  IconCameraAperture,
  IconTrash,
  IconLeft,
  IconRight,
  IconSeparator,
  IconUpdate,
  IconCopyTo,
  IconCard,
  IconTile,
  IconJustified,
  IconFilmstrip,
  IconRestore,
  IconRefresh,
  IconPhotoSearch,
  IconPersonSearch,
  IconFolderSearch,
  IconCalendarMonth,
  IconCalendarDay,
} from '@/common/icons';

const thumbnailPlaceholder = new URL('@/assets/images/image-file.png', import.meta.url).href;

const props = defineProps({
  titlebar: String
});

/// i18n
const { locale, messages } = useI18n();
const localeMsg = computed(() => messages.value[locale.value] as any);
const uiStore = useUIStore();

const contentTitle = ref("");

// album's folder mode
const showFolderFiles = computed(() => 
  Boolean(config.main.sidebarIndex === 0 && libConfig.album.id && libConfig.album.id > 0 && !libConfig.album.selected)
);

// progress bar
const thumbCount = ref(0);      // thumbnail count (from 0 to fileList.length)
const showProgressBar = ref(false); // show progress bar

// div elements
const contentViewDiv = ref<HTMLDivElement | null>(null);
const gridViewDiv = ref<HTMLDivElement | null>(null);

// file list
const fileList = ref<any[]>([]);
const totalFileCount = ref(0);    // total files' count
const totalFileSize = ref(0);     // total files' size

const selectedItemIndex = ref(-1);

// mutil select mode
const selectMode = ref(false);
const selectedCount = ref(0);
const selectedSize = ref(0);  // selected files size
const selectedFiles = computed(() => selectMode.value ? fileList.value.filter(f => f.isSelected) : []);

// quick view
const showQuickView = ref(false);
const quickViewMediaRef = ref<any>(null);
const quickViewZoomFit = ref(true);

// film strip view
const filmStripMediaRef = ref<any>(null);
const filmStripZoomFit = ref(true);

// toolbar state for MediaViewer
const imageScale = ref(1);
const imageMinScale = ref(0);
const imageMaxScale = ref(10);
const isSlideShow = ref(false);

// Request ID tracking to prevent race conditions during async content updates
let currentContentRequestId = 0;

const onScale = (event: any) => {
  imageScale.value = event.scale;
  imageMinScale.value = event.minScale;
  imageMaxScale.value = event.maxScale;
};

const videoRef = ref<HTMLVideoElement | null>(null);             // preview video reference

// info panel splitter
const isDraggingInfoPanel = ref(false);

// message box
const showRenameMsgbox = ref(false);  // show rename message box
const renamingFileName = ref<{name?: string, ext?: string}>({}); // extract the file name to {name, ext}

const showMoveTo = ref(false);
const showImageEditor = ref(false);
const showCopyTo = ref(false);
const showTrashMsgbox = ref(false);
const showCommentMsgbox = ref(false);
const errorMessage = ref('');

// Unsaved changes confirmation
const showUnsavedChangesMsgbox = ref(false);
const pendingAction = ref<(() => void) | null>(null);
const fileInfoRef = ref<any>(null);

const confirmSave = async () => {
  showUnsavedChangesMsgbox.value = false;
  if (fileInfoRef.value) {
    const success = await fileInfoRef.value.quickSave();
    if (success && pendingAction.value) {
      pendingAction.value();
      pendingAction.value = null;
    }
  }
};

const confirmDiscard = () => {
  showUnsavedChangesMsgbox.value = false;
  // Clear active adjustments in store to "discard" the changes
  uiStore.clearActiveAdjustments();
  if (pendingAction.value) {
    pendingAction.value();
    pendingAction.value = null;
  }
};

const cancelDiscard = () => {
  showUnsavedChangesMsgbox.value = false;
  pendingAction.value = null;
};

// Check if current file has unsaved changes
const hasUnsavedChanges = computed(() => {
  if (!config.infoPanel.show) return false;
  const currentFile = fileList.value[selectedItemIndex.value];
  return uiStore.hasActiveChanges(currentFile);
});

const checkUnsavedChanges = (action: () => void) => {
  if (hasUnsavedChanges.value) {
    pendingAction.value = action;
    showUnsavedChangesMsgbox.value = true;
  } else {
    action();
  }
};

// tagging dialog
const showTaggingDialog = ref(false);
const fileIdsToTag = ref<number[]>([]);

// grid view
const gridScrollContainerRef = ref<HTMLElement | null>(null);
const gridViewRef = ref<any>(null);

const scrollPosition = ref(0);    // scrollbar position (item index)

const containerHeight = ref(0);   // container height
const containerWidth = ref(0);    // container width
const layoutContentHeight = ref(0); // reported content height from GridView
const layoutVersion = ref(0);     // version to force layout update
const gap = 8;                    // Gap between items (must match GridView)

const itemWidth = computed(() => {
  if (config.settings.grid.style === 0) {
    return config.settings.grid.size + 20; // size + padding/border/gap(20)
  } else if (config.settings.grid.style === 1) {
    return config.settings.grid.size;
  } else if (config.settings.grid.style === 2) {
    return config.settings.grid.size; // Approximation for Justified View
  }
  return 0;
});

const itemSize = computed(() => {
  if (config.settings.grid.style === 0) {
    let labelHeight = 0
    if (config.settings.grid.labelPrimary > 0 ) labelHeight += 16;      // height of text-sm
    if (config.settings.grid.labelSecondary > 0 ) labelHeight += 16;    // height of text-xs
    return config.settings.grid.size + 20 + labelHeight; // size + padding/border/gap(20) + labels
  } else if (config.settings.grid.style === 1) {
    return itemWidth.value + gap / 2;
  } else if (config.settings.grid.style === 2) {
    return config.settings.grid.size;
  }
  return 0;
});

const columnCount = computed(() => {
  if (containerWidth.value <= 0 || itemWidth.value <= 0) return 4;
  return Math.max(1, Math.floor(containerWidth.value / itemWidth.value));
});

const visibleItemCount = computed(() => {
  if (containerHeight.value <= 0 || itemSize.value <= 0) return 20;
  const rows = Math.floor(containerHeight.value / itemSize.value);
  return rows * columnCount.value;
});

const timelineData = ref<any[]>([]);  // timeline markers for scrollbar

const toolTipRef = ref<any>(null);
const isProcessing = ref(false);  // show processing status
const isLoading = ref(false);     // show loading status in GridView (for empty file list)

const searchBoxRef = ref<any>(null);

// Store current query params for virtual scrolling
const currentQueryParams = ref({
  searchFileType: 0,
  sortType: 0,
  sortOrder: 0,
  searchFileName: "",
  searchAllSubfolders: "",
  searchFolder: "",
  startDate: 0,
  endDate: 0,
  make: "",
  model: "",
  locationAdmin1: "",
  locationName: "",
  isFavorite: false,
  tagId: 0,
  personId: 0,
});

// ai image search params
const currentImageSearchParams = ref({
  searchText: "",
  fileId: 0,
  threshold: 0,
  limit: 0,
});

// Similar Search Mode State
const tempViewMode = ref<'none' | 'similar' | 'album' | 'person'>('none');
const currentTitleIcon = computed(() => {
  switch (tempViewMode.value) {
    case 'none':
      if (contentTitle.value) {
        switch (config.main.sidebarIndex) {
          case 0: 
            switch (libConfig.album.id) {
              case 0: return IconPhotoAll;
              default: return libConfig.album.selected ? IconPhotoAll : IconFolderExpanded;
            }
          case 1: 
            switch (libConfig.favorite.folderId) {
              case 0: return IconFavorite;
              case 1: return IconFolderFavorite;
              default: return IconFolderFavorite;
            }
          case 2: return config.calendar.isMonthly ? IconCalendarMonth : IconCalendarDay;
          case 3: return IconPhotoSearch;
          case 4: return IconPersonSearch;
          case 5: return IconTag;
          case 6: return IconLocation;
          case 7: return IconCameraAperture;
          default: return IconFiles;
        }
      }
      return null;
    case 'similar': return IconPhotoSearch;
    case 'album': return IconFolderSearch;
    case 'person': return IconPersonSearch;
    default: return null;
  }
});

const backupState = ref<any>(null);

let unlistenKeydown: () => void;
let unlistenImageViewer: () => void;
let unlistenFaceIndexProgress: (() => void) | null = null;

// more menuitems
const moreMenuItems = computed(() => {
  const baseItems = [
    {
      label: localeMsg.value.menu.select.all,
      icon: IconCheckAll,
      action: () => {
        for (let i = 0; i < fileList.value.length; i++) {
          fileList.value[i].isSelected = true;
        }
        selectMode.value = true;
      }
    },
    {
      label: localeMsg.value.menu.select.none,
      icon: IconCheckNone,
      action: () => {
        for (let i = 0; i < fileList.value.length; i++) {
          fileList.value[i].isSelected = false;
        }
        selectMode.value = true;
      }
    },
    {
      label: localeMsg.value.menu.select.invert,
      action: () => {
        for (let i = 0; i < fileList.value.length; i++) {
          fileList.value[i].isSelected = !fileList.value[i].isSelected;
        }
        selectMode.value = true;
      }
    },
    {
      label: "-",   // separator
      action: null
    }
  ];

  const fileOperationItems = [
    {
      label: localeMsg.value.menu.file.move_to,
      icon: IconMoveTo,
      disabled: selectedCount.value === 0,
      action: () => {
        showMoveTo.value = true;
      }
    },
    {
      label: localeMsg.value.menu.file.copy_to,
      disabled: selectedCount.value === 0,
      icon: IconCopyTo,
      action: () => {
        showCopyTo.value = true;
      }
    },
    {
      label: isMac ? localeMsg.value.menu.file.move_to_trash : localeMsg.value.menu.file.delete,
      icon: IconTrash,
      disabled: selectedCount.value === 0,
      action: () => {
        showTrashMsgbox.value = true;
      }
    },
  ];

  const metadataItems = [
    { 
      label: "-",   // separator
      action: null
    },
    {
      label: localeMsg.value.menu.meta.favorite,
      icon: IconFavorite,
      disabled: selectedCount.value === 0,
      action: () => {
        selectModeSetFavorites(true);
      }
    },
    {
      label: localeMsg.value.menu.meta.unfavorite,
      icon: IconUnFavorite,
      disabled: selectedCount.value === 0,
      action: () => {
        selectModeSetFavorites(false);
      }
    },
    {
      label: localeMsg.value.menu.meta.tag,
      icon: IconTag,
      disabled: selectedCount.value === 0,
      action: () => {
        clickTag();
      }
    }
  ];

  let items = [...baseItems];
  items.push(...fileOperationItems);
  items.push(...metadataItems);

  return items;
});

let resizeObserver: ResizeObserver | null = null;

onMounted(() => {
  if (gridScrollContainerRef.value) {
    resizeObserver = new ResizeObserver((entries) => {
      for (const entry of entries) {
        containerHeight.value = entry.contentRect.height;
        containerWidth.value = entry.contentRect.width;
      }
    });
    resizeObserver.observe(gridScrollContainerRef.value);
    
    // Initial values
    containerHeight.value = gridScrollContainerRef.value.clientHeight;
    containerWidth.value = gridScrollContainerRef.value.clientWidth;
  }
});

onBeforeUnmount(() => {
  stopSlideShow();
  if (resizeObserver) {
    resizeObserver.disconnect();
  }
  if (unlistenKeydown) unlistenKeydown();
  if (unlistenImageViewer) unlistenImageViewer();
});

// New event handlers for GridView
function handleItemClicked(index: number, shiftKey: boolean = false) {
  if (index === selectedItemIndex.value) {
    if (selectMode.value) {
      handleItemSelectToggled(index, shiftKey);
    }
    return;
  }
  
  checkUnsavedChanges(() => {
    selectedItemIndex.value = index;
    if (selectMode.value) {
      handleItemSelectToggled(index, shiftKey);
    }
  });
}

// Double click grid view item
function handleItemDblClicked(index: number) {
  if (index === selectedItemIndex.value) {
    if (!config.settings.grid.showFilmStrip) {
      // quick view
      showQuickView.value = true;
      quickViewZoomFit.value = true;
    }
    return;
  }
  
  checkUnsavedChanges(() => {
    selectedItemIndex.value = index;

    if (!config.settings.grid.showFilmStrip) {
      // quick view
      showQuickView.value = true;
      quickViewZoomFit.value = true;
    }
  });
}

// Track last selected index for shift-click range selection
const lastSelectedIndex = ref(-1);

function handleItemSelectToggled(index: number, shiftKey: boolean = false) {
  if (shiftKey && lastSelectedIndex.value !== -1 && lastSelectedIndex.value !== index) {
    // Range selection: select all items between lastSelectedIndex and index
    const start = Math.min(lastSelectedIndex.value, index);
    const end = Math.max(lastSelectedIndex.value, index);
    
    // Set all items in range to the same selection state as the target item
    const targetState = !fileList.value[index].isSelected;
    for (let i = start; i <= end; i++) {
      fileList.value[i].isSelected = targetState;
    }
  } else {
    // Single toggle
    fileList.value[index].isSelected = !fileList.value[index].isSelected;
  }
  
  // Update last selected index
  lastSelectedIndex.value = index;
}

function clickRename() {
  renamingFileName.value = extractFileName(fileList.value[selectedItemIndex.value].name);
  showRenameMsgbox.value = true;
}

async function clickSetAlbumCover() {
  const file = fileList.value[selectedItemIndex.value];
  const albumId = libConfig.album.id || file?.album_id;
  
  if (file && albumId) {
    try {
      await setAlbumCover(albumId, file.id);
      await tauriEmit('album-cover-changed', { albumId: albumId, fileId: file.id });
      toolTipRef.value.showTip(localeMsg.value.tooltip.set_album_cover.success);
    } catch (error) {
      toolTipRef.value.showTip(localeMsg.value.tooltip.set_album_cover.failed, true);
    }
  }
}

function handleItemAction(payload: { action: string, index: number }) {
  if (isSlideShow.value) return;

  const { action, index } = payload;
  selectedItemIndex.value = index; // Ensure the item for the action is selected

  const actionMap = {
    'open': () => openImageViewer(selectedItemIndex.value, true),
    'edit': () => showImageEditor.value = true,
    'copy': () => clickCopyImage(fileList.value[selectedItemIndex.value].file_path),
    'rename': clickRename,
    'move-to': () => showMoveTo.value = true,
    'copy-to': () => showCopyTo.value = true,
    'trash': () => showTrashMsgbox.value = true,
    'album-folder': () => {
      const selectedFile = fileList.value[selectedItemIndex.value];
      if (selectedFile) {
        enterAlbumPreviewMode(selectedFile);
      }
    },
    'reveal': () => revealFolder(getFolderPath(fileList.value[selectedItemIndex.value].file_path)),
    'favorite': toggleFavorite,
    'rotate': clickRotate,
    'tag': clickTag,
    'comment': () => showCommentMsgbox.value = true,
    'search-similar': () => enterSimilarSearchMode(fileList.value[selectedItemIndex.value]),
    'find-person': () => enterPersonSearchMode(fileList.value[selectedItemIndex.value]),
    'set-album-cover': clickSetAlbumCover,
  };

  if ((actionMap as any)[action]) {
    // Check for unsaved changes before performing action, especially if it might change the context
    // Most actions here operate on `index` which becomes the selected index. 
    // If index is different from current selectedItemIndex, we should check.
    
    if (index !== selectedItemIndex.value) {
        checkUnsavedChanges(() => {
             (actionMap as any)[action]();
        });
    } else {
         (actionMap as any)[action]();
    }
  }
}

function requestNavigate(direction: 'prev' | 'next') {
  checkUnsavedChanges(() => {
    const viewer = showQuickView.value ? quickViewMediaRef.value : (config.settings.grid.showFilmStrip ? filmStripMediaRef.value : null);
    
    if (direction === 'next') {
      if (viewer) {
        viewer.triggerNext();
      } else {
        performNavigate('next');
      }
    } else {
      if (viewer) {
        viewer.triggerPrev();
      } else {
        performNavigate('prev');
      }
    }
  });
}

function performNavigate(direction: 'prev' | 'next') {
  checkUnsavedChanges(() => {
    if (direction === 'next') {
      if (selectedItemIndex.value < fileList.value.length - 1) {
        selectedItemIndex.value += 1;
      }
    } else {
      if (selectedItemIndex.value > 0) {
        selectedItemIndex.value -= 1;
      }
    }
  });
}

function updateScrollPosition(currentScrollTop: number, currentScrollHeight: number) {
    if (!config.settings.grid.showFilmStrip) {
      // Calculate max scroll top
      const totalRows = Math.ceil(totalFileCount.value / columnCount.value);
      const topPadding = 48;
      const bottomPadding = config.settings.showStatusBar ? 32 : 4;
      
      // Determine effective scroll height
      // If provided (from event), use it. Otherwise calculate based on layout.
      let scrollHeight = currentScrollHeight;
      if (!scrollHeight) {
         const contentHeight = (config.settings.grid.style === 2 && layoutContentHeight.value > 0) 
            ? layoutContentHeight.value 
            : (totalRows * itemSize.value);
         scrollHeight = contentHeight + topPadding + bottomPadding;
      }

      const maxScrollTop = Math.max(0, scrollHeight - containerHeight.value);
      
      if (maxScrollTop <= 0) {
        scrollPosition.value = 0;
      } else {
        const ratio = Math.min(1, Math.max(0, currentScrollTop / maxScrollTop));
        const maxIndex = Math.max(1, totalFileCount.value - visibleItemCount.value);
        scrollPosition.value = Math.round(ratio * maxIndex);
      }
    } else if (config.settings.grid.showFilmStrip) {
      // Fallback for filmstrip or other layouts (horizontal)
      const rowIndex = Math.floor(currentScrollTop / itemSize.value);
      scrollPosition.value = rowIndex * columnCount.value;
    }
}

function handleGridScroll(event: any) {
  if (event && event.target) {
    updateScrollPosition(
        event.target.scrollTop, 
        event.target.scrollHeight
    );
  }
}

function handleLayoutUpdate({ height }: { height: number }) {
  layoutContentHeight.value = height;
  if (gridViewRef.value) {
    updateScrollPosition(gridViewRef.value.getScrollTop(), 0);
  }
}

function handleScrollUpdate(newIndex: number) {
  scrollPosition.value = newIndex;
  
  if (!config.settings.grid.showFilmStrip && gridViewRef.value) {
    // Calculate ratio (0 to 1)
    const maxIndex = Math.max(1, totalFileCount.value - visibleItemCount.value);
    const ratio = Math.min(1, Math.max(0, newIndex / maxIndex));
    
    // Calculate max scroll top
    const totalRows = Math.ceil(totalFileCount.value / columnCount.value);
    const topPadding = 48;
    const bottomPadding = config.settings.showStatusBar ? 32 : 4;
    
    // Use reported layout height if available (style 2), otherwise calculate
    const contentHeight = (config.settings.grid.style === 2 && layoutContentHeight.value > 0) 
      ? layoutContentHeight.value 
      : (totalRows * itemSize.value);
      
    const totalHeight = contentHeight + topPadding + bottomPadding;
    const maxScrollTop = Math.max(0, totalHeight - containerHeight.value);
    
    const newScrollTop = ratio * maxScrollTop;
    
    gridViewRef.value.scrollToPosition(newScrollTop);
  } else if (gridScrollContainerRef.value) {
    // For filmstrip or other layouts, fallback to simple calculation or existing logic
    // But filmstrip uses horizontal scroll, handled differently.
    // This block might be for fallback.
    const rowIndex = Math.floor(newIndex / columnCount.value);
    const newScrollTop = rowIndex * itemSize.value;
    gridScrollContainerRef.value.scrollTop = newScrollTop;
  }
}

// Keyboard navigation actions
const keyActions = {
  ArrowDown: () => {
    checkUnsavedChanges(() => {
      if (gridViewRef.value) {
        if (config.settings.grid.style === 2) {
          // Use geometry-aware navigation for Justified View
          const nextIndex = gridViewRef.value.getNextItemIndex(selectedItemIndex.value, 'down');
          selectedItemIndex.value = nextIndex !== -1 ? nextIndex : selectedItemIndex.value;
        } else {
          selectedItemIndex.value = Math.min(selectedItemIndex.value + gridViewRef.value.getColumnCount(), fileList.value.length - 1);
        }
      }
    });
  },
  ArrowUp: () => {
    checkUnsavedChanges(() => {
      if (gridViewRef.value) {
        if (config.settings.grid.style === 2) {
          // Use geometry-aware navigation for Justified View
          const nextIndex = gridViewRef.value.getNextItemIndex(selectedItemIndex.value, 'up');
          selectedItemIndex.value = nextIndex !== -1 ? nextIndex : selectedItemIndex.value;
        } else {
          selectedItemIndex.value = Math.max(selectedItemIndex.value - gridViewRef.value.getColumnCount(), 0);
        }
      }
    });
  },
  Home: () => {
    checkUnsavedChanges(() => {
      selectedItemIndex.value = 0;
    });
  },
  End: () => {
    checkUnsavedChanges(() => {
      selectedItemIndex.value = fileList.value.length - 1;
    });
  },
  '/': () => searchBoxRef.value.focusInput(),
};

// Local keydown handler for navigation (prevents default browser behavior)
function handleLocalKeyDown(event: KeyboardEvent) {
  // Check for input targets (prevent toggle while typing)
  const target = event.target as HTMLElement;
  if (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA' || target.isContentEditable) {
    return;
  }

  // Check if there are modal dialogs
  if (uiStore.inputStack.length > 0) {
    return;
  }

  if (event.key === 'Escape') {
    if (uiStore.isFullScreen) {
      uiStore.isFullScreen = false;
      event.preventDefault();
      return;
    } else if (showQuickView.value) {
      showQuickView.value = false;
      stopSlideShow();
      event.preventDefault();
      return;
    } else if (selectMode.value) {
      handleSelectMode(false);
      event.preventDefault();
      return;
    } else if (tempViewMode.value !== 'none') {
      exitTempViewMode();
      event.preventDefault();
      return;
    }
  }

  if (selectedItemIndex.value < 0 || fileList.value.length === 0) {
    return;
  }

  if (isIndexing.value) {
    return;
  }

  // Disable keyboard events during slideshow (except Escape)
  if (isSlideShow.value && event.key !== 'Escape') {
    return;
  }

  const handledKeys = ['ArrowUp', 'ArrowDown', 'ArrowLeft', 'ArrowRight', 'Home', 'End', 'Enter', 'Space', ' '];

  if (event.key === 'Enter' && !event.metaKey && !event.ctrlKey) {
    if (!showQuickView.value && !config.settings.grid.showFilmStrip) {
      showQuickView.value = true;
      quickViewZoomFit.value = true;
    }
  } 
  else if (event.key === 'Space' || event.key === ' ') {
    if (showQuickView.value) {
      quickViewZoomFit.value = !quickViewZoomFit.value;
    } else if (!config.settings.grid.showFilmStrip) {
      showQuickView.value = true;
      quickViewZoomFit.value = true;
    }
  }

  if (handledKeys.includes(event.key)) {
    event.preventDefault();
  }

  if (event.key === 'ArrowRight') {
    requestNavigate('next');
  } else if (event.key === 'ArrowLeft') {
    requestNavigate('prev');
  }
}

// Global keydown handler (from Tauri)
const handleKeyDown = (e: any) => {
  if (uiStore.inputStack.length > 0) {
    return;
  }

  const { key, metaKey } = e.payload;

  // Disable global shortcuts during slideshow (except Escape for safety, though handled in local)
  if (isSlideShow.value && key !== 'Escape') {
    return;
  }

  const isCmdKey = isMac ? metaKey : e.payload.ctrlKey;

  if (isCmdKey && key === 'Enter') {   // Open shortcut
    openImageViewer(selectedItemIndex.value, true);
  } else if (isCmdKey && key.toLowerCase() === 'c') {   // Copy shortcut
    clickCopyImage(fileList.value[selectedItemIndex.value].file_path);
  } else if (isCmdKey && key.toLowerCase() === 's') {
    enterSimilarSearchMode(fileList.value[selectedItemIndex.value]);
  } else if (isCmdKey && key.toLowerCase() === 'e') {
    showImageEditor.value = true;
  } else if (isCmdKey && key.toLowerCase() === 'f') {
    toggleFavorite();
  } else if (isCmdKey && key.toLowerCase() === 't') {
    clickTag();
  } else if (isCmdKey && key.toLowerCase() === 'r') {
    clickRename();
  } else if ((isMac && metaKey && key === 'Backspace') || (!isMac && key === 'Delete')) {
    showTrashMsgbox.value = true;
  } else if ((keyActions as any)[key]) {
    (keyActions as any)[key]();
  }
};

// --- Indexing Logic ---
let unlistenIndexProgress: (() => void) | undefined;
let unlistenIndexFinished: (() => void) | undefined;
let unlistenTriggerNextAlbum: (() => void) | undefined;
let unlistenRefreshContent: (() => void) | undefined;

async function processNextAlbum() {
  if (libConfig.index.albumQueue.length > 0) {
    const albumId = libConfig.index.albumQueue[0];
    const album = await getAlbum(albumId);
    if (album) {
      libConfig.index.albumName = album.name;
      libConfig.index.indexed = 0;
      libConfig.index.total = 0;
      await indexAlbum(albumId);
    } else {
      // album not found (maybe deleted), remove from queue and process next
      libConfig.index.albumQueue.shift();
      processNextAlbum();
    }
  } else {
    libConfig.index.status = 0;
  }
}

// Check if current album is being indexed
const isIndexing = computed(() => {
  return config.main.sidebarIndex === 0 && // Album mode
         !!libConfig.album.id && libConfig.album.id > 0 && // Valid album
         libConfig.index.albumQueue.includes(libConfig.album.id);
});

watch(isIndexing, (val) => {
  // Always remove first to clear any stale entries
  uiStore.removeInputHandler('indexing');
  
  if (val) {
    showQuickView.value = false;
    // Push fresh handler when indexing starts
    uiStore.pushInputHandler('indexing');
  } else {
    updateContent();
  }
});

// Cancel indexing for current album
async function cancelIndexing() {
  const albumId = libConfig.album.id;
  const index = libConfig.index.albumQueue.indexOf(albumId);
  if (index === -1) return;

  if (index === 0) {
    // Active one: remove and restart processing for next
    libConfig.index.albumQueue.shift();
    libConfig.index.indexed = 0;
    libConfig.index.total = 0;
    
    // reset status
    libConfig.index.status = 0;

    // Call backend to cancel
    cancelIndexingApi(albumId);
    
    // trigger processing next
    // Do not call immediately to avoid parallel indexing
    setTimeout(() => {
      processNextAlbum();
    }, 1000);

  } else {
    // Pending one: just remove
    libConfig.index.albumQueue.splice(index, 1);
  }
}

onMounted( async() => {
  window.addEventListener('keydown', handleLocalKeyDown);
  unlistenKeydown = await listen('global-keydown', handleKeyDown);

  unlistenImageViewer = await listen('message-from-image-viewer', async (event) => {
    const { message } = event.payload as any;
    switch (message) {
      case 'request-file-at-index':
        const requestIndex = (event.payload as any).index;
        const pane = (event.payload as any).pane === 'right' ? 'right' : 'left';
        const file = fileList.value[requestIndex];
        if (file) {
           const imageWindow = await WebviewWindow.getByLabel('imageviewer');
           if (imageWindow) {
             imageWindow.emit('update-img', {
               fileId: file.id,
               fileIndex: requestIndex,
               fileCount: fileList.value.length,
               pane,
             });
           }
        }
        break;
      default:
        break;
    }
  });

  // Indexing listeners
  unlistenIndexProgress = await listenIndexProgress(async (event: any) => {
    const { album_id, current, total } = event.payload;
    if (libConfig.index.albumQueue.length > 0 && libConfig.index.albumQueue[0] === album_id) {
        libConfig.index.indexed = current;
        libConfig.index.total = total;
    }
  });

  unlistenIndexFinished = await listenIndexFinished(async (event: any) => {
    const { album_id } = event.payload;
    // notify album list to update cover
    await tauriEmit('album-cover-changed', { albumId: album_id, fileId: null });
    if (libConfig.index.albumQueue.length > 0 && libConfig.index.albumQueue[0] === album_id) {
      libConfig.index.albumQueue.shift();
      if (libConfig.index.albumQueue.length > 0) {
        processNextAlbum();
      } else {
        libConfig.index.status = 0;
      }
    }
  });

  // listen for external refresh requests (e.g. from folder context menu)
  unlistenRefreshContent = await listen('refresh-content', () => {
    updateContent();
  });

  unlistenTriggerNextAlbum = await listen('trigger-next-album', () => {
      processNextAlbum();
  });

  if (libConfig.index.albumQueue.length > 0) {
     if (libConfig.index.status === 1) {
        processNextAlbum();
     } else {
        libConfig.index.status = 1;
     }
  }

  // Face Indexing listeners
  unlistenFaceIndexProgress = await listenFaceIndexProgress((event: any) => {
    // Clear file list if in Person view (sidebarIndex === 4) and file list is not empty
    if (config.main.sidebarIndex === 4 && fileList.value.length > 0) {
      fileList.value = [];
      totalFileCount.value = 0;
      totalFileSize.value = 0;
      scrollPosition.value = 0;
      selectedItemIndex.value = -1;
    }
  });
});

onBeforeUnmount(() => {
  window.removeEventListener('keydown', handleLocalKeyDown);
  // unlisten
  unlistenImageViewer();
  if (unlistenKeydown) unlistenKeydown();
  if (unlistenTriggerNextAlbum) unlistenTriggerNextAlbum();
  if (unlistenIndexProgress) unlistenIndexProgress();
  if (unlistenIndexFinished) unlistenIndexFinished();
  if (unlistenRefreshContent) unlistenRefreshContent();
  if (unlistenFaceIndexProgress) unlistenFaceIndexProgress();
});

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

/// watch language
watch(() => config.settings.language, (newLanguage) => {
    locale.value = newLanguage; // update locale based on config.settings.language
    updateContent();
});

// Load tags when info panel is opened
watch(() => config.infoPanel.show, async (newShow) => {
  if (newShow && selectedItemIndex.value >= 0 && selectedItemIndex.value < fileList.value.length) {
    const file = fileList.value[selectedItemIndex.value];
    if (file.has_tags && !file.tags) {
      // Load tags if has_tags is true but tags not yet loaded
      fileList.value[selectedItemIndex.value] = {
        ...file,
        tags: await getTagsForFile(file.id)
      };
    }
  }
});

watch(() => libConfig.index.status, (newStatus) => {
  if (newStatus === 1 && libConfig.index.albumQueue.length > 0) {
    processNextAlbum();
  }
});

watch(() => libConfig.index.albumQueue.length, (newLength) => {
   if (newLength > 0 && libConfig.index.status === 0) {
       libConfig.index.status = 1; 
   }
});

/// watch image search params
watch(
  () => [
    libConfig.search.searchText, 
    libConfig.search.similarImageHistoryIndex, 
    config.search.searchType
  ],
  () => {
    setTimeout(() => {
      // Only update content if we are currently in the Image Search view (index 1)
      if (config.main.sidebarIndex === 3) {
        scrollPosition.value = 0;   // reset file scroll position
        selectedItemIndex.value = 0; // reset selected item index to 0
        
        // Also reset the GridView scroll position
        if (gridViewRef.value) {
          gridViewRef.value.scrollToPosition(0);
        }
        
        updateContent();
  
        // Reset ImageViewer context if open (without focusing/showing it)
        openImageViewer(selectedItemIndex.value, false, true);
      }
    }, 0);
  }
);

/// watch for file list changes
watch(
  () => [
    config.main.sidebarIndex,      // toolbar index
    libConfig.album.id, libConfig.album.folderId, libConfig.album.folderPath, libConfig.album.selected, // album
    libConfig.favorite.albumId, libConfig.favorite.folderId, libConfig.favorite.folderPath,   // favorite files and folder
    libConfig.search.fileName, config.search.fileType, config.search.sortType, config.search.sortOrder, // search and sort 
    libConfig.person.id,                                                              // person
    libConfig.calendar.year, libConfig.calendar.month, libConfig.calendar.date,       // calendar
    libConfig.tag.id,                                                                 // tag
    libConfig.location.admin1, libConfig.location.name,                               // location
    libConfig.camera.make, libConfig.camera.model,                                    // camera 
  ], 
  () => {
    // Clear active adjustments when the file list changes to avoid unnecessary confirmation dialogs
    uiStore.clearActiveAdjustments();

    // Skip if in temp view mode to prevent race conditions
    if (tempViewMode.value !== 'none') return;
    
    setTimeout(() => {
      // Double check in case tempViewMode changed during setTimeout
      if (tempViewMode.value !== 'none') return;
      
      scrollPosition.value = 0;   // reset file scroll position
      selectedItemIndex.value = 0; // reset selected item index to 0
      
      // Also reset the GridView scroll position
      if (gridViewRef.value) {
        gridViewRef.value.scrollToPosition(0);
      }
      
      updateContent();
  
      // Reset ImageViewer context if open (without focusing/showing it)
      openImageViewer(selectedItemIndex.value, false, true);
    }, 0);
  }, 
  { immediate: true }
);

// watch for selected item (not in select mode)
watch(() => selectedItemIndex.value, (newIndex, oldIndex) => {
  if(oldIndex >= 0 && oldIndex !== newIndex && fileList.value[oldIndex]?.rotate >= 360) {
    fileList.value[oldIndex].rotate %= 360;
  }
  updateSelectedImage(newIndex);
});

// watch for selected items in the file list (select mode)
watch(
  () => fileList.value.map(file => ({ isSelected: file.isSelected, size: file.size })),
  () => {
    selectedCount.value = fileList.value.filter(f => f.isSelected).length;
    selectedSize.value = fileList.value.reduce((total, f) => total + (f.isSelected ? f.size : 0), 0);
  }
);

// watch for show preview or layout change
watch(() => config.settings.grid.style, () => {
  updateSelectedImage(selectedItemIndex.value);
  stopSlideShow();
});

function toggleFilmstripView() {
  showQuickView.value = false;
  config.settings.grid.showFilmStrip = !config.settings.grid.showFilmStrip;
  if (config.settings.grid.showFilmStrip) {
    filmStripZoomFit.value = true;
  }
}

function cycleGridStyle() {
  showQuickView.value = false;
  // Cycle between 0, 1, 2 (Card, Tile, Justified)
  config.settings.grid.style = (config.settings.grid.style + 1) % 3;
}

// Track pending requests to avoid duplicates
const pendingRequests = new Set();

async function fetchDataRange(start: number, end: number) {
  // Clamp range
  start = Math.max(0, start);
  end = Math.min(totalFileCount.value, end);
  
  if (start >= end) return;

  // Fetch in chunks
  const chunkSize = 200;
  const startChunk = Math.floor(start / chunkSize);
  const endChunk = Math.floor((end - 1) / chunkSize);

  for (let i = startChunk; i <= endChunk; i++) {
    const chunkStart = i * chunkSize;
    
    // Check if we need to load this chunk
    // Optimization: Check the first item of the chunk.
    if (fileList.value[chunkStart] && fileList.value[chunkStart].isPlaceholder) {
      const key = `${chunkStart}-${chunkSize}`;
      if (pendingRequests.has(key)) {
        console.log(`Skipping pending chunk: ${key}`);
        continue;
      }
      
      pendingRequests.add(key);
      
      // We don't await here to allow parallel fetching of chunks, 
      // but we track pending requests to avoid duplicate fetches.
      getQueryFiles(currentQueryParams.value, chunkStart, chunkSize)
        .then(newFiles => {
          if (newFiles) {
            // Update fileList and collect reactive references
            const filesToFetch = [];
            for (let j = 0; j < newFiles.length; j++) {
              if (chunkStart + j < fileList.value.length) {
                const existingItem = fileList.value[chunkStart + j];
                // Preserve client-side state
                const isSelected = existingItem ? existingItem.isSelected : false;
                const rotate = existingItem ? (existingItem.rotate || 0) : 0;

                fileList.value[chunkStart + j] = { 
                  ...newFiles[j], 
                  isSelected,
                  rotate: rotate || newFiles[j].rotate || 0
                };
                filesToFetch.push(fileList.value[chunkStart + j]);

                // Update ImageViewer if the selected file is loaded
                if (chunkStart + j === selectedItemIndex.value) {
                  if (selectedItemIndex.value === 0) {
                    openImageViewer(selectedItemIndex.value, false, true);
                  } else {
                    openImageViewer(selectedItemIndex.value, false);
                  }
                  updateSelectedImage(selectedItemIndex.value);
                }
              }
            }
            // Fetch thumbnails for these files
            if (filesToFetch.length > 0) {
              getFileListThumb(filesToFetch);
            }
            // Trigger layout update since file dimensions might have changed
            layoutVersion.value++;
          }
        })
        .catch(err => {
            console.error(`Error fetching chunk ${key}:`, err);
        })
        .finally(() => {
          pendingRequests.delete(key);
        });
    } else {
        // console.log(`Chunk already loaded or invalid: ${chunkStart}`);
    }
  }
}

// Track last visible range to avoid redundant fetches
let lastVisibleRange = { start: -1, end: -1 };

function handleVisibleRangeUpdate({ startIndex, endIndex }: { startIndex: number, endIndex: number }) {
  // Skip if the range hasn't changed significantly
  if (lastVisibleRange.start === startIndex && lastVisibleRange.end === endIndex) {
    return;
  }
  
  lastVisibleRange = { start: startIndex, end: endIndex };
  
  // Fetch data for visible range + buffer
  const buffer = 200;
  fetchDataRange(startIndex - buffer, endIndex + buffer);
}

// get file list 
async function getFileList(
  {
    searchFileType = config.search.fileType, 
    sortType = config.search.sortType, 
    sortOrder = config.search.sortOrder, 
    searchFileName = '', 
    searchAllSubfolders = '',
    searchFolder = '', 
    startDate = 0, 
    endDate = 0, 
    make = '', 
    model = '', 
    locationAdmin1 = '', 
    locationName = '', 
    isFavorite = false, 
    tagId = 0,
    personId = 0
  } = {},
  requestId: number, 
) { 
  // Update current query params with all fields
  currentQueryParams.value = {
    searchFileType,
    sortType,
    sortOrder,
    searchFileName,
    searchAllSubfolders,
    searchFolder,
    startDate,
    endDate,
    make,
    model,
    locationAdmin1,
    locationName,
    isFavorite,
    tagId,
    personId,
  };

  // Set loading state
  isLoading.value = true;

  try {
    // Get count and sum first
    const result = await getQueryCountAndSum(currentQueryParams.value);
    
    // Check if the request is still valid. 
    if (requestId !== currentContentRequestId) {
      return;
    }

    if (result) {
      totalFileCount.value = result[0];
      totalFileSize.value = result[1];
      
      // Get timeline data for date-based sorts
      getQueryTimeLine(currentQueryParams.value).then(data => {
        if (requestId === currentContentRequestId) {
          timelineData.value = data;
        }
      });
      
      // Initialize fileList with placeholders
      fileList.value = Array.from({ length: totalFileCount.value }).map((_, i) => ({
        id: 'ph-' + i,
        isPlaceholder: true,
        name: '',
        size: 0,
      }));
      if (totalFileCount.value === 0) {
        openImageViewer(0, false, true);
      }
      
      // Reset visible range tracking when changing views
      lastVisibleRange = { start: -1, end: -1 };
    } else {
      fileList.value = [];
      openImageViewer(0, false, true);
    }
  } catch (err) {
    console.error('getFileList error:', err);
    if (requestId === currentContentRequestId) {
      fileList.value = [];
      openImageViewer(0, false, true);
    }
  } finally {
    // Only clear loading state if this is still the active request
    if (requestId === currentContentRequestId) {
      isLoading.value = false;
    }
  }
}

async function getImageSearchFileList(searchText: string, fileId: number, requestId: number) {
  currentImageSearchParams.value = {
    searchText,
    fileId,
    threshold: config.imageSearchThresholds[config.settings.imageSearch.thresholdIndex],
    limit: config.settings.imageSearch.limit,
  };

  // set loading state
  isLoading.value = true;
  timelineData.value = []; // reset timeline data

  try {
    // Check if the request is still valid. 
    if (requestId !== currentContentRequestId) {
      return;
    }
    
    searchSimilarImages(currentImageSearchParams.value).then(result => {
      if (result) {
        fileList.value = result;
        totalFileCount.value = fileList.value.length;
        totalFileSize.value = fileList.value.reduce((total, file) => total + file.size, 0);
        openImageViewer(0, false, true);

        // Reset visible range tracking when changing views
        lastVisibleRange = { start: -1, end: -1 };
        
        // Update search history with the first result's file_id
        if (searchText && result.length > 0) {
          const history = libConfig.search.searchHistory as any[];
          const index = history.findIndex((item: any) => {
             const text = typeof item === 'string' ? item : item.text;
             return text === searchText;
          });

          if (index !== -1) {
             const item = history[index];
             const firstId = result[0].id;

             // Always update the history item's fileId to the latest first result
             if (typeof item === 'string') {
               history[index] = { text: item, fileId: firstId };
             } else {
               item.fileId = firstId;
             }
          }
        }

        // Fetch thumbnails for the search results
        getFileListThumb(fileList.value);
      }
    });
  } catch (err) {
    console.error('getImageSearchFileList error:', err);
    if (requestId === currentContentRequestId) {
      fileList.value = [];
    }
  } finally {
    // Only clear loading state if this is still the active request
    if (requestId === currentContentRequestId) {
      isLoading.value = false;
    }
  }
}

async function updateContent() {
  // Reset temp view mode on any content update
  tempViewMode.value = 'none';
  showQuickView.value = false;
  isSlideShow.value = false;
  stopSlideShow();

  backupState.value = null;
  
  // Increment request ID to cancel any previous thumbnail generation and reset queue
  currentThumbRequestId++;
  thumbCount.value = 0;

  const requestId = ++currentContentRequestId;
  const newIndex = config.main.sidebarIndex;

  // Reset file list immediately to reflect UI change
  fileList.value = [];
  isLoading.value = true;

  if(newIndex === 0) {   // album  
    if(libConfig.album.id === null) {
      contentTitle.value = "";
    } else if(libConfig.album.id === 0) {   // all files
      contentTitle.value = localeMsg.value.album.all_files;
      getFileList({}, requestId);
    } else {
      getAlbum(libConfig.album.id).then(async album => {
        if (requestId !== currentContentRequestId) return;
        if(album) {
          if(libConfig.album.selected) {     
            // album is selected, show all files including subfolders
            contentTitle.value = album.name;
            getFileList({ searchAllSubfolders: libConfig.album.folderPath }, requestId);
          } else {                        
            // folder is selected, show files in the folder
          contentTitle.value = getFolderName(album.path) + getRelativePath(libConfig.album.folderPath || "", album.path);
            
            // Get files from file system (not from DB) and generate thumbnails if needed
            const [folderFiles, newCount, updatedCount] = await getFolderFiles(libConfig.album.folderId, libConfig.album.folderPath, false);
            if (requestId !== currentContentRequestId) return;
            
            console.log(`getFolderFiles: found ${newCount} new files and updated ${updatedCount} files.`);

            fileList.value = folderFiles || [];
            totalFileCount.value = fileList.value.length;
            totalFileSize.value = fileList.value.reduce((total, file) => total + file.size, 0);
            openImageViewer(0, false, true);

            // Fetch timeline data for the folder
            currentQueryParams.value = {
              searchFileType: config.search.fileType,
              sortType: config.search.sortType,
              sortOrder: config.search.sortOrder,
              searchFileName: "",
              searchAllSubfolders: "",
              searchFolder: libConfig.album.folderPath || "",
              startDate: 0,
              endDate: 0,
              make: "",
              model: "",
              locationAdmin1: "",
              locationName: "",
              isFavorite: false,
              tagId: 0,
              personId: 0,
            };
            getQueryTimeLine(currentQueryParams.value).then(data => {
              if (requestId === currentContentRequestId) timelineData.value = data;
            });

            // Get the thumbnail count to show progress bar if needed
            getFolderThumbCount(libConfig.album.folderId).then(count => {
              if (requestId === currentContentRequestId) {
                console.log('updateContent - thumbCount:', count);
                showProgressBar.value = count < fileList.value.length; 
              }
            });

            // Always get all thumbnails for a folder (generate if not exist)
            getFileListThumb(fileList.value);
          }
        } else {
          contentTitle.value = "";
        }
      });
    }
  }
  else if(newIndex === 1) {   // favorite
    if(libConfig.favorite.folderId === null) {
      contentTitle.value = "";
    } else {
      if(libConfig.favorite.folderId === 0) { // favorite files
        contentTitle.value = localeMsg.value.favorite.files;
        getFileList({ isFavorite: true }, requestId);
      } else {                // favorite folders
        getAlbum(libConfig.favorite.albumId).then(album => {
          if (requestId !== currentContentRequestId) return;
          if(album) {
            contentTitle.value = getFolderName(album.path) + getRelativePath(libConfig.favorite.folderPath || "", album.path);
            getFileList({ searchAllSubfolders: libConfig.favorite.folderPath || "" }, requestId);
          } else {
            contentTitle.value = "";
          }
        });
      }
    }
  }
  else if(newIndex === 2) {   // calendar
    if(libConfig.calendar.year === null) {
      contentTitle.value = "";
    } else if (libConfig.calendar.year === -1) {  // on this day
      contentTitle.value = localeMsg.value.calendar.on_this_day;
      getFileList({ startDate: -1, endDate: -1 }, requestId);
    } else {
      if (libConfig.calendar.month === -1) {          // yearly
        contentTitle.value = formatDate(libConfig.calendar.year!, 1, 1, localeMsg.value.format.year);
      } else if (libConfig.calendar.date === -1) {    // monthly
        contentTitle.value = formatDate(libConfig.calendar.year!, libConfig.calendar.month!, 1, localeMsg.value.format.month);
      } else {                                    // daily
        contentTitle.value = formatDate(libConfig.calendar.year!, libConfig.calendar.month!, libConfig.calendar.date!, localeMsg.value.format.date_long);
      }
      const [startDate, endDate] = getCalendarDateRange(libConfig.calendar.year!, libConfig.calendar.month!, libConfig.calendar.date!);
      getFileList({ startDate, endDate }, requestId);
    }
  }
  else if(newIndex === 3) {   // image search
    if(config.search.searchType === 0) {   // search
      if (libConfig.search.searchText) {
        contentTitle.value = localeMsg.value.search.search_images + ' - ' + libConfig.search.searchText;
        getImageSearchFileList(libConfig.search.searchText, 0, requestId);
      } else {
        contentTitle.value = localeMsg.value.search.search_images;
      }
    } else if (config.search.searchType === 1) { // similar
      const index = libConfig.search.similarImageHistoryIndex;
      if (index >= 0 && index < libConfig.search.similarImageHistory.length) {
        const file = await getFileInfo(libConfig.search.similarImageHistory[index]);
        contentTitle.value = localeMsg.value.search.similar_images + ' - ' + file.name;
        getImageSearchFileList("", libConfig.search.similarImageHistory[index], requestId);
      } else {
        contentTitle.value = localeMsg.value.search.similar_images;
      }
    } else {   // filename search
      if (libConfig.search.fileName) {
        contentTitle.value = localeMsg.value.search.filename_search + ' - ' + libConfig.search.fileName;
        getFileList({ searchFileName: libConfig.search.fileName, sortType: 1, sortOrder: 0 }, requestId); // sort by name
      } else {
        contentTitle.value = localeMsg.value.search.filename_search;
      }
    }
  } 
  else if(newIndex === 4) {   // person
    if (libConfig.person.id === null) {
      contentTitle.value = "";
    } else {
      contentTitle.value = libConfig.person.name || `${localeMsg.value.sidebar.person}`;
      getFileList({ personId: libConfig.person.id }, requestId);
    }
  }
  else if(newIndex === 5) {   // tag
    if (libConfig.tag.id === null) {
      contentTitle.value = "";
    } else {
      getTagName(libConfig.tag.id).then(tagName => {
        if (requestId !== currentContentRequestId) return;
        if (tagName) {
          contentTitle.value = tagName;
          getFileList({ tagId: libConfig.tag.id || 0 }, requestId);
        } else {
          contentTitle.value = "";
        }
      });
    }
  }
  else if(newIndex === 6) {   // location
    if(libConfig.location.admin1 === null) {
      contentTitle.value = "";
    } else {
      if(libConfig.location.name) {
        contentTitle.value = `${libConfig.location.admin1} > ${libConfig.location.name}`;
        getFileList({ locationAdmin1: libConfig.location.admin1, locationName: libConfig.location.name }, requestId);
      } else {
        contentTitle.value = `${libConfig.location.admin1}`;
        getFileList({ locationAdmin1: libConfig.location.admin1 }, requestId);
      } 
    }
  }
  else if(newIndex === 7) {   // camera
    if(libConfig.camera.make === null) {
      contentTitle.value = "";
    } else {
      if(libConfig.camera.model) {
        contentTitle.value = `${libConfig.camera.make} > ${libConfig.camera.model}`;
        getFileList({ make: libConfig.camera.make, model: libConfig.camera.model }, requestId);
      } else {
        contentTitle.value = `${libConfig.camera.make}`;
        getFileList({ make: libConfig.camera.make }, requestId);
      } 
    }
  } 

  if(fileList.value.length === 0) {
    isLoading.value = false;
  }
}

// --- Similar Search Mode Logic ---
function enterSimilarSearchMode(file: any) {
  if (file.file_type !== 1 && file.file_type !== 3) {
    return;
  }

  // Increment request ID to cancel any previous thumbnail generation and reset queue
  currentThumbRequestId++;
  thumbCount.value = 0;

  // 1. Backup current state
  if (tempViewMode.value === 'none') {
    backupState.value = {
      fileList: [...fileList.value],
      totalFileCount: totalFileCount.value,
      totalFileSize: totalFileSize.value,
      contentTitle: contentTitle.value,
      selectedItemIndex: selectedItemIndex.value,
      scrollPosition: scrollPosition.value,
      timelineData: [...timelineData.value],
      currentQueryParams: { ...currentQueryParams.value },
      thumbCount: thumbCount.value,
      showProgressBar: showProgressBar.value,
      
      // GridView specific backup
      scrollTop: gridViewRef.value ? gridViewRef.value.getScrollTop() : 0,
    };
  }

  // 2. Set mode
  tempViewMode.value = 'similar';
  showQuickView.value = false;
  
  // 3. Update Title to indicate context
  contentTitle.value = localeMsg.value.search.similar_images + ' - ' + file.name;

  // Update similar image search history
  const existingIndex = (libConfig.search.similarImageHistory as any[]).findIndex(item => item === file.id);
  
  if (existingIndex !== -1) {
    libConfig.search.similarImageHistoryIndex = existingIndex;
  } else {
    (libConfig.search.similarImageHistory as any[]).unshift(file.id);
    libConfig.search.similarImageHistoryIndex = 0;
    
    if (libConfig.search.similarImageHistory.length > config.search.maxSearchHistory) {
      (libConfig.search.similarImageHistory as any[]).pop();
    }
  }

  // 4. Perform Search (reusing existing API call logic)
  const requestId = ++currentContentRequestId;
  
  // Reset list for loading state
  fileList.value = [];
  totalFileCount.value = 0;
  totalFileSize.value = 0;
  
  // Reset scroll and selection
  scrollPosition.value = 0;
  selectedItemIndex.value = 0;
  if (gridViewRef.value) {
    gridViewRef.value.scrollToPosition(0);
  }
  
  getImageSearchFileList("", file.id, requestId);
}

// --- Person Search Mode Logic ---
async function enterPersonSearchMode(file: any) {
  if (!file || !file.id) {
    return;
  }

  // fetch faces
  const faces = await getFacesForFile(file.id);
  if (!faces || faces.length === 0) {
     toolTipRef.value?.showTip(localeMsg.value.tooltip.not_found.person || "No person found", false);
     return;
  }

  // Find first face with person_id
  const face = faces.find((f: any) => f.person_id && f.person_id > 0);
  if (!face) {
     toolTipRef.value?.showTip(localeMsg.value.tooltip.not_found.person || "No person found", false);
     return;
  }

  // Increment request ID to cancel any previous thumbnail generation and reset queue
  currentThumbRequestId++;
  thumbCount.value = 0;

  // 1. Backup current state
  if (tempViewMode.value === 'none') {
    backupState.value = {
      fileList: [...fileList.value],
      totalFileCount: totalFileCount.value,
      totalFileSize: totalFileSize.value,
      contentTitle: contentTitle.value,
      selectedItemIndex: selectedItemIndex.value,
      scrollPosition: scrollPosition.value,
      timelineData: [...timelineData.value],
      currentQueryParams: { ...currentQueryParams.value },
      thumbCount: thumbCount.value,
      showProgressBar: showProgressBar.value,
      scrollTop: gridViewRef.value ? gridViewRef.value.getScrollTop() : 0,
    };
  }

  // 2. Set mode
  tempViewMode.value = 'person';
  showQuickView.value = false;

  // 3. Update libConfig.person to reflect the found person
  libConfig.person.id = face.person_id;
  libConfig.person.name = face.person_name || null;

  // 4. Update Title to indicate context
  contentTitle.value = face.person_name || localeMsg.value.sidebar.person;

  // 5. Perform Search
  const requestId = ++currentContentRequestId;
  
  // Reset list for loading state
  fileList.value = [];
  totalFileCount.value = 0;
  totalFileSize.value = 0;
  
  // Reset scroll and selection
  scrollPosition.value = 0;
  selectedItemIndex.value = 0;
  if (gridViewRef.value) {
    gridViewRef.value.scrollToPosition(0);
  }
  
  getFileList({ personId: face.person_id }, requestId);
}

function enterAlbumPreviewMode(file: any) {
  if (!file.album_id) return;

  // 1. Backup current state
  if (tempViewMode.value === 'none') {
    backupState.value = {
      fileList: [...fileList.value],
      totalFileCount: totalFileCount.value,
      totalFileSize: totalFileSize.value,
      contentTitle: contentTitle.value,
      selectedItemIndex: selectedItemIndex.value,
      scrollPosition: scrollPosition.value,
      timelineData: [...timelineData.value],
      currentQueryParams: { ...currentQueryParams.value },
      thumbCount: thumbCount.value,
      showProgressBar: showProgressBar.value,
      
      // GridView specific backup
      scrollTop: gridViewRef.value ? gridViewRef.value.getScrollTop() : 0,
    };
  }
  
  // Increment request ID to cancel any previous thumbnail generation and reset queue
  currentThumbRequestId++;
  thumbCount.value = 0;

  // 2. Set mode
  tempViewMode.value = 'album';
  showQuickView.value = false;
  
  // 3. Update Title and Fetch Files
  const folderPath = getFolderPath(file.file_path);

  getAlbum(file.album_id).then((album: any) => {
    if (album) {
      if(folderPath === album.path) { // current folder is root
        contentTitle.value = album.name;
      } else {
        contentTitle.value = getFolderName(album.path) + getRelativePath(folderPath || "", album.path);
      };
    }
  });

  // 4. Fetch Files
  const requestId = ++currentContentRequestId;
  
  // Reset list for loading state
  fileList.value = [];
  totalFileCount.value = 0;
  totalFileSize.value = 0;
  
  // Reset scroll and selection
  scrollPosition.value = 0;
  selectedItemIndex.value = 0;
  if (gridViewRef.value) {
    gridViewRef.value.scrollToPosition(0);
  }
  
  getFileList({ searchFolder: folderPath }, requestId);
}

function exitTempViewMode() {
  if (!backupState.value) return;

  const state = backupState.value;
  
  // 1. Restore State
  fileList.value = state.fileList;
  totalFileCount.value = state.totalFileCount;
  totalFileSize.value = state.totalFileSize;
  contentTitle.value = state.contentTitle;
  selectedItemIndex.value = state.selectedItemIndex;
  scrollPosition.value = state.scrollPosition;
  timelineData.value = state.timelineData;
  currentQueryParams.value = state.currentQueryParams;
  thumbCount.value = state.thumbCount;
  showProgressBar.value = state.showProgressBar;

  // Increment request ID to cancel any previous thumbnail generation (from temp view)
  currentThumbRequestId++;

  // 2. Reset Mode
  tempViewMode.value = 'none';
  showQuickView.value = false;
  backupState.value = null;

  // 3. Restore Scroll Position (need to wait for Vue to re-render the list)
  // Using nextTick or a small timeout
  setTimeout(() => {
    if (gridViewRef.value) {
      gridViewRef.value.scrollToPosition(state.scrollTop);
    }
  }, 0);
  
  // 4. Ensure the originally selected item is highlighted/visible logic is handled by the restored index
  updateSelectedImage(selectedItemIndex.value);
}

function handleTitleClick() {
  switch (tempViewMode.value) {
    case 'similar':
      config.main.sidebarIndex = 3;   // search tab
      config.search.searchType = 1;   // similar image 
      break;
    case 'person':
      config.main.sidebarIndex = 4;   // person tab
      break;
    case 'album':
      config.main.sidebarIndex = 0;   // album tab

      // Get first file to extract album info
      const file = fileList.value[0];
      if (!file || !file.album_id || !file.folder_id) return;
      
      const folderPath = getFolderPath(file.file_path);
      
      // Set libConfig.album to select this folder in Album tab
      libConfig.album.id = file.album_id;
      libConfig.album.folderId = file.folder_id;
      libConfig.album.folderPath = folderPath;
      libConfig.album.selected = false;
      
      // Emit event to trigger album expansion in AlbumList
      tauriEmit('expand-album-folder', { albumId: file.album_id, folderPath: folderPath });
      break;
    default:
      break;
  }

  exitTempViewMode();
}

// update the file info from the image editor
const onImageEdited = (success: boolean) => {
  if (success) {
    showImageEditor.value = false;
    updateFile(fileList.value[selectedItemIndex.value]);
  } else {
    toolTipRef.value.showTip(localeMsg.value.tooltip.save_image.failed, true);
  }
}

const clickCopyImage = async (filePath: string) => {
  if (isProcessing.value) return;

  isProcessing.value = true;

  let success = false;
  try {
    success = await copyImage(filePath);
  } finally {
    isProcessing.value = false;
    if (success) {
      toolTipRef.value.showTip(localeMsg.value.tooltip.copy_image.success);
    } else {
      toolTipRef.value.showTip(localeMsg.value.tooltip.copy_image.failed, true);
    }
  }
}

const onRenameFile = async (newName: string) => {
  if(selectedItemIndex.value >= 0) {
    const file = fileList.value[selectedItemIndex.value];
    const fileName = combineFileName(newName, renamingFileName.value.ext ?? '');
    const newFilePath = await renameFile(file.id, file.file_path, fileName );
    if(newFilePath) {
      console.log('onRenameFile:', newFilePath);
      file.name = fileName;
      file.file_path = newFilePath;
      showRenameMsgbox.value = false;
      errorMessage.value = '';
    } else {
      errorMessage.value = localeMsg.value.msgbox.rename_file.error;
    }
  }
}

const onMoveTo = async () => {
  if (selectMode.value && selectedCount.value > 0) {    // multi-select mode
    const moves = fileList.value
      .filter(item => item.isSelected)
      .map(async item => {
        const movedFile = await moveFile(item.id, item.file_path, libConfig.destFolder.folderId, libConfig.destFolder.folderPath);
        if(movedFile) {
          console.log('onMoveTo:', movedFile);
          removeFromFileList(fileList.value.indexOf(item));
        }
      });
    await Promise.all(moves); // parallelize DB updates
    selectMode.value = false; // exit multi-select mode
  } 
  else if(selectedItemIndex.value >= 0) {               // single select mode
    const file = fileList.value[selectedItemIndex.value];
    const movedFile = await moveFile(file.id, file.file_path, libConfig.destFolder.folderId, libConfig.destFolder.folderPath);
    if(movedFile) {
      console.log('onMoveTo:', movedFile);
      removeFromFileList(selectedItemIndex.value);
    }
  }
  showMoveTo.value = false;
}

const onCopyTo = async () => {
  if (selectMode.value && selectedCount.value > 0) {    // multi-select mode
    const copies = fileList.value
      .filter(item => item.isSelected)
      .map(async item => {
        const copiedFile = await copyFile(item.file_path, libConfig.destFolder.folderPath);
        if(copiedFile) {
          console.log('onCopyTo:', copiedFile);
        }
      });
    await Promise.all(copies); // parallelize DB updates
    selectMode.value = false; // exit multi-select mode
  } 
  else if(selectedItemIndex.value >= 0) {               // single select mode
    const file = fileList.value[selectedItemIndex.value];
    const copiedFile = await copyFile(file.file_path, libConfig.destFolder.folderPath);
    if(copiedFile) {
      console.log('onCopyTo:', copiedFile);
    }
  }
  showCopyTo.value = false;
}

const onTrashFile = async () => {
  if (selectMode.value && selectedCount.value > 0) {     // multi-select mode
    const deletes = fileList.value
      .filter(item => item.isSelected)
      .map(async item => {
        await deleteFileAlways(item);
        removeFromFileList(fileList.value.indexOf(item));
      });
    await Promise.all(deletes); // parallelize DB updates
    selectMode.value = false; // exit multi-select mode
  } 
  else if(selectedItemIndex.value >= 0) {               // single select mode
    await deleteFileAlways(fileList.value[selectedItemIndex.value]);
    removeFromFileList(selectedItemIndex.value);
  }
  showTrashMsgbox.value = false;
  updateSelectedImage(selectedItemIndex.value);
}

// delete a file always (trash or delete from db)
async function deleteFileAlways(file: any) {
  const deletedFile = await deleteFile(file.id, file.file_path);
  if(deletedFile) {
    console.log('clickDeleteFile - trashed file:', file.file_path);
  } else {
    console.log('clickDeleteFile - delete db file:', file.file_path);
    await deleteDbFile(file.id);
  }
}

// remove an file item from the list and update the selected item index
function removeFromFileList(index: number = 0) {
  // remove file from list
  fileList.value.splice(index, 1);
  
  // update total file count and size
  totalFileCount.value = fileList.value.length;
  totalFileSize.value = fileList.value.reduce((total, file) => total + file.size, 0);
  
  // update selected item index (ensure it's always a valid number)
  if (fileList.value.length > 0) {
    selectedItemIndex.value = Math.min(index, fileList.value.length - 1);
  } else {
    selectedItemIndex.value = -1;
  }
}

// update the file info from the file
const updateFile = async (file: any) => {
  try {
    const updatedFile = await updateFileInfo(file.id, file.file_path);
    if (updatedFile) {
      Object.assign(file, updatedFile);
      await updateThumbForFile(file);
      await updateSelectedImage(selectedItemIndex.value);

      // Force Image.vue to reload the saved image by briefly nullifying file_path
      // to trigger its filePath watcher (since the path itself hasn't changed, only the version)
      const savedPath = file.file_path;
      file.file_path = '';
      await nextTick();
      file.file_path = savedPath;

      // Clear CSS filter adjustments after image reload is triggered
      uiStore.clearActiveAdjustments();
    }
  } catch (err) {
    console.error('Failed to update file info:', err);
  }
}

// force-update the thumbnail for the file
const updateThumbForFile = async (file: any) => {
  const thumb = await getFileThumb(file.id, file.file_path, file.file_type, file.e_orientation || 0, config.settings.thumbnailSize, true);
  if(thumb) {
    if(thumb.error_code === 0) {
      file.thumbnail = `data:image/jpeg;base64,${thumb.thumb_data_base64}`;
    } else if(thumb.error_code === 1) {
      file.thumbnail = thumbnailPlaceholder;
    }
  }
}

// toggle the selected file's favorite status (selectMode = false)
const toggleFavorite = async () => {
  if (selectedItemIndex.value >= 0) {
    const item = fileList.value[selectedItemIndex.value];
    item.is_favorite = !item.is_favorite;
    // update the favorite status in the database
    await setFileFavorite(item.id, item.is_favorite);
  }
};

// set selected files' favorite status (selectMode = true)
const selectModeSetFavorites = async (isFavorite: boolean) => {
  if (selectMode.value && selectedCount.value > 0) {
    const updates = fileList.value
      .filter(item => item.isSelected)
      .map(async item => {
        item.is_favorite = isFavorite;
        // update the favorite status in the database
        return setFileFavorite(item.id, isFavorite);
      }); 
    await Promise.all(updates); // parallelize DB updates
  }
}

// slide show
let slideShowIntervalId: NodeJS.Timeout | null = null;

function toggleSlideShow() {
  isSlideShow.value = !isSlideShow.value;
  if (isSlideShow.value) {
    startSlideShow();
  } else {
    stopSlideShow();
  }
}

function clearSlideShowTimer() {
  if (slideShowIntervalId) {
    clearInterval(slideShowIntervalId);
    slideShowIntervalId = null;
  }
}

// Check if current file is a video
function isCurrentFileVideo() {
  const currentFile = fileList.value[selectedItemIndex.value];
  return currentFile?.file_type === 2;
}

// Advance to next slide (handles looping)
function advanceSlideShow() {
  if (fileList.value.length === 0) return;
  
  if (selectedItemIndex.value >= fileList.value.length - 1) {
    selectedItemIndex.value = 0; // Loop
  } else {
    selectedItemIndex.value++;
  }
  
  // Schedule next advance based on file type
  scheduleNextSlide();
}

// Schedule the next slide transition
function scheduleNextSlide() {
  clearSlideShowTimer();
  
  if (!isSlideShow.value) return;
  
  // If current file is video, don't set timer - video's ended event will trigger next
  if (isCurrentFileVideo()) {
    return;
  }
  
  // For images, use the configured interval
  const interval = getSlideShowInterval(config.settings.slideShowInterval) * 1000;
  slideShowIntervalId = setTimeout(() => {
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

watch(() => config.settings.slideShowInterval, () => {
  if (isSlideShow.value && !isCurrentFileVideo()) {
    scheduleNextSlide();
  }
});

// set file rotate
const clickRotate = async () => {
  if (selectedItemIndex.value >= 0) {
    fileList.value[selectedItemIndex.value].rotate += 90;

    // notify the image viewer
    tauriEmit('message-from-content', { message: 'rotate', fileId: fileList.value[selectedItemIndex.value].id });

    // update the rotate status in the database
    setFileRotate(fileList.value[selectedItemIndex.value].id, fileList.value[selectedItemIndex.value].rotate);
  }
};

// set file tag
const clickTag = async () => {
  console.log('clickTag');
  if (selectMode.value) {
    fileIdsToTag.value = fileList.value
      .filter(file => file.isSelected)
      .map(file => file.id);
  } else if (selectedItemIndex.value >= 0) {
    fileIdsToTag.value = [fileList.value[selectedItemIndex.value].id];
  } else {
    fileIdsToTag.value = [];
  }
  showTaggingDialog.value = true;
}

const onEditComment = async (newComment: any) => {
  if (selectedItemIndex.value >= 0) {
    const file = fileList.value[selectedItemIndex.value];
    const result = await editFileComment(file.id, newComment);
    if(result) {
      console.log('onEditComment:', newComment);
      file.comments = newComment;
      showCommentMsgbox.value = false;
    }
  }
}

const handleSelectMode = (value: any) => {
  if(fileList.value.length === 0 || showQuickView.value || isIndexing.value) return;

  selectMode.value = value;
  if(!selectMode.value) {
    for (let i = 0; i < fileList.value.length; i++) {
      fileList.value[i].isSelected = false;
    }
  } else {
    showQuickView.value = false;
  }
};

const handleFileTypeSelect = (option: any, extendOption: any) => {
  selectMode.value = false;   // exit multi-select mode
  config.search.fileType = option;
};

const handleSortTypeSelect = (option: any, extendOption: any) => {
  selectMode.value = false;   // exit multi-select mode
  config.search.sortType = option;
  config.search.sortOrder = extendOption;
};

// file type options
const fileTypeOptions = computed(() => {
  return getSelectOptions(localeMsg.value.toolbar.filter?.file_type_options);
});

// sort type options
const sortOptions = computed(() => {
  return getSelectOptions(localeMsg.value.toolbar.filter?.sort_type_options);
});

// sort extend options
const sortExtendOptions = computed(() => {
  return getSelectOptions(localeMsg.value.toolbar.filter?.sort_order_options);
});

// update image when the select file is changed
async function updateSelectedImage(index: number) {
  if(index < 0 || index >= fileList.value.length) return;

  // update the tags for the selected file
  if(config.infoPanel.show && fileList.value[index].has_tags) {
    fileList.value[index].tags = await getTagsForFile(fileList.value[index].id);
  }
}

// click ok in tagging dialog
function updateFileHasTags(fileIds: number[]) {
  for (const fileId of fileIds) {
    const index = fileList.value.findIndex(f => f.id === fileId);
    if (index !== -1) {
      const newFile = fileList.value[index];
      getFileHasTags(fileId).then(hasTags => {
        newFile.has_tags = hasTags;
      });
    }
  }
  showTaggingDialog.value = false;
}

// Helper to yield to main thread
const yieldToMain = () => new Promise(resolve => setTimeout(resolve, 0));

// Track current thumbnail request to enable cancellation when switching folders
let currentThumbRequestId = 0;

// Get the thumbnail for the files (non-blocking, runs in background)
// Automatically cancels when a new request starts (e.g., switching folders)
async function getFileListThumb(files: any[], offset = 0, concurrencyLimit = 4) {
  // Use current request ID to check for cancellation
  const requestId = currentThumbRequestId;
  
  let activeRequests = 0;

  const getThumbForFile = async (file: any) => {
    // Check if this request is still valid before fetching
    if (requestId !== currentThumbRequestId) {
      return null; // Request cancelled
    }
    
    const thumb = await getFileThumb(file.id, file.file_path, file.file_type, file.e_orientation || 0, config.settings.thumbnailSize, false);
    
    // Check again after async operation
    if (requestId !== currentThumbRequestId) {
      return null; // Request cancelled
    }
    
    if(thumb) {
      if(thumb.error_code === 0) {
        file.thumbnail = `data:image/jpeg;base64,${thumb.thumb_data_base64}`;
      } else if(thumb.error_code === 1) {
        file.thumbnail = thumbnailPlaceholder;
      }
      thumbCount.value++; 
    }
    return file;
  };

  const runWithConcurrencyLimit = async (files: any[]) => {
    const queue: any[] = [];
    let processedCount = 0;

    for (let i = offset; i < files.length; i++) {
      // Check if request was cancelled before starting new file
      if (requestId !== currentThumbRequestId) {
        console.log(`Thumbnail generation cancelled (request ${requestId} superseded by ${currentThumbRequestId})`);
        return; // Stop processing
      }
      
      if (activeRequests >= concurrencyLimit) {
        await Promise.race(queue); // Wait for the first promise to complete
      }

      const filePromise = getThumbForFile(files[i])
        .then((file) => {
          // Remove the finished promise from the queue
          queue.splice(queue.indexOf(filePromise), 1);
          activeRequests--;
          processedCount++;
          return file;
        })
        .catch((error) => {
          queue.splice(queue.indexOf(filePromise), 1);
          activeRequests--;
          processedCount++;
          console.log('Error fetching thumbnail:', error);
        });

      queue.push(filePromise);
      activeRequests++;

      // Yield to main thread every 10 files to keep UI responsive
      if (i > 0 && i % 10 === 0) {
        await yieldToMain();
      }
    }

    // Wait for remaining promises (only if not cancelled)
    if (requestId === currentThumbRequestId && queue.length > 0) {
      await Promise.all(queue);
    }
  };

  // Run in background - don't block caller
  runWithConcurrencyLimit(files).then(() => {
    // Only log if this request wasn't cancelled
    if (requestId === currentThumbRequestId) {
      console.log('All thumbnails fetched successfully.');
    }
  });
}

// Open the image viewer window
async function openImageViewer(index: number, newViewer = false, syncFromFileListChange = false) {

  const webViewLabel = 'imageviewer';

  const fileCount = fileList.value.length;
  const isRealFile = (item: any) => !!item && !item.isPlaceholder && typeof item.id === 'number';
  const getRealFileAt = (targetIndex: number) => {
    if (targetIndex < 0 || targetIndex >= fileCount) return null;
    const file = fileList.value[targetIndex];
    return isRealFile(file) ? file : null;
  };

  let leftIndex = index;
  let rightIndex = -1;

  if (syncFromFileListChange) {
    if (fileCount === 0) {
      leftIndex = -1;
      rightIndex = -1;
    } else if (fileCount === 1) {
      leftIndex = 0;
      rightIndex = 0;
    } else {
      leftIndex = 0;
      rightIndex = 1;
    }
  }

  const leftFile = getRealFileAt(leftIndex);
  const rightFile = getRealFileAt(rightIndex);
  const leftFileId = leftFile ? leftFile.id : 0;
  const rightFileId = rightFile ? rightFile.id : 0;
  
  // create a new window if it doesn't exist
  let imageWindow = await WebviewWindow.getByLabel(webViewLabel);
  if (!imageWindow) {
    if (newViewer) {
      imageWindow = new WebviewWindow(webViewLabel, {
        url: `/image-viewer?fileId=${leftFileId}&fileIndex=${leftIndex}&fileCount=${fileCount}`,
        title: 'Image Viewer',
        width: 1200,
        height: 800,
        minWidth: 800,
        minHeight: 600,
        resizable: true,
        visible: false, // Start hidden, will show after mount
        transparent: true, // Prevent white flash on show (Tauri 2.x workaround)
        decorations: isMac,
        ...(isMac && {
          titleBarStyle: 'overlay',
          hiddenTitle: true,
          minimizable: true,
        }),
      });

      imageWindow.once('tauri://created', () => {
        console.log('ImageViewer window created');
        videoRef.value?.pause();  // pause video playing in preview pane
      });

      imageWindow.once('tauri://close-requested', () => {
        imageWindow?.close();
      });

      imageWindow.once('tauri://error', (e) => {
        console.error('Error creating ImageViewer window:', e);
      });
    }
  } else {    // update the existing window
    await imageWindow.emit('update-img', { 
      fileId: leftFileId, 
      fileIndex: leftIndex,   // selected file index
      fileCount: fileCount, // total files length
      pane: 'left',
      resetSplit: newViewer,
      // filePath: encodedFilePath, 
      // nextFilePath: nextEncodedFilePath,
    });

    if (syncFromFileListChange) {
      await imageWindow.emit('update-img', {
        fileId: rightFileId,
        fileIndex: rightIndex,
        fileCount: fileCount,
        pane: 'right',
      });
    }

    if(newViewer) {
      imageWindow.show();
    }
    videoRef.value?.pause();  // pause video playing in preview pane
  }
}

/// Dragging the film strip view splitter
// function startDraggingfilmStripView(event: MouseEvent) {
//   isDraggingFilmStripView.value = true;
//   document.addEventListener('mousemove', handleMouseMove);
//   document.addEventListener('mouseup', stopDragging);
// }

/// Dragging the info panel splitter
function startDraggingInfoPanelSplitter(event: MouseEvent) {
  isDraggingInfoPanel.value = true;
  document.addEventListener('mousemove', handleMouseMove);
  document.addEventListener('mouseup', stopDragging);
}

/// handle mouse move event
function handleMouseMove(event: MouseEvent) {
  if (!contentViewDiv.value || !gridViewDiv.value) {
    return;
  }
  if (isDraggingInfoPanel.value) {
    const contentViewWidth = contentViewDiv.value.clientWidth;
    const newWidth = contentViewDiv.value.getBoundingClientRect().right - event.clientX - 2; // -2: border width(2px)
    const newWidthPercent = (newWidth * 100) / contentViewWidth;

    // Limit width between 20% and 80%
    config.infoPanel.width = Math.min(Math.max(newWidthPercent, 20), 80);
  }
}

function stopDragging() {
  // isDraggingFilmStripView.value = false;
  isDraggingInfoPanel.value = false;
  document.removeEventListener('mousemove', handleMouseMove);
  document.removeEventListener('mouseup', stopDragging);
}
</script>

<style scoped>
</style>

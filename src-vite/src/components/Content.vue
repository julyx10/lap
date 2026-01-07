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
        <IconImageSearch v-if="tempViewMode === 'similar'" class="t-icon-size-sm shrink-0 text-primary"/>
        <IconFolderExpanded v-if="tempViewMode === 'album'" class="t-icon-size-sm shrink-0 text-primary"/>
        <component v-if="contentTitle && !isTempViewMode" :is=contentIcon class="t-icon-size-sm shrink-0"/>
        <div class="cursor-default overflow-hidden whitespace-pre text-ellipsis">
          <span :class="isTempViewMode ? 'text-primary' : ''" data-tauri-drag-region>{{ contentTitle }}</span>
        </div>
        <TButton v-if="isTempViewMode" 
          :icon="IconRestore" 
          :buttonSize="'medium'"
          :selected="true"
          @click="exitTempViewMode" 
        />
      </div>

      <!-- toolbar -->
      <div class="flex items-center gap-2 shrink-0">

        <!-- select mode -->
        <div tabindex="-1"
          :class="[
            'px-2 py-1 h-8 flex flex-row items-center rounded-box border border-content focus:outline-none text-sm shrink-0',
            fileList.length === 0 ? 'text-base-content/30' : 'border-base-content/30 hover:bg-base-100 hover:text-base-content cursor-pointer',
            selectMode ? 'border-primary' : ''
          ]"
          @click="handleSelectMode(true)"
        >
          <TButton v-if="selectMode"
            :icon="IconClose"
            :buttonSize="'small'"
            @click.stop="handleSelectMode(false)" 
          />
          <span class="px-1">{{ selectMode ? $t('toolbar.filter.select_count', { count: selectedCount }) : $t('toolbar.filter.select_mode') }}</span>
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
          :disabled="fileList.length === 0 || config.main.sidebarIndex === 1 || isTempViewMode"
          @select="handleFileTypeSelect"
        />

        <!-- sort type options -->
        <DropDownSelect
          :options="sortOptions"
          :defaultIndex="config.search.sortType"
          :extendOptions="sortExtendOptions"
          :defaultExtendIndex="config.search.sortOrder"
          :disabled="fileList.length === 0 || config.main.sidebarIndex === 1 || isTempViewMode"
          @select="handleSortTypeSelect"
        />

        <!-- select and layout section -->
        <div class="flex flex-row items-center">
          <IconSeparator class="t-icon-size-sm text-base-content/30" />

          <!-- toggle layout -->
          <TButton
            :icon="config.content.showFilmStrip ? IconGallery : IconGrid"
            :iconStyle="{ transform: `rotate(${config.settings.previewPosition === 0 ? 0 : 180}deg)`, transition: 'transform 0.3s ease-in-out' }" 
            @click="toggleGridViewLayout"
          />

          <!-- toggle info panel -->
          <TButton
            :icon="config.infoPanel.show ? IconSideBarOn : IconSideBarOff"
            @click="config.infoPanel.show = !config.infoPanel.show"
          />
        </div>
      </div>
    </div>

    <!-- progress bar -->
    <div v-if="config.main.sidebarIndex === 1 && fileList.length > 0 && showProgressBar" class="absolute top-11 left-0 right-0 z-50">
      <ProgressBar :percent="Number(((thumbCount / fileList.length) * 100).toFixed(0))" />
    </div>
      
    <!-- content view -->
    <div ref="contentViewDiv" class="flex-1 flex flex-row overflow-hidden">
      <div class="relative flex-1 flex flex-row overflow-hidden">
        <div ref="gridViewDiv" 
          :class="[
            'flex-1 flex',
            config.settings.previewPosition === 0 ? 'flex-col-reverse' : 'flex-col',
            config.content.showFilmStrip ? (config.settings.showStatusBar ? 'mt-12 mb-8' : 'mt-12 mb-1') : ''
          ]"
        >
          <div class="relative" 
            :class="{ 'flex-1': !config.content.showFilmStrip }"
            :style="{ height: config.content.showFilmStrip ? config.content.filmStripPaneHeight + 'px' : '' }"
          >
            <!-- grid view -->
            <div ref="gridScrollContainerRef" class="absolute px-1 w-full h-full">
              <GridView ref="gridViewRef"
                :selected-item-index="selectedItemIndex"
                :fileList="fileList"
                :showFolderFiles="config.main.sidebarIndex === 0 && !config.album.selected ? true : false"
                :selectMode="selectMode"
                :loading="isLoading"
                @item-clicked="handleItemClicked"
                @item-dblclicked="handleItemDblClicked"
                @item-select-toggled="handleItemSelectToggled"
                @item-action="handleItemAction"
                @visible-range-update="handleVisibleRangeUpdate"
                @scroll="handleGridScroll"
              />
              <!-- Navigation buttons -->
              <div v-if="config.content.showFilmStrip && fileList.length > 0" 
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

          <!-- splitter -->
          <div v-if="config.content.showFilmStrip" 
            :class="[ 
              'h-1 hover:bg-primary cursor-ns-resize transition-colors',
              isDraggingFilmStripView ? 'bg-primary' : 'bg-base-300'
            ]" 
            @mousedown="startDraggingfilmStripView"
          ></div>

          <!-- film strip preview -->
          <div v-if="config.content.showFilmStrip" ref="previewDiv" 
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
              />
            </div>
          </div> <!-- film strip preview -->
        </div> <!-- grid view -->

        <!-- custom scrollbar -->
        <div v-if="!config.content.showFilmStrip && fileList.length > 0" class="mt-12 shrink-0" :class="[ config.settings.showStatusBar ? 'mb-8' : 'mb-1' ]">
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
          class="absolute inset-0 z-[60] flex items-center justify-center bg-base-200/80 backdrop-blur-md overflow-hidden"
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
            />
          </div>
        </div>

        <!-- Scanning Overlay -->
        <div v-if="isScanning" class="absolute inset-0 z-50 bg-base-100/80 backdrop-blur-sm flex flex-col items-center justify-center gap-4 text-base-content/30">
          <IconUpdate class="mx-1 w-8 h-8" 
            :class="config.scan.albumQueue[0] === config.album.id ? 'animate-spin' : ''" 
          />  
          <span class="text-lg text-center">{{ config.scan.albumQueue[0] === config.album.id
            ? $t('search.scan.scanning', { album: config.scan.albumName, count: config.scan.count.toLocaleString(), total: config.scan.total.toLocaleString() }) 
            : $t('search.scan.wait_scan') 
          }}</span>
          <span class="text-sm text-center">{{ $t('search.scan.description') }}</span>
          <button class="btn btn-primary" @click="cancelScanning">
            <IconClose class="w-5 h-5" />
            {{ $t('search.scan.cancel') }}
          </button>
        </div>
      </div>

      <!-- info panel splitter -->
      <div v-if="config.infoPanel.show && !uiStore.isFullScreen"
        class="w-1 shrink-0 transition-colors mt-12"
        :class="{
          'mb-8': config.settings.showStatusBar,
          'mb-1': !config.settings.showStatusBar,
          'hover:bg-primary cursor-ew-resize': config.infoPanel.show,
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
            :fileInfo="fileList[selectedItemIndex]" 
            @close="config.infoPanel.show = false" 
          />
        </div>
      </transition>
    </div>

    <!-- status bar -->
    <div v-if="config.settings.showStatusBar" class="absolute px-2 h-8 bottom-0 left-0 right-0 z-30 flex items-center justify-between text-sm cursor-default"
      :class="config.settings.showStatusBar ? 'bg-base-300/80 backdrop-blur-md' : 'pointer-events-none'"
    >
      <!-- file status -->
      <div v-if="fileList.length > 0" class="flex gap-4 items-center flex-1 min-w-0 overflow-hidden">
        <div class="flex items-center flex-shrink-0">
          <IconFileSearch class="t-icon-size-xs mr-1" />
          <span v-if="selectedItemIndex >= 0"> {{ selectedItemIndex + 1 + '/' }}</span>
          <span>{{ $t('statusbar.files_summary', { count: totalFileCount.toLocaleString(), size: formatFileSize(totalFileSize) }) }}</span>
        </div>

        <template v-if="selectedItemIndex >= 0">
          <div class="flex items-center gap-1 flex-shrink-0">
            <component :is="selectMode ? IconCheckAll : IconChecked" class="t-icon-size-xs" />
            <span>
              {{ selectMode
                ? $t('toolbar.filter.select_count', { count: selectedCount.toLocaleString() }) + ' (' + formatFileSize(selectedSize) + ')'
                : shortenFilename(fileList[selectedItemIndex]?.name, 32) + ' (' + formatFileSize(fileList[selectedItemIndex]?.size) + ')'
              }}
            </span>
          </div>

          <div class="flex items-center gap-1 flex-shrink-0">
            <component :is="fileList[selectedItemIndex]?.file_type === 1 ? IconPhoto : IconVideo" class="t-icon-size-xs" />
            <span> {{ formatDimensionText(fileList[selectedItemIndex]?.width, fileList[selectedItemIndex]?.height) }} </span>
          </div>

          <div v-if="config.content.showFilmStrip || showQuickView" class="flex items-center gap-1 flex-shrink-0">
            <component :is="imageScale >= 1 ? IconZoomIn : IconZoomOut" class="t-icon-size-xs" />
            <span> {{ (imageScale * 100).toFixed(0) }}% </span>
          </div>

          <div v-if="fileList[selectedItemIndex]?.e_model" class="flex items-center gap-1 flex-shrink-0">
            <IconCamera class="t-icon-size-xs" />
            <span> {{ fileList[selectedItemIndex]?.e_model }} {{ fileList[selectedItemIndex]?.e_lens_model ? ' (' + fileList[selectedItemIndex]?.e_lens_model + ')' : '' }}</span>
          </div>

          <div v-if="fileList[selectedItemIndex]?.e_focal_length || fileList[selectedItemIndex]?.e_exposure_time || fileList[selectedItemIndex]?.e_f_number || fileList[selectedItemIndex]?.e_iso_speed || fileList[selectedItemIndex]?.e_exposure_bias" class="flex items-center gap-1 flex-shrink-0">
            <IconCameraAperture class="t-icon-size-xs" />
            <span> {{ formatCaptureSettings(fileList[selectedItemIndex]?.e_focal_length, fileList[selectedItemIndex]?.e_exposure_time, fileList[selectedItemIndex]?.e_f_number, fileList[selectedItemIndex]?.e_iso_speed, fileList[selectedItemIndex]?.e_exposure_bias) }}</span>
          </div>

          <div v-if="fileList[selectedItemIndex]?.geo_name" class="flex items-center gap-1 flex-shrink-0">
            <IconLocation class="t-icon-size-xs" />
            <span> {{ fileList[selectedItemIndex]?.geo_name }}</span>
          </div>
        </template>
      </div>
    </div>
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
    @ok="clickRenameFile"
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
    @ok="clickMoveTo"
    @cancel="showMoveTo = false"
  />

  <!-- copy to -->
  <MoveTo
    v-if="showCopyTo"
    :title="`${$t('msgbox.copy_to.title', { source: selectMode ? $t('toolbar.filter.select_count', { count: selectedCount.toLocaleString() }) : shortenFilename(fileList[selectedItemIndex].name, 32) })}`"
    :message="$t('msgbox.copy_to.content')"
    :OkText="$t('msgbox.copy_to.ok')"
    :cancelText="$t('msgbox.cancel')"
    @ok="clickCopyTo"
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
    @ok="clickTrashFile"
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
    @ok="clickEditComment"
    @cancel="showCommentMsgbox = false"
  />

  <ToolTip ref="toolTipRef" />

</template>

<script setup lang="ts">

import { ref, watch, computed, onMounted, onBeforeUnmount } from 'vue';
import { emit as tauriEmit, listen } from '@tauri-apps/api/event';
import { WebviewWindow } from '@tauri-apps/api/webviewWindow';
import { useI18n } from 'vue-i18n';
import { useUIStore } from '@/stores/uiStore';
import { getAlbum, getQueryCountAndSum, getQueryTimeLine, getQueryFiles, getFolderFiles, getFolderThumbCount,
         copyImage, renameFile, moveFile, copyFile, deleteFile, deleteDbFile, editFileComment, getFileThumb, getFileInfo,
         setFileRotate, getFileHasTags, setFileFavorite, getTagsForFile, searchSimilarImages, generateEmbedding, 
         revealFolder, getTagName, scanAlbum, listenScanProgress, listenScanFinished, setAlbumCover,
         updateFileInfo, cancelScan} from '@/common/api';  
import { config } from '@/common/config';
import { isWin, isMac, setTheme,
         formatFileSize, formatDate, getCalendarDateRange, getRelativePath, 
         extractFileName, combineFileName, getFolderPath, getSelectOptions, 
         shortenFilename, formatDimensionText, formatCaptureSettings, getSlideShowInterval } from '@/common/utils';

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
  IconCalendar,
  IconLocation,
  IconCamera,
  IconTrash,
  IconPhoto,
  IconVideo,
  IconCameraAperture,
  IconFileSearch,
  IconLeft,
  IconRight,
  IconSeparator,
  IconUpdate,
  IconZoomIn,
  IconZoomOut,
  IconCopyTo,
  IconGrid,
  IconGallery,
  IconImageSearch,
  IconSearch,
  IconRestore,
  IconRestoreScreen,
} from '@/common/icons';

import { useFileMenuItems } from '@/common/fileMenu';

const thumbnailPlaceholder = new URL('@/assets/images/image-file.png', import.meta.url).href;

const props = defineProps({
  titlebar: String
});

/// i18n
const { locale, messages } = useI18n();
const localeMsg = computed(() => messages.value[locale.value] as any);
const uiStore = useUIStore();

const contentIcon = computed(() => {
  const index = config.main.sidebarIndex;
  
  switch (index) {
    case 0: 
      switch (config.album.id) {
        case 0: return IconPhotoAll;
        default: return IconFolderExpanded;
      }
    case 1: return IconSearch;
    case 2: 
      switch (config.favorite.folderId) {
        case 0: return IconFavorite;
        case 1: return IconFolderFavorite;
        default: return IconFolderFavorite;
      }
    case 3: return IconTag;
    case 4: return IconCalendar;
    case 5: return IconLocation;
    case 6: return IconCamera;
    default: return IconFiles;
  }
});
const contentTitle = ref("");

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

// const showFolderFiles = computed(() => !!(config.main.sidebarIndex === 0 && config.album.id && config.album.id !== 0));

// const selectedFile = computed(() => {
//   if (selectedItemIndex.value >= 0 && selectedItemIndex.value < fileList.value.length) {
//     return fileList.value[selectedItemIndex.value];
//   }
//   return null;
// });

// const singleFileMenuItems = useFileMenuItems(
//   selectedFile,
//   localeMsg,
//   isMac,
//   showFolderFiles,
//   (action) => handleItemAction({ action, index: selectedItemIndex.value })
// );

// Request ID tracking to prevent race conditions during async content updates
let currentContentRequestId = 0;

const onScale = (event: any) => {
  imageScale.value = event.scale;
  imageMinScale.value = event.minScale;
  imageMaxScale.value = event.maxScale;
};

// film strip view splitter
const isDraggingFilmStripView = ref(false);      // dragging splitter to resize film strip view
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

// tagging dialog
const showTaggingDialog = ref(false);
const fileIdsToTag = ref<number[]>([]);

// grid view
const gridScrollContainerRef = ref<HTMLElement | null>(null);
const gridViewRef = ref<any>(null);

const scrollPosition = ref(0);    // scrollbar position (item index)

const containerHeight = ref(0);   // container height
const containerWidth = ref(0);    // container width
const gap = 8;                    // Gap between items (must match GridView)

const itemWidth = computed(() => {
  if (config.settings.grid.style === 0) {
    return config.settings.grid.size + gap * 2;
  } else if (config.settings.grid.style === 1) {
    return config.settings.grid.size;
  }
  return 0;
});

const itemSize = computed(() => {
  if (config.settings.grid.style === 0) {
    let labelHeight = 0
    if (config.settings.grid.labelPrimary > 0 ) labelHeight += 20;      // height of text-sm
    if (config.settings.grid.labelSecondary > 0 ) labelHeight += 16;    // height of text-xs
    return itemWidth.value + gap / 2 + labelHeight;
  } else if (config.settings.grid.style === 1) {
    return itemWidth.value + gap / 2;
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
  isShowHidden: false,
  tagId: 0,
});

// ai image search params
const currentImageSearchParams = ref({
  searchText: "",
  fileId: 0,
  threshold: 0,
  limit: 0,
  isShowHidden: false,
});

// Similar Search Mode State
const tempViewMode = ref<'none' | 'similar' | 'album'>('none');
const isTempViewMode = computed(() => tempViewMode.value !== 'none');
const backupState = ref<any>(null);

let unlistenKeydown: () => void;
let unlistenImageViewer: () => void;

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
function handleItemClicked(index: number) {
  selectedItemIndex.value = index;
  if (selectMode.value) {
    fileList.value[index].isSelected = !fileList.value[index].isSelected;
  }
}

// Double click grid view item
function handleItemDblClicked(index: number) {
  selectedItemIndex.value = index;

  if (!config.content.showFilmStrip) {
    // quick view
    showQuickView.value = true;
    quickViewZoomFit.value = true;
  }
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

async function clickSetAlbumCover() {
  const file = fileList.value[selectedItemIndex.value];
  if (file && config.album.id) {
    try {
      await setAlbumCover(config.album.id, file.id);
      await tauriEmit('album-cover-changed', { albumId: config.album.id, fileId: file.id });
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
    'rename': () => {
      renamingFileName.value = extractFileName(fileList.value[selectedItemIndex.value].name);
      showRenameMsgbox.value = true;
    },
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
    'set-album-cover': clickSetAlbumCover,
  };

  if ((actionMap as any)[action]) {
    (actionMap as any)[action]();
  }
}

function requestNavigate(direction: 'prev' | 'next') {
  const viewer = showQuickView.value ? quickViewMediaRef.value : (config.content.showFilmStrip ? filmStripMediaRef.value : null);
  
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
}

function performNavigate(direction: 'prev' | 'next') {
  if (direction === 'next') {
    if (selectedItemIndex.value < fileList.value.length - 1) {
      selectedItemIndex.value += 1;
    }
  } else {
    if (selectedItemIndex.value > 0) {
      selectedItemIndex.value -= 1;
    }
  }
}

function handleGridScroll(event: any) {
  if (event && event.target) {
    const scrollTop = event.target.scrollTop;
    
    if (!config.content.showFilmStrip) {
      // Calculate max scroll top
      const totalRows = Math.ceil(totalFileCount.value / columnCount.value);
      const topPadding = 48;
      const bottomPadding = config.settings.showStatusBar ? 32 : 4;
      const totalHeight = totalRows * itemSize.value + topPadding + bottomPadding;
      const maxScrollTop = Math.max(0, totalHeight - containerHeight.value);
      
      if (maxScrollTop <= 0) {
        scrollPosition.value = 0;
      } else {
        const ratio = Math.min(1, Math.max(0, scrollTop / maxScrollTop));
        const maxIndex = Math.max(1, totalFileCount.value - visibleItemCount.value);
        scrollPosition.value = Math.round(ratio * maxIndex);
      }
    } else {
      // Fallback for filmstrip or other layouts (horizontal)
      const rowIndex = Math.floor(scrollTop / itemSize.value);
      scrollPosition.value = rowIndex * columnCount.value;
    }
  }
}

function handleScrollUpdate(newIndex: number) {
  scrollPosition.value = newIndex;
  
  if (!config.content.showFilmStrip && gridViewRef.value) {
    // Calculate ratio (0 to 1)
    const maxIndex = Math.max(1, totalFileCount.value - visibleItemCount.value);
    const ratio = Math.min(1, Math.max(0, newIndex / maxIndex));
    
    // Calculate max scroll top
    const totalRows = Math.ceil(totalFileCount.value / columnCount.value);
    const topPadding = 48;
    const bottomPadding = config.settings.showStatusBar ? 32 : 4;
    const totalHeight = totalRows * itemSize.value + topPadding + bottomPadding;
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
    if (gridViewRef.value) {
      selectedItemIndex.value = Math.min(selectedItemIndex.value + gridViewRef.value.getColumnCount(), fileList.value.length - 1);
    }
  },
  ArrowUp: () => {
    if (gridViewRef.value) {
      selectedItemIndex.value = Math.max(selectedItemIndex.value - gridViewRef.value.getColumnCount(), 0);
    }
  },
  Home: () => selectedItemIndex.value = 0,
  End: () => selectedItemIndex.value = fileList.value.length - 1,
  '/': () => searchBoxRef.value.focusInput(),
};

// Local keydown handler for navigation (prevents default browser behavior)
function handleLocalKeyDown(event: KeyboardEvent) {
  // Check for input targets (prevent toggle while typing)
  const target = event.target as HTMLElement;
  if (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA' || target.isContentEditable) {
    return;
  }

  if (selectedItemIndex.value < 0 || fileList.value.length === 0) {
    return;
  }

  if (isScanning.value) {
    return;
  }

  // Disable keyboard events during slideshow (except Escape)
  if (isSlideShow.value && event.key !== 'Escape') {
    return;
  }

  const handledKeys = ['ArrowUp', 'ArrowDown', 'ArrowLeft', 'ArrowRight', 'Home', 'End', 'Enter', 'Space', ' '];

  if (event.key === 'Space' || event.key === ' ') {
    if(!config.content.showFilmStrip) {
      showQuickView.value = !showQuickView.value;
      quickViewZoomFit.value = true; 
    }
  }
  else if (event.key === 'Escape' && showQuickView.value) {
    if (uiStore.isFullScreen) {
      uiStore.isFullScreen = false;
    } else {
      showQuickView.value = false;
      stopSlideShow();
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
    clickRotate();
  } else if ((isMac && metaKey && key === 'Backspace') || (!isMac && key === 'Delete')) {
    showTrashMsgbox.value = true;
  } else if ((keyActions as any)[key]) {
    (keyActions as any)[key]();
  } else if (key === 'Escape') {
    if (selectMode.value) {
      handleSelectMode(false);
    }
  }
};

// --- Scanning Logic ---
let unlistenScanProgress: (() => void) | undefined;
let unlistenScanFinished: (() => void) | undefined;
let unlistenTriggerNextAlbum: (() => void) | undefined;

async function processNextAlbum() {
  if (config.scan.albumQueue.length > 0) {
    const albumId = config.scan.albumQueue[0];
    const album = await getAlbum(albumId);
    if (album) {
      config.scan.albumName = album.name;
      config.scan.count = 0;
      config.scan.total = 0;
      await scanAlbum(albumId);
    } else {
      // album not found (maybe deleted), remove from queue and process next
      config.scan.albumQueue.shift();
      processNextAlbum();
    }
  } else {
    config.scan.status = 0;
  }
}

// Check if current album is being scanned
const isScanning = computed(() => {
  return config.main.sidebarIndex === 0 && // Album mode
         config.album.id !== null && config.album.id !== 0 && // Valid album
         config.scan.albumQueue.includes(config.album.id);
});

watch(isScanning, (val) => {
  if (val) {
    uiStore.pushInputHandler('scanning');
  } else {
    if (uiStore.isInputActive('scanning')) {
        uiStore.popInputHandler();
    }
    updateContent()
  }
});

// Cancel scanning for current album
async function cancelScanning() {
  const albumId = config.album.id;
  const index = config.scan.albumQueue.indexOf(albumId);
  if (index === -1) return;

  if (index === 0) {
    // Active one: remove and restart processing for next
    config.scan.albumQueue.shift();
    config.scan.count = 0;
    config.scan.total = 0;
    
    // reset status
    config.scan.status = 0;

    // Call backend to cancel
    cancelScan(albumId);
    
    // trigger processing next
    // Do not call immediately to avoid parallel indexing
    setTimeout(() => {
      processNextAlbum();
    }, 1000);

  } else {
    // Pending one: just remove
    config.scan.albumQueue.splice(index, 1);
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
        const file = fileList.value[requestIndex];
        if (file) {
           const imageWindow = await WebviewWindow.getByLabel('imageviewer');
           if (imageWindow) {
             imageWindow.emit('update-img', {
               fileId: file.id,
               fileIndex: requestIndex,
               fileCount: fileList.value.length,
             });
           }
        }
        break;
      default:
        break;
    }
  });

  // Scanning listeners
  unlistenScanProgress = await listenScanProgress((event: any) => {
    const { album_id, current, total } = event.payload;
    if (config.scan.albumQueue.length > 0 && config.scan.albumQueue[0] === album_id) {
        config.scan.count = current;
        config.scan.total = total;
    }
  });

  unlistenScanFinished = await listenScanFinished(async (event: any) => {
    const { album_id } = event.payload;
    // notify album list to update cover
    await tauriEmit('album-cover-changed', { albumId: album_id, fileId: null });
    if (config.scan.albumQueue.length > 0 && config.scan.albumQueue[0] === album_id) {
      config.scan.albumQueue.shift();
      if (config.scan.albumQueue.length > 0) {
        processNextAlbum();
      } else {
        config.scan.status = 0;
      }
    }
  });

  unlistenTriggerNextAlbum = await listen('trigger-next-album', () => {
      processNextAlbum();
  });

  if (config.scan.albumQueue.length > 0) {
     if (config.scan.status === 1) {
        processNextAlbum();
     } else {
        config.scan.status = 1;
     }
  }
});

onBeforeUnmount(() => {
  window.removeEventListener('keydown', handleLocalKeyDown);
  // unlisten
  unlistenImageViewer();
  if (unlistenKeydown) unlistenKeydown();
  if (unlistenTriggerNextAlbum) unlistenTriggerNextAlbum();
  if (unlistenScanProgress) unlistenScanProgress();
  if (unlistenScanFinished) unlistenScanFinished();
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

watch(() => config.scan.status, (newStatus) => {
  if (newStatus === 1 && config.scan.albumQueue.length > 0) {
    processNextAlbum();
  }
});

watch(() => config.scan.albumQueue.length, (newLength) => {
   if (newLength > 0 && config.scan.status === 0) {
       config.scan.status = 1; 
   }
});

/// watch image search params
watch(
  () => [
    config.search.searchText, 
    config.search.similarImageHistoryIndex, 
    config.search.searchType
  ],
  () => {
    setTimeout(() => {
      // Only update content if we are currently in the Image Search view (index 1)
      if (config.main.sidebarIndex === 1) {
        scrollPosition.value = 0;   // reset file scroll position
        selectedItemIndex.value = 0; // reset selected item index to 0
        
        // Also reset the GridView scroll position
        if (gridViewRef.value) {
          gridViewRef.value.scrollToPosition(0);
        }
        
        updateContent();
  
        // Reset ImageViewer context if open (without focusing/showing it)
        openImageViewer(selectedItemIndex.value, false);
      }
    }, 0);
  }
);

/// watch for file list changes
watch(
  () => [
    config.main.sidebarIndex,      // toolbar index
    config.album.id, config.album.folderId, config.album.folderPath,                 // home(album)
    config.favorite.albumId, config.favorite.folderId, config.favorite.folderPath,   // favorite files and folder
    config.tag.id,                                                                   // tag
    config.calendar.year, config.calendar.month, config.calendar.date,               // calendar
    config.location.admin1, config.location.name,                                    // location
    config.camera.make, config.camera.model,                                         // camera 
    config.search.fileName, config.search.fileType, config.search.sortType, config.search.sortOrder, // search and sort 
  ], 
  () => {
    setTimeout(() => {
      scrollPosition.value = 0;   // reset file scroll position
      selectedItemIndex.value = 0; // reset selected item index to 0
      
      // Also reset the GridView scroll position
      if (gridViewRef.value) {
        gridViewRef.value.scrollToPosition(0);
      }
      
      updateContent();
  
      // Reset ImageViewer context if open (without focusing/showing it)
      openImageViewer(selectedItemIndex.value, false);
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
watch(() => [config.content.showFilmStrip], ([newLayout]) => {
  if (newLayout) {
    updateSelectedImage(selectedItemIndex.value);
  }
});

function toggleGridViewLayout() {
  showQuickView.value = false;
  config.content.showFilmStrip = !config.content.showFilmStrip;
  filmStripZoomFit.value = true;
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
                fileList.value[chunkStart + j] = newFiles[j];
                filesToFetch.push(fileList.value[chunkStart + j]);

                // Update ImageViewer if the selected file is loaded
                if (chunkStart + j === selectedItemIndex.value) {
                  openImageViewer(selectedItemIndex.value, false);
                  updateSelectedImage(selectedItemIndex.value);
                }
              }
            }
            // Fetch thumbnails for these files
            if (filesToFetch.length > 0) {
              getFileListThumb(filesToFetch);
            }
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
    isShowHidden = false,
    tagId = 0
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
    isShowHidden,
    tagId,
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
      
      // Reset visible range tracking when changing views
      lastVisibleRange = { start: -1, end: -1 };
    } else {
      fileList.value = [];
    }
  } catch (err) {
    console.error('getFileList error:', err);
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

async function getImageSearchFileList(searchText: string, fileId: number, requestId: number) {
  currentImageSearchParams.value = {
    searchText,
    fileId,
    threshold: config.settings.imageSearch.threshold[config.settings.imageSearch.thresholdIndex],
    limit: config.settings.imageSearch.limit,
    isShowHidden: false,
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

        // Reset visible range tracking when changing views
        lastVisibleRange = { start: -1, end: -1 };
        
        // Update search history with the first result's file_id
        if (searchText && result.length > 0) {
          const history = config.search.searchHistory as any[];
          const index = history.findIndex((item: any) => {
             const text = typeof item === 'string' ? item : item.text;
             return text === searchText;
          });

          if (index !== -1) {
             const item = history[index];
             if (typeof item !== 'string' && !item.file_id) {
               item.file_id = result[0].id;
             } else if (typeof item === 'string') {
               history[index] = { text: item, file_id: result[0].id };
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

  const requestId = ++currentContentRequestId;
  const newIndex = config.main.sidebarIndex;

  // Reset file list immediately to reflect UI change
  fileList.value = [];
  isLoading.value = true;

  if(newIndex === 0) {   // album  
    if(config.album.id === null) {
      contentTitle.value = "";
    } else if(config.album.id === 0) {   // all files
      contentTitle.value = localeMsg.value.album.all_files;
      getFileList({}, requestId);
    } else {
      getAlbum(config.album.id).then(async album => {
        if (requestId !== currentContentRequestId) return;
        if(album) {
          if(config.album.folderPath === album.path) { // current folder is root
            contentTitle.value = album.name;
            getFileList({ searchAllSubfolders: config.album.folderPath, isShowHidden: true }, requestId);
          } else {
            contentTitle.value = album.name + getRelativePath(config.album.folderPath || "", album.path);
            getFileList({ searchFolder: config.album.folderPath, isShowHidden: true }, requestId);
          };
        } else {
          contentTitle.value = "";
        }
      });
    }
  }
  else if(newIndex === 1) {   // image search
    if(config.search.searchType === 0) {   // search
      if (config.search.searchText) {
        contentTitle.value = localeMsg.value.search.search_images + ' - ' + config.search.searchText;
        getImageSearchFileList(config.search.searchText, 0, requestId);
      } else {
        contentTitle.value = localeMsg.value.search.search_images;
      }
    } else if (config.search.searchType === 1) { // similar
      const index = config.search.similarImageHistoryIndex;
      if (index >= 0 && index < config.search.similarImageHistory.length) {
        const file = await getFileInfo(config.search.similarImageHistory[index]);
        contentTitle.value = localeMsg.value.search.similar_images + ' - ' + file.name;
        getImageSearchFileList("", config.search.similarImageHistory[index], requestId);
      } else {
        contentTitle.value = localeMsg.value.search.similar_images;
      }
    } else {   // filename search
      if (config.search.fileName) {
        contentTitle.value = localeMsg.value.search.filename_search + ' - ' + config.search.fileName;
        getFileList({ searchFileName: config.search.fileName, sortType: 1, sortOrder: 0 }, requestId); // sort by name
      } else {
        contentTitle.value = localeMsg.value.search.filename_search;
      }
    }
  } 
  else if(newIndex === 2) {   // favorite
    if(config.favorite.folderId === null) {
      contentTitle.value = "";
    } else {
      if(config.favorite.folderId === 0) { // favorite files
        contentTitle.value = localeMsg.value.favorite.files;
        getFileList({ isFavorite: true }, requestId);
      } else {                // favorite folders
        getAlbum(config.favorite.albumId).then(album => {
          if (requestId !== currentContentRequestId) return;
          if(album) {
            contentTitle.value = localeMsg.value.favorite.folders + getRelativePath(config.favorite.folderPath || "", album.path);
            getFileList({ searchAllSubfolders: config.favorite.folderPath || "" }, requestId);
          } else {
            contentTitle.value = "";
          }
        });
      }
    }
  }
  else if(newIndex === 3) {   // tag
    if (config.tag.id === null) {
      contentTitle.value = "";
    } else {
      getTagName(config.tag.id).then(tagName => {
        if (requestId !== currentContentRequestId) return;
        if (tagName) {
          contentTitle.value = tagName;
          getFileList({ tagId: config.tag.id || 0 }, requestId);
        } else {
          contentTitle.value = "";
        }
      });
    }
  }
  else if(newIndex === 4) {   // calendar
    if(config.calendar.year === null) {
      contentTitle.value = "";
    } else {
      if (config.calendar.month === -1) {          // yearly
        contentTitle.value = formatDate(config.calendar.year, 1, 1, localeMsg.value.format.year);
      } else if (config.calendar.date === -1) {    // monthly
        contentTitle.value = formatDate(config.calendar.year, config.calendar.month, 1, localeMsg.value.format.month);
      } else {                                    // daily
        contentTitle.value = formatDate(config.calendar.year, config.calendar.month, config.calendar.date, localeMsg.value.format.date_long);
      }
      const [startDate, endDate] = getCalendarDateRange(config.calendar.year, config.calendar.month, config.calendar.date);
      getFileList({ startDate, endDate }, requestId);
    }
  }
  else if(newIndex === 5) {   // location
    if(config.location.admin1 === null) {
      contentTitle.value = "";
    } else {
      if(config.location.name) {
        contentTitle.value = `${config.location.admin1} > ${config.location.name}`;
        getFileList({ locationAdmin1: config.location.admin1, locationName: config.location.name }, requestId);
      } else {
        contentTitle.value = `${config.location.admin1}`;
        getFileList({ locationAdmin1: config.location.admin1 }, requestId);
      } 
    }
  }
  else if(newIndex === 6) {   // camera
    if(config.camera.make === null) {
      contentTitle.value = "";
    } else {
      if(config.camera.model) {
        contentTitle.value = `${config.camera.make} > ${config.camera.model}`;
        getFileList({ make: config.camera.make, model: config.camera.model }, requestId);
      } else {
        contentTitle.value = `${config.camera.make}`;
        getFileList({ make: config.camera.make }, requestId);
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
  const existingIndex = (config.search.similarImageHistory as any[]).findIndex(item => item === file.id);
  
  if (existingIndex !== -1) {
    config.search.similarImageHistoryIndex = existingIndex;
  } else {
    (config.search.similarImageHistory as any[]).unshift(file.id);
    config.search.similarImageHistoryIndex = 0;
    
    if (config.search.similarImageHistory.length > config.search.maxSearchHistory) {
      (config.search.similarImageHistory as any[]).pop();
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
        contentTitle.value = album.name + getRelativePath(folderPath || "", album.path);
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
  
  getFileList({ searchFolder: folderPath, isShowHidden: true }, requestId);
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

// update the file info from the image editor
const onImageEdited = (success: boolean) => {
  if (success) {
    toolTipRef.value.showTip(localeMsg.value.tooltip.save_image.success);
    showImageEditor.value = false;

    // update file info if save as is overwrite
    if(config.imageEditor.saveAs === 0) {
      updateFile(fileList.value[selectedItemIndex.value]);
    }
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

// click rename menu item
const clickRenameFile = async (newName: string) => {
  if(selectedItemIndex.value >= 0) {
    const file = fileList.value[selectedItemIndex.value];
    const fileName = combineFileName(newName, renamingFileName.value.ext);
    const newFilePath = await renameFile(file.id, file.file_path, fileName );
    if(newFilePath) {
      console.log('clickRenameFile:', newFilePath);
      file.name = fileName;
      file.file_path = newFilePath;
      showRenameMsgbox.value = false;
      errorMessage.value = '';
    } else {
      errorMessage.value = localeMsg.value.msgbox.rename_file.error;
    }
  }
}

// click move to menu item
const clickMoveTo = async () => {
  if (selectMode.value && selectedCount.value > 0) {    // multi-select mode
    const moves = fileList.value
      .filter(item => item.isSelected)
      .map(async item => {
        const movedFile = await moveFile(item.id, item.file_path, config.destFolder.folderId, config.destFolder.folderPath);
        if(movedFile) {
          console.log('clickMoveTo:', movedFile);
          removeFromFileList(fileList.value.indexOf(item));
        }
      });
    await Promise.all(moves); // parallelize DB updates
    selectMode.value = false; // exit multi-select mode
  } 
  else if(selectedItemIndex.value >= 0) {               // single select mode
    const file = fileList.value[selectedItemIndex.value];
    const movedFile = await moveFile(file.id, file.file_path, config.destFolder.folderId, config.destFolder.folderPath);
    if(movedFile) {
      console.log('clickMoveTo:', movedFile);
      removeFromFileList(selectedItemIndex.value);
    }
  }
  showMoveTo.value = false;
}

// click copy to menu item
const clickCopyTo = async () => {
  if (selectMode.value && selectedCount.value > 0) {    // multi-select mode
    const copies = fileList.value
      .filter(item => item.isSelected)
      .map(async item => {
        const copiedFile = await copyFile(item.file_path, config.destFolder.folderPath);
        if(copiedFile) {
          console.log('clickCopyTo:', copiedFile);
        }
      });
    await Promise.all(copies); // parallelize DB updates
    selectMode.value = false; // exit multi-select mode
  } 
  else if(selectedItemIndex.value >= 0) {               // single select mode
    const file = fileList.value[selectedItemIndex.value];
    const copiedFile = await copyFile(file.file_path, config.destFolder.folderPath);
    if(copiedFile) {
      console.log('clickCopyTo:', copiedFile);
    }
  }
  showCopyTo.value = false;
}

// click trash menu item
const clickTrashFile = async () => {
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
  if (isProcessing.value) return;

  isProcessing.value = true;

  let success = false;
  try {
    const updatedFile = await updateFileInfo(file.id, file.file_path);
    if (updatedFile) {
      Object.assign(file, updatedFile);
      await updateThumbForFile(file);
      await updateSelectedImage(selectedItemIndex.value);
      success = true;
    }
  } finally {
    isProcessing.value = false;
    // if (success) {
    //   toolTipRef.value.showTip(localeMsg.value.tooltip.update_image.success);
    // } else {
    //   toolTipRef.value.showTip(localeMsg.value.tooltip.update_image.failed, true);
    // }
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

function startSlideShow() {
  clearSlideShowTimer(); // Clear existing if any
  const interval = getSlideShowInterval(config.settings.slideShowInterval) * 1000;
  slideShowIntervalId = setInterval(() => {
    if (fileList.value.length === 0) return;
    
    if (selectedItemIndex.value >= fileList.value.length - 1) {
      selectedItemIndex.value = 0; // Loop (user requested: skip to first one)
    } else {
      selectedItemIndex.value++;
    }
  }, interval);
}

function stopSlideShow() {
  isSlideShow.value = false;
  clearSlideShowTimer();
}

watch(() => config.settings.slideShowInterval, () => {
  if (isSlideShow.value) {
    startSlideShow();
  }
});

watch(() => config.content.showFilmStrip, () => {
  stopSlideShow();
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

// edit comment
const clickEditComment = async (newComment: any) => {
  if (selectedItemIndex.value >= 0) {
    const file = fileList.value[selectedItemIndex.value];
    const result = await editFileComment(file.id, newComment);
    if(result) {
      console.log('clickEditComment:', newComment);
      file.comments = newComment;
      showCommentMsgbox.value = false;
    }
  }
}

const handleSelectMode = (value: any) => {
  if(fileList.value.length === 0) return;

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

// embedding queue
const embeddingQueue: any[] = [];
let isGeneratingEmbeddings = false;

async function processEmbeddingQueue() {
  if (isGeneratingEmbeddings) return;
  isGeneratingEmbeddings = true;

  while (embeddingQueue.length > 0) {
    const file = embeddingQueue.shift();
    // check if the file still exists in the fileList and need embedding
    // (user might have navigated away, but we can still process if we want, 
    // but better to check if it's still relevant or just process it for background indexing)
    
    // Generates embedding one by one to avoid blocking main thread/ui
    await generateEmbedding(file.id);
    file.has_embedding = true;

    // Small delay to yield to main thread if needed, though await above yields.
    // await new Promise(resolve => setTimeout(resolve, 10)); 
  }

  isGeneratingEmbeddings = false;
}

// Get the thumbnail for the files
async function getFileListThumb(files: any[], offset = 0, concurrencyLimit = 8) {
  const result = [];
  let activeRequests = 0;
  thumbCount.value = 0;

  const getThumbForFile = async (file: any) => {
    const thumb = await getFileThumb(file.id, file.file_path, file.file_type, file.e_orientation || 0, config.settings.thumbnailSize, false);
    if(thumb) {
      if(thumb.error_code === 0) {
        file.thumbnail = `data:image/jpeg;base64,${thumb.thumb_data_base64}`;
      } else if(thumb.error_code === 1) {
        file.thumbnail = thumbnailPlaceholder;
      }
      thumbCount.value++; 
      
      // generate embedding if not exist
      if ((file.file_type === 1 || file.file_type === 3) && !file.has_embedding) {
        // queue it to generate in background
        embeddingQueue.push(file);
      }
    }
    return file;
  };

  const runWithConcurrencyLimit = async (files: any[]) => {
    const queue: any[] = [];

    for (let i = offset; i < files.length; i++) {
      if (activeRequests >= concurrencyLimit) {
        await Promise.race(queue); // Wait for the first promise to complete
      }

      const filePromise = getThumbForFile(files[i])
        .then((file) => {
          // Remove the finished promise from the queue
          queue.splice(queue.indexOf(filePromise), 1);
          activeRequests--;
          return file;
        })
        .catch((error) => {
          console.log('Error fetching thumbnail:', error);
        });

      queue.push(filePromise);
      activeRequests++;
    }

    return Promise.all(queue);
  };

  result.push(...await runWithConcurrencyLimit(files));
  console.log('All thumbnails fetched successfully.');
  
  // start processing embedding queue
  if (embeddingQueue.length > 0) {
    processEmbeddingQueue();
  }
}

// Open the image viewer window
async function openImageViewer(index: number, newViewer = false) {

  const webViewLabel = 'imageviewer';

  const fileCount = fileList.value.length;

  const file = index >= 0 && index < fileCount ? fileList.value[index] : null;
  const fileId = file ? file.id : 0;
  
  // create a new window if it doesn't exist
  let imageWindow = await WebviewWindow.getByLabel(webViewLabel);
  if (!imageWindow) {
    if (newViewer) {
      imageWindow = new WebviewWindow(webViewLabel, {
        url: `/image-viewer?fileId=${fileId}&fileIndex=${index}&fileCount=${fileCount}`,
        title: 'Image Viewer',
        width: 1200,
        height: 800,
        minWidth: 800,
        minHeight: 600,
        resizable: true,
        decorations: isMac,
        transparent: isWin,
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
      fileId: fileId, 
      fileIndex: index,   // selected file index
      fileCount: fileCount, // total files length
      // filePath: encodedFilePath, 
      // nextFilePath: nextEncodedFilePath,
    });
    if(newViewer) {
      imageWindow.show();
    }
    videoRef.value?.pause();  // pause video playing in preview pane
  }
}

/// Dragging the film strip view splitter
function startDraggingfilmStripView(event: MouseEvent) {
  isDraggingFilmStripView.value = true;
  document.addEventListener('mousemove', handleMouseMove);
  document.addEventListener('mouseup', stopDragging);
}

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
  } else if(isDraggingFilmStripView.value) {
    const gridViewDivRect = gridViewDiv.value.getBoundingClientRect();
    const container = gridScrollContainerRef.value;
    let verticalSpacing = 0;
    if (container) {
      const style = window.getComputedStyle(container);
      verticalSpacing = parseFloat(style.paddingTop) + parseFloat(style.paddingBottom) + 
                        parseFloat(style.marginTop) + parseFloat(style.marginBottom);
    }

    const newHeight = 
      config.settings.previewPosition === 0 ? 
        gridViewDivRect.bottom - event.clientY - 2 - verticalSpacing :
        event.clientY - gridViewDivRect.top - 2 - verticalSpacing; // -2: border width(2px)
    
    const totalHeight = gridViewDiv.value.clientHeight;
    const minHeight = Math.max(totalHeight * 0.1, 80);
    const maxHeight = Math.min(totalHeight * 0.5, 320);

    // if scrollbar is visible, subtract the scrollbar height
    const scrollbarWidth = gridScrollContainerRef.value 
      ? gridScrollContainerRef.value.scrollWidth > gridScrollContainerRef.value.clientWidth 
        ? gridScrollContainerRef.value.offsetHeight - gridScrollContainerRef.value.clientHeight 
        : 0 
      : 0;
    config.content.filmStripPaneHeight = Math.min(Math.max(newHeight, minHeight), maxHeight) - scrollbarWidth;
  }
}

function stopDragging() {
  isDraggingFilmStripView.value = false;
  isDraggingInfoPanel.value = false;
  document.removeEventListener('mousemove', handleMouseMove);
  document.removeEventListener('mouseup', stopDragging);
}
</script>

<style scoped>
</style>

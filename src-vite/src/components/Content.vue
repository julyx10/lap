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
    <div :class="
      [
        'absolute top-0 left-0 right-0 pr-1 h-12 flex flex-row flex-wrap items-center justify-between bg-base-300/80 backdrop-blur-sm z-30',
        config.home.showLeftPane ? 'pl-1' : 'pl-18'
      ]" 
      data-tauri-drag-region
    >
      <!-- title -->
      <div class="flex flex-row items-center min-w-0 flex-1" data-tauri-drag-region>
        <TButton v-if="!config.home.showLeftPane || config.home.sidebarIndex === 0"
          :icon="config.home.showLeftPane ? IconLeftPaneOn : IconLeftPaneOff"
          @click="config.home.showLeftPane = !config.home.showLeftPane"
        />
        <IconSeparator v-if="!config.home.showLeftPane || config.home.sidebarIndex === 0" class="t-icon-size-sm text-base-content/30" />
        <component v-if="contentTitle" :is=contentIcon class="t-icon-size-sm shrink-0"/>
        <div class="mx-2 cursor-default overflow-hidden whitespace-pre text-ellipsis">
          {{ contentTitle }}
        </div>
      </div>

      <!-- toolbar -->
      <div class="flex items-center space-x-2 shrink-0">

        <!-- search box -->
        <SearchBox ref="searchBoxRef" v-model="config.search.text" @click.stop="selectMode = false" /> 

        <!-- select mode -->
        <div tabindex="-1"
          :class="[
            'px-2 py-1 h-8 flex flex-row items-center rounded-box border border-content focus:outline-none text-sm shrink-0 cursor-pointer',
            selectMode ? 'border-primary' : 'border-base-content/30 hover:bg-base-100 hover:text-base-content'
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
          @select="handleFileTypeSelect"
        />

        <!-- sort type options -->
        <DropDownSelect
          :options="sortOptions"
          :defaultIndex="config.search.sortType"
          :extendOptions="sortExtendOptions"
          :defaultExtendIndex="config.search.sortOrder"
          @select="handleSortTypeSelect"
        />

        <!-- separator -->
        <div class="flex flex-row items-center">
          <IconSeparator class="t-icon-size-sm text-base-content/30" />
          <!-- grid view layout -->
          <TButton
            :icon="config.content.layout === 0 ? IconGrid : IconGallery"
            :iconStyle="{ transform: `rotate(${config.settings.filmStripView.previewPosition === 0 ? 0 : 180}deg)`, transition: 'transform 0.3s ease-in-out' }" 
            @click="toggleGridViewLayout"
          />
          <!-- show info panel -->
          <TButton
            :icon="config.infoPanel.show ? IconPreview : IconPreviewOff"
            @click="config.infoPanel.show = !config.infoPanel.show"
          />
        </div>
      </div>
    </div>

    <!-- progress bar -->
    <div v-if="config.home.sidebarIndex === 1 && fileList.length > 0 && showProgressBar" class="absolute top-11 left-0 right-0 z-50">
      <ProgressBar :percent="Number(((thumbCount / fileList.length) * 100).toFixed(0))" />
    </div>
      
    <!-- content view -->
    <div ref="contentViewDiv" class="flex-1 flex flex-row overflow-hidden">

      <div ref="gridViewDiv" 
        :class="[
          'flex-1 flex',
          config.settings.filmStripView.previewPosition === 0 ? 'flex-col-reverse' : 'flex-col'
        ]"
      >
        <div class="relative" :class="{ 'flex-1': config.content.layout === 0 }">
          
          <!-- grid view -->
          <div ref="gridScrollContainerRef" 
            :class="[
              config.content.layout === 0 ? 'absolute w-full h-full overflow-hidden' : 'overflow-x-hidden overflow-y-hidden',
              (config.content.layout === 0 || (config.content.layout === 1 && config.settings.filmStripView.previewPosition === 1)) ? 'pt-12' : '',
              config.settings.showStatusBar ? 
                (config.content.layout === 0 ? 'pb-8' : (config.content.layout === 1 && config.settings.filmStripView.previewPosition === 0) ? 'pb-8' : ''
              ) : (
                config.content.layout === 0 ? 'pb-1' : (config.content.layout === 1 && config.settings.filmStripView.previewPosition === 0) ? 'mb-1' : ''
              )
            ]"
            >
            <GridView ref="gridViewRef"
              :style="{ height: config.content.layout === 1 ? config.content.filmStripPaneHeight + 'px' : '' }"
              :selected-item-index="selectedItemIndex"
              :fileList="fileList"
              :showFolderFiles="showFolderFiles"
              :selectMode="selectMode"
              @item-clicked="handleItemClicked"
              @item-dblclicked="handleItemDblClicked"
              @item-select-toggled="handleItemSelectToggled"
              @item-action="handleItemAction"
              @request-scroll="handleRequestScroll"
              @visible-range-update="handleVisibleRangeUpdate"
              @scroll="handleGridScroll"
            />
          </div>

          <!-- Navigation buttons -->
          <div v-if="config.content.layout === 1 && fileList.length > 0" 
            class="absolute z-10 inset-1 flex items-center justify-between pointer-events-none"
            :class="[
              (config.content.layout === 0 || (config.content.layout === 1 && config.settings.filmStripView.previewPosition === 1)) ? 'pt-12' : '',
              config.settings.showStatusBar ? 
                (config.content.layout === 0 ? 'pb-8' : (config.content.layout === 1 && config.settings.filmStripView.previewPosition === 0) ? 'pb-8' : ''
              ) : (
                config.content.layout === 0 ? 'pb-1' : (config.content.layout === 1 && config.settings.filmStripView.previewPosition === 0) ? 'mb-1' : ''
              )
            ]"
          >
            <button 
              :class="[
                'p-2 rounded-full pointer-events-auto bg-base-100/30', 
                selectedItemIndex > 0 ? 'text-base-content/70 hover:text-base-content hover:bg-base-100/70 cursor-pointer' : 'text-base-content/30'
              ]"
              @click="handleNavigate('prev')"
              @dblclick.stop
            >
              <IconLeft class="w-8 h-8" />
            </button>
            <button 
              :class="[
                'p-2 rounded-full pointer-events-auto bg-base-100/30', 
                selectedItemIndex < fileList.length - 1 ? 'text-base-content/70 hover:text-base-content hover:bg-base-100/70 cursor-pointer' : 'text-base-content/30'
              ]"
              @click="handleNavigate('next')" 
              @dblclick.stop
            >
              <IconRight class="w-8 h-8" />
            </button> 
          </div>
        </div>

        <!-- splitter -->
        <div v-if="config.content.layout === 1" 
          :class="[ 
            'h-1 hover:bg-primary cursor-ns-resize transition-colors',
            isDraggingFilmStripView ? 'bg-primary' : 'bg-base-300'
          ]" 
          @mousedown="startDraggingfilmStripView"
        ></div>

        <!-- preview -->
        <div v-if="config.content.layout === 1" ref="previewDiv" 
          class="flex-1 rounded-box bg-base-200 overflow-hidden"
          :class="[ config.settings.filmStripView.previewPosition === 0 ? 'mt-12' : (config.settings.showStatusBar ? 'mb-8' : 'mb-1') ]"
        >
          <div v-if="selectedItemIndex >= 0 && selectedItemIndex < fileList.length"
            class="w-full h-full flex items-center justify-center"
            @dblclick="filmStripViewZoomFit = !filmStripViewZoomFit"
          >
            <Image v-if="fileList[selectedItemIndex]?.file_type === 1"
              :filePath="fileList[selectedItemIndex]?.file_path" 
              :rotate="fileList[selectedItemIndex]?.rotate ?? 0" 
              :isZoomFit="filmStripViewZoomFit"
            ></Image>
            
            <Video v-if="fileList[selectedItemIndex]?.file_type === 2"
              :filePath="fileList[selectedItemIndex]?.file_path"
              :rotate="fileList[selectedItemIndex]?.rotate ?? 0"
              :isZoomFit="filmStripViewZoomFit"
            ></Video>
          </div>

        </div> <!-- preview -->

      </div> <!-- grid view -->

      <!-- custom scrollbar -->
      <div v-if="config.content.layout === 0 && fileList.length > 0" class="w-8 mt-12 shrink-0" :class="[ config.settings.showStatusBar ? 'mb-8' : 'mb-1' ]">
        <ScrollBar
          :total="totalFileCount"
          :pageSize="visibleItemCount"
          :modelValue="scrollPosition"
          :markers="timelineMarkers"
          @update:modelValue="handleScrollUpdate"
        ></ScrollBar>
      </div>

      <!-- info panel splitter -->
      <div
        class="w-1 shrink-0 transition-colors"
        :class="{
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
        <div v-if="config.infoPanel.show" 
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
    <div v-if="config.settings.showStatusBar"
      class="absolute px-2 h-8 bottom-0 left-0 right-0 z-30 bg-base-300/80 backdrop-blur-sm flex gap-4 text-sm cursor-default"
    >
      <div class="flex items-center gap-1 flex-shrink-0">
        <IconFileSearch class="t-icon-size-xs" />
        <span >
          {{ $t('statusbar.files_summary', { count: totalFileCount.toLocaleString(), size: formatFileSize(totalFileSize) }) }}
        </span>
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

        <div v-if="fileList[selectedItemIndex]?.e_model" class="flex items-center gap-1 flex-shrink-0">
          <IconCamera class="t-icon-size-xs" />
          <span> {{ fileList[selectedItemIndex]?.e_model }} ({{ fileList[selectedItemIndex]?.e_lens_model }})</span>
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

  <!-- image editor -->
  <ImageEditor 
    v-if="showImageEditor"
    :fileInfo="fileList[selectedItemIndex]" 
    @ok="onImageEdited" 
    @cancel="showImageEditor = false"
  />

  <!-- rename -->
  <MessageBox
    v-if="showRenameMsgbox"
    :title="$t('msgbox.rename_file.title')"
    :showInput="true"
    :inputText="renamingFileName.name"
    :inputPlaceholder="$t('msgbox.rename_file.placeholder')"
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

import { ref, watch, computed, onMounted, onBeforeUnmount, defineAsyncComponent } from 'vue';
import { emit, listen } from '@tauri-apps/api/event';
import { WebviewWindow } from '@tauri-apps/api/webviewWindow';
import { useI18n } from 'vue-i18n';
import { useUIStore } from '@/stores/uiStore';
import { getAlbum, getQueryCountAndSum, getQueryFiles, getFolderFiles, getFolderThumbCount, getTagName,
         copyImage, renameFile, moveFile, copyFile, editFileComment, getFileThumb, revealFolder, updateFileInfo,
         setFileFavorite, setFileRotate, getFileHasTags, deleteFile, deleteDbFile, getTagsForFile } from '@/common/api';  
import { config } from '@/common/config';
import { isWin, isMac, setTheme,
         formatFileSize, formatDate, getCalendarDateRange, getRelativePath, 
         extractFileName, combineFileName, getFolderPath, getAssetSrc, getSelectOptions, shortenFilename, formatDimensionText, formatCaptureSettings } from '@/common/utils';

import SearchBox from '@/components/SearchBox.vue';
import DropDownSelect from '@/components/DropDownSelect.vue';
import ContextMenu from '@/components/ContextMenu.vue';
import ProgressBar from '@/components/ProgressBar.vue';
import GridView  from '@/components/GridView.vue';
import Image from '@/components/Image.vue';
const Video = defineAsyncComponent(() => import('@/components/Video.vue')); // dynamic import
import MessageBox from '@/components/MessageBox.vue';
import MoveTo from '@/components/MoveTo.vue';
import ToolTip from '@/components/ToolTip.vue';
import TButton from '@/components/TButton.vue';
import TaggingDialog from '@/components/TaggingDialog.vue';
import ImageEditor from '@/components/ImageEditor.vue';
import FileInfo from '@/components/FileInfo.vue';
import ScrollBar from '@/components/ScrollBar.vue';

import {
  IconHome,
  IconLeftPaneOn,
  IconLeftPaneOff,
  IconPreview,
  IconPreviewOff,
  IconArrowDown,
  IconClose,
  IconCheckAll,
  IconCheckNone,
  IconFavorite,
  IconUnFavorite,
  IconFolderFavorite,
  IconMoveTo,
  IconFiles,
  IconFolder,
  IconSearch,
  IconChecked,
  IconComment,
  IconTag,
  IconCalendar,
  IconLocation,
  IconCamera,
  IconTrash,
  IconError,
  IconPhoto,
  IconVideo,
  IconCameraAperture,
  IconFileSearch,
  IconGallery,
  IconGrid,
  IconLeft,
  IconRight,
  IconSeparator,
} from '@/common/icons';

const thumbnailPlaceholder = new URL('@/assets/images/image-file.png', import.meta.url).href;

const props = defineProps({
  titlebar: String
});

/// i18n
const { locale, messages } = useI18n();
const localeMsg = computed(() => messages.value[locale.value] as any);
const uiStore = useUIStore();

// title of the content 
const contentIcon = computed(() => {
  const index = config.home.sidebarIndex;
  
  switch (index) {
    case 0: return IconHome;
    case 1: return IconFolder;
    case 2: return config.favorite.folderId && config.favorite.folderId > 0 ? IconFolderFavorite : IconFavorite;
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

// config.favorite.folderId = 0: means favorite files
const showFolderFiles = computed(() =>
 config.home.sidebarIndex === 1 || (config.home.sidebarIndex === 2 && config.favorite.folderId !== 0)
);

// mutil select mode
const selectMode = ref(false);
const selectedCount = ref(0);
const selectedSize = ref(0);  // selected files size

// film strip view splitter
const isDraggingFilmStripView = ref(false);      // dragging splitter to resize film strip view
const filmStripViewZoomFit = ref(true); // film strip view zoom fit
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
const gap = 4;                    // Gap between items (must match GridView)

const itemSize = computed(() => config.settings.grid.size + gap);

const columnCount = computed(() => {
  if (containerWidth.value <= 0 || itemSize.value <= 0) return 4;
  return Math.max(1, Math.floor(containerWidth.value / itemSize.value));
});

const visibleItemCount = computed(() => {
  if (containerHeight.value <= 0 || itemSize.value <= 0) return 20;
  const rows = Math.floor(containerHeight.value / itemSize.value);
  return rows * columnCount.value;
});

const timelineMarkers = ref<{ label: string, position: number }[]>([]);  // timeline markers for scrollbar

const toolTipRef = ref<any>(null);
const isProcessing = ref(false);  // show processing status

const searchBoxRef = ref<any>(null);

// Store current query params for virtual scrolling
const currentQueryParams = ref({
  searchFolder: "",
  startDate: "",
  endDate: "",
  make: "",
  model: "",
  locationAdmin1: "",
  locationName: "",
  isFavorite: false,
  tagId: 0
});

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

function handleItemDblClicked(index: number) {
  selectedItemIndex.value = index;
  openImageViewer(index, true);
}

function handleItemSelectToggled(index: number) {
  fileList.value[index].isSelected = !fileList.value[index].isSelected;
}

function handleItemAction(payload: { action: string, index: number }) {
  const { action, index } = payload;
  selectedItemIndex.value = index; // Ensure the item for the action is selected

  const actionMap = {
    'open': () => openImageViewer(selectedItemIndex.value, true),
    'edit': () => showImageEditor.value = true,
    'copy': () => clickCopyImage(fileList.value[selectedItemIndex.value].file_path),
    'update-from-file': () => updateFile(fileList.value[selectedItemIndex.value]),
    'rename': () => {
      renamingFileName.value = extractFileName(fileList.value[selectedItemIndex.value].name);
      showRenameMsgbox.value = true;
    },
    'move-to': () => showMoveTo.value = true,
    'copy-to': () => showCopyTo.value = true,
    'trash': () => showTrashMsgbox.value = true,
    'goto-folder': () => {
      const selectedFile = fileList.value[selectedItemIndex.value];
      if (selectedFile) {
        config.album.id = selectedFile.album_id;
        config.album.folderId = selectedFile.folder_id;
        config.album.folderPath = getFolderPath(selectedFile.file_path);
        config.home.sidebarIndex = 1;
      }
    },
    'reveal': () => revealFolder(getFolderPath(fileList.value[selectedItemIndex.value].file_path)),
    'favorite': toggleFavorite,
    'rotate': clickRotate,
    'tag': clickTag,
    'comment': () => showCommentMsgbox.value = true,
  };

  if ((actionMap as any)[action]) {
    (actionMap as any)[action]();
  }
}

function handleNavigate(direction: 'prev' | 'next') {
  if (direction === 'next') {
    selectedItemIndex.value = Math.min(selectedItemIndex.value + 1, fileList.value.length - 1);
  } else {
    selectedItemIndex.value = Math.max(selectedItemIndex.value - 1, 0);
  }
}

const handleWheel = (event: WheelEvent) => {
  if (config.content.layout !== 1) return;
  const el = gridScrollContainerRef.value;
  if (el) {
    const delta = Math.abs(event.deltaX) > Math.abs(event.deltaY) ? event.deltaX : event.deltaY;
    el.scrollLeft += delta;
    event.preventDefault();
  }
};

function handleGridScroll(event: any) {
  if (event && event.target) {
    const scrollTop = event.target.scrollTop;
    // Convert scrollTop (pixels) to item index
    const rowIndex = Math.floor(scrollTop / itemSize.value);
    scrollPosition.value = rowIndex * columnCount.value;
  }
}

function handleScrollUpdate(newIndex: number) {
  scrollPosition.value = newIndex;
  // Convert item index to scrollTop (pixels)
  const rowIndex = Math.floor(newIndex / columnCount.value);
  const newScrollTop = rowIndex * itemSize.value;

  if (config.content.layout === 0 && gridViewRef.value) {
    gridViewRef.value.scrollToPosition(newScrollTop);
  } else if (gridScrollContainerRef.value) {
    gridScrollContainerRef.value.scrollTop = newScrollTop;
  }
}

function handleRequestScroll(index: number) {
  if (isDraggingInfoPanel.value || isDraggingFilmStripView.value) return;

  // Using setTimeout to ensure the DOM has been fully updated and rendered
  setTimeout(() => {
    if (config.content.layout === 0 && gridViewRef.value) {
       gridViewRef.value.scrollToItem(index);
    } else if (config.content.layout === 1) {
       // Existing logic for Filmstrip
       const item = document.getElementById(`item-${index}`);
       const container = gridScrollContainerRef.value;
       if (!item || !container) return;
       
       const containerRect = container.getBoundingClientRect();
       const itemRect = item.getBoundingClientRect();

       const newScrollLeft = container.scrollLeft + (itemRect.left - containerRect.left) - (containerRect.width / 2) + (itemRect.width / 2);

       container.scrollTo({
         left: newScrollLeft,
         behavior: 'smooth',
       });
    }
  }, 100);
}

// function handleScroll() {
//   // This is now mainly for Filmstrip view (Layout 1) which uses native scrolling on container
//   const el = gridScrollContainerRef.value;
//   if (!el) return;

//   if (config.content.layout === 1) { // layout 1: carousel
//     if (el.scrollLeft + el.clientWidth >= el.scrollWidth - 200) {
//       handleNextPage();
//     }
//   }
// }

// function handleNextPage() {
// }

// Keyboard navigation actions
const keyActions = {
  ArrowDown: () => {
    if (gridViewRef.value) {
      selectedItemIndex.value = Math.min(selectedItemIndex.value + gridViewRef.value.getColumnCount(), fileList.value.length - 1);
    }
  },
  ArrowRight: () => selectedItemIndex.value = Math.min(selectedItemIndex.value + 1, fileList.value.length - 1),
  ArrowUp: () => {
    if (gridViewRef.value) {
      selectedItemIndex.value = Math.max(selectedItemIndex.value - gridViewRef.value.getColumnCount(), 0);
    }
  },
  ArrowLeft: () => selectedItemIndex.value = Math.max(selectedItemIndex.value - 1, 0),
  Home: () => selectedItemIndex.value = 0,
  End: () => selectedItemIndex.value = fileList.value.length - 1,
  '/': () => searchBoxRef.value.focusInput(),
};

// Local keydown handler for navigation (prevents default browser behavior)
function handleLocalKeyDown(event: KeyboardEvent) {
  // Keys that we are handling manually for navigation, to prevent default browser behavior (scrolling).
  const handledKeys = ['ArrowUp', 'ArrowDown', 'ArrowLeft', 'ArrowRight', 'Home', 'End', ' '];

  if (handledKeys.includes(event.key)) {
    event.preventDefault();
  }
}

// Global keydown handler (from Tauri)
const handleKeyDown = (e: any) => {
  if (uiStore.inputStack.length > 0) {
    return;
  }

  const { key, metaKey } = e.payload;
  const isCmdKey = isMac ? metaKey : e.payload.ctrlKey;

  if (isCmdKey && key === 'Enter') {   // Open shortcut
    openImageViewer(selectedItemIndex.value, true);
  } else if (isCmdKey && key.toLowerCase() === 'c') {   // Copy shortcut
    clickCopyImage(fileList.value[selectedItemIndex.value].file_path);
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

onMounted( async() => {
  unlistenKeydown = await listen('global-keydown', handleKeyDown);

  unlistenImageViewer = await listen('message-from-image-viewer', (event) => {
    const { message } = event.payload as any;
    switch (message) {
      case 'home':
        selectedItemIndex.value = 0;
        break;
      case 'end':
        selectedItemIndex.value = fileList.value.length - 1;
        break;
      case 'prev':
        selectedItemIndex.value = Math.max(selectedItemIndex.value - 1, 0);
        break;
      case 'next':
        selectedItemIndex.value = Math.min(selectedItemIndex.value + 1, fileList.value.length - 1);
        break;
      case 'trash':
        clickTrashFile();
        break;
      case 'favorite':
        toggleFavorite();
        break;
      case 'rotate':
        clickRotate();
        break;
      case 'update-tags':
        updateFileHasTags([fileList.value[selectedItemIndex.value].id]);
        break;
      default:
        break;
    }
  });
});

onBeforeUnmount(() => {
  // unlisten
  unlistenImageViewer();
  if (unlistenKeydown) {
    unlistenKeydown();
  }
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

/// watch for file list changes
watch(
  () => [
    config.home.sidebarIndex,      // toolbar index
    config.favorite.albumId, config.favorite.folderId, config.favorite.folderPath,   // favorite files and folder
    config.album.id, config.album.folderId, config.album.folderPath,                 // album
    config.tag.id,                                                                 // tag
    config.calendar.year, config.calendar.month, config.calendar.date,               // calendar
    config.camera.make, config.camera.model,                                        // camera 
    config.location.admin1, config.location.name,                                   // location
    config.search.text, config.search.fileType, config.search.sortType, config.search.sortOrder,  // search and sort 
  ], 
  () => {
    scrollPosition.value = 0;   // reset file scroll position
    updateContent();
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
watch(() => [config.content.layout, config.infoPanel.show, config.infoPanel.tabIndex], ([newLayout, newShow, newTabIndex]) => {
  if (newLayout === 1 || (newShow && newTabIndex === 1)) {
    updateSelectedImage(selectedItemIndex.value);
  }
});

function toggleGridViewLayout() {
  config.content.layout = config.content.layout === 0 ? 1 : 0;
  filmStripViewZoomFit.value = true;
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

  console.log(`fetchDataRange: ${start}-${end}, chunks: ${startChunk}-${endChunk}`);

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
      console.log(`Fetching chunk: ${key}`);
      
      // Fetch data
      const { searchFolder, startDate, endDate, make, model, locationAdmin1, locationName, isFavorite, tagId } = currentQueryParams.value;
      
      // We don't await here to allow parallel fetching of chunks, 
      // but we track pending requests to avoid duplicate fetches.
      getQueryFiles(searchFolder, startDate, endDate, make, model, locationAdmin1, locationName, isFavorite, tagId, chunkStart, chunkSize)
        .then(newFiles => {
          console.log(`Chunk loaded: ${key}, items: ${newFiles?.length}`);
          if (newFiles) {
            // Update fileList and collect reactive references
            const filesToFetch = [];
            for (let j = 0; j < newFiles.length; j++) {
              if (chunkStart + j < fileList.value.length) {
                fileList.value[chunkStart + j] = newFiles[j];
                filesToFetch.push(fileList.value[chunkStart + j]);
              }
            }
            // Fetch thumbnails for these files
            if (filesToFetch.length > 0) {
                console.log(`Fetching thumbnails for chunk: ${key}, count: ${filesToFetch.length}`);
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

function handleVisibleRangeUpdate({ startIndex, endIndex }: { startIndex: number, endIndex: number }) {
  // Fetch data for visible range + buffer
  console.log(`handleVisibleRangeUpdate: ${startIndex}-${endIndex}`);
  const buffer = 200;
  fetchDataRange(startIndex - buffer, endIndex + buffer);
}

// get file list 
async function getFileList(
  searchFolder: string, 
  startDate: string, 
  endDate: string, 
  make: string, 
  model: string, 
  locationAdmin1: string, 
  locationName: string, 
  isFavorite: boolean, 
  tagId: number, 
  offset = 0, 
  limit = -1
) { 
  // Update current query params
  currentQueryParams.value = {
    searchFolder, 
    startDate, 
    endDate, 
    make, 
    model, 
    locationAdmin1,   
    locationName, 
    isFavorite, 
    tagId
  };

  // Reset file list
  fileList.value = [];
  totalFileCount.value = 0;
  totalFileSize.value = 0;
  
  // Get count and sum first
  const result = await getQueryCountAndSum(
    searchFolder, 
    startDate, 
    endDate, 
    make, 
    model, 
    locationAdmin1, 
    locationName, 
    isFavorite, 
    tagId
  );
  
  if (result) {
    totalFileCount.value = result[0];
    totalFileSize.value = result[1];
    
    // Initialize fileList with placeholders
    // We use a large array. Vue 3's reactivity system can handle this, 
    // but for 100k items, shallowRef might be better for fileList if performance is an issue.
    // However, RecycleScroller needs to detect changes.
    // Let's use standard ref for now.
    fileList.value = Array.from({ length: totalFileCount.value }).map((_, i) => ({
      id: 'ph-' + i,
      isPlaceholder: true,
      name: '',
      size: 0,
    }));
    
    // Fetch initial data (first page)
    // fetchDataRange(0, 100);
  }
}

async function updateContent() {
  const newIndex = config.home.sidebarIndex;

  if(newIndex === 0) {        // home
    contentTitle.value = localeMsg.value.home.title;
    await getFileList("", "", "", "", "", "", "", false, 0);
  } 
  else if(newIndex === 1) {   // album
    if(config.album.id === null) {
      contentTitle.value = "";
      fileList.value = [];
    } else {
      const album = await getAlbum(config.album.id);
      if(album) {
        if(config.album.folderPath === album.path) { // current folder is root
          contentTitle.value = album.name;
        } else {
          contentTitle.value = album.name + getRelativePath(config.album.folderPath || "", album.path);
        };
        fileList.value = await getFolderFiles(config.album.folderId, config.album.folderPath);
        totalFileCount.value = fileList.value.length;
        totalFileSize.value = fileList.value.reduce((total, file) => total + file.size, 0);

        // get the thumbnail count
        await getFolderThumbCount(config.album.folderId).then(count => {
          console.log('updateContent - thumbCount:', count);
          showProgressBar.value = count < fileList.value.length; // show progress bar if the thumbnail count is less than the file list length
        });

        // always get all thumbnail for a folder (gen thumbnail if not exist)
        getFileListThumb(fileList.value);
      } else {
        contentTitle.value = "";
        fileList.value = [];
      }
    }
  }
  else if(newIndex === 2) {   // favorite
    if(config.favorite.folderId === null) {
      contentTitle.value = "";
      fileList.value = [];
    } else {
      if(config.favorite.folderId === 0) { // favorite files
        contentTitle.value = localeMsg.value.favorite.files;
        await getFileList("", "", "", "", "", "", "", true, 0);
      } else {                // favorite folders
        const album = await getAlbum(config.favorite.albumId);
        if(album) {
          contentTitle.value = localeMsg.value.favorite.folders + getRelativePath(config.favorite.folderPath, album.path);
          await getFileList(config.favorite.folderPath, "", "", "", "", "", "", false, 0);
        } else {
          contentTitle.value = "";
          fileList.value = [];
        }
      }
    }
  }
  else if(newIndex === 3) {   // tag
    if (config.tag.id === null) {
      contentTitle.value = "";
      fileList.value = [];
    } else {
      const tagName = await getTagName(config.tag.id);
      if (tagName) {
        contentTitle.value = tagName;
        await getFileList("", "", "", "", "", "", "", false, config.tag.id);
      } else {
        contentTitle.value = "";
        fileList.value = [];
      }
    }
  }
  else if(newIndex === 4) {   // calendar
    if(config.calendar.year === null) {
      contentTitle.value = "";
      fileList.value = [];
    } else {
      if (config.calendar.month === -1) {          // yearly
        contentTitle.value = formatDate(config.calendar.year, 1, 1, localeMsg.value.format.year);
      } else if (config.calendar.date === -1) {    // monthly
        contentTitle.value = formatDate(config.calendar.year, config.calendar.month, 1, localeMsg.value.format.month);
      } else {                                    // daily
        contentTitle.value = formatDate(config.calendar.year, config.calendar.month, config.calendar.date, localeMsg.value.format.date_long);
      }
      const [startDate, endDate] = getCalendarDateRange(config.calendar.year, config.calendar.month, config.calendar.date);
      await getFileList("", startDate, endDate, "", "", "", "", false, 0);
    }
  }
  else if(newIndex === 5) {   // location
    if(config.location.admin1 === null) {
      contentTitle.value = "";
      fileList.value = [];
    } else {
      if(config.location.name) {
        contentTitle.value = `${config.location.admin1} > ${config.location.name}`;
        await getFileList("", "", "", "", "", config.location.admin1, config.location.name, false, 0);
      } else {
        contentTitle.value = `${config.location.admin1}`;
        await getFileList("", "", "", "", "", config.location.admin1, "", false, 0);
      } 
    }
  }
  else if(newIndex === 6) {   // camera
    if(config.camera.make === null) {
      contentTitle.value = "";
      fileList.value = [];
    } else {
      if(config.camera.model) {
        contentTitle.value = `${config.camera.make} > ${config.camera.model}`;
        await getFileList("", "", "", config.camera.make, config.camera.model, "", "", false, 0);
      } else {
        contentTitle.value = `${config.camera.make}`;
        await getFileList("", "", "", config.camera.make, "", "", "", false, 0);
      } 
    }
  } 

  if(config.search.text) {
    contentTitle.value += ' - ' + localeMsg.value.toolbar.search.title + ': ' + config.search.text;
  }

  refreshFileList();
}

// update the file info from the image editor
const onImageEdited = () => {
  console.log('onImageEdited:', fileList.value[selectedItemIndex.value]);
  updateFile(fileList.value[selectedItemIndex.value]);

  showImageEditor.value = false;
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
function removeFromFileList(index: number) {
  // remove file from list
  fileList.value.splice(index, 1);
  
  // update total file count and size
  totalFileCount.value = fileList.value.length;
  totalFileSize.value = fileList.value.reduce((total, file) => total + file.size, 0);
  
  // update selected item index
  selectedItemIndex.value = Math.min(index, fileList.value.length - 1);
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
    if (success) {
      toolTipRef.value.showTip(localeMsg.value.tooltip.update_image.success);
    } else {
      toolTipRef.value.showTip(localeMsg.value.tooltip.update_image.failed, true);
    }
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
    
    // notify the image viewer
    emit('message-from-content', { message: 'favorite', favorite: item.is_favorite });

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

        // notify the image viewer
        if(selectedItemIndex.value === fileList.value.indexOf(item)) {
          emit('message-from-content', { message: 'favorite', favorite: item.is_favorite });
        }

        // update the favorite status in the database
        return setFileFavorite(item.id, isFavorite);
      }); 
    await Promise.all(updates); // parallelize DB updates
  }
}

// set file rotate
const clickRotate = async () => {
  if (selectedItemIndex.value >= 0) {
    fileList.value[selectedItemIndex.value].rotate += 90;

    // notify the image viewer
    emit('message-from-content', { message: 'rotate' });

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
  selectMode.value = value;
  if(!selectMode.value) {
    for (let i = 0; i < fileList.value.length; i++) {
      fileList.value[i].isSelected = false;
    }
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

function refreshFileList() {
  selectMode.value = false;   // exit multi-select mode

  if(fileList.value.length > 0) {
    if(scrollPosition.value === 0) {  // reset file scroll position
      selectedItemIndex.value = 0;
      updateSelectedImage(selectedItemIndex.value);
    }
  } else {
    selectedItemIndex.value = -1;
  }
  console.log('refreshFileList:', fileList.value);
}

// update image when the select file is changed
async function updateSelectedImage(index: number) {
  if(index < 0 || index >= fileList.value.length) return;

  // update the tags for the selected file
  if(config.infoPanel.show && fileList.value[index].has_tags) {
    fileList.value[index].tags = await getTagsForFile(fileList.value[index].id);
  }

  // update image viewer if the viewer is open
  openImageViewer(index, false);
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
      config.settings.filmStripView.previewPosition === 0 ? 
        gridViewDivRect.bottom - event.clientY - 2 - verticalSpacing :
        event.clientY - gridViewDivRect.top - 2 - verticalSpacing; // -2: border width(2px)
    
    const totalHeight = gridViewDiv.value.clientHeight;
    const minHeight = Math.max(totalHeight * 0.1, 120);
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

/// stop dragging the splitter
function stopDragging() {
  isDraggingFilmStripView.value = false;
  isDraggingInfoPanel.value = false;
  document.removeEventListener('mousemove', handleMouseMove);
  document.removeEventListener('mouseup', stopDragging);
}
</script>

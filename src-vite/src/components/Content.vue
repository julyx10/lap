<template>

  <div class="flex-1 flex flex-col select-none"
    :class="{ 'opacity-50 pointer-events-none': uiStore.isInputActive('AlbumList-edit') }"
  >

    <!-- Loading overlay -->
    <transition name="fade">
      <div v-if="isProcessing" class="absolute inset-0 bg-base-100/50 flex items-center justify-center z-50 rounded-box">
        <span class="loading loading-dots text-primary"></span>
      </div>
    </transition>

    <!-- title bar -->
    <div class="relative px-2 pt-1 min-h-12 flex flex-row flex-wrap items-center justify-between" data-tauri-drag-region>


      <!-- title -->
      <div v-if="contentTitle.length > 0" class="flex flex-row items-center min-w-0 flex-1" data-tauri-drag-region>
        <component :is=contentIcon class="mr-2 t-icon-size-sm shrink-0"/>
        <div class="mr-2 cursor-default overflow-hidden whitespace-pre text-ellipsis">
          {{ contentTitle }}
        </div>
      </div>

      <!-- toolbar -->
      <div class="ml-auto h-6 flex flex-row items-center space-x-2 shrink-0">

        <!-- search box - filter file name -->
        <SearchBox ref="searchBoxRef" v-model="config.search.text" @click.stop="selectMode = false" /> 

        <!-- select mode -->
        <div tabindex="-1"
          :class="[
            'px-2 py-1 h-8 flex flex-row items-center rounded-md border border-content focus:outline-none text-sm shrink-0 cursor-pointer',
            selectMode ? 'border-primary' : 'border-base-content/30 hover:border-base-content/70'
          ]"
          @click="handleSelectMode(true)"
        >
          <TButton v-if="selectMode"
            :icon="IconClose"
            :buttonSize="'small'"
            @click.stop="handleSelectMode(false)" 
          />
          <span class="px-1">{{ selectMode ? $t('toolbar.filter.select_count', { count: selectedCount }) : $t('toolbar.filter.select_mode') }}</span>
          <DropDownMenu v-if="selectMode"
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

        <div class="flex flex-row items-center">
          <!-- grid view layout -->
          <TButton
            :icon="config.content.layout === 0 ? IconGrid : IconGallery"
            @click="config.content.layout = config.content.layout === 0 ? 1 : 0"
          />

          <!-- show info -->
          <TButton
            :icon="IconInformation"
            :selected="config.content.showFileInfo"
            @click="config.content.showFileInfo = !config.content.showFileInfo"
          />
        </div>
      </div>
    </div>

    <ProgressBar v-if="config.home.sidebarIndex === 1 && fileList.length > 0 && showProgressBar" :percent="Number(((thumbCount / fileList.length) * 100).toFixed(0))" />
    <span v-else class="h-0.5 w-full"></span>

    <div ref="contentViewDiv" class="mt-1 flex-1 flex flex-row overflow-hidden">

      <div ref="gridViewDiv" class="flex-1 flex flex-col">
        <!-- preview -->
        <div v-show="config.content.layout === 1" ref="previewDiv" 
          class="p-1 rounded-ss-lg overflow-hidden bg-base-200"
          :style="{ height: (gridViewDiv?.clientHeight - config.content.galleryPaneHeight) + 'px' }"
        >
          <div v-if="selectedItemIndex >= 0 && selectedItemIndex < fileList.length"
            class="relative w-full h-full flex items-center justify-center"
            :style="{ width: previewPaneSize.width + 'px', height: previewPaneSize.height + 'px' }"
            @dblclick="openImageViewer(selectedItemIndex, true)"
          >
            <template v-if="fileList[selectedItemIndex]?.file_type === 1">
              <Image v-if="imageSrc"
                :src="imageSrc" 
                :rotate="fileList[selectedItemIndex]?.rotate ?? 0" 
                :isZoomFit="true"
              ></Image>
              <div v-if="loadImageError" class="h-full flex flex-col items-center justify-center text-base-content/30">
                <IconError class="w-8 h-8 mb-2" />
                <span>{{ $t('image_viewer.failed') }}</span>
              </div>
            </template>

            <template v-if="fileList[selectedItemIndex]?.file_type === 2">
              <Video v-if="videoSrc"
                ref="videoRef"
                :src="videoSrc"
                :rotate="fileList[selectedItemIndex]?.rotate ?? 0"
                :isZoomFit="true"
              ></Video>
            </template>
          </div>

          <div v-else class="h-full flex flex-col items-center justify-center text-base-content/30">
            <IconSearch class="w-8 h-8" />
            <span>{{ $t('tooltip.not_found.files') }}</span>
          </div>
        </div>

        <!-- gallery view splitter -->
        <div v-if="config.content.layout === 1" 
          :class="[ 
            'h-1 hover:bg-base-content/70 cursor-ns-resize transition-colors',
            isDraggingGalleryView ? 'bg-base-content/70' : 'bg-base-200'
          ]" 
          @mousedown="startDraggingGalleryView"
        ></div>

        <!-- grid view -->
        <GridView ref="gridViewRef"
          v-model:selectItemIndex="selectedItemIndex"
          :fileList="fileList"
          :showFolderFiles="showFolderFiles"
          :selectMode="selectMode"
        />
      </div>

      <!-- file info splitter -->
      <div v-if="config.content.showFileInfo" 
        :class="[
          'w-1 hover:bg-base-content/70 cursor-ew-resize transition-colors', 
          isDraggingFileInfo ? 'bg-base-content/70' : 'bg-base-200'
        ]" 
        @mousedown="startDraggingFileInfoSplitter"
      ></div>

      <!-- file info pane -->
      <transition
        enter-active-class="transition-transform duration-200"
        leave-active-class="transition-transform duration-200"
        enter-from-class="translate-x-full"
        enter-to-class="translate-x-0"
        leave-from-class="translate-x-0"
        leave-to-class="translate-x-full"
      >
        <div v-if="config.content.showFileInfo" :style="{ width: config.content.fileInfoPaneWidth + '%' }">
          <FileInfo :fileInfo="fileList[selectedItemIndex]" @close="config.content.showFileInfo = false" />
        </div>
      </transition>

    </div>

    <!-- status bar -->
    <div v-if="config.settings.showStatusBar && totalFileCount > 0"
      class="p-2 border-t border-base-content/30 flex gap-4 text-sm cursor-default"
    >
      <div class="flex items-center gap-1 flex-shrink-0">
        <IconFileSearch class="t-icon-size-xs" />
        <span >
          {{ $t('statusbar.files_summary', { count: totalFileCount.toLocaleString(), size: formatFileSize(totalFileSize) }) }}
          {{ hasMoreFiles ? '...' : '' }}
        </span>
      </div>

      <template v-if="selectedItemIndex >= 0">
        <div class="flex items-center gap-1 flex-shrink-0">
          <component :is="selectMode ? IconCheckAll : IconChecked" class="t-icon-size-xs" />
          <span>
            {{ selectMode
              ? $t('toolbar.filter.select_count', { count: selectedCount.toLocaleString() }) + ' (' + formatFileSize(selectedSize) + ')'
              : shortenFilename(fileList[selectedItemIndex]?.name) + ' (' + formatFileSize(fileList[selectedItemIndex]?.size) + ')'
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
    :title="`${$t('msgbox.move_to.title', { source: selectMode ? $t('toolbar.filter.select_count', { count: selectedCount.toLocaleString() }) : fileList[selectedItemIndex].name })}`"
    :message="$t('msgbox.move_to.content')"
    :OkText="$t('msgbox.move_to.ok')" 
    :cancelText="$t('msgbox.cancel')"
    @ok="clickMoveTo"
    @cancel="showMoveTo = false"
  />

  <!-- copy to -->
  <MoveTo
    v-if="showCopyTo"
    :title="`${$t('msgbox.copy_to.title', { source: selectMode ? $t('toolbar.filter.select_count', { count: selectedCount.toLocaleString() }) : fileList[selectedItemIndex].name })}`"
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

import { ref, watch, computed, onMounted, onBeforeUnmount, onUnmounted, defineAsyncComponent } from 'vue';
import { emit, listen } from '@tauri-apps/api/event';
import { WebviewWindow } from '@tauri-apps/api/webviewWindow';
import { useI18n } from 'vue-i18n';
import { useUIStore } from '@/stores/uiStore';
import { getAlbum, getDbFiles, getFolderFiles, getFolderThumbCount, getTagName,
         copyImage, renameFile, moveFile, copyFile, editFileComment, getFileThumb, revealFolder, updateFileInfo,
         setFileFavorite, setFileRotate, getFileHasTags, deleteFile, deleteDbFile} from '@/common/api';  
import { config, isWin, isMac, setTheme,
         formatFileSize, formatDate, getCalendarDateRange, getRelativePath, 
         extractFileName, combineFileName, getFolderPath, getAssetSrc, getSelectOptions, shortenFilename, formatDimensionText, formatCaptureSettings } from '@/common/utils';

import SearchBox from '@/components/SearchBox.vue';
import DropDownSelect from '@/components/DropDownSelect.vue';
import DropDownMenu from '@/components/DropDownMenu.vue';
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

import {
  IconHome,
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
  IconInformation,
} from '@/common/icons';

const thumbnailPlaceholder = new URL('@/assets/images/image-file.png', import.meta.url).href;

const props = defineProps({
  titlebar: String
});

/// i18n
const { locale, messages } = useI18n();
const localeMsg = computed(() => messages.value[locale.value]);
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
const fileList = ref([]);
const fileListOffset = ref(0); // offset of the file list (for pagination)
const hasMoreFiles = ref(true); // has more files to load (for pagination)
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

// gallery view splitter
const isDraggingGalleryView = ref(false);      // dragging splitter to resize gallery view
const previewDiv = ref(null);
const previewPaneSize = ref({ width: 100, height: 100 });
const imageSrc = ref('');         // preview image source
const videoSrc = ref('');         // preview video source
const videoRef = ref(null);       // preview video reference
const loadImageError = ref(false);   // Track if there was an error loading the image

// file info splitter
const isDraggingFileInfo = ref(false);

// message box
const showRenameMsgbox = ref(false);  // show rename message box
const renamingFileName = ref({}); // extract the file name to {name, ext}

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
const gridViewRef = ref(null);

const toolTipRef = ref(null);
const isProcessing = ref(false);  // show processing status

const searchBoxRef = ref(null);

let resizeObserver;

let unlistenKeydown: () => void;
let unlistenGridView: () => void;
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

const handleKeyDown = (e: KeyboardEvent) => {
  const { key } = e.payload;
  switch (key) {
    // case ' ':
    //   config.imageViewer.showFileInfo = !config.imageViewer.showFileInfo;
    //   break;
    case 'Escape':
      if (selectMode.value) {
        handleSelectMode(false);
      }
      break;
  }
};

onMounted( async() => {
  // FIXME: ResizeObserver loop completed with undelivered notifications.
  resizeObserver = new ResizeObserver(entries => {
    for (let entry of entries) {
      previewPaneSize.value = {
        width: entry.contentRect.width,
        height: entry.contentRect.height,
      };
    }
  });

  if (previewDiv.value) {
    // Observe preview pane size changes
    resizeObserver.observe(previewDiv.value);
  }

  unlistenKeydown = await listen('global-keydown', handleKeyDown);

  unlistenGridView = await listen('message-from-grid-view', (event) => {
    const { message } = event.payload;
    switch (message) {
      case 'search':
        searchBoxRef.value.focusInput();
        break;
      case 'select':
        fileList.value[event.payload.index].isSelected = !fileList.value[event.payload.index].isSelected;
        break;
      case 'open':
        openImageViewer(selectedItemIndex.value, true);
        break;
      case 'copy': // copy image to clipboard
        clickCopyImage(fileList.value[selectedItemIndex.value].file_path);
        break;
      case 'edit':
        showImageEditor.value = true;
        break;
      case 'update-from-file':
        updateFile(fileList.value[selectedItemIndex.value]);
        break;
      case 'rename':
        renamingFileName.value = extractFileName(fileList.value[selectedItemIndex.value].name);
        showRenameMsgbox.value = true;
        break;
      case 'move-to':
        showMoveTo.value = true;
        break;
      case 'copy-to':
        showCopyTo.value = true;
        break;
      case 'trash':
        showTrashMsgbox.value = true;
        break;
      case 'goto-folder':
        const selectedFile = fileList.value[selectedItemIndex.value];
        if (selectedFile) {
          config.album.id = selectedFile.album_id;
          config.album.folderId = selectedFile.folder_id;
          config.album.folderPath = getFolderPath(selectedFile.file_path);
          config.home.sidebarIndex = 1;
        }
        break;
      case 'reveal':
        revealFolder(getFolderPath(fileList.value[selectedItemIndex.value].file_path));
        break;
      case 'favorite':
        toggleFavorite();
        break;
      case 'rotate':
        clickRotate();
        break;
      case 'tag':
        clickTag();
        break;
      case 'comment':
        showCommentMsgbox.value = true;
        break;
      case 'next-page':
        if(hasMoreFiles.value) {
          fileListOffset.value += config.content.pageSize;
        }
        break;
      default:
        break;
    }
  });

  unlistenImageViewer = await listen('message-from-image-viewer', (event) => {
    const { message } = event.payload;
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
  unlistenGridView();
  unlistenImageViewer();
  if (unlistenKeydown) {
    unlistenKeydown();
  }
});

onUnmounted(() => {
  if (resizeObserver && previewDiv.value) {
    resizeObserver.unobserve(previewDiv.value);
    resizeObserver.disconnect();
  }
});

/// watch appearance
watch(() => config.settings.appearance, (newAppearance) => {
  setTheme(newAppearance);
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
    fileListOffset.value = 0;   // reset file list offset
    updateContent();
  }, 
  { immediate: true }
);

// watch for file list size changes
watch(() => fileList.value.length, (newValue) => {
  totalFileCount.value = newValue;
  totalFileSize.value = fileList.value.reduce((total, file) => total + file.size, 0);
});

// watch for file list offset
watch(() => fileListOffset.value, (newValue) => {
  if(newValue > 0) {
    updateContent();
  }
});

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

// watch for show preview on/off
watch(() => config.content.layout, (newValue) => {
  if(newValue === 1) {
    if(fileList.value[selectedItemIndex.value].file_type === 1) {
      getImageSrc(selectedItemIndex.value);
    } else if(fileList.value[selectedItemIndex.value].file_type === 2) {
      getVideoSrc(selectedItemIndex.value);
    }
  } else {
    imageSrc.value = '';
    videoSrc.value = '';
  }
  gridViewRef.value.scrollToItem(selectedItemIndex.value); 
});

async function getFileList(searchFolder, startDate, endDate, make, model, locationAdmin1, locationName, isFavorite, tagId, offset) { 
  const newFiles = await getDbFiles(searchFolder, startDate, endDate, make, model, locationAdmin1, locationName, isFavorite, tagId, offset);
  hasMoreFiles.value = newFiles.length === config.content.pageSize;

  if (offset === 0) {
    fileList.value = newFiles;
  } else {
    fileList.value.push(...newFiles);
  }
}

async function updateContent() {
  const newIndex = config.home.sidebarIndex;

  if(newIndex === 0) {        // home
    contentTitle.value = localeMsg.value.home.title;
    await getFileList("", "", "", "", "", "", "", false, 0, fileListOffset.value);
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
          contentTitle.value = album.name + getRelativePath(config.album.folderPath, album.path);
        };
        fileList.value = await getFolderFiles(config.album.folderId, config.album.folderPath);
        hasMoreFiles.value = false;  // getFolderFiles always get all files

        // get the thumbnail count
        await getFolderThumbCount(config.album.folderId).then(count => {
          console.log('updateContent - thumbCount:', count);
          showProgressBar.value = count < fileList.value.length; // show progress bar if the thumbnail count is less than the file list length
        });
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
        await getFileList("", "", "", "", "", "", "", true, 0, fileListOffset.value);
      } else {                // favorite folders
        const album = await getAlbum(config.favorite.albumId);
        if(album) {
          contentTitle.value = localeMsg.value.favorite.folders + getRelativePath(config.favorite.folderPath, album.path);
          await getFileList(config.favorite.folderPath, "", "", "", "", "", "", false, 0, fileListOffset.value);
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
        await getFileList("", "", "", "", "", "", "", false, config.tag.id, fileListOffset.value);
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
      await getFileList("", startDate, endDate, "", "", "", "", false, 0, fileListOffset.value);
    }
  }
  else if(newIndex === 5) {   // location
    if(config.location.admin1 === null) {
      contentTitle.value = "";
      fileList.value = [];
    } else {
      if(config.location.name) {
        contentTitle.value = `${config.location.admin1} > ${config.location.name}`;
        await getFileList("", "", "", "", "", config.location.admin1, config.location.name, false, 0, fileListOffset.value);
      } else {
        contentTitle.value = `${config.location.admin1}`;
        await getFileList("", "", "", "", "", config.location.admin1, "", false, 0, fileListOffset.value);
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
        await getFileList("", "", "", config.camera.make, config.camera.model, "", "", false, 0, fileListOffset.value);
      } else {
        contentTitle.value = `${config.camera.make}`;
        await getFileList("", "", "", config.camera.make, "", "", "", false, 0, fileListOffset.value);
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
async function deleteFileAlways(file) {
  const deletedFile = await deleteFile(file.id, file.file_path);
  if(deletedFile) {
    console.log('clickDeleteFile - trashed file:', file.file_path);
  } else {
    console.log('clickDeleteFile - delete db file:', file.file_path);
    await deleteDbFile(file.id);
  }
}

// remove an file item from the list and update the selected item index
function removeFromFileList(index) {
  fileList.value.splice(index, 1);
  selectedItemIndex.value = Math.min(index, fileList.value.length - 1);
  // update the preview
  if(config.content.layout === 1 && selectedItemIndex.value >= 0) {
    if(fileList.value[selectedItemIndex.value].file_type === 1) {
      getImageSrc(selectedItemIndex.value);
    } else if(fileList.value[selectedItemIndex.value].file_type === 2) {
      getVideoSrc(selectedItemIndex.value);
    }
  }
}

// update the file info from the file
const updateFile = async (file) => {
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
const updateThumbForFile = async (file) => {
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
const clickEditComment = async (newComment) => {
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

const handleSelectMode = (value) => {
  selectMode.value = value;
  if(!selectMode.value) {
    for (let i = 0; i < fileList.value.length; i++) {
      fileList.value[i].isSelected = false;
    }
  }
};

const handleFileTypeSelect = (option, extendOption) => {
  selectMode.value = false;   // exit multi-select mode
  config.search.fileType = option;
};

const handleSortTypeSelect = (option, extendOption) => {
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

// get selected image source
const getImageSrc = async (index) => {
  if(index < 0 || index >= fileList.value.length) {
    imageSrc.value = '';
    return;
  }
  
  let filePath = fileList.value[index].file_path;
  console.log('getImageSrc:', filePath);
  try {
    loadImageError.value = false;
    
    let currentIndex = index;
    const convertedSrc = getAssetSrc(filePath);
    if(!convertedSrc) {
      imageSrc.value = '';
      loadImageError.value = true;
      return;
    }
    // Check if the selected item has changed since the invocation
    if (currentIndex === index) {
      // imageSrc.value = `data:image/jpeg;base64,${convertedSrc}`;
      imageSrc.value = convertedSrc;
    }
  } catch (error) {
    imageSrc.value = '';
    loadImageError.value = true;
    console.error('getImageSrc error:', error);
  }
}

// get selected video source
const getVideoSrc = async (index) => {
  if(index < 0 || index >= fileList.value.length) {
    videoSrc.value = '';
    return;
  }

  let filePath = fileList.value[index].file_path;
  try {
    const convertedSrc = getAssetSrc(filePath);
    console.log('Content getVideoSrc - converted src:', convertedSrc);
    videoSrc.value = convertedSrc;
  } catch (error) {
    videoSrc.value = '';
    console.error('Content getVideoSrc error:', error);
  }
}

function refreshFileList() {
  selectMode.value = false;   // exit multi-select mode

  if(fileList.value.length > 0) {
    if(fileListOffset.value === 0) {  // file list is changed
      selectedItemIndex.value = 0;
      updateSelectedImage(selectedItemIndex.value);
    }
    
    getFileListThumb(fileList.value, fileListOffset.value); 
  } else {
    selectedItemIndex.value = -1;
  }
  console.log('refreshFileList:', fileList.value);
}

// update image when the select file is changed
function updateSelectedImage(index) {
  // scroll to the selected item
  gridViewRef.value.scrollToItem(index); 

  // update the preview
  if(config.content.layout === 1) {
    if(index < 0 || index >= fileList.value.length || !fileList.value[index].file_type) {
      return;
    }
    if(fileList.value[index].file_type === 1) {
      getImageSrc(index);
    } else if(fileList.value[index].file_type === 2) {
      getVideoSrc(index);
    }
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
async function getFileListThumb(files, offset, concurrencyLimit = 8) {
  const result = [];
  let activeRequests = 0;
  thumbCount.value = 0;

  const getThumbForFile = async (file) => {
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

  const runWithConcurrencyLimit = async (files) => {
    const queue = [];

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
  const encodedFilePath = file ? encodeURIComponent(file.file_path) : '';

  // preload the next image for smooth transition
  const nextFile = index + 1 >= 0 && index + 1 < fileCount ? fileList.value[index + 1] : null;
  const nextEncodedFilePath = nextFile ? encodeURIComponent(nextFile.file_path) : '';
  
  // create a new window if it doesn't exist
  let imageWindow = await WebviewWindow.getByLabel(webViewLabel);
  if (!imageWindow) {
    if (newViewer) {
      imageWindow = new WebviewWindow(webViewLabel, {
        url: `/image-viewer?fileId=${fileId}&fileIndex=${index}&fileCount=${fileCount}` + 
                           `&filePath=${encodedFilePath}&nextFilePath=${nextEncodedFilePath}`,
        title: 'Image Viewer',
        width: 1200,
        height: 800,
        minWidth: 800,
        minHeight: 600,
        resizable: true,
        decorations: isMac,
        transparent: isWin,
        ...(isMac && {
          titleBarStyle: 'Overlay',
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
      filePath: encodedFilePath, 
      nextFilePath: nextEncodedFilePath,
    });
    if(newViewer) {
      imageWindow.show();
    }
    videoRef.value?.pause();  // pause video playing in preview pane
  }
}

/// Dragging the gallery view splitter
function startDraggingGalleryView(event: MouseEvent) {
  isDraggingGalleryView.value = true;
  document.addEventListener('mousemove', handleMouseMove);
  document.addEventListener('mouseup', stopDragging);
}

/// Dragging the file info splitter
function startDraggingFileInfoSplitter(event: MouseEvent) {
  isDraggingFileInfo.value = true;
  document.addEventListener('mousemove', handleMouseMove);
  document.addEventListener('mouseup', stopDragging);
}

/// handle mouse move event
function handleMouseMove(event: MouseEvent) {
  if (isDraggingFileInfo.value) {
    const windowWidth = document.documentElement.clientWidth - 4; // -4: border width(2px) * 2
    const leftPosition = contentViewDiv.value.getBoundingClientRect().left - 2;  // -2: border width(2px)

    // Limit width between 20% and 80%
    config.content.fileInfoPaneWidth = Math.min(Math.max(((windowWidth - event.clientX)*100) / (windowWidth - leftPosition), 20), 80); 
  } else if(isDraggingGalleryView.value) {
    const gridViewDivRect = gridViewDiv.value.getBoundingClientRect();
    const newHeight = gridViewDivRect.bottom - event.clientY;
    
    const totalHeight = gridViewDiv.value.clientHeight;
    const minHeight = totalHeight * 0.2;
    const maxHeight = totalHeight * 0.8;

    config.content.galleryPaneHeight = Math.min(Math.max(newHeight, minHeight), maxHeight);
  }
}

/// stop dragging the splitter
function stopDragging() {
  isDraggingGalleryView.value = false;
  isDraggingFileInfo.value = false;
  document.removeEventListener('mousemove', handleMouseMove);
  document.removeEventListener('mouseup', stopDragging);
}
</script>

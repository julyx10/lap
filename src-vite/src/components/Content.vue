<template>

  <div class="flex-1 flex flex-col">

    <!-- title bar -->
    <div class="px-4 pt-1 min-h-12 flex flex-row items-center justify-between select-none" data-tauri-drag-region>

      <!-- title -->
      <div class="mr-2 cursor-default" data-tauri-drag-region>{{ contentTitle }}</div>

      <!-- toolbar -->
      <div class="h-6 flex flex-row items-center space-x-4 flex-shrink-0">

        <!-- search box -->
        <SearchBox ref="searchBoxRef" v-model="config.searchText" /> 
        
        <!-- select mode -->
        <button tabindex="-1"
          :class="[
            'px-2 py-1 flex flex-row items-center rounded-md border t-color-text-hover text-sm flex-shrink-0 transition-all duration-300',
            selectMode ? 't-color-border-selected' : 't-color-border t-color-border-hover'
          ]"
          @click="handleSelectMode(true)"
        >
          <IconClose v-if="selectMode" class="t-icon-size-sm t-icon-hover" @click.stop="handleSelectMode(false)" />
          <span class="px-1">{{ selectMode ? $t('file_list_select_count', { count: selectedCount }) : $t('file_list_select_mode') }}</span>
          <DropDownMenu v-if="selectMode"
            :iconMenu="IconArrowDown"
            :menuItems="moreMenuItems"
            :smallIcon="true"
            @click.stop
          />
        </button>

        <!-- sorting options -->
        <DropDownSelect
          :options="sortingOptions"
          :defaultIndex="config.sortingType"
          :extendOptions="sortingExtendOptions"
          :defaultExtendIndex="config.sortingDirection"
          @select="handleSortingSelect"
        />

        <!-- file type options -->
        <DropDownSelect
          :options="fileTypeOptions"
          :defaultIndex="config.fileType"
          @select="handleFileTypeSelect"
        />
        <!-- preview -->
        <component 
          :is="config.showPreview ? IconPreview : IconPreviewOff" 
          :class="[
            't-icon-size flex-shrink-0',
            config.showPreview ? 't-icon-focus t-icon-focus-hover': 't-icon-hover'
          ]" 
          @click="config.showPreview = !config.showPreview"
        />
      </div>
    </div>

    <ProgressBar v-if="fileList.length > 0" :percent="Number(((thumbCount / fileList.length) * 100).toFixed(0))" />

    <div ref="divListView" class="mt-1 flex-1 flex flex-row overflow-hidden">
      <div class="flex-1 flex flex-col">
        <!-- grid view -->
        <GridView  
          v-model:selectItemIndex="selectedItemIndex"
          :fileList="config.searchText.length > 0 ? searchedFileList : fileList"
          :selectMode="selectMode"
        />
        
        <!-- status bar -->
        <div v-if="config.showStatusBar" 
          class="mx-2 p-2 min-h-8 border-t border-gray-700 flex flex-row items-center justify-start text-sm select-none cursor-default"
        >
          <IconFile class="t-icon-size-xs flex-shrink-0" />
          <div class="pl-1 pr-4 whitespace-nowrap">
            {{ $t('files_summary', { count: fileList.length.toLocaleString() }) + ' (' + formatFileSize(totalFilesSize) + ')' }} 
          </div>

          <IconSearch v-if="config.searchText.length > 0" class="t-icon-size-xs flex-shrink-0" />
          <div v-if="config.searchText.length > 0" class="pl-1 pr-4 whitespace-nowrap">
            {{ $t('files_summary', { count: searchedFileList.length.toLocaleString() }) + ' (' + formatFileSize(searchedFileSize) + ')' }} 
          </div>

          <component v-if="selectedItemIndex >= 0"
            :is="selectMode ? IconCheckAll : IconChecked" 
            class="t-icon-size-xs flex-shrink-0" 
          />
          <div v-if="selectedItemIndex >=0" 
            class="px-1 w-0 flex-1 overflow-hidden whitespace-nowrap text-ellipsis"
          >
            {{
              selectMode 
                ? $t('file_list_select_count', { count: selectedCount.toLocaleString() }) + ' (' + formatFileSize(selectedSize) + ')'
                : fileList[selectedItemIndex]?.name + ' (' + formatFileSize(fileList[selectedItemIndex]?.size) + ')' 
            }}
          </div>
        </div>

      </div>

      <!-- splitter -->
      <div v-if="config.showPreview" 
        class="w-1 hover:bg-sky-700 cursor-ew-resize transition-colors" 
        @mousedown="startDragging"
      ></div>

      <!-- preview pane -->
      <transition
        enter-active-class="transition-transform duration-200"
        leave-active-class="transition-transform duration-200"
        enter-from-class="translate-x-full"
        enter-to-class="translate-x-0"
        leave-from-class="translate-x-0"
        leave-to-class="translate-x-full"
      >
        <div v-show="config.showPreview" ref="previewDiv" 
          class="p-1 t-color-bg rounded-ss-lg overflow-hidden"
          :style="{ width: config.previewPaneWidth + '%' }"
        >
          <div v-if="selectedItemIndex >= 0 && selectedItemIndex < fileList.length"
            :style="{ width: previewPaneSize.width + 'px', height: previewPaneSize.height + 'px' }"
            @dblclick="openImageViewer(selectedItemIndex, true)"
          >
            <Image ref="imageRef" 
              :src="imageSrc" 
              :rotate="fileList[selectedItemIndex]?.rotate ?? 0" 
              :isZoomFit="true"
            />
          </div>

          <div v-else class="h-full flex items-center justify-center">
            <p >{{ $t('image_view_no_image') }}</p>
          </div>
      
        </div>
      </transition>

    </div>
  </div>

  <!-- rename -->
  <MessageBox
    v-if="showRenameMsgbox"
    :title="$t('msgbox_rename_file_title')"
    :message="$t('msgbox_rename_file_content')"
    :showInput="true"
    :inputText="renamingFileName.name"
    :OkText="$t('msgbox_rename_file_ok')"
    :cancelText="$t('msgbox_cancel')"
    :errorMessage="errorMessage"
    @ok="clickRenameFile"
    @cancel="showRenameMsgbox = false"
    @reset="errorMessage = ''"
  />

  <!-- move to -->
  <MoveTo
    v-if="showMoveTo"
    :title="`${$t('msgbox_move_to_title', { source: selectMode ? $t('file_list_select_count', { count: selectedCount.toLocaleString() }) : fileList[selectedItemIndex].name })}`"
    :message="$t('msgbox_move_to_content')"
    :OkText="$t('msgbox_move_to_ok')"
    :cancelText="$t('msgbox_cancel')"
    @ok="clickMoveTo"
    @cancel="showMoveTo = false"
  />

  <!-- copy to -->
  <MoveTo
    v-if="showCopyTo"
    :title="`${$t('msgbox_copy_to_title', { source: selectMode ? $t('file_list_select_count', { count: selectedCount.toLocaleString() }) : fileList[selectedItemIndex].name })}`"
    :message="$t('msgbox_copy_to_content')"
    :OkText="$t('msgbox_copy_to_ok')"
    :cancelText="$t('msgbox_cancel')"
    @ok="clickCopyTo"
    @cancel="showCopyTo = false"
  />

  <!-- delete -->
  <MessageBox
    v-if="showDeleteMsgbox"
    :title="$t('msgbox_delete_file_title')"
    :message="`${$t('msgbox_delete_file_content', { file: selectMode ? $t('file_list_select_count', { count: selectedCount.toLocaleString() }) : fileList[selectedItemIndex].name })}`"
    :OkText="$t('msgbox_delete_file_ok')"
    :cancelText="$t('msgbox_cancel')"
    :warningOk="true"
    @ok="clickDeleteFile"
    @cancel="showDeleteMsgbox = false"
  />

  <ToolTip ref="toolTipRef" />

</template>

<script setup lang="ts">

import { ref, watch, computed, onMounted, onBeforeUnmount, onUnmounted } from 'vue';
import { emit, listen } from '@tauri-apps/api/event';
import { WebviewWindow } from '@tauri-apps/api/webviewWindow';
import { useI18n } from 'vue-i18n';
import { getAlbum, getAllFiles, getFolderFiles, getCalendarFiles, getCameraFiles,
         copyImage, renameFile, moveFile, copyFile, deleteFile, getFileThumb, revealFolder, getFileImage,
         setFileFavorite, setFileRotate } from '@/common/api';
import { config, isWin, isMac, 
         formatFileSize, formatDate, getRelativePath, localeComp, 
         extractFileName, combineFileName, getFolderPath } from '@/common/utils';

import SearchBox from '@/components/SearchBox.vue';
import DropDownSelect from '@/components/DropDownSelect.vue';
import DropDownMenu from '@/components/DropDownMenu.vue';
import ProgressBar from '@/components/ProgressBar.vue';
import GridView  from '@/components/GridView.vue';
import Image from '@/components/Image.vue';
import MessageBox from '@/components/MessageBox.vue';
import MoveTo from '@/components/MoveTo.vue';
import ToolTip from '@/components/ToolTip.vue';

import {
  IconPreview,
  IconPreviewOff,
  IconArrowDown,
  IconClose,
  IconCheckAll,
  IconCheckNone,
  IconFavorite,
  IconUnFavorite,
  IconMoveTo,
  IconDelete,
  IconFile,
  IconSearch,
  IconChecked,
} from '@/common/icons';

const props = defineProps({
  titlebar: String
});

/// i18n
const { locale, messages } = useI18n();
const localeMsg = computed(() => messages.value[locale.value]);

// title of the content 
const contentTitle = ref("");   

// progress bar
const thumbCount = ref(0);      // thumbnail count (from 0 to fileList.length)

// file list view
const divListView = ref(null);

// file list
const fileList = ref([]);
const totalFilesSize = ref(0);   // total files size
const selectedItemIndex = ref(-1);
// const selectedFile = ref({}); // current selected file item

// mutil select mode
const selectMode = ref(false);
const selectedCount = ref(0);
const selectedSize = ref(0);  // selected files size

// search text
const searchedFileList = ref([]);    // filter fileList by searchText
const searchedFileSize = ref(0);

// preview 
const isDraggingSplitter = ref(false);      // dragging splitter to resize preview pane
const previewDiv = ref(null);
const previewPaneSize = ref({ width: 100, height: 100 });
const imageSrc = ref('');         // preview image source

// message box
const showRenameMsgbox = ref(false);  // show rename message box
const renamingFileName = ref({}); // extract the file name to {name, ext}

const showMoveTo = ref(false);
const showCopyTo = ref(false);
const showDeleteMsgbox = ref(false);
const errorMessage = ref('');

const toolTipRef = ref(null);
const searchBoxRef = ref(null);

let resizeObserver;

let unlistenGridView: () => void;
let unlistenImageViewer: () => void;

// more menuitems
const moreMenuItems = computed(() => {
  return [
    {
      label: localeMsg.value.menu_item_select_all,
      icon: IconCheckAll,
      action: () => {
        for (let i = 0; i < fileList.value.length; i++) {
          fileList.value[i].isSelected = true;
        }
        selectMode.value = true;
      }
    },
    {
      label: localeMsg.value.menu_item_select_none,
      icon: IconCheckNone,
      action: () => {
        for (let i = 0; i < fileList.value.length; i++) {
          fileList.value[i].isSelected = false;
        }
        selectMode.value = true;
      }
    },
    {
      label: localeMsg.value.menu_item_select_invert,
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
    },
    {
      label: localeMsg.value.menu_item_move_to,
      icon: IconMoveTo,
      disabled: selectedCount.value === 0,
      action: () => {
        showMoveTo.value = true;
      }
    },
    {
      label: localeMsg.value.menu_item_copy_to,
      disabled: selectedCount.value === 0,
      action: () => {
        showCopyTo.value = true;
      }
    },
    {
      label: localeMsg.value.menu_item_delete,
      icon: IconDelete,
      disabled: selectedCount.value === 0,
      action: () => {
        showDeleteMsgbox.value = true;
      }
    },
    {
      label: "-",   // separator
      action: () => {}
    },
    {
      label: localeMsg.value.menu_item_favorite,
      icon: IconFavorite,
      disabled: selectedCount.value === 0,
      action: () => {
        selectModeSetFavorites(true);
      }
    },
    {
      label: localeMsg.value.menu_item_unfavorite,
      icon: IconUnFavorite,
      disabled: selectedCount.value === 0,
      action: () => {
        selectModeSetFavorites(false);
      }
    },
  ];
});


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

  unlistenGridView = await listen('message-from-grid-view', (event) => {
    const { message } = event.payload;
    console.log('content - message-from-grid-view:', message);
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
        copyImage(fileList.value[selectedItemIndex.value].file_path).then(() => {
          toolTipRef.value.showTip(localeMsg.value.tooltip_copy_image_success);
        });
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
      case 'delete':
        showDeleteMsgbox.value = true;
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
      default:
        break;
    }
  });

  unlistenImageViewer = await listen('message-from-image-viewer', (event) => {
    const { message } = event.payload;
    console.log('content - message-from-image-viewer:', message);
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
      case 'delete':
        clickDeleteFile();
        break;
      case 'favorite':
        toggleFavorite();
        break;
      case 'rotate':
        clickRotate();
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
});

onUnmounted(() => {
  if (resizeObserver && previewDiv.value) {
    resizeObserver.unobserve(previewDiv.value);
    resizeObserver.disconnect();
  }
});

/// watch language
watch(() => config.language, (newLanguage) => {
    locale.value = newLanguage; // update locale based on config.language
});

/// watch home
watch(() => config.toolbarIndex, async(newIndex) => {
  if(newIndex === 0) { // home
    contentTitle.value = localeMsg.value.home;
    fileList.value = await getAllFiles();  // get all files
    refreshFileList();
  } 
}, { immediate: true });

/// watch favorites
watch(() => [config.toolbarIndex, config.favoriteAlbumId, config.favoriteFolderId, config.favoriteFolderPath], 
            async ([newIndex, newAlbumId, newFolderId, newFolderPath]) => {
  if(newIndex === 1) {
    if(newFolderId === 0) { // 0: favorite files
      contentTitle.value = localeMsg.value.favorite_files;
      fileList.value = await getAllFiles(true); // true: only get favorite files
    } else {                // else: favorite folders
      const album = await getAlbum(newAlbumId);
      if(album) {
        contentTitle.value = album.name + getRelativePath(newFolderPath, album.path);
      };
      fileList.value = await getFolderFiles(newFolderId, newFolderPath);
    }
    refreshFileList();
  }
}, { immediate: true });

/// watch album
watch(() => [config.toolbarIndex, config.albumId, config.albumFolderId, config.albumFolderPath], 
            async ([newIndex, newAlbumId, newFolderId, newFolderPath]) => {
  if(newIndex === 2) {
    if (newAlbumId) {
      const album = await getAlbum(newAlbumId);
      if(album) {
        if(newFolderPath === album.path) { // current folder is root
          contentTitle.value = album.name;
        } else {
          contentTitle.value = album.name + getRelativePath(newFolderPath, album.path);
        };

        fileList.value = await getFolderFiles(newFolderId, newFolderPath);
        refreshFileList();
      } 
    } else {
      contentTitle.value = localeMsg.value.album;
      fileList.value = [];
    }
  }
}, { immediate: true });

// watch calandar
watch(() => [config.toolbarIndex, config.calendarYear, config.calendarMonth, config.calendarDate], 
            async ([newIndex, year, month, date]) => {
  if(newIndex === 3) {
    if (year && month && date) {
      if (config.calendarDate === -1) {     // monthly
        contentTitle.value = formatDate(config.calendarYear, config.calendarMonth, 1, localeMsg.value.month_format);
      } else if (config.calendarDate > 0) { // daily
        contentTitle.value = formatDate(config.calendarYear, config.calendarMonth, config.calendarDate, localeMsg.value.date_format_long);
      }
      fileList.value = await getCalendarFiles(year, month, date);
      refreshFileList()
    } else {
      contentTitle.value = localeMsg.value.calendar;
      fileList.value = [];
    }
  }
}, { immediate: true });

// watch location
// TODO: impl location

// watch people
// TODO: impl people 

// watch camera
watch(() => [config.toolbarIndex, config.cameraMake, config.cameraModel], 
            async ([newIndex, newMake, newModel]) => {
  if(newIndex === 6) {
    if(newMake) {
      if(newModel) {
        contentTitle.value = `${config.cameraMake} > ${config.cameraModel}`;
        fileList.value = await getCameraFiles(config.cameraMake, newModel);
      } else {
        contentTitle.value = `${config.cameraMake}`;
        fileList.value = await getCameraFiles(config.cameraMake, "");
      }
      refreshFileList()
    } else {
      contentTitle.value = localeMsg.value.camera;
      fileList.value = [];
    }

  }
}, { immediate: true });

// watch for changes in the file list (selected item index or file list length)
watch(() => [selectedItemIndex.value, fileList.value], () => {
  // update the preview
  if(config.showPreview) {
    getImageSrc();
  }
  // update image viewer if the viewer is open
  openImageViewer(selectedItemIndex.value, false);

  // update all files' size
  totalFilesSize.value = fileList.value.reduce((total, file) => { return total + file.size; }, 0);
  
  // update selected files' count and size
  selectedCount.value = fileList.value.filter(file => file.isSelected).length;
  selectedSize.value = fileList.value.reduce((total, file) => { return total + (file.isSelected ? file.size : 0); }, 0);
}, { deep: true });

// watch searchtext
watch(() => [config.searchText, fileList.value], ([newSearchText, newFileList]) => {
  if(newSearchText.length > 0) {
    searchedFileList.value = newFileList.filter(file => file.name.toLowerCase().includes(newSearchText.trim().toLowerCase()));

    // update searched files' size
    searchedFileSize.value = searchedFileList.value.reduce((total, file) => { return total + file.size; }, 0);

    // reset select item index
    selectedItemIndex.value = Math.min(0, searchedFileList.value.length - 1);
  } else {
    selectedItemIndex.value = Math.min(0, newFileList.length - 1);
  }
});

// watch preview
watch(() => config.showPreview, (showPreview) => {
  if (showPreview) {
    getImageSrc();
  }
});

// get selected image source
const getImageSrc = async () => {
  if(selectedItemIndex.value < 0 || selectedItemIndex.value >= fileList.value.length) {
    imageSrc.value = '';
    return;
  }
  
  let filePath = fileList.value[selectedItemIndex.value].file_path;
  console.log('getImageSrc:', filePath);
  try {
    let currentIndex = selectedItemIndex.value;
    const imageBase64 = await getFileImage(filePath);
    // Check if the selected item has changed since the invocation
    if (currentIndex === selectedItemIndex.value) {
      imageSrc.value = `data:image/jpeg;base64,${imageBase64}`;
    }
  } catch (error) {
    imageSrc.value = '';
    console.error('getImageSrc error:', error);
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

// sorting type options
const sortingOptions = computed(() => {
  return getSelectOptions(localeMsg.value.file_list_sorting_options);
});

// sorting extend options
const sortingExtendOptions = computed(() => {
  return getSelectOptions(localeMsg.value.file_list_sorting_extend_options);
});

const handleSortingSelect = (option, extendOption) => {
  console.log('Order option:', option, extendOption);
  config.sortingType = option;
  config.sortingDirection = extendOption;
  sortFileList(fileList.value, config.sortingType, config.sortingDirection)
};

// file type options
const fileTypeOptions = computed(() => {
  return getSelectOptions(localeMsg.value.file_list_file_type_options);
});

const handleFileTypeSelect = (option, extendOption) => {
  config.fileType = option;
};

function getSelectOptions(options) {
  const result = [];
  for (let i = 0; i < options.length; i++) {
    result.push({ label: options[i], value: i });
  }
  return result;
}

function refreshFileList() {
  selectMode.value = false;   // exit multi-select mode

  if(fileList.value.length > 0) {
    selectedItemIndex.value = 0;

    sortFileList(fileList.value, config.sortingType, config.sortingDirection);
    getFileListThumb(fileList.value); 
  } else {
    selectedItemIndex.value = -1;
  }
  console.log('refreshFileList:', fileList.value);
}

// Sort the file list based on the sorting type and direction
function sortFileList(files, sortingType, sortingDirection) {
  fileList.value = [...files].sort((a, b) => {
    let result = 0;

    switch (sortingType) {
      case 0:   // name
        // switch (config.language) {
        //   case 'zh':
        //     result = a.name.localeCompare(b.name, 'zh-Hans-CN');
        //     break;
        //   case 'ja':
        //     result = a.name.localeCompare(b.name, 'ja-JP');
        //     break;
        //   default:
        //     result = a.name.localeCompare(b.name);
        //     break;
        // }
        result = localeComp(config.language, a.name, b.name);
        break;
      case 1:   // size
        result = a.size - b.size;
        break;
      case 2:   // resulution
        if(a.width === b.width) {
          result = a.height - b.height;
        } else {
          result = a.width - b.width;
        }
        break;
      case 3:   // created time
        result = a.created_at - b.created_at;
        break;
      case 4:   // modified time
        result = a.modified_at - b.modified_at;
        break;
      case 5:   // taken time
        if(a.e_date_time && b.e_date_time) {
          result = a.e_date_time - b.e_date_time;
        } else {
          result = a.modified_at - b.modified_at;
        }
        break;
      default:
        return 0; // No sorting if the sorting type is unrecognized
    }

    return sortingDirection === 0 ? result : -result;  // reverse the result if descending
  });
}

// Get the thumbnail for the files
async function getFileListThumb(files, concurrencyLimit = 8) {
  const result = [];
  let activeRequests = 0;
  thumbCount.value = 0;

  const getThumbForFile = async (file) => {
    const thumb = await getFileThumb(file.id, file.file_path, file.e_orientation || 0, config.thumbnailImageSize);
    if(thumb) {
      file.thumbnail = `data:image/jpeg;base64,${thumb.thumb_data_base64}`;
      thumbCount.value++;
    }
    return file;
  };

  const runWithConcurrencyLimit = async (files) => {
    const queue = [];

    for (let i = 0; i < files.length; i++) {
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
          minimizable: false,
        }),
      });

      imageWindow.once('tauri://created', () => {
        console.log('ImageViewer window created');
      });

      imageWindow.once('tauri://close-requested', () => {
        imageWindow?.close();
        console.log('ImageViewer window is closing');
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
  }
}

/// Dragging the splitter
function startDragging(event) {
  isDraggingSplitter.value = true;
  document.addEventListener('mousemove', handleMouseMove);
  document.addEventListener('mouseup', stopDragging);
}

/// stop dragging the splitter
function stopDragging() {
  isDraggingSplitter.value = false;
  document.removeEventListener('mousemove', handleMouseMove);
  document.removeEventListener('mouseup', stopDragging);
}

/// handle mouse move event
function handleMouseMove(event) {
  // console.log('handleMouseMove:', document.documentElement.clientWidth, event.clientX, leftPosition);
  if (isDraggingSplitter.value) {
    const windowWidth = document.documentElement.clientWidth - 4; // -4: border width(2px) * 2
    const leftPosition = divListView.value.getBoundingClientRect().left - 2;  // -2: border width(2px)

    // Limit width between 20% and 80%
    config.previewPaneWidth = Math.min(Math.max(((windowWidth - event.clientX)*100) / (windowWidth - leftPosition), 20), 80); 
  }
}

// click rename menu item
const clickRenameFile = async (newName) => {
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
      errorMessage.value = localeMsg.value.msgbox_rename_file_error;
    }
  }
}

// click move to menu item
const clickMoveTo = async () => {
  if (selectMode.value && selectedCount.value > 0) {    // multi-select mode
    const moves = fileList.value
      .filter(item => item.isSelected)
      .map(async item => {
        const movedFile = await moveFile(item.id, item.file_path, config.destFolderId, config.destFolderPath);
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
    const movedFile = await moveFile(file.id, file.file_path, config.destFolderId, config.destFolderPath);
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
        const copiedFile = await copyFile(item.file_path, config.destFolderPath);
        if(copiedFile) {
          console.log('clickCopyTo:', copiedFile);
        }
      });
    await Promise.all(copies); // parallelize DB updates
    selectMode.value = false; // exit multi-select mode
  } 
  else if(selectedItemIndex.value >= 0) {               // single select mode
    const file = fileList.value[selectedItemIndex.value];
    const copiedFile = await copyFile(file.file_path, config.destFolderPath);
    if(copiedFile) {
      console.log('clickCopyTo:', copiedFile);
    }
  }
  showCopyTo.value = false;
}

// click delete menu item
const clickDeleteFile = async () => {
  if (selectMode.value && selectedCount.value > 0) {     // multi-select mode
    const deletes = fileList.value
      .filter(item => item.isSelected)
      .map(async item => {
        const deletedFile = await deleteFile(item.id, item.file_path);
        if(deletedFile) {
          console.log('clickDeleteFile:', deletedFile);
          removeFromFileList(fileList.value.indexOf(item));
        }
      });
    await Promise.all(deletes); // parallelize DB updates
    selectMode.value = false; // exit multi-select mode
  } 
  else if(selectedItemIndex.value >= 0) {               // single select mode
    const file = fileList.value[selectedItemIndex.value];
    const deletedFile = await deleteFile(file.id, file.file_path);
    if(deletedFile) {
      console.log('clickDeleteFile:', deletedFile);
      removeFromFileList(selectedItemIndex.value);
    }
  }
  showDeleteMsgbox.value = false;
}

// remove an file item from the list and update the selected item index
function removeFromFileList(index) {
  fileList.value.splice(index, 1);
  selectedItemIndex.value = Math.min(index, fileList.value.length - 1);
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

</script>

<style scoped>
* {
  user-select: none;
}
</style>
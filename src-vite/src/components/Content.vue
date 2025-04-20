<template>

  <div class="flex-1 flex flex-col">

    <!-- title bar -->
    <div class="px-4 pt-1 min-h-12 flex flex-row items-center justify-between select-none" data-tauri-drag-region>

      <!-- title -->
      <div class="mr-2 cursor-default" data-tauri-drag-region>{{ contentTitle }}</div>

      <!-- toolbar -->
      <div class="h-6 flex flex-row items-center space-x-4 flex-shrink-0">

        <!-- search box -->
        <SearchBox v-model="searchText" /> 
        
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

        <!-- filter options -->
        <DropDownSelect
          :options="filterOptions"
          :defaultIndex="config.filterType"
          @select="handleFilterSelect"
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

    <div>
      <ProgressBar v-if="fileList.length > 0" :percent="Number(((thumbCount / fileList.length) * 100).toFixed(0))" />
    </div>

    <div ref="divListView" class="mt-1 flex-1 flex flex-row overflow-hidden">
      <div class="flex-1 flex flex-col">
        <!-- grid view -->
        <GridView  
          v-model:selectItemIndex="selectedItemIndex"
          :fileList="fileList"
          :selectMode="selectMode"
        />
        
        <!-- status bar -->
        <div v-if="config.showStatusBar" 
          class="mx-2 p-2 min-h-8 border-t border-gray-700 flex flex-row items-center justify-start text-sm select-none cursor-default"
        >
          <IconFile class="t-icon-size-xs flex-shrink-0" />
          <div class="pl-1 pr-4 whitespace-nowrap">
            {{ $t('files_summary', { count: fileList.length }) + ' (' + formatFileSize(totalFilesSize) + ')' }} 
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
                ? $t('file_list_select_count', { count: selectedCount }) + ' (' + formatFileSize(selectedSize) + ')'
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

</template>

<script setup lang="ts">

import { ref, watch, computed, onMounted, onBeforeUnmount, onUnmounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { WebviewWindow } from '@tauri-apps/api/webviewWindow';
import { useI18n } from 'vue-i18n';
import { getAlbum, getAllFiles, getFolderFiles, getCalendarFiles, getCameraFiles } from '@/common/api';
import { config, isWin, isMac, formatFileSize, formatDate, getRelativePath, localeComp } from '@/common/utils';

import SearchBox from '@/components/SearchBox.vue';
import DropDownSelect from '@/components/DropDownSelect.vue';
import DropDownMenu from '@/components/DropDownMenu.vue';
import ProgressBar from '@/components/ProgressBar.vue';
import GridView  from '@/components/GridView.vue';
import Image from '@/components/Image.vue';

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
  IconPhoto,
  IconFolder,
  IconChecked,
  IconUnchecked,
  IconSearch,
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
// const originalFileList = ref([]);
const fileList = ref([]);   // file list by filtering and sorting
const totalFilesSize = ref(0);   // total files size
const selectedItemIndex = ref(-1);

// mutil select mode
const selectMode = ref(false);
const selectedCount = ref(0);
const selectedSize = ref(0);  // selected files size

// search text
const searchText = ref('');

// preview 
const isDraggingSplitter = ref(false);      // dragging splitter to resize preview pane
const previewDiv = ref(null);
const previewPaneSize = ref({ width: 100, height: 100 });
const imageSrc = ref('');         // preview image source
let resizeObserver;

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
    // {
    //   label: localeMsg.value.menu_item_move_to,
    //   icon: IconMoveTo,
    //   action: () => {}
    // },
    // {
    //   label: localeMsg.value.menu_item_copy_to,
    //   action: () => {}
    // },
    {
      label: localeMsg.value.menu_item_delete,
      icon: IconDelete,
      action: () => {
        if(selectedItemIndex.value >= 0) {
          deleteFile(selectedItemIndex.value);
        }
      }
    },
    {
      label: "-",   // separator
      action: () => {}
    },
    {
      label: localeMsg.value.menu_item_favorite,
      icon: IconFavorite,
      action: () => {
        if(selectedItemIndex.value >= 0) {
          fileList.value[selectedItemIndex.value].is_favorite = true;
        }
      }
    },
    {
      label: localeMsg.value.menu_item_unfavorite,
      icon: IconUnFavorite,
      action: () => {
        if(selectedItemIndex.value >= 0) {
          fileList.value[selectedItemIndex.value].is_favorite = false;
        }
      }
    },
  ];
});

let unlistenTitleBar: () => void;
let unlistenGridView: () => void;
let unlistenImageViewer: () => void;

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

  unlistenTitleBar = await listen('message-from-titlebar', (event) => {
    const { message, search } = event.payload;
    console.log('content: message-from-titlebar:', message, search);
    switch (message) {
      case 'search':
        searchText.value = search;
        refreshFileList();
        break;
      default:
        break;
    }
  });

  unlistenGridView = await listen('message-from-grid-view', (event) => {
    const { message } = event.payload;
    console.log('content - message-from-grid-view:', message);
    switch (message) {
      case 'open-image-viewer':
        openImageViewer(selectedItemIndex.value, true);
        break;
      case 'update-image-viewer':
        openImageViewer(selectedItemIndex.value, false);
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
        deleteFile(selectedItemIndex.value);  // delete the selected file from list
        break;
      default:
        break;
    }
  });
});

onBeforeUnmount(() => {
  // unlisten
  unlistenTitleBar();
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
  // update the selected count
  selectedCount.value = fileList.value.filter(file => file.isSelected).length;

  // update total file size
  totalFilesSize.value = fileList.value.reduce((total, file) => {
    return total + file.size;
  }, 0);

  // update selected file size
  selectedSize.value = fileList.value.reduce((total, file) => {
    return total + (file.isSelected ? file.size : 0);
  }, 0);

  if(config.showPreview) {
    getImageSrc();
  }
}, { deep: true });   // deep watch: because isSelected is a property of each file object

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
    const imageBase64 = await invoke('get_file_image', { filePath });
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

// filter options
const filterOptions = computed(() => {
  return getSelectOptions(localeMsg.value.file_list_filter_options);
});

const handleFilterSelect = (option, extendOption) => {
  console.log('Filter option:', option);
  config.filterType = option;
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

    // filterFileList(originalFileList.value, searchText.value);
    sortFileList(fileList.value, config.sortingType, config.sortingDirection);
    getFileThumb(fileList.value); 
  } else {
    selectedItemIndex.value = -1;
  }
  console.log('refreshFileList:', fileList.value);
}

// Filter the file list based on the search text
// function filterFileList(files, filter) {
//   if (filter.trim() === '') {
//     fileList.value = files;
//   } else {
//     fileList.value = files.filter(file => file.name.toLowerCase().includes(filter.toLowerCase()));
//   }
// }

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

// Delete the file from the list and update the selected item index
function deleteFile(index) {
  fileList.value.splice(index, 1);
  selectedItemIndex.value = Math.min(index, fileList.value.length - 1);
  openImageViewer(selectedItemIndex.value, false);  // update the image viewer
}

// Get the thumbnail for the files
async function getFileThumb(files, concurrencyLimit = 8) {
  try {
    const result = [];
    let activeRequests = 0;
    thumbCount.value = 0;

    const getThumbForFile = async (file) => {
      const thumb = await invoke('get_file_thumb', {
        fileId: file.id,
        filePath: file.file_path,
        orientation: file.e_orientation || 0, // Simplified orientation
        thumbnailSize: config.thumbnailImageSize,
      });

      file.thumbnail = `data:image/jpeg;base64,${thumb.thumb_data_base64}`;
      thumbCount.value++;
      // console.log('getFileThumb:', file);
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

  } catch (error) {
    console.log('getFileThumb error:', error);
  }
}

// Open the image viewer window
async function openImageViewer(index: number, createNew = false) {
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
    if (createNew) {
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
    if(createNew) {
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

</script>

<style scoped>
* {
  user-select: none;
}
</style>
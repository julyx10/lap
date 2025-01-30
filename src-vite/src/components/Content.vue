<template>

  <div class="flex-1 flex flex-col">

    <!-- title bar -->
    <div class="px-4 pt-1 flex flex-row items-center justify-between" style="user-select: none;">

      <div class="mr-2 flex-1 flex flex-col">
        <span>{{ contentTitle }}</span>
        <span class="text-sm">
          {{ $t('files_summary', { count: fileList.length }) }}
        </span>
      </div>

      <div class="h-6 flex flex-row items-center space-x-4">
        <button
          :class="[
            'px-2 py-1 flex flex-row items-center rounded-md border t-color-border  text-sm',
          ]"
          @click="handleSelectMode(true)"
        >
          <IconClose v-if="selectMode" class="t-icon-size-sm t-icon-hover" @click.stop="handleSelectMode(false)" />
          <span class="px-1">{{ selectMode ? $t('file_list_select_count', { count: selectedCount }) : $t('file_list_select_mode') }}</span>
          <DropDownMenu v-if="selectMode"
            :iconMenu="IconMore"
            :menuItems="moreMenuItems"
            :alignRight="true"
            @click.stop
          />
        </button>


        <DropDownSelect
          :options="sortingOptions"
          :defaultIndex="config.sortingType"
          :extendOptions="sortingExtendOptions"
          :defaultExtendIndex="config.sortingDirection"
          @select="handleSortingSelect"
        />
        <DropDownSelect
          :options="filterOptions"
          :defaultIndex="config.filterType"
          @select="handleFilterSelect"
        />
        <component 
          :is="config.showPreview ? IconPreview : IconPreviewOff" 
          class="t-icon-size t-icon-hover" 
          @click="config.showPreview = !config.showPreview"
        />
      </div>
    </div>

    <div>
      <ProgressBar v-if="fileList.length > 0" :percent="Number(((thumbCount / fileList.length) * 100).toFixed(0))" />
    </div>

    <div ref="divListView" class="mt-1 flex-1 flex flex-row overflow-hidden">
      <!-- grid view -->
      <GridView  
        v-model="selectedItemIndex"
        :fileList="fileList"
        :selectMode="selectMode"
      />

      <!-- splitter -->
      <div v-if="config.showPreview" class="w-1 hover:bg-sky-700 cursor-ew-resize" @mousedown="startDragging"></div>

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

import { ref, watch, computed, onMounted, onUnmounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

import { WebviewWindow } from '@tauri-apps/api/webviewWindow';
import { format } from 'date-fns';
import { useI18n } from 'vue-i18n';
import { useConfigStore } from '@/stores/configStore';
import { separator, THUMBNAIL_SIZE, FILES_PAGE_SIZE, formatDate, getFolderPath } from '@/common/utils';

import DropDownSelect from '@/components/DropDownSelect.vue';
import DropDownMenu from '@/components/DropDownMenu.vue';
import ProgressBar from '@/components/ProgressBar.vue';
import GridView  from '@/components/GridView.vue';
import Image from '@/components/Image.vue';

import IconPreview from '@/assets/preview-on.svg';
import IconPreviewOff from '@/assets/preview-off.svg';
import IconMore from '@/assets/more.svg';
import IconArrowDown from '@/assets/arrow-down.svg';

import IconClose from '@/assets/close.svg';
import IconSelectAll from '@/assets/checkbox-checkall.svg';
import IconSelectNone from '@/assets/checkbox-uncheckall.svg';
import IconFavorite from '@/assets/heart-solid.svg';
import IconUnFavorite from '@/assets/heart.svg';
import IconRotate from '@/assets/rotate-right.svg';
import IconCopy from '@/assets/copy.svg';
import IconRename from '@/assets/rename.svg';
import IconRefresh from '@/assets/refresh.svg';
import IconCopyTo from '@/assets/copy-to.svg';
import IconMoveTo from '@/assets/move-to.svg';
import IconDelete from '@/assets/trash.svg';
import IconOpenFolder from '@/assets/folder-open.svg';
import { tr } from 'date-fns/locale';

const props = defineProps({
  titlebar: String
});

/// i18n
const { locale, messages } = useI18n();
const localeMsg = computed(() => messages.value[locale.value]);

// config store
const config = useConfigStore();

// title of the content 
const contentTitle = ref("");   

// progress bar
const thumbCount = ref(0);      // thumbnail count (from 0 to fileList.length)

// file list view
const divListView = ref(null);

// file list
// const originalFileList = ref([]);
const fileList = ref([]);   // file list by filtering and sorting
const selectedItemIndex = ref(-1);

// select mode
const selectMode = ref(false);
const selectedCount = ref(0);

// search text
const searchText = ref('');

// preview 
const isDraggingSplitter = ref(false);      // dragging splitter to resize preview pane
const previewDiv = ref(null);
const previewPaneSize = ref({ width: 100, height: 100 });
const imageSrc = ref(null);         // preview image source
let resizeObserver;

// image viewer
const isImageViewerOpen  = ref(false);  // show image viewer(new window)

onMounted(() => {
  console.log('content mounted');
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
  
  console.log('previewDiv:', previewDiv.value);
});

onUnmounted(() => {
  if (resizeObserver && previewDiv.value) {
    resizeObserver.unobserve(previewDiv.value);
    resizeObserver.disconnect();
  }
});

listen('message-from-titlebar', (event) => {
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

listen('message-from-grid-view', (event) => {
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

listen('message-from-image-viewer', (event) => {
  const { message } = event.payload;
  console.log('content - message-from-image-viewer:', message);
  switch (message) {
    case 'home':
      selectedItemIndex.value = 0;
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

/// watch language
watch(() => config.language, (newLanguage) => {
    locale.value = newLanguage; // update locale based on config.language
});

/// watch home (all files) and favorite
watch(() => config.toolbarIndex, newIndex => {
  if(newIndex === 0) { // home
    contentTitle.value = localeMsg.value.home;
    getAllFiles();  // get all files
    selectedItemIndex.value = -1;
  } else if(newIndex === 1) { // favorite
    contentTitle.value = localeMsg.value.favorite;
    getAllFiles(true); // get favorite files
    selectedItemIndex.value = -1;
  }
}, { immediate: true });

/// watch album
watch(() => [config.toolbarIndex, config.albumId, config.albumFolderId], async ([newIndex, newAlbumId, newFolderId]) => {
  if(newIndex === 2) {
    if (newAlbumId) {
      try {
        const album = await invoke('get_album', { albumId: newAlbumId });

        if(config.albumFolderPath === album.path) { // current folder is root
          contentTitle.value = album.name;
        } else {
          const relative_path = config.albumFolderPath.replace(album.path, '').split(separator).join(' > ');
          contentTitle.value = album.name + relative_path;
        };

        getFolderFiles();
        selectedItemIndex.value = -1;
      } catch (error) {
        console.error('get_album error:', error);
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
      getCalendarFiles(year, month, date);
      selectedItemIndex.value = -1;
    } else {
      contentTitle.value = localeMsg.value.calendar;
      fileList.value = [];
    }
  }
}, { immediate: true });

// watch location

// watch people

// watch camera
watch(() => [config.toolbarIndex, config.cameraMake, config.cameraModel], async ([newIndex, newMake, newModel]) => {
  if(newIndex === 6) {
    if(newMake) {
      if(newModel) {
        contentTitle.value = `${config.cameraMake} > ${config.cameraModel}`;
        getCameraFiles(config.cameraMake, newModel);
        selectedItemIndex.value = -1;
      } else {
        contentTitle.value = `${config.cameraMake}`;
        getCameraFiles(config.cameraMake, "");
        selectedItemIndex.value = -1;
      }
    } else {
      contentTitle.value = localeMsg.value.camera;
      fileList.value = [];
    }

  }
}, { immediate: true });

// watch for changes in the file list (selected item index or file list length)
watch(() => [selectedItemIndex.value, fileList.value, isImageViewerOpen.value], () => {
  // update the selected count
  selectedCount.value = fileList.value.filter(file => file.isSelected).length;
  getImageSrc();
}, { deep: true });   // deep watch: because isSelected is a property of each file object

// get selected image source
const getImageSrc = async () => {
  if(selectedItemIndex.value < 0 || selectedItemIndex.value >= fileList.value.length) {
    imageSrc.value = '';
    return;
  }
  
  // prevent loading image when the image viewer is open
  if(isImageViewerOpen.value) {
    imageSrc.value = fileList.value[selectedItemIndex.value].thumbnail || '';
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

/// get all files
async function getAllFiles(isFavorite = false, offset = 0, pageSize = FILES_PAGE_SIZE) {
  try {
    fileList.value = await invoke('get_all_files', { isFavorite, offset, pageSize });
    refreshFileList(); // get fileList(apply filter and sorting)
  } catch (error) {
    console.error('getAllFiles error:', error);
  }
}

/// get all files under the path
async function getFolderFiles() {
  try {
    // read the list of files
    fileList.value = await invoke('get_folder_files', { folderId: config.albumFolderId, path: config.albumFolderPath });
    refreshFileList();
  } catch (error) {
    console.error('getFolderFiles error:', error);
  }
};

/// get all files of calendar
async function getCalendarFiles(year, month, date) {
  try {
    if (date === -1) { // -1 means selecting a month
      // get the first and last days of the month.
      let startDate = format(new Date(year, month - 1, 1), 'yyyy-MM-dd');
      let endDate = format(new Date(year, month, 0), 'yyyy-MM-dd');
      fileList.value = await invoke('get_files_by_date_range', { startDate, endDate });
    } else {  // otherwise, get files by date
      let dateStr = format(new Date(year, month - 1, date), 'yyyy-MM-dd');
      fileList.value = await invoke('get_files_by_date', { date: dateStr });
    }
    refreshFileList();
  } catch (error) {
    console.error('getCalendarFiles error:', error);
  }
}

/// get all files under the camera make and model
async function getCameraFiles(make, model) {
  try {
    fileList.value = await invoke('get_camera_files', { make, model });
    refreshFileList();
  } catch (error) {
    console.error('getCameraFiles error:', error);
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

// more menuitems
const moreMenuItems = computed(() => {
  return [
    {
      label: localeMsg.value.menu_item_select_all,
      icon: IconSelectAll,
      action: () => {
        for (let i = 0; i < fileList.value.length; i++) {
          fileList.value[i].isSelected = true;
        }
        selectMode.value = true;
      }
    },
    {
      label: localeMsg.value.menu_item_select_none,
      icon: IconSelectNone,
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
      action: null
    },
    {
      label: localeMsg.value.menu_item_move_to,
      icon: IconMoveTo,
      action: () => {}
    },
    {
      label: localeMsg.value.menu_item_copy_to,
      action: () => {}
    }
  ];
});

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
  selectMode.value = false;

  // filterFileList(originalFileList.value, searchText.value);
  sortFileList(fileList.value, config.sortingType, config.sortingDirection);
  getFileThumb(fileList.value); 
  console.log('fileList:', fileList.value);
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
        switch (config.language) {
          case 'zh':
            result = a.name.localeCompare(b.name, 'zh-Hans-CN');
            break;
          case 'ja':
            result = a.name.localeCompare(b.name, 'ja-JP');
            break;
          default:
            result = a.name.localeCompare(b.name);
            break;
        }
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
        thumbnailSize: THUMBNAIL_SIZE
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
        width: 800,
        height: 600,
        transparent: true,
        decorations: false,
      });

      imageWindow.once('tauri://created', () => {
        isImageViewerOpen.value = true;
        console.log('ImageViewer window created');
      });

      imageWindow.once('tauri://close-requested', () => {
        isImageViewerOpen.value = false;
        imageWindow.close();
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
      imageWindow.setFocus();
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

    // Limit width between 10% and 90%
    config.previewPaneWidth = Math.min(Math.max(((windowWidth - event.clientX)*100) / (windowWidth - leftPosition), 10), 90); 
  }
}

</script>

<style scoped>
* {
  user-select: none;
}
</style>
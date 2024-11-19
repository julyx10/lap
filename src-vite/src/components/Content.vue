<template>

  <div class="flex-1 flex flex-col">

    <!-- title bar -->
    <!-- <div class="px-4" style="user-select: none;"> -->
      <div class="px-4 pt-1 flex flex-row items-center justify-between" style="user-select: none;">

        <div class="flex-1 flex flex-col">
          <span>{{ contentTitle }}</span>
          <span class="text-sm">
            {{ $t('files_summary', { files: fileList.length }) }}
          </span>
        </div>

        <div class="h-6 flex space-x-4">
          <SliderInput v-model="config.gridSize" :min="120" :max="320" :step="10" label="" />
          <IconFitWidth 
            class="t-icon-size t-icon-hover"
            :class="{ 't-icon-focus': config.isFitWidth }"
            @click="config.isFitWidth = !config.isFitWidth" 
          />
          <component 
            :is="config.isFavorite ? IconFavorite : IconUnFavorite" 
            class="t-icon-hover hover:text-red-600"
            :class="{ 'text-red-600': config.isFavorite }"
            @click="config.isFavorite = !config.isFavorite"
          />
          <component :is="IconTag" class="t-icon-hover" />
          <component 
            :is="config.sortingAsc ? IconSortingAsc : IconSortingDesc" 
            class="t-icon-hover" 
            @click="toggleSortingOrder" />
          <component 
            :is="config.showPreview ? IconPreview : IconPreviewOff" 
            class="t-icon-hover" 
            :class="{ 't-icon-focus': config.showPreview }"
            @click="config.showPreview = !config.showPreview"
          />
        </div>
      </div>
    <!-- </div> -->

    <div>
      <ProgressBar v-if="fileList.length > 0" :percent="Number(((thumbCount / fileList.length) * 100).toFixed(0))" />
    </div>

    <div ref="divGridView" class="mt-1 flex-1 flex flex-row overflow-hidden">
      <!-- grid view -->
      <GridView v-if="fileList.length > 0" 
        :fileList="fileList"
        :gridSize="Number(config.gridSize)" 
        :isFitWidth="config.isFitWidth"
        v-model="selectedItemIndex"
      />
      <div v-else class="min-w-32 flex-1 flex flex-row items-center justify-center">
        <p>{{ $t('file_list_no_files') }}</p>
      </div>

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
        <div v-if="config.showPreview" class="t-color-bg rounded-ss-lg" :style="{ width: config.previewPaneWidth + '%' }">
          <div v-if="selectedItemIndex >= 0 && selectedItemIndex < fileList.length"
            class="h-full flex flex-col items-center justify-center cursor-pointer break-all"
            @dblclick="openImageViewer(selectedItemIndex, true)">
            <img class="h-full w-full p-1 rounded-lg object-contain" :src="imageSrc" @load="onImageLoad" />
            <div class="fixed p-2 bottom-0 flex flex-col items-center text-sm">
              <p>{{ fileList[selectedItemIndex].name }}</p>
              <div class="flex space-x-4">
                <!-- <p>{{ formatFileSize(fileList[selectedItemIndex].size) }}</p> -->
                <p>{{ formatTimestamp(fileList[selectedItemIndex].modified_at, $t('date_time_format')) }}</p>
                <!-- <p>{{ fileList[selectedItemIndex].width }}x{{ fileList[selectedItemIndex].height }}</p> -->
              </div>
            </div>
          </div>
      
          <div v-else class="h-full flex items-center justify-center">
            <p>{{ $t('preview_no_file') }}</p>
          </div>
        </div>
      </transition>

    </div>
  </div>

</template>


<script setup lang="ts">

import { ref, watch, computed, onMounted, onBeforeUnmount } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

import { WebviewWindow } from '@tauri-apps/api/webviewWindow';
import { format } from 'date-fns';
import { useI18n } from 'vue-i18n';
import { useConfigStore } from '@/stores/configStore';
import { separator, THUMBNAIL_SIZE, FILES_PAGE_SIZE, formatFileSize, formatTimestamp, formatDate } from '@/common/utils';

import SliderInput from '@/components/SliderInput.vue';
import ProgressBar from '@/components/ProgressBar.vue';
import GridView  from '@/components/GridView.vue';

import IconFitWidth from '@/assets/fit-width.svg';
import IconUnFavorite from '@/assets/heart.svg';
import IconFavorite from '@/assets/heart-solid.svg';
import IconTag from '@/assets/tag.svg';
import IconSortingAsc from '@/assets/sorting-asc.svg';
import IconSortingDesc from '@/assets/sorting-desc.svg';
import IconPreview from '@/assets/preview-on.svg';
import IconPreviewOff from '@/assets/preview-off.svg';

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

// grid view div
const divGridView = ref(null);

// file list
const fileList = ref([]);
const selectedItemIndex = ref(-1);

// preview 
const isDragging = ref(false);      // dragging splitter to resize preview pane
const imageSrc = ref(null);         // preview image source

// show image viewer
const isImageViewerOpen  = ref(false); 


// onMounted(() => {
//   document.addEventListener('mouseup', stopDragging);
// })

// onBeforeUnmount(() => {
//   document.removeEventListener('mouseup', stopDragging);
// })

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
    case 'prev':
      selectedItemIndex.value = Math.max(selectedItemIndex.value - 1, 0);
      break;
    case 'next':
      selectedItemIndex.value = Math.min(selectedItemIndex.value + 1, fileList.value.length - 1);
      break;
    default:
      break;
  }
});

/// Dragging the splitter
function startDragging(event) {
  isDragging.value = true;
  document.addEventListener('mousemove', handleMouseMove);
  document.addEventListener('mouseup', stopDragging);
}

/// stop dragging the splitter
function stopDragging() {
  isDragging.value = false;
  document.removeEventListener('mousemove', handleMouseMove);
  document.removeEventListener('mouseup', stopDragging);
}

/// handle mouse move event
function handleMouseMove(event) {
  // console.log('handleMouseMove:', document.documentElement.clientWidth, event.clientX, leftPosition);
  if (isDragging.value) {
    const windowWidth = document.documentElement.clientWidth - 4; // -4: border width(2px) * 2
    const leftPosition = divGridView.value.getBoundingClientRect().left - 2;  // -2: border width(2px)

    // Limit width between 10% and 90%
    config.previewPaneWidth = Math.min(Math.max(((windowWidth - event.clientX)*100) / (windowWidth - leftPosition), 10), 90); 
  }
}

/// watch language
watch(() => config.language, (newLanguage) => {
    locale.value = newLanguage; // update locale based on config.language
});

/// watch home (all files)
watch(() => config.toolbarIndex, newIndex => {
  if(newIndex === 0) {
    contentTitle.value = localeMsg.value.home;
    getAllFiles();
  }
});

/// watch album
watch(() => [config.toolbarIndex, config.albumId, config.albumFolderId], async ([newIndex, newAlbumId, newFolderId]) => {
  if(newIndex === 1) {
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
  if(newIndex === 2) {
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
  if(newIndex === 5) {
    if(newMake) {
      if(newModel) {
        contentTitle.value = `${config.cameraMake} > ${config.cameraModel}`;
        getCameraFiles(config.cameraMake, newModel);
      } else {
        contentTitle.value = `${config.cameraMake}`;
        getCameraFiles(config.cameraMake, "");
        // fileList.value = [];
      }
    } else {
      contentTitle.value = localeMsg.value.camera;
      fileList.value = [];
    }

  }
}, { immediate: true });

// watch for changes in the file list
watch(() => selectedItemIndex.value, (newIndex) => {
  console.log('watch - selectedItemIndex:', newIndex);

  if (newIndex >= 0 && newIndex < fileList.value.length && fileList.value[newIndex].thumbnail ) {
    imageSrc.value = fileList.value[newIndex].thumbnail;
  } else {
    imageSrc.value = '/src/assets/photo.svg';
  }
});

watch(isImageViewerOpen, (show) => {
  console.log('watch - isImageViewerOpen:', show);
  onImageLoad();
});

const onImageLoad = async () => {
  // prevent loading image when the image viewer is open
  if(isImageViewerOpen.value) {
    return;
  }

  let filePath = fileList.value[selectedItemIndex.value].file_path;
  console.log('onImageLoad:', filePath);
  try {
    const imageBase64 = await invoke('get_file_image', { filePath });
    imageSrc.value = `data:image/jpeg;base64,${imageBase64}`;
  } catch (error) {
    // imageSrc.value = '/src/assets/photo.svg';
    console.error('onImageLoad error:', error);
  }
}

/// toggle the sorting order
function toggleSortingOrder() {
  config.sortingAsc = !config.sortingAsc;
  fileList.value = [...fileList.value].reverse();
  if (selectedItemIndex.value >= 0) {
    selectedItemIndex.value = fileList.value.length - 1 - selectedItemIndex.value;
  }
}

/// get all files
async function getAllFiles() {
  try {
    fileList.value = await invoke('get_all_files', { isFavorite: config.isFavorite, offset: 0, pageSize: FILES_PAGE_SIZE });
    sortFileList(config.sortingType, config.sortingAsc);
    getFileThumb(fileList.value); 
    console.log('getAllFiles:', fileList.value);
  } catch (error) {
    console.error('getAllFiles error:', error);
  }
}

/// get all files under the path
async function getFolderFiles() {
  try {
    // read the list of files
    fileList.value = await invoke('get_folder_files', { folderId: config.albumFolderId, path: config.albumFolderPath });
    sortFileList(config.sortingType, config.sortingAsc);
    getFileThumb(fileList.value);
    console.log('getFolderFiles:', fileList.value);
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
      fileList.value = await invoke('get_files_by_date_range', { startDate: startDate, endDate: endDate });
    } else {  // otherwise, get files by date
      let dateStr = format(new Date(year, month - 1, date), 'yyyy-MM-dd');
      fileList.value = await invoke('get_files_by_date', { date: dateStr });
    }
    sortFileList(config.sortingType, config.sortingAsc);
    getFileThumb(fileList.value);
    console.log('getCalendarFiles:', fileList.value);
  } catch (error) {
    console.error('getCalendarFiles error:', error);
  }
}

/// get all files under the camera make and model
async function getCameraFiles(make, model) {
  try {
    fileList.value = await invoke('get_camera_files', { make: make, model: model });
    sortFileList(config.sortingType, config.sortingAsc);
    getFileThumb(fileList.value); 
    console.log('getCameraFiles:', fileList.value);
  } catch (error) {
    console.error('getCameraFiles error:', error);
  }
}

// Sort the file list based on the sorting type and order
function sortFileList(sortingType, isAccending) {
  fileList.value = [...fileList.value].sort((a, b) => {
    let result = 0;

    switch (sortingType) {
      case "name":
        result = a.name.localeCompare(b.name);
        break;
      case "size":
        result = a.size - b.size;
        break;
      case "created_date":
        result = a.created_at - b.created_at;
        break;
      case "modified_date":
        result = a.modified_at - b.modified_at;
        break;
      case "taken_date":
        if(a.e_date_time && b.e_date_time) {
          result = a.e_date_time - b.e_date_time;
        } else {
          result = a.modified_at - b.modified_at;
        }
        break;
      default:
        return 0; // No sorting if the sorting type is unrecognized
    }

    return isAccending ? result : -result;
  });
}

/// get the thumbnail for each file in mutil-thread
// async function getFileThumb(files) {
//   try {
//     const thumbnailPromises = files.map(async (file) => {
//       console.log('getFileThumb:', file.file_path);

//       const thumb = await invoke('get_file_thumb', { 
//         fileId: file.id,
//         filePath: file.file_path,
//         orientation: file.e_orientation || 0,
//         thumbnailSize: THUMBNAIL_SIZE
//       });

//       file.thumbnail = `data:image/jpeg;base64,${thumb.thumb_data_base64}`;
//       console.log('getFileThumb:', file);
//     });

//     // Wait for all thumbnail promises to resolve in parallel
//     await Promise.all(thumbnailPromises);

//   } catch (error) {
//     console.error('getFileThumb error:', error);
//   }
// }

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
  if (index < 0 || index >= fileCount) {
    return;
  }

  const file = fileList.value[index];
  const encodedFilePath = encodeURIComponent(file.file_path);
  let imageWindow = await WebviewWindow.getByLabel(webViewLabel);

  // create a new window if it doesn't exist
  if (!imageWindow) {
    if (createNew) {
      imageWindow = new WebviewWindow(webViewLabel, {
        url: `/image-viewer?fileId=${file.id}&filePath=${encodedFilePath}&fileIndex=${index}&fileCount=${fileCount}`,
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
      fileId: file.id, 
      filePath: encodedFilePath, 
      fileIndex: index,   // selected file index
      fileCount: fileCount, // total files length
    });
  }
}

</script>

<style scoped>
* {
  user-select: none;
}
</style>
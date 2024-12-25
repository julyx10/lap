<template>

  <div class="flex-1 flex flex-col">

    <!-- title bar -->
    <div class="px-4 pt-1 flex flex-row items-center justify-between" style="user-select: none;">

      <div class="flex-1 flex flex-col">
        <span>{{ contentTitle }}</span>
        <span class="text-sm">
          {{ $t('files_summary', { files: fileList.length }) }}
        </span>
      </div>

      <div class="h-6 flex space-x-4">
        <IconEdit 
          class="t-icon-size t-icon-hover"
          :class="{ 't-icon-focus': isEditing }"
          @click="isEditing = !isEditing" 
        />
        <component 
          :is="isEditing ? IconLeft : IconRight" 
          class="t-icon-hover" 
          @click="isEditing = !isEditing" 
        />
        <SliderInput 
          v-model="config.gridSize" 
          :min="120" 
          :max="320" 
          :step="10" 
          label=""
        />
        <IconFitWidth 
          class="t-icon-size t-icon-hover"
          :class="{ 't-icon-focus': config.isFitWidth }"
          @click="config.isFitWidth = !config.isFitWidth" 
        />
        <component 
          :is="config.sortingAsc ? IconSortingAsc : IconSortingDesc" 
          class="t-icon-hover" 
          @click="toggleSortingOrder" 
        />
        <component 
          :is="config.showPreview ? IconPreview : IconPreviewOff" 
          class="t-icon-hover" 
          :class="{ 't-icon-focus': config.showPreview }"
          @click="config.showPreview = !config.showPreview"
        />
      </div>
    </div>

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
        <div v-if="config.showPreview" ref="previewDiv" 
          class="p-1 t-color-bg rounded-ss-lg overflow-hidden"
          :style="{ width: config.previewPaneWidth + '%' }"
        >
          <div v-if="selectedItemIndex >= 0 && selectedItemIndex < fileList.length"
            :style="{ width: previewPaneSize.width + 'px', height: previewPaneSize.height + 'px' }"
            @dblclick="openImageViewer(selectedItemIndex, true)"
          >
            <!-- <img
              class="h-full w-full p-1 rounded-lg object-contain" 
              :src="imageSrc"
              :style="{ transform: `rotate(${fileList[selectedItemIndex].rotate ?? 0}deg)`, transition: 'none' }"
              @load="onImageLoad" 
            /> -->
            <Image v-if="imageSrc" 
              ref="imageRef" 
              :src="imageSrc" 
              :rotate="fileList[selectedItemIndex]?.rotate ?? 0" 
              :isZoomFit="config.isZoomFit"
            />

            <!-- file name -->
            <!-- <div class="fixed p-2 bottom-0 flex flex-col items-center text-sm"> -->
              <!-- <p>{{ fileList[selectedItemIndex].name }}</p> -->
              <!-- <div class="flex space-x-4"> -->
                <!-- <p>{{ formatFileSize(fileList[selectedItemIndex].size) }}</p> -->
                <!-- <p>{{ formatTimestamp(fileList[selectedItemIndex].modified_at, $t('date_time_format')) }}</p> -->
                <!-- <p>{{ fileList[selectedItemIndex].width }}x{{ fileList[selectedItemIndex].height }}</p> -->
              <!-- </div> -->
            <!-- </div> -->

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

import { ref, watch, computed, onMounted, onUnmounted } from 'vue';
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
import Image from '@/components/Image.vue';

import IconEdit from '@/assets/edit.svg';
import IconLeft from '@/assets/arrow-left.svg';
import IconRight from '@/assets/arrow-right.svg';
import IconFitWidth from '@/assets/fit-width.svg';
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
const previewDiv = ref(null);
const previewPaneSize = ref({ width: 100, height: 100 });
let resizeObserver;
const imageSrc = ref(null);         // preview image source

// dragging preview pane splitter
const isDraggingSplitter = ref(false);      // dragging splitter to resize preview pane

// show image viewer(new window)
const isImageViewerOpen  = ref(false); 

// edit mode (allow mutilple selection)
const isEditing = ref(false);

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
    case 'save':
      saveFile(selectedItemIndex.value);  // save the selected file from list
      break;
    case 'delete':
      deleteFile(selectedItemIndex.value);  // delete the selected file from list
      break;
    default:
      break;
  }
});

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
    getAllFiles();  // get all files
  }
}, { immediate: true });

/// watch favorites
watch(() => config.toolbarIndex, newIndex => {
  if(newIndex === 1) {
    contentTitle.value = localeMsg.value.favorite;
    getAllFiles(true); // get favorite files
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

// watch for changes in the file list (selected item index or file list length)
watch(() => [selectedItemIndex.value, fileList.value.length], ([newIndex, len]) => {
  console.log('watch - selectedItemIndex:', newIndex);

  if (newIndex >= 0 && newIndex < fileList.value.length && fileList.value[newIndex].thumbnail ) {
    // imageSrc.value = fileList.value[newIndex].thumbnail;
    onImageLoad();
  } else {
    // imageSrc.value = '/src/assets/photo.svg';
  }
});

// wether the image viewer is open
watch(isImageViewerOpen, (show) => {
  console.log('watch - isImageViewerOpen:', show);
  onImageLoad();
});

const onImageLoad = async () => {
  // prevent loading image when the image viewer is open
  if(isImageViewerOpen.value || selectedItemIndex.value < 0 || selectedItemIndex.value >= fileList.value.length) {
    return;
  }

  let filePath = fileList.value[selectedItemIndex.value].file_path;
  console.log('onImageLoad:', filePath);
  try {
    let currentIndex = selectedItemIndex.value;
    const imageBase64 = await invoke('get_file_image', { filePath });
    // Check if the selected item has changed since the invocation
    if (currentIndex === selectedItemIndex.value) {
      imageSrc.value = `data:image/jpeg;base64,${imageBase64}`;
    }
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
async function getAllFiles(isFavorite = false) {
  try {
    fileList.value = await invoke('get_all_files', { isFavorite: isFavorite, offset: 0, pageSize: FILES_PAGE_SIZE });
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

// Save the file
function saveFile(index) {
  const file = fileList.value[index];
  // invoke('save_file', { fileId: file.id, filePath: file.file_path });
}

// Delete the file from the list and update the selected item index
function deleteFile(index) {
  fileList.value.splice(index, 1);
  selectedItemIndex.value = Math.min(index, fileList.value.length - 1);
  openImageViewer(selectedItemIndex.value, false);  // update the image viewer
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

</script>

<style scoped>
* {
  user-select: none;
}
</style>
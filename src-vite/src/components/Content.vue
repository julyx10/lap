<template>

  <div class="flex-1 flex flex-col">

    <!-- title bar -->
    <div class="px-4 py-1" style="user-select: none;">
      <div class="flex flex-row items-center justify-between">

        <div class="flex-1 flex flex-col">
          <span>{{ title }}</span>
          <span v-if="gToolbarIndex === 1" class="text-sm">
            {{ $t('folder_summary', { folders: subFolderList.length, files: fileList.length }) }}
          </span>
          <span v-else class="text-sm">
            {{ $t('files_summary', { files: fileList.length }) }}
          </span>
        </div>

        <div class="flex space-x-4">
          <IconFitWidth 
            class="t-icon-size t-icon-hover"
            :class="{ 't-icon-focus': isFitWidth }"
            @click="toggleFitWidth" />
          <IconUnFavorite v-if="!isFavorite" class="t-icon-hover hover:text-red-600" @click="toggleFavorite" />
          <IconFavorite v-if="isFavorite" class="t-icon-hover text-red-600 hover:text-red-600" @click="toggleFavorite" />
          <component :is="IconTag" class="t-icon-hover" />
          <component :is="sortingAsc ? IconSortingAsc : IconSortingDesc" class="t-icon-hover" @click="toggleSortingOrder" />
          <component :is="showPreview ? IconPreview : IconPreviewOff" class="t-icon-hover" @click="togglePreview" />
        </div>
        
      </div>

    </div>

    <div>
      <ProgressBar v-if="fileList.length > 0" :percent="Number(((thumbCount / fileList.length) * 100).toFixed(0))" />
    </div>

    <div class="my-1 flex-1 flex flex-row overflow-hidden">
      <!-- grid view -->
      <GridView :fileList="fileList" :isFitWidth="isFitWidth"/>
      <!-- <TableView :fileList="fileList"/> -->

      <!-- splitter -->
      <div v-if="showPreview" class="w-1 hover:bg-sky-700 cursor-ew-resize" @mousedown="startDragging"></div>

      <!-- preview pane -->
      <div v-if="showPreview" class="w-96 t-color-bg rounded-ss-lg" :style="{ width: previewPaneWidth + 'px' }">
        <div v-if="gContentIndex >= 0 && gContentIndex < fileList.length" 
          class="h-full flex flex-col items-center justify-center break-all"
        >
          <img class="h-full w-full p-1 rounded-lg object-contain" :src="imageSrc" @load="onImageLoad" />
          <div class="fixed p-2 bottom-0 flex flex-col items-center"> 
            <p>{{ fileList[gContentIndex].name }}</p>
            <p>{{ formatFileSize(fileList[gContentIndex].size) }}</p>
            <!-- <p>{{ fileList[gContentIndex].width }}x{{ fileList[gContentIndex].height }}</p> -->
            <p></p>
          </div>
        </div>

        <div v-else class="h-full flex items-center justify-center">
          <p>{{ $t('preview_no_file') }}</p>
        </div>

      </div>

    </div>

  </div>

</template>


<script setup>
import { ref, watch, computed, inject, onMounted, onBeforeUnmount } from 'vue';
import { invoke } from '@tauri-apps/api';
// import TableView from '@/components/TableView.vue';
import ProgressBar from '@/components/ProgressBar.vue';
import GridView  from '@/components/GridView.vue';
import { THUMBNAIL_SIZE, FILES_PAGE_SIZE, formatFileSize, formatDate } from '@/common/utils';
import { format } from 'date-fns';

/// i18n
import { useI18n } from 'vue-i18n';
const { locale, messages } = useI18n();
const localeMsg = computed(() => messages.value[locale.value]);

// Import the SVG file as a Vue component
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

// global states
const gToolbarIndex = inject('gToolbarIndex'); // global toolbar index
const gAlbums = inject('gAlbums');       // global albums
const gAlbumId = inject('gAlbumId');     // global album id
const gFolderId = inject('gFolderId');   // global folder id

const gCalendarYear = inject('gCalendarYear');    // global calendar year
const gCalendarMonth = inject('gCalendarMonth');  // global calendar month
const gCalendarDate = inject('gCalendarDate');    // global calendar date

const gCameraMake = inject('gCameraMake');     // global camera make
const gCameraModel = inject('gCameraModel');   // global camera model

const gContentIndex = inject('gContentIndex'); // global selected item index

// file list
const fileList = ref([]);
const subFolderList = ref([]);  // sub-folder list in the current folder

const currentFolder = ref('');
const currentCamera = ref({make: null, model: null});

// toolbar status
const isFitWidth = ref(false); // fit width status
const isFavorite = ref(false); // favorite status

// sorting
const sortingAsc = ref(true); // sorting order
const sortingType = ref('taken_date'); // sorting type

// progress bar
const thumbCount = ref(0);      // thumbnail count (from 0 to fileList.length)

// preview 
const showPreview = ref(false);     // show image preview
const previewPaneWidth = ref(300);  // using for resizing the preview pane
const isDragging = ref(false);      // dragging splitter to resize preview pane
const imageSrc = ref('/src/assets/photo.svg'); // preview image source

onMounted(() => {
  document.addEventListener('mouseup', stopDragging);
})

onBeforeUnmount(() => {
  document.removeEventListener('mouseup', stopDragging);
})


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
  if (isDragging.value) {
    const windowWidth = document.documentElement.clientWidth -2 ; // -2 for border width(1px) * 2
    previewPaneWidth.value = Math.max(windowWidth - event.clientX, 200); 
  }
}


/// auto update the titlebar when reference data changes
const title = computed(() => {
  let title = '';
  // let selectedFileName = fileList.value.length > 0 && gContentIndex.value > -1 ? ` > ${fileList.value[gContentIndex.value].name}` : '';
  
  switch (gToolbarIndex.value) {
    case 1:   // album
      if (gAlbumId.value) {
        // get the selected album
        const album = gAlbums.value.find(album => album.id === gAlbumId.value);

        if(gFolderId.value === album.folderId) { // current folder is album path
          currentFolder.value = album;
          title += `${album.name}`;
        } else {  // get the select folder
          currentFolder.value = getFolder(album, gFolderId.value);
          title += `${album.name} > ${currentFolder.value.name}`;
        }
      } 
      break;
    case 2:  // calendar
      if (gCalendarYear.value && gCalendarMonth.value && gCalendarDate.value) {
        if (gCalendarDate.value === -1) {     // monthly
          title = formatDate(gCalendarYear.value, gCalendarMonth.value, 1, localeMsg.value.month_format);
        } else if (gCalendarDate.value > 0) { // daily
          title = formatDate(gCalendarYear.value, gCalendarMonth.value, gCalendarDate.value, localeMsg.value.date_format_long);
        }
      }
      break;
    case 3:  // map
      title += `${props.titlebar}`;
      break;
    case 4:  // people
      title += `${props.titlebar}`;
      break;
    case 5:  // camera
      if (gCameraMake.value) {
        if (gCameraModel.value) {
          currentCamera.value = { make: gCameraMake.value , model: gCameraModel.value };
          title += `${gCameraMake.value} > ${gCameraModel.value}`;
        } else {
          title += `${gCameraMake.value}`;
        }
      } 
      break;
    default:
      title = props.titlebar;
  }

  return title.length > 0 ? title : props.titlebar;
  // return title.length > 0 ? title + selectedFileName : props.titlebar;
});


/// Watch for changes in album_id and update filelist accordingly
watch(gAlbumId, async (newAlbumId) => {
  console.log('watch - gAlbumId:', newAlbumId);
  // no album is selected
  if (!newAlbumId) {
    fileList.value = [];
  }
});


/// Watch for changes in toolbar index and update filelist accordingly
watch([gToolbarIndex, isFavorite], async ([newIndex, newFavorite]) => {
  console.log('watch - gToolbarIndex:', newIndex);
  if(newIndex) {
    fileList.value = [];
  }

  switch(newIndex) {
    case 0:
      getAllFiles();
      break;
    case 1:   // album
      if (gAlbumId.value) {
        readFolder(currentFolder.value.path);
      };
      break;
    case 2:   // calendar
      if (gCalendarYear.value && gCalendarMonth.value && gCalendarDate.value) {
        getCalendarFiles(gCalendarYear.value, gCalendarMonth.value, gCalendarDate.value);
      }
      break;
    case 3:   // map
      break;
    case 4:   // people
      break;
    case 5:   // camera
      if (gCameraMake.value) {
        getCameraFiles(gCameraMake.value, gCameraModel.value);
      };
      break;
  }
});


/// Watch for changes in filePath and update filelist accordingly
watch(currentFolder, async (newFolder) => {
  console.log('watch - currentFolder:', newFolder);
  if (newFolder) {
    // read the files in the new folder
    readFolder(newFolder.path);
  }
});

watch([gCalendarYear, gCalendarMonth, gCalendarDate], async ([year, month, date]) => {
  console.log('watch - gCalendarYear:', year, month, date);
  if (year && month && date) {
    getCalendarFiles(year, month, date);
  }
});

watch(gCameraModel, async (newModel) => {
  console.log('watch - gCameraModel:', newModel);
  if(newModel) {
    getCameraFiles(gCameraMake.value, newModel);
  } else {
    fileList.value = [];
  }
});

// watch for changes in the file list
watch(gContentIndex, (newIndex) => {
  if (newIndex >= 0 && newIndex < fileList.value.length) {
    imageSrc.value = fileList.value[newIndex].thumbnail ? fileList.value[newIndex].thumbnail : '/src/assets/photo.svg';
  } else {
    imageSrc.value = '/src/assets/photo.svg';
  }
});

const onImageLoad = async () => {
  let filePath = fileList.value[gContentIndex.value].file_path;
  console.log('onImageLoad:', filePath);
  try {
    const imageBase64 = await invoke('get_file_image', { filePath });
    imageSrc.value = `data:image/jpeg;base64,${imageBase64}`;
  } catch (error) {
    imageSrc.value = '/src/assets/photo.svg';
  }
}

/// toggle the fit width status
function toggleFitWidth() {
  isFitWidth.value = !isFitWidth.value;
}

/// toggle the favorite status
function toggleFavorite() {
  isFavorite.value = !isFavorite.value;
}

/// toggle the sorting order
function toggleSortingOrder() {
  sortingAsc.value = !sortingAsc.value;
  fileList.value = [...fileList.value].reverse();
  if (gContentIndex.value >= 0) {
    gContentIndex.value = fileList.value.length - 1 - gContentIndex.value;
  }
  console.log('toggleSortingOrder:', sortingAsc.value, fileList.value);
}

/// toggle the preview status
function togglePreview() {
  showPreview.value = !showPreview.value;
}


/// get the selected sub-folder by folder id
function getFolder(folder, folderId) {
  if (folder.id === folderId) {
    return folder;
  } else if (folder.children) {
    for (let child of folder.children) {
        const result = getFolder(child, folderId);
        if (result) {
          return result;
        }
    }
  }
  return null;
}


/// read all files under the path
async function readFolder(path) {
  try {
    // read the list of files
    fileList.value = await invoke('read_folder', { folderId: gFolderId.value, path: path });

    // reverse the fileList if sorting order is descending
    sortFileList(sortingType.value, sortingAsc.value);
    console.log('readFolder:', fileList.value);

    getFileThumb(fileList.value);
  } catch (error) {
    console.error('readFolder error:', error);
  }
};


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

/// get all files
async function getAllFiles() {
  try {
    fileList.value = await invoke('get_all_files', { isFavorite: isFavorite.value, offset: 0, pageSize: FILES_PAGE_SIZE });
    console.log('getAllFiles:', fileList.value);

    getFileThumb(fileList.value); 
  } catch (error) {
    console.error('getAllFiles error:', error);
  }
}


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
    console.log('getCalendarFiles:', fileList.value);

    getFileThumb(fileList.value);
  } catch (error) {
    console.error('getCalendarFiles error:', error);
  }
}


/// get all files under the camera make and model
async function getCameraFiles(make, model) {
  try {
    fileList.value = await invoke('get_camera_files', { make: make, model: model });
    console.log('getCameraFiles:', fileList.value);

    getFileThumb(fileList.value); 
  } catch (error) {
    console.error('getCameraFiles error:', error);
  }
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
      // console.log('getFileThumb:', file.file_path);
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

</script>

<style scoped>
* {
  user-select: none;
}
</style>
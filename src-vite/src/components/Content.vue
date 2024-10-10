<template>

  <div class="flex flex-col w-full">

    <!-- title bar -->
    <div class="p-2" style="user-select: none;">
      <div class=" flex flex-row items-center justify-between">
        {{ title }}
        <div class="flex space-x-4">
          <IconUnFavorite v-if="!isFavorite" class="t-icon-hover hover:text-red-600" @click="toggleFavorite" />
          <IconFavorite v-if="isFavorite" class="t-icon-hover text-red-600 hover:text-red-600" @click="toggleFavorite" />
          <component :is="IconTag" class="t-icon-hover" />
          <component :is="sortingAsc ? IconSortingAsc : IconSortingDesc" class="t-icon-hover" @click="toggleSortingOrder" />
        </div>
      </div>
      <div>
        {{ fileList.length }} files
      </div>
    </div>

    <!-- grid view -->
    <GridView :fileList="fileList"/>
    <!-- <TableView :fileList="fileList"/> -->

  </div>

</template>


<script setup>
import { ref, watch, computed, inject  } from 'vue';
import { invoke } from '@tauri-apps/api';
// import TableView from '@/components/TableView.vue';
import GridView  from '@/components/GridView.vue';
import { THUMBNAIL_SIZE } from '@/common/utils';

/// i18n
import { useI18n } from 'vue-i18n';
const { locale, messages } = useI18n();
const msg = computed(() => messages.value[locale.value]);

// Import the SVG file as a Vue component
import IconUnFavorite from '@/assets/heart.svg';
import IconFavorite from '@/assets/heart-solid.svg';
import IconTag from '@/assets/tag.svg';
import IconSortingAsc from '@/assets/sorting-asc.svg';
import IconSortingDesc from '@/assets/sorting-desc.svg';


const props = defineProps({
  titlebar: String
});


const gToolbarIndex = inject('gToolbarIndex'); // global toolbar index
const gAlbums = inject('gAlbums');       // global albums
const gAlbumId = inject('gAlbumId');     // global album id
const gFolderId = inject('gFolderId');   // global folder id

const gSelectItemIndex = inject('gSelectItemIndex'); // global selected item index

const gCameraMake = inject('gCameraMake');     // global camera make
const gCameraModel = inject('gCameraModel');   // global camera model

const currentFolder = ref('');
const currentCamera = ref({make: null, model: null});
const fileList = ref([]);

const isFavorite = ref(false); // favorite status

const sortingAsc = ref(true); // sorting order
const sortingType = ref('size'); // sorting type

/// auto update the titlebar when reference data changes
const title = computed(() => {
  let subTitle = '';
  let selectedFileName = fileList.value.length > 0 && gSelectItemIndex.value > -1 ? ` > ${fileList.value[gSelectItemIndex.value].name}` : '';
  
  if (gToolbarIndex.value === 1) {    // album view
    if (gAlbumId.value) {
      // get the selected album
      const album = gAlbums.value.find(album => album.id === gAlbumId.value);

      if(gFolderId.value === album.folderId) { // current folder is album path
        currentFolder.value = album;
        subTitle += ` > ${album.name}`;
      } else {  // get the select folder
        currentFolder.value = getFolder(album, gFolderId.value);
        subTitle += ` > ${album.name} > ${currentFolder.value.name}`;
      }
    }
  } else if (gToolbarIndex.value === 5) {   // camera view
    if (gCameraMake.value) {
      if (gCameraModel.value) {
        currentCamera.value = { make: gCameraMake.value , model: gCameraModel.value };
        subTitle += ` > ${gCameraMake.value} > ${gCameraModel.value}`;
      } else {
        subTitle += ` > ${gCameraMake.value}`;
      }
    }
  }
  return props.titlebar + subTitle + selectedFileName;
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
watch(gToolbarIndex, async (newIndex) => {
  console.log('watch - gToolbarIndex:', newIndex);
  if (newIndex === 1) {
    if (gAlbumId.value) {
      await getFiles(currentFolder.value.path);
    }
  } else if (newIndex === 5) {
    if (gCameraMake.value) {
      await getCameraFiles(gCameraMake.value, gCameraModel.value);
    }
  }
});


/// Watch for changes in filePath and update filelist accordingly
watch(currentFolder, async (newFolder) => {
  console.log('watch - currentFolder:', newFolder);
  if (newFolder) {
    // reset the selected item index
    gSelectItemIndex.value = -1;  // before get files
    // Fetch the files in the new folder
    await getFiles(newFolder.path);
  }
});


watch(gCameraModel, async (newModel) => {
  console.log('watch - gCameraModel:', newModel);
  if(newModel) {
    await getCameraFiles(gCameraMake.value, newModel);
  } else {
    fileList.value = [];
  }
});

/// toggle the favorite status
function toggleFavorite() {
  isFavorite.value = !isFavorite.value;
}

/// toggle the sorting order
function toggleSortingOrder() {
  sortingAsc.value = !sortingAsc.value;
  fileList.value = [...fileList.value].reverse();
  if (gSelectItemIndex.value >= 0) {
    gSelectItemIndex.value = fileList.value.length - 1 - gSelectItemIndex.value;
  }
  console.log('toggleSortingOrder:', sortingAsc.value, fileList.value);
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


/// get all files under the path
async function getFiles(path) {
  try {
    // Fetch the list of files
    fileList.value = await invoke('get_files', { folderId: gFolderId.value, path: path });

    // reverse the fileList if sorting order is descending
    sortFileList(sortingType.value, sortingAsc.value);
    console.log('invoke - getFiles:', fileList.value);

    await getFileThumb(fileList.value);
  } catch (error) {
    console.error('invoke - getFiles error:', error);
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
      case "date":
        result = a.modified_at - b.modified_at;
        break;
      default:
        return 0; // No sorting if the sorting type is unrecognized
    }

    return isAccending ? result : -result;
  });
}


/// get all files under the camera make and model
async function getCameraFiles(make, model) {
  try {
    fileList.value = await invoke('get_camera_files', { make: make, model: model });
    console.log('invoke - getCameraFiles:', fileList.value);

    await getFileThumb(fileList.value); 
  } catch (error) {
    console.error('invoke - getCameraFiles error:', error);
  }
}


/// get the thumbnail for each file in mutil-thread
async function getFileThumb(files) {
  try {
    const thumbnailPromises = files.map(async (file) => {
      console.log('getFileThumb:', file.file_path);

      const thumb = await invoke('get_file_thumb', { 
        fileId: file.id,
        filePath: file.file_path,
        orientation: file.e_orientation ? file.e_orientation : 0,
        thumbnailSize: THUMBNAIL_SIZE
      });

      file.thumbnail = `data:image/jpeg;base64,${thumb.thumb_data_base64}`;
      console.log('invoke - getFileThumb:', file);
    });

    // Wait for all thumbnail promises to resolve in parallel
    await Promise.all(thumbnailPromises);

  } catch (error) {
    console.error('invoke - getFileThumb error:', error);
  }
}

</script>
  
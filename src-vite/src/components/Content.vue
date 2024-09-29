<template>

  <!-- title bar -->
  <div class="absolute flex flex-row px-2 py-3 items-center justify-between w-full">
    {{ title }}
    <div class="flex">
      <!-- <v-slider
        class="w-32" color="primary" thumb-size="18"  thumb-color="secondary"
        @change="changeThumbnailSize"
      /> -->
      <IconPhoto class="p-1 hover:text-gray-200 transition-colors duration-300" @click="clickPhoto()" />
      <IconVideo class="p-1 hover:text-gray-200 transition-colors duration-300" @click="clickVideo()" />
      <IconMusic class="p-1 hover:text-gray-200 transition-colors duration-300" @click="" />
    </div>
  </div>
  <!-- file list view -->
  <GridView :fileList="fileList"/>
  <!-- <TableView :fileList="fileList"/> -->

</template>


<script setup>
import { ref, watch, computed, inject  } from 'vue';
import { invoke } from '@tauri-apps/api';
import TableView from '@/components/TableView.vue';
import GridView  from '@/components/GridView.vue';
import { THUMBNAIL_SIZE } from '@/common/utils';

/// i18n
import { useI18n } from 'vue-i18n';
const { locale, messages } = useI18n();
const msg = computed(() => messages.value[locale.value]);

// Import the SVG file as a Vue component
import IconPhoto from '@/assets/photo.svg';  
import IconVideo from '@/assets/film.svg';  
import IconMusic from '@/assets/musical-note.svg';  


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


/// click file to open a new windows to display the image
async function clickPhoto() {
  console.log('clickPhoto...');
}


/// click video to open a new windows to display the video
const clickVideo = () => {
  console.log('clickVideo...');
};


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
    console.log('invoke - getFiles:', fileList.value);

    await getFileThumb(fileList.value);
  } catch (error) {
    console.error('invoke - getFiles error:', error);
  }
};


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
  
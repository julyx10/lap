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
  <!-- table -->
  <!-- <TableView :filePath="currentFolder.path" :fileList="fileList"/> -->
  <GridView :fileList="fileList"/>
  <!-- <GridView :filePath="currentFolder.path" :fileList="fileList"/> -->

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

const gCameraMake = inject('gCameraMake');     // global camera make
const gCameraModel = inject('gCameraModel');   // global camera model

const currentFolder = ref('');
const currentCamera = ref({make: null, model: null});
const fileList = ref([]);

// use a token that signals when the current folder changes, 
// and check this token inside the thumbnail generation loop
let cancelToken = { cancelled: false };


/// auto update the titlebar when reference data changes
const title = computed(() => {
  let subTitle = ' > ';
  
  if (gToolbarIndex.value === 1) {    // album view
    if (gAlbumId.value) {
      // get the selected album
      const album = gAlbums.value.find(album => album.id === gAlbumId.value);

      if(gFolderId.value === album.folderId) { // current folder is album path
        currentFolder.value = album;
        subTitle += album.name;
      } else {  // get the select folder
        currentFolder.value = getFolder(album, gFolderId.value);
        subTitle += `${album.name} > ${currentFolder.value.name}`;
      }
    }
  } else if (gToolbarIndex.value === 5) {   // camera view
    if (gCameraMake.value) {
      if (gCameraModel.value) {
        currentCamera.value = { make: gCameraMake.value , model: gCameraModel.value };
        subTitle += `${gCameraMake.value} > ${gCameraModel.value}`;
      } else {
        subTitle += gCameraMake.value;
      }
    }
  }
  return props.titlebar + subTitle;
});


/// Watch for changes in album_id and update filelist accordingly
watch(gAlbumId, async (newAlbumId) => {
  // no album is selected
  if (!newAlbumId) {
    fileList.value = [];
  }
});


/// Watch for changes in toolbar index and update filelist accordingly
watch(gToolbarIndex, async (newIndex) => {
  console.log('gToolbarIndex:', newIndex);
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
  if (newFolder) {
    console.log('currentFolder:', newFolder);

    // Invalidate ongoing thumbnail fetching when folder changes
    cancelToken.cancelled = true;

    // Create a new cancel token for the new folder
    cancelToken = { cancelled: false };

    // Fetch the files in the new folder
    await getFiles(newFolder.path);
  }
});


watch(currentCamera, async (newCamera) => {
  console.log('currentCamera:', newCamera);
  if(newCamera.make) {
    await getCameraFiles(newCamera.make, newCamera.model);
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
    console.log('getFiles:', fileList.value);

    // Get thumbnails in batches of 5
    const chunkSize = 5;
    for (let i = 0; i < fileList.value.length; i += chunkSize) {
      // Get the next batch of files
      const fileChunk = fileList.value.slice(i, i + chunkSize);
      await getFileThumb(fileChunk, cancelToken); // Pass the current chunk to get thumbnails
    }
  } catch (error) {
    console.error('getFiles error:', error);
  }
};


/// get all files under the camera make and model
async function getCameraFiles(make, model) {
  try {
    fileList.value = await invoke('get_camera_files', { make: make, model: model });
    console.log('getCameraFiles:', fileList.value);

    await getFileThumb(fileList.value, cancelToken); 
  } catch (error) {
    console.error('getCameraFiles error:', error);
  }
}

/// get the thumbnail for each file in mutil-thread
async function getFileThumb(files, token) {
  try {
    // Create an array of promises for each file's thumbnail generation
    const thumbnailPromises = files.map(async (file) => {
      // Check if the operation has been cancelled
      if (token.cancelled) {
        console.log('getFileThumb -- Thumbnail generation cancelled');
        return;
      }

      console.log('getFileThumb:', file.file_path);

      const thumb = await invoke('get_file_thumb', { 
        fileId: file.id,
        filePath: file.file_path,
        orientation: file.e_orientation ? file.e_orientation : 0,
        thumbnailSize: THUMBNAIL_SIZE
      });
      console.log('getFileThumb:', thumb);

      if (!token.cancelled) {
        file.thumbnail = `data:image/jpeg;base64,${thumb.thumb_data_base64}`;
        console.log('getFileThumb:', file);
      }
    });

    // Wait for all thumbnail promises to resolve in parallel
    await Promise.all(thumbnailPromises);

  } catch (error) {
    console.error('getFileThumb error:', error);
  }
}


</script>
  
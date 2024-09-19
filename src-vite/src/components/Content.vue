<template>

  <!-- title bar -->
  <div class="absolute flex flex-row px-2 py-3 items-center justify-between w-full">
    {{ title }}
    <div class="flex">
      <IconPhoto class="p-1 hover:text-gray-200 transition-colors duration-300" @click="clickPhoto()" />
      <IconVideo class="p-1 hover:text-gray-200 transition-colors duration-300" @click="clickVideo()" />
      <IconMusic class="p-1 hover:text-gray-200 transition-colors duration-300" @click="" />
    </div>
  </div>

  <TableView :filePath="currentFolder.path" :fileList="fileList"/>

</template>


<script setup>
import { ref, watch, computed, inject  } from 'vue';
import { invoke } from '@tauri-apps/api';
import TableView from '@/components/TableView.vue';

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

const gAlbums = inject('gAlbums');         // global albums
const gAlbumId = inject('gAlbumId');     // global album id
const gFolderId = inject('gFolderId');   // global folder id

const currentFolder = ref('');
const fileList = ref([]);

// use a token that signals when the currentFolder changes, 
// and check this token inside the thumbnail generation loop
let cancelToken = { cancelled: false };

/// Display the titlebar
const title = computed(() => {
  // album view
  if (gToolbarIndex.value === 1) {
    if (gAlbumId.value) {
      // get the selected album
      const album = gAlbums.value.find(album => album.id === gAlbumId.value);

      if(gFolderId.value) {
        // get the select folder
        currentFolder.value = getFolder(album, gFolderId.value);
        console.log('currentFolder...', currentFolder.value);

        return album.name + ' > ' + currentFolder.value.name;
      } else {
        return album.name + ' > ' + msg.value.allphotos;
      }
    } else {
      return props.titlebar;
    }
  }
});

/// Watch for changes in album_id and update filelist accordingly
watch(gAlbumId, async (newAlbumId) => {
  // no album is selected
  if (!newAlbumId) {
    fileList.value = [];
  }
});

/// Watch for changes in filePath and update filelist accordingly
watch(currentFolder, async (newFolder) => {
  if (newFolder) {
    // Invalidate ongoing thumbnail fetching when folder changes
    cancelToken.cancelled = true;

    // Create a new cancel token for the new folder
    cancelToken = { cancelled: false };

    // Fetch the files in the new folder
    await getFiles(newFolder.path);
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


/// get the selected sub-folder of the album
function getFolder(album, child_id) {
  if (album.id === child_id) {
    return album;
  } else if (album.children) {
    for (let child of album.children) {
        const result = getFolder(child, child_id);
        if (result) {
          return result;
        }
    }
  }
  return null;
}


/// try to get all files under the path
async function getFiles(path) {
  try {
    // Fetch the list of files
    fileList.value = await invoke('get_files', { folderId: gFolderId.value, path: path });

    // Once file list are retrieved, get thumbnail for each file
    getFileThumb(cancelToken)
    
    console.log('getFiles:', fileList.value);
  } catch (error) {
    console.error('getFiles error:', error);
  }
};


/// Get the thumbnail for each file in mutil-thread
async function getFileThumb(token) {
  try {
    // Create an array of promises for each file's thumbnail generation
    const thumbnailPromises = fileList.value.map(async (file) => {
      // Check if the operation has been cancelled
      if (token.cancelled) {
        console.log('getFileThumb -- Thumbnail generation cancelled');
        return;
      }

      const filePath = `${currentFolder.value.path}\\${file.name}`;
      console.log('getFileThumb:', filePath);

      const thumbnail = await invoke('get_file_thumb', { fileId: file.id, path: filePath });
      if (!token.cancelled) {
        // Convert each Base64 string into a data URL for display
        file.thumbnail = `data:image/png;base64,${thumbnail}`;
      }
    });

    // Wait for all thumbnail promises to resolve in parallel
    await Promise.all(thumbnailPromises);

  } catch (error) {
    console.error('getFileThumb error:', error);
  }
}


</script>
  
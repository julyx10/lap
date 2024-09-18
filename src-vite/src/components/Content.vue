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

  <TableView :file_path="current_folder.path" :file_list="file_list"/>

</template>


<script setup>
import { ref, watch, computed, inject  } from 'vue';
// import { invoke } from '@tauri-apps/api/tauri';
import { invoke } from '@tauri-apps/api';
import TableView from '@/components/TableView.vue';

/// i18n
import { useI18n } from 'vue-i18n';
const { locale, messages } = useI18n();
const msg = computed(() => messages.value[locale.value]);

// Import the SVG file as a Vue component
// import IconStar from '@/assets/star.svg';
// import IconTag from '@/assets/tag.svg';  
import IconPhoto from '@/assets/photo.svg';  
import IconVideo from '@/assets/film.svg';  
import IconMusic from '@/assets/musical-note.svg';  


const props = defineProps({
  titlebar: String
});

const g_toolbar_index = inject('g_toolbar_index'); // global toolbar index

const g_albums = inject('g_albums');         // global albums
const g_album_id = inject('g_album_id');     // global album id
const g_folder_id = inject('g_folder_id');   // global folder id

const current_folder = ref('');
const file_list = ref([]);

// use a token that signals when the current_folder changes, 
// and check this token inside the thumbnail generation loop
let cancelToken = { cancelled: false };

/// Display the titlebar
const title = computed(() => {
  // album view
  if (g_toolbar_index.value === 1) {
    if (g_album_id.value) {
      // get the selected album
      const album = g_albums.value.find(album => album.id === g_album_id.value);

      if(g_folder_id.value) {
        // get the select folder
        current_folder.value = getFolder(album, g_folder_id.value);
        console.log('current_folder...', current_folder.value);

        return album.name + ' > ' + current_folder.value.name;
      } else {
        return album.name + ' > ' + msg.value.allphotos;
      }
    } else {
      return props.titlebar;
    }
  }
});

/// Watch for changes in album_id and update filelist accordingly
watch(g_album_id, async (new_album_id) => {
  // no album is selected
  if (!new_album_id) {
    file_list.value = [];
  }
});

/// Watch for changes in file_path and update filelist accordingly
watch(current_folder, async (new_folder) => {
  if (new_folder) {
    // Invalidate ongoing thumbnail fetching when folder changes
    cancelToken.cancelled = true;

    // Create a new cancel token for the new folder
    cancelToken = { cancelled: false };

    // Fetch the files in the new folder
    await getFiles(new_folder.path);
  }
});


/// click file to open a new windows to display the image
async function clickPhoto() {

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
    file_list.value = await invoke('get_files', { folderId: g_folder_id.value, path: path });

    // Once file list are retrieved, get thumbnail for each file
    getFileThumb(cancelToken)
    
    console.log('getFiles:', file_list.value);
  } catch (error) {
    console.error('getFiles error:', error);
  }
};


/// Get the thumbnail for each file in mutil-thread
async function getFileThumb(token) {
  try {
    // Create an array of promises for each file's thumbnail generation
    const thumbnailPromises = file_list.value.map(async (file) => {
      // Check if the operation has been cancelled
      if (token.cancelled) {
        console.log('getFileThumb -- Thumbnail generation cancelled');
        return;
      }

      const file_path = `${current_folder.value.path}\\${file.name}`;
      console.log('getFileThumb:', file_path);

      const thumbnail = await invoke('get_file_thumb', { fileId: file.id, path: file_path });
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
  
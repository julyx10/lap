<template>

<!-- title bar -->
<div class="absolute flex flex-row px-2 py-3 items-center justify-between w-full">
  {{ title }}
  <div class="flex">
    <IconStar class="p-1 hover:text-gray-200 transition-colors duration-300" @click="" />
    <IconTag  class="p-1 hover:text-gray-200 transition-colors duration-300" @click="" />
  </div>
</div>

<!-- list view -->
<div ref="scrollableDiv" class="flex-1 mt-12 overflow-auto scrollbar-thin scrollbar-thumb-gray-700 scrollbar-track-gray-800">
  <table class="min-w-full divide-y divide-gray-600 cursor-pointer">
    <thead>
      <tr>
        <th 
          v-for="(column, index) in msg.file_info_columns" :key="index"
          :class="['text-left',
            index === 0  || index === 5 ? 'text-center' : '',
          ]"
        > {{ column }}</th>
      </tr>
    </thead>
    <tbody>
      <tr 
        v-for="(file, index) in file_list" :key="index" 
        :class="['hover:bg-gray-700', 
          index === selected_file_index ? 'text-gray-300 bg-gray-600' : '',
        ]" 
        @click="clickFile(index)"
      >
        <td class="p-1">
          <img :src="file.thumbnail ? file.thumbnail : '/src/assets/photo.svg'" alt="Thumbnail"/>
          <!-- <img :src="file.thumbnail ? '/public/vite.svg': file.thumbnail" alt="Thumbnail"/> -->
        </td>
        <td>{{ file.name }}</td>
        <td>{{ formatTimestamp(file.created_at) }}</td>
        <td>{{ formatTimestamp(file.modified_at) }}</td>
        <td>{{ file.e_model }}</td>
        <td class="text-right">{{ formatFileSize(file.size) }}</td>
      </tr>
    </tbody>
  </table>
</div>

</template>
  
<script setup>
import { ref, watch, computed, inject  } from 'vue';
import { invoke } from '@tauri-apps/api';
import {formatTimestamp, formatFileSize } from '@/utils';

/// i18n
import { useI18n } from 'vue-i18n';
const { locale, messages } = useI18n();
const msg = computed(() => messages.value[locale.value]);

// Import the SVG file as a Vue component
import IconStar from '@/assets/star.svg';
import IconTag from '@/assets/tag.svg';

const props = defineProps({
  titlebar: String
});

const g_toolbar_index = inject('g_toolbar_index'); // global toolbar index

const g_albums = inject('g_albums');         // global albums
const g_album_id = inject('g_album_id');     // global album id
const g_folder_id = inject('g_folder_id');   // global folder id

const current_folder = ref('');
const file_list = ref([]);
const selected_file_index = ref(null);
const scrollableDiv = ref(null);

const getAlbumById = (id) => g_albums.value.find(album => album.id === id);


/// Display the titlebar
const title = computed(() => {
  // album view
  if (g_toolbar_index.value === 1) {
    if (g_album_id.value) {
      const album = getAlbumById(g_album_id.value);

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
  if (!new_album_id) {
    file_list.value = [];
  } else {
    // 
    // await addFiles(getAlbumById(new_album_id).path);
  }
});

/// Watch for changes in file_path and update filelist accordingly
watch(current_folder, async (new_folder) => {
  if (new_folder) {
    await getFiles(new_folder.path);

    selected_file_index.value = null;

    // set scrollbar position when change currrent folder
    scrollableDiv.value.scrollTop = 0;
  }
});

/// Watch for changes in selected_file_index
watch (selected_file_index, (new_index) => {
  if (new_index !== null) {
    console.log('selected_file_index...', file_list.value[new_index]);
  }
});

/// Click a file
function clickFile(index) {
  selected_file_index.value = index;
}


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
    // Fetch the list of files from the backend (using Tauri invoke)
    file_list.value = await invoke('get_files', { folderId: g_folder_id.value, path: path });
    console.log('getFiles:', file_list.value);

    // Once files are retrieved, offload thumbnail fetching to a Web Worker
    getFileThumb()
    // getThumbnail()
    // fetchThumbnailsWithWorker(path);
  } catch (error) {
    console.error('getFiles error:', error);
  }
};

/// 
async function getFileThumb() {
  try {
    for (let file of file_list.value) {
      const file_path = current_folder.value.path + '\\' + file.name;
      
      const thumbnail = await invoke('get_file_thumb', { fileId: file.id, path: file_path });
      console.log('getFileThumb:', file, file_path, thumbnail);
      // Convert each Base64 string into a data URL for display
      file.thumbnail = `data:image/png;base64,${thumbnail}`;
    }
  } catch (error) {
    console.error('getFileThumb error:', error);
  }
};

// async function getThumbnail() {
//   try {
//     for (let file of file_list.value) {
//       const file_path = current_folder.value.path + '\\' + file.name;
//       console.log('getThumbnail:', file, file_path);

//       // Send the task to the worker
//       worker.postMessage({ fileId: file.id, path: file_path });

//       // Handle the response from the worker
//       worker.onmessage = function (event) {
//         const { fileId, thumbnail } = event.data;
//         const fileToUpdate = file_list.value.find(f => f.id === fileId);
//         if (fileToUpdate) {
//           fileToUpdate.thumbnail = `data:image/png;base64,${thumbnail}`;
//         }
//       };
//     }
//   } catch (error) {
//     console.error('getThumbnail error:', error);
//     }
// }


// // Function to fetch thumbnails using a Web Worker
// function fetchThumbnailsWithWorker(path) {
//   // const worker = new Worker('/thumbnailWorker.js');
//   // Import the worker using Vite's special worker syntax
//   const worker = new Worker(new URL('../workers/thumbnailWorker.js', import.meta.url), { type: 'module' });

//   // Extract cloneable properties (id, name, etc.) from file_list
//   const cloneableFileList = file_list.value.map(file => ({
//     id: file.id,
//     name: file.name
//   }));

//   // Post message to the worker with the simplified file list and folder path
//   worker.postMessage({
//     fileList: cloneableFileList, 
//     currentFolderPath: path
//   });

//   // Listen for messages from the worker
//   worker.onmessage = (e) => {
//     const { success, data, error } = e.data;

//     if (success) {
//       // Update file_list with thumbnails returned by the worker
//       data.forEach((item) => {
//         const file = file_list.value.find(f => f.id === item.file.id);
//         if (file) {
//           file.thumbnail = item.thumbnail;
//         }
//       });
//     } else {
//       console.error('Thumbnail fetch failed:', error);
//     }

//     // Terminate the worker once done
//     worker.terminate();
//   };

// }

</script>
  
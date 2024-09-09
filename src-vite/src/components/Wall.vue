<template>

<div class="flex items-center justify-between">
  {{ title }}
  <div class="flex">
    <IconStar class="p-1 hover:text-gray-200 transition-colors duration-300" @click="" />
    <IconTag  class="p-1 hover:text-gray-200 transition-colors duration-300" @click="" />
  </div>
</div>

<div>

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
        <td class="text-center py-1">{{ index + 1 }}</td>
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

// watch(g_folder_id, async (new_folder_id) => {
//   if (!new_folder_id) {
//     // current_folder.value = getFolder(getAlbumById(g_album_id.value), new_folder_id);
//     console.log('current_folder... is null', new_folder_id);
//     await getImageFiles(getAlbumById(g_album_id.value).path);

//   } else {
//     console.log('current_folder...', new_folder_id);
//   }
// });

/// Watch for changes in file_path and update filelist accordingly
watch(current_folder, async (new_folder) => {
  if (new_folder) {
    await addFiles(new_folder.path);

    selected_file_index.value = null;
  }
});


/// try to add all files under the path
async function addFiles(path) {
  try {
    file_list.value = await invoke('add_files', { folderId: g_folder_id.value, path: path });;
    console.log('addFiles:', file_list.value);
  } catch (error) {
    console.error('addFiles error:', error);
  }
};


/// Click a file
function clickFile(index) {
  selected_file_index.value = index;
}

/// Watch for changes in selected_file_index
watch (selected_file_index, (new_index) => {
  if (new_index !== null) {
    console.log('selected_file_index...', file_list.value[new_index]);
  }
});

</script>
  
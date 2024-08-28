<template>

<div class="flex items-center justify-between">
  {{ title }}
  <div class="flex">
    <IconStar class="p-1 hover:text-gray-200 transition-colors duration-300" @click="" />
    <IconTag  class="p-1 hover:text-gray-200 transition-colors duration-300" @click="" />
  </div>
</div>

<div>

  <table class="min-w-full divide-y divide-gray-200">
    <thead>
      <tr>
        <th>ID</th>
        <th class="p-1">Name</th>
        <th class="p-1">Size</th>
        <th class="p-1">Created</th>
        <th class="p-1">Modified</th>
        <th class="p-1">Accessed</th>
      </tr>
    </thead>
    <tbody>
      <tr v-for="(file, index) in current_files" :key="index">
        <td>{{ index + 1 }}</td>
        <td>{{ file.file_name }}</td>
        <td>{{ file.file_size }}</td>
        <td>{{ file.created.nanos_since_epoch }}</td>
        <td>{{ file.modified }}</td>
        <td>{{ file.accessed }}</td>
      </tr>
    </tbody>
  </table>

</div>

</template>
  
<script setup>
import { ref, watch, computed, inject  } from 'vue';
import { invoke } from '@tauri-apps/api';

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
const current_files = ref([]);

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
    await getImageFiles(new_folder.path);
  }
});


async function getImageFiles(path) {
  try {
    const result = await invoke('read_image_files', { path: path });
    current_files.value = result;
    console.log('getImageFiles:', current_files.value);
  } catch (error) {
    console.error('getImageFiles error:', error);
  }
};

</script>
  
<template>

<div class="flex items-center justify-between">
  {{ title }}, {{filelist.length}} files
  <div class="flex">
    <IconStar class="p-1 hover:text-gray-200 transition-colors duration-300" @click="" />
    <IconTag  class="p-1 hover:text-gray-200 transition-colors duration-300" @click="" />
  </div>
</div>

<div>
  <h2></h2>
  <ul>
    <li v-for="(file, index) in filelist" :key="index">
      <!-- <img :src="file" :alt="`Image ${index + 1}`" class="image-preview" /> -->
      <p>{{ (index + 1) + ' - ' + file }}</p>
    </li>
  </ul>
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
const g_albums = inject('g_albums');               // global albums
const g_album_index = inject('g_album_index'); // global album index
const g_child_id = inject('g_child_id');       // global folder id

const file_path = ref('');
const filelist = ref([]);

// Watch for changes in file_path and update filelist accordingly
watch(file_path, async (newFilePath) => {
  if (newFilePath) {
    await getImageFiles(file_path.value);
  }
});

async function getImageFiles(path) {
  try {
    const result = await invoke('read_image_files', { path: path });
    filelist.value = result;
    console.log('getImageFiles:', filelist.value);
  } catch (error) {
    console.error('getImageFiles error:', error);
  }
};


/// Display the titlebar
const title = computed(() => {
  console.log('wall title:', msg.value);

  // album view
  if (g_toolbar_index.value === 1) {
    if (g_album_index.value >= 0) {
      if(g_child_id.value < 0) {
        return g_albums.value[g_album_index.value].name + ' > ' + msg.value.allphotos;
      } else {
        // get the folder path
        let folder = getChildren(g_albums.value[g_album_index.value], g_child_id.value);
        file_path.value = folder.path;
        console.log('file_path...', file_path.value);

        return g_albums.value[g_album_index.value].name + ' > ' + folder.name;
      }
    } else {
      return props.titlebar;
    }
  }
});





/// get the selected sub-folder of the album
function getChildren(album, child_id) {
  if (album.id === child_id) {
    return album;
  } else if (album.children) {
    for (let child of album.children) {
        const result = getChildren(child, child_id);
        if (result) {
          return result;
        }
    }
  }
  return null;
}

</script>
  
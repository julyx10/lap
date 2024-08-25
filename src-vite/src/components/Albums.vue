<template>
    
  <!-- folder pane title -->
  <div class="flex px-1 py-3 items-center justify-between" style="user-select: none;">
    <div>
      {{ titlebar }}
    </div>
    <div class="flex h-6">
      <IconAdd class="p-1 hover:text-gray-200 transition-colors duration-300" @click="addAlbum" />
      <IconRemove  
        :class="[
          'p-1 ', 
          g_album_index >= 0 ? 'hover:text-gray-200 transition-colors duration-300' : 'text-gray-700'
        ]" 
        @click="removeAlbum" />
      <IconRefresh class="p-1 hover:text-gray-200 transition-colors duration-300" @click="refreshAlbum"/>
    </div>
  </div>

  <!-- Display the fetched albums -->
  <ul v-if="g_albums.length > 0">
    <li v-for="(album, index) in g_albums" :key="index" style="user-select: none;" >
      <div 
        :class="[
          'p-2 flex items-center whitespace-nowrap hover:bg-gray-700', 
          g_album_index === index ? 'text-gray-300 bg-gray-800' : ''
        ]"
        @click="selectAlbum(index)"
      >
        <component :is="album.is_expanded ? IconFolderOpen : IconFolder" class="size-6 pr-1 flex-shrink-0" />
        {{ album.name }}
      </div>
      <FolderTree 
        v-if="album.is_expanded && album.children" 
        :children="album.children" 
        :album_index="index"
      />
    </li>
  </ul>

  <!-- Display message if no albums are found -->
  <div v-else-if="g_albums.length === 0" class=" pt-12 text-center">
    {{ $t('no_albums') }}
  </div>

</template>


<script setup lang="ts">

import { inject, onMounted  } from 'vue';
import { invoke } from '@tauri-apps/api';
import FolderTree from './AlbumsFolders.vue';

// toolbar icons
import IconAdd from '@/assets/folder-plus.svg';
import IconRemove from '@/assets/folder-minus.svg';
import IconRefresh from '@/assets/arrow-path.svg';

// folder icon
import IconFolder from '@/assets/folder.svg';
import IconFolderOpen from '@/assets/folder-open.svg';

const props = defineProps({
  titlebar: {
    type: String,
    required: true
  }
});

const g_albums = inject('g_albums');           // global albums
const g_album_index = inject('g_album_index'); // global album index
const g_child_id = inject('g_child_id');     // global folder id


// Fetch albums on mount
onMounted(() => {
  if (g_albums.value.length === 0) {
    getAlbums();
  }
});


/// Add albums
const addAlbum = async () => {
  try {
    const result = await invoke('add_album');
    await refreshAlbum(); // Refresh albums

    console.log('Add album...', result);
  } catch (error) {
    console.error('Failed to add album:', error);
  }
};


/// Remove albums
const removeAlbum = async () => {
  try {
    if (g_album_index.value >= 0) {
      const result = await invoke('remove_album', { id: g_albums.value[g_album_index.value].id });
      await refreshAlbum(); // Refresh albums
      
      g_album_index.value = - 1;
      console.log('Remove album...', result);
    } else {
      console.error('No album selected', g_album_index.value);
    }
  } catch (error) {
    console.error('Failed to remove album:', error);
  }
};


/// Refresh albums
const refreshAlbum = async () => {
  await getAlbums(); // Refresh albums
  console.log('Refresh albums');
};


/// Select album (on click)
const selectAlbum = async (index) => {
  try {
    g_album_index.value = index;
    g_child_id.value = -1;

    // Toggle album expansion
    g_albums.value[index].is_expanded = !g_albums.value[index].is_expanded; 
    
    if (g_albums.value[index].is_expanded && !g_albums.value[index].children) {
      // Fetch folder tree
      const folders = await invoke('read_folders', { path: g_albums.value[index].path });
      g_albums.value[index].children = folders.children;
    }
    console.log('Select album...', g_album_index, g_albums.value);
  } catch (error) {
    console.error('Error fetching folder tree:', error);
  }
};


/// get children folders
async function getAlbums() {
  try {
    const fetchedAlbums = await invoke('get_albums');
    console.log('fetchedAlbums...', fetchedAlbums);
    if (fetchedAlbums) {
      g_albums.value = fetchedAlbums.map(album => ({
        ...album, 
        is_expanded: false,
        children: null,
      }));
    } 
    console.log('getAlbums...', g_albums.value);

  } catch (error) {
    console.error('Failed to fetch albums:', error);
  }
};


</script>
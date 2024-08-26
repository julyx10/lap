<template>
    
  <!-- folder pane title -->
  <div class="flex px-1 py-3 items-center justify-between" style="user-select: none;">
    <div>
      {{ titlebar }}
    </div>
    <div class="flex h-6">
      <IconAdd class="p-1 hover:text-gray-200 transition-colors duration-300" @click="clickAdd" />
      <IconRemove  
        :class="[
          'p-1 ', 
          g_album_id ? 'hover:text-gray-200 transition-colors duration-300' : 'text-gray-700'
        ]" 
        @click="clickRemove" />
      <IconRefresh class="p-1 hover:text-gray-200 transition-colors duration-300" @click="clickRefresh"/>
    </div>
  </div>

  <!-- Display the fetched albums -->
  <ul v-if="g_albums.length > 0">
    <li v-for="album in g_albums" :key="album.id" style="user-select: none;" >
      <div 
        :class="[
          'p-2 flex items-center whitespace-nowrap hover:bg-gray-700', 
          g_album_id === album.id ? 'text-gray-300 bg-gray-800' : ''
        ]"
        @click="clickAlbum(album)"
        @dblclick="dblclickAlbum(album)"
      >
        <component :is="album.is_expanded ? IconFolderOpen : IconFolder" class="size-6 pr-1 flex-shrink-0" />
        {{ album.name }}
      </div>

      <Folders v-if="album.is_expanded" :album_id="album.id" :children="album.children" />
    </li>
  </ul>

  <!-- Display message if no albums are found -->
  <div v-else class="pt-12 text-center">
    {{ $t('no_albums') }}
  </div>

</template>


<script setup lang="ts">

import { inject, onMounted  } from 'vue';
import { invoke } from '@tauri-apps/api';
import Folders from './AlbumsFolders.vue';

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

const g_albums = inject('g_albums');         // global albums
const g_album_id = inject('g_album_id');     // global album id
const g_folder_id = inject('g_folder_id');   // global folder id


// Fetch albums on mount
onMounted(() => {
  if (g_albums.value.length === 0) {
    getAlbums();
  }
});

const getAlbumById = (id) => g_albums.value.find(album => album.id === id);

/// Add albums
const clickAdd = async () => {
  try {
    const result = await invoke('add_album');
    await clickRefresh(); // Refresh albums

    console.log('Add album...', result);
  } catch (error) {
    console.error('Failed to add album:', error);
  }
};


/// Remove albums
const clickRemove = async () => {
  try {
    if (g_album_id.value) {
      const result = await invoke('remove_album', { id: getAlbumById(g_album_id.value).id });
      await clickRefresh(); // Refresh albums
      
      g_album_id.value = null;
      console.log('Remove album...', result);
    } else {
      console.error('No album selected', g_album_id.value);
    }
  } catch (error) {
    console.error('Failed to remove album:', error);
  }
};


/// Refresh albums
const clickRefresh = async () => {
  await getAlbums(); // Refresh albums
  console.log('Refresh albums');
};


/// click a album to expand or collapse next level folders
const clickAlbum = async (album) => {
  try {
    g_album_id.value = album.id;
    g_folder_id.value = null;

    // Toggle album expansion
    album.is_expanded = !album.is_expanded; 
    
    if (album.is_expanded && !album.children) {
      // Fetch folder tree
      const folders = await invoke('read_folders', { path: album.path });
      album.children = folders.children;
    }
    console.log('click album...', album);
  } catch (error) {
    console.error('Error fetching folder tree:', error);
  }
};


/// double click a album to expand all levels sub-folders
const dblclickAlbum = async (album) => {
  // try {
  //   g_album_index.value = index;
  //   g_folder_id.value = -1;

  //   // expand the album folder
  //   g_albums.value[index].is_expanded = true; 
    
  //   if (!g_albums.value[index].children) {
  //     // Fetch folder tree
  //     const folders = await invoke('read_folders', { path: g_albums.value[index].path });
  //     g_albums.value[index].children = folders.children;
  //   }
  //   console.log('dblclick album...', g_album_index, g_albums.value);
  // } catch (error) {
  //   console.error('Error fetching folder tree:', error);
  // }
};


/// get children folders
async function getAlbums() {
  try {
    const fetchedAlbums = await invoke('get_albums');
    // console.log('fetchedAlbums...', fetchedAlbums);
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
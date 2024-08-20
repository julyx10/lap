<template>
    
  <!-- folder pane title -->
  <div class="flex px-1 py-3 items-center justify-between" style="user-select: none;">
    <div class="text-xl">
      {{ titlebar }}
    </div>
    <div class="flex justify-end size-8">
      <img src="/img64/add.png" alt="Add Folder" class="p-1 my-1 hover:bg-gray-700 hover:rounded" @click="addAlbum" />
      <img src="/img64/reduce.png" alt="Remove a Folder" class="p-1 my-1 hover:bg-gray-700 hover:rounded" @click="removeAlbum"/>
      <img src="/img64/refresh.png" alt="Refresh all Folders" class="p-1 my-1 hover:bg-gray-700 hover:rounded" @click="refreshAlbum"/>
    </div>
  </div>

  <!-- Display the fetched albums -->
  <ul v-if="albums.length > 0">
    <li v-for="(album, index) in albums" 
      :key="index" 
      style="user-select: none;" 
      @click="selectAlbum(index)"
    >
      <div :class="['flex items-center whitespace-nowrap hover:bg-gray-700 hover:rounded',
                     index === album_index ? 'bg-gray-800 rounded' : '']">
        <img class="w-8 p-1" src="/img64/folder.png" alt="Folder">
        {{ album.name }}
      </div>
      <FolderTree v-if="albums[index].folders" :folders="albums[index].folders.children" />
    </li>
  </ul>

  <!-- Display message if no albums are found -->
  <p v-else-if="albums.length === 0" class="font-bold text-gray-500">
    No albums available.
  </p>

  <!-- Display error message if there is an error -->
  <p v-else-if="error" class="text-red-600">{{ error }}</p>

</template>


<script setup lang="ts">

import { ref, shallowRef, onMounted  } from 'vue';
import { invoke } from '@tauri-apps/api';
import FolderTree from './FolderTree.vue';

const props = defineProps({
  titlebar: {
    type: String,
    required: true
  }
});

/// Albums
const albums = ref([]);           // albums list
const album_index = ref(null);    // selected album index
// const folders = shallowRef({});   // folders tree
const error = ref(null);


/// get folders (on mount)
const getAlbums = async () => {
  try {
    const fetchedAlbums = await invoke('get_albums');
    if (Array.isArray(fetchedAlbums)) {
      albums.value = fetchedAlbums.map(album => ({
        ...album, 
        is_collapse: false,
        folders: null,
      }));
      console.log('getAlbums...', albums.value);
    } 
  } catch (error) {
    console.error('Failed to fetch albums:', error);
  }
};
onMounted(getAlbums);


/// Add albums
const addAlbum = async () => {
  try {
    const result = await invoke('add_album');
    console.log('Add album:', result);

    refreshAlbum(); // Refresh albums

  } catch (error) {
    console.error('Failed to add album:', error);
  }
};

/// Remove albums
const removeAlbum = async () => {
  try {
    if (album_index.value === null) {
      console.error('No album selected', album_index.value);
      return;
    }
    const result = await invoke('remove_album', { id: albums.value[album_index.value].id });
    console.log('Remove album:', result);
    album_index.value = null

    refreshAlbum(); // Refresh albums

  } catch (error) {
    console.error('Failed to remove album:', error);
  }
};

/// Refresh albums
const refreshAlbum = async () => {
  await getAlbums(); // Refresh albums
  console.log('Refresh folders clicked');
};


/// Select album (on click)
const selectAlbum = async (index) => {
  try {
    album_index.value = index;
    // Fetch folder tree
    const folders = await invoke('get_folder_tree', { path: albums.value[index].location });
    albums.value[index].folders = folders;
    console.log('Select album...', album_index.value, albums.value);
  } catch (error) {
    console.error('Error fetching folder tree:', error);
  }
};

</script>
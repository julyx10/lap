<template>
    
  <!-- folder pane title -->
  <div class="flex px-1 py-3 items-center justify-between" style="user-select: none;">
    <div class="text-xl">
      {{ titlebar }}
    </div>
    <div class="flex justify-end size-8">
      <img src="/img64/add.png" alt="Add Folder" class="p-1 my-1 hover:bg-gray-700 hover:rounded" @click="addFolder" />
      <img src="/img64/reduce.png" alt="Remove a Folder" class="p-1 my-1 hover:bg-gray-700 hover:rounded" @click="removeFolder"/>
      <img src="/img64/refresh.png" alt="Refresh all Folders" class="p-1 my-1 hover:bg-gray-700 hover:rounded" @click="refreshFolders"/>
    </div>
  </div>

  <!-- Display the fetched albums -->
  <ul v-if="albums.length > 0">
    <li v-for="(album, index) in albums" 
      :key="index" 
      style="user-select: none;" 
      @click="fetchFolderTree(index)"
    >
      <div class="flex items-center hover:bg-gray-700 hover:rounded">
        <img class="size-8 p-1" src="/img64/folder.png" alt="Folder">
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
const albums = ref([]);             // albums list
const folders = shallowRef({});   // folders tree
const error = ref(null);


/// get folders
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


/// Folder Tree
const fetchFolderTree = async (index) => {
  try {
    const folders = await invoke('get_folder_tree', { path: albums.value[index].location });
    albums.value[index].folders = folders;
    console.log('fetchFolderTree...', albums.value);
  } catch (error) {
    console.error('Error fetching folder tree:', error);
  }
  console.log('fetch folder tree...', folders.value);
};


/// Add, Remove, Refresh Folders
const addFolder = async () => {
  // Implement add folder logic
  console.log('Add folder clicked');
};

const removeFolder = async () => {
  // Implement remove folder logic
  console.log('Remove folder clicked');
};

const refreshFolders = async () => {
  await getAlbums(); // Refresh albums
  console.log('Refresh folders clicked');
};

</script>
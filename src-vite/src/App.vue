<template>

  <div class="flex h-full m2 bg-gray-900">

    <!-- left toolbar -->
    <div id="leftToolbar" class="w-12 min-w-12 h-full p-1 border-r border-gray-800">
      <img @click="fetchAlbums" src="/img64/pictures.png"  alt="All pictures" class="p-1 my-2 hover:bg-gray-700 hover:rounded" />
      <img src="/img64/favorites.png" alt="favorites" class="p-1 my-2 hover:bg-gray-700 hover:rounded" />
      <img @click="fetchFolderTree" src="/img64/folder.png" alt="folder" class="p-1 my-2 hover:bg-gray-700 hover:rounded"/>
      <!-- <img id="ltbAddFolder" src="/img64/add.png" alt="Add Folder" class="p-1 my-2 hover:bg-gray-700 hover:rounded" /> -->
      
    </div>

    <!-- left pane -->
    <div id="leftPane" class="w-96 min-w-12 h-full text-white p-4 overflow-auto" >
      Folder: <span id="folderName" class=""></span>
      <div id="folderTree" class="p-2"></div>

      <!-- Display the fetched albums -->
      <ul v-if="albums.length > 0">
        <li v-for="album in albums" :key="album.id">
          {{ album.name }}
        </li> 
      </ul>

      <!-- Display message if no albums are found -->
      <p v-if="albums.length === 0" class="font-bold text-gray-500">
        No albums available.
      </p>

      <!-- Display error message if there is an error -->
      <p v-if="error" class="text-red-600">{{ error }}</p>

      <div>
        <h1>Folder Tree</h1>
        <FolderTree :folderTree="folderTree" />
      </div>

    </div>

    <!-- splitter -->
    <div id="splitter" class="w-2 bg-gray-800 hover:bg-gray-700 cursor-ew-resize" data-direction="horizontal"></div>
    
    <!-- right pane -->
    <div id="rightPane" class="flex-grow min-w-96 h-full bg-gray-800 text-white p-4 overflow-auto">
      <h2 class="text-xl fond-bold">Right Pane</h2>
    </div>

  </div>

</template>


<script setup lang="ts">

import { ref, onMounted, shallowRef  } from 'vue';
import { invoke } from '@tauri-apps/api';
import FolderTree from './components/FolderTree.vue';

/// Albums
const albums = ref([]);
const error = ref(null);

const fetchAlbums = async () => {
  console.log('Fetching albums...');
  try {
    albums.value = await invoke('get_albums');
  } catch (error) {
    error.value = 'Failed to load albums: ' + error.message;
    console.error('Failed to fetch albums:', error);
  }
};

/// Folder Tree
const folderTree = shallowRef({});

const fetchFolderTree = async () => {
  try {
    const tree = await invoke('get_folder_tree', { path: 'C:\\Users\\liuli\\Pictures' });
    console.log('Folder tree:', tree);
    folderTree.value = tree;
  } catch (error) {
    console.error('Error fetching folder tree:', error);
  }
};

// onMounted(fetchFolderTree);

</script>


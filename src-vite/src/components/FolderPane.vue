<template>
    
  <!-- folder pane title -->
  <div class="flex items-center justify-between bg-gray-800 p-2 rounded">
    <div class="flex">
      {{ titlebar }} -
    </div>
    <div class="flex justify-end size-4">
      <img id="ltbAddFolder" src="/img64/add.png" alt="Add Folder" class="hover:bg-gray-700 hover:rounded" @click="" />
      <img id="ltbRemoveFolder" src="/img64/reduce.png" alt="Remove a Folder" class="hover:bg-gray-700 hover:rounded"
        @click="" />
      <img id="ltbRefreshFolders" src="/img64/refresh.png" alt="Refresh all Folders"
        class="hover:bg-gray-700 hover:rounded" @click="fetchFolderTree" />
    </div>
  </div>

  <!-- <div id="folderTree" class="p-2"></div> -->

  <!-- Display the fetched albums -->
  <ul v-if="albums.length > 0">
    <li v-for="album in albums" :key="album.id">
      {{ album.name }}
    </li>
  </ul>

  <!-- Display message if no albums are found -->
  <p v-else-if="albums.length === 0" class="font-bold text-gray-500">
    No albums available.
  </p>

  <!-- Display error message if there is an error -->
  <p v-else-if="error" class="text-red-600">{{ error }}</p>

  <!-- <div>
    <FolderTree :folderTree="folderTree" />
  </div> -->

</template>

<script setup lang="ts">

import { ref, shallowRef, defineProps, onMounted  } from 'vue';
import { invoke } from '@tauri-apps/api';

const props = defineProps({
  titlebar: {
    type: String,
    required: true
  }
});

/// Albums
const albums = ref([]);
const error = ref(null);


/// get folders
const getFolders = async () => {
  console.log('get folders...');
  try {
    albums.value = await invoke('get_albums');
  }
  catch (error) {
    error.value = 'Failed to load folders: ' + error.message;
    console.error('Failed to fetch folders:', error);
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

onMounted(getFolders);

</script>
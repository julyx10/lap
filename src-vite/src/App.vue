<template>

  <div class="flex h-full m2 bg-gray-900">

    <!-- left toolbar -->
    <div id="leftToolbar" class="w-12 min-w-12 h-full p-1 border-r border-gray-800">
      <img src="/img64/pictures.png" alt="All pictures" class="p-1 my-2 hover:bg-gray-700 hover:rounded" />
      <img src="/img64/favorites.png" alt="favorites" class="p-1 my-2 hover:bg-gray-700 hover:rounded" />
      <img src="/img64/folder.png" alt="folder" class="p-1 my-2 hover:bg-gray-700 hover:rounded" />
      <img id="/img64/ltbAddFolder" src="/img64/add.png" alt="Add Folder" class="p-1 my-2 hover:bg-gray-700 hover:rounded" />
    </div>

    <!-- left pane -->
    <div id="leftPane" class="w-96 min-w-12 h-full text-white p-4 overflow-auto" >
      Folder: <span id="folderName" class=""></span>
      <div id="folderTree" class="p-2"></div>

      <!-- Display the fetched albums -->
      <ul v-if="albums.length > 0">
        <li v-for="album in albums" :key="album.id">
          {{ album.name }} - {{ album.location }}
        </li>
      </ul>

      <!-- Display message if no albums are found -->
      <p v-if="albums.length === 0" class="font-bold text-gray-500">
        No albums available.
      </p>

      <!-- Button to fetch albums -->
      <button @click="fetchAlbums" class="mt-4 px-4 py-2 bg-blue-500 text-white rounded">
        Load Albums
      </button>

      <!-- Display error message if there is an error -->
      <p v-if="error" class="text-red-600">{{ error }}</p>

    </div>

    <!-- splitter -->
    <div id="splitter" class="w-2 bg-gray-800 hover:bg-gray-700 cursor-ew-resize" data-direction="horizontal"></div>
    
    <!-- right pane -->
    <div id="rightPane" class="flex-grow min-w-96 h-full bg-gray-800 text-white p-4 overflow-auto">
      <h2 class="text-xl fond-bold">Right Pane</h2>
    </div>

  </div>

</template>


<!-- script of vue -->
<script setup lang="ts">

  import { ref } from 'vue';
  import { invoke } from '@tauri-apps/api';

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

</script>


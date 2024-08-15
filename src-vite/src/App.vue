<template>
  <div>
    <h1 class="text-3xl font-bold text-red-600">Albums</h1>
    
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

</template>

<script>
  import AppLogic from './AppLogic';

  export default {
    ...AppLogic
  };
</script>


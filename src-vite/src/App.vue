<template>
  <router-view />
</template>
 
<script setup>

import { ref, provide } from 'vue';

// albums
provide('gAlbums', ref([]));    // all albums

// content list view
provide('gShowImageViewer', ref(false)); // show image viewer

</script>

<template>
  <router-view />
</template>
 
<script setup>

import { ref, provide } from 'vue';

/// global variables
provide('gToolbarIndex',  ref(1));   // toolbar index

provide('gAlbums', ref([]));      // all albums
provide('gAlbumId', ref(null));   // current album id (selected)
provide('gFolderId', ref(null));  // current folder id (selected)

provide('gCameras', ref([]));        // all cameras
provide('gCameraMake', ref(null));   // camera make
provide('gCameraModel', ref(null));  // camera model

</script>

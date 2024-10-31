<template>
  <router-view />
</template>
 
<script setup>

import { ref, provide } from 'vue';

// albums
provide('gAlbums', ref([]));    // all albums

// calendar
provide('gCalendarYear',  ref(null));  // current year (...2024)
provide('gCalendarMonth', ref(null));  // current month (1-12)
provide('gCalendarDate',  ref(null));  // current date (1-31), -1 means selecting a month

// cameras
provide('gCameras', ref([]));        // all cameras
provide('gCameraMake', ref(null));   // current camera make
provide('gCameraModel', ref(null));  // current camera model

// content list view
provide('gContentIndex', ref(-1));  // selected list item index
provide('gShowImageViewer', ref(false)); // show image viewer

</script>

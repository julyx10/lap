<template>
  <router-view />
</template>
 
<script setup>

import { ref, provide, onMounted, onUnmounted } from 'vue';
// import { invoke } from '@tauri-apps/api/tauri';

/// global variables
// provide('gAppConfig', ref({}));     // app configuration

provide('gToolbarIndex',  ref(1));   // toolbar index

// albums
provide('gAlbums', ref([]));      // all albums
provide('gAlbumId', ref(null));   // current album id (selected)
provide('gFolderId', ref(null));  // current folder id (selected)

// calendar
provide('gCalendarYear', ref(2024));  // calendar year (...2024)
provide('gCalendarMonth', ref(1));    // calendar month (1-12)
provide('gCalendarDate', ref(1));     // calendar date (1-31)

// cameras
provide('gCameras', ref([]));        // all cameras
provide('gCameraMake', ref(null));   // camera make
provide('gCameraModel', ref(null));  // camera model

// content list view
provide('gContentIndex', ref(-1));  // selected list item index

// onMounted(() => {
//   console.log('App.vue mounted');
//   gAppConfig.value = loadConfig();
//   console.log('gAppConfig: ---', gAppConfig.value);
// })


// onUnmounted(() => {
//   saveConfig(gAppConfig.value);
//   console.log('App.vue unmounted');
// })


// const loadConfig = async () => {
//   try {
//     const config = await invoke('load_config');
//     console.log('Loaded config:', config);
//     return config;
//   } catch (error) {
//     console.error('Failed to load config:', error);
//     return null;
//   }
// };


// const saveConfig = async (config) => {
//   try {
//     await invoke('save_config', { config });
//     console.log('Saved config:', config);
//   } catch (error) {
//     console.error('Failed to save config:', error);
//   }
// };

</script>

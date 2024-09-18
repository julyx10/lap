<template>
    <div>
      <img :src="imageSrc" alt="Image" v-if="imageSrc" />
    </div>
  </template>
  
  <script setup>
  
  import { ref, onMounted } from 'vue';
  import { listen } from '@tauri-apps/api/event';
  
  const imageSrc = ref('');
  
  onMounted(() => {
    listen('image-data', (event) => {
      // Set the image source as the Base64-encoded data
      imageSrc.value = `data:image/jpeg;base64,${event.payload.data}`;
    });
  });
  
  </script>
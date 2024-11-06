<template>
  <router-view />
</template>
 
<script setup>

import { ref, provide } from 'vue';

// content list view
provide('gShowImageViewer', ref(false)); // show image viewer

</script>

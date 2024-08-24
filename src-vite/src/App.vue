<template>

  <div class="flex h-screen bg-gray-900 text-gray-500">

    <!-- left toolbar -->
    <div ref="toolbar" class="w-10 px-2 border-gray-800">
      <component
        v-for="(item, index) in icons"
        :key="index"
        :is="item.icon"
        :class="[
          'my-5 hover:text-gray-100 transition-colors duration-300', 
          activeButtonId === index ? 'text-gray-300' : ''
        ]"
        @click="item.onClick"
      />
    </div>
      
    <!-- navigation pane -->
    <div 
      v-if="activeButtonId > 0" 
      class="w-96 p-2 min-w-10 overflow-auto scrollbar-thin scrollbar-thumb-gray-700 scrollbar-track-gray-800" 
      :style="{ width: leftPaneWidth + 'px' }"
    >
      <Albums v-if="activeButtonId === 1" titlebar="Albums"/>
      <Calendar v-else-if="activeButtonId === 2" titlebar="Calendar"/>
    </div>

    <!-- splitter -->
    <div 
      v-if="activeButtonId > 0" 
      class="w-1 hover:bg-sky-700 cursor-ew-resize" 
      @mousedown="startDragging" 
    ></div>
    
    <!-- content area -->
    <div class="flex-1 p-4 bg-gray-800 overflow-auto scrollbar-thin scrollbar-thumb-gray-700 scrollbar-track-gray-800">
      <Files titlebar="Files"/>
    </div>

  </div>

</template>
 

<script setup lang="ts">

import { ref, provide, computed, onMounted, onBeforeUnmount } from 'vue';
import Albums from '@/components/Albums.vue';
import Calendar from '@/components/Calendar.vue';
import Files from '@/components/Content.vue';


/// global variables
provide('g_albums', ref([]));
provide('g_album_index', ref(-1));  // index of the selected album
provide('g_child_id', ref(-1));     // id of the selected sub-folder


/// Toolbar
// Import SVG files as a Vue component
import IconHome from '@/assets/home.svg';
import IconAlbums from '@/assets/photo.svg';
import IconCalendar from '@/assets/calendar.svg';

const icons = [
  { icon: IconHome, onClick: clickHome },
  { icon: IconAlbums, onClick: clickAlbums },
  { icon: IconCalendar, onClick: clickCalendar },
];

const activeButtonId = ref(0);

function clickHome() {
  activeButtonId.value = 0;
}

function clickAlbums() {
  activeButtonId.value = 1;
}

function clickCalendar() {
  activeButtonId.value = 2;
}


/// Splitter
const toolbar = ref(null);
const leftPaneWidth = ref(300); // Default width of the left pane
const isDragging = ref(false);

function startDragging(event) {
  isDragging.value = true;
  document.addEventListener('mousemove', handleMouseMove);
  document.addEventListener('mouseup', stopDragging);
}

function handleMouseMove(event) {
  if (isDragging.value && toolbar.value) {
    const toolbarWidth = toolbar.value.offsetWidth;
    leftPaneWidth.value = Math.max(event.clientX - toolbarWidth, 100); // Adjust for toolbar width and minimum width
  }
}

function stopDragging() {
  isDragging.value = false;
  document.removeEventListener('mousemove', handleMouseMove);
  document.removeEventListener('mouseup', stopDragging);
}

onMounted(() => {
  document.addEventListener('mouseup', stopDragging);
})

onBeforeUnmount(() => {
  document.removeEventListener('mouseup', stopDragging);
})

</script>

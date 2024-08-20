<template>

  <div class="flex h-screen bg-gray-900 text-gray-400">

    <!-- left toolbar -->
    <div ref="toolbar" class="w-12 p-2 border-gray-800">
      <img
        v-for="button in buttons"
        :key="button.id"
        :src="button.image"
        :alt="button.label"
        @click="markAsActive(button.id)"
        :class="[
          'p-1 my-2 hover:bg-gray-600 hover:rounded',
          button.id === activeButtonId ? 'bg-gray-700 rounded' : ''
        ]"
      />
    </div>

    <!-- left pane -->
    <div 
      v-if="activeButtonId > 0" 
      class="w-96 p-2 min-w-12 overflow-auto scrollbar-thin scrollbar-thumb-gray-700 scrollbar-track-gray-800" 
      :style="{ width: leftPaneWidth + 'px' }"
    >
      <Folders v-if="activeButtonId === 1" :titlebar="getButtonLabel"/>
      <Calendar v-else-if="activeButtonId === 2" :titlebar="getButtonLabel"/>
    </div>

    <!-- splitter -->
    <div 
      v-if="activeButtonId > 0" 
      class="w-2 bg-gray-800 hover:bg-gray-700 cursor-ew-resize" 
      @mousedown="startDragging" 
    ></div>
    
    <!-- right pane -->
    <div class="flex-1 p-4 bg-gray-800 overflow-auto">
      <Files titlebar="Files"/>
    </div>

  </div>

</template>
 

<script setup lang="ts">

import { ref, computed, onMounted, onBeforeUnmount } from 'vue';
import Folders from './components/Folders.vue';
import Calendar from './components/Calendar.vue';
import Files from './components/Files.vue';

/// Toolbar
const buttons = ref([
  { id: 0, label: 'All pictures', image: '/img64/pictures.png' },
  { id: 1, label: 'Folders', image: '/img64/folder.png' },
  { id: 2, label: 'Calendar', image: '/img64/calendar.png' },
  // { id: 3, label: 'Favorites', image: '/img64/favorites.png' },
  // { id: 4, label: 'Tags', image: '/img64/tags.png' },
  // { id: 6, label: 'Settings', image: '/img64/settings.png' }
]);

const activeButtonId = ref(0);

function markAsActive(buttonId) {
  activeButtonId.value = buttonId;
}

const getButtonLabel = computed(() => {
  return buttons.value[activeButtonId.value]?.label || '';
});


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

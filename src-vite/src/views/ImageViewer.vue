<template>

<div 
  class="flex relative justify-center items-center h-screen w-screen bg-gray-800 text-gray-500 overflow-hidden"
  @wheel="zoomImage"
  @mousedown="startDragging"
  @mouseup="stopDragging"
  @mousemove="dragImage"
  @mouseleave="stopDragging"
>
  <img
  :src="imageSrc"
  alt="Image Viewer"
  v-if="imageSrc"
  :style="imgStyle"
  class="max-w-full max-h-full object-contain transition-transform duration-150"
  />
  <p v-else>Loading...</p>
  
  <p class="absolute bottom-0 text-gray-500 bg-gray-900 bg-opacity-10 p-2 rounded-lg"> {{ filePath }}</p>
</div>

</template>


<script setup lang="ts">
import { onMounted, onUnmounted, ref, computed} from 'vue';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/tauri';

// Reactive variable to hold the image source
const filePath = ref('');
const imageSrc = ref(null);

// zoom scaling, dragging state, and position
const scale = ref(1); // Default zoom scale
const isDragging = ref(false); // Track if the image is being dragged
const startX = ref(0); // Store initial X position when dragging starts
const startY = ref(0); // Store initial Y position when dragging starts
const translateX = ref(0); // X axis translation (dragging)
const translateY = ref(0); // Y axis translation (dragging)
const lastTranslateX = ref(0); // Last stored X position after drag ends
const lastTranslateY = ref(0); // Last stored Y position after drag ends


// Computed style for the image, combining zoom and translation
const imgStyle = computed(() => ({
  transform: `scale(${scale.value}) translate(${translateX.value}px, ${translateY.value}px)`,
}));


onMounted(() => {
  // Retrieve the file path from the query parameter on initial load
  const urlParams = new URLSearchParams(window.location.search);
  const encodedFilePath = urlParams.get('file');
  if (encodedFilePath) {
    filePath.value = decodeURIComponent(encodedFilePath);
    loadImage();
  }

  // Listen for the 'update-image' event to update the image
  listen('update-image', (event) => {
    const newEncodedFilePath = event.payload.file;
    filePath.value = decodeURIComponent(newEncodedFilePath);
    loadImage();
  });
});


/// Load the image from the file path
async function loadImage() {
  try {
    const imageBase64 = await invoke('get_file_image', { path: filePath.value });
    imageSrc.value = `data:image/jpeg;base64,${imageBase64}`;
  } catch (error) {
    console.error('Error fetching image data:', error);
  }
}

// Function to handle zooming with the mouse wheel
function zoomImage(event) {
  event.preventDefault();

  // Adjust the scale factor based on the wheel scroll (positive = zoom in, negative = zoom out)
  const zoomSpeed = 0.2; // Change this value to adjust zoom speed
  const delta = event.deltaY < 0 ? zoomSpeed : -zoomSpeed;

  // Update the scale factor, with a minimum and maximum zoom level
  scale.value = Math.min(Math.max(0.5, scale.value + delta), 5); // Limit zoom between 0.5x and 5x
}

// Start dragging when the mouse button is pressed
function startDragging(event) {
  isDragging.value = true;
  startX.value = event.clientX - lastTranslateX.value;
  startY.value = event.clientY - lastTranslateY.value;
}

// Stop dragging when the mouse button is released
function stopDragging() {
  if (isDragging.value) {
    lastTranslateX.value = translateX.value;
    lastTranslateY.value = translateY.value;
  }
  isDragging.value = false;
}

// Drag the image while the mouse is moved
function dragImage(event) {
  if (isDragging.value) {
    translateX.value = event.clientX - startX.value;
    translateY.value = event.clientY - startY.value;
  }
}

</script>

<style scoped>
/* Disable text selection while dragging */
* {
  user-select: none;
}
</style>
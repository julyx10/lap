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
    <p v-else>{{ loadError ? loadError : 'Loading...'}}</p>

    <!-- <p class="absolute bottom-0 text-gray-500 bg-gray-900 bg-opacity-10 p-2 rounded-lg">{{ filePath }}</p> -->

    <div v-if="fileInfo" class="absolute top-0 left-0 text-gray-500 bg-gray-900 bg-opacity-10 p-2 rounded-lg hover:text-sky-700">
      <p>{{ fileInfo.name }}</p>
      <p>{{ fileInfo.file_path }}</p>
      <p>File Size: {{ formatFileSize(fileInfo.size) }}</p>
      <p>Resolution: {{ fileInfo.resolution }}</p>
      <p>Created: {{ formatTimestamp(fileInfo.created_at) }}</p>
      <p>Modified: {{ formatTimestamp(fileInfo.modified_at) }}</p>
      <p></p>
      <p>Carema Make: {{ fileInfo.e_make }} </p>
      <p>Carema Model: {{ fileInfo.e_model }}</p>
      <p>Date Time: {{ fileInfo.e_date_time }}</p>
      <p>Exposure Time: {{ fileInfo.e_exposure_time }}</p>
      <p>Focus: {{ fileInfo.e_f_number }}</p>
      <p>ISO: {{ fileInfo.e_iso_speed }}</p>
      <p>Focal Length: {{ fileInfo.e_focal_length }}</p>
    </div>
  </div>

</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/tauri';
import { formatTimestamp, formatFileSize } from '@/common/utils';

const fileInfo = ref(null);       // File info
const filePath = ref('');         // File path
const imageSrc = ref(null);
const loadError = ref(null);

// Zoom scaling, dragging state, and position
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
  try {
    const urlParams = new URLSearchParams(window.location.search);

    // Load the image from the file path
    const encodedFilePath = urlParams.get('filePath');
    filePath.value = decodeURIComponent(encodedFilePath);

    loadImage(filePath.value);

    // Load the file info
    loadFileInfo(urlParams.get('fileId'));
    console.log('fileInfo:', fileInfo.value);

    // Listen for the 'update-image' event to update the image
    listen('update-image', (event) => {
      // Update the image source
      const newEncodedFilePath = event.payload.filePath;
      filePath.value = decodeURIComponent(newEncodedFilePath);

      loadImage(filePath.value);

      // load the file info
      loadFileInfo(event.payload.fileId);
    });
  } catch (error) {
    loadError.value = error;
    imageSrc.value = null;
    console.error('Error loading image:', error);
  }
});


// Load the image from the file path
async function loadImage(filePath) {
  try {
    const imageBase64 = await invoke('get_file_image', { filePath });
    imageSrc.value = `data:image/jpeg;base64,${imageBase64}`;
  } catch (error) {
    loadError.value = error;
    imageSrc.value = null;
    console.error('Error fetching image data:', error);
  }
}


// Load the file info from the file ID
async function loadFileInfo(fileId) {
  try {
    fileInfo.value = await invoke('get_file_info', { fileId: parseInt(fileId, 10) });
  } catch (error) {
    console.error('Error fetching file info:', error);
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
    // Account for zoom level when dragging
    translateX.value = (event.clientX - startX.value) / scale.value;
    translateY.value = (event.clientY - startY.value) / scale.value;
  }
}
</script>

<style scoped>
/* Disable text selection while dragging */
* {
  user-select: none;
}
</style>

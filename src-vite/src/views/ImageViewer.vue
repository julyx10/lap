<template>
  
  <div
    class="flex relative justify-center items-center h-screen w-screen bg-gray-800 text-gray-500 overflow-hidden"
    @wheel="zoomImage"
  >
    <img
      :src="imageSrc"
      alt="Image Viewer"
      v-if="imageSrc"
      :style="imgStyle"
      class="max-w-full max-h-full object-contain transition-transform duration-150"
      @mousedown="startDragging"
      @mouseup="stopDragging"
      @mousemove="dragImage"
      @mouseleave="stopDragging"
    />
    <!-- <v-img v-if="imageSrc" lazy-src="a" :src="imageSrc"></v-img> -->

    <p v-else>{{ loadError ? loadError : 'Loading...'}}</p>

    <!-- <p class="absolute bottom-0 text-gray-500 bg-gray-900 bg-opacity-10 p-2 rounded-lg">{{ filePath }}</p> -->

    <table v-if="fileInfo" class="absolute top-0 left-0 border-separate border-spacing-1 text-gray-500 bg-gray-900 bg-opacity-10 rounded-lg hover:text-gray-500">
      <tr>
        <td>File Name</td>
        <td>{{ fileInfo.name }}</td>
      </tr>
      <tr>
        <td>Resolution</td>
        <td>{{ fileInfo.width }}x{{ fileInfo.height }}</td>
      </tr>
      <tr>
        <td>File Size</td>
        <td>{{ formatFileSize(fileInfo.size) }}</td>
      </tr>
      <tr>
        <td>Created</td>
        <td>{{ formatTimestamp(fileInfo.created_at) }}</td>
      </tr>
      <tr>
        <td>Modified</td>
        <td>{{ formatTimestamp(fileInfo.modified_at) }}</td>
      </tr>
      <tr>
        <td>Camera Make</td>
        <td>{{ fileInfo.e_make }}</td>
      </tr>
      <tr>
        <td>Camera Model</td>
        <td>{{ fileInfo.e_model }}</td>
      </tr>
      <tr>
        <td>Date Taken</td>
        <td>{{ fileInfo.e_date_time }}</td>
      </tr>
      <tr>
        <td>Exposure Time</td>
        <td>{{ fileInfo.e_exposure_time }}</td>
      </tr>
      <tr>
        <td>Aperture</td>
        <td>{{ fileInfo.e_f_number }}</td>
      </tr>
      <tr>
        <td>ISO Speed</td>
        <td>{{ fileInfo.e_iso_speed }}</td>
      </tr>
      <tr>
        <td>Focal Length</td>
        <td>{{ fileInfo.e_focal_length }}</td>
      </tr>
      <!-- <tr>
        <td>Color Type</td>
        <td>{{ fileInfo.i_color_type }}</td>
      </tr>
      <tr>
        <td>Bit Depth</td>
        <td>{{ fileInfo.i_bit_depth }}</td>
      </tr>
      <tr>
        <td>Alpha Channel</td>
        <td>{{ fileInfo.i_has_alpha }}</td>
      </tr> -->
    </table>

  </div>

</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/tauri';
import { formatTimestamp, formatFileSize } from '@/common/utils';
import { c } from 'naive-ui';

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
    console.log('fileInfo: ---', fileInfo.value);
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
  console.log('startDragging:', event);
  isDragging.value = true;
  startX.value = event.clientX - lastTranslateX.value;
  startY.value = event.clientY - lastTranslateY.value;
}

// Stop dragging when the mouse button is released
function stopDragging(event) {
  console.log('stopDragging', event);
  if (isDragging.value) {
    lastTranslateX.value = translateX.value;
    lastTranslateY.value = translateY.value;
  }
  isDragging.value = false;
}

// Drag the image while the mouse is moved
function dragImage(event) {
  if (isDragging.value) {
    console.log('dragImage:', event);
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

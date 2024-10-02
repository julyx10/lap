<template>
  <div class="flex flex-row bg-gray-800 text-gray-500  overflow-hidden">

    <!-- image -->
    <div 
      class="relative flex-1 flex justify-center items-center h-screen overflow-hidden"
    >
      <div class="absolute z-10 flex items-center justify-start left-0 w-20 h-full group hover:text-gray-200 transition-colors duration-300" 
        @click="clickPrev"
      >
        <IconLeft class="hidden group-hover:block" />
      </div>
      <img v-if="imageSrc"
        class="max-h-full max-w-full transition-transform duration-150"
        :src="imageSrc"
        alt="Image Viewer"
        :style="imgStyle"
        @wheel="zoomImage"
        @mousedown="startDragging"
        @mouseup="stopDragging"
        @mousemove="dragImage"
        @mouseleave="stopDragging"
      />
      <p v-else>{{ loadError ? loadError : 'Loading...'}}</p>
      <div class="absolute z-10 flex items-center justify-end right-0 w-20 h-full group hover:text-gray-200 transition-colors duration-300" 
        @click="clickNext"
      >
        <IconRight class="hidden group-hover:block" />
      </div>
    </div>
    <FileInfo v-if="showFileInfo" class="" :fileId="Number(fileId)" @close="clickCloseFileInfo"/>
  </div>
    
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { emit, listen } from '@tauri-apps/api/event';
import { useRouter } from 'vue-router';
import { invoke } from '@tauri-apps/api/tauri';
import FileInfo from '@/components/FileInfo.vue';

import IconZoomIn from '@/assets/arrows-pointing-in.svg'; 
import IconZoomOut from '@/assets/arrows-pointing-out.svg'; 
import IconGlassPlus from '@/assets/magnifying-glass-plus.svg'; 
import IconGlassMinus from '@/assets/magnifying-glass-minus.svg'; 
import IconLeft from '@/assets/chevron-left.svg';
import IconRight from '@/assets/chevron-right.svg';

const router = useRouter();

const fileId = ref(null);
const filePath = ref('');         // File path
const imageSrc = ref(null);
const loadError = ref(null);

const showToolbar = ref(true);
const showFileInfo = ref(true);


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
    filePath.value = decodeURIComponent(urlParams.get('filePath'));
    loadImage(filePath.value);

    fileId.value = urlParams.get('fileId');

    // Listen for the 'update-url' event to update the image
    listen('update-img', (event) => {
      filePath.value = decodeURIComponent(event.payload.filePath);
      loadImage(filePath.value);

      fileId.value = event.payload.fileId;
    });
  } catch (error) {
    loadError.value = error;
    imageSrc.value = null;
    console.error('Error loading image:', error);
  }
});

// Emit a message to the main window to go to the previous image
function clickPrev() {
  console.log('clickPrev');
  emit('message-from-image-viewer', { message: 'prev' });
}

function clickNext() {
  console.log('clickNext');
  emit('message-from-image-viewer', { message: 'next' });
}

// Close the file info panel
function clickCloseFileInfo() {
  showFileInfo.value = false;
}


// Load the image from the file path
async function loadImage(filePath) {
  console.log('loadImage:', filePath);
  try {
    const imageBase64 = await invoke('get_file_image', { filePath });
    imageSrc.value = `data:image/jpeg;base64,${imageBase64}`;
  } catch (error) {
    loadError.value = error;
    imageSrc.value = null;
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

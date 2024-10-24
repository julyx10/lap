<template>

  <div class="relative w-screen h-screen flex flex-col border border-gray-800 rounded-lg shadow-lg overflow-hidden">
    <TitleBar v-if="!isFullScreen" titlebar="jc-photo" viewName="ImageViewer"/>

    <!-- Toolbar -->
    <div 
      :class="[
        'absolute left-1/2 z-50 transform -translate-x-1/2 h-10 flex flex-row items-center justify-center space-x-5 t-color-text',
        isFullScreen ? '-translate-y-8 hover:translate-y-0 transition-transform duration-300 ease-in-out' : ''
      ]"
    >
      <IconPrev class="t-icon-size-sm t-icon-hover" @click="clickPrev" />
      <IconNext class="t-icon-size-sm t-icon-hover" @click="clickNext" />
      <IconZoomIn class="t-icon-size-sm t-icon-hover" @click="scale += 0.5" />
      <IconZoomOut class="t-icon-size-sm t-icon-hover" @click="scale -= 0.5" />
      <component :is="imageFit ? IconFitScreen1 : IconFitScreen2" class="t-icon-size-sm t-icon-hover" @click="toggleFitScreen" />

      <IconRotateRight class="t-icon-size-sm t-icon-hover" @click="rotateImage"/>

      <IconUnFavorite v-if="!fileInfo" class="t-icon-size-sm t-icon-disabled"/>
      <IconUnFavorite v-else-if="fileInfo.is_favorite === null || fileInfo.is_favorite === false" class="t-icon-size-sm t-icon-hover" @click="toggleFavorite" />
      <IconFavorite   v-else-if="fileInfo.is_favorite === true" class="t-icon-size-sm t-icon-hover" @click="toggleFavorite" />

      <IconFileInfo :class="['t-icon-size-sm t-icon-hover', showFileInfo ? 't-icon-focus' : '']" @click="clickShowFileInfo" />
      <IconSave class="t-icon-size-sm t-icon-hover" />
      <!-- <IconFullScreen v-if="!isFullScreen" class="t-icon-size-sm t-icon-hover" @click="setFullScreen" /> -->
      <!-- <IconRestoreScreen v-if="isFullScreen" class="t-icon-size-sm t-icon-hover" @click="exitFullScreen" /> -->
      <component :is="isFullScreen ? IconRestoreScreen : IconFullScreen" class="t-icon-size-sm t-icon-hover" @click="toggleFullScreen" />
    </div>

    <div class="flex t-color-text t-color-bg h-screen overflow-hidden">
      <!-- image area -->
      <div class="relative flex-1 flex justify-center items-center overflow-hidden" 
        @wheel="zoomImage" 
      >
        <!-- left  -->
        <div v-if="fileIndex > 0"
          class="absolute left-0 w-20 h-full z-10 flex items-center justify-start cursor-pointer group" 
          @click="clickPrev"
        >
        <div class="m-3 p-2 t-color-bg-light rounded-full hidden group-hover:block ">
            <IconLeft class=" t-icon-hover"/>
          </div>
        </div>

        <div v-if="imageSrc" 
          @load="onImageLoad"
          @mousedown="startDragging" 
          @mouseup="stopDragging"
          @mousemove="dragImage" 
          @mouseleave="stopDragging"
        >
          <img 
            ref="image"
            :src="imageSrc"
            :style="imageStyle"
            :class="[
              '',
              imageFit ? 'object-contain' : 'object-none'
            ]" 
            alt="Image Viewer" 
            draggable="false"
          />
        </div>
        <div v-else>
          {{ loadError ? loadError : $t('image_view_loading') }}
        </div>

        <!-- right -->
        <div v-if="fileIndex < fileCount - 1"
          class="absolute right-0 w-20 h-full z-10 flex items-center justify-end cursor-pointer group" 
          @click="clickNext"
        >
          <div class="m-3 p-2 t-color-bg-light rounded-full hidden group-hover:block ">
            <IconRight class=" t-icon-hover"/>
          </div>
        </div>

      </div>

      <!-- File Info -->
      <transition
        enter-active-class="transition-transform duration-200"
        leave-active-class="transition-transform duration-200"
        enter-from-class="translate-x-full"
        enter-to-class="translate-x-0"
        leave-from-class="translate-x-0"
        leave-to-class="translate-x-full"
      >
        <FileInfo v-if="showFileInfo" :fileInfo="fileInfo" @close="closeFileInfo" />
      </transition>
    </div>

  </div>

</template>


<script setup lang="ts">

import { ref, computed, onMounted, onUnmounted } from 'vue';
import { appWindow } from '@tauri-apps/api/window';
import { emit, listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/tauri';
import TitleBar from '@/components/TitleBar.vue';
import FileInfo from '@/components/FileInfo.vue';

import IconPrev from '@/assets/nav-prev.svg';
import IconNext from '@/assets/nav-next.svg';
import IconZoomIn from '@/assets/zoom-in.svg';
import IconZoomOut from '@/assets/zoom-out.svg';
import IconFitScreen1 from '@/assets/fit-screen1.svg';
import IconFitScreen2 from '@/assets/fit-screen2.svg';
import IconRotateRight from '@/assets/rotate-right.svg';
import IconUnFavorite from '@/assets/heart.svg';
import IconFavorite from '@/assets/heart-solid.svg';
import IconFileInfo from '@/assets/information.svg';
import IconSave from '@/assets/save.svg';
import IconFullScreen from '@/assets/full-screen-max.svg';
import IconRestoreScreen from '@/assets/full-screen-min.svg';
import IconLeft from '@/assets/arrow-left.svg';
import IconRight from '@/assets/arrow-right.svg';
import { c } from 'naive-ui';

const fileId = ref(null);
const filePath = ref('');      // File path
const fileIndex = ref(0);      // Index of the current file
const fileCount = ref(0);      // Total number of files
const fileInfo = ref(null);
const showFileInfo = ref(false); // Show the file info panel

const image = ref(null); // Image reference
const imageSrc = ref(null);
const loadError = ref(null);
const imageFit = ref(true); // true: fit screen; false: original size

const isFullScreen = ref(false); // Track if the window is full screen
const isMaximized  = ref(false); // Track if the window is maximized

// Image rotation angle
const rotation  = ref(0); 
// Zoom scaling 
const scale = ref(1); // Default zoom scale
const scaledWidth = ref(0); // Scaled width of the image
const scaledHeight = ref(0); // Scaled height of the image
// Dragging state, and position
const isDragging = ref(false); // Track if the image is being dragged
const startX = ref(0); // Store initial X position when dragging starts
const startY = ref(0); // Store initial Y position when dragging starts
const translateX = ref(0); // X axis translation (dragging)
const translateY = ref(0); // Y axis translation (dragging)
const lastTranslateX = ref(0); // Last stored X position after drag ends
const lastTranslateY = ref(0); // Last stored Y position after drag ends

// Computed style for the image, combining zoom and translation
const imageStyle = computed(() => ({
  transform: `rotate(${rotation.value}deg) scale(${scale.value}) translate(${translateX.value}px, ${translateY.value}px)`,
  transition: isDragging.value ? 'none' : 'transform 0.2s ease-in-out',
}));

onMounted(async() => {
  window.addEventListener('keydown', handleKeyDown);
  isFullScreen.value = await appWindow.isMaximized();

  try {
    const urlParams = new URLSearchParams(window.location.search);

    // Load the image from the file path
    filePath.value = decodeURIComponent(urlParams.get('filePath'));
    await loadImage(filePath.value);
    
    fileId.value = urlParams.get('fileId');
    fileIndex.value = Number(urlParams.get('fileIndex'));
    fileCount.value = Number(urlParams.get('fileCount'));
    await loadFileInfo(fileId.value);
    
    // Listen for the 'update-url' event to update the image
    listen('update-img', (event) => {
      filePath.value = decodeURIComponent(event.payload.filePath);
      loadImage(filePath.value);

      fileId.value = event.payload.fileId;
      fileIndex.value = Number(event.payload.fileIndex);
      fileCount.value = Number(event.payload.fileCount);
      loadFileInfo(fileId.value);

      // Reset the image zoom and rotation
      resetScale();
      rotation.value = 0;
    });

    if (image.value.complete) {
      onImageLoad();  // If the image is already loaded, initialize it
    }

  } catch (error) {
    loadError.value = error;
    imageSrc.value = null;
    console.error('Error loading image:', error);
  }
});

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeyDown);
});


// Load the image from the file path
async function loadImage(filePath) {
  try {
    const imageBase64 = await invoke('get_file_image', { filePath });
    imageSrc.value = `data:image/jpeg;base64,${imageBase64}`;
    console.log('loadImage:', filePath);
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
    console.log('loadFileInfo: ---', fileInfo.value);
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
  scale.value = Math.min(Math.max(0.1, scale.value + delta), 10); // Limit zoom between 0.5x and 5x
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
  // if (isDragging.value) {
  //   lastTranslateX.value = translateX.value;
  //   lastTranslateY.value = translateY.value;
  // }
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


function handleKeyDown(event) {
  const navigationKeys = ['ArrowDown', 'ArrowUp', 'ArrowLeft', 'ArrowRight', 'Enter', 'Escape'];
  
  // Disable default behavior for certain keys
  if (navigationKeys.includes(event.key)) {
    event.preventDefault();
  }

  switch (event.key) {
    case 'ArrowDown':
      break;
    case 'ArrowRight':
      clickNext();
      break;
    case 'ArrowUp':
      break;
    case 'ArrowLeft':
      clickPrev();
      break;
    case 'Enter':
      clickShowFileInfo();
      break;
    case 'Escape':
      if (showFileInfo.value) {
        closeFileInfo();
      } else if (isFullScreen.value) {
        exitFullScreen(); // Exit full screen
      } else {
        appWindow.close(); // Close the window
      }
      break;
  }
}


// Function to be called when the image loads
const onImageLoad = () => {
  const img = image.value;
  scaledWidth.value = img.naturalWidth;
  scaledHeight.value = img.naturalHeight;
};

// Function to fit the image to the screen
const toggleFitScreen = () => {
  imageFit.value = !imageFit.value;
  // const windowWidth = window.innerWidth;
  // const windowHeight = window.innerHeight;
  
  // const img = image.value;
  
  // // Only proceed if the image has been loaded
  // if (img) {
  //   const originalWidth = img.naturalWidth;
  //   const originalHeight = img.naturalHeight;

  //   // Calculate the scale to fit the image within the screen
  //   const widthScale = windowWidth / originalWidth;
  //   const heightScale = windowHeight / originalHeight;

  //   // Use the smaller scale to fit the image
  //   scale.value = Math.min(widthScale, heightScale);
  //   scaledWidth.value = originalWidth * scale.value;
  //   scaledHeight.value = originalHeight * scale.value;
  // }
};

// Function to reset the image to 1:1 scale
const resetScale = () => {
  scale.value = 1;
  const img = image.value;

  // Only proceed if the image has been loaded
  if (img) {
    scaledWidth.value = img.naturalWidth;
    scaledHeight.value = img.naturalHeight;
  }
};

// toggle favorite status
const toggleFavorite = async() => {
  if (fileInfo.value.is_favorite === null) {
    fileInfo.value.is_favorite = true;
  } else {
    fileInfo.value.is_favorite = !fileInfo.value.is_favorite;
  }

  // set db status
  await invoke('set_file_favorite', { 
    fileId: Number(fileId.value), 
    isFavorite: fileInfo.value.is_favorite 
  })
}

// Function to maximize the window and setup full screen
const toggleFullScreen = async () => {
  if (!isFullScreen.value) {  // enter full screen
    isMaximized.value = await appWindow.isMaximized(); // Check if the window is maximized

    await appWindow.setFullscreen(true);
    isFullScreen.value = true;

    if (!isMaximized.value) {
      await appWindow.maximize();
    }
  } else {  // exit full screen
    await appWindow.setFullscreen(false);
    isFullScreen.value = false;

    if (isMaximized.value) {
      await appWindow.unmaximize();
    }
  }
}

// Method to rotate the image by 90 degrees
const rotateImage = () => {
  console.log('rotateImage');
  rotation.value += 90;
}


// Emit a message to the main window to go to the previous image
function clickPrev() {
  console.log('clickPrev');
  emit('message-from-image-viewer', { message: 'prev' });
}

function clickNext() {
  console.log('clickNext');
  emit('message-from-image-viewer', { message: 'next' });
}


function clickShowFileInfo() {
  showFileInfo.value = !showFileInfo.value;
}


// Close the file info panel from the child component
function closeFileInfo() {
  showFileInfo.value = false;
}


</script>

<style scoped>
/* Disable text selection while dragging */
* {
  user-select: none;
}
</style>

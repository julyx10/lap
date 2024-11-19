<template>

  <div 
    :class="[
      'relative w-screen h-screen flex flex-col border t-color-border overflow-hidden',
      isFullScreen ? 'fixed top-0 left-0 z-50' : 'rounded-lg shadow-lg'
    ]">
    <TitleBar v-if="!isFullScreen" titlebar="jc-photo" viewName="ImageViewer"/>

    <!-- Toolbar -->
    <div 
      :class="[
        'absolute left-1/2 z-50 transform -translate-x-1/2 h-10 flex flex-row items-center justify-center space-x-5 t-color-text',
        isFullScreen ? '-translate-y-8 hover:translate-y-0 transition-transform duration-300 ease-in-out' : ''
      ]"
    >
      <IconPrev 
        :class="[
          't-icon-size-sm',
          fileIndex > 0 ? 't-icon-hover' : 't-icon-disabled'
        ]" 
        @click="clickPrev" 
      />
      <IconNext 
        :class="[
          't-icon-size-sm',
          fileIndex < fileCount -1 ? 't-icon-hover' : 't-icon-disabled'
        ]" 
        @click="clickNext" 
      />
      <IconZoomIn  class="t-icon-size-sm t-icon-hover" @click="scale += 0.5" />
      <IconZoomOut class="t-icon-size-sm t-icon-hover" @click="scale -= 0.5" />
      <component :is="isZoomFit ? IconZoomFit : IconZoomOriginal" class="t-icon-size-sm t-icon-hover" @click="toggleZoomFit" />
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
      <!-- zoom area -->
      <div ref="zoomRef" class="relative flex-1 flex justify-center items-center overflow-hidden" 
        @wheel="zoomImage" 
      >
        <!-- debug -->
        <div v-if="true" class="absolute top-0 left-0 m-2 p-2 text-sm text-sky-300 bg-gray-800 opacity-50 rounded-lg z-50">
          <p>scale: {{ scale ? scale.toFixed(2) : '' }}</p>
          <p>startX, startY: ({{ startX.toFixed(2) }}, {{ startY.toFixed(2) }})</p>
          <p>translateX, translateY: ({{ translateX.toFixed(2) }}, {{ translateY.toFixed(2) }})</p>
          <p>lastTranslateX, lastTranslateY: ({{ lastTranslateX.toFixed(2) }}, {{ lastTranslateY.toFixed(2) }})</p>
        </div>

        <!-- prev   -->
        <div v-if="fileIndex > 0"
          class="absolute left-0 w-20 h-full z-10 flex items-center justify-start cursor-pointer group" 
          @click="clickPrev"
        >
          <div class="m-3 p-2 t-color-bg-light rounded-full hidden group-hover:block ">
            <IconLeft class="t-icon-size t-icon-hover"/>
          </div>
        </div>

        <!-- image -->
        <!-- <img v-if="imageSrc" 
          ref="imageRef"
          :src="imageSrc"
          :style="imageStyle"
          alt="Image Viewer" 
          draggable="false"
          @mousedown="startDragging" 
          @mouseup="stopDragging"
          @mousemove="dragImage" 
          @mouseleave="stopDragging"
        /> -->
        <Image v-if="imageSrc" :src="imageSrc" />
        <p v-else>
          {{ loadError ? $t('image_view_failed') + ': ' + filePath : $t('image_view_loading') }}
        </p>

        <!-- next -->
        <div v-if="fileIndex < fileCount - 1"
          class="absolute right-0 w-20 h-full z-10 flex items-center justify-end cursor-pointer group" 
          @click="clickNext"
        >
          <div class="m-3 p-2 t-color-bg-light rounded-full hidden group-hover:block ">
            <IconRight class="t-icon-size t-icon-hover"/>
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

import { ref, watch, computed, onMounted, onUnmounted } from 'vue';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { emit, listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';
import { useI18n } from 'vue-i18n';
import { useConfigStore } from '@/stores/configStore';

import TitleBar from '@/components/TitleBar.vue';
import Image from '@/components/Image.vue';
import FileInfo from '@/components/FileInfo.vue';

import IconPrev from '@/assets/nav-prev.svg';
import IconNext from '@/assets/nav-next.svg';
import IconZoomIn from '@/assets/zoom-in.svg';
import IconZoomOut from '@/assets/zoom-out.svg';
import IconZoomFit from '@/assets/fit-screen1.svg';
import IconZoomOriginal from '@/assets/fit-screen2.svg';
import IconRotateRight from '@/assets/rotate-right.svg';
import IconUnFavorite from '@/assets/heart.svg';
import IconFavorite from '@/assets/heart-solid.svg';
import IconFileInfo from '@/assets/information.svg';
import IconSave from '@/assets/save.svg';
import IconFullScreen from '@/assets/full-screen-max.svg';
import IconRestoreScreen from '@/assets/full-screen-min.svg';
import IconLeft from '@/assets/arrow-left.svg';
import IconRight from '@/assets/arrow-right.svg';

/// i18n
const { locale, messages } = useI18n();
// const localeMsg = computed(() => messages.value[locale.value]);

// config store
const config = useConfigStore();

const appWindow = getCurrentWebviewWindow()

const fileId = ref(null);
const filePath = ref('');      // File path
const fileIndex = ref(0);      // Index of the current file
const fileCount = ref(0);      // Total number of files
const fileInfo = ref(null);
const showFileInfo = ref(false); // Show the file info panel

const zoomRef = ref(null);    // Zoom div reference
const isZoomFit = ref(null);  // true: zoom to fit window; false: original size

const imageRef = ref(null); // Image reference
const imageSrc = ref(null);
const loadError = ref(false); // Track if there was an error loading the image

const isFullScreen = ref(false); // Track if the window is full screen
const isMaximized  = ref(false); // Track if the window is maximized

// Image rotation angle
const rotation  = ref(0); 
// Zoom scaling 
const scale = ref(null);         // Default zoom scale
// const scaledWidth = ref(0);   // Scaled width of the image
// const scaledHeight = ref(0);  // Scaled height of the image
// const imageWidth = ref(0);    // Image width
// const imageHeight = ref(0);   // Image height

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


// Listen for the 'update-url' event to update the image
listen('update-img', async (event) => {
  filePath.value = decodeURIComponent(event.payload.filePath);
  await loadImage(filePath.value);

  fileId.value = event.payload.fileId;
  fileIndex.value = Number(event.payload.fileIndex);
  fileCount.value = Number(event.payload.fileCount);
  await loadFileInfo(fileId.value);

  // Reset the image zoom and rotation
  // resetScale();
  // rotation.value = 0;
});

listen('message-from-home', (event) => {
  const { message } = event.payload;
  console.log('message-from-home:', message);
  switch (message) {
    case 'closed':
      appWindow.close(); // Close the window
      break;
    default:
      break;
  }
});

/// watch language
watch(() => config.language, (newLanguage) => {
    locale.value = newLanguage; // update locale based on config.language
});

/// watch zoom fit
watch (() => isZoomFit.value, (newVal) => {
  console.log('isZoomFit:', newVal);
  if (newVal) {
    zoomFit();
  } else {
    resetScale();
  }
});

onMounted(async() => {
  window.addEventListener('keydown', handleKeyDown);

  const urlParams = new URLSearchParams(window.location.search);
  // Load the image from the file path
  filePath.value = decodeURIComponent(urlParams.get('filePath'));
  await loadImage(filePath.value);
  
  // set default zoom: fit scrren
  // isZoomFit.value = true;
  // scale.value = 0.5;
  
  fileId.value = urlParams.get('fileId');
  fileIndex.value = Number(urlParams.get('fileIndex'));
  fileCount.value = Number(urlParams.get('fileCount'));
  await loadFileInfo(fileId.value);
  
  zoomFit()
  isFullScreen.value = await appWindow.isMaximized();
});

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeyDown);
});

// Load the image from the file path
async function loadImage(filePath) {
  try {
    loadError.value = false;

    const imageBase64 = await invoke('get_file_image', { filePath });
    imageSrc.value = `data:image/jpeg;base64,${imageBase64}`;
    console.log('loadImage:', filePath);
  } catch (error) {
    imageSrc.value = null;
    loadError.value = true;
    console.error('loadImage:', error);
  }
}

// Load the file info from the file ID
async function loadFileInfo(fileId) {
  try {
    fileInfo.value = await invoke('get_file_info', { fileId: parseInt(fileId, 10) });
    console.log('loadFileInfo:', fileInfo.value);
  } catch (error) {
    console.error('loadFileInfo:', error);
  }
}

// Function to handle zooming with the mouse wheel
function zoomImage(event) {
  event.preventDefault();
  const zoomSpeed = 1.2; // Change this value to adjust zoom speed
  const delta = event.deltaY < 0 ? zoomSpeed : (1 / zoomSpeed);
  scale.value = Math.min(Math.max(0.1, scale.value * delta), 10); // Limit zoom between 0.5x and 5x

  lastTranslateX.value = lastTranslateX.value * scale.value;
  lastTranslateY.value = lastTranslateY.value * scale.value;
}

// Start dragging when the mouse button is pressed
function startDragging(event) {
  console.log('startDragging:', event);
  isDragging.value = true;
  startX.value = (event.clientX - lastTranslateX.value) / scale.value;
  startY.value = (event.clientY - lastTranslateY.value) / scale.value;
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

// Stop dragging when the mouse button is released
function stopDragging(event) {
  console.log('stopDragging', event);

  lastTranslateX.value = translateX.value;
  lastTranslateY.value = translateY.value;
  isDragging.value = false;
}

const zoomFit = () => {
  // Only proceed if the image has been loaded
  if (imageRef.value) {
    // const imgPosition = imageRef.value.getBoundingClientRect();
    const zommPosition = zoomRef.value.getBoundingClientRect();

    // console.log('zoomFit:', imgPosition, zommPosition);

    // Calculate the scale factor to fit the image to the container
    const scaleWidth = zommPosition.width / fileInfo.value.width;
    const scaleHeight = zommPosition.height / fileInfo.value.height;
    scale.value = Math.min(scaleWidth, scaleHeight);

    // Center the image in the container
    // translateX.value = (zommPosition.width - imgPosition.width * scale.value) / 2;
    // translateY.value = (zommPosition.height - imgPosition.height * scale.value) / 2;

  }
};

// Function to reset the image to 1:1 scale
const resetScale = () => {
  scale.value = 1;

  // reset the image position
  // translateX.value = 0; // Reset the start X position
  // translateY.value = 0; // Reset the start Y position
  // lastTranslateX.value = 0; // Reset the last X position
  // lastTranslateY.value = 0; // Reset the last Y position
};

// zoom to fit the image to the screen
const toggleZoomFit = () => {
  isZoomFit.value = !isZoomFit.value;
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
    isMaximized.value = appWindow.isMaximized(); // Check if the window is maximized

    await appWindow.setFullscreen(true);
    await appWindow.setResizable(false); // Disable window resizing
    // await appWindow.setSkipTaskbar(true); // Hide the window from the taskbar
    // await appWindow.setAlwaysOnTop(true); // Keep the window on top of other windows
    isFullScreen.value = true;

    // if (!isMaximized.value) {
    //   await appWindow.maximize();
    //   await appWindow.setSkipTaskbar(true); // Hide the window from the taskbar
    // }
  } else {  // exit full screen
    await appWindow.setFullscreen(false);
    await appWindow.setResizable(true); // Disable window resizing

    isFullScreen.value = false;

    if (isMaximized.value) {
      await appWindow.unmaximize();
    }
  }
}

// Handle keyboard shortcuts
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
      } else {
        appWindow.close(); // Close the window
      }
      break;
  }
}

// Method to rotate the image by 90 degrees
const rotateImage = () => {
  rotation.value += 90;
}

// Emit a message to the main window to go to the previous image
function clickPrev() {
  emit('message-from-image-viewer', { message: 'prev' });
}

function clickNext() {
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

<template>

  <div 
    :class="[
      'relative w-screen h-screen flex flex-col border t-color-border overflow-hidden',
      isFullScreen ? 'fixed top-0 left-0 z-50' : 'rounded-lg shadow-lg'
    ]">
    <TitleBar v-if="!isFullScreen" titlebar="jc-photo" viewName="ImageViewer"/>

    <!-- Toolbar -->
    <div id="responsiveDiv"
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
      <IconZoomIn  class="t-icon-size-sm t-icon-hover" @click="clickZoomIn" />
      <IconZoomOut class="t-icon-size-sm t-icon-hover" @click="clickZoomOut" />
      <component :is="isZoomFit ? IconZoomFit : IconZoomOriginal" class="t-icon-size-sm t-icon-hover" @click="toggleZoomFit" />
      <IconRotateRight class="t-icon-size-sm t-icon-hover" @click="clickRotate"/>
      <IconUnFavorite v-if="!fileInfo" class="t-icon-size-sm t-icon-disabled"/>
      <IconUnFavorite v-else-if="fileInfo.is_favorite === null || fileInfo.is_favorite === false" class="t-icon-size-sm t-icon-hover" @click="toggleFavorite" />
      <IconFavorite   v-else-if="fileInfo.is_favorite === true" class="t-icon-size-sm t-icon-hover" @click="toggleFavorite" />
      <IconFileInfo :class="['t-icon-size-sm t-icon-hover', showFileInfo ? 't-icon-focus' : '']" @click="clickShowFileInfo" />
      <IconSave class="t-icon-size-sm t-icon-hover" @click="clickSave"/>
      <!-- <IconFullScreen v-if="!isFullScreen" class="t-icon-size-sm t-icon-hover" @click="setFullScreen" /> -->
      <!-- <IconRestoreScreen v-if="isFullScreen" class="t-icon-size-sm t-icon-hover" @click="exitFullScreen" /> -->
      <component :is="isFullScreen ? IconRestoreScreen : IconFullScreen" class="t-icon-size-sm t-icon-hover" @click="toggleFullScreen" />
    </div>

    <div class="flex t-color-text t-color-bg h-screen overflow-hidden">

      <!-- image container -->
      <div class="relative flex-1 flex justify-center items-center overflow-hidden">
        
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
        <Image v-if="imageSrc" ref="imageRef" 
          :src="imageSrc" :width="fileInfo?.width" :height="fileInfo?.height"
        />
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

      </div> <!-- image container -->

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
      </transition> <!-- File Info -->

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

const isZoomFit = ref(null);  // true: zoom to fit container; false: original size(scale = 1)

const imageRef = ref(null); // Image reference
const imageSrc = ref(null);
const loadError = ref(false); // Track if there was an error loading the image

const isFullScreen = ref(false); // Track if the window is full screen
// const isMaximized  = ref(false); // Track if the window is maximized

/// watch language
watch(() => config.language, (newLanguage) => {
    locale.value = newLanguage; // update locale based on config.language
});

// Listen for the 'update-url' event to update the image
listen('update-img', async (event) => {
  filePath.value = decodeURIComponent(event.payload.filePath);
  await loadImage(filePath.value);

  fileId.value = event.payload.fileId;
  fileIndex.value = Number(event.payload.fileIndex);
  fileCount.value = Number(event.payload.fileCount);
  await loadFileInfo(fileId.value);
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

onMounted(async() => {
  window.addEventListener('keydown', handleKeyDown);
  // isFullScreen.value = await appWindow.isMaximized();

  const urlParams = new URLSearchParams(window.location.search);
  // Load the image from the file path
  filePath.value = decodeURIComponent(urlParams.get('filePath'));
  await loadImage(filePath.value);
  
  fileId.value = urlParams.get('fileId');
  fileIndex.value = Number(urlParams.get('fileIndex'));
  fileCount.value = Number(urlParams.get('fileCount'));
  await loadFileInfo(fileId.value);
  
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

// Emit a message to the main window to go to the previous image
function clickPrev() {
  emit('message-from-image-viewer', { message: 'prev' });
}

function clickNext() {
  emit('message-from-image-viewer', { message: 'next' });
}

const clickZoomIn = () => {
  if(imageRef.value) {
    imageRef.value.zoomIn();
  }
};

const clickZoomOut = () => {
  if(imageRef.value) {
    imageRef.value.zoomOut();
  }
};

const toggleZoomFit = () => {
  if(imageRef.value) {
    isZoomFit.value = !isZoomFit.value;
    isZoomFit.value ? imageRef.value.zoomFit() : imageRef.value.zoomReset();
  }
};

const clickRotate = () => {
  if(imageRef.value) {
    imageRef.value.rotateRight();
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
    // isMaximized.value = appWindow.isMaximized(); // Check if the window is maximized

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

    // if (isMaximized.value) {
    //   await appWindow.unmaximize();
    // }
  }
}

function clickShowFileInfo() {
  showFileInfo.value = !showFileInfo.value;
}

// Close the file info panel from the child component
function closeFileInfo() {
  showFileInfo.value = false;
}

function clickSave() {
  emit('message-from-image-viewer', { message: 'save' });
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

</script>

<style scoped>
/* Disable text selection while dragging */
* {
  user-select: none;
}

@media (max-width: 700px) {
  #responsiveDiv {
    visibility: hidden;
  }
}
@media (min-width: 700px) {
  #responsiveDiv {
    visibility: visible;
  }
}

</style>

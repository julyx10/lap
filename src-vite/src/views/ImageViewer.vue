<template>

  <div 
    :class="[
      'relative w-screen h-screen flex flex-col border t-color-border overflow-hidden',
      config.isFullScreen ? 'fixed top-0 left-0 z-50' : 'rounded-lg shadow-lg'
    ]">
    <TitleBar v-if="!config.isFullScreen"
      :titlebar="`jc-photo - ${fileIndex + 1}/${fileCount}`"
      viewName="ImageViewer"
    />

    <!-- Toolbar -->
    <div id="responsiveDiv"
      :class="[
        'absolute left-1/2 z-50 transform -translate-x-1/2 h-10 flex flex-row items-center justify-center space-x-5 t-color-text',
        config.isFullScreen ? '-translate-y-8 hover:translate-y-0 transition-transform duration-300 ease-in-out' : ''
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
      <IconZoomIn
        :class="[
          't-icon-size-sm',
          fileIndex >= 0 && imageScale < imageMaxScale ? 't-icon-hover' : 't-icon-disabled'
        ]"
        @click="clickZoomIn" 
      />
      <IconZoomOut
        :class="[
          't-icon-size-sm',
          fileIndex >= 0 && imageScale > imageMinScale ? 't-icon-hover' : 't-icon-disabled'
        ]"
        @click="clickZoomOut" 
      />
      <component 
        :is="config.isZoomFit ? IconZoomFit : IconZoomOriginal" 
        :class="[
          't-icon-size-sm',
          fileIndex >= 0 ? 't-icon-hover' : 't-icon-disabled'
        ]" 
        @click="toggleZoomFit" 
      />
      <IconRotateRight 
        :class="[
          't-icon-size-sm',
          fileIndex >= 0 ? 't-icon-hover' : 't-icon-disabled',
          (fileInfo?.rotate ?? 0) % 360 !== 0 ? 't-icon-focus' : ''
        ]" 
        :style="{ transform: `rotate(${(fileInfo?.rotate ?? 0)}deg)`, transition: 'transform 0.3s ease-in-out' }" 
        @click="clickRotate"
      />
      <IconUnFavorite v-if="!fileInfo" class="t-icon-size-sm t-icon-disabled"/>
      <IconUnFavorite v-else-if="fileInfo.is_favorite === null || fileInfo.is_favorite === false" class="t-icon-size-sm t-icon-hover" @click="toggleFavorite" />
      <IconFavorite   v-else-if="fileInfo.is_favorite === true" class="t-icon-size-sm t-icon-hover" @click="toggleFavorite" />
      <IconFileInfo :class="['t-icon-size-sm t-icon-hover', showFileInfo ? 't-icon-focus' : '']" @click="clickShowFileInfo" />
      <IconSave 
        :class="[
          't-icon-size-sm',
          fileIndex >= 0 ? 't-icon-hover' : 't-icon-disabled'
        ]"
        @click="clickSave"
      />
      <IconDelete
        :class="[
          't-icon-size-sm',
          fileIndex >= 0 ? 't-icon-hover' : 't-icon-disabled'
        ]"
        @click="clickDelete"
      />
      <!-- <IconFullScreen v-if="!config.isFullScreen" class="t-icon-size-sm t-icon-hover" @click="setFullScreen" /> -->
      <!-- <IconRestoreScreen v-if="config.isFullScreen" class="t-icon-size-sm t-icon-hover" @click="exitFullScreen" /> -->
      <component :is="config.isFullScreen ? IconRestoreScreen : IconFullScreen" class="t-icon-size-sm t-icon-hover" @click="toggleFullScreen" />
    </div>

    <div class="flex t-color-text t-color-bg h-screen overflow-hidden">

      <!-- image container -->
      <div ref="viewerContainer" class="relative flex-1 flex justify-center items-center overflow-hidden">
        
        <!-- show zoom scale -->
        <transition name="fade">
          <div v-if="isScaleChanged" class="absolute left-1/2 top-5 px-2 py-1 z-10 t-color-bg opacity-50 rounded-lg">
            <slot>{{(imageScale * 100).toFixed(0)}} %</slot>
          </div>
        </transition>

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
        <template v-if="fileIndex >= 0">
          <Image v-if="imageSrc" 
            ref="imageRef" 
            :src="imageSrc" 
            :rotate="fileInfo?.rotate ?? 0" 
            :isZoomFit="config.isZoomFit"
          />
          <p v-else>
            {{ loadError ? $t('image_view_failed') + ': ' + filePath : $t('image_view_loading') }}
          </p>
        </template>

        <!-- no image selected -->
        <p v-else>
          {{ $t('image_view_no_image') }}
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
        <FileInfo v-if="showFileInfo" 
          :fileInfo="fileInfo" 
          :fileIndex="fileIndex" 
          :fileCount="fileCount" 
          @close="closeFileInfo" 
        />
      </transition> <!-- File Info -->

    </div>

  </div>

</template>


<script setup lang="ts">

import { ref, watch, onMounted, onUnmounted } from 'vue';
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
import IconDelete from '@/assets/trash.svg';
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

// input parameters
const fileId = ref(0);       // File ID
const fileIndex = ref(0);       // Index of the current file
const fileCount = ref(0);       // Total number of files
const filePath = ref('');       // File path
const nextFilePath = ref('');   // Next file path to preload

const fileInfo = ref(null);
const showFileInfo = ref(false); // Show the file info panel

const imageRef = ref(null);     // Image reference
const imageSrc = ref(null);
const imageCache = new Map();   // Cache images to prevent reloading
const loadError = ref(false);   // Track if there was an error loading the image

const imageScale = ref(1);          // Image scale
const imageMinScale = ref(0);       // Minimum image scale
const imageMaxScale = ref(10);      // Maximum image scale
const isScaleChanged = ref(false);  // Scale changed state

onMounted(async() => {
  window.addEventListener('keydown', handleKeyDown);
  // isFullScreen.value = await appWindow.isMaximized();

  const urlParams = new URLSearchParams(window.location.search);
  
  fileId.value    = Number(urlParams.get('fileId'));
  fileIndex.value = Number(urlParams.get('fileIndex'));
  fileCount.value = Number(urlParams.get('fileCount'));
  filePath.value     = decodeURIComponent(urlParams.get('filePath'));
  nextFilePath.value = decodeURIComponent(urlParams.get('nextFilePath'));
});

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeyDown);
});

// Listen for the 'update-url' event to update the image
listen('update-img', async (event) => {
  fileId.value    = Number(event.payload.fileId);
  fileIndex.value = Number(event.payload.fileIndex);
  fileCount.value = Number(event.payload.fileCount);
  filePath.value     = decodeURIComponent(event.payload.filePath);
  nextFilePath.value = decodeURIComponent(event.payload.nextFilePath);
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

listen('message-from-image', (event) => {
  const { message } = event.payload;
  console.log('ImageViewer.vue: message-from-image:', message);
  switch (message) {
    case 'scale':
      imageScale.value = event.payload.scale;
      imageMinScale.value = event.payload.minScale;
      imageMaxScale.value = event.payload.maxScale;
      break;
    case 'rotate':
      fileInfo.value.rotate = event.payload.rotate;
      saveRotate(fileId.value, fileInfo.value.rotate);
      break;
    default:
      break;
  }
});

// watch language
watch(() => config.language, (newLanguage) => {
    locale.value = newLanguage; // update locale based on config.language
});

// watch full screen
watch(() => config.isFullScreen, async (newFullScreen) => {
  await appWindow.setFullscreen(newFullScreen);
  await appWindow.setResizable(!newFullScreen);
}, { immediate: true }); 

// watch file changed
watch(() => fileId.value, async () => {
  await loadImage(filePath.value);    // set imageSrc
  await loadFileInfo(fileId.value);   // set fileInfo
  preLoadImage(nextFilePath.value);   // Preload the next image in the background
});

// watch scale
watch(() => imageScale.value, () => {
  isScaleChanged.value = true;
  
  setTimeout(() => {
    isScaleChanged.value = false;
  }, 1000);
});

// Load the image from the file path
async function loadImage(filePath) {
  try {
    loadError.value = false;

    // Check if the image is already cached
    if (imageCache.has(filePath)) {
      imageSrc.value = imageCache.get(filePath);
      console.log('loadImage - cache get:', filePath);
    } else {
      const imageBase64 = await invoke('get_file_image', { filePath });
      imageSrc.value = `data:image/jpeg;base64,${imageBase64}`;
      imageCache.set(filePath, imageSrc.value);
      console.log('loadImage - cache set:', filePath);
    }
  } catch (error) {
    imageSrc.value = null;
    loadError.value = true;
    console.error('loadImage:', error);
  }
}

// Preload the image from the file path
async function preLoadImage(filePath) {
  try {
    if (filePath.length > 0 && !imageCache.has(filePath)) {
      const imageBase64 = await invoke('get_file_image', { filePath });
      const imageSrc = `data:image/jpeg;base64,${imageBase64}`;
      imageCache.set(filePath, imageSrc);
      console.log('preLoadImage - cache set:', filePath);
    }
  } catch (error) {
    console.error('preLoadImage:', error);
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
  if(fileIndex.value < 0) return;

  if(imageRef.value) {
    imageRef.value.zoomIn();
  }
};

const clickZoomOut = () => {
  if(fileIndex.value < 0) return;

  if(imageRef.value) {
    imageRef.value.zoomOut();
  }
};

const toggleZoomFit = () => {
  if(fileIndex.value < 0) return;

  config.isZoomFit =!config.isZoomFit;
};

const clickRotate = () => {
  if(fileIndex.value < 0) return;

  if(imageRef.value) {
    imageRef.value.rotateRight();
  }
};

const saveRotate = async(fileId, fileRotate) => {
  try {
    await invoke('set_file_rotate', { fileId: fileId, rotate: fileRotate % 360 });
  } catch (error) {
    console.error('saveRotate:', error);
  }
}

// toggle favorite status
const toggleFavorite = async() => {
  if(fileIndex.value < 0) return;

  fileInfo.value.is_favorite = fileInfo.value.is_favorite === null ? true : !fileInfo.value.is_favorite;

  // set db status
  try {
    await invoke('set_file_favorite', { fileId: fileId.value, isFavorite: fileInfo.value.is_favorite });
  } catch (error) {
    console.error('toggleFavorite:', error);
  }
}

// Function to maximize the window and setup full screen
const toggleFullScreen = () => {
  config.isFullScreen = !config.isFullScreen;
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

const clickDelete = async() => {
  // get current timestamp
  fileInfo.value.deleted_at = fileInfo.value.deleted_at || fileInfo.value.deleted_at !== 0 ? 0 : Math.floor(Date.now() / 1000);

  try {
    await invoke('set_file_delete', { fileId: fileId.value, deletedAt: fileInfo.value.deleted_at });
  } catch (error) {
    console.error('clickDelete:', error);
  }
  emit('message-from-image-viewer', { message:'delete' });
}

// Handle keyboard shortcuts
function handleKeyDown(event) {
  const navigationKeys = ['ArrowDown', 'ArrowUp', 'ArrowLeft', 'ArrowRight', 'Enter', 'Escape', 'Space'];
  
  // Disable default behavior for certain keys
  if (navigationKeys.includes(event.key)) {
    event.preventDefault();
  }

  switch (event.key) {
    case 'ArrowUp':
      clickZoomIn();
      break;
    case 'ArrowLeft':
      clickPrev();
      break;
    case 'ArrowDown':
      clickZoomOut();
      break;
    case 'ArrowRight':
      clickNext();
      break;
    case 'f':
      toggleFullScreen();
      break;
    case 's':
      clickSave();
      break;
    case 'r':
      clickRotate();
      break;
    case 'i':
    case 'Enter':
      clickShowFileInfo();
      break;
    case ' ':
      toggleZoomFit();
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

/* fade transition */
.fade-enter-active, .fade-leave-active {
  transition: opacity 0.5s ease; /* Adjust duration and easing as needed */
}
.fade-enter-from, .fade-leave-to {
  opacity: 0; /* Initial and final opacity for fading in and out */
}
.fade-enter-to, .fade-leave-from {
  opacity: 0.5; /* Final opacity when fully visible */
}
</style>

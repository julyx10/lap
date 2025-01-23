<template>

  <div 
    :class="[
      'relative w-screen h-screen flex flex-col overflow-hidden',
      config.isFullScreen ? 'fixed top-0 left-0 z-50' : 'border t-color-border rounded-lg shadow-lg'
    ]"
  >
    <!-- title bar -->
    <TitleBar v-if="!config.isFullScreen"
      :titlebar="`jc-photo ${localeMsg.image_view_title} - ${fileIndex + 1}/${fileCount}`"
      viewName="ImageViewer"
    />

    <!-- Toolbar -->
    <div id="responsiveDiv"
      :class="[
        'absolute px-2 left-1/2 rounded-lg t-color-bg z-50 transform -translate-x-1/2 h-10 flex flex-row items-center justify-center space-x-5 t-color-text',
        config.isFullScreen && !config.isPinned ? '-translate-y-8 opacity-0 hover:translate-y-0 hover:opacity-50 transition-transform duration-300 ease-in-out' : '',
        config.isFullScreen && config.isPinned ? 'opacity-80 transition-transform duration-300 ease-in-out' : ''
      ]"
    >
      <IconPrev 
        :class="[
          't-icon-size',  
          fileIndex > 0 ? 't-icon-hover' : 't-icon-disabled'
        ]" 
        @click="clickPrev" 
      />
      <IconNext 
        :class="[
          't-icon-size',
          fileIndex < fileCount -1 ? 't-icon-hover' : 't-icon-disabled'
        ]" 
        @click="clickNext" 
      />
      <component 
        :is="autoPlay ? IconPause : IconPlay" 
        :class="[
          't-icon-size',
          fileIndex >= 0 ? 't-icon-hover' : 't-icon-disabled'
        ]" 
        @click="autoPlay = !autoPlay" 
      />  
      <IconZoomIn
        :class="[
          't-icon-size',
          fileIndex >= 0 && imageScale < imageMaxScale ? 't-icon-hover' : 't-icon-disabled'
        ]"
        @click="clickZoomIn" 
      />
      <IconZoomOut
        :class="[
          't-icon-size',
          fileIndex >= 0 && imageScale > imageMinScale ? 't-icon-hover' : 't-icon-disabled'
        ]"
        @click="clickZoomOut" 
      />
      <component 
        :is="config.isZoomFit ? IconZoomFit : IconZoomOriginal" 
        :class="[
          't-icon-size',
          fileIndex >= 0 ? 't-icon-hover' : 't-icon-disabled'
        ]" 
        @click="toggleZoomFit" 
      />
      <IconUnFavorite v-if="!fileInfo" class="t-icon-size t-icon-disabled"/>
      <IconUnFavorite v-else-if="fileInfo.is_favorite === null || fileInfo.is_favorite === false" class="t-icon-size t-icon-hover" @click="toggleFavorite" />
      <IconFavorite   v-else-if="fileInfo.is_favorite === true" class="t-icon-size t-icon-hover" @click="toggleFavorite" />
      <IconRotateRight 
        :class="[
          't-icon-size',
          fileIndex >= 0 ? 't-icon-hover' : 't-icon-disabled',
        ]" 
        :style="{ transform: `rotate(${(fileInfo?.rotate ?? 0)}deg)`, transition: 'transform 0.3s ease-in-out' }" 
        @click="clickRotate"
      />
      <IconFileInfo :class="['t-icon-size t-icon-hover', config.showFileInfo ? 't-icon-focus' : '']" @click="clickShowFileInfo" />
      <IconSave 
        :class="[
          't-icon-size',
          fileIndex >= 0 ? 't-icon-hover' : 't-icon-disabled'
        ]"
        @click="clickSave"
      />
      <IconDelete
        :class="[
          't-icon-size',
          fileIndex >= 0 ? 't-icon-hover' : 't-icon-disabled'
        ]"
        @click="clickDelete"
      />
      <!-- <IconFullScreen v-if="!config.isFullScreen" class="t-icon-size t-icon-hover" @click="setFullScreen" /> -->
      <!-- <IconRestoreScreen v-if="config.isFullScreen" class="t-icon-size t-icon-hover" @click="exitFullScreen" /> -->
      <component 
        :is="config.isFullScreen ? IconRestoreScreen : IconFullScreen" 
        class="t-icon-size t-icon-hover" 
        @click="toggleFullScreen" 
      />
      
      <IconSeparator v-show="config.isFullScreen" class="t-icon-size t-icon-disabled" />
      <component v-show="config.isFullScreen"
        :is="config.isPinned ? IconPin : IconUnPin" 
        :class="[
          't-icon-size',
          fileIndex >= 0 ? 't-icon-hover' : 't-icon-disabled'
        ]" 
        @click="config.isPinned = !config.isPinned" 
      />
      <IconClose v-show="config.isFullScreen" class="t-icon-size t-icon-hover" @click="appWindow.close()" />
    </div>

    <!-- content -->
    <div class="flex t-color-text t-color-bg h-screen overflow-hidden">

      <!-- image container -->
      <div ref="viewerContainer" class="relative flex-1 flex justify-center items-center overflow-hidden">
        
        <!-- show zoom scale -->
        <transition name="fade">
          <div v-if="isScaleChanged" class="absolute left-1/2 top-12 px-2 py-1 z-10 t-color-bg opacity-50 rounded-lg">
            <slot>{{(imageScale * 100).toFixed(0)}} %</slot>
          </div>
        </transition>

        <!-- prev   -->
        <div v-if="fileIndex > 0"
          class="absolute left-0 w-40 h-full z-10 flex items-center justify-start cursor-pointer group" 
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
          class="absolute right-0 w-40 h-full z-10 flex items-center justify-end cursor-pointer group" 
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
        <FileInfo v-if="config.showFileInfo" 
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

import { ref, watch, computed, onMounted, onUnmounted } from 'vue';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { emit, listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';
import { useI18n } from 'vue-i18n';
import { useConfigStore } from '@/stores/configStore';

import TitleBar from '@/components/TitleBar.vue';
import Image from '@/components/Image.vue';
import FileInfo from '@/components/FileInfo.vue';

import IconPlay from '@/assets/play.svg';
import IconPause from '@/assets/pause.svg';
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
import IconPin from '@/assets/pin-filled.svg';
import IconUnPin from '@/assets/pin.svg';
import IconLeft from '@/assets/arrow-left.svg';
import IconRight from '@/assets/arrow-right.svg';
import IconSeparator from '@/assets/separator.svg';
import IconClose from '@/assets/close.svg';


/// i18n
const { locale, messages } = useI18n();
const localeMsg = computed(() => messages.value[locale.value]);

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
// const showFileInfo = ref(false); // Show the file info panel

const imageRef = ref(null);     // Image reference
const imageSrc = ref(null);
const imageCache = new Map();   // Cache images to prevent reloading
const loadError = ref(false);   // Track if there was an error loading the image

const autoPlay = ref(false);        // Auto play state
let timer = null;                   // Timer for auto play

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

listen('message-from-image', (event) => {
  const { message } = event.payload;
  console.log('ImageViewer.vue: message-from-image:', message);
  switch (message) {
    case 'scale':
      imageScale.value = event.payload.scale;
      imageMinScale.value = event.payload.minScale;
      imageMaxScale.value = event.payload.maxScale;
      break;
    default:
      break;
  }
});

// watch language
watch(() => config.language, (newLanguage) => {
    console.log('Language changed to:', newLanguage);
    locale.value = newLanguage; // update locale based on config.language
});

// watch full screen
watch(() => config.isFullScreen, async (newFullScreen) => {
  await appWindow.setFullscreen(newFullScreen);
  await appWindow.setResizable(!newFullScreen);
  // await appWindow.setDecorations(false);
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

watch(() => fileIndex.value, async (newIndex) => {
  if(newIndex === -1) {
    autoPlay.value = false;
  } 
});

watch(() => [autoPlay.value, config.autoPlayInterval], ([newAutoPlay, newInterVal]) => {
  console.log('autoPlay:', newAutoPlay, newInterVal);
  if(newAutoPlay) {
    clearInterval(timer);
    timer = setInterval(() => {
      clickNext();
    }, newInterVal * 1000);
  } else {
    clearInterval(timer);
  }
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
  if(autoPlay.value && fileIndex.value === fileCount.value - 1) {
    emit('message-from-image-viewer', { message: 'home' });
  } else {
    emit('message-from-image-viewer', { message: 'next' });
  }
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

    fileInfo.value.rotate += 90;
    saveRotate(fileId.value, fileInfo.value.rotate);
    // update grid view
    emit('message-from-image-viewer', { message: 'rotate', rotate: fileInfo.value.rotate });
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
  emit('message-from-image-viewer', { message: 'favorite', favorite: fileInfo.value.is_favorite });

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
  config.showFileInfo = !config.showFileInfo;
}

// Close the file info panel from the child component
function closeFileInfo() {
  config.showFileInfo = false;
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
    case 'ArrowLeft':
      clickPrev();
      break;
    case 'ArrowRight':
      clickNext();
      break;
    case 'ArrowUp':
      clickZoomIn();
      break;
    case 'ArrowDown':
      clickZoomOut();
      break;
    case 'p':
      autoPlay.value = !autoPlay.value;
      break;
    case 'f':
      toggleFavorite();
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
      appWindow.close(); // Close the window
      break;
  }
}

</script>

<style scoped>
/* Disable text selection while dragging */
* {
  user-select: none;
}

@media (max-width: 800px) {
  #responsiveDiv {
    visibility: hidden;
  }
}
@media (min-width: 800px) {
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

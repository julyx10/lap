<template>

  <div 
    :class="[
      'relative w-screen h-screen flex flex-col overflow-hidden',
      config.isFullScreen ? 'fixed top-0 left-0 z-50' : '',
    ]"
  >
    <!-- title bar -->
    <TitleBar v-if="!config.isFullScreen"
      :titlebar="`jc-photo ${localeMsg.image_view_title}${fileIndex >= 0 ? ` - ${fileIndex + 1}/${fileCount}` : ''}`"
      viewName="ImageViewer"
    />

    <!-- Toolbar -->
    <div 
      :class="[
        'absolute -top-0.5 left-1/2 bg-transparent z-40 transform -translate-x-1/2 group',
        config.isFullScreen ? 'h-14' : 'h-10',
      ]"
      data-tauri-drag-region
    >
      <div id="responsiveDiv"
        :class="[
          'px-4 h-10 space-x-5 rounded-lg flex flex-row items-center justify-center t-color-bg t-color-text',
          config.isFullScreen && !config.isPinned ? '-translate-y-8 opacity-0 group-hover:translate-y-1 group-hover:opacity-50 transition-transform duration-300 ease-in-out' : '',
          config.isFullScreen && config.isPinned ? 'opacity-80 translate-y-1 transition-transform duration-300 ease-in-out' : ''
        ]"
      >
        <IconPrev 
          :class="[
            't-icon-size',  
            fileIndex > 0 ? 't-icon-hover' : 't-icon-disabled'
          ]" 
          @click="clickPrev()" 
        />
        <IconNext 
          :class="[
            't-icon-size',
            fileIndex >=0 && fileIndex < fileCount -1 ? 't-icon-hover' : 't-icon-disabled'
          ]" 
          @click="clickNext()" 
        />
        <component 
          :is="autoPlay ? IconPause : IconPlay" 
          :class="[
            't-icon-size',
            fileIndex >= 0 ? 't-icon-hover' : 't-icon-disabled'
          ]" 
          @click="clickPlay()" 
        />  
        <IconZoomIn
          :class="[
            't-icon-size',
            fileIndex >= 0 && imageScale < imageMaxScale ? 't-icon-hover' : 't-icon-disabled'
          ]"
          @click="clickZoomIn()" 
        />
        <IconZoomOut
          :class="[
            't-icon-size',
            fileIndex >= 0 && imageScale > imageMinScale ? 't-icon-hover' : 't-icon-disabled'
          ]"
          @click="clickZoomOut()" 
        />
        <component 
          :is="config.isZoomFit ? IconZoomFit : IconZoomOriginal" 
          :class="[
            't-icon-size',
            fileIndex >= 0 ? 't-icon-hover' : 't-icon-disabled'
          ]" 
          @click="toggleZoomFit()" 
        />
        <!-- <IconUnFavorite v-if="!fileInfo" class="t-icon-size t-icon-disabled"/>
        <IconUnFavorite v-else-if="fileInfo.is_favorite === null || fileInfo.is_favorite === false" class="t-icon-size t-icon-hover" @click="toggleFavorite()" />
        <IconFavorite   v-else-if="fileInfo.is_favorite === true" class="t-icon-size t-icon-hover" @click="toggleFavorite()" /> -->
        <IconFavorite 
          :class="[
            't-icon-size', 
            fileInfo ? 't-icon-hover' : 't-icon-disabled',
            fileInfo?.is_favorite ? 't-color-text-focus t-icon-focus-hover' : '',
          ]" 
          @click="toggleFavorite()" 
        />
        <IconRotate
          :class="[
            't-icon-size',
            fileIndex >= 0 ? 't-icon-hover' : 't-icon-disabled',
            iconRotate % 360 > 0 ? 't-color-text-focus' : '',
          ]" 
          :style="{ transform: `rotate(${(iconRotate)}deg)`, transition: 'transform 0.3s ease-in-out' }" 
          @click="clickRotate()"
        />
        <DropDownMenu
          :iconMenu="IconMore"
          :menuItems="moreMenuItems"
          :disabled="fileIndex === -1"
          @click.stop
        />

        <component v-if="isWin"
          :is="config.isFullScreen ? IconRestoreScreen : IconFullScreen" 
          class="t-icon-size t-icon-hover" 
          @click="toggleFullScreen()" 
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
    </div>

    <!-- content -->
    <div class="flex t-color-text t-color-bg h-screen overflow-hidden">
      <!-- image container -->
      <div ref="viewerContainer" class="relative flex-1 flex justify-center items-center overflow-hidden">
        
        <!-- show zoom scale -->
        <transition name="fade">
          <div v-if="isScaleChanged" 
            :class="[
              'absolute left-1/2  px-2 py-1 z-10 t-color-bg opacity-50 rounded-lg',
              config.isFullScreen && config.isPinned ? 'top-12' : 'top-2'
            ]"
          >
            <slot>{{(imageScale * 100).toFixed(0)}} %</slot>
          </div>
        </transition>

        <!-- prev   -->
        <div v-if="fileIndex > 0"
          class="absolute left-0 w-40 h-full z-10 flex items-center justify-start cursor-pointer group" 
          @click="clickPrev()"
        >
          <div class="m-3 p-2 t-color-bg-light rounded-full hidden group-hover:block ">
            <IconLeft class="t-icon-size t-icon-hover"/>
          </div>
        </div>

        <!-- image -->
        <Image v-if="fileIndex >= 0" 
          ref="imageRef" 
          :src="imageSrc" 
          :rotate="fileInfo?.rotate ?? 0" 
          :isZoomFit="config.isZoomFit"
          @dblclick="toggleZoomFit()"
        />
        <p v-else>
          {{ loadError ? $t('image_view_failed') + ': ' + filePath : $t('image_view_loading') }}
        </p>

        <!-- no image selected -->
        <p v-else>
          {{ $t('image_view_no_image') }}
        </p>

        <!-- next -->
        <div v-if="fileIndex < fileCount - 1"
          class="absolute right-0 w-40 h-full z-10 flex items-center justify-end cursor-pointer group" 
          @click="clickNext()"
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
import { useI18n } from 'vue-i18n';
import { config, isWin, isMac } from '@/common/utils';
import { getFileInfo, getFileImage, setFileFavorite } from '@/common/api';

import TitleBar from '@/components/TitleBar.vue';
import Image from '@/components/Image.vue';
import FileInfo from '@/components/FileInfo.vue';
import DropDownMenu from '@/components/DropDownMenu.vue';

import { 
  IconPrev,
  IconNext,
  IconPlay,
  IconPause,
  IconZoomIn,
  IconZoomOut,
  IconZoomFit,
  IconZoomOriginal,
  IconUnFavorite,
  IconFavorite,
  IconRotate,
  IconMore,
  IconEdit,
  IconPrint,
  IconRename,
  IconDelete,
  IconCopy,
  IconMoveTo,
  IconProperties,
  IconFullScreen,
  IconRestoreScreen,
  IconPin,
  IconUnPin,
  IconLeft,
  IconRight,
  IconSeparator,
  IconClose
 } from '@/common/icons';

/// i18n
const { locale, messages } = useI18n();
const localeMsg = computed(() => messages.value[locale.value]);

const appWindow = getCurrentWebviewWindow()

// input parameters
const fileId = ref(0);       // File ID
const fileIndex = ref(0);       // Index of the current file
const fileCount = ref(0);       // Total number of files
const filePath = ref('');       // File path
const nextFilePath = ref('');   // Next file path to preload

const fileInfo = ref(null);
const iconRotate = ref(0);      // icon rotation angle
// const showFileInfo = ref(false); // Show the file info panel

const imageRef = ref(null);     // Image reference
const imageSrc = ref('');
const imageCache = new Map();   // Cache images to prevent reloading
const loadError = ref(false);   // Track if there was an error loading the image

const autoPlay = ref(false);        // Auto play state
let timer = null;                   // Timer for auto play

const imageScale = ref(1);          // Image scale
const imageMinScale = ref(0);       // Minimum image scale
const imageMaxScale = ref(10);      // Maximum image scale
const isScaleChanged = ref(false);  // Scale changed state

// more menuitems
const moreMenuItems = computed(() => {
  return [
    {
      label: localeMsg.value.menu_item_edit,
      icon: IconEdit,
      action: () => {
        console.log('Edit:', filePath.value);
      }
    },
    {
      label: localeMsg.value.menu_item_print,
      icon: IconPrint,
      action: () => {
        console.log('Print:', filePath.value);
      }
    },
    {
      label: localeMsg.value.menu_item_copy,
      icon: IconCopy,
      shortcut: isMac ? '⌘C' : 'Ctrl+C',
      action: () => {
        console.log('Copy:', filePath.value);
      }
    },
    {
      label: "-",   // separator
      action: null
    },
    {
      label: localeMsg.value.menu_item_rename,
      icon: IconRename,
      action: () => {
        console.log('Rename:', filePath.value);
      }
    },
    {
      label: localeMsg.value.menu_item_move_to,
      icon: IconMoveTo,
      action: () => {}
    },
    {
      label: localeMsg.value.menu_item_copy_to,
      action: () => {}
    },
    {
      label: localeMsg.value.menu_item_delete,
      icon: IconDelete,
      shortcut: isMac ? '⌘⌫' : 'Del',
      action: () => {
        clickDelete();
      }
    },
    {
      label: "-",   // separator
      action: null
    },

    {
      label: "-",   // separator
      action: null
    },
    {
      label: localeMsg.value.menu_item_properties,
      icon: IconProperties,
      shortcut: isMac ? '⌘I' : 'Ctrl+I',
      action: () => {
        clickShowFileInfo();
      }
    }
  ];
});

let unlistenResize: () => void;
let unlistenImg: () => void;
let unlistenImage: () => void;
let unlistenGridView: () => void;

onMounted(async() => {
  window.addEventListener('keydown', handleKeyDown);
  unlistenResize = await appWindow.listen('tauri://resize', handleResize);     // macOS: Listen for full screen change

  const urlParams = new URLSearchParams(window.location.search);
  
  fileId.value    = Number(urlParams.get('fileId'));
  fileIndex.value = Number(urlParams.get('fileIndex'));
  fileCount.value = Number(urlParams.get('fileCount'));
  filePath.value     = decodeURIComponent(urlParams.get('filePath'));
  nextFilePath.value = decodeURIComponent(urlParams.get('nextFilePath'));

  // Listen 
  unlistenImg = await listen('update-img', async (event) => {
    fileId.value    = Number(event.payload.fileId);
    fileIndex.value = Number(event.payload.fileIndex);
    fileCount.value = Number(event.payload.fileCount);
    filePath.value     = decodeURIComponent(event.payload.filePath);
    nextFilePath.value = decodeURIComponent(event.payload.nextFilePath);
    console.log('update-img', fileId.value, fileIndex.value, fileCount.value, filePath.value )
  });

  unlistenImage = await listen('message-from-image', (event) => {
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

  unlistenGridView = await listen('message-from-grid-view', (event) => {
    const { message } = event.payload;
    console.log('ImageViewer.vue: message-from-grid-view:', message);
    switch (message) {
      case 'favorite':
        toggleFavorite();
        break;
      case 'rotate':
        if(imageRef.value) {
          imageRef.value.rotateRight();
          iconRotate.value += 90;
        }
        break;
      default:
        break;
    }
  });

});

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeyDown);
  
  // unlisten
  unlistenResize();
  unlistenImg();
  unlistenImage();
  unlistenGridView();
});

// Handle keyboard shortcuts
function handleKeyDown(event) {
  const key = event.key;
  const isCmdKey = isMac ? event.metaKey : event.ctrlKey;

  if (isCmdKey && key.toLowerCase() === 'c') {
    event.preventDefault();
    // copyItem();
  } else if (isCmdKey && key.toLowerCase() === 'p') {
    event.preventDefault();
    autoPlay.value = !autoPlay.value;
  } else if (isCmdKey && key.toLowerCase() === 'f') {
    event.preventDefault();
    toggleFavorite();
  } else if (isCmdKey && key.toLowerCase() === 'r') {
    event.preventDefault();
    clickRotate();
  } else if (isCmdKey && key.toLowerCase() === 'i') {
    event.preventDefault();
    clickShowFileInfo();
  } else if (key === 'Delete' || key === 'Backspace') {
    event.preventDefault();
    // deleteItem();
  } else if (keyActions[key]) {
    event.preventDefault();
    keyActions[key]();
  }
}

const keyActions = {
  ArrowLeft:  () => clickPrev(),
  ArrowRight: () => clickNext(),
  Home:       () => clickHome(),
  End:        () => clickEnd(),
  ArrowUp:    () => clickZoomIn(),
  ArrowDown:  () => clickZoomOut(),
  ' ':        () => toggleZoomFit(),
  Escape:     () => appWindow.close(),
};

// Handle resize event
const handleResize = async () => {
  if(isMac) {
    config.isFullScreen = await appWindow.isFullscreen();
    console.log('handleFullScreenChange:', config.isFullScreen);
  }
};

// watch language
watch(() => config.language, (newLanguage) => {
    console.log('Language changed to:', newLanguage);
    locale.value = newLanguage; // update locale based on config.language
});

// watch full screen (win only)
watch(() => config.isFullScreen, async (newFullScreen) => {
  if(!isWin) return;
  await appWindow.setFullscreen(newFullScreen);
  await appWindow.setResizable(!newFullScreen);
  // await appWindow.setDecorations(false);
}); 

// watch file changed
watch(() => fileId.value, async () => {
  fileInfo.value = await getFileInfo(fileId.value);
  iconRotate.value = fileInfo.value.rotate || 0;
  loadImage(filePath.value);
});

// watch scale
watch(() => imageScale.value, () => {
  isScaleChanged.value = true;
  
  setTimeout(() => {
    isScaleChanged.value = false;
  }, 1000);
});

// watch file index
watch(() => fileIndex.value, async (newIndex) => {
  if(newIndex === -1) {
    autoPlay.value = false;
    iconRotate.value = 0; // reset rotation
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
  if(filePath.length === 0) {
    console.log('loadImage - filePath is empty');
    return;
  }
  try {
    loadError.value = false;

    // Check if the image is already cached
    if (imageCache.has(filePath)) {
      imageSrc.value = imageCache.get(filePath);
    } else {
      const imageBase64 = await getFileImage(filePath);
      if (imageBase64) {
        imageSrc.value = `data:image/jpeg;base64,${imageBase64}`;
        imageCache.set(filePath, imageSrc.value);
      }
    }

    // Preload the next image
    preLoadImage(nextFilePath.value);
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
      const imageBase64 = await getFileImage(filePath);
      if (imageBase64) {
        const imageSrc = `data:image/jpeg;base64,${imageBase64}`;
        imageCache.set(filePath, imageSrc);
      }
    }
  } catch (error) {
    console.error('preLoadImage:', error);
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

function clickHome() {
  emit('message-from-image-viewer', { message: 'home' });
}

function clickEnd() {
  emit('message-from-image-viewer', { message: 'end' });
}

function clickPlay() {
  autoPlay.value = !autoPlay.value;
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
  config.isZoomFit =!config.isZoomFit;
};

// rotate image
const clickRotate = () => {
  if(imageRef.value) {
    imageRef.value.rotateRight();
    iconRotate.value += 90;

    emit('message-from-image-viewer', { message: 'rotate' });
  }
};

// toggle favorite status
const toggleFavorite = async() => {
  fileInfo.value.is_favorite = fileInfo.value.is_favorite === null ? true : !fileInfo.value.is_favorite;
  emit('message-from-image-viewer', { message: 'favorite', favorite: fileInfo.value.is_favorite });

  // set db status
  await setFileFavorite(fileId.value, fileInfo.value.is_favorite);
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

// TODO: implement delete
const clickDelete = async() => {
  // if(fileIndex.value < 0) return;

  // // get current timestamp
  // fileInfo.value.deleted_at = fileInfo.value.deleted_at || fileInfo.value.deleted_at !== 0 ? 0 : Math.floor(Date.now() / 1000);

  // try {
  //   await invoke('set_file_delete', { fileId: fileId.value, deletedAt: fileInfo.value.deleted_at });
  // } catch (error) {
  //   console.error('clickDelete:', error);
  // }
  // emit('message-from-image-viewer', { message:'delete' });
}


</script>

<style scoped>
/* Disable text selection while dragging */
* {
  user-select: none;
}

@media (max-width: 600px) {
  #responsiveDiv {
    visibility: hidden;
  }
}
@media (min-width: 600px) {
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

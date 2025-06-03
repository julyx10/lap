<template>

  <div
    :class="[
      'relative w-screen h-screen flex flex-col overflow-hidden bg-base-300 text-base-content/70',
      config.isFullScreen ? 'fixed top-0 left-0 z-50' : '',
    ]"
  >
    <!-- title bar -->
    <TitleBar v-if="!config.isFullScreen"
      :titlebar="isWin ? `jc-photo ${localeMsg.image_view_title}${fileIndex >= 0 ? ` - ${fileIndex + 1}/${fileCount}` : ''}` : ''"
      viewName="ImageViewer"
    />

    <!-- Toolbar -->
    <div 
      :class="[
        'absolute left-1/2 z-40 bg-transparent transform -translate-x-1/2 select-none group',
      ]"
      data-tauri-drag-region
    >
      <div id="responsiveDiv"
        :class="[
          'px-4 h-12 space-x-2 rounded-lg flex flex-row items-center justify-center bg-base-300',
          config.isFullScreen && !config.isPinned ? '-translate-y-8 opacity-0 group-hover:translate-y-2 group-hover:opacity-80 transition-transform duration-300 ease-in-out' : '',
          config.isFullScreen && config.isPinned ? 'opacity-80 translate-y-2 transition-transform duration-300 ease-in-out' : ''
        ]"
      >
        <TButton
          :icon="IconPrev"
          :disabled="fileIndex <= 0"
          :tooltip="$t('image_view_toolbar_prev')"
          @click="clickPrev()" 
        />
        <TButton
          :icon="IconNext"
          :disabled="fileIndex < 0 || fileIndex >= fileCount - 1"
          :tooltip="$t('image_view_toolbar_next')"
          @click="clickNext()" 
        />
        <TButton
          :icon="autoPlay ? IconPause : IconPlay"
          :disabled="fileIndex < 0"
          :tooltip="autoPlay ? $t('image_view_toolbar_pause') : $t('image_view_toolbar_play') + ` (${getPlayInterval(config.autoPlayInterval)}s)`"
          @click="clickPlay()" 
        />
        <TButton
          :icon="IconZoomIn"
          :disabled="fileIndex < 0 || imageScale >= imageMaxScale"
          :tooltip="$t('image_view_toolbar_zoom_in')"
          @click="clickZoomIn()" 
        />
        <TButton
          :icon="IconZoomOut"
          :disabled="fileIndex < 0 || imageScale <= imageMinScale"
          :tooltip="$t('image_view_toolbar_zoom_out')"
          @click="clickZoomOut()"
        />
        <TButton
          :icon="config.isZoomFit ? IconZoomFit : IconZoomOriginal"
          :disabled="fileIndex < 0"
          :tooltip="config.isZoomFit ? $t('image_view_toolbar_zoom_fit') : $t('image_view_toolbar_zoom_actual')"
          @click="toggleZoomFit()"
        />
        <TButton
          :icon="IconFavorite"
          :disabled="fileIndex < 0"
          :selected="fileInfo?.is_favorite"
          :tooltip="$t('image_view_toolbar_favorite')"
          @click="toggleFavorite()"
        />
        <TButton
          :icon="IconRotate"
          :disabled="fileIndex < 0"
          :selected="iconRotate % 360 > 0"
          :iconStyle="{ transform: `rotate(${(iconRotate)}deg)`, transition: 'transform 0.3s ease-in-out' }" 
          :tooltip="$t('image_view_toolbar_rotate')"
          @click="clickRotate()"
        />

        <TButton v-if="isWin"
          :icon="config.isFullScreen ? IconFullScreen : IconRestoreScreen"
          :tooltip="config.isFullScreen ? $t('image_view_toolbar_fullscreen') : $t('image_view_toolbar_exit_fullscreen')"
          @click="toggleFullScreen()"
        />

        <DropDownMenu
          :iconMenu="IconMore"
          :menuItems="moreMenuItems"
          :disabled="fileIndex === -1"
          @click.stop
        />

        <TButton v-show="config.isFullScreen"
          :icon="IconSeparator"
          :disabled="true"
        />

        <TButton v-show="config.isFullScreen"
          :icon="config.isPinned ? IconPin : IconUnPin"
          :disabled="fileIndex < 0"
          :tooltip="config.isPinned ? $t('image_view_toolbar_pin') : $t('image_view_toolbar_unpin')"
          @click="config.isPinned = !config.isPinned"
        />
        <TButton v-show="config.isFullScreen"
          :icon="IconClose"
          :tooltip="$t('image_view_toolbar_close')"
          @click="appWindow.close()"
        />
      </div>
    </div>

    <!-- content -->
    <div ref="divContentView" class="flex h-screen overflow-hidden">
      <!-- image container -->
      <div ref="viewerContainer" class="relative flex-1 flex justify-center items-center overflow-hidden select-none ">
        
        <!-- show zoom scale -->
        <transition name="fade">
          <div v-if="isScaleChanged" 
            :class="[
              'absolute left-1/2  px-2 py-1 z-10 opacity-50 rounded-lg',
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
          <div class="m-3 p-2 rounded-full hidden group-hover:block ">
            <TButton 
              :icon="IconLeft" 
              :buttonClasses="'rounded-full'"
            />
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
          <div class="m-3 p-2 rounded-full hidden group-hover:block ">
            <TButton 
              :icon="IconRight" 
              :buttonClasses="'rounded-full'"
            />
          </div>
        </div>

        <!-- comments -->
        <div v-if="config.showComment && fileInfo?.comments?.length > 0" 
          class="absolute flex m-2 p-2 bottom-0 left-0 right-0 text-sm bg-base-100 opacity-60 rounded-lg select-text" 
        >
          <IconComment class="t-icon-size-sm shrink-0 mr-2"></IconComment>
          {{ fileInfo?.comments }}
        </div>

      </div> <!-- image container -->

      <!-- splitter -->
      <div v-if="config.showFileInfo" 
        class="w-1 mt-1 hover:bg-primary cursor-ew-resize transition-colors" 
        @mousedown="startDragging"
      ></div>

      <!-- File Info -->
      <transition
        enter-active-class="transition-transform duration-200"
        leave-active-class="transition-transform duration-200"
        enter-from-class="translate-x-full"
        enter-to-class="translate-x-0"
        leave-from-class="translate-x-0"
        leave-to-class="translate-x-full"
      >
        <div v-if="config.showFileInfo" ref="previewDiv" 
          :style="{ width: config.fileInfoPanelWidth + '%' }"
        >
          <FileInfo 
            :fileInfo="fileInfo" 
            :fileIndex="fileIndex" 
            :fileCount="fileCount" 
            @close="closeFileInfo" 
          />
        </div>
      </transition> <!-- File Info -->

    </div>

  </div>
  
  <!-- delete -->
  <MessageBox
    v-if="showDeleteMsgbox"
    :title="$t('msgbox_delete_file_title')"
    :message="`${$t('msgbox_delete_file_content', { file: fileInfo?.name })}`"
    :OkText="$t('msgbox_delete_file_ok')"
    :cancelText="$t('msgbox_cancel')"
    :warningOk="true"
    @ok="clickDeleteFile"
    @cancel="showDeleteMsgbox = false"
  />

  <ToolTip ref="toolTipRef" />

</template>


<script setup lang="ts">

import { ref, watch, computed, onMounted, onUnmounted } from 'vue';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { emit, listen } from '@tauri-apps/api/event';
import { useI18n } from 'vue-i18n';
import { config, isWin, isMac, setTheme, getPlayInterval } from '@/common/utils';
import { copyImage, getFileInfo, getFileImage } from '@/common/api';

import TitleBar from '@/components/TitleBar.vue';
import TButton from '@/components/TButton.vue';
import Image from '@/components/Image.vue';
import FileInfo from '@/components/FileInfo.vue';
import DropDownMenu from '@/components/DropDownMenu.vue';
import MessageBox from '@/components/MessageBox.vue';
import ToolTip from '@/components/ToolTip.vue';

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
  IconTrash,
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
  IconClose,
  IconComment
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

const showDeleteMsgbox = ref(false);

const isDraggingSplitter = ref(false); // Dragging state for the splitter
const divContentView = ref(null); // Reference to the content view

const toolTipRef = ref(null);

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
      label: localeMsg.value.menu_item_copy,
      icon: IconCopy,
      shortcut: isMac ? '⌘C' : 'Ctrl+C',
      action: () => {
        clickCopy();
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
      label: "-",   // separator
      action: null
    },
    // {
    //   label: localeMsg.value.menu_item_rename,
    //   icon: IconRename,
    //   action: () => {
    //     console.log('Rename:', filePath.value);
    //   }
    // },
    // {
    //   label: localeMsg.value.menu_item_move_to,
    //   icon: IconMoveTo,
    //   action: () => {
    //   }
    // },
    // {
    //   label: localeMsg.value.menu_item_copy_to,
    //   action: () => {
    //   }
    // },
    {
      label: localeMsg.value.menu_item_trash,
      icon: IconTrash,
      shortcut: isMac ? '⌘⌫' : 'Del',
      action: () => {
        showDeleteMsgbox.value = true;
      }
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
    },
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
      case 'showfileinfo':
        clickShowFileInfo();
        break;
      default:
        break;
    }
  });

  unlistenGridView = await listen('message-from-content', (event) => {
    const { message } = event.payload;
    console.log('message-from-content:', message);
    switch (message) {
      case 'favorite':
        fileInfo.value.is_favorite = event.payload.favorite;
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
    clickCopy();
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
  } else if((isMac && event.metaKey && key === 'Backspace') || (!isMac && key === 'Delete')) {
    event.preventDefault();
    showDeleteMsgbox.value = true;
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

/// watch appearance
watch(() => config.appearance, (newAppearance) => {
  setTheme(newAppearance);
});

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

watch(() => [autoPlay.value, config.autoPlayInterval], ([newAutoPlay, newInterval]) => {
  if(newAutoPlay) {
    clearInterval(timer);
    timer = setInterval(() => {
      clickNext();
    }, getPlayInterval(newInterval) * 1000);
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

// toggle favorite status
const toggleFavorite = () => {
  emit('message-from-image-viewer', { message: 'favorite' });
}

// rotate image
const clickRotate = () => {
  emit('message-from-image-viewer', { message: 'rotate' });
};

// click copy image
const clickCopy = async() => {
  copyImage(filePath.value).then(() => {
    toolTipRef.value.showTip(localeMsg.value.tooltip_copy_image_success);
  });
}

const clickDeleteFile = async() => {
  emit('message-from-image-viewer', { message: 'delete' });
  showDeleteMsgbox.value = false;
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

/// Dragging the splitter
function startDragging(event) {
  isDraggingSplitter.value = true;
  document.addEventListener('mousemove', handleMouseMove);
  document.addEventListener('mouseup', stopDragging);
}

/// stop dragging the splitter
function stopDragging() {
  isDraggingSplitter.value = false;
  document.removeEventListener('mousemove', handleMouseMove);
  document.removeEventListener('mouseup', stopDragging);
}

/// handle mouse move event
function handleMouseMove(event) {
  // console.log('handleMouseMove:', document.documentElement.clientWidth, event.clientX, leftPosition);
  if (isDraggingSplitter.value) {
    const windowWidth = document.documentElement.clientWidth - 4; // -4: border width(2px) * 2
    const leftPosition = divContentView.value.getBoundingClientRect().left - 2;  // -2: border width(2px)

    // Limit width between 10% and 50%
    config.fileInfoPanelWidth = Math.min(Math.max(((windowWidth - event.clientX)*100) / (windowWidth - leftPosition), 10), 50); 
  }
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

<template>

  <div
    :class="[
      'relative w-screen h-screen flex flex-col overflow-hidden bg-base-300 text-base-content/70',
      config.isFullScreen ? 'fixed top-0 left-0 z-50' : '',
    ]"
  >
    <!-- title bar -->
    <TitleBar v-if="!config.isFullScreen"
      :titlebar="isWin ? `jc-photo ${localeMsg.image_viewer.title}${fileIndex >= 0 ? ` - ${fileIndex + 1}/${fileCount}` : ''}` : ''"
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
          :tooltip="$t('image_viewer.toolbar.prev')"
          @click="clickPrev()" 
        />
        <div class="min-w-10 text-center text-base-content/30">
          {{ fileIndex + 1 }}/{{ fileCount }}
        </div>
        <TButton
          :icon="IconNext"
          :disabled="fileIndex < 0 || fileIndex >= fileCount - 1"
          :tooltip="$t('image_viewer.toolbar.next')"
          @click="clickNext()" 
        />
        <TButton
          :icon="isSlideShow ? IconPause : IconPlay"
          :disabled="fileIndex < 0"
          :tooltip="isSlideShow ? $t('image_viewer.toolbar.pause') : $t('image_viewer.toolbar.slide_show') + ` (${getSlideShowInterval(config.slideShowInterval)}s)`"
          @click="clickSlideShow()" 
        />
        <TButton
          :icon="IconZoomOut"
          :disabled="fileIndex < 0 || imageScale <= imageMinScale"
          :tooltip="$t('image_viewer.toolbar.zoom_out')"
          @click="clickZoomOut()"
        />
        <TButton
          :icon="IconZoomIn"
          :disabled="fileIndex < 0 || imageScale >= imageMaxScale"
          :tooltip="$t('image_viewer.toolbar.zoom_in')"
          @click="clickZoomIn()" 
        />
        <TButton
          :icon="!config.isZoomFit ? IconZoomFit : IconZoomActual"
          :disabled="fileIndex < 0"
          :tooltip="!config.isZoomFit ? $t('image_viewer.toolbar.zoom_fit') : $t('image_viewer.toolbar.zoom_actual')"
          @click="toggleZoomFit()"
        />
        <TButton
          :icon="IconFavorite"
          :disabled="fileIndex < 0"
          :selected="fileInfo?.is_favorite"
          :tooltip="$t('image_viewer.toolbar.favorite')"
          @click="toggleFavorite()"
        />
        <TButton
          :icon="IconRotate"
          :disabled="fileIndex < 0"
          :selected="iconRotate % 360 > 0"
          :iconStyle="{ transform: `rotate(${(iconRotate)}deg)`, transition: 'transform 0.3s ease-in-out' }" 
          :tooltip="$t('image_viewer.toolbar.rotate')"
          @click="clickRotate()"
        />
        <TButton
          :icon="IconTag"
          :disabled="fileIndex < 0"
          :selected="fileInfo?.has_tags"
          :tooltip="$t('image_viewer.toolbar.tag')"
          @click="clickTag()"
        />

        <TButton v-if="isWin"
          :icon="!config.isFullScreen ? IconFullScreen : IconRestoreScreen"
          :tooltip="!config.isFullScreen ? $t('image_viewer.toolbar.fullscreen') : $t('image_viewer.toolbar.exit_fullscreen')"
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
          :tooltip="!config.isPinned ? $t('image_viewer.toolbar.pin') : $t('image_viewer.toolbar.unpin')"
          @click="config.isPinned = !config.isPinned"
        />
        <TButton v-show="config.isFullScreen"
          :icon="IconClose"
          :tooltip="$t('image_viewer.toolbar.close')"
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
              'absolute left-1/2 px-2 py-1 z-10 bg-base-100 text-base-content opacity-50 rounded-lg',
              config.isFullScreen && config.isPinned ? 'top-20' : 'top-10'
            ]"
          >
            <slot>{{(imageScale * 100).toFixed(0)}} %</slot>
          </div>
        </transition>

        <template v-if="fileIndex >= 0">
          <!-- prev button -->
          <div v-if="fileIndex > 0"
            class="absolute left-0 w-40 h-full z-10 flex items-center justify-start cursor-pointer group" 
            @click="clickPrev()"
          >
            <div class="m-3 p-2 rounded-full hidden group-hover:block ">
              <TButton :icon="IconLeft" :buttonClasses="'rounded-full'" />
            </div>
          </div>
          <!-- next button -->
          <div v-if="fileIndex < fileCount - 1"
            class="absolute right-0 w-40 h-full z-10 flex items-center justify-end cursor-pointer group" 
            @click="clickNext()"
          >
            <div class="m-3 p-2 rounded-full hidden group-hover:block ">
              <TButton :icon="IconRight" :buttonClasses="'rounded-full'" />
            </div>
          </div>

          <!-- image -->
          <Image v-if="fileInfo?.file_type === 1 && imageSrc" 
            ref="imageRef" 
            :src="imageSrc" 
            :rotate="fileInfo?.rotate ?? 0" 
            :isZoomFit="config.isZoomFit"
            @dblclick="toggleZoomFit()"
          ></Image>

          <!-- video -->
          <Video v-else-if="fileInfo?.file_type === 2 && videoSrc"
            ref="videoRef"
            :src="videoSrc"
            :rotate="fileInfo?.rotate ?? 0"
            :isZoomFit="config.isZoomFit"
            @dblclick="toggleZoomFit()"
          ></Video>
          <div v-else class="h-full flex flex-col items-center justify-center text-base-content/30">
            <IconError class="w-8 h-8 mb-2" />
            <span>{{ $t('image_viewer.failed') }}</span>
          </div>

          <!-- comments -->
          <div v-if="config.showComment && fileInfo?.comments?.length > 0" 
            class="absolute flex m-2 p-2 bottom-0 left-0 right-0 text-sm bg-base-100 opacity-60 rounded-lg select-text" 
          >
            <IconComment class="t-icon-size-sm shrink-0 mr-2"></IconComment>
            {{ fileInfo?.comments }}
          </div>
        </template>

        <!-- no image selected -->
        <div v-else class="flex flex-col items-center justify-center w-full h-full text-base-content/30">
          <IconSearch class="w-8 h-8" />
          <span>{{ $t('tooltip.not_found.files') }}</span>
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
    :title="$t('msgbox.delete_file.title')"
    :message="`${$t('msgbox.delete_file.content', { file: fileInfo?.name })}`"
    :OkText="$t('msgbox.delete_file.ok')"
    :cancelText="$t('msgbox.cancel')"
    :warningOk="true"
    @ok="clickDeleteFile"
    @cancel="showDeleteMsgbox = false"
  />

  <!-- tag -->
  <TaggingDialog 
    v-if="showTaggingDialog"
    :fileIds="fileIdsToTag"
    @ok="updateFileHasTags"
    @cancel="showTaggingDialog = false"
  />

  <ToolTip ref="toolTipRef" />

</template>


<script setup lang="ts">

import { ref, watch, computed, onMounted, onUnmounted, defineAsyncComponent } from 'vue';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { emit, listen } from '@tauri-apps/api/event';
import { convertFileSrc } from '@tauri-apps/api/core';
import { useI18n } from 'vue-i18n';
import { config, isWin, isMac, setTheme, getSlideShowInterval } from '@/common/utils';
import { copyImage, getFileInfo, getFileImage, getTagsForFile, getFileHasTags } from '@/common/api';

import TitleBar from '@/components/TitleBar.vue';
import TButton from '@/components/TButton.vue';
import Image from '@/components/Image.vue';
const Video = defineAsyncComponent(() => import('@/components/Video.vue')); // dynamic import

import FileInfo from '@/components/FileInfo.vue';
import DropDownMenu from '@/components/DropDownMenu.vue';
import MessageBox from '@/components/MessageBox.vue';
import ToolTip from '@/components/ToolTip.vue';
import TaggingDialog from '@/components/TaggingDialog.vue';

import { 
  IconPrev,
  IconNext,
  IconPlay,
  IconPause,
  IconZoomIn,
  IconZoomOut,
  IconZoomFit,
  IconZoomActual,
  IconFavorite,
  IconRotate,
  IconMore,
  IconEdit,
  IconPrint,
  IconSearch,
  IconTrash,
  IconCopy,
  IconProperties,
  IconFullScreen,
  IconRestoreScreen,
  IconPin,
  IconUnPin,
  IconLeft,
  IconRight,
  IconSeparator,
  IconClose,
  IconComment,
  IconTag,
  IconError,
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
const videoRef = ref(null);     // Video reference
const imageSrc = ref('');       // Image source
const videoSrc = ref('');       // Video source
const imageCache = new Map();   // Cache images to prevent reloading
const loadError = ref(false);   // Track if there was an error loading the image

const isSlideShow = ref(false);     // Slide show state
let timer = null;                   // Timer for slide show

const imageScale = ref(1);          // Image scale
const imageMinScale = ref(0);       // Minimum image scale
const imageMaxScale = ref(10);      // Maximum image scale
const isScaleChanged = ref(false);  // Scale changed state

const showDeleteMsgbox = ref(false);
const showTaggingDialog = ref(false);
const fileIdsToTag = ref<number[]>([]);

const isDraggingSplitter = ref(false); // Dragging state for the splitter
const divContentView = ref(null); // Reference to the content view

const toolTipRef = ref(null);

// more menuitems
const moreMenuItems = computed(() => {
  return [
    {
      label: localeMsg.value.menu.file.edit,
      icon: IconEdit,
      action: () => {
        console.log('Edit:', filePath.value);
      }
    },
    {
      label: localeMsg.value.menu.file.copy,
      icon: IconCopy,
      shortcut: isMac ? '⌘C' : 'Ctrl+C',
      action: () => {
        clickCopy();
      }
    },
    {
      label: localeMsg.value.menu.file.print,
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
    //   label: localeMsg.value.menu.rename,
    //   icon: IconRename,
    //   action: () => {
    //     console.log('Rename:', filePath.value);
    //   }
    // },
    // {
    //   label: localeMsg.value.menu.move_to,
    //   icon: IconMoveTo,
    //   action: () => {
    //   }
    // },
    // {
    //   label: localeMsg.value.menu.copy_to,
    //   action: () => {
    //   }
    // },
    {
      label: isMac ? localeMsg.value.menu.file.move_to_trash : localeMsg.value.menu.file.delete,
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
      label: localeMsg.value.menu.file.properties,
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
        if(videoRef.value) {
          videoRef.value.rotateRight();
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

  if(showDeleteMsgbox.value || showTaggingDialog.value) {
    return;
  }

  if (isCmdKey && key.toLowerCase() === 'c') {
    clickCopy();
  } else if (isCmdKey && key.toLowerCase() === 'p') {
    isSlideShow.value = !isSlideShow.value;
  } else if (isCmdKey && key.toLowerCase() === 'f') {
    toggleFavorite();
  } else if (isCmdKey && key.toLowerCase() === 'r') {
    clickRotate();
  } else if (isCmdKey && key.toLowerCase() === 't') {
    clickTag();
  } else if (isCmdKey && key.toLowerCase() === 'i') {
    clickShowFileInfo();
  } else if((isMac && event.metaKey && key === 'Backspace') || (!isMac && key === 'Delete')) {
    showDeleteMsgbox.value = true;
  } else if (keyActions[key]) {
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
  '=':        () => clickZoomIn(),
  '-':        () => clickZoomOut(),
  '0':        () => clickZoomActual(),
  ' ':        () => toggleZoomFit(),
  Escape:     () => closeWindow(),
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

  // get the file's tags
  if(fileInfo.value.has_tags) {
    fileInfo.value.tags = await getTagsForFile(fileId.value);
  }

  iconRotate.value = fileInfo.value.rotate || 0;

  // load the file based on the file type
  if(fileInfo.value.file_type === 1) {
    loadImage(filePath.value);
  } else if(fileInfo.value.file_type === 2) {
    loadVideo(filePath.value);
  }
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
    isSlideShow.value = false;
    iconRotate.value = 0; // reset rotation
  } 
});

watch(() => [isSlideShow.value, config.slideShowInterval], ([newIsSlideShow, newInterval]) => {
  if(newIsSlideShow) {
    clearInterval(timer);
    timer = setInterval(() => {
      clickNext();
    }, getSlideShowInterval(newInterval) * 1000);
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
      else {
        imageSrc.value = '';
      }
    }

    // Preload the next image
    preLoadImage(nextFilePath.value);
  } catch (error) {
    imageSrc.value = '';
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

// Load the video from the file path
async function loadVideo(filePath) {
  if(filePath.length === 0) {
    console.log('loadVideo - filePath is empty');
    return;
  }
  try {
    const convertedSrc = convertFileSrc(filePath);
    console.log('loadVideo - original path:', filePath);
    console.log('loadVideo - converted src:', convertedSrc);
    videoSrc.value = convertedSrc;
  } catch (error) {
    console.error('loadVideo:', error);
  }
}

function clickPrev() {
  emit('message-from-image-viewer', { message: 'prev' });
}

function clickNext() {
  if(isSlideShow.value && fileIndex.value === fileCount.value - 1) {
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

function clickSlideShow() {
  isSlideShow.value = !isSlideShow.value;
}

const clickZoomIn = () => {
  if(imageRef.value) {
    imageRef.value.zoomIn();
  }
  else if(videoRef.value) {
    videoRef.value.zoomIn();
  }
};

const clickZoomOut = () => {
  if(imageRef.value) {
    imageRef.value.zoomOut();
  }
  else if(videoRef.value) {
    videoRef.value.zoomOut();
  }
};

const clickZoomActual = () => {
  if(imageRef.value) {
    imageRef.value.zoomActual();
  }
  else if(videoRef.value) {
    videoRef.value.zoomActual();
  }
};

const toggleZoomFit = () => {
  config.isZoomFit =!config.isZoomFit;
};

const closeWindow = () => {
  if(config.isFullScreen) {
    config.isFullScreen = false;
    appWindow.setFocus();
  } else {
    appWindow.close();
  }
}

// toggle favorite status
const toggleFavorite = () => {
  emit('message-from-image-viewer', { message: 'favorite' });
}

// rotate image
const clickRotate = () => {
  emit('message-from-image-viewer', { message: 'rotate' });
};

// tag image
const clickTag = () => {
  fileIdsToTag.value = [fileId.value];
  showTaggingDialog.value = true;
};

// click ok in tagging dialog
function updateFileHasTags(fileIds: number[]) {
  if(fileIds.length === 0) return;
  getFileHasTags(fileIds[0]).then(hasTags => {
    fileInfo.value.has_tags = hasTags;
  });
  showTaggingDialog.value = false;
}

// click copy image
const clickCopy = async() => {
  copyImage(filePath.value).then(() => {
    toolTipRef.value.showTip(localeMsg.value.tooltip.copy_image.success);
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

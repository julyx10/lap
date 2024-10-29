<template>
  <div ref="scrollable" class="flex-1 overflow-auto t-scrollbar">
    <div id="gridView" 
      class="px-2 grid gap-2"
      :style="{ gridTemplateColumns: `repeat(auto-fit, minmax(${gridSize}px, 1fr))` }"
    >
      <div 
        v-for="(file, index) in fileList" 
        :key="index"
        :id="'item-' + index"
        :class="[
          'p-2 border-2 rounded-lg hover:text-gray-300 hover:bg-gray-600 cursor-pointer transition duration-200', 
          index === gContentIndex ? 'border-sky-500' : 'border-gray-800'
        ]"
        @click="clickFile(index)"
        @dblclick="openImageViewer(index, true)"
      >
        <div class="flex flex-col items-center">
          <img v-if="file.thumbnail"
            :src="file.thumbnail" 
            :class="[
              'rounded  transition duration-200', 
              isFitWidth ? 'object-cover' : 'object-contain'
            ]"
            :style="{ width: `${gridSize}px`, height: `${gridSize}px` }"
          />
          <div v-else 
            class="rounded flex items-center justify-center"
            :style="{ width: `${gridSize}px`, height: `${gridSize}px` }"
          >
            <!-- <IconFolder class="size-1/2"/> -->
            <IconPhoto class="size-1/2"/>
          </div>
          <p class="pt-1 text-sm text-center">{{ shortenFilename(file.name) }}</p>
          <p class="text-sm">{{ file.width }}x{{ file.height }}</p>
          <!-- <p class="text-sm">{{ file.e_model }}</p> -->
          <!-- <p class="text-sm">{{ formatFileSize(file.size) }}</p> -->
        </div>
      </div>

    </div>
  </div>
</template>


<script setup lang="ts">

import { ref, inject, watch, onMounted, onUnmounted } from 'vue';
// import VueLazyload from 'vue-lazyload';
import { listen } from '@tauri-apps/api/event';
import { WebviewWindow } from '@tauri-apps/api/webviewWindow';
import { shortenFilename, formatFileSize } from '@/common/utils';

// import IconFolder from '@/assets/240-folder.svg';
import IconPhoto from '@/assets/photo.svg';

const props = defineProps({
  fileList: {
    type: Array,
    required: true,
  },
  gridSize: {
    type: Number,
    default: 200,     // from 100 to 400
  },
  isFitWidth: Boolean,
});

// import { useI18n } from 'vue-i18n';
// const { locale, messages } = useI18n();
// const msg = computed(() => messages.value[locale.value]);

const gContentIndex = inject('gContentIndex'); // global selected item index
const gShowImageViewer = inject('gShowImageViewer'); // global show image viewer

const scrollable = ref(null); // Ref for the scrollable element

onMounted(() => {
  window.addEventListener('keydown', handleKeyDown);

  listen('message-from-image-viewer', (event) => {
    const { message } = event.payload;
    console.log('message-from-image-viewer:', message);
    switch (message) {
      case 'prev':
        gContentIndex.value = Math.max(gContentIndex.value - 1, 0);
        break;
      case 'next':
        gContentIndex.value = Math.min(gContentIndex.value + 1, props.fileList.length - 1);
        break;
      case 'close':
        gShowImageViewer.value = false;
        break;
      default:
        break;
    }
  });
});


onUnmounted(() => {
  window.removeEventListener('keydown', handleKeyDown);
});


// Select the file
function clickFile(index: number) {
  gContentIndex.value = index;
}

const keyActions = {
  ArrowDown: ()  => gContentIndex.value = Math.min(gContentIndex.value + getColumnCount(), props.fileList.length - 1),
  ArrowRight: () => gContentIndex.value = Math.min(gContentIndex.value + 1, props.fileList.length - 1),
  ArrowUp: ()    => gContentIndex.value = Math.max(gContentIndex.value - getColumnCount(), 0),
  ArrowLeft: ()  => gContentIndex.value = Math.max(gContentIndex.value - 1, 0),
  // Home: ()       => gContentIndex.value = 0,
  // End: ()        => gContentIndex.value = props.fileList.length - 1,
  Enter: () => openImageViewer(gContentIndex.value, true),
  Space: () => openImageViewer(gContentIndex.value, true),
};

// Handle keydown event
function handleKeyDown(event) {
  if (keyActions[event.key]) {
    event.preventDefault(); // Prevent the default action
    keyActions[event.key](); 
  }
}

// watch for changes in the file list
watch(() => props.fileList, (newList) => {
  gContentIndex.value = - 1;

  const element = scrollable.value; // Get the scrollable element
  element.scrollTop = 0;
});

// watch for changes in the selected item index
watch (() => gContentIndex.value, (newIndex) => {
  openImageViewer(newIndex);
  scrollToItem(newIndex);
});


// Open the image viewer window
async function openImageViewer(index: number, createNew = false) {
  const webViewLabel = 'imageviewer';

  const fileCount = props.fileList.length;
  if (index < 0 || index >= fileCount) {
    return;
  }

  const file = props.fileList[index];
  const encodedFilePath = encodeURIComponent(file.file_path);
  let imageWindow = await WebviewWindow.getByLabel(webViewLabel);

  // create a new window if it doesn't exist
  if (!imageWindow) {
    if (createNew) {
      imageWindow = new WebviewWindow(webViewLabel, {
        url: `/image-viewer?fileId=${file.id}&filePath=${encodedFilePath}&fileIndex=${index}&fileCount=${fileCount}`,
        title: 'Image Viewer',
        width: 800,
        height: 600,
        transparent: true,
        decorations: false,
      });

      imageWindow.once('tauri://created', () => {
        gShowImageViewer.value = true;
        console.log('ImageViewer window created');
      });

      imageWindow.once('tauri://close-requested', () => {
        gShowImageViewer.value = false;
        imageWindow.close();
        console.log('ImageViewer window is closing');
      });

      imageWindow.once('tauri://error', (e) => {
        console.error('Error creating ImageViewer window:', e);
      });
    }
  } else {    // update the existing window
    await imageWindow.emit('update-img', { 
      fileId: file.id, 
      filePath: encodedFilePath, 
      fileIndex: index,   // selected file index
      fileCount: fileCount, // total files length
    });
  }
};

// make the selected item always visible in a scrollable container
function scrollToItem(index) {
  const item = document.getElementById(`item-${index}`);
  if (item) {
    item.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
  }
};

// function to get the number of columns in the grid
function getColumnCount() {
  // get the first element with the id 'gridView'
  const gridContainer = document.querySelector('#gridView');

  const computedStyle = getComputedStyle(gridContainer);
  const gridTemplateColumns = computedStyle.gridTemplateColumns;

  // Split by space to account for grid definitions
  const columnCount = gridTemplateColumns.split(' ').length;

  console.log('getColumnCount:', columnCount);

  return columnCount;
}

</script>


<style scoped>
* {
  user-select: none;
}
</style>

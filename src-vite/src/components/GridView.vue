<template>
  <div ref="scrollable" class="flex-1 overflow-auto t-scrollbar">
    <div id="gridView" class="px-2 grid grid-cols-[repeat(auto-fit,minmax(200px,1fr))] gap-2">
      
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
          <img 
            :src="file.thumbnail ? file.thumbnail : '/src/assets/photo.svg'" 
            :class="[
              'w-48 h-48 rounded', 
              isFitWidth ? 'object-cover' : 'object-contain'
            ]"
          />
          <p class="text-center">{{ shortenFilename(file.name) }}</p>
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
import { WebviewWindow } from '@tauri-apps/api/window';
import { shortenFilename, formatFileSize } from '@/common/utils';

const props = defineProps({
  fileList: {
    type: Array,
    required: true,
  },
  isFitWidth: Boolean,
});

// import { useI18n } from 'vue-i18n';
// const { locale, messages } = useI18n();
// const msg = computed(() => messages.value[locale.value]);

const gContentIndex = inject('gContentIndex'); // global selected item index
const scrollable = ref(null); // Ref for the scrollable element

onMounted(() => {
  window.addEventListener('keydown', handleKeyDown);

  listen('message-from-image-viewer', (event) => {
    const { message } = event.payload;
    switch (message) {
      case 'prev':
        gContentIndex.value = Math.max(gContentIndex.value - 1, 0);
        break;
      case 'next':
        gContentIndex.value = Math.min(gContentIndex.value + 1, props.fileList.length - 1);
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
function openImageViewer(index: number, createNew = false) {
  const fileCount = props.fileList.length;
  if (index < 0 || index >= fileCount) {
    return;
  }

  const file = props.fileList[index];
  const encodedFilePath = encodeURIComponent(file.file_path);
  let imageWindow = WebviewWindow.getByLabel('imageviewer');

  if (imageWindow) {
    imageWindow.emit('update-img', { 
      fileId: file.id, 
      filePath: encodedFilePath, 
      fileIndex: index,   // selected file index
      fileCount: fileCount, // total files length
    });
  } else if (createNew) {
    imageWindow = new WebviewWindow('imageviewer', {
      url: `/image-viewer?fileId=${file.id}&filePath=${encodedFilePath}&fileIndex=${index}&fileCount=${fileCount}`,
      title: 'Image Viewer',
      width: 800,
      height: 600,
      transparent: true,
      decorations: false,
      // skipTaskbar: true,
    });

    imageWindow.once('tauri://created', () => {
      console.log('ImageViewer window created');
    });

    imageWindow.once('tauri://error', (e) => {
      console.error('Error creating ImageViewer window:', e);
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

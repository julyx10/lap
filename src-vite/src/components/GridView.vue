<template>
  <div ref="scrollableDiv" class="flex-1 overflow-auto t-scrollbar">
    <div class="px-2 gap-2 grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-6 2xl:grid-cols-8">
      <div 
        v-for="(file, index) in fileList" 
        :key="index" 
        :class="[
          'p-2 border-2 rounded-lg hover:text-gray-300 hover:bg-gray-600 cursor-pointer transition duration-200', 
          index === gSelectItemIndex ? 'border-sky-500' : 'border-gray-800'
        ]"
        @click="clickFile(index)"
        @dblclick="openImageViewer(index, true)"
      >
        <div class="flex flex-col items-center">
          <img 
            :src="file.thumbnail ? file.thumbnail : '/src/assets/photo.svg'" 
            class="w-full h-36 object-cover rounded "
          />
          <p class="text-center">{{ shortenFilename(file.name) }}</p>
          <p class="text-sm">{{ file.width }}x{{ file.height }}</p>
          <!-- <p class="text-sm">{{ file.e_model }}</p> -->
          <!-- <p class="text-sm text-right text-gray-400">{{ formatFileSize(file.size) }}</p> -->
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, inject, watch, computed, onMounted, onUnmounted } from 'vue';
import { listen } from '@tauri-apps/api/event';
import { WebviewWindow } from '@tauri-apps/api/window';
import { formatFileSize } from '@/common/utils';
import { shortenFilename } from '../common/utils';
import { useI18n } from 'vue-i18n';

const { locale, messages } = useI18n();
const msg = computed(() => messages.value[locale.value]);

const props = defineProps({
  fileList: {
    type: Array,
    required: true,
  },
});

const gSelectItemIndex = inject('gSelectItemIndex'); // global selected item index

const fileListLength = computed(() => props.fileList.length);
const scrollableDiv = ref(null);


onMounted(() => {
  window.addEventListener('keydown', handleKeyDown);

  listen('message-from-image-viewer', (event) => {
    const { message } = event.payload;
    switch (message) {
      case 'prev':
        gSelectItemIndex.value = Math.max(gSelectItemIndex.value - 1, 0);
        break;
      case 'next':
        gSelectItemIndex.value = Math.min(gSelectItemIndex.value + 1, fileListLength.value - 1);
        break;
      default:
        break;
    }
  });
});


onUnmounted(() => {
  window.removeEventListener('keydown', handleKeyDown);
});


// watch for changes in the selected item index
watch (() => gSelectItemIndex.value, (newIndex) => {
  openImageViewer(newIndex);
});


// Handle keydown event
function handleKeyDown(event) {
  if (event.key === 'ArrowDown' || event.key === 'ArrowUp' || event.key === 'Enter') {
    event.preventDefault();   // disable default event
  }

  if (event.key === 'ArrowDown') {
    gSelectItemIndex.value = Math.min(gSelectItemIndex.value + 1, fileListLength.value - 1);
  } else if (event.key === 'ArrowUp') {
    gSelectItemIndex.value = Math.max(gSelectItemIndex.value - 1, 0);
  } else if (event.key === 'Enter') {
    openImageViewer(gSelectItemIndex.value, true);
  }
}

// Select the file
function clickFile(index: number) {
  gSelectItemIndex.value = index;
}


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
      resizable: true,
    });

    imageWindow.once('tauri://created', () => {
      console.log('ImageViewer window created');
    });

    imageWindow.once('tauri://error', (e) => {
      console.error('Error creating ImageViewer window:', e);
    });
  }
};

</script>


<style scoped>
* {
  user-select: none;
}
</style>

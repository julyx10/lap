<template>
  <div ref="scrollableDiv" class="flex-1 mt-12 overflow-auto scrollbar-thin scrollbar-thumb-gray-700 scrollbar-track-gray-800">
    <div class="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-6 2xl:grid-cols-8 gap-2 p-2">
      <div 
        v-for="(file, index) in fileList" 
        :key="index" 
        class="bg-gray-800 rounded p-2 cursor-pointer hover:bg-gray-700 transition duration-200"
        :class="index === selectedFileIndex ? 'text-gray-400 bg-gray-700' : ''"
        @click="clickFile(index)"
        @dblclick="dlbClickFile(index)"
      >
        <div class="flex flex-col items-center">
          <img 
            :src="file.thumbnail ? file.thumbnail : '/src/assets/photo.svg'" 
            alt="Thumbnail" 
            class="w-full h-36 object-cover rounded "
          />
          <p class="text-center">{{ shortenFilename(file.name) }}</p>
          <p class="text-sm">{{ file.width }}x{{ file.height }}</p>
          <p class="text-sm">{{ file.e_model }}</p>
          <!-- <p class="text-sm text-right text-gray-400">{{ formatFileSize(file.size) }}</p> -->
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, computed, onMounted, onUnmounted } from 'vue';
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

const selectedFileIndex = ref(null);
const fileListLength = computed(() => props.fileList.length);
const scrollableDiv = ref(null);

onMounted(() => {
  window.addEventListener('keydown', handleKeyDown);
});

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeyDown);
});

watch(selectedFileIndex, (new_index) => {
  if (new_index !== null) {
    console.log('selectedFileIndex...', props.fileList[new_index]);
  }
});

function handleKeyDown(event) {
  if (event.key === 'ArrowDown') {
    selectedFileIndex.value = Math.min(selectedFileIndex.value + 1, fileListLength.value - 1);
  } else if (event.key === 'ArrowUp') {
    selectedFileIndex.value = Math.max(selectedFileIndex.value - 1, 0);
  } else if (event.key === 'Enter') {
    dlbClickFile(selectedFileIndex.value);
  }
}

function clickFile(index: number) {
  selectedFileIndex.value = index;
}

function dlbClickFile(index: number) {
  if (index < 0 || index >= props.fileList.length) {
    return;
  }

  const file = props.fileList[index];
  const encodedFilePath = encodeURIComponent(file.file_path);
  let imageWindow = WebviewWindow.getByLabel('imageviewer');

  if (imageWindow) {
    imageWindow.emit('update-image', { fileId: file.id, filePath: encodedFilePath });
  } else {
    imageWindow = new WebviewWindow('imageviewer', {
      url: `/image-viewer?fileId=${file.id}&filePath=${encodedFilePath}`,
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

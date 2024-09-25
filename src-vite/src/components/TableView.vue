<template>

  <div ref="scrollableDiv" class="flex-1 mt-12 overflow-auto scrollbar-thin scrollbar-thumb-gray-700 scrollbar-track-gray-800">
    <table class="min-w-full divide-y divide-gray-600 cursor-pointer">
      <thead>
        <tr>
          <th 
            v-for="(column, index) in msg.file_info_columns" :key="index"
            :class="['text-left',
              index === 0  || index === 5 ? 'text-center' : '',
            ]"
          > {{ column }}</th>
        </tr>
      </thead>
      <tbody>
        <tr 
          v-for="(file, index) in fileList" :key="index" 
          :class="['hover:bg-gray-700', 
            index === selectedFileIndex ? 'text-gray-300 bg-gray-600' : '',
          ]" 
          @click="clickFile(index)"
          @dblclick="dlbClickFile(index)"
        >
          <td class="p-1">
            <img :src="file.thumbnail ? file.thumbnail : '/src/assets/photo.svg'" alt="Thumbnail"/>
          </td>
          <td>{{ file.name }}</td>
          <td>{{ file.width }}x{{ file.height }}</td>
          <td>{{ formatTimestamp(file.created_at) }}</td>
          <td>{{ formatTimestamp(file.modified_at) }}</td>
          <td>{{ file.e_model }}</td>
          <td class="text-right">{{ formatFileSize(file.size) }}</td>
        </tr>
      </tbody>
    </table>
  </div>

</template>


<script setup lang="ts">

import { ref, inject, watch, computed, onMounted, onUnmounted } from 'vue';
import { WebviewWindow } from '@tauri-apps/api/window';
import { formatTimestamp, formatFileSize } from '@/common/utils';
import { getFullPath } from '../common/utils';

/// i18n
import { useI18n } from 'vue-i18n';
const { locale, messages } = useI18n();
const msg = computed(() => messages.value[locale.value]);

const props = defineProps({
  filePath: {
    type: String,
    required: false,
  },
  fileList: {
    type: Array,
    required: true,
  },
});

const selectedFileIndex = ref(null);
const fileListLength = computed(() => props.fileList.length);
const scrollableDiv = ref(null);


onMounted(() => {
  // Add global keydown listener
  window.addEventListener('keydown', handleKeyDown);
});

onUnmounted(() => {
  // Remove keydown listener when component is unmounted
  window.removeEventListener('keydown', handleKeyDown);
});


/// Watch for changes in filePath
watch(() => props.filePath, (new_path) => {
  console.log('watch filePath:', new_path);
  if(new_path) {
    selectedFileIndex.value = null;
    scrollableDiv.value.scrollTop = 0;
  }
}, { deep: true });


/// Watch for changes in selectedFileIndex
watch (selectedFileIndex, (new_index) => {
  if (new_index !== null) {
    console.log('selectedFileIndex...', props.fileList[new_index]);
  }
});


// Function to handle keydown event and change the index
function handleKeyDown(event) {
  if (event.key === 'ArrowDown') {
    // Increase the index, but ensure it doesn't exceed the file list length
    selectedFileIndex.value = Math.min(selectedFileIndex.value + 1, fileListLength.value - 1);
  } else if (event.key === 'ArrowUp') {
    // Decrease the index, but ensure it doesn't go below 0
    selectedFileIndex.value = Math.max(selectedFileIndex.value - 1, 0);
  } else if (event.key === 'Enter') {
    // Open the selected file
    dlbClickFile(selectedFileIndex.value);
  }
}


/// Click a file to select it
function clickFile(index: number) {
  selectedFileIndex.value = index;
}


/// Double-click a file to open it
function dlbClickFile(index: number) {
  // Check if the index is valid
  if(index < 0 || index >= props.fileList.length) {
    return;
  }

  const file = props.fileList[index];

  // get the file path and encode it
  const filePath = getFullPath(props.filePath, file.name);
  const encodedFilePath = encodeURIComponent(filePath);

  // Check if the window is already open
  let imageWindow = WebviewWindow.getByLabel('imageviewer');

  if (imageWindow) {
    // If window exists, emit an event to update the image
    imageWindow.emit('update-image', { fileId: file.id, filePath: encodedFilePath });
  } else {
    // Create a new window to display the image
    imageWindow = new WebviewWindow('imageviewer', {
      url: `/image-viewer?fileId=${file.id}&filePath=${encodedFilePath}`,
      title: 'Image Viewer',
      width: 800,
      height: 600,
      resizable: true,
    });

    imageWindow.once('tauri://created', async () => {
      console.log('ImageViewer window created');
    });

    imageWindow.once('tauri://error', (e) => {
      console.error('Error creating ImageViewer window:', e);
    });
  }
};


</script>

<style scoped>
/* Disable text selection while dragging */
* {
  user-select: none;
}
</style>
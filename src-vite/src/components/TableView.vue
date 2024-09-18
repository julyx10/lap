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
          v-for="(file, index) in file_list" :key="index" 
          :class="['hover:bg-gray-700', 
            index === selected_file_index ? 'text-gray-300 bg-gray-600' : '',
          ]" 
          @click="clickFile(index)"
          @dblclick="dlbClickFile(index)"
        >
          <td class="p-1">
            <img :src="file.thumbnail ? file.thumbnail : '/src/assets/photo.svg'" alt="Thumbnail"/>
          </td>
          <td>{{ file.name }}</td>
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

import { ref, watch, computed, inject  } from 'vue';
import { WebviewWindow } from '@tauri-apps/api/window';
import { invoke } from '@tauri-apps/api';
import {formatTimestamp, formatFileSize } from '@/common/utils';

/// i18n
import { useI18n } from 'vue-i18n';
const { locale, messages } = useI18n();
const msg = computed(() => messages.value[locale.value]);

const props = defineProps({
  file_path: {
    type: String,
    required: false,
  },
  file_list: {
    type: Array,
    required: true,
  },
});

const selected_file_index = ref(null);
const scrollableDiv = ref(null);

/// Watch for changes in file_path
watch(() => props.file_path, (new_path) => {
  console.log('watch file_path:', new_path);
  if(new_path) {
    selected_file_index.value = null;
    scrollableDiv.value.scrollTop = 0;
  }
}, { deep: true });


/// Watch for changes in selected_file_index
watch (selected_file_index, (new_index) => {
  if (new_index !== null) {
    console.log('selected_file_index...', props.file_list[new_index]);
  }
});


/// Click a file to select it
function clickFile(index) {
  selected_file_index.value = index;
}


/// Double-click a file to open it
function dlbClickFile(index) {
  // Check if the index is valid
  if(index < 0 || index >= props.file_list.length) {
    return;
  }

  // Create a new window to display the image
  const newWindow = new WebviewWindow('imageview', {
    url: '/image',
    title: 'Image Viewer',
    width: 800,
    height: 600,
    resizable: true,
  });

  newWindow.once('tauri://created', async () => {
    try {
      // Get the file path
      const file_path = `${props.file_path}\\${props.file_list[index].name}`;
      console.log('dlbClickFile...', file_path);

      const imageBase64 = await invoke('get_file_image', { path: file_path });
      newWindow.emit('image-data', { data: imageBase64 });
    } catch (error) {
      console.error('Error fetching image data:', error);
    }
  });

  newWindow.once('tauri://error', (e) => {
    console.error('Error creating window:', e);
  });
};


</script>

  
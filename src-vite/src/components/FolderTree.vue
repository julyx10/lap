<template>
  <ul v-if="folders.length > 0">
    <li 
      v-for="(folder, index) in folders" 
      :key="index"
      @click="toggleFolder(index)"
    >
      <div class="pl-4 flex items-center hover:bg-gray-700 hover:rounded">
        <img class="w-8 p-1" src="/img64/folder.png" alt="Folder">
        {{ folder.name }}
      </div>
      <FolderTree :folders="folders[index].children" />
      <!-- Recursively display subfolders -->
        <!-- <FolderTree v-if="expandedFolders.includes(folder.id)" :folders="folder.children" /> -->
    </li>
  </ul>
  <!-- <ul v-else-if="folders.length === 0">
    <li class="pl-4 py-1">
      No folders available.
    </li>
  </ul> -->
</template>


<script setup lang="ts">

import { ref, defineProps, onMounted } from 'vue';
import FolderTree from './FolderTree.vue';

const props = defineProps({
  folders: {
    type: Array,
    required: true
  }
});


/// Display the folders on mount
onMounted(() => {
  console.log('Folders:', props.folders);
});

// const expandedFolders = ref<number[]>([]);

// const toggleFolder = (folderId: number) => {
//   if (expandedFolders.value.includes(folderId)) {
//     expandedFolders.value = expandedFolders.value.filter(id => id !== folderId);
//   } else {
//     expandedFolders.value.push(folderId);
//   }
// };
</script>


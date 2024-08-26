<template>

  <ul v-if="children && children.length > 0">
    <li v-for="child in children" :key="child.id" class="pl-4">
      <div 
        :class="[
          'm-1 flex items-center whitespace-nowrap hover:bg-gray-700', 
          g_folder_id === child.id ? 'text-gray-300' : ''
        ]" 
        @click="clickFolderName(child)"
      >
        <IconRight
          :class="[
            'p-1 flex-shrink-0 transition-transform', 
            child.is_expanded && child.children && child.children.length > 0 ? 'rotate-90' : ''
          ]"
          @click="clickExpandIcon($event, child)"
        />

        {{ child.name }}

      </div>

      <Folders v-if="child.is_expanded" :album_id="album_id" :children="child.children" />

    </li>
  </ul>

</template>


<script setup lang="ts">

import { inject } from 'vue';
import { invoke } from '@tauri-apps/api';
import Folders from './AlbumsFolders.vue';

// folder icon
import IconRight from '@/assets/chevron-right.svg';

const props = defineProps({
  album_id: {
    type: Number,
    required: true,
  },
  children: {
    type: Array,
    required: false,
  },
});

const g_album_id = inject('g_album_id');   // global album id
const g_folder_id = inject('g_folder_id'); // global folder id


/// click folder to select
const clickFolderName = (folder) => {
  g_album_id.value = props.album_id;
  g_folder_id.value = folder.id;
  
  console.log('clickFolderName...', folder);
};

/// click expand icon to toggle folder expansion
const clickExpandIcon = async (event: Event, folder) => {
  // Prevents clickFolder() from being triggered
  event.stopPropagation(); 

  // toggle folder expansion
  folder.is_expanded = !folder.is_expanded;

  if (folder.is_expanded && !folder.children) {
    try {
      // Fetch subfolder tree
      const subfolders = await invoke('read_folders', { path: folder.path });
      folder.children = subfolders.children;
    } catch (error) {
      console.error('Error fetching subfolders:', error);
    }
  }

  console.log('clickExpand...', folder);
};

</script>


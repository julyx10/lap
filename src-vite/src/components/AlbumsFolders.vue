<template>

  <ul>
    <li v-for="(child, index) in children" :key="index" class="pl-4">
      <div 
        :class="[
          'm-1 flex items-center whitespace-nowrap hover:bg-gray-700', 
          g_child_id === child.id ? 'text-gray-300' : ''
        ]" 
        @click="toggleFolder(index)">
        <IconRight
          :class="[
            'p-1 flex-shrink-0 transition-transform', 
            child.is_expanded && child.children.length > 0 ? 'rotate-90' : ''
          ]"
        />
        {{ child.name }}
      </div>
      <Folders v-if="child.is_expanded" :children="child.children" :album_index="album_index"/>
    </li>
  </ul>

</template>


<script setup lang="ts">

import { ref, inject, onMounted } from 'vue';
import Folders from './AlbumsFolders.vue';

// folder icon
import IconRight from '@/assets/chevron-right.svg';

const props = defineProps({
  children: {
    type: Array,
    required: true
  },
  album_index: {   // optional
    type: Number,
    required: false
  }
});

const g_album_index = inject('g_album_index'); // global album index
const g_child_id = inject('g_child_id'); // global folder id

/// Display the children on mount
onMounted(() => {
  console.log('Children Folder:', props.children);
});


/// Toggle folder expansion
const toggleFolder = (index: number) => {
  console.log('Toggle folder:', props.children[index]);
  props.children[index].is_expanded = !props.children[index].is_expanded;

  g_child_id.value = props.children[index].id;
  g_album_index.value = props.album_index;
};

</script>


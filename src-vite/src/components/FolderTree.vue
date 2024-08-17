<template>
  <div class="pl-4">
    <ul>
      <li v-for="(item, index) in folderTree.children" :key="index">
        <div class="flex items-center hover:bg-gray-700 hover:rounded">
          <img class="p-1 mr-1 w-8 h-8" src="/img64/folder.png" alt="">
          {{ item.name }}
        </div>
        <FolderTree v-if="item.is_dir" :folderTree="item" />
      </li>
    </ul>
  </div>
</template>

<script setup>
import { defineProps } from 'vue';

const props = defineProps({
  folderTree: Object
});
</script>

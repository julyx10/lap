<template>

<div class="flex-1 flex flex-col overflow-auto" style="user-select: none;">
    
    <!-- title bar -->
    <div class="px-2 py-3 h-12 flex items-center justify-between" data-tauri-drag-region>
      <span class="cursor-default" data-tauri-drag-region>{{ titlebar }}</span>

      <div class="flex">
        <IconRefresh class="t-icon-size-sm t-icon-hover" @click="clickReload"/>
      </div>
    </div>
    
    <!-- people -->
    <div class="flex-grow overflow-auto t-scrollbar">
    </div>  

  </div>
  
</template>


<script setup lang="ts">
import { IconRefresh } from '@/common/icons';

const props = defineProps({
  titlebar: {
    type: String,
    required: true
  }
});

function clickReload() {
  console.log('clickReload');
}

</script>
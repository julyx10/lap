<template>

<div class="flex-1 flex flex-col overflow-auto" style="user-select: none;">
    
    <!-- title bar -->
    <div class="px-2 py-3 h-12 flex items-center justify-between" data-tauri-drag-region>
      <span class="cursor-default" data-tauri-drag-region>{{ titlebar }}</span>
      <!-- <TButton :icon="IconRefresh" @click="clickReload"/> -->
    </div>
    
    <!-- people -->
    <div class="grow overflow-auto">
    </div>  

  </div>
  
</template>


<script setup lang="ts">
import { IconRefresh } from '@/common/icons';

import TButton from '@/components/TButton.vue';

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
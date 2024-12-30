<template>

<div class="flex-1 flex flex-col overflow-auto" style="user-select: none;">
    
    <!-- title bar -->
    <div class="px-2 py-3 h-12 flex items-center justify-between">
      <span>{{ titlebar }}</span>

      <div class="flex">
        <IconReload class="t-icon-size-sm t-icon-hover" @click="clickReload"/>
      </div>
    </div>
    
    <!-- people -->
    <div class="flex-grow overflow-auto t-scrollbar">
    </div>  

  </div>
  
</template>


<script setup lang="ts">

import IconReload from '@/assets/reload.svg';

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
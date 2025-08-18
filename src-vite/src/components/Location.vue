<template>

  <div class="w-full flex flex-col select-none">
    
    <!-- title bar -->
    <div class="px-2 py-3 h-12 flex items-center justify-between whitespace-nowrap" data-tauri-drag-region>
      <span class="cursor-default">{{ titlebar }}</span>
    </div>
    
    <!-- location list -->
    <div v-if="allLocations.length > 0" class="flex-grow overflow-x-hidden overflow-y-auto">

      <ul>
        <li v-for="location in allLocations" :key="location.id">
          <div class="my-1 mr-1 h-8 flex items-center rounded border-l-2 border-base-200 hover:bg-base-content/10 whitespace-nowrap cursor-pointer group">
            <span class="mx-1">{{ location.name }}</span>
          </div>
        </li>
      </ul>

    </div>  

    <!-- No Locations Found Message -->
    <div v-else class="mt-10 flex flex-col items-center justify-center text-base-content/30 ">
      <IconLocation class="w-8 h-8" />
      <span class="mt-2 whitespace-nowrap">{{ $t('tooltip.not_found.location') }}</span>
    </div>

  </div>
  
</template>


<script setup lang="ts">

import { ref } from 'vue';
import { IconLocation } from '@/common/icons';

const props = defineProps({
  titlebar: {
    type: String,
    required: true
  }
});

const allLocations = ref<any[]>([]);

</script>
<template>
    
  <div class="flex-1 flex flex-col overflow-auto" style="user-select: none;">

    <!-- title bar -->
    <div class="px-2 py-3 h-12 flex items-center justify-between">
      <span>{{ titlebar }}</span>
      <IconAdd class="t-icon-size-sm t-icon-hover" @click="clickAdd" />
    </div>

    <SelectAlbum ref="selectAlbumRef" 
      v-model:albumId="config.albumId"
      v-model:folderId="config.albumFolderId"
      v-model:folderPath="config.albumFolderPath"
      :componentId="0"
    />
  </div> 

</template>

<script setup lang="ts">
import { useConfigStore } from '@/stores/configStore';

import SelectAlbum from '@/components/SelectAlbum.vue';

// svg icons
import IconAdd from '@/assets/plus.svg';

const props = defineProps({
  titlebar: {
    type: String,
    required: true
  }
});

// config store
const config = useConfigStore();

/// Add albums
const clickAdd = () => {
  selectAlbumRef.value.clickNewAlbum();
};

</script>

<template>
    
  <div class="fixed inset-0 flex items-center justify-center bg-gray-900 bg-opacity-60 z-50 backdrop-blur-0">
    <div class="min-w-96 min-h-96 max-w-[50%] max-h-[80%] p-4 flex flex-col t-color-bg-light rounded-lg shadow-lg overflow-auto t-scrollbar">

      <!-- titlebar -->
      <div class="mb-2 flex items-center justify-between">
        {{ title }}
        <!-- {{ title }} {{ config.destFolderPath? '\'' + config.destFolderPath + '\'' : '' }} -->
        <IconClose class="ml-2 t-icon-size-sm t-icon-hover" @click="clickCancel" />
      </div>

      <!-- message -->
      <div class="mb-2">
        {{ message }}
      </div>

      <!-- select album and folder -->
      <SelectAlbum ref="selectAlbumRef" 
        v-model:albumId="config.destAlbumId"
        v-model:folderId="config.destFolderId"
        v-model:folderPath="config.destFolderPath"
        :componentId="1"
      />

      <!-- action buttons -->
      <div class="mt-4 flex justify-end space-x-4">
        <!-- <button v-if="cancelText.length > 0" 
          :class="[
            'mr-auto px-4 py-1 rounded-full t-color-bg-light text-nowrap',
            config.destAlbumId > 0 ? 't-color-bg-highlight-hover t-icon-hover' : 't-icon-disabled'
          ]"
          @click="clickNewFolder"
        >{{$t('msgbox_new_folder_title')}}</button> -->

        <button v-if="cancelText.length > 0" 
          class="px-4 py-1 rounded-full t-color-bg-light t-color-bg-hover t-icon-hover text-nowrap" 
          @click="clickCancel"
        >{{ cancelText }}</button>

        <button 
          :class="[
            'px-4 py-1 rounded-full t-color-bg-light text-nowrap', 
            config.destAlbumId > 0 ? 't-color-bg-highlight-hover t-icon-hover' : 't-icon-disabled'
          ]" 
          @click="clickOk"
        >{{ OkText }}</button>

      </div>

    </div>

  </div>

</template>

<script setup lang="ts">
import { onMounted, onUnmounted } from 'vue';
import { config } from '@/common/utils';

import { IconClose } from '@/common/icons';
import SelectAlbum from '@/components/SelectAlbum.vue';

const props = defineProps({
  title: {
    type: String,
    required: true
  },
  message: {
    type: String,
    required: true
  },
  OkText: {
    type: String, 
    default: 'OK'
  },
  cancelText: { 
    type: String, 
    default: 'Cancel' 
  },  
});

const emit = defineEmits(['ok', 'cancel']);

onMounted(() => {
  window.addEventListener('keydown', handleKeyDown);
});

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeyDown);
});

function handleKeyDown(event) {
  event.stopPropagation();
  switch (event.key) {
    case 'Enter':
      clickOk();
      break;
    case 'Escape':
      clickCancel();
      break;
    default:
      break;
  }
}

const clickOk = () => {
  emit('ok');
};

const clickCancel = () => {
  emit('cancel');
};

</script>
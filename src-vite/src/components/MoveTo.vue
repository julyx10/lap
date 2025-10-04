<template>
    
  <dialog id="moveToDialog" class="modal">

    <div class="w-[600px] max-h-[80%] p-4 flex flex-col text-base-content/70 bg-base-100 border border-base-content/30 rounded-box">

      <!-- titlebar -->
      <div class="mb-2 flex items-center justify-between text-wrap break-all">
        {{ title }} 
        <!-- {{ title }} {{ config.destFolderPath? '\'' + config.destFolderPath + '\'' : '' }} -->
        <TButton
          :icon="IconClose"
          :buttonSize="'small'"
          @click="clickCancel"
        />
      </div>

      <!-- message -->
      <!-- <div class="mb-2">
        {{ message }}
      </div> -->

      <!-- select album and folder -->
      <div class="border border-base-content/10 pl-1 rounded overflow-auto">
        <AlbumList ref="albumListRef" 
          v-model:albumId="config.destAlbumId"
          v-model:folderId="config.destFolderId"
          v-model:folderPath="config.destFolderPath"
          :componentId="1"
        />
      </div>

      <!-- action buttons -->
      <div class="mt-4 flex justify-end space-x-4">
        <button v-if="cancelText.length > 0" 
          class="px-4 py-1 rounded-lg hover:bg-base-content/30 cursor-pointer" 
          @click="clickCancel"
        >{{ cancelText }}</button>

        <button 
          :class="[
            'px-4 py-1 rounded-lg', 
            config.destAlbumId > 0 ? 'hover:bg-primary cursor-pointer' : 'text-base-content/30 cursor-default'
          ]" 
          @click="clickOk"
        >{{ OkText }}</button>
      </div>

    </div>

  </dialog> 

</template>

<script setup lang="ts">
import { onMounted, onUnmounted } from 'vue';
import { config } from '@/common/utils';
import { listen } from '@tauri-apps/api/event';

import { IconClose } from '@/common/icons';
import AlbumList from '@/components/AlbumList.vue';
import TButton from '@/components/TButton.vue';

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

let unlistenKeydown: () => void;

onMounted(async () => {
  const moveToDialog = document.getElementById('moveToDialog');
  moveToDialog.showModal();

  unlistenKeydown = await listen('global-keydown', handleKeyDown);
});

onUnmounted(() => {
  if (unlistenKeydown) {
    unlistenKeydown();
  }
});

function handleKeyDown(event) {
  const { key } = event.payload;
  switch (key) {
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
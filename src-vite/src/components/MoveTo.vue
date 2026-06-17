<template>
  <ModalDialog :title="title" :width="500" @cancel="clickCancel">
    <!-- select album and folder -->
    <div class="h-[400px] overflow-auto">
      <AlbumList ref="albumListRef" 
        selectionSource="destFolder"
        :showTotalCount="false"
      />
    </div>

    <!-- action buttons -->
    <div class="mt-4 flex justify-end space-x-4">
      <button v-if="cancelText.length > 0" 
        class="t-button-default" 
        @click="clickCancel"
      >{{ cancelText }}</button>

      <button 
        class="t-button-primary" 
        :disabled="(libConfig.destFolder.albumId ?? 0) == 0 || libConfig.destFolder.selected"
        @click="clickOk"
      >{{ OkText }}</button>
    </div>
  </ModalDialog>
</template>

<script setup lang="ts">
import { onMounted, onUnmounted } from 'vue';
import { libConfig } from '@/common/config';
import { listen, type Event } from '@tauri-apps/api/event';
import { useUIStore } from '@/stores/uiStore';

import ModalDialog from '@/components/ModalDialog.vue';
import AlbumList from '@/components/AlbumList.vue';

interface KeyPayload {
  key: string;
}

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
const uiStore = useUIStore();

let unlistenKeydown: () => void;

onMounted(async () => {
  unlistenKeydown = await listen<KeyPayload>('global-keydown', handleKeyDown);
  uiStore.pushInputHandler('MoveTo');
});

onUnmounted(() => {
  if (unlistenKeydown) {
    unlistenKeydown();
  }
  uiStore.removeInputHandler('MoveTo');
});

function handleKeyDown(event: Event<KeyPayload>) {
  if (!uiStore.isInputActive('MoveTo')) return;

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
  if ((libConfig.destFolder.albumId ?? 0) > 0 && !libConfig.destFolder.selected) {
    emit('ok');
  }
};

const clickCancel = () => {
  emit('cancel');
};

</script>

<template>
  <ModalDialog :title="title" :width="440" @cancel="clickCancel">
    <div class="flex flex-col gap-3">
      <div v-if="message" class="text-sm whitespace-pre-line warp-break-words">
        {{ message }}
      </div>

      <div v-if="filePath" class="flex flex-col gap-1 rounded-box bg-base-100/40 px-3 py-2">
        <div class="text-xs text-base-content/50">{{ fileLabel }}</div>
        <div class="text-sm text-base-content break-all">{{ filePath }}</div>
      </div>

      <div class="mt-2 flex justify-end space-x-4">
        <button
          class="px-4 py-1 rounded-box hover:bg-base-100 hover:text-base-content cursor-pointer"
          @click="clickCancel"
        >
          {{ cancelText }}
        </button>

        <button
          class="px-4 py-1 rounded-box hover:bg-base-100 hover:text-base-content cursor-pointer"
          @click="clickSkip"
        >
          {{ skipText }}
        </button>

        <button
          class="px-4 py-1 rounded-box hover:bg-primary hover:text-primary-content cursor-pointer"
          @click="clickContinue"
        >
          {{ continueText }}
        </button>
      </div>
    </div>
  </ModalDialog>
</template>

<script setup lang="ts">
import { onMounted, onUnmounted } from 'vue';
import { useUIStore } from '@/stores/uiStore';
import ModalDialog from '@/components/ModalDialog.vue';

const props = defineProps({
  title: {
    type: String,
    required: true,
  },
  message: {
    type: String,
    default: '',
  },
  fileLabel: {
    type: String,
    default: '',
  },
  filePath: {
    type: String,
    default: '',
  },
  continueText: {
    type: String,
    default: 'Continue',
  },
  skipText: {
    type: String,
    default: 'Skip',
  },
  cancelText: {
    type: String,
    default: 'Cancel',
  },
});

const emit = defineEmits(['continue', 'skip', 'cancel']);
const uiStore = useUIStore();

function handleKeyDown(event: KeyboardEvent) {
  if (!uiStore.isInputActive('IndexRecoveryDialog')) return;

  switch (event.key) {
    case 'Enter':
      clickContinue();
      break;
    case 'Escape':
      clickCancel();
      break;
    default:
      break;
  }
}

function clickContinue() {
  emit('continue');
}

function clickSkip() {
  emit('skip');
}

function clickCancel() {
  emit('cancel');
}

onMounted(() => {
  window.addEventListener('keydown', handleKeyDown);
  uiStore.pushInputHandler('IndexRecoveryDialog');
});

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeyDown);
  uiStore.removeInputHandler('IndexRecoveryDialog');
});
</script>

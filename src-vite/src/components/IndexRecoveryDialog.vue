<template>
  <ModalDialog :title="title" :width="440" @cancel="clickCancel">
    <div class="flex flex-col gap-2 select-none">
      <div v-if="message" class="text-sm wrap-break-word text-base-content/30">
        {{ message }}
      </div>

      <div class="text-sm text-base-content/30">{{ fileLabel }}</div>
      <div v-if="filePath" class="flex items-start gap-2 rounded-box border border-base-content/5 px-3 py-2">
        <div class="min-w-0 flex-1 flex flex-col gap-1">
          <div class="text-sm text-base-content/70 font-medium break-all">{{ fileName }}</div>
          <div class="text-xs text-base-content/30 break-all">{{ filePath }}</div>
        </div>
        <TButton
          :icon="IconExternal"
          buttonSize="small"
          :tooltip="isMac ? $t('menu.file.reveal_in_finder') : $t('menu.file.reveal_in_file_explorer')"
          @click.stop="revealCurrentFile"
        />
      </div>

      <label class="mt-1 flex items-center gap-2 text-sm cursor-pointer">
        <input v-model="skipThisFile" type="checkbox" class="checkbox checkbox-primary checkbox-sm" />
        <span>{{ skipLabel }}</span>
      </label>

      <div class="mt-2 flex justify-end space-x-4">
        <button
          class="t-button-default"
          @click="clickCancel"
        >
          {{ cancelText }}
        </button>

        <button
          class="t-button-primary"
          @click="clickContinue"
        >
          {{ continueText }}
        </button>
      </div>
    </div>
  </ModalDialog>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue';
import { useUIStore } from '@/stores/uiStore';
import { revealPath } from '@/common/api';
import { isMac } from '@/common/utils';
import { IconExternal } from '@/common/icons';
import ModalDialog from '@/components/ModalDialog.vue';
import TButton from '@/components/TButton.vue';

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
  skipLabel: {
    type: String,
    default: 'Skip this file and continue',
  },
  cancelText: {
    type: String,
    default: 'Cancel',
  },
});

const emit = defineEmits(['continue', 'cancel']);
const uiStore = useUIStore();
const skipThisFile = ref(false);
const fileName = computed(() => {
  if (!props.filePath) return '';
  const normalized = props.filePath.replace(/\\/g, '/');
  return normalized.split('/').pop() || props.filePath;
});

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
  emit('continue', skipThisFile.value);
}

function clickCancel() {
  emit('cancel');
}

function revealCurrentFile() {
  if (props.filePath) {
    void revealPath(props.filePath);
  }
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

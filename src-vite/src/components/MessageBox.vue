<template>

  <div class="fixed inset-0 flex items-center justify-center bg-gray-900 bg-opacity-60 z-50 backdrop-blur-0">
    <div class="w-96 p-4 t-color-bg-light rounded-lg shadow-lg">
      <div class="mb-2 flex items-center justify-between">
        {{ title }}
        <IconCancel class="t-icon-size-sm t-icon-hover" @click="clickCancel" />
      </div>
      <div class="my-4">
        {{ message }}
        <input v-if="showInput"
          ref="inputRef"
          v-model="inputValue"
          type="text"
          maxlength="255"
          class="px-2 py-1 my-2 w-full border rounded-md t-input-color-bg t-color-border t-input-focus"
          @input="validateInput"
          @keydown.enter="clickOk"
        />
        <p v-if="errorMessage.length > 0" class="text-red-600 text-xs">{{ errorMessage }}</p>
      </div>

      <div class="flex justify-end space-x-4">
        <button v-if="cancelText.length > 0" 
          class="px-4 py-1 rounded-full t-color-bg-light t-color-bg-hover t-icon-hover" 
          @click="clickCancel"
        >
          {{ cancelText }}
        </button>
        <button 
          :class="['px-4 py-1 rounded-full t-color-bg-light', 
            okButtonClasses
          ]" 
          @click="clickOk"
        >
          {{ OkText }}
        </button>
      </div>

    </div>
  </div>

</template>

<script setup lang="ts">

import { ref, computed, onMounted, onUnmounted } from 'vue';
import { isValidFileName } from '@/common/utils';
import { useI18n } from 'vue-i18n';

import IconCancel from '@/assets/close.svg';

const props = defineProps({
  title: { 
    type: String, 
    required: true
  },
  message: { 
    type: String, 
    required: true 
  },
  showInput: {      // show input text box or not
    type: Boolean, 
    default: false 
  },
  inputText: { 
    type: String, 
    default: '' 
  },
  OkText: { 
    type: String, 
    default: 'OK'
  },
  cancelText: { 
    type: String, 
    default: 'Cancel' 
  },
  warningOk: {        // show warning color for OK button
    type: Boolean, 
    default: false 
  },
});

const { locale, messages } = useI18n();
const localeMsg = computed(() => messages.value[locale.value]);

const emit = defineEmits(['ok', 'cancel']);

// input 
const inputRef = ref(null);
const inputValue = ref(props.inputText);
const errorMessage = ref('');

const okButtonClasses = computed(() => {
  return !props.showInput || inputValue.value.trim().length > 0
    ? (props.warningOk ? 't-color-bg-warning-hover t-icon-hover' : 't-color-bg-highlight-hover t-icon-hover')
    : 't-icon-disabled';
});

onMounted(() => {
  window.addEventListener('keydown', handleKeyDown);
  // window.addEventListener('keyup', handleKeyDown);
  if(inputRef.value) {
    inputRef.value.focus();
  }
});

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeyDown);
  // window.removeEventListener('keyup', handleKeyDown);
});


const validateInput = () => {
  if (!isValidFileName(inputValue.value)) {
    errorMessage.value = localeMsg.value.msgbox_input_filename_error;
  } else {
    errorMessage.value = '';
  }
};

function handleKeyDown(event) {
  event.stopPropagation();
  // event.preventDefault();

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
  if(props.showInput) {
    if (inputValue.value.trim().length > 0 && !errorMessage.value) {
      emit('ok', inputValue.value.trim());
    } else {
      return;
    }
  } else {
    emit('ok');
  }
};

const clickCancel = () => {
  emit('cancel');
};

</script>

<style scoped>
</style>

<template>

  <dialog id="msg_box_dlg" class="modal">
    <div class="modal-box">
      <div class="mb-2 flex items-center justify-between">
        {{ title }}
        <IconClose class="t-icon-size-sm t-icon-hover" @click="clickCancel" />
      </div>
      <div class="mt-4">
        {{ message }}
        <input v-if="showInput && !multiLine"
          ref="inputRef"
          v-model="inputValue"
          type="text"
          maxlength="255"
          class="px-2 py-1 my-2 w-full border rounded-md t-input-color-bg t-color-border t-input-focus"
          @input="validateInput"
          @keydown.enter="clickOk"
        />
        <textarea v-if="showInput && multiLine"
          ref="inputRef"
          v-model="inputValue"
          rows="4"
          minrows="1"
          class="px-2 py-1 my-2 w-full border rounded-md t-input-color-bg t-color-border t-input-focus t-scrollbar-dark resize-y min-h-[30px] max-h-[200px]"
          @input="validateInput"
          @keydown.enter="clickOk"
        ></textarea>
        <p class="h-4 text-red-600 text-xs">{{ inputErrorMessage }}</p>
      </div>

      <div class="modal-action">
        <form method="dialog">
          <button v-if="cancelText.length > 0" 
            class="btn" 
            @click="clickCancel"
          >
            {{ cancelText }}
          </button>
          <button 
            :class="['btn', 
              okButtonClasses
            ]" 
            @click="clickOk"
          >
            {{ OkText }}
          </button>
        </form>
      </div>

    </div>
  </dialog>

</template>

<script setup lang="ts">

import { ref, watch, computed, onMounted, onUnmounted } from 'vue';
import { isValidFileName } from '@/common/utils';
import { useI18n } from 'vue-i18n';
import { IconClose } from '@/common/icons';

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
  needValidateInput: { // validate input text
    type: Boolean, 
    default: false 
  },
  multiLine: { 
    type: Boolean, 
    default: false 
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
  errorMessage: { 
    type: String, 
    default: '' 
  }
});

/// i18n
const { locale, messages } = useI18n();
const localeMsg = computed(() => messages.value[locale.value]);

const emit = defineEmits(['ok', 'cancel', 'reset']);

// input 
const inputRef = ref(null);
const inputValue = ref(props.inputText);
const needValidateInput = ref(props.needValidateInput);
const inputErrorMessage = ref('');

const okButtonClasses = computed(() => {
  return !props.showInput || !props.needValidateInput || inputValue.value.trim().length > 0
    ? (props.warningOk ? 'btn-error' : 'btn-primary')
    : 'btn-disabled';
});

onMounted(() => {
  window.addEventListener('keydown', handleKeyDown);
  inputRef.value?.focus();
});

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeyDown);
});

watch(() => props.errorMessage, (newValue) => {
  if (newValue.length > 0) {
    inputErrorMessage.value = newValue;
    emit('reset'); // reset error message
  }
});

const validateInput = () => {
  if ( needValidateInput.value && !isValidFileName(inputValue.value)) {
    inputErrorMessage.value = localeMsg.value.msgbox_input_filename_error;
  } else {
    inputErrorMessage.value = '';
  }
};

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
  if(props.showInput) {
    if ( !props.needValidateInput || inputValue.value.trim().length > 0 && !inputErrorMessage.value) {
      emit('ok', inputValue.value);
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

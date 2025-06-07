<template>

  <dialog id="messageBoxDialog" class="modal">
    <div class="w-96 p-4 text-base-content/70 bg-base-100 border border-base-content/30 rounded-box">

      <!-- title bar -->
      <div class="mb-2 flex items-center justify-between">
        {{ title }}
        <TButton
          :icon="IconClose"
          :buttonSize="'small'"
          @click="clickCancel"
        />
      </div>

      <!-- content -->
      <div class="mt-4">
        {{ message }}
        <input v-if="showInput && !multiLine"
          ref="inputRef"
          v-model="inputValue"
          type="text"
          maxlength="255"
          class="px-2 py-1 my-2 w-full input"
          @input="validateInput"
          @keydown.enter="clickOk"
        />
        <textarea v-if="showInput && multiLine"
          ref="inputRef"
          v-model="inputValue"
          rows="4"
          minrows="1"
          class="px-2 py-1 my-2 w-full textarea min-h-[30px] max-h-[200px]"
          @input="validateInput"
          @keydown.enter="clickOk"
        ></textarea>
        <p class="h-4 text-error text-xs">{{ inputErrorMessage }}</p>
      </div>

      <!-- cancel and OK buttons -->
      <div class="mt-2 flex justify-end space-x-4">
        <button v-if="cancelText.length > 0"
          class="px-4 py-1 rounded-lg hover:bg-base-content/30 cursor-pointer" 
          @click="clickCancel"
        >{{ cancelText }}</button>
        
        <button 
          :class="[
            'px-4 py-1 rounded-lg', 
            okButtonClasses,
          ]" 
          @click="clickOk"
        >{{ OkText }}</button>
      </div>

    </div>
  </dialog>

</template>

<script setup lang="ts">

import { ref, watch, computed, onMounted, onUnmounted } from 'vue';
import { isValidFileName } from '@/common/utils';
import { useI18n } from 'vue-i18n';
import { IconClose } from '@/common/icons';

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
    ? (props.warningOk ? 'hover:bg-error cursor-pointer' : 'hover:bg-primary cursor-pointer')
    : 'text-base-content/30 cursor-default';
});

onMounted(() => {
  messageBoxDialog.showModal();

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
    inputErrorMessage.value = localeMsg.value.msgbox.input.file_name_invalid;
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

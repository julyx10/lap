<template>
  <ModalDialog :title="title" :width="400" @cancel="clickCancel">
    <div v-if="message" class="text-sm whitespace-pre-line wrap-break-word select-none">
      {{ message }}
    </div>

    <label
      v-if="checkboxText"
      class="mt-3 flex items-start gap-2 text-sm cursor-pointer select-none"
    >
      <input
        v-model="checkboxValue"
        type="checkbox"
        class="checkbox checkbox-xs checkbox-error mt-0.5"
        @change="emit('checkbox-change', checkboxValue)"
      />
      <span class="leading-5">{{ checkboxText }}</span>
    </label>

    <div class="flex flex-row items-center text-sm">
      <input v-if="showInput && !multiLine"
        ref="inputRef"
        v-model="inputValue"
        type="text"
        maxlength="255"
        :placeholder="inputPlaceholder"
        class="px-2 py-1 flex-1 min-w-0 input"
        @input="validateInput"
      />
      <span v-if="inputExtension" class="label px-2 text-sm">.{{ inputExtension }}</span>
    </div>

    <textarea v-if="showInput && multiLine"
      ref="inputRef"
      v-model="inputValue"
      rows="4"
      minrows="1"
      :placeholder="inputPlaceholder"
      class="px-2 py-1 w-full textarea min-h-[30px] max-h-[200px]"
      @input="validateInput"
    ></textarea>

    <p class="h-4 text-error text-xs">{{ inputErrorMessage }}</p>

    <!-- buttons -->
    <div class="mt-2 flex justify-end space-x-4">
      <button v-if="thirdText.length > 0"
        :class="['px-4 py-1 rounded-box cursor-pointer', warningThird ? 'hover:bg-error hover:text-base-100' : 'hover:bg-base-100 hover:text-base-content']" 
        @click="clickThird"
      >{{ thirdText }}</button>

      <button v-if="cancelText.length > 0"
        class="t-button-default" 
        @click="clickCancel"
      >{{ cancelText }}</button>
      
      <button 
        :class="warningOk ? 't-button-error' : 't-button-primary'" 
        :disabled="showInput && !isInputValid"
        @click="clickOk"
      >{{ OkText }}</button>
    </div>
  </ModalDialog>
</template>

<script setup lang="ts">
import { ref, watch, computed, onMounted, onUnmounted } from 'vue';
import { isValidFileName } from '@/common/utils';
import { useI18n } from 'vue-i18n';
import { useUIStore } from '@/stores/uiStore';
import ModalDialog from '@/components/ModalDialog.vue';

const props = defineProps({
  title: { 
    type: String, 
    required: true
  },
  message: { 
    type: String, 
    required: false
  },
  showInput: {      // show input text box or not
    type: Boolean, 
    default: false 
  },
  inputText: { 
    type: String, 
    default: '' 
  },
  inputPlaceholder: {
    type: String, 
    default: '' 
  },
  inputExtension: {
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
  },
  thirdText: {
    type: String,
    default: ''
  },
  warningThird: {
    type: Boolean,
    default: false
  },
  checkboxText: {
    type: String,
    default: ''
  },
  checkboxChecked: {
    type: Boolean,
    default: false
  }
});

/// i18n
const { locale, messages } = useI18n();
const localeMsg = computed(() => messages.value[locale.value] as any);
const uiStore = useUIStore();

const emit = defineEmits(['ok', 'cancel', 'reset', 'third', 'checkbox-change']);

// input 
const inputRef = ref<HTMLInputElement | null>(null);
const inputValue = ref(props.inputText);
const needValidateInput = ref(props.needValidateInput);
const inputErrorMessage = ref('');
const checkboxValue = ref(props.checkboxChecked);

const isInputValid = computed(() =>
  !needValidateInput.value || (inputValue.value.trim().length > 0 && isValidFileName(inputValue.value))
);

onMounted(async () => {
  window.addEventListener('keydown', handleKeyDown);
  uiStore.pushInputHandler('MessageBox');

  if(props.showInput) { 
    setTimeout(() => {
      inputRef.value?.focus();
    }, 50); // 50ms delay
  }
});

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeyDown);
  uiStore.removeInputHandler('MessageBox');
});

watch(() => props.errorMessage, (newValue) => {
  if (newValue.length > 0) {
    inputErrorMessage.value = newValue;
    emit('reset'); // reset error message
  }
});

watch(() => props.checkboxChecked, (newValue) => {
  checkboxValue.value = newValue;
});

const validateInput = () => {
  if ( needValidateInput.value && !isValidFileName(inputValue.value)) {
    inputErrorMessage.value = localeMsg.value.msgbox.input.file_name_invalid;
  } else {
    inputErrorMessage.value = '';
  }
};

function handleKeyDown(event: KeyboardEvent) {
  if (!uiStore.isInputActive('MessageBox')) return;

  const { key } = event;
  const activeElement = document.activeElement;
  const isInputOrTextarea = activeElement?.tagName === 'INPUT' || activeElement?.tagName === 'TEXTAREA';

  switch (key) {
    case 'Enter':
      // Don't trigger OK if user is in an input or textarea
      if (!isInputOrTextarea) {
        event.preventDefault();
        clickOk();
      }
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

const clickThird = () => {
  emit('third');
};
</script>

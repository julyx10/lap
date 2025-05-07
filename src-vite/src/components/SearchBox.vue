<template>
  <div 
    class="group relative shrink-0 transition-all duration-300 overflow-hidden"
    :class="[isFocused || inputValue.length > 0 ? 'w-48' : 'w-10']"
  >
    <input 
      tabindex="-1"
      ref="searchInput"
      type="text"
      v-model="inputValue"
      :placeholder="isFocused ? $t('search_placeholder') : ''"
      :class="[
        'py-1 w-full text-sm border rounded-md t-input-color-bg t-color-border t-input-focus transition-colors duration-300',
        isFocused || inputValue.length > 0 ? 'px-8' : 'px-2 cursor-pointer t-color-border-group-hover '
      ]"
      @focus="isFocused = true"
      @blur="handleBlur"
      @input="handleInput"
    />

    <!-- Search Icon (Inside input when focused) -->
    <IconSearch
      class="absolute left-2 top-1/2 transform -translate-y-1/2 t-icon-size-sm t-icon-group-hover"
      @click="focusInput"
    />

    <!-- Cancel Icon (Only show when there's input) -->
    <IconClose v-if="inputValue.length > 0" 
      class="absolute right-2 top-1/2 transform -translate-y-1/2 t-icon-size-sm t-icon-group-hover"
      @click="clickCancel"
    />
  </div>

</template>


<script setup lang="ts">
import { ref, watch, onMounted, onUnmounted } from 'vue';
import { IconClose, IconSearch } from '@/common/icons';

const props = defineProps({
  modelValue: {
    type: String,
    required: true,
  },
});

const emit = defineEmits(['update:modelValue']);

// input value
const inputValue = ref('');
const isFocused = ref(false);
const searchInput = ref(null);

onMounted(() => {
  window.addEventListener('keydown', handleKeyDown);
});

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeyDown);
});

watch(() => props.modelValue, (newValue) => { 
  inputValue.value = newValue; 
}, { immediate: true });

function handleKeyDown(event) {
  switch (event.key) {
    case 'Enter':
      event.preventDefault();
      clickSearch();
      break;
    case 'Escape':
      event.preventDefault();
      if (isFocused.value) {
        clickCancel();
      }
      break;
    default:
      break;
  }
}

const focusInput = () => {
  if(!isFocused.value) {
    isFocused.value = true;
    searchInput.value?.focus();
  }
};

const handleBlur = () => {
  if (inputValue.value.length === 0) {
    isFocused.value = false;
  }
};

function handleInput(event) {
  // if (inputValue.value.length === 0)
  //   emit('update:modelValue', inputValue.value);
};

const clickCancel = () => {
  inputValue.value = '';
  isFocused.value = false;
  searchInput.value?.blur();
  emit('update:modelValue', '');
};

const clickSearch = () => {
  isFocused.value = false;
  searchInput.value?.blur();
  emit('update:modelValue', inputValue.value.trim());
};

defineExpose({ 
  focusInput
});

</script>
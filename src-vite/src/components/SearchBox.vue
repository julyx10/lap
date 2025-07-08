<template>
  <div 
    class="group relative shrink-0 overflow-hidden group"
    :class="[isFocused || searchValue.length > 0 ? 'w-48' : 'w-9']"
  >
    <input 
      tabindex="-1"
      ref="searchInputRef"
      type="text"
      v-model="inputValue"
      :placeholder="searchValue.length > 0 ? searchValue :  (isFocused ? $t('toolbar.search.placeholder') : '')"
      :class="[
        'py-1 h-8 w-full text-sm input bg-transparent transition-colors duration-300',
        isFocused || searchValue.length > 0 ? 'px-8 focus:border' : 'px-2 group-hover:border-base-content/70 cursor-pointer'
      ]"
      @focus="isFocused = true"
      @blur="handleBlur"
      @input="handleInput"
    />

    <!-- Search Icon (Inside input when focused) -->
    <IconSearch
      class="absolute left-2 top-1/2 transform -translate-y-1/2 w-4 h-4 group-hover:text-base-content/70 cursor-pointer"
      @click="focusInput"
    />
    <!-- Cancel Icon (Only show when there's input) -->
    <IconClose v-if="searchValue.length > 0" 
      class="absolute right-2 top-1/2 transform -translate-y-1/2 w-4 h-4 hover:text-base-content cursor-pointer z-10 transition-all duration-200"
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
const searchValue = ref(props.modelValue);
const searchInputRef = ref(null);

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
    searchInputRef.value?.focus();
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
  updateSearch(inputValue.value);
};

const clickSearch = () => {
  updateSearch(inputValue.value.trim());
};

const updateSearch = (value) => {
  emit('update:modelValue', value);
  searchValue.value = value;

  searchInputRef.value?.blur();
  isFocused.value = false;
};

defineExpose({ 
  focusInput
});

</script>
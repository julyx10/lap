<template>

  <div >
    <!-- Search Box -->
    <input
      type="text"
      class="px-2 py-1 w-full text-sm border rounded-md t-input-color-bg t-color-border t-input-focus"
      v-model="inputValue"
      :placeholder="$t('search_placeholder')"
      @input="handleInput"
    />
    <!-- Search Icon -->
    <IconCancel v-if="inputValue.length > 0" 
      class="absolute right-8 top-1/2 t-icon-size-sm t-icon-hover transform -translate-y-1/2 " 
      @click="clickCancel" 
    />
    <IconSearch 
      :class="[
        'absolute right-2 top-1/2 t-icon-size-sm transform -translate-y-1/2',
        inputValue.length > 0 ? 't-icon-hover' : 't-icon-disabled'
      ]" 
      @click="clickSearch" 
    />
  </div>

</template>


<script setup lang="ts">
import { ref, watch, onMounted, onUnmounted } from 'vue';

import IconCancel from '@/assets/close.svg';
import IconSearch from '@/assets/search.svg';

const props = defineProps({
  modelValue: {
    type: String,
    required: true,
  },
});

const emit = defineEmits(['update:modelValue']);

// input value
const inputValue = ref(props.modelValue);

onMounted(() => {
  window.addEventListener('keydown', handleKeyDown);
});


onUnmounted(() => {
  window.removeEventListener('keydown', handleKeyDown);
});

// Watch for changes in the modelValue from the parent component
watch(() => props.modelValue, (newValue) => { 
  inputValue.value = newValue; 
});

function handleKeyDown(event) {
  switch (event.key) {
    case 'Enter':
      event.preventDefault();
      clickSearch();
      break;
    default:
      break;
  }
}

function handleInput(event) {
  if (inputValue.value.length === 0)
    emit('update:modelValue', inputValue.value);
};

const clickCancel = () => {
  console.log('clickCancel');
  
  inputValue.value = '';
  emit('update:modelValue', '');
};

const clickSearch = () => {
  console.log('clickSearch');

  if (inputValue.value.length === 0) return;
  emit('update:modelValue', inputValue.value.trim());
};

</script>
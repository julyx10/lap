<template>

  <div >
    <!-- Search Box -->
    <input
      type="text"
      class="px-2 py-1 w-full text-sm border rounded-md t-input-color-bg t-color-border t-input-focus"
      v-model="inputValue"
      :placeholder="$t('search_placeholder')"
      @input="updateValue"
      @mousedown.stop
    />
    <!-- Search Icon -->
    <IconSearch class="absolute right-2 top-1/2 transform -translate-y-1/2 h-5 w-5" @click="clickSearch" />
  </div>

</template>


<script setup lang="ts">
import { ref, watch } from 'vue';

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

// Watch for changes in the modelValue from the parent component
watch(() => props.modelValue, (newValue) => { 
  inputValue.value = newValue; 
});

// Update slider value
const updateValue = () => {
  emit('update:modelValue', inputValue.value);
};

const clickSearch = () => {
  console.log('clickSearch');
};

</script>
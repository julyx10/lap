<template>
    <div class="relative inline-block text-left">
      <!-- Dropdown Trigger -->
      <button
        @click="toggleDropdown"
        class="px-2 py-1 w-full inline-flex justify-center rounded-md border t-color-border t-icon-hover t-color-border-hover text-sm "
      >
        <span class="px-1 "> {{ $t('file_list_sorting') }}: {{ selectedLabel }}</span>
        <IconArrowDown 
          class="t-icon-size-sm t-icon-hover" 
          :style="{ transform: `rotate(${(isDropDown ? 180 : 0)}deg`, transition: 'transform 0.3s ease-in-out' }" 
        />
      </button>
  
      <!-- Dropdown Menu -->
      <div
        v-if="isDropDown"
        class="absolute right-0 mt-1 min-w-32 rounded-md shadow-lg t-color-bg-light border t-color-border z-10"
      >
        <button
          v-for="option in options"
          :key="option.value"
          @click="selectOption(option)"
          class="p-1 w-full flex flex-row space-x-1 text-sm t-color-bg-hover whitespace-nowrap"
        >
          <IconDot class="t-icon-size-sm t-icon-hover" /> 
          <span>{{ option.label }}</span>
        </button>
      </div>
    </div>
  </template>
  
<script setup>
import { ref, watch, defineEmits } from 'vue';

import IconArrowDown from '@/assets/arrow-down.svg';
import IconDot from '@/assets/dot.svg';
import IconCheck from '@/assets/check.svg';

  // Props
const props = defineProps({
    options: {
      type: Array,
      required: true,
      default: () => [],
    },
    defaultLabel: {
      type: String,
      default: 'Select an option',
    },
  });
  
  // Emits
  const emit = defineEmits(['select']);
  
  // State
  const isDropDown = ref(false);
  const selectedLabel = ref(props.defaultLabel);
  
  // Methods
  const toggleDropdown = () => {
    isDropDown.value = !isDropDown.value;
  };
  
  const selectOption = (option) => {
    selectedLabel.value = option.label;
    emit('select', option);
    isDropDown.value = false;
  };
  </script>
  
  <style scoped>
  </style>
  
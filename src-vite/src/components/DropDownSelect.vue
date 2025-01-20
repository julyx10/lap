<template>
  <div ref="dropdown" class="relative inline-block text-left">

    <!-- Dropdown Trigger -->
    <button
      class="px-2 py-1 w-full inline-flex justify-center rounded-md border t-color-border t-icon-hover t-color-border-hover text-sm "
      @click="toggleDropdown"
    >
      <span class="pl-1 pr-2">{{ options[optionIndex].label }} {{ extendOptions.length > 0 ? ' | ' + extendOptions[extendIndex].label : '' }}</span>
      <IconArrowDown class="t-icon-size-sm t-icon-hover" />
    </button>

    <!-- Dropdown Menu -->
    <transition name="fade">
      <div v-if="isDropDown"
        class="absolute right-0 my-1 rounded-md shadow-lg t-color-bg-light border t-color-border z-50"
      >
        <!-- menu group 1 -->
        <button v-for="(option, index) in options"
          class="pl-1 pr-4 py-1 w-full flex flex-row space-x-1 t-color-bg-hover t-color-text-hover text-sm whitespace-nowrap"
          :key="index"
          @click="selectOption(index)"
        >
          <IconDot v-if="optionIndex === index" class="t-icon-size-sm t-icon-hover" /> 
          <span v-else class="t-icon-size-sm"></span>
          <span>{{ option.label }}</span>
        </button>

        <!-- menu group 2 -->
        <button v-for="(option, index) in extendOptions"
          :class="[
            'pl-1 pr-4 py-1 w-full flex flex-row space-x-1 t-color-bg-hover t-color-text-hover text-sm whitespace-nowrap', 
            index === 0 ? 'border-t t-color-border' : ''
          ]"
          :key="index"
          @click="selectExtendOption(index)"
        >
          <IconDot v-if="extendIndex === index" class="t-icon-size-sm t-icon-hover" /> 
          <span v-else class="t-icon-size-sm"></span>
          <span>{{ option.label }}</span>
        </button>

      </div>
    </transition> 

  </div>
  
</template>
  
<script setup>
import { ref, onMounted, onBeforeUnmount } from 'vue';

import IconArrowDown from '@/assets/arrow-down.svg';
import IconDot from '@/assets/dot.svg';

  // Props
const props = defineProps({
  options: {
    type: Array,
    required: true,
  },
  defaultIndex: {
    type: Number,
    default: 0,
  },
  extendOptions: {
    type: Array,
    default: () => [],
  },
  defaultExtendIndex: {
    type: Number,
    default: 0,
  },
});

// Emits
const emit = defineEmits(['select']);

// State
const dropdown = ref(null);
const isDropDown = ref(false);
const optionIndex = ref(props.defaultIndex);
const extendIndex = ref(props.defaultExtendIndex);

// Add event listener when the component is mounted
onMounted(() => {
  document.addEventListener('click', handleClickOutside);
});

// Remove event listener when the component is destroyed
onBeforeUnmount(() => {
  document.removeEventListener('click', handleClickOutside);
});

// Methods
const toggleDropdown = () => {
  isDropDown.value = !isDropDown.value;
};

// Close dropdown when clicking outside
const handleClickOutside = (event) => {
  console.log('click outside');
  if (dropdown.value && !dropdown.value.contains(event.target)) {
    isDropDown.value = false;
  }
};

const selectOption = (index) => {
  optionIndex.value = index;
  emit('select', optionIndex.value, extendIndex.value);
  isDropDown.value = false;
};

const selectExtendOption = (index) => { 
  extendIndex.value = index;
  emit('select', optionIndex.value, extendIndex.value);
  isDropDown.value = false;
};

</script>

<style scoped>

/* fade transition */
.fade-enter-active, .fade-leave-active {
  transition: opacity 0.3s ease;
}
.fade-enter-from, .fade-leave-to {
  opacity: 0;
}
.fade-enter-to, .fade-leave-from {
  opacity: 1;
}

</style>
  
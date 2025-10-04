<template>
  <div ref="dropdown" class="relative inline-block text-left">

    <!-- Dropdown Trigger -->
    <button tabindex="-1"
      class="px-2 py-1 w-full h-8 flex items-center rounded-md border border-base-content/30 hover:border-base-content/70 transition-colors duration-300 cursor-pointer text-sm whitespace-nowrap "
      text=""
      @click="toggleDropdown"
    >
      {{ options[optionIndex].label }}{{ extendOptions.length > 0 ? (extendIndex == 0 ? '↑' : '↓') : '' }}
      <TButton
        :icon="IconArrowDown"
        :buttonSize="'small'"
      />
    </button>

    <!-- Dropdown Menu -->
    <div v-if="isDropDown"
      class="menu mt-1 text-base-content/70 bg-base-100 border border-base-content/30 absolute rounded-box shadow-lg z-50"
    >
      <!-- menu group 1 -->
      <button v-for="(option, index) in options"
        class="p-1 flex flex-row hover:bg-base-content/10 hover:rounded cursor-pointer text-sm whitespace-nowrap "
        :key="index"
        @click="selectOption(index)"
      >
        <div class="w-5">
          <IconDot v-if="optionIndex === index" class="w-5" /> 
        </div>
        <span class="mr-4">{{ option.label }}</span>
      </button>

      <!-- seperator -->
      <div v-if="extendOptions.length > 0 " class="mx-2 my-1 border-t border-base-content/10"></div>

      <!-- menu group 2 -->
      <button v-for="(option, index) in extendOptions"
        class="p-1 flex flex-row hover:bg-base-content/10 hover:rounded cursor-pointer text-sm whitespace-nowrap "
        :key="index"
        @click="selectExtendOption(index)"
      >
        <IconDot v-if="extendIndex === index" class="w-5" /> 
        <span v-else class="w-5"></span>
        <span>{{ option.label }}</span>
      </button>

    </div>

  </div>
  
</template>
  
<script setup>
import { ref, onMounted, onBeforeUnmount } from 'vue';
import { IconArrowDown, IconDot } from '@/common/icons';

import TButton from '@/components/TButton.vue';

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

onMounted(() => {
  document.addEventListener('mousedown', handleClickOutside, { capture: true });
});

onBeforeUnmount(() => {
  document.removeEventListener('mousedown', handleClickOutside, { capture: true });
});

const toggleDropdown = () => {
  isDropDown.value = !isDropDown.value;
};

// Close dropdown when clicking outside
const handleClickOutside = (event) => {
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
  
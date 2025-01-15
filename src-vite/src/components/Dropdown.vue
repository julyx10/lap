<template>
  <div class="relative inline-block text-left">

    <!-- Dropdown Trigger -->
    <button
      @click="toggleDropdown"
      class="px-2 py-1 w-full inline-flex justify-center rounded-md border t-color-border t-icon-hover t-color-border-hover text-sm "
    >
      <span class="pl-1 pr-2"> {{ $t('file_list_sorting') }}: {{ options[optionIndex].label }}</span>
      <IconArrowDown 
        class="t-icon-size-sm t-icon-hover" 
      />
    </button>

    <!-- Dropdown Menu -->
    <transition name="fade">
      <div
        v-if="isDropDown"
        class="absolute right-0 my-1 min-w-32 rounded-md shadow-lg t-color-bg-light border t-color-border z-50"
      >
        <!-- menu group 1 -->
        <button v-for="(option, index) in options"
          :class="[
            'pl-1 pr-4 py-1 w-full flex flex-row space-x-1 t-color-bg-hover text-sm whitespace-nowrap', 
          ]"
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
            'pl-1 pr-4 py-1 w-full flex flex-row space-x-1 t-color-bg-hover text-sm whitespace-nowrap', 
            index === 0 ? 'border-t t-color-border' : ''
          ]"
          :key="index"
          @click="selectOption(index)"
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
import { ref } from 'vue';

import IconArrowDown from '@/assets/arrow-down.svg';
import IconDot from '@/assets/dot.svg';
// import IconCheck from '@/assets/check.svg';

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
    default: false,
  },
  defaultExtendIndex: {
    type: Number,
    default: 0,
  },
});

// Emits
const emit = defineEmits(['dropdown']);

// State
const isDropDown = ref(false);
const optionIndex = ref(props.defaultIndex);
const extendIndex = ref(props.defaultExtendIndex);

// Methods
const toggleDropdown = () => {
  isDropDown.value = !isDropDown.value;
};

const selectOption = (index) => {
  if(index < props.options.length) {
    optionIndex.value = index;
    emit('dropdown', props.options[optionIndex.value]);
  } else {
    extendIndex.value = index - props.options.length;
    emit('dropdown', props.extendOptions[extendIndex.value]);
  }
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
  
<template>
  <div ref="dropdown" class="relative inline-block text-left">

    <!-- Dropdown Trigger -->
    <button tabindex="-1"
      class="px-2 py-1 w-full h-8 flex items-center rounded-box border border-base-content/30 hover:bg-base-100 hover:text-base-content transition-colors duration-300 cursor-pointer text-sm whitespace-nowrap "
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
    <teleport to="body">
      <transition name="fade">
        <div v-if="isDropDown"
          ref="menu"
          class="menu mt-1 text-base-content/70 bg-base-200/80 backdrop-blur-sm border border-base-content/30 absolute rounded-box shadow-lg z-[500]"
          :style="menuStyle"
        >
          <!-- menu group 1 -->
          <button v-for="(option, index) in options"
            class="p-1 flex flex-row hover:bg-base-100 hover:text-base-content hover:rounded-box cursor-pointer text-sm whitespace-nowrap "
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
            class="p-1 flex flex-row hover:bg-base-100 hover:text-base-content hover:rounded-box cursor-pointer text-sm whitespace-nowrap "
            :key="index"
            @click="selectExtendOption(index)"
          >
            <IconDot v-if="extendIndex === index" class="w-5" /> 
            <span v-else class="w-5"></span>
            <span>{{ option.label }}</span>
          </button>

        </div>
      </transition>
    </teleport>

  </div>
  
</template>
  
<script setup>
import { ref, onMounted, onBeforeUnmount, nextTick } from 'vue';
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
const menu = ref(null);
const isDropDown = ref(false);
const menuStyle = ref({});
const optionIndex = ref(props.defaultIndex);
const extendIndex = ref(props.defaultExtendIndex);

onMounted(() => {
  document.addEventListener('mousedown', handleClickOutside, { capture: true });
  document.addEventListener('keydown', handleKeyDown);
});

onBeforeUnmount(() => {
  document.removeEventListener('mousedown', handleClickOutside, { capture: true });
  document.removeEventListener('keydown', handleKeyDown);
});

// Handle Escape key press
const handleKeyDown = (event) => {
  if (isDropDown.value && event.key === 'Escape') {
    isDropDown.value = false;
  }
};

const toggleDropdown = async () => {
  isDropDown.value = !isDropDown.value;

  if (isDropDown.value) {
    await nextTick(); // Ensure menu is rendered before measuring

    const rect = dropdown.value.getBoundingClientRect();
    const menuRect = menu.value.getBoundingClientRect();
    
    const padding = 8; 
    const menuWidth = menuRect.width;
    const menuHeight = menuRect.height;
    const winWidth = window.innerWidth;
    const winHeight = window.innerHeight;
    const scrollY = window.scrollY;

    let top = rect.bottom + scrollY;
    let left = rect.left + window.scrollX;

    // Check bottom boundary
    if (top + menuHeight > winHeight + scrollY - padding) {
      // Not enough space below, try to place above
      top = rect.top - menuHeight + scrollY;
    }

    // Check top boundary (after potentially flipping)
    if (top < scrollY + padding) {
      // Still not enough space (menu is too tall), align to top
      top = scrollY + padding;
    }

    // Check right boundary
    if (left + menuWidth > winWidth - padding) {
      // Align to the right edge
      left = winWidth - menuWidth - padding;
    }

    // Check left boundary
    if (left < padding) {
      // Align to the left edge
      left = padding;
    }

    menuStyle.value = { top: `${top}px`, left: `${left}px` };
  }
};

// Close dropdown when clicking outside
const handleClickOutside = (event) => {
  if (dropdown.value && !dropdown.value.contains(event.target) && menu.value && !menu.value.contains(event.target)) {
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

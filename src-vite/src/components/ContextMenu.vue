<template>
  <div ref="dropdown" class="relative inline-block text-left">

    <!-- Dropdown Trigger -->
    <slot name="trigger" :toggle="toggleDropdown">
      <!-- Default trigger button when no slot provided -->
      <TButton
        :icon="iconMenu"
        :buttonSize="smallIcon ? 'small' : 'medium'"
        :disabled="disabled"
        @click="toggleDropdown" 
        @dblclick.stop
      />
    </slot>
    <!-- Dropdown Menu -->
    <teleport to="body">
      <transition name="fade">
        <div v-if="isDropDown" 
          ref="menu"
          class="menu text-base-content/70 bg-base-200/80 backdrop-blur-md border border-base-content/30 absolute rounded-box shadow-lg z-500"
          :style="menuStyle"
        >
          <!-- menu items -->
          <template v-for="(item, index) in resolvedMenuItems">
            <button v-if="!item.hidden"
              :class="[
                item.label === '-' ? 'mx-2 my-1 border-t border-base-content/30' : 'w-full px-2 py-1 flex justify-between text-sm whitespace-nowrap',
                item.disabled ? 'text-base-content/30' : 'hover:bg-base-100/30 hover:text-base-content hover:rounded-box cursor-pointer',
              ]"
              :key="index"
              @click="handleClick(item)"
            >      
              <div v-if="item.label !== '-'" class="w-full flex items-center">
                <div class="w-5">
                  <component class="t-icon-size-sm" :is="item.icon" ></component>
                </div>
                <span class="ml-2 mr-4">{{ item.label }}</span>
                <span class="ml-auto text-base-content/30">{{ item.shortcut }}</span>
              </div>
            </button>
          </template>
        </div>
      </transition> 
    </teleport>

  </div>
  
</template>
  
<script setup>
import { ref, shallowRef, onMounted, onBeforeUnmount, nextTick, useSlots, watch } from 'vue';

import TButton from '@/components/TButton.vue';

const slots = useSlots();

// Props
const props = defineProps({
  iconMenu : {
    type: Object, // SVG is typically imported as an object
    required: false, // Not required when using slot
    default: null,
  },
  menuItems: {
    type: [Array, Function], // Accept Array or Function for lazy generation
    required: true,
  },
  smallIcon: {
    type: Boolean,
    default: false,
  },
  disabled: {
    type: Boolean,
    default: false,    
  }
});

// Resolved menu items (computed on demand)
// Use shallowRef to avoid making icon components reactive
const resolvedMenuItems = shallowRef([]);

// Emits
const emit = defineEmits(['select']);

// State
const dropdown = ref(null);
const menu = ref(null);
const isDropDown = ref(false);
const menuStyle = ref({});

// Add event listener when the component is mounted
onMounted(() => {
  // Wait to attach until open
});

// Remove event listener when the component is destroyed
onBeforeUnmount(() => {
  document.removeEventListener('mousedown', handleClickOutside, { capture: true });
  document.removeEventListener('keydown', handleKeyDown);
});

// Watch for dropdown state to attach/detach listeners
watch(isDropDown, (val) => {
  if (val) {
    document.addEventListener('mousedown', handleClickOutside, { capture: true });
    document.addEventListener('keydown', handleKeyDown);
  } else {
    document.removeEventListener('mousedown', handleClickOutside, { capture: true });
    document.removeEventListener('keydown', handleKeyDown);
  }
});

// Handle Escape key press
const handleKeyDown = (event) => {
  if (isDropDown.value && event.key === 'Escape') {
    event.stopPropagation();
    isDropDown.value = false;
  }
};

// Open dropdown menu at specific coordinates or relative to the trigger
const open = async (x, y) => {
  if (props.disabled) return;

  isDropDown.value = true;

  // Resolve menu items (call function if provided)
  resolvedMenuItems.value = typeof props.menuItems === 'function' 
    ? props.menuItems() 
    : props.menuItems;

  await nextTick(); // Ensure menu is rendered before measuring

  const menuRect = menu.value.getBoundingClientRect();
  
  const padding = 8; // A smaller padding for a snug fit.
  const menuWidth = menuRect.width;
  const menuHeight = menuRect.height;
  const winWidth = window.innerWidth;
  const winHeight = window.innerHeight;
  const scrollY = window.scrollY;

  let top, left;

  if (x !== undefined && y !== undefined) {
    // Open at specific coordinates (e.g., context menu)
    top = y + scrollY;
    left = x + window.scrollX;
  } else {
    // Open relative to trigger button
    const rect = dropdown.value.getBoundingClientRect();
    top = rect.bottom + scrollY;
    left = rect.left + window.scrollX;

    // Check bottom boundary for trigger-relative positioning
    if (top + menuHeight > winHeight + scrollY - padding) {
      top = rect.top - menuHeight + scrollY;
    }
  }

  // Common boundary checks
  // Check bottom boundary (for context menu or general overflow)
  if (top + menuHeight > winHeight + scrollY - padding) {
     top = winHeight + scrollY - menuHeight - padding;
  }

  // Check top boundary
  if (top < scrollY + padding) {
    top = scrollY + padding;
  }

  // Check right boundary
  if (left + menuWidth > winWidth - padding) {
    left = winWidth - menuWidth - padding;
  }

  // Check left boundary
  if (left < padding) {
    left = padding;
  }

  menuStyle.value = { top: `${top}px`, left: `${left}px` };
};

// Toggle dropdown menu
const toggleDropdown = async () => {
  if (isDropDown.value) {
    isDropDown.value = false;
  } else {
    open();
  }
};

// Close dropdown when clicking outside
const handleClickOutside = (event) => {
  if (dropdown.value && !dropdown.value.contains(event.target) && menu.value && !menu.value.contains(event.target)) {
    isDropDown.value = false;
  }
};

const handleClick = (item) => {
  if (!item.disabled && item.action && typeof item.action === 'function') {
    item.action();
    isDropDown.value = false;
  }
};

defineExpose({
  open
});


</script>

<template>
  <div ref="dropdown" class="relative inline-block text-left">

    <!-- Dropdown Trigger -->
    <TButton
      :icon="iconMenu"
      :buttonSize="smallIcon ? 'small' : 'medium'"
      :disabled="disabled"
      @click="toggleDropdown" 
    />
    <!-- Dropdown Menu -->
    <teleport to="body">
      <transition name="fade">
        <div v-if="isDropDown" 
          ref="menu"
          class="menu text-base-content/70 bg-base-100 border border-base-content/30 absolute rounded-box shadow-lg z-50"
          :style="menuStyle"
        >
          <!-- menu items -->
          <template v-for="(item, index) in menuItems">
            <button v-if="!item.hidden"
              :class="[
                item.label === '-' ? 'mx-2 my-1 border-t border-base-content/30' : 'w-full px-2 py-1 flex justify-between text-sm whitespace-nowrap',
                item.disabled ? 'text-base-content/30' : 'hover:bg-base-content/10 hover:rounded cursor-pointer',
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
import { ref, onMounted, onBeforeUnmount, nextTick } from 'vue';

import TButton from '@/components/TButton.vue';

// Props
const props = defineProps({
  iconMenu : {
    type: Object, // SVG is typically imported as an object
    required: true,
  },
  menuItems: {
    type: Array,
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

// Emits
const emit = defineEmits(['select']);

// State
const dropdown = ref(null);
const menu = ref(null);
const isDropDown = ref(false);
const menuStyle = ref({});

// Add event listener when the component is mounted
onMounted(() => {
  document.addEventListener('mousedown', handleClickOutside, { capture: true });
  document.addEventListener('keydown', handleKeyDown);
});

// Remove event listener when the component is destroyed
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

// Toggle dropdown menu
const toggleDropdown = async () => {
  if (props.disabled) return;

  isDropDown.value = !isDropDown.value;

  if (isDropDown.value) {
    await nextTick(); // Ensure menu is rendered before measuring

    const rect = dropdown.value.getBoundingClientRect();
    const menuRect = menu.value.getBoundingClientRect();
    
    const padding = 8; // A smaller padding for a snug fit.
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

const handleClick = (item) => {
  if (!item.disabled && item.action && typeof item.action === 'function') {
    item.action();
    isDropDown.value = false;
  }
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
  
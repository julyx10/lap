<template>
  <div ref="dropdown" class="relative inline-block text-left">

    <!-- Dropdown Trigger -->
    <component 
      :is="iconMenu" 
      :class="[
        smallIcon ? 't-icon-size-sm' : 't-icon-size', 
        isDropDown ? 't-icon-selected' : '',
        disabled ? 't-icon-disabled' : 't-icon-hover'
      ]" 
      @click="toggleDropdown"
    />

    <!-- Dropdown Menu -->
    <teleport to="body">
      <transition name="fade">
        <div v-if="isDropDown" 
          ref="menu"
          class="absolute rounded-md shadow-lg t-color-bg-light border t-color-border z-50"
          :style="menuStyle"
        >
          <!-- menu items -->
            <button v-for="(item, index) in menuItems"
              :class="[
                'pl-2 pr-4 w-full flex  flex-row justify-between text-sm whitespace-nowrap', 
                item.label === '-' ? 'border-t t-color-border' : 'py-1 t-color-text t-color-bg-hover t-color-text-hover'
              ]"
              :key="index"
              @click="handleClick(item)"
            >
              <template v-if="item.label !== '-'">
                <div class="t-icon-size-sm mr-2">
                  <component :is="item.icon" ></component>
                </div>
                <span>{{ item.label }}</span>
                <span class="pl-4 ml-auto">{{ item.shortcut }}</span>
              </template>
            </button>
        </div>
      </transition> 
    </teleport>

  </div>
  
</template>
  
<script setup>
import { ref, onMounted, onBeforeUnmount, nextTick } from 'vue';

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
  document.addEventListener('click', handleClickOutside);
});

// Remove event listener when the component is destroyed
onBeforeUnmount(() => {
  document.removeEventListener('click', handleClickOutside);
});

// Toggle dropdown menu
const toggleDropdown = async () => {
  if (props.disabled) return;

  isDropDown.value = !isDropDown.value;

  if (isDropDown.value) {
    await nextTick(); // Ensure menu is rendered before measuring

    const rect = dropdown.value.getBoundingClientRect();
    const menuRect = menu.value.getBoundingClientRect();
    
    let top = rect.bottom + window.scrollY;
    let left = rect.left + window.scrollX;

    const padding = 20; // Space from edges
    const menuWidth = menuRect.width;
    const menuHeight = menuRect.height;

    if (left + menuWidth > window.innerWidth - padding) {
      left = window.innerWidth - menuWidth - padding;
    }

    if (top + menuHeight > window.innerHeight + window.scrollY - padding) {
      top = rect.top - menuHeight + window.scrollY; // Display above
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
  if (item.action && typeof item.action === 'function') {
    item.action();
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
  
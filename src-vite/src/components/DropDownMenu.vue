<template>
  <div ref="dropdown" class="relative inline-block text-left">

    <!-- Dropdown Trigger -->
    <component 
      :is="iconMenu" 
      :class="['t-icon-size t-icon-hover', isDropDown ? 't-icon-selected' : '']" 
      @click="toggleDropdown"
    />

    <!-- Dropdown Menu -->
    <transition name="fade">
      <div v-if="isDropDown"
        class="absolute right-0 my-1 rounded-md shadow-lg t-color-bg-light border t-color-border z-50"
      >
        <!-- menu items -->
        <button v-for="(item, index) in menuItems"
          :class="[
            'pl-2 pr-4 w-full flex flex-row justify-between text-sm whitespace-nowrap', 
            item.label === '-' ? 'border-t t-color-border' : 'py-1 t-color-bg-hover t-color-text-hover'
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

  </div>
  
</template>
  
<script setup>
import { ref, onMounted, onBeforeUnmount } from 'vue';

  // Props
const props = defineProps({
  iconMenu : {
    type: Object, // SVG is typically imported as an object
    required: true,
  },
  menuItems: {
    type: Array,
    required: true,
  }
});

// Emits
const emit = defineEmits(['select']);

// State
const dropdown = ref(null);
const isDropDown = ref(false);

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
  if (dropdown.value && !dropdown.value.contains(event.target)) {
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
  
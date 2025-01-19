<template>
  <div ref="dropdown" class="relative inline-block text-left">

    <!-- Dropdown Trigger -->
    <component 
      :is="iconMenu" 
      class="t-icon-size t-icon-hover" 
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
            'px-4 w-full flex text-sm whitespace-nowrap', 
            item.label === '-' ? 'border-t t-color-border' : 'py-1 t-color-bg-hover'
          ]"
          :key="index"
          @click="selectItem(item)"
        >
          <span v-if="item.label !== '-'">{{ item.label }}</span>
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

const selectItem = (item) => {
  emit('select', item);
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
  
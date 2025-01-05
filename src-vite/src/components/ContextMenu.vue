<template>
  <div
    v-if="visible"
    :style="menuStyle"
    class="fixed t-color-bg-light border t-color-border shadow-lg rounded-md z-50 min-w-[150px] overflow-hidden"
  >
    <ul>
      <li
        v-for="item in menuItems"
        :key="item.label"
        @click="handleClick(item)"
        class="px-4 py-2 t-color-bg-hover cursor-pointer"
      >
        {{ item.label }}
      </li>
    </ul>
  </div>
</template>

<script setup>
import { ref } from 'vue';

// Props for dynamic menu items and visibility control
const props = defineProps({
  menuItems: {
    type: Array,
    required: true,
    default: () => [],
  },
});

const visible = ref(false);
const menuStyle = ref({
  top: '0',
  left: '0',
});

// Show the context menu at the specified position
const showMenu = (event) => {
  visible.value = true;
  menuStyle.value.top = `${event.clientY}px`;
  menuStyle.value.left = `${event.clientX}px`;
  document.addEventListener('click', hideMenu);
};

// Hide the context menu
const hideMenu = () => {
  visible.value = false;
  document.removeEventListener('click', hideMenu);
};

// Handle menu item click
const handleClick = (item) => {
  if (item.action && typeof item.action === 'function') {
    item.action();
  }
  hideMenu();
};

// Expose the `showMenu` method to parent components
defineExpose({ showMenu });
</script>
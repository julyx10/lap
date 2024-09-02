<template>
  <div v-if="visible" :style="menuStyle" class="absolute bg-gray-700">
    <ul class="list-none m-0 p-0">
      <li v-for="(item, index) in items" class="py-2 px-3 cursor-pointer hover:bg-red-800" :key="index" @click="handleClick(item)">
        {{ item.label }}
      </li>
    </ul>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount } from 'vue';

const props = defineProps({
  items: {
    type: Array,
    required: true,
  },
});

const emit = defineEmits(['select']);

const visible = ref(false);
const menuStyle = ref({ top: '0px', left: '0px' });

const handleClick = (item) => {
  emit('select', item);
  visible.value = false;
};

const showMenu = (event) => {
  event.preventDefault();
  menuStyle.value = { top: `${event.clientY}px`, left: `${event.clientX}px` };
  visible.value = true;
};

const hideMenu = () => {
  visible.value = false;
};

onMounted(() => {
  document.addEventListener('contextmenu', showMenu);
  document.addEventListener('click', hideMenu);
});

onBeforeUnmount(() => {
  document.removeEventListener('contextmenu', showMenu);
  document.removeEventListener('click', hideMenu);
});
</script>

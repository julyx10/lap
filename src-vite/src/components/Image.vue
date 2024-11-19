<template>
  <div
    ref="container"
    class="relative overflow-hidden w-full h-full cursor-pointer"
    @mousedown="startDragging"
    @mousemove="onDragging"
    @mouseup="stopDragging"
    @mouseleave="stopDragging"
    @wheel="onZoom"
  >
    <img
      ref="image"
      :src="src"
      alt="Zoomable and Draggable"
      :style="imageStyle"
      class="absolute top-0 left-0 select-none transition-transform ease-out"
      draggable="false"
    />
  </div>
</template>

<script setup>
import { ref, computed } from 'vue';

// Props
const props = defineProps({
  src: {
    type: String,
    required: true,
  },
});

// Reactive state
const container = ref(null);
const image = ref(null);
const position = ref({ x: 0, y: 0 }); // Image position
const scale = ref(1);                 // Image scale (zoom level)
const isDragging = ref(false);        // Dragging state
const lastMousePosition = ref({ x: 0, y: 0 }); // Last mouse position for drag calculations

// Computed style for the image
const imageStyle = computed(() => ({
  transform: `translate(${position.value.x}px, ${position.value.y}px) scale(${scale.value})`,
  cursor: isDragging.value ? 'grabbing' : 'grab',
}));

// Methods
const startDragging = (event) => {
  isDragging.value = true;
  lastMousePosition.value = { x: event.clientX, y: event.clientY };
};

const onDragging = (event) => {
  if (!isDragging.value) return;

  const deltaX = event.clientX - lastMousePosition.value.x;
  const deltaY = event.clientY - lastMousePosition.value.y;

  position.value.x += deltaX;
  position.value.y += deltaY;

  lastMousePosition.value = { x: event.clientX, y: event.clientY };
};

const stopDragging = () => {
  isDragging.value = false;
};

const onZoom = (event) => {
  event.preventDefault();

  const zoomFactor = 0.1; // Adjust sensitivity
  const newScale = scale.value - event.deltaY * zoomFactor / 100;

  // Clamp zoom level
  scale.value = Math.min(Math.max(newScale, 0.1), 10); // Between 0.1x and 10x zoom
};
</script>

<style scoped>
/* Optional: Add styles for smoother interactions */
</style>

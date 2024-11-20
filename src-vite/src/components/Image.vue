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
      class="absolute top-0 left-0 select-none transition-transform ease-out"
      :src="src"
      :style="imageStyle"
      draggable="false"
    />
  </div>
</template>

<script setup>
import { ref, computed, onMounted } from 'vue';

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

// Container and image sizes
const containerSize = ref({ width: 0, height: 0 });
const imageSize = ref({ width: 0, height: 0 });

// Computed style for the image
const imageStyle = computed(() => ({
  transform: `translate(${position.value.x}px, ${position.value.y}px) scale(${scale.value})`,
  cursor: isDragging.value ? 'grabbing' : 'grab',
}));

// Ensure image stays within container
const clampPosition = () => {
  const scaledWidth = imageSize.value.width * scale.value;
  const scaledHeight = imageSize.value.height * scale.value;

  // Calculate boundaries for the image inside the container
  const minX = Math.min(0, containerSize.value.width - scaledWidth);
  const minY = Math.min(0, containerSize.value.height - scaledHeight);
  const maxX = Math.max(0, containerSize.value.width - scaledWidth);
  const maxY = Math.max(0, containerSize.value.height - scaledHeight);

  position.value.x = Math.max(minX, Math.min(position.value.x, maxX));
  position.value.y = Math.max(minY, Math.min(position.value.y, maxY));
};

// Methods
const startDragging = (event) => {
  if(imageSize.value.width * scale.value <= containerSize.value.width &&
     imageSize.value.height * scale.value <= containerSize.value.height) return;
     
  isDragging.value = true;
  lastMousePosition.value = { x: event.clientX, y: event.clientY };
};

const onDragging = (event) => {
  if (!isDragging.value) return;

  const deltaX = event.clientX - lastMousePosition.value.x;
  const deltaY = event.clientY - lastMousePosition.value.y;

  position.value.x += deltaX;
  position.value.y += deltaY;

  clampPosition(); // Adjust position to stay within bounds

  lastMousePosition.value = { x: event.clientX, y: event.clientY };
};

const stopDragging = () => {
  isDragging.value = false;
};

const zoomIn = () => {
  console.log('zoomIn');
  scale.value = Math.min(scale.value * 2, 10);
  clampPosition(); // Adjust position to ensure the image stays within bounds
};

const zoomOut = () => {
  scale.value = Math.max(scale.value / 2, 0.1);
  clampPosition(); // Adjust position to ensure the image stays within bounds
};

const zoomFit = () => {
  const containerAspectRatio = containerSize.value.width / containerSize.value.height;
  const imageAspectRatio = imageSize.value.width / imageSize.value.height;

  if (containerAspectRatio > imageAspectRatio) {
    scale.value = containerSize.value.height / imageSize.value.height;
  } else {
    scale.value = containerSize.value.width / imageSize.value.width;
  }

  position.value = { x: 0, y: 0 };
};

const zoomReset = () => {
  scale.value = 1;
  position.value = { x: 0, y: 0 };
};

const rotateRight = () => {
  // Rotate the image
};

const onZoom = (event) => {
  event.preventDefault();

  const zoomFactor = 0.1; // Adjust sensitivity
  const oldScale = scale.value;
  const newScale = Math.min(Math.max(oldScale - event.deltaY * zoomFactor / 100, 0.1), 10); // Clamp zoom level

  // Adjust position to zoom around the cursor
  const rect = container.value.getBoundingClientRect();
  const cursorX = event.clientX - rect.left;
  const cursorY = event.clientY - rect.top;

  const imageOffsetX = cursorX - position.value.x;
  const imageOffsetY = cursorY - position.value.y;

  const scaleRatio = newScale / oldScale;

  position.value.x -= imageOffsetX * (scaleRatio - 1);
  position.value.y -= imageOffsetY * (scaleRatio - 1);

  scale.value = newScale;

  clampPosition(); // Adjust position to ensure the image stays within bounds
};

// Initialize container and image sizes
const updateSizes = () => {
  containerSize.value = {
    width: container.value.offsetWidth,
    height: container.value.offsetHeight,
  };
  imageSize.value = {
    width: image.value.naturalWidth,
    height: image.value.naturalHeight,
  };
};

onMounted(() => {
  updateSizes();
  window.addEventListener('resize', updateSizes);
});

// Expose methods
defineExpose({ 
  zoomIn, 
  zoomOut,
  zoomFit,
  zoomReset,
  rotateRight
});

</script>

<style scoped>
/* Optional: Add styles for smoother interactions */
</style>

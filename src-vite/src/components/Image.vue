<template>

  <div
    ref="container"
    class="w-full h-full overflow-hidden cursor-pointer"
    @mousedown="startDragging"
    @mousemove="onDragging"
    @mouseup="stopDragging"
    @mouseleave="stopDragging"
    @wheel="onZoom"
  >
    <div class="absolute left-0 top-0 p-2 z-10 text-red-700 flex items-center justify-center">
      scale: {{ scale.toFixed(2) }}<br />
      containerSize: {{ containerSize }}<br />
      imageSize: {{ imageSize }}<br />
      position: {{ position }}<br />
      imageStyle: {{ imageStyle }}<br />
      mousePos: {{ mousePos }}<br />
    </div>
    <img
      ref="image"
      :class="[
        '',
        isDragging ? 'cursor-grabbing' : 'cursor-grab',
      ]"
      :src="src"
      :style="imageStyle"
      draggable="false"
    />
  </div>
</template>

<script setup>
import { ref, computed, onMounted, onBeforeUnmount, watch } from 'vue';

// Props
const props = defineProps({
  src: {
    type: String,
    required: true,
  },
  width: {
    type: Number,
    default: 0,
  },
  height: {
    type: Number,
    default: 0,
  },
  isZoomFit: {
    type: Boolean,
    default: false,
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

const mousePos = ref({x: 0, y: 0})
let resizeObserver;

// Computed style for the image
const imageStyle = computed(() => ({
  position: 'absolute',
  // transform: `scale(${scale.value})`,
  transform: `translate(${position.value.x}px, ${position.value.y}px) scale(${scale.value})`,
  // left: `${position.value.x}px`,
  // top: `${position.value.y}px`,
  // width: `${imageSize.value.width}px`,
  // height: `${imageSize.value.height}px`,
  minWidth: `${imageSize.value.width}px`,
  minHeight: `${imageSize.value.height}px`,
}));


onMounted(() => {
  resizeObserver = new ResizeObserver(entries => {
    for (let entry of entries) {
      containerSize.value = {
        width: entry.contentRect.width,
        height: entry.contentRect.height,
      };
    }
  });

  if (container.value) {
    resizeObserver.observe(container.value);
  }
});

onBeforeUnmount(() => {
  if (resizeObserver && container.value) {
    resizeObserver.unobserve(container.value);
  }
});


watch(() => [props.width, props.height], ([width, height]) => {
  if (width && height) {
    imageSize.value = { width, height };
    console.log('updateImageSize: ', imageSize.value);
    updateZoom();
  } 
});

watch(() => props.isZoomFit, (isZoomFit) => {
  console.log('isZoomFit: ', isZoomFit);
  updateZoom();
});

watch(() => containerSize.value, (size) => {
  console.log('updateContainerSize: ', size);
  updateZoom();
});

const updateZoom = () => {
  props.isZoomFit ? zoomFit(): zoomReset();
};

const zoomFit = () => {
  console.log('zoomFit');
  const containerAspectRatio = containerSize.value.width / containerSize.value.height;
  const imageAspectRatio = imageSize.value.width / imageSize.value.height;

  if (containerAspectRatio > imageAspectRatio) {
    scale.value = containerSize.value.height / imageSize.value.height;
  } else {
    scale.value = containerSize.value.width / imageSize.value.width;
  }

  const offsetX = (containerSize.value.width - imageSize.value.width) / 2;
  const offsetY = (containerSize.value.height - imageSize.value.height) / 2;

  position.value = { x: offsetX, y: offsetY };
};

const zoomReset = () => {
  console.log('zoomReset');
  scale.value = 1;

  const offsetX = (containerSize.value.width - imageSize.value.width) / 2;
  const offsetY = (containerSize.value.height - imageSize.value.height) / 2;

  position.value = { 
    x: Math.max(0, offsetX),
    y: Math.max(0, offsetY), 
  };
};

// mouse wheel zoom
const onZoom = (event) => {
  console.log('onZoom');
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

  // clampPosition(); // Adjust position to ensure the image stays within bounds
};

const zoomIn = () => {
  console.log('zoomIn');
  scale.value = Math.min(scale.value * 2, 10);
  // clampPosition(); // Adjust position to ensure the image stays within bounds
};

const zoomOut = () => {
  console.log('zoomOut');
  scale.value = Math.max(scale.value / 2, 0.1);
  // clampPosition(); // Adjust position to ensure the image stays within bounds
};

const rotateRight = () => {
  console.log('rotateRight');
  // Rotate the image
};

// Ensure image stays within container
const clampPosition = () => {
  if(imageSize.value.width * scale.value > containerSize.value.width) {
    position.value.x = Math.min(Math.max(position.value.x, containerSize.value.width - imageSize.value.width * scale.value), 0);
  } 
  if(imageSize.value.height * scale.value > containerSize.value.height) {
    position.value.y = Math.min(Math.max(position.value.y, containerSize.value.height - imageSize.value.height * scale.value), 0);
  }
};

// drag image if image size is larger than container size
const startDragging = (event) => {
  isDragging.value = true;
  lastMousePosition.value = { x: event.clientX, y: event.clientY };
};

const onDragging = (event) => {
  // mousePos.value = {x: event.clientX * scale.value, y: event.clientY * scale.value}

  if (!isDragging.value) return;

  const deltaX = imageSize.value.width * scale.value <= containerSize.value.width ? 0 : event.clientX - lastMousePosition.value.x;
  const deltaY = imageSize.value.height * scale.value <= containerSize.value.height ? 0 : event.clientY - lastMousePosition.value.y;

  position.value.x += deltaX;
  position.value.y += deltaY;

  clampPosition(); // Adjust position to stay within bounds

  lastMousePosition.value = { x: event.clientX, y: event.clientY };
};

const stopDragging = () => {
  isDragging.value = false;
};

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
</style>

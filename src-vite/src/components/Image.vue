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
    </div>
    <img
      ref="image"
      :class="[
        'transition-transform ease-out',
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

let resizeObserver;

// Computed style for the image
const imageStyle = computed(() => ({
  transform: ` scale(${scale.value})`,
  // transform: `translate(${position.value.x}px, ${position.value.y}px) scale(${scale.value})`,
  // left: `${position.value.x}px`,
  // top: `${position.value.y}px`,
  // width: `${imageSize.value.width}px`,
  // height: `${imageSize.value.height}px`,
  // minWidth: `${imageSize.value.width}px`,
  // minHeight: `${imageSize.value.height}px`,
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

// const updateImageSize = () => {
//   imageSize.value = {
//     width: image.value.naturalWidth,
//     height: image.value.naturalHeight,
//   };
//   console.log('updateImageSize: ', imageSize.value);
//   // zoomFit();
// };

watch(() => [props.width, props.height], ([width, height]) => {
  if (width && height) {
    imageSize.value = { width, height };
    console.log('updateImageSize: ', imageSize.value);
    zoomFit();
  } 
});

watch(() => containerSize.value, (size) => {
  console.log('updateContainerSize: ', size);
  zoomFit();
});
// const updateContainerSize = () => {
//   containerSize.value = {
//     width: container.value.offsetWidth,
//     height: container.value.offsetHeight,
//   };
//   console.log('updateContainerSize: ', containerSize.value);
// };

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

  // clampPosition(); // Adjust position to stay within bounds

  lastMousePosition.value = { x: event.clientX, y: event.clientY };
};

const stopDragging = () => {
  isDragging.value = false;
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

const zoomFit = () => {
  console.log('zoomFit');
  const containerAspectRatio = containerSize.value.width / containerSize.value.height;
  const imageAspectRatio = imageSize.value.width / imageSize.value.height;

  if (containerAspectRatio > imageAspectRatio) {
    scale.value = containerSize.value.height / imageSize.value.height;
  } else {
    scale.value = containerSize.value.width / imageSize.value.width;
  }

  const offsetX = (containerSize.value.width - imageSize.value.width * scale.value) / 2;
  const offsetY = (containerSize.value.height - imageSize.value.height * scale.value) / 2;

  position.value = { x: offsetX, y: offsetY };
};


const zoomReset = () => {
  console.log('zoomReset');
  scale.value = 1;
  // position.value = { x: 0, y: 0 };
};

const rotateRight = () => {
  console.log('rotateRight');
  // Rotate the image
};

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

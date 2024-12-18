<template>

  <div
    ref="container"
    class="relative w-full h-full overflow-hidden cursor-pointer"
    @mousedown="startDragging"
    @mousemove="onDragging"
    @mouseup="stopDragging"
    @mouseleave="stopDragging"
    @dblclick="zoomFit"
    @wheel="onZoom"
  >
    <!-- DEBUG -->
    <!-- <table class="absolute left-0 bottom-0 p-2 z-10 text-sky-700 bg-gray-100 opacity-50 text-sm">
      <tr>
        <td>scale</td>
        <td>{{ scale }}</td>
      </tr>
      <tr>
        <td>isZoomFit</td>
        <td>{{ isZoomFit }}</td>
      </tr>
      <tr>
        <td>imageRotate</td>
        <td>{{ imageRotate }}</td>
      </tr>
      <tr>
        <td>containerSize</td>
        <td>{{ containerSize }}</td>
      </tr>
      <tr>
        <td>containerPos</td>
        <td>{{ containerPos }}</td>
      </tr>
      <tr>
        <td>imageSize</td>
        <td>{{ imageSize }}</td>
      </tr>
      <tr>
        <td>imageSizeRotated</td>
        <td>{{ imageSizeRotated }}</td>
      </tr>
      <tr>
        <td>position</td>
        <td>{{ position }}</td>
      </tr>
    </table> -->

    <img v-if="src"
      ref="image"
      :class="isDragging && isGrabbing ? 'cursor-grabbing' : 'cursor-grab'"
      :src="src"
      :style="imageStyle"
      draggable="false"
      @load="onImageLoad($event.target)"
    />

  </div>
</template>

<script setup>
import { ref, watch, computed, onMounted, onBeforeUnmount } from 'vue';
import { emit } from '@tauri-apps/api/event';

// Props
const props = defineProps({
  src: {
    type: String,
    required: true,
  },
  rotate: {
    type: Number,
    default: 0,
  },  
  isZoomFit: {
    type: Boolean,
    default: false,
  },
});

// container
const container = ref(null);
const containerSize = ref({ width: 0, height: 0 });
const containerPos = ref({ x: 0, y: 0 });

// Image
const image = ref(null);              // image element
const imageRotate = ref(0);           // Image rotation
const imageSize = ref({ width: 0, height: 0 }); // actually image size
const imageSizeRotated = ref({ width: 0, height: 0 });  // image size after rotation
const isZoomFit = ref(false);         // Zoom to fit image in container
const scale = ref(1);                 // Image scale (zoom level)
const position = ref({ x: 0, y: 0 }); // Image position (top-left corner)
const isDragging = ref(false);        // Dragging state
const lastMousePosition = ref({ x: 0, y: 0 }); // Last mouse position for drag calculations
const isGrabbing = ref(false);        // Grabbing state

let resizeObserver;

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
    // Observe container size changes
    resizeObserver.observe(container.value);

    // Get container position
    const rect = container.value.getBoundingClientRect();
    containerPos.value = { x: rect.left, y: rect.top };
  }
});

onBeforeUnmount(() => {
  if (resizeObserver && container.value) {
    resizeObserver.unobserve(container.value);
  }
});

// watch src changes
watch(() => props.src, () => {
  isZoomFit.value = props.isZoomFit;
}, { immediate: true });

// watch rotate changes
watch(() => props.rotate, (newRotate) => {
  imageRotate.value = newRotate;
});

watch(() => imageRotate.value, (newValue) => {
  console.log('imageRotate:', newValue, imageRotate.value);
  // swap image width and height
  if (newValue % 180 === 90) {
    imageSizeRotated.value = { width: imageSize.value.height, height: imageSize.value.width };
  } else {
    imageSizeRotated.value = { width: imageSize.value.width,  height: imageSize.value.height };
  }

  if(isZoomFit.value) {
    zoomFit();
  } else {
    clampPosition();
  }

  emit('message-from-image', { message: 'rotate', rotate: imageRotate.value });
});

// watch zoom fit changes
watch(() => props.isZoomFit, (newValue) => {
  isZoomFit.value = newValue;
  updateZoom();
});

// watch container or image size changes
watch(() => [containerSize.value, imageSize.value], () => {
  if(isZoomFit.value) {
    zoomFit();
  } else {
    clampPosition();
  }
});

// display zoom scale for a while
watch(() => scale.value, (newScale) => {
  emit('message-from-image', { message: 'scale', scale: newScale });
});

// Computed style for the image
const imageStyle = computed(() => {
  return {
    minWidth:  `${imageSize.value.width}px`,
    minHeight: `${imageSize.value.height}px`,
    transform: `translate(${position.value.x}px, ${position.value.y}px) scale(${scale.value}) rotate(${imageRotate.value}deg)`,
    transition: isDragging.value ? 'none' : 'transform 0.3s ease-in-out',
  };
});

// watch image load
const onImageLoad = (img) => {
  imageSize.value = { width: img.naturalWidth, height: img.naturalHeight };

  // swap image width and height
  if (imageRotate.value % 180 === 90) {
    imageSizeRotated.value = { width: imageSize.value.height, height: imageSize.value.width };
  } else {
    imageSizeRotated.value = { width: imageSize.value.width,  height: imageSize.value.height };
  }

  updateZoom();
}

const rotateRight = () => {
  imageRotate.value += 90;
};

const updateZoom = () => {
  isZoomFit.value ? zoomFit() : zoomReset();
};

// Zoom to fit image in container
const zoomFit = () => {
  console.log('zoomFit');
  const containerAspectRatio = containerSize.value.width / containerSize.value.height;
  const imageAspectRatio = imageSizeRotated.value.width / imageSizeRotated.value.height;

  if (containerAspectRatio > imageAspectRatio) {
    scale.value = containerSize.value.height / imageSizeRotated.value.height;
  } else {
    scale.value = containerSize.value.width / imageSizeRotated.value.width;
  }

  // set position to center
  position.value = { 
    x: (containerSize.value.width - imageSize.value.width) / 2,
    y: (containerSize.value.height - imageSize.value.height) / 2,
  };
};

// Reset zoom level and position
const zoomReset = () => {
  console.log('zoomReset');
  scale.value = 1;

  // set position to center
  position.value = { 
    x: (containerSize.value.width - imageSize.value.width) / 2,
    y: (containerSize.value.height - imageSize.value.height) / 2,
  };
};

// mouse wheel zoom
const onZoom = (event) => {
  event.preventDefault();
  
  const zoomFactor = 0.1; // Adjust sensitivity
  const newScale = Math.min(Math.max(scale.value / (1 + event.deltaY * zoomFactor / 100), 0.1), 10); // Clamp zoom level
  zoomImage(event.clientX - containerPos.value.x, event.clientY - containerPos.value.y, newScale);
};

const zoomIn = () => {
  const newScale = Math.min(scale.value * 2, 10);
  zoomImage(containerSize.value.width / 2, containerSize.value.height / 2, newScale);
};

const zoomOut = () => {
  const newScale = Math.max(scale.value / 2, 0.1);
  zoomImage(containerSize.value.width / 2, containerSize.value.height / 2, newScale);
};

// drag image if image size is larger than container size
const startDragging = (event) => {
  isDragging.value = true;
  lastMousePosition.value = { x: event.clientX, y: event.clientY };
};

const onDragging = (event) => {
  if (!isDragging.value) 
    return;

  const deltaX = imageSizeRotated.value.width * scale.value <= containerSize.value.width ? 0 : event.clientX - lastMousePosition.value.x;
  const deltaY = imageSizeRotated.value.height * scale.value <= containerSize.value.height ? 0 : event.clientY - lastMousePosition.value.y;

  position.value.x += deltaX;
  position.value.y += deltaY;
  lastMousePosition.value = { x: event.clientX, y: event.clientY };

  clampPosition(); // Adjust position to stay within bounds
};

const stopDragging = () => {
  isDragging.value = false;
};

// Zoom image at cursor position
function zoomImage(cursorX, cursorY, newScale) {
  if(newScale === scale.value) 
    return;

  // 2024-12-05: finally to impl the function below :(
  const imageOffsetX = ((scale.value - newScale) * ((cursorX - position.value.x) - imageSize.value.width / 2)) / scale.value;
  const imageOffsetY = ((scale.value - newScale) * ((cursorY - position.value.y) - imageSize.value.height / 2)) / scale.value;
  position.value.x += imageOffsetX;
  position.value.y += imageOffsetY;

  scale.value = newScale;
  isZoomFit.value = false;

  clampPosition();
}

// Ensure image stays within container
function clampPosition() {
  const paddingX = (imageSizeRotated.value.width * scale.value - imageSize.value.width) / 2;
  const paddingY = (imageSizeRotated.value.height * scale.value - imageSize.value.height) / 2;
  const maxX = containerSize.value.width  - imageSizeRotated.value.width * scale.value + paddingX;
  const maxY = containerSize.value.height - imageSizeRotated.value.height * scale.value + paddingY;

  isGrabbing.value = false;
  if(imageSizeRotated.value.width * scale.value > containerSize.value.width) {
    position.value.x = Math.min(Math.max(position.value.x, maxX), paddingX);
    isGrabbing.value = true;
  } else {
    position.value.x = (containerSize.value.width - imageSize.value.width) / 2;
  }
  if(imageSizeRotated.value.height * scale.value > containerSize.value.height) {
    position.value.y = Math.min(Math.max(position.value.y, maxY), paddingY);
    isGrabbing.value = true;
  } else {
    position.value.y = (containerSize.value.height - imageSize.value.height) / 2;
  }
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

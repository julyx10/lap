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
    <table class="absolute left-0 bottom-0 p-2 z-10 text-sky-700 bg-gray-100 opacity-50 text-sm">
      <tr>
        <td>scale</td>
        <td>{{ scale }}</td>
      </tr>
      <tr>
        <td>isZoomFit</td>
        <td>{{ isZoomFit }} {{ isImageSrcChanged }}</td>
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
        <td>position</td>
        <td>{{ position }}</td>
      </tr>
    </table>

    <img ref="image"
      :class="isDragging && !isZoomFit ? 'cursor-grabbing' : 'cursor-grab'"
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

// container
const container = ref(null);                // Container element
const containerSize = ref({ width: 0, height: 0 }); // Container size
const containerPos = ref({ x: 0, y: 0 }); // Container position

// Image
const image = ref(null);
const isImageSrcChanged = ref(false);
const imageSize = ref({ width: 0, height: 0 });
const position = ref({ x: 0, y: 0 }); // Image position
const scale = ref(1);                 // Image scale (zoom level)
const rotate = ref(0);                // Image rotation
const isZoomFit = ref(false);         // Zoom to fit image in container
const isDragging = ref(false);        // Dragging state
const lastMousePosition = ref({ x: 0, y: 0 }); // Last mouse position for drag calculations

let resizeObserver;

// Computed style for the image
const imageStyle = computed(() => {
  return {
    // position: 'absolute',
    minWidth: `${imageSize.value.width}px`,
    minHeight: `${imageSize.value.height}px`,
    // transformOrigin: '0 0',
    transform: `translate(${position.value.x}px, ${position.value.y}px) scale(${scale.value}) rotate(${rotate.value}deg)`,
    transition: isDragging.value || isImageSrcChanged.value ? '' : 'transform 0.3s ease-in-out, opacity 0.3s ease-in-out', // enable animation
    // opacity: isImageSrcChanged.value ? 0.8 : 1,
  };
});

onMounted(() => {
  isZoomFit.value = props.isZoomFit;

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

// watch image src changes
watch(() => props.src, (newValue, oldSrc) => {
  rotate.value = 0;
  console.log('updateImageSrc: ', newValue);
  isImageSrcChanged.value = true; // disable animation

  // imageSize.value = { width: props.width, height: props.height };
  // // set position to center
  // position.value = { 
  //   x: (containerSize.value.width - imageSize.value.width) / 2,
  //   y: (containerSize.value.height - imageSize.value.height) / 2,
  // };
  updateZoom();

  // enable animation
  setTimeout(() => {
    isImageSrcChanged.value = false;
  }, 300);
});

// watch image size changes
watch(() => [props.width, props.height], ([width, height]) => {
  if (width && height) {
    imageSize.value = { width, height };
    console.log('updateImageSize: ', imageSize.value);
    updateZoom();
  } 
});

// watch zoom fit changes
watch(() => props.isZoomFit, (newValue) => {
  isZoomFit.value = newValue;
  console.log('isZoomFit: ', isZoomFit);
  updateZoom();
});

// watch container size changes
watch(() => containerSize.value, (size) => {
  console.log('updateContainerSize: ', size);

  if(isZoomFit.value) {
    zoomFit();
  } else {
    clampPosition();
  }
});

const updateZoom = () => {
  isZoomFit.value ? zoomFit(): zoomReset();
};

// Zoom to fit image in container
const zoomFit = () => {
  console.log('zoomFit');
  const containerAspectRatio = containerSize.value.width / containerSize.value.height;
  const imageAspectRatio = imageSize.value.width / imageSize.value.height;

  if (containerAspectRatio > imageAspectRatio) {
    scale.value = containerSize.value.height / imageSize.value.height;
  } else {
    scale.value = containerSize.value.width / imageSize.value.width;
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
  
  console.log('onZoom', event.deltaY);
  const zoomFactor = 0.1; // Adjust sensitivity
  const newScale = Math.min(Math.max(scale.value / (1 + event.deltaY * zoomFactor / 100), 0.1), 10); // Clamp zoom level
  zoomImage(event.clientX - containerPos.value.x, event.clientY - containerPos.value.y, newScale);
};

const zoomIn = () => {
  console.log('zoomIn');
  const newScale = Math.min(scale.value * 2, 10);
  zoomImage(containerSize.value.width / 2, containerSize.value.height / 2, newScale);
};

const zoomOut = () => {
  console.log('zoomOut');
  const newScale = Math.max(scale.value / 2, 0.1);
  zoomImage(containerSize.value.width / 2, containerSize.value.height / 2, newScale);
};

const rotateRight = () => {
  console.log('rotateRight');
  rotate.value = rotate.value + 90;

  // swap image width and height
  // if (rotate.value % 180 === 90) {
  //   const temp = imageSize.value;
  //   imageSize.value = { width: temp.height, height: temp.width };
  // }
};

// drag image if image size is larger than container size
const startDragging = (event) => {
  isDragging.value = true;
  lastMousePosition.value = { x: event.clientX, y: event.clientY };
};

const onDragging = (event) => {
  if (!isDragging.value) return;

  const deltaX = imageSize.value.width * scale.value <= containerSize.value.width ? 0 : event.clientX - lastMousePosition.value.x;
  const deltaY = imageSize.value.height * scale.value <= containerSize.value.height ? 0 : event.clientY - lastMousePosition.value.y;

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
  // 2024-12-05: finally to impl the function below :(
  const imageOffsetX = ( (scale.value - newScale) * ( (cursorX - position.value.x) - imageSize.value.width / 2) ) / scale.value;
  const imageOffsetY = ( (scale.value - newScale) * ( (cursorY - position.value.y) - imageSize.value.height / 2) ) / scale.value;
  position.value.x += imageOffsetX;
  position.value.y += imageOffsetY;

  scale.value = newScale;
  isZoomFit.value = false;

  clampPosition();
}

// Ensure image stays within container
function clampPosition() {
  const paddingX = imageSize.value.width * (scale.value - 1) / 2 + 10;
  const paddingY = imageSize.value.height * (scale.value - 1) / 2 + 10;
  const maxX = containerSize.value.width - imageSize.value.width - paddingX;
  const maxY = containerSize.value.height - imageSize.value.height - paddingY;

  if(imageSize.value.width * scale.value > containerSize.value.width) {
    position.value.x = Math.min(Math.max(position.value.x, maxX), paddingX);
  } else {
    position.value.x = (containerSize.value.width - imageSize.value.width) / 2;
  }
  if(imageSize.value.height * scale.value > containerSize.value.height) {
    position.value.y = Math.min(Math.max(position.value.y, maxY), paddingY);
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

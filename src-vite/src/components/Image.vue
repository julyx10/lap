<template>

  <div
    ref="container"
    class="relative w-full h-full overflow-hidden cursor-pointer"
    @mousedown="startDragging"
    @mousemove="onDragging"
    @mouseup="stopDragging"
    @mouseleave="stopDragging"
    @wheel="onZoom"
  >
    <!-- DEBUG -->
    <table class="absolute left-0 top-0 p-2 z-10 text-sky-700 bg-gray-100 opacity-50 text-sm">
      <tr>
        <td>scale</td>
        <td>{{ scale }}</td>
      </tr>
      <tr>
        <td>isZoomFit</td>
        <td>{{ isZoomFit }}</td>
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

    <img
      ref="image"
      :class="[
        '',
        isDragging && !isZoomFit ? 'cursor-grabbing' : 'cursor-grab',
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

// container
const container = ref(null);                // Container element
const containerSize = ref({ width: 0, height: 0 }); // Container size
const containerPos = ref({ x: 0, y: 0 }); // Container position

// Image
const image = ref(null);
const imageSize = ref({ width: 0, height: 0 });
const position = ref({ x: 0, y: 0 }); // Image position
const scale = ref(1);                 // Image scale (zoom level)
const isZoomFit = ref(false);         // Zoom to fit image in container
const isDragging = ref(false);        // Dragging state
const lastMousePosition = ref({ x: 0, y: 0 }); // Last mouse position for drag calculations

let resizeObserver;

// Computed style for the image
const imageStyle = computed(() => ({
  transform: `translate(${position.value.x}px, ${position.value.y}px) scale(${scale.value})`,
  transformOrigin: '0 0',
  minWidth: `${imageSize.value.width}px`,
  minHeight: `${imageSize.value.height}px`,
}));


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

  position.value = { 
    x: (containerSize.value.width - imageSize.value.width * scale.value) / 2,
    y: (containerSize.value.height - imageSize.value.height * scale.value) / 2,
  };
};

// Reset zoom level and position
const zoomReset = () => {
  console.log('zoomReset');
  scale.value = 1;
  position.value = { 
    x: (containerSize.value.width - imageSize.value.width) / 2,
    y: (containerSize.value.height - imageSize.value.height) / 2,
  };
};

// mouse wheel zoom
const onZoom = (event) => {
  console.log('onZoom', event.deltaY);
  event.preventDefault();

  const zoomFactor = 0.1; // Adjust sensitivity
  const oldScale = scale.value;
  const newScale = Math.min(Math.max(oldScale / (1 + event.deltaY * zoomFactor / 100), 0.1), 10); // Clamp zoom level

  const imageOffsetX = (event.clientX - containerPos.value.x - position.value.x) * (oldScale - newScale) / oldScale;
  const imageOffsetY = (event.clientY - containerPos.value.y - position.value.y) * (oldScale - newScale) / oldScale;

  position.value.x += imageOffsetX;
  position.value.y += imageOffsetY;

  scale.value = newScale;
  isZoomFit.value = false;

  clampPosition(); // Adjust position to ensure the image stays within bounds
};

const zoomIn = () => {
  console.log('zoomIn');
  scale.value = Math.min(scale.value * 2, 10);
  isZoomFit.value = false;

  clampPosition(); // Adjust position to ensure the image stays within bounds
};

const zoomOut = () => {
  console.log('zoomOut');
  scale.value = Math.max(scale.value / 2, 0.1);
  isZoomFit.value = false;

  clampPosition(); // Adjust position to ensure the image stays within bounds
};

const rotateRight = () => {
  console.log('rotateRight');
  // Rotate the image
};

// Ensure image stays within container
const clampPosition = () => {
  const maxX = (containerSize.value.width - imageSize.value.width * scale.value);
  const maxY = (containerSize.value.height - imageSize.value.height * scale.value);

  if(imageSize.value.width * scale.value > containerSize.value.width) {
    position.value.x = Math.min(Math.max(position.value.x, maxX), 0);
  } else {
    position.value.x = (containerSize.value.width - imageSize.value.width * scale.value) / 2;
  }
  if(imageSize.value.height * scale.value > containerSize.value.height) {
    position.value.y = Math.min(Math.max(position.value.y, maxY), 0);
  } else {
    position.value.y = (containerSize.value.height - imageSize.value.height * scale.value) / 2;
  }
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

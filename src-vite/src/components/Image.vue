<template>

  <div
    ref="container"
    class="relative w-full h-full overflow-hidden cursor-pointer"
    @mousedown="startDragging"
    @mousemove="onDragging"
    @mouseup="stopDragging"
    @mouseleave="stopDragging"
    @dblclick="zoomFit"
    @wheel="onMouseWheel"
  >
    <!-- DEBUG -->
    <table v-if="config.debugMode" class="absolute left-0 bottom-0 p-2 z-10 text-sky-700 bg-gray-100 opacity-50 text-sm">
      <tbody>
        <tr>
          <td>activeImage</td>
          <td>{{ activeImage }}</td>
        </tr>
        <tr>
          <td>position</td>
          <td>{{ position[0] }}, {{ position[1] }}</td>
        </tr>   
        <tr>
          <td>scale</td>
          <td>{{ scale[0] }}, {{ scale[1] }}</td>
        </tr>
        <tr>
          <td>imageRotate</td>
          <td>{{ imageRotate[0] }}, {{ imageRotate[1] }}</td>
        </tr>
        <tr>
          <td>imageSize</td>
          <td>{{ imageSize[0] }}, {{ imageSize[1] }}</td>
        </tr>
        <tr>
          <td>imageSizeRotated</td>
          <td>{{ imageSizeRotated[0] }}, {{ imageSizeRotated[1] }}</td>
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
      </tbody>
    </table>

    <img 
      v-for="(src, index) in imageSrc"
      v-show="activeImage === index"
      :ref="'image' + (index + 1)"
      :key="index"
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
import { useConfigStore } from '@/stores/configStore';

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

// config store
const config = useConfigStore();

// container
const container = ref(null);
const containerSize = ref({ width: 0, height: 0 });
const containerPos = ref({ x: 0, y: 0 });
const isZoomFit = ref(false);               // Zoom to fit image in container

// image
const activeImage = ref(1);                 // which image is active (0 or 1)
const imageSrc = ref(['', '']);             // image source 1
const position = ref([{ x: 0, y: 0 }, { x: 0, y: 0 }]); // Image position (top-left corner)
const scale = ref([1, 1]);                  // Image scale (zoom level)
const minScale = ref(0);                    // Minimum zoom level
const maxScale = ref(10);                   // Maximum zoom level
const imageRotate = ref([0, 0]);            // Image rotation
const imageSize = ref([{ width: 0, height: 0 }, { width: 0, height: 0 }] );       // actually image size
const imageSizeRotated = ref([{ width: 0, height: 0 }, { width: 0, height: 0 }]); // image size after rotation

const isDragging = ref(false);              // Dragging state
const lastMousePosition = ref({ x: 0, y: 0 }); // Last mouse position for drag calculations
const isGrabbing = ref(false);              // Grabbing state

let resizeObserver;
let positionObserver;

onMounted(() => {
  // observe container size changes
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

    // Initial position calculation
    updatePosition();

    // Observe position changes using requestAnimationFrame
    startPositionObserver();
  }
});

onBeforeUnmount(() => {
  if (resizeObserver && container.value) {
    resizeObserver.unobserve(container.value);
  }
});

const updatePosition = () => {
  if (container.value) {
    const rect = container.value.getBoundingClientRect();
    containerPos.value = { x: rect.left, y: rect.top };
  }
};

const startPositionObserver = () => {
  const observePosition = () => {
    updatePosition();
    positionObserver = requestAnimationFrame(observePosition);
  };
  positionObserver = requestAnimationFrame(observePosition);
};

const stopPositionObserver = () => {
  if (resizeObserver && container.value) {
    resizeObserver.unobserve(container.value);
    resizeObserver.disconnect();
  }
  stopPositionObserver();
};

// watch src changes
watch(() => props.src, (newSrc) => {
  isZoomFit.value = props.isZoomFit;

  // preload to the hide image, then swap the image when loaded
  imageSrc.value[activeImage.value ^ 1] = newSrc;
  imageRotate.value[activeImage.value ^ 1] = props.rotate;
}, { immediate: true });

// watch rotate changes
watch(() => props.rotate, (newRotate) => {
  imageRotate.value[activeImage.value] = newRotate;
});

watch(() => imageRotate.value[activeImage.value], (newValue) => {
  // swap image width and height
  if (newValue % 180 === 90) {
    imageSizeRotated.value[activeImage.value] = { 
      width: imageSize.value[activeImage.value].height, 
      height: imageSize.value[activeImage.value].width 
    };
  } else {
    imageSizeRotated.value[activeImage.value] = { 
      width: imageSize.value[activeImage.value].width,  
      height: imageSize.value[activeImage.value].height 
    };
  }

  if(isZoomFit.value) {
    zoomFit();
  } else {
    clampPosition();
  }
});

// display zoom scale for a while
watch(() => scale.value[activeImage.value], (newValue) => {
  emit('message-from-image', { 
    message: 'scale', 
    scale: newValue, 
    minScale: minScale.value, 
    maxScale: maxScale.value 
  });
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

// Computed style for the image
const imageStyle = computed(() => {
  return {
    minWidth:  `${imageSize.value[activeImage.value].width}px`,
    minHeight: `${imageSize.value[activeImage.value].height}px`,
    transform: `translate(${position.value[activeImage.value].x}px, ${position.value[activeImage.value].y}px) 
                scale(${scale.value[activeImage.value]}) 
                rotate(${imageRotate.value[activeImage.value]}deg)`,
    transition: !isDragging.value ? 'transform 0.3s ease-in-out' : 'none',
  };
});

// watch image load
const onImageLoad = (img) => {
  // toggle to active image when loaded (0 -> 1, 1 -> 0)
  activeImage.value = activeImage.value ^ 1;

  imageSize.value[activeImage.value] = { width: img.naturalWidth, height: img.naturalHeight };

  // swap image width and height
  if (imageRotate.value[activeImage.value] % 180 === 90) {
    imageSizeRotated.value[activeImage.value] = { 
      width: imageSize.value[activeImage.value].height, 
      height: imageSize.value[activeImage.value].width 
    };
  } else {
    imageSizeRotated.value[activeImage.value] = { 
      width: imageSize.value[activeImage.value].width,  
      height: imageSize.value[activeImage.value].height 
    };
  }
  updateZoom();
}

const rotateRight = () => {
  imageRotate.value[activeImage.value] += 90;
};

const updateZoom = () => {
  console.log('updateZoom');
  isZoomFit.value ? zoomFit() : zoomReset();

  // set the hide image to the same position
  imageSrc.value[activeImage.value ^ 1] = '';
  position.value[activeImage.value ^ 1] = position.value[activeImage.value];
};

// Zoom to fit image in container
const zoomFit = () => {
  console.log('zoomFit');
  const containerAspectRatio = containerSize.value.width / containerSize.value.height;
  const imageAspectRatio = imageSizeRotated.value[activeImage.value].width / imageSizeRotated.value[activeImage.value].height;

  if (containerAspectRatio > imageAspectRatio) {
    scale.value[activeImage.value] = containerSize.value.height / imageSizeRotated.value[activeImage.value].height;
  } else {
    scale.value[activeImage.value] = containerSize.value.width / imageSizeRotated.value[activeImage.value].width;
  }

  // set position to center
  position.value[activeImage.value] = { 
    x: (containerSize.value.width - imageSize.value[activeImage.value].width) / 2,
    y: (containerSize.value.height - imageSize.value[activeImage.value].height) / 2,
  };
};

// Reset zoom level and position
const zoomReset = () => {
  console.log('zoomReset');
  scale.value[activeImage.value] = 1;

  // set position to center
  position.value[activeImage.value] = { 
    x: (containerSize.value.width - imageSize.value[activeImage.value].width) / 2,
    y: (containerSize.value.height - imageSize.value[activeImage.value].height) / 2,
  };
};

// mouse wheel zoom
const onMouseWheel = (event) => {
  event.preventDefault();
  if(config.mouseWheelMode === 0) {  // 0: previous/next image
    if(event.ctrlKey) {     // ctrl + mouse wheel: zoom in / out
      mouseZoom(event);
    } else{
      emit('message-from-image-viewer', { message: event.deltaY < 0 ? 'prev' : 'next' });
    }
  } else if(config.mouseWheelMode === 1) {  // 1: zoom in / out
    mouseZoom(event);
  }
};

const mouseZoom = (event) => {
  const zoomFactor = 0.1; // Adjust sensitivity
  minScale.value = Math.min(
    0.1, 
    Math.min(
      containerSize.value.width / imageSizeRotated.value[activeImage.value].width, 
      containerSize.value.height / imageSizeRotated.value[activeImage.value].height
    ) * 0.5
  );

  // Clamp zoom level
  const newScale = Math.min(Math.max(scale.value[activeImage.value] / (1 + event.deltaY * zoomFactor / 100), minScale.value), maxScale.value);
  zoomImage(event.clientX - containerPos.value.x, event.clientY - containerPos.value.y, newScale);
};

const zoomIn = () => {
  const newScale = Math.min(scale.value[activeImage.value] * 2, maxScale.value);
  zoomImage(containerSize.value.width / 2, containerSize.value.height / 2, newScale);
};

const zoomOut = () => {
  minScale.value = Math.min(
    0.1, 
    Math.min(
      containerSize.value.width / imageSizeRotated.value[activeImage.value].width, 
      containerSize.value.height / imageSizeRotated.value[activeImage.value].height
    ) * 0.5
  );

  const newScale = Math.max(scale.value[activeImage.value] / 2, minScale.value);
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

  const deltaX = imageSizeRotated.value[activeImage.value].width * scale.value[activeImage.value] <= containerSize.value.width ? 0 : event.clientX - lastMousePosition.value.x;
  const deltaY = imageSizeRotated.value[activeImage.value].height * scale.value[activeImage.value] <= containerSize.value.height ? 0 : event.clientY - lastMousePosition.value.y;

  position.value[activeImage.value].x += deltaX;
  position.value[activeImage.value].y += deltaY;
  lastMousePosition.value = { x: event.clientX, y: event.clientY };

  clampPosition(); // Adjust position to stay within bounds
};

const stopDragging = () => {
  isDragging.value = false;
};

// Zoom image at cursor position
function zoomImage(cursorX, cursorY, newScale) {
  if(newScale === scale.value[activeImage.value]) 
    return;

  const imageOffsetX = ((scale.value[activeImage.value] - newScale) * ((cursorX - position.value[activeImage.value].x) - imageSize.value[activeImage.value].width / 2)) / scale.value[activeImage.value];
  const imageOffsetY = ((scale.value[activeImage.value] - newScale) * ((cursorY - position.value[activeImage.value].y) - imageSize.value[activeImage.value].height / 2)) / scale.value[activeImage.value];
  position.value[activeImage.value].x += imageOffsetX;
  position.value[activeImage.value].y += imageOffsetY;

  scale.value[activeImage.value] = newScale;
  isZoomFit.value = false;

  clampPosition();
}

// Ensure image stays within container
function clampPosition() {
  const paddingX = (imageSizeRotated.value[activeImage.value].width * scale.value[activeImage.value] - imageSize.value[activeImage.value].width) / 2;
  const paddingY = (imageSizeRotated.value[activeImage.value].height * scale.value[activeImage.value] - imageSize.value[activeImage.value].height) / 2;
  const maxX = containerSize.value.width  - imageSizeRotated.value[activeImage.value].width * scale.value[activeImage.value] + paddingX;
  const maxY = containerSize.value.height - imageSizeRotated.value[activeImage.value].height * scale.value[activeImage.value] + paddingY;

  isGrabbing.value = false;
  if(imageSizeRotated.value[activeImage.value].width * scale.value[activeImage.value] > containerSize.value.width) {
    position.value[activeImage.value].x = Math.min(Math.max(position.value[activeImage.value].x, maxX), paddingX);
    isGrabbing.value = true;
  } else {
    position.value[activeImage.value].x = (containerSize.value.width - imageSize.value[activeImage.value].width) / 2;
  }
  if(imageSizeRotated.value[activeImage.value].height * scale.value[activeImage.value] > containerSize.value.height) {
    position.value[activeImage.value].y = Math.min(Math.max(position.value[activeImage.value].y, maxY), paddingY);
    isGrabbing.value = true;
  } else {
    position.value[activeImage.value].y = (containerSize.value.height - imageSize.value[activeImage.value].height) / 2;
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

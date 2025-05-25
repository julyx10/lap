<template>

  <div
    ref="container"
    class="relative w-full h-full overflow-hidden cursor-pointer"
    @mousedown="handleMouseDown"
    @mousemove="handleMouseMove"
    @mouseup="handleMouseUp"
    @mouseleave="handleMouseLeave"
    @wheel="handleWheel"
    @dblclick="zoomFit"
  >
    <img 
      v-for="(src, index) in imageSrc"
      v-show="activeImage === index"
      ref="activeImageEl"
      :key="`img-${index}`"
      :src="src"
      :class="isDragging && isGrabbing ? 'cursor-grabbing' : 'cursor-grab'"
      :style="{
        minWidth:  `${imageSize[index].width}px`,
        minHeight: `${imageSize[index].height}px`,
        transform: `translate(${position[index].x}px, ${position[index].y}px) 
        scale(${scale[index]}) 
        rotate(${imageRotate[index]}deg)`,
        transition: !isDragging ? 'transform 0.3s ease-in-out' : 'none',
        willChange: 'transform',
      }"
      draggable="false"
      @load="onImageLoad($event.target)"
    />

  </div>
</template>

<script setup lang="ts">
import { ref, shallowRef, triggerRef, watch, onMounted, onBeforeUnmount } from 'vue';
import { emit } from '@tauri-apps/api/event';
import { config } from '@/common/utils';

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
const isZoomFit = ref(false);               // Zoom to fit image in container

// image
const activeImage = ref(1);                 // which image is active (0 or 1)
const imageSrc = ref(['', '']);             // image source 1
const position = shallowRef([{ x: 0, y: 0 }, { x: 0, y: 0 }]); // Image position (top-left corner)
const scale = ref([1, 1]);                  // Image scale (zoom level)
const minScale = ref(0);                    // Minimum zoom level
const maxScale = ref(10);                   // Maximum zoom level
const imageRotate = ref([0, 0]);            // Image rotation
const imageSize = ref([{ width: 0, height: 0 }, { width: 0, height: 0 }] );       // actually image size
const imageSizeRotated = ref([{ width: 0, height: 0 }, { width: 0, height: 0 }]); // image size after rotation

const isDragging = ref(false);              // Dragging state
const isGrabbing = ref(false);              // Grabbing state
const lastMousePosition = ref({ x: 0, y: 0 }); // Last mouse position for drag calculations
const mousePosition = ref({ x: 0, y: 0 });  // Current mouse position

// TODO: use requestAnimationFrame to improve performance
let animationFrameId: number | null = null;
const latestMouseEvent = ref<MouseEvent | null>(null);

// macOS touchpad wheel - accumulate delta values until they reach a threshold
let wheelDeltaAccumulator = 0;
let wheelThreshold = 10;

const activeImageEl = ref<HTMLImageElement | null>(null)

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
    resizeObserver.observe(container.value);  // Observe container size changes
    updatePosition();   // Initial position calculation
    startPositionObserver();  // Observe position changes using requestAnimationFrame
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
watch(() => props.src, () => {
  isZoomFit.value = props.isZoomFit;

  // preload to the hide image, then swap the image when loaded
  imageSrc.value[activeImage.value ^ 1] = props.src;
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
  updateZoomFit();
});

// watch container or image size changes
watch(() => [containerSize.value, imageSize.value], () => {
  if(isZoomFit.value) {
    zoomFit();
  } else {
    clampPosition();
  }
});

// load image
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
  updateZoomFit();
}

const rotateRight = () => {
  imageRotate.value[activeImage.value] += 90;
};

const updateZoomFit = () => {
  console.log('updateZoomFit');
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

  const scale = containerAspectRatio > imageAspectRatio 
    ? containerSize.value.height / imageSizeRotated.value[activeImage.value].height
    : containerSize.value.width / imageSizeRotated.value[activeImage.value].width;

  // set position to center
  zoomImage(containerSize.value.width / 2, containerSize.value.height / 2, scale);
};

// Reset zoom level and position
const zoomReset = () => {
  console.log('zoomReset');
  zoomImage(mousePosition.value.x - containerPos.value.x, mousePosition.value.y - containerPos.value.y, 1);
};

// start dragging
const handleMouseDown = (event) => {
  isDragging.value = true;
  lastMousePosition.value = { x: event.clientX, y: event.clientY };
};

const handleMouseMove = (event: MouseEvent) => {
  // update mouse position
  mousePosition.value = { x: event.clientX, y: event.clientY };

  if (!isDragging.value) return;

  latestMouseEvent.value = event;

  if (animationFrameId) {
    cancelAnimationFrame(animationFrameId);
  }

  animationFrameId = requestAnimationFrame(updateDragPosition);
};

// stop dragging
const handleMouseUp = () => {
  isDragging.value = false;
};

// mouse leave
// reset mouse position to the center when leaving the container
const handleMouseLeave = () => {
  // purpose: when clicking zoom fit/reset, the image will be centered
  // and the mouse position will be set to the center of the container
  mousePosition.value = { x: containerSize.value.width / 2, y: containerSize.value.height / 2 };
};

const updateDragPosition = () => {
  const event = latestMouseEvent.value;
  if (!event) return;

  const imgIndex = activeImage.value;
  const scaleVal = scale.value[imgIndex];

  const imageRotatedSize = imageSizeRotated.value[imgIndex];
  const container = containerSize.value;

  const deltaX =
    imageRotatedSize.width * scaleVal <= container.width
      ? 0
      : event.clientX - lastMousePosition.value.x;
  const deltaY =
    imageRotatedSize.height * scaleVal <= container.height
      ? 0
      : event.clientY - lastMousePosition.value.y;

  position.value[imgIndex].x += deltaX;
  position.value[imgIndex].y += deltaY;

  lastMousePosition.value = { x: event.clientX, y: event.clientY };

  clampPosition();

  animationFrameId = null; // reset animation frame ID
};

// mouse wheel zoom
const handleWheel = (event) => {
  event.preventDefault();

  // macbook touchpad
  const isTouchPad = Math.abs(event.deltaY) < 4 && event.deltaMode === 0;
  // console.log('handleWheel', event.deltaX, event.deltaY, isTouchPad);

  if(isTouchPad) {
    // accumulate delta values until they reach a threshold
    wheelDeltaAccumulator += event.deltaY;
    if(Math.abs(wheelDeltaAccumulator) < wheelThreshold) {
      return;
    }
    wheelDeltaAccumulator = 0;
  }

  const zoomFactor = isTouchPad ? 1 : 0.1; // Adjust sensitivity

  if(config.mouseWheelMode === 0) {  // 0: previous/next image
    if(event.ctrlKey) {     // ctrl + mouse wheel: zoom in / out
      wheelZoom(event, zoomFactor);
    } else{
      emit('message-from-image-viewer', { message: event.deltaY < 0 ? 'prev' : 'next' });
    }
  } else if(config.mouseWheelMode === 1) {  // 1: zoom in / out
    wheelZoom(event, zoomFactor);
  }
};

// wheel zoom
// zoomFactor: Adjust sensitivity
const wheelZoom = (event, zoomFactor) => {
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

// Zoom image at cursor position
function zoomImage(cursorX, cursorY, newScale) {
  const imageOffsetX = ((scale.value[activeImage.value] - newScale) * ((cursorX - position.value[activeImage.value].x) - imageSize.value[activeImage.value].width / 2)) / scale.value[activeImage.value];
  const imageOffsetY = ((scale.value[activeImage.value] - newScale) * ((cursorY - position.value[activeImage.value].y) - imageSize.value[activeImage.value].height / 2)) / scale.value[activeImage.value];
  position.value[activeImage.value].x += imageOffsetX;
  position.value[activeImage.value].y += imageOffsetY;

  scale.value[activeImage.value] = newScale;
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
  triggerRef(position);
};

// Expose methods
defineExpose({ 
  zoomIn, 
  zoomOut,
  rotateRight
});

</script>

<style scoped>
</style>

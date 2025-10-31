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
        transition: !isDragging && !noTransition ? 'transform 0.3s ease-in-out' : 'none',
        willChange: 'transform',
      }"
      draggable="false"
      @load="onImageLoad($event.target)"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, shallowRef, triggerRef, watch, onMounted, onBeforeUnmount, nextTick } from 'vue';
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
const imageSrc = ref(['', '']);             // image source
const position = shallowRef([{ x: 0, y: 0 }, { x: 0, y: 0 }]); // Image position (top-left corner)
const scale = ref([1, 1]);                  // Image scale (zoom level)
const minScale = ref(0);                    // Minimum zoom level
const maxScale = ref(10);                   // Maximum zoom level
const imageRotate = ref([0, 0]);            // Image rotation
const imageSize = ref([{ width: 0, height: 0 }, { width: 0, height: 0 }]);       // actual image size
const imageSizeRotated = ref([{ width: 0, height: 0 }, { width: 0, height: 0 }]); // image size after rotation

const isDragging = ref(false);              // Dragging state
const isGrabbing = ref(false);              // Grabbing state
const noTransition = ref(false);            // Disable transition temporarily
const lastMousePosition = ref({ x: 0, y: 0 }); // Last mouse position for drag calculations
const mousePosition = ref({ x: 0, y: 0 });  // Current mouse position

let animationFrameId: number | null = null;
const latestMouseEvent = ref<MouseEvent | null>(null);

// macOS touchpad wheel - accumulate delta values until they reach a threshold
let wheelDeltaAccumulator = 0;
let wheelThreshold = 10;

const activeImageEl = ref<HTMLImageElement | null>(null);

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
    resizeObserver.disconnect();
  }
  if (positionObserver) cancelAnimationFrame(positionObserver);
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

// watch src changes
watch(() => props.src, () => {
  isZoomFit.value = props.isZoomFit;

  // preload to the hide image, then swap the image when loaded
  const nextImageIndex = activeImage.value ^ 1;
  scale.value[nextImageIndex] = 1;
  position.value[nextImageIndex] = { x: 0, y: 0 };
  imageSrc.value[nextImageIndex] = props.src;
  imageRotate.value[nextImageIndex] = props.rotate;
}, { immediate: true });

// watch rotate changes
watch(() => props.rotate, (newRotate) => {
  imageRotate.value[activeImage.value] = newRotate;
});

watch(() => imageRotate.value[activeImage.value], (newValue) => {
  const imgIndex = activeImage.value;
  const imgSize = imageSize.value[imgIndex];
  
  // swap image width and height
  if (newValue % 180 === 90) {
    imageSizeRotated.value[imgIndex] = { 
      width: imgSize.height, 
      height: imgSize.width 
    };
  } else {
    imageSizeRotated.value[imgIndex] = { 
      width: imgSize.width,  
      height: imgSize.height 
    };
  }

  if (isZoomFit.value) {
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

// watch container or image size changes with debouncing
let debounceTimeout: number | null = null;
watch(() => [containerSize.value, imageSize.value], () => {
  if (debounceTimeout) clearTimeout(debounceTimeout);
  debounceTimeout = setTimeout(() => {
    if (isZoomFit.value) {
      zoomFit();
    } else {
      clampPosition();
    }
  }, 100); // Debounce for 100ms
});

// load image
const onImageLoad = async (img) => {
  noTransition.value = true; // Set early to prevent transitions during load

  const nextIndex = activeImage.value ^ 1;
  imageSize.value[nextIndex] = { width: img.naturalWidth, height: img.naturalHeight };

  // swap image width and height
  if (imageRotate.value[nextIndex] % 180 === 90) {
    imageSizeRotated.value[nextIndex] = { 
      width: img.naturalHeight, 
      height: img.naturalWidth 
    };
  } else {
    imageSizeRotated.value[nextIndex] = { 
      width: img.naturalWidth,  
      height: img.naturalHeight 
    };
  }

  activeImage.value = nextIndex;

  const applyZoom = () => {
    if (!isZoomFit.value) {
      const cSize = containerSize.value;
      const cPos = containerPos.value;
      mousePosition.value = { x: cPos.x + cSize.width / 2, y: cPos.y + cSize.height / 2 };
    }
    updateZoomFit();
    setTimeout(() => {
      noTransition.value = false;
    }, 500);
  };

  if (containerSize.value.width > 0) {
    applyZoom();
  } else {
    const unwatch = watch(containerSize, (newSize) => {
      if (newSize.width > 0) {
        applyZoom();
        unwatch();
      }
    });
  }
};

const rotateRight = () => {
  imageRotate.value[activeImage.value] += 90;
};

const updateZoomFit = () => {
  console.log('updateZoomFit');
  isZoomFit.value ? zoomFit() : zoomReset();

  // set the hide image to the same position
  const nextImageIndex = activeImage.value ^ 1;
  imageSrc.value[nextImageIndex] = '';
  position.value[nextImageIndex] = position.value[activeImage.value];
};

// Zoom to fit image in container
const zoomFit = () => {
  console.log('zoomFit');
  const container = containerSize.value;
  const imgRotatedSize = imageSizeRotated.value[activeImage.value];
  
  const containerAspectRatio = container.width / container.height;
  const imageAspectRatio = imgRotatedSize.width / imgRotatedSize.height;

  const scale = containerAspectRatio > imageAspectRatio 
    ? container.height / imgRotatedSize.height
    : container.width / imgRotatedSize.width;

  // set position to center
  zoomImage(container.width / 2, container.height / 2, scale);
};

// Reset zoom level and position
const zoomReset = () => {
  console.log('zoomReset');
  const mousePos = mousePosition.value;
  const containerPosVal = containerPos.value;
  zoomImage(mousePos.x - containerPosVal.x, mousePos.y - containerPosVal.y, 1);
};

// start dragging
const handleMouseDown = (event) => {
  console.log('handleMouseDown', event.button);
  event.preventDefault();

  if (event.button === 0) {     // left click: drag image
    isDragging.value = true;
    lastMousePosition.value = { x: event.clientX, y: event.clientY };
  } else if (event.button === 2) { // right click: toggle zoom fit
    // TODO: use context menu
    // isZoomFit.value = !isZoomFit.value;
    // updateZoomFit();
  } else if (event.button === 1) { // middle button
    emit('message-from-image', { message: 'showfileinfo' });
  } else if (event.button === 3) {  // back button
    emit('message-from-image-viewer', { message: 'prev' });
  } else if (event.button === 4) {  // forward button
    emit('message-from-image-viewer', { message: 'next' });
  } 
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
  const container = containerSize.value;
  mousePosition.value = { x: container.width / 2, y: container.height / 2 };
};

const updateDragPosition = () => {
  const event = latestMouseEvent.value;
  if (!event) return;

  const imgIndex = activeImage.value;
  const scaleVal = scale.value[imgIndex];
  const imageRotatedSize = imageSizeRotated.value[imgIndex];
  const container = containerSize.value;
  const lastPos = lastMousePosition.value;

  const scaledWidth = imageRotatedSize.width * scaleVal;
  const scaledHeight = imageRotatedSize.height * scaleVal;

  const deltaX = scaledWidth <= container.width ? 0 : event.clientX - lastPos.x;
  const deltaY = scaledHeight <= container.height ? 0 : event.clientY - lastPos.y;

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

  if (isTouchPad) {
    // accumulate delta values until they reach a threshold
    wheelDeltaAccumulator += event.deltaY;
    if (Math.abs(wheelDeltaAccumulator) < wheelThreshold) {
      return;
    }
    wheelDeltaAccumulator = 0;
  }

  const zoomFactor = isTouchPad ? 1 : 0.1; // Adjust sensitivity

  if (config.mouseWheelMode === 0) {  // 0: previous/next image
    if (event.ctrlKey) {     // ctrl + mouse wheel: zoom in / out
      wheelZoom(event, zoomFactor);
    } else {
      emit('message-from-image-viewer', { message: event.deltaY < 0 ? 'prev' : 'next' });
    }
  } else if (config.mouseWheelMode === 1) {  // 1: zoom in / out
    wheelZoom(event, zoomFactor);
  }
};

// wheel zoom
const wheelZoom = (event, zoomFactor) => {
  const container = containerSize.value;
  const imgRotatedSize = imageSizeRotated.value[activeImage.value];
  
  minScale.value = Math.min(
    0.1, 
    Math.min(
      container.width / imgRotatedSize.width, 
      container.height / imgRotatedSize.height
    ) * 0.5
  );

  // Clamp zoom level
  const newScale = Math.min(Math.max(scale.value[activeImage.value] / (1 + event.deltaY * zoomFactor / 100), minScale.value), maxScale.value);
  const containerPosVal = containerPos.value;
  zoomImage(event.clientX - containerPosVal.x, event.clientY - containerPosVal.y, newScale);
};

const zoomIn = () => {
  const newScale = Math.min(scale.value[activeImage.value] * 2, maxScale.value);
  const container = containerSize.value;
  zoomImage(container.width / 2, container.height / 2, newScale);
};

const zoomOut = () => {
  const container = containerSize.value;
  const imgRotatedSize = imageSizeRotated.value[activeImage.value];
  
  minScale.value = Math.min(
    0.1, 
    Math.min(
      container.width / imgRotatedSize.width, 
      container.height / imgRotatedSize.height
    ) * 0.5
  );

  const newScale = Math.max(scale.value[activeImage.value] / 2, minScale.value);
  zoomImage(container.width / 2, container.height / 2, newScale);
};

const zoomActual = () => {
  const container = containerSize.value;
  zoomImage(container.width / 2, container.height / 2, 1);
};

// Zoom image at cursor position
function zoomImage(cursorX, cursorY, newScale) {
  const imgIndex = activeImage.value;
  const currentScale = scale.value[imgIndex];
  const pos = position.value[imgIndex];
  const imgSize = imageSize.value[imgIndex];
  
  const imageOffsetX = ((currentScale - newScale) * ((cursorX - pos.x) - imgSize.width / 2)) / currentScale;
  const imageOffsetY = ((currentScale - newScale) * ((cursorY - pos.y) - imgSize.height / 2)) / currentScale;
  
  pos.x += imageOffsetX;
  pos.y += imageOffsetY;

  scale.value[imgIndex] = newScale;
  clampPosition();
}

// Ensure image stays within container
function clampPosition() {
  const imgIndex = activeImage.value;
  const imgRotatedSize = imageSizeRotated.value[imgIndex];
  const imgSize = imageSize.value[imgIndex];
  const scaleVal = scale.value[imgIndex];
  const container = containerSize.value;
  const pos = position.value[imgIndex];

  const paddingX = (imgRotatedSize.width * scaleVal - imgSize.width) / 2;
  const paddingY = (imgRotatedSize.height * scaleVal - imgSize.height) / 2;
  const maxX = container.width - imgRotatedSize.width * scaleVal + paddingX;
  const maxY = container.height - imgRotatedSize.height * scaleVal + paddingY;

  isGrabbing.value = false;
  if (imgRotatedSize.width * scaleVal > container.width) {
    pos.x = Math.min(Math.max(pos.x, maxX), paddingX);
    isGrabbing.value = true;
  } else {
    pos.x = (container.width - imgSize.width) / 2;
  }
  if (imgRotatedSize.height * scaleVal > container.height) {
    pos.y = Math.min(Math.max(pos.y, maxY), paddingY);
    isGrabbing.value = true;
  } else {
    pos.y = (container.height - imgSize.height) / 2;
  }
  triggerRef(position);
};

// Expose methods
defineExpose({ 
  zoomIn, 
  zoomOut,
  zoomActual,
  rotateRight
});

</script>
<template>
  <div
    ref="container"
    class="relative w-full h-full overflow-hidden cursor-pointer"
  >
    <!-- main image -->
    <img 
      v-for="(src, index) in imageSrc"
      v-show="activeImage === index"
      ref="activeImageEl"
      :key="`img-${index}`"
      :src="src"
      :class="isGrabbing ? (isDraggingImage ? 'cursor-grabbing' : 'cursor-grab') : 'cursor-pointer'"
      :style="{
        position: 'absolute',
        minWidth:  `${imageSize[index].width}px`,
        minHeight: `${imageSize[index].height}px`,
        transform: `translate(${position[index].x}px, ${position[index].y}px) 
                    scale(${scale[index]}) 
                    rotate(${imageRotate[index]}deg)`,
        transition: !isDraggingImage && !noTransition ? (isDraggingNavBox ? 'transform 0.2s ease-out' : 'transform 0.3s ease-in-out') : 'none',
        willChange: 'transform',
      }"
      draggable="false"
      @load="onImageLoad($event.target as HTMLImageElement)"
      @mousedown="handleImageMouseDown"
      @mousemove="handleImageMouseMove"
      @mouseup="handleImageMouseUp"
      @mouseleave="handleImageMouseLeave"
      @wheel="handleImageWheel"
      @dblclick="zoomFit"
    />

    <!-- Navigator view -->
    <transition name="fade">
      <!-- nav container -->
      <div v-if="(config.settings.navigatorViewMode === 0 && isGrabbing) || config.settings.navigatorViewMode === 1" 
        class="absolute right-4 bottom-4 outline outline-gray-50 overflow-hidden shadow-lg shadow-gray-500 z-20" 
        :style="navContainerStyle"
        @wheel="handleNavBoxWheel"
        @click="handleNavBoxClick"
        @dblclick="zoomFit"
      >
        <!-- nav image -->
        <img :src="imageSrc[activeImage]" :style="navImageStyle" draggable="false" />
        <!-- nav box -->
        <div class="absolute top-0 left-0 border-2 border-primary cursor-move" 
          :style="navBoxStyle" 
          @mousedown="handleNavBoxMouseDown"
          @mousemove="handleNavBoxMouseMove"
          @mouseup="handleNavBoxMouseUp"
          @mouseleave="handleNavBoxMouseLeave"
        ></div>
      </div>
    </transition>

    <!-- debug -->
    <!-- <div class="absolute top-4 right-4">
      <table class="text-sm text-base-content/30 border-separate border-spacing-2 bg-base-100 rounded-box p-2">
        <tbody>
          <tr>
            <td>containerSize</td>
            <td>{{ containerSize.width.toFixed(0) }} x {{ containerSize.height.toFixed(0) }}</td>
          </tr>
          <tr>
            <td>imageSize</td>
            <td>{{ imageSize[activeImage].width }} x {{ imageSize[activeImage].height }}</td>
          </tr>
          <tr>
            <td>imageSizeRotated</td>
            <td>{{ imageSizeRotated[activeImage].width }} x {{ imageSizeRotated[activeImage].height }}</td>
          </tr>
          <tr>
            <td>scale</td>
            <td>{{ scale[activeImage].toFixed(2) }}</td>
          </tr>
          <tr>
            <td>position</td>
            <td class="w-40">{{ position[activeImage].x.toFixed(2) }}, {{ position[activeImage].y.toFixed(2) }}</td>
          </tr>
          <tr>
            <td>mousePosition</td>
            <td>{{ mousePosition.x.toFixed(2) }}, {{ mousePosition.y.toFixed(2) }}</td>
          </tr>
        </tbody>
      </table>
    </div> -->

  </div>
</template>

<script setup lang="ts">
import { ref, shallowRef, triggerRef, watch, onMounted, onBeforeUnmount, computed } from 'vue';
import { emit } from '@tauri-apps/api/event';
import { config } from '@/common/config';

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

const isDraggingImage = ref(false);         // Dragging state
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

let resizeObserver: ResizeObserver | null = null;
let positionObserver: number | null = null;

// navigator view mode
const navContainerSize = computed(() => {
  const max_size = config.settings.navigatorViewSize;
  const aspectRatio = imageSizeRotated.value[activeImage.value].width / imageSizeRotated.value[activeImage.value].height;
  if(aspectRatio >= 1) {
    return {
      width: max_size,
      height: Math.round(max_size / aspectRatio),
    };
  } else {
    return {
      width: Math.round(max_size * aspectRatio),
      height: max_size,
    };
  }
});

// navigator container style
const navContainerStyle = computed(() => {
  return {
    width: `${navContainerSize.value.width}px`,
    height: `${navContainerSize.value.height}px`,
  };
});

// navigator image style
const navImageStyle = computed(() => {
  const rotation = imageRotate.value[activeImage.value];
  const imgSize = imageSize.value[activeImage.value];

  let scale;
  if (rotation % 180 !== 0) {
    scale = Math.min(navContainerSize.value.width / imgSize.height, navContainerSize.value.height / imgSize.width);
  } else {
    scale = Math.min(navContainerSize.value.width / imgSize.width, navContainerSize.value.height / imgSize.height);
  }

  return {
    minWidth: `${imgSize.width}px`,
    minHeight: `${imgSize.height}px`,
    position: 'absolute',
    left: `${(navContainerSize.value.width - imgSize.width) / 2}px`,
    top: `${(navContainerSize.value.height - imgSize.height) / 2}px`,
    transform: `scale(${scale}) rotate(${rotation}deg)`,
    transformOrigin: 'center center',
  };
});

// navigator box style
const navBoxStyle = computed(() => {
  const mainScale = scale.value[activeImage.value];
  const imgSize = imageSize.value[activeImage.value];
  const imgRotatedSize = imageSizeRotated.value[activeImage.value];
  const rotation = imageRotate.value[activeImage.value];
  const isRotated = rotation % 180 !== 0;

  if (!isGrabbing.value || mainScale <= 0 || imgSize.width === 0 || imgRotatedSize.width === 0) {
    return { display: 'none' };
  }

  let navScale;
  if (isRotated) {
    navScale = Math.min(navContainerSize.value.width / imgSize.height, navContainerSize.value.height / imgSize.width);
  } else {
    navScale = Math.min(navContainerSize.value.width / imgSize.width, navContainerSize.value.height / imgSize.height);
  }

  const finalW = isRotated ? (imgSize.height * navScale) : (imgSize.width * navScale);
  const worldRatio = finalW / imgRotatedSize.width;

  const boxWidth = (containerSize.value.width / mainScale) * worldRatio;
  const boxHeight = (containerSize.value.height / mainScale) * worldRatio;

  const cW = containerSize.value.width;
  const cH = containerSize.value.height;
  const iW = imageSize.value[activeImage.value].width;
  const iH = imageSize.value[activeImage.value].height;

  const posX = position.value[activeImage.value].x;
  const posY = position.value[activeImage.value].y;
  const rot = imageRotate.value[activeImage.value];

  // Find viewport center on un-scaled, un-rotated image content
  const V_x = (cW / 2) - (posX + iW / 2);
  const V_y = (cH / 2) - (posY + iH / 2);

  const V_scaled_x = V_x / mainScale;
  const V_scaled_y = V_y / mainScale;

  const angle = rot * Math.PI / 180;
  const cos_a = Math.cos(angle);
  const sin_a = Math.sin(angle);

  // CCW rotation to get coordinates in image's local system
  const p_content_center_x = V_scaled_x * cos_a - V_scaled_y * sin_a;
  const p_content_center_y = V_scaled_x * sin_a + V_scaled_y * cos_a;

  // Find this point's position in the navContainer
  const p_nav_scaled_x = p_content_center_x * navScale;
  const p_nav_scaled_y = p_content_center_y * navScale;

  // CW rotation to place point in the rotated navImage's system
  const p_final_x = p_nav_scaled_x * cos_a + p_nav_scaled_y * sin_a + navContainerSize.value.width / 2;
  const p_final_y = -p_nav_scaled_x * sin_a + p_nav_scaled_y * cos_a + navContainerSize.value.height / 2;

  // Calculate navBox top-left from its center
  const boxX = p_final_x - boxWidth / 2;
  const boxY = p_final_y - boxHeight / 2;

  return {
    width: `${boxWidth}px`,
    height: `${boxHeight}px`,
    transform: `translate(${boxX}px, ${boxY}px)`,
    boxShadow: `0 0 0 9999px color-mix(in srgb, var(--color-base-200) 20%, transparent)`,
  };
});

const isDraggingNavBox = ref(false);
const initialNavBoxClickPos = ref({ x: 0, y: 0 });
const isDraggingNavBoxMoved = ref(false);

const handleNavBoxMouseDown = (event: MouseEvent) => {
  event.preventDefault();
  event.stopPropagation();
  isDraggingNavBox.value = true;
  lastMousePosition.value = { x: event.clientX, y: event.clientY };
  initialNavBoxClickPos.value = { x: event.clientX, y: event.clientY }; // Record initial position
  isDraggingNavBoxMoved.value = false; // Reset moved flag
};

const handleNavBoxMouseMove = (event: MouseEvent) => {
  if (!isDraggingNavBox.value) return;

  // Check if mouse has moved significantly to consider it a drag
  const dx = event.clientX - initialNavBoxClickPos.value.x;
  const dy = event.clientY - initialNavBoxClickPos.value.y;
  if (Math.sqrt(dx * dx + dy * dy) > 5) { // Threshold of 5 pixels
    isDraggingNavBoxMoved.value = true;
  }

  // update mouse position
  mousePosition.value = { x: event.clientX, y: event.clientY };
  latestMouseEvent.value = event;

  // No need to run more than once per frame
  if (animationFrameId) return;

  animationFrameId = requestAnimationFrame(() => {
    updateNavBoxDragPosition();
    animationFrameId = null;
  });
};

const handleNavBoxMouseUp = () => {
  isDraggingNavBox.value = false;
  handleImageMouseLeave();
};

const handleNavBoxMouseLeave = () => {
  // reset mouse position to the center of the container
  const container = containerSize.value;
  mousePosition.value = { x: container.width / 2, y: container.height / 2 };
};

const handleNavBoxWheel = (event: WheelEvent) => {
  event.preventDefault();
  event.stopPropagation();

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

  wheelZoom(event, zoomFactor);
};

const handleNavBoxClick = (event: MouseEvent) => {
  event.preventDefault();
  event.stopPropagation();

  // Suppress click if it was a drag
  if (isDraggingNavBoxMoved.value) {
    isDraggingNavBoxMoved.value = false;
    return;
  }

  const imgIndex = activeImage.value;
  const mainScale = scale.value[imgIndex];
  const imgSize = imageSize.value[imgIndex];
  const container = containerSize.value;
  const rotation = imageRotate.value[imgIndex];

  // 1. Get click coordinates relative to navContainer
  const navContainerRect = (event.currentTarget as HTMLElement).getBoundingClientRect();
  const clickX_navContainer = event.clientX - navContainerRect.left;
  const clickY_navContainer = event.clientY - navContainerRect.top;

  // 2. Perform Inverse Transformation Chain
  const nav_cW = navContainerSize.value.width;
  const nav_cH = navContainerSize.value.height;
  const angle = rotation * Math.PI / 180;
  const cos_a = Math.cos(angle);
  const sin_a = Math.sin(angle);

  // Reverse CW rotation (apply CCW rotation to vector from navContainer center)
  const p_final_relative_x = clickX_navContainer - nav_cW / 2;
  const p_final_relative_y = clickY_navContainer - nav_cH / 2;

  const p_nav_scaled_x = p_final_relative_x * cos_a + p_final_relative_y * sin_a; // CCW rotation
  const p_nav_scaled_y = -p_final_relative_x * sin_a + p_final_relative_y * cos_a; // CCW rotation

  // Recalculate navScale (same logic as in navBoxStyle)
  const isRotated = rotation % 180 !== 0;
  let actualNavScale;
  if (isRotated) {
    actualNavScale = Math.min(nav_cW / imgSize.height, nav_cH / imgSize.width);
  } else {
    actualNavScale = Math.min(nav_cW / imgSize.width, nav_cH / imgSize.height);
  }

  // Reverse scaling by navScale
  const p_content_center_x = p_nav_scaled_x / actualNavScale;
  const p_content_center_y = p_nav_scaled_y / actualNavScale;

  // Reverse CCW rotation (apply CW rotation to p_content_center)
  const V_scaled_x = p_content_center_x * cos_a - p_content_center_y * sin_a; // CW rotation
  const V_scaled_y = p_content_center_x * sin_a + p_content_center_y * cos_a; // CW rotation

  // Reverse scaling by mainScale
  const V_x = V_scaled_x * mainScale;
  const V_y = V_scaled_y * mainScale;

  // Calculate new posX, posY
  const cW = container.width;
  const cH = container.height;
  const iW = imgSize.width;
  const iH = imgSize.height;

  const newPosX = (cW / 2) - V_x - (iW / 2);
  const newPosY = (cH / 2) - V_y - (iH / 2);

  position.value[imgIndex] = { x: newPosX, y: newPosY };
  clampPosition();
};

const updateNavBoxDragPosition = () => {
  const event = latestMouseEvent.value;
  const imgIndex = activeImage.value;
  const imgSize = imageSize.value[imgIndex];
  if (!event || imgSize.width === 0) return;

  const d_box_x = event.clientX - lastMousePosition.value.x;
  const d_box_y = event.clientY - lastMousePosition.value.y;

  const rotation = imageRotate.value[imgIndex];
  const isRotated = rotation % 180 !== 0;
  let navScale;
  if (isRotated) {
    navScale = Math.min(navContainerSize.value.width / imgSize.height, navContainerSize.value.height / imgSize.width);
  } else {
    navScale = Math.min(navContainerSize.value.width / imgSize.width, navContainerSize.value.height / imgSize.height);
  }

  if (navScale > 0) {
    const mainScale = scale.value[imgIndex];
    const d_pos_x = - (d_box_x / navScale) * mainScale;
    const d_pos_y = - (d_box_y / navScale) * mainScale;
    
    position.value[imgIndex].x += d_pos_x;
    position.value[imgIndex].y += d_pos_y;
  }

  lastMousePosition.value = { x: event.clientX, y: event.clientY };
  clampPosition();
};

onMounted(() => {
  // observe container size changes
  resizeObserver = new ResizeObserver(entries => {
    for (let entry of entries) {
      containerSize.value = {
        width: entry.contentRect.width,
        height: entry.contentRect.height,
      };
      mousePosition.value = { x: entry.contentRect.width / 2, y: entry.contentRect.height / 2 };
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
    const rect = (container.value as HTMLElement).getBoundingClientRect();
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
let debounceTimeout: NodeJS.Timeout | null = null;
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
const onImageLoad = async (img: HTMLImageElement) => {
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
    // If not in "zoom to fit" mode, perform a calculated zoom to the cursor position.
    if (!isZoomFit.value) {
      const imgIndex = activeImage.value;
      const imgSize = imageSize.value[imgIndex];
      const container = containerSize.value;
      
      // Use current mouse position relative to container
      const cursorX = mousePosition.value.x - containerPos.value.x;
      const cursorY = mousePosition.value.y - containerPos.value.y;

      // Calculate a conceptual "before" state, as if the image was fitted to the container.
      const fitScale = Math.min(container.width / imgSize.width, container.height / imgSize.height);
      const initialPos = {
        x: (container.width - imgSize.width) / 2,
        y: (container.height - imgSize.height) / 2,
      };

      // Now, use the logic from zoomImage to transition from the "fit" state to the 100% state.
      const newScale = 1;
      const imageOffsetX = ((fitScale - newScale) * ((cursorX - initialPos.x) - imgSize.width / 2)) / fitScale;
      const imageOffsetY = ((fitScale - newScale) * ((cursorY - initialPos.y) - imgSize.height / 2)) / fitScale;
      
      scale.value[imgIndex] = newScale;
      position.value[imgIndex] = {
        x: initialPos.x + imageOffsetX,
        y: initialPos.y + imageOffsetY,
      };
      triggerRef(position);
      clampPosition();

      // Also update the other image's position to match for smooth transitions
      const otherImageIndex = activeImage.value ^ 1;
      imageSrc.value[otherImageIndex] = '';
      position.value[otherImageIndex] = position.value[activeImage.value];
    } else {
      // For isZoomFit, the original logic is fine.
      updateZoomFit();
    }

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
const handleImageMouseDown = (event: MouseEvent) => {
  event.preventDefault();

  if (event.button === 0) {     // left click: drag image
    isDraggingImage.value = true;
    lastMousePosition.value = { x: event.clientX, y: event.clientY };
  } else if (event.button === 2) { // right click: toggle zoom fit
    // TODO: use context menu
    // isZoomFit.value = !isZoomFit.value;
    // updateZoomFit();
  } else if (event.button === 1) { // middle button
    // emit('message-from-image', { message: 'showInfoPanel' });
  } else if (event.button === 3) {  // back button
    emit('message-from-image-viewer', { message: 'prev' });
  } else if (event.button === 4) {  // forward button
    emit('message-from-image-viewer', { message: 'next' });
  } 
};

const handleImageMouseMove = (event: MouseEvent) => {
  // update mouse position
  mousePosition.value = { x: event.clientX, y: event.clientY };

  if (!isDraggingImage.value) return;

  latestMouseEvent.value = event;

  if (animationFrameId) {
    cancelAnimationFrame(animationFrameId);
  }

  animationFrameId = requestAnimationFrame(updateDragPosition);
};

// stop dragging
const handleImageMouseUp = () => {
  isDraggingImage.value = false;
};

// mouse leave
// reset mouse position to the center when leaving the container
const handleImageMouseLeave = () => {
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
const handleImageWheel = (event: WheelEvent) => {
  event.preventDefault();
  event.stopPropagation();

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

  if (config.settings.mouseWheelMode === 0) {  // 0: previous/next image
    if (event.ctrlKey) {     // ctrl + mouse wheel: zoom in / out
      wheelZoom(event, zoomFactor);
    } else {
      emit('message-from-image-viewer', { message: event.deltaY < 0 ? 'prev' : 'next' });
    }
  } else if (config.settings.mouseWheelMode === 1) {  // 1: zoom in / out
    wheelZoom(event, zoomFactor);
  }
};

// wheel zoom
const wheelZoom = (event: WheelEvent, zoomFactor: number) => {
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
function zoomImage(cursorX: number, cursorY: number, newScale: number) {
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
  if (Math.floor(imgRotatedSize.width * scaleVal) > container.width) {
    pos.x = Math.min(Math.max(pos.x, maxX), paddingX);
    isGrabbing.value = true;
  } else {
    pos.x = (container.width - imgSize.width) / 2;
  }
  if (Math.floor(imgRotatedSize.height * scaleVal) > container.height) {
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
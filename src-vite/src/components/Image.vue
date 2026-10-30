<template>
  <div
    ref="container"
    class="relative isolate w-full h-full overflow-hidden cursor-pointer"
    style="touch-action: none;"
    @wheel="handleImageWheel"
  >

    <!-- Loading overlay -->
    <transition name="fade">
      <div
        v-if="isLoading && !showInlineLoading && Number(fileType) !== 3"
        class="absolute inset-0 bg-base-100/50 flex items-center justify-center z-50 rounded-box"
      >
        <span class="loading loading-dots text-primary"></span>
      </div>
    </transition>

    <transition name="fade">
      <div
        v-if="isLoading && showInlineLoading && Number(fileType) !== 3"
        class="absolute left-2.5 bottom-2.5 z-50 pointer-events-none"
      >
        <span class="loading loading-spinner loading-xs text-primary/70"></span>
      </div>
    </transition>

    <div v-if="Number(fileType) === 3 && !offlinePreview" data-raw-preview-controls class="absolute left-4 bottom-4 z-60 flex items-center gap-2" @dblclick.stop @mousedown.stop>
      <button 
        class="inline-flex h-10 items-center gap-1 rounded-box px-3 bg-base-100/70 hover:bg-base-100 text-sm font-medium shadow transition-colors cursor-pointer focus:outline-none"
        :class="effectiveRawSource !== 'pair' && (rawSourceIsOverride || (rawPairLabel && config.settings.groupRawJpegPairs)) ? 'text-primary hover:text-primary' : 'text-base-content/70 hover:text-base-content'" :aria-pressed="effectiveRawSource !== 'pair'" :title="rawSwitchTitle" @click.stop="switchRaw">
        <span>RAW</span>
        <span v-if="rawSourceLabel && !rawLoadingVisible" class="inline-flex items-center gap-1">
          <span>· {{ rawSourceLabel }}</span>
          <component :is="rawModeSource === 'embedded' ? IconEmbedded : rawModeSource === 'brightened' ? IconBrightnessUp : IconBrightness" class="w-5 h-5" aria-hidden="true" />
        </span>
        <span v-if="rawLoadingVisible" class="loading loading-spinner loading-xs" aria-hidden="true"></span>
      </button>
      <button v-if="rawPairLabel && config.settings.groupRawJpegPairs" 
        class="inline-flex h-10 items-center rounded-box px-3 bg-base-100/70 hover:bg-base-100 text-sm font-medium shadow transition-colors cursor-pointer focus:outline-none"
        :class="effectiveRawSource === 'pair' ? 'text-primary hover:text-primary' : 'text-base-content/70 hover:text-base-content'" :aria-pressed="effectiveRawSource === 'pair'" :title="t('settings.raw.show_pair', { format: rawPairLabel })" @click.stop="selectRawPair">{{ rawPairLabel }}</button>
    </div>

    <!-- Error overlay -->
    <transition name="fade">
      <div v-if="loadError" class="absolute inset-0 bg-base-100/50 flex items-center justify-center z-50 rounded-box">
        <div class="h-full flex flex-col items-center justify-center text-base-content/30">
          <IconError class="w-8 h-8 mb-2" />
          <span>{{ $t(offlinePreview ? 'tooltip.not_found.files' : 'image_viewer.failed') }}</span>
        </div>
      </div>
    </transition>

    <!-- main image -->
    <TransitionGroup :name="transitionName" @after-leave="handleTransitionEnd">
      <div 
        v-for="(src, index) in imageSrc"
        v-show="activeImage === index"
        :key="`img-${imageFilePath[index] || 'empty'}-${index}`"
        class="slide-wrapper absolute inset-0 w-full h-full pointer-events-none overflow-hidden"
      >
        <img
          ref="activeImageEl"
          :src="src"
          :class="isGrabbing ? (isDraggingImage ? 'cursor-grabbing' : 'cursor-grab') : 'cursor-pointer'"
          :style="getImageStyle(index)"
          draggable="false"
          @mousedown="handleImageMouseDown"
          @mousemove="handleImageMouseMove"
          @mouseup="handleImageMouseUp"
          @mouseleave="handleImageMouseLeave"
          @dblclick.stop="toggleZoomFit"
          @contextmenu.prevent.stop="$emit('context-menu', $event)"
        />
      </div>
    </TransitionGroup>

    <!-- AF areas share the image's exact layout, zoom and rotation transform. -->
    <div v-if="showAfAreas && !loadError && imageFilePath[activeImage] === filePath"
      class="absolute inset-0 pointer-events-none overflow-hidden" aria-hidden="true">
      <div :style="{ ...getImageStyle(activeImage), pointerEvents: 'none', filter: 'none' }">
        <div v-for="(region, index) in afAreas" :key="index"
          class="absolute box-border"
          :style="{
            left: `${region.x * 100}%`, top: `${region.y * 100}%`,
            width: `${region.width * 100}%`, height: `${region.height * 100}%`,
            border: `${2 / scale[activeImage]}px solid ${region.focused ? '#22c55e' : '#ef4444'}`,
          }" />
      </div>
    </div>

    <!-- Faces Overlay -->
    <div 
      v-if="showFaceOverlay && faces.length > 0 && !isDraggingImage && !isSlideShow"
      class="absolute inset-0 w-full h-full pointer-events-none overflow-hidden"
    >
      <div
        class="absolute"
        :style="{
          width: `${imageSize[activeImage].width}px`,
          height: `${imageSize[activeImage].height}px`,
          transform: `translate(${position[activeImage].x}px, ${position[activeImage].y}px) 
                      scale(${scale[activeImage]}) 
                      rotate(${imageRotate[activeImage]}deg)`,
          transition: !isDraggingImage && !noTransition && !isResizingContainer ? (isDraggingNavBox ? 'transform 0.2s ease-out' : 'transform 0.3s ease-in-out') : 'none',
        }"
      >
        <div
          v-for="face in faces"
          :key="face.id"
          v-show="face.person_id === libConfig.person.id"
          class="absolute group"
          :style="{
            left: `${face.bbox.x}px`,
            top: `${face.bbox.y}px`,
            width: `${face.bbox.width}px`,
            height: `${face.bbox.height}px`,
          }"
        >
          <!-- Corner: Top Left -->
          <div 
            class="absolute top-0 left-0 w-1/8 h-1/8 border-secondary/50 rounded-tl-lg"
            :style="{ 
              borderTopWidth: `${2 / scale[activeImage]}px`, 
              borderLeftWidth: `${2 / scale[activeImage]}px` 
            }"
          ></div>
          
          <!-- Corner: Top Right -->
          <div 
            class="absolute top-0 right-0 w-1/8 h-1/8 border-secondary/50 rounded-tr-lg"
            :style="{ 
              borderTopWidth: `${2 / scale[activeImage]}px`, 
              borderRightWidth: `${2 / scale[activeImage]}px` 
            }"
          ></div>

          <!-- Corner: Bottom Left -->
          <div 
            class="absolute bottom-0 left-0 w-1/8 h-1/8 border-secondary/50 rounded-bl-lg"
            :style="{ 
              borderBottomWidth: `${2 / scale[activeImage]}px`, 
              borderLeftWidth: `${2 / scale[activeImage]}px` 
            }"
          ></div>

          <!-- Corner: Bottom Right -->
          <div 
            class="absolute bottom-0 right-0 w-1/8 h-1/8 border-secondary/50 rounded-br-lg"
            :style="{ 
              borderBottomWidth: `${2 / scale[activeImage]}px`, 
              borderRightWidth: `${2 / scale[activeImage]}px` 
            }"
          ></div>

          <!-- Person Name Tag (Hover) -->
          <!-- <div 
            class="hidden group-hover:flex absolute left-1/2 -translate-x-1/2 
                   bg-black/60 backdrop-blur-sm text-white text-xs font-medium 
                   px-2 py-1 rounded shadow-sm whitespace-nowrap z-10 pointer-events-none"
            :style="{
              bottom: '100%',
              marginBottom: `${8 / scale[activeImage]}px`,
              transform: `translateX(-50%) scale(${1 / scale[activeImage]})`,
              transformOrigin: 'bottom center',
            }"
          >
            {{ face.person_id ? `Person ${face.person_id}` : 'Unknown' }}
          </div> -->
        </div>
      </div>
    </div>

    <!-- Navigator view -->
    <transition name="fade">
      <div v-if="showNavigator" class="absolute right-4 bottom-4 z-20"
        :style="{ width: `${navContainerSize.width}px`, height: `${navContainerSize.height}px` }">
      <ImageNavigator
        class="h-full w-full outline outline-gray-50 shadow-lg shadow-gray-500"
        :style="{ width: `${navContainerSize.width}px`, height: `${navContainerSize.height}px` }"
        :source="displayThumbnailSrc || getThumbUrl(props.fileId, false, config.settings.thumbnailSize, props.fileVersion)"
        :viewport="navigatorViewport"
        :sharpness="config.settings.navigatorSharpnessGrid"
        @mouseenter="pauseNavigatorAutoHide"
        @mouseleave="resetNavigatorAutoHide"
        @navigate="navigateImage"
        @zoom="zoomNavigator"
        @toggle-fit="toggleZoomFit"
        @dragging="isDraggingNavBox = $event"
      />
      </div>
    </transition>

  </div>
</template>

<script setup lang="ts">
import { ref, shallowRef, triggerRef, watch, onMounted, onBeforeUnmount, computed, nextTick } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { useUIStore } from '@/stores/uiStore';
import { config, libConfig } from '@/common/config';
import { SIDEBAR, GESTURE_LOCK_THRESHOLD, HORIZONTAL_NAV_THRESHOLD, MOUSE_DRAG_NAV_THRESHOLD } from '@/common/constants';
import {
  getAssetSrc,
  getPreviewUrl,
  shouldUseBackendPreview,
  getFileExtension,
  getThumbUrl,
  getThumbnailDataUrl,
  getThumbnailDataUrlInflight,
  isWin,
  setThumbnailDataUrlInflight,
} from '@/common/utils';
import { getFacesForFile, getFileThumbById, getFfmpegBackedImageExtensions } from '@/common/api';
import { RawFace, Face } from '@/common/types';
import { rawDisplayKey, getRawDisplayOptions, appendRawDisplayParams, nextRawPreviewMode, type RawPreviewSource, type RawDisplayOptions } from '@/common/rawDisplay';
import { useI18n } from 'vue-i18n';
import { useToast } from '@/common/toast';
import { createDragPreview, isWindowDragEdge, startNativeFileDrag } from '@/common/nativeDrag';

import { checkFileAccessibility, isCloudOnlyFile } from '@/common/api';
import { setFileAccessibility } from '@/common/availability';
import { IconError, IconBrightness, IconBrightnessUp, IconEmbedded } from '@/common/icons';
import ImageNavigator from '@/components/ImageNavigator.vue';

const { t } = useI18n();
const toast = useToast();
// Props
const props = defineProps({
  showAfAreas: { type: Boolean, default: false },
  originalUnavailable: { type: Boolean, default: false },
  filePath: {
    type: String,
    required: false,
  },
  nextFilePath: {
    type: String,
    default: '',
  },
  rotate: {
    type: Number,
    default: 0,
  },  
  isZoomFit: {
    type: Boolean,
    default: false,
  },
  isSlideShow: {
    type: Boolean,
    default: false,
  },
  slideShowTransitionMode: {
    type: Number,
    default: 0,
  },
  fileId: {
    type: Number,
    required: false,
  },
  fileType: {
    type: Number,
    default: 1,
  },
  rawPairPath: {
    type: String,
    default: '',
  },
  fileVersion: {
    type: Number,
    default: 0,
  },
  imageWidth: {
    type: Number,
    default: 0,
  },
  imageHeight: {
    type: Number,
    default: 0,
  },
  thumbnailSrc: {
    type: String,
    default: '',
  },
  showThumbnailPlaceholder: {
    type: Boolean,
    default: false,
  },
  suppressAutoNavigator: { type: Boolean, default: false },
  showInlineLoading: {
    type: Boolean,
    default: false,
  },
});

const emit = defineEmits(['message-from-image-viewer', 'scale', 'update:isZoomFit', 'viewport-change', 'context-menu']);

type AfArea = { x: number; y: number; width: number; height: number; focused: boolean };
const afAreas = ref<AfArea[]>([]);
let afAreaRequest = 0;
let afAreaTimer: ReturnType<typeof setTimeout> | undefined;
let afAreaBusy = false;
let pendingAfArea: { request: number; filePath: string } | null = null;
const loadAfAreas = async () => {
  if (afAreaBusy || !pendingAfArea) return;
  const target = pendingAfArea;
  pendingAfArea = null;
  afAreaBusy = true;
  try {
    const regions = await invoke<AfArea[]>('get_af_areas', { filePath: target.filePath });
    if (target.request === afAreaRequest) afAreas.value = regions;
  } catch {
    // Unsupported or unreadable metadata must never interrupt image viewing.
  } finally {
    afAreaBusy = false;
    // Only the latest settled image may follow an in-flight request.
    if (pendingAfArea) void loadAfAreas();
  }
};
watch(() => [props.showAfAreas, props.filePath, props.fileVersion, props.originalUnavailable], () => {
  const request = ++afAreaRequest;
  clearTimeout(afAreaTimer);
  pendingAfArea = null;
  afAreas.value = [];
  if (!props.showAfAreas || !props.filePath || props.originalUnavailable) return;
  const filePath = props.filePath;
  // Holding an arrow key must not launch an unbounded number of RAW parses.
  afAreaTimer = setTimeout(() => {
    pendingAfArea = { request, filePath };
    void loadAfAreas();
  }, 120);
}, { immediate: true });
onBeforeUnmount(() => {
  ++afAreaRequest;
  clearTimeout(afAreaTimer);
  pendingAfArea = null;
});

const uiStore = useUIStore();

// container
const container = ref(null);
const containerSize = ref({ width: 0, height: 0 });
const containerPos = ref({ x: 0, y: 0 });
const isZoomFit = ref(props.isZoomFit);     // Zoom to fit image in container

// image
const activeImage = ref(1);                 // which image is active (0 or 1)
const imageSrc = ref(['', '']);             // image source
const imageFilePath = ref(['', '']);        // source file path for each buffer image
const position = shallowRef([{ x: 0, y: 0 }, { x: 0, y: 0 }]); // Image position (top-left corner)
const scale = ref([1, 1]);                  // Image scale (zoom level)
const minScale = ref(0.1);                    // Minimum zoom level
const maxScale = ref(10);                   // Maximum zoom level
const getActualSizeScale = () => 1 / (window.devicePixelRatio || 1);
const getDisplayScale = (scaleValue: number) => scaleValue * (window.devicePixelRatio || 1);
const imageRotate = ref([0, 0]);            // Image rotation
const imageNaturalSize = ref([{ width: 0, height: 0 }, { width: 0, height: 0 }]);
const imageSize = ref([{ width: 0, height: 0 }, { width: 0, height: 0 }]);       // actual image size
const imageSizeRotated = ref([{ width: 0, height: 0 }, { width: 0, height: 0 }]); // image size after rotation

const isDraggingImage = ref(false);         // Dragging state
const isGrabbing = ref(false);              // Grabbing state
const navigatorAutoVisible = ref(true);
const showNavigator = computed(() =>
  config.settings.navigatorViewMode === 1
  || (config.settings.navigatorViewMode === 0 && !props.suppressAutoNavigator && isGrabbing.value && navigatorAutoVisible.value)
);
let navigatorAutoHideTimer: ReturnType<typeof setTimeout> | null = null;
const isResizingContainer = ref(false);
let resizeTransitionFrame = 0;
const noTransition = ref(false);            // Disable transition temporarily
const lastMousePosition = ref({ x: 0, y: 0 }); // Last mouse position for drag calculations
const mousePosition = ref({ x: 0, y: 0 });  // Current mouse position
const mouseDragNavDeltaX = ref(0);
const mouseDragNavDeltaY = ref(0);
const mouseDragNavTriggered = ref(false);

const faces = ref<any[]>([]); // Store faces for the current image
const showFaceOverlay = computed(() =>
  config.settings.face.enabled && config.main.sidebarIndex === SIDEBAR.PERSON
);

let animationFrameId: number | null = null;
const latestMouseEvent = ref<MouseEvent | null>(null);

// macOS touchpad wheel - accumulate delta values until they reach a threshold
let wheelDeltaAccumulator = 0;
let wheelThreshold = 10;
// Improved gesture handling for touchpad
const gestureType = ref<'none' | 'zoom' | 'nav'>('none');
let horizontalDeltaAccumulator = 0;
let verticalDeltaAccumulator = 0;
let gestureResetTimeout: NodeJS.Timeout | null = null;
let hasNavigatedThisGesture = false;
let lastDeltaX = 0;

const isWheelZooming = ref(false);
let wheelZoomTimeout: NodeJS.Timeout | null = null;

// Touchpad detection - sticky once detected
let isTouchpadDevice = false;

// Touchscreen gesture state
const activeTouchPointers = new Map<number, { x: number; y: number }>();
let pinchStartDistance = 0;
let pinchStartScale = 1;
let pinchCenter = { x: 0, y: 0 };
let isPinching = false;
// Single-finger pan
let panPointerId: number | null = null;
let panLastPos = { x: 0, y: 0 };
// Single-finger swipe navigation (only used when image fits / cannot pan)
let touchSwipeStart = { x: 0, y: 0 };
let touchSwipeTriggered = false;
// Suppresses synthesized mouse events while any touch is active so the
// existing mouse drag handler doesn't double-pan with our pointer logic.
const isTouchActive = ref(false);

// Swipe state
const navDirection = ref<'next' | 'prev' | ''>('');
const isSliding = computed(() => {
  if (props.isSlideShow) {
    return props.slideShowTransitionMode === 0;
  }
  return navDirection.value !== '';
});
const transitionName = computed(() => {
  if (props.isSlideShow) {
    if (props.slideShowTransitionMode === 1) return 'slideshow-fade';
    if (props.slideShowTransitionMode === 2) return '';
    return 'slide-next';
  }
  if (navDirection.value) {
    return navDirection.value === 'next' ? 'slide-next' : 'slide-prev';
  }
  return '';
});

const getImageStyle = (index: number) => ({
  position: 'absolute',
  // RAW placeholders and full previews can have different intrinsic sizes.
  // Use the same layout box as the centering, zoom and rotation calculations.
  width: `${imageSize.value[index].width}px`,
  height: `${imageSize.value[index].height}px`,
  maxWidth: 'none', // Override Tailwind's img max-width: 100%; zoom handles fitting.
  transform: `translate3d(${position.value[index].x}px, ${position.value[index].y}px, 0)
              scale(${scale.value[index]})
              rotate(${imageRotate.value[index]}deg)`,
  transition: !isSliding.value && !isDraggingImage.value && !noTransition.value && !isResizingContainer.value && !isWheelZooming.value
    ? (isDraggingNavBox.value ? 'transform 0.2s ease-out' : 'transform 0.3s ease-in-out')
    : 'none',
  willChange: 'transform',
  backfaceVisibility: 'hidden',
  pointerEvents: 'auto',
  filter: adjustmentStyle.value(index === activeImage.value ? imageSrc.value[activeImage.value] : imageSrc.value[index]),
});
// loading and error overlays
const isLoading = ref(false);
const loadError = ref(false);
const offlinePreview = ref(false);
let loadingTimeout: NodeJS.Timeout | null = null;

// Show the RAW spinner only after loading has persisted for a moment, so fast
// loads don't flash it. Resets immediately when loading finishes.
const rawLoadingVisible = ref(false);
const currentLoadingId = ref(0);
let rawLoadingTimer: ReturnType<typeof setTimeout> | null = null;
watch([isLoading, currentLoadingId], ([loading]) => {
  if (rawLoadingTimer) { clearTimeout(rawLoadingTimer); rawLoadingTimer = null; }
  rawLoadingVisible.value = false;
  if (loading) {
    rawLoadingTimer = setTimeout(() => { rawLoadingVisible.value = true; rawLoadingTimer = null; }, 200);
  }
});

const activeImageEl = ref<HTMLImageElement | null>(null);
type LoadedImage = { src: string; naturalWidth: number; naturalHeight: number; raw?: { source: RawPreviewSource; unavailable: boolean; pair: string | null } };
const preloadCache = new Map<string, Promise<LoadedImage>>();
const rawOverride = ref<RawDisplayOptions | null>(null);
const rawSource = ref<RawPreviewSource>('');
const rawRequestPending = ref(false);
const rawSelectionVersion = ref(0);
const resolvedRawPairLabel = ref<string | null>(null);
const rawPairLabel = computed(() => resolvedRawPairLabel.value ?? (props.rawPairPath ? /\.(heic|heif|hif)$/i.test(props.rawPairPath) ? 'HEIC' : 'JPEG' : ''));
const rawEmbeddedUnavailable = ref(false);
const rawFullSize = ref(false);
let rawUpgradeFailed = false;
let rawUpgradeRetryAt = 0;
const rawViewerId = crypto.randomUUID();
let rawUpgradeTimer: ReturnType<typeof setTimeout> | null = null;
const requestedRawOptions = computed(() => rawOverride.value || getRawDisplayOptions());
// The RAW's own preview mode, independent of whether the paired JPEG/HEIC is
// the source currently on screen — the RAW button always shows this mode.
const rawModeSource = computed<'embedded' | 'rendered' | 'brightened'>(() => {
  const o = requestedRawOptions.value;
  const actual = rawSource.value;
  // While a mode switch is in flight, show the requested mode right away (the
  // spinner already signals loading) instead of waiting for the response.
  const switching = rawRequestPending.value && !!rawOverride.value;
  if (!switching && (actual === 'embedded' || actual === 'rendered' || actual === 'brightened')) {
    return actual;
  }
  if (o.mode === 'embedded' && !rawEmbeddedUnavailable.value) return 'embedded';
  return o.autoBright ? 'brightened' : 'rendered';
});
// What is actually on screen: the pair (JPEG/HEIC) or the RAW. Drives which
// button is highlighted. Prefer the response-confirmed source; before it
// arrives, predict from the requested options (only when a pair exists).
const effectiveRawSource = computed<RawPreviewSource>(() => {
  const actual = rawSource.value;
  const o = requestedRawOptions.value;
  if (actual === 'pair' || (o.preferPair && !!rawPairLabel.value)) return 'pair';
  return rawModeSource.value;
});
const rawSourceLabel = computed(() => rawSource.value === 'image' ? '' : rawSource.value || rawOverride.value ? t(`settings.raw.source_${rawModeSource.value}`) : '');
const rawSourceIsOverride = computed(() => {
  const defaults = getRawDisplayOptions();
  const defaultSource = defaults.preferPair && rawPairLabel.value ? 'pair'
    : defaults.mode === 'embedded' ? 'embedded' : defaults.autoBright ? 'brightened' : 'rendered';
  return effectiveRawSource.value !== defaultSource;
});
const nextRawMode = computed(() => {
  const requested = requestedRawOptions.value;
  const current = rawRequestPending.value && rawOverride.value
    ? requested.preferPair ? 'pair' : requested.mode === 'embedded' && !rawEmbeddedUnavailable.value ? 'embedded' : requested.autoBright ? 'brightened' : 'rendered'
    : rawSource.value;
  if (effectiveRawSource.value === 'pair') return rawModeSource.value;
  return nextRawPreviewMode(current === 'image' ? rawModeSource.value : current, rawEmbeddedUnavailable.value, requested);
});
const rawSwitchTitle = computed(() => t('settings.raw.switch_to', { source: t(`settings.raw.source_${nextRawMode.value}`) }));
function switchRaw() {
  const mode = nextRawMode.value;
  rawRequestPending.value = true;
  rawSelectionVersion.value++;
  rawOverride.value = { mode: mode === 'embedded' ? 'embedded' : 'rendered', autoBright: mode === 'embedded' ? getRawDisplayOptions().autoBright : mode === 'brightened', preferPair: false };
}
function selectRawPair() {
  if (rawSource.value === 'pair' && !rawRequestPending.value) return;
  rawRequestPending.value = true;
  rawSelectionVersion.value++;
  rawOverride.value = { ...requestedRawOptions.value, preferPair: true };
}
let rawAbortController: AbortController | null = null;
const rawObjectUrls = new Set<string>();
watch(() => props.filePath, () => {
  rawOverride.value = null;
  rawSource.value = '';
  resolvedRawPairLabel.value = null;
  rawEmbeddedUnavailable.value = false;
  rawRequestPending.value = false;
  rawFullSize.value = false;
}, { flush: 'sync' });
watch([rawDisplayKey, rawSelectionVersion, () => props.fileVersion, () => props.filePath], () => {
  rawFullSize.value = false;
  rawUpgradeFailed = false;
  rawUpgradeRetryAt = 0;
}, { flush: 'sync' });
watch(rawDisplayKey, () => { rawOverride.value = null; }, { flush: 'sync' });

async function loadRawImage(filePath: string, fullSize = false): Promise<LoadedImage> {
  rawAbortController?.abort();
  const controller = new AbortController();
  rawAbortController = controller;
  const url = new URL(getPreviewUrl(props.fileId, filePath, false, props.fileVersion));
  const options = requestedRawOptions.value;
  appendRawDisplayParams(url.searchParams, options);
  url.searchParams.set('rawViewerId', rawViewerId);
  if (fullSize) url.searchParams.set('rawFullSize', 'true');
  const response = await fetch(url.toString(), { signal: controller.signal });
  if (!response.ok) throw new Error('RAW preview failed');
  const blob = await response.blob();
  if (controller.signal.aborted) throw new Error('RAW preview cancelled');
  const src = URL.createObjectURL(blob);
  rawObjectUrls.add(src);
  try {
    const img = new Image();
    img.src = src;
    await img.decode();
    if (controller.signal.aborted) throw new Error('RAW preview cancelled');
    return { src, naturalWidth: img.naturalWidth, naturalHeight: img.naturalHeight, raw: {
      source: (response.headers.get('X-Raw-Source') || '') as RawPreviewSource,
      unavailable: response.headers.get('X-Raw-Embedded-Unavailable') === 'true',
      pair: options.preferPair ? response.headers.get('X-Raw-Pair') || '' : null,
    } };
  } catch (error) {
    URL.revokeObjectURL(src);
    rawObjectUrls.delete(src);
    throw error;
  }
}

watch(() => [scale.value[activeImage.value], rawSource.value, rawRequestPending.value, containerSize.value.width, containerSize.value.height], (current, previous) => {
  if (rawUpgradeTimer) clearTimeout(rawUpgradeTimer);
  if (rawUpgradeRetryAt && Date.now() >= rawUpgradeRetryAt
    && [0, 3, 4].some(index => current[index] !== previous[index])) {
    rawUpgradeFailed = false;
    rawUpgradeRetryAt = 0;
  }
  const index = activeImage.value;
  const natural = imageNaturalSize.value[index];
  const layout = imageSize.value[index];
  const pixelScale = scale.value[index] * (window.devicePixelRatio || 1);
  if (Number(props.fileType) !== 3 || !['rendered', 'brightened'].includes(rawSource.value)
    || rawFullSize.value || rawUpgradeFailed || rawRequestPending.value || isLoading.value
    || imageFilePath.value[index] !== props.filePath || props.originalUnavailable
    || (layout.width * pixelScale <= natural.width && layout.height * pixelScale <= natural.height)) return;
  const loadingId = currentLoadingId.value;
  const selectionVersion = rawSelectionVersion.value;
  const filePath = String(props.filePath);
  rawUpgradeTimer = setTimeout(async () => {
    rawUpgradeTimer = null;
    if (loadingId !== currentLoadingId.value || selectionVersion !== rawSelectionVersion.value || isLoading.value || rawRequestPending.value) return;
    rawRequestPending.value = true;
    try {
      const loaded = await loadRawImage(filePath, true);
      if (loadingId !== currentLoadingId.value || selectionVersion !== rawSelectionVersion.value) {
        URL.revokeObjectURL(loaded.src);
        rawObjectUrls.delete(loaded.src);
        return;
      }
      // A full-size render that fell back to the embedded preview must not
      // replace the sharper rendered preview or flip the user's selected mode.
      if (loaded.raw && loaded.raw.source !== 'rendered' && loaded.raw.source !== 'brightened') {
        URL.revokeObjectURL(loaded.src);
        rawObjectUrls.delete(loaded.src);
        rawUpgradeFailed = true;
        rawUpgradeRetryAt = Date.now() + 5000;
        return;
      }
      const slot = activeImage.value;
      const size = imageSize.value[slot];
      const previous = imageSrc.value[slot];
      const rotate = imageRotate.value[slot];
      // setImageSlot recomputes imageSizeRotated from props.rotate; preserve the
      // viewer's in-place rotation (and its rotated box) across the swap.
      const rotated = imageSizeRotated.value[slot];
      setImageSlot(slot, filePath, loaded.src, loaded.naturalWidth, loaded.naturalHeight, size.width, size.height);
      imageRotate.value[slot] = rotate;
      imageSizeRotated.value[slot] = rotated;
      if (loaded.raw) {
        rawSource.value = loaded.raw.source;
        rawEmbeddedUnavailable.value = loaded.raw.unavailable;
        rawFullSize.value = loaded.raw.source === 'rendered' || loaded.raw.source === 'brightened';
      }
      if (rawObjectUrls.delete(previous)) URL.revokeObjectURL(previous);
    } catch {
      if (loadingId === currentLoadingId.value && selectionVersion === rawSelectionVersion.value) {
        rawUpgradeFailed = true;
        rawUpgradeRetryAt = Date.now() + 5000;
        toast.warning(t('image_viewer.raw_render_unavailable'), { placement: 'bottom-right' });
      }
    } finally {
      if (loadingId === currentLoadingId.value && selectionVersion === rawSelectionVersion.value) rawRequestPending.value = false;
    }
  }, 150);
});

let resizeObserver: ResizeObserver | null = null;
const suppressViewportEmit = ref(false);
let warmImageTimeout: NodeJS.Timeout | null = null;
let warmImageIdleId: number | null = null;
const resolvedThumbnailSrc = ref('');
const resolvedThumbnailFileId = ref(0);
const displayThumbnailSrc = computed(() => props.thumbnailSrc || resolvedThumbnailSrc.value);

// Keep this list in Rust, alongside the preview implementation, so adding a
// new FFmpeg-backed format cannot reintroduce the placeholder/preview race.
let ffmpegBackedPreviewExtensions: Promise<Set<string>> | null = null;

function getFfmpegBackedPreviewExtensions() {
  if (!ffmpegBackedPreviewExtensions) {
    ffmpegBackedPreviewExtensions = getFfmpegBackedImageExtensions()
      .then((extensions: string[]) => new Set(extensions.map((extension) => extension.toLowerCase())));
  }
  return ffmpegBackedPreviewExtensions;
}

function waitForNextPaint() {
  return new Promise<void>((resolve) => {
    requestAnimationFrame(() => resolve());
  });
}

// inline loading for formats that require backend preview decoding
const showInlineLoading = computed(() =>
  props.showInlineLoading || (shouldUseBackendPreview(props.filePath, Number(props.fileType || 0)) && !!displayThumbnailSrc.value)
);

async function getEffectiveThumbnailSrc() {
  if (props.thumbnailSrc) return props.thumbnailSrc;
  const fileId = props.fileId;
  if (!fileId) return '';
  const thumbUrl = getThumbUrl(fileId, false, config.settings.thumbnailSize, props.fileVersion);
  if (!isWin) return thumbUrl;
  if (thumbUrl.startsWith('data:')) {
    resolvedThumbnailSrc.value = thumbUrl;
    resolvedThumbnailFileId.value = fileId;
    return thumbUrl;
  }
  if (resolvedThumbnailSrc.value && resolvedThumbnailFileId.value === fileId) return resolvedThumbnailSrc.value;

  const inflight = getThumbnailDataUrlInflight(fileId, config.settings.thumbnailSize);
  const dataUrl = await (inflight || setThumbnailDataUrlInflight(
    fileId,
    config.settings.thumbnailSize,
    getFileThumbById(fileId, config.settings.thumbnailSize, false)
      .then(thumb => getThumbnailDataUrl(thumb, '', false, config.settings.thumbnailSize, props.filePath, props.fileVersion))
  ));
  if (props.fileId === fileId && !props.thumbnailSrc && dataUrl) {
    resolvedThumbnailSrc.value = dataUrl;
    resolvedThumbnailFileId.value = fileId;
  }
  return props.fileId === fileId ? dataUrl || thumbUrl : '';
}

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

const adjustmentStyle = computed(() => (src: string) => {
  if (!uiStore.activeAdjustments.filePath) return '';
  if (uiStore.isInputActive('EditImage')) return '';
  
  // Check if current image matches the one being edited
  // We need to compare paths. The src might be a full URL (asset://...) while filePath is absolute path
  // But usually we can check if src contains the filePath or if we have the original filePath prop
  
  // Simpler: Check if the currently viewing file path matches the one in adjustment
  // defined in props.filePath or we can use the passed in src if we can decode it.
  
  // Actually, props.filePath is dependable for the *current* image, but we have two images in the DOM (previous and current).
  // The 'src' in the v-for loop is the asset URL.
  // Let's try to match loosely or use props.filePath if activeImage index matches
  
  // Better approach:
  // calculated adjustments should only apply if the source file matches the edited file.
  // However, mapping src (asset url) back to file path is tricky without util.
  // But we know 'src-vite' uses 'asset://' + filepath or similar.
  // Let's use a simpler heuristic: if props.filePath matches store's filePath, apply to the active image.
  
  if (uiStore.activeAdjustments.filePath === props.filePath) {
     const adj = uiStore.activeAdjustments;
     const parts = [];
     if (adj.brightness !== 0) parts.push(`brightness(${100 + adj.brightness}%)`);
     if (adj.contrast !== 0) parts.push(`contrast(${100 + adj.contrast}%)`);
     if (adj.saturation !== 100) parts.push(`saturate(${adj.saturation}%)`);
     if (adj.hue !== 0) parts.push(`hue-rotate(${adj.hue}deg)`);
     if (adj.blur > 0) parts.push(`blur(${adj.blur}px)`);
     if (adj.filter === 'grayscale') parts.push('grayscale(100%)');
     if (adj.filter === 'sepia') parts.push('sepia(100%)');
     if (adj.filter === 'invert') parts.push('invert(100%)');
     
     return parts.join(' ');
  }
  return '';
});

function loadImageResource(filePath?: string) {
  if (!filePath) {
    return Promise.reject(new Error('Missing file path'));
  }

  if (Number(props.fileType) === 3) return loadRawImage(filePath);

  const cached = preloadCache.get(filePath);
  if (cached) {
    return cached;
  }

  const loadPromise = new Promise<{ src: string; naturalWidth: number; naturalHeight: number }>((resolve, reject) => {
    let src = '';

    const img = new Image();
    img.decoding = 'async';

    img.onload = () => {
      img.decode()
        .then(() => {
          resolve({
            src,
            naturalWidth: img.naturalWidth,
            naturalHeight: img.naturalHeight,
          });
        })
        .catch(() => {
          resolve({
            src,
            naturalWidth: img.naturalWidth,
            naturalHeight: img.naturalHeight,
          });
        });
    };

    img.onerror = () => {
      preloadCache.delete(filePath);
      reject(new Error(`Error loading image: ${filePath}`));
    };

    if (shouldUseBackendPreview(filePath, Number(props.fileType || 0))) {
      src = getPreviewUrl(
        props.fileId,
        filePath,
        false,
        props.fileVersion,
      );
      if (!src) {
        preloadCache.delete(filePath);
        reject(new Error(`Failed to resolve RAW/TIFF preview source: ${filePath}`));
        return;
      }
      img.src = src;
      return;
    }

    try {
      src = getAssetSrc(filePath, props.fileVersion);
    } catch (error) {
      preloadCache.delete(filePath);
      reject(error);
      return;
    }

    if (!src) {
      preloadCache.delete(filePath);
      reject(new Error('Failed to resolve asset source'));
      return;
    }

    img.src = src;
  });

  preloadCache.set(filePath, loadPromise);
  return loadPromise;
}

let warmImageRequest = 0;
function warmImage(filePath?: string) {
  if (!filePath || filePath === props.filePath || shouldUseBackendPreview(filePath, Number(props.fileType || 0))) {
    return;
  }

  if (warmImageTimeout) {
    clearTimeout(warmImageTimeout);
    warmImageTimeout = null;
  }
  if (warmImageIdleId !== null && typeof window !== 'undefined' && 'cancelIdleCallback' in window) {
    window.cancelIdleCallback(warmImageIdleId);
    warmImageIdleId = null;
  }

  const request = ++warmImageRequest;
  warmImageTimeout = setTimeout(() => {
    warmImageTimeout = null;
    if (props.nextFilePath !== filePath || filePath === props.filePath) return;

    const runWarmup = async () => {
      warmImageIdleId = null;
      if (props.nextFilePath !== filePath || filePath === props.filePath) return;

      // Preloading is not an explicit request to download the next original.
      if (await isCloudOnlyFile(filePath)) return;
      if (request !== warmImageRequest || props.nextFilePath !== filePath || filePath === props.filePath) return;
      void loadImageResource(filePath).catch(() => {
        // Ignore preload failures and let the main load path surface errors.
      });
    };

    if (typeof window !== 'undefined' && 'requestIdleCallback' in window) {
      warmImageIdleId = window.requestIdleCallback(runWarmup, { timeout: 300 });
      return;
    }

    queueMicrotask(runWarmup);
  }, 150);
}

function cancelWarmImageScheduling() {
  warmImageRequest++;
  if (warmImageTimeout) {
    clearTimeout(warmImageTimeout);
    warmImageTimeout = null;
  }
  if (warmImageIdleId !== null && typeof window !== 'undefined' && 'cancelIdleCallback' in window) {
    window.cancelIdleCallback(warmImageIdleId);
    warmImageIdleId = null;
  }
}

function clearStalePreloadEntries(activeFilePath?: string, nextFilePath?: string) {
  const keep = new Set<string>();
  if (activeFilePath) keep.add(activeFilePath);
  if (nextFilePath) keep.add(nextFilePath);

  for (const key of preloadCache.keys()) {
    if (!keep.has(key)) {
      preloadCache.delete(key);
    }
  }

}

function loadPlaceholderResource(src?: string) {
  return new Promise<{ src: string; naturalWidth: number; naturalHeight: number }>((resolve, reject) => {
    if (!src) {
      reject(new Error('Missing placeholder src'));
      return;
    }

    const img = new Image();
    img.decoding = 'async';
    img.onload = () => {
      img.decode()
        .then(() => resolve({ src, naturalWidth: img.naturalWidth, naturalHeight: img.naturalHeight }))
        .catch(() => resolve({ src, naturalWidth: img.naturalWidth, naturalHeight: img.naturalHeight }));
    };
    img.onerror = () => reject(new Error('Failed to load placeholder image'));
    img.src = src;
  });
}

function getCompatibleLayout(
  naturalWidth: number,
  naturalHeight: number,
  preferredWidth: number,
  preferredHeight: number,
) {
  if (!preferredWidth || !preferredHeight || !naturalWidth || !naturalHeight) {
    return { width: naturalWidth, height: naturalHeight };
  }

  const naturalRatio = naturalWidth / naturalHeight;
  const preferredRatio = preferredWidth / preferredHeight;
  return Math.abs(naturalRatio - preferredRatio) / naturalRatio <= 0.01
    ? { width: preferredWidth, height: preferredHeight }
    : { width: naturalWidth, height: naturalHeight };
}

function setImageSlot(
  slotIndex: number,
  filePath: string,
  src: string,
  naturalWidth: number,
  naturalHeight: number,
  layoutWidth: number = naturalWidth,
  layoutHeight: number = naturalHeight,
) {
  imageNaturalSize.value[slotIndex] = { width: naturalWidth, height: naturalHeight };
  imageSrc.value[slotIndex] = src;
  imageFilePath.value[slotIndex] = filePath;
  imageRotate.value[slotIndex] = props.rotate;
  imageSize.value[slotIndex] = {
    width: layoutWidth,
    height: layoutHeight,
  };

  if (Math.abs(props.rotate % 180) === 90) {
    imageSizeRotated.value[slotIndex] = {
      width: layoutHeight,
      height: layoutWidth,
    };
  } else {
    imageSizeRotated.value[slotIndex] = {
      width: layoutWidth,
      height: layoutHeight,
    };
  }
}

const isDraggingNavBox = ref(false);
const navigatorViewport = computed(() => ({
  ...getViewportState(),
  viewportWidth: containerSize.value.width,
  viewportHeight: containerSize.value.height,
  rotate: imageRotate.value[activeImage.value],
  pannable: isGrabbing.value,
}));
function navigateImage(point: { normX: number; normY: number }) {
  noTransition.value = true;
  applyViewportState({ ...getViewportState(), ...point });
  requestAnimationFrame(() => { noTransition.value = false; });
}
function zoomNavigator(factor: number) {
  const size = containerSize.value;
  zoomImage(size.width / 2, size.height / 2,
    Math.min(maxScale.value, Math.max(minScale.value, scale.value[activeImage.value] * factor)));
}

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
    const el = container.value as HTMLElement;
    el.addEventListener('pointerdown', handlePinchPointerDown);
    el.addEventListener('pointermove', handlePinchPointerMove, { passive: false });
    el.addEventListener('pointerup', handlePinchPointerEnd);
    el.addEventListener('pointercancel', handlePinchPointerEnd);
    el.addEventListener('pointerleave', handlePinchPointerEnd);
  }
  // Global capture-phase fallback: in WebView2 some touchpad pinch events can
  // be intercepted before they bubble to the container element. Listening at
  // window with capture+passive:false guarantees we see them and can preventDefault.
  window.addEventListener('wheel', handleGlobalPinchWheel, { capture: true, passive: false });
});

onBeforeUnmount(() => {
  document.removeEventListener('mousemove', trackImageDragOut, true);
  document.documentElement.removeEventListener('mouseleave', trackImageDragOut);
  document.removeEventListener('mouseup', handleImageMouseUp, true);
  rawAbortController?.abort();
  for (const url of rawObjectUrls) URL.revokeObjectURL(url);
  if (rawUpgradeTimer) clearTimeout(rawUpgradeTimer);
  rawObjectUrls.clear();
  cancelAnimationFrame(resizeTransitionFrame);
  if (debounceTimeout) clearTimeout(debounceTimeout);
  if (rawLoadingTimer) clearTimeout(rawLoadingTimer);
  if (resizeObserver && container.value) {
    resizeObserver.unobserve(container.value);
    resizeObserver.disconnect();
  }
  if (container.value) {
    const el = container.value as HTMLElement;
    el.removeEventListener('pointerdown', handlePinchPointerDown);
    el.removeEventListener('pointermove', handlePinchPointerMove);
    el.removeEventListener('pointerup', handlePinchPointerEnd);
    el.removeEventListener('pointercancel', handlePinchPointerEnd);
    el.removeEventListener('pointerleave', handlePinchPointerEnd);
  }
  window.removeEventListener('wheel', handleGlobalPinchWheel, { capture: true });
  if (loadingTimeout) { // Clear timeout on unmount
    clearTimeout(loadingTimeout);
  }
  if (navigatorAutoHideTimer) {
    clearTimeout(navigatorAutoHideTimer);
  }
  cancelWarmImageScheduling();
});

function handleGlobalPinchWheel(event: WheelEvent) {
  if (!event.ctrlKey) return;
  if (!container.value) return;
  const rect = (container.value as HTMLElement).getBoundingClientRect();
  if (
    event.clientX < rect.left || event.clientX > rect.right ||
    event.clientY < rect.top || event.clientY > rect.bottom
  ) return;
  event.preventDefault();
  event.stopPropagation();
  isWheelZooming.value = true;
  if (wheelZoomTimeout) clearTimeout(wheelZoomTimeout);
  wheelZoomTimeout = setTimeout(() => { isWheelZooming.value = false; }, 200);
  applyZoomFromWheel(event);
}

// Touchpad pinch is dispatched by Chromium as small-delta ctrl+wheel events
// (deltaY = -log(scale_change) * 96), while Ctrl+mouse-wheel sends deltaY ~100
// per notch. Use the browser-matching exp formula for pinch and the gentler
// notch-based wheelZoom for mouse so each input feels native.
function applyZoomFromWheel(event: WheelEvent) {
  const isPinch = event.ctrlKey && event.deltaMode === 0 && Math.abs(event.deltaY) < 50;
  if (!isPinch) {
    wheelZoom(event, 1);
    return;
  }
  const currentScale = scale.value[activeImage.value];
  let newScale = currentScale * Math.exp(-event.deltaY / 96);
  newScale = Math.min(Math.max(newScale, minScale.value), maxScale.value);
  if (container.value) {
    const rect = (container.value as HTMLElement).getBoundingClientRect();
    const containerSizeVal = containerSize.value;
    const x = ((event.clientX - rect.left) * containerSizeVal.width) / rect.width;
    const y = ((event.clientY - rect.top) * containerSizeVal.height) / rect.height;
    zoomImage(x, y, newScale);
  }
}

function handlePinchPointerDown(event: PointerEvent) {
  if (event.pointerType !== 'touch') return;
  activeTouchPointers.set(event.pointerId, { x: event.clientX, y: event.clientY });
  isTouchActive.value = true;
  if (activeTouchPointers.size === 1) {
    // Single-finger pan: track this pointer. Mark isDraggingImage so the
    // 0.3s CSS transform transition is suppressed and the image follows the
    // finger in real time (the existing mouse drag relies on the same flag).
    panPointerId = event.pointerId;
    panLastPos = { x: event.clientX, y: event.clientY };
    touchSwipeStart = { x: event.clientX, y: event.clientY };
    touchSwipeTriggered = false;
    isDraggingImage.value = true;
  } else if (activeTouchPointers.size === 2) {
    // Promote to pinch: cancel any in-flight pan
    panPointerId = null;
    const pts = Array.from(activeTouchPointers.values());
    pinchStartDistance = Math.hypot(pts[0].x - pts[1].x, pts[0].y - pts[1].y);
    pinchStartScale = scale.value[activeImage.value];
    pinchCenter = {
      x: (pts[0].x + pts[1].x) / 2,
      y: (pts[0].y + pts[1].y) / 2,
    };
    isPinching = true;
    isDraggingImage.value = false;
    // Suppress CSS transitions so the image tracks fingers in real time.
    isWheelZooming.value = true;
    if (wheelZoomTimeout) {
      clearTimeout(wheelZoomTimeout);
      wheelZoomTimeout = null;
    }
  }
}

function handlePinchPointerMove(event: PointerEvent) {
  if (event.pointerType !== 'touch') return;
  if (!activeTouchPointers.has(event.pointerId)) return;
  activeTouchPointers.set(event.pointerId, { x: event.clientX, y: event.clientY });

  // Pinch path
  if (isPinching && activeTouchPointers.size === 2 && pinchStartDistance > 0) {
    event.preventDefault();
    const pts = Array.from(activeTouchPointers.values());
    const distance = Math.hypot(pts[0].x - pts[1].x, pts[0].y - pts[1].y);
    if (distance < 1) return;
    let newScale = pinchStartScale * (distance / pinchStartDistance);
    newScale = Math.min(Math.max(newScale, minScale.value), maxScale.value);
    if (container.value) {
      const rect = (container.value as HTMLElement).getBoundingClientRect();
      const containerSizeVal = containerSize.value;
      const x = ((pinchCenter.x - rect.left) * containerSizeVal.width) / rect.width;
      const y = ((pinchCenter.y - rect.top) * containerSizeVal.height) / rect.height;
      zoomImage(x, y, newScale);
    }
    return;
  }

  // Single-finger path: pan when the image is bigger than the container,
  // otherwise interpret horizontal motion as a swipe to navigate prev/next.
  if (panPointerId === event.pointerId && activeTouchPointers.size === 1) {
    event.preventDefault();
    const imgIndex = activeImage.value;

    if (!isGrabbing.value && !props.isSlideShow) {
      if (touchSwipeTriggered) return;
      const totalX = event.clientX - touchSwipeStart.x;
      const totalY = event.clientY - touchSwipeStart.y;
      const absX = Math.abs(totalX);
      const absY = Math.abs(totalY);
      if (absX >= MOUSE_DRAG_NAV_THRESHOLD && absX > absY) {
        const direction = totalX < 0 ? 'next' : 'prev';
        navDirection.value = direction;
        emit('message-from-image-viewer', { message: direction });
        touchSwipeTriggered = true;
        isDraggingImage.value = false;
      }
      return;
    }

    const scaleVal = scale.value[imgIndex];
    const imgRotatedSize = imageSizeRotated.value[imgIndex];
    const containerSizeVal = containerSize.value;
    const scaledWidth = imgRotatedSize.width * scaleVal;
    const scaledHeight = imgRotatedSize.height * scaleVal;
    const dx = scaledWidth <= containerSizeVal.width ? 0 : event.clientX - panLastPos.x;
    const dy = scaledHeight <= containerSizeVal.height ? 0 : event.clientY - panLastPos.y;
    panLastPos = { x: event.clientX, y: event.clientY };
    if (dx === 0 && dy === 0) return;
    position.value[imgIndex].x += dx;
    position.value[imgIndex].y += dy;
    clampPosition();
  }
}

function handlePinchPointerEnd(event: PointerEvent) {
  if (event.pointerType !== 'touch') return;
  activeTouchPointers.delete(event.pointerId);
  if (event.pointerId === panPointerId) panPointerId = null;
  if (activeTouchPointers.size < 2) {
    pinchStartDistance = 0;
  }
  if (activeTouchPointers.size === 0) {
    isPinching = false;
    isTouchActive.value = false;
    isDraggingImage.value = false;
    if (wheelZoomTimeout) clearTimeout(wheelZoomTimeout);
    wheelZoomTimeout = setTimeout(() => {
      isWheelZooming.value = false;
    }, 50);
  }
}

const updatePosition = () => {
  if (container.value) {
    const rect = (container.value as HTMLElement).getBoundingClientRect();
    containerPos.value = { x: rect.left, y: rect.top };
  }
};

// Preloaded next images must not retain a different RAW display policy.
watch(rawDisplayKey, () => preloadCache.clear(), { flush: 'sync' });

async function showOfflinePreview(filePath: string, loadingId: number) {
  if (loadingId !== currentLoadingId.value) return;
  if (loadingTimeout) { clearTimeout(loadingTimeout); loadingTimeout = null; }
  isLoading.value = false;
  rawRequestPending.value = false;
  if (imageFilePath.value[activeImage.value] === filePath && imageSrc.value[activeImage.value]) {
    offlinePreview.value = true;
    const size = imageNaturalSize.value[activeImage.value];
    maxScale.value = Math.max(scale.value[activeImage.value], Math.min(1, size.width / (imageSize.value[activeImage.value].width || 1)));
    return;
  }
  let loaded: LoadedImage | null = null;
  try {
    const cached = await preloadCache.get(filePath);
    if (cached) loaded = await loadPlaceholderResource(cached.src);
  } catch { /* Try thumbnail next. */ }
  if (!loaded && props.fileId) {
    const preview = getPreviewUrl(props.fileId, filePath, false, props.fileVersion);
    try { loaded = await loadPlaceholderResource(`${preview}${preview.includes('?') ? '&' : '?'}cachedOnly=true`); } catch { /* Try thumbnail next. */ }
  }
  if (!loaded) {
    try { loaded = await loadPlaceholderResource(await getEffectiveThumbnailSrc()); } catch { /* No local preview. */ }
  }
  if (loadingId !== currentLoadingId.value) return;
  offlinePreview.value = true;
  if (!loaded) {
    imageSrc.value = ['', ''];
    imageFilePath.value = ['', ''];
    loadError.value = true;
    return;
  }
  const slot = activeImage.value ^ 1;
  setImageSlot(slot, filePath, loaded.src, loaded.naturalWidth, loaded.naturalHeight);
  scale.value[slot] = 1;
  position.value[slot] = { x: 0, y: 0 };
  isZoomFit.value = true;
  maxScale.value = 1;
  onImageReady(slot);
}

// Watch file changes and the selected RAW preview source.
watch([
  () => props.filePath,
  () => props.fileVersion,
  () => Number(props.fileType || 0) === 3 ? `${rawDisplayKey()}:${rawSelectionVersion.value}:${JSON.stringify(rawOverride.value)}` : '',
  () => props.originalUnavailable,
], async ([newFilePath, newFileVersion, newRawThumbnailSource], [oldFilePath, oldFileVersion, oldRawThumbnailSource, oldUnavailable]) => {
  // Cancel previous loading
  if (rawUpgradeTimer) { clearTimeout(rawUpgradeTimer); rawUpgradeTimer = null; }
  rawAbortController?.abort();
  currentLoadingId.value++;
  const loadingId = currentLoadingId.value;
  cancelWarmImageScheduling();
  if (
    newFilePath
    && newFilePath === oldFilePath
    && (newFileVersion !== oldFileVersion || newRawThumbnailSource !== oldRawThumbnailSource || (oldUnavailable && !props.originalUnavailable))
  ) {
    preloadCache.delete(newFilePath);
  }
  clearStalePreloadEntries(newFilePath || '', props.nextFilePath || '');

  if (loadingTimeout) {
    clearTimeout(loadingTimeout);
    loadingTimeout = null;
  }

  loadError.value = false; // Reset error state
  offlinePreview.value = false;
  maxScale.value = 10;

  if (!newFilePath) {
    isLoading.value = false;
    return;
  }

  if (props.originalUnavailable) {
    await showOfflinePreview(String(newFilePath), loadingId);
    return;
  }
  if (Number(props.fileType) === 3) isLoading.value = true;

  // Set timeout to show loading overlay if loading takes too long
  loadingTimeout = setTimeout(() => {
    isLoading.value = true;
  }, 500);

  const usesBackendPreview = shouldUseBackendPreview(newFilePath, Number(props.fileType || 0));
  const ffmpegExtensionsPromise = getFfmpegBackedPreviewExtensions();
  const isRawPreview = Number(props.fileType || 0) === 3;

  try {
    const usesRealtimePreview = (await ffmpegExtensionsPromise).has(getFileExtension(newFilePath).toLowerCase());
    if (loadingId !== currentLoadingId.value) return;
    const imageResultPromise = loadImageResource(newFilePath)
      .then((loaded) => ({ kind: 'image' as const, loaded }));
    const keepCurrentRaw = isRawPreview && newFilePath === oldFilePath && !!rawSource.value;
    const thumbnailResultPromise = !keepCurrentRaw && (!isRawPreview || requestedRawOptions.value.mode === 'embedded' || requestedRawOptions.value.preferPair) && !usesRealtimePreview && (usesBackendPreview || props.showThumbnailPlaceholder)
      ? getEffectiveThumbnailSrc()
        .then(async (src) => {
          if (!src) return { kind: 'thumbnail' as const, placeholder: null };
          try {
            return { kind: 'thumbnail' as const, placeholder: await loadPlaceholderResource(src) };
          } catch {
            return { kind: 'thumbnail' as const, placeholder: null };
          }
        })
        .catch(() => ({ kind: 'thumbnail' as const, placeholder: null }))
      : Promise.resolve({ kind: 'thumbnail' as const, placeholder: null });
    const firstResult = await Promise.race([imageResultPromise, thumbnailResultPromise]);
    let hasPreviewPlaceholder = false;

    if (firstResult.kind === 'thumbnail' && firstResult.placeholder) {
      if (loadingId !== currentLoadingId.value) return;
      const nextImageIndex = activeImage.value ^ 1;
      const layout = getCompatibleLayout(
        firstResult.placeholder.naturalWidth,
        firstResult.placeholder.naturalHeight,
        props.imageWidth,
        props.imageHeight,
      );
      setImageSlot(
        nextImageIndex,
        newFilePath,
        firstResult.placeholder.src,
        firstResult.placeholder.naturalWidth,
        firstResult.placeholder.naturalHeight,
        layout.width,
        layout.height,
      );
      onImageReady(nextImageIndex, true);
      hasPreviewPlaceholder = true;
      await nextTick();
      await waitForNextPaint();
    }

    const loaded = firstResult.kind === 'image'
      ? firstResult.loaded
      : (await imageResultPromise).loaded;
    if (loadingId !== currentLoadingId.value) {
      if (loaded.raw) { URL.revokeObjectURL(loaded.src); rawObjectUrls.delete(loaded.src); }
      return;
    }

    if (loadingTimeout) {
      clearTimeout(loadingTimeout);
      loadingTimeout = null;
    }
    isLoading.value = false;

    nextTick(() => {
      if (loadingId !== currentLoadingId.value) {
        if (loaded.raw) { URL.revokeObjectURL(loaded.src); rawObjectUrls.delete(loaded.src); }
        return;
      }
      if (loaded.raw) {
        rawRequestPending.value = false;
        rawSource.value = loaded.raw.source;
        // RAW-only requests do not query companions; retain the known pair.
        if (loaded.raw.pair !== null) resolvedRawPairLabel.value = loaded.raw.pair;
        rawEmbeddedUnavailable.value ||= loaded.raw.unavailable;
      }
      isZoomFit.value = props.isZoomFit;
      const activeIndex = activeImage.value;
      const showingPlaceholderForCurrentFile = hasPreviewPlaceholder
        && imageFilePath.value[activeIndex] === newFilePath;

      if (showingPlaceholderForCurrentFile) {
        noTransition.value = true;
        // Reuse the placeholder layout so replacing source pixels does not
        // change the displayed geometry. In particular, RAW thumbnails use
        // the complete image dimensions, preserving their scale on replacement.
        const placeholderLayout = getCompatibleLayout(
          loaded.naturalWidth,
          loaded.naturalHeight,
          imageSize.value[activeIndex].width,
          imageSize.value[activeIndex].height,
        );
        setImageSlot(
          activeIndex,
          newFilePath,
          loaded.src,
          loaded.naturalWidth,
          loaded.naturalHeight,
          placeholderLayout.width,
          placeholderLayout.height,
        );

        if (isRawPreview && containerSize.value.width > 0) {
          // Keep the thumbnail's current transform. Both the thumbnail and
          // complete RAW use the same layout dimensions, so scale and position
          // describe the identical viewport regardless of auto-fit state.
          clampPosition(true);
        } else if (containerSize.value.width > 0 && isZoomFit.value) {
          updateZoomFit(true);
        }

        setTimeout(() => {
          noTransition.value = false;
        }, 150);
      } else {
        const nextImageIndex = activeIndex ^ 1;
        scale.value[nextImageIndex] = 1;
        position.value[nextImageIndex] = { x: 0, y: 0 };
        const layout = isRawPreview
          ? getCompatibleLayout(loaded.naturalWidth, loaded.naturalHeight, props.imageWidth, props.imageHeight)
          : { width: loaded.naturalWidth, height: loaded.naturalHeight };
        setImageSlot(
          nextImageIndex,
          newFilePath,
          loaded.src,
          loaded.naturalWidth,
          loaded.naturalHeight,
          layout.width,
          layout.height,
        );
        onImageReady(nextImageIndex);
      }
      for (const url of rawObjectUrls) {
        if (!imageSrc.value.includes(url)) { URL.revokeObjectURL(url); rawObjectUrls.delete(url); }
      }
      warmImage(props.nextFilePath);
    });
  } catch (e) {
    console.error("Error getting asset source:", e);
    if (loadingId !== currentLoadingId.value) return;
    if (loadingTimeout) {
      clearTimeout(loadingTimeout);
      loadingTimeout = null;
    }
    isLoading.value = false;
    rawRequestPending.value = false;
    try {
      if (!await checkFileAccessibility(String(newFilePath))) {
        if (loadingId !== currentLoadingId.value) return;
        setFileAccessibility(String(newFilePath), false);
        await showOfflinePreview(String(newFilePath), loadingId);
        return;
      }
    } catch { /* Keep the decoding error if access could not be checked. */ }
    if (loadingId !== currentLoadingId.value) return;
    if (isRawPreview && newFilePath === oldFilePath && rawSource.value) {
      // A RAW mode switch that the decoder can't fulfil (e.g. some NEF variants
      // only have an embedded preview). Keep the current image; explain why.
      toast.warning(t('image_viewer.raw_render_unavailable'), { placement: 'bottom-right' });
    } else {
      loadError.value = true;
    }
  }
}, { immediate: true });

watch(() => props.fileId, () => {
  resolvedThumbnailSrc.value = '';
  resolvedThumbnailFileId.value = 0;
});

// watch thumbnail source changes to update placeholder if original is still loading
watch(displayThumbnailSrc, async (newThumbSrc) => {
  if (props.originalUnavailable || offlinePreview.value) return;
  if (Number(props.fileType) === 3) return;
  if (!newThumbSrc) return;
  const currentFilePath = props.filePath;
  if (!currentFilePath) return;
  const loadingId = currentLoadingId.value;

  const usesBackendPreview = shouldUseBackendPreview(currentFilePath, Number(props.fileType || 0));
  const ffmpegExtensions = await getFfmpegBackedPreviewExtensions();
  if (!usesBackendPreview || ffmpegExtensions.has(getFileExtension(currentFilePath).toLowerCase())) return;

  // Only update if we are still waiting for the full image OR if we are currently showing a stale placeholder
  const activeIndex = activeImage.value;
  
  // We check if it's the full original image by checking the src. 
  // For backend preview, the full image src is from getPreviewUrl.
  const fullImageSrc = getPreviewUrl(
    props.fileId,
    currentFilePath,
    false,
    props.fileVersion,
  );
  const isCurrentlyShowingFullImage = imageSrc.value[activeIndex] === fullImageSrc;
  
  if (isCurrentlyShowingFullImage) return;

  try {
    const placeholder = await loadPlaceholderResource(newThumbSrc);
    // Never downgrade a full image that finished loading while the placeholder was decoding.
    if (loadingId !== currentLoadingId.value || offlinePreview.value || imageSrc.value.includes(fullImageSrc)) return;
    const layout = getCompatibleLayout(
      placeholder.naturalWidth,
      placeholder.naturalHeight,
      props.imageWidth,
      props.imageHeight,
    );
    // Check if we haven't switched files since we started loading the placeholder
    if (props.filePath === currentFilePath) {
      // If we are currently showing a placeholder for this file, just update it in place
      if (imageFilePath.value[activeIndex] === currentFilePath) {
        noTransition.value = true;
        setImageSlot(
          activeIndex,
          currentFilePath,
          placeholder.src,
          placeholder.naturalWidth,
          placeholder.naturalHeight,
          layout.width,
          layout.height,
        );
        // Important: update layout after size change
        if (isZoomFit.value) {
          updateZoomFit(true);
        } else {
          clampPosition(true);
        }
        setTimeout(() => { noTransition.value = false; }, 150);
      } else {
        // We might be in a slot transition, but showing the other slot's placeholder?
        // Let's just update the target slot if it's assigned to this file
        const targetIndex = imageFilePath.value[0] === currentFilePath ? 0 : (imageFilePath.value[1] === currentFilePath ? 1 : -1);
        if (targetIndex !== -1) {
          setImageSlot(
            targetIndex,
            currentFilePath,
            placeholder.src,
            placeholder.naturalWidth,
            placeholder.naturalHeight,
            layout.width,
            layout.height,
          );
        }
      }
    }
  } catch {
    // ignore
  }
});

// watch fileId / face toggle changes to fetch faces
watch(() => [props.fileId, config.settings.face.enabled], async ([newFileId, faceEnabled]) => {
  faces.value = []; // Clear previous faces
  if (faceEnabled && newFileId) {
    const result = await getFacesForFile(newFileId);
    if (result && result.length > 0) {
      // Parse bbox JSON string for each face
      faces.value = result.map((face: RawFace) => {
        try {
          return {
            ...face,
            bbox: JSON.parse(face.bbox)
          };
        } catch (e) {
          console.error("Error parsing face bbox", e);
          return null;
        }
      }).filter((f: Face | null) => f);
    }
  }
}, { immediate: true });

// watch rotate changes
watch(() => props.rotate, (newRotate) => {
  const activeIndex = activeImage.value;
  const inactiveIndex = activeIndex ^ 1;
  const currentFilePath = props.filePath || '';

  // Update only the buffer(s) that actually render the current file.
  // This avoids mutating the leaving slide during navigation.
  if (imageFilePath.value[activeIndex] === currentFilePath) {
    imageRotate.value[activeIndex] = newRotate;
  }
  if (imageFilePath.value[inactiveIndex] === currentFilePath) {
    imageRotate.value[inactiveIndex] = newRotate;
  }
});

watch(() => imageRotate.value[activeImage.value], (newValue) => {
  const imgIndex = activeImage.value;
  const imgSize = imageSize.value[imgIndex];
  
  // swap image width and height
  if (Math.abs(newValue % 180) === 90) {
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
  emit('scale', { 
    scale: newValue, 
    displayScale: getDisplayScale(newValue),
    minScale: minScale.value, 
    maxScale: maxScale.value 
  });
});

function resetNavigatorAutoHide() {
  if (navigatorAutoHideTimer) {
    clearTimeout(navigatorAutoHideTimer);
    navigatorAutoHideTimer = null;
  }
  navigatorAutoVisible.value = true;
  if (config.settings.navigatorViewMode === 0) {
    navigatorAutoHideTimer = setTimeout(() => {
      navigatorAutoVisible.value = false;
      navigatorAutoHideTimer = null;
    }, 3000);
  }
}

function pauseNavigatorAutoHide() {
  if (navigatorAutoHideTimer) {
    clearTimeout(navigatorAutoHideTimer);
    navigatorAutoHideTimer = null;
  }
}

watch(
  () => [
    config.settings.navigatorViewMode,
    scale.value[activeImage.value],
    position.value[activeImage.value].x,
    position.value[activeImage.value].y,
  ],
  resetNavigatorAutoHide,
  { immediate: true }
);

// watch zoom fit changes
watch(() => props.isZoomFit, (newValue) => {
  isZoomFit.value = newValue;
  updateZoomFit();
});

// A resize changes the coordinate system, not the user's zoom intent. Apply
// its fit/position before paint instead of displaying the old layout for 100ms
// and then animating from that stale position (especially after Teleport).
watch(containerSize, (size) => {
  if (size.width <= 0 || size.height <= 0) return;
  const image = imageSizeRotated.value[activeImage.value];
  if (image.width <= 0 || image.height <= 0) return;
  isResizingContainer.value = true;
  cancelAnimationFrame(resizeTransitionFrame);
  if (isZoomFit.value) zoomFit();
  else clampPosition();
  // Keep transitions disabled through a painted frame, including consecutive
  // resize notifications from the native fullscreen animation.
  resizeTransitionFrame = requestAnimationFrame(() => {
    resizeTransitionFrame = requestAnimationFrame(() => {
      isResizingContainer.value = false;
    });
  });
}, { flush: 'sync' });

// Image loading retains its existing deferred layout update.
let debounceTimeout: NodeJS.Timeout | null = null;
watch(() => imageSize.value, () => {
  if (debounceTimeout) clearTimeout(debounceTimeout);
  debounceTimeout = setTimeout(() => {
    if (isZoomFit.value) {
      zoomFit();
    } else {
      clampPosition();
    }
  }, 100); // Debounce for 100ms
});

// Called when the new image is fully loaded and ready to be shown
const onImageReady = (nextIndex: number, preserveLoading: boolean = false) => {
  // A placeholder is ready before the full image. It needs the same viewport
  // initialization, but must not dismiss the full-image loading indicator.
  if (!preserveLoading) {
    if (loadingTimeout) {
      clearTimeout(loadingTimeout);
      loadingTimeout = null;
    }
    isLoading.value = false;
  }
  noTransition.value = true;
  // A viewport belongs to the image content, not to its pixel dimensions.
  // Capture it before switching buffers so the next image can convert the
  // normalized viewport to its own rendered position.
  const previousViewport = getViewportState();
  const previousImageIndex = activeImage.value;
  const hasPreviousImage = imageSize.value[previousImageIndex].width > 0
    && imageSize.value[previousImageIndex].height > 0;
  // Transitioning active image
  activeImage.value = nextIndex;

  const applyZoom = () => {
    // If not in "zoom to fit" mode, perform a calculated zoom to the cursor position.
    if (!isZoomFit.value) {
      const imgIndex = activeImage.value;
      const imgSize = imageSize.value[imgIndex];
      const container = containerSize.value;
      if (hasPreviousImage) {
        // Never carry over absolute x/y offsets: they describe the previous
        // image's dimensions. Rebuild the position from normalized viewport
        // coordinates so image navigation and thumbnail replacement retain
        // the same relative point in the image.
        applyViewportState(previousViewport, true);
        // The silent restore avoids sync feedback, but consumers still need
        // the new file's dimensions and identity after the buffer switch.
        emit('viewport-change', getViewportState());
      } else {
        // Original logic: reset to center or zoom to cursor
        
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
        const newScale = getActualSizeScale();
        const imageOffsetX = ((fitScale - newScale) * ((cursorX - initialPos.x) - imgSize.width / 2)) / fitScale;
        const imageOffsetY = ((fitScale - newScale) * ((cursorY - initialPos.y) - imgSize.height / 2)) / fitScale;
        
        scale.value[imgIndex] = newScale;
        position.value[imgIndex] = {
          x: initialPos.x + imageOffsetX,
          y: initialPos.y + imageOffsetY,
        };
        triggerRef(position);
        clampPosition(true);
      }

      // Also update the other image's position to match for smooth transitions
      // const otherImageIndex = activeImage.value ^ 1;
      // imageSrc.value[otherImageIndex] = '';
      // position.value[otherImageIndex] = position.value[activeImage.value];
    } else {
      // For isZoomFit, the original logic is fine.
      updateZoomFit(true);
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

const rotateView = (delta = 90) => {
  imageRotate.value[activeImage.value] += delta;
};

const toggleZoomFit = () => {
  if (props.isSlideShow) return;
  emit('update:isZoomFit', !props.isZoomFit);
};

const updateZoomFit = (force: boolean = false) => {
  console.log('updateZoomFit');
  isZoomFit.value ? zoomFit(force) : zoomReset(force);

  // set the hide image to the same position
  // const nextImageIndex = activeImage.value ^ 1;
  // imageSrc.value[nextImageIndex] = '';
  // position.value[nextImageIndex] = position.value[activeImage.value];
};

// Zoom to fit image in container
const zoomFit = (force: boolean = false) => {
  console.log('zoomFit');
  const container = containerSize.value;
  const imgRotatedSize = imageSizeRotated.value[activeImage.value];
  
  const containerAspectRatio = container.width / container.height;
  const imageAspectRatio = imgRotatedSize.width / imgRotatedSize.height;

  const scale = containerAspectRatio > imageAspectRatio 
    ? container.height / imgRotatedSize.height
    : container.width / imgRotatedSize.width;

  // set position to center
  zoomImage(container.width / 2, container.height / 2, scale, force);
};

// Reset zoom level and position
const zoomReset = (force: boolean = false) => {
  console.log('zoomReset');
  updatePosition();
  const mousePos = mousePosition.value;
  const containerPosVal = containerPos.value;
  zoomImage(mousePos.x - containerPosVal.x, mousePos.y - containerPosVal.y, getActualSizeScale(), force);
};

let dragOutStart: { x: number; y: number; path: string; preview: number[] } | null = null;
function trackImageDragOut(event: MouseEvent) {
  const start = dragOutStart;
  if (!start || !(event.buttons & 1)) return;
  if (Math.hypot(event.clientX - start.x, event.clientY - start.y) < 6 || !isWindowDragEdge(event)) return;
  mouseDragNavDeltaX.value = 0;
  mouseDragNavDeltaY.value = 0;
  finishImageMouseDrag(false);
  void startNativeFileDrag([start.path], start.preview).catch(error => {
    console.error('Native image drag failed:', error);
    toast.error(t('tooltip.drag_out.failed'));
  });
}

// start dragging
const handleImageMouseDown = (event: MouseEvent) => {
  if (isTouchActive.value) return; // touch path owns this gesture
  event.preventDefault();
  updatePosition();

  if (event.button === 0) {     // left click: drag image
    if (!props.originalUnavailable && !props.isSlideShow && props.filePath) {
      dragOutStart = {
        x: event.clientX, y: event.clientY, path: props.filePath,
        preview: createDragPreview(event.target as HTMLImageElement),
      };
      document.addEventListener('mousemove', trackImageDragOut, true);
      document.documentElement.addEventListener('mouseleave', trackImageDragOut);
      document.addEventListener('mouseup', handleImageMouseUp, true);
    }
    isDraggingImage.value = true;
    lastMousePosition.value = { x: event.clientX, y: event.clientY };
    mouseDragNavDeltaX.value = 0;
    mouseDragNavDeltaY.value = 0;
    mouseDragNavTriggered.value = false;
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
  if (isTouchActive.value) return; // touch path owns this gesture
  // update mouse position
  mousePosition.value = { x: event.clientX, y: event.clientY };
  updatePosition();

  if (!isDraggingImage.value) return;

  latestMouseEvent.value = event;

  if (animationFrameId) {
    cancelAnimationFrame(animationFrameId);
  }

  animationFrameId = requestAnimationFrame(updateDragPosition);
};

// stop dragging
const handleImageMouseUp = () => finishImageMouseDrag(true);

function finishImageMouseDrag(navigate: boolean) {
  if (animationFrameId) cancelAnimationFrame(animationFrameId);
  animationFrameId = null;
  // A quick swipe may release before its scheduled frame runs.
  if (navigate && isDraggingImage.value && latestMouseEvent.value) updateDragPosition();
  latestMouseEvent.value = null;
  if (navigate && mouseDragNavTriggered.value && Math.abs(mouseDragNavDeltaX.value) >= MOUSE_DRAG_NAV_THRESHOLD) {
    const direction = mouseDragNavDeltaX.value > 0 ? 'prev' : 'next';
    navDirection.value = direction;
    emit('message-from-image-viewer', { message: direction });
  }
  dragOutStart = null;
  document.removeEventListener('mousemove', trackImageDragOut, true);
  document.documentElement.removeEventListener('mouseleave', trackImageDragOut);
  document.removeEventListener('mouseup', handleImageMouseUp, true);
  isDraggingImage.value = false;
  mouseDragNavDeltaX.value = 0;
  mouseDragNavDeltaY.value = 0;
  mouseDragNavTriggered.value = false;
}

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
  if (!event || !isDraggingImage.value) return;

  const imgIndex = activeImage.value;
  const scaleVal = scale.value[imgIndex];
  const imageRotatedSize = imageSizeRotated.value[imgIndex];
  const container = containerSize.value;
  const lastPos = lastMousePosition.value;

  const scaledWidth = imageRotatedSize.width * scaleVal;
  const scaledHeight = imageRotatedSize.height * scaleVal;
  const rawDeltaX = event.clientX - lastPos.x;
  const rawDeltaY = event.clientY - lastPos.y;
  const canPan = scaledWidth > container.width || scaledHeight > container.height;

  // In zoom-fit mode, horizontal drag acts like touchpad swipe navigation
  // only when the image cannot be panned in current viewport.
  if (isZoomFit.value && !props.isSlideShow && !canPan) {
    mouseDragNavDeltaX.value += rawDeltaX;
    mouseDragNavDeltaY.value += rawDeltaY;

    if (!mouseDragNavTriggered.value) {
      const absX = Math.abs(mouseDragNavDeltaX.value);
      const absY = Math.abs(mouseDragNavDeltaY.value);
      if (absX >= MOUSE_DRAG_NAV_THRESHOLD && absX > absY) {
        // Commit mouse swipe on release, so dragging onward to the window
        // edge can hand the original file to the OS instead of changing images.
        mouseDragNavTriggered.value = true;
      }
    }

    lastMousePosition.value = { x: event.clientX, y: event.clientY };
    animationFrameId = null;
    return;
  }

  const deltaX = scaledWidth <= container.width ? 0 : event.clientX - lastPos.x;
  const deltaY = scaledHeight <= container.height ? 0 : event.clientY - lastPos.y;

  position.value[imgIndex].x += deltaX;
  position.value[imgIndex].y += deltaY;

  lastMousePosition.value = { x: event.clientX, y: event.clientY };

  clampPosition();

  animationFrameId = null; // reset animation frame ID
};

// Simple reset - clear all swipe state
function resetSwipeState() {
  gestureType.value = 'none';
  horizontalDeltaAccumulator = 0;
  verticalDeltaAccumulator = 0;
  hasNavigatedThisGesture = false;
  lastDeltaX = 0;
  navDirection.value = '';
  noTransition.value = false;
  if (gestureResetTimeout) {
    clearTimeout(gestureResetTimeout);
    gestureResetTimeout = null;
  }
}

function handleTransitionEnd() {
  navDirection.value = '';
}

// mouse wheel zoom
function handleImageWheel(event: WheelEvent) {
  event.preventDefault();
  event.stopPropagation();
  updatePosition();

  // Touchpad pinch (and Ctrl+wheel) arrives as a wheel event with ctrlKey=true.
  // Treat it as a direct zoom, bypassing the swipe/nav gesture-detection path.
  if (event.ctrlKey) {
    isWheelZooming.value = true;
    if (wheelZoomTimeout) clearTimeout(wheelZoomTimeout);
    wheelZoomTimeout = setTimeout(() => {
      isWheelZooming.value = false;
    }, 200);
    applyZoomFromWheel(event);
    return;
  }

  // Simple touchpad detection: if there's horizontal delta, it's a touchpad
  // Mouse wheels only scroll vertically (deltaY only)
  // Once detected, stays true for the session (sticky)
  if (event.deltaX !== 0) {
    isTouchpadDevice = true;
  }
  
  const isTouchPad = isTouchpadDevice;

  // Reset timeout - when no events for 150ms, reset gesture state
  if (gestureResetTimeout) clearTimeout(gestureResetTimeout);
  gestureResetTimeout = setTimeout(() => {
    resetSwipeState();
  }, 150);

  if (isTouchPad) {
    // If already navigated this gesture, check if speed increased
    if (hasNavigatedThisGesture) {
      const speedIncreased = Math.abs(event.deltaX) > Math.abs(lastDeltaX) + 5;
      if (!speedIncreased) {
        lastDeltaX = event.deltaX;
        return; // Block - not a new intentional flick
      }
      // Speed increased - allow new navigation
      hasNavigatedThisGesture = false;
      horizontalDeltaAccumulator = 0;
    }
    lastDeltaX = event.deltaX;

    // Determine gesture direction
    if (gestureType.value === 'none') {
      horizontalDeltaAccumulator += event.deltaX;
      verticalDeltaAccumulator += event.deltaY;

      const absX = Math.abs(horizontalDeltaAccumulator);
      const absY = Math.abs(verticalDeltaAccumulator);

      if (absX > GESTURE_LOCK_THRESHOLD || absY > GESTURE_LOCK_THRESHOLD) {
        gestureType.value = absX > absY ? 'nav' : 'zoom';
      }
      return;
    }

    if (gestureType.value === 'nav') {
      horizontalDeltaAccumulator += event.deltaX;

      // Trigger navigation when threshold reached
      if (Math.abs(horizontalDeltaAccumulator) >= HORIZONTAL_NAV_THRESHOLD) {
        const direction = horizontalDeltaAccumulator > 0 ? 'next' : 'prev';
        navDirection.value = direction;
        hasNavigatedThisGesture = true;
        horizontalDeltaAccumulator = 0;
        gestureType.value = 'none'; // Allow new image to be centered
        emit('message-from-image-viewer', { message: direction });
      }
      return;
    }
  }

  // If we're here and it's a touchpad, gestureType must be 'zoom' or it's a regular mouse
  isWheelZooming.value = true;
  if (wheelZoomTimeout) clearTimeout(wheelZoomTimeout);
  wheelZoomTimeout = setTimeout(() => {
    isWheelZooming.value = false;
  }, 200);

  const zoomFactor = isTouchPad ? 1 : 0.1; // Adjust sensitivity

  // Touchpad always zooms; mouseWheelMode only affects regular mouse
  if (isTouchPad) {
    wheelZoom(event, zoomFactor);
  } else if (config.settings.mouseWheelMode === 0) {  // 0: previous/next image
    if (event.ctrlKey) {     // ctrl + mouse wheel: zoom in / out
      wheelZoom(event, zoomFactor);
    } else {
      emit('message-from-image-viewer', { message: event.deltaY < 0 ? 'prev' : 'next' });
    }
  } else if (config.settings.mouseWheelMode === 1) {  // 1: zoom in / out
    wheelZoom(event, zoomFactor);
  }
}

// wheel zoom - Industry standard fixed-step approach
function wheelZoom(event: WheelEvent, zoomFactor: number) {
  const currentScale = scale.value[activeImage.value];
  
  // Normalize delta to wheel "notches" (standard mouse ~100 per notch)
  let delta = event.deltaY;
  if (event.deltaMode === 1) { // DOM_DELTA_LINE
    delta *= 40;
  } else if (event.deltaMode === 2) { // DOM_DELTA_PAGE
    delta *= 800;
  }
  
  // Convert to notches (standard mice report ~100 per notch)
  // Use sign for direction, clamp magnitude for consistency
  const notches = Math.sign(delta) * (Math.abs(delta) / 100);
  
  // Fixed 20% zoom per notch
  const ZOOM_FACTOR = 0.2;
  const multiplier = Math.pow(1 + ZOOM_FACTOR, -notches);
  
  let newScale = currentScale * multiplier;
  newScale = Math.min(Math.max(newScale, minScale.value), maxScale.value);

  // Zoom at cursor position in container layout coordinates.
  if (container.value) {
    const rect = (container.value as HTMLElement).getBoundingClientRect();
    const containerSizeVal = containerSize.value;
    const x = ((event.clientX - rect.left) * containerSizeVal.width) / rect.width;
    const y = ((event.clientY - rect.top) * containerSizeVal.height) / rect.height;
    zoomImage(x, y, newScale);
    return;
  }

  // Fallback for rare lifecycle gaps.
  const containerPosVal = containerPos.value;
  zoomImage(event.clientX - containerPosVal.x, event.clientY - containerPosVal.y, newScale);
}

const zoomIn = () => {
  const newScale = Math.min(scale.value[activeImage.value] * 2, maxScale.value);
  const container = containerSize.value;
  zoomImage(container.width / 2, container.height / 2, newScale);
};

const zoomOut = () => {
  const container = containerSize.value;
  const imgRotatedSize = imageSizeRotated.value[activeImage.value];
  
  // Calculate potential min scale based on fit, but respect hard floor of 0.1 if desired?
  // User asked for "lowest scale 10%". 
  // We'll prioritize 0.1, but if fitting requires less, we might have a conflict.
  // Assuming 0.1 is the hard floor relevant to the user's request.
  
  const fitScale = Math.min(
    container.width / imgRotatedSize.width, 
    container.height / imgRotatedSize.height
  );
  
  // Update minScale to be at least 0.1. 
  // If we want to allow "Overview mode" smaller than 0.1 we can adjust.
  // But given the "bug report" of 0%, we stick to 0.1.
  minScale.value = 0.1;

  const newScale = Math.max(scale.value[activeImage.value] / 2, minScale.value);
  const containerPosVal = containerPos.value;
  zoomImage(container.width / 2, container.height / 2, newScale);
};

const zoomActual = () => {
  const container = containerSize.value;
  zoomImage(container.width / 2, container.height / 2, getActualSizeScale());
};

// Zoom image at cursor position
function zoomImage(cursorX: number, cursorY: number, newScale: number, force: boolean = false) {
  const imgIndex = activeImage.value;
  const currentScale = scale.value[imgIndex];
  const pos = position.value[imgIndex];
  const imgSize = imageSize.value[imgIndex];
  
  const imageOffsetX = ((currentScale - newScale) * ((cursorX - pos.x) - imgSize.width / 2)) / currentScale;
  const imageOffsetY = ((currentScale - newScale) * ((cursorY - pos.y) - imgSize.height / 2)) / currentScale;
  
  pos.x += imageOffsetX;
  pos.y += imageOffsetY;

  scale.value[imgIndex] = newScale;
  clampPosition(force);
}

function getViewportState() {
  const imgIndex = activeImage.value;
  const imgSize = imageSize.value[imgIndex];
  const scaleVal = scale.value[imgIndex];
  const container = containerSize.value;
  const pos = position.value[imgIndex];

  // Store the image-content point under the viewport center, not the rendered
  // x/y offset. That makes the viewport independent from source dimensions.
  const dx = container.width / 2 - (pos.x + imgSize.width / 2);
  const dy = container.height / 2 - (pos.y + imgSize.height / 2);
  const angle = imageRotate.value[imgIndex] * Math.PI / 180;
  const cos = Math.cos(angle);
  const sin = Math.sin(angle);
  const contentX = imgSize.width / 2 + (dx * cos + dy * sin) / scaleVal;
  const contentY = imgSize.height / 2 + (-dx * sin + dy * cos) / scaleVal;

  return {
    scale: scaleVal,
    normX: Math.min(Math.max(contentX / imgSize.width, 0), 1),
    normY: Math.min(Math.max(contentY / imgSize.height, 0), 1),
    sourceWidth: imgSize.width,
    sourceHeight: imgSize.height,
    fileId: props.fileId,
    viewportWidth: container.width,
    viewportHeight: container.height,
    rotate: imageRotate.value[imgIndex],
    pannable: isGrabbing.value,
  };
}

function applyViewportState(viewport: { scale?: number; normX?: number; normY?: number }, silent = false) {
  if (!viewport || typeof viewport.scale !== 'number') return;

  const imgIndex = activeImage.value;
  const imgSize = imageSize.value[imgIndex];
  const container = containerSize.value;
  const safeScale = Math.min(Math.max(viewport.scale, minScale.value), maxScale.value);

  scale.value[imgIndex] = safeScale;

  const normX = Math.min(Math.max(viewport.normX ?? 0.5, 0), 1);
  const normY = Math.min(Math.max(viewport.normY ?? 0.5, 0), 1);
  const localX = (normX - 0.5) * imgSize.width * safeScale;
  const localY = (normY - 0.5) * imgSize.height * safeScale;
  const angle = imageRotate.value[imgIndex] * Math.PI / 180;
  const cos = Math.cos(angle);
  const sin = Math.sin(angle);
  const rotatedX = localX * cos - localY * sin;
  const rotatedY = localX * sin + localY * cos;

  position.value[imgIndex].x = container.width / 2 - imgSize.width / 2 - rotatedX;
  position.value[imgIndex].y = container.height / 2 - imgSize.height / 2 - rotatedY;

  if (silent) {
    // Sync path: disable transition for this frame to avoid trailing.
    noTransition.value = true;
  }

  suppressViewportEmit.value = silent;
  clampPosition(true);
  suppressViewportEmit.value = false;

  if (silent) {
    requestAnimationFrame(() => {
      noTransition.value = false;
    });
  }
}

// Ensure image stays within container
function clampPosition(force: boolean = false) {
  // Skip clamping during horizontal swipe to avoid jitter
  if (!force && gestureType.value === 'nav') return;
  
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

  if (!suppressViewportEmit.value) {
    emit('viewport-change', getViewportState());
  }
};

// Expose methods
defineExpose({ 
  zoomIn, 
  zoomOut,
  zoomActual,
  rotateView,
  getViewportState,
  applyViewportState,
  zoomNavigator,
  navigateImage,
  toggleZoomFit,
  getCurrentImageSrc: () => imageSrc.value[activeImage.value] || '',
  clearPreloadCache: (filePath?: string) => {
    if (filePath) {
      preloadCache.delete(filePath);
    } else {
      preloadCache.clear();
    }
  },
});

</script>

<style scoped>
/* Slideshow / Swipe transition */
.slide-next-enter-active,
.slide-next-leave-active,
.slide-prev-enter-active,
.slide-prev-leave-active {
  transition: transform 0.6s cubic-bezier(0.4, 0, 0.2, 1);
  will-change: transform;
  backface-visibility: hidden;
  transform: translateZ(0);
  contain: paint;
}

/* next: current leaves left, new enters from right */
.slide-next-enter-from {
  transform: translate3d(100%, 0, 0);
}
.slide-next-leave-to {
  transform: translate3d(-100%, 0, 0);
}

/* prev: current leaves right, new enters from left */
.slide-prev-enter-from {
  transform: translate3d(-100%, 0, 0);
}
.slide-prev-leave-to {
  transform: translate3d(100%, 0, 0);
}

.slide-next-enter-active,
.slide-prev-enter-active {
  z-index: 2;
}

.slide-next-leave-active,
.slide-prev-leave-active {
  z-index: 1;
}

.slideshow-fade-enter-active,
.slideshow-fade-leave-active {
  transition:
    opacity 0.5s cubic-bezier(0.22, 1, 0.36, 1),
    filter 0.5s ease;
  will-change: opacity, filter;
}

.slideshow-fade-enter-active {
  z-index: 2;
}

.slideshow-fade-leave-active {
  z-index: 1;
}

.slideshow-fade-enter-from {
  opacity: 0;
  filter: brightness(0.88);
}

.slideshow-fade-enter-to {
  opacity: 1;
  filter: brightness(1);
}

.slideshow-fade-leave-from {
  opacity: 1;
  filter: brightness(1);
}

.slideshow-fade-leave-to {
  opacity: 0;
  filter: brightness(0.72);
}

</style>

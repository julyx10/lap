<template>
  <div 
    class="w-full h-full relative flex items-center justify-center group"
    @mousemove="handleMouseMove"
    @mouseleave="handleMouseLeave"
    ref="containerRef"
  >
    <!-- Toolbar -->
    <div 
      id="responsiveDiv"
      :class="computedToolbarClass"
      data-tauri-drag-region
    >
      <TButton
        :icon="IconPrev"
        :disabled="fileIndex <= 0 || isSlideShow"
        :tooltip="$t('image_viewer.toolbar.prev') + ` (${fileIndex + 1}/${fileCount})`"
        @click="triggerPrev" 
      />
      <TButton
        :icon="IconNext"
        :disabled="fileIndex < 0 || fileIndex >= fileCount - 1 || isSlideShow"
        :tooltip="$t('image_viewer.toolbar.next') + ` (${fileIndex + 1}/${fileCount})`"
        @click="triggerNext" 
      />
      <TButton
        :icon="isSlideShow ? IconPause : IconPlay"
        :disabled="fileIndex < 0"
        :tooltip="(isSlideShow ? $t('image_viewer.toolbar.pause') : $t('image_viewer.toolbar.slide_show')) + ` (${getSlideShowInterval(config.settings.slideShowInterval)}s)`"
        @click="$emit('toggle-slide-show')" 
      />
      <TButton
        :icon="IconZoomOut"
        :disabled="fileIndex < 0 || imageScale <= imageMinScale"
        :tooltip="$t('image_viewer.toolbar.zoom_out') + ` (${(imageScale * 100).toFixed(0)}%)`"
        @click="zoomOut"
      />
      <TButton
        :icon="IconZoomIn"
        :disabled="fileIndex < 0 || imageScale >= imageMaxScale"
        :tooltip="$t('image_viewer.toolbar.zoom_in') + ` (${(imageScale * 100).toFixed(0)}%)`"
        @click="zoomIn" 
      />
      <TButton
        :icon="!isZoomFit ? IconZoomFit : IconZoomActual"
        :disabled="fileIndex < 0"
        :tooltip="(!isZoomFit ? $t('image_viewer.toolbar.zoom_fit') : $t('image_viewer.toolbar.zoom_actual')) + ` (${(imageScale * 100).toFixed(0)}%)`"
        @click="$emit('update:isZoomFit', !isZoomFit)"
      />
      <TButton
        :icon="config.imageViewer.isPinned ? IconPin : IconUnPin"
        :disabled="fileIndex < 0"
        :tooltip="!config.imageViewer.isPinned ? $t('image_viewer.toolbar.pin') : $t('image_viewer.toolbar.unpin')"
        @click="config.imageViewer.isPinned = !config.imageViewer.isPinned"
      />
      <TButton v-if="isWin"
        :icon="!config.imageViewer.isFullScreen ? IconFullScreen : IconRestoreScreen"
        :tooltip="!config.imageViewer.isFullScreen ? $t('image_viewer.toolbar.fullscreen') : $t('image_viewer.toolbar.exit_fullscreen')"
        @click="config.imageViewer.isFullScreen = !config.imageViewer.isFullScreen"
      />

      <TButton v-if="config.imageViewer.isFullScreen"
        :icon="IconSeparator"
        :disabled="true"
      />

      <TButton v-if="config.imageViewer.isFullScreen"
        :icon="config.imageViewer.isPinned ? IconPin : IconUnPin"
        :disabled="fileIndex < 0"
        :tooltip="!config.imageViewer.isPinned ? $t('image_viewer.toolbar.pin') : $t('image_viewer.toolbar.unpin')"
        @click="config.imageViewer.isPinned = !config.imageViewer.isPinned"
      />
      <TButton v-if="config.imageViewer.isFullScreen"
        :icon="IconClose"
        :tooltip="$t('image_viewer.toolbar.close')"
        @click="$emit('close')"
      />
    </div>

    <!-- Previous Button (Overlay) -->
    <button 
      v-if="showNavButton && hasPrevious && !isSlideShow"
      class="absolute left-2 top-1/2 -translate-y-1/2 z-[70] p-2 rounded-full bg-base-100/30 hover:bg-base-100/70 backdrop-blur-md text-base-content/70 transition-opacity duration-300"
      :class="[ isHoverLeft ? 'opacity-100 pointer-events-auto' : 'opacity-0 pointer-events-none' ]"
      @click.stop="triggerPrev"
      @dblclick.stop
    >
      <IconLeft class="w-8 h-8" />
    </button>

    <!-- Next Button (Overlay) -->
    <button 
      v-if="showNavButton && hasNext && !isSlideShow"
      class="absolute right-2 top-1/2 -translate-y-1/2 z-[70] p-2 rounded-full bg-base-100/30 hover:bg-base-100/70 backdrop-blur-md text-base-content/70 transition-opacity duration-300"
      :class="[ isHoverRight ? 'opacity-100 pointer-events-auto' : 'opacity-0 pointer-events-none' ]"
      @click.stop="triggerNext"
      @dblclick.stop
    >
      <IconRight class="w-8 h-8" />
    </button>

    <Image v-if="file?.file_type === 1"
      ref="mediaRef"
      :filePath="file?.file_path" 
      :rotate="file?.rotate ?? 0" 
      :isZoomFit="isZoomFit"
      @update:isZoomFit="(val: boolean) => $emit('update:isZoomFit', val)"
      @scale="(e) => $emit('scale', e)"
    ></Image>
    
    <Video v-if="file?.file_type === 2"
      ref="mediaRef"
      :filePath="file?.file_path"
      :rotate="file?.rotate ?? 0"
      :isZoomFit="isZoomFit"
    ></Video>

    <ToolTip ref="toolTipRef" />
  </div>
</template>

<script setup lang="ts">
import { defineAsyncComponent, ref, computed } from 'vue';
import { useI18n } from 'vue-i18n';
import { config } from '@/common/config';
import { isWin, getSlideShowInterval } from '@/common/utils';
import Image from '@/components/Image.vue';
import ToolTip from '@/components/ToolTip.vue';
import TButton from '@/components/TButton.vue';
import { 
  IconLeft, 
  IconRight,
  IconPrev,
  IconNext,
  IconPlay,
  IconPause,
  IconZoomIn,
  IconZoomOut,
  IconZoomFit,
  IconZoomActual,
  IconFullScreen,
  IconRestoreScreen,
  IconPin,
  IconUnPin,
  IconSeparator,
  IconClose,
} from '@/common/icons';

const Video = defineAsyncComponent(() => import('@/components/Video.vue'));

const props = defineProps({
  file: {
    type: Object,
    default: null
  },
  isZoomFit: {
    type: Boolean,
    default: true
  },
  showNavButton: {
    type: Boolean,
    default: false
  },
  hasPrevious: {
    type: Boolean,
    default: false
  },
  hasNext: {
    type: Boolean,
    default: false
  },
  // Added ImageToolbar props
  fileIndex: {
    type: Number,
    default: -1
  },
  fileCount: {
    type: Number,
    default: 0
  },
  isSlideShow: {
    type: Boolean,
    default: false
  },
  imageScale: {
    type: Number,
    default: 1
  },
  imageMinScale: {
    type: Number,
    default: 0
  },
  imageMaxScale: {
    type: Number,
    default: 10
  }
});


const emit = defineEmits(['prev', 'next', 'toggle-slide-show', 'close', 'scale', 'update:isZoomFit']);

const { locale, messages } = useI18n();
const localeMsg = computed(() => messages.value[locale.value]);

const containerRef = ref<HTMLElement | null>(null);
const mediaRef = ref<any>(null);
const toolTipRef = ref<any>(null);
const isHoverLeft = ref(false);
const isHoverRight = ref(false);
const isHoverTop = ref(false);
const isHoverBottom = ref(false);
const toolbarPosition = ref<'top' | 'bottom'>('top');

function handleMouseMove(e: MouseEvent) {
  if (!containerRef.value) return;
  
  const rect = containerRef.value.getBoundingClientRect();
  const x = e.clientX - rect.left;
  const y = e.clientY - rect.top;
  const width = rect.width;
  const height = rect.height;
  
  if (width > 0 && height > 0) {
    isHoverLeft.value = x < width * 0.1;
    isHoverRight.value = x > width * 0.9;
    isHoverTop.value = y < height * 0.1;
    isHoverBottom.value = y > height * 0.9;

    if (y < height * 0.5) {
      toolbarPosition.value = 'top';
    } else {
      toolbarPosition.value = 'bottom';
    }
  }
}

function handleMouseLeave() {
  isHoverLeft.value = false;
  isHoverRight.value = false;
  isHoverTop.value = false;
  isHoverBottom.value = false;
}

const computedToolbarClass = computed(() => {
  const baseClasses = 'absolute left-1/2 z-[80] px-4 h-12 space-x-2 bg-base-100/30 hover:bg-base-100/70 backdrop-blur-md rounded-box flex flex-row items-center justify-center transform -translate-x-1/2 select-none transition-all duration-300 ease-in-out';
  
  const isPinned = config.imageViewer.isPinned;
  // If pinned, always show at top (or last known position? Standard is top). Let's stick to top for pinned.
  if (isPinned) {
    return `${baseClasses} top-2 translate-y-0 opacity-100`;
  }

  if (toolbarPosition.value === 'bottom') {
    // Bottom position
    if (isHoverBottom.value) {
      return `${baseClasses} bottom-2 translate-y-0 opacity-100`;
    } else {
      // Hidden at bottom
      return `${baseClasses} bottom-2 translate-y-4 opacity-0`;
    }
  } else {
    // Top position (default)
    if (isHoverTop.value) {
      return `${baseClasses} top-2 translate-y-0 opacity-100`;
    } else {
      // Hidden at top
      return `${baseClasses} top-2 -translate-y-4 opacity-0`;
    }
  }
});

// Expose methods for parent component (ImageViewer)
const zoomIn = () => mediaRef.value?.zoomIn();
const zoomOut = () => mediaRef.value?.zoomOut();
const zoomActual = () => mediaRef.value?.zoomActual();
const rotateRight = () => mediaRef.value?.rotateRight();
const showMessage = (message: string, isWarning: boolean = false) => toolTipRef.value?.showTip(message, isWarning);
const showTip = (message: string, isWarning: boolean = false) => toolTipRef.value?.showTip(message, isWarning);

const triggerPrev = () => {
  if (props.hasPrevious) {
    emit('prev');
  } else {
    showTip((localeMsg.value as any).tooltip.image_viewer.first_image);
  }
}

const triggerNext = () => {
  if (props.hasNext) {
    emit('next');
  } else {
    showTip((localeMsg.value as any).tooltip.image_viewer.last_image);
  }
}

defineExpose({
  zoomIn,
  zoomOut,
  zoomActual,
  rotateRight,
  showMessage,
  triggerPrev,
  triggerNext
});
</script>

<style scoped>
/* Disable text selection while dragging */
* {
  user-select: none;
}
 
@media (max-width: 600px) {
  #responsiveDiv {
    visibility: hidden;
  }
}
@media (min-width: 600px) {
  #responsiveDiv {
    visibility: visible;
  }
}
</style>

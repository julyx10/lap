<template>
  <div 
    class="w-full h-full relative flex flex-col items-center justify-center group"
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
      <!-- File Name (Pinned Mode) -->
      <div 
        v-if="props.mode === 2 && !isFullScreen" 
        class="absolute left-20 text-sm text-base-content/70 truncate select-none"
        :style="{ maxWidth: filenameMaxWidth + 'px' }"
        data-tauri-drag-region
      >
        {{ fileIndex + 1 }}/{{ fileCount }} {{ file?.name }}
      </div>
      
      <div ref="buttonsRef" class="flex items-center space-x-1">
        <TButton
          :icon="IconPrev"
          :disabled="fileIndex <= 0 || isSlideShow"
          :tooltip="$t('image_viewer.toolbar.prev')"
          @click="triggerPrev" 
        />
        <TButton
          :icon="IconNext"
          :disabled="fileIndex < 0 || fileIndex >= fileCount - 1 || isSlideShow"
          :tooltip="$t('image_viewer.toolbar.next')"
          @click="triggerNext" 
        />
        <TButton
          :icon="isSlideShow ? IconPause : IconPlay"
          :disabled="fileIndex < 0"
          :selected="isSlideShow"
          :tooltip="(isSlideShow ? $t('image_viewer.toolbar.pause') : $t('image_viewer.toolbar.slide_show')) + ` (${getSlideShowInterval(config.settings.slideShowInterval)}s)`"
          @click="handleToggleSlideShow" 
        />
        <TButton
          :icon="IconZoomOut"
          :disabled="fileIndex < 0 || imageScale <= imageMinScale || isSlideShow"
          :tooltip="$t('image_viewer.toolbar.zoom_out') + ` (${(imageScale * 100).toFixed(0)}%)`"
          @click="zoomOut"
        />
        <TButton
          :icon="IconZoomIn"
          :disabled="fileIndex < 0 || imageScale >= imageMaxScale || isSlideShow"
          :tooltip="$t('image_viewer.toolbar.zoom_in') + ` (${(imageScale * 100).toFixed(0)}%)`"
          @click="zoomIn" 
        />
        <TButton
          :icon="!isZoomFit ? IconZoomFit : IconZoomActual"
          :disabled="fileIndex < 0 || isSlideShow"
          :tooltip="(!isZoomFit ? $t('image_viewer.toolbar.zoom_fit') : $t('image_viewer.toolbar.zoom_actual')) + ` (${(imageScale * 100).toFixed(0)}%)`"
          @click="$emit('update:isZoomFit', !isZoomFit)"
        />
        <template v-if="showExtraIcons && mode !== 2">
          <IconSeparator class="t-icon-size-sm text-base-content/30" />
          <TButton
            :icon="IconFavorite"
            :disabled="fileIndex < 0 || isSlideShow"
            :selected="file?.is_favorite && !isSlideShow"
            :tooltip="file?.is_favorite ? $t('menu.meta.unfavorite') : $t('menu.meta.favorite')"
            @click="$emit('item-action', { action: 'favorite', index: fileIndex })"
          />
          <TButton
            :icon="IconTag"
            :disabled="fileIndex < 0 || isSlideShow"
            :selected="file?.has_tags && !isSlideShow"
            :tooltip="$t('menu.meta.tag')"
            @click="$emit('item-action', { action: 'tag', index: fileIndex })"
          />
          <TButton
            :icon="IconComment"
            :disabled="fileIndex < 0 || isSlideShow"
            :selected="!!file?.comments && !isSlideShow"
            :tooltip="$t('menu.meta.comment')"
            @click="$emit('item-action', { action: 'comment', index: fileIndex })"
          />
          <TButton
            :icon="IconRotate"
            :disabled="fileIndex < 0 || isSlideShow"
            :iconStyle="{ transform: `rotate(${file?.rotate ?? 0}deg)`, transition: 'transform 0.3s' }"
            :selected="file?.rotate % 360 > 0 && !isSlideShow"
            :tooltip="$t('menu.meta.rotate')"
            @click="$emit('item-action', { action: 'rotate', index: fileIndex })"
          />
        </template>
        <ContextMenu v-if="mode !== 2"
          :iconMenu="IconMore"
          :menuItems="singleFileMenuItems"
          :disabled="fileIndex < 0 || isSlideShow"
          @click.stop
        />
        <IconSeparator v-if="mode !== 2" class="t-icon-size-sm text-base-content/30" />
        <TButton v-if="mode === 0"
          :icon="!isFullScreen ? IconFullScreen : IconRestoreScreen"
          :tooltip="!isFullScreen ? $t('image_viewer.toolbar.fullscreen') : $t('image_viewer.toolbar.exit_fullscreen')"
          @click="$emit('toggle-full-screen')"
        />
        <TButton v-if="mode !== 2"
          :icon="config.mediaViewer.isPinned ? IconPin : IconUnPin"
          :disabled="fileIndex < 0"
          :tooltip="!config.mediaViewer.isPinned ? $t('image_viewer.toolbar.pin') : $t('image_viewer.toolbar.unpin')"
          @click="config.mediaViewer.isPinned = !config.mediaViewer.isPinned"
        />
        <TButton
          v-if="mode === 0 && config.mediaViewer.isPinned"
          :icon="IconClose"
          :tooltip="$t('image_viewer.toolbar.close')"
          @click.stop="$emit('close')"
        />
      </div>
    </div>

    <!-- Previous Button (Overlay) -->
    <button 
      v-if="hasPrevious && !isSlideShow"
      class="absolute left-2 top-1/2 -translate-y-1/2 z-[70] p-2 rounded-full bg-base-100/30 hover:text-base-content hover:bg-base-100/80 backdrop-blur-md cursor-pointer"
      :class="[ isHoverLeft ? 'opacity-100 pointer-events-auto' : 'opacity-0 pointer-events-none' ]"
      @click.stop="triggerPrev"
      @dblclick.stop
    >
      <IconLeft class="w-8 h-8" />
    </button>

    <!-- Next Button (Overlay) -->
    <button 
      v-if="hasNext && !isSlideShow"
      class="absolute right-2 top-1/2 -translate-y-1/2 z-[70] p-2 rounded-full bg-base-100/30 hover:text-base-content hover:bg-base-100/80 backdrop-blur-md cursor-pointer"
      :class="[ isHoverRight ? 'opacity-100 pointer-events-auto' : 'opacity-0 pointer-events-none' ]"
      @click.stop="triggerNext"
      @dblclick.stop
    >
      <IconRight class="w-8 h-8" />
    </button>

    <!-- Close Button (Top Right) -->
    <button 
      v-if="mode === 0 && !config.mediaViewer.isPinned"
      class="absolute right-2 top-2 z-[90] p-2 rounded-full text-base-content/70 hover:text-base-content hover:bg-base-100/70 cursor-pointer"
      @click.stop="$emit('close')"
      @dblclick.stop
    >
      <IconClose class="w-4 h-4" />
    </button>

    <div class="flex-1 w-full min-h-0 relative">
      <Image v-if="file?.file_type === 1"
        ref="mediaRef"
        :filePath="file?.file_path" 
        :rotate="file?.rotate ?? 0" 
        :isZoomFit="isZoomFit"
        :isSlideShow="isSlideShow"
        @update:isZoomFit="(val: boolean) => $emit('update:isZoomFit', val)"
        @scale="(e) => $emit('scale', e)"
        @message-from-image-viewer="handleMessageFromImageViewer"
      ></Image>
      
      <Video v-if="file?.file_type === 2"
        ref="mediaRef"
        :filePath="file?.file_path"
        :rotate="file?.rotate ?? 0"
        :isZoomFit="isZoomFit"
        :isSlideShow="isSlideShow"
      ></Video>
    </div>

    <ToolTip ref="toolTipRef" />
  </div>
</template>

<script setup lang="ts">
import { defineAsyncComponent, ref, computed, onMounted, onBeforeUnmount } from 'vue';
import { useI18n } from 'vue-i18n';
import { config, libConfig } from '@/common/config';
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
  IconMore,
  IconFavorite,
  IconFavoriteFilled,
  IconTag,
  IconComment,
  IconRotate,
} from '@/common/icons';
import { isMac } from '@/common/utils';
import ContextMenu from '@/components/ContextMenu.vue';
import { useFileMenuItems } from '@/common/fileMenu';

const Video = defineAsyncComponent(() => import('@/components/Video.vue'));

const props = defineProps({
  // 0: quick view, 1: filmstrip, 2: image viewer
  mode: {
    type: Number,
    default: 0
  },
  isFullScreen: {
    type: Boolean,
    default: false
  },
  file: {
    type: Object,
    default: null
  },
  hasPrevious: {
    type: Boolean,
    default: false
  },
  hasNext: {
    type: Boolean,
    default: false
  },
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
  },
  isZoomFit: {
    type: Boolean,
    default: true
  },
});

const emit = defineEmits(['prev', 'next', 'toggle-slide-show', 'close', 'scale', 'update:isZoomFit', 'item-action', 'toggle-full-screen']);

const { locale, messages } = useI18n();
const localeMsg = computed(() => messages.value[locale.value] as any);

const containerRef = ref<HTMLElement | null>(null);
const mediaRef = ref<any>(null);
const toolTipRef = ref<any>(null);
const isHoverLeft = ref(false);
const isHoverRight = ref(false);
const isHoverTop = ref(false);
const isHoverBottom = ref(false);
const toolbarPosition = ref<'top' | 'bottom'>('top');

// Responsive toolbar
const containerWidth = ref(0);
const buttonsRef = ref<HTMLElement | null>(null);
const buttonsWidth = ref(0);
const filenameMaxWidth = computed(() => {
  if (containerWidth.value > 0 && buttonsWidth.value > 0) {
    const val = (containerWidth.value / 2) - (buttonsWidth.value / 2) - 100;
    return Math.max(0, val);
  }
  return 200; // Fallback
});
const showExtraIcons = computed(() => containerWidth.value > 600);
let resizeObserver: ResizeObserver | null = null;

onMounted(() => {
  resizeObserver = new ResizeObserver((entries) => {
    for (const entry of entries) {
      if (entry.target === containerRef.value) {
        containerWidth.value = entry.contentRect.width;
      } else if (entry.target === buttonsRef.value) {
        buttonsWidth.value = entry.contentRect.width;
      }
    }
  });

  if (containerRef.value) {
    resizeObserver.observe(containerRef.value);
  }
  if (buttonsRef.value) {
    resizeObserver.observe(buttonsRef.value);
  }
});

onBeforeUnmount(() => {
  if (resizeObserver) {
    resizeObserver.disconnect();
  }
});

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
    isHoverTop.value = y < 60;
    isHoverBottom.value = y > height - 60;

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
  const commonClasses = 'absolute z-[80] h-10 flex flex-row items-center justify-center select-none';
  
  const isPinned = props.isFullScreen ? false : (props.mode === 2 ? true : config.mediaViewer.isPinned);

  if (isPinned) {
    // Fixed Top Bar
    return `${commonClasses} relative top-0 left-0 w-full`;
  } else {
    // Floating Hover Bar
    const floatingClasses = 'left-1/2 -translate-x-1/2 px-2 rounded-box bg-base-100/30 hover:bg-base-100/70 transition-[opacity,transform] duration-300 ease-in-out';
    
    if (toolbarPosition.value === 'bottom') {
       if (isHoverBottom.value) {
          return `${commonClasses} ${floatingClasses} bottom-2 opacity-100`;
       } else {
          return `${commonClasses} ${floatingClasses} bottom-2 opacity-0`; 
       }
    } else {
       if (isHoverTop.value) {
          return `${commonClasses} ${floatingClasses} top-2 opacity-100`;
       } else {
          return `${commonClasses} ${floatingClasses} top-2 opacity-0`;
       }
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

const handleToggleSlideShow = () => {
  if (!props.isSlideShow) {
    emit('update:isZoomFit', true);
  }
  emit('toggle-slide-show');
}

const handleMessageFromImageViewer = (payload: { message: string }) => {
  if (payload.message === 'prev') {
    triggerPrev();
  } else if (payload.message === 'next') {
    triggerNext();
  }
};

defineExpose({
  zoomIn,
  zoomOut,
  zoomActual,
  rotateRight,
  showMessage,
  triggerPrev,
  triggerNext
});

const showFolderFiles = computed(() => {
  return !!(config.main.sidebarIndex === 0 && libConfig.album.id && libConfig.album.id !== 0);
});

const selectedFile = computed(() => props.file);

const singleFileMenuItems = computed(() => {
  if (props.mode === 2) return [];

  return useFileMenuItems(
    selectedFile,
    localeMsg,
    isMac,
    showFolderFiles,
    (action) => emit('item-action', { action, index: props.fileIndex })
  ).value;
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

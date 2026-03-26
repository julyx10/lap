<template>
  <div 
    :class="['w-full relative flex flex-col items-center justify-center', toolbarOnly ? '' : 'h-full group']"
    @mousemove="handleMouseMove"
    @mouseleave="handleMouseLeave"
    @contextmenu.prevent="handleContextMenu"
    ref="containerRef"
  >
    <!-- Toolbar -->
    <div 
      v-if="showToolbar"
      id="responsiveDiv"
      :class="computedToolbarClass"
      :style="showWindowControls && isWin ? { paddingLeft: '144px', paddingRight: '144px' } : undefined"
      data-tauri-drag-region
    >
      <!-- App Icon + Title (left side, ImageViewer on Windows) -->
      <div v-if="isWin && mode === 2" class="absolute left-0 top-0 h-10 flex items-center px-3 select-none" data-tauri-drag-region>
        <img :src="iconLogo" class="w-5 h-5 mr-2 rounded" data-tauri-drag-region />
        <span class="text-nowrap text-sm text-base-content/70 overflow-hidden whitespace-pre text-ellipsis" data-tauri-drag-region>
          {{ $t('image_viewer.title') }}
        </span>
      </div>
      <div ref="buttonsRef" class="flex items-center space-x-1">
        <TButton
          :icon="IconPrev"
          :disabled="fileIndex <= 0 || isSlideShow || !canInteract"
          :tooltip="$t('image_viewer.toolbar.prev')"
          @click="triggerPrev" 
        />
        <TButton
          :icon="IconNext"
          :disabled="fileIndex < 0 || fileIndex >= fileCount - 1 || isSlideShow || !canInteract"
          :tooltip="$t('image_viewer.toolbar.next')"
          @click="triggerNext" 
        />
        <div class="flex items-center gap-0.5">
          <TButton
            :icon="isSlideShow ? IconPause : IconPlay"
            :disabled="fileIndex < 0 || !canSlideShow || !canInteract"
            :selected="isSlideShow && canSlideShow"
            :tooltip="!canSlideShow
              ? $t('image_viewer.toolbar.slide_show')
              : (isSlideShow ? $t('image_viewer.toolbar.pause') : $t('image_viewer.toolbar.slide_show'))"
            shortcut="P"
            @click="handleToggleSlideShow"
          />
          <ContextMenu v-if="isSlideShow && canSlideShow"
            :menuItems="slideShowIntervalMenuItems"
            :disabled="fileIndex < 0 || !canSlideShow || !canInteract"
            @open-change="handleMenuOpenChange"
            @click.stop
          >
            <template #trigger="{ toggle }">
              <button
                class="h-7 min-w-[38px] px-1.5 inline-flex items-center justify-center gap-0.5 rounded-box text-[11px] font-medium tabular-nums transition-colors"
                :class="[
                  fileIndex < 0 || !canSlideShow || !canInteract
                    ? 'cursor-default text-base-content/30'
                    : 'text-base-content/45 hover:bg-base-100/30 hover:text-base-content/75'
                ]"
                :disabled="fileIndex < 0 || !canSlideShow || !canInteract"
                :title="$t('settings.image_view.slide_show_interval', { second: getSlideShowInterval(effectiveSlideShowIntervalIndex) })"
                @click.stop="toggle"
              >
                <span>{{ currentSlideShowIntervalLabel }}</span>
                <IconArrowDown class="w-3 h-3" />
              </button>
            </template>
          </ContextMenu>
        </div>
        <TButton
          :icon="IconZoomOut"
          :disabled="fileIndex < 0 || imageScale <= imageMinScale || isSlideShow || !canInteract"
          :tooltip="$t('image_viewer.toolbar.zoom_out')"
          shortcut="-"
          @click="handleZoomOut"
        />
        <TButton
          :icon="IconZoomIn"
          :disabled="fileIndex < 0 || imageScale >= imageMaxScale || isSlideShow || !canInteract"
          :tooltip="$t('image_viewer.toolbar.zoom_in')"
          shortcut="="
          @click="handleZoomIn" 
        />
        <TButton
          :icon="!isZoomFit ? IconZoomFit : IconZoomActual"
          :disabled="fileIndex < 0 || isSlideShow || !canInteract"
          :tooltip="!isZoomFit ? $t('image_viewer.toolbar.zoom_fit') : $t('image_viewer.toolbar.zoom_actual')"
          shortcut="Space"
          @click="$emit('update:isZoomFit', !isZoomFit)"
        />
        <template v-if="showExtraIcons">
          <IconSeparator class="t-icon-size-sm text-base-content/30" />
          <TButton
            :icon="file?.is_favorite ? IconHeartFilled : IconHeart"
            :disabled="fileIndex < 0 || isSlideShow || !canInteract"
            :selected="file?.is_favorite && !isSlideShow"
            :tooltip="file?.is_favorite ? $t('menu.meta.unfavorite') : $t('menu.meta.favorite')"
            shortcut="F"
            @click="$emit('item-action', { action: 'favorite', index: fileIndex })"
          />
          <ContextMenu
            :menuItems="ratingMenuItems"
            :disabled="fileIndex < 0 || isSlideShow || !canInteract"
            @open-change="handleMenuOpenChange"
            @click.stop
          >
            <template #trigger="{ toggle }">
              <TButton
                :icon="Number(file?.rating || 0) > 0 ? IconStarFilled : IconStar"
                :disabled="fileIndex < 0 || isSlideShow || !canInteract"
                :selected="Number(file?.rating || 0) > 0 && !isSlideShow"
                :tooltip="$t('favorite.ratings')"
                shortcut="0-5"
                @click.stop="toggle"
              />
            </template>
          </ContextMenu>
          <TButton
            :icon="IconTag"
            :disabled="fileIndex < 0 || isSlideShow || !canInteract"
            :selected="file?.has_tags && !isSlideShow"
            :tooltip="$t('menu.meta.tag')"
            shortcut="T"
            @click="$emit('item-action', { action: 'tag', index: fileIndex })"
          />
          <TButton
            :icon="IconComment"
            :disabled="fileIndex < 0 || isSlideShow || !canInteract"
            :selected="!!file?.comments && !isSlideShow"
            :tooltip="$t('menu.meta.comment')"
            shortcut="C"
            @click="$emit('item-action', { action: 'comment', index: fileIndex })"
          />
          <TButton
            :icon="IconRotate"
            :disabled="fileIndex < 0 || isSlideShow || !canInteract"
            :iconStyle="{ transform: `rotate(${file?.rotate ?? 0}deg)`, transition: 'transform 0.3s' }"
            :selected="file?.rotate % 360 > 0 && !isSlideShow"
            :tooltip="$t('menu.meta.rotate')"
            shortcut="R"
            @click="$emit('item-action', { action: 'rotate', index: fileIndex })"
          />
          <!-- <TButton
            v-if="mode !== 2"
            :icon="IconFileInfo"
            :disabled="fileIndex < 0 || isSlideShow || !canInteract"
            :tooltip="$t('menu.meta.info')"
            shortcut="I"
            @click="$emit('item-action', { action: 'info', index: fileIndex })"
          /> -->
        </template>
        <!-- Split/Sync Viewport Buttons (ImageViewer mode only) -->
        <template v-if="mode === 2">
          <IconSeparator class="t-icon-size-sm text-base-content/30" />
          <TButton
            :icon="IconLink"
            :selected="isSplit && isSyncViewport"
            :disabled="!isSplit"
            :tooltip="isSplit
              ? (isSyncViewport ? $t('image_viewer.toolbar.sync_viewport_off') : $t('image_viewer.toolbar.sync_viewport_on'))
              : $t('image_viewer.toolbar.sync_viewport_need_split')"
            @click="$emit('item-action', { action: 'toggle-sync-viewport' })"
          />
          <TButton
            :icon="IconSplitOn"
            :selected="isSplit"
            :tooltip="isSplit ? $t('image_viewer.toolbar.split_off') : $t('image_viewer.toolbar.split_on')"
            @click="$emit('item-action', { action: 'toggle-split' })"
          />
        </template>
        <ContextMenu v-if="mode !== 2"
          ref="contextMenuRef"
          :iconMenu="IconMore"
          :menuItems="singleFileMenuItems"
          :disabled="fileIndex < 0 || isSlideShow || !canInteract"
          @open-change="handleMenuOpenChange"
          @click.stop
        />
        <IconSeparator v-if="mode !== 2" class="t-icon-size-sm text-base-content/30" />
        <TButton
          v-if="mode === 2"
          :icon="!isFullScreen ? IconFullScreen : IconRestoreScreen"
          :tooltip="!isFullScreen ? $t('image_viewer.toolbar.fullscreen') : $t('image_viewer.toolbar.exit_fullscreen')"
          :disabled="!canInteract"
          @click="$emit('toggle-full-screen')"
        />
        <TButton v-if="mode !== 2 && !isFullScreen"
          :icon="config.mediaViewer.isPinned ? IconPin : IconUnPin"
          :disabled="fileIndex < 0 || !canInteract"
          :tooltip="!config.mediaViewer.isPinned ? $t('image_viewer.toolbar.pin') : $t('image_viewer.toolbar.unpin')"
          @click="config.mediaViewer.isPinned = !config.mediaViewer.isPinned"
        />
        <TButton
          v-if="mode === 0 && config.mediaViewer.isPinned"
          :icon="IconClose"
          :tooltip="$t('image_viewer.toolbar.close')"
          :disabled="!canInteract"
          @click.stop="$emit('close')"
        />
      </div>
    </div>

    <!-- Window Control Buttons (top-right) -->
    <div v-if="showToolbar && showWindowControls && isWin" class="absolute top-0 right-0 z-90 flex items-center" @mousedown.stop>
      <IconWinMinus 
        class="p-3 w-12 h-10 text-base-content/70 hover:text-base-content hover:bg-base-100 transition-colors duration-300 cursor-pointer" 
        @click.stop="minimizeWindow" 
      />
      <component :is="isMaximized ? IconWinRestore : IconWinMaximize" 
        class="p-3 w-12 h-10 text-base-content/70 hover:text-base-content hover:bg-base-100 transition-colors duration-300 cursor-pointer" 
        @click.stop="toggleMaximizeWindow" 
      />
      <IconClose 
        class="p-3 w-12 h-10 text-base-content/70 hover:text-base-content hover:bg-red-500 transition-colors duration-300 cursor-pointer" 
        @mousedown.stop="$emit('close')"
        @click.stop="$emit('close')" 
      />
    </div>

    <!-- Elements below only rendered when not toolbar-only -->
    <template v-if="!toolbarOnly">
    <!-- Close Button (Top Right) -->
    <button 
      v-if="mode === 0 && !config.mediaViewer.isPinned && !isFullScreen"
      class="absolute right-2 top-2 z-90 p-2 rounded-full text-base-content/70 hover:text-base-content hover:bg-base-100/70 cursor-pointer"
      @click.stop="$emit('close')"
      @dblclick.stop
    >
      <IconClose class="w-4 h-4" />
    </button>

    <div
      v-if="showStatusBadges && quickViewStatusBadges.length > 0"
      class="pointer-events-none absolute inset-x-0 top-0 z-80 h-16"
    ></div>
    <div
      v-if="showStatusBadges && quickViewStatusBadges.length > 0"
      :class="[
        'pointer-events-none absolute left-2 z-90 flex max-w-[calc(100%-4rem)] flex-wrap gap-1',
        props.mode === 2 || config.mediaViewer.isPinned || isFullScreen ? 'top-12' : 'top-2',
      ]"
    >
      <div
        v-for="badge in quickViewStatusBadges"
        :key="badge.key"
        :class="['thumb-badge', badge.highlight ? 'thumb-badge-highlight' : 'thumb-badge-muted']"
      >
        <component
          v-if="badge.icon"
          :is="badge.icon"
          class="h-3.5 w-3.5 shrink-0"
        />
        <span v-if="badge.label" class="leading-none">{{ badge.label }}</span>
      </div>
    </div>

    <div ref="mediaAreaRef" class="flex-1 w-full min-h-0 relative" @dblclick="$emit('media-dblclick')">
      <!-- Previous Button (Overlay, media-area anchored) -->
      <button 
        v-if="!isSlideShow && showOverlayNav"
        class="absolute left-2 -translate-y-1/2 z-70 p-2 rounded-full bg-base-100/30 backdrop-blur-md transition-opacity duration-200"
        :style="{ top: navButtonsTop }"
        :class="[ 
          isHoverLeft ? (hasPrevious ? 'opacity-100 pointer-events-auto hover:text-base-content hover:bg-base-100/80 cursor-pointer' : 'opacity-30 cursor-default') : 'opacity-0 pointer-events-none' 
        ]"
        :disabled="!hasPrevious"
        @click.stop="triggerPrev"
        @dblclick.stop
      >
        <IconLeft class="w-8 h-8" />
      </button>

      <!-- Next Button (Overlay, media-area anchored) -->
      <button 
        v-if="!isSlideShow && showOverlayNav"
        class="absolute right-2 -translate-y-1/2 z-70 p-2 rounded-full bg-base-100/30 backdrop-blur-md transition-opacity duration-200"
        :style="{ top: navButtonsTop }"
        :class="[ 
          isHoverRight ? (hasNext ? 'opacity-100 pointer-events-auto hover:text-base-content hover:bg-base-100/80 cursor-pointer' : 'opacity-30 cursor-default') : 'opacity-0 pointer-events-none' 
        ]"
        :disabled="!hasNext"
        @click.stop="triggerNext"
        @dblclick.stop
      >
        <IconRight class="w-8 h-8" />
      </button>

      <Image v-if="file?.file_type === 1 || file?.file_type === 3"
        ref="mediaRef"
        :filePath="file?.file_path" 
        :fileId="file?.id"
        :fileType="file?.file_type"
        :thumbnailSrc="file?.thumbnail || ''"
        :nextFilePath="nextFilePath"
        :rotate="file?.rotate ?? 0" 
        :isZoomFit="isZoomFit"
        :isSlideShow="isSlideShow"
        :slideShowTransitionMode="slideShowTransitionMode"
        @update:isZoomFit="(val: boolean) => $emit('update:isZoomFit', val)"
        @scale="(e) => $emit('scale', e)"
        @viewport-change="(e) => $emit('viewport-change', e)"
        @message-from-image-viewer="handleMessageFromImageViewer"
      ></Image>
      
      <Video v-if="file?.file_type === 2"
        ref="mediaRef"
        :filePath="file?.file_path"
        :rotate="file?.rotate ?? 0"
        :isZoomFit="isZoomFit"
        :isSlideShow="isSlideShow"
        @scale="(e) => $emit('scale', e)"
        @viewport-change="(e) => $emit('viewport-change', e)"
        @message-from-video-viewer="handleMessageFromImageViewer"
        @slideshow-next="emit('slideshow-next')"
      ></Video>
    </div>

    <ToolTip ref="toolTipRef" />
    </template>
  </div>
</template>

<script setup lang="ts">
import { defineAsyncComponent, ref, computed, onMounted, onBeforeUnmount } from 'vue';
import { useI18n } from 'vue-i18n';
import { getCurrentWindow } from '@tauri-apps/api/window';
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
  IconArrowDown,
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
  IconHeart,
  IconHeartFilled,
  IconStar,
  IconStarFilled,
  IconTag,
  IconComment,
  IconRotate,
  IconFileInfo,
  IconDot,
  IconWinMinus,
  IconWinMaximize,
  IconWinRestore,
  IconLink,
  IconSplitOn,
} from '@/common/icons';
import { isMac } from '@/common/utils';
import ContextMenu from '@/components/ContextMenu.vue';
import iconLogo from '@/assets/images/logo.png';
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
  nextFilePath: {
    type: String,
    default: ''
  },
  isSlideShow: {
    type: Boolean,
    default: false
  },
  canSlideShow: {
    type: Boolean,
    default: true
  },
  slideShowIntervalIndex: {
    type: Number,
    default: null
  },
  canInteract: {
    type: Boolean,
    default: true
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
  isSplit: {
    type: Boolean,
    default: false
  },
  isSyncViewport: {
    type: Boolean,
    default: false
  },
  showWindowControls: {
    type: Boolean,
    default: false
  },
  showToolbar: {
    type: Boolean,
    default: true
  },
  showOverlayNav: {
    type: Boolean,
    default: true
  },
  toolbarOnly: {
    type: Boolean,
    default: false
  },
  forceToolbarVisible: {
    type: Boolean,
    default: false
  },
});

const emit = defineEmits([
  'prev', 
  'next', 
  'toggle-slide-show', 
  'update:slideShowIntervalIndex',
  'close', 
  'scale', 
  'update:isZoomFit', 
  'item-action', 
  'toggle-full-screen', 
  'slideshow-next', 
  'media-dblclick', 
  'viewport-change'
]);

const { locale, messages } = useI18n();
const localeMsg = computed(() => messages.value[locale.value] as any);

const contextMenuRef = ref<any>(null);
const containerRef = ref<HTMLElement | null>(null);
const mediaAreaRef = ref<HTMLElement | null>(null);
const mediaRef = ref<any>(null);
const toolTipRef = ref<any>(null);
const isHoverLeft = ref(false);
const isHoverRight = ref(false);
const isHoverTop = ref(false);
const isHoverBottom = ref(false);
const toolbarPosition = ref<'top' | 'bottom'>('top');
const hasOpenMenu = ref(false);
const navButtonsTop = ref('50%');

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
// Window control state (Windows + ImageViewer mode)
const winAppWindow = isWin ? getCurrentWindow() : null;
const isMaximized = ref(false);

const minimizeWindow = () => winAppWindow?.minimize();
const toggleMaximizeWindow = () => {
  winAppWindow?.isMaximized().then((maximized) => {
    if (maximized) {
      isMaximized.value = false;
      winAppWindow?.unmaximize();
    } else {
      isMaximized.value = true;
      winAppWindow?.maximize();
    }
  });
};
const closeWindow = () => winAppWindow?.close();
const effectiveSlideShowIntervalIndex = computed(() => {
  return props.slideShowIntervalIndex ?? config.settings.slideShowInterval;
});
const currentSlideShowIntervalLabel = computed(() => `${getSlideShowInterval(effectiveSlideShowIntervalIndex.value)}s`);
const slideShowTransitionMode = computed(() => Number(config.settings.slideShowTransition ?? 0));
// const ratingButtonTooltip = computed(() => {
//   const rating = Number(props.file?.rating || 0);
//   return rating > 0 ? `${localeMsg.value.favorite.ratings}: ${rating}` : localeMsg.value.favorite.ratings;
// });
const ratingMenuItems = computed(() => {
  const rating = Number(props.file?.rating || 0);
  return [
    {
      label: localeMsg.value.favorite.clear_rating,
      icon: IconStar,
      shortcut: '0',
      action: () => emit('item-action', { action: 'rating-0', index: props.fileIndex }),
    },
    { label: '-', action: null },
    {
      label: localeMsg.value.favorite.five_stars,
      icon: rating === 5 ? IconStarFilled : IconStar,
      shortcut: '5',
      action: () => emit('item-action', { action: 'rating-5', index: props.fileIndex }),
    },
    {
      label: localeMsg.value.favorite.four_stars,
      icon: rating === 4 ? IconStarFilled : IconStar,
      shortcut: '4',
      action: () => emit('item-action', { action: 'rating-4', index: props.fileIndex }),
    },
    {
      label: localeMsg.value.favorite.three_stars,
      icon: rating === 3 ? IconStarFilled : IconStar,
      shortcut: '3',
      action: () => emit('item-action', { action: 'rating-3', index: props.fileIndex }),
    },
    {
      label: localeMsg.value.favorite.two_stars,
      icon: rating === 2 ? IconStarFilled : IconStar,
      shortcut: '2',
      action: () => emit('item-action', { action: 'rating-2', index: props.fileIndex }),
    },
    {
      label: localeMsg.value.favorite.one_star,
      icon: rating === 1 ? IconStarFilled : IconStar,
      shortcut: '1',
      action: () => emit('item-action', { action: 'rating-1', index: props.fileIndex }),
    },
  ];
});

const slideShowIntervalOptions = [1, 3, 5, 10, 15, 30];
const slideShowIntervalMenuItems = computed(() => {
  const currentInterval = getSlideShowInterval(effectiveSlideShowIntervalIndex.value);
  return slideShowIntervalOptions.map((seconds, index) => ({
    label: `${seconds}s`,
    icon: currentInterval === seconds ? IconDot : null,
    action: () => {
      if (props.slideShowIntervalIndex !== null) {
        emit('update:slideShowIntervalIndex', index);
      } else {
        config.settings.slideShowInterval = index;
      }
    },
  }));
});

const quickViewStatusBadges = computed(() => {
  const badges: Array<{ key: string; icon: any; label?: string; highlight?: boolean }> = [];
  const rating = Number(props.file?.rating || 0);

  if (props.file?.is_favorite) {
    badges.push({
      key: 'favorite',
      icon: IconHeartFilled,
      label: rating > 0 ? `${rating}` : undefined,
      highlight: true,
    });
  } else if (rating > 0) {
    badges.push({
      key: 'rating',
      icon: IconStarFilled,
      label: `${rating}`,
      highlight: true,
    });
  }

  return badges;
});

const showStatusBadges = computed(() => {
  return !props.isFullScreen && (props.mode === 0 || props.mode === 1 || props.mode === 2);
});
let resizeObserver: ResizeObserver | null = null;
const updateNavButtonsTop = () => {
  if (!containerRef.value || !mediaAreaRef.value) {
    navButtonsTop.value = '50%';
    return;
  }
  const containerRect = containerRef.value.getBoundingClientRect();
  const mediaRect = mediaAreaRef.value.getBoundingClientRect();
  if (containerRect.height <= 0 || mediaRect.height <= 0) {
    navButtonsTop.value = '50%';
    return;
  }
  const centerY = mediaRect.top - containerRect.top + mediaRect.height / 2;
  navButtonsTop.value = `${Math.round(centerY)}px`;
};

onMounted(() => {
  resizeObserver = new ResizeObserver((entries) => {
    for (const entry of entries) {
      if (entry.target === containerRef.value) {
        containerWidth.value = entry.contentRect.width;
      } else if (entry.target === buttonsRef.value) {
        buttonsWidth.value = entry.contentRect.width;
      }
    }
    updateNavButtonsTop();
  });

  if (containerRef.value) {
    resizeObserver.observe(containerRef.value);
  }
  if (mediaAreaRef.value) {
    resizeObserver.observe(mediaAreaRef.value);
  }
  if (buttonsRef.value) {
    resizeObserver.observe(buttonsRef.value);
  }
  updateNavButtonsTop();
  window.addEventListener('resize', updateNavButtonsTop);
});

onBeforeUnmount(() => {
  window.removeEventListener('resize', updateNavButtonsTop);
  if (resizeObserver) {
    resizeObserver.disconnect();
  }
});

function handleMouseMove(e: MouseEvent) {
  if (!containerRef.value) return;

  // Toolbar hover logic: based on the full viewer container.
  const containerRect = containerRef.value.getBoundingClientRect();
  if (containerRect.width > 0 && containerRect.height > 0) {
    const containerY = e.clientY - containerRect.top;
    const containerHeight = containerRect.height;
    isHoverTop.value = containerY < 60;
    isHoverBottom.value = containerY > containerHeight - 60;
    toolbarPosition.value = containerY < containerHeight * 0.5 ? 'top' : 'bottom';
  }

  // Prev/next hover logic: based on actual media display area (accounts for panels/toolbar layout).
  if (!mediaAreaRef.value) return;
  const mediaRect = mediaAreaRef.value.getBoundingClientRect();
  if (mediaRect.width <= 0 || mediaRect.height <= 0) {
    isHoverLeft.value = false;
    isHoverRight.value = false;
    return;
  }

  const mediaX = e.clientX - mediaRect.left;
  const mediaY = e.clientY - mediaRect.top;
  const withinMediaY = mediaY >= 0 && mediaY <= mediaRect.height;
  isHoverLeft.value = withinMediaY && mediaX >= 0 && mediaX < mediaRect.width * 0.1;
  isHoverRight.value = withinMediaY && mediaX <= mediaRect.width && mediaX > mediaRect.width * 0.9;
}

function handleMouseLeave() {
  isHoverLeft.value = false;
  isHoverRight.value = false;
  isHoverTop.value = false;
  isHoverBottom.value = false;
}

function handleContextMenu(e: MouseEvent) {
  if (contextMenuRef.value) {
    contextMenuRef.value.open(e.clientX, e.clientY);
  }
}

const computedToolbarClass = computed(() => {
  const commonClasses = 'absolute z-80 h-10 flex flex-row items-center justify-center select-none';

  if (props.isFullScreen && props.mode === 2) {
    const floatingClasses = 'left-1/2 top-4 -translate-x-1/2 px-2 rounded-box bg-base-100/30 hover:bg-base-100/70 transition-[opacity,transform] duration-300 ease-in-out';
    return `${commonClasses} ${floatingClasses} ${(props.forceToolbarVisible || isHoverTop.value || hasOpenMenu.value) ? 'opacity-100' : 'opacity-0'}`;
  }

  const isPinned = props.mode === 2 ? true : config.mediaViewer.isPinned;

  if (isPinned) {
    // Fixed Top Bar
    return `${commonClasses} relative top-0 left-0 w-full`;
  } else {
    // Floating Hover Bar
    const floatingClasses = 'left-1/2 -translate-x-1/2 px-2 rounded-box bg-base-100/30 hover:bg-base-100/70 transition-[opacity,transform] duration-300 ease-in-out';
    
    if (toolbarPosition.value === 'bottom') {
       if (isHoverBottom.value || hasOpenMenu.value) {
          if (props.file.file_type === 2) {
            return `${commonClasses} ${floatingClasses} bottom-8 opacity-100`;
          } else {
            return `${commonClasses} ${floatingClasses} bottom-4 opacity-100`;
          }
       } else {
          if (props.file.file_type === 2) {
            return `${commonClasses} ${floatingClasses} bottom-8 opacity-0`;
          } else {
            return `${commonClasses} ${floatingClasses} bottom-4 opacity-0`;
          }
       }
    } else {
       if (isHoverTop.value || hasOpenMenu.value) {
          return `${commonClasses} ${floatingClasses} top-4 opacity-100`;
       } else {
          return `${commonClasses} ${floatingClasses} top-4 opacity-0`;
       }
    }
  }
});

const handleMenuOpenChange = (isOpen: boolean) => {
  hasOpenMenu.value = isOpen;
};

// Expose methods for parent component (ImageViewer)
const zoomIn = () => mediaRef.value?.zoomIn();
const zoomOut = () => mediaRef.value?.zoomOut();
const zoomActual = () => mediaRef.value?.zoomActual();
const rotateRight = () => mediaRef.value?.rotateRight();
const togglePlay = () => mediaRef.value?.togglePlay?.();
const getViewportState = () => mediaRef.value?.getViewportState?.();
const applyViewportState = (viewport: any, silent = false) => mediaRef.value?.applyViewportState?.(viewport, silent);
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

const handleZoomIn = () => {
  if (props.toolbarOnly || !mediaRef.value) {
    emit('item-action', { action: 'zoom-in', index: props.fileIndex });
    return;
  }
  zoomIn();
};

const handleZoomOut = () => {
  if (props.toolbarOnly || !mediaRef.value) {
    emit('item-action', { action: 'zoom-out', index: props.fileIndex });
    return;
  }
  zoomOut();
};

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
  togglePlay,
  getViewportState,
  applyViewportState,
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

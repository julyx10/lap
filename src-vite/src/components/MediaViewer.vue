<template>
  <Teleport to="body" :disabled="!previewFullScreen">
  <div 
    :class="['w-full flex flex-col items-center justify-center', previewFullScreen ? 'fixed inset-0 z-[200]' : 'relative', isFullScreen ? 'bg-base-200' : '', toolbarOnly ? '' : 'h-full group']"
    :style="toolbarOnly ? undefined : viewBackgroundStyle"
    @mousemove="handleMouseMove"
    @mouseover="handleRawControlsHover"
    @mouseleave="handleMouseLeave"
    @contextmenu.prevent
    ref="containerRef"
  >
    <!-- Toolbar -->
    <div 
      v-if="showToolbar"
      id="responsiveDiv"
      :class="computedToolbarClass"
      data-tauri-drag-region
    >
      <!-- App Icon + Title (left side, ImageViewer on Windows) -->
      <div v-if="showDesktopWindowControls && mode === 2 && showWindowControlsBar" class="absolute left-0 top-0 h-10 flex items-center px-3 select-none" data-tauri-drag-region>
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
                    : 'text-base-content/30 hover:bg-base-100/30 hover:text-base-content/70'
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
          :shortcut="shortcut('view.zoomOut')"
          @click="handleZoomOut"
        />
        <TButton
          :icon="IconZoomIn"
          :disabled="fileIndex < 0 || imageScale >= imageMaxScale || isSlideShow || !canInteract"
          :tooltip="$t('image_viewer.toolbar.zoom_in')"
          :shortcut="shortcut('view.zoomIn')"
          @click="handleZoomIn" 
        />
        <TButton
          :icon="!isZoomFit ? IconZoomFit : IconZoomActual"
          :disabled="fileIndex < 0 || isSlideShow || !canInteract"
          :tooltip="!isZoomFit ? $t('image_viewer.toolbar.zoom_fit') : $t('image_viewer.toolbar.zoom_actual')"
          :shortcut="shortcut('view.zoomFit')"
          @click="$emit('update:isZoomFit', !isZoomFit)"
        />
        <TButton
          :icon="IconFocus"
          :selected="showAfAreas"
          :disabled="fileIndex < 0 || !canInteract || ![1, 3].includes(Number(file?.file_type))"
          :tooltip="$t('image_viewer.toolbar.af_area')"
          @click="showAfAreas = !showAfAreas"
        />
        <template v-if="showExtraIcons">
          <IconSeparator class="t-icon-size-sm text-base-content/30" />
          <TButton
            :icon="file?.is_favorite ? IconHeartFilled : IconHeart"
            :disabled="fileIndex < 0 || isSlideShow || !canInteract"
            :selected="file?.is_favorite && !isSlideShow"
            :tooltip="file?.is_favorite ? $t('menu.meta.unfavorite') : $t('menu.meta.favorite')"
            :shortcut="shortcut('meta.favorite')"
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
                :tooltip="$t('rating.title')"
                :shortcut="ratingShortcutLabel"
                @click.stop="toggle"
              />
            </template>
          </ContextMenu>
          <ContextMenu
            :menuItems="cullingMenuItems"
            :disabled="fileIndex < 0 || isSlideShow || !canInteract"
            @open-change="handleMenuOpenChange"
            @click.stop
          >
            <template #trigger="{ toggle }">
              <TButton
                :icon="Number(file?.culling_flag ?? file?.cullingFlag ?? 0) === 1 ? IconFlagFilled : Number(file?.culling_flag ?? file?.cullingFlag ?? 0) === 2 ? IconFlagOff : IconFlag"
                :disabled="fileIndex < 0 || isSlideShow || !canInteract"
                :selected="Number(file?.culling_flag ?? file?.cullingFlag ?? 0) > 0 && !isSlideShow"
                :tooltip="$t('culling.title')"
                @click.stop="toggle"
              />
            </template>
          </ContextMenu>
          <TButton
            :icon="IconTag"
            :disabled="fileIndex < 0 || isSlideShow || !canInteract"
            :selected="file?.has_tags && !isSlideShow"
            :tooltip="$t('menu.meta.tag')"
            :shortcut="shortcut('meta.tag')"
            @click="$emit('item-action', { action: 'tag', index: fileIndex })"
          />
          <TButton
            :icon="IconStack"
            :disabled="fileIndex < 0 || isSlideShow || !canInteract"
            :selected="file?.has_collections && !isSlideShow"
            :tooltip="$t('menu.meta.collection')"
            :shortcut="shortcut('meta.collection')"
            @click="$emit('item-action', { action: 'add-to-collection', index: fileIndex })"
          />
          <TButton
            :icon="IconComment"
            :disabled="fileIndex < 0 || isSlideShow || !canInteract"
            :selected="!!file?.comments && !isSlideShow"
            :tooltip="$t('menu.meta.comment')"
            :shortcut="shortcut('meta.comment')"
            @click="$emit('item-action', { action: 'comment', index: fileIndex })"
          />
          <TButton
            :icon="IconRotate"
            :disabled="fileIndex < 0 || isSlideShow || !canInteract"
            :iconStyle="{ transform: `rotate(${file?.rotate ?? 0}deg)`, transition: 'transform 0.3s' }"
            :selected="(file?.rotate ?? 0) % 360 !== 0 && !isSlideShow"
            :tooltip="$t('menu.meta.rotate')"
            :shortcut="shortcut('meta.rotate')"
            @click="$emit('item-action', { action: 'rotate', index: fileIndex })"
          />
          <!-- <TButton
            v-if="mode !== 2"
            :icon="IconFileInfo"
            :disabled="fileIndex < 0 || isSlideShow || !canInteract"
            :tooltip="$t('menu.meta.info')"
            :shortcut="shortcut('meta.info')"
            @click="$emit('item-action', { action: 'info', index: fileIndex })"
          /> -->
        </template>
        <!-- Linked viewport control (Compare mode only) -->
        <template v-if="mode === 2 && showSyncViewportControl">
          <IconSeparator class="t-icon-size-sm text-base-content/30" />
          <TButton
            :icon="IconLink"
            :selected="isSyncViewport"
            :tooltip="isSyncViewport ? $t('image_viewer.toolbar.sync_viewport_off') : $t('image_viewer.toolbar.sync_viewport_on')"
            @click="$emit('item-action', { action: 'toggle-sync-viewport' })"
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
          :icon="!isFullScreen ? IconFullScreen : IconRestoreScreen"
          :tooltip="!isFullScreen ? $t('image_viewer.toolbar.fullscreen') : $t('image_viewer.toolbar.exit_fullscreen')"
          :disabled="!canInteract"
          @click="toggleFullScreen"
        />
        <TButton v-if="mode !== 2 && !isFullScreen"
          :icon="config.mediaViewer.isPinned ? IconPin : IconUnPin"
          :disabled="fileIndex < 0 || !canInteract"
          :tooltip="!config.mediaViewer.isPinned ? $t('image_viewer.toolbar.pin') : $t('image_viewer.toolbar.unpin')"
          @click="toggleToolbarPin"
        />
        <TButton
          v-if="isFullScreen || (mode === 0 && config.mediaViewer.isPinned)"
          :icon="IconClose"
          :tooltip="$t('image_viewer.toolbar.close')"
          :disabled="!canInteract"
          @click.stop="$emit('close')"
        />
      </div>
    </div>

    <!-- Window Control Buttons (top-right) -->
    <div v-if="showWindowControlsBar && showWindowControls && showDesktopWindowControls" class="absolute top-0 right-0 z-90 flex items-center" @mousedown.stop>
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
        @click.stop="$emit('close')" 
      />
    </div>

    <!-- Elements below only rendered when not toolbar-only -->
    <template v-if="!toolbarOnly">
    <!-- Close Button (Top Right) -->
    <button 
      v-if="mode === 0 && !config.mediaViewer.isPinned && !isFullScreen"
      class="preview-press-button absolute right-2 top-2 z-90 p-2 rounded-full text-base-content/70 bg-base-100/30 hover:text-base-content hover:bg-base-100/70 cursor-pointer"
      @click.stop="$emit('close')"
      @dblclick.stop
    >
      <span class="preview-press-icon flex pointer-events-none"><IconClose class="w-5 h-5" /></span>
    </button>

    <div
      ref="mediaAreaRef"
      class="flex-1 w-full min-h-0 relative"
      @dblclick="$emit('media-dblclick')"
      @contextmenu.prevent="handleBackgroundContextMenu"
    >
      <Transition name="fade">
      <div
        v-if="showStatusBadges && (quickViewStatusBadges.length > 0 || statusFeedbackVisible)"
        class="pointer-events-none absolute left-4 top-4 z-30 flex max-w-[calc(100%-2.5rem)] flex-wrap gap-1"
      >
        <div v-if="unfavoriteFeedback && !file?.is_favorite" class="thumb-badge"
          role="status" :aria-label="$t('menu.meta.unfavorite')">
          <IconHeart class="h-3.5 w-3.5" />
        </div>
        <div v-else-if="statusFeedbackVisible && quickViewStatusBadges.length === 0" class="thumb-badge">—</div>
        <div
          v-for="badge in quickViewStatusBadges"
          :key="badge.key"
          class="thumb-badge"
        >
          <template v-if="badge.icons?.length">
            <div class="flex items-center gap-0.5">
              <component
                :is="entry.icon"
                v-for="(entry, index) in badge.icons"
                :key="`${badge.key}-${index}`"
                class="h-3.5 w-3.5 shrink-0"
                :style="entry.style"
              />
            </div>
          </template>
          <component
            v-else-if="badge.icon"
            :is="badge.icon"
            :class="['h-3.5 w-3.5 shrink-0', badge.iconClass]"
            :style="badge.iconStyle"
          />
          <span v-if="badge.label" class="leading-none">{{ badge.label }}</span>
          <component
            v-if="badge.trailingIcon"
            :is="badge.trailingIcon"
            :class="['h-3.5 w-3.5 shrink-0', badge.trailingIconClass]"
          />
        </div>
      </div>
      </Transition>

      <!-- Previous Button (Overlay, media-area anchored) -->
      <button
        v-if="!isSlideShow && (showOverlayNav || previewFullScreen)"
        class="preview-press-button absolute left-2 top-1/2 -translate-y-1/2 z-70 p-2 rounded-full bg-base-100/30 backdrop-blur-md transition-opacity duration-200"
        :class="[
          isHoverLeft ? (hasPrevious ? 'opacity-100 pointer-events-auto hover:text-base-content hover:bg-base-100/80 cursor-pointer' : 'opacity-30 cursor-default') : 'opacity-0 pointer-events-none'
        ]"
        :disabled="!hasPrevious"
        @click.stop="triggerPrev"
        @dblclick.stop
      >
        <span class="preview-press-icon flex pointer-events-none"><IconLeft class="w-8 h-8" /></span>
      </button>

      <!-- Next Button (Overlay, media-area anchored) -->
      <button
        v-if="!isSlideShow && (showOverlayNav || previewFullScreen)"
        class="preview-press-button absolute right-2 top-1/2 -translate-y-1/2 z-70 p-2 rounded-full bg-base-100/30 backdrop-blur-md transition-opacity duration-200"
        :class="[
          isHoverRight ? (hasNext ? 'opacity-100 pointer-events-auto hover:text-base-content hover:bg-base-100/80 cursor-pointer' : 'opacity-30 cursor-default') : 'opacity-0 pointer-events-none'
        ]"
        :disabled="!hasNext"
        @click.stop="triggerNext"
        @dblclick.stop
      >
        <span class="preview-press-icon flex pointer-events-none"><IconRight class="w-8 h-8" /></span>
      </button>

      <div
        v-if="isLivePhotoLike"
        :class="[
          'absolute inset-0 z-10 transition-opacity duration-150',
          isLivePhotoPlaying ? 'opacity-100' : 'pointer-events-none opacity-0',
        ]"
      >
        <Video
          class="h-full w-full"
          :filePath="livePhotoVideoPath"
          :rotate="file?.rotate ?? 0"
          :isZoomFit="isZoomFit"
          :isSlideShow="isSlideShow"
          :isActive="isPlaybackActive && isLivePhotoPlaying"
          :playOnActivate="true"
          :viewportState="livePhotoViewport"
          :showControls="false"
          :showPlayOverlay="false"
          @message-from-video-viewer="handleMessageFromImageViewer"
          @slideshow-next="emit('slideshow-next')"
          @context-menu="handleContextMenu"
          @pointerdown.capture="handleOverlayPointerDown"
          @pointerup.capture="handleOverlayPointerUp"
          @pointercancel.capture="resetOverlayPointer"
        ></Video>
      </div>

      <div
        v-if="file?.file_type === 1 || file?.file_type === 3"
        :class="[
          isLivePhotoLike
            ? 'absolute inset-0 z-20 transition-opacity duration-150'
            : 'contents',
          isLivePhotoLike && (isLivePhotoPlaying ? 'pointer-events-none opacity-0' : 'opacity-100'),
        ]"
      >
        <Image
          ref="mediaRef"
          :showAfAreas="showAfAreas"
          :filePath="file?.file_path"
          :fileId="file?.id"
          :fileType="file?.file_type"
          :originalUnavailable="originalUnavailable"
          :rawPairPath="file?.media_subtype === 'raw_jpeg_pair' ? file?.live_photo_video_path : ''"
          :fileVersion="file?.modified_at || 0"
          :imageWidth="file?.width"
          :imageHeight="file?.height"
          :thumbnailSrc="file?.thumbnail || ''"
          :showThumbnailPlaceholder="showThumbnailPlaceholder"
          :showInlineLoading="mode === 2"
          :suppressAutoNavigator="suppressAutoNavigator && !previewFullScreen"
          :nextFilePath="nextFilePath"
          :rotate="file?.rotate ?? 0"
          :isZoomFit="isZoomFit"
          :isSlideShow="isSlideShow"
          :slideShowTransitionMode="slideShowTransitionMode"
          @update:isZoomFit="(val: boolean) => $emit('update:isZoomFit', val)"
          @scale="(e) => $emit('scale', e)"
          @viewport-change="(e) => $emit('viewport-change', e)"
          @message-from-image-viewer="handleMessageFromImageViewer"
          @context-menu="handleContextMenu"
          @pointerdown.capture="handleOverlayPointerDown"
          @pointerup.capture="handleOverlayPointerUp"
          @pointercancel.capture="resetOverlayPointer"
        ></Image>
      </div>

      <button
        v-if="isLivePhotoLike"
        class="absolute left-4 bottom-4 z-60 inline-flex h-10 items-center gap-2 rounded-box bg-base-100/70 px-3 text-sm font-medium text-base-content/70 shadow"
        :class="originalUnavailable ? 'cursor-default opacity-50' : 'hover:bg-base-100 hover:text-base-content cursor-pointer'"
        :disabled="originalUnavailable"
        @mouseenter="startLivePhotoPreview"
        @mouseleave="isLivePhotoPlaying = false"
        @click.stop
        @dblclick.stop
      >
        <IconLivePhoto class="h-4 w-4" />
        <span class="text-xs font-semibold uppercase tracking-wider leading-none">{{ isLivePhoto ? t('image_viewer.live') : t('image_viewer.motion') }}</span>
      </button>

      <div
        v-if="file?.file_type === 2"
        class="absolute inset-0 z-20"
      >
        <Video
          ref="mediaRef"
          class="h-full w-full"
          :filePath="file?.file_path"
          :fileId="file?.id"
          :thumbnailSrc="file?.thumbnail || ''"
          :originalUnavailable="originalUnavailable"
          :rotate="file?.rotate ?? 0"
          :isZoomFit="isZoomFit"
          :isSlideShow="isSlideShow"
          :isActive="isPlaybackActive"
          @scale="(e) => $emit('scale', e)"
          @viewport-change="(e) => $emit('viewport-change', e)"
          @message-from-video-viewer="handleMessageFromImageViewer"
          @slideshow-next="emit('slideshow-next')"
          @context-menu="handleContextMenu"
          @pointerdown.capture="handleOverlayPointerDown"
          @pointerup.capture="handleOverlayPointerUp"
          @pointercancel.capture="resetOverlayPointer"
        ></Video>
      </div>
    </div>

    </template>

    <ContextMenu
      ref="backgroundContextMenuRef"
      class="absolute h-0 w-0 overflow-hidden pointer-events-none"
      :menuItems="viewBackgroundMenuItems"
      @open-change="handleMenuOpenChange"
    >
      <template #trigger><span /></template>
    </ContextMenu>
  </div>
  </Teleport>
</template>

<script setup lang="ts">
import { OVERLAY_CLICK_MOVE_TOLERANCE } from '@/common/constants';
import { defineAsyncComponent, ref, computed, watch, onMounted, onUpdated, onBeforeUnmount, type Component, type CSSProperties } from 'vue';
import { useI18n } from 'vue-i18n';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { runPreviewWindowOperation, recoverPreviewWindowFocus } from '@/common/previewWindow';
import { config, libConfig } from '@/common/config';
import { useToast } from '@/common/toast';
import { isWin, isMac, isLinux, getSlideShowInterval } from '@/common/utils';
import { getShortcutLabel, ShortcutActionId, ShortcutPlatform, VIEW_BACKGROUND_SHORTCUTS } from '@/common/shortcuts';
import { isOriginalUnavailable, setFileAccessibility } from '@/common/availability';
import { getMotionPhotoVideoPath, checkFileAccessibility } from '@/common/api';

import Image from '@/components/Image.vue';
import IconFocus from '@/assets/icons/focus.svg';
import { storeToRefs } from 'pinia';
import { useUIStore } from '@/stores/uiStore';
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
  IconFlag,
  IconFlagFilled,
  IconFlagOff,
  IconStar,
  IconStarFilled,
  IconStack,
  IconTag,
  IconComment,
  IconRotate,
  IconFileInfo,
  IconDot,
  IconWinMinus,
  IconWinMaximize,
  IconWinRestore,
  IconLink,
  IconPalette,
  IconVideoPlay,
  IconLivePhoto,
  IconExclamation,
} from '@/common/icons';
import ContextMenu from '@/components/ContextMenu.vue';
import iconLogo from '@/assets/images/icon.png';
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
  suppressAutoNavigator: { type: Boolean, default: false },
  showThumbnailPlaceholder: {
    type: Boolean,
    default: false,
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
  showSyncViewportControl: {
    type: Boolean,
    default: false,
  },
  isPlaybackActive: {
    type: Boolean,
    default: true
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

const isLivePhotoPlaying = ref(false);
const isLivePhoto = computed(() => props.file?.media_subtype === 'live_photo' && !!props.file?.live_photo_video_path);
const isMotionPhoto = computed(() => props.file?.media_subtype === 'motion_photo');
// Both Apple Live Photos and Android Motion Photos play a short video layered
// over the still image, so they share the same UI/playback path.
const isLivePhotoLike = computed(() => isLivePhoto.value || isMotionPhoto.value);
const motionPhotoVideoPath = ref<string | null>(null);
const livePhotoVideoPath = computed(() =>
  isLivePhoto.value ? props.file?.live_photo_video_path : motionPhotoVideoPath.value,
);
const livePhotoViewport = ref<Record<string, number | boolean> | null>(null);
let motionPhotoVideoRequestSeq = 0;

function startLivePhotoPreview() {
  if (originalUnavailable.value || !livePhotoVideoPath.value) return;
  emit('activate');
  const viewport = mediaRef.value?.getViewportState?.();
  // Preserve the still image's visible region for the Live Photo preview.
  // Video.applyViewportState() remaps that normalized viewport to the video's
  // own source dimensions, so mismatched image/video sizes still line up.
  livePhotoViewport.value = viewport
    ? { ...viewport, isZoomFit: false }
    : null;
  isLivePhotoPlaying.value = true;
}

watch(
  () => [props.file?.id, props.file?.media_subtype, props.file?.modified_at],
  async ([fileId, mediaSubtype]) => {
    const requestSeq = ++motionPhotoVideoRequestSeq;
    isLivePhotoPlaying.value = false;
    livePhotoViewport.value = null;
    motionPhotoVideoPath.value = null;
    // For Motion Photos the embedded MP4 has to be extracted (cached) before
    // the generic video pipeline can play it.
    if (mediaSubtype === 'motion_photo' && fileId != null) {
      try {
        const path = await getMotionPhotoVideoPath(fileId);
        if (requestSeq === motionPhotoVideoRequestSeq) {
          motionPhotoVideoPath.value = path;
        }
      } catch (error) {
        if (requestSeq === motionPhotoVideoRequestSeq) {
          console.error('Failed to prepare motion photo video:', error);
        }
      }
    }
  },
  { immediate: true },
);

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
  'viewport-change',
  'view-background-change',
  'activate',
]);

const { locale, messages, t } = useI18n();
const localeMsg = computed(() => messages.value[locale.value] as any);

const contextMenuRef = ref<any>(null);
const backgroundContextMenuRef = ref<any>(null);
const containerRef = ref<HTMLElement | null>(null);
const mediaAreaRef = ref<HTMLElement | null>(null);
const mediaRef = ref<any>(null);
const toast = useToast();
const { showAfAreas } = storeToRefs(useUIStore());
const isHoverLeft = ref(false);
const isHoverRight = ref(false);
const isHoverTop = ref(false);
const isHoverBottom = ref(false);
const toolbarPosition = ref<'top' | 'bottom'>('top');
const hasOpenMenu = ref(false);

// Responsive toolbar
const containerWidth = ref(0);
const buttonsRef = ref<HTMLElement | null>(null);
const buttonsWidth = ref(0);
const rawControlsWidth = ref(0);
const rawControlsHovered = ref(false);
let observedRawControls: HTMLElement | null = null;
const rawToolbarNeedsTop = computed(() => rawControlsWidth.value > 0
  && containerWidth.value < buttonsWidth.value + 2 * rawControlsWidth.value + 48);

function measureRawControls() {
  const controls = containerRef.value?.querySelector<HTMLElement>('[data-raw-preview-controls]') || null;
  if (controls !== observedRawControls) {
    if (observedRawControls) resizeObserver?.unobserve(observedRawControls);
    observedRawControls = controls;
    if (controls) resizeObserver?.observe(controls);
  }
  rawControlsWidth.value = controls?.offsetWidth || 0;
  if (!rawControlsWidth.value) rawControlsHovered.value = false;
}

onUpdated(measureRawControls);

function handleRawControlsHover(e: MouseEvent) {
  rawControlsHovered.value = !!(e.target as Element)?.closest('[data-raw-preview-controls]');
  if (rawControlsHovered.value) handleMouseLeave();
}
const filenameMaxWidth = computed(() => {
  if (containerWidth.value > 0 && buttonsWidth.value > 0) {
    const val = (containerWidth.value / 2) - (buttonsWidth.value / 2) - 100;
    return Math.max(0, val);
  }
  return 200; // Fallback
});
const showExtraIcons = computed(() => containerWidth.value > 600);
// Window control state (Windows + ImageViewer mode)
const showDesktopWindowControls = isWin || isLinux;
// Preview owns its fullscreen session; the standalone viewer delegates to its window.
const previewFullScreen = ref(false);
const isFullScreen = computed(() => props.isFullScreen || previewFullScreen.value);
let fullscreenBusy = false;
let previewDisposed = false;
let windowWasFullScreen = false;
let windowWasMaximized = false;
let unlistenPreviewResize: (() => void) | undefined;
function stopPreviewResizeListener() {
  unlistenPreviewResize?.();
  unlistenPreviewResize = undefined;
}

async function restorePreviewWindow() {
  const appWindow = getCurrentWindow();
  if (!windowWasFullScreen) {
    if (await appWindow.isFullscreen()) await appWindow.setFullscreen(false);
    if (isWin && windowWasMaximized) {
      await new Promise(resolve => setTimeout(resolve, 80));
      await appWindow.maximize();
    }
  }
  await recoverPreviewWindowFocus();
}

async function exitPreviewFullScreen() {
  return runPreviewWindowOperation(async (cancelFocusRecovery) => {
    if (!previewFullScreen.value) return;
    cancelFocusRecovery();
    fullscreenBusy = true;
    try {
      await restorePreviewWindow();
      previewFullScreen.value = false;
      // Closing a preview restores the window without forgetting its viewing mode.
      if (!previewDisposed) config.mediaViewer.isFullScreen = false;
      stopPreviewResizeListener();
    } catch (error) {
      console.error('Failed to exit preview fullscreen', error);
    } finally {
      fullscreenBusy = false;
    }
  });
}

async function toggleFullScreen() {
  if (props.mode === 2) {
    emit('toggle-full-screen');
    return;
  }
  if (fullscreenBusy) return;
  if (previewFullScreen.value) return exitPreviewFullScreen();
  fullscreenBusy = true;
  return runPreviewWindowOperation(async (cancelFocusRecovery) => {
    if (previewDisposed) { fullscreenBusy = false; return; }
    cancelFocusRecovery();
    stopPreviewResizeListener();
    const appWindow = getCurrentWindow();
    let capturedWindowState = false;
    try {
      windowWasFullScreen = await appWindow.isFullscreen();
      windowWasMaximized = await appWindow.isMaximized();
      capturedWindowState = true;
      if (!windowWasFullScreen) {
        if (isWin && windowWasMaximized) {
          await appWindow.unmaximize();
          await new Promise(resolve => setTimeout(resolve, 80));
        }
        await appWindow.setFullscreen(true);
      }
      if (previewDisposed) {
        await restorePreviewWindow();
        return;
      }
      previewFullScreen.value = true;
      unlistenPreviewResize = await appWindow.onResized(async () => {
        if (!fullscreenBusy && previewFullScreen.value && !(await appWindow.isFullscreen())) {
          void exitPreviewFullScreen();
        }
      });
      if (previewDisposed) {
        unlistenPreviewResize?.();
        await restorePreviewWindow();
      } else {
        config.mediaViewer.isFullScreen = true;
      }
    } catch (error) {
      console.error('Failed to enter preview fullscreen', error);
      unlistenPreviewResize?.();
      if (capturedWindowState) {
        await restorePreviewWindow().catch(error => console.error('Failed to restore preview window', error));
      }
      previewFullScreen.value = false;
    } finally {
      fullscreenBusy = false;
    }
  });
}

const desktopAppWindow = showDesktopWindowControls ? getCurrentWindow() : null;
const isMaximized = ref(false);

const minimizeWindow = () => desktopAppWindow?.minimize();
const toggleMaximizeWindow = () => {
  desktopAppWindow?.isMaximized().then((maximized) => {
    if (maximized) {
      isMaximized.value = false;
      desktopAppWindow?.unmaximize();
    } else {
      isMaximized.value = true;
      desktopAppWindow?.maximize();
    }
  });
};
const closeWindow = () => desktopAppWindow?.close();
const shortcutPlatform: ShortcutPlatform = isMac ? 'mac' : (isLinux ? 'linux' : 'windows');
const shortcut = (actionId: ShortcutActionId) => getShortcutLabel(actionId, shortcutPlatform);
const ratingShortcutLabel = computed(() => {
  const first = shortcut('meta.rating.clear');
  const last = shortcut('meta.rating.five');
  return first && last ? `${first}-${last}` : '';
});
const effectiveSlideShowIntervalIndex = computed(() => {
  return props.slideShowIntervalIndex ?? config.settings.slideShowInterval;
});
const currentSlideShowIntervalLabel = computed(() => `${getSlideShowInterval(effectiveSlideShowIntervalIndex.value)}s`);
const slideShowTransitionMode = computed(() => Number(config.settings.slideShowTransition ?? 0));
const viewBackgroundColors = ['transparent', '#000000', '#333333', '#808080', '#d3d3d3', '#ffffff'];
const viewBackgroundSwatches: CSSProperties[] = [
  { background: 'linear-gradient(135deg, var(--color-base-100) 0 50%, var(--color-base-300) 50% 100%)' },
  ...viewBackgroundColors.slice(1).map(backgroundColor => ({ backgroundColor })),
];
const viewBackgroundStyle = computed(() => {
  const backgroundColor = viewBackgroundColors[Number(config.settings.viewBackground ?? 0)] ?? viewBackgroundColors[0];
  return isFullScreen.value && backgroundColor === 'transparent'
    ? undefined
    : { backgroundColor };
});
const viewBackgroundMenuItems = computed(() => {
  const labels = localeMsg.value.settings.image_view.view_background_options || [];
  const selectedBackground = Number(config.settings.viewBackground ?? 0);
  return [
    {
      label: localeMsg.value.settings.shortcuts.actions.cycle_background,
      icon: IconPalette,
      shortcut: shortcut('view.cycleBackground'),
      action: cycleViewBackground,
    },
    { label: '-', action: null },
    ...VIEW_BACKGROUND_SHORTCUTS.map(({ actionId, value }) => ({
    label: labels[value],
    swatch: viewBackgroundSwatches[value],
    selected: selectedBackground === value,
    shortcut: shortcut(actionId),
    action: () => emit('view-background-change', value),
    })),
  ];
});
// const ratingButtonTooltip = computed(() => {
//   const rating = Number(props.file?.rating || 0);
//   return rating > 0 ? `${localeMsg.value.rating.title}: ${rating}` : localeMsg.value.rating.title;
// });
const ratingMenuItems = computed(() => {
  const rating = Number(props.file?.rating || 0);
  return [
    {
      label: localeMsg.value.rating.clear_rating,
      icon: IconStar,
      shortcut: shortcut('meta.rating.clear'),
      action: () => emit('item-action', { action: 'rating-0', index: props.fileIndex }),
    },
    { label: '-', action: null },
    {
      label: localeMsg.value.rating.five_stars,
      icon: rating === 5 ? IconStarFilled : IconStar,
      shortcut: shortcut('meta.rating.five'),
      action: () => emit('item-action', { action: 'rating-5', index: props.fileIndex }),
    },
    {
      label: localeMsg.value.rating.four_stars,
      icon: rating === 4 ? IconStarFilled : IconStar,
      shortcut: shortcut('meta.rating.four'),
      action: () => emit('item-action', { action: 'rating-4', index: props.fileIndex }),
    },
    {
      label: localeMsg.value.rating.three_stars,
      icon: rating === 3 ? IconStarFilled : IconStar,
      shortcut: shortcut('meta.rating.three'),
      action: () => emit('item-action', { action: 'rating-3', index: props.fileIndex }),
    },
    {
      label: localeMsg.value.rating.two_stars,
      icon: rating === 2 ? IconStarFilled : IconStar,
      shortcut: shortcut('meta.rating.two'),
      action: () => emit('item-action', { action: 'rating-2', index: props.fileIndex }),
    },
    {
      label: localeMsg.value.rating.one_star,
      icon: rating === 1 ? IconStarFilled : IconStar,
      shortcut: shortcut('meta.rating.one'),
      action: () => emit('item-action', { action: 'rating-1', index: props.fileIndex }),
    },
  ];
});

const cullingMenuItems = computed(() => [
  {
    label: localeMsg.value.culling.picks,
    icon: Number(props.file?.culling_flag ?? props.file?.cullingFlag ?? 0) === 1 ? IconFlagFilled : IconFlag,
    shortcut: shortcut('meta.culling.pick'),
    action: () => emit('item-action', { action: 'culling-pick', index: props.fileIndex }),
  },
  {
    label: localeMsg.value.culling.rejected,
    icon: Number(props.file?.culling_flag ?? props.file?.cullingFlag ?? 0) === 2 ? IconFlagFilled : IconFlagOff,
    shortcut: shortcut('meta.culling.reject'),
    action: () => emit('item-action', { action: 'culling-reject', index: props.fileIndex }),
  },
  {
    label: localeMsg.value.culling.unreviewed,
    icon: Number(props.file?.culling_flag ?? props.file?.cullingFlag ?? 0) === 0 ? IconFlagFilled : IconFlag,
    shortcut: shortcut('meta.culling.unreviewed'),
    action: () => emit('item-action', { action: 'culling-unreviewed', index: props.fileIndex }),
  },
]);

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

type StatusBadge = {
  key: string;
  icon?: Component;
  icons?: Array<{
    icon: Component;
    style?: CSSProperties;
  }>;
  label?: string;
  iconClass?: string;
  iconStyle?: CSSProperties;
  trailingIcon?: Component;
  trailingIconClass?: string;
};

const normalizedFileRotate = computed(() => {
  const rotate = Number(props.file?.rotate || 0) % 360;
  return rotate < 0 ? rotate + 360 : rotate;
});

const quickViewStatusBadges = computed<StatusBadge[]>(() => {
  const badges: StatusBadge[] = [];
  const metaIcons: StatusBadge['icons'] = [];
  const rating = Number(props.file?.rating || 0);
  const cullingFlag = Number(props.file?.culling_flag ?? props.file?.cullingFlag ?? 0);
  const cullingIcon = cullingFlag === 1
    ? IconFlagFilled
    : cullingFlag === 2
      ? IconFlagOff
      : undefined;
  const cullingIconClass = cullingFlag === 1
    ? 'text-primary'
    : cullingFlag === 2
      ? 'text-error'
      : undefined;

  if (props.file?.is_favorite) {
    badges.push({
      key: 'favorite',
      icon: IconHeartFilled,
      iconClass: 'text-error',
      label: rating > 0 ? `${rating}` : undefined,
      trailingIcon: cullingIcon,
      trailingIconClass: cullingIconClass,
    });
  } else if (rating > 0) {
    badges.push({
      key: 'rating',
      icon: IconStarFilled,
      iconClass: 'text-warning',
      label: `${rating}`,
      trailingIcon: cullingIcon,
      trailingIconClass: cullingIconClass,
    });
  } else if (cullingIcon) {
    badges.push({
      key: cullingFlag === 1 ? 'culling-pick' : 'culling-reject',
      icon: cullingIcon,
      iconClass: cullingIconClass,
    });
  }
  if (props.file?.has_tags) metaIcons.push({ icon: IconTag });
  if (props.file?.has_collections) metaIcons.push({ icon: IconStack });
  if (props.file?.comments?.length > 0) metaIcons.push({ icon: IconComment });
  if (normalizedFileRotate.value > 0) {
    metaIcons.push({
      icon: IconRotate,
      style: { transform: `rotate(${normalizedFileRotate.value}deg)` },
    });
  }
  if (metaIcons.length > 0) {
    badges.push({
      key: 'meta',
      icons: metaIcons,
    });
  }
  if (isOriginalUnavailable(props.file)) {
    badges.push({ key: 'unavailable', icon: IconExclamation, iconClass: 'text-warning/70' });
  }

  return badges;
});

const statusFeedbackVisible = ref(false);
const unfavoriteFeedback = ref(false);
let statusFeedbackTimer: ReturnType<typeof setTimeout> | undefined;
function sameStatusBadges(badges: StatusBadge[], previous: StatusBadge[]) {
  return badges.length === previous.length && badges.every((badge, index) => {
    const other = previous[index];
    return badge.key === other.key && badge.icon === other.icon && badge.label === other.label
      && badge.iconClass === other.iconClass && badge.trailingIcon === other.trailingIcon
      && badge.trailingIconClass === other.trailingIconClass
      && badge.icons?.length === other.icons?.length
      && (badge.icons || []).every((entry, i) => entry.icon === other.icons![i].icon
        && JSON.stringify(entry.style) === JSON.stringify(other.icons![i].style));
  });
}
function clearStatusFeedback() {
  clearTimeout(statusFeedbackTimer);
  statusFeedbackVisible.value = false;
  unfavoriteFeedback.value = false;
}
watch(
  [quickViewStatusBadges, () => props.file?.id, () => libConfig._libraryId, isFullScreen],
  ([badges, fileId, libraryId, fullscreen], [previousBadges, previousId, previousLibrary, wasFullscreen]) => {
    if (!fullscreen || !wasFullscreen || fileId !== previousId || libraryId !== previousLibrary) {
      clearStatusFeedback();
      return;
    }
    if (sameStatusBadges(badges, previousBadges)) return;
    clearStatusFeedback();
    statusFeedbackVisible.value = true;
    unfavoriteFeedback.value = previousBadges.some(badge => badge.key === 'favorite')
      && !badges.some(badge => badge.key === 'favorite');
    statusFeedbackTimer = setTimeout(clearStatusFeedback, 1200);
  },
);

const fullScreenToolbarVisible = computed(() =>
  props.forceToolbarVisible || (props.showToolbar && (isHoverTop.value || hasOpenMenu.value)),
);
const showStatusBadges = computed(() => isFullScreen.value
  ? fullScreenToolbarVisible.value || statusFeedbackVisible.value
  : props.mode === 0 || props.mode === 2,
);

const showWindowControlsBar = computed(() => {
  return props.showToolbar && !(showDesktopWindowControls && props.mode === 2 && props.isFullScreen);
});
let resizeObserver: ResizeObserver | null = null;

onMounted(() => {
  resizeObserver = new ResizeObserver((entries) => {
    measureRawControls();
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
  // Restore on opening Quick Preview. Merely mounting the embedded filmstrip
  // pane (including at app startup) must not take over the main window.
  if (props.mode === 0 && config.mediaViewer.isFullScreen) {
    void toggleFullScreen();
  }
});

onBeforeUnmount(() => {
  clearStatusFeedback();
  previewDisposed = true;
  stopPreviewResizeListener();
  void exitPreviewFullScreen();
  if (resizeObserver) {
    resizeObserver.disconnect();
  }
});

function toggleToolbarPin() {
  if (config.mediaViewer.isPinned) {
    toolbarPosition.value = config.mediaViewer.pinnedPosition === 'bottom' ? 'bottom' : 'top';
  } else {
    config.mediaViewer.pinnedPosition = toolbarPosition.value;
  }
  config.mediaViewer.isPinned = !config.mediaViewer.isPinned;
}

function handleMouseMove(e: MouseEvent) {
  handleRawControlsHover(e);
  if (rawControlsHovered.value) return;
  if (!containerRef.value) return;

  const containerRect = containerRef.value.getBoundingClientRect();
  if (containerRect.width <= 0 || containerRect.height <= 0) return;
  const containerY = e.clientY - containerRect.top;
  const containerHeight = containerRect.height;
  if (!config.mediaViewer.isPinned) {
    toolbarPosition.value = containerY < containerHeight * 0.5 ? 'top' : 'bottom';
  }

  if (!mediaAreaRef.value) {
    isHoverTop.value = containerY < 60;
    isHoverBottom.value = containerY > containerHeight - 60;
    return;
  }

  // Prev/next hover logic: based on actual media display area (accounts for panels/toolbar layout).
  const mediaRect = mediaAreaRef.value.getBoundingClientRect();
  if (mediaRect.width <= 0 || mediaRect.height <= 0) {
    isHoverLeft.value = false;
    isHoverRight.value = false;
    isHoverTop.value = containerY < 60;
    isHoverBottom.value = containerY > containerHeight - 60;
    return;
  }

  const mediaX = e.clientX - mediaRect.left;
  const mediaY = e.clientY - mediaRect.top;
  const withinMediaY = mediaY >= 0 && mediaY <= mediaRect.height;
  isHoverLeft.value = withinMediaY && mediaX >= 0 && mediaX < mediaRect.width * 0.1;
  isHoverRight.value = withinMediaY && mediaX <= mediaRect.width && mediaX > mediaRect.width * 0.9;
  const isHoveringNavigation = isHoverLeft.value || isHoverRight.value;
  isHoverTop.value = !isHoveringNavigation && containerY < 60;
  isHoverBottom.value = !isHoveringNavigation && containerY > containerHeight - 60;
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

function handleBackgroundContextMenu(e: MouseEvent) {
  const target = e.target as HTMLElement;
  if (target.closest('button')) return;
  backgroundContextMenuRef.value?.open(e.clientX, e.clientY);
}

function cycleViewBackground() {
  config.cycleViewBackground();
  emit('view-background-change', config.settings.viewBackground);
}

const computedToolbarClass = computed(() => {
  const commonClasses = 'absolute z-80 h-10 flex flex-row items-center justify-center select-none';

  if (isFullScreen.value) {
    const floatingClasses = 'left-1/2 top-4 -translate-x-1/2 px-2 rounded-box bg-base-100/30 hover:bg-base-100/70 transition-[opacity,transform] duration-300 ease-in-out';
    return `${commonClasses} ${floatingClasses} ${fullScreenToolbarVisible.value ? 'opacity-100' : 'opacity-0'}`;
  }

  const isPinned = props.mode === 2 ? true : config.mediaViewer.isPinned;

  if (isPinned) {
    // Keep the edge where the floating toolbar was pinned.
    if (props.mode !== 2 && config.mediaViewer.pinnedPosition === 'bottom') {
      return `${commonClasses} relative bottom-0 left-0 w-full order-last`;
    }
    return `${commonClasses} relative top-0 left-0 w-full`;
  } else {
    // Floating Hover Bar
    const floatingClasses = 'left-1/2 -translate-x-1/2 px-2 rounded-box bg-base-100/30 hover:bg-base-100/70 transition-[opacity,transform] duration-300 ease-in-out';
    
    if (toolbarPosition.value === 'bottom' && !rawToolbarNeedsTop.value) {
       if (rawControlsHovered.value && !hasOpenMenu.value) {
         return `${commonClasses} ${floatingClasses} bottom-4 invisible opacity-0 pointer-events-none`;
       }
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
       if (isHoverTop.value || (rawToolbarNeedsTop.value && isHoverBottom.value) || hasOpenMenu.value) {
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
const rotateView = (delta = 90) => mediaRef.value?.rotateView(delta);
const togglePlay = () => mediaRef.value?.togglePlay?.();
const getViewportState = () => mediaRef.value?.getViewportState?.();
const applyViewportState = (viewport: any, silent = false) => mediaRef.value?.applyViewportState?.(viewport, silent);
const getCurrentImageSrc = () => mediaRef.value?.getCurrentImageSrc?.() || '';
const clearPreloadCache = (filePath?: string) => mediaRef.value?.clearPreloadCache?.(filePath);
const showMessage = (message: string, isWarning: boolean = false) => {
  if (isWarning) {
    toast.warning(message, { placement: 'bottom-right' });
    return;
  }
  toast.info(message, { placement: 'bottom-right' });
};

const triggerPrev = () => {
  if (props.hasPrevious) {
    emit('prev');
  } else {
    showMessage((localeMsg.value as any).tooltip.image_viewer.first_image);
  }
}

const triggerNext = () => {
  if (props.hasNext) {
    emit('next');
  } else {
    showMessage((localeMsg.value as any).tooltip.image_viewer.last_image);
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

const handleOverlayClick = () =>{
  if(props.mode === 0){
    emit('close')
  }
}

let overlayPointer: { id: number; x: number; y: number } | null = null;

const resetOverlayPointer = () => {
  overlayPointer = null;
};

const isOverlayBackdropTarget = (event: PointerEvent) => {
  const target = event.target;
  const currentTarget = event.currentTarget;

  if (!(currentTarget instanceof HTMLElement)) return false;
  if (!(target instanceof HTMLElement)) return target === currentTarget;
  if (target === currentTarget) return true;

  if (target.closest(
    'img, video, button, a, input, select, textarea, label, [role="button"], [data-image-navigator], .vjs-tech, .vjs-control-bar, .vjs-control, .vjs-big-play-button, .vjs-menu'
  )) {
    return false;
  }

  return currentTarget.contains(target);
};

const handleOverlayPointerDown = (event: PointerEvent) => {
  if ((event.pointerType === 'mouse' || event.pointerType === 'pen') && event.button !== 0) {
    resetOverlayPointer();
    return;
  }
  const isBackdropTarget = isOverlayBackdropTarget(event);
  if (hasOpenMenu.value && isBackdropTarget) {
    resetOverlayPointer();
    return;
  }
  overlayPointer = isBackdropTarget
    ? { id: event.pointerId, x: event.clientX, y: event.clientY }
    : null;
};

const handleOverlayPointerUp = (event: PointerEvent) => {
  if ((event.pointerType === 'mouse' || event.pointerType === 'pen') && event.button !== 0) {
    resetOverlayPointer();
    return;
  }
  const pointer = overlayPointer;
  resetOverlayPointer();
  if (
    !pointer
    || pointer.id !== event.pointerId
    || !isOverlayBackdropTarget(event)
    || Math.abs(event.clientX - pointer.x) > OVERLAY_CLICK_MOVE_TOLERANCE
    || Math.abs(event.clientY - pointer.y) > OVERLAY_CLICK_MOVE_TOLERANCE
  ) {
    return;
  }
  handleOverlayClick();
};

const handleMessageFromImageViewer = (payload: { message: string }) => {
  if (payload.message === 'prev') {
    triggerPrev();
  } else if (payload.message === 'next') {
    triggerNext();
  } else if (payload.message === 'close') {
    handleOverlayClick();
  }
};

defineExpose({
  isFullScreen,
  exitPreviewFullScreen,
  zoomIn,
  zoomOut,
  zoomActual,
  rotateView,
  togglePlay,
  getViewportState,
  applyViewportState,
  navigateImage: (point: { normX: number; normY: number }) => mediaRef.value?.navigateImage?.(point),
  zoomNavigator: (factor: number) => mediaRef.value?.zoomNavigator?.(factor),
  toggleZoomFit: () => mediaRef.value?.toggleZoomFit?.(),
  getCurrentImageSrc,
  clearPreloadCache,
  showMessage,
  triggerPrev,
  triggerNext
});

const originalUnavailable = computed(() => isOriginalUnavailable(props.file));
watch(originalUnavailable, unavailable => {
  if (unavailable) isLivePhotoPlaying.value = false;
});
let accessRequest = 0;
async function refreshOriginalAccess() {
  const path = props.file?.file_path;
  const request = ++accessRequest;
  const libraryId = libConfig._libraryId;
  if (!path) return;
  try {
    const available = await checkFileAccessibility(path);
    if (request === accessRequest && libraryId === libConfig._libraryId) setFileAccessibility(path, available);
  } catch { /* An IPC failure is not evidence that the original is offline. */ }
}
watch(() => [props.file, props.file?.file_path, libConfig._libraryId], refreshOriginalAccess, { immediate: true });

const selectedFile = computed(() => props.file);

const singleFileMenuItems = computed(() => {
  if (props.mode === 2) return [];

  return useFileMenuItems(
    selectedFile,
    localeMsg,
    isMac,
    t,
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

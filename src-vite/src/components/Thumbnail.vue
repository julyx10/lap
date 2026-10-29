<template>
  <div
    :class="[
      'border-2 flex flex-col items-center cursor-pointer group',
      thumbnailCornerClass,
      isTransitionDisabled ? 'transition-none' : 'transition-[background-color,color] ease-in-out duration-150 ',
      config.settings.grid.style === 0 ? 'p-1 w-fit h-fit' : 'w-full h-full',
      isActive
        ? (isContentActive ? 'border-primary' : 'border-primary/50')
        : 'border-transparent',
      config.settings.grid.style === 0 && isSelected ? 'bg-base-100 hover:bg-base-100' : 'hover:bg-base-100/30 hover:text-base-content ',
    ]"
    @click="(event: MouseEvent) => $emit('clicked', { shiftKey: event.shiftKey, metaKey: event.metaKey, ctrlKey: event.ctrlKey })"
    @dblclick="handleDoubleClick"
    @contextmenu="handleContextMenu"
  >
    <div
      ref="containerRef"
      data-thumbnail-container
      :class="[thumbnailCornerClass, 'relative flex items-center justify-center overflow-hidden bg-base-200/70']"
      :style="layoutStyle"
      @pointerenter="startMediaPreview"
      @pointerleave="stopMediaPreview"
    >
      <IconPhoto v-if="!isThumbnailLoaded" class="absolute size-8 text-base-content/25" />
      <!-- image -->
      <img
        v-if="thumbnailSrc"
        :src="thumbnailSrc"
        draggable="false"
        :class="{
          'group-hover:scale-115': shouldScaleThumbnail,
          'scale-115': shouldScaleThumbnail && isSelected,
          'object-contain': fitRawPairPreview || (!isGeometryGridStyle && config.settings.grid.scaling === 0),
          'object-cover': !fitRawPairPreview && (isGeometryGridStyle || config.settings.grid.scaling === 1),
          'object-fill': !isGeometryGridStyle && config.settings.grid.scaling === 2,
          'transition-all': !isTransitionDisabled && normalizedRotate === 0,
          'opacity-0': !isThumbnailLoaded,
          'opacity-100': isThumbnailLoaded,
        }"
        :style="imgStyle"
        loading="eager"
        @load="handleThumbnailLoad"
        @error="retryThumbnail"
      />
      <video
        v-if="showVideoPreview"
        ref="previewVideoRef"
        class="pointer-events-none absolute inset-0 transition-opacity duration-100"
        :class="{
          'object-contain': !isGeometryGridStyle && config.settings.grid.scaling === 0,
          'object-cover': isGeometryGridStyle || config.settings.grid.scaling === 1,
          'object-fill': !isGeometryGridStyle && config.settings.grid.scaling === 2,
          'scale-115': shouldScaleThumbnail,
          'opacity-100': isVideoPreviewReady,
          'opacity-0': !isVideoPreviewReady,
        }"
        :style="imgStyle"
        :poster="thumbnailSrc"
        draggable="false"
        muted
        autoplay
        loop
        playsinline
        preload="metadata"
        @canplay="isVideoPreviewReady = true"
        @playing="isVideoPreviewReady = true"
        @error="stopMediaPreview"
      ></video>
      <img
        v-if="showAnimatedImagePreview"
        :src="animatedImagePreviewSrc"
        draggable="false"
        class="pointer-events-none absolute inset-0 transition-opacity duration-100"
        :class="{
          'object-contain': !isGeometryGridStyle && config.settings.grid.scaling === 0,
          'object-cover': isGeometryGridStyle || config.settings.grid.scaling === 1,
          'object-fill': !isGeometryGridStyle && config.settings.grid.scaling === 2,
          'scale-115': shouldScaleThumbnail,
          'opacity-100': isAnimatedImagePreviewReady,
          'opacity-0': !isAnimatedImagePreviewReady,
        }"
        :style="previewMediaStyle"
        @load="isAnimatedImagePreviewReady = true"
        @error="stopMediaPreview"
      />

      <div v-if="originalUnavailable" class="pointer-events-none absolute right-0.5 bottom-0.5 z-20 thumb-badge" :aria-label="t('offline.original_unavailable')">
        <IconExclamation class="h-3.5 w-3.5 shrink-0 text-warning/70" />
      </div>
      <div v-else-if="file.cloud_only" class="absolute right-0.5 bottom-0.5 z-20 thumb-badge" :title="cloudOnlyLabel" :aria-label="cloudOnlyLabel">
        <IconCloud class="h-3.5 w-3.5 shrink-0" />
      </div>

      <!-- status badges -->
      <div
        v-if="statusBadges.length > 0"
        class="pointer-events-none absolute inset-x-0 top-0 h-16"
      ></div>
      <div
        v-if="statusBadges.length > 0"
        class="pointer-events-none absolute left-0.5 top-0.5 z-10 flex max-w-[calc(100%-2.5rem)] flex-wrap gap-0.5"
      >
        <div
          v-for="badge in statusBadges"
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
          <span v-if="badge.label" class="leading-none">
            {{ badge.label }}
          </span>
          <component
            v-if="badge.trailingIcon"
            :is="badge.trailingIcon"
            :class="['h-3.5 w-3.5 shrink-0', badge.trailingIconClass]"
            :style="badge.trailingIconStyle"
          />
        </div>
      </div>

      <!-- bottom badges -->
      <div
        v-if="thumbnailBadge"
        :class="[
          'pointer-events-none absolute left-0.5 z-10 flex items-center gap-0.5',
          hasBottomMediaBadges ? 'bottom-6' : 'bottom-0.5',
        ]"
      >
        <div class="thumb-badge">
          {{ thumbnailBadge }}
        </div>
      </div>
      <div
        v-if="hasBottomMediaBadges"
        class="pointer-events-none absolute left-0.5 bottom-0.5 z-10 flex items-center gap-0.5"
      >
        <div
          v-if="isLivePhoto || isMotionPhoto"
          class="thumb-badge"
        >
          <IconLivePhoto class="h-3.5 w-3.5 shrink-0" />
          <span class="leading-none">{{ isLivePhoto ? t('image_viewer.live') : t('image_viewer.motion') }}</span>
        </div>
        <div
          v-if="isRawJpegPair"
          class="thumb-badge"
        >
          <span class="leading-none">{{ rawJpegPairBadge }}</span>
        </div>
        <div
          v-if="videoDurationBadge"
          class="thumb-badge"
        >
          {{ videoDurationBadge }}
        </div>
        <div
          v-if="dedupStatus"
          class="thumb-badge"
          :class="dedupStatus === 'keep' ? 'text-base-content/70' : 'text-error/70'"
        >
          {{ dedupStatus === 'keep' ? 'KEEP' : 'DUP' }}
        </div>
      </div>

      <!-- select checkbox -->
      <div v-if="selectMode" class="absolute right-0.5 top-0.5">
        <label class="flex items-center text-primary cursor-pointer" @click.stop @dblclick.stop>
          <input
            type="checkbox"
            class="checkbox checkbox-sm border-base-content/30 hover:border-base-content/70"
            :class="isSelected ? (isContentActive ? 'checkbox-primary' : 'checkbox-primary opacity-50') : ''"
            :checked="isSelected"
            @click.stop="(event: MouseEvent) => $emit('select-toggled', event.shiftKey)"
          />
        </label>
      </div>

      <!-- per-file menu; multi-select uses the shared context menu -->
      <div v-if="!selectMode" class="absolute right-0.5 top-0.5">
        <ContextMenu
          ref="contextMenuRef"
          :class="[
            !isSelected ? 'invisible group-hover:visible bg-base-300/30 rounded-box' : 'bg-base-300/30 rounded-box'
          ]"
          :iconMenu="IconMore"
          :menuItems="menuItems"
          :smallIcon="true"
        />
      </div>
    </div>

    <!-- label -->
    <div 
      v-if="config.settings.grid.style === 0" 
      class="flex flex-col items-center" 
      :class="{ 
        'text-primary': isSelected && isContentActive,
        'text-primary/50': isSelected && !isContentActive,
      }"
      :style="{ width: layoutStyle.width }"
    >
      <span 
        class="w-full text-sm text-center whitespace-pre text-nowrap text-ellipsis overflow-hidden"
        :title="getGridLabelTooltip(file, config.settings.grid.labelPrimary)"
      >
        {{ getGridLabelText(file, config.settings.grid.labelPrimary) }}
      </span>
      <span 
        class="w-full text-xs text-center whitespace-pre text-nowrap text-ellipsis overflow-hidden"
        :title="getGridLabelTooltip(file, config.settings.grid.labelSecondary)"
      >
        {{ getGridLabelText(file, config.settings.grid.labelSecondary) }}
      </span>
    </div>

  </div>
</template>

<script setup lang="ts">
import { computed, nextTick, ref, watch, toRef, onBeforeUnmount, type CSSProperties, type Component } from 'vue';
import { useI18n } from 'vue-i18n';
import { useUIStore } from '@/stores/uiStore';
import { config } from '@/common/config';
import { THUMBNAIL_BADGE } from '@/common/constants';
import { isMac, shortenFilename, formatFileSize, formatDimensionText, formatDuration, formatTimestamp, formatCaptureSettings, formatCaptureSettingValue, formatCameraInfo, getAssetSrc, getThumbUrl, getFileExtension } from '@/common/utils';
import { isWebViewVideoPlaybackDisabled, getGStreamerAvailability } from '@/common/video';
import { claimHoverPreview, releaseHoverPreview } from '@/common/hoverPreview';
import ContextMenu from '@/components/ContextMenu.vue';
import { isOriginalUnavailable } from '@/common/availability';
import { isCloudOnlyFile } from '@/common/api';
import { useFileMenuItems } from '@/common/fileMenu';

import { 
  IconMore,
  IconExclamation,
  IconCloud,
  IconPhoto,
  IconHeartFilled,
  IconTag,
  IconStack,
  IconRotate,
  IconComment,
  IconStarFilled,
  IconFlagFilled,
  IconFlagOff,
  IconLivePhoto
} from '@/common/icons';

const props = defineProps({
  file: {
    type: Object,
    required: true,
  },
  isSelected: {
    type: Boolean,
    default: false,
  },
  isActive: {
    type: Boolean,
    default: false,
  },
  selectMode: {
    type: Boolean,
    default: false,
  },
  querySource: {
    type: String,
    default: '',
  },
  dedupStatus: {
    type: String as () => 'keep' | 'dup' | '',
    default: '',
  },
  gridSize: {
    type: Number,
    required: true,
  },
});

const emit = defineEmits([
    'clicked',
    'dblclicked',
    'select-toggled',
    'action',
    'selection-context-menu',
]);

const isTransitionDisabled = ref(false);
let transitionTimeout: NodeJS.Timeout | null = null;

const containerRef = ref<HTMLElement | null>(null);
const contextMenuRef = ref<InstanceType<typeof ContextMenu> | null>(null);
const previewVideoRef = ref<HTMLVideoElement | null>(null);
const containerWidth = ref(0);
const containerHeight = ref(0);
let resizeObserver: ResizeObserver | null = null;
let previewTimer: ReturnType<typeof setTimeout> | null = null;
let animatedImagePreviewTimer: ReturnType<typeof setTimeout> | null = null;
const showVideoPreview = ref(false);
const isVideoPreviewReady = ref(false);
const showAnimatedImagePreview = ref(false);
const isAnimatedImagePreviewReady = ref(false);
const originalUnavailable = computed(() => isOriginalUnavailable(props.file));
const isVideoFile = computed(() => props.file?.file_type === 2);
const isLivePhoto = computed(() => props.file?.media_subtype === 'live_photo' && !!props.file?.live_photo_video_path);
const isMotionPhoto = computed(() => props.file?.media_subtype === 'motion_photo');
const isRawJpegPair = computed(() => config.settings.groupRawJpegPairs && props.file?.media_subtype === 'raw_jpeg_pair');
const rawJpegPairBadge = computed(() => {
  const extension = getFileExtension(props.file?.live_photo_video_path || '').toLowerCase();
  return ['heic', 'heif', 'hif'].includes(extension) ? 'RAW+HEIC' : 'RAW+JPEG';
});
const previewVideoPath = computed(() => isLivePhoto.value ? props.file.live_photo_video_path : props.file?.file_path);
const canPreviewVideo = computed(() => (
  !originalUnavailable.value && (isVideoFile.value || isLivePhoto.value)
  && !!previewVideoPath.value
  && !isWebViewVideoPlaybackDisabled(previewVideoPath.value)
  // The companion video is checked live before playback.
  && (isLivePhoto.value || !props.file.cloud_only)
));
const ANIMATABLE_IMAGE_EXTENSIONS = new Set(['gif', 'png', 'apng', 'webp', 'avif']);
const isAnimatableImageFile = computed(() => ANIMATABLE_IMAGE_EXTENSIONS.has(
  getFileExtension(props.file?.name || props.file?.file_path || '').toLowerCase(),
));
const isGifFile = computed(() => getFileExtension(props.file?.name || props.file?.file_path || '').toLowerCase() === 'gif');
const animatedImagePreviewSrc = computed(() => getAssetSrc(props.file?.file_path || '', Number(props.file?.modified_at || 0)));
const canPreviewAnimatedImage = computed(() => !originalUnavailable.value && !props.file?.cloud_only
  && isAnimatableImageFile.value && !!animatedImagePreviewSrc.value);
const isGeometryGridStyle = computed(() => config.settings.grid.style === 2 || config.settings.grid.style === 3);
// Pair previews can have a different crop than the RAW dimensions used by layout.
const fitRawPairPreview = computed(() => isGeometryGridStyle.value && isRawJpegPair.value
  && config.settings.groupRawJpegPairs && config.settings.rawPairDisplaySource === 'jpeg'
  && !!props.file?.live_photo_video_path);
const shouldScaleThumbnail = computed(() => !fitRawPairPreview.value
  && (config.settings.grid.style === 1 || isGeometryGridStyle.value));
const thumbnailCornerClass = computed(() => (
  config.settings.grid.thumbnailCorners === 1 ? 'rounded-none' : 'rounded-box'
));
const thumbnailSrc = ref(props.file.thumbnail || '');
const isThumbnailLoaded = ref(false);
let thumbnailRetryCount = 0;

watch(
  [() => props.file?.id, () => props.file?.thumbnail],
  ([, src]) => {
    thumbnailSrc.value = String(src || '');
    isThumbnailLoaded.value = false;
    thumbnailRetryCount = 0;
  },
  { immediate: true },
);

function handleThumbnailLoad() {
  isThumbnailLoaded.value = true;
}

function retryThumbnail() {
  const isThumbnailProtocol = thumbnailSrc.value.startsWith('thumb://localhost')
    || thumbnailSrc.value.startsWith('http://thumb.localhost')
    || thumbnailSrc.value.startsWith('https://thumb.localhost');
  if (thumbnailRetryCount > 0 || !isThumbnailProtocol) {
    return;
  }
  thumbnailRetryCount++;
  isThumbnailLoaded.value = false;
  thumbnailSrc.value = getThumbUrl(props.file.id, true, config.settings.thumbnailSize, Number(props.file.modified_at || 0));
}

// Robust ResizeObserver setup using watch to handle v-if
watch(containerRef, (el) => {
  if (resizeObserver) {
    resizeObserver.disconnect();
    resizeObserver = null;
  }
  if (el) {
    resizeObserver = new ResizeObserver((entries) => {
      for (const entry of entries) {
        containerWidth.value = entry.contentRect.width;
        containerHeight.value = entry.contentRect.height;
      }
    });
    resizeObserver.observe(el);
  }
});

onBeforeUnmount(() => {
  if (resizeObserver) {
    resizeObserver.disconnect();
  }
  stopMediaPreview();
});

watch(() => config.settings.grid.style, () => {
  isTransitionDisabled.value = true;
  if (transitionTimeout) {
    clearTimeout(transitionTimeout);
  }
  transitionTimeout = setTimeout(() => {
    isTransitionDisabled.value = false;
  }, 500);
});

watch(() => props.file.rotate, () => {
  isTransitionDisabled.value = true;
  if (transitionTimeout) {
    clearTimeout(transitionTimeout);
  }
  transitionTimeout = setTimeout(() => {
    isTransitionDisabled.value = false;
  }, 500);
});

watch(() => [props.file.file_path, props.file.live_photo_video_path], () => {
  stopMediaPreview();
});

function startMediaPreview() {
  if (!canPreviewVideo.value && !canPreviewAnimatedImage.value) return;
  claimHoverPreview(stopMediaPreview);
  startVideoPreview();
  startAnimatedImagePreview();
}

function stopMediaPreview() {
  stopVideoPreview();
  stopAnimatedImagePreview();
  releaseHoverPreview(stopMediaPreview);
}

let videoPreviewRequest = 0;

function startVideoPreview() {
  if (!canPreviewVideo.value || previewTimer || showVideoPreview.value) return;

  const request = ++videoPreviewRequest;
  previewTimer = setTimeout(async () => {
    previewTimer = null;
    const available = await getGStreamerAvailability();
    if (request !== videoPreviewRequest || !available) return;
    if (!canPreviewVideo.value || !previewVideoPath.value) return;
    // The catalog flag can be stale; playing an online-only video would download it.
    if (await isCloudOnlyFile(previewVideoPath.value) || request !== videoPreviewRequest) return;

    isVideoPreviewReady.value = false;
    showVideoPreview.value = true;
    await nextTick();

    const video = previewVideoRef.value;
    if (!video || request !== videoPreviewRequest) return;

    video.src = getAssetSrc(previewVideoPath.value);
    video.muted = true;

    try {
      await video.play();
    } catch {
      stopMediaPreview();
    }
  }, 400);
}

function stopVideoPreview() {
  videoPreviewRequest++;
  if (previewTimer) {
    clearTimeout(previewTimer);
    previewTimer = null;
  }

  const video = previewVideoRef.value;
  if (video) {
    video.pause();
    video.removeAttribute('src');
    video.load();
  }

  isVideoPreviewReady.value = false;
  showVideoPreview.value = false;
}

let animatedImagePreviewRequest = 0;

function startAnimatedImagePreview() {
  if (!canPreviewAnimatedImage.value || animatedImagePreviewTimer || showAnimatedImagePreview.value) return;

  const request = ++animatedImagePreviewRequest;
  const filePath = props.file?.file_path || '';
  animatedImagePreviewTimer = setTimeout(async () => {
    animatedImagePreviewTimer = null;
    if (!canPreviewAnimatedImage.value) return;
    // The catalog flag can be stale; loading an online-only image would download it.
    if (await isCloudOnlyFile(filePath)) return;
    if (request !== animatedImagePreviewRequest || props.file?.file_path !== filePath
      || !canPreviewAnimatedImage.value) return;
    isAnimatedImagePreviewReady.value = false;
    showAnimatedImagePreview.value = true;
  }, 400);
}

function stopAnimatedImagePreview() {
  animatedImagePreviewRequest++;
  if (animatedImagePreviewTimer) {
    clearTimeout(animatedImagePreviewTimer);
    animatedImagePreviewTimer = null;
  }
  isAnimatedImagePreviewReady.value = false;
  showAnimatedImagePreview.value = false;
}

function handleDoubleClick(event: MouseEvent) {
  if (isGifFile.value) {
    // The hover preview and viewer must not share an animated GIF resource.
    // Tear down the thumbnail image first, then give the viewer a fresh asset
    // URL so Chromium creates a new decoder from frame zero.
    stopMediaPreview();
    const filePath = props.file?.file_path;
    if (filePath) {
      uiStore.updateFileVersion(filePath);
    }
  }

  emit('dblclicked', {
    shiftKey: event.shiftKey,
    metaKey: event.metaKey,
    ctrlKey: event.ctrlKey,
  });
}

function handleContextMenu(event: MouseEvent) {
  event.preventDefault();
  event.stopPropagation();
  if (props.selectMode) {
    emit('selection-context-menu', event);
    return;
  }
  if (!props.isSelected) {
    emit('clicked', false);
  }
  contextMenuRef.value?.open?.(event.clientX, event.clientY);
}


const layoutStyle = computed(() => {
  const { style } = config.settings.grid;
  const size = props.gridSize;
  if (style === 0) return { width: `${size}px`, height: `${size}px` };
  if (style === 1) return { width: '100%', height: `${size}px` };
  return { width: '100%', height: '100%' };
});

const imgStyle = computed((): CSSProperties => {
  const { style } = config.settings.grid;
  const size = props.gridSize;
  const isRotated = props.file.rotate && props.file.rotate % 180 !== 0;

  if (isRotated) {
    let w = containerWidth.value;
    let h = containerHeight.value;

    // Optimization: For fixed-size grid (style 0), we know dimensions immediately
    if ((w === 0 || h === 0) && style === 0) {
      w = size;
      h = size;
    }

    if (w > 0 && h > 0) {
      return {
        position: 'absolute',
        left: '50%',
        top: '50%',
        width: `${h}px`,
        height: `${w}px`,
        maxWidth: 'none',
        maxHeight: 'none',
        flex: 'none',
        transform: `translate(-50%, -50%) rotate(${props.file.rotate}deg)`,
        opacity: 1,
      };
    }
    
    // Fallback: Hide until dimensions are known to prevent blinking/glitches
    return { opacity: 0 };
  }

  // Standard behavior for non-swapped rotations (0, 180, 360...)
  return {
    ...layoutStyle.value,
    transform: `rotate(${props.file.rotate || 0}deg)`,
    opacity: 1,
  } as CSSProperties;
});

const previewMediaStyle = computed((): CSSProperties => {
  const { opacity: _opacity, ...style } = imgStyle.value;
  return style;
});

const uiStore = useUIStore();
const isContentActive = computed(() =>
  uiStore.activePane === 'content' && uiStore.inputStack.length === 0
);
const { locale, messages, t } = useI18n();
const cloudOnlyLabel = computed(() => t(props.file?.metadata_pending ? 'offline.cloud_only_pending' : 'offline.cloud_only'));
const localeMsg = computed(() => messages.value[locale.value] as any);

const menuItems = useFileMenuItems(
  toRef(props, 'file'),
  localeMsg,
  isMac,
  t,
  (action) => emit('action', action),
);

const getGridLabelText = (file: any, option: number) => {
  switch (option) {
    case 0: return '';
    case 1: return shortenFilename(file.name) || ' ';
    case 2: return formatFileSize(file.size) || ' ';
    case 3: return formatDimensionText(file.width, file.height) || ' ';
    case 4: return formatTimestamp(file.taken_date, 'date_time') || ' ';
    case 5: return file.geo_name || ' ';
    case 6: return formatCameraInfo(file.e_make, file.e_model) || ' ';
    case 7: return file.e_lens_model || ' ';
    case 8: return formatCaptureSettings(file.e_focal_length, file.e_exposure_time, file.e_f_number, file.e_iso_speed, file.e_exposure_bias) || ' ';
    default: return '';
  }
};

const getGridLabelTooltip = (file: any, option: number) => {
  if (option === 1) return file.name;
  const text = getGridLabelText(file, option);
  return text === ' ' ? '' : text;
};

type ThumbnailBadge = {
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
  trailingIconStyle?: CSSProperties;
};

const normalizedRotate = computed(() => {
  const rotate = Number(props.file.rotate || 0) % 360;
  return rotate < 0 ? rotate + 360 : rotate;
});

const videoDurationBadge = computed(() => {
  if (props.file?.file_type !== 2) return '';

  const duration = Number(props.file?.duration);
  if (!Number.isFinite(duration)) return '';

  const formattedDuration = formatDuration(duration);
  return formattedDuration;
});

const thumbnailBadge = computed(() => {
  const summary = Number(config.settings.grid.thumbnailBadge || 0);
  if (summary === THUMBNAIL_BADGE.EMPTY) return '';

  const file = props.file || {};
  switch (summary) {
    case THUMBNAIL_BADGE.FILE_FORMAT: {
      const extension = getFileExtension(file.name || file.file_path || '').trim();
      return extension ? extension.toUpperCase() : '';
    }
    case THUMBNAIL_BADGE.ISO:
      return file.e_iso_speed ? `ISO ${formatCaptureSettingValue(file.e_iso_speed)}` : '';
    case THUMBNAIL_BADGE.SHUTTER_SPEED:
      return formatCaptureSettingValue(file.e_exposure_time);
    case THUMBNAIL_BADGE.APERTURE: {
      const fNumber = formatCaptureSettingValue(file.e_f_number).trim();
      return fNumber ? (fNumber.toLowerCase().startsWith('f/') ? fNumber : `f/${fNumber}`) : '';
    }
    case THUMBNAIL_BADGE.FOCAL_LENGTH:
      return formatCaptureSettingValue(file.e_focal_length);
    case THUMBNAIL_BADGE.EXPOSURE:
      return formatCaptureSettingValue(file.e_exposure_bias);
    default:
      return '';
  }
});

const hasBottomMediaBadges = computed(() => (
  isLivePhoto.value || isMotionPhoto.value || isRawJpegPair.value || Boolean(videoDurationBadge.value) || Boolean(props.dedupStatus)
));

const statusBadges = computed<ThumbnailBadge[]>(() => {
  const badges: ThumbnailBadge[] = [];
  const metaIcons: ThumbnailBadge['icons'] = [];
  const rating = Number(props.file.rating || 0);
  const cullingFlag = Number(props.file.culling_flag ?? props.file.cullingFlag ?? 0);
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

  if (props.file.is_favorite) {
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
  
  if (props.file.has_tags) metaIcons.push({ icon: IconTag });
  if (props.file.has_collections) metaIcons.push({ icon: IconStack });
  if (props.file.comments?.length > 0) metaIcons.push({ icon: IconComment });
  if (normalizedRotate.value > 0) {
    metaIcons.push({
      icon: IconRotate,
      style: { transform: `rotate(${normalizedRotate.value}deg)` },
    });
  }
  if (metaIcons.length > 0) {
    badges.push({
      key: 'meta',
      icons: metaIcons,
    });
  }

  return badges;
});
</script>

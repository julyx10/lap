<template>
  <div
    :class="[
      'border-2 rounded-box flex flex-col items-center cursor-pointer group',
      isTransitionDisabled ? 'transition-none' : 'transition-[background-color,color] ease-in-out duration-150 ',
      config.settings.grid.style === 0 ? 'p-1 w-fit h-fit' : 'w-full h-full',
      isActive
        ? (isContentActive ? 'border-primary' : 'border-primary/50')
        : 'border-transparent',
      config.settings.grid.style === 0 && isSelected ? 'bg-base-100 hover:bg-base-100' : 'hover:bg-base-100/30 hover:text-base-content ',
    ]"
    @click="(event: MouseEvent) => $emit('clicked', { shiftKey: event.shiftKey, metaKey: event.metaKey, ctrlKey: event.ctrlKey })"
    @dblclick="(event: MouseEvent) => $emit('dblclicked', { shiftKey: event.shiftKey, metaKey: event.metaKey, ctrlKey: event.ctrlKey })"
    @contextmenu="handleContextMenu"
  >
    <div
      ref="containerRef"
      class="rounded-box relative flex items-center justify-center overflow-hidden bg-base-200/70"
      :style="layoutStyle"
      @pointerenter="startVideoPreview"
      @pointerleave="stopVideoPreview"
    >
      <!-- image -->
      <img
        v-if="thumbnailSrc"
        :src="thumbnailSrc"
        draggable="false"
        :class="{
          'group-hover:scale-115': shouldScaleThumbnail,
          'scale-115': shouldScaleThumbnail && isSelected,
          'object-contain': !isGeometryGridStyle && config.settings.grid.scaling === 0,
          'object-cover': isGeometryGridStyle || config.settings.grid.scaling === 1,
          'object-fill': !isGeometryGridStyle && config.settings.grid.scaling === 2,
          'transition-all': !isTransitionDisabled && normalizedRotate === 0,
          'opacity-0': !isThumbnailLoaded,
          'opacity-100': isThumbnailLoaded,
        }"
        :style="imgStyle"
        loading="lazy"
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
        @error="stopVideoPreview"
      ></video>

      <!-- status badges -->
      <div
        v-if="statusBadges.length > 0"
        class="pointer-events-none absolute inset-x-0 top-0 h-16"
      ></div>
      <div
        v-if="statusBadges.length > 0"
        class="pointer-events-none absolute left-0.5 top-0.5 z-10 flex max-w-[calc(100%-2.5rem)] flex-wrap gap-1"
      >
        <div
          v-for="badge in statusBadges"
          :key="badge.key"
          class="thumb-badge thumb-badge-muted"
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
        </div>
      </div>

      <!-- video duration and dedup badges -->
      <div
        v-if="videoDurationBadge || dedupStatus"
        class="pointer-events-none absolute left-0.5 bottom-0.5 z-10 flex items-center gap-1"
      >
        <div
          v-if="videoDurationBadge"
          class="thumb-badge thumb-badge-muted"
        >
          {{ videoDurationBadge }}
        </div>
        <div
          v-if="dedupStatus"
          class="thumb-badge"
          :class="dedupStatus === 'keep' ? 'text-primary/70' : 'text-error/70'"
        >
          {{ dedupStatus === 'keep' ? 'KEEP' : 'DUP' }}
        </div>
      </div>

      <!-- select checkbox -->
      <div v-if="selectMode" class="absolute right-0.5 top-0.5">
        <label class="flex items-center text-primary cursor-pointer" @click.stop @dblclick.stop>
          <input
            type="checkbox"
            class="checkbox checkbox-sm"
            :class="isSelected ? (isContentActive ? 'checkbox-primary' : 'checkbox-primary opacity-50') : ''"
            :checked="isSelected"
            @click.stop="(event: MouseEvent) => $emit('select-toggled', event.shiftKey)"
          />
        </label>
      </div>

      <!-- context menu (non-select only; in select mode a single shared menu is
           owned by the parent and opened via the select-contextmenu event) -->
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
import { isMac, shortenFilename, formatFileSize, formatDimensionText, formatDuration, formatTimestamp, formatCaptureSettings, formatCameraInfo, getAssetSrc, getThumbUrl } from '@/common/utils';
import { isWebViewVideoPlaybackDisabled } from '@/common/video';
import ContextMenu from '@/components/ContextMenu.vue';
import { useFileMenuItems } from '@/common/fileMenu';

import { 
  IconMore,
  IconHeartFilled,
  IconTag,
  IconRotate,
  IconComment,
  IconStarFilled,
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
});

const emit = defineEmits([
    'clicked',
    'dblclicked',
    'select-toggled',
    'action',
    'select-contextmenu'
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
const showVideoPreview = ref(false);
const isVideoPreviewReady = ref(false);
const isVideoFile = computed(() => props.file?.file_type === 2);
const canPreviewVideo = computed(() => (
  isVideoFile.value
  && !!props.file?.file_path
  && !isWebViewVideoPlaybackDisabled(props.file.file_path)
));
const isGeometryGridStyle = computed(() => config.settings.grid.style === 2 || config.settings.grid.style === 3);
const shouldScaleThumbnail = computed(() => config.settings.grid.style === 1 || isGeometryGridStyle.value);
const thumbnailSrc = ref(props.file.thumbnail || '');
const isThumbnailLoaded = ref(false);
let thumbnailRetryCount = 0;

watch(
  () => [props.file?.id, props.file?.thumbnail],
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
  thumbnailSrc.value = getThumbUrl(props.file.id, true, config.settings.thumbnailSize);
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
  stopVideoPreview();
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

watch(() => props.file.file_path, () => {
  stopVideoPreview();
});

function startVideoPreview() {
  if (!canPreviewVideo.value || previewTimer || showVideoPreview.value) return;

  previewTimer = setTimeout(async () => {
    previewTimer = null;
    if (!canPreviewVideo.value || !props.file?.file_path) return;

    isVideoPreviewReady.value = false;
    showVideoPreview.value = true;
    await nextTick();

    const video = previewVideoRef.value;
    if (!video) return;

    video.src = getAssetSrc(props.file.file_path);
    video.muted = true;

    try {
      await video.play();
    } catch {
      stopVideoPreview();
    }
  }, 400);
}

function stopVideoPreview() {
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

function handleContextMenu(event: MouseEvent) {
  event.preventDefault();
  event.stopPropagation();
  // In multi-select mode a single shared menu (owned by the parent) acts on the
  // whole selection; just forward the cursor position and let the parent decide
  // whether and where to open it.
  if (props.selectMode) {
    // Pass this thumbnail's own selection state up; the parent shouldn't re-derive
    // it from an index (which can disagree under grouping/virtualization).
    emit('select-contextmenu', { x: event.clientX, y: event.clientY, isSelected: props.isSelected });
    return;
  }
  if (!props.isSelected) {
    emit('clicked', false);
  }
  contextMenuRef.value?.open?.(event.clientX, event.clientY);
}


const layoutStyle = computed(() => {
  const { style, size } = config.settings.grid;
  if (style === 0) return { width: `${size}px`, height: `${size}px` };
  if (style === 1) return { width: '100%', height: `${size}px` };
  return { width: '100%', height: '100%' };
});

const imgStyle = computed((): CSSProperties => {
  const { style, size } = config.settings.grid;
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

const uiStore = useUIStore();
const isContentActive = computed(() =>
  uiStore.activePane === 'content' && uiStore.inputStack.length === 0
);
const { locale, messages, t } = useI18n();
const localeMsg = computed(() => messages.value[locale.value] as any);

const isCollectionView = computed(() => props.querySource === 'collection');

const menuItems = useFileMenuItems(
  toRef(props, 'file'),
  localeMsg,
  isMac,
  t,
  (action) => emit('action', action),
  { isCollectionView },
);

const getGridLabelText = (file: any, option: number) => {
  switch (option) {
    case 0: return '';
    case 1: return shortenFilename(file.name) || ' ';
    case 2: return formatFileSize(file.size) || ' ';
    case 3: return formatDimensionText(file.width, file.height) || ' ';
    case 4: return formatTimestamp(file.taken_date, localeMsg.value.format.date_time) || ' ';
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
};

const normalizedRotate = computed(() => {
  const rotate = Number(props.file.rotate || 0) % 360;
  return rotate < 0 ? rotate + 360 : rotate;
});

const videoDurationBadge = computed(() => {
  return props.file?.file_type === 2
    ? formatDuration(props.file.duration)
    : '';
});

const statusBadges = computed<ThumbnailBadge[]>(() => {
  const badges: ThumbnailBadge[] = [];
  const rating = Number(props.file.rating || 0);
  const metaIcons: ThumbnailBadge['icons'] = [];

  if (props.file.is_favorite) {
    badges.push({
      key: 'favorite',
      icon: IconHeartFilled,
      iconClass: 'text-error',
      label: rating > 0 ? `${rating}` : undefined,
    });
  } else if (rating > 0) {
    badges.push({
      key: 'rating',
      icon: IconStarFilled,
      iconClass: 'text-warning',
      label: `${rating}`,
    });
  }
  
  if (props.file.has_tags) {
    metaIcons.push({
      icon: IconTag,
    });
  }

  if (props.file.comments?.length > 0) {
    metaIcons.push({
      icon: IconComment,
    });
  }

  if (normalizedRotate.value > 0) {
    metaIcons.push({
      icon: IconRotate,
      style: {
        transform: `rotate(${normalizedRotate.value}deg)`,
      },
    });
  }

  if (metaIcons.length > 0) {
    const visibleIcons = metaIcons.slice(0, 3);
    const extraCount = metaIcons.length - visibleIcons.length;
    badges.push({
      key: 'meta',
      icons: visibleIcons,
      label: extraCount > 0 ? `+${extraCount}` : undefined,
    });
  }

  return badges;
});
</script>

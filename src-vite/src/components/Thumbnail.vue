<template>
  <div
    :class="[
      'border-2 flex flex-col items-center cursor-pointer group',
      isTransitionDisabled ? 'transition-none' : 'transition-all ease-in-out duration-300 ',
      config.settings.grid.style === 0 ? 'p-1 rounded-box w-fit h-fit' : 'w-full h-full',
      isSelected && !isTransitionDisabled ? (uiStore.inputStack.length > 0 ? 'border-base-content/30' : 'border-primary') : 'border-transparent',
      config.settings.grid.style === 0 && isSelected ? 'bg-base-100 hover:bg-base-100' : 'hover:bg-base-100/30 hover:text-base-content ',
    ]"
    @click="(event: MouseEvent) => $emit('clicked', event.shiftKey)"
    @dblclick="$emit('dblclicked')"
  >
    <div v-if="file.thumbnail" 
      ref="containerRef"
      :class="[
        'relative flex items-center justify-center overflow-hidden', 
        config.settings.grid.style === 0 ? 'rounded-box' : 'w-full h-full',
      ]"
      :style="layoutStyle">
      <!-- image -->
      <img :src="file.thumbnail"
        class="duration-300"
        :class="{
          'group-hover:scale-115': config.settings.grid.style === 1 || config.settings.grid.style === 2,
          'scale-115': (config.settings.grid.style === 1 || config.settings.grid.style === 2) && isSelected,
          'object-contain': config.settings.grid.style !== 2 && config.settings.grid.scaling === 0,
          'object-cover': config.settings.grid.style === 2 || config.settings.grid.scaling === 1,
          'object-fill': config.settings.grid.style !== 2 && config.settings.grid.scaling === 2,
          'transition-all': !isTransitionDisabled,
        }"
        :style="imgStyle"
        loading="lazy"
      />

      <!-- status badges -->
      <div
        v-if="statusBadges.length > 0"
        class="pointer-events-none absolute inset-x-0 top-0 h-16 bg-linear-to-b from-black/48 via-black/12 to-transparent"
      />
      <div
        v-if="statusBadges.length > 0"
        class="pointer-events-none absolute left-1 top-1 z-10 flex max-w-[calc(100%-2.5rem)] flex-wrap gap-1"
      >
        <div
          v-for="badge in statusBadges"
          :key="badge.key"
          :class="['thumb-badge', badge.highlight ? 'thumb-badge-highlight' : 'thumb-badge-muted']"
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
            class="h-3.5 w-3.5 shrink-0"
            :style="badge.iconStyle"
          />
          <span v-if="badge.label" class="leading-none">
            {{ badge.label }}
          </span>
        </div>
      </div>

      <!-- select checkbox -->
      <div v-if="selectMode" class="absolute right-1 top-0.5">
        <component 
          :is="file?.isSelected ? IconChecked : IconUnChecked" 
          :class="[
            't-icon-size-sm', 
            file?.isSelected && uiStore.inputStack.length === 0 ? 'text-primary hover:text-primary' : 'text-base-content/30 hover:text-base-content/70'
          ]" 
          @click.stop="(event: MouseEvent) => $emit('select-toggled', event.shiftKey)"
        />
      </div>

      <!-- context menu -->
      <div v-if="!selectMode" class="absolute right-0 top-0">
        <ContextMenu
          :class="[
            !isSelected ? 'invisible group-hover:visible' : ''
          ]"
          :iconMenu="IconMore"
          :menuItems="menuItems"
          :smallIcon="true"
        />
      </div>
    </div>
    
    <!-- skeleton for loading thumbnail -->
    <div v-else 
      :class="[
        'relative flex items-center justify-center overflow-hidden skeleton', 
        config.settings.grid.style === 0 ? 'rounded-box' : '',
      ]"
      :style="layoutStyle"
    ></div>

    <!-- label -->
    <div 
      v-if="config.settings.grid.style === 0" 
      class="flex flex-col items-center" 
      :class="{ 'text-primary': isSelected }"
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
import { computed, ref, watch, toRef, onBeforeUnmount, type CSSProperties, type Component } from 'vue';
import { useI18n } from 'vue-i18n';
import { useUIStore } from '@/stores/uiStore';
import { config } from '@/common/config';
import { isMac, shortenFilename, formatFileSize, formatDimensionText, formatDuration, formatTimestamp, formatCaptureSettings, formatCameraInfo } from '@/common/utils';
import ContextMenu from '@/components/ContextMenu.vue';
import { useFileMenuItems } from '@/common/fileMenu';

import { 
  IconMore,
  IconHeartFilled,
  IconTag,
  IconRotate,
  IconChecked,
  IconUnChecked,
  IconComment,
  IconClock,
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
  selectMode: {
    type: Boolean,
    default: false,
  },
  showFolderFiles: {
    type: Boolean,
    default: false,
  }
});

const emit = defineEmits([
    'clicked', 
    'dblclicked', 
    'select-toggled', 
    'action'
]);

const isTransitionDisabled = ref(false);
let transitionTimeout: NodeJS.Timeout | null = null;

const containerRef = ref<HTMLElement | null>(null);
const containerWidth = ref(0);
const containerHeight = ref(0);
let resizeObserver: ResizeObserver | null = null;

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
const { locale, messages } = useI18n();
const localeMsg = computed(() => messages.value[locale.value] as any);

const menuItems = useFileMenuItems(
  toRef(props, 'file'),
  localeMsg,
  isMac,
  toRef(props, 'showFolderFiles'),
  (action) => emit('action', action)
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
  highlight?: boolean;
  iconStyle?: CSSProperties;
};

const normalizedRotate = computed(() => {
  const rotate = Number(props.file.rotate || 0) % 360;
  return rotate < 0 ? rotate + 360 : rotate;
});

const statusBadges = computed<ThumbnailBadge[]>(() => {
  const badges: ThumbnailBadge[] = [];
  const rating = Number(props.file.rating || 0);
  const isVideo = props.file.file_type === 2;
  const metaIcons: ThumbnailBadge['icons'] = [];

  if (props.file.is_favorite) {
    badges.push({
      key: 'favorite',
      icon: IconHeartFilled,
      highlight: true,
      label: rating > 0 ? `${rating}` : undefined,
    });
  } else if (rating > 0) {
    badges.push({
      key: 'rating',
      icon: IconStarFilled,
      label: `${rating}`,
      highlight: true,
    });
  }
  
  if (isVideo) {
    badges.push({
      key: 'duration',
      icon: IconClock,
      label: formatDuration(props.file.duration),
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
        transition: 'transform 0.3s ease-in-out',
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

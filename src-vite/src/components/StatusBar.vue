<template>
  <div :class="containerClass">
    <div
      v-if="hasData"
      class="flex gap-4 items-center flex-1 min-w-0 overflow-hidden whitespace-nowrap text-base-content/70"
    >
      <div class="flex items-center shink-0">
        <IconList class="t-icon-size-xs mr-1" />
        <span v-if="selectedItemIndex >= 0">{{ (selectedItemIndex + 1).toLocaleString() + '/' }}</span>
        <span>{{ $t('statusbar.files_summary', { count: totalFileCount.toLocaleString(), size: formatFileSize(totalFileSize) }) }}</span>
      </div>

      <template v-if="selectedItemIndex >= 0 && hasRealSelectedFile">
        <div class="flex items-center gap-1 shink-0">
          <component :is="currentFile?.file_type === 1 || currentFile?.file_type === 3 ? IconPhoto : IconVideo" class="t-icon-size-xs" />
          <span> {{ shortenFilename(currentFile?.name, 32) + ' (' + formatFileSize(currentFile?.size || 0) + ')' }} </span>
        </div>

        <div class="flex items-center gap-1 shink-0">
          <IconAspectRatio class="t-icon-size-xs" />
          <span>{{ formatDimensionText(currentFile?.width, currentFile?.height, true) }}</span>
        </div>

        <div class="flex items-center gap-1 shink-0">
          <IconCalendarDay class="t-icon-size-xs" />
          <span>{{ formatTimestamp(currentFile?.taken_date, 'date_time') }}</span>
        </div>
        
        <div v-if="showFilmStrip || showQuickView || showScale" class="flex items-center gap-1 shink-0">
          <component :is="imageScale >= 1 ? IconZoomIn : IconZoomOut" class="t-icon-size-xs" />
          <span>{{ (imageScale * 100).toFixed(0) }}%</span>
        </div>

        <span v-if="focusStatus" class="shrink-0" role="status" data-focus-status>{{ $t(`image_viewer.focus_${focusStatus}`) }}</span>

        <div v-if="currentFile?.e_model" class="flex items-center gap-1 shink-0">
          <IconCamera class="t-icon-size-xs" />
          <span>{{ currentFile?.e_model }} {{ currentFile?.e_lens_model ? ' (' + currentFile?.e_lens_model + ')' : '' }}</span>
        </div>

        <div
          v-if="currentFile?.e_focal_length || currentFile?.e_exposure_time || currentFile?.e_f_number || currentFile?.e_iso_speed || currentFile?.e_exposure_bias"
          class="flex items-center gap-1 shink-0"
        >
          <IconCameraAperture class="t-icon-size-xs" />
          <span>{{ formatCaptureSettings(currentFile?.e_focal_length, currentFile?.e_exposure_time, currentFile?.e_f_number, currentFile?.e_iso_speed, currentFile?.e_exposure_bias) }}</span>
        </div>

        <div v-if="currentFile?.geo_name" class="flex items-center gap-1 shink-0">
          <IconLocation class="t-icon-size-xs" />
          <span>{{ currentFile?.geo_name }}</span>
        </div>
      </template>
    </div>
    <div v-else-if="emptyMessage" class="flex items-center flex-1 min-w-0 text-base-content/70">
      <IconList class="t-icon-size-xs mr-1 shrink-0" />
      <span>{{ emptyMessage }}</span>
    </div>
    <div
      v-if="showUpdateIcon"
      class="absolute right-1 shrink-0 px-1 py-1 flex items-center gap-1 rounded-full bg-base-300 text-primary/70 text-[11px] font-mono"
    >
      <component :is="updateIconComponent" class="t-icon-size-xs shrink-0" :class="{ 'animate-spin': isUpdateAnimating }" />
      <span v-if="scanText" class="truncate text-right">{{ scanText }}</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { loadFocusInfo } from '@/common/focusInfo.js';
import { config, libConfig } from '@/common/config';
import { orientedFocusRect } from '@/common/focusPoint.js';
import {
  formatFileSize,
  shortenFilename,
  formatDimensionText,
  formatCaptureSettings,
  formatTimestamp,
} from '@/common/utils';
import {
  IconAspectRatio,
  IconPhoto,
  IconVideo,
  IconCamera,
  IconCameraAperture,
  IconLocation,
  IconList,
  IconZoomIn,
  IconZoomOut,
  IconUpdate,
  IconUpdateDot,
  IconCalendarDay,
} from '@/common/icons';

const props = defineProps({
  fileList: {
    type: Array as () => any[],
    default: () => [],
  },
  selectedItemIndex: {
    type: Number,
    default: -1,
  },
  totalFileCount: {
    type: Number,
    default: 0,
  },
  totalFileSize: {
    type: Number,
    default: 0,
  },
  showFilmStrip: {
    type: Boolean,
    default: false,
  },
  showQuickView: {
    type: Boolean,
    default: false,
  },
  imageScale: {
    type: Number,
    default: 1,
  },
  selectedFile: {
    type: Object as () => any,
    default: null,
  },
  showScale: {
    type: Boolean,
    default: false,
  },
  isEmbedded: {
    type: Boolean,
    default: false,
  },
  showUpdateIcon: {
    type: Boolean,
    default: false,
  },
  isUpdateAnimating: {
    type: Boolean,
    default: false,
  },
  updateIcon: {
    type: String as () => 'update' | 'dot',
    default: 'update',
  },
  scanText: {
    type: String,
    default: '',
  },
  emptyMessage: {
    type: String,
    default: '',
  },
});

const updateIconComponent = computed(() =>
  props.updateIcon === 'dot' ? IconUpdateDot : IconUpdate
);

const hasData = computed(() => props.fileList.length > 0 || !!props.selectedFile);
const currentFile = computed(() => {
  if (props.selectedFile) return props.selectedFile;
  return props.fileList[props.selectedItemIndex];
});
const focusStatus = ref('');
watch(() => [config.settings.showFocus, currentFile.value?.file_path, currentFile.value?.file_type,
  currentFile.value?.modified_at, props.showFilmStrip || props.showQuickView || props.showScale] as const,
  async ([enabled, path, type, , viewing], _, onCleanup) => {
    let stale = false;
    onCleanup(() => { stale = true; });
    focusStatus.value = '';
    if (!enabled || !viewing || !path || ![1, 3].includes(Number(type))) return;
    try {
      const info = await loadFocusInfo(path, currentFile.value?.modified_at || 0, libConfig._libraryId);
      if (stale) return;
      focusStatus.value = !orientedFocusRect(info) ? 'unavailable'
        : info?.focus_result === 'focus' ? 'result_focus'
        : info?.focus_result === 'out_of_focus' ? 'result_out_of_focus' : 'result_unknown';
    } catch {
      if (!stale) focusStatus.value = 'unavailable';
    }
  }, { immediate: true });

const hasRealSelectedFile = computed(() => {
  const file = currentFile.value;
  return !!file && !file.isPlaceholder;
});
const containerClass = computed(() => {
  const base = 'px-2 h-8 flex items-center justify-between text-sm cursor-default bg-base-300';
  if (props.isEmbedded) return base;
  return `${base} absolute bottom-0 left-0 right-0 z-30`;
});
</script>

<template>
  <div :class="containerClass">
    <div
      v-if="hasData"
      class="flex gap-4 items-center flex-1 min-w-0 overflow-hidden whitespace-nowrap"
    >
      <div class="flex items-center shink-0">
        <IconFileSearch class="t-icon-size-xs mr-1" />
        <span v-if="selectedItemIndex >= 0">{{ (selectedItemIndex + 1).toLocaleString() + '/' }}</span>
        <span>{{ $t('statusbar.files_summary', { count: totalFileCount.toLocaleString(), size: formatFileSize(totalFileSize) }) }}</span>
      </div>

      <template v-if="selectedItemIndex >= 0">
        <div class="flex items-center gap-1 shink-0">
          <component :is="selectMode ? IconCheckAll : IconChecked" class="t-icon-size-xs" />
          <span>
            {{ selectMode
              ? $t('toolbar.filter.select_count', { count: selectedCount.toLocaleString() }) + ' (' + formatFileSize(selectedSize) + ')'
              : shortenFilename(currentFile?.name, 32) + ' (' + formatFileSize(currentFile?.size || 0) + ')'
            }}
          </span>
        </div>

        <div class="flex items-center gap-1 shink-0">
          <component :is="currentFile?.file_type === 1 ? IconPhoto : IconVideo" class="t-icon-size-xs" />
          <span>{{ formatDimensionText(currentFile?.width, currentFile?.height) }}</span>
        </div>

        <div v-if="showFilmStrip || showQuickView || showScale" class="flex items-center gap-1 shink-0">
          <component :is="imageScale >= 1 ? IconZoomIn : IconZoomOut" class="t-icon-size-xs" />
          <span>{{ (imageScale * 100).toFixed(0) }}%</span>
        </div>

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

        <div class="flex items-center gap-1 shink-0">
          <IconClock class="t-icon-size-xs" />
          <span>{{ formatTimestamp(currentFile?.taken_date, $t('format.date_time')) }}</span>
        </div>
      </template>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import {
  formatFileSize,
  shortenFilename,
  formatDimensionText,
  formatCaptureSettings,
  formatTimestamp,
} from '@/common/utils';
import {
  IconCheckAll,
  IconChecked,
  IconPhoto,
  IconVideo,
  IconCamera,
  IconCameraAperture,
  IconLocation,
  IconClock,
  IconFileSearch,
  IconZoomIn,
  IconZoomOut,
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
  selectMode: {
    type: Boolean,
    default: false,
  },
  selectedCount: {
    type: Number,
    default: 0,
  },
  selectedSize: {
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
});

const hasData = computed(() => props.fileList.length > 0 || !!props.selectedFile);
const currentFile = computed(() => {
  if (props.selectedFile) return props.selectedFile;
  return props.fileList[props.selectedItemIndex];
});
const containerClass = computed(() => {
  const base = 'px-2 h-8 flex items-center justify-between text-sm cursor-default bg-base-300/80 backdrop-blur-md';
  if (props.isEmbedded) return base;
  return `${base} absolute bottom-0 left-0 right-0 z-30`;
});
</script>

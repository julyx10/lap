<template>
  <div :class="containerClass" @contextmenu.prevent.stop="menu?.open($event.clientX, $event.clientY)">
    <div class="flex h-full flex-1 min-w-0 items-center gap-4 overflow-hidden whitespace-nowrap text-base-content/70">
      <div v-if="fileList.length || selectedFile || selectionCount || emptyMessage" class="flex items-center gap-1 shrink-0">
        <IconList class="t-icon-size-xs" />
        <span>{{ emptyMessage || $t('statusbar.items_summary', { count: itemPosition, size: formatFileSize(totalFileSize) }) }}</span>
        <span v-if="selectionCount">{{ $t('statusbar.selection_summary', { count: selectionCount.toLocaleString(), size: formatFileSize(selectionSize) }) }}</span>
      </div>
      <div v-if="!selectionCount && hasRealSelectedFile" ref="fieldsContainer" class="flex flex-1 flex-nowrap items-center min-w-0 h-full overflow-hidden gap-x-4">
        <div
          v-for="(field, index) in displayedFields"
          :key="field.key"
          class="flex h-full items-center gap-1"
          :class="index === displayedFields.length - 1 ? 'flex-1 min-w-0' : 'flex-none'"
        >
          <component :is="field.icon" class="t-icon-size-xs shrink-0" aria-hidden="true" />
          <span class="truncate" :title="field.value">{{ field.value }}</span>
          <span v-if="field.key === 'name' && (isLivePhoto || rawPairBadge)" class="inline-flex shrink-0 items-center gap-0.5 rounded bg-base-content/10 px-1 text-[10px] font-medium">
            <IconLivePhoto v-if="isLivePhoto" class="size-3" aria-hidden="true" />
            {{ isLivePhoto ? $t('image_viewer.live') : rawPairBadge }}
          </span>
          <span v-if="field.key === 'name'" class="shrink-0">({{ formatFileSize(getMediaItemSize(currentFile)) }})</span>
        </div>
      </div>
      <div v-if="!selectionCount && hasRealSelectedFile" ref="measureContainer" class="pointer-events-none invisible absolute left-0 top-0 flex h-full w-max gap-x-4" aria-hidden="true">
        <div v-for="field in visibleFields" :key="`measure-${field.key}`" class="flex h-full items-center gap-1 whitespace-nowrap">
          <component :is="field.icon" class="t-icon-size-xs shrink-0" aria-hidden="true" />
          <span>{{ field.value }}</span>
          <span v-if="field.key === 'name' && (isLivePhoto || rawPairBadge)" class="inline-flex shrink-0 items-center gap-0.5 rounded bg-base-content/10 px-1 text-[10px] font-medium">
            <IconLivePhoto v-if="isLivePhoto" class="size-3" aria-hidden="true" />
            {{ isLivePhoto ? $t('image_viewer.live') : rawPairBadge }}
          </span>
          <span v-if="field.key === 'name'" class="shrink-0">({{ formatFileSize(getMediaItemSize(currentFile)) }})</span>
        </div>
      </div>
    </div>
    <ContextMenu ref="menu" :menu-items="menuItems"><template #trigger></template></ContextMenu>
    <StatusBarFieldsDialog v-if="showFieldsDialog" @cancel="showFieldsDialog = false" />
    <div
      v-if="showUpdateIcon"
      class="ml-2 max-w-[40%] shrink-0 px-1 py-1 flex items-center gap-1 rounded-full bg-base-300 text-primary text-[11px] font-mono"
    >
      <component :is="updateIconComponent" class="t-icon-size-xs shrink-0" :class="{ 'animate-spin': isUpdateAnimating }" />
      <span v-if="scanText" class="truncate text-right">{{ scanText }}</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import { config } from '@/common/config';
import { normalizeStatusBarFields } from '@/common/statusBarFields';
import ContextMenu from '@/components/ContextMenu.vue';
import StatusBarFieldsDialog from '@/components/StatusBarFieldsDialog.vue';
import {
  formatFileSize,
  formatRawPairBadge,
  getMediaItemSize,
  shortenFilename,
  formatDimensionText,
  formatTakenDate,
  formatCaptureSettings,
  formatCameraInfo,
  getFolderPath,
} from '@/common/utils';
import {
  IconAspectRatio,
  IconPhoto,
  IconVideo,
  IconList,
  IconLivePhoto,
  IconUpdate,
  IconUpdateDot,
  IconCalendarDay,
  IconCameraAperture,
  IconCamera,
  IconLocation,
  IconFile,
  IconFolder,
  IconFolders,
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
  selectionCount: { type: Number, default: 0 },
  selectionSize: { type: Number, default: 0 },
  totalFileCount: {
    type: Number,
    default: 0,
  },
  totalFileSize: {
    type: Number,
    default: 0,
  },
  selectedFile: {
    type: Object as () => any,
    default: null,
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

const { t } = useI18n();
const menu = ref<InstanceType<typeof ContextMenu> | null>(null);
const showFieldsDialog = ref(false);
const menuItems = computed(() => [
  { label: t('statusbar.customize'), action: () => { showFieldsDialog.value = true; } },
]);
const currentFile = computed(() => {
  if (props.selectedFile) return props.selectedFile;
  return props.fileList[props.selectedItemIndex];
});
const normalizedRotate = computed(() => ((Number(currentFile.value?.rotate || 0) % 360) + 360) % 360);
const itemPosition = computed(() => {
  const total = props.totalFileCount.toLocaleString();
  return props.selectionCount || props.selectedItemIndex < 0 ? total : `${(props.selectedItemIndex + 1).toLocaleString()} / ${total}`;
});
const isLivePhoto = computed(() =>
  currentFile.value?.media_subtype === 'live_photo' && !!currentFile.value?.live_photo_video_path
);
const rawPairBadge = computed(() => currentFile.value?.media_subtype === 'raw_jpeg_pair'
  ? formatRawPairBadge(currentFile.value.live_photo_video_path)
  : '');
const hasRealSelectedFile = computed(() => {
  const file = currentFile.value;
  return !!file && !file.isPlaceholder;
});
const visibleFields = computed(() => {
  const file = currentFile.value;
  if (!file) return [];
  const dimension = file.width && file.height ? formatDimensionText(file.width, file.height, true) : '';
  const fields = {
    name: { icon: file.file_type === 1 || file.file_type === 3 ? IconPhoto : IconVideo, value: shortenFilename(file.name, 32) },
    dimension: { icon: IconAspectRatio, value: dimension && (normalizedRotate.value ? `${dimension} · ${normalizedRotate.value}°` : dimension) },
    folder: { icon: IconFolder, value: getFolderPath(file.file_path) },
    album_name: { icon: IconFolders, value: file.album_name },
    colorInfo: {
      icon: IconFile,
      value: [file.icc_profile, file.color_space, file.bit_depth > 0 ? `${file.bit_depth} bit` : ''].filter(Boolean).join(' · '),
    },
    takenDate: { icon: IconCalendarDay, value: formatTakenDate(file) },
    exposure: { icon: IconCameraAperture, value: formatCaptureSettings(file.e_focal_length, file.e_exposure_time, file.e_f_number, file.e_iso_speed, file.e_exposure_bias) },
    camera: { icon: IconCamera, value: formatCameraInfo(file.e_make, file.e_model) },
    lens: { icon: IconCameraAperture, value: file.e_lens_model },
    location: { icon: IconLocation, value: file.geo_name || file.geo_admin2 || file.geo_admin1 || file.geo_cc },
  } as Record<string, { icon: typeof IconFile; value: string }>;
  return normalizeStatusBarFields(config.settings.statusBarFields)
    .filter(field => field.visible && fields[field.key].value)
    .map(({ key }) => ({ key, ...fields[key] }));
});
const containerClass = computed(() => {
  const base = 'px-2 h-8 flex items-center justify-between text-sm cursor-default bg-base-300';
  if (props.isEmbedded) return base;
  return `${base} absolute bottom-0 left-0 right-0 z-30`;
});
const fieldsContainer = ref<HTMLElement | null>(null);
const measureContainer = ref<HTMLElement | null>(null);
const fitCount = ref(0);
const displayedFields = computed(() => visibleFields.value.slice(0, fitCount.value));
const FIELD_GAP = 16;
const MIN_LAST_FIELD_WIDTH = 32;
let fieldsResizeObserver: ResizeObserver | null = null;
let fitScheduled = false;

function scheduleFit() {
  if (fitScheduled) return;
  fitScheduled = true;
  void nextTick(() => {
    fitScheduled = false;
    measureFit();
  });
}

function measureFit() {
  if (!fieldsContainer.value || !measureContainer.value || visibleFields.value.length === 0) {
    fitCount.value = 0;
    return;
  }
  const available = fieldsContainer.value.clientWidth;
  const widths = Array.from(measureContainer.value.children, child => child.getBoundingClientRect().width);
  if (!available) {
    fitCount.value = 0;
    return;
  }
  let count = widths.length;
  while (count > 0) {
    const before = widths.slice(0, count - 1).reduce((sum, width) => sum + width, 0);
    const gaps = FIELD_GAP * Math.max(0, count - 1);
    if (before + gaps + widths[count - 1] <= available) break;
    if (before + gaps + MIN_LAST_FIELD_WIDTH <= available) break;
    count--;
  }
  fitCount.value = count;
}

onMounted(() => {
  fieldsResizeObserver = new ResizeObserver(scheduleFit);
  if (fieldsContainer.value) fieldsResizeObserver.observe(fieldsContainer.value);
  scheduleFit();
});
onBeforeUnmount(() => {
  fieldsResizeObserver?.disconnect();
});
watch(fieldsContainer, (element) => {
  fieldsResizeObserver?.disconnect();
  if (element) fieldsResizeObserver?.observe(element);
  scheduleFit();
}, { flush: 'post' });
watch(visibleFields, scheduleFit, { deep: true });
</script>

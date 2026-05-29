<template>
  <div class="w-full h-full rounded-box bg-base-200 flex flex-col overflow-hidden">
    <!-- Header & Close -->
    <div class="flex items-center w-full shrink-0 px-2 mb-2">
      <div class="flex-1 pl-1">
        <span class="text-[11px] font-bold uppercase tracking-[0.22em] text-base-content/35">
          {{ $t('file_info.title') }}
        </span>
      </div>
      <div class="mt-2 flex items-center gap-1">
        <TButton
          :icon="IconRefresh"
          :tooltip="$t('menu.file.refresh_file_info')"
          :buttonSize="'small'"
          :disabled="!fileInfo"
          @click.stop="emit('refreshFileInfo')"
        />
        <TButton
          :icon="IconClose"
          :tooltip="$t('msgbox.close')"
          :buttonSize="'small'"
          @click.stop="$emit('close')"
        />
      </div>
    </div>

    <!-- Info Content -->
    <div v-if="fileInfo" class="mb-2 px-2 flex-1 overflow-y-auto overflow-x-hidden flex flex-col">

      <!-- Preview Section -->
      <div class="group/thumbnail border-t border-base-content/5 px-1 py-3 space-y-3">
        <div
          class="flex items-center gap-2 cursor-pointer text-base-content/70 hover:text-base-content transition-all duration-200 ease-in-out"
          @click.stop="togglePreview"
        >
          <component :is="fileInfo?.file_type === 2 ? IconVideo : IconPhoto" class="w-4 h-4" />
          <span class="font-bold mr-auto uppercase text-xs tracking-wide">{{ $t('file_info.preview') }}</span>
          <TButton
            :icon="showPreviewPanel ? IconArrowDown : IconArrowUp"
            :buttonSize="'small'"
          />
        </div>

        <Transition
          @before-enter="onBeforeEnter"
          @enter="onEnter"
          @after-enter="onAfterEnter"
          @leave="onLeave"
        >
          <div v-if="showPreviewPanel" class="overflow-hidden">
            <div class="mb-2 flex items-center gap-2">
              <div role="tablist" class="tabs tabs-xs flex-1">
                <button
                  role="tab"
                  :class="['tab', !isHistogramPreview ? 'tab-active text-primary' : '']"
                  @click.stop="setPreviewMode('thumbnail')"
                >
                  {{ $t('file_info.thumbnail') }}
                </button>
                <button
                  v-if="canShowHistogram"
                  role="tab"
                  :class="['tab', isHistogramPreview ? 'tab-active text-primary' : '']"
                  @click.stop="setPreviewMode('histogram')"
                >
                  {{ $t('file_info.histogram') }}
                </button>
              </div>
              <span
                v-if="previewFormatLabel"
                class="inline-flex shrink-0 items-center rounded-box border border-base-content/20 bg-base-300/80 px-1.5 text-[10px] font-bold uppercase tracking-wide text-base-content/80"
              >
                {{ previewFormatLabel }}
              </span>
            </div>

            <div
              v-if="!isHistogramPreview"
              class="relative w-full overflow-hidden rounded-box border border-base-content/5 shadow-sm transition-[padding-top] duration-200 ease-out"
              :style="{ paddingTop: `${75 * previewScale}%` }"
              @pointerleave="stopPreviewVideo"
            >
              <div
                v-if="!showVideoPreview"
                class="absolute top-2 left-2 flex bg-base-100/30 hover:bg-base-100/70 rounded-box z-10 cursor-pointer opacity-0 pointer-events-none transition-opacity duration-150 group-hover/thumbnail:opacity-100 group-hover/thumbnail:pointer-events-auto"
              >
                <TButton
                  :icon="IconZoomOut"
                  :tooltip="$t('map.zoom_out')"
                  :disabled="previewScale <= previewScaleOptions[previewScaleOptions.length - 1]"
                  @click.stop="decreasePreviewScale"
                />
                <TButton
                  :icon="IconZoomIn"
                  :tooltip="$t('map.zoom_in')"
                  :disabled="previewScale >= previewScaleOptions[0]"
                  @click.stop="increasePreviewScale"
                />
              </div>
              <div class="absolute inset-0">
                <img
                  v-if="fileInfo?.thumbnail"
                  :src="fileInfo.thumbnail"
                  class="h-full w-full object-contain"
                  :style="previewImageStyle"
                />
                <video
                  v-if="showVideoPreview"
                  ref="previewVideoRef"
                  class="pointer-events-none absolute inset-0 h-full w-full object-contain bg-black transition-opacity duration-100"
                  :class="isVideoPreviewReady ? 'opacity-100' : 'opacity-0'"
                  :style="previewImageStyle"
                  :poster="fileInfo?.thumbnail"
                  muted
                  autoplay
                  loop
                  playsinline
                  preload="metadata"
                  @canplay="isVideoPreviewReady = true"
                  @playing="isVideoPreviewReady = true"
                  @error="stopPreviewVideo"
                />
                <div v-if="!fileInfo?.thumbnail && !showVideoPreview" class="flex h-full w-full items-center justify-center bg-base-content/5">
                  <component
                    :is="fileInfo?.file_type === 2 ? IconVideo : IconPhoto"
                    class="w-10 h-10 text-base-content/20"
                  />
                </div>
              </div>
              <button
                v-if="isVideoFile && !showVideoPreview"
                type="button"
                class="absolute inset-0 z-10 flex items-center justify-center bg-black/10 text-base-content/70 opacity-0 pointer-events-none transition-all duration-150 group-hover/thumbnail:opacity-100 group-hover/thumbnail:pointer-events-auto hover:bg-black/20"
                @click.stop="playPreviewVideo"
              >
                <span class="flex h-12 w-12 items-center justify-center rounded-full bg-base-100/70 shadow-sm transition-transform hover:scale-105">
                  <IconVideoPlay class="h-7 w-7" />
                </span>
              </button>

              <!-- <div
                v-if="fileInfo?.is_favorite"
                class="absolute top-2 right-2 rounded-full bg-error/90 p-1 shadow-sm"
              >
                <IconHeartFilled class="w-3.5 h-3.5 text-white" />
              </div> -->
            </div>
            <div v-else class="rounded-box border border-base-content/5 bg-base-300/30 p-3 shadow-sm">
              <ImageHistogram :source="fileInfo?.thumbnail || ''" />
            </div>
          </div>
        </Transition>
      </div>

      <!-- File Info Section -->
      <div class="border-t border-base-content/5 px-1 py-4 space-y-3">

        <div class="flex items-center gap-2 cursor-pointer text-base-content/70 hover:text-base-content transition-all duration-200 ease-in-out" 
          @click.stop="toggleBasicInfo"
        >
          <IconFile class="w-4 h-4" />
          <span class="font-bold mr-auto uppercase text-xs tracking-wide">{{ $t('file_info.general') }}</span>
          <TButton
            :icon="showBasicInfoPanel ? IconArrowDown : IconArrowUp"
            :buttonSize="'small'"
          />
        </div>

        <Transition
          @before-enter="onBeforeEnter"
          @enter="onEnter"
          @after-enter="onAfterEnter"
          @leave="onLeave"
        >
          <div v-if="showBasicInfoPanel" class="grid grid-cols-[80px_1fr] gap-y-1 gap-x-4 text-xs overflow-hidden">
            <!-- Name -->
            <div class="flex items-center text-[10px] uppercase tracking-widest font-bold text-base-content/25 h-6">{{ $t('file_info.name') }}</div>
            <div class="group flex items-center gap-1">
              <div
                v-if="isRenaming"
                class="flex items-center w-full min-w-0"
              >
                <input
                  ref="renameInputRef"
                  v-model="renamingName"
                  class="font-bold text-xs text-base-content input input-xs input-bordered p-1 h-6 leading-6 w-full min-w-0"
                  @blur="finishRename"
                  @keydown.enter="finishRename"
                  @keydown.esc="cancelRename"
                  @click.stop
                />
                <span
                  v-if="renamingExt"
                  class="ml-1 text-xs font-semibold text-base-content/45 whitespace-nowrap"
                >.{{ renamingExt }}</span>
              </div>
              <span v-else 
                class="font-semibold text-xs text-base-content/65 break-all flex-1 min-w-0"
              >{{ fileInfo?.name }}</span>
              <TButton
                v-if="!isRenaming"
                :icon="IconEdit"
                :tooltip="$t('menu.file.rename')"
                :buttonSize="'small'"
                :class="['opacity-0 group-hover:opacity-100 transition-opacity duration-200 ease-in-out']"
                @click.stop="startRename"
              />
            </div>

            <!-- Album -->
            <div class="flex items-center text-[10px] uppercase tracking-widest font-bold text-base-content/25 h-6">{{ $t('file_info.album_name') }}</div>
            <div class="flex items-center text-xs font-semibold text-base-content/65 break-all">{{ fileInfo?.album_name }}</div>

            <!-- Path -->
            <div class="flex items-center text-[10px] uppercase tracking-widest font-bold text-base-content/25 h-6">{{ $t('file_info.folder') }}</div>
            <div class="flex items-center gap-1 min-w-0">
              <IconFolderExpanded class="w-3.5 h-3.5 shrink-0 text-base-content/65" />
              <div class="breadcrumbs p-0 min-h-0 overflow-hidden min-w-0">
                <ul class="min-w-0 flex-nowrap overflow-hidden">
                <li
                  v-for="(item, idx) in folderBreadcrumbs"
                  :key="`${item.path}-${idx}`"
                  class="min-w-0 max-w-full overflow-hidden"
                >
                  <a
                    v-if="idx < folderBreadcrumbs.length - 1"
                    class="block max-w-48 truncate cursor-pointer transition-colors text-xs font-semibold text-base-content/65 hover:text-base-content"
                    @click.stop="emit('navigateFolder', item.path)"
                  >{{ item.label }}</a>
                  <span
                    v-else
                    class="block max-w-48 truncate text-xs font-semibold text-base-content/65 cursor-pointer hover:text-base-content"
                    @click.stop="emit('navigateFolder', item.path)"
                  >{{ item.label }}</span>
                </li>
                </ul>
              </div>
            </div>

            <!-- Size -->
            <div class="flex items-center text-[10px] uppercase tracking-widest font-bold text-base-content/25 h-6">{{ $t('file_info.size') }}</div>
            <div class="flex items-center text-xs font-semibold text-base-content/65">{{ formatFileSize(fileInfo?.size) }}</div>

            <!-- Dimension -->
            <div class="flex items-center text-[10px] uppercase tracking-widest font-bold text-base-content/25 h-6">{{ $t('file_info.dimension') }}</div>
            <div class="flex items-center text-xs font-semibold text-base-content/65">{{ formatDimensionText(fileInfo?.width, fileInfo?.height) }}</div>

            <!-- Duration -->
            <template v-if="fileInfo?.file_type === 2">
              <div class="flex items-center text-[10px] uppercase tracking-widest font-bold text-base-content/25 h-6">{{ $t('file_info.duration') }}</div>
              <div class="flex items-center text-xs font-semibold text-base-content/65">{{ formatDuration(fileInfo?.duration) }}</div>
            </template>

            <!-- Created At -->
            <div class="flex items-center text-[10px] uppercase tracking-widest font-bold text-base-content/25 h-6">{{ $t('file_info.created_at') }}</div>
            <div class="flex items-center text-xs font-semibold text-base-content/65">{{ formatTimestamp(fileInfo?.created_at, $t('format.date_time')) }}</div>

            <!-- Modified At -->
            <div class="flex items-center text-[10px] uppercase tracking-widest font-bold text-base-content/25 h-6">{{ $t('file_info.modified_at') }}</div>
            <div class="flex items-center text-xs font-semibold text-base-content/65">{{ formatTimestamp(fileInfo?.modified_at, $t('format.date_time')) }}</div>

            <!-- Last Scan -->
            <template v-if="fileInfo?.last_scan_time && fileInfo.last_scan_time > 0">
              <div class="flex items-center text-[10px] uppercase tracking-widest font-bold text-base-content/25 h-6">{{ $t('file_info.last_scan_time') }}</div>
              <div class="flex items-center text-xs font-semibold text-base-content/65">{{ formatRelativeTime(fileInfo.last_scan_time / 1000, $t) }}</div>
            </template>

            <div class="flex items-center text-[10px] uppercase tracking-widest font-bold text-base-content/25 h-6">{{ $t('menu.meta.favorite') }}</div>
            <div class="h-6 flex items-center gap-0.5">
              <button
                class="btn btn-ghost btn-xs min-h-0 h-6 w-6 p-0 mr-1"
                :title="fileInfo?.is_favorite ? $t('menu.meta.unfavorite') : $t('menu.meta.favorite')"
                @click.stop="emit('toggleFavorite')"
              >
                <component
                  :is="fileInfo?.is_favorite ? IconHeartFilled : IconHeart"
                  class="w-3.5 h-3.5"
                  :class="fileInfo?.is_favorite ? 'text-error' : 'text-base-content/70'"
                />
              </button>
              <div class="w-px h-4 bg-base-content/10 mr-1"></div>
              <span class="mr-1 text-[11px] font-medium text-base-content/50">{{ $t('favorite.ratings') }}</span>
              <button
                v-for="rating in [1, 2, 3, 4, 5]"
                :key="rating"
                class="btn btn-ghost btn-xs min-h-0 h-6 w-6 p-0"
                :title="getRatingLabel(rating)"
                @click.stop="emit('setRating', (fileInfo?.rating || 0) === rating ? 0 : rating)"
              >
                <component
                  :is="(fileInfo?.rating || 0) >= rating ? IconStarFilled : IconStar"
                  class="w-3.5 h-3.5"
                  :class="(fileInfo?.rating || 0) >= rating ? 'text-warning' : 'text-base-content/30'"
                />
              </button>
            </div>

            <!-- Tags -->
            <div class="flex items-center text-[10px] uppercase tracking-widest font-bold text-base-content/25 min-h-6 py-1">{{ $t('file_info.tags') }}</div>
            <div class="group flex items-center min-h-6 gap-1">
              <div class="text-xs font-semibold text-base-content/65 flex flex-wrap gap-1 flex-1 min-w-0">
                <template v-if="fileInfo?.tags && fileInfo.tags.length">
                  <span
                    v-for="tag in fileInfo.tags"
                    :key="tag.id"
                    class="badge badge-sm badge-outline border-base-content/30 bg-base-content/30 font-medium"
                  >{{ tag.name }}</span>
                </template>
              </div>
              <TButton
                :icon="IconEdit"
                :tooltip="$t('menu.meta.tag')"
                :buttonSize="'small'"
                :class="['opacity-0 group-hover:opacity-100 transition-opacity duration-200 ease-in-out']"
                @click.stop="emit('quickEditTag')"
              />
            </div>

            <!-- Comment -->
            <div class="flex items-start text-[10px] uppercase tracking-widest font-bold text-base-content/25 py-1">{{ $t('file_info.comment') }}</div>
            <div class="group flex items-start gap-1">
              <div class="text-xs font-semibold text-base-content/65 wrap-break-words whitespace-pre-wrap flex-1 min-w-0">{{ fileInfo?.comments }}</div>
              <TButton
                :icon="IconEdit"
                :tooltip="$t('menu.meta.comment')"
                :buttonSize="'small'"
                :class="['opacity-0 group-hover:opacity-100 transition-opacity duration-200 ease-in-out']"
                @click.stop="emit('quickEditComment')"
              />
            </div>

            <!-- Rotate Display -->
            <template v-if="fileInfo?.rotate && fileInfo?.rotate !== 0">
              <div class="flex items-center text-[10px] uppercase tracking-widest font-bold text-base-content/25">{{ $t('menu.meta.rotate') }}</div>
              <div class="flex items-center gap-2 min-h-6">
                <span class="text-xs font-semibold text-base-content/65">{{ normalizedRotate }}°</span>
                <TButton
                  :icon="IconRotate"
                  :tooltip="$t('menu.meta.rotate')"
                  :buttonSize="'small'"
                  @click.stop="emit('rotate')"
                />
              </div>
            </template>
          </div>
        </Transition>
      </div>

      <!-- Metadata Section -->
      <div class="border-t border-base-content/5 px-1 py-4 space-y-3">

        <div class="flex items-center gap-2 cursor-pointer text-base-content/70 hover:text-base-content" @click.stop="toggleMetadata">
          <IconCameraAperture class="w-4 h-4 " /> 
          <span class="font-bold mr-auto uppercase text-xs tracking-wide">{{ $t('file_info.metadata') }}</span>
          <TButton
            :icon="showMetadataPanel ? IconArrowDown : IconArrowUp"
            :buttonSize="'small'"
          />
        </div>

        <Transition
          @before-enter="onBeforeEnter"
          @enter="onEnter"
          @after-enter="onAfterEnter"
          @leave="onLeave"
        >
          <div v-if="showMetadataPanel" class="grid grid-cols-[80px_1fr] gap-y-1 gap-x-4 text-xs overflow-hidden">
            <!-- Camera -->
            <div class="flex items-center text-[10px] uppercase tracking-widest font-bold text-base-content/25 h-6">{{ $t('file_info.camera') }}</div>
            <div class="flex items-center text-xs font-semibold text-base-content/65">{{ formatCameraInfo(fileInfo?.e_make, fileInfo?.e_model) }}</div>

            <!-- Lens -->
            <div class="flex items-center text-[10px] uppercase tracking-widest font-bold text-base-content/25 h-6">{{ $t('file_info.lens') }}</div>
            <div class="flex items-center text-xs font-semibold text-base-content/65">{{ fileInfo?.e_lens_model }}</div>

            <!-- Capture Settings -->
            <div class="flex items-center text-[10px] uppercase tracking-widest font-bold text-base-content/25 h-6">{{ $t('file_info.capture_settings') }}</div>
            <div class="flex items-center text-xs font-semibold text-base-content/65">{{ formatCaptureSettings(fileInfo?.e_focal_length, fileInfo?.e_exposure_time, fileInfo?.e_f_number, fileInfo?.e_iso_speed, fileInfo?.e_exposure_bias) }}</div>

            <!-- Software -->
            <div class="flex items-center text-[10px] uppercase tracking-widest font-bold text-base-content/25 h-6">{{ $t('file_info.software') }}</div>
            <div class="flex items-center text-xs font-semibold text-base-content/65">{{ fileInfo?.e_software }}</div>

            <!-- Taken By -->
            <div class="flex items-center text-[10px] uppercase tracking-widest font-bold text-base-content/25 h-6">{{ $t('file_info.taken_by') }}</div>
            <div class="flex items-center text-xs font-semibold text-base-content/65">{{ fileInfo?.e_artist }}</div>

            <!-- Copyright -->
            <div class="flex items-center text-[10px] uppercase tracking-widest font-bold text-base-content/25 h-6">{{ $t('file_info.copyright') }}</div>
            <div class="flex items-center text-xs font-semibold text-base-content/65">{{ fileInfo?.e_copyright }}</div>

            <!-- Taken At -->
            <div class="flex items-center text-[10px] uppercase tracking-widest font-bold text-base-content/25 h-6">{{ $t('file_info.taken_at') }}</div>
            <div class="flex items-center text-xs font-semibold text-base-content/65">{{ fileInfo?.e_date_time }}</div>

            <!-- Description -->
            <div class="flex items-start text-[10px] uppercase tracking-widest font-bold text-base-content/25 py-1">{{ $t('file_info.description') }}</div>
            <div class="flex items-center text-xs font-semibold text-base-content/65 wrap-break-words py-1">{{ fileInfo?.e_description }}</div>

            <!-- Geo Location -->
            <div class="flex items-center text-[10px] uppercase tracking-widest font-bold text-base-content/25 h-6">{{ $t('file_info.geo_location') }}</div>
            <div class="flex items-center text-xs font-semibold text-base-content/65">{{ formatGeoLocation() }}</div>
          </div>
        </Transition>
      </div>

      <!-- Map View -->
      <div v-if="fileInfo?.gps_latitude && fileInfo?.gps_longitude" 
        class="border-t border-base-content/5 px-1 py-4 space-y-3 flex flex-col transition-[flex-grow]" 
        :class="{ 'flex-1 min-h-[300px] shrink-0': showMapPanel }">
        <div class="flex items-center gap-2 cursor-pointer text-base-content/70 hover:text-base-content shrink-0" @click.stop="toggleMapPanel">
          <IconLocation class="w-4 h-4 " /> 
          <span class="font-bold mr-auto uppercase text-xs tracking-wide">{{ $t('file_info.map') }}</span>
          <TButton
            :icon="showMapPanel ? IconArrowDown : IconArrowUp"
            :buttonSize="'small'"
          />
        </div>

        <Transition
          @before-enter="onBeforeEnter"
          @enter="onEnter"
          @after-enter="onAfterEnter"
          @leave="onLeave"
        >
          <div v-if="showMapPanel" class="flex-1 flex flex-col min-h-0">
            <div class="w-full rounded-box relative z-0 flex-1 min-h-0 border border-base-content/5">
              <MapView
                :lat="fileInfo.gps_latitude ? Number(fileInfo.gps_latitude) : 0"
                :lon="fileInfo.gps_longitude ? Number(fileInfo.gps_longitude) : 0"
                :label="fileInfo.geo_name || fileInfo.name || 'Lap'"
              />
            </div>
          </div>
        </Transition>
      </div>
    </div>

    <div v-else class="mb-2 px-2 flex-1 overflow-y-auto overflow-x-hidden flex flex-col">
      <div class="p-4 flex-1 flex items-center justify-center">
        <div class="text-center text-base-content/40 space-y-3 max-w-[260px]">
          <IconFile class="w-8 h-8 mx-auto text-base-content/30" />
          <p class="text-xs font-medium">{{ $t('file_info.empty_title') }}</p>
          <p class="text-xs text-base-content/40">{{ $t('file_info.empty_desc') }}</p>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, nextTick, computed, watch, onBeforeUnmount } from 'vue';
import { useI18n } from 'vue-i18n';
import { useToast } from '@/common/toast';
import { useUIStore } from '@/stores/uiStore';
import { config } from '@/common/config';
import { renameFile, editImage, getAlbum } from '@/common/api';
import { 
  extractFileName, 
  getFileExtension,
  getFolderPath, 
  getFolderName,
  buildFolderBreadcrumbs,
  formatDimensionText, 
  formatFileSize, 
  formatTimestamp,
  formatRelativeTime,
  formatDuration,
  formatCaptureSettings,
  formatCameraInfo,
  getCountryName,
  combineFileName,
  isValidFileName,
  getAssetSrc
} from '@/common/utils';
import { 
  IconClose, IconLocation, IconArrowDown, IconArrowUp, IconCameraAperture, 
  IconFile, IconFolderSearch, IconHeart, IconHeartFilled, IconStar, IconStarFilled, IconEdit,
  IconFolderExpanded,
  IconPhoto,
  IconRefresh,
  IconRotate,
  IconVideo,
  IconVideoPlay,
  IconZoomIn,
  IconZoomOut,
} from '@/common/icons';
import TButton from '@/components/TButton.vue';
import ImageHistogram from '@/components/ImageHistogram.vue';
import MapView from '@/components/MapView.vue';

const props = defineProps({
  fileInfo: {
    type: Object,
    required: false
  },
});

const { locale, messages } = useI18n();
const localeMsg = computed(() => messages.value[locale.value] as any);
const uiStore = useUIStore();


const emit = defineEmits([
  'close',
  'success',
  'toggleFavorite',
  'setRating',
  'rotate',
  'quickEditTag',
  'quickEditComment',
  'navigateFolder',
  'refreshFileInfo',
]);

const toast = useToast();
const showPreviewPanel = computed(() => config.infoPanel.showPreview);
const previewScaleOptions = [1, 0.75, 0.5, 0.25];
const previewScale = computed({
  get: () => Number(config.infoPanel.previewScale || 1),
  set: (value: number | string) => {
    const numeric = Number(value);
    config.infoPanel.previewScale = previewScaleOptions.includes(numeric) ? numeric : 1;
  },
});
const showBasicInfoPanel = computed(() => config.infoPanel.showBasicInfo);
const showMetadataPanel = computed(() => config.infoPanel.showMetadata);
const showMapPanel = computed(() => config.infoPanel.showMap);
const isVideoFile = computed(() => Number(props.fileInfo?.file_type || 0) === 2);
const canShowHistogram = computed(() => !isVideoFile.value);
const activePreviewMode = computed(() => canShowHistogram.value ? config.infoPanel.previewMode : 'thumbnail');
const isHistogramPreview = computed(() => activePreviewMode.value === 'histogram');
const previewVideoRef = ref<HTMLVideoElement | null>(null);
const showVideoPreview = ref(false);
const isVideoPreviewReady = ref(false);
const normalizedRotate = computed(() => {
  const rotate = Number(props.fileInfo?.rotate || 0) % 360;
  return rotate < 0 ? rotate + 360 : rotate;
});
const previewImageStyle = computed(() => {
  const rotate = normalizedRotate.value;
  const isQuarterTurn = rotate % 180 !== 0;
  return {
    transform: `rotate(${rotate}deg) scale(${isQuarterTurn ? 0.84 : 1})`,
    transformOrigin: 'center center',
  };
});
const previewFormatLabel = computed(() => {
  const formatLabel = (props.fileInfo?.format_label || '').trim();
  if (formatLabel) {
    return formatLabel.toUpperCase();
  }

  const name = props.fileInfo?.name || '';
  const filePath = props.fileInfo?.file_path || '';
  const extension = getFileExtension(name || filePath).trim();
  if (!extension) return '';
  if (Number(props.fileInfo?.file_type || 0) === 3) return 'RAW';
  return extension.toUpperCase();
});
function togglePreview() {
  config.infoPanel.showPreview = !config.infoPanel.showPreview;
}

function setPreviewMode(mode: 'thumbnail' | 'histogram') {
  if (mode === 'histogram' && !canShowHistogram.value) return;
  config.infoPanel.previewMode = mode;
}

async function playPreviewVideo() {
  if (!isVideoFile.value || !props.fileInfo?.file_path || showVideoPreview.value) return;
  isVideoPreviewReady.value = false;
  showVideoPreview.value = true;
  await nextTick();

  const video = previewVideoRef.value;
  if (!video) return;

  video.src = getAssetSrc(props.fileInfo.file_path);
  video.muted = true;

  try {
    await video.play();
  } catch {
    stopPreviewVideo();
  }
}

function stopPreviewVideo() {
  const video = previewVideoRef.value;
  if (video) {
    video.pause();
    video.removeAttribute('src');
    video.load();
  }

  isVideoPreviewReady.value = false;
  showVideoPreview.value = false;
}

watch(
  () => [props.fileInfo?.id, props.fileInfo?.file_path, showPreviewPanel.value, isHistogramPreview.value],
  stopPreviewVideo
);

onBeforeUnmount(stopPreviewVideo);

function increasePreviewScale() {
  const index = previewScaleOptions.indexOf(previewScale.value);
  if (index > 0) {
    previewScale.value = previewScaleOptions[index - 1];
  }
}

function decreasePreviewScale() {
  const index = previewScaleOptions.indexOf(previewScale.value);
  if (index >= 0 && index < previewScaleOptions.length - 1) {
    previewScale.value = previewScaleOptions[index + 1];
  }
}

function toggleBasicInfo() {
  config.infoPanel.showBasicInfo = !config.infoPanel.showBasicInfo;
}

function toggleMetadata() {
  config.infoPanel.showMetadata = !config.infoPanel.showMetadata;
}

function toggleMapPanel() {
  config.infoPanel.showMap = !config.infoPanel.showMap;
}

function getRatingLabel(rating: number) {
  const keys: Record<number, string> = {
    5: 'five_stars',
    4: 'four_stars',
    3: 'three_stars',
    2: 'two_stars',
    1: 'one_star',
  };
  const key = keys[rating];
  return localeMsg.value.favorite?.[key] || `${rating}★`;
}

const quickSave = async (): Promise<boolean> => {
  if (!props.fileInfo) return false;
  if (uiStore.activeAdjustments.filePath !== props.fileInfo.file_path) return true;

  const adj = uiStore.activeAdjustments as any;
  const ext = getFileExtension(props.fileInfo.name).toLowerCase();
  const outputFormat = (ext === 'jpg' || ext === 'jpeg') ? 'jpg' : ext;

  const editParams = {
    sourceFilePath: props.fileInfo.file_path,
    destFilePath: props.fileInfo.file_path,
    outputFormat,
    quality: 80,
    orientation: props.fileInfo.e_orientation || 1,
    flipHorizontal: false,
    flipVertical: false,
    rotate: 0,
    crop: { x: 0, y: 0, width: 0, height: 0 },
    resize: {
      width: adj.resize?.width ?? props.fileInfo.width,
      height: adj.resize?.height ?? props.fileInfo.height,
    },
    filter: adj.filter || null,
    brightness: adj.brightness ? adj.brightness : null,
    contrast: adj.contrast ? adj.contrast : null,
    blur: adj.blur ? adj.blur : null,
    hue_rotate: adj.hue ? adj.hue : null,
    saturation: adj.saturation !== 100 ? adj.saturation / 100.0 : null,
  };

  try {
    const success = await editImage(editParams);
    if (!success) {
      toast.error(localeMsg.value.tooltip.save_image.failed);
      return false;
    }

    uiStore.updateFileVersion(props.fileInfo.file_path);
    uiStore.clearActiveAdjustments();
    emit('success');
    toast.success(localeMsg.value.tooltip.save_image.success);
    return true;
  } catch {
    toast.error(localeMsg.value.tooltip.save_image.failed);
    return false;
  }
};

// Rename logic
const isRenaming = ref(false);
const renamingName = ref('');
const renamingExt = ref('');
const renameInputRef = ref<HTMLInputElement | null>(null);
const albumRootPath = ref('');
let albumRootRequestSeq = 0;

const folderBreadcrumbs = computed(() => {
  const folderPath = getFolderPath(props.fileInfo?.file_path);
  if (!folderPath) return [];
  return buildFolderBreadcrumbs(folderPath, albumRootPath.value, props.fileInfo?.album_name || getFolderName(albumRootPath.value));
});

watch(
  () => props.fileInfo?.album_id,
  async (albumId) => {
    const requestSeq = ++albumRootRequestSeq;
    albumRootPath.value = '';
    if (!albumId) return;
    const album = await getAlbum(albumId);
    if (requestSeq !== albumRootRequestSeq) return;
    if (props.fileInfo?.album_id !== albumId) return;
    albumRootPath.value = album?.path || '';
  },
  { immediate: true }
);

const startRename = () => {
  if (!props.fileInfo) return;
  
  const { name, ext } = extractFileName(props.fileInfo.name);
  renamingName.value = name;
  renamingExt.value = ext;
  isRenaming.value = true;
  uiStore.pushInputHandler('FileInfo-rename');
  
  nextTick(() => {
    if (renameInputRef.value) {
      renameInputRef.value.focus();
      renameInputRef.value.select();
    }
  });
};

const cancelRename = () => {
  isRenaming.value = false;
  renamingExt.value = '';
  uiStore.removeInputHandler('FileInfo-rename');
};

const finishRename = async () => {
  if (!isRenaming.value || !props.fileInfo) return;

  const newName = renamingName.value.trim();
  const { ext } = extractFileName(props.fileInfo.name);
  
  // Validation
  if (!newName || !isValidFileName(newName)) {
    // Optionally show error toast
    console.warn('Invalid filename');
    cancelRename();
    return;
  }

  const fullNewName = combineFileName(newName, ext);
  
  // If no change, just cancel
  if (fullNewName === props.fileInfo.name) {
    cancelRename();
    return;
  }

  // Call API
  const newPath = await renameFile(props.fileInfo.id, props.fileInfo.file_path, fullNewName);
  
  if (newPath) {
    // Update local props to reflect change immediately (assuming parent passes object ref)
    props.fileInfo.name = fullNewName;
    props.fileInfo.file_path = newPath;
  } else {
    // Optionally show error
    console.error('Rename failed');
  }

  cancelRename();
};

function formatGeoLocation() {
  const info = props.fileInfo;
  if (!info) return "";

  const fields = [
    info.geo_name,
    info.geo_admin2,
    info.geo_admin1,
    info.geo_cc ? getCountryName(info.geo_cc, locale.value) : info.geo_cc,
  ];

  return fields.filter(Boolean).join(", ");
}

const onBeforeEnter = (el: any) => {
  el.style.opacity = '0';
  el.style.height = '0';
}

const onEnter = (el: any) => {
  el.style.transition = 'all 0.1s ease';
  // Check scrollHeight to know final height
  el.style.height = el.scrollHeight + 'px';
  el.style.opacity = '1';
}

const onAfterEnter = (el: any) => {
  el.style.height = '';
}

const onLeave = (el: any) => {
  el.style.transition = 'all 0.1s ease';
  // Force height back to explicit pixel value for animation
  el.style.height = el.scrollHeight + 'px';
  // Force repaint to ensure transition triggers
  // eslint-disable-next-line no-unused-expressions
  el.offsetHeight; 
  el.style.height = '0';
  el.style.opacity = '0';
}
defineExpose({
  quickSave
});
</script>

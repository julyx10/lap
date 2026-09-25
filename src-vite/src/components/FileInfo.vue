<template>
  <div class="w-full h-full rounded-box bg-base-200 flex flex-col overflow-hidden">
    <!-- Header & Close -->
    <div class="my-2 px-2 flex items-center w-full shrink-0">
      <div class="flex-1 pl-1">
        <span class="text-sm font-semibold text-primary/70">
          {{ $t('file_info.title') }}
        </span>
      </div>
      <div class="flex items-center gap-1">
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
          class="flex items-center gap-1 cursor-pointer text-base-content/70 hover:text-base-content transition-all duration-200 ease-in-out"
          @click.stop="togglePreview"
        >
          <IconRight
            class="w-3 h-3 transition-transform duration-200"
            :class="{ 'rotate-90': showPreviewPanel }"
            @click.stop="togglePreview"
          />
          <span class="py-1 font-bold mr-auto uppercase text-xs tracking-wide text-base-content/30">{{ $t('file_info.preview') }}</span>
          <div v-if="showPreviewPanel" role="tablist" class="tabs tabs-xs shrink-0">
            <button
              role="tab"
              :class="['tab', !isHistogramPreview ? 'tab-active text-primary' : '']"
              @click.stop="setPreviewMode('thumbnail')"
            >{{ $t('file_info.thumbnail') }}</button>
            <button
              v-if="canShowHistogram"
              role="tab"
              :class="['tab', isHistogramPreview ? 'tab-active text-primary' : '']"
              @click.stop="setPreviewMode('histogram')"
            >{{ $t('file_info.histogram') }}</button>
          </div>
          <!-- <span
            v-if="showPreviewPanel && previewTagLabel"
            class="badge badge-xs flex items-center gap-0.5 bg-base-100/30 text-base-content/70"
          >
            <IconLivePhoto v-if="isLivePhoto && !isHistogramPreview" class="h-3 w-3 shrink-0" />
            {{ previewTagLabel }}
          </span> -->
        </div>

        <Transition
          @before-enter="onBeforeEnter"
          @enter="onEnter"
          @after-enter="onAfterEnter"
          @leave="onLeave"
        >
          <div v-if="showPreviewPanel" class="overflow-hidden">
            <div
              v-if="!isHistogramPreview"
              class="relative w-full overflow-hidden rounded-box border border-base-content/5 shadow-sm transition-[padding-top] duration-200 ease-out"
              :style="{ paddingTop: `${75 * previewScale}%` }"
              @pointerenter="playVideoPreviewOnHover"
              @pointerleave="stopPreviewVideo"
            >
              <div
                class="absolute top-2 left-2 flex bg-base-100/30 hover:bg-base-100/70 rounded-box z-20 cursor-pointer opacity-0 pointer-events-none transition-opacity duration-150 group-hover/thumbnail:opacity-100 group-hover/thumbnail:pointer-events-auto"
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
              <div
                class="absolute inset-0"
                :class="readonly ? 'cursor-default' : 'cursor-pointer'"
                @click.stop="!readonly && $emit('openViewer')"
              >
                <img
                  v-if="fileInfo?.thumbnail"
                  :src="fileInfo.thumbnail"
                  class="h-full w-full object-contain"
                  :style="previewImageStyle"
                />
                <video
                  v-if="showVideoPreview"
                  ref="previewVideoRef"
                  class="pointer-events-none absolute inset-0 h-full w-full object-contain"
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
                ></video> 
                <div v-if="!fileInfo?.thumbnail && !showVideoPreview" class="flex h-full w-full items-center justify-center bg-base-content/5">
                  <component
                    :is="fileInfo?.file_type === 2 ? IconVideo : IconPhoto"
                    class="w-10 h-10 text-base-content/30"
                  />
                </div>
              </div>
            </div>
            <div v-else class="rounded-box border border-base-content/5 bg-base-300/30 p-3 shadow-sm">
              <ImageHistogram :source="fileInfo?.thumbnail || ''" />
            </div>
          </div>
        </Transition>
      </div>

      <!-- File Info Section -->
      <div class="group/general border-t border-base-content/5 px-1 py-3 space-y-3">

        <div class="flex items-center gap-1 cursor-pointer text-base-content/70 hover:text-base-content transition-all duration-200 ease-in-out" 
          @click.stop="toggleBasicInfo"
        >
          <IconRight
            class="w-3 h-3 transition-transform duration-200"
            :class="{ 'rotate-90': showBasicInfoPanel }"
            @click.stop="toggleBasicInfo"
          />
          <span class="py-1 font-bold mr-auto uppercase text-xs tracking-wide text-base-content/30">{{ $t('file_info.general') }}</span>
          <div v-if="showBasicInfoPanel && isRawJpegPair" role="tablist" class="tabs tabs-xs shrink-0">
            <button
              role="tab"
              :class="['tab', generalInfoTab === 'raw' ? 'tab-active text-primary' : '']"
              @click.stop="generalInfoTab = 'raw'"
            >RAW</button>
            <button
              role="tab"
              :class="['tab', generalInfoTab === 'companion' ? 'tab-active text-primary' : '']"
              :disabled="!rawJpegCompanion"
              @click.stop="generalInfoTab = 'companion'"
            >{{ rawJpegCompanionLabel }}</button>
          </div>
        </div>

        <Transition
          @before-enter="onBeforeEnter"
          @enter="onEnter"
          @after-enter="onAfterEnter"
          @leave="onLeave"
        >
          <div v-if="showBasicInfoPanel" class="overflow-hidden">
            <div class="pl-4 grid grid-cols-[84px_minmax(0,1fr)] gap-y-1.5 gap-x-4 text-xs">
            <!-- Name -->
            <div class="flex items-center text-[11px] text-base-content/45 h-6">{{ $t('file_info.name') }}</div>
            <div class="group/field flex items-center gap-1">
              <div
                v-if="isPrimaryGeneralInfo && isRenaming"
                class="flex items-center w-full min-w-0"
              >
                <input
                  ref="renameInputRef"
                  v-model="renamingName"
                  class="text-[12px] text-base-content input input-xs input-bordered p-1 h-6 leading-6 w-full min-w-0"
                  @blur="finishRename"
                  @keydown.enter="finishRename"
                  @keydown.esc="cancelRename"
                  @click.stop
                />
                <span
                  v-if="renamingExt"
                  class="ml-1 text-[12px] text-base-content/70 whitespace-nowrap"
                >.{{ renamingExt }}</span>
              </div>
              <span v-else
                class="text-[12px] font-medium text-base-content/80 break-all flex-1 min-w-0"
                :class="{ 'cursor-text': isPrimaryGeneralInfo && !readonly }"
                @dblclick.stop="isPrimaryGeneralInfo && !readonly && startRename()"
              >{{ generalFileInfo?.name }}</span>
            </div>

            <!-- Path -->
            <div class="flex items-center text-[11px] text-base-content/45 h-6">{{ $t('file_info.folder') }}</div>
            <Breadcrumb
              :icon="IconFolder"
              :items="generalFolderBreadcrumbs"
              :disabled="readonly"
              size="small"
              @navigate="(path: string) => emit('navigateFolder', path)"
            />

            <!-- Album -->
            <div class="flex items-center text-[11px] text-base-content/45 h-6">{{ $t('file_info.album_name') }}</div>
            <div class="flex items-center min-w-0 gap-1.5">
              <span class="w-5 h-5 rounded-full overflow-hidden bg-base-300/70 ring-1 ring-base-content/5 shrink-0 flex items-center justify-center">
                <img v-if="albumCoverUrl" :src="albumCoverUrl" class="w-full h-full object-cover" />
                <IconFolder v-else class="w-3.5 h-3.5 text-base-content/30" />
              </span>
              <span class="min-w-0 text-[12px] font-medium text-base-content/80 break-all">{{ generalFileInfo?.album_name }}</span>
            </div>

            <!-- Size -->
            <div class="flex items-center text-[11px] text-base-content/45 h-6">{{ $t('file_info.size') }}</div>
            <div class="flex items-center text-[12px] text-base-content/75">{{ formatFileSize(generalFileInfo?.size) }}</div>

            <!-- Dimension -->
            <div class="flex items-center text-[11px] text-base-content/45 h-6">{{ $t('file_info.dimension') }}</div>
            <div class="flex items-center text-[12px] text-base-content/75">{{ formatDimensionText(generalFileInfo?.width, generalFileInfo?.height, true) }}</div>

            <!-- Duration -->
            <template v-if="generalFileInfo?.file_type === 2">
              <div class="flex items-center text-[11px] text-base-content/45 h-6">{{ $t('file_info.duration') }}</div>
              <div class="flex items-center text-[12px] text-base-content/75">{{ formatDuration(generalFileInfo?.duration) }}</div>
            </template>

            <div class="col-span-2">
              <div class="grid grid-cols-[84px_1fr] gap-y-1.5 gap-x-4">
                <!-- Created At -->
                <div class="flex items-center text-[11px] text-base-content/45 h-6">{{ $t('file_info.created_at') }}</div>
                <div class="flex items-center text-[12px] text-base-content/75">{{ formatTimestamp(generalFileInfo?.created_at, $t('format.date_time')) }}</div>

                <!-- Modified At -->
                <div class="flex items-center text-[11px] text-base-content/45 h-6">{{ $t('file_info.modified_at') }}</div>
                <div class="flex items-center text-[12px] text-base-content/75">{{ formatTimestamp(generalFileInfo?.modified_at, $t('format.date_time')) }}</div>

                <!-- Last Scan -->
                <template v-if="generalFileInfo?.last_scan_time && generalFileInfo.last_scan_time > 0">
                  <div class="flex items-center text-[11px] text-base-content/45 h-6">{{ $t('file_info.last_scan_time') }}</div>
                  <div class="flex min-h-6 items-center gap-2">
                    <!-- <span class="text-[12px] text-base-content/75">{{ formatTimestamp(fileInfo.last_scan_time / 1000, $t('format.date_time')) }}</span> -->
                    <span class="text-[11px] text-base-content/40">{{ formatRelativeTime(generalFileInfo.last_scan_time / 1000, $t) }}</span>
                  </div>
                </template>
              </div>
            </div>

            <template v-if="isPrimaryGeneralInfo">
            <div class="flex items-center text-[11px] text-base-content/45 h-6">{{ $t('file_info.marks') }}</div>
            <FavoriteRatingControl
              :favorite="Boolean(fileInfo?.is_favorite)"
              :rating="Number(fileInfo?.rating || 0)"
              :culling="Number(fileInfo?.culling_flag ?? fileInfo?.cullingFlag ?? 0)"
              :disabled="readonly"
              label-class="text-base-content/30"
              inactive-rating-class="text-base-content/70"
              @favorite="emit('toggleFavorite')"
              @rating="(rating) => emit('setRating', rating)"
              @culling="(cullingFlag) => emit('setCulling', cullingFlag)"
            />

            <!-- Tags -->
            <template v-if="displayTags.length">
              <div class="flex items-center text-[11px] text-base-content/45 min-h-6 py-1.5">{{ $t('file_info.tags') }}</div>
              <div class="group/field flex items-center min-h-6 gap-1">
                <div
                  class="text-[12px] text-base-content/75 flex flex-wrap gap-1 flex-1 min-w-0"
                  :class="{ 'cursor-pointer': !readonly }"
                  @click.stop="!readonly && emit('quickEditTag')"
                >
                  <span
                    v-for="tag in displayTags"
                    :key="tag.id"
                    :title="tag.group_name"
                    class="badge badge-sm badge-outline border-base-content/20 bg-base-content/5 font-medium text-base-content/75"
                  >{{ tag.name }}</span>
                </div>
              </div>
            </template>

            <!-- Collections -->
            <template v-if="fileCollections.length">
              <div class="flex items-center text-[11px] text-base-content/45 min-h-6 py-1.5">{{ $t('collection.title') }}</div>
              <div class="group/field flex items-center min-h-6 gap-1">
                <div class="flex items-center min-h-6 gap-x-3 gap-y-1 flex-wrap flex-1 min-w-0">
                  <button
                    v-for="collection in fileCollections"
                    :key="collection.id"
                    type="button"
                    class="inline-flex items-center gap-1 text-[12px] font-medium text-base-content/70 transition-colors"
                    :class="readonly ? 'cursor-default' : 'hover:text-base-content cursor-pointer'"
                    :disabled="readonly"
                    @click.stop="!readonly && emit('quickEditCollection')"
                  >
                    <IconBookmark class="h-3.5 w-3.5 shrink-0" />
                    {{ collection.name }}
                  </button>
                </div>
              </div>
            </template>

            <!-- Comment -->
            <template v-if="fileInfo?.comments">
              <div class="flex items-start text-[11px] text-base-content/45 py-1.5">{{ $t('file_info.comment') }}</div>
              <div class="group/field flex items-start gap-1">
                <div
                  class="text-[12px] leading-5 text-base-content/75 wrap-break-words whitespace-pre-wrap flex-1 min-w-0"
                  :class="{ 'cursor-pointer': !readonly }"
                  @click.stop="!readonly && emit('quickEditComment')"
                >{{ fileInfo?.comments }}</div>
              </div>
            </template>

            <!-- Rotate Display -->
            <template v-if="fileInfo?.rotate && fileInfo?.rotate !== 0">
              <div class="flex items-center text-[11px] text-base-content/45 h-6">{{ $t('menu.meta.rotate') }}</div>
              <div class="flex items-center gap-2 min-h-6">
                <span class="text-[12px] text-base-content/75">{{ normalizedRotate }}°</span>
                <TButton
                  :icon="IconRotate"
                  :tooltip="$t('menu.meta.rotate')"
                  :buttonSize="'small'"
                  :disabled="readonly"
                  @click.stop="!readonly && emit('rotate')"
                />
              </div>
            </template>

            <!-- People -->
            <template v-if="config.settings.face.enabled && filePersons.length">
              <div class="flex items-center text-[11px] text-base-content/45 min-h-6 py-1.5">{{ $t('sidebar.people') }}</div>
              <div class="flex items-center min-h-6 gap-x-3 gap-y-1 flex-wrap">
                <button
                  v-for="person in filePersons"
                  :key="person.id"
                  type="button"
                  class="inline-flex items-center gap-1.5 text-[12px] font-medium text-base-content/70 transition-colors"
                  :class="readonly ? 'cursor-default' : 'hover:text-primary cursor-pointer'"
                  :disabled="readonly"
                  @click.stop="navigatePerson(person)"
                >
                  <span class="w-5 h-5 rounded-full overflow-hidden bg-base-300/70 ring-1 ring-base-content/5 shrink-0 flex items-center justify-center">
                    <img
                      v-if="person.thumbnail"
                      :src="'data:image/jpeg;base64,' + person.thumbnail"
                      class="w-full h-full object-cover"
                    />
                    <IconPerson v-else class="w-3.5 h-3.5 text-base-content/30" />
                  </span>
                  {{ person.name || person.id }}
                </button>
              </div>
            </template>
            </template>
            
            </div>
          </div>
        </Transition>
      </div>

      <!-- Metadata Section -->
      <div class="border-t border-base-content/5 px-1 py-4 space-y-3">

        <div class="flex items-center gap-1 cursor-pointer text-base-content/70 hover:text-base-content" @click.stop="toggleMetadata">
          <IconRight
            class="w-3 h-3 transition-transform duration-200"
            :class="{ 'rotate-90': showMetadataPanel }"
            @click.stop="toggleMetadata"
          />
          <span class="font-bold mr-auto uppercase text-xs tracking-wide text-base-content/30">{{ $t('file_info.metadata') }}</span>
        </div>

        <Transition
          @before-enter="onBeforeEnter"
          @enter="onEnter"
          @after-enter="onAfterEnter"
          @leave="onLeave"
        >
          <div v-if="showMetadataPanel" class="pl-4 grid grid-cols-[84px_1fr] gap-y-1.5 gap-x-4 text-xs overflow-hidden">
            <!-- Camera -->
            <div class="flex items-center text-[11px] text-base-content/45 h-6">{{ $t('file_info.camera') }}</div>
            <div
              :class="['flex items-center text-[12px] text-base-content/75', hasCamera && !readonly ? 'cursor-pointer hover:text-primary' : '']"
              @click.stop="navigateCamera"
            >{{ formatCameraInfo(fileInfo?.e_make, fileInfo?.e_model) }}</div>

            <!-- Lens -->
            <div class="flex items-center text-[11px] text-base-content/45 h-6">{{ $t('file_info.lens') }}</div>
            <div
              :class="['flex items-center text-[12px] text-base-content/75', hasLens && !readonly ? 'cursor-pointer hover:text-primary' : '']"
              @click.stop="navigateLens"
            >{{ fileInfo?.e_lens_model }}</div>

            <!-- Capture Settings -->
            <div class="flex items-center text-[11px] text-base-content/45 h-6">{{ $t('file_info.capture_settings') }}</div>
            <div class="flex items-center text-[12px] text-base-content/75">{{ formatCaptureSettings(fileInfo?.e_focal_length, fileInfo?.e_exposure_time, fileInfo?.e_f_number, fileInfo?.e_iso_speed, fileInfo?.e_exposure_bias) }}</div>

            <!-- Software -->
            <div class="flex items-center text-[11px] text-base-content/45 h-6">{{ $t('file_info.software') }}</div>
            <div class="flex items-center text-[12px] text-base-content/75">{{ fileInfo?.e_software }}</div>

            <!-- Taken By -->
            <div class="flex items-center text-[11px] text-base-content/45 h-6">{{ $t('file_info.taken_by') }}</div>
            <div class="flex items-center text-[12px] text-base-content/75">{{ fileInfo?.e_artist }}</div>

            <!-- Copyright -->
            <div class="flex items-center text-[11px] text-base-content/45 h-6">{{ $t('file_info.copyright') }}</div>
            <div class="flex items-center text-[12px] text-base-content/75">{{ fileInfo?.e_copyright }}</div>

            <!-- Taken At -->
            <div class="flex items-center text-[11px] text-base-content/45 h-6">{{ $t('file_info.taken_at') }}</div>
            <div class="flex items-center text-[12px] text-base-content/75">{{ fileInfo?.e_date_time }}</div>

            <!-- Description -->
            <div class="flex items-start text-[11px] text-base-content/45 py-1.5">{{ $t('file_info.description') }}</div>
            <div class="flex items-center text-[12px] leading-5 text-base-content/75 wrap-break-words py-1.5">{{ fileInfo?.e_description }}</div>

            <!-- Geo Location -->
            <div class="flex items-center text-[11px] text-base-content/45 h-6">{{ $t('file_info.geo_location') }}</div>
            <div
              :class="['flex items-center text-[12px] text-base-content/75', hasLocation && !readonly ? 'cursor-pointer hover:text-primary' : '']"
              @click.stop="navigateLocation"
            >{{ formatGeoLocation() }}</div>
          </div>
        </Transition>
      </div>

      <!-- Map View -->
      <div v-if="fileInfo?.gps_latitude && fileInfo?.gps_longitude" 
        class="border-t border-base-content/5 px-1 py-4 space-y-3 flex flex-col transition-[flex-grow]" 
        :class="{ 'flex-1 min-h-75 shrink-0': showMapPanel }">
        <div class="flex items-center gap-1 cursor-pointer text-base-content/70 hover:text-base-content shrink-0" @click.stop="toggleMapPanel">
          <IconRight
            class="w-3 h-3 transition-transform duration-200"
            :class="{ 'rotate-90': showMapPanel }"
            @click.stop="toggleMapPanel"
          />
          <span class="font-bold mr-auto uppercase text-xs tracking-wide text-base-content/30">{{ $t('file_info.map') }}</span>
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
        <div class="text-center text-base-content/30 space-y-3 max-w-[260px]">
          <IconFile class="w-8 h-8 mx-auto text-base-content/30" />
          <p class="text-xs font-medium">{{ $t('file_info.empty_title') }}</p>
          <p class="text-xs text-base-content/30">{{ $t('file_info.empty_desc') }}</p>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, nextTick, computed, watch, onBeforeUnmount, onMounted } from 'vue';
import { listen } from '@tauri-apps/api/event';
import { useI18n } from 'vue-i18n';
import { useToast } from '@/common/toast';
import { useUIStore } from '@/stores/uiStore';
import { config, libConfig } from '@/common/config';
import { isWebViewVideoPlaybackDisabled, getGStreamerAvailability } from '@/common/video';
import { getTagsForFile, renameFile, editImage, getAlbum, getFileCollections, getFileInfo, getMotionPhotoVideoPath, revealPath, getFacesForFile, getPersonThumbnail } from '@/common/api';
import { 
  extractFileName, 
  getFileExtension,
  getFolderPath,
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
  getAssetSrc,
  getThumbUrl,
} from '@/common/utils';
import {
  IconClose,
  IconRight,
  IconFile,
  IconFolder,
  IconPhoto,
  IconRotate,
  IconVideo,
  IconZoomIn,
  IconZoomOut,
  IconLivePhoto,
  IconBookmark,
  IconPerson,
} from '@/common/icons';
import Breadcrumb from '@/components/Breadcrumb.vue';
import TButton from '@/components/TButton.vue';
import FavoriteRatingControl from '@/components/FavoriteRatingControl.vue';
import ImageHistogram from '@/components/ImageHistogram.vue';
import MapView from '@/components/MapView.vue';

const props = defineProps({
  fileInfo: {
    type: Object,
    required: false
  },
  readonly: {
    type: Boolean,
    default: false
  },
});

const { locale, messages, t } = useI18n();
const localeMsg = computed(() => messages.value[locale.value] as any);
const uiStore = useUIStore();


const emit = defineEmits([
  'close',
  'success',
  'toggleFavorite',
  'setRating',
  'setCulling',
  'rotate',
  'quickEditTag',
  'quickEditCollection',
  'quickEditComment',
  'navigateFolder',
  'openViewer',
  'navigateMetadata',
  'navigatePerson',
]);

const displayTags = ref<any[]>([]);
let tagsRequest = 0;
let tagsDisposed = false;
let stopTags: (() => void) | undefined;
async function refreshTags() {
  const id = props.fileInfo?.id; const library = libConfig._libraryId; const request = ++tagsRequest;
  if (!id) { displayTags.value = []; return; }
  const tags = await getTagsForFile(id);
  if (!tagsDisposed && request === tagsRequest && props.fileInfo?.id === id && libConfig._libraryId === library && tags) displayTags.value = tags;
}
watch(() => [props.fileInfo?.id, props.fileInfo?.tags, libConfig._libraryId], () => {
  displayTags.value = props.fileInfo?.tags || []; void refreshTags();
}, { immediate: true });
onMounted(async () => { const stop = await listen('tags-changed', refreshTags); if (tagsDisposed) stop(); else stopTags = stop; });
onBeforeUnmount(() => { tagsDisposed = true; tagsRequest++; stopTags?.(); });

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
const isLivePhoto = computed(() => (
  props.fileInfo?.media_subtype === 'live_photo' && !!props.fileInfo?.live_photo_video_path
));
const isMotionPhoto = computed(() => props.fileInfo?.media_subtype === 'motion_photo');
const isRawJpegPair = computed(() => (
  props.fileInfo?.media_subtype === 'raw_jpeg_pair' && !!props.fileInfo?.live_photo_video_id
));
const rawJpegCompanion = ref<any>(null);
const generalInfoTab = ref<'raw' | 'companion'>('raw');
let rawJpegCompanionRequestSeq = 0;
const rawJpegCompanionLabel = computed(() => {
  const extension = getFileExtension(
    rawJpegCompanion.value?.name || props.fileInfo?.live_photo_video_path || '',
  ).toLowerCase();
  return ['heic', 'heif', 'hif'].includes(extension) ? 'HEIC' : 'JPEG';
});
const generalFileInfo = computed(() => (
  generalInfoTab.value === 'companion' && rawJpegCompanion.value
    ? rawJpegCompanion.value
    : props.fileInfo
));
const isPrimaryGeneralInfo = computed(() => generalInfoTab.value === 'raw');
const motionPhotoVideoPath = ref<string | null>(null);
let motionPhotoVideoRequestSeq = 0;
const previewVideoPath = computed(() => {
  if (isLivePhoto.value) return props.fileInfo?.live_photo_video_path;
  if (isMotionPhoto.value) return motionPhotoVideoPath.value;
  return props.fileInfo?.file_path;
});
const canPreviewVideo = computed(() => (
  (isVideoFile.value || isLivePhoto.value || isMotionPhoto.value)
  && !!previewVideoPath.value
  && !isWebViewVideoPlaybackDisabled(previewVideoPath.value)
));
const canShowHistogram = computed(() => !isVideoFile.value);
const activePreviewMode = computed(() => canShowHistogram.value ? config.infoPanel.previewMode : 'thumbnail');
const isHistogramPreview = computed(() => activePreviewMode.value === 'histogram');
const histogramChannelLabel = computed(() => {
  const storedMask = Number(config.infoPanel.histogramChannels);
  const mask = storedMask === 16 || !Number.isInteger(storedMask) || storedMask < 0 || storedMask > 15
    ? 15
    : storedMask;
  const labels = [
    { bit: 1, label: 'L' },
    { bit: 2, label: 'R' },
    { bit: 4, label: 'G' },
    { bit: 8, label: 'B' },
  ];
  return labels
    .filter((item) => Boolean(mask & item.bit))
    .map((item) => item.label)
    .join('');
});
const previewVideoRef = ref<HTMLVideoElement | null>(null);
const showVideoPreview = ref(false);
const isVideoPreviewReady = ref(false);
const fileCollections = ref<Array<{ id: number; name: string }>>([]);
let fileCollectionsRequestSeq = 0;
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
  if (props.fileInfo?.media_subtype === 'live_photo') {
    return t('image_viewer.live');
  }
  if (props.fileInfo?.media_subtype === 'motion_photo') {
    return t('image_viewer.motion');
  }

  const formatLabel = (props.fileInfo?.format_label || '').trim();
  const isRaw = Number(props.fileInfo?.file_type || 0) === 3;
  if (formatLabel && !(isRaw && formatLabel.toUpperCase() === 'RAW')) {
    return formatLabel.toUpperCase();
  }

  const name = props.fileInfo?.name || '';
  const filePath = props.fileInfo?.file_path || '';
  const extension = getFileExtension(name || filePath).trim();
  if (!extension) return '';
  return extension.toUpperCase();
});
const previewTagLabel = computed(() => (
  isHistogramPreview.value ? histogramChannelLabel.value : previewFormatLabel.value
));
function togglePreview() {
  config.infoPanel.showPreview = !config.infoPanel.showPreview;
}

function setPreviewMode(mode: 'thumbnail' | 'histogram') {
  if (mode === 'histogram' && !canShowHistogram.value) return;
  config.infoPanel.previewMode = mode;
}

let videoPreviewRequest = 0;
async function playPreviewVideo() {
  const request = ++videoPreviewRequest;
  const available = await getGStreamerAvailability();
  if (request !== videoPreviewRequest || !available) return;
  if (!canPreviewVideo.value || !previewVideoPath.value || showVideoPreview.value) return;
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
    stopPreviewVideo();
  }
}

function playVideoPreviewOnHover() {
  if (canPreviewVideo.value) {
    void playPreviewVideo();
  }
}

function stopPreviewVideo() {
  videoPreviewRequest++;
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
  () => [props.fileInfo?.id, props.fileInfo?.media_subtype, props.fileInfo?.modified_at] as const,
  async ([fileId, mediaSubtype]) => {
    const requestSeq = ++motionPhotoVideoRequestSeq;
    motionPhotoVideoPath.value = null;
    if (!fileId || mediaSubtype !== 'motion_photo') return;
    try {
      const path = await getMotionPhotoVideoPath(Number(fileId));
      if (requestSeq === motionPhotoVideoRequestSeq) {
        motionPhotoVideoPath.value = path;
      }
    } catch (error) {
      if (requestSeq === motionPhotoVideoRequestSeq) {
        console.error('Failed to prepare motion photo video:', error);
      }
    }
  },
  { immediate: true },
);

watch(
  () => [
    props.fileInfo?.id,
    props.fileInfo?.file_path,
    props.fileInfo?.live_photo_video_path,
    props.fileInfo?.media_subtype,
    showPreviewPanel.value,
    isHistogramPreview.value,
  ],
  stopPreviewVideo
);

onBeforeUnmount(stopPreviewVideo);

watch(
  () => [props.fileInfo?.id, Boolean(props.fileInfo?.has_collections), props.fileInfo?.collectionVersion] as const,
  async ([fileId, hasCollections]) => {
    const requestSeq = ++fileCollectionsRequestSeq;
    fileCollections.value = [];
    if (!fileId || !hasCollections) return;
    const collections = await getFileCollections(Number(fileId));
    if (requestSeq !== fileCollectionsRequestSeq || Number(props.fileInfo?.id) !== Number(fileId)) return;
    fileCollections.value = Array.isArray(collections) ? collections : [];
  },
  { immediate: true }
);

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

const quickSave = async (): Promise<boolean> => {
  if (!props.fileInfo) return false;
  if (uiStore.activeAdjustments.filePath !== props.fileInfo.file_path) return true;

  const adj = uiStore.activeAdjustments as any;
  const ext = getFileExtension(props.fileInfo.name).toLowerCase();
  const outputFormat = ['jpg', 'jpeg', 'jfif'].includes(ext) ? 'jpg' : ext;

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
const albumCoverUrl = ref('');
let albumRootRequestSeq = 0;

const generalFolderBreadcrumbs = computed(() => {
  const folderPath = getFolderPath(generalFileInfo.value?.file_path);
  if (!folderPath) return [];
  return buildFolderBreadcrumbs(folderPath, albumRootPath.value);
});
function revealFileInFolder() {
  if (props.fileInfo?.file_path) {
    revealPath(props.fileInfo.file_path);
  }
}

watch(
  () => props.fileInfo?.album_id,
  async (albumId) => {
    const requestSeq = ++albumRootRequestSeq;
    albumRootPath.value = '';
    albumCoverUrl.value = '';
    if (!albumId) return;
    const album = await getAlbum(albumId);
    if (requestSeq !== albumRootRequestSeq) return;
    if (props.fileInfo?.album_id !== albumId) return;
    albumRootPath.value = album?.path || '';
    const coverFileId = Number(album?.cover_file_id || 0);
    if (coverFileId > 0) {
      albumCoverUrl.value = getThumbUrl(coverFileId, false, config.settings.thumbnailSize);
    }
  },
  { immediate: true }
);

watch(
  () => [props.fileInfo?.id, props.fileInfo?.media_subtype, props.fileInfo?.live_photo_video_id] as const,
  async ([fileId, mediaSubtype, companionId]) => {
    const requestSeq = ++rawJpegCompanionRequestSeq;
    generalInfoTab.value = 'raw';
    rawJpegCompanion.value = null;
    if (!fileId || mediaSubtype !== 'raw_jpeg_pair' || !companionId) return;
    const companion = await getFileInfo(Number(companionId));
    if (requestSeq !== rawJpegCompanionRequestSeq || Number(props.fileInfo?.id) !== Number(fileId)) return;
    rawJpegCompanion.value = companion || null;
  },
  { immediate: true },
);

const startRename = () => {
  if (!props.fileInfo || props.readonly) return;
  
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

// Clickable metadata values: jump to the corresponding sidebar view.
const hasCamera = computed(() => !!(props.fileInfo?.e_make || props.fileInfo?.e_model));
const hasLens = computed(() => !!props.fileInfo?.e_lens_model);
const hasLocation = computed(() => !!(props.fileInfo?.geo_cc || props.fileInfo?.geo_admin1 || props.fileInfo?.geo_name));

function navigateCamera() {
  if (!hasCamera.value || props.readonly) return;
  emit('navigateMetadata', { type: 'camera', make: props.fileInfo?.e_make || null, model: props.fileInfo?.e_model || null });
}

function navigateLens() {
  if (!hasLens.value || props.readonly) return;
  emit('navigateMetadata', { type: 'lens', lensMake: props.fileInfo?.e_lens_make || null, lensModel: props.fileInfo?.e_lens_model || null });
}

function navigateLocation() {
  if (!hasLocation.value || props.readonly) return;
  emit('navigateMetadata', { type: 'location', cc: props.fileInfo?.geo_cc || null, admin1: props.fileInfo?.geo_admin1 || null, name: props.fileInfo?.geo_name || null });
}

// People recognized in this file (deduplicated by person id).
const filePersons = ref<Array<{ id: number; name: string; thumbnail: string }>>([]);
let filePersonsRequestSeq = 0;

async function loadFilePersons(fileId: number) {
  const seq = ++filePersonsRequestSeq;
  if (!fileId || fileId <= 0) {
    if (seq === filePersonsRequestSeq) filePersons.value = [];
    return;
  }
  const faces = await getFacesForFile(fileId);
  if (seq !== filePersonsRequestSeq) return;

  // Dedupe persons by id, keeping the name from the face.
  const personsById = new Map<number, string>();
  for (const face of faces || []) {
    const id = Number(face?.person_id || 0);
    if (id > 0 && !personsById.has(id)) {
      personsById.set(id, face?.person_name || '');
    }
  }
  if (personsById.size === 0) {
    filePersons.value = [];
    return;
  }

  // Fetch each person's face thumbnail from the backend (People list is
  // paginated on the frontend, so we can't rely on locally loaded thumbnails).
  const persons = await Promise.all(
    Array.from(personsById.entries()).map(async ([id, name]) => {
      const thumbnail = (await getPersonThumbnail(id)) || '';
      return { id, name, thumbnail };
    }),
  );
  if (seq !== filePersonsRequestSeq) return;
  filePersons.value = persons;
}

watch(() => props.fileInfo?.id, (id) => {
  void loadFilePersons(Number(id || 0));
}, { immediate: true });

function navigatePerson(person: { id: number; name: string }) {
  if (!person?.id || props.readonly) return;
  emit('navigatePerson', { personId: person.id, personName: person.name });
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

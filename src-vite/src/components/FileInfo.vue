<template>
  <div class="w-full h-full rounded-box bg-base-200 flex flex-col overflow-hidden">
    <!-- Tabs & Close -->
    <div class="flex items-center w-full shrink-0 px-2 mb-2">
      <div role="tablist" class="tabs tabs-sm tabs-border flex-1">
        <a role="tab" class="tab tab-active mx-1">{{ $t('info_panel.tabs[0]') }}</a>
      </div>
      <div class="mt-2 flex items-center gap-1">
        <TButton
          :icon="IconClose"
          :tooltip="$t('msgbox.close')"
          :buttonSize="'small'"
          @click.stop="$emit('close')"
        />
      </div>
    </div>

    <!-- Info Content -->
    <div v-if="fileInfo" class="mb-2 px-2 flex-1 overflow-y-auto overflow-x-hidden space-y-3 flex flex-col bg-base-200/50">

      <!-- Title / Thumbnail -->
      <!-- <div class="flex justify-center p-2">
        <div class="relative w-16 h-16 rounded-box shrink-0 shadow-sm border border-base-content/5">
          <img v-if="fileInfo?.thumbnail" :src="fileInfo.thumbnail" :style="thumbnailStyle" class="w-full h-full object-cover rounded-box" />
          <div v-else-if="fileInfo" class="w-full h-full skeleton object-cover rounded-box"></div>
          <div v-else class="w-full h-full bg-base-content/5 rounded-box"></div>
          
          <div v-if="fileInfo?.is_favorite" class="absolute -bottom-1 -right-1 drop-shadow-md">
            <IconHeartFilled class="t-icon-size-xs" />
          </div>
        </div>
      </div> -->

      <!-- File Info Section -->
      <div class="rounded-box p-3 space-y-3 bg-base-300/30 border border-base-content/5 shadow-sm">

        <div class="flex items-center gap-2 cursor-pointer text-base-content/70 hover:text-base-content transition-all duration-200 ease-in-out" 
          @click.stop="config.infoPanel.showBasicInfo = !config.infoPanel.showBasicInfo"
        >
          <IconFile class="w-4 h-4" />
          <span class="font-bold mr-auto uppercase text-xs tracking-wide">{{ $t('file_info.title') }}</span>
          <TButton
            :icon="config.infoPanel.showBasicInfo ? IconArrowUp : IconArrowDown"
            :buttonSize="'small'"
          />
        </div>

        <Transition
          @before-enter="onBeforeEnter"
          @enter="onEnter"
          @after-enter="onAfterEnter"
          @leave="onLeave"
        >
          <div v-if="config.infoPanel.showBasicInfo" class="grid grid-cols-[80px_1fr] gap-y-3 gap-x-4 text-xs overflow-hidden">
            <!-- Name -->
            <div class="text-[10px] uppercase tracking-widest font-bold text-base-content/25 py-1">{{ $t('file_info.name') }}</div>
            <div class="flex items-center">
              <input 
                v-if="isRenaming"
                ref="renameInputRef"
                v-model="renamingName" 
                class="font-bold text-xs text-base-content input input-xs input-bordered p-1 h-6 leading-6 w-full"
                @blur="finishRename"
                @keydown.enter="finishRename"
                @keydown.esc="cancelRename"
                @click.stop
              />
              <span v-else 
                class="font-semibold text-xs text-base-content/65 break-all cursor-text hover:bg-base-content/10 rounded px-1 -mx-1 transition-colors"
                @click.stop="startRename"
              >{{ fileInfo?.name }}</span>
            </div>

            <!-- Album -->
            <div class="text-[10px] uppercase tracking-widest font-bold text-base-content/25">{{ $t('file_info.album_name') }}</div>
            <div class="text-xs font-semibold text-base-content/65 break-all">{{ fileInfo?.album_name }}</div>

            <!-- Path -->
            <div class="text-[10px] uppercase tracking-widest font-bold text-base-content/25">{{ $t('file_info.folder') }}</div>
            <div class="text-xs font-semibold text-base-content/65 break-all">{{ getFolderPath(fileInfo?.file_path) }}</div>

            <!-- Size -->
            <div class="text-[10px] uppercase tracking-widest font-bold text-base-content/25">{{ $t('file_info.size') }}</div>
            <div class="text-xs font-semibold text-base-content/65">{{ formatFileSize(fileInfo?.size) }}</div>

            <!-- Dimension -->
            <div class="text-[10px] uppercase tracking-widest font-bold text-base-content/25">{{ $t('file_info.dimension') }}</div>
            <div class="text-xs font-semibold text-base-content/65">{{ formatDimensionText(fileInfo?.width, fileInfo?.height) }}</div>

            <!-- Duration -->
            <template v-if="fileInfo?.file_type === 2">
              <div class="text-[10px] uppercase tracking-widest font-bold text-base-content/25">{{ $t('file_info.duration') }}</div>
              <div class="text-xs font-semibold text-base-content/65">{{ formatDuration(fileInfo?.duration) }}</div>
            </template>

            <!-- Created At -->
            <div class="text-[10px] uppercase tracking-widest font-bold text-base-content/25">{{ $t('file_info.created_at') }}</div>
            <div class="text-xs font-semibold text-base-content/65">{{ formatTimestamp(fileInfo?.created_at, $t('format.date_time')) }}</div>

            <!-- Modified At -->
            <div class="text-[10px] uppercase tracking-widest font-bold text-base-content/25">{{ $t('file_info.modified_at') }}</div>
            <div class="text-xs font-semibold text-base-content/65">{{ formatTimestamp(fileInfo?.modified_at, $t('format.date_time')) }}</div>

            <div class="flex items-center text-[10px] uppercase tracking-widest font-bold text-base-content/25">{{ $t('menu.meta.favorite') }}</div>
            <div class="h-6 flex items-center gap-0.5">
              <button
                class="btn btn-ghost btn-xs min-h-0 h-6 w-6 p-0 mr-1"
                :title="fileInfo?.is_favorite ? $t('menu.meta.unfavorite') : $t('menu.meta.favorite')"
                @click.stop="emit('toggleFavorite')"
              >
                <component
                  :is="fileInfo?.is_favorite ? IconHeartFilled : IconHeart"
                  class="w-3.5 h-3.5"
                  :class="fileInfo?.is_favorite ? 'text-error' : 'text-base-content/30'"
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
            <div class="text-[10px] uppercase tracking-widest font-bold text-base-content/25">{{ $t('file_info.tags') }}</div>
            <div class="group flex items-start gap-1">
              <div class="text-xs font-semibold text-base-content/65 flex flex-wrap gap-1 flex-1 min-w-0">
                <template v-if="fileInfo?.tags && fileInfo.tags.length">
                  <span v-for="tag in fileInfo.tags" :key="tag.id" class="badge badge-sm badge-ghost font-medium">{{ tag.name }}</span>
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
            <div class="text-[10px] uppercase tracking-widest font-bold text-base-content/25">{{ $t('file_info.comment') }}</div>
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
          </div>
        </Transition>
      </div>

      <!-- Metadata Section -->
      <div class="rounded-box p-3 space-y-3 bg-base-300/30 border border-base-content/5 shadow-sm">

        <div class="flex items-center gap-2 cursor-pointer text-base-content/70 hover:text-base-content" @click.stop="config.infoPanel.showMetadata = !config.infoPanel.showMetadata">
          <IconCameraAperture class="w-4 h-4 " /> 
          <span class="font-bold mr-auto uppercase text-xs tracking-wide">{{ $t('file_info.metadata') }}</span>
          <TButton
            :icon="config.infoPanel.showMetadata ? IconArrowUp : IconArrowDown"
            :buttonSize="'small'"
          />
        </div>

        <Transition
          @before-enter="onBeforeEnter"
          @enter="onEnter"
          @after-enter="onAfterEnter"
          @leave="onLeave"
        >
          <div v-if="config.infoPanel.showMetadata" class="grid grid-cols-[80px_1fr] gap-y-3 gap-x-4 text-xs overflow-hidden">
            <!-- Camera -->
            <div class="text-[10px] uppercase tracking-widest font-bold text-base-content/25">{{ $t('file_info.camera') }}</div>
            <div class="text-xs font-semibold text-base-content/65">{{ formatCameraInfo(fileInfo?.e_make, fileInfo?.e_model) }}</div>

            <!-- Lens -->
            <div class="text-[10px] uppercase tracking-widest font-bold text-base-content/25">{{ $t('file_info.lens') }}</div>
            <div class="text-xs font-semibold text-base-content/65">{{ fileInfo?.e_lens_model }}</div>

            <!-- Capture Settings -->
            <div class="text-[10px] uppercase tracking-widest font-bold text-base-content/25">{{ $t('file_info.capture_settings') }}</div>
            <div class="text-xs font-semibold text-base-content/65">{{ formatCaptureSettings(fileInfo?.e_focal_length, fileInfo?.e_exposure_time, fileInfo?.e_f_number, fileInfo?.e_iso_speed, fileInfo?.e_exposure_bias) }}</div>

            <!-- Software -->
            <div class="text-[10px] uppercase tracking-widest font-bold text-base-content/25">{{ $t('file_info.software') }}</div>
            <div class="text-xs font-semibold text-base-content/65">{{ fileInfo?.e_software }}</div>

            <!-- Taken By -->
            <div class="text-[10px] uppercase tracking-widest font-bold text-base-content/25">{{ $t('file_info.taken_by') }}</div>
            <div class="text-xs font-semibold text-base-content/65">{{ fileInfo?.e_artist }}</div>

            <!-- Copyright -->
            <div class="text-[10px] uppercase tracking-widest font-bold text-base-content/25">{{ $t('file_info.copyright') }}</div>
            <div class="text-xs font-semibold text-base-content/65">{{ fileInfo?.e_copyright }}</div>

            <!-- Taken At -->
            <div class="text-[10px] uppercase tracking-widest font-bold text-base-content/25">{{ $t('file_info.taken_at') }}</div>
            <div class="text-xs font-semibold text-base-content/65">{{ fileInfo?.e_date_time }}</div>

            <!-- Description -->
            <div class="text-[10px] uppercase tracking-widest font-bold text-base-content/25">{{ $t('file_info.description') }}</div>
            <div class="text-xs font-semibold text-base-content/65 wrap-break-words">{{ fileInfo?.e_description }}</div>

            <!-- Geo Location -->
            <div class="text-[10px] uppercase tracking-widest font-bold text-base-content/25">{{ $t('file_info.geo_location') }}</div>
            <div class="text-xs font-semibold text-base-content/65">{{ formatGeoLocation() }}</div>
          </div>
        </Transition>
      </div>

      <!-- Map View -->
      <div v-if="fileInfo?.gps_latitude && fileInfo?.gps_longitude" 
        class="rounded-box p-3 space-y-3 bg-base-300/30 border border-base-content/5 shadow-sm flex flex-col transition-[flex-grow]" 
        :class="{ 'flex-1 min-h-[300px]': config.infoPanel.showMap }">
        <div class="flex items-center gap-2 cursor-pointer text-base-content/70 hover:text-base-content shrink-0" @click.stop="config.infoPanel.showMap = !config.infoPanel.showMap">
          <IconLocation class="w-4 h-4 " /> 
          <span class="font-bold mr-auto uppercase text-xs tracking-wide">{{ $t('file_info.map') }}</span>
          <TButton
            :icon="config.infoPanel.showMap ? IconArrowUp : IconArrowDown"
            :buttonSize="'small'"
          />
        </div>

        <Transition
          @before-enter="onBeforeEnter"
          @enter="onEnter"
          @after-enter="onAfterEnter"
          @leave="onLeave"
        >
          <div v-if="config.infoPanel.showMap" class="overflow-hidden flex-1 flex flex-col min-h-0">
            <div class="w-full rounded-box overflow-hidden relative z-0 flex-1 min-h-[200px] border border-base-content/5">
              <MapView
                :lat="fileInfo.gps_latitude ? Number(fileInfo.gps_latitude) : 0"
                :lon="fileInfo.gps_longitude ? Number(fileInfo.gps_longitude) : 0"
              />
            </div>
          </div>
        </Transition>
      </div>
    </div>

    <ToolTip ref="toolTipRef" />
  </div>
</template>

<script setup lang="ts">
import { ref, nextTick, computed } from 'vue';
import { useI18n } from 'vue-i18n';
import { useUIStore } from '@/stores/uiStore';
import { config } from '@/common/config';
import { renameFile, editImage } from '@/common/api';
import { 
  extractFileName, 
  getFileExtension,
  getFolderPath, 
  formatDimensionText, 
  formatFileSize, 
  formatTimestamp,
  formatDuration,
  formatCaptureSettings,
  formatCameraInfo,
  getCountryName,
  combineFileName,
  isValidFileName
} from '@/common/utils';
import { 
  IconClose, IconLocation, IconArrowDown, IconArrowUp, IconCameraAperture, 
  IconFile, IconHeart, IconHeartFilled, IconStar, IconStarFilled, IconEdit,
} from '@/common/icons';
import TButton from '@/components/TButton.vue';
import ToolTip from '@/components/ToolTip.vue';
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
  'quickEditTag',
  'quickEditComment',
]);

const toolTipRef = ref<InstanceType<typeof ToolTip> | null>(null);

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
    flipHorizontal: !!adj.flipX,
    flipVertical: !!adj.flipY,
    rotate: adj.rotate || 0,
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
      toolTipRef.value?.showTip(localeMsg.value.tooltip.save_image.failed, true);
      return false;
    }

    uiStore.updateFileVersion(props.fileInfo.file_path);
    uiStore.clearActiveAdjustments();
    emit('success');
    toolTipRef.value?.showTip(localeMsg.value.tooltip.save_image.success);
    return true;
  } catch {
    toolTipRef.value?.showTip(localeMsg.value.tooltip.save_image.failed, true);
    return false;
  }
};

// Rename logic
const isRenaming = ref(false);
const renamingName = ref('');
const renameInputRef = ref<HTMLInputElement | null>(null);

const startRename = () => {
  if (!props.fileInfo) return;
  
  const { name } = extractFileName(props.fileInfo.name);
  renamingName.value = name;
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

<template>
  <div class="w-full h-full rounded-box bg-base-200 flex flex-col overflow-hidden">
    <!-- Info Content -->
    <div class="my-2 px-2 flex-1 overflow-y-auto overflow-x-hidden space-y-2 flex flex-col">

      <!-- Title -->
      <div class="flex items-center">
        <img v-if="fileInfo?.thumbnail" :src="fileInfo.thumbnail" class="w-10 h-10 object-cover rounded-box shrink-0" />
        <div v-else-if="fileInfo" class="w-10 h-10 skeleton object-cover rounded-box shrink-0"></div>
        <div v-else class="w-10 h-10 bg-base-content/5 rounded-box shrink-0"></div>
        <input 
          v-if="isRenaming"
          ref="renameInputRef"
          v-model="renamingName" 
          class="ml-2 mr-auto font-bold text-sm text-base-content input input-xs input-bordered p-1 h-6 leading-6"
          @blur="finishRename"
          @keydown.enter="finishRename"
          @keydown.esc="cancelRename"
          @click.stop
        />
        <span v-else 
          class="ml-2 mr-auto font-bold text-sm text-base-content/70 break-all cursor-text hover:bg-base-content/10 rounded px-1 -mx-1 transition-colors"
          @click.stop="startRename"
        >{{ fileInfo?.name }}</span>
        
        <!-- <TButton v-if="fileInfo && !fileInfo?.has_embedding"
          :icon="IconUpdate"
          :buttonSize="'small'"
          @click.stop="$emit('update')"
        /> -->
        <TButton
          :icon="IconClose"
          :buttonSize="'small'"
          @click.stop="$emit('close')"
        />
      </div>

      <!-- File Info Section -->
      <div class="rounded-box p-2 space-y-3 border border-base-content/5 shadow-sm">

        <div class="flex items-center gap-2 cursor-pointer text-base-content/70 hover:text-base-content transition-all duration-200 ease-in-out" 
          @click.stop="config.infoPanel.showBasicInfo = !config.infoPanel.showBasicInfo"
        >
          <IconFileInfo class="w-4 h-4 " /> 
          <span class="font-bold mr-auto">{{ $t('file_info.title') }}</span>
          <TButton
            :icon="config.infoPanel.showBasicInfo ? IconArrowUp : IconArrowDown"
            :buttonSize="'small'"
          />
        </div>

        <Transition
          @before-enter="onBeforeEnter"
          @enter="onEnter"
          @leave="onLeave"
        >
          <div v-if="config.infoPanel.showBasicInfo" class="grid grid-cols-[100px_1fr] gap-y-3 gap-x-4 text-sm overflow-hidden">
            <!-- Album -->
            <div class="font-medium text-base-content/30 tracking-wide">{{ $t('file_info.album_name') }}</div>
            <div class="text-base-content/70 break-all">{{ fileInfo?.album_name }}</div>

            <!-- Path -->
            <div class="font-medium text-base-content/30 tracking-wide">{{ $t('file_info.folder') }}</div>
            <div class="text-base-content/70 break-all">{{ getFolderPath(fileInfo?.file_path) }}</div>

            <!-- Size -->
            <div class="font-medium text-base-content/30 tracking-wide">{{ $t('file_info.size') }}</div>
            <div class="text-base-content/70">{{ formatFileSize(fileInfo?.size) }}</div>

            <!-- Dimension -->
            <div class="font-medium text-base-content/30 tracking-wide">{{ $t('file_info.dimension') }}</div>
            <div class="text-base-content/70">{{ formatDimensionText(fileInfo?.width, fileInfo?.height) }}</div>

            <!-- Duration -->
            <template v-if="fileInfo?.file_type === 2">
              <div class="font-medium text-base-content/30 tracking-wide">{{ $t('file_info.duration') }}</div>
              <div class="text-base-content/70">{{ formatDuration(fileInfo?.duration) }}</div>
            </template>

            <!-- Created At -->
            <div class="font-medium text-base-content/30 tracking-wide">{{ $t('file_info.created_at') }}</div>
            <div class="text-base-content/70">{{ formatTimestamp(fileInfo?.created_at, $t('format.date_time')) }}</div>

            <!-- Modified At -->
            <div class="font-medium text-base-content/30 tracking-wide">{{ $t('file_info.modified_at') }}</div>
            <div class="text-base-content/70">{{ formatTimestamp(fileInfo?.modified_at, $t('format.date_time')) }}</div>

            <!-- Tags -->
            <div class="font-medium text-base-content/30 tracking-wide">{{ $t('file_info.tags') }}</div>
            <div class="text-base-content/70 flex flex-wrap gap-1">
              <template v-if="fileInfo?.tags && fileInfo.tags.length">
                <span v-for="tag in fileInfo.tags" :key="tag.id" class="badge">{{ tag.name }}</span>
              </template>
            </div>

            <!-- Comment -->
            <div class="font-medium text-base-content/30 tracking-wide">{{ $t('file_info.comment') }}</div>
            <div class="text-base-content/70 break-words whitespace-pre-wrap">{{ fileInfo?.comments }}</div>
          </div>
        </Transition>
      </div>

      <!-- Metadata Section -->
      <div class="rounded-box p-2 space-y-3 border border-base-content/5 shadow-sm">

        <div class="flex items-center gap-2 cursor-pointer text-base-content/70 hover:text-base-content" @click.stop="config.infoPanel.showMetadata = !config.infoPanel.showMetadata">
          <IconCamera class="w-4 h-4 " /> 
          <span class="font-bold mr-auto">{{ $t('file_info.metadata') }}</span>
          <TButton
            :icon="config.infoPanel.showMetadata ? IconArrowUp : IconArrowDown"
            :buttonSize="'small'"
          />
        </div>

        <Transition
          @before-enter="onBeforeEnter"
          @enter="onEnter"
          @leave="onLeave"
        >
          <div v-if="config.infoPanel.showMetadata" class="grid grid-cols-[100px_1fr] gap-y-3 gap-x-4 text-sm overflow-hidden">
            <!-- Camera -->
            <div class="font-medium text-base-content/30 tracking-wide">{{ $t('file_info.camera') }}</div>
            <div class="text-base-content/70">{{ formatCameraInfo(fileInfo?.e_make, fileInfo?.e_model) }}</div>

            <!-- Lens -->
            <div class="font-medium text-base-content/30 tracking-wide">{{ $t('file_info.lens') }}</div>
            <div class="text-base-content/70">{{ fileInfo?.e_lens_model }}</div>

            <!-- Capture Settings -->
            <div class="font-medium text-base-content/30 tracking-wide">{{ $t('file_info.capture_settings') }}</div>
            <div class="text-base-content/70">{{ formatCaptureSettings(fileInfo?.e_focal_length, fileInfo?.e_exposure_time, fileInfo?.e_f_number, fileInfo?.e_iso_speed, fileInfo?.e_exposure_bias) }}</div>

            <!-- Software -->
            <div class="font-medium text-base-content/30 tracking-wide">{{ $t('file_info.software') }}</div>
            <div class="text-base-content/70">{{ fileInfo?.e_software }}</div>

            <!-- Taken By -->
            <div class="font-medium text-base-content/30 tracking-wide">{{ $t('file_info.taken_by') }}</div>
            <div class="text-base-content/70">{{ fileInfo?.e_artist }}</div>

            <!-- Copyright -->
            <div class="font-medium text-base-content/30 tracking-wide">{{ $t('file_info.copyright') }}</div>
            <div class="text-base-content/70">{{ fileInfo?.e_copyright }}</div>

            <!-- Taken At -->
            <div class="font-medium text-base-content/30 tracking-wide">{{ $t('file_info.taken_at') }}</div>
            <div class="text-base-content/70">{{ fileInfo?.e_date_time }}</div>

            <!-- Description -->
            <div class="font-medium text-base-content/30 tracking-wide">{{ $t('file_info.description') }}</div>
            <div class="text-base-content/70 break-words">{{ fileInfo?.e_description }}</div>

            <!-- Geo Location -->
            <div class="font-medium text-base-content/30 tracking-wide">{{ $t('file_info.geo_location') }}</div>
            <div class="text-base-content/70">{{ formatGeoLocation() }}</div>
          </div>
        </Transition>
      </div>

      <!-- Map View -->
      <div v-if="fileInfo?.gps_latitude && fileInfo?.gps_longitude" 
        class="rounded-box p-2 space-y-3 border border-base-content/5 shadow-sm flex flex-col transition-[flex-grow]" 
        :class="{ 'flex-1 min-h-[300px]': config.infoPanel.showMap }">
        <div class="flex items-center gap-2 cursor-pointer text-base-content/70 hover:text-base-content shrink-0" @click.stop="config.infoPanel.showMap = !config.infoPanel.showMap">
          <IconLocation class="w-4 h-4 " /> 
          <span class="font-bold mr-auto">{{ $t('file_info.location') }}</span>
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
            <div class="w-full rounded-box overflow-hidden relative z-0 flex-1 min-h-[200px]">
              <MapView
                :lat="fileInfo.gps_latitude ? Number(fileInfo.gps_latitude) : 0"
                :lon="fileInfo.gps_longitude ? Number(fileInfo.gps_longitude) : 0"
              />
            </div>
          </div>
        </Transition>
      </div>
    </div>
  </div>

</template>

<script setup lang="ts">
import { ref, nextTick, computed } from 'vue';
import { useI18n } from 'vue-i18n';
import { useUIStore } from '@/stores/uiStore';
import { config } from '@/common/config';
import { renameFile } from '@/common/api';
import { 
  formatTimestamp, formatFileSize, formatDuration, formatDimensionText, 
  getFolderPath, formatCaptureSettings, formatCameraInfo, getCountryName,
  extractFileName, combineFileName, isValidFileName
} from '@/common/utils';
import { IconClose, IconFileInfo, IconCamera, IconLocation, IconArrowDown, IconArrowUp } from '@/common/icons';
import TButton from '@/components/TButton.vue';

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
  'close'
]);

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

</script>
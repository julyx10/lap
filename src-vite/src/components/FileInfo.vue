<template>
  <div class="w-full h-full rounded-box bg-base-200 flex flex-col overflow-hidden">
    <!-- Info Content -->
    <div class="my-2 px-2 flex-1 overflow-y-auto overflow-x-hidden space-y-2">
      
      <!-- File Info Section -->
      <div class="rounded-box p-2 space-y-2 border border-base-content/5">
        <div class="flex items-center gap-2">
          <img v-if="fileInfo?.thumbnail" :src="fileInfo.thumbnail" class="w-10 h-10 object-cover rounded-box shrink-0" />
          <div v-else-if="fileInfo" class="w-10 h-10 skeleton object-cover rounded-box shrink-0"></div>
          <div v-else class="w-10 h-10 bg-base-content/5 rounded-box shrink-0"></div>
          <!-- <span class="font-bold text-sm text-base-content/70">{{ $t('file_info.title') }}</span> -->
          <span class="font-bold text-sm text-base-content/70 overflow-hidden truncate">{{ fileInfo?.name }}</span>
        </div>

        <div class="grid grid-cols-[100px_1fr] gap-y-3 gap-x-4 text-sm">
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
      </div>

      <!-- Metadata Section -->
      <div class="rounded-box p-2 space-y-3 border border-base-content/5">
        <div class="flex items-center gap-2">
          <IconCamera class="w-4 h-4 text-base-content/30" /> 
          <span class="font-bold text-sm text-base-content/30">{{ $t('file_info.metadata') }}</span>
        </div>

        <div class="grid grid-cols-[100px_1fr] gap-y-3 gap-x-4 text-sm">
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
      </div>

      <!-- Map View -->
      <div v-if="config.infoPanel.showMap && fileInfo?.gps_latitude && fileInfo?.gps_longitude" class="">
        <MapView
          :lat="fileInfo.gps_latitude ? Number(fileInfo.gps_latitude) : 0"
          :lon="fileInfo.gps_longitude ? Number(fileInfo.gps_longitude) : 0"
        />
      </div>
    </div>

  </div>

</template>

<script setup lang="ts">
import { ref } from 'vue';
import { useI18n } from 'vue-i18n';
import { config } from '@/common/config';
import { formatTimestamp, formatFileSize, formatDuration, formatDimensionText, getFolderPath, formatCaptureSettings, formatCameraInfo, getCountryName } from '@/common/utils';
import { IconCamera } from '@/common/icons';

import MapView from '@/components/MapView.vue';

const props = defineProps({
  fileInfo: {
    type: Object,
    required: false
  },
});

const { locale } = useI18n();

const emit = defineEmits([
  'close'
]);

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

// emit close event
// function clickClose() {
//   emit('close');
// }

</script>
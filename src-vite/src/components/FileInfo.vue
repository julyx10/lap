<template>
  <div class="h-full rounded-l-lg w-full bg-base-200 flex flex-col select-none">
    <!-- Title bar -->
    <div class="p-2 flex items-center justify-between">
      <span class="p-1 font-bold">{{ $t('file_info.title') }}</span>
      <TButton
        :icon="IconClose"
        :buttonSize="'small'"
        @click="clickClose"
      />
    </div>

    <!-- File Info table -->
    <div class="flex-1 pl-1 pb-2 overflow-x-hidden overflow-y-auto">
      <table v-if="fileInfo" class="w-full text-sm border-separate border-spacing-2">
        <!-- general file info -->
        <tbody>
          <tr>
            <td colspan="2">
              <div class="flex items-center">
                <IconFile class="w-4 h-4" /> 
                <span class="ml-1 font-bold">{{ $t('file_info.general_info') }}</span>
              </div>
            </td>
          </tr>
          <tr>
            <td class="text-nowrap">{{ $t('file_info.album_name') }}</td>
            <td class="text-wrap break-all">{{ fileInfo.album_name }}</td>
          </tr>
          <tr>
            <td class="text-nowrap">{{ $t('file_info.path') }}</td>
            <td class="text-wrap break-all">{{ getFolderPath(fileInfo.file_path) }}</td>
            <!-- <td>
              <input
              type="text"
              readonly
              :value="fileInfo.file_path"
              class="py-1 w-full border-none focus:border-none focus:ring-0 focus:outline-none"
              />
            </td> -->
          </tr>
          <tr>
            <td class="text-nowrap">{{ $t('file_info.name') }}</td>
            <td class="text-wrap break-all">{{ fileInfo.name }}</td>
          </tr>
          <tr>
            <td class="text-nowrap">{{ $t('file_info.size') }}</td>
            <td>{{ formatFileSize(fileInfo.size) }}</td>
          </tr>
          <tr>
            <td class="text-nowrap">{{ $t('file_info.dimension') }}</td>
            <td>{{ formatDimensionText(fileInfo.width, fileInfo.height) }}</td>
          </tr>
          <tr v-if="fileInfo.file_type === 2">
            <td class="text-nowrap">{{ $t('file_info.duration') }}</td>
            <td>{{ formatDuration(fileInfo.duration) }}</td>
          </tr>
          <tr>
            <td class="text-nowrap">{{ $t('file_info.created_at') }}</td>
            <td>{{ formatTimestamp(fileInfo.created_at, $t('format.date_time')) }}</td>
          </tr>
          <tr>
            <td class="text-nowrap">{{ $t('file_info.modified_at') }}</td>
            <td>{{ formatTimestamp(fileInfo.modified_at, $t('format.date_time')) }}</td>
          </tr>
          <tr>
            <td class="text-nowrap">{{ $t('file_info.tags') }}</td>
            <td>
              <span v-if="fileInfo.tags && fileInfo.tags.length">
                {{ fileInfo.tags.map(tag => tag.name).join(', ') }}
              </span>
            </td>
          </tr>
          <tr>
            <td class="text-nowrap">{{ $t('file_info.comment') }}</td>
            <td class="text-wrap">{{ fileInfo.comments }}</td>
          </tr>
        </tbody>

        <!-- exif info -->
        <tbody>
          <tr>
            <td colspan="2">
              <div class="mt-2 flex items-center">
                <IconCamera class="w-4 h-4" /> 
                <span class="ml-1 font-bold">{{ $t('file_info.exif_info') }}</span>
              </div>
            </td>
          </tr>
          <tr>
            <td class="text-nowrap">{{ $t('file_info.camera') }}</td>
            <td>{{ fileInfo.e_make }} {{ fileInfo.e_model }}</td>
          </tr>
          <tr>
            <td class="text-nowrap">{{ $t('file_info.lens') }}</td>
            <td>{{ fileInfo.e_lens_model }}</td>
          </tr>
          <tr>
            <td class="text-nowrap">{{ $t('file_info.capture_settings') }}</td>
            <td>{{ formatCaptureSettings(fileInfo.e_focal_length, fileInfo.e_exposure_time, fileInfo.e_f_number, fileInfo.e_iso_speed, fileInfo.e_exposure_bias) }}</td>
          </tr>
          <tr>
            <td class="text-nowrap">{{ $t('file_info.software') }}</td>
            <td>{{ fileInfo.e_software }}</td>
          </tr>
          <tr>
            <td class="text-nowrap">{{ $t('file_info.taken_by') }}</td>
            <td>{{ fileInfo.e_artist }}</td>
          </tr>
          <tr>
            <td class="text-nowrap">{{ $t('file_info.taken_at') }}</td>
            <td>{{ fileInfo.e_date_time }}</td>
          </tr>
          <tr>
            <td class="text-nowrap">{{ $t('file_info.description') }}</td>
            <td>{{ fileInfo.e_description }}</td>
          </tr>
          <tr>
            <td class="text-nowrap">{{ $t('file_info.geo_location') }}</td>
            <td class="flex flex-row justify-between items-center gap-2 pr-2">
              {{ formatGeoLocation() }}
              <!-- <TButton v-if="fileInfo.gps_latitude && fileInfo.gps_longitude"
                :icon="config.fileInfo.showMap ? IconMapDefault : IconMapOff"
                :buttonSize="'small'"
                @click="config.fileInfo.showMap = !config.fileInfo.showMap"
              /> -->
            </td>
          </tr>
        </tbody>
      </table>

      <!-- Map view -->
      <div class="pl-2 pr-4" v-if="config.fileInfo.showMap && fileInfo?.gps_latitude && fileInfo?.gps_longitude">
        <MapView
          :lat="fileInfo.gps_latitude ? Number(fileInfo.gps_latitude) : 0"
          :lon="fileInfo.gps_longitude ? Number(fileInfo.gps_longitude) : 0"
        />
      </div>
    </div>

  </div>

</template>

<script setup lang="ts">

import { config, formatTimestamp, formatFileSize, formatDuration, formatDimensionText, getFolderPath, formatCaptureSettings } from '@/common/utils';
import { IconClose, IconFile, IconCamera, IconMapDefault, IconMapOff } from '@/common/icons';

import TButton from '@/components/TButton.vue';
import MapView from '@/components/MapView.vue';

const props = defineProps({
  fileInfo: {
    type: Object,
    required: false
  }
});

const emit = defineEmits([
  'close'
]);

function formatGeoLocation() {
  if (props.fileInfo.geo_name) {
    if (props.fileInfo.geo_admin2) {
      return `${props.fileInfo.geo_name}, ${props.fileInfo.geo_admin2}, ${props.fileInfo.geo_admin1}, ${props.fileInfo.geo_cc}`;
    }
    if (props.fileInfo.geo_admin1) {
      return `${props.fileInfo.geo_name}, ${props.fileInfo.geo_admin1}, ${props.fileInfo.geo_cc}`;
    }
    if (props.fileInfo.geo_cc) {
      return `${props.fileInfo.geo_name}, ${props.fileInfo.geo_cc}`;
    }
  }
  return '';
}

// emit close event
function clickClose() {
  emit('close');
}

</script>
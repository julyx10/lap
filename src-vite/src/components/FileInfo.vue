<template>
  <div class="my-1 p-2 w-72 t-color-bg-light rounded-l-lg">
    <!-- Title bar -->
    <div class="h-6 flex items-center justify-between">
      <p class="p-1 font-bold">{{ $t('file_info_title') }}</p>
      <IconClose class="p-1 t-icon-size t-icon-hover" @click="clickClose" />
    </div>

    <!-- File Info table -->
    <div class="overflow-auto t-scrollbar" :style="{ maxHeight: 'calc(100vh - 100px)' }">
      <table v-if="fileInfo" class="text-sm text-nowrap border-separate border-spacing-2">
        <tr>
          <td>{{ $t('file_info_name') }}</td>
          <td class="text-wrap break-all">{{ fileInfo.name }}</td>
        </tr>
        <tr>
          <td>{{ $t('file_info_resolution') }}</td>
          <td>{{ fileInfo.width }}x{{ fileInfo.height }}</td>
        </tr>
        <tr>
          <td>{{ $t('file_info_size') }}</td>
          <td>{{ formatFileSize(fileInfo.size) }}</td>
        </tr>
        <tr>
          <td>{{ $t('file_info_created') }}</td>
          <td>{{ formatTimestamp(fileInfo.created_at, $t('date_time_format')) }}</td>
        </tr>
        <tr>
          <td>{{ $t('file_info_modified') }}</td>
          <td>{{ formatTimestamp(fileInfo.modified_at, $t('date_time_format')) }}</td>
        </tr>
        <tr>
          <td>{{ $t('file_info_camera_make') }}</td>
          <td>{{ fileInfo.e_make }}</td>
        </tr>
        <tr>
          <td>{{ $t('file_info_camera_model') }}</td>
          <td>{{ fileInfo.e_model }}</td>
        </tr>
        <tr>
          <td>{{ $t('file_info_date_taken') }}</td>
          <td>{{ fileInfo.e_date_time }}</td>
        </tr>
        <tr>
          <td>{{ $t('file_info_exposure_time') }}</td>
          <td>{{ fileInfo.e_exposure_time }}</td>
        </tr>
        <tr>
          <td>{{ $t('file_info_aperture') }}</td>
          <td>{{ fileInfo.e_f_number }}</td>
        </tr>
        <tr>
          <td>{{ $t('file_info_focal_length') }}</td>
          <td>{{ fileInfo.e_focal_length }}</td>
        </tr>
        <tr>
          <td>{{ $t('file_info_iso') }}</td>
          <td>{{ fileInfo.e_iso_speed }}</td>
        </tr>
        <tr>
          <td>{{ $t('file_info_flash') }}</td>
          <td>{{ fileInfo.e_flash }}</td>
        </tr>
        <tr>
          <td>{{ $t('file_info_color_type') }}</td>
          <td>{{ fileInfo.i_color_type }}</td>
        </tr>
        <tr>
          <td>{{ $t('file_info_bit_depth') }}</td>
          <td>{{ fileInfo.i_bit_depth }}</td>
        </tr>
        <tr>
          <td>{{ $t('file_info_alpha_channel') }}</td>
          <td>{{ fileInfo.i_has_alpha }}</td>
        </tr>
        <tr>
          <td>{{ $t('file_info_gps_latitude') }}</td>
          <td>{{ fileInfo.gps_latitude }}</td>
        </tr>
        <tr>
          <td>{{ $t('file_info_gps_longitude') }}</td>
          <td>{{ fileInfo.gps_longitude }}</td>
        </tr>
        <tr>
          <td>{{ $t('file_info_gps_altitude') }}</td>
          <td>{{ fileInfo.gps_altitude }}</td>
        </tr>
        <tr>
          <td>{{ $t('file_info_comment') }}</td>
          <td>{{ fileInfo.comments }}</td>
        </tr>        
      </table>
    </div>

  </div>

</template>

<script setup lang="ts">

import { formatTimestamp, formatFileSize } from '@/common/utils';

import IconClose from '@/assets/close.svg';

const props = defineProps({
  fileInfo: {
    type: Object,
    required: true
  }
});

const emit = defineEmits([
  'close'
]);

// emit close event
function clickClose() {
  emit('close');
}

</script>
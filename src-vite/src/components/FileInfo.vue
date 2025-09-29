<template>
  <div class="my-1 p-2 h-full w-full bg-base-200 rounded-l-lg">
    <!-- Title bar -->
    <div class="h-6 flex items-center justify-between">
      <span class="p-1 font-bold text-base-content/30 ">{{ $t('file_info.title') }}</span>
      <TButton
        :icon="IconClose"
        :buttonSize="'small'"
        @click="clickClose"
      />
    </div>

    <!-- File Info table -->
    <div class="overflow-x-hidden overflow-y-auto" :style="{ maxHeight: 'calc(100vh - 100px)' }">
      <table v-if="fileInfo" class="text-sm text-base-content/30 border-separate border-spacing-2">
        <tbody class="align-top">
          <tr>
            <td class="text-nowrap">{{ $t('file_info.album_name') }}</td>
            <td class="text-wrap break-all">{{ fileInfo.album_name }}</td>
          </tr>
          <tr>
            <td class="text-nowrap">{{ $t('file_info.name') }}</td>
            <td class="text-wrap break-all">{{ fileInfo.name }}</td>
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
            <td class="text-nowrap">{{ $t('file_info.size') }}</td>
            <td>{{ formatFileSize(fileInfo.size) }}</td>
          </tr>
          <tr>
            <td class="text-nowrap">{{ $t('file_info.dimension') }}</td>
            <td>{{ fileInfo.width }}x{{ fileInfo.height }}</td>
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
            <td class="text-nowrap">{{ $t('file_info.camera_make') }}</td>
            <td>{{ fileInfo.e_make }}</td>
          </tr>
          <tr>
            <td class="text-nowrap">{{ $t('file_info.camera_model') }}</td>
            <td>{{ fileInfo.e_model }}</td>
          </tr>
          <tr>
            <td class="text-nowrap">{{ $t('file_info.date_taken') }}</td>
            <td>{{ fileInfo.e_date_time }}</td>
          </tr>
          <tr>
            <td class="text-nowrap">{{ $t('file_info.exposure_time') }}</td>
            <td>{{ fileInfo.e_exposure_time }}</td>
          </tr>
          <tr>
            <td class="text-nowrap">{{ $t('file_info.aperture') }}</td>
            <td>{{ fileInfo.e_f_number }}</td>
          </tr>
          <tr>
            <td class="text-nowrap">{{ $t('file_info.focal_length') }}</td>
            <td>{{ fileInfo.e_focal_length }}</td>
          </tr>
          <tr>
            <td class="text-nowrap">{{ $t('file_info.iso') }}</td>
            <td>{{ fileInfo.e_iso_speed }}</td>
          </tr>
          <tr>
            <td class="text-nowrap">{{ $t('file_info.flash') }}</td>
            <td>{{ fileInfo.e_flash }}</td>
          </tr>
          <tr>
            <td class="text-nowrap">{{ $t('file_info.color_type') }}</td>
            <td>{{ fileInfo.i_color_type }}</td>
          </tr>
          <tr>
            <td class="text-nowrap">{{ $t('file_info.bit_depth') }}</td>
            <td>{{ fileInfo.i_bit_depth }}</td>
          </tr>
          <tr>
            <td class="text-nowrap">{{ $t('file_info.alpha_channel') }}</td>
            <td>{{ fileInfo.i_has_alpha }}</td>
          </tr>
          <tr>
            <td class="text-nowrap">{{ $t('file_info.gps_latitude') }}</td>
            <td>{{ fileInfo.gps_latitude }}</td>
          </tr>
          <tr>
            <td class="text-nowrap">{{ $t('file_info.gps_longitude') }}</td>
            <td>{{ fileInfo.gps_longitude }}</td>
          </tr>
          <tr>
            <td class="text-nowrap">{{ $t('file_info.gps_altitude') }}</td>
            <td>{{ fileInfo.gps_altitude }}</td>
          </tr>
          <tr>
            <td class="text-nowrap">{{ $t('file_info.tags') }}</td>
            <td>
              <span v-if="fileInfo.tags && fileInfo.tags.length">
                {{ fileInfo.tags.map(tag => tag.name).join(', ') }}
              </span>
              <!-- <span v-else>{{ $t('tag.not_found') }}</span> -->
            </td>
          </tr>
          <tr>
            <td class="text-nowrap">{{ $t('file_info.comment') }}</td>
            <td class="text-wrap">{{ fileInfo.comments }}</td>
          </tr>
        </tbody>
      </table>
    </div>

  </div>

</template>

<script setup lang="ts">

import { formatTimestamp, formatFileSize, formatDuration, getFolderPath } from '@/common/utils';
import { IconClose } from '@/common/icons';

import TButton from '@/components/TButton.vue';

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
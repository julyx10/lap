<template>
  <div class="my-1 p-2 w-72 t-color-bg-light rounded-l-lg">
    <!-- Title bar -->
    <div class="h-6 flex items-center justify-between">
      <p class="p-1 font-bold">{{ $t('file_info_title') }}</p>
      <IconClose class="p-1 t-icon-hover" @click="clickClose" />
    </div>

    <!-- File Info table -->
    <div class="overflow-auto t-scrollbar" :style="{ maxHeight: 'calc(100vh - 100px)' }">
      <table v-if="fileInfo" class="text-sm text-nowrap border-separate border-spacing-1">
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
          <td>{{ formatTimestamp(fileInfo.created_at) }}</td>
        </tr>
        <tr>
          <td>{{ $t('file_info_modified') }}</td>
          <td>{{ formatTimestamp(fileInfo.modified_at) }}</td>
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
          <td> NYI </td>
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

// import { computed } from 'vue';
import { formatTimestamp, formatFileSize } from '@/common/utils';

import IconClose from '@/assets/close.svg';

/// i18n
// import { useI18n } from 'vue-i18n';
// const { locale, messages } = useI18n();
// const localeMsg = computed(() => messages.value[locale.value]);

const props = defineProps({
  // fileId: {
  //   type: Number,
  //   required: true,
  // },
  fileInfo: {
    type: Object,
    required: true
  }
});

const emit = defineEmits([
  'close'
]);

// File info
// const fileInfo = ref(null);

// Load the file info when the component is mounted
// onMounted(async () => {
//   await loadFileInfo(props.fileId);
// });


// Watch for changes in the file ID
// watch(() => props.fileId, async (newId) => {
//   await loadFileInfo(newId);
// });


// Load the file info from the file ID
// async function loadFileInfo(fileId) {
//   try {
//     fileInfo.value = await invoke('get_file_info', { fileId: parseInt(fileId, 10) });
//     console.log('fileInfo: ---', fileInfo.value);
//   } catch (error) {
//     console.error('Error fetching file info:', error);
//   }
// }


// emit close event
function clickClose() {
  emit('close');
}

</script>
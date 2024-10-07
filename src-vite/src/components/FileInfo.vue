<template>
  <div class="my-1 p-2 w-72 t-color-bg-light rounded-l-lg">
    <!-- Title bar -->
    <div class="h-6 flex items-center justify-between">
      <p class="p-1 font-bold">Information</p>
      <IconClose class="p-1 t-icon-hover" @click="clickClose" />
    </div>

    <!-- File Info table -->
    <div class="overflow-auto t-scrollbar" :style="{ maxHeight: 'calc(100vh - 100px)' }">
      <table v-if="fileInfo" class="text-sm text-nowrap border-separate border-spacing-1">
        <tr>
          <td>File Name</td>
          <td class="text-wrap break-all">{{ fileInfo.name }}</td>
        </tr>
        <tr>
          <td>Resolution</td>
          <td>{{ fileInfo.width }}x{{ fileInfo.height }}</td>
        </tr>
        <tr>
          <td>File Size</td>
          <td>{{ formatFileSize(fileInfo.size) }}</td>
        </tr>
        <tr>
          <td>Created</td>
          <td>{{ formatTimestamp(fileInfo.created_at) }}</td>
        </tr>
        <tr>
          <td>Modified</td>
          <td>{{ formatTimestamp(fileInfo.modified_at) }}</td>
        </tr>
        <tr>
          <td>Camera Make</td>
          <td>{{ fileInfo.e_make }}</td>
        </tr>
        <tr>
          <td>Camera Model</td>
          <td>{{ fileInfo.e_model }}</td>
        </tr>
        <tr>
          <td>Date Taken</td>
          <td>{{ fileInfo.e_date_time }}</td>
        </tr>
        <tr>
          <td>Exposure Time</td>
          <td>{{ fileInfo.e_exposure_time }}</td>
        </tr>
        <tr>
          <td>Aperture</td>
          <td>{{ fileInfo.e_f_number }}</td>
        </tr>
        <tr>
          <td>ISO Speed</td>
          <td>{{ fileInfo.e_iso_speed }}</td>
        </tr>
        <tr>
          <td>Focal Length</td>
          <td>{{ fileInfo.e_focal_length }}</td>
        </tr>
        <tr>
          <td>Color Type</td>
          <td>{{ fileInfo.i_color_type }}</td>
        </tr>
        <tr>
          <td>Bit Depth</td>
          <td>{{ fileInfo.i_bit_depth }}</td>
        </tr>
        <tr>
          <td>Alpha Channel</td>
          <td>{{ fileInfo.i_has_alpha }}</td>
        </tr>
        <tr>
          <td>GPS Latitude</td>
          <td>{{ fileInfo.gps_latitude }}</td>
        </tr>
        <tr>
          <td>GPS Longitude</td>
          <td>{{ fileInfo.gps_longitude }}</td>
        </tr>
        <tr>
          <td>GPS Altitude</td>
          <td>{{ fileInfo.gps_altitude }}</td>
        </tr>
        <tr>
          <td>Comments</td>
          <td>{{ fileInfo.comments }}</td>
        </tr>        
      </table>
    </div>

  </div>

</template>

<script setup lang="ts">

// import { ref, watch, onMounted } from 'vue';
// import { invoke } from '@tauri-apps/api';
import { formatTimestamp, formatFileSize } from '@/common/utils';

import IconClose from '@/assets/close.svg';

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
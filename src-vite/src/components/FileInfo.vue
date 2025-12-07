<template>
  <div class="w-full h-full rounded-box bg-base-200 flex flex-col overflow-hidden">
    <!-- Title bar -->
    <!-- <div class="px-2 py-1 flex items-center justify-between shrink-0">
      <div role="tablist" class="tabs-sm tabs-border" >
        <a 
          role="tab"
          class="tab"
          :class="config.infoPanel.tabIndex === 0 ? 'tab-active' : ''" 
          @click="config.infoPanel.tabIndex = 0"
        >
          {{ $t('info_panel.tabs[0]') }}
        </a>
        <a 
          role="tab"
          class="tab"
          :class="config.infoPanel.tabIndex === 1 ? 'tab-active' : ''" 
          @click="config.infoPanel.tabIndex = 1"
        >
          {{ $t('info_panel.tabs[1]') }}
        </a>
      </div>
    </div> -->

    <!-- Info Content -->
    <!-- <div v-if="config.infoPanel.tabIndex === 0" class="flex-1 overflow-y-auto overflow-x-hidden p-2 space-y-2"> -->
    <div class="flex-1 overflow-y-auto overflow-x-hidden p-2 space-y-2">
      
      <!-- File Info Section -->
      <div class="rounded-xl p-4 space-y-2 border border-base-content/5">
        <div class="flex items-center gap-2 pb-2">
          <IconFileInfo class="w-4 h-4" /> 
          <span class="font-bold text-sm">{{ $t('file_info.file_info') }}</span>
        </div>
        
        <div class="grid grid-cols-[100px_1fr] gap-y-3 gap-x-4 text-sm">
          <!-- Album -->
          <div class="font-medium text-base-content/50 tracking-wide">{{ $t('file_info.album_name') }}</div>
          <div class="text-base-content break-all">{{ fileInfo?.album_name }}</div>

          <!-- Path -->
          <div class="font-medium text-base-content/50 tracking-wide">{{ $t('file_info.path') }}</div>
          <div class="text-base-content break-all">{{ getFolderPath(fileInfo?.file_path) }}</div>

          <!-- Name -->
          <div class="font-medium text-base-content/50 tracking-wide">{{ $t('file_info.name') }}</div>
          <div class="text-base-content break-all font-medium">{{ fileInfo?.name }}</div>

          <!-- Size -->
          <div class="font-medium text-base-content/50 tracking-wide">{{ $t('file_info.size') }}</div>
          <div class="text-base-content">{{ formatFileSize(fileInfo?.size) }}</div>

          <!-- Dimension -->
          <div class="font-medium text-base-content/50 tracking-wide">{{ $t('file_info.dimension') }}</div>
          <div class="text-base-content">{{ formatDimensionText(fileInfo?.width, fileInfo?.height) }}</div>

          <!-- Duration -->
          <template v-if="fileInfo?.file_type === 2">
            <div class="font-medium text-base-content/50 tracking-wide">{{ $t('file_info.duration') }}</div>
            <div class="text-base-content">{{ formatDuration(fileInfo?.duration) }}</div>
          </template>

          <!-- Created At -->
          <div class="font-medium text-base-content/50 tracking-wide">{{ $t('file_info.created_at') }}</div>
          <div class="text-base-content">{{ formatTimestamp(fileInfo?.created_at, $t('format.date_time')) }}</div>

          <!-- Modified At -->
          <div class="font-medium text-base-content/50 tracking-wide">{{ $t('file_info.modified_at') }}</div>
          <div class="text-base-content">{{ formatTimestamp(fileInfo?.modified_at, $t('format.date_time')) }}</div>

          <!-- Tags -->
          <div class="font-medium text-base-content/50 tracking-wide">{{ $t('file_info.tags') }}</div>
          <div class="text-base-content flex flex-wrap gap-1">
            <template v-if="fileInfo?.tags && fileInfo.tags.length">
              <span v-for="tag in fileInfo.tags" :key="tag.id" class="badge">{{ tag.name }}</span>
            </template>
          </div>

          <!-- Comment -->
          <div class="font-medium text-base-content/50 tracking-wide">{{ $t('file_info.comment') }}</div>
          <div class="text-base-content break-words whitespace-pre-wrap">{{ fileInfo?.comments }}</div>
        </div>
      </div>

      <!-- Metadata Section -->
      <div class="rounded-xl p-4 space-y-3 border border-base-content/5">
        <div class="flex items-center gap-2 pb-2">
          <IconCamera class="w-4 h-4" /> 
          <span class="font-bold text-sm">{{ $t('file_info.metadata') }}</span>
        </div>

        <div class="grid grid-cols-[100px_1fr] gap-y-3 gap-x-4 text-sm">
          <!-- Camera -->
          <div class="font-medium text-base-content/50 tracking-wide">{{ $t('file_info.camera') }}</div>
          <div class="text-base-content">{{ formatCameraInfo(fileInfo?.e_make, fileInfo?.e_model) }}</div>

          <!-- Lens -->
          <div class="font-medium text-base-content/50 tracking-wide">{{ $t('file_info.lens') }}</div>
          <div class="text-base-content">{{ fileInfo?.e_lens_model }}</div>

          <!-- Capture Settings -->
          <div class="font-medium text-base-content/50 tracking-wide">{{ $t('file_info.capture_settings') }}</div>
          <div class="text-base-content">{{ formatCaptureSettings(fileInfo?.e_focal_length, fileInfo?.e_exposure_time, fileInfo?.e_f_number, fileInfo?.e_iso_speed, fileInfo?.e_exposure_bias) }}</div>

          <!-- Software -->
          <div class="font-medium text-base-content/50 tracking-wide">{{ $t('file_info.software') }}</div>
          <div class="text-base-content">{{ fileInfo?.e_software }}</div>

          <!-- Taken By -->
          <div class="font-medium text-base-content/50 tracking-wide">{{ $t('file_info.taken_by') }}</div>
          <div class="text-base-content">{{ fileInfo?.e_artist }}</div>

          <!-- Copyright -->
          <div class="font-medium text-base-content/50 tracking-wide">{{ $t('file_info.copyright') }}</div>
          <div class="text-base-content">{{ fileInfo?.e_copyright }}</div>

          <!-- Taken At -->
          <div class="font-medium text-base-content/50 tracking-wide">{{ $t('file_info.taken_at') }}</div>
          <div class="text-base-content">{{ fileInfo?.e_date_time }}</div>

          <!-- Description -->
          <div class="font-medium text-base-content/50 tracking-wide">{{ $t('file_info.description') }}</div>
          <div class="text-base-content break-words">{{ fileInfo?.e_description }}</div>

          <!-- Geo Location -->
          <div class="font-medium text-base-content/50 tracking-wide">{{ $t('file_info.geo_location') }}</div>
          <div class="text-base-content">{{ formatGeoLocation() }}</div>
        </div>
      </div>

      <!-- Map View -->
      <div v-if="config.fileInfo.showMap && fileInfo?.gps_latitude && fileInfo?.gps_longitude" class="">
        <MapView
          :lat="fileInfo.gps_latitude ? Number(fileInfo.gps_latitude) : 0"
          :lon="fileInfo.gps_longitude ? Number(fileInfo.gps_longitude) : 0"
        />
      </div>
    </div>

    <!-- Preview -->
    <!-- <div v-if="config.infoPanel.tabIndex === 1" ref="previewDiv" 
      class="flex-1 rounded-box overflow-hidden"
      @dblclick="infoPanelZoomFit = !infoPanelZoomFit"
    >
      <Image v-if="fileInfo?.file_type === 1"
        :filePath="fileInfo?.file_path"
        :rotate="fileInfo?.rotate ?? 0" 
        :isZoomFit="infoPanelZoomFit"
      ></Image>

      <Video v-if="fileInfo?.file_type === 2"
        :filePath="fileInfo?.file_path"
        :rotate="fileInfo?.rotate ?? 0"
        :isZoomFit="infoPanelZoomFit"
      ></Video>
    </div> -->

  </div>

</template>

<script setup lang="ts">
import { ref } from 'vue';
import { config } from '@/common/config';
import { formatTimestamp, formatFileSize, formatDuration, formatDimensionText, getFolderPath, formatCaptureSettings, formatCameraInfo } from '@/common/utils';
import { IconFileInfo, IconCamera } from '@/common/icons';

import MapView from '@/components/MapView.vue';
// import Image from '@/components/Image.vue';
// import Video from '@/components/Video.vue';

const props = defineProps({
  fileInfo: {
    type: Object,
    required: false
  },
});

const infoPanelZoomFit = ref(true);

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
    info.geo_cc,
  ];

  return fields.filter(Boolean).join(", ");
}

// emit close event
// function clickClose() {
//   emit('close');
// }

</script>
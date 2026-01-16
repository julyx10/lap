<template>

  <div class="w-full h-full flex flex-col" style="user-select: none;">
    
    <!-- title bar -->
    <div class="p-1 h-12 flex items-start justify-end whitespace-nowrap" data-tauri-drag-region>
      <TButton
        :icon="config.camera.sortCount ? IconSortingCount : IconSortingName"
        @click="config.camera.sortCount = !config.camera.sortCount"
      />
    </div>

    <!-- camera -->
    <div v-if="cameras.length > 0" class="flex-1 overflow-x-hidden overflow-y-auto">
      <ul>
        <li v-for="camera in sortedCameras">
          <div 
            :class="[
              'mx-1 p-1 h-10 flex items-center rounded-box whitespace-nowrap cursor-pointer group', 
              libConfig.camera.make === camera.make && !libConfig.camera.model ? 'text-primary bg-base-100 hover:bg-base-100' : 'hover:text-base-content hover:bg-base-100/30',
            ]"
            @click="clickCameraMake(camera)"
          >
            <IconRight
              :class="[
                'p-1 w-6 h-6 shrink-0 transition-transform', 
                camera.is_expanded ? 'rotate-90' : ''
              ]"
              @click.stop="clickExpandCamera(camera)"
            />
            <span class="flex-1 overflow-hidden whitespace-pre text-ellipsis">{{ camera.make }}</span>
            <span class="mx-1 text-xs tabular-nums text-base-content/30">{{ camera.counts.reduce((a, b) => a + b, 0).toLocaleString() }}</span>
          </div>
          <ul v-if="camera.is_expanded && camera.models.length > 0">
            <li v-for="(model, index) in camera.models" class="pl-4"> 
              <div 
                :class="[
                  'ml-3 mr-1 p-1 h-8 flex items-center rounded-box whitespace-nowrap cursor-pointer group', 
                  libConfig.camera.model === model ? 'text-primary bg-base-100 hover:bg-base-100' : 'hover:text-base-content hover:bg-base-100/30',
                ]" 
                @click="clickCameraModel(camera.make, model)"
              >
                <div class="px-1 whitespace-pre text-ellipsis overflow-hidden">
                  <span>{{ model }}</span>
                  <span class="text-[10px] tabular-nums text-base-content/30 ml-1">({{ camera.counts[index].toLocaleString() }})</span>
                </div>
              </div>
            </li>
          </ul>
        </li>
      </ul>
    </div>

    <!-- Display message if no data are found -->
    <div v-else class="mt-8 px-2 flex flex-col items-center justify-center text-base-content/30">
      <!-- <IconCamera class="w-8 h-8" /> -->
      <span class="text-sm">{{ $t('tooltip.not_found.camera') }}</span>
    </div>
  </div>

</template>


<script setup lang="ts">

import { ref, onMounted, computed } from 'vue';
import { config, libConfig } from '@/common/config';
import { getCameraInfo } from '@/common/api';
import { IconRight, IconSortingCount, IconSortingName } from '@/common/icons';
import TButton from '@/components/TButton.vue';

const props = defineProps({
  titlebar: {
    type: String,
    required: true
  }
});

const cameras = ref<any[]>([]);

const sortedCameras = computed(() => {
  if (config.camera.sortCount) {
    return [...cameras.value].sort((a, b) => {
      const countA = (a.counts || []).reduce((sum: number, c: number) => sum + c, 0);
      const countB = (b.counts || []).reduce((sum: number, c: number) => sum + c, 0);
      return countB - countA;
    });
  }
  return cameras.value;
});

onMounted(async () => {
  if (cameras.value.length === 0) {
    await getCameras();

    if (cameras.value.length === 0) {
      (libConfig.camera as any).make = null;
      (libConfig.camera as any).model = null;
    }

    if(libConfig.camera.make && libConfig.camera.model) {
      let camera = cameras.value.find(camera => camera.make === libConfig.camera.make)
      if(camera) {
        camera.is_expanded = true;     // expand selected camera
      } else {
        (libConfig.camera as any).make = null;
        (libConfig.camera as any).model = null;
      }
    }
  }
});

/// click camera icon to expand or collapse models
function clickExpandCamera(camera: any) {
  camera.is_expanded = !camera.is_expanded; 
};

/// click a camera to select it
function clickCameraMake(camera: any) {
  (libConfig.camera as any).make = camera.make;
  (libConfig.camera as any).model = null;

  camera.is_expanded = true;
}

/// click a camera to select it
function clickCameraModel(make: string, model: string) {
  (libConfig.camera as any).make = make;
  (libConfig.camera as any).model = model;
}

/// get cameras from db
async function getCameras() {
  const fetchedCameras = await getCameraInfo();
  if (fetchedCameras) {
    cameras.value = fetchedCameras.map((camera: any) => ({
      ...camera, 
      is_expanded: false,
    }));
  }
};

</script>
<template>

  <div class="w-full flex flex-col" style="user-select: none;">
    
    <!-- title bar -->
    <div class="px-2 py-3 h-12 flex items-center justify-between whitespace-nowrap" data-tauri-drag-region>
      <span class="cursor-default" data-tauri-drag-region>{{ titlebar }}</span>
      <!-- <TButton :icon="IconRefresh" @click="clickReload"/> -->
    </div>

    <!-- camera -->
    <div v-if="cameras.length > 0" class="flex-1 overflow-x-hidden overflow-y-auto">
      <ul>
        <li v-for="camera in cameras">
          <div 
            :class="[
              'my-1 mr-1 h-8 flex items-center rounded border-l-2 border-base-200 hover:bg-base-content/10 whitespace-nowrap cursor-pointer', 
              { 
                'text-base-content': config.camera.make === camera.make, 
                'bg-base-content/10 border-primary'  : config.camera.make === camera.make && !config.camera.model 
              }
            ]"
            @click="clickCameraMake(camera)"
          >
            <!-- <component :is="camera.is_expanded ? IconFolderExpanded : IconFolder" class="mx-1 h-5  shrink-0" @click.stop="clickExpandCamera(camera)"/> -->
            <IconCamera
              :class="[
                'mx-1 h-5 shrink-0 transition-transform', 
              ]"
              @click.stop="clickExpandCamera(camera)"
            />
            <div class="overflow-hidden whitespace-pre text-ellipsis">
             {{ camera.make }}
            </div>
          </div>
          <ul v-if="camera.is_expanded && camera.models.length > 0">
            <li v-for="model in camera.models" class="pl-4">
              <div 
                :class="[
                  'm-1 pl-3 flex items-center rounded border-l-2 border-base-200 hover:bg-base-content/10 whitespace-nowrap cursor-pointer', 
                  config.camera.model === model ? 'bg-base-content/10 border-primary' : ''
                ]" 
                @click="clickCameraModel(camera.make, model)"
              >
                <!-- <IconCircle class="mx-1 h-5 shrink-0" /> -->
                <div class="overflow-hidden whitespace-pre text-ellipsis">
                  {{ model }}
                </div>
              </div>
            </li>
          </ul>
        </li>
      </ul>
    </div>

    <!-- Display message if no data are found -->
    <div v-else class="mt-10 flex flex-col items-center justify-center text-base-content/30">
      <IconCamera class="w-8 h-8" />
      <span class="mt-2">{{ $t('tooltip.not_found.camera') }}</span>
    </div>
  </div>

</template>


<script setup lang="ts">

import { ref, onMounted } from 'vue';
import { config } from '@/common/config';
import { getCameraInfo } from '@/common/api';
import { IconRight, IconCamera } from '@/common/icons';

const props = defineProps({
  titlebar: {
    type: String,
    required: true
  }
});

const cameras = ref([]);

onMounted(async () => {
  if (cameras.value.length === 0) {
    await getCameras();

    if(config.camera.make && config.camera.model) {
      let camera = cameras.value.find(camera => camera.make === config.camera.make)
      if(camera) {
        camera.is_expanded = true;     // expand selected camera
      } else {
        config.camera.make = null;
        config.camera.model = null;
      }
    }
  }
});

/// reload cameras
function clickReload() {
  getCameras();
  config.camera.make = "";
  config.camera.model = "";
};

/// click camera icon to expand or collapse models
function clickExpandCamera(camera) {
  camera.is_expanded = !camera.is_expanded; 
};

/// click a camera to select it
function clickCameraMake(camera) {
  config.camera.make = camera.make;
  config.camera.model = "";

  camera.is_expanded = true;
}

/// click a camera to select it
function clickCameraModel(make, model) {
  config.camera.make = make;
  config.camera.model = model;
}

/// get cameras from db
async function getCameras() {
  const fetchedCameras = await getCameraInfo();
  if (fetchedCameras) {
    cameras.value = fetchedCameras.map(camera => ({
      ...camera, 
      is_expanded: false,
    }));
  }
};

</script>
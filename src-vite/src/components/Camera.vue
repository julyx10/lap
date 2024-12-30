<template>

  <div class="flex-1 flex flex-col overflow-auto" style="user-select: none;">
    
    <!-- title bar -->
    <div class="px-2 py-3 h-12 flex items-center justify-between">
      <span>{{ titlebar }}</span>

      <div class="flex">
        <IconReload class="t-icon-size-sm t-icon-hover" @click="clickReload"/>
      </div>
    </div>

    <!-- camera -->
    <div v-if="cameras.length > 0" class="flex-grow overflow-auto t-scrollbar">
      <ul>
        <li v-for="camera in cameras">
          <div 
            :class="[
              'm-1 h-8 border-l-2 flex items-center whitespace-nowrap border-transparent t-color-bg-hover cursor-pointer', 
              { 
                't-color-text-selected': config.cameraMake === camera.make, 
                't-color-bg-selected t-color-border-selected transition-colors duration-300'  : config.cameraMake === camera.make && !config.cameraModel 
              }
            ]"
            @click="clickCameraMake(camera)"
          >
            <component :is="camera.is_expanded ? IconFolderOpen : IconFolder" class="mx-1 h-5  flex-shrink-0" @click.stop="clickExpandCamera(camera)"/>
            {{ camera.make }}
          </div>
          <ul v-if="camera.is_expanded && camera.models.length > 0">
            <li v-for="model in camera.models" class="pl-4">
              <div 
                :class="[
                  'm-1 border-l-2 flex items-center whitespace-nowrap t-color-bg-hover cursor-pointer', 
                  config.cameraModel === model ? 't-color-text-selected t-color-bg-selected t-color-border-selected transition-colors duration-300' : 'border-gray-900'
                ]" 
                @click="clickCameraModel(camera.make, model)"
              >
                <IconCamera class="mx-1 h-5 flex-shrink-0" />
                {{ model }}
              </div>
            </li>
          </ul>
        </li>
      </ul>
    </div>

    <!-- Display message if no data are found -->
    <div v-else class="mt-10 flex items-center justify-center">
      {{ $t('no_cameras_data') }}
    </div>

  </div>

</template>


<script setup lang="ts">

import { ref, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { useConfigStore } from '@/stores/configStore';

// toolbar icons
import IconReload from '@/assets/reload.svg';
import IconFolder from '@/assets/folder.svg';
import IconFolderOpen from '@/assets/folder-open.svg';
import IconRight from '@/assets/arrow-right.svg';
import IconCamera from '@/assets/camera.svg';

const props = defineProps({
  titlebar: {
    type: String,
    required: true
  }
});

// config store
const config = useConfigStore();

const cameras = ref([]);

onMounted(async () => {
  if (cameras.value.length === 0) {
    await getCameras();

    if(config.cameraMake && config.cameraModel) {
      let camera = cameras.value.find(camera => camera.make === config.cameraMake)
      if(camera) {
        camera.is_expanded = true;     // expand selected camera
      } else {
        config.cameraMake = null;
        config.cameraModel = null;
      }
    }
  }
});

/// reload cameras
function clickReload() {
  getCameras();
  config.cameraMake = null;
  config.cameraModel = null;
};

/// click camera icon to expand or collapse models
function clickExpandCamera(camera) {
  camera.is_expanded = !camera.is_expanded; 
};

/// click a camera to select it
function clickCameraMake(camera) {
  config.cameraMake = camera.make;
  config.cameraModel = null;

  camera.is_expanded = true;
}

/// click a camera to select it
function clickCameraModel(make, model) {
  config.cameraMake = make;
  config.cameraModel = model;
}

/// get cameras from db
async function getCameras() {
  try {
    const fetchedCameras = await invoke('get_camera_info');
    if (fetchedCameras) {
      cameras.value = fetchedCameras.map(camera => ({
        ...camera, 
        is_expanded: false,
      }));
    }
    console.log('getCameras...', cameras.value);
  } catch (error) {
    console.error('Failed to fetch camera info:', error);
  }
};

</script>
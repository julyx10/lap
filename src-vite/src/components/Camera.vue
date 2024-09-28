<template>
  
  <!-- title bar -->
  <div class="absolute flex flex-row px-2 py-3 items-center justify-between w-full" style="user-select: none;">
    <div>
      {{ titlebar }}
    </div>
    <div class="flex flex-row h-6">
      <IconRefresh class="p-1 hover:text-gray-200 transition-colors duration-300" @click="clickRefresh"/>
    </div>
  </div>

  <!-- camera -->
  <div v-if="gCameras.length > 0" class="flex-1 mt-12 overflow-auto scrollbar-thin scrollbar-thumb-gray-700 scrollbar-track-gray-800">
    <ul>
      <li v-for="camera in gCameras" style="user-select: none;" >
        <div 
          :class="[
            'p-2 flex items-center whitespace-nowrap hover:bg-gray-700', 
            { 
              'text-gray-300': gCameraMake === camera.make, 
              'bg-gray-800': gCameraMake === camera.make && !gCameraModel 
            }
          ]"
          @click="clickCameraMake(camera)"
        >
          <component :is="camera.is_expanded ? IconFolderOpen : IconFolder" class="size-6 pr-1 flex-shrink-0" @click.stop="clickExpandCamera(camera)"/>
          {{ camera.make }}
        </div>
        <ul v-if="camera.is_expanded && camera.models.length > 0">
          <li v-for="model in camera.models" class="pl-4">
            <div 
              :class="[
                'm-1 flex items-center whitespace-nowrap hover:bg-gray-700', 
                gCameraModel === model ? 'text-gray-300 bg-gray-800' : ''
              ]" 
              @click="clickCameraModel(camera.make, model)"
            >
              <IconMinus class="p-1 flex-shrink-0" />
              {{ model }}
            </div>
          </li>
        </ul>
      </li>
    </ul>
  </div>

  <!-- Display message if no albums are found -->
  <div v-else class="flex items-center justify-center w-full">
    {{ $t('no_cameras') }}
  </div>

</template>


<script setup lang="ts">

import { ref, inject, computed, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api';
import { appWindow } from '@tauri-apps/api/window';

/// i18n
import { useI18n } from 'vue-i18n';
const { locale, messages } = useI18n();
const localeMessages = computed(() => messages.value[locale.value]);

// toolbar icons
import IconRefresh from '@/assets/arrow-path.svg';

// folder icon
import IconFolder from '@/assets/folder.svg';
import IconFolderOpen from '@/assets/folder-open.svg';

import IconMinus from '@/assets/minus.svg';

const gCameras = inject('gCameras');
const gCameraMake = inject('gCameraMake');
const gCameraModel = inject('gCameraModel');

const props = defineProps({
  titlebar: {
    type: String,
    required: true
  }
});

// const cameras = ref([]);

onMounted(() => {
  if (gCameras.value.length === 0) {
    clickRefresh();
  }
});


/// refresh cameras
const clickRefresh = async () => {
  await getCameras();
  console.log('get cameras...');
};


/// get cameras from db
async function getCameras() {
  try {
    const fetchedCameras = await invoke('get_camera_info');
    if (fetchedCameras) {
      gCameras.value = fetchedCameras.map(camera => ({
        ...camera, 
        is_expanded: false,
      }));

      gCameraMake.value = null;
      gCameraModel.value = null;
    }
    console.log('getCameras...', gCameras.value);

  } catch (error) {
    console.error('Failed to fetch camera info:', error);
  }
};


/// click camera icon to expand or collapse models
function clickExpandCamera(camera) {
  console.log('clickExpandCamera...', camera);
  camera.is_expanded = !camera.is_expanded; 
};

/// click a camera to select it
function clickCameraMake(camera) {
  console.log('clickCameraMake...', camera);
  gCameraMake.value = camera.make;
  gCameraModel.value = null;
}

/// click a camera to select it
function clickCameraModel(make, model) {
  console.log('clickCameraModel...', make, model);
  gCameraMake.value = make;
  gCameraModel.value = model;
}

</script>
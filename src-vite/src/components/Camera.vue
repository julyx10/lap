<template>
  
  <!-- title bar -->
  <div class="absolute px-2 py-3 w-full flex flex-row items-center justify-between" style="user-select: none;">
    <div>
      {{ titlebar }}
    </div>
    <div class="h-6 flex flex-row ">
      <IconRefresh class="p-1 t-icon-hover" @click="clickRefresh"/>
    </div>
  </div>

  <!-- camera -->
  <div v-if="gCameras.length > 0" class="flex-1 mt-12 overflow-auto t-scrollbar">
    <ul>
      <li v-for="camera in gCameras" style="user-select: none;" >
        <div 
          :class="[
            'p-2 flex items-center whitespace-nowrap t-color-bg-hover', 
            { 
              't-color-text-selected': gCameraMake === camera.make, 
              't-color-bg-selected'  : gCameraMake === camera.make && !gCameraModel 
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
                'm-1 border-l-2 flex items-center whitespace-nowrap t-color-bg-hover', 
                gCameraModel === model ? 't-color-text-selected t-color-bg-selected border-sky-500 transition-colors duration-300' : 'border-gray-900'
              ]" 
              @click="clickCameraModel(camera.make, model)"
            >
              <IconRight class="p-1 flex-shrink-0" />
              {{ model }}
            </div>
          </li>
        </ul>
      </li>
    </ul>
  </div>

  <!-- Display message if no albums are found -->
  <div v-else class="w-full flex items-center justify-center ">
    {{ $t('no_cameras') }}
  </div>

</template>


<script setup lang="ts">

import { ref, watch, inject, computed, onMounted } from 'vue';
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

import IconRight from '@/assets/chevron-right.svg';

const gSelectItemIndex = inject('gSelectItemIndex');

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

watch(gCameraMake, async (newMake) => {
  if (newMake) {
    console.log('watch gCameraMake...', newMake);
    gSelectItemIndex.value = -1;
  }
});

watch(gCameraModel, async (newModel) => {
  if (newModel) {
    console.log('watch gCameraModel...', newModel);
    gSelectItemIndex.value = -1;
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

      gSelectItemIndex.value = -1;

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
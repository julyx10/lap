<template>

  <div class="sidebar-panel">
    <div class="sidebar-panel-header">
      <span class="sidebar-panel-header-title">{{ localeMsg.sidebar.camera }}</span>
      <ContextMenu class="sidebar-panel-action" :menuItems="cameraPanelMenuItems" :iconMenu="IconMore" :smallIcon="true" />
    </div>

    <!-- camera -->
    <div v-if="cameras.length > 0" class="flex-1 overflow-x-hidden overflow-y-auto">
      <ul>
        <li v-for="camera in sortedCameras">
          <div
            :class="[
              'sidebar-item',
              libConfig.camera.make === camera.make && !libConfig.camera.model ? 'sidebar-item-selected' : 'sidebar-item-hover',
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
            <span class="sidebar-item-label">{{ camera.make }}</span>
            <span class="sidebar-item-count">{{ camera.counts.reduce((a: number, b: number) => a + b, 0).toLocaleString() }}</span>
          </div>
          <ul v-if="camera.is_expanded && camera.models.length > 0">
            <li v-for="(model, index) in camera.models" class="pl-4"> 
              <div
                :class="[
                  'sidebar-item sidebar-item-compact ml-3',
                  libConfig.camera.model === model ? 'sidebar-item-selected' : 'sidebar-item-hover',
                ]"
                @click="clickCameraModel(camera.make, model)"
              >
                <span class="sidebar-item-label">{{ model }}</span>
                <span class="text-[10px] tabular-nums text-base-content/30 ml-1">({{ camera.counts[index].toLocaleString() }})</span>
              </div>
            </li>
          </ul>
        </li>
      </ul>
    </div>

    <!-- Display message if no data are found -->
    <div v-else class="mt-8 px-2 flex flex-col items-center justify-center text-base-content/30">
        <IconCamera class="w-8 h-8 mb-2" />
        <span class="text-sm text-center">{{ $t('tooltip.not_found.camera') }}</span>
    </div>
  </div>

</template>


<script setup lang="ts">

import { ref, onMounted, computed } from 'vue';
import { useI18n } from 'vue-i18n';
import { config, libConfig } from '@/common/config';
import { getCameraInfo } from '@/common/api';
import { IconCamera, IconDot, IconMore, IconRight } from '@/common/icons';
import ContextMenu from '@/components/ContextMenu.vue';

const props = defineProps({
  titlebar: {
    type: String,
    required: true
  }
});

const { locale, messages } = useI18n();
const localeMsg = computed(() => messages.value[locale.value] as any);

const cameras = ref<any[]>([]);

const sortedCameras = computed(() => {
  if (config.leftPanel.sortCount) {
    return [...cameras.value].sort((a, b) => {
      const countA = (a.counts || []).reduce((sum: number, c: number) => sum + c, 0);
      const countB = (b.counts || []).reduce((sum: number, c: number) => sum + c, 0);
      return countB - countA;
    });
  }
  return cameras.value;
});

const cameraPanelMenuItems = computed(() => [
  {
    label: localeMsg.value.menu.sort.sort_by_name,
    icon: config.leftPanel.sortCount ? null : IconDot,
    action: () => { config.leftPanel.sortCount = false; },
  },
  {
    label: localeMsg.value.menu.sort.sort_by_count,
    icon: config.leftPanel.sortCount ? IconDot : null,
    action: () => { config.leftPanel.sortCount = true; },
  },
]);

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

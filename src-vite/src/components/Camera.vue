<template>

  <div class="sidebar-panel">
    <div class="sidebar-panel-header">
      <div role="tablist" class="sidebar-header-tabs">
        <button
          role="tab"
          :class="['sidebar-header-tab', activeTab === 'camera' ? 'tab-active' : '']"
          @click="setActiveTab('camera')"
        >
          {{ localeMsg.sidebar.camera }}
        </button>
        <button
          role="tab"
          :class="['sidebar-header-tab', activeTab === 'lens' ? 'tab-active' : '']"
          @click="setActiveTab('lens')"
        >
          {{ $t('file_info.lens') }}
        </button>
      </div>
    </div>

    <div v-if="activeItems.length > 0" class="flex-1 overflow-x-hidden overflow-y-auto">
      <ul>
        <li v-for="item in sortedItems" :key="item.make">
          <div
            :class="[
              'sidebar-item',
              isMakeSelected(item.make) ? 'sidebar-item-selected' : 'sidebar-item-hover',
            ]"
            @click="clickMake(item)"
          >
            <IconRight
              :class="[
                'p-1 w-6 h-6 shrink-0 transition-transform',
                item.is_expanded ? 'rotate-90' : ''
              ]"
              @click.stop="clickExpand(item)"
            />
            <span class="sidebar-item-label">{{ item.make }}</span>
            <span class="sidebar-item-count">{{ item.counts.reduce((a: number, b: number) => a + b, 0).toLocaleString() }}</span>
          </div>
          <ul v-if="item.is_expanded && item.models.length > 0">
            <li v-for="(model, index) in item.models" :key="`${item.make}-${model}`" class="pl-4">
              <div
                :class="[
                  'sidebar-item sidebar-item-compact ml-3',
                  isModelSelected(model) ? 'sidebar-item-selected' : 'sidebar-item-hover',
                ]"
                @click="clickModel(item.make, model)"
              >
                <span class="sidebar-item-label">{{ model }}</span>
                <span class="text-[10px] tabular-nums text-base-content/30 ml-1">({{ item.counts[index].toLocaleString() }})</span>
              </div>
            </li>
          </ul>
        </li>
      </ul>
    </div>

    <!-- Display message if no data are found -->
    <div v-else class="mt-2 px-2 flex flex-col items-center justify-center text-base-content/30">
      <!-- <IconCamera class="w-8 h-8 mb-2" /> -->
      <span class="text-sm text-center">{{ $t('tooltip.not_found.camera_hint') }}</span>
    </div>
  </div>

</template>


<script setup lang="ts">

import { ref, onMounted, computed, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import { config, libConfig } from '@/common/config';
import { getCameraInfo, getLensInfo } from '@/common/api';
import { IconRight } from '@/common/icons';

const props = defineProps({
  titlebar: {
    type: String,
    required: true
  }
});

const { locale, messages } = useI18n();
const localeMsg = computed(() => messages.value[locale.value] as any);

const cameras = ref<any[]>([]);
const lenses = ref<any[]>([]);

const activeTab = computed(() => {
  return (libConfig.camera as any).tab === 'lens' ? 'lens' : 'camera';
});

const activeItems = computed(() => {
  return activeTab.value === 'lens' ? lenses.value : cameras.value;
});

const sortedItems = computed(() => activeItems.value);

onMounted(async () => {
  if ((libConfig.camera as any).tab !== 'lens' && (libConfig.camera as any).tab !== 'camera') {
    (libConfig.camera as any).tab = 'camera';
  }

  if (cameras.value.length === 0 || lenses.value.length === 0) {
    await Promise.all([getCameras(), getLenses()]);
  }

  validateSelections();
  expandSelectedItem(cameras.value, (libConfig.camera as any).make, (libConfig.camera as any).model);
  expandSelectedItem(lenses.value, (libConfig.camera as any).lensMake, (libConfig.camera as any).lensModel);
});

watch(() => config.settings.categorySort, async () => {
  await Promise.all([getCameras(), getLenses()]);
});

function restoreExpandedItem(items: any[], selectedMake: string | null, selectedModel: string | null) {
  if (!selectedMake) return;

  const item = items.find(data => data.make === selectedMake);
  if (!item) return;

  item.is_expanded = true;

  if (selectedModel && !item.models.includes(selectedModel)) {
    if (items === cameras.value) {
      (libConfig.camera as any).model = null;
    } else {
      (libConfig.camera as any).lensModel = null;
    }
  }
}

function setActiveTab(tab: 'camera' | 'lens') {
  (libConfig.camera as any).tab = tab;
}

function isMakeSelected(make: string) {
  if (activeTab.value === 'lens') {
    return (libConfig.camera as any).lensMake === make && !(libConfig.camera as any).lensModel;
  }
  return (libConfig.camera as any).make === make && !(libConfig.camera as any).model;
}

function isModelSelected(model: string) {
  if (activeTab.value === 'lens') {
    return (libConfig.camera as any).lensModel === model;
  }
  return (libConfig.camera as any).model === model;
}

function clickExpand(item: any) {
  item.is_expanded = !item.is_expanded;
}

function clickMake(item: any) {
  if (activeTab.value === 'lens') {
    (libConfig.camera as any).lensMake = item.make;
    (libConfig.camera as any).lensModel = null;
  } else {
    (libConfig.camera as any).make = item.make;
    (libConfig.camera as any).model = null;
  }
  item.is_expanded = true;
}

function clickModel(make: string, model: string) {
  if (activeTab.value === 'lens') {
    (libConfig.camera as any).lensMake = make;
    (libConfig.camera as any).lensModel = model;
  } else {
    (libConfig.camera as any).make = make;
    (libConfig.camera as any).model = model;
  }
}

/// get cameras from db
async function getCameras() {
  const fetchedCameras = await getCameraInfo(config.settings.categorySort);
  if (fetchedCameras) {
    cameras.value = fetchedCameras.map((camera: any) => ({
      ...camera, 
      is_expanded: false,
    }));
    restoreExpandedItem(cameras.value, (libConfig.camera as any).make, (libConfig.camera as any).model);
  }
}

async function getLenses() {
  const fetchedLenses = await getLensInfo(config.settings.categorySort);
  if (fetchedLenses) {
    lenses.value = fetchedLenses.map((lens: any) => ({
      ...lens,
      is_expanded: false,
    }));
    restoreExpandedItem(lenses.value, (libConfig.camera as any).lensMake, (libConfig.camera as any).lensModel);
  }
}

function expandSelectedItem(items: any[], selectedMake: string | null, selectedModel: string | null) {
  if (!selectedMake || !selectedModel) return;
  const item = items.find(data => data.make === selectedMake);
  if (item) item.is_expanded = true;
}

function validateSelections() {
  if (cameras.value.length === 0) {
    (libConfig.camera as any).make = null;
    (libConfig.camera as any).model = null;
  } else if ((libConfig.camera as any).make) {
    const hasMake = cameras.value.some(camera => camera.make === (libConfig.camera as any).make);
    if (!hasMake) {
      (libConfig.camera as any).make = null;
      (libConfig.camera as any).model = null;
    }
  }

  if (lenses.value.length === 0) {
    (libConfig.camera as any).lensMake = null;
    (libConfig.camera as any).lensModel = null;
    if (activeTab.value === 'lens') {
      (libConfig.camera as any).tab = 'camera';
    }
  } else if ((libConfig.camera as any).lensMake) {
    const hasLensMake = lenses.value.some(lens => lens.make === (libConfig.camera as any).lensMake);
    if (!hasLensMake) {
      (libConfig.camera as any).lensMake = null;
      (libConfig.camera as any).lensModel = null;
    }
  }
}

</script>

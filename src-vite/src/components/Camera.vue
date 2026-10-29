<template>

  <div class="sidebar-panel">
    <div class="sidebar-panel-header">
      <div class="sidebar-header-tabs" role="tablist" :aria-label="props.titlebar">
        <button
          v-for="tab in cameraTabs"
          :key="tab.value"
          type="button"
          role="tab"
          class="sidebar-header-tab"
          :class="{ 'tab-active': activeTab === tab.value }"
          :aria-selected="activeTab === tab.value"
          @click="setActiveTab(tab.value)"
        >
          {{ tab.label }}<template v-if="tab.count > 0"> ({{ tab.count.toLocaleString() }})</template>
        </button>
      </div>
      <SortMenuButton v-model="config.settings.cameraSort" kind="category" />
    </div>

    <!-- search -->
    <div v-if="activeItems.length > 0" class="mx-1 mb-2 px-1 shrink-0">
      <div
        :class="[
          'h-8 flex items-center rounded-box transition-colors bg-base-100/40',
          isSearchFocused ? 'border-2 border-primary' : 'border border-base-content/10 hover:border-base-content/30',
        ]"
        @click="searchInputRef?.focus()"
      >
        <IconSearch
          class="ml-2 w-4 h-4 shrink-0"
          :class="isSearchFocused ? 'text-primary/70' : 'text-base-content/30'"
        />
        <input
          ref="searchInputRef"
          type="text"
          v-model="search"
          :placeholder="activeTab === 'lens' ? $t('menu.camera_panel.lens_search') : $t('menu.camera_panel.camera_search')"
          class="w-full min-w-0 bg-transparent border-none focus:ring-0 px-2 text-sm placeholder-base-content/30 focus:outline-none"
          maxlength="255"
          @focus="isSearchFocused = true"
          @blur="isSearchFocused = false"
        />
        <button
          v-if="search"
          type="button"
          class="mr-1 p-1 rounded-box text-base-content/30 hover:text-base-content/70"
          @click.stop="search = ''; searchInputRef?.focus()"
        >
          <IconClose class="w-4 h-4" />
        </button>
      </div>
    </div>

    <div v-if="sortedItems.length > 0" class="flex-1 overflow-x-hidden overflow-y-auto">
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
                (item.is_expanded || hasSearch) ? 'rotate-90' : ''
              ]"
              @click.stop="clickExpand(item)"
            />
            <span class="sidebar-item-label">{{ item.make }}</span>
            <span class="sidebar-item-count">{{ (item.parent_count ?? item.counts.reduce((a: number, b: number) => a + b, 0)).toLocaleString() }}</span>
          </div>
          <ul v-if="(item.is_expanded || hasSearch) && item.models.length > 0">
            <li v-for="(model, index) in item.models" :key="`${item.make}-${model}`" class="pl-4">
              <div
                :class="[
                  'sidebar-item sidebar-item-compact ml-2',
                  isModelSelected(model) ? 'sidebar-item-selected' : 'sidebar-item-hover',
                ]"
                @click="clickModel(item.make, model)"
              >
                <component :is="activeTab === 'lens' ? IconCameraAperture : IconCamera" class="mx-1 w-4 h-4 shrink-0" />
                <span class="sidebar-item-label">{{ model }}</span>
                <span class="sidebar-item-count">{{ item.counts[index].toLocaleString() }}</span>
              </div>
            </li>
          </ul>
        </li>
      </ul>
    </div>

    <!-- Display message if no data are found -->
    <div v-else-if="!isLoadingCameraInfo" class="mt-2 px-2 flex flex-col items-center justify-center text-base-content/30">
      <span class="text-sm text-center">{{ search ? (activeTab === 'lens' ? $t('tooltip.not_found.lens') : $t('tooltip.not_found.camera')) : $t('tooltip.not_found.camera_hint') }}</span>
    </div>
  </div>

</template>


<script setup lang="ts">

import { ref, onMounted, onUnmounted, computed, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import { fileInfoRevision } from '@/common/fileInfoRefresh';
import { config, libConfig } from '@/common/config';
import { getCameraInfo, getLensInfo } from '@/common/api';
import { SIDEBAR } from '@/common/constants';
import { IconCamera, IconCameraAperture, IconRight, IconSearch, IconClose } from '@/common/icons';
import SortMenuButton from './SortMenuButton.vue';

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
const search = ref('');
const searchInputRef = ref<HTMLInputElement | null>(null);
const isSearchFocused = ref(false);
const isLoadingCameraInfo = ref(true);
let isCameraMounted = true;
let cameraRequestVersion = 0;

onUnmounted(() => {
  isCameraMounted = false;
  cameraRequestVersion++;
});

const activeTab = computed(() => {
  return config.camera.isCamera ? 'camera' : 'lens';
});

const activeItems = computed(() => {
  return activeTab.value === 'lens' ? lenses.value : cameras.value;
});

// Search mirrors the Tags behavior: a make (first-level) match keeps all its
// models; otherwise keep the item with only the matching models (and parallel
// counts). Shared by the list and the tab counts.
const searchQuery = computed(() => search.value.trim().toLowerCase());
const hasSearch = computed(() => !!search.value.trim());
function filterBySearch(items: any[], q: string) {
  if (!q) return items;
  const out: any[] = [];
  for (const item of items) {
    if (String(item.make || '').toLowerCase().includes(q)) { out.push(item); continue; }
    const models = Array.isArray(item.models) ? item.models : [];
    const counts = Array.isArray(item.counts) ? item.counts : [];
    const keep: number[] = [];
    models.forEach((m: string, i: number) => { if (String(m).toLowerCase().includes(q)) keep.push(i); });
    if (keep.length > 0) {
      out.push({ ...item, models: keep.map((i) => models[i]), counts: keep.map((i) => counts[i]) });
    }
  }
  return out;
}
const filteredCameras = computed(() => filterBySearch(cameras.value, searchQuery.value));
const filteredLenses = computed(() => filterBySearch(lenses.value, searchQuery.value));

const cameraTabs = computed(() => [
  {
    value: 'camera' as const,
    label: localeMsg.value.menu.camera_panel?.camera_title || 'Cameras',
    count: filteredCameras.value.reduce((s: number, c: any) => s + (Array.isArray(c.models) ? c.models.length : 0), 0),
  },
  {
    value: 'lens' as const,
    label: localeMsg.value.menu.camera_panel?.lens_title || 'Lenses',
    count: filteredLenses.value.reduce((s: number, l: any) => s + (Array.isArray(l.models) ? l.models.length : 0), 0),
  },
]);

// Backend returns items already ordered by cameraSort (shared by the camera
// and lens tabs); the search box filters client-side, and the tab counts follow.
const sortedItems = computed(() => (activeTab.value === 'lens' ? filteredLenses.value : filteredCameras.value));

onMounted(async () => {
  await loadCameraInfo();
  expandSelectedItem(cameras.value, (libConfig.camera as any).make, (libConfig.camera as any).model);
  expandSelectedItem(lenses.value, (libConfig.camera as any).lensMake, (libConfig.camera as any).lensModel);
});

// Keep the current filter even if its last file moves out of this category.
watch(fileInfoRevision, async () => {
  if (libConfig.activePane === 'main' && config.main.sidebarIndex === SIDEBAR.CAMERA) await loadCameraInfo(true);
});

watch(() => config.settings.groupRawJpegPairs, async () => {
  if (libConfig.activePane === 'main' && config.main.sidebarIndex === SIDEBAR.CAMERA) await loadCameraInfo(true);
});

// Only refresh the active view. Inactive panel data is refreshed on re-entry.
watch(() => [config.settings.cameraSort], async () => {
  if (libConfig.activePane === 'main' && config.main.sidebarIndex === SIDEBAR.CAMERA) await loadCameraInfo();
});

watch(() => [config.main.sidebarIndex, libConfig.activePane], async () => {
  if (libConfig.activePane === 'main' && config.main.sidebarIndex === SIDEBAR.CAMERA) await loadCameraInfo();
});

async function loadCameraInfo(preserveFilter = false) {
  const requestVersion = ++cameraRequestVersion;
  const libraryId = libConfig._libraryId;
  isLoadingCameraInfo.value = true;
  try {
    const [fetchedCameras, fetchedLenses] = await Promise.all([
      getCameraInfo(config.settings.cameraSort),
      getLensInfo(config.settings.cameraSort),
    ]);
    if (!isCameraMounted || requestVersion !== cameraRequestVersion || libraryId !== libConfig._libraryId) return;

    if (fetchedCameras) {
      cameras.value = fetchedCameras.map((camera: any) => ({ ...camera, is_expanded: false }));
      restoreExpandedItem(cameras.value, (libConfig.camera as any).make, (libConfig.camera as any).model);
    }
    if (fetchedLenses) {
      lenses.value = fetchedLenses.map((lens: any) => ({ ...lens, is_expanded: false }));
      restoreExpandedItem(lenses.value, (libConfig.camera as any).lensMake, (libConfig.camera as any).lensModel);
    }
    if (!preserveFilter) validateSelections();
  } finally {
    if (isCameraMounted && requestVersion === cameraRequestVersion) {
      isLoadingCameraInfo.value = false;
    }
  }
}

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
  config.camera.isCamera = tab === 'camera';
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
      config.camera.isCamera = true;
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

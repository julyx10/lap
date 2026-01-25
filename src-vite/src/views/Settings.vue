<template>
  <div class="w-screen h-screen flex flex-col bg-base-300 text-base-content/70 overflow-hidden">
    <!-- Title Bar -->
    <TitleBar :titlebar="$t('sidebar.settings')" :resizable="false" viewName="Settings" class="shrink-0 z-50" />

    <div class="flex flex-1 overflow-hidden relative">
      <!-- Sidebar -->
      <div class="w-40 m-1 p-2 bg-base-200/30 flex flex-col rounded-box overflow-y-auto shrink-0 select-none">
        <div
          v-for="(tab, index) in ['settings.general.title', 'settings.grid_view.title', 'settings.image_view.title', 'settings.image_search.title', 'settings.about.title']"
          :key="index"
          :class="[
            'p-3 rounded-box cursor-pointer transition-all duration-200 font-medium flex items-center',
            config.settings.tabIndex === index 
              ? 'bg-base-100 text-primary' 
              : 'hover:text-base-content hover:bg-base-100/30'
          ]"
          @click="config.settings.tabIndex = index"
        >
          {{ $t(tab) }}
        </div>
      </div>

      <!-- Main Content -->
      <div class="p-2 mr-1 flex-1 overflow-y-auto scrollbar-hide bg-base-300 cursor-default">
        <div class="">
          
          <!-- General Tab -->
          <div v-if="config.settings.tabIndex === 0" class="flex flex-col">
            <!-- Language -->
            <div class="flex items-center justify-between p-2 rounded-box hover:bg-base-100/10 transition-colors duration-200">
              <div class="flex flex-col gap-0.5">
                <div>{{ $t('settings.general.select_language') }}</div>
              </div>
              <select class="select select-bordered select-sm min-w-32" v-model="config.settings.language">
                <option v-for="(lang, index) in languages" :key="index" :value="lang.value">{{ lang.label }}</option>
              </select>
            </div>
            <!-- Appearance -->
            <div class="flex items-center justify-between p-2 rounded-box hover:bg-base-100/10 transition-colors duration-200">
              <div class="flex flex-col gap-0.5">
                <div>{{ $t('settings.general.appearance') }}</div>
              </div>
              <select class="select select-bordered select-sm min-w-32" v-model="config.settings.appearance">
                <option v-for="(item, index) in appearanceOptions" :key="index" :value="item.value">{{ item.label }}</option>
              </select>
            </div>
            <!-- Theme -->
            <div class="flex items-center justify-between p-2 rounded-box hover:bg-base-100/10 transition-colors duration-200">
              <div class="flex flex-col gap-0.5">
                <div>{{ $t('settings.general.theme') }}</div>
              </div>
              <select class="select select-bordered select-sm min-w-32" v-model="currentTheme">
                <option v-for="(option, index) in themeOptions" :key="index" :value="option.value">{{ option.label }}</option>
              </select>
            </div>
            <!-- Show Button Text -->
            <div class="flex items-center justify-between p-2 rounded-box hover:bg-base-100/10 transition-colors duration-200">
              <div class="flex flex-col gap-0.5">
                <div>{{ $t('settings.general.show_button_text') }}</div>
              </div>
              <input type="checkbox" class="toggle toggle-primary toggle-sm" v-model="config.settings.showButtonText" />
            </div>
            <!-- Show Tooltips -->
            <div class="flex items-center justify-between p-2 rounded-box hover:bg-base-100/10 transition-colors duration-200">
              <div class="flex flex-col gap-0.5">
                <div>{{ $t('settings.general.show_tool_tip') }}</div>
              </div>
              <input type="checkbox" class="toggle toggle-primary toggle-sm" v-model="config.settings.showToolTip" />
            </div>
            <!-- Show Status Bar -->
            <div class="flex items-center justify-between p-2 rounded-box hover:bg-base-100/10 transition-colors duration-200">
              <div class="flex flex-col gap-0.5">
                <div>{{ $t('settings.general.show_status_bar') }}</div>
              </div>
              <input type="checkbox" class="toggle toggle-primary toggle-sm" v-model="config.settings.showStatusBar" />
            </div>
             <!-- Show Map -->
            <div class="flex items-center justify-between p-2 rounded-box hover:bg-base-100/10 transition-colors duration-200">
              <div class="flex flex-col gap-0.5">
                <div>{{ $t('settings.general.show_map') }}</div>
                <div class="text-xs text-base-content/30">{{ $t('settings.general.show_map_hint') }}</div>
              </div>
              <input type="checkbox" class="toggle toggle-primary toggle-sm" v-model="config.settings.showMap" />
            </div>
          </div>

          <!-- Grid View Tab -->
          <div v-else-if="config.settings.tabIndex === 1" class="flex flex-col">
             <!-- Style -->
            <div class="flex items-center justify-between p-2 rounded-box hover:bg-base-100/10 transition-colors duration-200">
              <div class="flex flex-col gap-0.5">
                <div>{{ $t('settings.grid_view.style') }}</div>
              </div>
              <select class="select select-bordered select-sm min-w-32" v-model="config.settings.grid.style">
                <option v-for="(option, index) in gridStyleOptions" :key="index" :value="option.value">{{ option.label }}</option>
              </select>
            </div>
            <!-- Size -->
            <div class="flex items-center justify-between p-2 min-h-12 rounded-box hover:bg-base-100/10 transition-colors duration-200">
              <div class="flex flex-col gap-0.5">
                <div>{{ $t('settings.grid_view.size') }}</div>
              </div>
              <div class="w-48 flex justify-end">
                <SliderInput v-model="config.settings.grid.size" :min="120" :max="320" :step="10" label="" />
              </div>
            </div>
             <!-- Scaling -->
            <div class="flex items-center justify-between p-2 rounded-box hover:bg-base-100/10 transition-colors duration-200">
              <div class="flex flex-col gap-0.5">
                <div>{{ $t('settings.grid_view.scaling') }}</div>
              </div>
              <select class="select select-bordered select-sm min-w-32" v-model="config.settings.grid.scaling">
                <option v-for="(option, index) in gridScalingOptions" :key="index" :value="option.value">{{ option.label }}</option>
              </select>
            </div>
            <!-- Primary Label -->
            <div class="flex items-center justify-between p-2 rounded-box hover:bg-base-100/10 transition-colors duration-200">
              <div class="flex flex-col gap-0.5">
                <div>{{ $t('settings.grid_view.label_primary') }}</div>
                <div class="text-xs text-base-content/30">{{ $t('settings.grid_view.label_primary_hint') }}</div>
              </div>
              <select class="select select-bordered select-sm min-w-32" v-model="config.settings.grid.labelPrimary">
                 <option v-for="(option, index) in gridLabelOptions" :key="index" :value="option.value">{{ option.label }}</option>
              </select>
            </div>
            <!-- Secondary Label -->
            <div class="flex items-center justify-between p-2 rounded-box hover:bg-base-100/10 transition-colors duration-200">
              <div class="flex flex-col gap-0.5">
                <div>{{ $t('settings.grid_view.label_secondary') }}</div>
                <div class="text-xs text-base-content/30">{{ $t('settings.grid_view.label_secondary_hint') }}</div>
              </div>
              <select class="select select-bordered select-sm min-w-32" v-model="config.settings.grid.labelSecondary">
                 <option v-for="(option, index) in gridLabelOptions" :key="index" :value="option.value">{{ option.label }}</option>
              </select>
            </div>
          </div>

          <!-- Image View Tab -->
          <div v-else-if="config.settings.tabIndex === 2" class="flex flex-col">
            <!-- Preview Position -->
             <div class="flex items-center justify-between p-2 rounded-box hover:bg-base-100/10 transition-colors duration-200">
              <div class="flex flex-col gap-0.5">
                <div>{{ $t('settings.filmstrip_view.preview_position') }}</div>
              </div>
              <select class="select select-bordered select-sm min-w-32" v-model="config.settings.previewPosition">
                 <option v-for="(option, index) in filmStripViewPreviewPositionOptions" :key="index" :value="option.value">{{ option.label }}</option>
              </select>
            </div>
            <!-- Navigator View -->
            <div class="flex items-center justify-between p-2 rounded-box hover:bg-base-100/10 transition-colors duration-200">
              <div class="flex flex-col gap-0.5">
                <div>{{ $t('settings.image_view.navigator_view') }}</div>
              </div>
              <select class="select select-bordered select-sm min-w-32" v-model="config.settings.navigatorViewMode">
                 <option v-for="(option, index) in navigatorViewModeOptions" :key="index" :value="option.value">{{ option.label }}</option>
              </select>
            </div>
            <!-- Navigator Size -->
            <div class="flex items-center justify-between p-2 rounded-box hover:bg-base-100/10 transition-colors duration-200">
              <div class="flex flex-col gap-0.5">
                <div>{{ $t('settings.image_view.navigator_view__size') }}</div>
              </div>
               <select class="select select-bordered select-sm min-w-32" v-model="config.settings.navigatorViewSize">
                 <option v-for="(option, index) in navigatorViewSizeOptions" :key="index" :value="option.value">{{ option.label }}</option>
              </select>
            </div>
            <!-- Mouse Wheel -->
            <div class="flex items-center justify-between p-2 rounded-box hover:bg-base-100/10 transition-colors duration-200">
              <div class="flex flex-col gap-0.5">
                <div>{{ $t('settings.image_view.mouse_wheel') }}</div>
              </div>
               <select class="select select-bordered select-sm min-w-32" v-model="config.settings.mouseWheelMode">
                 <option v-for="(item, index) in wheelOptions" :key="index" :value="item.value">{{ item.label }}</option>
              </select>
            </div>
            <!-- Slide Show -->
            <div class="flex items-center justify-between p-2 min-h-12 rounded-box hover:bg-base-100/10 transition-colors duration-200">
              <div class="flex flex-col gap-0.5">
                <div class="text-nowrap">{{ $t('settings.image_view.slide_show_interval', { second: getSlideShowInterval(config.settings.slideShowInterval) }) }}</div>
              </div>
              <div class="w-48 flex justify-end">
                <SliderInput v-model="config.settings.slideShowInterval" :min="0" :max="5" :step="1" label="" />
              </div>
            </div>
            <!-- Auto Play Video -->
             <div class="flex items-center justify-between p-2 rounded-box hover:bg-base-100/10 transition-colors duration-200">
              <div class="flex flex-col gap-0.5">
                <div>{{ $t('settings.image_view.auto_play_video') }}</div>
              </div>
              <input type="checkbox" class="toggle toggle-primary toggle-sm" v-model="config.settings.autoPlayVideo" />
            </div>
            <!-- Show Comment -->
            <div class="flex items-center justify-between p-2 rounded-box hover:bg-base-100/10 transition-colors duration-200">
              <div class="flex flex-col gap-0.5">
                <div>{{ $t('settings.image_view.show_comment') }}</div>
              </div>
              <input type="checkbox" class="toggle toggle-primary toggle-sm" v-model="config.settings.showComment" />
            </div>
          </div>

          <!-- Image Search Tab -->
          <div v-else-if="config.settings.tabIndex === 3" class="flex flex-col overflow-hidden">
             <!-- Threshold -->
             <div class="flex items-center justify-between p-2 rounded-box hover:bg-base-100/10 transition-colors duration-200">
              <div class="flex flex-col gap-0.5">
                <div>{{ $t('settings.image_search.similarity') }}</div>
                <div class="text-xs text-base-content/30">{{ $t('settings.image_search.similarity_hint') }}</div>
              </div>
               <select class="select select-bordered select-sm min-w-32" v-model="config.settings.imageSearch.thresholdIndex">
                 <option v-for="(option, index) in similarityOptions" :key="index" :value="option.value">{{ option.label }}</option>
              </select>
            </div>
            <!-- Face Cluster -->
            <div class="flex items-center justify-between p-2 rounded-box hover:bg-base-100/10 transition-colors duration-200">
              <div class="flex flex-col gap-0.5">
                <div>{{ $t('settings.face_recognition.similarity') }}</div>
                <div class="text-xs text-base-content/30">{{ $t('settings.face_recognition.cluster_threshold_hint') }}</div>
              </div>
               <select class="select select-bordered select-sm min-w-32" v-model="config.settings.face.clusterThresholdIndex">
                 <option v-for="(option, index) in faceClusterOptions" :key="index" :value="option.value">{{ option.label }}</option>
              </select>
            </div>
          </div>

          <!-- About Tab -->
          <div v-else-if="config.settings.tabIndex === 4" class="py-2">
             <SettingsAbout />
          </div>

        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">

import { ref, watch, computed, onMounted, onUnmounted } from 'vue';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { emit } from '@tauri-apps/api/event';
import { debounce } from 'lodash';
import { useI18n } from 'vue-i18n';
import { config } from '@/common/config';
import { setTheme, getSlideShowInterval } from '@/common/utils';

import TitleBar from '@/components/TitleBar.vue';
import SliderInput from '@/components/SliderInput.vue';
import SettingsAbout from '@/components/SettingsAbout.vue';

/// i18n
const { locale, messages } = useI18n();
const localeMsg = computed(() => messages.value[config.settings.language] as any);

const appWindow = getCurrentWebviewWindow()

const languages = [
  { label: 'English', value: 'en' },
  { label: '中文',    value: 'zh' },
];

const appearanceOptions = computed(() => {
  const options = localeMsg.value.settings.general.appearance_options;
  return Array.from({ length: options.length }, (_, i) => ({
    label: options[i],
    value: i,
  }));
});

// Define the theme options
const themeOptions = computed(() => {
  const options = config.settings.appearance === 0 
    ? localeMsg.value.settings.general.theme_options_light 
    : localeMsg.value.settings.general.theme_options_dark;

  const result = [];
  for (let i = 0; i < options.length; i++) {
    result.push({ label: options[i], value: i });
  }
  return result;
});

const currentTheme = computed({
  get() {
    return config.settings.appearance === 0 ? config.settings.lightTheme : config.settings.darkTheme;
  },
  set(value) {
    config.settings.appearance === 0 ? config.settings.lightTheme = value : config.settings.darkTheme = value;
  }
});

// Define the wheel options using computed to react to language changes
const wheelOptions = computed(() => {
  const options = localeMsg.value.settings.image_view.mouse_wheel_options; // returns an array
  return [
    { label: options[0], value: 0 },  // 0: previous / next
    { label: options[1], value: 1 },  // 1: zoom in / out
  ];
});

// Define the grid scaling options
const gridScalingOptions = computed(() => {
  const options = localeMsg.value.settings.grid_view.scaling_options;
  const result = [];

  for (let i = 0; i < options.length; i++) {
    result.push({ label: options[i], value: i });
  }

  return result;
});

// Define the grid style options
const gridStyleOptions = computed(() => {
  const options = localeMsg.value.settings.grid_view.style_options;
  const result = [];

  for (let i = 0; i < options.length; i++) {
    result.push({ label: options[i], value: i });
  }

  return result;
});

// Define the grid label options
const gridLabelOptions = computed(() => {
  const options = localeMsg.value.settings.grid_view.label_options;
  const result = [];

  for (let i = 0; i < options.length; i++) {
    result.push({ label: options[i], value: i });
  }

  return result;
});

// Define the navigator view mode options
const navigatorViewModeOptions = computed(() => {
  const options = localeMsg.value.settings.image_view.navigator_view_options;
  const result = [];

  for (let i = 0; i < options.length; i++) {
    result.push({ label: options[i], value: i });
  }

  return result;
});

// Define the navigator view size options
const navigatorViewSizeOptions = computed(() => {
  const options = localeMsg.value.settings.image_view.navigator_view_size_options;
  const result = [];

  for (let i = 0; i < options.length; i++) {
    result.push({ label: options[i], value: parseInt(options[i].split('(')[1].split('px')[0]) });
  }

  return result;
});

const filmStripViewPreviewPositionOptions = computed(() => {
  const options = localeMsg.value.settings.filmstrip_view.preview_position_options;
  const result = [];

  for (let i = 0; i < options.length; i++) {
    result.push({ label: options[i], value: i });
  }

  return result;
});

// Define the similarity options
const similarityOptions = computed(() => {
  const options = localeMsg.value.settings.image_search.similarity_options;
  // Use getter to retrieve thresholds
  const values = config.imageSearchThresholds ?? [0.8, 0.6, 0.4, 0.25]; 
  // Map index dummy as the value since v-model is thresholdIndex
  return values.map((val, i) => ({ label: options[i], value: i }));
});

// Define the face cluster threshold options
const faceClusterOptions = computed(() => {
  const options = localeMsg.value.settings.face_recognition?.cluster_threshold_options || 
    ['Very High', 'High', 'Medium', 'Low'];
  // Map index as value since v-model is clusterThresholdIndex
  return options.map((label: string, i: number) => ({ label, value: i }));
});

onMounted(async () => {
  window.addEventListener('keydown', handleKeyDown);
  
  // Show window after mount
  await appWindow.show();
});

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeyDown);
});

// general settings
watch(() => config.settings.tabIndex, (newValue) => {
  emit('settings-settingsTabIndex-changed', newValue);
});
watch(() => config.settings.appearance, (newValue) => {
  setTheme(newValue, newValue === 0 ? config.settings.lightTheme : config.settings.darkTheme);
  emit('settings-appearance-changed', newValue);
});
watch(() => config.settings.lightTheme, (newValue) => {
  setTheme(config.settings.appearance, newValue);
  emit('settings-lightTheme-changed', newValue);
});
watch(() => config.settings.darkTheme, (newValue) => {
  setTheme(config.settings.appearance, newValue);
  emit('settings-darkTheme-changed', newValue);
});
watch(() => config.settings.language, (newValue) => {
  locale.value = newValue;
  emit('settings-language-changed', newValue);
});
watch(() => config.settings.showButtonText, (newValue) => {
  emit('settings-showButtonText-changed', newValue);
});
watch(() => config.settings.showToolTip, (newValue) => {
  emit('settings-showToolTip-changed', newValue);
});
watch(() => config.settings.showStatusBar, (newValue) => {
  emit('settings-showStatusBar-changed', newValue);
});
watch(() => config.settings.showMap, (newValue) => {
  emit('settings-showMap-changed', newValue);
});
watch(() => config.settings.showComment, (newValue) => {
  emit('settings-showComment-changed', newValue);
});
watch(() => config.settings.debugMode, (newValue) => {
  emit('settings-debugMode-changed', newValue);
});

// grid view settings
watch(() => config.settings.grid.size, debounce((newValue: number) => {
  emit('settings-gridSize-changed', newValue);
}, 100));
watch(() => config.settings.grid.style, (newValue) => {
  emit('settings-gridStyle-changed', newValue);
});
watch(() => config.settings.grid.scaling, (newValue) => {
  emit('settings-gridScaling-changed', newValue);
});
watch(() => config.settings.grid.labelPrimary, (newValue) => {
  emit('settings-gridLabelPrimary-changed', newValue);
});
watch(() => config.settings.grid.labelSecondary, (newValue) => {
  emit('settings-gridLabelSecondary-changed', newValue);
});
watch(() => config.settings.previewPosition, (newValue) => {
  emit('settings-filmStripViewPreviewPosition-changed', newValue);
});

// image viewer settings
watch(() => config.settings.mouseWheelMode, (newValue) => {
  emit('settings-mouseWheelMode-changed', newValue);
});
watch(() => config.settings.slideShowInterval, (newValue) => {
  emit('settings-slideShowInterval-changed', newValue);
});
watch(() => config.settings.navigatorViewMode, (newValue) => {
  emit('settings-navigatorViewMode-changed', newValue);
});
watch(() => config.settings.navigatorViewSize, (newValue) => {
  emit('settings-navigatorViewSize-changed', newValue);
});
watch(() => config.settings.autoPlayVideo, (newValue) => {
  emit('settings-autoPlayVideo-changed', newValue);
});

// image search settings
watch(() => config.settings.imageSearch.thresholdIndex, (newValue) => {
  emit('settings-imageSearchThresholdIndex-changed', newValue);
});
watch(() => config.settings.imageSearch.limit, (newValue) => {
  emit('settings-imageSearchLimit-changed', newValue);
});

// face settings
watch(() => config.settings.face.clusterThresholdIndex, (newValue) => {
  emit('settings-faceClusterThresholdIndex-changed', newValue);
});

// Handle keyboard shortcuts
function handleKeyDown(event: KeyboardEvent) {
  const navigationKeys = ['Tab', 'Escape'];
  
  // Disable default behavior for certain keys
  if (navigationKeys.includes(event.key)) {
    event.preventDefault();
  }

  switch (event.key) {
    case 'Tab':
      config.settings.tabIndex += 1;
      config.settings.tabIndex = config.settings.tabIndex % 5; // 5 tabs
      break;
    case 'Escape':
      appWindow.close(); // Close the window
      break;
  }
}
</script>
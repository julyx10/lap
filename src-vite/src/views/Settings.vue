<template>

  <div class="w-screen h-screen flex flex-col overflow-hidden select-none bg-base-200 text-base-content/70">
    <!-- Title Bar -->
    <TitleBar :titlebar="$t('sidebar.settings')" :resizable="false" viewName="Settings"/>

    <!-- Main Content -->
    <div 
      class="flex-1 flex p-4 overflow-auto" 
    >
      <!-- Tabs -->
      <div class="w-32 font-bold">
        <div
          v-for="(tab, index) in ['settings.general.title', 'settings.grid_view.title', 'settings.image_viewer.title', 'settings.about.title']"
          :key="index"
          :class="[
            'mb-4 px-1 border-l-2 cursor-pointer transition-colors duration-300', 
            config.settings.tabIndex === index ? 'text-base-content border-primary' : 'border-transparent hover:text-base-content',
          ]"
          @click="config.settings.tabIndex = index"
        >
          {{ $t(tab) }}
        </div>
      </div>
    
      <div class="grow px-2">

        <!-- General tab -->
        <section v-if="config.settings.tabIndex === 0">
          <!-- select language -->
          <div class="flex items-center justify-between mb-4">
            <label>{{ $t('settings.general.select_language') }}</label>
            <select class="select" v-model="config.settings.language">
              <option v-for="(lang, index) in languages" :key="index" :value="lang.value">
                {{ lang.label }}
              </option>
            </select>
          </div>
          <!-- Appearance -->
          <div class="flex items-center justify-between mb-4">
            <label>{{ $t('settings.general.appearance') }}</label>
            <select class="select" v-model="config.settings.appearance">
              <option v-for="(item, index) in appearanceOptions" :key="index" :value="item.value">
                {{ item.label }}
              </option>
            </select>
          </div>
          <!-- Color theme -->
          <div class="flex items-center justify-between mb-4">
            <label>{{ $t('settings.general.theme') }}</label>
            <select class="select" v-model="currentTheme">
              <option v-for="(option, index) in themeOptions" :key="index" :value="option.value">
                {{ option.label }}
              </option>
            </select>
          </div>
          <!-- Show button text -->
          <div class="flex items-center justify-between mb-4">
            <label>{{ $t('settings.general.show_button_text') }}</label>
            <input type="checkbox" class="toggle toggle-primary" v-model="config.settings.showButtonText" />
          </div>
          <!-- Show button tooltip -->
          <div class="flex items-center justify-between mb-4">
            <label>{{ $t('settings.general.show_tool_tip') }}</label>
            <input type="checkbox" class="toggle toggle-primary" v-model="config.settings.showToolTip" />
          </div>
          <!-- Show status bar -->
          <div class="flex items-center justify-between mb-4">
            <label>{{ $t('settings.general.show_status_bar') }}</label>
            <input type="checkbox" class="toggle toggle-primary" v-model="config.settings.showStatusBar" />
          </div>
          <!-- Debug Mode -->
          <!-- <div class="flex items-center justify-between mb-4">
            <label>{{ $t('settings.general.debug_mode') }}</label>
            <input type="checkbox" class="toggle toggle-primary" v-model="config.settings.debugMode" />
          </div> -->

        </section>

        <!-- Grid view tab -->
        <section v-if="config.settings.tabIndex === 1">

          <!-- Grid Size -->
          <div class="flex items-center justify-between mb-4">
            <label for="grid_size" >{{ $t('settings.grid_view.size') }}</label>
            <SliderInput 
              v-model="config.settings.grid.size" 
              :min="120" 
              :max="320" 
              :step="10" 
              label=""
            />
          </div>

          <!-- Grid Scaling -->
          <div class="flex items-center justify-between mb-4">
            <label>{{ $t('settings.grid_view.scaling') }}</label>
            <select class="select" v-model="config.settings.grid.scaling">
              <option v-for="(option, index) in gridScalingOptions" :key="index" :value="option.value">
                {{ option.label }}
              </option>
            </select>
          </div>

          <!-- Primary Label -->
          <div class="flex items-center justify-between mb-4">
            <label>{{ $t('settings.grid_view.label_primary') }}</label>
            <select class="select" v-model="config.settings.grid.labelPrimary">
              <option v-for="(option, index) in gridLabelOptions" :key="index" :value="option.value">
                {{ option.label }}
              </option>
            </select>
          </div>

            <!-- Secondary Label -->
            <div class="flex items-center justify-between mb-4">
              <label>{{ $t('settings.grid_view.label_secondary') }}</label>
              <select class="select" v-model="config.settings.grid.labelSecondary">
                <option v-for="(option, index) in gridLabelOptions" :key="index" :value="option.value">
                  {{ option.label }}
                </option>
              </select>
            </div>

          <!-- Preview Position -->
          <div class="flex items-center justify-between mb-4">
            <label>{{ $t('settings.filmstrip_view.preview_position') }}</label>
            <select class="select" v-model="config.settings.filmStripView.previewPosition">
              <option v-for="(option, index) in filmStripViewPreviewPositionOptions" :key="index" :value="option.value">
                {{ option.label }}
              </option>
            </select>
          </div>
        </section>

        <!-- Image Viewer tab -->
        <section v-else-if="config.settings.tabIndex === 2">

          <!-- Show navigator view -->
          <div class="flex items-center justify-between mb-4">
            <label>{{ $t('settings.image_viewer.navigator_view') }}</label>
            <select class="select" v-model="config.settings.navigatorViewMode">
              <option v-for="(option, index) in navigatorViewModeOptions" :key="index" :value="option.value">
                {{ option.label }}
              </option>
            </select>
          </div>

          <!-- Navigator view size -->
          <div class="flex items-center justify-between mb-4">
            <label>{{ $t('settings.image_viewer.navigator_view__size') }}</label>
            <select class="select" v-model="config.settings.navigatorViewSize">
              <option v-for="(option, index) in navigatorViewSizeOptions" :key="index" :value="option.value">
                {{ option.label }}
              </option>
            </select>
          </div>

          <!-- mouse wheel mode -->
          <div class="flex items-center justify-between mb-4">
            <label>{{ $t('settings.image_viewer.mouse_wheel') }}</label>
            <select class="select" v-model="config.settings.mouseWheelMode">
              <option v-for="(item, index) in wheelOptions" :key="index" :value="item.value">
                {{ item.label }}
              </option>
            </select>
          </div>

          <!-- slide show interval -->
          <div class="flex items-center justify-between mb-4">
            <label for="autoplay-interval" >{{ $t('settings.image_viewer.slide_show_interval', { second: getSlideShowInterval(config.settings.slideShowInterval) }) }}</label>
            <SliderInput 
              v-model="config.settings.slideShowInterval" 
              :min="0" 
              :max="5" 
              :step="1" 
              label=""
            />
          </div>

          <!-- Auto play video -->
          <div class="flex items-center justify-between mb-4">
            <label>{{ $t('settings.image_viewer.auto_play_video') }}</label>
            <input type="checkbox" class="toggle toggle-primary" v-model="config.settings.autoPlayVideo" />
          </div>

          <!-- Show comment -->
          <div class="flex items-center justify-between mb-4">
            <label>{{ $t('settings.image_viewer.show_comment') }}</label>
            <input type="checkbox" class="toggle toggle-primary" v-model="config.settings.showComment" />
          </div>
        </section>

        <!-- About Section -->
        <section v-else-if="config.settings.tabIndex === 3">

          <div class="flex flex-col items-center justify-between mb-4">
            <!-- <label class="font-bold mb-4">jc-photo</label> -->

            <table class="w-full">
              <tbody>
                <tr>
                  <td>{{ $t('settings.about.package.name') }}</td>
                  <td>{{ packageInfo.name }}</td>
                </tr>
                <tr>
                  <td>{{ $t('settings.about.package.version') }}</td>
                  <td>{{ packageInfo.version }}</td>
                </tr>
                <tr>
                  <td>{{ $t('settings.about.package.build_time') }}</td>
                  <td>{{ buildTime }}</td>
                </tr>
                <!-- <tr>
                  <td>{{ $t('settings.about.package_website') }}</td>
                  <td>{{ packageInfo.homepage }}</td>
                </tr>
                <tr>
                  <td>{{ $t('settings.about.package_license') }}</td>
                  <td>{{ packageInfo.license}}</td>
                </tr> -->
                <tr>
                  <td>{{ $t('settings.about.package.author') }}</td>
                  <td>{{ packageInfo.authors[0]}}</td>
                </tr>
                <tr>
                  <td colspan="2" class="h-4"> </td>
                </tr>
                <tr>
                  <td>{{ $t('settings.about.storage.total_file_count') }}</td>
                  <td>{{ totalFileCount.toLocaleString() }}</td>
                </tr>
                <tr>
                  <td>{{ $t('settings.about.storage.total_file_size') }}</td>
                  <td>{{ formatFileSize(totalFileSize) }}</td>
                </tr>
                <tr>
                  <td>{{ $t('settings.about.storage.file_path') }}</td>
                   <td>
                    <input
                      type="text"
                      :value="storageFileInfo.file_path"
                      readonly
                      class="py-1 w-full border-none focus:border-none focus:ring-0 focus:outline-none"
                    />
                   </td>
                </tr>
                <tr>
                  <td>{{ $t('settings.about.storage.file_size') }}</td>
                  <td>{{ formatFileSize(storageFileInfo.file_size) }}</td>
                </tr>
              </tbody>
            </table>
          </div>
        </section>
      </div>
    </div>  
  </div>
</template>

<script setup>

import { ref, watch, computed, onMounted, onUnmounted } from 'vue';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { emit } from '@tauri-apps/api/event';
import { debounce } from 'lodash';
import { useI18n } from 'vue-i18n';
import { config } from '@/common/config';
import { setTheme, getSlideShowInterval, formatFileSize } from '@/common/utils';
import { getPackageInfo, getBuildTime, getStorageFileInfo, getDbCountAndSum } from '@/common/api';

import TitleBar from '@/components/TitleBar.vue';
import SliderInput from '@/components/SliderInput.vue';

/// i18n
const { locale, messages } = useI18n();
const localeMsg = computed(() => messages.value[config.settings.language]);

const appWindow = getCurrentWebviewWindow()

// get all db files' count and sum(without pagination)
const totalFileCount = ref(0);
const totalFileSize = ref(0);

// storage file info
const packageInfo = ref(null);
const buildTime = ref('');
const storageFileInfo = ref('');

const languages = [
  { label: 'English', value: 'en' },
  { label: '日本語',  value: 'ja' },
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
  const options = localeMsg.value.settings.image_viewer.mouse_wheel_options; // returns an array
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
  const options = localeMsg.value.settings.image_viewer.navigator_view_options;
  const result = [];

  for (let i = 0; i < options.length; i++) {
    result.push({ label: options[i], value: i });
  }

  return result;
});

// Define the navigator view size options
const navigatorViewSizeOptions = computed(() => {
  const options = localeMsg.value.settings.image_viewer.navigator_view_size_options;
  const result = [];

  for (let i = 0; i < options.length; i++) {
    result.push({ label: options[i], value: parseInt(options[i].split('(')[1].split('px')[0]) });
  }

  return result;
});

// Define the preview position options
const filmStripViewPreviewPositionOptions = computed(() => {
  const options = localeMsg.value.settings.filmstrip_view.preview_position_options;
  const result = [];

  for (let i = 0; i < options.length; i++) {
    result.push({ label: options[i], value: i });
  }

  return result;
});

onMounted(() => {
  window.addEventListener('keydown', handleKeyDown);
});

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeyDown);
});

// Handle the settings tab index change
watch(() => config.settings.tabIndex, (newValue) => {
  if (newValue === 3) {   // about tab
    // Get package info
    getPackageInfo().then((info) => {
      packageInfo.value = info;
    });
    // Get build time
    getBuildTime().then((time) => {
      buildTime.value = time;
    });
    // Get storage file info
    getStorageFileInfo().then((file) => {
      storageFileInfo.value = file;
    });
    // Get db file info
    getDbCountAndSum().then((info) => {
      if(info) {
        [totalFileCount.value, totalFileSize.value] = info;
      }
    });
  }
}, { immediate: true });

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
watch(() => config.settings.showComment, (newValue) => {
  emit('settings-showComment-changed', newValue);
});
watch(() => config.settings.debugMode, (newValue) => {
  emit('settings-debugMode-changed', newValue);
});

// grid view settings
watch(() => config.settings.grid.size, debounce((newValue) => {
  emit('settings-gridSize-changed', newValue);
}, 100));
watch(() => config.settings.grid.scaling, (newValue) => {
  emit('settings-gridScaling-changed', newValue);
});
watch(() => config.settings.grid.labelPrimary, (newValue) => {
  emit('settings-gridLabelPrimary-changed', newValue);
});
watch(() => config.settings.grid.labelSecondary, (newValue) => {
  emit('settings-gridLabelSecondary-changed', newValue);
});
watch(() => config.settings.filmStripView.previewPosition, (newValue) => {
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
// Handle keyboard shortcuts
function handleKeyDown(event) {
  const navigationKeys = ['Tab', 'Escape'];
  
  // Disable default behavior for certain keys
  if (navigationKeys.includes(event.key)) {
    event.preventDefault();
  }

  switch (event.key) {
    case 'Tab':
      config.settings.tabIndex += 1;
      config.settings.tabIndex = config.settings.tabIndex % 4; // 4 tabs
      break;
    case 'Escape':
      appWindow.close(); // Close the window
      break;
  }
}
</script>
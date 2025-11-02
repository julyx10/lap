<template>

  <div class="w-screen h-screen flex flex-col overflow-hidden select-none bg-base-300 text-base-content/70">
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
            'mb-4 px-1 border-l-2 cursor-pointer', 
            config.settingsTabIndex === index ? 'text-base-content border-primary transition-colors duration-300' : 'border-transparent ',
          ]"
          @click="config.settingsTabIndex = index"
        >
          {{ $t(tab) }}
        </div>
      </div>
    
      <div class="grow px-2">

        <!-- General tab -->
        <section v-if="config.settingsTabIndex === 0">
          <!-- select language -->
          <div class="flex items-center justify-between mb-4">
            <label for="language-select">{{ $t('settings.general.select_language') }}</label>
            <select id="language-select" class="select" v-model="config.language">
              <option v-for="(lang, index) in languages" 
                :key="index" 
                :value="lang.value"
              >
                {{ lang.label }}
              </option>
            </select>
          </div>
          <!-- Appearance -->
          <div class="flex items-center justify-between mb-4">
            <label for="mouse-wheel">{{ $t('settings.general.appearance') }}</label>
            <select id="mouse-wheel" class="select" v-model="config.appearance">
              <option 
                v-for="(item, index) in appearanceOptions" 
                :key="index" 
                :value="item.value" 
              >
                {{ item.label }}
              </option>
            </select>
          </div>
          <!-- Show button text -->
          <div class="flex items-center justify-between mb-4">
            <label for="show-button-text" >{{ $t('settings.general.show_button_text') }}</label>
            <input type="checkbox" class="toggle" v-model="config.showButtonText" />
          </div>
          <!-- Show button tooltip -->
          <div class="flex items-center justify-between mb-4">
            <label for="show-tool-tip" >{{ $t('settings.general.show_tool_tip') }}</label>
            <input type="checkbox" class="toggle" v-model="config.showToolTip" />
          </div>
          <!-- Show status bar -->
          <div class="flex items-center justify-between mb-4">
            <label for="show-status-bar" >{{ $t('settings.general.show_status_bar') }}</label>
            <input type="checkbox" class="toggle" v-model="config.showStatusBar" />
          </div>
          <!-- Show comment -->
          <div class="flex items-center justify-between mb-4">
            <label for="show-comment" >{{ $t('settings.image_viewer.show_comment') }}</label>
            <input type="checkbox" class="toggle" v-model="config.showComment" />
          </div>
          <!-- Debug Mode -->
          <!-- <div class="flex items-center justify-between mb-4">
            <label for="debug-mode" >{{ $t('settings.general.debug_mode') }}</label>
            <input type="checkbox" class="toggle" v-model="config.debugMode" />
          </div> -->

        </section>

        <!-- Grid view tab -->
        <section v-if="config.settingsTabIndex === 1">

          <!-- Grid Size -->
          <div class="flex items-center justify-between mb-4">
            <label for="grid_size" >{{ $t('settings.grid_view.size') }}</label>
            <SliderInput 
              v-model="config.grid.size" 
              :min="120" 
              :max="320" 
              :step="10" 
              label=""
            />
          </div>

          <!-- Grid Scaling -->
          <div class="flex items-center justify-between mb-4">
            <label for="grid_scaling-select">{{ $t('settings.grid_view.scaling') }}</label>
            <select id="grid_scaling-select" class="select" v-model="config.grid.scaling">
              <option v-for="(option, index) in gridScalingOptions" 
                :key="index" 
                :value="option.value"
              >
                {{ option.label }}
              </option>
            </select>
          </div>

          <!-- Primary Label -->
          <div class="flex items-center justify-between mb-4">
            <label for="grid_label_primary-select">{{ $t('settings.grid_view.label_primary') }}</label>
            <select id="grid_label_primary-select" class="select" v-model="config.grid.labelPrimary">
              <option v-for="(option, index) in gridLabelOptions" 
                :key="index"
                :value="option.value"
              >
                {{ option.label }}
              </option>
            </select>
          </div>

            <!-- Secondary Label -->
            <div class="flex items-center justify-between mb-4">
              <label for="grid_label_secondary-select">{{ $t('settings.grid_view.label_secondary') }}</label>
              <select id="grid_label_secondary-select" class="select" v-model="config.grid.labelSecondary">
                <option v-for="(option, index) in gridLabelOptions" 
                  :key="index" 
                  :value="option.value"
                >
                  {{ option.label }}
                </option>
              </select>
            </div>
        </section>

        <!-- Image Viewer tab -->
        <section v-else-if="config.settingsTabIndex === 2">

          <!-- mouse wheel mode -->
          <div class="flex items-center justify-between mb-4">
            <label for="mouse-wheel">{{ $t('settings.image_viewer.mouse_wheel') }}</label>
            <select id="mouse-wheel" class="select" v-model="config.mouseWheelMode">
              <option 
                v-for="(item, index) in wheelOptions" 
                :key="index" 
                :value="item.value" 
              >
                {{ item.label }}
              </option>
            </select>
          </div>

          <!-- slide show interval -->
          <div class="flex items-center justify-between mb-4">
            <label for="autoplay-interval" >{{ $t('settings.image_viewer.slide_show_interval', { second: getSlideShowInterval(config.slideShowInterval) }) }}</label>
            <SliderInput 
              v-model="config.slideShowInterval" 
              :min="0" 
              :max="5" 
              :step="1" 
              label=""
            />
          </div>

          <!-- Auto play video -->
          <div class="flex items-center justify-between mb-4">
            <label for="auto-play-video" >{{ $t('settings.image_viewer.auto_play_video') }}</label>
            <input type="checkbox" class="toggle" v-model="config.autoPlayVideo" />
          </div>
        </section>

        <!-- About Section -->
        <section v-else-if="config.settingsTabIndex === 3">

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
import { config, setTheme, getSlideShowInterval, formatFileSize } from '@/common/utils';
import { getPackageInfo, getBuildTime, getStorageFileInfo, getDbCountAndSum } from '@/common/api';

import TitleBar from '@/components/TitleBar.vue';
import SliderInput from '@/components/SliderInput.vue';

/// i18n
const { locale, messages } = useI18n();
const localeMsg = computed(() => messages.value[config.language]);

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

onMounted(() => {
  window.addEventListener('keydown', handleKeyDown);
});

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeyDown);
});

// Handle the settings tab index change
watch(() => config.settingsTabIndex, (newValue) => {
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
watch(() => config.settingsTabIndex, (newValue) => {
  emit('settings-settingsTabIndex-changed', newValue);
});
watch(() => config.appearance, (newValue) => {
  setTheme(newValue);
  emit('settings-appearance-changed', newValue);
});
watch(() => config.language, (newValue) => {
  locale.value = newValue;
  emit('settings-language-changed', newValue);
});
watch(() => config.showHiddenAlbum, (newValue) => {
  emit('settings-showHiddenAlbum-changed', newValue);
});
watch(() => config.showButtonText, (newValue) => {
  emit('settings-showButtonText-changed', newValue);
});
watch(() => config.showToolTip, (newValue) => {
  emit('settings-showToolTip-changed', newValue);
});
watch(() => config.showStatusBar, (newValue) => {
  emit('settings-showStatusBar-changed', newValue);
});
watch(() => config.showComment, (newValue) => {
  emit('settings-showComment-changed', newValue);
});
watch(() => config.debugMode, (newValue) => {
  emit('settings-debugMode-changed', newValue);
});

// grid view settings
watch(() => config.grid.size, debounce((newValue) => {
  emit('settings-gridSize-changed', newValue);
}, 100));
watch(() => config.grid.scaling, (newValue) => {
  emit('settings-gridScaling-changed', newValue);
});
watch(() => config.grid.labelPrimary, (newValue) => {
  emit('settings-gridLabelPrimary-changed', newValue);
});
watch(() => config.grid.labelSecondary, (newValue) => {
  emit('settings-gridLabelSecondary-changed', newValue);
});

// image viewer settings
watch(() => config.mouseWheelMode, (newValue) => {
  emit('settings-mouseWheelMode-changed', newValue);
});
watch(() => config.slideShowInterval, (newValue) => {
  emit('settings-slideShowInterval-changed', newValue);
});
watch(() => config.autoPlayVideo, (newValue) => {
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
      config.settingsTabIndex += 1;
      config.settingsTabIndex = config.settingsTabIndex % 4; // 4 tabs
      break;
    case 'Escape':
      appWindow.close(); // Close the window
      break;
  }
}
</script>
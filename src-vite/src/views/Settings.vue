<template>

  <div class="w-screen h-screen flex flex-col border t-color-border overflow-hidden select-none">
    <!-- Title Bar -->
    <TitleBar :titlebar="$t('settings')" :resizable="false" viewName="Settings"/>

    <!-- Main Content -->
    <div 
      class="flex-1 flex p-4 t-color-bg t-color-text overflow-auto t-scrollbar-dark" 
      @contextmenu.prevent
    >
      <!-- Tabs -->
      <div class="w-32 font-bold">
        <div
          v-for="(tab, index) in ['settings_general', 'settings_thumbnail', 'settings_image_viewer', 'settings_about']"
          :key="index"
          :class="[
            'mb-4 px-1 border-l-2 border-transparent cursor-pointer', 
            tabIndex === index ? 't-color-text-selected t-color-border-selected transition-colors duration-300' : '',
          ]"
          @click="tabIndex = index"
        >
          {{ $t(tab) }}
        </div>
      </div>
    
      <div class="flex-grow px-2">

        <!-- General tab -->
        <section v-if="tabIndex === 0">

          <!-- select language -->
          <div class="flex items-center justify-between mb-4">
            <label for="language-select">{{ $t('settings_general_select_language') }}</label>
            <select id="language-select" v-model="config.language" class="t-select">
              <option v-for="(lang, index) in languages" 
                :key="index" 
                :value="lang.value"
                class="t-option"
              >
                {{ lang.label }}
              </option>
            </select>
            <!-- <n-config-provider :theme-overrides="themeOverrides">
              <n-select
                id="language-select"
                v-model:value="config.language" 
                class="border-none focus:ring-0 focus:outline-none"
                :options="languages" 
                label-field="label"
                value-field="value"
                size="small"
                :style="{
                  border: 'none',
                  outline: 'none'
                }"
              />
            </n-config-provider> -->

          </div>

          <!-- Show button text -->
          <!-- <div class="flex items-center justify-between mb-4">
            <label for="show-button-text" >{{ $t('settings_general_show_button_text') }}</label>
            <input id="show-button-text" type="checkbox" v-model="config.showButtonText"/>
          </div> -->

          <div class="flex items-center justify-between mb-4">
            <label for="show-button-text" >{{ $t('settings_general_show_button_text') }}</label>
            <n-config-provider :theme-overrides="themeOverrides">
              <n-switch id="show-button-text" v-model:value="config.showButtonText" size="small" />
            </n-config-provider>
          </div>

          <!-- Dark Mode -->
          <div class="flex items-center justify-between mb-4">
            <label for="dark-mode" >{{ $t('settings_general_dark_mode') }}</label>
            <n-config-provider :theme-overrides="themeOverrides">
              <n-switch id="dark-mode" v-model:value="config.darkMode" size="small" />
            </n-config-provider>
          </div>

          <!-- Debug Mode -->
          <div class="flex items-center justify-between mb-4">
            <label for="debug-mode" >{{ $t('settings_general_debug_mode') }}</label>
            <n-config-provider :theme-overrides="themeOverrides">
              <n-switch id="debug-mode" v-model:value="config.debugMode" size="small" />
            </n-config-provider>
          </div>

        </section>

        <!-- Thumbnail tab -->
        <section v-if="tabIndex === 1">

          <!-- Thumbnail Size -->
          <div class="flex items-center justify-between mb-4">
            <label for="thumbnail_size" >{{ $t('settings_thumbnail_size') }}</label>
            <SliderInput 
              v-model="config.thumbnailSize" 
              :min="120" 
              :max="320" 
              :step="10" 
              label=""
            />
          </div>

          <!-- Thumbnail Image Scaling -->
          <div class="flex items-center justify-between mb-4">
            <label for="thumbnail_image-select">{{ $t('settings_thumbnail_scaling') }}</label>
            <select id="thumbnail_image-select" v-model="config.thumbnailScalingOption" class="t-select">
              <option v-for="(option, index) in thumbnailIScalingOptions" 
                :key="index" 
                :value="option.value"
                class="t-option"
              >
                {{ option.label }}
              </option>
            </select>
          </div>

          <!-- Primary Label -->
          <div class="flex items-center justify-between mb-4">
            <label for="thumbnail_primary-select">{{ $t('settings_thumbnail_label_primary') }}</label>
            <select id="thumbnail_primary-select" v-model="config.thumbnailLabelPrimaryOption" class="t-select">
              <option v-for="(option, index) in thumbnailLabelOptions" 
                :key="index" 
                :value="option.value"
                class="t-option"
              >
                {{ option.label }}
              </option>
            </select>
          </div>

          <!-- Secondary Label -->
          <div class="flex items-center justify-between mb-4">
            <label for="thumbnail_secondary-select">{{ $t('settings_thumbnail_label_secondary') }}</label>
            <select id="thumbnail_secondary-select" v-model="config.thumbnailLabelSecondaryOption" class="t-select">
              <option v-for="(option, index) in thumbnailLabelOptions" 
                :key="index" 
                :value="option.value"
                class="t-option"
              >
                {{ option.label }}
              </option>
            </select>
          </div>

        </section>

        <!-- Image Viewer tab -->
        <section v-else-if="tabIndex === 2">

          <!-- mouse wheel mode -->
          <div class="flex items-center justify-between mb-4">
            <label for="mouse-wheel">{{ $t('settings_image_viewer_mouse_wheel') }}</label>
            <select id="mouse-wheel" v-model="config.mouseWheelMode" class="t-select">
              <option 
                v-for="(item, index) in wheelOptions" 
                :key="index" 
                :value="item.value" 
                class="t-option"
              >
                {{ item.label }}
              </option>
            </select>
          </div>

          <!-- auto play interval -->
          <div class="flex items-center justify-between mb-4">
            <label for="autoplay-interval" >{{ $t('settings_image_viewer_autoplay_interval', {second: config.autoPlayInterval}) }}</label>
            <SliderInput 
              v-model="config.autoPlayInterval" 
              :min="1" 
              :max="30" 
              :step="1" 
              label=""
            />
          </div>

        </section>

        <!-- About Section -->
        <section v-else-if="tabIndex === 3">

          <div class="flex flex-col items-center justify-between mb-4">
            <label class="font-bold">jc-photo</label>
            <label>{{ $t('settings_about_version', { version: '0.1.0' }) }}</label>
            <label>{{ $t('settings_about_author', { author: '@liulichuan' }) }}</label>
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
import { useConfigStore } from '@/stores/configStore';
import { NConfigProvider, NSwitch, NSelect } from 'naive-ui';

import TitleBar from '@/components/TitleBar.vue';
import SliderInput from '@/components/SliderInput.vue';

/// i18n
const { locale, messages } = useI18n();
const localeMsg = computed(() => messages.value[config.language]);

// config store
const config = useConfigStore();

// custom naive ui theme
const themeOverrides = {
  common: {
    primaryColor: '#4BA2E3',    // focus color
  },
}

const appWindow = getCurrentWebviewWindow()

const tabIndex = ref(0);

const languages = [
  { label: 'English', value: 'en' },
  { label: '日本語',  value: 'ja' },
  { label: '中文',    value: 'zh' },
];

// Define the wheel options using computed to react to language changes
const wheelOptions = computed(() => {
  const options = localeMsg.value.settings_image_viewer_mouse_wheel_options; // returns an array
  return [
    { label: options[0], value: 0 },  // 0: previous / next
    { label: options[1], value: 1 },  // 1: zoom in / out
  ];
});

// Define the thumbnail options
const thumbnailIScalingOptions = computed(() => {
  const options = localeMsg.value.settings_thumbnail_scaling_options;
  const result = [];

  for (let i = 0; i < options.length; i++) {
    result.push({ label: options[i], value: i });
  }

  return result;
});

const thumbnailLabelOptions = computed(() => {
  const options = localeMsg.value.settings_thumbnail_label_options;
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

// general settings
watch(() => config.language, (newValue) => {
  locale.value = newValue;
  emit('settings-language-changed', newValue);
});
watch(() => config.showButtonText, (newValue) => {
  emit('settings-showButtonText-changed', newValue);
});

// thumbnail settings
watch(() => config.thumbnailSize, debounce((newValue) => {
    emit('settings-thumbnailSize-changed', newValue);
  }, 100) // Adjust the delay as needed
);
watch(() => config.thumbnailScalingOption, (newValue) => {
  emit('settings-thumbnailScalingOption-changed', newValue);
});
watch(() => config.thumbnailLabelPrimaryOption, (newValue) => {
  emit('settings-thumbnailLabelPrimaryOption-changed', newValue);
});
watch(() => config.thumbnailLabelSecondaryOption, (newValue) => {
  emit('settings-thumbnailLabelSecondaryOption-changed', newValue);
});

// image viewer settings
watch(() => config.mouseWheelMode, (newValue) => {
  emit('settings-mouseWheelMode-changed', newValue);
});
watch(() => config.autoPlayInterval, (newValue) => {
  emit('settings-autoPlayInterval-changed', newValue);
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
      tabIndex.value += 1;
      tabIndex.value = tabIndex.value % 4; // 4 tabs
      break;
    case 'Escape':
      appWindow.close(); // Close the window
      break;
  }
}
</script>
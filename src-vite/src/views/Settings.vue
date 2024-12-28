<template>

  <div class="w-screen h-screen flex flex-col border t-color-border rounded-lg overflow-hidden">
    <!-- Title Bar -->
    <TitleBar :titlebar="$t('settings')" :resizable="false" viewName="Settings"/>

    <!-- Main Content -->
    <div class="flex-1 flex p-4 t-color-bg t-color-text overflow-auto t-scrollbar-dark">

      <!-- General Settings Section -->
      <div class="w-32 font-bold">
        <div :class="['cursor-pointer mb-4', tabIndex === 0 ? 't-color-text-selected' : '']" @click="tabIndex = 0">
          {{ $t('settings_general') }}
        </div>
        <div :class="['cursor-pointer mb-4', tabIndex === 1 ? 't-color-text-selected' : '']" @click="tabIndex = 1">
          {{ $t('settings_image_viewer') }}
        </div>
      </div>
    
      <div class="flex-grow">

        <section v-if="tabIndex === 0">
          <!-- select language -->
          <div class="flex items-center justify-between mb-4">
            <label for="language-select">{{ $t('settings_general_select_language') }}</label>
            <select id="language-select" v-model="config.language"
              class="px-2 py-1 text-sm border rounded-md t-input-color-bg t-color-border t-input-focus"
            >
              <option v-for="(lang, index) in languages" 
                :key="index" 
                :value="lang.value"
                class="t-option"
              >
                {{ lang.label }}
              </option>
            </select>
          </div>

          <!-- Show button text -->
          <div class="flex items-center justify-between mb-4">
            <label for="show-button-text" >{{ $t('settings_general_show_button_text') }}</label>
            <input id="show-button-text" type="checkbox" v-model="config.showButtonText"/>
          </div>

          <!-- Dark Mode -->
          <div class="flex items-center justify-between mb-4">
            <label for="dark-mode" >{{ $t('settings_general_dark_mode') }}</label>
            <input id="dark-mode" type="checkbox" v-model="config.darkMode" />
          </div>

          <!-- Debug Mode -->
          <div class="flex items-center justify-between mb-4">
            <label for="debug-mode" >{{ $t('settings_general_debug_mode') }}</label>
            <input id="debug-mode" type="checkbox" v-model="config.debugMode" />
          </div>

        </section>

        <section v-else-if="tabIndex === 1">

          <!-- mouse wheel mode -->
          <div class="flex items-center justify-between mb-4">
            <label for="mouse-wheel">{{ $t('settings_image_viewer_mouse_wheel') }}</label>
            <select id="language-select" v-model="config.mouseWheelMode"
              class="px-2 py-1 text-sm border rounded-md t-input-color-bg t-color-border t-input-focus"
            >
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

      </div>

    </div>  

  </div>
  
</template>

<script setup>

import { ref, watch, computed } from 'vue';
import { emit } from '@tauri-apps/api/event';
import { useI18n } from 'vue-i18n';
import { useConfigStore } from '@/stores/configStore';

import TitleBar from '@/components/TitleBar.vue';
import SliderInput from '@/components/SliderInput.vue';

/// i18n
const { locale, messages } = useI18n();
const localeMsg = computed(() => messages.value[config.language]);

// config store
const config = useConfigStore();

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

// watch selected language
watch(() => config.language, (newValue) => {
  console.log('Language changed to:', newValue);
  locale.value = newValue;
  emit('settings-language-changed', newValue);
});

// watch show button text
watch(() => config.showButtonText, (newValue) => {
  console.log('Show Button Text:', newValue);
  emit('settings-showButtonText-changed', newValue);
});
// watch mouse whell mode
watch(() => config.mouseWheelMode, (newValue) => {
  console.log('Mouse Wheel Mode:', newValue);
  emit('settings-mouseWheelMode-changed', newValue);
});
// watch autoplay interval
watch(() => config.autoPlayInterval, (newValue) => {
  console.log('Auto Play Interval:', newValue);
  emit('settings-autoPlayInterval-changed', newValue);
});

</script>
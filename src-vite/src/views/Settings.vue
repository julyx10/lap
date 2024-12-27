<template>

  <div class="w-screen h-screen flex flex-col border t-color-border rounded-lg overflow-hidden">
    <!-- Title Bar -->
    <TitleBar :titlebar="$t('settings')" :resizable="false" :hasSearch="false"/>

    <!-- Main Content -->
    <div class="flex-1 flex p-4 t-color-bg t-color-text overflow-auto t-scrollbar-dark">

      <!-- General Settings Section -->
      <div class="w-32 text-lg font-bold mb-2">
        <div >
          {{ $t('settings_general') }}
        </div>
        <div >
          {{ $t('settings_image_viewer') }}
        </div>
      </div>
    
      <div class="flex-grow">

        <section>
          <!-- select language -->
          <div class="mb-4">
            <label for="language-select" class="block text-lg">{{ $t('settings_general_select_language') }}</label>
            <select id="language-select" v-model="config.language"
              class="px-2 py-1 w-full text-sm border rounded-md t-input-color-bg t-color-border t-input-focus"
            >
              <option v-for="(lang, index) in languages" :key="index" :value="lang.value">
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

      </div>

    </div>  

  </div>
  
</template>

<script setup>

import { watch } from 'vue';
import { emit } from '@tauri-apps/api/event';
import { useI18n } from 'vue-i18n';
import { useConfigStore } from '@/stores/configStore';

import TitleBar from '@/components/TitleBar.vue';

/// i18n
const { locale, messages } = useI18n();
// const localeMsg = computed(() => messages.value[config.language]);

// config store
const config = useConfigStore();

const languages = [
      { label: 'English', value: 'en' },
      { label: '中文', value: 'zh' },
      { label: '日本語', value: 'jp' },
    ];

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

</script>
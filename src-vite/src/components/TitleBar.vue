<template>

  <!-- Custom Title Bar -->
  <div class="w-full h-10 flex items-center justify-between t-color-bg select-none">
    
    <!-- Left Section (Title) -->
    <span class="ml-4 t-color-text" @mousedown="appWindow.startDragging()">{{ titlebar }}</span>

    <!-- Draggable Area with Search Box -->
    <div id="titlebar" class="flex-grow h-full flex justify-center items-center t-color-text " @mousedown="appWindow.startDragging()">
      <div class="relative w-1/3">
        <!-- Search Box -->
        <input
          type="text"
          placeholder="Search..."
          class="px-2 py-1 w-full bg-transparent border t-color-border rounded-lg focus:outline-0 focus:bg-gray-800"
          @mousedown.stop
        />
        <!-- Search Icon -->
        <IconSearch class="absolute right-2 top-1/2 transform -translate-y-1/2 h-5 w-5" />
      </div>
    </div>

    <!-- Window Control Buttons -->
    <div class="h-full flex items-center">
      <IconMinus class="t-window-btn" @click="minimizeWindow" />
      <IconMaximize v-if="!isMaximized" class="t-window-btn" @click="toggleMaximizeWindow" />
      <IconRestore v-if="isMaximized" class="t-window-btn" @click="toggleMaximizeWindow" />
      <IconClose class="t-window-btn hover:bg-red-600" @click="closeWindow" />
    </div>

  </div>

</template>




<script setup>
import { ref } from 'vue';
import { appWindow } from '@tauri-apps/api/window';

import IconSearch from '@/assets/search.svg';
import IconMinus from '@/assets/window-minus.svg';
import IconMaximize from '@/assets/window-maximize.svg';
import IconRestore from '@/assets/window-restore.svg';
import IconClose from '@/assets/close.svg';

const props = defineProps({
  titlebar: String
});

const isMaximized = ref(false);

const minimizeWindow = () => {
  appWindow.minimize();
};

const toggleMaximizeWindow = () => {
  appWindow.isMaximized().then((maximized) => {
    if (maximized) {
      isMaximized.value = false;
      appWindow.unmaximize();
    } else {
      isMaximized.value = true;
      appWindow.maximize();
    }
  });
};

const closeWindow = () => {
  appWindow.close();
};

</script>

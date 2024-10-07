<template>

  <!-- Custom Title Bar -->
  <div class="w-full h-10 flex items-center justify-between t-color-bg select-none">

    <span class="ml-4 t-color-text" @mousedown="appWindow.startDragging()">{{ titlebar }}</span>

    <!-- Draggable Area -->
    <div id="titlebar" class="flex-grow h-full" @mousedown="appWindow.startDragging()"></div>
    
    <div class="h-full flex items-center ">
      <IconMinus    class="t-window-btn" @click="minimizeWindow" />
      <IconMaximize class="t-window-btn" v-if="!isMaximized" @click="toggleMaximizeWindow" />
      <IconRestore  class="t-window-btn" v-if="isMaximized" @click="toggleMaximizeWindow" />
      <IconClose    class="t-window-btn hover:bg-red-600" @click="closeWindow" />
    </div>
  </div>

</template>


<script setup>
import { ref } from 'vue';
import { appWindow } from '@tauri-apps/api/window';

import IconMinus from '@/assets/minus.svg';
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

<template>

  <!-- Custom Title Bar -->
  <div class="w-full h-10 flex items-center justify-between t-color-bg select-none">

    <span class="ml-4 t-color-text">{{ titlebar }}</span>

    <!-- Draggable Area -->
    <div id="titlebar" class="flex-grow h-full" @mousedown="appWindow.startDragging()"></div>
    
    <div class="h-full flex items-center ">
      <IconMinus    class="p-3 size-10 t-color-text t-icon-hover t-color-bg-hover" @click="minimizeWindow" />
      <IconMaximize class="p-3 size-10 t-color-text t-icon-hover t-color-bg-hover" @click="toggleMaximizeWindow" />
      <IconClose    class="p-3 size-10 t-color-text t-icon-hover hover:bg-red-600" @click="closeWindow" />
    </div>
  </div>

</template>


<script setup>

import { appWindow } from '@tauri-apps/api/window';

import IconMinus from '@/assets/minus.svg';
import IconMaximize from '@/assets/maximize-1.svg';
import IconClose from '@/assets/close.svg';

const props = defineProps({
  titlebar: String
});

const minimizeWindow = () => {
  appWindow.minimize();
};

const toggleMaximizeWindow = () => {
  appWindow.isMaximized().then((maximized) => {
    if (maximized) {
      appWindow.unmaximize();
    } else {
      appWindow.maximize();
    }
  });
};

const closeWindow = () => {
  appWindow.close();
};

</script>

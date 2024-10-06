<template>

  <!-- Custom Title Bar -->
  <div id="titlebar" class="w-full h-10 flex items-center justify-between t-color-bg select-none">

    <span class="text-white ml-4">{{ titlebar }}</span>

    <!-- Draggable Area -->
    <div class="flex-grow h-full" style="-webkit-app-region: drag;"></div>
    
    <div class="h-full flex items-center space-x-4">
      <IconMinus    class="p-1 t-color-text t-icon-hover t-color-bg-hover" @click="minimizeWindow" />
      <IconMaximize class="p-1 t-color-text t-icon-hover t-color-bg-hover" @click="toggleMaximizeWindow" />
      <IconClose    class="p-1 t-color-text t-icon-hover hover:bg-red-600" @click="closeWindow" />
    </div>
  </div>

</template>


<script setup>

import { appWindow } from '@tauri-apps/api/window';

import IconMinus from '@/assets/minus.svg';
import IconMaximize from '@/assets/arrows-pointing-out.svg';
import IconClose from '@/assets/x-mark.svg';

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


<style>
/* Ensure the full app is styled properly */
html,
body {
  margin: 0;
  padding: 0;
  height: 100%;
}
</style>

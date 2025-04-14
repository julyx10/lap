<template>

  <!-- Custom Title Bar -->
  <div 
    class="w-full h-10 flex items-center justify-between font-bold t-color-bg select-none cursor-default"
    @contextmenu.prevent
    data-tauri-drag-region
  >
    
    <!-- Title Name -->
    <span 
      :class="[
        't-color-text text-nowrap',
        isWin ? 'ml-4' : '',
        isMac ? 'm-auto' : ''
      ]" 
      data-tauri-drag-region
    >
      {{ titlebar }}
    </span>

    <!-- <div id="titlebar" class="flex-grow h-full flex justify-center items-center t-color-text" data-tauri-drag-region> -->
      <!-- <SearchBox 
        v-if="viewName==='Home'" 
        class="relative w-1/3 min-w-[100px] max-w-[400px] invisible md:visible" 
        id="responsiveDiv"
        v-model="searchValue"
        @mousedown.stop
      /> -->
    <!-- </div> -->

    <!-- Window Control Buttons -->
    <div v-if="isWin" class="h-full flex items-center">
      <IconMinus v-if="resizable" class="t-window-btn" @click="minimizeWindow" />
      <component v-if="resizable" :is="isMaximized ? IconRestore : IconMaximize" class="t-window-btn" @click="toggleMaximizeWindow" />
      <IconClose class="t-window-btn hover:bg-red-600" @click="closeWindow" />
    </div>

  </div>

</template>

<script setup>

import { ref, watch } from 'vue';
import { emit } from '@tauri-apps/api/event';
import { getCurrentWindow  } from '@tauri-apps/api/window';
import { isWin, isMac } from '@/common/utils';

// import SearchBox from '@/components/SearchBox.vue';

import { 
  IconMinus,
  IconMaximize,
  IconRestore,
  IconClose 
} from '@/common/icons';

const props = defineProps({
  titlebar: {
    type: String,
    required: true,
  },
  viewName: {
    type: String,
    required: false,
  },
  resizable: {
    type: Boolean,
    default: true,
  }
});

const searchValue = ref('');

const appWindow = getCurrentWindow();
const isMaximized = ref(false);

watch(() => searchValue.value, (newValue) => { 
  console.log('searchValue:', newValue);
  emit('message-from-titlebar', { message: 'search', search: searchValue.value });
});

// drag window
// const onMousedown = (e) => {
//   if (e.detail === 1 && !isMaximized.value) {   // 1: single click
//     appWindow.startDragging();
//   }
// };

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

<style>
@media (max-width: 400px) {
  #responsiveDiv {
    visibility: hidden;
  }
}
@media (min-width: 400px) {
  #responsiveDiv {
    visibility: visible;
  }
}
</style>
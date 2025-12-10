<template>

  <!-- Custom Title Bar -->
  <div 
    :class="[
      'w-full flex items-center justify-between font-bold select-none cursor-default',
      viewName==='ImageViewer' ? 'h-12' : 'h-10',
    ]"
    @contextmenu.prevent
    data-tauri-drag-region
  >
    <!-- Title Name -->
    <span 
      :class="[
        'text-nowrap',
        isWin ? 'ml-4' : '',
        isMac ? 'm-auto' : ''
      ]" 
      data-tauri-drag-region
    >
      {{ titlebar }}
    </span>

    <!-- <div id="titlebar" class="grow h-full flex justify-center items-center" data-tauri-drag-region> -->
      <!-- <SearchBox 
        v-if="viewName==='Home'" 
        class="relative w-1/3 min-w-[100px] max-w-[400px] invisible md:visible" 
        id="responsiveDiv"
        v-model="searchValue"
        @mousedown.stop
      /> -->
    <!-- </div> -->

    <!-- Window Control Buttons -->
    <div v-if="isWin" class="h-10 mb-auto flex items-center">
      <IconWinMinus v-if="resizable" 
        class="p-3 w-12 h-full text-base-content/70 hover:text-base-content hover:bg-base-100 transition-colors duration-300" 
        @click="minimizeWindow" 
      />
      <component v-if="resizable" :is="isMaximized ? IconWinRestore : IconWinMaximize" 
        class="p-3 w-12 h-full text-base-content/70 hover:text-base-content hover:bg-base-100 transition-colors duration-300" 
        @click="toggleMaximizeWindow" 
      />
      <IconClose 
        class="p-3 w-12 h-full text-base-content/70 hover:text-base-content hover:bg-red-500 transition-colors duration-300" 
        @click="closeWindow" 
      />
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
  IconWinMinus,
  IconWinMaximize,
  IconWinRestore,
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
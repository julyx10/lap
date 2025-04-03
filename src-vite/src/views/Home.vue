<template>
  
  <div class="w-screen h-screen flex flex-col overflow-hidden">
    <!-- Title Bar -->
    <TitleBar v-if="isWin" titlebar="jc-photo" viewName="Home"/>

    <!-- Main Content -->
    <div class="flex-1 flex t-color-bg t-color-text overflow-hidden">

      <!-- left toolbar -->
      <div tabindex="-1"
        ref="divToolbar" 
        class="pt-10 pb-4 z-10 flex flex-col justify-between t-color-bg" style="user-select: none; min-width: 68px;"
        @contextmenu.prevent
        data-tauri-drag-region
      >
        <!-- toolbar items -->
        <div class="flex flex-col items-center space-y-6" data-tauri-drag-region>
          <div v-for="(item, index) in toolbars" 
            class="flex flex-col items-center t-icon-hover" 
            :key="index" 
            @click="clickToolbarItem(index)"
          >
            <component 
              :is="item.icon" 
              :class="['t-icon-size', config.toolbarIndex === index ? 't-icon-focus' : '']" 
            />
            <p v-if="config.showButtonText" 
              :class="['text-xs', config.toolbarIndex === index ? 't-color-text-focus' : '']">
              {{ item.text }}
            </p>
          </div>
        </div>

        <!-- settings icon -->
        <div class="flex flex-col items-center t-icon-hover" @click="clickSettings" data-tauri-drag-region>
          <IconSettings :class="['t-icon-size']"  />
          <p v-if="config.showButtonText" 
              class="text-xs">
              {{ $t('settings') }}
            </p>
        </div>
      </div>

      <!-- left pane -->
      <transition
        enter-active-class="transition-transform duration-200"
        enter-from-class="-translate-x-full"
        enter-to-class="translate-x-0"
        leave-active-class="transition-transform duration-200"
        leave-from-class="translate-x-0"
        leave-to-class="-translate-x-full"
      >
        <div v-show="config.toolbarIndex > 1 && showLeftPane" 
          class="w-96 min-w-32 py-1 flex" 
          :style="{ width: config.leftPaneWidth + 'px' }"
        >
          <Album    v-show="config.toolbarIndex === 2" :titlebar="$t('album')"/>
          <Calendar v-show="config.toolbarIndex === 3" :titlebar="$t('calendar')"/>
          <Location v-show="config.toolbarIndex === 4" :titlebar="$t('location')"/>
          <People   v-show="config.toolbarIndex === 5" :titlebar="$t('people')"/>
          <Camera   v-show="config.toolbarIndex === 6" :titlebar="$t('camera')"/>
        </div>
      </transition>
      
      <!-- splitter -->
      <div v-if="config.toolbarIndex > 1" class="w-1 hover:bg-sky-700 cursor-ew-resize" 
        @mousedown="startDraggingSplitter"
        @mouseup="stopDraggingSplitter"
      ></div>
       
      <!-- content area -->
      <div 
        :class="[
          'flex-1 flex relative t-color-bg-light',
          isWin ? 'rounded-tl-lg' : '',
        ]"
      >
        <Content :titlebar="toolbars[config.toolbarIndex].text"/>
      </div>
    </div>
  </div>

</template>
   

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { WebviewWindow } from '@tauri-apps/api/webviewWindow';
import { useConfigStore } from '@/stores/configStore';
import { isWin, isMac } from '@/common/utils';

// vue components
import TitleBar from '@/components/TitleBar.vue';
import Album from '@/components/Album.vue';
import Calendar from '@/components/Calendar.vue';
import Camera from '@/components/Camera.vue';
import Location from '@/components/Location.vue';
import People from '@/components/People.vue';
import Content from '@/components/Content.vue';

import {
  IconHome,
  IconUnFavorite,
  IconFolder,
  IconCalendar,
  IconLocation,
  IconPeople,
  IconCamera,
  IconSettings,
} from '@/common/icons';

/// i18n
import { useI18n } from 'vue-i18n';
const { locale, messages } = useI18n();
const localeMsg = computed(() => messages.value[locale.value]);

// config store
const config = useConfigStore();

// const appWindow = getCurrentWebviewWindow()

// toolbar 
const toolbars = computed(() =>  [
  { icon: IconHome,     text: localeMsg.value.home },
  { icon: IconUnFavorite, text: localeMsg.value.favorite },
  { icon: IconFolder,    text: localeMsg.value.album },
  { icon: IconCalendar, text: localeMsg.value.calendar },
  { icon: IconLocation, text: localeMsg.value.location },
  { icon: IconPeople,   text: localeMsg.value.people }, 
  { icon: IconCamera,   text: localeMsg.value.camera },
  // { icon: IconTag,      text: localeMsg.value.tag },
]);

const showLeftPane = ref(true);

/// Splitter for resizing the left pane
const divToolbar = ref(null);
const isDraggingSplitter = ref(false);

onMounted(() => {
  window.addEventListener('keydown', handleKeyDown);
  // isFullScreen.value = await appWindow.isMaximized();
});

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeyDown);
});

// Handle keydown event
function handleKeyDown(event) {
  // if (event.key === 'Escape') {
  //   appWindow.minimize();
  // }
};

const clickToolbarItem = async (index) => {
  if(config.toolbarIndex === index) {
    showLeftPane.value = !showLeftPane.value;
  } else {
    showLeftPane.value = true;
  }
  config.toolbarIndex = index;

  // await appWindow.setTitle(toolbars.value[index].text);
};

// Dragging the splitter
function startDraggingSplitter(event) {
  isDraggingSplitter.value = true;
  document.addEventListener('mousemove', handleMouseMove);
  document.addEventListener('mouseup', stopDraggingSplitter);
}

// Stop dragging the splitter
function stopDraggingSplitter() {
  isDraggingSplitter.value = false;
  document.removeEventListener('mousemove', handleMouseMove);
  document.removeEventListener('mouseup', stopDraggingSplitter);
}

// Handle mouse move event
function handleMouseMove(event) {
  if (isDraggingSplitter.value && divToolbar.value) {
    const toolbarWidth = divToolbar.value.offsetWidth + 1;   // 1: border width
    config.leftPaneWidth = Math.max(event.clientX - toolbarWidth, 100); // Adjust for toolbar width and minimum width
  }
}

/// click settings icon
async function clickSettings() {
  // check if the settings window is already open
  const settingsWindow = await WebviewWindow.getByLabel('settings');
  if (settingsWindow) {
    settingsWindow.show();
    settingsWindow.setFocus();
    return;
  }

  const options = {
    url: '/settings',
    title: 'Settings',
    width: 600,
    height: 400,
    resizable: false,
    decorations: isMac, // true for macOS, false for Windows
    transparent: isWin, // true for Windows, false for macOS
    ...(isMac && {
      titleBarStyle: 'Overlay',
      hiddenTitle: true,
      minimizable: false,
    }),
  };

  // create a new settings window
  const newSettingsWindow = new WebviewWindow('settings', options);
  
  newSettingsWindow.once('tauri://created', () => {
    console.log('settings window created');
  });

  newSettingsWindow.once('tauri://close-requested', () => {
    newSettingsWindow.close();
    console.log('settings window closed');
  });
}

</script>

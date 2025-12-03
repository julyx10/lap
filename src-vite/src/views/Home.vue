<template>
  
  <div class="w-screen h-screen flex flex-col overflow-hidden select-none bg-base-300 text-base-content/70">
    <!-- Title Bar -->
    <TitleBar v-if="isWin" titlebar="jc-photo" viewName="Home"/>

    <!-- Main Content -->
    <div class="flex-1 flex overflow-hidden">

      <transition
        enter-active-class="transition-all duration-200 ease-in-out overflow-hidden"
        leave-active-class="transition-all duration-200 ease-in-out overflow-hidden"
        enter-from-class="!w-0 opacity-0"
        enter-to-class="opacity-100"
        leave-from-class="opacity-100"
        leave-to-class="!w-0 opacity-0"
      >
        <!-- left pane -->
        <div v-if="config.home.showLeftPane"
          :class="[
            'relative flex bg-base-200 rounded-box my-1 ml-1 z-10 select-none', 
            config.home.sidebarIndex === 0 || !showPanel ? 'mt-12 mb-8': '',
            isDraggingSplitter ? 'no-transition' : 'transition-all duration-200 ease-in-out',
          ]"
          :style="{ width: config.home.sidebarIndex > 0  && showPanel ? config.home.leftPaneWidth + 'px' : '64px' }"
          data-tauri-drag-region
        >
          <!-- side bar -->
          <div 
            :class="[
              'fixed min-w-16 bottom-10 flex flex-col items-center space-y-2', 
              isWin ? 'top-2' : 'top-14'
            ]" 
            data-tauri-drag-region
          >
            <div v-for="(item, index) in buttons" :key="index">
              <TButton 
                :buttonSize="'large'" 
                :icon="item.icon" 
                :text="item.text" 
                :selected="config.home.sidebarIndex === index"
                @click="clickSidebar(index)"
              />
            </div>

            <TButton class="mt-auto"
              :buttonSize="'large'" 
              :icon="IconSettings" 
              :text="$t('sidebar.settings')" 
              @click="clickSettings"
            />
          </div>

          <!-- panel-->
          <div v-if="config.home.sidebarIndex > 0 && showPanel" class="ml-16 pr-0.5 flex-1 overflow-hidden">
            <component :is="buttons[config.home.sidebarIndex].component" :titlebar="buttons[config.home.sidebarIndex].text"/>
          </div>

        </div>
      </transition> 
      
      <!-- splitter -->
      <div
        class="w-1 transition-colors shrink-0"
        :class="{
          'hover:bg-primary cursor-ew-resize': config.home.showLeftPane && config.home.sidebarIndex > 0,
          'bg-primary': config.home.showLeftPane && config.home.sidebarIndex > 0 && isDraggingSplitter,
        }" 
        @mousedown="startDraggingSplitter"
        @mouseup="stopDraggingSplitter"
      ></div>
       
      <!-- content area -->
      <div 
        :class="[
          'flex-1 flex relative',
          isWin ? 'rounded-tl-box' : '',
        ]"
      >
        <Content :titlebar="buttons[config.home.sidebarIndex].text"/>
      </div>
    </div>

    <!-- logo -->
    <div class="fixed bottom-2 left-3 text-[12px] text-base-content/10 transition-all duration-200 ease-in-out">
      <span>jc-photo</span>
    </div>
  </div>

</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import { useI18n } from 'vue-i18n';
import { WebviewWindow } from '@tauri-apps/api/webviewWindow';
import { config } from '@/common/config';
import { isWin, isMac } from '@/common/utils';

// vue components
import Album from '@/components/Album.vue';
import Favorite from '@/components/Favorite.vue';
import Tag from '@/components/Tag.vue';
import Calendar from '@/components/Calendar.vue';
import Location from '@/components/Location.vue';
// import People from '@/components/People.vue';
import Camera from '@/components/Camera.vue';
import TitleBar from '@/components/TitleBar.vue';
import TButton from '@/components/TButton.vue';
import Content from '@/components/Content.vue';

import {
  IconHome,
  IconFolder,
  IconFavorite,
  IconTag,
  IconCalendar,
  IconLocation,
  IconPeople,
  IconCamera,
  IconSettings,
} from '@/common/icons';

/// i18n
const { locale, messages } = useI18n();
const localeMsg = computed(() => messages.value[locale.value]);

// buttons 
const buttons = computed(() =>  [
  { 
    icon: IconHome,
    component: null,
    text: localeMsg.value.sidebar.home
  },
  { 
    icon: IconFolder,  
    component: Album,
    text: localeMsg.value.sidebar.album 
  },
  { 
    icon: IconFavorite, 
    component: Favorite,
    text: localeMsg.value.sidebar.favorite 
  },
  { 
    icon: IconTag,
    component: Tag,
    text: localeMsg.value.sidebar.tag 
  },
  { 
    icon: IconCalendar, 
    component: Calendar,
    text: localeMsg.value.sidebar.calendar 
  },
  { 
    icon: IconLocation, 
    component: Location, 
    text: localeMsg.value.sidebar.location 
  },
  { 
    icon: IconCamera,  
    component: Camera,
    text: localeMsg.value.sidebar.camera 
  },
  // { icon: IconPeople, component: People, text: localeMsg.value.sidebar.people }, 
]);

/// Splitter for resizing the left pane
const isDraggingSplitter = ref(false);

const showPanel = ref(true);

// click sidebar
function clickSidebar(index: number) {
  if (config.home.sidebarIndex === index) {
    showPanel.value = !showPanel.value;
  } else {
    showPanel.value = true;
    config.home.sidebarIndex = index;
  }
}

// Dragging the splitter
function startDraggingSplitter(event: MouseEvent) {
  if(!config.home.showLeftPane) return; // no left pane or left pane is hidden

  isDraggingSplitter.value = true;
  document.addEventListener('mousemove', handleMouseMove);
  document.addEventListener('mouseup', stopDraggingSplitter);
}

// Stop dragging the splitter
function stopDraggingSplitter(event: MouseEvent) {
  isDraggingSplitter.value = false;
  document.removeEventListener('mousemove', handleMouseMove);
  document.removeEventListener('mouseup', stopDraggingSplitter);
}

// Handle mouse move event
function handleMouseMove(event: MouseEvent) {
  if (isDraggingSplitter.value) {
    const maxLeftPaneWidth = window.innerWidth / 2;
    config.home.leftPaneWidth = Math.max(160, Math.min(event.clientX - 6, maxLeftPaneWidth)); // -2: border width(2px)
  }
}

/// click settings icon
async function clickSettings() {
  // check if the settings window is already open
  const settingsWindow = await WebviewWindow.getByLabel('settings');
  if (settingsWindow) {
    settingsWindow.show();
    return;
  }

  const options = {
    url: '/settings',
    title: 'Settings',
    width: 600,
    height: 400,
    minWidth: 600,
    minHeight: 400,
    resizable: true,
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

<template>
  
  <div class="w-screen h-screen flex flex-col overflow-hidden select-none bg-base-300 text-base-content/70">
    <!-- Title Bar -->
    <TitleBar v-if="isWin" titlebar="jc-photo" viewName="Home"/>

    <!-- Main Content -->
    <div class="flex-1 flex overflow-hidden">

      <!-- left side bar -->
      <div
        ref="divSideBar" 
        :class="[
          'pb-4 rounded-r-lg z-10 bg-base-200',
          isWin ? 'pt-2' : 'pt-10'
        ]" 
        style="user-select: none; min-width: 68px;"
        data-tauri-drag-region
      >
        <!-- button items -->
        <div class="h-full flex flex-col items-center" data-tauri-drag-region>

          <div class="space-y-2" >
            <div v-for="(item, index) in buttons" 
              :key="index" 
            >
              <TButton 
                :buttonSize="'large'" 
                :icon="item.icon" 
                :text="item.text" 
                :selected="config.home.sidebarIndex === index"
                @click="clickButton(index)"
              />
            </div>
          </div>

          <TButton class="mt-auto"
            :buttonSize="'large'" 
            :icon="IconSettings" 
            :text="$t('sidebar.settings')" 
            @click="clickSettings"
          />
        </div>

      </div>

      <!-- left pane -->
      <transition
        enter-from-class="left-pane-hide"
        enter-to-class="left-pane-show"
        leave-from-class="left-pane-show"
        leave-to-class="left-pane-hide"
      >
        <div v-show="config.home.sidebarIndex > 0 && showLeftPane" 
          :class="['py-1 flex bg-base-200 left-pane overflow-hidden rounded-r-lg', { 'no-transition': isDraggingSplitter }]" 
          :style="{ '--left-pane-width': config.home.leftPaneWidth + 'px' }"
        >
          <component :is="buttons[config.home.sidebarIndex].component" :titlebar="buttons[config.home.sidebarIndex].text"/>
        </div>
      </transition>
      
      <!-- splitter -->
      <div 
        :class="[
          'w-1 transition-colors',
          config.home.sidebarIndex > 0 && showLeftPane ? 'hover:bg-primary cursor-ew-resize' : '',
          config.home.sidebarIndex > 0 && showLeftPane && isDraggingSplitter ? 'bg-primary' : 'bg-base-300'
        ]" 
        @mousedown="startDraggingSplitter"
        @mouseup="stopDraggingSplitter"
      ></div>
       
      <!-- content area -->
      <div 
        :class="[
          'flex-1 flex relative bg-base-300',
          isWin ? 'rounded-tl-lg' : '',
        ]"
      >
        <Content :titlebar="buttons[config.home.sidebarIndex].text"/>
      </div>
    </div>
  </div>

</template>
   

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue';
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

import { listen } from '@tauri-apps/api/event';

let unlistenKeydown: () => void;

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

const showLeftPane = ref(true);

/// Splitter for resizing the left pane
const divSideBar = ref(null);
const isDraggingSplitter = ref(false);

onMounted(async () => {
  unlistenKeydown = await listen('global-keydown', handleKeyDown);
  // isFullScreen.value = await appWindow.isMaximized();
});

onUnmounted(() => {
  if (unlistenKeydown) {
    unlistenKeydown();
  }
});

// Handle keydown event
function handleKeyDown(event) {
  const { key, ctrlKey, metaKey } = event.payload;
  // if (event.key === 'Escape') {
  //   appWindow.minimize();
  // }
};

const clickButton = async (index) => {
  if(config.home.sidebarIndex === index) {
    showLeftPane.value = !showLeftPane.value;
  } else {
    showLeftPane.value = true;
  }
  config.home.sidebarIndex = index;

  // await appWindow.setTitle(buttons.value[index].text);
};

// Dragging the splitter
function startDraggingSplitter(event) {
  if(config.home.sidebarIndex <= 0 || !showLeftPane.value) return; // no left pane or left pane is hidden

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
  if (isDraggingSplitter.value && divSideBar.value) {
    const toolbarWidth = divSideBar.value.offsetWidth + 2;   // 2: border width(2px) * 2
    config.home.leftPaneWidth = Math.max(event.clientX - toolbarWidth, 100); // Adjust for toolbar width and minimum width
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
<style scoped>
.left-pane {
  width: var(--left-pane-width);
  transition: width 200ms ease;
  will-change: width;
}
.left-pane-hide {
  width: 0 !important;
}
.left-pane-show {
  width: var(--left-pane-width) !important;
}
.no-transition {
  transition: none !important;
}
</style>

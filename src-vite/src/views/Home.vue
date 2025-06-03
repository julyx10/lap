<template>
  
  <div class="w-screen h-screen flex flex-col overflow-hidden select-none text-base-content/70">
    <!-- Title Bar -->
    <TitleBar v-if="isWin" titlebar="jc-photo" viewName="Home"/>

    <!-- Main Content -->
    <div class="flex-1 flex overflow-hidden">

      <!-- left toolbar -->
      <div tabindex="-1"
        ref="divToolbar" 
        :class="[
          'pt-10 pb-4 z-10 flex flex-col justify-between bg-base-200',
          isWin ? 'pt-4' : 'pt-10'
        ]" 
        style="user-select: none; min-width: 68px;"
        data-tauri-drag-region
      >
        <!-- toolbar items -->
        <div class="h-full flex flex-col items-center" data-tauri-drag-region>

          <div class="space-y-2" >
            <div v-for="(item, index) in toolbars" 
              :key="index" 
              @click="clickToolbarItem(index)"
            >
              <TButton 
                :buttonSize="'large'" 
                :icon="item.icon" 
                :text="item.text" 
                :selected="config.toolbarIndex === index"
                @click="$emit('clickToolbarItem', index)"
              />
            </div>
          </div>

          <TButton class="mt-auto"
            :buttonSize="'large'" 
            :icon="IconSettings" 
            :text="$t('settings')" 
            @click="clickSettings"
          />
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
        <div v-show="config.toolbarIndex > 0 && showLeftPane" 
          class="w-96 min-w-32 py-1 flex bg-base-200" 
          :style="{ width: config.leftPaneWidth + 'px' }"
        >
          <Album    v-show="config.toolbarIndex === 1" :titlebar="$t('album')"/>
          <Favorite v-show="config.toolbarIndex === 2" :titlebar="$t('favorite')"/>
          <Tag      v-show="config.toolbarIndex === 3" :titlebar="$t('tag')"/>
          <Calendar v-show="config.toolbarIndex === 4" :titlebar="$t('calendar')"/>
          <Location v-show="config.toolbarIndex === 5" :titlebar="$t('location')"/>
          <!-- <People   v-show="config.toolbarIndex === 6" :titlebar="$t('people')"/> -->
          <Camera   v-show="config.toolbarIndex === 6" :titlebar="$t('camera')"/>
          <Trash    v-show="config.toolbarIndex === 7" :titlebar="$t('trash')"/>
        </div>
      </transition>
      
      <!-- splitter -->
      <div v-if="config.toolbarIndex > 0" 
        class="w-1 hover:bg-primary cursor-ew-resize transition-colors bg-base-300" 
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
        <Content :titlebar="toolbars[config.toolbarIndex].text"/>
      </div>
    </div>
  </div>

</template>
   

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { useI18n } from 'vue-i18n';
import { WebviewWindow } from '@tauri-apps/api/webviewWindow';
import { config, isWin, isMac } from '@/common/utils';

// vue components
import Album from '@/components/Album.vue';
import Favorite from '@/components/Favorite.vue';
import Tag from '@/components/Tag.vue';
import Calendar from '@/components/Calendar.vue';
import Location from '@/components/Location.vue';
// import People from '@/components/People.vue';
import Camera from '@/components/Camera.vue';
import Trash from '@/components/Trash.vue';
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
  IconTrash,
  IconSettings,
} from '@/common/icons';

/// i18n
const { locale, messages } = useI18n();
const localeMsg = computed(() => messages.value[locale.value]);

// toolbar 
const toolbars = computed(() =>  [
  { icon: IconHome,     text: localeMsg.value.home },
  { icon: IconFolder,   text: localeMsg.value.album },
  { icon: IconFavorite, text: localeMsg.value.favorite },
  { icon: IconTag,      text: localeMsg.value.tag },
  { icon: IconCalendar, text: localeMsg.value.calendar },
  { icon: IconLocation, text: localeMsg.value.location },
  // { icon: IconPeople,   text: localeMsg.value.people }, 
  { icon: IconCamera,   text: localeMsg.value.camera },
  { icon: IconTrash,   text: localeMsg.value.trash },
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

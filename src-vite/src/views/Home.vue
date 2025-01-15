<template>
  
  <div class="w-screen h-screen flex flex-col overflow-hidden">
    <!-- Title Bar -->
    <TitleBar titlebar="jc-photo" viewName="Home"/>

    <!-- Main Content -->
    <div class="flex-1 flex t-color-bg t-color-text overflow-hidden">

      <!-- left toolbar -->
      <div 
        ref="divToolbar" 
        class="min-w-12 my-4 flex flex-col justify-between z-10 t-color-bg" style="user-select: none;"
        @contextmenu.prevent
      >
        <!-- toolbar items -->
        <div class="flex flex-col items-center space-y-6">
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

        <!-- draggable area -->
        <div class="flex-grow" @mousedown="dragWindow"></div>

        <!-- settings icon -->
        <div class="flex flex-col items-center t-icon-hover" @click="clickSettings">
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
      <div class="flex-1 flex relative t-color-bg-light rounded-ss-lg">
        <Content :titlebar="toolbars[config.toolbarIndex].text"/>
      </div>
    </div>
  </div>

</template>
   

<script setup lang="ts">

import { ref, computed } from 'vue';
import { WebviewWindow } from '@tauri-apps/api/webviewWindow';
import { getCurrentWindow  } from '@tauri-apps/api/window';
import { useConfigStore } from '@/stores/configStore';

// vue components
import TitleBar from '@/components/TitleBar.vue';
import Album from '@/components/Albums.vue';
import Calendar from '@/components/Calendar.vue';
import Camera from '@/components/Camera.vue';
import Location from '@/components/Location.vue';
import People from '@/components/People.vue';
import Content from '@/components/Content.vue';

/// Toolbar svg icons
import IconHome from '@/assets/home.svg';
import IconFavorite from '@/assets/heart.svg';
import IconAlbum from '@/assets/photo.svg';
import IconCalendar from '@/assets/calendar.svg';
import IconLocation from '@/assets/map-pin.svg';
import IconPeople from '@/assets/user.svg';
import IconCamera from '@/assets/camera.svg';
// import IconTag from '@/assets/tag.svg';
import IconSettings from '@/assets/settings.svg';

/// i18n
import { useI18n } from 'vue-i18n';
const { locale, messages } = useI18n();
const localeMsg = computed(() => messages.value[locale.value]);

// config store
const config = useConfigStore();

// toolbar 
const toolbars = computed(() =>  [
  { icon: IconHome,     text: localeMsg.value.home },
  { icon: IconFavorite, text: localeMsg.value.favorite },
  { icon: IconAlbum,    text: localeMsg.value.album },
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

const clickToolbarItem = (index) => {
  if(config.toolbarIndex === index) {
    showLeftPane.value = !showLeftPane.value;
  } else {
    showLeftPane.value = true;
  }
  config.toolbarIndex = index;
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

// drag toolbar to move window
async function dragWindow() {
  if (await getCurrentWindow().isMaximized()) {
    return;
  }
  getCurrentWindow().startDragging();
}

/// click settings icon
async function clickSettings() {
  // check if the settings window is already open
  const settingsWindow = await WebviewWindow.getByLabel('settings');
  if (settingsWindow) {
    settingsWindow.setFocus();
    return;
  }

  // create a new settings window
  const newSettingsWindow = new WebviewWindow('settings', {
    url: '/settings',
    title: 'Settings',
    width: 600,
    height: 400,
    resizable: false,
    transparent: true,
    decorations: false,
  });
  
  newSettingsWindow.once('tauri://created', () => {
    console.log('settings window created');
  });

  newSettingsWindow.once('tauri://close-requested', () => {
    newSettingsWindow.close();
    console.log('settings window closed');
  });
}

</script>

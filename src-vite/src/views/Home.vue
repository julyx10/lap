<template>
  
  <div class="w-screen h-screen flex flex-col border t-color-border rounded-lg shadow-lg overflow-hidden">
    <!-- Title Bar -->
    <TitleBar titlebar="jc-photo" viewName="Home"/>

    <!-- Main Content -->
    <div class="flex-1 flex t-color-bg t-color-text overflow-hidden">

      <!-- left toolbar -->
      <div ref="divToolbar" class="w-12 my-3 flex flex-col justify-between">
        <div class="flex flex-col items-center space-y-6">
          <div v-for="(item, index) in toolbars" 
            class="flex flex-col items-center t-icon-hover" 
            :key="index" 
            @click="config.toolbarIndex = index"
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
        <div class="flex flex-col items-center t-icon-hover">
          <IconSettings @click="clickSettings" />
          <p v-if="config.showButtonText" 
              class="text-xs">
              {{ $t('settings') }}
            </p>
        </div>
      </div>

      <!-- left pane -->
      <div v-if="config.toolbarIndex > 0" class="w-96 min-w-32 pb-1 flex" :style="{ width: config.leftPaneWidth + 'px' }">
        <Album    v-show="config.toolbarIndex === 1" :titlebar="$t('album')"/>
        <Calendar v-show="config.toolbarIndex === 2" :titlebar="$t('calendar')"/>
        <Location v-show="config.toolbarIndex === 3" :titlebar="$t('location')"/>
        <People   v-show="config.toolbarIndex === 4" :titlebar="$t('people')"/>
        <Camera   v-show="config.toolbarIndex === 5" :titlebar="$t('camera')"/>
      </div>

      <!-- splitter -->
      <div v-if="config.toolbarIndex > 0" class="w-1 hover:bg-sky-700 cursor-ew-resize" @mousedown="startDragging"></div>
      
      <!-- content area -->
      <div class="flex-1 flex relative t-color-bg-light rounded-ss-lg">
        <Content :titlebar="toolbars[config.toolbarIndex].text"/>
      </div>
    </div>
  </div>

</template>
   

<script setup lang="ts">

import { ref, computed, onMounted, onUnmounted, onBeforeUnmount } from 'vue';
import { WebviewWindow } from '@tauri-apps/api/webviewWindow';
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
import IconAlbum from '@/assets/photo.svg';
import IconCalendar from '@/assets/calendar.svg';
import IconLocation from '@/assets/map-pin.svg';
import IconPeople from '@/assets/user.svg';
import IconCamera from '@/assets/camera.svg';
// import IconFavorite from '@/assets/heart.svg';
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
  { icon: IconAlbum,    text: localeMsg.value.album },
  { icon: IconCalendar, text: localeMsg.value.calendar },
  { icon: IconLocation, text: localeMsg.value.location },
  { icon: IconPeople,   text: localeMsg.value.people }, 
  { icon: IconCamera,   text: localeMsg.value.camera },
  // { icon: IconFavorite, text: localeMsg.value.favorite },
  // { icon: IconTag,      text: localeMsg.value.tag },
]);

/// Splitter for resizing the left pane
const divToolbar = ref(null);
const isDragging = ref(false);

const emit = defineEmits(['close-home']);

onMounted(() => {
  document.addEventListener('mouseup', stopDragging);
})

onBeforeUnmount(() => {
  document.removeEventListener('mouseup', stopDragging);
  emit('close-home');
})

// Dragging the splitter
function startDragging(event) {
  isDragging.value = true;
  document.addEventListener('mousemove', handleMouseMove);
  document.addEventListener('mouseup', stopDragging);
}

// Stop dragging the splitter
function stopDragging() {
  isDragging.value = false;
  document.removeEventListener('mousemove', handleMouseMove);
  document.removeEventListener('mouseup', stopDragging);
}

// Handle mouse move event
function handleMouseMove(event) {
  if (isDragging.value && divToolbar.value) {
    const toolbarWidth = divToolbar.value.offsetWidth + 1;   // 1: border width
    config.leftPaneWidth = Math.max(event.clientX - toolbarWidth, 100); // Adjust for toolbar width and minimum width
  }
}

/// click settings icon
function clickSettings() {
  const settingsWindow = new WebviewWindow('settings', {
    url: '/settings',
    title: 'Settings',
    width: 640,
    height: 480,
    resizable: false,
    transparent: true,
    decorations: false,
  });
  
  settingsWindow.once('tauri://created', () => {
    console.log('settings window created');
  });
}

</script>

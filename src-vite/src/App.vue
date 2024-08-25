<template>

  <div class="flex h-screen bg-gray-900 text-gray-500">

    <!-- left toolbar -->
    <div ref="toolbar" class="px-2 flex flex-col justify-between">
      <div>
        <component
        v-for="(item, index) in toolbars"
        :key="index"
        :is="item.icon"
        :class="[
          'my-5 hover:text-gray-100 transition-colors duration-300', 
          toolbar_index === index ? 'text-gray-300' : ''
        ]"
        @click="clickToolbar(index)"
      />
      </div>
      <div>
        <IconSettings class="my-5 hover:text-gray-100 transition-colors duration-300" @click="clickSettings" />
        <IconBug class="my-5 hover:text-gray-100 transition-colors duration-300" @click="clickDebug" />

      </div>

    </div>
      
    <!-- navigation pane -->
    <div 
      v-if="toolbar_index > 0" 
      class="w-96 p-2 min-w-10 overflow-auto scrollbar-thin scrollbar-thumb-gray-700 scrollbar-track-gray-800" 
      :style="{ width: leftPaneWidth + 'px' }"
    >
      <Album v-if="toolbar_index === 1" :titlebar="$t('album')"/>
      <Calendar v-else-if="toolbar_index === 2" :titlebar="$t('calendar')"/>
    </div>

    <!-- splitter -->
    <div 
      v-if="toolbar_index > 0" 
      class="w-1 hover:bg-sky-700 cursor-ew-resize" 
      @mousedown="startDragging" 
    ></div>
    
    <!-- content area -->
    <div class="flex-1 p-4 bg-gray-800 overflow-auto scrollbar-thin scrollbar-thumb-gray-700 scrollbar-track-gray-800">
      <Wall :titlebar="toolbars[toolbar_index].text"/>
    </div>

  </div>

</template>
 

<script setup lang="ts">

import { ref, computed, provide, onMounted, onBeforeUnmount } from 'vue';

/// i18n
import { useI18n } from 'vue-i18n';
const { locale, messages } = useI18n();
const localeMessages = computed(() => messages.value[locale.value]);

import Album from '@/components/Albums.vue';
import Calendar from '@/components/Calendar.vue';
import Wall from '@/components/Wall.vue';


/// global variables
// albums
provide('g_albums', ref([]));       // all albums
provide('g_album_index', ref(-1));  // index of the selected album
provide('g_child_id', ref(-1));     // id of the selected sub-folder


/// Toolbar
// Import SVG files as a Vue component
import IconHome from '@/assets/home.svg';
import IconAlbum from '@/assets/photo.svg';
import IconCalendar from '@/assets/calendar.svg';
import IconPeople from '@/assets/user.svg';
import IconMap from '@/assets/map-pin.svg';
import IconCamera from '@/assets/camera.svg';
import IconSettings from '@/assets/cog-6-tooth.svg';
import IconBug from '@/assets/bug-ant.svg';

// toolbar 
const toolbars = computed(() =>  [
  { icon: IconHome,     text: localeMessages.value.home },
  { icon: IconAlbum,    text: localeMessages.value.album },
  { icon: IconCalendar, text: localeMessages.value.calendar },
  { icon: IconPeople,   text: localeMessages.value.people }, 
  { icon: IconMap,      text: localeMessages.value.map },
  { icon: IconCamera,   text: localeMessages.value.camera },
]);

const toolbar_index = ref(1); // index of the clicked icon
provide('g_toolbar_index', toolbar_index);   


function clickToolbar(index) {
  console.log("clickToolbar...", toolbars.value[index].text);  
  toolbar_index.value = index;
}

function clickSettings() {
  console.log('clickSettings...')
}

function clickDebug() {
  console.log('clickDebug...')

  // toggle locales
  if (locale.value === 'en') {
    locale.value = 'zh'
  } else {
    locale.value = 'en'
  }
}

/// Splitter for resizing the left pane
const toolbar = ref(null);
const leftPaneWidth = ref(300); // Default width of the left pane
const isDragging = ref(false);

function startDragging(event) {
  isDragging.value = true;
  document.addEventListener('mousemove', handleMouseMove);
  document.addEventListener('mouseup', stopDragging);
}

function handleMouseMove(event) {
  if (isDragging.value && toolbar.value) {
    const toolbarWidth = toolbar.value.offsetWidth;
    leftPaneWidth.value = Math.max(event.clientX - toolbarWidth, 100); // Adjust for toolbar width and minimum width
  }
}

function stopDragging() {
  isDragging.value = false;
  document.removeEventListener('mousemove', handleMouseMove);
  document.removeEventListener('mouseup', stopDragging);
}

onMounted(() => {
  document.addEventListener('mouseup', stopDragging);
})

onBeforeUnmount(() => {
  document.removeEventListener('mouseup', stopDragging);
})

</script>

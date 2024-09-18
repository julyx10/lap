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
        <IconBug 
          :class="[
            'my-5 hover:text-gray-100 transition-colors duration-300',
            isDebugMenuOpen ? 'text-gray-300' : ''
          ]" 
          @click="clickDebug" 
        />
      </div>
    </div>
      
    <!-- navigation pane -->
    <div v-if="toolbar_index > 0" class="flex relative h-screen w-96 min-w-32" :style="{ width: leftPaneWidth + 'px' }">
      <Album v-if="toolbar_index === 1" :titlebar="$t('album')"/>
      <Calendar v-else-if="toolbar_index === 2" :titlebar="$t('calendar')"/>
    </div>

    <!-- splitter -->
    <div v-if="toolbar_index > 0" class="w-1 hover:bg-sky-700 cursor-ew-resize" @mousedown="startDragging"></div>
    
    <!-- content area -->
    <div class="flex flex-1 relative h-screen bg-gray-800">
      <Content :titlebar="toolbars[toolbar_index].text"/>
    </div>

    <!-- debug area -->
    <div class="m-2 text-sm" v-if="isDebugMenuOpen">
      <button class="p-2 my-2 text-gray-200 bg-sky-800 rounded hover:bg-sky-600" @click="menuAction('locale')">Toggle Locale</button>
    </div>
    
    <!-- <ContextMenu :items="menuItems" @select="handleSelectedItem" /> -->
  </div>

</template>
   

<script setup lang="ts">

import { ref, computed, provide, onMounted, onBeforeUnmount } from 'vue';
import { WebviewWindow } from '@tauri-apps/api/window';
// import ContextMenu from '@/components/ContextMenu.vue';

/// i18n
import { useI18n } from 'vue-i18n';
const { locale, messages } = useI18n();
const localeMessages = computed(() => messages.value[locale.value]);

import Album from '@/components/Albums.vue';
import Calendar from '@/components/Calendar.vue';
import Content from '@/components/Content.vue';


/// global variables
// albums
provide('g_albums', ref([]));       // all albums
provide('g_album_id', ref(null));   // current album id
provide('g_folder_id', ref(null));  // current folder id


/// Toolbar
// Import SVG files as a Vue component
import IconHome from '@/assets/home.svg';
import IconAlbum from '@/assets/folder.svg';
import IconCalendar from '@/assets/calendar.svg';
import IconMap from '@/assets/map-pin.svg';
import IconPeople from '@/assets/user.svg';
import IconCamera from '@/assets/camera.svg';
import IconStar from '@/assets/star.svg';
import IconTag from '@/assets/tag.svg';
import IconSettings from '@/assets/cog-6-tooth.svg';
import IconBug from '@/assets/bug-ant.svg';

// toolbar 
const toolbars = computed(() =>  [
  { icon: IconHome,     text: localeMessages.value.home },
  { icon: IconAlbum,    text: localeMessages.value.album },
  { icon: IconCalendar, text: localeMessages.value.calendar },
  { icon: IconMap,      text: localeMessages.value.map },
  { icon: IconPeople,   text: localeMessages.value.people }, 
  { icon: IconCamera,   text: localeMessages.value.camera },
  { icon: IconStar,     text: localeMessages.value.star },
  { icon: IconTag,      text: localeMessages.value.tag },
]);

const toolbar_index = ref(1); // index of the clicked icon
provide('g_toolbar_index', toolbar_index);   


/// click toolbar icons
function clickToolbar(index) {
  console.log("clickToolbar...", toolbars.value[index].text);  
  toolbar_index.value = index;
}

/// click settings icon
function clickSettings() {
  console.log('clickSettings...')

  // Open the "About" window
  const aboutWindow = new WebviewWindow('about', {
    url: '/about', // This will route to the "About" page
    width: 600,
    height: 400,
  });
  aboutWindow.once('tauri://created', () => {
    console.log('New window created');
  });
}


const menuItems = ref([
  { label: 'Toggle Locale', action: 'toggle-locale' },
  { label: 'Option 2', action: 'option2' },
  { label: 'Option 3', action: 'option3' },
]);

const handleSelectedItem = (item) => {
  console.log('handleSelect:', item);
  switch (item.action) {
    case 'toggle-locale':
      if (locale.value === 'en') {
        locale.value = 'zh';
      } else {
        locale.value = 'en';
      }
  }
};

/// debug menu
const isDebugMenuOpen = ref(false);

function clickDebug() {
  console.log('clickDebug...');
  isDebugMenuOpen.value = !isDebugMenuOpen.value;
}

function menuAction(action) {
  console.log('menuAction...', action);
  switch (action) {
    case 'locale':
      if (locale.value === 'en') {
        locale.value = 'zh';
      } else {
        locale.value = 'en';
      }
      break;
    default:
      console.log('menuAction...', action);
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

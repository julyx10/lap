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
          gToolbarIndex === index ? 'text-gray-300' : ''
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
  <div v-if="gToolbarIndex > 0" class="flex relative h-screen w-96 min-w-32" :style="{ width: leftPaneWidth + 'px' }">
    <Album v-if="gToolbarIndex === 1" :titlebar="$t('album')"/>
    <Calendar v-else-if="gToolbarIndex === 2" :titlebar="$t('calendar')"/>
  </div>

  <!-- splitter -->
  <div v-if="gToolbarIndex > 0" class="w-1 hover:bg-sky-700 cursor-ew-resize" @mousedown="startDragging"></div>
  
  <!-- content area -->
  <div class="flex flex-1 relative h-screen bg-gray-800">
    <Content :titlebar="toolbars[gToolbarIndex].text"/>
  </div>

  <!-- debug area -->
  <div v-if="isDebugMenuOpen" class="flex flex-col m-2 text-sm">
    <button class="p-2 my-2 text-gray-200 bg-sky-800 rounded hover:bg-sky-600" @click="menuAction('locale')">Toggle Locale</button>
    <button class="p-2 my-2 text-gray-200 bg-sky-800 rounded hover:bg-sky-600" @click="openImage()">Open Image</button>
    <button class="p-2 my-2 text-gray-200 bg-sky-800 rounded hover:bg-sky-600" @click="clickAbout()">About</button>
  </div>
  
</div>

</template>
   

<script setup lang="ts">

import { ref, computed, inject, onMounted, onBeforeUnmount } from 'vue';
import { open } from '@tauri-apps/api/dialog'; // Tauri dialog API to open the file picker
import { WebviewWindow } from '@tauri-apps/api/window';


/// i18n
import { useI18n } from 'vue-i18n';
const { locale, messages } = useI18n();
const localeMessages = computed(() => messages.value[locale.value]);

import Album from '@/components/Albums.vue';
import Calendar from '@/components/Calendar.vue';
import Content from '@/components/Content.vue';


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

const gToolbarIndex = inject('gToolbarIndex'); // global toolbar index
const isDebugMenuOpen = ref(false); // debug menu

/// click toolbar icons
function clickToolbar(index) {
  console.log("clickToolbar...", toolbars.value[index].text);  

  gToolbarIndex.value = index;
}

/// click settings icon
function clickSettings() {
  console.log('clickSettings...');

  const settingsWindow = new WebviewWindow('settings', {
    url: '/settings',
    title: 'Settings',
    width: 600,
    height: 400,
  });
  
  settingsWindow.once('tauri://created', () => {
    console.log('New window created');
  });
}

/// open image
const openImage = async () => {
  try {
    const selectedFile = await open({
      multiple: false,
      filters: [
        {
          name: 'Image Files',
          extensions: ['png', 'jpg', 'jpeg', 'gif'],
        },
      ],
    });

    if (selectedFile) {
      console.log('Selected file:', selectedFile);
      const imageWindow = new WebviewWindow('image', {
        url: `/image-viewer?file=${encodeURIComponent(Array.isArray(selectedFile) ? selectedFile[0] : selectedFile)}`,
        title: 'Image Viewer',
        width: 800,
        height: 600,
        resizable: true,
      });

      imageWindow.once('tauri://created', () => {
        console.log('Image window created');
      });
    }
  } catch (error) {
    console.error('Failed to open file:', error);
  }
};

/// click about icon
function clickAbout() {
  console.log('clickAbout...');

  const aboutWindow = new WebviewWindow('about', {
    url: '/about',
    title: 'About',
    width: 600,
    height: 400,
    center: true,
  });

  aboutWindow.once('tauri://created', () => {
    console.log('About window created');
  });
}


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


/////////////////////////////////////////////////////////////////////////
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

<template>
  
  <div class="w-screen h-screen flex flex-col border t-color-border rounded-lg shadow-lg overflow-hidden">
    <!-- Title Bar -->
    <TitleBar titlebar="jc-photo" viewName="Home"/>

    <!-- Main Content -->
    <div class="flex-1 flex t-color-bg t-color-text overflow-hidden">

      <!-- left toolbar -->
      <div ref="toolbar" class="w-12 my-3 flex flex-col justify-between">
        <div class="flex flex-col items-center space-y-5">
          <component
            v-for="(item, index) in toolbars"
            :key="index"
            :is="item.icon"
            :class="['t-icon-size t-icon-hover', gToolbarIndex === index ? 't-icon-focus' : '']" 
            @click="clickToolbar(index)"
          />
        </div>
        <div class="flex flex-col items-center space-y-5">
          <IconSettings class="t-icon-hover" @click="clickSettings" />
          <IconBug 
            :class="['t-icon-hover', isDebugMenuOpen ? 't-icon-selected' : '']" 
            @click="clickDebug" 
          />
        </div>
      </div>
        
      <!-- left pane -->
      <div v-if="gToolbarIndex > 0" class="w-96 min-w-32 pb-1 flex" :style="{ width: leftPaneWidth + 'px' }">
        <Album         v-if="gToolbarIndex === 1" :titlebar="$t('album')"/>
        <Calendar v-else-if="gToolbarIndex === 2" :titlebar="$t('calendar')"/>
        <Map      v-else-if="gToolbarIndex === 3" :titlebar="$t('map')"/>
        <People   v-else-if="gToolbarIndex === 4" :titlebar="$t('people')"/>
        <Camera   v-else-if="gToolbarIndex === 5" :titlebar="$t('camera')"/>
      </div>

      <!-- splitter -->
      <div v-if="gToolbarIndex > 0" class="w-1 hover:bg-sky-700 cursor-ew-resize" @mousedown="startDragging"></div>
      
      <!-- content area -->
      <div class="flex-1 flex relative t-color-bg-light rounded-ss-lg">
        <Content :titlebar="toolbars[gToolbarIndex].text"/>
      </div>

      <!-- debug -->
      <div v-if="isDebugMenuOpen" class="flex flex-col m-2 space-y-2">
        <button class="p-2 t-color-bg-light rounded-lg t-icon-hover" @click="menuAction('locale')">Toggle Locale</button>
        <button class="p-2 t-color-bg-light rounded-lg t-icon-hover" @click="openImage()">Open Image</button>
        <button class="p-2 t-color-bg-light rounded-lg t-icon-hover" @click="toggleTheme()">Toggle Theme</button>
      </div>
      
    </div>
  
  </div>

</template>
   

<script setup lang="ts">

import { ref, computed, inject, onMounted, onBeforeUnmount } from 'vue';
import { open } from '@tauri-apps/plugin-dialog'; // Tauri dialog API to open the file picker
import { WebviewWindow } from '@tauri-apps/api/webviewWindow';

/// i18n
import { useI18n } from 'vue-i18n';
const { locale, messages } = useI18n();
const localeMsg = computed(() => messages.value[locale.value]);

import TitleBar from '@/components/TitleBar.vue';
import Album from '@/components/Albums.vue';
import Calendar from '@/components/Calendar.vue';
import Camera from '@/components/Camera.vue';
import Map from '@/components/Map.vue';
import People from '@/components/People.vue';
import Content from '@/components/Content.vue';


/// Toolbar
// Import SVG files as a Vue component
import IconHome from '@/assets/home.svg';
import IconAlbum from '@/assets/photo.svg';
import IconCalendar from '@/assets/calendar.svg';
import IconMap from '@/assets/map-pin.svg';
import IconPeople from '@/assets/user.svg';
import IconCamera from '@/assets/camera.svg';
// import IconFavorite from '@/assets/heart.svg';
// import IconTag from '@/assets/tag.svg';
import IconSettings from '@/assets/settings.svg';
import IconBug from '@/assets/bug.svg';

// toolbar 
const toolbars = computed(() =>  [
  { icon: IconHome,     text: localeMsg.value.home },
  { icon: IconAlbum,    text: localeMsg.value.album },
  { icon: IconCalendar, text: localeMsg.value.calendar },
  { icon: IconMap,      text: localeMsg.value.map },
  { icon: IconPeople,   text: localeMsg.value.people }, 
  { icon: IconCamera,   text: localeMsg.value.camera },
  // { icon: IconFavorite, text: localeMsg.value.favorite },
  // { icon: IconTag,      text: localeMsg.value.tag },
]);

const gToolbarIndex = inject('gToolbarIndex'); // global toolbar index
const isDebugMenuOpen = ref(false); // debug menu

/// Splitter for resizing the left pane
const toolbar = ref(null);
const leftPaneWidth = ref(300); // Default width of the left pane
const isDragging = ref(false);


onMounted(() => {
  document.addEventListener('mouseup', stopDragging);
})

onBeforeUnmount(() => {
  document.removeEventListener('mouseup', stopDragging);
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
  if (isDragging.value && toolbar.value) {
    const toolbarWidth = toolbar.value.offsetWidth + 1;   // 1: border width
    leftPaneWidth.value = Math.max(event.clientX - toolbarWidth, 100); // Adjust for toolbar width and minimum width
  }
}


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
    width: 640,
    height: 480,
    resizable: false,
    transparent: true,
    decorations: false,
  });
  
  settingsWindow.once('tauri://created', () => {
    console.log('New window created');
  });
}


/// click debug icon
function clickDebug() {
  console.log('clickDebug...');
  isDebugMenuOpen.value = !isDebugMenuOpen.value;
}


/// menu actions
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


/// debug - open image
// const openImage = async () => {
//   try {
//     const selectedFile = await open({
//       multiple: false,
//       filters: [
//         {
//           name: 'Image Files',
//           extensions: ['png', 'jpg', 'jpeg', 'gif'],
//         },
//       ],
//     });

//     if (selectedFile) {
//       console.log('Selected file:', selectedFile);
//       const imageWindow = new WebviewWindow('image', {
//         url: `/image-viewer?file=${encodeURIComponent(Array.isArray(selectedFile) ? selectedFile[0] : selectedFile)}`,
//         title: 'Image Viewer',
//         width: 800,
//         height: 600,
//         resizable: true,
//       });

//       imageWindow.once('tauri://created', () => {
//         console.log('Image window created');
//       });
//     }
//   } catch (error) {
//     console.error('Failed to open file:', error);
//   }
// };


/// debug - toggle theme
function toggleTheme() {
  console.log('toggleTheme...');
}

</script>

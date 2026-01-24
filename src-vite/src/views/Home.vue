<template>
  
  <div class="w-screen h-screen flex flex-col overflow-hidden select-none bg-base-300 text-base-content/70">
    <!-- Title Bar -->
    <TitleBar v-if="isWin" titlebar="Lap" viewName="Home"/>

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
        <div v-if="config.main.showLeftPane && !uiStore.isFullScreen"
          :class="[
            'relative flex bg-base-200 rounded-box my-1 ml-1 z-10 select-none', 
            !showPanel ? 'mt-12 mb-8': '',
            isDraggingSplitter ? 'no-transition' : 'transition-all duration-200 ease-in-out',
          ]"
          :style="{ width: showPanel ? config.main.leftPaneWidth + 'px' : '64px' }"
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
                :disabled="config.main.albumCount === 0 && index !== 0"
                :selected="config.main.sidebarIndex === index"
                @click="clickSidebar(index)"
              />
            </div>

            <div class="flex-1"></div>

            <TButton class="mt-auto"
              :buttonSize="'large'" 
              :icon="IconSettings" 
              :text="$t('sidebar.settings')" 
              @click="clickSettings"
            />
          </div>

          <!-- panel-->
          <div v-if="showPanel" class="ml-16 pr-0.5 flex-1 flex flex-col overflow-hidden">
            <!-- library title -->
            <div class="mb-2 p-2 h-10 flex items-center justify-between whitespace-nowrap shrink-0" data-tauri-drag-region>
              
              <!-- Library dropdown selector -->
              <ContextMenu
                :menuItems="libraryMenuItems"
              >
                <template #trigger="{ toggle }">
                  <button 
                    class="px-2 py-1 flex items-center gap-1 rounded-box text-base-content/70 hover:bg-base-100/30 hover:text-base-content cursor-pointer transition-colors"
                    @click="toggle"
                  >
                    <span class="overflow-hidden whitespace-pre text-ellipsis max-w-32">{{ currentLibrary?.name || 'Library' }}</span>
                    <IconArrowDown class="w-3 h-3 shrink-0 opacity-50" />
                  </button>
                </template>
              </ContextMenu>
              
              <!-- tab-specific action buttons -->
              <div class="flex items-center gap-1">
                <!-- Album: Add Album -->
                <TButton v-if="config.main.sidebarIndex === 0"
                  :icon="IconAdd"
                  :tooltip="$t('album.edit.title_add')"
                  @click="panelRef?.albumListRef?.clickNewAlbum()"
                />
                
                <!-- Calendar: Order -->
                <template v-if="config.main.sidebarIndex === 2">
                  <div role="tablist" class="tabs tabs-xs tabs-border flex-nowrap">
                    <a role="tab" :class="['tab', {'tab-active': config.calendar.isMonthly}]" @click="panelRef?.switchToMonthlyView()">{{ $t('toolbar.tooltip.monthly') }}</a>
                    <a role="tab" :class="['tab', {'tab-active': !config.calendar.isMonthly}]" @click="panelRef?.switchToDailyView()">{{ $t('toolbar.tooltip.daily') }}</a>
                  </div>
                  <TButton 
                    :icon="config.calendar.sortingAsc ? IconSortingAsc : IconSortingDesc"
                    :tooltip="config.calendar.sortingAsc ? $t('toolbar.tooltip.time_asc') : $t('toolbar.tooltip.time_desc')"
                    @click="config.calendar.sortingAsc = !config.calendar.sortingAsc"
                  />
                </template>

                <!-- Search: Clear History -->
                <TButton v-if="config.main.sidebarIndex === 3"
                  :icon="IconTrash"
                  :tooltip="$t('toolbar.tooltip.clear_history')"
                  @click="panelRef?.showClearConfirmation()"
                />
                
                <!-- Person: Index Faces -->
                <TButton v-if="config.main.sidebarIndex === 4"
                  :icon="IconRefresh"
                  :tooltip="$t('toolbar.tooltip.index_faces')"
                  @click="panelRef?.clickIndexFaces()"
                />

                <!-- Tag: Add Tag  -->
                <TButton v-if="config.main.sidebarIndex === 5"
                  :icon="IconAdd"
                  :tooltip="$t('tag.add_tag')"
                  @click="panelRef?.clickAddTag()"
                />

                <!-- person, tag, location, camera: Order -->
                <TButton v-if="config.main.sidebarIndex === 4 || config.main.sidebarIndex === 5 || config.main.sidebarIndex === 6 || config.main.sidebarIndex === 7"
                  :icon="config.leftPanel.sortCount ? IconSortingName : IconSortingCount"
                  :tooltip="$t('toolbar.tooltip.sort')"
                  @click="config.leftPanel.sortCount = !config.leftPanel.sortCount"
                />
              </div>
            </div>

            <!-- Component panel (flex-1 to fill remaining space) -->
            <div class="flex-1 overflow-hidden">
              <component ref="panelRef" :is="buttons[config.main.sidebarIndex].component" :titlebar="buttons[config.main.sidebarIndex].text"/>
            </div>
          </div>

        </div>
      </transition> 
      
      <!-- splitter -->
      <div v-if="!uiStore.isFullScreen"
        class="w-1 transition-colors shrink-0"
        :class="{
          'hover:bg-primary cursor-col-resize': config.main.showLeftPane && showPanel,
          'bg-primary': config.main.showLeftPane && showPanel && isDraggingSplitter,
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
        <Content :titlebar="buttons[config.main.sidebarIndex].text"/>
      </div>
    </div>

    <!-- logo -->
    <div class="fixed bottom-2 left-6 text-[12px] text-base-content/10">
      <span>{{ appName }}</span>
    </div>

    <!-- Manage Libraries Dialog -->
    <ManageLibraries
      v-if="showManageLibraries"
      @ok="onManageLibrariesOk"
      @cancel="showManageLibraries = false"
    />
  </div>

</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { useI18n } from 'vue-i18n';
import { WebviewWindow } from '@tauri-apps/api/webviewWindow';
import { getName } from '@tauri-apps/api/app';
import { config, libConfig } from '@/common/config';
import { useUIStore } from '@/stores/uiStore';
import { isWin, isMac } from '@/common/utils';
import { getAppConfig, switchLibrary, cancelIndexing, cancelFaceIndex } from '@/common/api';

// vue components
import Album from '@/components/Album.vue';
import ImageSearch from '@/components/ImageSearch.vue';
import Favorite from '@/components/Favorite.vue';
import Tag from '@/components/Tag.vue';
import Calendar from '@/components/Calendar.vue';
import Location from '@/components/Location.vue';
import Person from '@/components/Person.vue';
import Camera from '@/components/Camera.vue';
import TitleBar from '@/components/TitleBar.vue';
import TButton from '@/components/TButton.vue';
import Content from '@/components/Content.vue';
import ContextMenu from '@/components/ContextMenu.vue';
import ManageLibraries from '@/components/ManageLibraries.vue';

import {
  IconFavorite,
  IconTag,
  IconLocation,
  IconPerson,
  IconCameraAperture,
  IconSearch,
  IconSettings,
  IconAlbums,
  IconDot,
  IconAdd,
  IconTrash,
  IconSortingAsc,
  IconSortingDesc,
  IconSortingCount,
  IconSortingName,
  IconArrowDown,
  IconCalendarDay,
  IconRefresh,
} from '@/common/icons';

/// i18n
const { locale, messages } = useI18n();
const localeMsg = computed(() => messages.value[locale.value] as any);

const uiStore = useUIStore();

// Panel component ref
const panelRef = ref<any>(null);
const showPanel = ref(true);

// Library state
interface Library {
  id: string;
  name: string;
  created_at: number;
  hidden: boolean;
}

interface AppConfig {
  current_library_id: string;
  libraries: Library[];
}

const appConfig = ref<AppConfig | null>(null);
const currentLibrary = computed(() => 
  appConfig.value?.libraries.find(l => l.id === appConfig.value?.current_library_id) || null
);

// Manage Libraries dialog state
const showManageLibraries = ref(false);

/// Splitter for resizing the left pane
const isDraggingSplitter = ref(false);

const appName = ref('');

// buttons 
const buttons = computed(() =>  [
  { 
    icon: IconAlbums,  
    component: Album,
    text: localeMsg.value.sidebar.album 
  },
  { 
    icon: IconFavorite, 
    component: Favorite,
    text: localeMsg.value.sidebar.favorite 
  },
  { 
    icon: IconCalendarDay, 
    component: Calendar,
    text: localeMsg.value.sidebar.calendar 
  },
  { 
    icon: IconSearch,
    component: ImageSearch,
    text: localeMsg.value.sidebar.search
  },
  { 
    icon: IconPerson, 
    component: Person, 
    text: localeMsg.value.sidebar.person 
  },
  { 
    icon: IconTag,
    component: Tag,
    text: localeMsg.value.sidebar.tag 
  },
  { 
    icon: IconLocation, 
    component: Location, 
    text: localeMsg.value.sidebar.location 
  },
  { 
    icon: IconCameraAperture,  
    component: Camera,
    text: localeMsg.value.sidebar.camera 
  },
]);

const libraryMenuItems = computed(() => {
  const items: any[] = [];
  
  // Add all libraries for switching
  if (appConfig.value?.libraries) {
    for (const lib of appConfig.value.libraries) {
      if (lib.hidden) continue; // Skip hidden libraries
      const isSelected = lib.id === appConfig.value.current_library_id;
      items.push({
        label: lib.name,
        icon: isSelected ? IconDot : null,
        action: () => {
          if (!isSelected) {
            doSwitchLibrary(lib.id);
          }
        }
      });
    }
  }
  items.push({
    label: "-",
    action: () => {}
  });
  items.push({
    label: localeMsg.value.menu.library.manage,
    // icon: IconEdit,
    action: () => {
      showManageLibraries.value = true;
    }
  });
  return items;
});


onMounted(async () => {
  appConfig.value = await getAppConfig();
  
  try {
    const name = await getName();
    if (name) appName.value = name;
  } catch (e) {
    console.error('Failed to get app name:', e);
  }
});

const doSwitchLibrary = async (libraryId: string) => {
  try {
    // Cancel any running indexing before switching
    if (libConfig.index.status > 0 && libConfig.index.albumQueue.length > 0) {
      for (const albumId of libConfig.index.albumQueue) {
        await cancelIndexing(albumId);
      }
    }
    
    // Cancel face indexing if running
    await cancelFaceIndex();
    
    // Save current library state before switching
    await libConfig.save();
    
    await switchLibrary(libraryId);
    
    // Fade out before reload to prevent flash
    document.body.style.transition = 'opacity 0.15s ease-out';
    document.body.style.opacity = '0';
    await new Promise(resolve => setTimeout(resolve, 150));
    
    // Reload the app to switch database
    window.location.reload();
  } catch (error) {
    console.error('Failed to switch library:', error);
    // Restore opacity if error occurs
    document.body.style.opacity = '1';
  }
};

const onManageLibrariesOk = async () => {
  appConfig.value = await getAppConfig();
};

// click sidebar
function clickSidebar(index: number) {
  if (config.main.sidebarIndex === index) {
    showPanel.value = !showPanel.value;
  } else {
    showPanel.value = true;
    config.main.sidebarIndex = index;
  }
}

// Dragging the splitter
function startDraggingSplitter(event: MouseEvent) {
  if(!config.main.showLeftPane) return; // no left pane or left pane is hidden

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
    config.main.leftPaneWidth = Math.max(160, Math.min(event.clientX - 6, maxLeftPaneWidth)); // -2: border width(2px)
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

  const options: any = {
    url: '/settings',
    title: 'Settings',
    width: 600,
    height: 400,
    minWidth: 600,
    minHeight: 400,
    resizable: true,
    visible: false, // Start hidden, will show after mount
    transparent: true, // Prevent white flash on show (Tauri 2.x workaround)
    decorations: isMac, // true for macOS, false for Windows
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

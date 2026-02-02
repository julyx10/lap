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
                    <IconStack class="w-5 h-5 shrink-0" />
                    <span class="overflow-hidden whitespace-pre text-ellipsis max-w-32">{{ currentLibrary?.name || 'Library' }}</span>
                    <IconArrowDown class="w-3 h-3 shrink-0 opacity-50" />
                  </button>
                </template>
              </ContextMenu>
              
              <!-- More Menu for all tabs -->
              <div class="ml-1">
                <!-- IconUpdate for Album Reorder Mode -->
                <TButton v-if="isAlbumReorderMode"
                  :icon="IconRestore"
                  :buttonSize="'medium'"
                  :selected="true"
                  @click="clickRestoreAlbumOrder"
                />
                <!-- Default Context Menu -->
                <ContextMenu v-else-if="moreMenuItems.length > 0" :menuItems="moreMenuItems">
                  <template #trigger="{ toggle }">
                    <TButton 
                      :icon="IconMore"
                      :buttonSize="'medium'"
                      @click="toggle"
                    />
                  </template>
                </ContextMenu>
              </div>  
            </div>

            <!-- Component panel (flex-1 to fill remaining space) -->
            <div class="flex-1 overflow-hidden">
              <component ref="panelRef" 
                :is="buttons[config.main.sidebarIndex].component" 
                :titlebar="buttons[config.main.sidebarIndex].text"
                @editDataChanged="onEditDataChanged"
              />
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
import Library from '@/components/Library.vue';
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
  IconDot,
  IconAdd,
  IconTrash,
  IconStack,
  IconArrowDown,
  IconCalendarDay,
  IconUpdate,
  IconMore,
  IconOrder,
  IconRestore,
  IconPhotoAll,
} from '@/common/icons';

const isAlbumReorderMode = ref(false);

const moreMenuItems = computed(() => {
  const index = config.main.sidebarIndex;
  
  // 0: Album
  if (index === 0) {
    return [
      {
        label: localeMsg.value.menu.album.add,
        icon: IconAdd,
        action: () => panelRef.value?.albumListRef?.clickNewAlbum()
      },
      { label: '-' },
      {
        label: localeMsg.value.menu.album.reorder || 'Reorder',
        icon: IconOrder,
        action: () => panelRef.value?.albumListRef?.clickReorder()
      }
    ];
  }
  
  // 1: Favorites - No menu items
  if (index === 1) {
    return [];
  }
  
  // 2: Calendar - Sort options
  if (index === 2) {
    // Calendar view toggle is handled by tabs in the template usually, 
    // but request specified menu item: "Time Asc, Time Desc (one is selected)"
    // The user request said: "Time Asc, Time Desc (one is selected per the status of config)"
    // This implies toggle logic or radio feel.
    return [
       {
        label: localeMsg.value.menu.calendar.monthly,
        icon: config.calendar.isMonthly ? IconDot : null,
        action: () => panelRef.value?.switchToMonthlyView()
      },
      {
        label: localeMsg.value.menu.calendar.daily,
        icon: config.calendar.isMonthly ? null : IconDot,
        action: () => panelRef.value?.switchToDailyView()
      },
      { label: '-' },
      {
        label: localeMsg.value.menu.calendar.time_asc,
        icon: config.calendar.sortingAsc ? IconDot : null,
        action: () => { config.calendar.sortingAsc = !config.calendar.sortingAsc; }
      },
      {
        label: localeMsg.value.menu.calendar.time_desc,
        icon: config.calendar.sortingAsc ? null : IconDot,
        action: () => { config.calendar.sortingAsc = !config.calendar.sortingAsc; }
      }
    ];
  }
  
  // 3: Search - Clear History
  if (index === 3) {
    return [
      {
        label: localeMsg.value.menu.search.clear_history,
        icon: IconTrash,
        action: () => panelRef.value?.showClearConfirmation()
      }
    ];
  }
  
  // 4: Person
  if (index === 4) {
    return [
      {
        label: localeMsg.value.menu.person.index_faces,
        icon: IconUpdate,
        action: () => panelRef.value?.clickIndexFaces()
      },
      {
        label: localeMsg.value.menu.person.reset_index,
        icon: IconTrash,
        action: () => panelRef.value?.clickResetFaces()
      },
      { label: '-' },
      {
        label: localeMsg.value.menu.sort.sort_by_name,
        icon: config.leftPanel.sortCount ? null : IconDot,
        action: () => config.leftPanel.sortCount = false
      },
      {
        label: localeMsg.value.menu.sort.sort_by_count,
        icon: config.leftPanel.sortCount ? IconDot : null,
        action: () => config.leftPanel.sortCount = true
      }
    ];
  }
  
  // 5: Tags
  if (index === 5) {
     return [
      {
        label: localeMsg.value.menu.tag.add,
        icon: IconAdd,
        action: () => panelRef.value?.clickAddTag()
      },
      { label: '-' },
      {
        label: localeMsg.value.menu.sort.sort_by_name,
        icon: config.leftPanel.sortCount ? null : IconDot,
        action: () => config.leftPanel.sortCount = false
      },
      {
        label: localeMsg.value.menu.sort.sort_by_count,
        icon: config.leftPanel.sortCount ? IconDot : null,
        action: () => config.leftPanel.sortCount = true
      }
    ];
  }
  
  // 6: Location & 7: Camera
  if (index === 6 || index === 7) {
    return [
      {
        label: localeMsg.value.menu.sort.sort_by_name,
        icon: config.leftPanel.sortCount ? null : IconDot,
        action: () => config.leftPanel.sortCount = false
      },
      {
        label: localeMsg.value.menu.sort.sort_by_count,
        icon: config.leftPanel.sortCount ? IconDot : null,
        action: () => config.leftPanel.sortCount = true
      }
    ];
  }

  return [];
});

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
    icon: IconPhotoAll,  
    component: Library,
    text: localeMsg.value.sidebar.library
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
    
    window.location.reload();
  } catch (error) {
    console.error('Failed to switch library:', error);
  }
};

const onManageLibrariesOk = async () => {
  const oldLibId = appConfig.value?.current_library_id;
  appConfig.value = await getAppConfig();

  if (oldLibId && appConfig.value?.current_library_id !== oldLibId) {
    window.location.reload();
  }
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

const onEditDataChanged = (isEdit: boolean) => {
  if (config.main.sidebarIndex === 0) { // Album tab
    isAlbumReorderMode.value = isEdit;
  }
};

const clickRestoreAlbumOrder = () => {
  panelRef.value?.albumListRef?.clickCloseEditList();
};

</script>

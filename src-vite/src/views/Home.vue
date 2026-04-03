<template>
  
  <div class="w-screen h-screen flex flex-col overflow-hidden select-none bg-base-300 text-base-content/70">
    <transition name="fade">
      <div
        v-if="isSwitchingLibrary"
        class="absolute inset-0 z-60 bg-base-300/60 backdrop-blur-sm flex items-center justify-center"
      >
        <span class="loading loading-spinner loading-lg text-primary"></span>
      </div>
    </transition>

    <transition name="fade">
      <div
        v-if="libraryMoveState.active"
        class="absolute inset-0 z-[550] bg-base-300/50 backdrop-blur-sm flex items-center justify-center pointer-events-auto"
      >
        <div class="rounded-box border border-base-content/10 bg-base-200/80 px-4 py-3 min-w-72 shadow-lg">
          <div class="flex items-center gap-3">
            <span class="loading loading-spinner loading-md text-primary"></span>
            <div class="min-w-0">
              <div class="text-sm text-base-content/80 truncate">
                {{ libraryMoveState.message }}
              </div>
              <div class="mt-1 text-xs text-base-content/50">
                {{ libraryMoveState.percent }}%
              </div>
            </div>
          </div>
        </div>
      </div>
    </transition>

    <!-- Title Bar -->
    <TitleBar v-if="isWin" titlebar="Lap" viewName="Home" :icon="iconLogo"/>

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
        <div v-if="config.leftPanel.show && !uiStore.isFullScreen"
          :class="[
            'relative flex bg-base-200 rounded-box my-1 ml-1 z-10 select-none', 
            !showPanel && isMac ? 'mt-12 mb-8': '',
            isDraggingSplitter ? 'no-transition' : 'transition-all duration-200 ease-in-out',
          ]"
          :style="{ width: showPanel ? config.leftPanel.width + 'px' : '64px' }"
          data-tauri-drag-region
        >
          <!-- side bar -->
          <div 
            :class="[
              'fixed top-14 min-w-16 bottom-10 flex flex-col items-center',
              config.settings.showButtonText ? 'space-y-3' : 'space-y-1' 
            ]" 
            data-tauri-drag-region
          >
            <div v-for="item in visibleButtons" :key="item.index">
              <TButton 
                :buttonSize="'large'" 
                :icon="item.icon" 
                :text="item.text" 
                :tooltip="(item as any).tooltip || ''"
                :selected="config.main.sidebarIndex === item.index"
                @click="clickSidebar(item.index)"
              />
            </div>

            <div class="flex-1"></div>

            <TButton 
              class="mt-auto"
              :class="showDebugBadge ? 'text-warning': ''"
              :buttonSize="'large'" 
              :icon="IconSettings" 
              :text="$t('sidebar.settings')" 
              @click="clickSettings"
            />
          </div>

          <!-- panel-->
          <div v-if="showPanel" class="ml-16 pr-0.5 flex-1 flex flex-col overflow-hidden">
            <!-- library title -->
            <div 
              class="mb-2 h-10 flex items-center justify-between whitespace-nowrap shrink-0"
              :class="config.settings.scale < 1 ? 'p-3' : 'p-1'"
              data-tauri-drag-region
            >
              
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

              <button
                v-if="updateAvailable || isInstallingUpdate || isUpdateReadyToRestart"
                class="badge badge-sm border-0 px-2 py-2 font-medium transition-colors"
                :class="isUpdateActionEnabled ? 'badge-primary cursor-pointer' : 'badge-neutral/60 cursor-default'"
                :disabled="isInstallingUpdate"
                :title="updateButtonTooltip"
                @click="handleUpdateAction"
              >
                <span v-if="isInstallingUpdate" class="loading loading-spinner loading-xs"></span>
                <span>{{ updateButtonText }}</span>
              </button>

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
          'hover:bg-primary cursor-col-resize': config.leftPanel.show && showPanel,
          'bg-primary': config.leftPanel.show && showPanel && isDraggingSplitter,
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
        <Content ref="contentRef" :titlebar="buttons[config.main.sidebarIndex].text"/>
      </div>
    </div>

    <!-- logo -->
    <div class="fixed bottom-2 left-6 text-[12px] text-base-content/10">
      <span>{{ appName }}</span>
    </div>

    <!-- Manage Libraries Dialog -->
    <ManageLibraries
      v-if="showManageLibraries"
      :move-storage="handleMoveLibraryStorage"
      :cancel-move-storage="handleCancelLibraryMove"
      :move-storage-state="libraryMoveState"
      @ok="onManageLibrariesOk"
      @cancel="showManageLibraries = false"
    />

    <ToolTip ref="toolTipRef" />
  </div>

</template>

<script setup lang="ts">
import { ref, computed, onBeforeUnmount, onMounted, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import { emit, listen } from '@tauri-apps/api/event';
import { WebviewWindow } from '@tauri-apps/api/webviewWindow';
import { getName } from '@tauri-apps/api/app';
import { config, libConfig } from '@/common/config';
import {
  LIBRARY_MOVE_CANCELLED_ERROR,
  pickLibraryMoveError,
  shouldResumeFace,
  sleep,
  waitForFaceIndexIdle,
} from '@/common/libraryMove';
import { useAppUpdater } from '@/common/updater';
import { useUIStore } from '@/stores/uiStore';
import { isWin, isMac, SCALE_VALUES } from '@/common/utils';
import {
  getAppConfig,
  switchLibrary,
  moveLibraryStorage,
  cancelLibraryStorageMove,
  cancelIndexing,
  cancelFaceIndex,
  isFaceIndexing,
  indexFaces,
  listenLibraryStorageMoveProgress,
} from '@/common/api';

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
import ToolTip from '@/components/ToolTip.vue';
import iconLogo from '@/assets/images/logo.png';

import {
  IconHeart,
  IconTag,
  IconLocation,
  IconPerson,
  IconCameraAperture,
  IconSearch,
  IconSettings,
  IconDot,
  IconStack,
  IconArrowDown,
  IconCalendarDay,
  IconPhotoAll,
} from '@/common/icons';

const isAlbumReorderMode = ref(false);
const isSwitchingLibrary = ref(false);
const SETTINGS_BASE_WIDTH = 600;
const SETTINGS_BASE_HEIGHT = 450;

/// i18n
const { locale, messages } = useI18n();
const localeMsg = computed(() => messages.value[locale.value] as any);

const uiStore = useUIStore();

// Panel component ref
const panelRef = ref<any>(null);
const contentRef = ref<any>(null);
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

interface LibraryMoveState {
  active: boolean;
  libraryId: string;
  phase: string;
  percent: number;
  bytesCopied: number;
  totalBytes: number;
  message: string;
  cancellable: boolean;
  cancelRequested: boolean;
  status: string;
}

interface FaceMoveState {
  wasRunning: boolean;
  stoppedCleanly: boolean;
}

const appConfig = ref<AppConfig | null>(null);

function getCurrentFaceClusterEpsilon() {
  const face = config.settings.face;
  const thresholdIndex = face?.clusterThresholdIndex ?? 2;
  const thresholds = config.faceClusterThresholds ?? [0.35, 0.45, 0.55, 0.65];
  return thresholds[thresholdIndex] ?? 0.55;
}

function resetLibraryMoveState() {
  libraryMoveState.value = {
    active: false,
    libraryId: '',
    phase: '',
    percent: 0,
    bytesCopied: 0,
    totalBytes: 0,
    message: '',
    cancellable: false,
    cancelRequested: false,
    status: 'idle',
  };
}

function assertLibraryMoveNotCancelled() {
  if (libraryMoveState.value.cancelRequested) {
    throw new Error(LIBRARY_MOVE_CANCELLED_ERROR);
  }
}
const currentLibrary = computed(() => 
  appConfig.value?.libraries.find(l => l.id === appConfig.value?.current_library_id) || null
);

// Manage Libraries dialog state
const showManageLibraries = ref(false);
const libraryMoveState = ref<LibraryMoveState>({
  active: false,
  libraryId: '',
  phase: '',
  percent: 0,
  bytesCopied: 0,
  totalBytes: 0,
  message: '',
  cancellable: false,
  cancelRequested: false,
  status: 'idle',
});

/// Splitter for resizing the left pane
const isDraggingSplitter = ref(false);

const appName = ref('');
const showDebugBadge = import.meta.env.DEV;
const toolTipRef = ref<InstanceType<typeof ToolTip> | null>(null);
let unlistenOpenPreferences: (() => void) | null = null;
let unlistenOpenAbout: (() => void) | null = null;
let unlistenStorageMoveProgress: (() => void) | null = null;
const {
  updateAvailable,
  isCheckingUpdate,
  isInstallingUpdate,
  isUpdateReadyToRestart,
  updateButtonTooltip,
  updateButtonText,
  isUpdateActionEnabled,
  checkForUpdates,
  handleUpdateAction,
} = useAppUpdater(localeMsg, toolTipRef);

// buttons
const buttons = computed(() =>  [
  { icon: IconPhotoAll, component: Library, text: localeMsg.value.sidebar.album },
  { icon: IconHeart, component: Favorite, text: localeMsg.value.sidebar.favorite },
  { icon: IconSearch, component: ImageSearch, text: localeMsg.value.sidebar.search },
  { icon: IconCalendarDay, component: Calendar, text: localeMsg.value.sidebar.calendar },
  { icon: IconTag, component: Tag, text: localeMsg.value.sidebar.tag },
  { icon: IconPerson, component: Person, text: localeMsg.value.sidebar.people, hidden: !config.settings.face.enabled },
  { icon: IconLocation, component: Location, text: localeMsg.value.sidebar.location },
  { icon: IconCameraAperture, component: Camera, text: localeMsg.value.sidebar.camera },
]);

const visibleButtons = computed(() =>
  buttons.value
    .map((item, index) => ({ ...item, index }))
    .filter(item => !item.hidden)
);

watch(() => config.settings.face.enabled, (enabled) => {
  if (!enabled && config.main.sidebarIndex === 5) {
    config.main.sidebarIndex = 0;
  }
});

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
  unlistenOpenPreferences = await listen('app-open-preferences', () => {
    void clickSettings();
  });
  unlistenOpenAbout = await listen('app-open-about', () => {
    void clickSettings(5);
  });
  unlistenStorageMoveProgress = await listenLibraryStorageMoveProgress((event: any) => {
    const payload = event?.payload || {};
    if (!libraryMoveState.value.active) return;
    if (String(payload.libraryId || '') !== libraryMoveState.value.libraryId) return;

    libraryMoveState.value = {
      ...libraryMoveState.value,
      phase: String(payload.phase || libraryMoveState.value.phase || ''),
      percent: Number(payload.percent || 0),
      bytesCopied: Number(payload.bytesCopied || 0),
      totalBytes: Number(payload.totalBytes || 0),
      message: String(payload.message || libraryMoveState.value.message || ''),
      cancellable: Boolean(payload.cancellable),
      cancelRequested: Boolean(payload.cancelRequested),
      status: String(payload.status || libraryMoveState.value.status || 'running'),
    };
  });

  appConfig.value = await getAppConfig();
  
  try {
    const name = await getName();
    if (name) appName.value = name;
  } catch (e) {
    console.error('Failed to get app name:', e);
  }

  void checkForUpdates(false);
});

onBeforeUnmount(() => {
  unlistenOpenPreferences?.();
  unlistenOpenPreferences = null;
  unlistenOpenAbout?.();
  unlistenOpenAbout = null;
  unlistenStorageMoveProgress?.();
  unlistenStorageMoveProgress = null;
});

const doSwitchLibrary = async (libraryId: string) => {
  try {
    isSwitchingLibrary.value = true;

    // Save current library state before switching (preserves the indexing queue)
    await libConfig.save();

    // Prevent auto-save during shutdown of the current library's background work.
    libConfig._initialized = false;

    // Cancel any running indexing before switching
    if (libConfig.index.status > 0 && libConfig.index.albumQueue.length > 0) {
      const queueCopy = [...libConfig.index.albumQueue];
      for (const albumId of queueCopy) {
        await cancelIndexing(albumId);
      }
    }
    
    // Cancel face indexing if running
    await cancelFaceIndex();
    
    await switchLibrary(libraryId);
    
    window.location.reload();
  } catch (error) {
    libConfig._initialized = true;
    isSwitchingLibrary.value = false;
    console.error('Failed to switch library:', error);
  }
};

const handleMoveLibraryStorage = async (libraryId: string, storageDir: string | null) => {
  const shouldQuiesceWrites = Boolean(appConfig.value?.current_library_id);
  const previousInitialized = Boolean(libConfig._initialized);
  let moveSnapshot: any = null;
  let moveInfo: any = null;
  let primaryError: any = null;
  let cleanupError: any = null;
  const faceMoveState: FaceMoveState = {
    wasRunning: false,
    stoppedCleanly: false,
  };

  try {
    libraryMoveState.value = {
      active: true,
      libraryId,
      phase: 'quiescing',
      percent: 0,
      bytesCopied: 0,
      totalBytes: 0,
      message: storageDir
        ? localeMsg.value.msgbox.manage_libraries.moving_storage
        : localeMsg.value.msgbox.manage_libraries.moving_to_default_storage,
      cancellable: true,
      cancelRequested: false,
      status: 'running',
    };

    if (shouldQuiesceWrites) {
      await libConfig.save();
      libConfig._initialized = false;

      if (contentRef.value?.prepareForLibraryStorageMove) {
        moveSnapshot = await contentRef.value.prepareForLibraryStorageMove();
      }
      assertLibraryMoveNotCancelled();

      const faceState = await isFaceIndexing({ strict: true });
      faceMoveState.wasRunning = Array.isArray(faceState) ? Boolean(faceState[0]) : Boolean(faceState);
      if (faceMoveState.wasRunning) {
        await cancelFaceIndex();
        await waitForFaceIndexIdle(isFaceIndexing);
        faceMoveState.stoppedCleanly = true;
      }
      assertLibraryMoveNotCancelled();
    }

    assertLibraryMoveNotCancelled();
    libraryMoveState.value = {
      ...libraryMoveState.value,
      phase: 'dispatching',
      cancellable: false,
    };
    moveInfo = await moveLibraryStorage(libraryId, storageDir);
    appConfig.value = await getAppConfig();
  } catch (error: any) {
    if (!moveSnapshot && error?.libraryMoveSnapshot) {
      moveSnapshot = error.libraryMoveSnapshot;
    }
    primaryError = error;
  } finally {
    if (shouldQuiesceWrites) {
      try {
        if (contentRef.value?.resumeAfterLibraryStorageMove) {
          await contentRef.value.resumeAfterLibraryStorageMove(moveSnapshot);
        }
        if (shouldResumeFace(faceMoveState)) {
          await indexFaces(getCurrentFaceClusterEpsilon());
        }
      } catch (error: any) {
        cleanupError = cleanupError || error;
        if (primaryError) {
          console.error('Failed to resume background work after library storage move error:', error);
        }
      } finally {
        libConfig._initialized = previousInitialized;
        if (libConfig._initialized) {
          try {
            await libConfig.save();
          } catch (error: any) {
            cleanupError = cleanupError || error;
            if (primaryError) {
              console.error('Failed to persist library config after library storage move error:', error);
            }
          }
        }
      }
    }

    resetLibraryMoveState();
  }

  const finalError = pickLibraryMoveError(primaryError, cleanupError);
  if (finalError) {
    throw finalError;
  }

  return moveInfo;
};

const handleCancelLibraryMove = async () => {
  if (!libraryMoveState.value.active || !libraryMoveState.value.libraryId) return;
  if (libraryMoveState.value.cancelRequested || !libraryMoveState.value.cancellable) return;

  libraryMoveState.value = {
    ...libraryMoveState.value,
    cancelRequested: true,
  };

  if (libraryMoveState.value.phase === 'quiescing') {
    return;
  }

  try {
    await cancelLibraryStorageMove(libraryMoveState.value.libraryId);
  } catch (error: any) {
    const message = error?.message || error?.toString?.() || String(error || '');
    if (message.includes('No active storage move')) {
      await sleep(100);
      if (libraryMoveState.value.active && libraryMoveState.value.phase !== 'quiescing') {
        await cancelLibraryStorageMove(libraryMoveState.value.libraryId);
      }
      return;
    }
    throw error;
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
  if(!config.leftPanel.show) return; // no left pane or left pane is hidden

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
    const pointerX = event.clientX;
    const maxLeftPaneWidth = window.innerWidth / 2;
    config.leftPanel.width = Math.max(160, Math.min(pointerX - 6, maxLeftPaneWidth)); // -2: border width(2px)
  }
}

/// click settings icon
async function clickSettings(tabIndex?: number) {
  if (typeof tabIndex === 'number') {
    config.settings.tabIndex = tabIndex;
    await emit('settings-settingsTabIndex-changed', tabIndex);
  }

  // check if the settings window is already open
  const settingsWindow = await WebviewWindow.getByLabel('settings');
  if (settingsWindow) {
    settingsWindow.show();
    return;
  }

  const options: any = {
    url: '/settings',
    title: 'Settings',
    width: Math.round(SETTINGS_BASE_WIDTH * getSettingsWindowScale()),
    height: Math.round(SETTINGS_BASE_HEIGHT * getSettingsWindowScale()),
    minWidth: Math.round(SETTINGS_BASE_WIDTH * getSettingsWindowScale()),
    minHeight: Math.round(SETTINGS_BASE_HEIGHT * getSettingsWindowScale()),
    resizable: true,
    maximizable: false,
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

function getSettingsWindowScale() {
  return SCALE_VALUES.find((item) => item === Number(config.settings.scale)) ?? 1;
}

const onEditDataChanged = (isEdit: boolean) => {
  if (config.main.sidebarIndex === 0) { // Album tab
    isAlbumReorderMode.value = isEdit;
  }
};

</script>

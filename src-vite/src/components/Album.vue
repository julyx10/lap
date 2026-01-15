<template>
    
  <div class="w-full h-full flex flex-col select-none">

    <!-- title bar -->
    <div class="p-1 h-12 flex items-center justify-end whitespace-nowrap" data-tauri-drag-region>
      <ContextMenu
        :iconMenu="IconMore" 
        :menuItems="libraryMenuItems"
        :disabled="albumListRef?.isEditList"
        :smallIcon="true"
      />
    </div>
    
    <!-- library -->
    <div 
      :class="[ 
        'mx-1 p-1 h-10 flex items-center rounded-box whitespace-nowrap group',
        albumListRef?.isEditList 
          ? 'text-base-content/30' 
          : (config.album.id === 0 ? 'text-primary bg-base-100 hover:bg-base-100 cursor-pointer' : 'hover:text-base-content hover:bg-base-100/30 cursor-pointer'),
      ]"
      @click="clickLibrary"
    >
      <IconPhotoAll class="mx-1 w-5 h-5 shrink-0" />
      <div class="overflow-hidden whitespace-pre text-ellipsis font-bold">
        <span>{{ currentLibrary?.name }}</span>
      </div>

      <!-- Right side: Count and Context Menu -->
      <div class="ml-auto flex items-center">
        <span v-if="totalCount > 0" class="px-1 text-xs text-base-content/30 mr-1">
          {{ totalCount.toLocaleString() }}
        </span>

        <!-- Library Context Menu -->
        <div v-if="!albumListRef?.isEditList" 
          class="text-base-content/30"
          :class="[
            config.album.id === 0 ? '' : 'hidden group-hover:block'
          ]"
          @click.stop
        >
          <ContextMenu 
            :iconMenu="IconMore" 
            :menuItems="albumsMenuItems"
            :smallIcon="true"
          />
        </div>
      </div>
    </div>

    <AlbumList ref="albumListRef" 
      :key="albumListKey"
      selectionSource="album"
    />

    <!-- Library Edit Dialog -->
    <LibraryEdit
      v-if="showLibraryEdit"
      :isNewLibrary="isNewLibrary"
      :libraryId="editingLibrary?.id || ''"
      :libraryName="editingLibrary?.name || ''"
      :createdAt="editingLibrary?.created_at || 0"
      @ok="onLibraryEditOk"
      @cancel="showLibraryEdit = false"
    />

    <!-- Remove Library Confirmation -->
    <MessageBox
      v-if="showRemoveLibraryMsgbox"
      :title="$t('msgbox.remove_library.title')"
      :message="$t('msgbox.remove_library.content', { library: currentLibrary?.name })"
      :OkText="$t('msgbox.remove_library.ok')"
      :cancelText="$t('msgbox.cancel')"
      :warningOk="true"
      @ok="confirmRemoveLibrary"
      @cancel="showRemoveLibraryMsgbox = false"
    />
  </div> 

</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { useI18n } from 'vue-i18n';
import { config } from '@/common/config';
import { useUIStore } from '@/stores/uiStore';

import { IconMore, IconAdd, IconOrder, IconRefresh, IconPhotoAll, IconEdit, IconLibraryAdd, IconLibraryRemove, IconDot } from '@/common/icons';
import { getTotalCountAndSum, getAppConfig, switchLibrary, removeLibrary, cancelIndexing } from '@/common/api';
import AlbumList from '@/components/AlbumList.vue';
import ContextMenu from '@/components/ContextMenu.vue';
import LibraryEdit from '@/components/LibraryEdit.vue';
import MessageBox from '@/components/MessageBox.vue';

interface Library {
  id: string;
  name: string;
  created_at: number;
}

interface AppConfig {
  current_library_id: string;
  libraries: Library[];
}

const props = defineProps({
  titlebar: {
    type: String,
    required: true
  }
});

/// i18n
const { locale, messages } = useI18n();
const localeMsg = computed(() => messages.value[locale.value] as any);
const uiStore = useUIStore();

const totalCount = ref(0);
const appConfig = ref<AppConfig | null>(null);
const currentLibrary = computed(() => 
  appConfig.value?.libraries.find(l => l.id === appConfig.value?.current_library_id) || null
);

// Library edit dialog state
const showLibraryEdit = ref(false);
const isNewLibrary = ref(false);
const editingLibrary = ref<Library | null>(null);

// Remove library confirmation
const showRemoveLibraryMsgbox = ref(false);

const albumListRef = ref<InstanceType<typeof AlbumList> | null>(null);

// refresh component
const albumListKey = ref(0);

onMounted(async () => {
  // Load app config
  appConfig.value = await getAppConfig();
  
  // get total count
  getTotalCountAndSum().then((result) => {
    if(result) {
      totalCount.value = result[0];
    }
  });
});

onUnmounted(() => {
  uiStore.removeInputHandler('AlbumList-edit');
});

const libraryMenuItems = computed(() => {
  const items: any[] = [];
  
  // Add all libraries
  if (appConfig.value?.libraries) {
    for (const lib of appConfig.value.libraries) {
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
    label: localeMsg.value.menu.library.add,
    icon: IconLibraryAdd,
    action: () => {
      isNewLibrary.value = true;
      editingLibrary.value = null;
      showLibraryEdit.value = true;
    }
  });
  
  return items;
});

const albumsMenuItems = computed(() => {
  return [
    {
      label: localeMsg.value.menu.album.add,
      icon: IconAdd,
      action: () => {
        albumListRef.value?.clickNewAlbum();
      }
    },
    {
      label: localeMsg.value.menu.album.refresh,
      icon: IconRefresh,
      action: async () => {
        albumListRef.value?.refreshAlbums(); 
      }
    },
    {
      label: localeMsg.value.menu.album.reorder,
      icon: IconOrder,
      action: () => {
        if (albumListRef.value) {
          albumListRef.value.isEditList = true;
        }
        uiStore.pushInputHandler('AlbumList-edit');
      }
    },
    {
      label: "-",   // separator
      action: () => {}
    },
    {
      label: localeMsg.value.menu.library.edit,
      icon: IconEdit,
      action: () => {
        isNewLibrary.value = false;
        editingLibrary.value = currentLibrary.value;
        showLibraryEdit.value = true;
      }
    },
    {
      label: localeMsg.value.menu.library.remove,
      icon: IconLibraryRemove,
      disabled: appConfig.value?.libraries.length === 1,
      action: () => {
        if (appConfig.value?.libraries.length === 1) return;
        showRemoveLibraryMsgbox.value = true;
      }
    }
  ];
});

const clickLibrary = () => {
  if(!albumListRef.value?.isEditList) {
    config.album.id = 0;
    config.album.folderId = null;
    config.album.folderPath = '';
    config.album.selected = false;
  }
};

const doSwitchLibrary = async (libraryId: string) => {
  try {
    // Cancel any running indexing before switching
    if (config.index.status > 0 && config.index.albumQueue.length > 0) {
      for (const albumId of config.index.albumQueue) {
        await cancelIndexing(albumId);
      }
    }
    
    await switchLibrary(libraryId);
    // Reload the app to switch database
    window.location.reload();
  } catch (error) {
    console.error('Failed to switch library:', error);
  }
};

const onLibraryEditOk = async (library: Library) => {
  showLibraryEdit.value = false;
  
  if (isNewLibrary.value) {
    // Switch to the new library
    await doSwitchLibrary(library.id);
  } else {
    // Reload config to get updated name
    appConfig.value = await getAppConfig();
  }
};

const confirmRemoveLibrary = async () => {
  showRemoveLibraryMsgbox.value = false;
  
  if (!currentLibrary.value) return;
  
  try {
    await removeLibrary(currentLibrary.value.id);
    // Reload app to switch to the new current library
    window.location.reload();
  } catch (error) {
    console.error('Failed to remove library:', error);
  }
};

</script>

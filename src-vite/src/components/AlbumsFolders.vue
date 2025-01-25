<template>
  
  <ul v-if="children && children.length > 0">
    <li v-for="child in children" :key="child.id" class="pl-4">
      <div 
        :class="[
          'pr-1 m-1 border-l-2 flex items-center whitespace-nowrap hover:bg-gray-700 cursor-pointer', 
          child.id && config.albumFolderId === child.id ? 't-color-text-selected t-color-bg-selected border-sky-500 transition-colors duration-300' : 'border-gray-900'
        ]" 
        @click="clickFolder(albumId, child)"
        @dblclick="clickExpandFolder($event, child)"
      >
        <span v-if="child.children && child.children.length == 0" class="w-6"></span>
        <IconRight v-else
          :class="[
            'p-1 t-icon-size flex-shrink-0 transition-transform', 
            child.is_expanded && child.children && child.children.length > 0 ? 'rotate-90' : ''
          ]"
          @click="clickExpandFolder($event, child)"
        />
        <span class="flex-grow">{{ child.name }}</span>
        <DropDownMenu v-if="config.albumFolderId === child.id"
          :iconMenu="IconMore"
          :menuItems="moreMenuItems"
          :alignRight="true"
        />
      </div>
      <AlbumsFolders v-if="child.is_expanded" 
        :albumId="albumId"
        :children="child.children" 
      />
    </li>
  </ul>

</template>


<script setup lang="ts">

import { ref, watch, computed, onMounted, onUnmounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { useI18n } from 'vue-i18n';
import { useConfigStore } from '@/stores/configStore';
import { openFileExplorer } from '@/common/utils';

import AlbumsFolders from '@/components/AlbumsFolders.vue';
import DropDownMenu from '@/components/DropDownMenu.vue';

// folder icon
import IconRight from '@/assets/arrow-right.svg';
import IconMore from '@/assets/more.svg';
import IconRefresh from '@/assets/refresh.svg';
import IconCopyTo from '@/assets/copy-to.svg';
import IconMoveTo from '@/assets/move-to.svg';
import IconRename from '@/assets/rename.svg';
import IconDelete from '@/assets/trash.svg';
import IconOpenFolder from '@/assets/folder-open.svg';

const props = defineProps({
  albumId: {    // album id
    type: Number, 
    required: true,
  },
  children: {   // subfolders
    type: Array,
    required: false,
  },
});
/// i18n
const { locale, messages } = useI18n();
const localeMsg = computed(() => messages.value[locale.value]);

// config store
const config = useConfigStore();

/// click folder to select
const clickFolder = async (albumId, folder) => {
  try {
    const result = await invoke('select_folder', {
      albumId: albumId,
      parentId: 0,
      folderPath: folder.path,
    });

    // update config
    config.albumId = albumId;
    config.albumFolderId   = result.id;
    config.albumFolderName = result.name;
    config.albumFolderPath = result.path;

    // insert new property 'id' to folder object
    folder.id = config.albumFolderId;

    console.log('clickFolder:', result);
  } catch (error) {
    console.error("Error adding folder:", error);
  }
};


/// click expand icon to toggle folder expansion
const clickExpandFolder = async (event: Event, folder) => {
  // Prevents clickFolder() from being triggered
  event.stopPropagation(); 

  // clickFolder(folder);

  folder.is_expanded = !folder.is_expanded;

  if (folder.is_expanded && !folder.children) {
    try {
      const subfolders = await invoke('expand_folder', { path: folder.path, isRecursive: false });
      folder.children = subfolders.children;
    } catch (error) {
      console.error('clickExpandFolder...', error);
    }
  }
};

// more menuitems
const moreMenuItems = computed(() => {
  return [
    {
      label: localeMsg.value.menu_item_refresh,
      icon: IconRefresh,
      action: () => {
      }
    },
    {
      label: localeMsg.value.menu_item_rename,
      icon: IconRename,
      action: () => {
      }
    },
    {
      label: localeMsg.value.menu_item_delete,
      icon: IconDelete,
      action: () => {
      }
    },
    {
      label: localeMsg.value.menu_item_move_to,
      icon: IconMoveTo,
      action: () => {}
    },
    {
      label: localeMsg.value.menu_item_copy_to,
      // icon: IconCopyTo,
      action: () => {}
    },
    {
      label: localeMsg.value.menu_item_new_folder,
      action: () => {
      }
    },
    {
      label: localeMsg.value.menu_item_open_folder,
      // icon: IconOpenFolder,
      action: () => {
        openFileExplorer(config.albumFolderPath);
      }
    }
  ];
});

</script>


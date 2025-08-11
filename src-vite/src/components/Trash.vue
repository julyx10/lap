<template>

  <div class="w-full flex flex-col select-none">

    <!-- title bar -->
    <div class="px-2 py-3 h-12 flex items-center justify-between" data-tauri-drag-region>
      <span class="cursor-default" data-tauri-drag-region>{{ titlebar }}</span>
      <DropDownMenu 
        :iconMenu="IconMore" 
        :menuItems="moreMenuItems"
      />
    </div>

    <!-- trash files -->
    <div :class="[
      'my-1 mr-1 h-8 flex items-center rounded border-l-2 border-base-200 hover:bg-base-content/10 whitespace-nowrap cursor-pointer',
      {
        'text-base-content bg-base-content/10 border-primary': config.trashFolderId === 0,
      }
    ]" @click="clickTrashFiles()">
      <IconTrash class="mx-1 w-5 h-5 shrink-0" />
      <div class="overflow-hidden whitespace-pre text-ellipsis">
        {{ $t('trash.files') }}
      </div>
    </div>

    <!-- trash folders -->
    <div class="mt-1 px-2 py-1 text-sm text-base-content/30 cursor-default">
      {{ $t('trash.folders') }}
    </div>
    <div v-if="trash_folders.length > 0" class="flex-grow overflow-x-hidden overflow-y-auto">
      <ul>
        <li v-for="folder in trash_folders" :key="folder.id">
          <div :class="[
            'my-1 mr-1 h-8 flex items-center rounded border-l-2 border-base-200 hover:bg-base-content/10 whitespace-nowrap cursor-pointer group',
            {
              'text-base-content bg-base-content/10 border-primary': config.trashFolderId === folder.id,
            }
          ]" @click="clickTrashFolder(folder)">
            <IconFolderTrash class="mx-1 h-5 shrink-0" />
            <div class="overflow-hidden whitespace-pre text-ellipsis">
              {{ folder.name }}
            </div>

            <DropDownMenu 
              :class="[
                'ml-auto px-1 rounded',
                config.trashFolderId != folder.id ? 'invisible group-hover:visible' : ''
              ]" 
              :iconMenu="IconMore" 
              :menuItems="trashFolderMenuItems" 
              :smallIcon="true" 
            />
          </div>
        </li>
      </ul>
    </div>

    <!-- Display message if no trash folders are found -->
    <div v-else class="mt-10 flex flex-col items-center justify-center text-base-content/30">
      <IconFolderTrash class="w-12 h-12" />
      <span class="mt-2">{{ $t('tooltip.not_found.folders') }}</span>
    </div>

  </div>

</template>

<script setup lang="ts">

import { ref, computed, onMounted } from 'vue';
import { useI18n } from 'vue-i18n';
import { config } from '@/common/utils';
import { getTrashFolders } from '@/common/api';

import DropDownMenu from '@/components/DropDownMenu.vue';

import {
  IconMore,
  IconTrashEmpty,
  IconTrash,
  IconTrashRestore,
  IconFolderTrash,
} from '@/common/icons';

const props = defineProps({
  titlebar: {
    type: String,
    required: true
  }
});

/// i18n
const { locale, messages } = useI18n();
const localeMsg = computed(() => messages.value[locale.value]);

// trash folders
const trash_folders = ref([]);

// more menuitems
const moreMenuItems = computed(() => {
  return [
    {
      label: localeMsg.value.menu.trash.empty,
      icon: IconTrashEmpty,
      action: () => {
        // emptyTrash();
      }
    },
  ];
});

// trash folder menuitems
const trashFolderMenuItems = computed(() => {
  return [
    {
      label: localeMsg.value.menu.trash.restore,
      icon: IconTrashRestore,
      action: () => {
        // restoreFromTrash();
      }
    },
    {
      label: localeMsg.value.menu.trash.delete,
      icon: IconTrash,
      action: () => {
        // deleteFromTrash();
      }
    }
  ];
});

onMounted(() => {
  if (trash_folders.value.length === 0) {
    // fetch trash folders
    getTrashFolders().then((folders) => {
      trash_folders.value = folders || [];
      console.log('trash_folders', trash_folders.value);
    });
  }
});

// click trash files
function clickTrashFiles() {
  console.log('clickTrashFiles');
  config.trashAlbumId = null;
  config.trashFolderId = 0;    // 0 means trash files
  config.trashFolderPath = '';
}

// click trash folder
function clickTrashFolder(folder: any) {
  console.log('clickTrashFolder', folder);
  config.trashAlbumId = folder.album_id;
  config.trashFolderId = folder.id;
  config.trashFolderPath = folder.path;
}

</script>
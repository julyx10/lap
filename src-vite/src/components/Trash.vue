<template>

  <div class="w-full flex flex-col select-none">

    <!-- title bar -->
    <div class="px-2 py-3 h-12 flex items-center justify-between whitespace-nowrap" data-tauri-drag-region>
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
        'text-base-content bg-base-content/10 border-primary': config.albumId === trashAlbumId,
      }
    ]" @click="clickTrash()">
      <IconTrash class="mx-1 w-5 h-5 shrink-0" />
      <div class="overflow-hidden whitespace-pre text-ellipsis">
        {{ $t('sidebar.trash') }}
      </div>
    </div>

  </div>

</template>

<script setup lang="ts">

import { ref, computed, onMounted } from 'vue';
import { useI18n } from 'vue-i18n';
import { config } from '@/common/utils';
import { getAlbum, getDbFiles, getTrashFolders, permanentlyDeleteFiles, permanentlyDeleteFolders } from '@/common/api';

import DropDownMenu from '@/components/DropDownMenu.vue';

import {
  IconMore,
  IconTrashEmpty,
  IconTrash,
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

const trashAlbumId = ref(null);

// more menuitems
const moreMenuItems = computed(() => {
  return [
    {
      label: localeMsg.value.menu.trash.empty,
      icon: IconTrashEmpty,
      action: async () => {
        const files = await getDbFiles(trashAlbumId.value, '', '', '', '', false, 0, true, 0);
        const fileIds = files.map(f => f.id);
        const folders = await getTrashFolders();
        const folderIds = folders.map(f => f.id);
        await permanentlyDeleteFiles(fileIds);
        await permanentlyDeleteFolders(folderIds);
      }
    },
  ];
});

onMounted(async () => {
  const trashAlbum = await getAlbum(0); // TODO: find a better way to get trash album
  if(trashAlbum) {
      trashAlbumId.value = trashAlbum.id;
  }
});

// click trash files
function clickTrash() {
  config.albumId = trashAlbumId.value;
  config.albumFolderPath = ''; // TODO: get trash album path
}

</script>
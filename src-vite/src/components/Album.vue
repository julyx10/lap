<template>
    
  <div class="w-full flex flex-col select-none">

    <!-- title bar -->
    <div class="px-2 py-3 h-12 flex items-center justify-between whitespace-nowrap" data-tauri-drag-region>
      <span class="cursor-default" data-tauri-drag-region>{{ titlebar }}</span>

      <TButton v-if="isEditList" 
        :icon="IconClose" 
        @click="clickCloseEditList"
      />
      <DropDownMenu v-else 
        :iconMenu="IconMore" 
        :menuItems="moreMenuItems"
      />
    </div>

    <AlbumList ref="albumListRef" 
      :key="albumListKey"
      v-model:albumId="config.albumId"
      v-model:folderId="config.albumFolderId"
      v-model:folderPath="config.albumFolderPath"
      :componentId="0"
    />
  </div> 

</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { useI18n } from 'vue-i18n';
import { config } from '@/common/utils';
import { listen } from '@tauri-apps/api/event';
import { useUIStore } from '@/stores/uiStore';

import { IconMore, IconAdd, IconOrder, IconRefresh, IconClose } from '@/common/icons';
import AlbumList from '@/components/AlbumList.vue';
import DropDownMenu from '@/components/DropDownMenu.vue';
import TButton from '@/components/TButton.vue';

const props = defineProps({
  titlebar: {
    type: String,
    required: true
  }
});

/// i18n
const { locale, messages } = useI18n();
const localeMsg = computed(() => messages.value[locale.value]);
const uiStore = useUIStore();

let unlistenKeydown: () => void;

onMounted(async () => {
  unlistenKeydown = await listen('global-keydown', handleKeyDown);
});

onUnmounted(() => {
  if (unlistenKeydown) {
    unlistenKeydown();
  }
  uiStore.popInputHandler();
});

const albumListRef = ref<AlbumList | null>(null);

const isEditList = ref(false);

// refresh component
const albumListKey = ref(0);

// more menuitems
const moreMenuItems = computed(() => {
  return [
    {
      label: localeMsg.value.menu.album.add,
      icon: IconAdd,
      action: () => {
        albumListRef.value.clickNewAlbum();
      }
    },
    {
      label: "-",   // separator
      action: () => {}
    },
    {
      label: localeMsg.value.menu.album.refresh,
      icon: IconRefresh,
      disabled: config.albumId === null,
      action: async () => {
        albumListRef.value.refreshAlbums(); 
      }
    },
    {
      label: localeMsg.value.menu.album.reorder,
      icon: IconOrder,
      disabled: config.albumId === null,
      action: () => {
        isEditList.value = true;
        albumListRef.value.isEditList = true;
        uiStore.pushInputHandler('AlbumList-edit');
      }
    },
  ];
});

const clickCloseEditList = () => {
  isEditList.value = false;
  albumListRef.value.isEditList = false;
  uiStore.popInputHandler();
};

const handleKeyDown = (event) => {
  if (isEditList.value && event.payload.key === 'Escape') {
    clickCloseEditList();
  }
};

</script>

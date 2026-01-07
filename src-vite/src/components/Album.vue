<template>
    
  <div class="w-full h-full flex flex-col select-none">

    <!-- title bar -->
    <div class="p-1 h-12 flex items-center justify-end whitespace-nowrap" data-tauri-drag-region>
      <div v-if="isEditList" class="pl-1 text-primary cursor-pointer" @click="clickCloseEditList">{{ $t('menu.album.reorder_exit') }}</div>
      <TButton v-if="isEditList" 
        :icon="IconRestore"
        :selected="true"
        @click="clickCloseEditList"
      />
      <ContextMenu v-else 
        :iconMenu="IconMore" 
        :menuItems="moreMenuItems"
      />
    </div>
    
    <!-- all files -->
    <div 
      :class="[ 
        'mx-1 p-1 h-10 flex items-center rounded-box whitespace-nowrap group',
        isEditList 
          ? 'text-base-content/30' 
          : (config.album.id === 0 ? 'text-primary bg-base-100 hover:bg-base-100 cursor-pointer' : 'hover:text-base-content hover:bg-base-100/30 cursor-pointer'),
      ]"
      @click="clickAllFiles"
    >
      <IconPhotoAll class="mx-1 w-5 h-5 shrink-0" />
      <div class="overflow-hidden whitespace-pre text-ellipsis">
        {{ $t('album.all_files') }}
      </div>
      <span v-if="totalCount > 0" class="ml-auto px-1 text-xs text-base-content/30">
        {{ totalCount.toLocaleString() }}
      </span>
    </div>

    <AlbumList ref="albumListRef" 
      :key="albumListKey"
      :componentId="0"
    />
  </div> 

</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { useI18n } from 'vue-i18n';
import { config } from '@/common/config';
import { listen } from '@tauri-apps/api/event';
import { useUIStore } from '@/stores/uiStore';

import { IconMore, IconAdd, IconOrder, IconRefresh, IconPhotoAll, IconRestore } from '@/common/icons';
import { getTotalCountAndSum } from '@/common/api';
import AlbumList from '@/components/AlbumList.vue';
import ContextMenu from '@/components/ContextMenu.vue';
import TButton from '@/components/TButton.vue';

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

let unlistenKeydown: () => void;
const totalCount = ref(0);

onMounted(async () => {
    unlistenKeydown = await listen('global-keydown', handleKeyDown);

    // get total count
    getTotalCountAndSum().then((result) => {
      if(result) {
        totalCount.value = result[0];
      }
    });
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
      action: async () => {
        albumListRef.value.refreshAlbums(); 
      }
    },
    {
      label: localeMsg.value.menu.album.reorder,
      icon: IconOrder,
      action: () => {
        isEditList.value = true;
        albumListRef.value.isEditList = true;
        uiStore.pushInputHandler('AlbumList-edit');
      }
    },
  ];
});

const clickAllFiles = () => {
  if(!isEditList.value) {
    config.album.id = 0;
    config.album.folderId = null;
    config.album.folderPath = '';
    config.album.selected = false;
  }
};

const clickCloseEditList = () => {
  isEditList.value = false;
  albumListRef.value.isEditList = false;
  uiStore.popInputHandler();
};

const handleKeyDown = (event: KeyboardEvent) => {
  if (isEditList.value && event.payload.key === 'Escape') {
    clickCloseEditList();
  }
};

</script>

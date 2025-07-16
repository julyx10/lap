<template>
    
  <div class="w-full flex flex-col select-none">

    <!-- title bar -->
    <div class="px-2 py-3 h-12 flex items-center justify-between" data-tauri-drag-region>
      <span class="cursor-default" data-tauri-drag-region>{{ titlebar }}</span>

      <TButton v-if="isEditList" 
        :icon="IconOk" 
        @click="clickOk"
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
import { ref, computed } from 'vue';
import { useI18n } from 'vue-i18n';
import { config } from '@/common/utils';

import { IconMore, IconAdd, IconEdit, IconRefresh, IconOk } from '@/common/icons';
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
      label: localeMsg.value.menu.album.edit_list,
      icon: IconEdit,
      action: () => {
        isEditList.value = true;
        albumListRef.value.isEditList = true;
      }
    },
    {
      label: localeMsg.value.menu.refresh,
      icon: IconRefresh,
      action: () => {
        albumListKey.value += 1;  // refresh component
      }
    }
  ];
});

const clickOk = () => {
  isEditList.value = false;
  albumListRef.value.isEditList = false;
};

</script>

<template>
    
  <div class="w-full flex flex-col" style="user-select: none;">

    <!-- title bar -->
    <div class="px-2 py-3 h-12 flex items-center justify-between" data-tauri-drag-region>
      <span class="cursor-default" data-tauri-drag-region>{{ titlebar }}</span>

      <IconOk v-if="isEditList" class="t-icon-size-sm t-icon-hover" @click="clickOk" />
      <DropDownMenu v-else :iconMenu="IconMore" :menuItems="moreMenuItems" />
    </div>

    <SelectAlbum ref="selectAlbumRef" 
      :key="selectAlbumKey"
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

import { IconMore, IconAdd, IconLink, IconEdit, IconRefresh, IconOk } from '@/common/icons';
import SelectAlbum from '@/components/SelectAlbum.vue';
import DropDownMenu from '@/components/DropDownMenu.vue';

const props = defineProps({
  titlebar: {
    type: String,
    required: true
  }
});

/// i18n
const { locale, messages } = useI18n();
const localeMsg = computed(() => messages.value[locale.value]);

const selectAlbumRef = ref<SelectAlbum | null>(null);

const isEditList = ref(false);

// refresh component
const selectAlbumKey = ref(0);

// more menuitems
const moreMenuItems = computed(() => {
  return [
    {
      label: localeMsg.value.menu_item_add_folder,
      icon: IconAdd,
      action: () => {
        selectAlbumRef.value.clickNewAlbum();
      }
    },
    {
      label: localeMsg.value.menu_item_add_url,
      icon: IconLink,
      action: () => {
        // TODO: add url to album
        console.log('add url to album');
      }
    },
    {
      label: "-",   // separator
      action: () => {}
    },
    {
      label: localeMsg.value.menu_item_edit_list,
      icon: IconEdit,
      action: () => {
        isEditList.value = true;
        selectAlbumRef.value.isEditList = true;
      }
    },
    {
      label: localeMsg.value.menu_item_refresh,
      icon: IconRefresh,
      action: () => {
        selectAlbumKey.value += 1;  // refresh component
      }
    }
  ];
});

const clickOk = () => {
  isEditList.value = false;
  selectAlbumRef.value.isEditList = false;
};

</script>

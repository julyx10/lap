<template>

  <div class="w-full flex flex-col select-none" >
      
      <!-- title bar -->
      <div class="px-2 py-3 h-12 flex items-center justify-between whitespace-nowrap" data-tauri-drag-region>
        <span class="pl-1 cursor-default" data-tauri-drag-region>{{ titlebar }}</span>
        <!-- <TButton :icon="IconRefresh" @click="clickRefresh"/> -->
      </div>
      
      <!-- favorite files -->
      <div 
        :class="[ 
          'mx-1 p-1 h-10 flex items-center rounded-box whitespace-nowrap cursor-pointer hover:bg-base-100 group',
          config.favorite.folderId === 0 ? 'text-primary' : 'hover:text-base-content',
        ]"
        @click="clickFavoriteFiles()"
      >
        <IconFavorite class="mx-1 w-5 h-5 shrink-0" />
        <div class="overflow-hidden whitespace-pre text-ellipsis">
          {{ $t('favorite.files') }}
        </div>
      </div>

      <!-- favorite folders -->
      <div class="my-2 px-2 py-1 text-sm text-base-content/30 cursor-default whitespace-nowrap">
        {{ $t('favorite.folders') }}
      </div>
      <div v-if="favorite_folders.length > 0" class="flex-grow overflow-x-hidden overflow-y-auto">
        <ul>
          <li v-for="folder in favorite_folders" :key="folder.id">
            <div 
              :class="[ 
                'mx-1 p-1 h-10 flex items-center rounded-box whitespace-nowrap cursor-pointer hover:bg-base-100 group', 
                config.favorite.folderId === folder.id ? 'text-primary' : 'hover:text-base-content',
              ]"
              @click="clickFavoriteFolder(folder)"
            >
              <IconFolderFavorite
                class="mx-1 h-5 shrink-0"
              />
              <div class="overflow-hidden whitespace-pre text-ellipsis">
                {{ folder.name }}
              </div>
              <ContextMenu
                :class="[
                  'ml-auto flex flex-row items-center text-base-content/30',
                  config.favorite.folderId != folder.id ? 'invisible group-hover:visible' : ''
                ]"
                :iconMenu="IconMore"
                :menuItems="favoriteFolderMenuItems"
                :smallIcon="true"
              />
            </div>
          </li>
        </ul>
      </div>

      <!-- Display message if no favorite folders are found -->
      <div v-else class="mt-10 flex flex-col items-center justify-center text-base-content/30 cursor-default">
        <IconFolderFavorite class="w-8 h-8" />
        <span class="mt-2 whitespace-nowrap">{{ $t('tooltip.not_found.folders') }}</span>
      </div>

    </div>
    
  </template>
  
<script setup lang="ts">

import { ref, computed, onMounted } from 'vue';
import { useI18n } from 'vue-i18n';
import { config } from '@/common/config';
import { getFavoriteFolders, setFolderFavorite, getAlbum } from '@/common/api';

import ContextMenu from '@/components/ContextMenu.vue';

import { 
  IconMore,
  IconFavorite, 
  IconFolderFavorite,
  IconUnFavorite, 
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

// favorite folders
const favorite_folders = ref([]);

// favorite folder menuitems
const favoriteFolderMenuItems = computed(() => {
  return [
    {
      label: localeMsg.value.menu.meta.unfavorite,
      icon: IconUnFavorite,
      action: () => {
        UnFavorite();
      }
    },
  ];
});

onMounted(() => {
  if (favorite_folders.value.length === 0) {
    // fetch favorite folders
    getFavoriteFolders().then((folders) => {
      favorite_folders.value = folders || [];

      // get album info
      favorite_folders.value.forEach((folder) => {
        getAlbum(folder.album_id).then((album) => {
          folder.is_hidden = album.is_hidden;
        });
      });

      console.log('favorite_folders', favorite_folders.value);
    });
  }
});

// click favorite files
function clickFavoriteFiles() {
  console.log('clickFavoriteFiles');
  config.favorite.albumId = null;
  config.favorite.folderId = 0;    // 0 means favorite files
  config.favorite.folderPath = '';
}

// click favorite folder
function clickFavoriteFolder(folder: any) {
  console.log('clickFavoriteFolder', folder);
  config.favorite.albumId = folder.album_id;
  config.favorite.folderId = folder.id;
  config.favorite.folderPath = folder.path;
}

// unfavorite
function UnFavorite() {
  console.log('UnFavorite');
  setFolderFavorite(config.favorite.folderId, false).then(() => {
    // get the index of the folder
    const index = favorite_folders.value.findIndex((f: any) => f.id === config.favorite.folderId);

    // remove the folder from the list
    favorite_folders.value = favorite_folders.value.filter((f: any) => f.id !== config.favorite.folderId);

    if (favorite_folders.value.length === 0) {
      config.favorite.folderId = 0;  // 0 means favorite files
      config.favorite.albumId = 0;
      config.favorite.folderPath = '';
    } else if (index === 0) {  // if the folder is the first one, set the first one as the favorite
      config.favorite.folderId = favorite_folders.value[index].id;
      config.favorite.albumId = favorite_folders.value[index].album_id;
      config.favorite.folderPath = favorite_folders.value[index].path;
    } else {
      config.favorite.folderId = favorite_folders.value[index - 1].id;
      config.favorite.albumId = favorite_folders.value[index - 1].album_id;
      config.favorite.folderPath = favorite_folders.value[index - 1].path;
    }

    console.log('favorite_folders', favorite_folders.value);
  });

}

</script>
<template>

  <div class="w-full flex flex-col" style="user-select: none;">
      
      <!-- title bar -->
      <div class="px-2 py-3 h-12 flex items-center justify-between" data-tauri-drag-region>
        <span class="cursor-default" data-tauri-drag-region>{{ titlebar }}</span>
        <IconRefresh class="t-icon-size-sm t-icon-hover" @click="clickRefresh"/>
      </div>
      
      <!-- favorite files -->
      <div 
        :class="[ 
          'my-1 mr-1 h-8 rounded border-l-2 flex items-center whitespace-nowrap border-transparent t-color-bg-hover cursor-pointer', 
          { 
            't-color-text-selected t-color-bg-selected t-color-border-selected transition-colors duration-300': config.favoriteFolderId === 0, 
          }
        ]"
        @click="clickFavoriteFiles()"
      >
        <IconFile
          class="mx-1 h-5 flex-shrink-0"
        />
        <div class="overflow-hidden whitespace-pre text-ellipsis">
          {{ $t('favorite_files') }}
        </div>
      </div>

      <!-- favorite folders -->
      <div class="mt-1 px-2 py-1 text-sm text-gray-500 cursor-default">
        {{ $t('favorite_folders') }}
      </div>
      <div v-if="favorite_folders.length > 0" class="overflow-x-hidden overflow-y-auto t-scrollbar-dark">
        <ul>
          <li v-for="folder in favorite_folders" :key="folder.id">
            <div 
              :class="[ 
                'my-1 mr-1 h-8 rounded border-l-2 flex items-center whitespace-nowrap border-transparent t-color-bg-hover cursor-pointer group', 
                { 
                  't-color-text-selected t-color-bg-selected t-color-border-selected transition-colors duration-300': config.favoriteFolderId === folder.id, 
                }
              ]"
              @click="clickFavoriteFolder(folder)"
            >
              <IconFolder
                class="mx-1 h-5 flex-shrink-0"
              />
              <div class="overflow-hidden whitespace-pre text-ellipsis">
              {{ folder.name }}
              </div>

              <DropDownMenu
                :class="[
                  'ml-auto px-1 rounded t-color-bg-selected',
                  config.favoriteFolderId != folder.id ? 'hidden group-hover:block' : ''
                ]"
                :iconMenu="IconMore"
                :menuItems="moreMenuItems"
              />
            </div>
          </li>
        </ul>
      </div>

      <!-- Display message if no favorite folders are found -->
      <div v-else class="mt-10 flex items-center justify-center">
        {{ $t('favorite_folders_empty') }}
      </div>

    </div>
    
  </template>
  
<script setup lang="ts">

import { ref, computed, onMounted } from 'vue';
import { useI18n } from 'vue-i18n';
import { config } from '@/common/utils';
import { getFavoriteFolders, setFolderFavorite } from '@/common/api';

import DropDownMenu from '@/components/DropDownMenu.vue';

import { 
  IconRefresh, 
  IconFile, 
  IconFolder,
  IconMore,
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

// more menuitems
const moreMenuItems = computed(() => {
  return [
    {
      label: localeMsg.value.menu_item_unfavorite,
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
    getFavorites();
  }
});

// get favorites
async function getFavorites() {
  const folders = await getFavoriteFolders();
  favorite_folders.value = folders;

  console.log('favorite_folders', folders);
}

// refresh component
function clickRefresh() {
  console.log('clickRefresh');
  getFavorites().then(() => {
    clickFavoriteFiles();
  });
}

// click favorite files
function clickFavoriteFiles() {
  console.log('clickFavoriteFiles');
  config.favoriteFolderId = 0;    // 0 means favorite files
}

// click favorite folder
function clickFavoriteFolder(folder: any) {
  console.log('clickFavoriteFolder', folder);
  config.favoriteAlbumId = folder.album_id;
  config.favoriteFolderId = folder.id;
  config.favoriteFolderPath = folder.path;
}

// unfavorite
function UnFavorite() {
  console.log('UnFavorite');
  setFolderFavorite(config.favoriteFolderId, false).then(() => {
    favorite_folders.value = favorite_folders.value.filter((f: any) => f.id !== config.favoriteFolderId);
  });
}

</script>
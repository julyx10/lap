<template>

  <div class="w-full flex flex-col select-none">

    <!-- title bar -->
    <div class="px-2 py-3 h-12 flex items-center justify-between" data-tauri-drag-region>
      <span class="cursor-default" data-tauri-drag-region>{{ titlebar }}</span>
      <!-- <TButton :icon="IconRefresh" @click="clickRefresh"/> -->
    </div>

    <!-- trash files -->
    <div :class="[
      'my-1 mr-1 h-8 flex items-center rounded border-l-2 border-base-200 hover:bg-base-content/10 whitespace-nowrap cursor-pointer',
      {
        'text-base-content bg-base-content/10 border-primary': config.favoriteFolderId === 0,
      }
    ]" @click="clickFavoriteFiles()">
      <IconTrash class="mx-1 h-5 shrink-0" />
      <div class="overflow-hidden whitespace-pre text-ellipsis">
        {{ $t('trash.files') }}
      </div>
    </div>

    <!-- trash folders -->
    <div class="mt-1 px-2 py-1 text-sm text-base-content/30 cursor-default">
      {{ $t('trash.folders') }}
    </div>
    <div v-if="favorite_folders.length > 0" class="flex-grow overflow-x-hidden overflow-y-auto">
      <ul>
        <li v-for="folder in favorite_folders" :key="folder.id">
          <div :class="[
            'my-1 mr-1 h-8 flex items-center rounded border-l-2 border-base-200 hover:bg-base-content/10 whitespace-nowrap cursor-pointer group',
            {
              'text-base-content bg-base-content/10 border-primary': config.favoriteFolderId === folder.id,
            }
          ]" @click="clickFavoriteFolder(folder)">
            <IconFolderTrash class="mx-1 h-5 shrink-0" />
            <div class="overflow-hidden whitespace-pre text-ellipsis">
              {{ folder.name }}
            </div>

            <DropDownMenu :class="[
              'ml-auto px-1 rounded',
              config.favoriteFolderId != folder.id ? 'hidden group-hover:block' : ''
            ]" :iconMenu="IconMore" :menuItems="moreMenuItems" :smallIcon="true" />
          </div>
        </li>
      </ul>
    </div>

    <!-- Display message if no favorite folders are found -->
    <div v-else class="mt-10 flex items-center justify-center">
      {{ $t('tooltip.not_found.folders') }}
    </div>

  </div>

</template>

<script setup lang="ts">

import { ref, computed, onMounted } from 'vue';
import { useI18n } from 'vue-i18n';
import { config } from '@/common/utils';
import { getFavoriteFolders, setFolderFavorite } from '@/common/api';

import DropDownMenu from '@/components/DropDownMenu.vue';
import TButton from '@/components/TButton.vue';

import {
  IconRefresh,
  IconTrash,
  IconFolderTrash,
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
      label: localeMsg.value.menu.unfavorite,
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
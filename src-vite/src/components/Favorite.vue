<template>
  <div class="sidebar-panel">
    <div
      :class="[
        'sidebar-item',
        libConfig.favorite.folderId === 0 && libConfig.favorite.rating === 0 ? 'sidebar-item-selected' : 'sidebar-item-hover',
      ]"
      @click="clickFavoriteFiles()"
    >
      <IconHeart class="mx-1 w-5 h-5 shrink-0" />
      <div class="sidebar-item-label">
        {{ $t('favorite.files') }}
      </div>
    </div>

    <div class="sidebar-panel-header">
      <span class="sidebar-panel-header-title">{{ $t('favorite.ratings') }}</span>
    </div>
    <div class="grow overflow-x-hidden overflow-y-auto">
      <ul>
        <li v-for="rating in [5, 4, 3, 2, 1]" :key="rating">
          <div
            :class="[
              'sidebar-item',
              libConfig.favorite.rating === rating ? 'sidebar-item-selected' : 'sidebar-item-hover',
            ]"
            @click="clickRating(rating)"
          >
            <div class="mx-1 flex items-center gap-0.5">
              <IconStarFilled
                v-for="index in rating"
                :key="index"
                class="w-4 h-4 shrink-0"
              />
            </div>
          </div>
        </li>
      </ul>
    </div>

    <!-- Hidden for now: favorite folders
    <div v-if="favorite_folders.length > 0" class="sidebar-panel-header">
      <span class="sidebar-panel-header-title">{{ $t('favorite.folders') }}</span>
    </div>
    <div class="grow overflow-x-hidden overflow-y-auto">
      <ul>
        <li v-for="folder in favorite_folders" :key="folder.id">
          <div
            :class="[
              'sidebar-item group',
              libConfig.favorite.folderId === folder.id ? 'sidebar-item-selected' : 'sidebar-item-hover',
            ]"
            @click="clickFavoriteFolder(folder)"
          >
            <IconFolderFavorite class="mx-1 h-5 shrink-0" />
            <div class="sidebar-item-label">
              {{ folder.name }}
            </div>
            <ContextMenu
              :class="[
                'ml-auto flex flex-row items-center text-base-content/30',
                libConfig.favorite.folderId != folder.id ? 'invisible group-hover:visible' : ''
              ]"
              :iconMenu="IconMore"
              :menuItems="favoriteFolderMenuItems"
              :smallIcon="true"
            />
          </div>
        </li>
      </ul>
    </div>
    -->
  </div>
</template>
  
<script setup lang="ts">
import { libConfig } from '@/common/config';

import { IconHeart, IconStarFilled } from '@/common/icons';

// Hidden for now: favorite folder support kept here for easy restore.
// import { ref, computed, onMounted } from 'vue';
// import { useI18n } from 'vue-i18n';
// import { getFavoriteFolders, setFolderFavorite } from '@/common/api';
// import { getFolderName } from '@/common/utils';
// import ContextMenu from '@/components/ContextMenu.vue';
// import { IconMore, IconFolderFavorite, IconUnFavorite } from '@/common/icons';

const props = defineProps({
  titlebar: {
    type: String,
    required: true
  }
});

// Hidden for now: favorite folders
// const { locale, messages } = useI18n();
// const localeMsg = computed(() => messages.value[locale.value] as any);
// interface FavoriteFolder {
//   id: number;
//   album_id: number;
//   path: string;
//   name?: string;
// }
// const favorite_folders = ref<FavoriteFolder[]>([]);
// const favoriteFolderMenuItems = computed(() => {
//   return [
//     {
//       label: localeMsg.value.menu.meta.unfavorite,
//       icon: IconUnFavorite,
//       action: () => {
//         UnFavorite();
//       }
//     },
//   ];
// });
// onMounted(() => {
//   if (favorite_folders.value.length === 0) {
//     getFavoriteFolders().then((folders) => {
//       favorite_folders.value = folders || [];
//       favorite_folders.value.forEach((folder) => {
//         folder.name = getFolderName(folder.path);
//       });
//     });
//   }
// });

// click favorite files
function clickFavoriteFiles() {
  libConfig.favorite.albumId = null;
  libConfig.favorite.folderId = 0;    // 0 means favorite files
  libConfig.favorite.folderPath = '';
  libConfig.favorite.rating = 0;
}

if (libConfig.favorite.folderId !== 0) {
  clickFavoriteFiles();
}

function clickRating(rating) {
  libConfig.favorite.albumId = null;
  libConfig.favorite.folderId = 0;
  libConfig.favorite.folderPath = '';
  libConfig.favorite.rating = rating;
}

// Hidden for now: favorite folders
// function clickFavoriteFolder(folder: any) {
//   libConfig.favorite.albumId = folder.album_id;
//   libConfig.favorite.folderId = folder.id;
//   libConfig.favorite.folderPath = folder.path;
// }
// function UnFavorite() {
//   setFolderFavorite(libConfig.favorite.folderId, false).then(() => {
//     const index = favorite_folders.value.findIndex((f: any) => f.id === libConfig.favorite.folderId);
//     favorite_folders.value = favorite_folders.value.filter((f: any) => f.id !== libConfig.favorite.folderId);
//     if (favorite_folders.value.length === 0) {
//       libConfig.favorite.folderId = 0;
//       libConfig.favorite.albumId = null;
//       libConfig.favorite.folderPath = '';
//     } else if (index === 0) {
//       libConfig.favorite.folderId = favorite_folders.value[index].id;
//       libConfig.favorite.albumId = favorite_folders.value[index].album_id;
//       libConfig.favorite.folderPath = favorite_folders.value[index].path;
//     } else {
//       libConfig.favorite.folderId = favorite_folders.value[index - 1].id;
//       libConfig.favorite.albumId = favorite_folders.value[index - 1].album_id;
//       libConfig.favorite.folderPath = favorite_folders.value[index - 1].path;
//     }
//   });
// }

</script>

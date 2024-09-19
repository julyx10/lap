<template>
  
<!-- title bar -->
<div class="absolute flex flex-row px-2 py-3 items-center justify-between w-full" style="user-select: none;">
  <div>
    {{ titlebar }}
  </div>
  <div class="flex flex-row h-6">
    <IconAdd class="p-1 hover:text-gray-200 transition-colors duration-300" @click="clickAdd" />
    <IconDelete  
      :class="[
        'p-1 ', 
        gAlbumId ? 'hover:text-gray-200 transition-colors duration-300' : 'text-gray-700'
      ]" 
      @click="showDeleteAlbumMsgbox = true" />
    <!-- <IconRefresh class="p-1 hover:text-gray-200 transition-colors duration-300" @click="clickRefresh"/> -->
  </div>
</div>

<!-- albums -->
<div v-if="gAlbums.length > 0" class="flex-1 mt-12 overflow-auto scrollbar-thin scrollbar-thumb-gray-700 scrollbar-track-gray-800">
  <ul>
    <li v-for="album in gAlbums" :key="album.id" style="user-select: none;" >
      <div 
        :class="[
          'p-2 flex items-center whitespace-nowrap hover:bg-gray-700', 
          gAlbumId === album.id ? 'text-gray-300 bg-gray-800' : ''
        ]"
        @click="clickAlbum(album)"
        @dblclick="dblclickAlbum(album)"
      >
        <component :is="album.is_expanded ? IconFolderOpen : IconFolder" class="size-6 pr-1 flex-shrink-0"  @click="clickExpandAlbum(album)"/>
        {{ album.name }} - {{ album.id }}
      </div>
      <Folders v-if="album.is_expanded" :album_id="album.id" :children="album.children" />
    </li>
  </ul>
</div>

<!-- Display message if no albums are found -->
<div v-else class="flex items-center justify-center w-full">
  {{ $t('no_albums') }}
</div>

<MessageBox
  v-if="showDeleteAlbumMsgbox"
  :visible="showDeleteAlbumMsgbox"
  :title="$t('delete_album_msgbox_title')"
  :message="$t('delete_album_msgbox_content')"
  :confirmText="$t('delete_album_msgbox_ok')"
  :cancelText="$t('delete_album_msgbox_cancel')"
  @confirm="clickDeleteConfirm"
  @cancel="showDeleteAlbumMsgbox = false"
  @close="showDeleteAlbumMsgbox = false"
/>

</template>


<script setup lang="ts">

import { ref, inject, computed, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api';
import { appWindow } from '@tauri-apps/api/window';
import Folders from '@/components/AlbumsFolders.vue';
import MessageBox from '@/components/MessageBox.vue';

/// i18n
import { useI18n } from 'vue-i18n';
const { locale, messages } = useI18n();
const localeMessages = computed(() => messages.value[locale.value]);

// toolbar icons
import IconAdd from '@/assets/folder-plus.svg';
import IconDelete from '@/assets/folder-minus.svg';
// import IconRefresh from '@/assets/arrow-path.svg';

// folder icon
import IconFolder from '@/assets/folder.svg';
import IconFolderOpen from '@/assets/folder-open.svg';

const gAlbums = inject('gAlbums');         // global albums
const gAlbumId = inject('gAlbumId');     // global album id
const gFolderId = inject('gFolderId');   // global folder id

const props = defineProps({
  titlebar: {
    type: String,
    required: true
  }
});

const showDeleteAlbumMsgbox = ref(false);

// Fetch albums on mount
onMounted(() => {
  if (gAlbums.value.length === 0) {
    getAlbums();
  }
});

const getAlbumById = (id) => gAlbums.value.find(album => album.id === id);

/// Add albums
const clickAdd = async () => {
  try {
    const new_album = await invoke('add_album', { window: appWindow, title: localeMessages.value.add_album_title });
    gAlbums.value.push(new_album);

    console.log('Add album...', new_album);
  } catch (error) {
    console.error('Failed to add album:', error);
  }
};


/// Delete an album
const clickDeleteConfirm = async () => {
  try {
    if (gAlbumId.value) {
      const result = await invoke('delete_album', { id: getAlbumById(gAlbumId.value).id });

      // delete the album from the list
      gAlbums.value = gAlbums.value.filter(album => album.id !== gAlbumId.value);
      gAlbumId.value = null;

      console.log('Delete album...', result);
    } else {
      console.error('No album selected', gAlbumId.value);
    }
  } catch (error) {
    console.error('Failed to delete album:', error);
  }
};


/// Refresh albums
// const clickRefresh = async () => {
//   await getAlbums(); // Refresh albums

//   gAlbumId.value = null;
//   gFolderId.value = null;

//   console.log('Refresh albums');
// };

/// click album icon to expand or collapse next level folders
const clickExpandAlbum = async (album) => {
  console.log('clickExpandAlbum...', album);

  // Toggle album expansion
  album.is_expanded = !album.is_expanded; 

  if (album.is_expanded && !album.children) {
    try {
      const folders = await invoke('expand_folder', { path: album.path, isRecursive: false });
      album.children = folders.children;
    } catch (error) {
      console.error('Error fetching folder tree:', error);
    }
  }
};

/// click a album to expand or collapse next level folders
const clickAlbum = async (album) => {
  console.log('clickAlbum...', album);

  if (gAlbumId.value != album.id) {
    gAlbumId.value = album.id;
    gFolderId.value = null;
  }
};


/// double click a album to expand all levels sub-folders
const dblclickAlbum = async (album) => {
  console.log('dblclickAlbum...', album);
  clickExpandAlbum(album);
};


/// get children folders
async function getAlbums() {
  try {
    const fetchedAlbums = await invoke('get_albums');
    // console.log('fetchedAlbums...', fetchedAlbums);
    if (fetchedAlbums) {
      gAlbums.value = fetchedAlbums.map(album => ({
        ...album, 
        is_expanded: false,
        children: null,
      }));
    } 
    console.log('getAlbums...', gAlbums.value);

  } catch (error) {
    console.error('Failed to fetch albums:', error);
  }
};


</script>
<template>
    
  <!-- folder pane title -->
  <div class="flex px-1 py-3 items-center justify-between" style="user-select: none;">
    <div>
      {{ titlebar }}
    </div>
    <div class="flex h-6">
      <IconAdd class="p-1 hover:text-gray-200 transition-colors duration-300" @click="clickAdd" />
      <IconDelete  
        :class="[
          'p-1 ', 
          g_album_id ? 'hover:text-gray-200 transition-colors duration-300' : 'text-gray-700'
        ]" 
        @click="showDeleteAlbumMsgbox = true" />
      <!-- <IconRefresh class="p-1 hover:text-gray-200 transition-colors duration-300" @click="clickRefresh"/> -->
    </div>
  </div>

  <!-- Display the fetched albums -->
  <ul v-if="g_albums.length > 0">
    <li v-for="album in g_albums" :key="album.id" style="user-select: none;" >
      <div 
        :class="[
          'p-2 flex items-center whitespace-nowrap hover:bg-gray-700', 
          g_album_id === album.id ? 'text-gray-300 bg-gray-800' : ''
        ]"
        @click="clickAlbum(album)"
        @dblclick="dblclickAlbum(album)"
      >
        <component :is="album.is_expanded ? IconFolderOpen : IconFolder" class="size-6 pr-1 flex-shrink-0" />
        {{ album.name }} - {{ album.id }}
      </div>

      <Folders v-if="album.is_expanded" :album_id="album.id" :children="album.children" />
    </li>
  </ul>

  <!-- Display message if no albums are found -->
  <div v-else class="pt-12 text-center">
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
import IconRefresh from '@/assets/arrow-path.svg';

// folder icon
import IconFolder from '@/assets/folder.svg';
import IconFolderOpen from '@/assets/folder-open.svg';

const g_albums = inject('g_albums');         // global albums
const g_album_id = inject('g_album_id');     // global album id
const g_folder_id = inject('g_folder_id');   // global folder id

const props = defineProps({
  titlebar: {
    type: String,
    required: true
  }
});

const showDeleteAlbumMsgbox = ref(false);

// Fetch albums on mount
onMounted(() => {
  if (g_albums.value.length === 0) {
    getAlbums();
  }
});

const getAlbumById = (id) => g_albums.value.find(album => album.id === id);

/// Add albums
const clickAdd = async () => {
  try {
    const new_album = await invoke('add_album', { window: appWindow, title: localeMessages.value.add_album_title });
    g_albums.value.push(new_album);

    console.log('Add album...', new_album);
  } catch (error) {
    console.error('Failed to add album:', error);
  }
};


/// Delete an album
const clickDeleteConfirm = async () => {
  try {
    if (g_album_id.value) {
      const result = await invoke('delete_album', { id: getAlbumById(g_album_id.value).id });

      // delete the album from the list
      g_albums.value = g_albums.value.filter(album => album.id !== g_album_id.value);
      g_album_id.value = null;

      console.log('Delete album...', result);
    } else {
      console.error('No album selected', g_album_id.value);
    }
  } catch (error) {
    console.error('Failed to delete album:', error);
  }
};


/// Refresh albums
// const clickRefresh = async () => {
//   await getAlbums(); // Refresh albums

//   g_album_id.value = null;
//   g_folder_id.value = null;

//   console.log('Refresh albums');
// };


/// click a album to expand or collapse next level folders
const clickAlbum = async (album) => {
  console.log('clickAlbum...', album);
  try {
    g_album_id.value = album.id;
    g_folder_id.value = null;

    // Toggle album expansion
    album.is_expanded = !album.is_expanded; 
    
    if (album.is_expanded && !album.children) {
      // Fetch folder tree
      const folders = await invoke('read_folders', { path: album.path });
      album.children = folders.children;
    }
  } catch (error) {
    console.error('Error fetching folder tree:', error);
  }
};


/// double click a album to expand all levels sub-folders
const dblclickAlbum = async (album) => {
  console.log('dblclickAlbum...', album);
};


/// get children folders
async function getAlbums() {
  try {
    const fetchedAlbums = await invoke('get_albums');
    // console.log('fetchedAlbums...', fetchedAlbums);
    if (fetchedAlbums) {
      g_albums.value = fetchedAlbums.map(album => ({
        ...album, 
        is_expanded: false,
        children: null,
      }));
    } 
    console.log('getAlbums...', g_albums.value);

  } catch (error) {
    console.error('Failed to fetch albums:', error);
  }
};


</script>
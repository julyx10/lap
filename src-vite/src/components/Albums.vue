<template>
    
  <div class="flex-1 flex flex-col overflow-auto" style="user-select: none;">

    <!-- title bar -->
    <div class="px-2 py-3 h-12 flex items-center justify-between">
      <span>{{ titlebar }}</span>

      <div class="flex">
        <IconAdd class="p-0.5 t-icon-hover" @click="clickAdd" />
        <IconDelete  
          :class="[
            'p-0.5', 
            config.albumId ? 't-icon-hover' : 't-icon-disabled'
          ]" 
          @click="config.albumId ? showDeleteAlbumMsgbox = true : ''" />
        <!-- <IconRefresh class="p-1 hover:text-gray-200 transition-colors duration-300" @click="clickRefresh"/> -->
      </div>
    </div>

    <!-- albums -->
    <div v-if="albums.length > 0" class="flex-grow overflow-auto t-scrollbar-dark">
      <ul>
        <li v-for="album in albums" :key="album.id">
          <div 
            :class="[
              'p-2 flex items-center whitespace-nowrap t-color-bg-hover cursor-pointer', 
              { 
                't-color-text-selected': config.albumId === album.id, 
                't-color-bg-selected': config.albumId === album.id && config.albumFolderId === album.folderId
              }
            ]"
            @click="clickAlbum(album)"
          >
            <component :is="album.is_expanded ? IconFolderOpen : IconFolder" 
              class="size-6 pr-1 flex-shrink-0" 
              @click.stop="clickExpandAlbum(album)"
            />
            {{ album.name }}
            <!-- {{ album.name }}({{ album.id }}) - {{ album.folderId }} -->
          </div>
          <Folders v-if="album.is_expanded" 
            :albumId="album.id"
            :albumName="album.name"
            :albumPath="album.path" 
            :parent="album.folderId" 
            :children="album.children" 
          />
        </li>
      </ul>
    </div>

    <!-- Display message if no albums are found -->
    <div v-else class="mt-10 flex items-center justify-center">
      {{ $t('no_albums') }}
    </div>

    <MessageBox
      v-if="showDeleteAlbumMsgbox"
      :visible="showDeleteAlbumMsgbox"
      :title="$t('delete_album_msgbox_title')"
      :message="`${$t('delete_album_msgbox_content', { album: getAlbumById(config.albumId).name })}`"
      :confirmText="$t('delete_album_msgbox_ok')"
      :cancelText="$t('delete_album_msgbox_cancel')"
      @confirm="clickDeleteConfirm"
      @cancel="showDeleteAlbumMsgbox = false"
      @close="showDeleteAlbumMsgbox = false"
    />
  </div>

</template>


<script setup lang="ts">

import { ref, computed, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { useI18n } from 'vue-i18n';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { useConfigStore } from '@/stores/configStore';
import { separator } from '@/common/utils';

import Folders from '@/components/AlbumsFolders.vue';
import MessageBox from '@/components/MessageBox.vue';

// svg icons
import IconAdd from '@/assets/folder-plus.svg';
import IconDelete from '@/assets/folder-minus.svg';
import IconFolder from '@/assets/folder.svg';
import IconFolderOpen from '@/assets/folder-open.svg';

const props = defineProps({
  titlebar: {
    type: String,
    required: true
  }
});

/// i18n
const { locale, messages } = useI18n();
const localeMsg = computed(() => messages.value[locale.value]);

// config store
const config = useConfigStore();

const albums = ref([]);

const appWindow = getCurrentWebviewWindow();

const showDeleteAlbumMsgbox = ref(false);

// Fetch albums on mount
onMounted( async () => {
  if (albums.value.length === 0) {
    await getAlbums();

    if (config.albumId)
      clickAlbum(getAlbumById(config.albumId));
  }
});

const getAlbumById = (id) => albums.value.find(album => album.id === id);

/// Add albums
const clickAdd = async () => {
  try {
    const new_album = await invoke('add_album', { window: appWindow, title: localeMsg.value.add_album_title });
    albums.value.push(new_album);

    console.log('Add album...', new_album);
  } catch (error) {
    console.log('Failed to add album:', error);
  }
};


/// Delete an album
const clickDeleteConfirm = async () => {
  try {
    if (config.albumId) {
      const result = await invoke('delete_album', { id: getAlbumById(config.albumId).id });

      // delete the album from the list
      albums.value = albums.value.filter(album => album.id !== config.albumId);
      
      config.albumId = null;
      config.albumName = null;
      config.albumPath = null;
      config.albumFolderId = null;
      config.albumFolderName = null;
      config.albumFolderPath = null;

      console.log('Delete album...', result);
    } else {
      console.log('No album selected', config.albumId);
    }
  } catch (error) {
    console.error('Failed to delete album:', error);
  }
};

/// click a album to select it
const clickAlbum = async (album) => {
  try {
    const result = await invoke('select_folder', {
      albumId: album.id, 
      albumPath: album.path,
      parentId: 0,      // 0 is root folder(album)
      folderPath: separator,
    });
    
    console.log('clickAlbum...', album, result);

    // insert a new property(album.folderId) 
    album.folderId = result.id;

    config.albumId = album.id;
    config.albumName = album.name;
    config.albumPath = album.path;
    config.albumFolderId = result.id;
    config.albumFolderName = result.name;
    config.albumFolderPath = result.path;
    
    console.log('add_folder result:', result);
  } catch (error) {
    console.error("Error adding folder:", error);
  }
};


/// click album icon to expand or collapse next level folders
const clickExpandAlbum = async (album) => {
  console.log('clickExpandAlbum...', album);

  clickAlbum(album);

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


/// get children folders
async function getAlbums() {
  try {
    const fetchedAlbums = await invoke('get_albums');
    if (fetchedAlbums) {
      albums.value = fetchedAlbums.map(album => ({
        ...album, 
        is_expanded: false,
        children: null,
      }));
    } 
    console.log('getAlbums...', albums.value);

  } catch (error) {
    console.error('Failed to fetch albums:', error);
  }
};


</script>
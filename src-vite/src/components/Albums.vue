<template>
    
  <div class="flex-1 flex flex-col overflow-auto" style="user-select: none;">

    <!-- title bar -->
    <div class="px-2 py-3 h-12 flex items-center justify-between">
      <span>{{ titlebar }}</span>

      <div class="flex">
        <IconAdd class="m-1 t-icon-size-sm t-icon-hover" @click="clickAdd" />
        <IconDelete  
          :class="[
            'm-1 t-icon-size-sm', 
            config.albumId ? 't-icon-hover' : 't-icon-disabled'
          ]" 
          @click="config.albumId ? showDeleteAlbumMsgbox = true : ''" />
        <!-- <IconRefresh class="p-1 hover:text-gray-200 transition-colors duration-300" @click="clickRefresh"/> -->
      </div>
    </div>

    <!-- albums -->
    <div v-if="albums.length > 0" class="flex-grow overflow-auto t-scrollbar-dark">

      <!-- drag to change albums' display order -->
      <VueDraggable v-model="albums" @end="onDragEnd">
        <div v-for="album in albums" :key="album.id">
          <div 
            :class="[
              'm-1 h-8 flex items-center border-l-2 border-transparent t-color-bg-hover whitespace-nowrap cursor-pointer', 
              { 
                't-color-text-selected': config.albumId === album.id, 
                't-color-bg-selected t-color-border-selected transition-colors duration-300': config.albumId === album.id && config.albumFolderId === album.folderId
              }
            ]"
            @click="clickAlbum(album)"
            @dblclick="dlbClickAlbum(album)"
          >
            <component :is="album.is_expanded ? IconFolderOpen : IconFolder" 
              class="mx-1 h-5 flex-shrink-0" 
              @click.stop="clickExpandAlbum(album)"
            />
            {{ album.name }}
          </div>
          <AlbumsFolders v-if="album.is_expanded" 
            :albumId="album.id"
            :children="album.children" 
          />
        </div>
      </VueDraggable>

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
import { VueDraggable } from 'vue-draggable-plus'
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { useConfigStore } from '@/stores/configStore';
import { separator } from '@/common/utils';

import AlbumsFolders from '@/components/AlbumsFolders.vue';
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

const appWindow = getCurrentWebviewWindow();
const showDeleteAlbumMsgbox = ref(false);

const albums = ref([]);
const getAlbumById = (id) => albums.value.find(album => album.id === id);


onMounted( async () => {
  if (albums.value.length === 0) {
    await getAlbums();

    if (config.albumId) {
      let album = getAlbumById(config.albumId);
      if (album.path === config.albumFolderPath) {  // album is selected
        clickAlbum(album);
      } else {    // album's sub-folder is selected
        clickExpandAlbum(album);

        let relative_folder_path = config.albumFolderPath.replace(album.path, '');
        expandFolderPath(album.id, album, relative_folder_path);
      }
    }
  }
});

/// get children folders
async function getAlbums() {
  try {
    const fetchedAlbums = await invoke('get_all_albums');
    if (fetchedAlbums) {
      albums.value = fetchedAlbums.map(album => ({
        ...album, 
        is_expanded: false,
        children: null,
      }));
    } 
    console.log('getAlbums...', albums.value);

  } catch (error) {
    console.error('getAlbums...', error);
  }
};

/// change albums' display order
const onDragEnd = async () => {
  for (let i = 0; i < albums.value.length; i++) {
    await invoke('set_album_display_order', { id: albums.value[i].id, displayOrder: i });
  }
}

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
      parentId: 0,      // 0 is root folder(album)
      folderPath: album.path,
    });

    // update config
    config.albumId = album.id;
    config.albumFolderId = result.id;
    config.albumFolderName = result.name;
    config.albumFolderPath = result.path;

    // insert a new property(album.folderId) 
    album.folderId = config.albumFolderId;
    console.log('clickAlbum...', album, result);
  } catch (error) {
    console.error("clickAlbum...", error);
  }
};

const dlbClickAlbum = async (album) => {
  clickAlbum(album);
  clickExpandAlbum(album);
};

/// click folder to select
const clickFolder = async (albumId, folder) => {
  try {
    const result = await invoke('select_folder', {
      albumId: albumId,
      parentId: 0,
      folderPath: folder.path,
    });

    // update config
    config.albumId = albumId;
    config.albumFolderId   = result.id;
    config.albumFolderName = result.name;
    config.albumFolderPath = result.path;

    // insert new property 'id' to folder object
    folder.id = config.albumFolderId;

    console.log('clickFolder:', result);
  } catch (error) {
    console.error("Error adding folder:", error);
  }
};

/// click album icon to expand or collapse next level folders
const clickExpandAlbum = async (album) => {
  // clickAlbum(album);
  
  album.is_expanded = !album.is_expanded; 
  
  if (album.is_expanded && !album.children) {
    try {
      const subfolders = await invoke('expand_folder', { path: album.path, isRecursive: false });
      album.children = subfolders.children;
    } catch (error) {
      console.error('clickExpandAlbum...', error);
    }
  }
};

/// expand folders along a given path
const expandFolderPath = async (albumId, folder, path) => {
  const pathArray = path.split(separator).filter(Boolean); // Split and remove empty strings
  let currentFolder = folder;

  for (let i = 0; i < pathArray.length; i++) {
    // get sub-folders
    if (!currentFolder.children) {
      try {
        const subFolders = await invoke('expand_folder', { path: currentFolder.path, isRecursive: false });
        currentFolder.children = subFolders.children;
        console.log('expandFolderPath...', currentFolder);
        if(currentFolder.children && currentFolder.children.length > 0) {
          for (let child of currentFolder.children) {
            if(child.name === pathArray[i]) {
              if( i < pathArray.length - 1) {
                child.is_expanded = true;
                currentFolder = child;
                break;
              } else {  // last folder
                clickFolder(albumId, child);
              }
            }
          }
        }
      } catch (error) {
        console.error('clickExpandAlbum...', error);
      }
    }
  }
}

</script>
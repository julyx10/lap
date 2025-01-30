<template>
    
  <div class="flex-1 flex flex-col overflow-auto" style="user-select: none;">

    <!-- title bar -->
    <div class="px-2 py-3 h-12 flex items-center justify-between">
      <span>{{ titlebar }}</span>
      <IconAdd class="m-1 t-icon-size-sm t-icon-hover" @click="clickAdd" />
    </div>

    <!-- albums -->
    <div v-if="albums.length > 0" class="flex-grow overflow-auto t-scrollbar-dark">

      <!-- drag to change albums' display order -->
      <VueDraggable v-model="albums" @end="onDragEnd">
        <div v-for="album in albums" :key="album.id">
          <div 
            :class="[
              'pr-1 m-1 h-8 flex items-center border-l-2 border-transparent t-color-bg-hover whitespace-nowrap cursor-pointer', 
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
            <span class="flex-grow">{{ album.name }}</span>
            <DropDownMenu v-if="config.albumId === album.id && config.albumFolderId === album.folderId"
              :iconMenu="IconMore"
              :menuItems="moreMenuItems"
              :alignRight="true"
            />
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

    <!-- rename album -->
    <MessageBox
      v-if="showRenameAlbumMsgbox"
      :title="$t('msgbox_rename_album_title')"
      :message="`${$t('msgbox_rename_album_content', { album: getAlbumById(config.albumId).name })}`"
      :showInput="true"
      :inputText="getAlbumById(config.albumId).name"
      :OkText="$t('msgbox_rename_album_ok')"
      :cancelText="$t('msgbox_cancel')"
      @ok="clickRenameConfirm"
      @cancel="showRenameAlbumMsgbox = false"
    />

    <!-- new folder -->
    <MessageBox
      v-if="showNewFolderMsgbox"
      :title="$t('msgbox_new_folder_title')"
      :message="`${$t('msgbox_new_folder_content', { album: getAlbumById(config.albumId).name })}`"
      :showInput="true"
      :inputText="''"
      :OkText="$t('msgbox_new_folder_ok')"
      :cancelText="$t('msgbox_cancel')"
      @ok="clickNewFolderConfirm"
      @cancel="showNewFolderMsgbox = false"
    />

    <!-- remove from albums -->
    <MessageBox
      v-if="showRemoveAlbumMsgbox"
      :title="$t('msgbox_remove_album_title')"
      :message="`${$t('msgbox_remove_album_content', { album: getAlbumById(config.albumId).name })}`"
      :OkText="$t('msgbox_remove_album_ok')"
      :cancelText="$t('msgbox_cancel')"
      @ok="clickRemoveConfirm"
      @cancel="showRemoveAlbumMsgbox = false"
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
import { separator, openFileExplorer } from '@/common/utils';

import AlbumsFolders from '@/components/AlbumsFolders.vue';
import DropDownMenu from '@/components/DropDownMenu.vue';
import MessageBox from '@/components/MessageBox.vue';

// svg icons
import IconAdd from '@/assets/plus.svg';
import IconRemove from '@/assets/minus.svg';
import IconFolder from '@/assets/folder.svg';
import IconFolderOpen from '@/assets/folder-open.svg';
import IconNewFolder from '@/assets/folder-plus.svg';
import IconMore from '@/assets/more.svg';
import IconRefresh from '@/assets/refresh.svg';
import IconCopyTo from '@/assets/copy-to.svg';
import IconMoveTo from '@/assets/move-to.svg';
import IconRename from '@/assets/rename.svg';
import IconDelete from '@/assets/trash.svg';

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

// message boxes
const showRenameAlbumMsgbox = ref(false);
const showNewFolderMsgbox = ref(false);
const showRemoveAlbumMsgbox = ref(false);

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

// more menuitems
const moreMenuItems = computed(() => {
  return [
    {
      label: localeMsg.value.menu_item_refresh,
      icon: IconRefresh,
      action: () => {
        dlbClickAlbum(getAlbumById(config.albumId));
      }
    },
    {
      label: localeMsg.value.menu_item_rename,
      icon: IconRename,
      action: () => {
        showRenameAlbumMsgbox.value = true;
      }
    },
    {
      label: "-",   // separator
      action: () => {}
    },
    {
      label: localeMsg.value.menu_item_new_folder,
      icon: IconNewFolder,
      action: () => {
        showNewFolderMsgbox.value = true;
      }
    },
    {
      label: localeMsg.value.menu_item_open_folder,
      // icon: IconOpenFolder,
      action: () => {
        openFileExplorer(config.albumFolderPath);
      }
    },
    {
      label: "-",   // separator
      action: () => {}
    },
    {
      label: localeMsg.value.menu_item_remove_from_album,
      icon: IconRemove,
      action: () => {
        showRemoveAlbumMsgbox.value = true;
      }
    }
  ];
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

/// Rename an album
const clickRenameConfirm = async (value) => {
  try {
    const result = await invoke('rename_album', { id: config.albumId, name: value });
    if(result) {
      console.log('Rename album...', result);
      let album = getAlbumById(config.albumId);
      album.name = value;
      showRenameAlbumMsgbox.value = false;
    }
  } catch (error) {
    console.error('Failed to rename album:', error);
  }
};

/// Create new folder
const clickNewFolderConfirm = async (value) => {
  try {
    showNewFolderMsgbox.value = false;
  } catch (error) {
    console.error('Failed to create new folder:', error);
  }
};

/// Remove an album from the list
const clickRemoveConfirm = async () => {
  try {
    if (config.albumId) {
      const result = await invoke('remove_album', { id: getAlbumById(config.albumId).id });

      // remove the album from the list
      albums.value = albums.value.filter(album => album.id !== config.albumId);
      
      config.albumId = null;
      config.albumFolderId = null;
      config.albumFolderName = null;
      config.albumFolderPath = null;
      console.log('Remove album...', result);

      showRemoveAlbumMsgbox.value = false;
    } else {
      console.log('No album selected', config.albumId);
    }
  } catch (error) {
    console.error('Failed to remove album:', error);
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
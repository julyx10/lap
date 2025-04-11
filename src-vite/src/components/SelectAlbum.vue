<template>
    <!-- albums -->
    <div v-if="albums.length > 0" class="flex-1 overflow-x-hidden overflow-y-auto rounded-lg t-color-bg t-scrollbar-dark">

      <!-- drag to change albums' display order -->
      <VueDraggable v-model="albums" :disabled="componentId === 1" :handle="'.drag-handle'" @end="onDragEnd">
        <div v-for="album in albums" :key="album.id">
          <div 
            :class="[
              'my-1 mx-1 pr-2 h-8 flex items-center rounded border-l-2 border-transparent t-color-bg-hover whitespace-nowrap cursor-pointer group drag-handle', 
              { 
                't-color-text-selected': selectedAlbumId === album.id, 
                't-color-bg-selected t-color-border-selected transition-colors duration-300': selectedAlbumId === album.id && selectedFolderId === album.folderId
              }
            ]"
            @click="clickAlbum(album)"
            @dblclick="dlbClickAlbum(album)"
          >
            <component :is="album.is_expanded ? IconFolderExpanded : IconFolder" 
              class="mx-1 h-5 flex-shrink-0" 
              @click.stop="clickExpandAlbum(album)"
            />
            <div class="overflow-hidden whitespace-pre text-ellipsis">
              {{ album.name }}
            </div>
            <DropDownMenu v-if="componentId === 0"
              class="ml-auto pl-1 hidden group-hover:block t-color-bg-selected"
              :iconMenu="IconMore"
              :menuItems="moreMenuItems"
              />
          </div>
          <SelectFolder v-if="album.is_expanded"
            ref="selectFolderRef"
            :children="album.children" 
            :rootAlbumId="album.id"
            :albumId="selectedAlbumId"
            :folderId="selectedFolderId"
            :folderPath="selectedFolderPath"
            :componentId="componentId"
          />
        </div>
      </VueDraggable>

    </div>

    <!-- Display message if no albums are found -->
    <div v-else class="mt-10 flex items-center justify-center">
      {{ $t('no_albums') }}
    </div>

    <!-- edit album -->
    <MessageBox
      v-if="showRenameMsgbox"
      :title="$t('msgbox_rename_album_title')"
      :message="$t('msgbox_rename_album_content')"
      :showInput="true"
      :inputText="getAlbumById(albumId).name"
      :OkText="$t('msgbox_rename_album_ok')"
      :cancelText="$t('msgbox_cancel')"
      @ok="clickRenameAlbum"
      @cancel="showRenameMsgbox = false"
    />

    <!-- new folder -->
    <MessageBox
      v-if="showNewFolderMsgbox"
      :title="$t('msgbox_new_folder_title')"
      :message="$t('msgbox_new_folder_content')"
      :showInput="true"
      :inputText="''"
      :OkText="$t('msgbox_new_folder_ok')"
      :cancelText="$t('msgbox_cancel')"
      :errorMessage="errorMessage"
      @ok="clickNewFolder"
      @cancel="showNewFolderMsgbox = false"
      @reset="errorMessage = ''"
    />

    <!-- remove from albums -->
    <MessageBox
      v-if="showRemoveMsgbox"
      :title="$t('msgbox_remove_album_title')"
      :message="`${$t('msgbox_remove_album_content', { album: getAlbumById(albumId).name })}`"
      :OkText="$t('msgbox_remove_album_ok')"
      :cancelText="$t('msgbox_cancel')"
      :warningOk="true"
      @ok="clickRemoveAlbum"
      @cancel="showRemoveMsgbox = false"
    />

</template>

<script setup lang="ts">

import { ref, watch, computed, onMounted } from 'vue';
import { listen } from '@tauri-apps/api/event';
import { useI18n } from 'vue-i18n';
import { VueDraggable } from 'vue-draggable-plus'
import { isMac, separator, openShellFolder } from '@/common/utils';
import { getAllAlbums, setDisplayOrder, addAlbum, renameAlbum, removeAlbum, createFolder, renameFolder, selectFolder, expandFolder } from '@/common/api';

import SelectFolder from '@/components/SelectFolder.vue';
import DropDownMenu from '@/components/DropDownMenu.vue';
import MessageBox from '@/components/MessageBox.vue';

import {
  IconEdit,
  IconRemove,
  IconFolder,
  IconFolderExpanded,
  IconNewFolder,
  IconMore,
} from '@/common/icons';

const props = defineProps({
  albumId: {    // album id
    type: Number,
    required: true
  },
  folderId: {   // folder id
    type: Number,
    required: true
  },
  folderPath: { // folder path
    type: String,
    required: true
  },
  componentId: {     // 0: album pane, 1: move/copy to mode(select destination folder)
    type: Number,
    required: true
  }
});

/// i18n
const { locale, messages } = useI18n();
const localeMsg = computed(() => messages.value[locale.value]);

const selectedAlbumId = ref(0);
const selectedFolderId = ref(0);
const selectedFolderPath = ref('');

const selectFolderRef = ref<SelectFolder | null>(null);

// message boxes
const showRenameMsgbox = ref(false);
const showNewFolderMsgbox = ref(false);
const showRemoveMsgbox = ref(false);
const errorMessage = ref('');

const albums = ref([]);
const getAlbumById = (id) => albums.value.find(album => album.id === id);

const emit = defineEmits(['update:albumId', 'update:folderId', 'update:folderPath']);

// more menuitems
const moreMenuItems = computed(() => {
  return [
    {
      label: localeMsg.value.menu_item_edit,
      icon: IconEdit,
      action: () => {
        showRenameMsgbox.value = true;
      }
    },
    {
      label: localeMsg.value.menu_item_remove_from_album,
      icon: IconRemove,
      action: () => {
        showRemoveMsgbox.value = true;
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
      label: "-",   // separator
      action: () => {}
    },
    {
      label: isMac ? localeMsg.value.menu_item_reveal_in_finder : localeMsg.value.menu_item_reveal_in_file_explorer,
      // icon: IconOpenFolder,
      action: () => {
        openShellFolder(getAlbumById(selectedAlbumId.value).path);
      }
    },


  ];
});

onMounted( async () => {
  if (albums.value.length === 0) {
    albums.value = await getAllAlbums();

    // expand and select the current album and folder
    if (props.albumId > 0) {
      let album = getAlbumById(props.albumId);
      if (album.path === props.folderPath) {  // album is selected
        clickAlbum(album);
      } else {    // album's sub-folder is selected
        clickExpandAlbum(album);

        let relative_folder_path = props.folderPath.replace(album.path, '');
        expandFolderPath(props.albumId, album, relative_folder_path);
      }
    }
  }
});

watch(() => [ props.albumId, props.folderId, props.folderPath ], ([ newAlbumId, newFolderId, newFolderPath ]) => {
  selectedAlbumId.value = newAlbumId;
  selectedFolderId.value = newFolderId;
  selectedFolderPath.value = newFolderPath;
}, { immediate: true });

watch(() => [ selectedAlbumId.value, selectedFolderId.value, selectedFolderPath.value ], ([ newAlbumId, newFolderId, newFolderPath ]) => {
  emitSelectedFolder(newAlbumId, newFolderId, newFolderPath);
});

// update selected album and folder
const emitSelectedFolder = (albumId, folderId, folderPath) => {
  emit('update:albumId', albumId);
  emit('update:folderId', folderId);
  emit('update:folderPath', folderPath);
};

// listen for messages from SelectFolder component
listen('message-from-select-folder', (event) => {
  console.log('message-from-select-folder:', event);
  if(event.payload.componentId === props.componentId) {
    selectedAlbumId.value = event.payload.albumId;
    selectedFolderId.value = event.payload.folderId;
    selectedFolderPath.value = event.payload.folderPath;
  };
});

/// change albums' display order
const onDragEnd = async () => {
  for (let i = 0; i < albums.value.length; i++) {
    await setDisplayOrder(albums.value[i].id, i);
  }
}

/// Add a new album
const clickNewAlbum = async () => {
  const new_album = await addAlbum();
  if(new_album) {
    albums.value.push(new_album);
  }
};

/// Rename an album
const clickRenameAlbum = async (newName) => {
  const renamedAlbum = await renameAlbum(selectedAlbumId.value, newName);
  if(renamedAlbum) {
    let album = getAlbumById(selectedAlbumId.value);
    album.name = newName;
    showRenameMsgbox.value = false;
  }
};

/// Remove an album from the list
const clickRemoveAlbum = async () => {
  const removedAlbum = await removeAlbum(selectedAlbumId.value);
  if(removedAlbum) {
    // remove the album from the list
    albums.value = albums.value.filter(album => album.id !== selectedAlbumId.value);
    showRemoveMsgbox.value = false;

    emitSelectedFolder(0, 0, '');
  }
};

/// click a album to select it
const clickAlbum = async (album) => {
  const selectedFolder = await selectFolder(album.id, 0, album.path); // parentId: 0 is root folder(album)
  if(selectedFolder) {
    // insert a new property(album.folderId) 
    album.folderId = selectedFolder.id;

    emitSelectedFolder(album.id, selectedFolder.id, selectedFolder.path);
  }
};

const dlbClickAlbum = async (album, alwaysExpand = false) => {
  clickAlbum(album);
  clickExpandAlbum(album, alwaysExpand);
};

/// click album icon to expand or collapse next level folders
const clickExpandAlbum = async (album, alwaysExpand = false) => {
  album.is_expanded = alwaysExpand ? alwaysExpand : !album.is_expanded; 
  
  if (album.is_expanded && !album.children) {
    const subFolders = await expandFolder(album.path, false);
    if(subFolders) {
      album.children = subFolders.children;
    }
  }
};

/// Create new folder
const clickNewFolder = async (folderName) => {
  const newFolderPath = await createFolder(selectedFolderPath.value, folderName);
  if(newFolderPath) {
    let album = getAlbumById(selectedAlbumId.value);
    let newFolder = -1;

    if(!album.children) {
      await clickExpandAlbum(album, true);
      newFolder = album.children.find(folder => folder.name === folderName);
    } else {
      album.is_expanded = true;
      album.children.push({ name: folderName, path: newFolderPath });
      newFolder = album.children[album.children.length - 1];
    }

    // select the new folder
    clickFolder(selectedAlbumId.value, newFolder);

    // close the message box
    showNewFolderMsgbox.value = false;

    errorMessage.value = '';
  } else {
    errorMessage.value = localeMsg.value.msgbox_new_folder_error;
  }
};

/// click folder to select
const clickFolder = async (albumId, folder) => {
  console.log('SelectAlbum.vue-clickFolder:', albumId, folder);
  const selectedFolder = await selectFolder(albumId, 0, folder.path); // parentId: 0 is root folder(album)
  if(selectedFolder) {
    selectedAlbumId.value = albumId;
    selectedFolderId.value = selectedFolder.id;
    selectedFolderPath.value = selectedFolder.path;
    // insert new property 'id' to folder object
    folder.id = selectedFolder.id;

    emitSelectedFolder(selectedAlbumId.value, selectedFolderId.value, selectedFolderPath.value);
  }
};

// function scrollToItem(id) {
//   const item = document.getElementById(`folder-${id}`);
//   if (item) {
//     item.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
//   }
// };

/// expand folders along a given path
const expandFolderPath = async (albumId, folder, path) => {
  const pathArray = path.split(separator).filter(Boolean); // Split and remove empty strings
  let currentFolder = folder;

  for (let i = 0; i < pathArray.length; i++) {
    // get sub-folders
    if (!currentFolder.children) {
      const subFolders = await expandFolder(currentFolder.path, false);
      if(subFolders) {
        currentFolder.children = subFolders.children;

        if(currentFolder.children && currentFolder.children.length > 0) {
          for (let child of currentFolder.children) {
            if(child.name === pathArray[i]) {
              if( i < pathArray.length - 1) {
                child.is_expanded = true;
                currentFolder = child;
                break;
              } else {  // last folder
                clickFolder(albumId, child);
                // scrollToItem(child.id);
              }
            }
          }
        }
      }
    }
  }
};

// Expose methods
defineExpose({ 
  clickNewAlbum,
  clickNewFolder
});

</script>

<style scoped>
/* .mask-fade-right {
  mask-image: linear-gradient(to left, transparent 0%, black 24px);
} */
</style>
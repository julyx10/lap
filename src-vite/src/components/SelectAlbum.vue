<template>
    <!-- albums -->
    <ul v-if="albums.length > 0" class="flex-1 overflow-x-hidden overflow-y-auto rounded-lg t-color-bg t-scrollbar-dark">

      <!-- drag to change albums' display order -->
      <VueDraggable 
        v-model="albums" 
        :disabled="componentId === 1"
        group="album-folder"
        :handle="'.drag-handle'" 
        :animation="200"
        @start="onDragStart"
        @end="onDragEnd" 
      >
        <li v-for="album in albums" :key="album.id">
          <div 
            :class="[
              'my-1 mx-1 pr-2 h-8 flex items-center rounded border-l-2 border-transparent whitespace-nowrap cursor-pointer group drag-handle', 
              { 
                't-color-bg-hover': !isDragging,
                't-color-text-selected': selectedAlbumId === album.id, 
                't-color-bg-selected t-color-border-selected ': selectedAlbumId === album.id && selectedFolderId === album.folderId
              }
            ]"
            @click="clickAlbum(album)"
            @dblclick="dlbClickAlbum(album)"
          >
            <component :is="album.is_expanded ? IconFolderExpanded : IconFolder" 
              class="mx-1 h-5 flex-shrink-0" 
              @click.stop="expandAlbum(album)"
            />
            <div class="overflow-hidden whitespace-pre text-ellipsis">
              {{ album.name }}
            </div>
            <DropDownMenu v-if="componentId === 0 && !isDragging"
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
        </li>
      </VueDraggable>

    </ul>

    <!-- Display message if no albums are found -->
    <div v-else-if="!isLoading" class="mt-10 flex items-center justify-center">
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

    <ToolTip ref="toolTipRef" />

</template>

<script setup lang="ts">

import { ref, watch, computed, onMounted, onBeforeUnmount } from 'vue';
import { listen } from '@tauri-apps/api/event';
import { useI18n } from 'vue-i18n';
import { VueDraggable } from 'vue-draggable-plus'
import { config, isMac, separator, openShellFolder } from '@/common/utils';
import { getAllAlbums, setDisplayOrder, addAlbum, renameAlbum, removeAlbum, createFolder, renameFolder, selectFolder, fetchFolder } from '@/common/api';

import SelectFolder from '@/components/SelectFolder.vue';
import DropDownMenu from '@/components/DropDownMenu.vue';
import MessageBox from '@/components/MessageBox.vue';
import ToolTip from '@/components/ToolTip.vue';

import {
  IconEdit,
  IconRemove,
  IconFolder,
  IconFolderExpanded,
  IconNewFolder,
  IconMore,
  IconRefresh,
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

let unlisten: () => void;

const selectedAlbumId = ref(0);
const selectedFolderId = ref(0);
const selectedFolderPath = ref('');

const selectFolderRef = ref<SelectFolder | null>(null);

// message boxes
const showRenameMsgbox = ref(false);
const showNewFolderMsgbox = ref(false);
const showRemoveMsgbox = ref(false);
const errorMessage = ref('');

const toolTipRef = ref(null);

const albums = ref([]);
const isLoading = ref(true);  // loading albums, set to true when albums are loading
const isDragging = ref(false);  // dragging albums, set to true when dragging albums

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
      label: isMac ? localeMsg.value.menu_item_reveal_in_finder : localeMsg.value.menu_item_reveal_in_file_explorer,
      // icon: IconOpenFolder,
      action: () => {
        openShellFolder(getAlbumById(selectedAlbumId.value).path);
      }
    },
    {
      label: "-",   // separator
      action: () => {}
    },
    {
      label: localeMsg.value.menu_item_refresh,
      icon: IconRefresh,
      action: async() => {
        const album = getAlbumById(selectedAlbumId.value);
        await expandAlbum(album, true);
      }
    }
  ];
});

onMounted( async () => {
  if (albums.value.length === 0) {
    albums.value = await getAllAlbums();
    isLoading.value = false;

    // expand and select the current album and folder
    if (props.albumId > 0) {
      let album = getAlbumById(props.albumId);

      if (album.path === props.folderPath) {  // album is selected
        clickAlbum(album);
      } else {    // album's sub-folder is selected
        clickFinalSubFolder(album, props.folderPath);
      }
    }
  }

  // listen for messages from SelectFolder component
  unlisten = await listen('message-from-select-folder', async(event) => {
    console.log('listen - message-from-select-folder:', event);
    const { message } = event.payload;
    switch (message) {
      case 'click-folder':
      case 'rename-folder':
      case 'delete-folder':
        if(event.payload.componentId === props.componentId) {
          selectedAlbumId.value = event.payload.albumId;
          selectedFolderId.value = event.payload.folderId;
          selectedFolderPath.value = event.payload.folderPath;
        };
        break;
      case 'refresh-folder':
        for (let album of albums.value) {
          if(album.id === config.destAlbumId) {
            await expandAlbum(album, true);

            // click to select the folder
            selectedAlbumId.value = config.destAlbumId;
            let folder = album.children.find(child => child.path === event.payload.folderPath);
            if(folder) {
              clickFolder(folder);
            }
            break;
          }
        }
        break;
      default:
        break;
    }
  });
});

onBeforeUnmount(() => {
  unlisten(); // Removes the listener
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

/// dlb click album to select it and expand/collapse its folders
const dlbClickAlbum = async (album) => {
  clickAlbum(album);
  expandAlbum(album);
};

/// click album icon to expand or collapse next level folders
const expandAlbum = async (album, forceRefresh = false) => {
  album.is_expanded = forceRefresh ? true : !album.is_expanded; 
  
  if (album.is_expanded && (!album.children || forceRefresh)) {
    const subFolders = await fetchFolder(album.path, false);
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
      await expandAlbum(album);
      newFolder = album.children.find(folder => folder.name === folderName);
    } else {
      album.is_expanded = true;
      album.children.push({ name: folderName, path: newFolderPath });
      newFolder = album.children[album.children.length - 1];
    }

    // select the new folder
    clickFolder(newFolder);

    // close the message box
    showNewFolderMsgbox.value = false;

    errorMessage.value = '';
  } else {
    errorMessage.value = localeMsg.value.msgbox_new_folder_error;
  }
};

/// click folder to select
const clickFolder = async (folder) => {
  console.log('SelectAlbum.vue-clickFolder:', folder);
  const selectedFolder = await selectFolder(selectedAlbumId.value, 0, folder.path); // parentId: 0 is root folder(album)
  if(selectedFolder) {
    // selectedAlbumId.value = selectedAlbumId.value;
    selectedFolderId.value = selectedFolder.id;
    selectedFolderPath.value = selectedFolder.path;
    // insert new property 'id' to folder object
    folder.id = selectedFolder.id;

    emitSelectedFolder(selectedAlbumId.value, selectedFolderId.value, selectedFolderPath.value);
  } else {
    toolTipRef.value.showTip(localeMsg.value.msgbox_select_folder_error);
  }
};

/// click the final sub-folder to select it
const clickFinalSubFolder = async (album, folderPath) => {
  console.log('clickFinalSubFolder:', album, folderPath);
  // expand the album's folder
  expandAlbum(album, true);

  // recursively expand the final sub-folder path
  let relative_folder_path = folderPath.replace(album.path, '');
  expandSubFolder(album, relative_folder_path);
};

/// expand sub-folders along a given path
const expandSubFolder = async (folder, path) => {
  const pathArray = path.split(separator).filter(Boolean); // Split and remove empty strings
  let currentFolder = folder;

  for (let i = 0; i < pathArray.length; i++) {
    // fetch sub-folders
    const subFolders = await fetchFolder(currentFolder.path, false);
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
              await clickFolder(child);
              scrollToItem(child.id);
            }
          }
        }
      }
    }
  }
};

// scroll to the selected folder
function scrollToItem(id) {
  console.log('scrollToItem:', id);
  const item = document.getElementById(`folder-${id}`);
  if (item) {
    item.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
  }
};

/// drag albums to change their display order
const onDragStart = () => {
  isDragging.value = true;
};

const onDragEnd = async () => {
  isDragging.value = false;
  
  // update the display order of albums
  for (let i = 0; i < albums.value.length; i++) {
    await setDisplayOrder(albums.value[i].id, i);
  }
}

// Expose methods
defineExpose({ 
  clickNewAlbum,
});

</script>

<style scoped>
/* .mask-fade-right {
  mask-image: linear-gradient(to left, transparent 0%, black 24px);
} */
</style>
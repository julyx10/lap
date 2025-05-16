<template>
  <ul v-if="children && children.length > 0">
    <li v-for="child in children"
      :key="child.id" 
      :id="'folder-' + child.id" 
      class="pl-4"
    >
      <div v-if="!child.is_deleted" 
        :class="[
          'my-1 mr-1 h-6 flex items-center t-color-bg-hover rounded border-l-2 border-transparent whitespace-nowrap cursor-pointer group', 
          {
            't-color-text-selected': config.albumFolderPath.includes(child.path),
            't-color-bg-selected t-color-border-selected': selectedFolderId === child.id
          }
        ]" 
        @click="clickFolder(rootAlbumId, child)"
        @dblclick="expandFolder(child)"
      >
        <span v-if="child.children && child.children.length == 0" class="shrink-0 t-icon-size"></span>
        <IconRight v-else
          :class="[
            'p-1 t-icon-size shrink-0 transition-transform', 
            child.is_expanded && child.children && child.children.length > 0 ? 'rotate-90' : ''
          ]"
          @click.stop="expandFolder(child)"
        />
        <input v-if="isRenamingFolder && selectedFolderId === child.id"
          ref="folderInputRef"
          type="text"
          maxlength="255"
          class="px-1 w-full border t-color-border-selected t-input-color-bg t-input-focus rounded"
          v-model="child.name"
          @keydown.enter = "clickRenameFolder(child.name)"
          @keydown.esc = "handleEscKey($event, child.id)"
          @blur = "clickRenameFolder(child.name)"
        > 
        <template v-else>
          <div class="overflow-hidden whitespace-pre text-ellipsis">
            {{ child.name }}
          </div>
          <div class="px-1 ml-auto flex flex-row items-center rounded">
            <IconFavorite v-if="child.is_favorite" 
              class="t-icon-size-sm t-color-text" 
            />
            <DropDownMenu v-show="componentId === 0 && !isRenamingFolder"
              :class="[
                selectedFolderId != child.id ? 'hidden group-hover:block' : ''
              ]"
              :iconMenu="IconMore"
              :menuItems="moreMenuItems"
            />
          </div>
        </template>
      </div>
      <AlbumFolder v-if="child.is_expanded && !child.is_deleted" 
        :key="child.id"
        :children="child.children" 
        :rootAlbumId="rootAlbumId"
        :albumId="selectedAlbumId"
        :folderId="selectedFolderId"
        :folderPath="selectedFolderPath"
        :componentId="componentId"
      />
    </li>
  </ul>

  <!-- new folder -->
  <MessageBox
    v-if="showNewFolderMsgbox"
    :title="$t('msgbox_new_folder_title')"
    :message="$t('msgbox_new_folder_content')"
    :showInput="true"
    :inputText="''"
    :needValidateInput="true"
    :OkText="$t('msgbox_new_folder_ok')"
    :cancelText="$t('msgbox_cancel')"
    @ok="clickNewFolder"
    @cancel="showNewFolderMsgbox = false"
  />

  <!-- delete folder -->
  <MessageBox
    v-if="showDeleteMsgbox"
    :title="$t('msgbox_delete_folder_title')"
    :message="`${$t('msgbox_delete_folder_content', { folder: getFolderById(selectedFolderId).name })}`"
    :OkText="$t('msgbox_delete_folder_ok')"
    :cancelText="$t('msgbox_cancel')"
    :warningOk="true"
    @ok="clickDeleteFolder"
    @cancel="showDeleteMsgbox = false"
  />

  <!-- move to -->
  <MoveTo
    v-if="showMoveTo"
    :title="`${$t('msgbox_move_to_title', { source: shortenFilename(getFolderById(selectedFolderId).name) })}`"
    :message="$t('msgbox_move_to_content')"
    :OkText="$t('msgbox_move_to_ok')"
    :cancelText="$t('msgbox_cancel')"
    @ok="clickMoveTo"
    @cancel="showMoveTo = false"
  />

  <!-- copy to -->
  <MoveTo
    v-if="showCopyTo"
    :title="`${$t('msgbox_copy_to_title', { source: shortenFilename(getFolderById(selectedFolderId).name) })}`"
    :message="$t('msgbox_copy_to_content')"
    :OkText="$t('msgbox_copy_to_ok')"
    :cancelText="$t('msgbox_cancel')"
    @ok="clickCopyTo"
    @cancel="showCopyTo = false"
  />

  <ToolTip ref="toolTipRef" />

</template>

<script setup lang="ts">

import { ref, watch, nextTick, computed } from 'vue';
import { emit } from '@tauri-apps/api/event';
import { useI18n } from 'vue-i18n';
import { config, isMac, shortenFilename, isValidFileName, scrollToFolder } from '@/common/utils';
import { createFolder, renameFolder, deleteFolder, selectFolder, fetchFolder, moveFolder, copyFolder, setFolderFavorite, revealFolder } from '@/common/api';

import AlbumFolder from '@/components/AlbumFolder.vue';
import DropDownMenu from '@/components/DropDownMenu.vue';
import MoveTo from '@/components/MoveTo.vue';
import MessageBox from '@/components/MessageBox.vue';
import ToolTip from '@/components/ToolTip.vue';

import {
  IconRight,
  IconMore,
  IconNewFolder,
  IconRename,
  IconMoveTo,
  IconDelete,
  IconFavorite,
  IconUnFavorite,
  IconRefresh,
} from '@/common/icons';

const props = defineProps({
  children: {       // subfolders
    type: Array,
    required: false,
  },
  rootAlbumId: {    // root album id
    type: Number, 
    required: true,
  },
  albumId: {    // selected album id (v-model value)
    type: Number, 
    required: true,
  },
  folderId: {   // selected folder id (v-model value)
    type: Number, 
    required: true,
  },
  folderPath: { // selected folder path (v-model value)
    type: String, 
    required: true,
  },
  componentId: {  // 0: album pane, 1: move/copy to mode(select destination folder)
    type: Number,
    default: false,
  }
});

/// i18n
const { locale, messages } = useI18n();
const localeMsg = computed(() => messages.value[locale.value]);

const getFolderById = (id) => props.children.find(child => child.id === id);

// current selected folder
const selectedAlbumId = ref(0);
const selectedFolderId = ref(0);
const selectedFolderPath = ref('');

// rename folder
const isRenamingFolder = ref(false);
const folderInputRef = ref([]);     // input text box ref
const originalFolderName = ref(''); // restore original folder name when cancel renaming(press ESC)

// message boxes
const showNewFolderMsgbox = ref(false);
const showDeleteMsgbox = ref(false);
const showMoveTo = ref(false);
const showCopyTo = ref(false);

const toolTipRef = ref(null);

// more menuitems
const moreMenuItems = computed(() => {
  const folder = getFolderById(selectedFolderId.value);

  return [
    {
      label: localeMsg.value.menu_item_new_folder,
      icon: IconNewFolder,
      action: () => {
        showNewFolderMsgbox.value = true; // show new folder message box
      }
    },
    {
      label: localeMsg.value.menu_item_rename,
      icon: IconRename,
      action: () => {
        isRenamingFolder.value = true;
        originalFolderName.value = folder.name;
        nextTick(() => {
          if (folderInputRef.value) {
            folderInputRef.value[0].focus();    // array of input elements
          }
        });
      }
    },
    {
      label: localeMsg.value.menu_item_move_to,
      icon: IconMoveTo,
      action: () => {
        showMoveTo.value = true;  // show move-to message box
      }
    },
    {
      label: localeMsg.value.menu_item_copy_to,
      // icon: IconCopyTo,
      action: () => {
        showCopyTo.value = true;  // show copy-to message box
      }
    },
    {
      label: localeMsg.value.menu_item_delete,
      icon: IconDelete,
      action: () => {
        showDeleteMsgbox.value = true;  // show delete message box
      }
    },
    {
      label: isMac ? localeMsg.value.menu_item_reveal_in_finder : localeMsg.value.menu_item_reveal_in_file_explorer,
      // icon: IconOpenFolder,
      action: () => {
        revealFolder(selectedFolderPath.value);
      }
    },
    {
      label: "-",   // separator
      action: null
    },
    {
      label: !folder?.is_favorite ? localeMsg.value.menu_item_favorite: localeMsg.value.menu_item_unfavorite,
      icon: !folder?.is_favorite ? IconFavorite : IconUnFavorite,
      action: () => {
        toggleFavorite();
      }
    },
    {
      label: "-",   // separator
      action: null
    },
    {
      label: localeMsg.value.menu_item_refresh,
      icon: IconRefresh,
      action: async() => {
        const folder = getFolderById(selectedFolderId.value);
        await expandFolder(folder, true); // force refresh
      }
    }
  ];
});

watch(() => [ props.albumId, props.folderId, props.folderPath ], ([ newAlbumId, newFolderId, newFolderPath ]) => {
  selectedAlbumId.value = newAlbumId;
  selectedFolderId.value = newFolderId;
  selectedFolderPath.value = newFolderPath;
}, { immediate: true });

watch(() => selectedFolderId.value, (newFolderId, oldFolderId) => {
  if (newFolderId && isRenamingFolder.value) {
    handleEscKey(new Event('keydown'), oldFolderId);
  }
});

/// click folder to select
const clickFolder = async (albumId, folder) => {
  console.log('AlbumFolder.vue-clickFolder:', albumId, folder);
  const selectedFolder = await selectFolder(albumId, 0, folder.path); // parentId: 0 is root folder(album)
  if (selectedFolder) {
    selectedAlbumId.value = albumId;
    selectedFolderId.value = selectedFolder.id;
    selectedFolderPath.value = selectedFolder.path;

    // insert new property 'id' to folder object
    folder.id = selectedFolder.id;
    
    emit('message-from-select-folder', { 
      message: 'click-folder',
      albumId: selectedAlbumId.value, 
      folderId: selectedFolderId.value, 
      folderPath: selectedFolderPath.value,
      componentId: props.componentId
    });
  } else {
    toolTipRef.value.showTip(localeMsg.value.msgbox_select_folder_error);
  }
};

/// click expand icon to toggle folder expansion
const expandFolder = async (folder, forceRefresh = false) => {
  folder.is_expanded = forceRefresh ? true : !folder.is_expanded;

  if (folder.is_expanded && (!folder.children || forceRefresh)) {
    const subFolders = await fetchFolder(folder.path, false);
    if (subFolders) {
      folder.children = subFolders.children;
    }
  }
};

/// Create new folder
const clickNewFolder = async (newFolderName) => {
  const newFolderPath = await createFolder(selectedFolderPath.value, newFolderName);
  if(newFolderPath) {
    let folder = getFolderById(selectedFolderId.value);
    if (!folder.children) folder.children = [];
    folder.children.push({ name: newFolderName, path: newFolderPath });
    showNewFolderMsgbox.value = false;

    expandFolder(folder, true).then(() => {
      let folderId = folder.children[folder.children.length - 1].id;
      console
      clickFolder(selectedAlbumId.value, folderId).then(() => {
        scrollToFolder(folderId);
      });
    });
  } else {
    console.log('AlbumFolder.vue-clickNewFolder', localeMsg.value.msgbox_new_folder_error);
    toolTipRef.value.showTip(localeMsg.value.msgbox_new_folder_error);
  }
};

/// Rename folder
const clickRenameFolder = async (newFolderName) => {
  // verfify new folder name is valid
  if (!newFolderName || newFolderName.trim().length === 0 || !isValidFileName(newFolderName)) {
    console.log('AlbumFolder.vue-clickRenameFolder: invalid folder name');
    return;
  }
  if (newFolderName === originalFolderName.value) {
    isRenamingFolder.value = false;   // no change
  } else {
    const newFolderPath = await renameFolder(selectedFolderPath.value, newFolderName);
    if(newFolderPath) {    // rename success
      let folder = getFolderById(selectedFolderId.value);
      folder.name = newFolderName;
      updateFolderPath(folder, selectedFolderPath.value, newFolderPath);

      // update selected folder path
      selectedFolderPath.value = newFolderPath;
      emit('message-from-select-folder', { 
        message: 'rename-folder', 
        albumId: selectedAlbumId.value, 
        folderId: selectedFolderId.value, 
        folderPath: selectedFolderPath.value,
        componentId: props.componentId
      });
      isRenamingFolder.value = false;
    }
  }
};

/// rename folder path and children paths
function updateFolderPath(folder, oldpath, newPath) {
    folder.path = newPath + folder.path.slice(oldpath.length);

    if (folder.children) {
        folder.children.forEach(child => {
            updateFolderPath(child, oldpath, newPath); // recursive
        });
    }
}

/// handle ESC key to cancel renaming folder
const handleEscKey = (event, folderID) => {
  event.preventDefault();

  let folder = getFolderById(folderID);
  folder.name = originalFolderName.value;

  isRenamingFolder.value = false; 
};

/// delete selected folder
const clickDeleteFolder = async () => {
  console.log('AlbumFolder.vue-clickDeleteFolder:', selectedFolderId.value);
  const isDeleted = await deleteFolder(selectedFolderPath.value);
  if (isDeleted) {
    let folder = getFolderById(selectedFolderId.value);
    folder.is_deleted = true;
    folder.id = 0; // remove id to avoid click folder again

    emit('message-from-select-folder', {
      message: 'delete-folder',
      albumId: selectedAlbumId.value, 
      folderId: 0, 
      folderPath: "",
      componentId: props.componentId
    });
    showDeleteMsgbox.value = false;
  } else {
    console.log('AlbumFolder.vue-clickDeleteFolder', localeMsg.value.msgbox_delete_folder_error);
    toolTipRef.value.showTip(localeMsg.value.msgbox_delete_folder_error);
  }
};

// move folder to dest folder
const clickMoveTo = async () => {
  try {
    console.log('AlbumFolder.vue-clickMoveTo:', selectedFolderPath.value, config.destAlbumId, config.destFolderPath);
    const newPath = await moveFolder(selectedFolderPath.value, config.destAlbumId, config.destFolderPath);
    if (newPath) {
      console.log('AlbumFolder.vue-clickMoveTo: move folder success:', newPath);
      // remove the folder from the current folder
      let folder = getFolderById(selectedFolderId.value);
      folder.is_deleted = true;
      folder.id = 0; // remove id to avoid click folder again
      
      // refresh the dest folder
      emit('message-from-select-folder', { 
        message: 'refresh-folder',
        folderPath: newPath,                   // select new folder
      });

      // close move-to dialog
      showMoveTo.value = false;
    } else {
      toolTipRef.value.showTip(localeMsg.value.msgbox_move_to_error);
    }
  } catch (error) {
    console.error('Failed to move folder:', error);
  }
};

// copy folder to dest folder
const clickCopyTo = async () => {
  try {
    console.log('AlbumFolder.vue-clickCopyTo:', selectedFolderPath.value, config.destFolderPath);
    const newPath = await copyFolder(selectedFolderPath.value, config.destFolderPath);
    if (newPath) {
      showCopyTo.value = false;

      // TODO:update folder after copy-to
    } else {
      toolTipRef.value.showTip(localeMsg.value.msgbox_copy_to_error);
    }
  } catch (error) {
    console.error('Failed to copy folder:', error);
  }
};


// toggle favorite folder
const toggleFavorite = async () => {
  const folder = getFolderById(selectedFolderId.value);
  folder.is_favorite = !folder.is_favorite;

  await setFolderFavorite(folder.id, folder.is_favorite);
};

</script>

<style scoped>
</style>
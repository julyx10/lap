<template>
  <ul v-if="children && children.length > 0">
    <li v-for="child in children"
      :key="child.id" 
      :id="'folder-' + child.id" 
      class="pl-4"
    >
      <div v-if="child.id != 0" 
        :class="[
          'mx-1 p-1 h-8 flex items-center rounded-box whitespace-nowrap cursor-pointer hover:bg-base-100 group',
          selectedFolderId === child.id && !isRenamingFolder ? 'text-primary' : 'hover:text-base-content',
        ]" 
        @click="clickFolder(rootAlbumId, child)"
        @dblclick="expandFolder(child)"
      >
        <span v-if="child.children && child.children.length == 0" class="w-6 shrink-0"></span>
        <IconRight v-else
          :class="[
            'p-1 w-6 h-6 shrink-0 transition-transform', 
            child.is_expanded && child.children && child.children.length > 0 ? 'rotate-90' : ''
          ]"
          @click.stop="expandFolder(child)"
        />
        <input v-if="isRenamingFolder && selectedFolderId === child.id"
          ref="folderInputRef"
          type="text"
          maxlength="255"
          class="input px-1 w-full text-base"
          v-model="child.name"
          @keydown.enter = "clickRenameFolder(child.name)"
          @keydown.esc = "handleEscKey($event, child.id)"
          @blur = "clickRenameFolder(child.name)"
        > 
        <template v-else>
          <div class="overflow-hidden whitespace-pre text-ellipsis">
            {{ child.name }}
          </div>
          <div class="ml-auto flex flex-row items-center text-base-content/30">
            <TButton v-if="child.is_favorite" 
              :icon="IconFavorite"
              :disabled="true"
              :buttonSize="'small'"
            />
            <ContextMenu v-if="componentId === 0 && !isRenamingFolder"
              :class="[
                selectedFolderId != child.id ? 'invisible group-hover:visible' : ''
              ]"
              :iconMenu="IconMore"
              :menuItems="moreMenuItems"
              :smallIcon="true"
            />
          </div>
        </template>
      </div>
      <AlbumFolder v-if="child.is_expanded && child.id != 0" 
        :key="child.id"
        :children="child.children" 
        :rootAlbumId="rootAlbumId"
        :isHiddenAlbum="isHiddenAlbum"
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
    :title="$t('msgbox.new_folder.title')"
    :showInput="true"
    :inputText="''"
    :inputPlaceholder="$t('msgbox.new_folder.placeholder')"
    :needValidateInput="true"
    :OkText="$t('msgbox.new_folder.ok')"
    :cancelText="$t('msgbox.cancel')"
    @ok="clickNewFolder"
    @cancel="showNewFolderMsgbox = false"
  />

  <!-- trash folder -->
  <MessageBox
    v-if="showTrashFolderMsgbox"
    :title="$t('msgbox.trash_folder.title')"
    :message="`${$t('msgbox.trash_folder.content', { folder: getFolderById(selectedFolderId).name })}`"
    :OkText="$t('msgbox.trash_folder.ok')"
    :cancelText="$t('msgbox.cancel')"
    :warningOk="true"
    @ok="clickTrashFolder"
    @cancel="showTrashFolderMsgbox = false"
  />

  <!-- move to -->
  <MoveTo
    v-if="showMoveTo"
    :title="`${$t('msgbox.move_to.title', { source: shortenFilename(getFolderById(selectedFolderId).name, 32) })}`"
    :message="$t('msgbox.move_to.content')"
    :OkText="$t('msgbox.move_to.ok')"
    :cancelText="$t('msgbox.cancel')"
    @ok="clickMoveTo"
    @cancel="showMoveTo = false"
  />

  <!-- copy to -->
  <MoveTo
    v-if="showCopyTo"
    :title="`${$t('msgbox.copy_to.title', { source: shortenFilename(getFolderById(selectedFolderId).name, 32) })}`"
    :message="$t('msgbox.copy_to.content')"
    :OkText="$t('msgbox.copy_to.ok')"
    :cancelText="$t('msgbox.cancel')"
    @ok="clickCopyTo"
    @cancel="showCopyTo = false"
  />

  <ToolTip ref="toolTipRef" />

</template>

<script setup lang="ts">

import { ref, watch, nextTick, computed } from 'vue';
import { emit } from '@tauri-apps/api/event';
import { useI18n } from 'vue-i18n';
import { useUIStore } from '@/stores/uiStore';
import { config } from '@/common/config';
import { isMac, shortenFilename, getFolderPath, isValidFileName, scrollToFolder } from '@/common/utils';
import { createFolder, renameFolder, selectFolder, fetchFolder, moveFolder, copyFolder, setFolderFavorite, revealFolder, deleteFolder } from '@/common/api';

import AlbumFolder from '@/components/AlbumFolder.vue';
import ContextMenu from '@/components/ContextMenu.vue';
import MoveTo from '@/components/MoveTo.vue';
import MessageBox from '@/components/MessageBox.vue';
import ToolTip from '@/components/ToolTip.vue';
import TButton from '@/components/TButton.vue';

import {
  IconRight,
  IconMore,
  IconNewFolder,
  IconRename,
  IconMoveTo,
  IconTrash,
  IconFavorite,
  IconUnFavorite,
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
  isHiddenAlbum: {    // is hidden album
    type: Boolean,
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
const uiStore = useUIStore();

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
const showTrashFolderMsgbox = ref(false);
const showMoveTo = ref(false);
const showCopyTo = ref(false);

const toolTipRef = ref(null);

// more menuitems
const moreMenuItems = computed(() => {
  const folder = getFolderById(selectedFolderId.value);

  return [
    {
      label: localeMsg.value.menu.file.new_folder,
      icon: IconNewFolder,
      action: () => {
        showNewFolderMsgbox.value = true; // show new folder message box
      }
    },
    {
      label: localeMsg.value.menu.file.rename,
      icon: IconRename,
      action: () => {
        isRenamingFolder.value = true;
        originalFolderName.value = folder.name;
        uiStore.pushInputHandler('AlbumFolder-rename');
        nextTick(() => {
          if (folderInputRef.value) {
            folderInputRef.value[0].focus();    // array of input elements
          }
        });
      }
    },
    {
      label: localeMsg.value.menu.file.move_to,
      icon: IconMoveTo,
      action: () => {
        showMoveTo.value = true;  // show move-to message box
      }
    },
    {
      label: localeMsg.value.menu.file.copy_to,
      // icon: IconCopyTo,
      action: () => {
        showCopyTo.value = true;  // show copy-to message box
      }
    },
    {
      label: isMac ? localeMsg.value.menu.file.move_to_trash : localeMsg.value.menu.file.delete,
      icon: IconTrash,
      action: () => {
        showTrashFolderMsgbox.value = true;
      }
    },
    // {
    //   label: isMac ? localeMsg.value.menu.file.reveal_in_finder : localeMsg.value.menu.file.reveal_in_file_explorer,
    //   // icon: IconOpenFolder,
    //   action: () => {
    //     revealFolder(selectedFolderPath.value);
    //   }
    // },
    {
      label: "-",   // separator
      action: null
    },
    {
      label: !folder?.is_favorite ? localeMsg.value.menu.meta.favorite: localeMsg.value.menu.meta.unfavorite,
      icon: !folder?.is_favorite ? IconFavorite : IconUnFavorite,
      action: () => {
        toggleFavorite();
      }
    },
    // {
    //   label: "-",   // separator
    //   action: null
    // },
    // {
    //   label: localeMsg.value.menu.refresh,
    //   icon: IconRefresh,
    //   action: async() => {
    //     const folder = getFolderById(selectedFolderId.value);
    //     await expandFolder(folder, true); // force refresh
    //   }
    // }
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
  const selectedFolder = await selectFolder(albumId, folder.path);
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
    toolTipRef.value.showTip(localeMsg.value.msgbox.select_folder.error);
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
    console.log('AlbumFolder.vue-clickNewFolder', localeMsg.value.msgbox.new_folder.error);
    toolTipRef.value.showTip(localeMsg.value.msgbox.new_folder.error);
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
    uiStore.popInputHandler();
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
      uiStore.popInputHandler();
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
  uiStore.popInputHandler();
};

// move folder to dest folder
const clickMoveTo = async () => {
  moveFolder(selectedFolderPath.value, config.destFolder.albumId, config.destFolder.folderPath).then((newPath) => {
    if (newPath) {
      // remove the folder from the current folder
      let folder = getFolderById(selectedFolderId.value);
      folder.id = 0; // remove id to avoid click folder again
      
      // refresh the dest folder
      emit('message-from-select-folder', { 
        message: 'refresh-folder',
        folderPath: newPath,                   // select new folder
      });

      // close move-to dialog
      showMoveTo.value = false;
    } else {
      toolTipRef.value.showTip(localeMsg.value.msgbox.move_to.error);
    }
  });
};

// copy folder to dest folder
const clickCopyTo = async () => {
  copyFolder(selectedFolderPath.value, config.destFolder.folderPath).then((newPath) => {
    if (newPath) {
      // close copy-to dialog
      showCopyTo.value = false;
    } else {
      toolTipRef.value.showTip(localeMsg.value.msgbox.copy_to.error);
    }
  });
};

/// trash selected folder
const clickTrashFolder = async () => {
  const isDeleted = await deleteFolder(selectedFolderId.value, selectedFolderPath.value);
  if (isDeleted) {
    let folder = getFolderById(selectedFolderId.value);
    folder.is_deleted = true;
    folder.id = 0; // remove id to avoid click folder again

    emit('message-from-select-folder', {
      message: 'trash-folder',
      albumId: selectedAlbumId.value, 
      folderId: 0, 
      folderPath: "",
      componentId: props.componentId
    });
    showTrashFolderMsgbox.value = false;
  } else {
    console.log('AlbumFolder.vue-clickTrashFolder', localeMsg.value.msgbox.trash_folder.error);
    toolTipRef.value.showTip(localeMsg.value.msgbox.trash_folder.error);
  }
};

// toggle favorite folder
const toggleFavorite = async () => {
  const folder = getFolderById(selectedFolderId.value);
  folder.is_favorite = !folder.is_favorite;

  await setFolderFavorite(folder.id, folder.is_favorite);
};

</script>
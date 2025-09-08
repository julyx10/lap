<template>
  <ul v-if="children && children.length > 0">
    <li v-for="child in children"
      :key="child.id" 
      :id="'folder-' + child.id" 
      class="pl-4"
    >
      <div v-if="child.id != 0" 
        :class="[
          'my-1 mr-1 h-6 flex items-center rounded border-l-2 whitespace-nowrap cursor-pointer group',
          {
            'bg-base-content/10 border-primary text-base-content': selectedFolderId === child.id && !isRenamingFolder,
            'hover:bg-base-content/10 border-transparent': selectedFolderId !== child.id || isRenamingFolder,
            'text-base-content/30': isHiddenAlbum,
            'text-base-content': getFolderPath(config.albumFolderPath).includes(child.path)
          }
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
          class="input px-1 w-full focus:border text-base"
          v-model="child.name"
          @keydown.enter = "clickRenameFolder(child.name)"
          @keydown.esc = "handleEscKey($event, child.id)"
          @blur = "clickRenameFolder(child.name)"
        > 
        <template v-else>
          <div class="overflow-hidden whitespace-pre text-ellipsis">
            {{ child.name }}
          </div>
          <div class="ml-auto flex flex-row items-center rounded">
            <IconFavorite v-if="child.is_favorite" 
              class="h-4 w-4" 
            />
            <DropDownMenu v-show="componentId === 0 && !isRenamingFolder"
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

  <!-- delete folder -->
  <MessageBox
    v-if="showDeleteFolderMsgbox"
    :title="$t('msgbox.delete_folder.title')"
    :message="`${$t('msgbox.delete_folder.content', { folder: getFolderById(selectedFolderId).name })}`"
    :OkText="$t('msgbox.delete_folder.ok')"
    :cancelText="$t('msgbox.cancel')"
    :warningOk="true"
    @ok="clickDeleteFolder"
    @cancel="showDeleteFolderMsgbox = false"
  />

  <!-- move to -->
  <MoveTo
    v-if="showMoveTo"
    :title="`${$t('msgbox.move_to.title', { source: shortenFilename(getFolderById(selectedFolderId).name) })}`"
    :message="$t('msgbox.move_to.content')"
    :OkText="$t('msgbox.move_to.ok')"
    :cancelText="$t('msgbox.cancel')"
    @ok="clickMoveTo"
    @cancel="showMoveTo = false"
  />

  <!-- copy to -->
  <MoveTo
    v-if="showCopyTo"
    :title="`${$t('msgbox.copy_to.title', { source: shortenFilename(getFolderById(selectedFolderId).name) })}`"
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
import { config, isMac, shortenFilename, getFolderPath, isValidFileName, scrollToFolder } from '@/common/utils';
import { createFolder, renameFolder, selectFolder, fetchFolder, moveFolder, copyFolder, setFolderFavorite, revealFolder, deleteFolder } from '@/common/api';

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
const showDeleteFolderMsgbox = ref(false);
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
        showDeleteFolderMsgbox.value = true;
      }
    },
    {
      label: isMac ? localeMsg.value.menu.file.reveal_in_finder : localeMsg.value.menu.file.reveal_in_file_explorer,
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

// move folder to dest folder
const clickMoveTo = async () => {
  moveFolder(selectedFolderPath.value, config.destAlbumId, config.destFolderPath).then((newPath) => {
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
  copyFolder(selectedFolderPath.value, config.destFolderPath).then((newPath) => {
    if (newPath) {
      // close copy-to dialog
      showCopyTo.value = false;
    } else {
      toolTipRef.value.showTip(localeMsg.value.msgbox.copy_to.error);
    }
  });
};

/// delete selected folder
const clickDeleteFolder = async () => {
  console.log('AlbumFolder.vue-clickDeleteFolder:', selectedFolderId.value);
  const isDeleted = await deleteFolder(selectedFolderId.value);
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
    showDeleteFolderMsgbox.value = false;
  } else {
    console.log('AlbumFolder.vue-clickDeleteFolder', localeMsg.value.msgbox.delete_folder.error);
    toolTipRef.value.showTip(localeMsg.value.msgbox.delete_folder.error);
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
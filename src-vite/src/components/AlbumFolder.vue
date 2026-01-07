<template>
  <ul v-if="children && children.length > 0">
    <li v-for="child in children"
      :key="child.id" 
      :id="'folder-' + child.id" 
      :class="{ 'pl-4': child.path !== folderPath }"
    >
      <div v-if="child.id != 0" 
        :class="[
          'mx-1 p-1 h-8 flex items-center rounded-box whitespace-nowrap cursor-pointer group',
          !config.album.selected && config.album.folderPath === child.path && !isRenamingFolder ? 'text-primary bg-base-100 hover:bg-base-100' : 'hover:text-base-content hover:bg-base-100/30',
        ]" 
        @click="clickFolder(albumId, child)"
        @dblclick="expandFolder(child)"
      >
        <span v-if="child.children && child.children.length == 0" class="w-6 shrink-0"></span>
        <component v-else-if="child.path === folderPath"
          :is="child.is_expanded && child.children && child.children.length > 0 ? IconFolderExpanded : IconFolderCollapsed"
          class="mx-1 w-6 h-6 shrink-0"
          @click.stop="expandFolder(child)"
        />
        <IconRight v-else
          :class="[
            'p-1 w-6 h-6 shrink-0 transition-transform', 
            child.is_expanded && child.children && child.children.length > 0 ? 'rotate-90' : ''
          ]"
          @click.stop="expandFolder(child)"
        />
        <input v-if="isRenamingFolder && config.album.folderPath === child.path"
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
                config.album.folderPath != child.path ? 'invisible group-hover:visible' : ''
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
        :isHiddenAlbum="isHiddenAlbum"
        :albumId="albumId"
        :folderPath="folderPath"
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
    :message="`${$t('msgbox.trash_folder.content', { folder: selectedFolder?.name })}`"
    :OkText="$t('msgbox.trash_folder.ok')"
    :cancelText="$t('msgbox.cancel')"
    :warningOk="true"
    @ok="clickTrashFolder"
    @cancel="showTrashFolderMsgbox = false"
  />

  <!-- move to -->
  <MoveTo
    v-if="showMoveTo"
    :title="`${$t('msgbox.move_to.title', { source: shortenFilename(selectedFolder?.name, 32) })}`"
    :message="$t('msgbox.move_to.content')"
    :OkText="$t('msgbox.move_to.ok')"
    :cancelText="$t('msgbox.cancel')"
    @ok="clickMoveTo"
    @cancel="showMoveTo = false"
  />

  <!-- copy to -->
  <MoveTo
    v-if="showCopyTo"
    :title="`${$t('msgbox.copy_to.title', { source: shortenFilename(selectedFolder?.name, 32) })}`"
    :message="$t('msgbox.copy_to.content')"
    :OkText="$t('msgbox.copy_to.ok')"
    :cancelText="$t('msgbox.cancel')"
    @ok="clickCopyTo"
    @cancel="showCopyTo = false"
  />

  <ToolTip ref="toolTipRef" />

</template>

<script setup lang="ts">

import { ref, nextTick, computed } from 'vue';
import { useI18n } from 'vue-i18n';
import { useUIStore } from '@/stores/uiStore';
import { config } from '@/common/config';
import { isMac, shortenFilename, isValidFileName, scrollToFolder } from '@/common/utils';
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
  IconCopyTo,
  IconFolderExpanded,
  IconFolderCollapsed,
} from '@/common/icons';

const props = defineProps({
  children: {       // subfolders
    type: Array,
    required: false,
  },
  isHiddenAlbum: {    // is hidden album
    type: Boolean,
    required: true,
  },
  albumId: {    // selected album id (v-model value)
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
const localeMsg = computed(() => messages.value[locale.value] as any);
const uiStore = useUIStore();

const getFolderById = (id) => props.children.find(child => child.id === id);
const selectedFolder = computed(() => getFolderById(config.album.folderId));

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
  const folder = selectedFolder.value;

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
      icon: IconCopyTo, 
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
    {
      label: isMac ? localeMsg.value.menu.file.reveal_in_finder : localeMsg.value.menu.file.reveal_in_file_explorer,
      // icon: IconOpenFolder,
      action: () => {
        revealFolder(config.album.folderPath);
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
  ];
});

/// click folder to select
const clickFolder = async (albumId: number, folder: any) => {
  console.log('AlbumFolder.vue-clickFolder:', albumId, folder);
  const selectedFolder = await selectFolder(albumId, folder.path);
  if (selectedFolder) {
    config.album.id = albumId;
    config.album.folderId = selectedFolder.id;
    config.album.folderPath = selectedFolder.path;
    config.album.selected = false;
  } else {
    toolTipRef.value.showTip(localeMsg.value.msgbox.select_folder.error);
  }
};

/// click expand icon to toggle folder expansion
const expandFolder = async (folder: any, forceRefresh = false) => {
  folder.is_expanded = forceRefresh ? true : !folder.is_expanded;

  if (folder.is_expanded && (!folder.children || forceRefresh)) {
    const subFolders = await fetchFolder(folder.path, false);
    if (subFolders) {
      folder.children = subFolders.children;
    }
  }
};

/// Create new folder
const clickNewFolder = async (newFolderName: string) => {
  const newFolderPath = await createFolder(config.album.folderPath, newFolderName);
  if(newFolderPath) {
    let folder = selectedFolder.value;
    if (!folder.children) folder.children = [];
    folder.children.push({ name: newFolderName, path: newFolderPath });
    showNewFolderMsgbox.value = false;

    expandFolder(folder, true).then(() => {
      let newFolder = folder.children[folder.children.length - 1];
      clickFolder(props.albumId, newFolder).then(() => {
        scrollToFolder(newFolder.id);
      });
    });
  } else {
    console.log('AlbumFolder.vue-clickNewFolder', localeMsg.value.msgbox.new_folder.error);
    toolTipRef.value.showTip(localeMsg.value.msgbox.new_folder.error);
  }
};

/// Rename folder
const clickRenameFolder = async (newFolderName: string) => {
  // verfify new folder name is valid
  if (!newFolderName || newFolderName.trim().length === 0 || !isValidFileName(newFolderName)) {
    console.log('AlbumFolder.vue-clickRenameFolder: invalid folder name');
    return;
  }
  if (newFolderName === originalFolderName.value) {
    isRenamingFolder.value = false;   // no change
    uiStore.popInputHandler();
  } else {
    const newFolderPath = await renameFolder(config.album.folderPath, newFolderName);
    if(newFolderPath) {    // rename success
      let folder = selectedFolder.value;
      folder.name = newFolderName;
      updateFolderPath(folder, config.album.folderPath, newFolderPath);

      // update selected folder path
      config.album.folderPath = newFolderPath;

      isRenamingFolder.value = false;
      uiStore.popInputHandler();
    }
  }
};

/// rename folder path and children paths
function updateFolderPath(folder: any, oldpath: string, newPath: string) {
    folder.path = newPath + folder.path.slice(oldpath.length);

    if (folder.children) {
        folder.children.forEach(child => {
            updateFolderPath(child, oldpath, newPath); // recursive
        });
    }
}

/// handle ESC key to cancel renaming folder
const handleEscKey = (event: KeyboardEvent, folderID: number) => {
  event.preventDefault();

  let folder = getFolderById(folderID);
  folder.name = originalFolderName.value;

  isRenamingFolder.value = false; 
  uiStore.popInputHandler();
};

// move folder to dest folder
const clickMoveTo = async () => {
  moveFolder(config.album.folderPath, config.destFolder.albumId, config.destFolder.folderPath).then((newPath) => {
    if (newPath) {
      // remove the folder from the current folder
      let folder = selectedFolder.value; 
      folder.id = 0; // remove id to avoid click folder again
      
      // close move-to dialog
      showMoveTo.value = false;
    } else {
      toolTipRef.value.showTip(localeMsg.value.msgbox.move_to.error);
    }
  });
};

// copy folder to dest folder
const clickCopyTo = async () => {
  copyFolder(config.album.folderPath, config.destFolder.folderPath).then((newPath) => {
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
  const isDeleted = await deleteFolder(config.album.folderId, config.album.folderPath);
  if (isDeleted) {
    let folder = selectedFolder.value;
    folder.is_deleted = true;
    folder.id = 0; // remove id to avoid click folder again

    showTrashFolderMsgbox.value = false;
  } else {
    console.log('AlbumFolder.vue-clickTrashFolder', localeMsg.value.msgbox.trash_folder.error);
    toolTipRef.value.showTip(localeMsg.value.msgbox.trash_folder.error);
  }
};

// toggle favorite folder
const toggleFavorite = async () => {
  const folder = selectedFolder.value;
  folder.is_favorite = !folder.is_favorite;

  await setFolderFavorite(folder.id, folder.is_favorite);
};

</script>
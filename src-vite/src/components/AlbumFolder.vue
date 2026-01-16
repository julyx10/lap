<template>
  <ul v-if="children && children.length > 0">
    <li v-for="child in (children as Folder[])"
      :key="child.id" 
      :id="'folder-' + child.id" 
      :class="{ 'pl-4': child.path !== rootPath }"
    >
      <div v-if="child.id != 0 || selection.folderPath.value == rootPath" 
        :class="[
          'mx-1 p-1 h-8 flex items-center rounded-box whitespace-nowrap cursor-pointer group',
          !selection.selected.value && selection.folderPath.value === child.path && !isRenamingFolder ? 'text-primary bg-base-100 hover:bg-base-100' : 'hover:text-base-content hover:bg-base-100/30',
        ]" 
        @click="clickFolder(albumId, child)"
        @dblclick="expandFolder(child)"
      >
        <!-- icon -->
        <component v-if="child.path === rootPath"
          :is="child.is_expanded ? IconFolderExpanded : IconFolderCollapsed"
          class="mx-1 w-6 h-6 shrink-0"
          @click.stop="expandFolder(child)"
        />
        <template v-else>
          <IconRight v-if="!child.children || child.children.length > 0"
            :class="[
              'p-1 w-6 h-6 shrink-0 transition-transform', 
              child.is_expanded ? 'rotate-90' : ''
            ]"
            @click.stop="expandFolder(child)"
          />
          <span v-else class="w-6 shrink-0"></span>
        </template>

        <!-- name -->
        <input v-if="isRenamingFolder && selection.folderPath.value === child.path"
          ref="folderInputRef"
          type="text"
          maxlength="255"
          class="input px-1 w-full text-base"
          v-model="child.name"
          @keydown.enter = "clickRenameFolder(child.name)"
          @keydown.esc = "handleEscKey($event, String(child.id))"
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
            <ContextMenu v-if="isMainPane && !isRenamingFolder"
              :class="[
                selection.folderPath.value != child.path ? 'invisible group-hover:visible' : ''
              ]"
              :iconMenu="IconMore"
              :menuItems="() => getMenuItemsForFolder(child)"
              :smallIcon="true"
            />
          </div>
        </template>
      </div>
      <AlbumFolder v-if="child.is_expanded && child.id != 0" 
        :key="child.id"
        :children="child.children" 
        :albumId="albumId"
        :rootPath="rootPath"
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
import { libConfig } from '@/common/config';
import { isMac, shortenFilename, isValidFileName, scrollToFolder } from '@/common/utils';
import { createFolder, renameFolder, fetchFolder, moveFolder, copyFolder, setFolderFavorite, revealFolder, deleteFolder } from '@/common/api';
import { Folder } from '@/common/types';
import { useAlbumSelection } from '@/composables/useAlbumSelection';

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

const props = defineProps<{
  children?: Folder[];      // subfolders
  albumId: number;          // album id for this folder tree
  rootPath: string;         // root folder path (album path)
}>();

// Inject selection context from AlbumList
const selection = useAlbumSelection();

// Computed to check if we're in main album pane (not MoveTo dialog)
const isMainPane = computed(() => selection.albumId.value !== undefined);

/// i18n
const { locale, messages } = useI18n();
const localeMsg = computed(() => messages.value[locale.value] as any);
const uiStore = useUIStore();

// Recursively find folder by path in the tree
const getFolderByPath = (children: Folder[] | undefined, path: string): Folder | null => {
  if (!children) return null;
  for (const child of children) {
    if (child.path === path) return child;
    if (path.startsWith(child.path)) {
      const found = getFolderByPath(child.children, path);
      if (found) return found;
    }
  }
  return null;
};

const selectedFolder = computed(() => getFolderByPath(props.children, selection.folderPath.value));

// rename folder
const isRenamingFolder = ref(false);
const folderInputRef = ref<HTMLInputElement[]>([]);     // input text box ref
const originalFolderName = ref(''); // restore original folder name when cancel renaming(press ESC)

// message boxes
const showNewFolderMsgbox = ref(false);
const showTrashFolderMsgbox = ref(false);
const showMoveTo = ref(false);
const showCopyTo = ref(false);

const toolTipRef = ref(null);

// more menuitems - function that takes the folder being right-clicked
const getMenuItemsForFolder = (folder: any) => {
  const isRoot = folder.path === props.folderPath;
  return [
    {
      label: localeMsg.value.menu.file.new_folder,
      icon: IconNewFolder,
      action: () => {
        showNewFolderMsgbox.value = true;
      }
    },
    {
      label: localeMsg.value.menu.file.rename,
      icon: IconRename,
      disabled: isRoot,
      action: () => {
        isRenamingFolder.value = true;
        originalFolderName.value = folder.name;
        uiStore.pushInputHandler('AlbumFolder-rename');
        nextTick(() => {
          if (folderInputRef.value) {
            folderInputRef.value[0].focus();
          }
        });
      }
    },
    {
      label: localeMsg.value.menu.file.move_to,
      icon: IconMoveTo,
      disabled: isRoot,
      action: () => {
        showMoveTo.value = true;
      }
    },
    {
      label: localeMsg.value.menu.file.copy_to,
      icon: IconCopyTo, 
      disabled: isRoot,
      action: () => {
        showCopyTo.value = true;
      }
    },
    {
      label: isMac ? localeMsg.value.menu.file.move_to_trash : localeMsg.value.menu.file.delete,
      icon: IconTrash,
      disabled: isRoot,
      action: () => {
        showTrashFolderMsgbox.value = true;
      }
    },
    {
      label: isMac ? localeMsg.value.menu.file.reveal_in_finder : localeMsg.value.menu.file.reveal_in_file_explorer,
      action: () => {
        revealFolder(folder.path);
      }
    },
    {
      label: "-",
      action: null
    },
    {
      label: !folder?.is_favorite ? localeMsg.value.menu.meta.favorite: localeMsg.value.menu.meta.unfavorite,
      icon: !folder?.is_favorite ? IconFavorite : IconUnFavorite,
      disabled: isRoot,
      action: () => {
        toggleFavorite();
      }
    },
  ];
};

/// click folder to select
const clickFolder = async (albumIdVal: number, folder: Folder) => {
  console.log('AlbumFolder.vue-clickFolder:', albumIdVal, folder);
  await selection.selectFolder(albumIdVal, folder);
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
  const newFolderPath = await createFolder(selection.folderPath.value, newFolderName);
  
  if (newFolderPath) {
    showNewFolderMsgbox.value = false;
    
    let folder = selectedFolder.value;
    if (folder) {
      if (!folder.children) folder.children = [];
      folder.children.push({ id: 0, name: newFolderName, path: newFolderPath });

      expandFolder(folder, true).then(() => {
        const newFolder = folder.children?.find((child: Folder) => child.path === newFolderPath);
        if (newFolder) {
          clickFolder(props.albumId, newFolder).then(() => {
            scrollToFolder(newFolder.id);
          });
        }
      });
    }
  } else {
    toolTipRef.value?.showTip(localeMsg.value.msgbox.new_folder.error);
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
    uiStore.removeInputHandler('AlbumFolder-rename');
  } else {
    const newFolderPath_ = await renameFolder(selection.folderPath.value, newFolderName);
    if(newFolderPath_) {    // rename success
      let folder = selectedFolder.value;
      if (folder) {
        folder.name = newFolderName;
        updateFolderPath(folder, selection.folderPath.value, newFolderPath_);
      }

      // update selected folder path
      selection.folderPath.value = newFolderPath_;

      isRenamingFolder.value = false;
      uiStore.removeInputHandler('AlbumFolder-rename');
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
const handleEscKey = (event: KeyboardEvent, folderId: string) => {
  event.preventDefault();

  if (selectedFolder.value) {
    selectedFolder.value.name = originalFolderName.value;
  }

  isRenamingFolder.value = false; 
  uiStore.removeInputHandler('AlbumFolder-rename');
};

// move folder to dest folder
const clickMoveTo = async () => {
  const movedFolderPath = selection.folderPath.value;
  const movedFolder = selectedFolder.value;
  const movedFolderName = movedFolder?.name;
  const destAlbumId = libConfig.destFolder.albumId;
  const destFolderPath = libConfig.destFolder.folderPath;
  
  moveFolder(movedFolderPath, destAlbumId, destFolderPath).then(async (newPath) => {
    if (newPath) {
      // remove the folder from the current folder's children
      if (props.children) {
        const index = (props.children as Folder[]).findIndex((child: Folder) => child.path === movedFolderPath);
        if (index !== -1) {
          (props.children as Folder[]).splice(index, 1);
        }
      }
      
      // close move-to dialog first
      showMoveTo.value = false;
      
      // Use selection context to navigate to the new location
      if (destAlbumId) {
        await selection.expandAndSelectFolder(destAlbumId, newPath);
      }
    } else {
      toolTipRef.value?.showTip(localeMsg.value.msgbox.move_to.error);
    }
  });
};

// copy folder to dest folder
const clickCopyTo = async () => {
  copyFolder(selection.folderPath.value, libConfig.destFolder.folderPath ?? '').then((newPath) => {
    if (newPath) {
      // close copy-to dialog
      showCopyTo.value = false;
    } else {
      toolTipRef.value?.showTip(localeMsg.value.msgbox.copy_to.error);
    }
  });
};

/// trash selected folder
const clickTrashFolder = async () => {
  const isDeleted = await deleteFolder(selection.folderPath.value);
  if (isDeleted) {
    const deletedFolderPath = selection.folderPath.value;
    
    // The deleted folder is a direct child of props.children in this component's context
    // (since the ContextMenu is rendered for each child in the v-for loop)
    // So we can directly remove it from props.children using splice
    if (props.children) {
      const index = (props.children as Folder[]).findIndex((child: Folder) => child.path === deletedFolderPath);
      if (index !== -1) {
        (props.children as Folder[]).splice(index, 1);
      }
    }
    
    // Navigate to parent folder (derive parent path from deleted folder's path)
    const lastSlashIndex = deletedFolderPath.lastIndexOf('/');
    const parentPath = lastSlashIndex > 0 ? deletedFolderPath.substring(0, lastSlashIndex) : props.rootPath;
    selection.folderPath.value = parentPath;
    
    // Try to find parent folder to get its id
    const parentFolder = getFolderByPath(props.children, parentPath);
    if (parentFolder) {
      selection.folderId.value = parentFolder.id;
    }
    
    showTrashFolderMsgbox.value = false;
  } else {
    console.log('AlbumFolder.vue-clickTrashFolder', localeMsg.value.msgbox.trash_folder.error);
    toolTipRef.value?.showTip(localeMsg.value.msgbox.trash_folder.error);
  }
};

// toggle favorite folder
const toggleFavorite = async () => {
  const folder = selectedFolder.value;
  if (!folder || !selection.folderId.value) {
    console.log('AlbumFolder.vue-toggleFavorite: folder or folderId not found');
    return;
  }
  folder.is_favorite = !folder.is_favorite;

  await setFolderFavorite(selection.folderId.value, folder.is_favorite ?? false);
};

</script>
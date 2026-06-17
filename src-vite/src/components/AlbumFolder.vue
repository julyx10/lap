<template>
  <ul
    v-if="children && children.length > 0"
    v-bind="treeRoot ? { tabindex: 0 } : {}"
    ref="treeRootRef"
    :data-folder-tree-root="treeRoot ? 'true' : undefined"
    class="outline-none"
    @keydown="handleLocalTreeKeyDown"
    @mousedown.capture="focusTreeRoot"
  >
    <li v-for="child in (children as Folder[])"
      :key="child.id" 
      :id="'folder-' + child.id" 
      :class="{ 'pl-4': child.path !== rootPath }"
    >
      <div v-if="child.id != 0 || selection.folderPath.value == rootPath"
        :data-file-drop-path="child.path"
        :data-file-drop-album-id="albumId"
        :class="[
          'p-1 h-8 flex items-center rounded-box whitespace-nowrap cursor-pointer group border-2',
          !selection.selected.value && selection.folderPath.value === child.path && !isRenamingFolder ? 'text-primary bg-base-100 hover:bg-base-100 border-transparent' : 'hover:text-base-content hover:bg-base-100/30 border-transparent',
        ]"
        @click="clickFolder(albumId, child)"
        @dblclick="expandFolder(child)"
        @contextmenu.prevent.stop="(e: MouseEvent) => handleFolderContextMenu(child, e)"
      >
        <IconRight
          :class="[
            'p-1 w-6 h-6 shrink-0 transition-transform',
            (!child.children || child.children.length > 0) && !child.is_excluded_from_search ? '' : 'opacity-0 pointer-events-none',
            child.is_expanded ? 'rotate-90' : ''
          ]"
          @click.stop="expandFolder(child)"
        />
        <IconFolder class="p-1 w-6 h-6 shrink-0"/>

        <!-- name -->
        <input v-if="isRenamingFolder && selection.folderPath.value === child.path"
          ref="folderInputRef"
          type="text"
          maxlength="255"
          class="input px-1 w-full text-base"
          v-model="child.name"
          @click.stop
          @mousedown.stop
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
              :icon="IconHeart"
              :disabled="true"
              :buttonSize="'small'"
            />
            <TButton v-if="child.is_excluded_from_search" 
              :icon="IconHide"
              :disabled="true"
              :buttonSize="'small'"
            />
            <ContextMenu v-if="allowContextMenu && !isRenamingFolder"
              :ref="(el: any) => { if (el) folderContextMenus[child.path] = el }"
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
      <AlbumFolder v-if="child.is_expanded && child.id != 0 && !child.is_excluded_from_search"
        :key="child.id"
        :children="child.children" 
        :albumId="albumId"
        :rootPath="rootPath"
        :allowContextMenu="allowContextMenu"
        :treeRoot="false"
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
    :title="trashFolderDialogTitle"
    :message="trashFolderDialogMessage"
    :OkText="trashFolderDialogOkText"
    :cancelText="$t('msgbox.cancel')"
    :warningOk="true"
    :checkboxText="$t('msgbox.permanent_delete.checkbox')"
    :checkboxChecked="deletePermanently"
    @ok="clickTrashFolder"
    @cancel="showTrashFolderMsgbox = false"
    @checkbox-change="deletePermanently = $event"
  />

  <!-- move within library -->
  <MoveTo
    v-if="showMoveTo"
    :title="`${$t('msgbox.move_to.title', { source: shortenFilename(selectedFolder?.name ?? '', 32) })}`"
    :message="$t('msgbox.move_to.content')"
    :OkText="$t('msgbox.move_to.ok')"
    :cancelText="$t('msgbox.cancel')"
    @ok="clickMoveTo"
    @cancel="showMoveTo = false"
  />

  <FileConflictDialog
    v-if="fileConflictDialog.show"
    :name="fileConflictDialog.name"
    :destination="fileConflictDialog.destination"
    :allowReplace="fileConflictDialog.allowReplace"
    @resolve="resolveFileConflict"
  />
</template>

<script setup lang="ts">

import { ref, nextTick, computed } from 'vue';
import { useI18n } from 'vue-i18n';
import { useUIStore } from '@/stores/uiStore';
import { config, libConfig } from '@/common/config';
import { isMac, shortenFilename, isValidFileName, getFolderPath, getFullPath, normalizePathForCompare, isWithinRootPath } from '@/common/utils';
import {
  createFolder, renameFolder, fetchFolder, getAllAlbums, moveFolder, moveFolderOutsideLibrary,
  copyFolder, checkFileExists, revealPath, deleteFolder, deleteFolderPermanently, recountAlbum,
  setFolderFavorite, setFolderSearchExcluded, hasImportableClipboard,
} from '@/common/api';
import { DEFAULT_PLATFORM, getShortcutLabel } from '@/common/shortcuts';
import { Album, Folder } from '@/common/types';
import { useAlbumSelection } from '@/composables/useAlbumSelection';

import AlbumFolder from '@/components/AlbumFolder.vue';
import ContextMenu from '@/components/ContextMenu.vue';
import MoveTo from '@/components/MoveTo.vue';
import MessageBox from '@/components/MessageBox.vue';
import TButton from '@/components/TButton.vue';
import FileConflictDialog from '@/components/FileConflictDialog.vue';
import { useToast } from '@/common/toast';
import { ask, open as openDialog } from '@tauri-apps/plugin-dialog';

import {
  IconRight,
  IconMore,
  IconNewFolder,
  IconRename,
  IconMove,
  IconTrash,
  IconHeart,
  IconHeartFilled,
  IconFolder,
  IconHide,
  IconUnhide,
  IconRefresh,
  IconClipboard
} from '@/common/icons';

// used for cross-component communication (Content.vue listens for this event)
import { emit as tauriEmit } from '@tauri-apps/api/event';

const props = withDefaults(defineProps<{
  children?: Folder[];      // subfolders
  albumId: number;          // album id for this folder tree
  rootPath: string;         // root folder path (album path)
  allowContextMenu?: boolean; // whether to show context menu
  treeRoot?: boolean;       // only root tree listens to keyboard
}>(), {
  treeRoot: true,
});

const emit = defineEmits<{
  rootRenamed: [payload: { albumId: number; newPath: string }];
}>();

// Inject selection context from AlbumList
const selection = useAlbumSelection();

/// i18n
const { locale, messages, t } = useI18n();
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

const trashFolderDialogTitle = computed(() =>
  deletePermanently.value
    ? t('msgbox.permanent_delete.title')
    : t('msgbox.move_to_trash.title')
);
const trashFolderDialogOkText = computed(() =>
  deletePermanently.value
    ? t('msgbox.permanent_delete.ok')
    : t('msgbox.move_to_trash.ok')
);
const trashFolderDialogMessage = computed(() =>
  deletePermanently.value
    ? t('msgbox.permanent_delete.folder_content', { folder: selectedFolder.value?.name || '' })
    : t('msgbox.move_to_trash.folder_content', { folder: selectedFolder.value?.name || '' })
);

// rename folder
const isRenamingFolder = ref(false);
const folderInputRef = ref<HTMLInputElement[]>([]);     // input text box ref
const originalFolderName = ref(''); // restore original folder name when cancel renaming(press ESC)

// message boxes
const showNewFolderMsgbox = ref(false);
const showTrashFolderMsgbox = ref(false);
const showMoveTo = ref(false);
const permanentDeleteChecked = ref(false);
const deletePermanently = ref(false);
const folderContextMenus = ref<Record<string, any>>({});

function handleFolderContextMenu(folder: Folder, event: MouseEvent) {
  clickFolder(props.albumId, folder);
  folderContextMenus.value[folder.path]?.open?.(event.clientX, event.clientY);
}
type FileConflictPolicy = 'skip' | 'keep_both' | 'replace';
const fileConflictDialog = ref({
  show: false,
  name: '',
  destination: '',
  allowReplace: true,
});
let fileConflictResolver: ((policy: FileConflictPolicy) => void) | null = null;

const toast = useToast();
const treeRootRef = ref<HTMLElement | null>(null);

// more menuitems - function that takes the folder being right-clicked
const getMenuItemsForFolder = async (folder: any) => {
  const isRoot = folder.path === props.rootPath;
  const canPaste = await hasImportableClipboard();
  return [
    {
      label: localeMsg.value.menu.file.new_folder,
      icon: IconNewFolder,
      action: () => {
        showNewFolderMsgbox.value = true;
      }
    },
    {
      label: t('menu.file.paste'),
      icon: IconClipboard,
      shortcut: getShortcutLabel('file.paste', DEFAULT_PLATFORM),
      disabled: !canPaste,
      action: () => {
        void tauriEmit('paste-clipboard-to-folder', {
          albumId: props.albumId,
          folderPath: folder.path,
        });
      }
    },
    {
      label: "-",
      action: null
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
            folderInputRef.value[0].focus();
          }
        });
      }
    },
    {
      label: t('menu.file.move_copy'),
      disabled: isRoot,
      children: [
        {
          label: t('menu.file.move_within_library'),
          icon: IconMove,
          disabled: isRoot,
          action: () => {
            showMoveTo.value = true;
          }
        },
        {
          label: t('menu.file.move_to_folder'),
          disabled: isRoot,
          action: () => {
            void clickMoveToFolder();
          }
        },
        {
          label: t('menu.file.copy_to_folder'),
          disabled: isRoot,
          action: () => {
            void clickCopyToFolder();
          }
        },
      ]
    },
    {
      label: isMac ? localeMsg.value.menu.file.reveal_in_finder : localeMsg.value.menu.file.reveal_in_file_explorer,
      action: () => {
        revealPath(folder.path);
      }
    },
    {
      label: "-",
      action: null
    },
    {
      label: !folder?.is_favorite ? localeMsg.value.menu.meta.favorite : localeMsg.value.menu.meta.unfavorite,
      icon: !folder?.is_favorite ? IconHeart : IconHeartFilled,
      action: () => {
        toggleFavorite();
      }
    },
    {
      label: folder?.is_excluded_from_search ? localeMsg.value.menu.album.include_in_search : localeMsg.value.menu.album.exclude_from_search,
      icon: folder?.is_excluded_from_search ? IconUnhide : IconHide,
      action: () => {
        toggleFolderSearchExcluded(folder);
      }
    },
    {
      label: localeMsg.value.menu.album.refresh,
      icon: IconRefresh,
      action: () => {
        tauriEmit('refresh-content');
        // always refresh the folder tree as well
        expandFolder(folder, true);
      }
    },
    {
      label: "-",
      action: null
    },
    {
      label: localeMsg.value.menu.file.move_to_trash,
      icon: IconTrash,
      disabled: isRoot,
      action: () => {
        deletePermanently.value = permanentDeleteChecked.value;
        showTrashFolderMsgbox.value = true;
      }
    },
  ];
};

/// click folder to select
const clickFolder = async (albumIdVal: number, folder: Folder) => {
  console.log('AlbumFolder.vue-clickFolder:', albumIdVal, folder);
  if (props.allowContextMenu) {
    uiStore.setActivePane('left-sidebar');
  }
  await selection.selectFolder(albumIdVal, folder);
};

/// click expand icon to toggle folder expansion
const expandFolder = async (folder: any, forceRefresh = false) => {
  if (folder.is_excluded_from_search) return;
  folder.is_expanded = forceRefresh ? true : !folder.is_expanded;

  if (folder.is_expanded && (!folder.children || forceRefresh)) {
    const subFolders = await fetchFolder(folder.path, false, config.settings.folderSort);
    if (subFolders) {
      folder.children = subFolders.children;
    }
  }
};

const shouldRenderFolder = (folder: Folder) =>
  folder.id !== 0 || selection.folderPath.value === props.rootPath;

const flattenVisibleFolders = (nodes: Folder[] | undefined, result: Folder[] = []) => {
  if (!nodes) return result;

  for (const node of nodes) {
    if (!shouldRenderFolder(node)) continue;
    result.push(node);
    if (node.is_expanded && node.id !== 0) {
      flattenVisibleFolders(node.children, result);
    }
  }

  return result;
};

const getParentFolder = (nodes: Folder[] | undefined, targetPath: string, parent: Folder | null = null): Folder | null => {
  if (!nodes) return null;

  for (const node of nodes) {
    if (node.path === targetPath) return parent;
    const found = getParentFolder(node.children, targetPath, node);
    if (found) return found;
  }

  return null;
};

const getFirstChildFolder = (folder: Folder | null): Folder | null => {
  if (!folder?.children) return null;
  return folder.children.find((child: Folder) => shouldRenderFolder(child)) ?? null;
};

const shouldHandleTreeNavigation = (key: string) => {
  if (!props.treeRoot) return false;
  if (selection.albumId.value !== props.albumId || selection.selected.value) return false;
  if (uiStore.inputStack.length > 0) return false;
  if (props.allowContextMenu && uiStore.activePane !== 'left-sidebar') return false;
  if (document.activeElement !== treeRootRef.value) return false;

  const navigationKeys = ['ArrowUp', 'ArrowDown', 'ArrowLeft', 'ArrowRight', 'Home', 'End', 'Enter'];
  return navigationKeys.includes(key) && !!selection.folderPath.value && selection.folderPath.value.startsWith(props.rootPath);
};

const selectFolder = async (folder: Folder | null) => {
  if (!folder) return;
  await clickFolder(props.albumId, folder);
};

const handleTreeKeyDown = async (event: { payload: { key: string } }) => {
  if (!shouldHandleTreeNavigation(event.payload.key)) return;

  const currentPath = selection.folderPath.value;
  const visibleFolders = flattenVisibleFolders(props.children);
  const currentIndex = visibleFolders.findIndex((folder) => folder.path === currentPath);
  if (currentIndex === -1) return;

  const currentFolder = visibleFolders[currentIndex];
  switch (event.payload.key) {
    case 'ArrowUp':
      await selectFolder(visibleFolders[Math.max(0, currentIndex - 1)] ?? null);
      break;
    case 'ArrowDown':
      await selectFolder(visibleFolders[Math.min(visibleFolders.length - 1, currentIndex + 1)] ?? null);
      break;
    case 'ArrowLeft':
      if (currentFolder.is_expanded) {
        currentFolder.is_expanded = false;
      } else if (currentFolder.path === props.rootPath) {
        if (props.allowContextMenu) {
          uiStore.setActivePane('left-sidebar');
        }
        selection.selectAlbum({
          id: props.albumId,
          path: props.rootPath,
          name: '',
        } as Album);
        const albumListRoot = document.querySelector('[data-album-list-root="true"]') as HTMLElement | null;
        albumListRoot?.focus({ preventScroll: true });
      } else {
        const parentFolder = getParentFolder(props.children, currentFolder.path);
        if (parentFolder) {
          parentFolder.is_expanded = false;
          await selectFolder(parentFolder);
        }
      }
      break;
    case 'ArrowRight':
      if (!currentFolder.children || currentFolder.children.length === 0) {
        const subFolders = await fetchFolder(currentFolder.path, false, config.settings.folderSort);
        if (subFolders) {
          currentFolder.children = subFolders.children;
        }
      }

      if (currentFolder.children && currentFolder.children.length > 0) {
        if (!currentFolder.is_expanded) {
          currentFolder.is_expanded = true;
        } else {
          await selectFolder(getFirstChildFolder(currentFolder));
        }
      }
      break;
    case 'Home':
      await selectFolder(visibleFolders[0] ?? null);
      break;
    case 'End':
      await selectFolder(visibleFolders[visibleFolders.length - 1] ?? null);
      break;
    case 'Enter':
      await selectFolder(currentFolder);
      break;
  }
};

const handleLocalTreeKeyDown = (event: KeyboardEvent) => {
  if (!shouldHandleTreeNavigation(event.key)) return;
  event.preventDefault();
  void handleTreeKeyDown({ payload: { key: event.key } });
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
          clickFolder(props.albumId, newFolder);
        }
      });
    }
  } else {
    toast.error(localeMsg.value.msgbox.new_folder.error);
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
    const oldFolderPath = selection.folderPath.value;
    const newFolderPath_ = await renameFolder(oldFolderPath, newFolderName);
    if(newFolderPath_) {    // rename success
      let folder = selectedFolder.value;
      if (folder) {
        folder.name = newFolderName;
        updateFolderPath(folder, oldFolderPath, newFolderPath_);
      }

      // update selected folder path
      selection.folderPath.value = newFolderPath_;

      if (oldFolderPath === props.rootPath) {
        emit('rootRenamed', {
          albumId: props.albumId,
          newPath: newFolderPath_,
        });
      }

      isRenamingFolder.value = false;
      uiStore.removeInputHandler('AlbumFolder-rename');
    }
  }
};

/// rename folder path and children paths
function updateFolderPath(folder: any, oldpath: string, newPath: string) {
    folder.path = newPath + folder.path.slice(oldpath.length);

    if (folder.children) {
        folder.children.forEach((child: Folder) => {
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

const focusTreeRoot = (event: MouseEvent) => {
  if (props.treeRoot) {
    // If clicking on an input, don't focus the tree root
    // This prevents the input from blurring when clicked
    if (event.target instanceof HTMLInputElement) {
      return;
    }
    if (props.allowContextMenu) {
      uiStore.setActivePane('left-sidebar');
    }
    treeRootRef.value?.focus({ preventScroll: true });
  }
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
      toast.error(localeMsg.value.msgbox.move_to.error);
    }
  });
};

const selectSystemDestination = async (title: string) => {
  const destination = await openDialog({
    title,
    multiple: false,
    directory: true,
  });
  return !destination || Array.isArray(destination) ? null : String(destination);
};

const requestFileConflict = (
  name: string,
  destination: string,
  allowReplace = true,
): Promise<FileConflictPolicy> => {
  fileConflictDialog.value = { show: true, name, destination, allowReplace };
  return new Promise(resolve => {
    fileConflictResolver = resolve;
  });
};

const resolveFileConflict = (result: { policy: FileConflictPolicy }) => {
  fileConflictDialog.value.show = false;
  fileConflictResolver?.(result.policy);
  fileConflictResolver = null;
};

const resolveFolderConflictPolicy = async (
  folder: Folder,
  destPath: string,
  sameDestinationMeansSkip: boolean,
) => {
  const destinationPath = getFullPath(destPath, folder.name);
  const isSamePath =
    normalizePathForCompare(folder.path) === normalizePathForCompare(destinationPath);
  if (sameDestinationMeansSkip && isSamePath) {
    toast.info(t('msgbox.file_conflict.same_folder'));
    return 'skip' as FileConflictPolicy;
  }
  return await checkFileExists(destinationPath)
    ? requestFileConflict(folder.name, destPath, !isSamePath)
    : 'keep_both' as FileConflictPolicy;
};

const removeSelectedFolderFromTree = () => {
  const folderPath = selection.folderPath.value;
  if (!props.children) return;
  const index = (props.children as Folder[]).findIndex(child => child.path === folderPath);
  if (index !== -1) {
    (props.children as Folder[]).splice(index, 1);
  }
};

const clickMoveToFolder = async () => {
  const folder = selectedFolder.value;
  if (!folder) return;

  const destPath = await selectSystemDestination(t('msgbox.move_to_folder.title'));
  if (!destPath) return;

  const oldAlbumId = Number(selection.albumId.value || 0);
  const albums = await getAllAlbums();
  const destinationAlbum = Array.isArray(albums)
    ? albums.find(album => isWithinRootPath(destPath, album.path))
    : null;
  const conflictPolicy = await resolveFolderConflictPolicy(folder, destPath, true);
  if (conflictPolicy === 'skip') return;

  if (!destinationAlbum) {
    const confirmed = await ask(
      t('msgbox.move_to_folder.warning', { source: folder.name, dest: destPath }),
      {
        title: t('msgbox.move_to_folder.confirm_title'),
        kind: 'warning',
        okLabel: t('msgbox.move_to_folder.ok'),
        cancelLabel: t('msgbox.cancel'),
      },
    );
    if (!confirmed) return;
  }

  const newPath = destinationAlbum
    ? await moveFolder(folder.path, destinationAlbum.id, destPath, conflictPolicy)
    : await moveFolderOutsideLibrary(folder.path, destPath, conflictPolicy);
  if (!newPath) {
    toast.error(t('msgbox.move_to_folder.error', { source: folder.name, dest: destPath }));
    return;
  }

  removeSelectedFolderFromTree();
  toast.success(t('msgbox.move_to_folder.success', { source: folder.name, dest: destPath }));
  const albumIds = new Set([oldAlbumId, Number(destinationAlbum?.id || 0)]);
  const refreshedAlbums = (
    await Promise.all(Array.from(albumIds).filter(id => id > 0).map(id => recountAlbum(id)))
  ).filter(Boolean);
  if (refreshedAlbums.length > 0) await tauriEmit('albums-refreshed', { albums: refreshedAlbums });
  if (destinationAlbum) {
    await selection.expandAndSelectFolder(destinationAlbum.id, newPath);
  } else {
    await selection.expandAndSelectFolder(oldAlbumId, getFolderPath(folder.path));
  }
  await tauriEmit('library-total-refreshed', { source: 'album-folder' });
  await tauriEmit('refresh-content');
};

const clickCopyToFolder = async () => {
  const folder = selectedFolder.value;
  if (!folder) return;

  const destPath = await selectSystemDestination(t('msgbox.copy_to_folder.title'));
  if (!destPath) return;

  const conflictPolicy = await resolveFolderConflictPolicy(folder, destPath, false);
  if (conflictPolicy === 'skip') return;

  const albums = await getAllAlbums();
  const destinationAlbum = Array.isArray(albums)
    ? albums.find(album => isWithinRootPath(destPath, album.path))
    : null;
  const newPath = await copyFolder(
    folder.path,
    destPath,
    Number(destinationAlbum?.id || 0),
    conflictPolicy,
  );
  if (newPath) {
    toast.success(t('msgbox.copy_to_folder.success', { source: folder.name, dest: destPath }));
    if (destinationAlbum) {
      const album = await recountAlbum(destinationAlbum.id);
      if (album) await tauriEmit('albums-refreshed', { albums: [album] });
      await tauriEmit('refresh-content');
    }
  } else {
    toast.error(t('msgbox.copy_to_folder.error', { source: folder.name, dest: destPath }));
  }
};

/// trash or permanently delete selected folder
const clickTrashFolder = async () => {
  const folderName = selectedFolder.value?.name || '';
  permanentDeleteChecked.value = deletePermanently.value;
  const deleteFn = deletePermanently.value ? deleteFolderPermanently : deleteFolder;
  const isDeleted = await deleteFn(selection.folderPath.value);
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
    toast.success(
      deletePermanently.value
        ? t('msgbox.permanent_delete.folder_success', { folder: folderName })
        : t('msgbox.move_to_trash.folder_success', { folder: folderName })
    );
  } else {
    toast.error(
      deletePermanently.value
        ? t('msgbox.permanent_delete.folder_error')
        : t('msgbox.move_to_trash.folder_error')
    );
  }
};

/// toggle folder favorite
const toggleFavorite = async () => {
  const folder = selectedFolder.value;
  if (!folder || !selection.folderId.value) {
    return;
  }
  folder.is_favorite = !folder.is_favorite;
  await setFolderFavorite(selection.folderId.value, folder.is_favorite ?? false);
};

/// toggle whether folder and children are excluded from search
const toggleFolderSearchExcluded = async (folder: Folder) => {
  if (!folder?.path || !props.albumId) {
    return;
  }

  const nextValue = !folder.is_excluded_from_search;
  const result = await setFolderSearchExcluded(props.albumId, folder.path, nextValue);
  if (result !== null) {
    folder.is_excluded_from_search = nextValue;
    const album = await recountAlbum(props.albumId);
    if (album) {
      tauriEmit('albums-refreshed', { albums: [album], refreshFolders: false });
    }
    tauriEmit('library-total-refreshed');
  }
};

</script>

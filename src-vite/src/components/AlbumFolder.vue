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
    <li v-for="child in visibleChildren"
      :key="child.id" 
      :id="'folder-' + child.id" 
      :class="{ 'pl-4': child.path !== rootPath }"
    >
      <div v-if="child.id != 0 || selection.folderPath.value == rootPath"
        :data-file-drop-path="unavailable || isFolderUnavailable(child.path) ? undefined : child.path"
        :data-file-drop-album-id="unavailable || isFolderUnavailable(child.path) ? undefined : albumId"
        class="group/folder"
        :class="folderClass(child)"
        @click="clickFolder(albumId, child)"
        @dblclick="!isFolderFiltering && expandFolder(child)"
        @contextmenu.prevent.stop="(e: MouseEvent) => handleFolderContextMenu(child, e)"
        @mouseenter="hoveredFolderPath = child.path"
        @mouseleave="hoveredFolderPath === child.path && (hoveredFolderPath = '')"
      >
        <IconRight
          :class="[
            'p-1 w-6 h-6 shrink-0 transition-transform',
            isFolderFiltering
              ? (shouldShowFilteredChildren(child) ? 'rotate-90 pointer-events-none' : 'opacity-0 pointer-events-none')
              : (child.has_subfolders && !child.is_excluded_from_search ? '' : 'opacity-0 pointer-events-none'),
            !isFolderFiltering && child.is_expanded ? 'rotate-90' : ''
          ]"
          @click.stop="!isFolderFiltering && expandFolder(child)"
        />
        <component :is="child.is_excluded_from_search ? IconFolderOff : IconFolder" class="p-1 w-6 h-6 shrink-0" />

        <!-- name -->
        <input v-if="isCreatingFolder && creatingFolderPath === child.path"
          :data-new-folder-path="child.path"
          type="text"
          maxlength="255"
          class="input px-1 w-full text-base"
          v-model="newFolderName"
          @click.stop
          @mousedown.stop
          @keydown.enter.prevent="confirmNewFolder"
          @keydown.esc.stop.prevent="cancelNewFolder"
          @blur="confirmNewFolder"
        >
        <input v-else-if="isRenamingFolder && selection.folderPath.value === child.path"
          ref="folderInputRef"
          type="text"
          maxlength="255"
          class="input px-1 w-full text-base"
          v-model="child.name"
          @click.stop
          @mousedown.stop
          @keydown.enter = "clickRenameFolder(child.name)"
          @keydown.esc.stop = "handleEscKey($event, String(child.id))"
          @blur = "clickRenameFolder(child.name)"
        > 
        <template v-else>
          <div class="overflow-hidden whitespace-pre text-ellipsis">
            {{ child.name }}
          </div>
          <IconExclamation v-if="unavailable || isFolderUnavailable(child.path)" class="ml-1 size-3 shrink-0 text-warning/70" :aria-label="t('offline.original_unavailable')" />
          <div class="ml-auto flex flex-row items-center text-base-content/30">
            <IconHeartFilled v-if="child.is_favorite" class="mr-1 w-4 h-4 shrink-0 text-primary/70" />
            <span
              v-if="!showFolderActions && getFolderFileCount(child.path) > 0"
              class="sidebar-item-count shrink-0"
            >
              {{ getFolderFileCount(child.path).toLocaleString() }}
            </span>
            <ContextMenu v-if="allowContextMenu && !isRenamingFolder && !isCreatingFolder"
              v-show="shouldShowFolderMenu(child)"
              :ref="(el: any) => { if (el) folderContextMenus[child.path] = el }"
              :iconMenu="IconMore"
              :menuItems="() => getMenuItemsForFolder(child)"
              :smallIcon="true"
            />
            <div
              v-if="showFolderActions && !isRenamingFolder && !isCreatingFolder"
              class="grid min-w-14 ml-1 items-center justify-items-end group/actions"
              @mousedown.stop
            >
              <span
                v-if="getFolderFileCount(child.path) > 0"
                class="sidebar-item-count col-start-1 row-start-1 group-hover/folder:invisible group-focus-within/actions:invisible"
                :class="{ invisible: isSelectedFolder(child) }"
              >{{ getFolderFileCount(child.path).toLocaleString() }}</span>
              <div
                class="col-start-1 row-start-1 flex items-center gap-1 transition-opacity group-hover/folder:opacity-100 group-hover/folder:pointer-events-auto focus-within:opacity-100 focus-within:pointer-events-auto"
                :class="isSelectedFolder(child) ? 'opacity-100' : 'opacity-0 pointer-events-none'"
              >
                <TButton
                  :icon="IconRename"
                  buttonSize="small"
                  buttonClasses="cursor-pointer"
                  :tooltip="$t('menu.file.rename')"
                  :disabled="unavailable || isFolderUnavailable(child.path)"
                  @click.stop="startRenameFolder(child)"
                />
                <TButton
                  :icon="IconTrash"
                  buttonSize="small"
                  buttonClasses="cursor-pointer hover:bg-error/10! hover:text-error!"
                  :tooltip="$t('menu.file.delete')"
                  :disabled="unavailable || isFolderUnavailable(child.path) || child.path === rootPath"
                  @click.stop="startDeleteFolder(child)"
                />
              </div>
            </div>
          </div>
        </template>
      </div>
      <AlbumFolder v-if="shouldRenderChildren(child)"
        :key="child.id"
        :children="child.children" 
        :albumId="albumId"
        :rootPath="rootPath"
        :allowContextMenu="allowContextMenu"
        :showFolderActions="showFolderActions"
        :unavailable="unavailable"
        :treeRoot="false"
        :filterVisiblePaths="filterVisiblePaths"
        :filterMatchedPaths="filterMatchedPaths"
        @folder-favorite-changed="emit('folderFavoriteChanged')"
        @folder-path-changed="emit('folderPathChanged')"
      />
    </li>
  </ul>

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

  <MessageBox
    v-if="showTrashFailedFolderMsgbox"
    :title="$t('msgbox.trash_failed.title')"
    :message="$t('msgbox.trash_failed.folder_content', { folder: selectedFolder?.name || '' })"
    :OkText="$t('msgbox.trash_failed.ok')"
    :cancelText="$t('msgbox.cancel')"
    :warningOk="true"
    @ok="confirmTrashFailedFolderDelete"
    @cancel="showTrashFailedFolderMsgbox = false"
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

import { ref, nextTick, computed, inject, provide, onBeforeUnmount } from 'vue';
import { useI18n } from 'vue-i18n';
import { useUIStore } from '@/stores/uiStore';
import { config, libConfig } from '@/common/config';
import { isMac, shortenFilename, isValidFileName, getFolderPath, getFullPath, normalizePathForCompare, isWithinRootPath } from '@/common/utils';
import {
  createFolder, renameFolder, fetchFolder, getAllAlbums, moveFolder, moveFolderOutsideLibrary,
  copyFolder, checkFileExists, revealPath, deleteFolder, deleteFolderPermanently, recountAlbum, selectFolder as selectFolderInDb,
  isDirectoryAccessible, setFolderFavorite, setFolderSearchExcluded, hasImportableClipboard, refreshAlbumSubfolders,
} from '@/common/api';
import { DEFAULT_PLATFORM, getShortcutLabel } from '@/common/shortcuts';
import { Album, Folder } from '@/common/types';
import { getFolderFileCount, useAlbumSelection } from '@/composables/useAlbumSelection';

import AlbumFolder from '@/components/AlbumFolder.vue';
import ContextMenu from '@/components/ContextMenu.vue';
import TButton from '@/components/TButton.vue';
import MoveTo from '@/components/MoveTo.vue';
import MessageBox from '@/components/MessageBox.vue';
import FileConflictDialog from '@/components/FileConflictDialog.vue';
import { isFolderUnavailable, setFolderAccessibility } from '@/common/availability';
import { useToast } from '@/common/toast';
import { ask, open as openDialog } from '@tauri-apps/plugin-dialog';

import {
  IconRight,
  IconExclamation,
  IconMore,
  IconNewFolder,
  IconRename,
  IconFolderArrowRight,
  IconTrash,
  IconFolder,
  IconFolderOff,
  IconRefresh,
  IconClipboard,
  IconDownload,
  IconHeart,
  IconHeartFilled
} from '@/common/icons';

// used for cross-component communication (Content.vue listens for this event)
import { emit as tauriEmit } from '@tauri-apps/api/event';

const NEW_FOLDER_CONTEXT = 'album-folder-new-folder-context';

const props = withDefaults(defineProps<{
  children?: Folder[];      // subfolders
  albumId: number;          // album id for this folder tree
  rootPath: string;         // root folder path (album path)
  allowContextMenu?: boolean; // whether to show context menu
  showFolderActions?: boolean;
  treeRoot?: boolean;       // only root tree listens to keyboard
  filterVisiblePaths?: string[];
  filterMatchedPaths?: string[];
  unavailable?: boolean;
}>(), {
  treeRoot: true,
});

const emit = defineEmits<{
  rootRenamed: [payload: { albumId: number; newPath: string }];
  folderFavoriteChanged: [];
  folderPathChanged: [];
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
const isFolderFiltering = computed(() => Array.isArray(props.filterVisiblePaths));
const visibleFolderPaths = computed(() => new Set(props.filterVisiblePaths || []));
const matchedFolderPaths = computed(() => new Set(props.filterMatchedPaths || []));
const visibleChildren = computed(() => (props.children || []).filter(folder =>
  !isFolderFiltering.value || visibleFolderPaths.value.has(folder.path)
));

const isSelectedFolder = (folder: Folder) => !selection.selected.value && selection.folderPath.value === folder.path && !isRenamingFolder.value;
const folderClass = (folder: Folder) => {
  const selected = isSelectedFolder(folder);
  const matched = isFolderFiltering.value && matchedFolderPaths.value.has(folder.path);
  return [
    'p-1 h-8 flex items-center rounded-box whitespace-nowrap cursor-pointer group border-2',
    selected
      ? 'text-primary! bg-base-100 hover:bg-base-100 border-transparent'
      : 'hover:text-base-content hover:bg-base-100/30 border-transparent',
    folder.is_excluded_from_search ? 'text-base-content/30! hover:text-base-content/30!' : '',
    matched ? 'text-primary/70' : isFolderFiltering.value && !selected ? 'text-base-content/60' : '',
  ];
};
const shouldShowFilteredChildren = (folder: Folder) =>
  Boolean(folder.children?.some(child => visibleFolderPaths.value.has(child.path)));
const shouldRenderChildren = (folder: Folder) => {
  if (folder.id === 0 || folder.is_excluded_from_search) return false;
  return isFolderFiltering.value ? shouldShowFilteredChildren(folder) : Boolean(folder.is_expanded);
};

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
const inheritedNewFolderContext = inject<any>(NEW_FOLDER_CONTEXT, null);
const newFolderContext = inheritedNewFolderContext || {
  isCreatingFolder: ref(false),
  isCreatingFolderRequest: ref(false),
  creatingFolderParent: ref<Folder | null>(null),
  creatingFolderPath: ref(''),
  newFolderName: ref(''),
};
if (!inheritedNewFolderContext) provide(NEW_FOLDER_CONTEXT, newFolderContext);
const {
  isCreatingFolder,
  isCreatingFolderRequest,
  creatingFolderParent,
  creatingFolderPath,
  newFolderName,
} = newFolderContext;

// message boxes
const showTrashFolderMsgbox = ref(false);
const showTrashFailedFolderMsgbox = ref(false);
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
let disposed = false;

async function startRenameFolder(folder: Folder) {
  if (isRenamingFolder.value || isCreatingFolder.value || props.unavailable) return;
  if (!await isDirectoryAccessible(folder.path)) return;
  if (disposed) return;
  await clickFolder(props.albumId, folder);
  if (disposed || selection.folderPath.value !== folder.path) return;
  originalFolderName.value = folder.name;
  isRenamingFolder.value = true;
  uiStore.pushInputHandler('AlbumFolder-rename');
  await nextTick();
  const input = Array.isArray(folderInputRef.value) ? folderInputRef.value[0] : folderInputRef.value;
  input?.focus();
  input?.select();
}

async function startDeleteFolder(folder: Folder) {
  if (isRenamingFolder.value || isCreatingFolder.value || props.unavailable || folder.path === props.rootPath) return;
  if (!await isDirectoryAccessible(folder.path)) return;
  if (disposed) return;
  await clickFolder(props.albumId, folder);
  if (disposed || selection.folderPath.value !== folder.path) return;
  deletePermanently.value = permanentDeleteChecked.value;
  showTrashFolderMsgbox.value = true;
}

// more menuitems - function that takes the folder being right-clicked
const getMenuItemsForFolder = async (folder: any) => {
  const isRoot = folder.path === props.rootPath;
  const canPaste = !props.unavailable && await hasImportableClipboard();
  const unavailable = props.unavailable || !await isDirectoryAccessible(folder.path);
  setFolderAccessibility(folder.path, !unavailable);
  return [
    {
      label: folder?.is_favorite ? localeMsg.value.menu.meta.unfavorite : localeMsg.value.menu.meta.favorite,
      icon: folder?.is_favorite ? IconHeartFilled : IconHeart,
      action: () => {
        void toggleFolderFavorite(folder);
      }
    },
    {
      label: "-",
      action: null
    },
    {
      label: localeMsg.value.menu.file.new_folder,
      disabled: unavailable,
      icon: IconNewFolder,
      action: () => { void startNewFolder(folder); }
    },
    {
      label: localeMsg.value.menu.file.rename,
      disabled: unavailable,
      icon: IconRename,
      action: () => { void startRenameFolder(folder); }
    },
    {
      label: `${t('import_files.title')}…`,
      icon: IconDownload,
      disabled: unavailable,
      action: () => {
        void tauriEmit('import-files-to-folder', {
          albumId: props.albumId,
          folderPath: folder.path,
        });
      }
    },
    {
      label: t('menu.file.paste'),
      icon: IconClipboard,
      shortcut: getShortcutLabel('file.paste', DEFAULT_PLATFORM),
      disabled: unavailable || !canPaste,
      action: () => {
        void tauriEmit('paste-clipboard-to-folder', {
          albumId: props.albumId,
          folderPath: folder.path,
        });
      }
    },
    {
      label: t('menu.file.move_copy'),
      disabled: unavailable || isRoot,
      children: [
        {
          label: t('menu.file.move_within_library'),
          icon: IconFolderArrowRight,
          disabled: unavailable || isRoot,
          action: () => {
            showMoveTo.value = true;
          }
        },
        {
          label: t('menu.file.move_to_folder'),
          disabled: unavailable || isRoot,
          action: () => {
            void clickMoveToFolder();
          }
        },
        {
          label: t('menu.file.copy_to_folder'),
          disabled: unavailable || isRoot,
          action: () => {
            void clickCopyToFolder();
          }
        },
      ]
    },
    {
      label: isMac ? localeMsg.value.menu.file.reveal_in_finder : localeMsg.value.menu.file.reveal_in_file_explorer,
      disabled: unavailable,
      action: () => {
        revealPath(folder.path);
      }
    },
    {
      label: "-",
      action: null
    },
    {
      label: localeMsg.value.menu.album.refresh_subfolders,
      icon: IconRefresh,
      disabled: unavailable || refreshingSubfolderPaths.value.has(folder.path),
      action: () => { void refreshSubfolders(folder); }
    },
    {
      label: folder?.is_excluded_from_search ? localeMsg.value.menu.album.include_in_search : localeMsg.value.menu.album.exclude_from_search,
      icon: folder?.is_excluded_from_search ? IconFolder : IconFolderOff,
      action: () => {
        toggleFolderSearchExcluded(folder);
      }
    },
    {
      label: "-",
      action: null
    },
    {
      label: localeMsg.value.menu.album.delete_folder,
      icon: IconTrash,
      disabled: unavailable || isRoot,
      action: () => { void startDeleteFolder(folder); }
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

const hoveredFolderPath = ref('');
const shouldShowFolderMenu = (folder: Folder) =>
  (!selection.selected.value && selection.folderPath.value === folder.path) ||
  hoveredFolderPath.value === folder.path;

const toggleFolderFavorite = async (folder: Folder) => {
  const persistedFolder = await selectFolderInDb(props.albumId, folder.path);
  const folderId = Number(persistedFolder?.id || 0);
  if (folderId <= 0) return;

  const nextValue = !folder.is_favorite;
  const result = await setFolderFavorite(folderId, nextValue);
  if (result !== null) {
    folder.is_favorite = nextValue;
    emit('folderFavoriteChanged');
  }
};

const refreshingSubfolderPaths = ref(new Set<string>());

const refreshSubfolders = async (folder: Folder) => {
  const paths = new Set(refreshingSubfolderPaths.value);
  paths.add(folder.path);
  refreshingSubfolderPaths.value = paths;
  try {
    await refreshAlbumSubfolders(props.albumId, folder.path);
    const refreshed = await fetchFolder(folder.path, true, config.settings.folderSort);
    if (refreshed) {
      folder.has_subfolders = refreshed.has_subfolders;
      folder.children = refreshed.children;
      folder.is_expanded = true;
    }
    toast.success(localeMsg.value.tooltip.refresh_subfolders.success);
  } catch (error) {
    console.error('Failed to refresh subfolders:', error);
    toast.error(localeMsg.value.tooltip.refresh_subfolders.failed);
  } finally {
    const nextPaths = new Set(refreshingSubfolderPaths.value);
    nextPaths.delete(folder.path);
    refreshingSubfolderPaths.value = nextPaths;
  }
};

/// click expand icon to toggle folder expansion
const expandFolder = async (folder: any, forceRefresh = false) => {
  if (folder.is_excluded_from_search) return;
  if (!forceRefresh && folder.has_subfolders === false) return;
  folder.is_expanded = forceRefresh ? true : !folder.is_expanded;

  if (!props.unavailable && !isFolderUnavailable(folder.path) && folder.is_expanded && (!folder.children || forceRefresh)) {
    const subFolders = await fetchFolder(folder.path, false, config.settings.folderSort);
    if (subFolders) {
      folder.has_subfolders = subFolders.has_subfolders;
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
  if (isFolderFiltering.value) return false;
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
      if (currentFolder.has_subfolders === false) break;
      if (!props.unavailable && !isFolderUnavailable(currentFolder.path) && (!currentFolder.children || currentFolder.children.length === 0)) {
        const subFolders = await fetchFolder(currentFolder.path, false, config.settings.folderSort);
        if (subFolders) {
          currentFolder.has_subfolders = subFolders.has_subfolders;
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

const focusNewFolderInput = async (select = false) => {
  await nextTick();
  const input = Array.from(treeRootRef.value?.querySelectorAll<HTMLInputElement>('input[data-new-folder-path]') || [])
    .find(element => element.dataset.newFolderPath === creatingFolderPath.value);
  input?.scrollIntoView({ block: 'nearest' });
  input?.focus();
  if (select) input?.select();
};

const startNewFolder = async (folder: Folder) => {
  if (props.unavailable || isFolderUnavailable(folder.path)) return;
  if (isCreatingFolder.value) return;
  const refreshed = await fetchFolder(folder.path, false, config.settings.folderSort);
  if (disposed) return;
  if (refreshed) {
    folder.children = refreshed.children || [];
    folder.has_subfolders = refreshed.has_subfolders;
  }
  const existingNames = new Set((folder.children || []).map(child => child.name.toLowerCase()));
  const baseName = t('msgbox.new_folder.title');
  let index = 0;
  let name = baseName;
  while (existingNames.has(name.toLowerCase())) name = `${baseName} ${++index}`;

  const path = getFullPath(folder.path, name);
  folder.children = [...(folder.children || []), { id: -1, name, path }];
  folder.is_expanded = true;
  creatingFolderParent.value = folder;
  creatingFolderPath.value = path;
  newFolderName.value = name;
  isCreatingFolder.value = true;
  uiStore.pushInputHandler('AlbumFolder-new');
  await focusNewFolderInput(true);
};

const cancelNewFolder = () => {
  const parent = creatingFolderParent.value;
  if (parent?.children) {
    parent.children = parent.children.filter(child => child.path !== creatingFolderPath.value);
  }
  isCreatingFolder.value = false;
  isCreatingFolderRequest.value = false;
  creatingFolderParent.value = null;
  creatingFolderPath.value = '';
  newFolderName.value = '';
  uiStore.removeInputHandler('AlbumFolder-new');
};

const confirmNewFolder = async () => {
  if (!isCreatingFolder.value || isCreatingFolderRequest.value) return;
  const parent = creatingFolderParent.value;
  const name = newFolderName.value.trim();
  if (!parent || !name || !isValidFileName(name)) return;

  isCreatingFolderRequest.value = true;
  let newFolderPath: string | null;
  try {
    newFolderPath = await createFolder(parent.path, name);
  } catch (error) {
    if (disposed) return;
    isCreatingFolderRequest.value = false;
    toast.error(`${localeMsg.value.msgbox.new_folder.error}: ${String(error)}`);
    await focusNewFolderInput();
    return;
  }
  if (disposed) return;
  if (!newFolderPath) {
    isCreatingFolderRequest.value = false;
    toast.error(localeMsg.value.msgbox.new_folder.error);
    await focusNewFolderInput();
    return;
  }

  const refreshed = await fetchFolder(parent.path, false, config.settings.folderSort);
  if (disposed) return;
  if (refreshed) {
    parent.children = refreshed.children || [];
    parent.has_subfolders = refreshed.has_subfolders;
  }
  parent.is_expanded = true;
  isCreatingFolder.value = false;
  isCreatingFolderRequest.value = false;
  creatingFolderParent.value = null;
  creatingFolderPath.value = '';
  newFolderName.value = '';
  uiStore.removeInputHandler('AlbumFolder-new');
  const newFolder = parent.children?.find(child => child.path === newFolderPath);
  if (newFolder) await clickFolder(props.albumId, newFolder);
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
    let newFolderPath_: string | null;
    try {
      newFolderPath_ = await renameFolder(oldFolderPath, newFolderName);
    } catch (error) {
      toast.error(String(error));
      return;
    }
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
      emit('folderPathChanged');

      isRenamingFolder.value = false;
      uiStore.removeInputHandler('AlbumFolder-rename');
      if (props.showFolderActions) {
        await tauriEmit('albums-refreshed', {
          albums: await getAllAlbums(),
          renamedFolder: { albumId: props.albumId, oldPath: oldFolderPath, newPath: newFolderPath_ },
        });
      }
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
      const destinationAlbumId = Number(destinationAlbum.id);
      if (!(libConfig.index.albumQueue as any[]).some(id => Number(id) === destinationAlbumId)) {
        libConfig.index.albumQueue.push(destinationAlbumId);
      }
      libConfig.index.pausedAlbumIds = (libConfig.index.pausedAlbumIds as any[]).filter(
        id => Number(id) !== destinationAlbumId,
      );
      libConfig.index.status = 1;
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
  let deleteResult = 0;
  let deleteErrored = false;
  try {
    deleteResult = await deleteFn(selection.folderPath.value);
  } catch (error) {
    deleteErrored = true;
    console.error('Failed to delete folder:', error);
  }
  if (deleteErrored) {
    toast.error(
      deletePermanently.value
        ? t('msgbox.permanent_delete.folder_error')
        : t('msgbox.move_to_trash.folder_error')
    );
    return;
  }
  if (!deletePermanently.value && !deleteResult) {
    showTrashFolderMsgbox.value = false;
    showTrashFailedFolderMsgbox.value = true;
    return;
  }
  const isDeleted = !!deleteResult;
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
    if (props.showFolderActions) {
      await selection.expandAndSelectFolder(props.albumId, parentPath);
      await tauriEmit('albums-refreshed', { albums: await getAllAlbums() });
      await tauriEmit('library-total-refreshed', { source: 'album-folder' });
      await tauriEmit('refresh-content');
    }
  } else {
    toast.error(
      deletePermanently.value
        ? t('msgbox.permanent_delete.folder_error')
        : t('msgbox.move_to_trash.folder_error')
    );
  }
};

const confirmTrashFailedFolderDelete = async () => {
  const previousDeletePermanently = deletePermanently.value;
  const previousPermanentDeleteChecked = permanentDeleteChecked.value;
  showTrashFailedFolderMsgbox.value = false;
  deletePermanently.value = true;
  try {
    await clickTrashFolder();
  } finally {
    deletePermanently.value = previousDeletePermanently;
    permanentDeleteChecked.value = previousPermanentDeleteChecked;
  }
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

defineExpose({
  isCreatingFolder,
  canCreateSelectedFolder: computed(() =>
    !isRenamingFolder.value && !selection.selected.value && !!selectedFolder.value &&
    !props.unavailable && !isFolderUnavailable(selectedFolder.value.path),
  ),
  startNewFolder: async () => {
    if (selectedFolder.value) await startNewFolder(selectedFolder.value);
  },
});

onBeforeUnmount(() => {
  disposed = true;
  if (props.treeRoot && isCreatingFolder.value) {
    cancelNewFolder();
  }
  if (isRenamingFolder.value) {
    uiStore.removeInputHandler('AlbumFolder-rename');
  }
});

</script>

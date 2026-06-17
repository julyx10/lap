<template>
    <!-- albums -->
    <ul
      v-if="albums.length > 0"
      ref="albumListRootRef"
      tabindex="0"
      data-album-list-root="true"
      class="flex-1 overflow-x-hidden overflow-y-auto rounded-box select-none outline-none"
      @keydown="handleLocalAlbumListKeyDown"
      @mousedown.capture="focusAlbumListRoot"
      @drop.stop
    >
      
      <!-- title -->
      <div v-if="isMainPane" class="sidebar-panel-header">
        <span class="sidebar-panel-header-title flex-1">{{ $t('album.album_list') }}</span>
        <TButton
          :icon="IconAdd"
          :buttonSize="'small'"
          :tooltip="$t('menu.album.add')"
          @click="clickNewAlbum"
        />
      </div>

      <!-- drag to change albums' display order -->
      <VueDraggable
        v-model="albums"
        :disabled="!isMainPane"
        group="album-folder"
        :handle="'.album-drag-handle'"
        :animation="200"
        @start="onDragStart"
        @end="onDragEnd"
      >
        <li
          v-for="album in albums"
          :key="album.id"
          :data-album-id="album.id"
          :data-selected-album-folder="
            selection.albumId.value === album.id && !selection.selected.value ? 'true' : undefined
          "
        >
          <div
            :data-file-drop-path="album.is_accessible === false ? undefined : album.path"
            :data-file-drop-album-id="album.is_accessible === false ? undefined : album.id"
            :class="[
              'mx-1 p-1 h-12 flex items-center rounded-box whitespace-nowrap cursor-pointer group border-2 border-transparent transition-all duration-200 ease-in-out album-drag-handle',
              selection.albumId.value === album.id
                ? (selection.selected.value ? 'text-primary bg-base-100 hover:bg-base-100' : 'text-base-content')
                : 'hover:text-base-content hover:bg-base-100/30',
            ]"
            @click.stop="clickAlbum(album)"
            @dblclick.stop="dlbClickAlbum(album)"
            @contextmenu.prevent.stop="(e: MouseEvent) => handleAlbumContextMenu(album, e)"
          >
            <IconRight
              class="p-1 w-6 h-6 shrink-0 transition-transform hover:text-base-content"
              :class="{ 'rotate-90': album.is_expanded }"
              @click.stop="expandAlbum(album)"
              @dblclick.stop
            />
            <div class="w-10 h-10 mr-2 rounded-box shrink-0 overflow-hidden border border-base-content/5 bg-base-content/5" @click.stop>
              <!-- Scanning / Paused / Queued -->
              <div v-if="isAlbumScanning(album.id)"
                class="w-full h-full flex items-center justify-center cursor-pointer"
                :title="$t('toolbar.tooltip.scanning')"
                @click="toggleIndexAlbum(album.id)"
              >
                <IconUpdate class="w-6 h-6 animate-spin" />
              </div>
              <div v-else-if="isAlbumPaused(album.id) || (Number(album.indexed) > 0 && Number(album.indexed) < Number(album.total))"
                class="w-full h-full flex items-center justify-center"
                :class="album.is_accessible === false ? 'cursor-not-allowed opacity-50' : 'cursor-pointer'"
                :title="$t('toolbar.tooltip.scan_paused')"
                @click="toggleIndexAlbum(album.id)"
              >
                <IconUpdateDot class="w-6 h-6" />
              </div>
              <div v-else-if="getAlbumIcon(album) === 'update'"
                class="w-full h-full flex items-center justify-center cursor-pointer hover:bg-base-100/30"
                :title="$t('toolbar.tooltip.scan_queued')"
                @click="toggleIndexAlbum(album.id)"
              >
                <IconUpdate class="w-6 h-6" />
              </div>
              <!-- Cover -->
              <img
                v-else-if="album.cover_file_id && albumCovers[album.id] && !albumCoverErrors[album.id]"
                :src="albumCovers[album.id]"
                class="w-full h-full object-cover"
                @error="albumCoverErrors[album.id] = true"
                @click="clickAlbum(album)"
              >
              <!-- Fallback -->
              <div v-else
                class="w-full h-full flex items-center justify-center cursor-pointer"
                @click="clickAlbum(album)"
              >
                <IconFolders class="w-6 h-6" />
              </div>
            </div>

            <div class="flex flex-col overflow-hidden">
              <div class="overflow-hidden whitespace-pre text-ellipsis">
                {{ album.name }}
              </div>
              <div
                v-if="album.is_accessible === false"
                class="text-xs overflow-hidden whitespace-nowrap text-ellipsis text-warning/70"
              >{{ $t('album.folder_unavailable.title') }}</div>
              <div
                v-else-if="album.description"
                class="text-xs overflow-hidden whitespace-nowrap text-ellipsis text-base-content/50"
              >{{ album.description }}</div>
            </div>

            <!-- Right side: Count and Status Icons -->
            <div class="ml-auto pl-1 flex items-center justify-center text-xs text-base-content/30">
              <span
                v-if="props.showTotalCount !== false && album.total"
                :class="selection.albumId.value === album.id && selection.selected.value ? 'hidden' : 'group-hover:hidden'"
              >
                {{ album.total.toLocaleString() }}
              </span>
            </div>  

            <div class="flex flex-row items-center text-base-content/30">
              <div v-if="isMainPane"
                :class="[
                  selection.albumId.value === album.id && selection.selected.value ? '' : 'hidden group-hover:block'
                ]"
              >
                <ContextMenu
                  :ref="(el: any) => { if (el) albumContextMenus[album.id] = el }"
                  :iconMenu="IconMore"
                  :menuItems="() => getMoreMenuItems(album)"
                  :smallIcon="true"
                />
              </div>
            </div>
          </div>
          <transition
            enter-active-class="transition-all duration-200 ease-out overflow-hidden"
            enter-from-class="max-h-0"
            enter-to-class="max-h-96"
          >
            <div
              v-if="album.is_expanded && getAlbumQueueIndex(album.id, libConfig.index.albumQueue as any[]) === -1"
              class="ml-6 mr-2 my-1 p-1 rounded-box bg-base-300/30 border border-base-content/5 shadow-sm"
            >
              <div
                v-if="album.is_accessible === false"
                class="px-2 py-3 flex items-start gap-2 text-base-content/50"
              >
                <IconFolderError class="mt-0.5 w-4 h-4 shrink-0" />
                <div class="min-w-0">
                  <div class="text-sm text-base-content/70">{{ $t('album.folder_unavailable.title') }}</div>
                  <div class="text-xs">{{ $t('album.folder_unavailable.description') }}</div>
                </div>
              </div>
              <AlbumFolder
                v-else
                :children="album.children" 
                :albumId="album.id"
                :rootPath="album.path"
                :allowContextMenu="isMainPane"
                @root-renamed="handleRootRenamed"
              />
            </div>
          </transition>
        </li>
      </VueDraggable>

    </ul>

    <!-- edit album information -->
    <AlbumEdit
      v-if="showAlbumEdit"
      :isNewAlbum="isNewAlbum"
      :albumId="isNewAlbum ? 0 : editingAlbumId"
      :inputName="isNewAlbum ? '' : editingAlbum?.name"
      :inputDescription="isNewAlbum ? '' : editingAlbum?.description"
      :albumPath="isNewAlbum ? newAlbumFolderPath : editingAlbum?.path"
      :albumCoverFileId="isNewAlbum ? undefined : editingAlbum?.cover_file_id"
      :createdAt="isNewAlbum ? '' : formatTimestamp(editingAlbum?.created_at ?? 0, $t('format.date_time'))"
      :modifiedAt="isNewAlbum ? '' : formatTimestamp(editingAlbum?.modified_at ?? 0, $t('format.date_time'))"
      :lastScanTime="isNewAlbum ? '' : formatTimestamp((editingAlbum?.last_scan_time ?? 0) / 1000, $t('format.date_time'))"
      @ok="clickEditAlbum"
      @cancel="showAlbumEdit = false"
    />

    <!-- Remove album dialog -->
    <MessageBox
      v-if="showRemoveAlbumMsgbox"
      :title="$t('msgbox.remove_album.title')"
      :message="$t('msgbox.remove_album.content', { album: selectedAlbum?.name })"
      :OkText="$t('msgbox.remove_album.ok')"
      :cancelText="$t('msgbox.cancel')"
      :warningOk="true"
      @ok="clickRemoveAlbum"
      @cancel="showRemoveAlbumMsgbox = false"
    />

</template>

<script setup lang="ts">

import { ref, watch, computed, onMounted, onBeforeUnmount, nextTick } from 'vue';
import { useI18n } from 'vue-i18n';
import { VueDraggable } from 'vue-draggable-plus'
import { listen, emit as tauriEmit } from '@tauri-apps/api/event';
import { config, libConfig } from '@/common/config';
import { useUIStore } from '@/stores/uiStore';
import {
  scrollToFolder,
  formatTimestamp,
  getThumbUrl,
  getThumbnailDataUrl,
  getThumbnailDataUrlInflight,
  isWin,
  setThumbnailDataUrlInflight,
  openFolderDialog,
} from '@/common/utils';
import { getAlbumQueueIndex, getAlbumScanState, getAlbumScanIcon, shouldAnimateAlbumScanIcon } from '@/common/scanStatus';
import { getAllAlbums, setDisplayOrder, addAlbum, editAlbum, removeAlbum, 
         fetchFolder, expandFinalFolder, getFileThumbById,
         getAlbum, hasImportableClipboard, isDirectoryAccessible, cancelIndexing as cancelIndexingApi, listenIndexProgress, listenIndexFinished } from '@/common/api';
import { DEFAULT_PLATFORM, getShortcutLabel } from '@/common/shortcuts';
import { Album, Folder } from '@/common/types';
import { useAlbumSelectionProvider, SelectionSource } from '@/composables/useAlbumSelection';

import AlbumFolder from '@/components/AlbumFolder.vue';
import AlbumEdit from '@/components/AlbumEdit.vue';
import ContextMenu from '@/components/ContextMenu.vue';
import MessageBox from '@/components/MessageBox.vue';
import TButton from '@/components/TButton.vue';

import {
  IconAdd,
  IconMore,
  IconEdit,
  IconRemove,
  IconUpdate,
  IconUpdateOff,
  IconUpdateDot,
  IconRight,
  IconFolders,
  IconClipboard,
  IconFolderError,
} from '@/common/icons';

const props = withDefaults(defineProps<{
  selectionSource: SelectionSource;
  showTotalCount?: boolean;
}>(), {
  showTotalCount: true,
});

/// i18n
const { t, locale, messages } = useI18n();
const localeMsg = computed(() => messages.value[locale.value] as any);
const uiStore = useUIStore();

// Set up the selection context using provide/inject
// Pass the expandAndSelectFolder callback so the composable can trigger folder expansion
const selection = useAlbumSelectionProvider(
  props.selectionSource,
  async (albumIdVal: number, folderPathVal: string) => {
    await clickFinalSubFolder(albumIdVal, folderPathVal);
  }
);

let unlistenAlbumCoverChanged: () => void;
let unlistenExpandAlbumFolder: (() => void) | undefined;
let unlistenIndexProgress: (() => void) | undefined;
let unlistenIndexFinished: (() => void) | undefined;
let unlistenAlbumsRefreshed: (() => void) | undefined;

// Computed to check if we're in main album pane
const isMainPane = computed(() => props.selectionSource === 'album');
const albumListRootRef = ref<HTMLElement | null>(null);

// message boxes
const showAlbumEdit = ref(false);           // show edit album
const showRemoveAlbumMsgbox = ref(false);   // show remove album

const albums = ref<Album[]>([]);
const albumCovers = ref<Record<number, string>>({});
const isNewAlbum = ref(false);
const newAlbumFolderPath = ref('');
const editingAlbumId = ref(0);
const isLoading = ref(true);    // loading albums
const isDragging = ref(false);  // dragging albums
const albumCoverErrors = ref<Record<number, boolean>>({});
const albumContextMenus = ref<Record<number, any>>({});

function handleAlbumContextMenu(album: Album, event: MouseEvent) {
  void (async () => {
    await clickAlbum(album);
    albumContextMenus.value[album.id]?.open?.(event.clientX, event.clientY);
  })();
}

const getAlbumById = (id: number) =>
  albums.value.find(album => Number(album.id) === Number(id));
const selectedAlbum = computed(() => getAlbumById(selection.albumId.value)) || {};
const editingAlbum = computed(() => getAlbumById(editingAlbumId.value));
const isAlbumQueued = (albumId: number) =>
  getAlbumQueueIndex(albumId, libConfig.index.albumQueue as any[]) >= 0;
const syncIndexStatus = () => {
  if ((libConfig.index.albumQueue as any[]).length > 0) {
    libConfig.index.status = 1;
  } else if ((libConfig.index.pausedAlbumIds as any[]).length > 0) {
    libConfig.index.status = 2;
  } else {
    libConfig.index.status = 0;
  }
};
const isAlbumPaused = (albumId: number | null | undefined) =>
  (libConfig.index.pausedAlbumIds as any[]).some(id => Number(id) === Number(albumId || 0));
const removePausedAlbum = (albumId: number | null | undefined) => {
  libConfig.index.pausedAlbumIds = (libConfig.index.pausedAlbumIds as any[]).filter(
    id => Number(id) !== Number(albumId || 0)
  );
};
const addPausedAlbum = (albumId: number | null | undefined) => {
  if (Number(albumId || 0) <= 0 || isAlbumPaused(albumId)) return;
  (libConfig.index.pausedAlbumIds as any[]).push(Number(albumId));
};
const getAlbumStatus = (album: any) =>
  getAlbumScanState({
    albumId: album?.id,
    albumQueue: libConfig.index.albumQueue as any[],
    pausedAlbumIds: libConfig.index.pausedAlbumIds as any[],
    status: Number(libConfig.index.status || 0),
  });
const isAlbumScanning = (albumId: number) =>
  getAlbumScanState({
    albumId,
    albumQueue: libConfig.index.albumQueue as any[],
    pausedAlbumIds: libConfig.index.pausedAlbumIds as any[],
    status: Number(libConfig.index.status || 0),
  }) === 'scanning';
const getAlbumIcon = (album: any) => getAlbumScanIcon(getAlbumStatus(album));
const shouldAnimateAlbumIcon = (album: any) => shouldAnimateAlbumScanIcon(getAlbumStatus(album));
const refreshAlbumAccess = async (album: Album) => {
  album.is_accessible = await isDirectoryAccessible(album.path);
  if (!album.is_accessible) {
    album.children = undefined;
  }
  return album.is_accessible;
};

const openAlbumEdit = async (albumId: number) => {
  if (!getAlbumById(albumId)) {
    const album = await getAlbum(albumId);
    if (!album) return;
    if (!getAlbumById(albumId)) {
      albums.value.push(album);
    }
  }
  editingAlbumId.value = albumId;
  isNewAlbum.value = false;
  showAlbumEdit.value = true;
};

// Get menu items for a specific album (function for lazy evaluation)
const getMoreMenuItems = async (album: any) => {
  const [canPaste, isAccessible] = await Promise.all([
    hasImportableClipboard(),
    refreshAlbumAccess(album),
  ]);
  return [
    {
      label: localeMsg.value.menu.album.edit,
      icon: IconEdit,
      action: () => openAlbumEdit(album.id)
    },
    {
      label: t('menu.file.paste'),
      icon: IconClipboard,
      shortcut: getShortcutLabel('file.paste', DEFAULT_PLATFORM),
      disabled: !canPaste || !isAccessible,
      action: () => {
        void tauriEmit('paste-clipboard-to-folder', {
          albumId: album.id,
          folderPath: album.path,
        });
      }
    },
    {
      label: isAlbumQueued(album.id)
        ? localeMsg.value.menu.album.pause_scan
        : localeMsg.value.menu.album.scan,
      icon: isAlbumQueued(album.id) ? IconUpdateOff : IconUpdate,
      disabled: !isAccessible && !isAlbumQueued(album.id),
      action: () => toggleIndexAlbum(album.id)
    },
    {
      label: "-",   // separator
      action: () => {}
    },
    {
      label: localeMsg.value.menu.album.remove,
      icon: IconRemove,
      action: () => {
        showRemoveAlbumMsgbox.value = true;
      }
    },
  ];
};

// Load cover thumbnail for a single album
const loadAlbumCover = async (
  albumId: number,
  coverFileId: number | null,
  bustCache = false,
) => {
  delete albumCoverErrors.value[albumId];
  if (coverFileId) {
    let url = getThumbUrl(coverFileId, bustCache, config.settings.thumbnailSize);
    if (isWin && !url.startsWith('data:')) {
      const inflight = getThumbnailDataUrlInflight(coverFileId, config.settings.thumbnailSize);
      const dataUrl = await (inflight || setThumbnailDataUrlInflight(
        coverFileId,
        config.settings.thumbnailSize,
        getFileThumbById(coverFileId, config.settings.thumbnailSize, false)
          .then(thumb => getThumbnailDataUrl(thumb, '', false, config.settings.thumbnailSize))
      ));
      url = dataUrl || url;
    }
    if (url) {
      albumCovers.value = {
        ...albumCovers.value,
        [albumId]: url,
      };
    }
  } else {
    delete albumCovers.value[albumId];
  }
};

const loadAlbumCovers = async () => {
  for (const album of albums.value) {
    await loadAlbumCover(album.id, album.cover_file_id ?? null);
  }
};

onMounted( async () => {
  if (albums.value.length === 0) {
    albums.value = await getAllAlbums();
    await loadAlbumCovers();
    isLoading.value = false;

    if (selection.albumId.value > 0) {
      // expand and select the current album and folder
      clickFinalSubFolder(selection.albumId.value, selection.folderPath.value);
    }
  }

  // listen for album-cover-changed event
  unlistenAlbumCoverChanged = await listen('album-cover-changed', async (event: any) => {
    const eventAlbumId = Number(event.payload?.albumId || 0);
    const fileId = Number(event.payload?.fileId || 0);
    const album = getAlbumById(eventAlbumId);
    if (album) {
      if (fileId) {
        // manual update
        album.cover_file_id = fileId;
      } else {
        // indexing finished update, reload album to get new cover
        const updatedAlbums = await getAllAlbums();
        const updatedAlbum = updatedAlbums.find((a: Album) => a.id === eventAlbumId);
        if (updatedAlbum) {
          album.cover_file_id = updatedAlbum.cover_file_id;
        }
      }
      
      // Update the cover in albumCovers
      await loadAlbumCover(eventAlbumId, album.cover_file_id ?? null, true);
    }
  });

  // Keep the sidebar folder selection in sync with content navigation.
  unlistenExpandAlbumFolder = await listen('expand-album-folder', async (event: any) => {
    const { albumId, folderPath } = event.payload;
    if (albumId && folderPath) {
      await clickFinalSubFolder(albumId, folderPath);
    }
  });

  // listen for index progress
  unlistenIndexProgress = await listenIndexProgress(async (event: any) => {
    const { album_id, current, total } = event.payload;
    const album = getAlbumById(album_id);
    if (album) {
      album.indexed = current;
      album.total = total;
    }
  });

  // listen for index finished
  unlistenIndexFinished = await listenIndexFinished(async (event: any) => {
    const { album_id } = event.payload;
    const album = getAlbumById(album_id);
    if (album) {
      const updatedAlbum = await getAlbum(album_id);
      if (updatedAlbum) {
        album.indexed = updatedAlbum.indexed;
        album.total = updatedAlbum.total;
        album.cover_file_id = updatedAlbum.cover_file_id;
        album.last_scan_time = updatedAlbum.last_scan_time;
        album.last_scan_count = updatedAlbum.last_scan_count;
        
        // Reload the cover thumbnail
        await loadAlbumCover(album_id, album.cover_file_id ?? null);
        
        // Refresh folder tree if album is expanded (to show newly indexed folders)
        if (album.is_expanded) {
          await expandAlbum(album, true); // forceRefresh = true
        }
      }
    }
  });

  unlistenAlbumsRefreshed = await listen('albums-refreshed', async (event: any) => {
    const refreshedAlbums = Array.isArray(event.payload?.albums) ? event.payload.albums : [];
    const refreshFolders = event.payload?.refreshFolders !== false;
    const selectedAlbumId = selection.albumId.value;
    const selectedFolderPath = selection.folderPath.value;
    const shouldRestoreSelectedFolder = !selection.selected.value && !!selectedFolderPath;

    for (const updatedAlbum of refreshedAlbums) {
      const albumId = Number(updatedAlbum?.id || 0);
      if (albumId <= 0) continue;
      const album = getAlbumById(albumId);
      if (!album) continue;

      album.total = updatedAlbum.total;
      album.indexed = updatedAlbum.indexed;
      album.last_scan_time = updatedAlbum.last_scan_time;
      album.last_scan_count = updatedAlbum.last_scan_count;
      const previousCoverFileId = Number(album.cover_file_id || 0);
      if (updatedAlbum.cover_file_id !== undefined) {
        album.cover_file_id = updatedAlbum.cover_file_id;
      }
      if (Number(album.cover_file_id || 0) !== previousCoverFileId) {
        await loadAlbumCover(albumId, album.cover_file_id ?? null, true);
      }
      if (refreshFolders && album.is_expanded) {
        if (shouldRestoreSelectedFolder && albumId === selectedAlbumId) {
          await clickFinalSubFolder(albumId, selectedFolderPath);
        } else {
          await expandAlbum(album, true);
        }
      }
    }
  });

});

watch(() => config.settings.folderSort, async () => {
  const selectedAlbumId = selection.albumId.value;
  const selectedFolderPath = selection.folderPath.value;
  const shouldRestoreFolderSelection = !selection.selected.value && !!selectedFolderPath;

  for (const album of albums.value) {
    if (album.is_expanded) {
      await expandAlbum(album, true);
    }
  }

  if (shouldRestoreFolderSelection && selectedAlbumId > 0) {
    await clickFinalSubFolder(selectedAlbumId, selectedFolderPath);
  }
});

onBeforeUnmount(() => {
  if (unlistenAlbumCoverChanged) unlistenAlbumCoverChanged();
  if (unlistenExpandAlbumFolder) unlistenExpandAlbumFolder();
  if (unlistenIndexProgress) unlistenIndexProgress();
  if (unlistenIndexFinished) unlistenIndexFinished();
  if (unlistenAlbumsRefreshed) unlistenAlbumsRefreshed();
  uiStore.removeInputHandler('AlbumListDrag');
});

/// Add a new album
const clickNewAlbum = async () => {
  const folderPath = await openFolderDialog(t('album.edit.select_folder'));
  if (!folderPath) return;
  newAlbumFolderPath.value = folderPath;
  editingAlbumId.value = 0;
  showAlbumEdit.value = true;
  isNewAlbum.value = true;
};

// Refresh albums function
const refreshAlbums = async () => {
  isLoading.value = true;
  try {
    albums.value = await getAllAlbums();
  } catch (error) {
    console.error('Failed to refresh albums:', error);
  } finally {
    isLoading.value = false;
    
    selection.albumId.value = 0;      // show all files
    selection.folderPath.value = "";
    selection.selected.value = false;
  }
};

const handleRootRenamed = (payload: { albumId: number; newPath: string }) => {
  const album = albums.value.find((item: any) => item.id === payload.albumId);
  if (!album) return;

  album.path = payload.newPath;
};

/// edit album information or add new album
const clickEditAlbum = async (folderPathParam: string, newName: string, newDescription: string, isNew: boolean) => {
  if (isNew) {
    // Add new album
    const newAlbum = await addAlbum(folderPathParam);
    if (newAlbum) {
      // Update album name and description if different from folder name
      if (newName !== newAlbum.name || newDescription) {
        await editAlbum(newAlbum.id, newName, newDescription);
        newAlbum.name = newName;
        newAlbum.description = newDescription;
      }
      albums.value.push(newAlbum);
      clickAlbum(newAlbum);
      showAlbumEdit.value = false;

      tauriEmit('albums-refreshed');
      tauriEmit('library-total-refreshed');

      // add the new album to the index queue
      libConfig.index.status = 1;
      removePausedAlbum(newAlbum.id);
      libConfig.index.albumQueue.push(newAlbum.id);   
    }
  } else {
    // Edit existing album
    const result = await editAlbum(editingAlbumId.value, newName, newDescription);
    if(result && editingAlbum.value) {
      editingAlbum.value.name = newName;
      editingAlbum.value.description = newDescription;
      tauriEmit('album-updated', { albumId: editingAlbumId.value, name: newName, description: newDescription });
      showAlbumEdit.value = false;
    }
  }
};

/// Index an album
const clickIndexAlbum = async (albumId: number) => {
  removePausedAlbum(albumId);
  if (getAlbumQueueIndex(albumId, libConfig.index.albumQueue as any[]) === -1) {
    libConfig.index.albumQueue.push(albumId);
  }
  // Always set status to 1 — handles both fresh start and resume from paused-in-queue
  libConfig.index.status = 1;
}

const toggleIndexAlbum = async (albumId: number) => {
  const state = getAlbumStatus({ id: albumId });
  if (state === 'scanning' || state === 'queued') {
    await clickCancelIndexAlbum(albumId);
  } else {
    const album = getAlbumById(albumId);
    if (!album || !(await refreshAlbumAccess(album))) return;
    await clickIndexAlbum(albumId);
  }
}

/// Cancel indexing for an album
const clickCancelIndexAlbum = async (albumId: number) => {
  const index = getAlbumQueueIndex(albumId, libConfig.index.albumQueue as any[]);
  if (index === -1) return;

  // Keep queue handling aligned with Content.vue cancel behavior.
  if (index === 0) {
    libConfig.index.albumQueue.shift();
    await cancelIndexingApi(albumId);
    addPausedAlbum(albumId);
    if (libConfig.index.albumQueue.length > 0) {
      // Resume queue on next waiting album.
      syncIndexStatus();
      setTimeout(() => {
        tauriEmit('trigger-next-album');
      }, 1000);
    } else {
      syncIndexStatus();
    }
  } else {
    libConfig.index.albumQueue.splice(index, 1);
    addPausedAlbum(albumId);
    syncIndexStatus();
  }
}

/// Remove an album from the list
const clickRemoveAlbum = async () => {
  const albumId = selection.albumId.value;
  if (albumId > 0 && isAlbumScanning(albumId)) {
    await clickCancelIndexAlbum(albumId);
  }

  const removedAlbum = await removeAlbum(selection.albumId.value);
  if(removedAlbum) {
    showRemoveAlbumMsgbox.value = false;

    // Keep scan state consistent when the removed album was queued or paused.
    libConfig.index.albumQueue = (libConfig.index.albumQueue as any[]).filter(
      id => Number(id) !== Number(albumId)
    );
    removePausedAlbum(albumId);
    if ((libConfig.index.albumQueue as any[]).length === 0 && (libConfig.index.pausedAlbumIds as any[]).length === 0) {
      libConfig.index.albumName = '';
      libConfig.index.phase = 'discovering';
      libConfig.index.discovered = 0;
      libConfig.index.processed = 0;
      libConfig.index.searchReady = 0;
      libConfig.index.indexed = 0;
      libConfig.index.total = 0;
      libConfig.index.searchTotal = 0;
      libConfig.index.failed = 0;
    }
    syncIndexStatus();

    // remove the album from the list
    albums.value = albums.value.filter(album => album.id !== albumId);
    showAlbumEdit.value = false; // Close the edit dialog if it's open

    tauriEmit('albums-refreshed');
    tauriEmit('library-total-refreshed');

    selection.resetSelection();
  }
};

/// click a album to select it
const clickAlbum = async (album: Album) => {
  if (isMainPane.value) {
    uiStore.setActivePane('left-sidebar');
  }

  // In MoveTo dialog, disable album selection and toggle expansion instead
  if (!isMainPane.value) {
    await expandAlbum(album);
    return;
  }

  const isAccessible = await refreshAlbumAccess(album);
  if (isAccessible && album.is_expanded && !album.children) {
    const subFolders = await fetchFolder(album.path, false, config.settings.folderSort);
    if (subFolders) {
      album.children = [subFolders];
    }
  }
  selection.selectAlbum(album);
};

/// dlb click album to select it and expand/collapse its folders
const dlbClickAlbum = async (album: any) => {
  await clickAlbum(album);
  await expandAlbum(album);
};

/// click album icon to expand or collapse next level folders
const expandAlbum = async (album: any, forceRefresh = false) => {
  const willExpand = forceRefresh ? true : !album.is_expanded;

  album.is_expanded = willExpand; 
  
  if (album.is_expanded && !(await refreshAlbumAccess(album))) {
    return;
  }
  if (album.is_expanded && (!album.children || forceRefresh)) {
    const subFolders = await fetchFolder(album.path, false, config.settings.folderSort);
    if(subFolders) {
      album.children = [subFolders];
    }
  }
};

/// click folder to select
const clickFolder = async (albumIdVal: number, folder: Folder) => {
  console.log('AlbumList.vue-clickFolder:', folder);
  if (isMainPane.value) {
    uiStore.setActivePane('left-sidebar');
  }
  await selection.selectFolder(albumIdVal, folder);
};

const focusAlbumListRoot = (event: MouseEvent) => {
  // If clicking on an input, don't focus the album list root
  // This prevents inputs inside (like folder renaming) from blurring
  if (event.target instanceof HTMLInputElement) {
    return;
  }
  if (isMainPane.value) {
    uiStore.setActivePane('left-sidebar');
  }
  albumListRootRef.value?.focus({ preventScroll: true });
};

const waitForNextFrame = () => new Promise<void>((resolve) => {
  window.requestAnimationFrame(() => resolve());
});

const focusExpandedFolderTree = async (albumId: number) => {
  await nextTick();
  await waitForNextFrame();
  const albumListRoot = albumListRootRef.value;
  const folderTreeRoot = albumListRoot?.querySelector(
    `[data-album-id="${albumId}"] [data-folder-tree-root="true"]`
  ) as HTMLElement | null;
  folderTreeRoot?.focus({ preventScroll: true });
};

const shouldHandleAlbumListNavigation = (key: string) => {
  if (uiStore.inputStack.length > 0 || isDragging.value) return false;
  if (isMainPane.value && uiStore.activePane !== 'left-sidebar') return false;
  if (document.activeElement !== albumListRootRef.value) return false;

  const navigationKeys = ['ArrowUp', 'ArrowDown', 'ArrowLeft', 'ArrowRight', 'Home', 'End', 'Enter'];
  return navigationKeys.includes(key) && albums.value.length > 0;
};

const handleAlbumListKeyDown = async (key: string) => {
  if (!shouldHandleAlbumListNavigation(key)) return;

  const currentIndex = albums.value.findIndex(album => album.id === selection.albumId.value);
  const fallbackIndex = currentIndex >= 0 ? currentIndex : 0;
  const currentAlbum = albums.value[fallbackIndex];
  if (!currentAlbum) return;

  switch (key) {
    case 'ArrowUp':
      selection.selectAlbum(albums.value[Math.max(0, fallbackIndex - 1)] ?? currentAlbum);
      break;
    case 'ArrowDown':
      selection.selectAlbum(albums.value[Math.min(albums.value.length - 1, fallbackIndex + 1)] ?? currentAlbum);
      break;
    case 'ArrowRight':
      if (selection.selected.value) {
        if (!currentAlbum.is_expanded || !currentAlbum.children || currentAlbum.children.length === 0) {
          await expandAlbum(currentAlbum);
        }

        const rootFolder = currentAlbum.children?.[0];
        if (rootFolder) {
          await clickFolder(currentAlbum.id, rootFolder);
          await focusExpandedFolderTree(currentAlbum.id);
        }
      }
      break;
    case 'Home':
      selection.selectAlbum(albums.value[0] ?? currentAlbum);
      break;
    case 'End':
      selection.selectAlbum(albums.value[albums.value.length - 1] ?? currentAlbum);
      break;
    case 'Enter':
      selection.selectAlbum(currentAlbum);
      break;
  }
};

const handleLocalAlbumListKeyDown = (event: KeyboardEvent) => {
  if (!shouldHandleAlbumListNavigation(event.key)) return;
  event.preventDefault();
  void handleAlbumListKeyDown(event.key);
};

/// click the final sub-folder to select it
const clickFinalSubFolder = async (albumIdVal: number, folderPathVal: string) => {

  console.log('AlbumList.vue-clickFinalSubFolder:', albumIdVal, folderPathVal);
  let album = getAlbumById(albumIdVal);
  if(!album) {
    return;
  }

  // If navigating to the album root path, select the root folder directly.
  // expandFinalFolder returns null for the root path (empty relative path),
  // so we handle it here instead.
  if (folderPathVal === album.path) {
    if (selection.selected.value) {
      clickAlbum(album);
      return;
    }
    await expandAlbum(album, true);
    const rootFolder = album.children?.[0];
    if (rootFolder) {
      await clickFolder(album.id, rootFolder);
      scrollToFolder(rootFolder.id);
    }
    return;
  }

  if (selection.selected.value) {  // album is selected
    clickAlbum(album);
  } else {    // album's sub-folder is selected
    // expand the album's folder
    await expandAlbum(album, true);

    // recursively expand the final sub-folder path
    expandFinalFolder(album, folderPathVal).then((folder: Folder | null) => {
      if(folder) {
        clickFolder(album.id, folder).then(() => {
          scrollToFolder(folder.id);
        });
      }
    });
  }
};

/// drag albums to change their display order
const onDragStart = () => {
  isDragging.value = true;
  uiStore.removeInputHandler('AlbumListDrag');
  uiStore.pushInputHandler('AlbumListDrag');
};

const onDragEnd = async () => {
  isDragging.value = false;
  setTimeout(() => uiStore.removeInputHandler('AlbumListDrag'), 0);
  
  // update the display order of albums
  for (let i = 0; i < albums.value.length; i++) {
    await setDisplayOrder(albums.value[i].id, i);
  }
}

// Expose methods
defineExpose({
  albums,
  clickNewAlbum,
  openAlbumEdit,
  refreshAlbums,
  clickFinalSubFolder,
});

</script>

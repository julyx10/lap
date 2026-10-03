<template>
  <div class="sidebar-panel min-h-0">
    <!-- albums -->
    <div v-if="isMainPane" class="sidebar-panel-header">
      <span class="sidebar-panel-header-title flex-1">
        {{ $t('album.album_list') }}<template v-if="albums.length > 0"> ({{ albums.length.toLocaleString() }})</template>
      </span>
      <TButton
        v-if="albums.length > 0"
        :icon="IconAdd"
        :buttonSize="'small'"
        :tooltip="$t('menu.album.add')"
        @click="clickNewAlbum"
      />
    </div>

    <div v-if="isMainPane" class="mx-1 mb-2 px-1 shrink-0">
      <div
        :class="[
          'h-8 flex items-center rounded-box transition-colors bg-base-100/40',
          isFolderSearchFocused ? 'border-2 border-primary' : 'border border-base-content/10 hover:border-base-content/30',
          !isLoading && albums.length === 0 ? 'opacity-50' : '',
        ]"
      >
        <IconSearch class="ml-2 w-4 h-4 shrink-0" :class="isFolderSearchFocused ? 'text-primary/70' : 'text-base-content/30'" />
        <input
          v-model="folderSearch"
          type="text"
          :disabled="!isLoading && albums.length === 0"
          :placeholder="$t('album.search_folders')"
          class="w-full min-w-0 bg-transparent border-none focus:ring-0 px-2 text-sm placeholder-base-content/30 focus:outline-none disabled:opacity-50"
          @focus="isFolderSearchFocused = true"
          @blur="isFolderSearchFocused = false"
          @keydown.esc.stop="folderSearch = ''"
        />
        <span v-if="isFolderSearchLoading" class="loading loading-spinner loading-xs mr-2 text-base-content/30"></span>
        <button
          type="button"
          :title="$t('album.favorite_folders_only')"
          :aria-pressed="favoriteFoldersOnly"
          :class="[
            'p-1 rounded-box disabled:opacity-30',
            favoriteFoldersOnly ? 'text-primary!' : 'text-base-content/30 hover:text-base-content/70',
          ]"
          :disabled="!isLoading && albums.length === 0"
          @click="favoriteFoldersOnly = !favoriteFoldersOnly"
        >
          <component :is="favoriteFoldersOnly ? IconHeartFilled : IconHeart" class="w-4 h-4 cursor-pointer" />
        </button>
        <button
          v-if="folderSearch"
          type="button"
          :disabled="!isLoading && albums.length === 0"
          class="mr-1 p-1 rounded-box text-base-content/30 hover:text-base-content/70 disabled:opacity-30"
          @click="folderSearch = ''"
        >
          <IconClose class="w-4 h-4" />
        </button>
      </div>
    </div>

    <ul
      ref="albumListRootRef"
      tabindex="0"
      data-album-list-root="true"
      class="min-h-0 flex-1 overflow-x-hidden overflow-y-auto rounded-box select-none outline-none"
      @keydown="handleLocalAlbumListKeyDown"
      @mousedown.capture="focusAlbumListRoot"
      @drop.stop
    >
      <!-- drag to change albums' display order -->
      <VueDraggable
        v-if="visibleAlbums.length > 0"
        v-model="albums"
        :disabled="!isMainPane || isFolderFiltering || reorderingAlbumId === null"
        group="album-folder"
        :handle="'.album-drag-handle'"
        :animation="200"
        @start="onDragStart"
        @end="onDragEnd"
      >
        <li
          v-for="album in visibleAlbums"
          :key="album.id"
          :data-album-id="album.id"
          :data-selected-album-folder="
            selection.albumId.value === album.id && !selection.selected.value ? 'true' : undefined
          "
        >
          <div
            :data-reordering-album="isReorderingAlbum(album) ? 'true' : undefined"
            :data-file-drop-path="album.is_accessible === false ? undefined : album.path"
            :data-file-drop-album-id="album.is_accessible === false ? undefined : album.id"
            :class="[
              'mx-1 p-1 h-12 flex items-center rounded-box whitespace-nowrap cursor-pointer group border-2 border-transparent transition-all duration-200 ease-in-out',
              selection.albumId.value === album.id
                ? (selection.selected.value ? `${isMainSourceActive ? 'text-primary' : 'text-base-content/70 bg-base-100/30 hover:bg-base-100/70'} bg-base-100 hover:bg-base-100` : 'text-base-content hover:bg-base-100/30')
                : 'hover:text-base-content hover:bg-base-100/30',
            ]"
            @click.stop="clickAlbum(album)"
            @dblclick.stop="dlbClickAlbum(album)"
            @contextmenu.prevent.stop="(e: MouseEvent) => handleAlbumContextMenu(album, e)"
          >
            <IconDragHandle
              v-if="isReorderingAlbum(album)"
              class="album-drag-handle p-1 w-6 h-6 shrink-0 cursor-move text-base-content/70 hover:text-base-content"
              :title="$t('menu.album.reorder')"
              @click.stop
              @dblclick.stop
            />
            <IconRight
              v-else
              :class="[
                'p-1 w-6 h-6 shrink-0 transition-transform hover:text-base-content',
                isFolderFiltering
                  ? (shouldShowFilteredFolderTree(album.id) ? 'rotate-90 pointer-events-none' : 'opacity-0 pointer-events-none')
                  : (album.is_expanded ? 'rotate-90' : ''),
              ]"
              @click.stop="!isFolderFiltering && toggleAlbumExpansion(album)"
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

            <div class="flex flex-col overflow-hidden" :class="album.is_accessible === false ? 'opacity-50' : ''">
              <div class="overflow-hidden whitespace-pre text-ellipsis">
                {{ album.name }}
              </div>
              <div
                v-if="album.description"
                class="text-xs overflow-hidden whitespace-nowrap text-ellipsis text-base-content/50"
              >{{ album.description }}</div>
            </div>

            <!-- Right side: Count and Status Icons -->
            <div class="ml-auto flex flex-row items-center text-base-content/30">
              <span
                v-if="props.showTotalCount !== false && getAlbumDisplayCount(album) > 0"
                class="sidebar-item-count shrink-0"
              >
                {{ getAlbumDisplayCount(album).toLocaleString() }}
              </span>
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
              v-if="isFolderFiltering ? shouldShowFilteredFolderTree(album.id) : album.is_expanded"
              class="ml-6 mr-2 my-1 p-1 rounded-box bg-base-300/30 border border-base-content/5 shadow-sm"
            >
              <AlbumFolder
                :children="isFolderFiltering ? getFilteredFolderTree(album.id) : album.children"
                :albumId="album.id"
                :rootPath="album.path"
                :unavailable="album.is_accessible === false"
                :allowContextMenu="isMainPane"
                :filterVisiblePaths="isFolderFiltering ? getVisibleFolderPaths(album.id) : undefined"
                :filterMatchedPaths="isFolderFiltering ? getMatchedFolderPaths(album.id) : undefined"
                @folder-favorite-changed="refreshFolderSearchFolders"
                @folder-path-changed="refreshFolderSearchFolders"
                @root-renamed="handleRootRenamed"
              />
            </div>
          </transition>
        </li>
      </VueDraggable>

      <li v-if="!isFolderSearchLoading && visibleAlbums.length === 0 && albums.length > 0" class="sidebar-empty text-sm">
        <span class="text-center">{{ $t('album.no_folders_found') }}</span>
      </li>
      <li v-else-if="!isLoading && albums.length === 0" class="sidebar-empty text-sm">
        <span class="text-center">{{ $t('tooltip.not_found.albums') }}</span>
      </li>
    </ul>

    <!-- edit album information -->
    <AlbumEdit
      v-if="showAlbumEdit"
      :albumId="isNewAlbum ? 0 : editingAlbumId"
      :busy="albumEditBusy"
      :initialFolderPath="isNewAlbum ? newAlbumFolderPath : ''"
      @ok="clickEditAlbum"
      @cancel="!albumEditBusy && (showAlbumEdit = false)"
    />

    <ImportOrganizeDialog
      v-if="importAlbum"
      :album="importAlbum"
      @complete="handleImportComplete"
      @cancel="importAlbum = null"
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
  </div>
</template>

<script setup lang="ts">

import { useToast } from '@/common/toast';
import { ref, watch, computed, onMounted, onBeforeUnmount, nextTick } from 'vue';
import { useI18n } from 'vue-i18n';
import { VueDraggable } from 'vue-draggable-plus'
import { listen, emit as tauriEmit } from '@tauri-apps/api/event';
import { config, libConfig } from '@/common/config';
import { useUIStore } from '@/stores/uiStore';
import {
  scrollToFolder,
  getThumbUrl,
  getThumbnailDataUrl,
  getThumbnailDataUrlInflight,
  isWin,
  setThumbnailDataUrlInflight,
  openFolderDialog,
} from '@/common/utils';
import { getAlbumQueueIndex, getAlbumScanState, getAlbumScanIcon, shouldAnimateAlbumScanIcon } from '@/common/scanStatus';
import { getAllAlbums, getAlbumVisibleCounts, getAllAlbumFolders, reorderAlbums, addAlbum, editAlbum, removeAlbum, 
         fetchFolder, expandFinalFolder, getFileThumbById,
         getAlbum, checkAlbumAccessibility, cancelIndexing as cancelIndexingApi, listenIndexProgress, listenIndexFinished, recountAlbum } from '@/common/api';
import { Album, Folder } from '@/common/types';
import { useAlbumSelectionProvider, SelectionSource } from '@/composables/useAlbumSelection';

import AlbumFolder from '@/components/AlbumFolder.vue';
import AlbumEdit from '@/components/AlbumEdit.vue';
import ImportOrganizeDialog from '@/components/ImportOrganizeDialog.vue';
import ContextMenu from '@/components/ContextMenu.vue';
import MessageBox from '@/components/MessageBox.vue';
import TButton from '@/components/TButton.vue';

import {
  IconAdd,
  IconDownload,
  IconMore,
  IconEdit,
  IconRemove,
  IconUpdate,
  IconUpdateOff,
  IconUpdateDot,
  IconRight,
  IconDragHandle,
  IconOrder,
  IconFolders,
  IconSearch,
  IconClose,
  IconHeart,
  IconHeartFilled,
} from '@/common/icons';
import { SIDEBAR } from '@/common/constants';

const props = withDefaults(defineProps<{
  selectionSource: SelectionSource;
  titlebar?: string;
  showTotalCount?: boolean;
}>(), {
  showTotalCount: true,
});

/// i18n
const { t, locale, messages } = useI18n();
const localeMsg = computed(() => messages.value[locale.value] as any);
const uiStore = useUIStore();

const getAlbumDisplayCount = (album: Album) => {
  return Number(libConfig.album.counts?.[String(album.id)] || 0);
};

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
let unlistenAlbumFolderPathsMigrated: (() => void) | undefined;
let albumCountRequest = 0;

async function refreshAlbumVisibleCounts() {
  const request = ++albumCountRequest;
  const libraryId = libConfig._libraryId;
  const counts = await getAlbumVisibleCounts();
  if (
    request !== albumCountRequest
    || libraryId !== libConfig._libraryId
    || !counts
  ) return;
  libConfig.album.counts = counts;
}

// Computed to check if we're in main album pane
const isMainPane = computed(() => props.selectionSource === 'album');
const isMainSourceActive = computed(() => libConfig.activePane !== 'collection');
const albumListRootRef = ref<HTMLElement | null>(null);

// message boxes
const albumEditBusy = ref(false);
const showAlbumEdit = ref(false);           // show edit album
const showRemoveAlbumMsgbox = ref(false);   // show remove album
const importAlbum = ref<Album | null>(null);

const albums = ref<Album[]>([]);
const albumCovers = ref<Record<number, string>>({});
const folderSearch = ref('');
const favoriteFoldersOnly = ref(false);
const isFolderSearchFocused = ref(false);
const isFolderSearchLoading = ref(false);
const folderSearchFolders = ref<AlbumFolderRecord[]>([]);
let folderSearchRequest = 0;
const isNewAlbum = ref(false);
const newAlbumFolderPath = ref('');
const editingAlbumId = ref(0);
const isLoading = ref(true);    // loading albums
const isDragging = ref(false);  // dragging albums
const reorderingAlbumId = ref<number | null>(null);
const albumCoverErrors = ref<Record<number, boolean>>({});

interface FilteredAlbumResult {
  album: Album;
  rootFolderMatches: boolean;
  hasMatches: boolean;
  visibleFolderPaths: string[];
  matchedFolderPaths: string[];
  folderTree: Folder[];
}

interface AlbumFolderRecord extends Folder {
  album_id: number;
}

const normalizedFolderSearch = computed(() => folderSearch.value.trim().toLocaleLowerCase());
const isFolderFiltering = computed(() =>
  isMainPane.value && (normalizedFolderSearch.value.length > 0 || favoriteFoldersOnly.value)
);
const folderSearchFoldersByAlbum = computed(() => {
  const foldersByAlbum = new Map<number, AlbumFolderRecord[]>();
  for (const folder of folderSearchFolders.value) {
    const albumId = Number(folder.album_id);
    const folders = foldersByAlbum.get(albumId) || [];
    folders.push(folder);
    foldersByAlbum.set(albumId, folders);
  }
  return foldersByAlbum;
});
const folderSearchCollator = new Intl.Collator(undefined, { numeric: true, sensitivity: 'base' });

function getRelativeFolderPath(folderPath: string, rootPath: string) {
  if (!folderPath.startsWith(rootPath)) return folderPath;
  return folderPath.slice(rootPath.length).replace(/^[\\/]+/, '');
}

function getFolderName(folderPath: string) {
  return folderPath.replace(/[\\/]+$/, '').split(/[\\/]/).filter(Boolean).pop() || folderPath;
}

function getFolderSearchPaths(folders: AlbumFolderRecord[], rootPath: string, query: string) {
  const visiblePaths = new Set<string>();
  const matchedPaths = new Set<string>();
  const hasQuery = query.length > 0;

  for (const folder of folders) {
    const relativePath = getRelativeFolderPath(folder.path, rootPath);
    const isMatch = !hasQuery || folder.name.toLocaleLowerCase().includes(query)
      || (/[\\/]/.test(query) && relativePath.toLocaleLowerCase().includes(query));
    if (isMatch) {
      matchedPaths.add(folder.path);
      let ancestorPath = folder.path;
      while (ancestorPath.startsWith(rootPath) && ancestorPath !== rootPath) {
        visiblePaths.add(ancestorPath);
        const separatorIndex = Math.max(ancestorPath.lastIndexOf('/'), ancestorPath.lastIndexOf('\\'));
        ancestorPath = separatorIndex >= 0 ? ancestorPath.slice(0, separatorIndex) : rootPath;
      }
    }
  }

  return {
    visibleFolderPaths: Array.from(visiblePaths),
    matchedFolderPaths: Array.from(matchedPaths),
    hasMatches: visiblePaths.size > 0,
  };
}

const compareFolders = (a: Folder, b: Folder) => {
  const aTime = a.modified_at || a.created_at || 0;
  const bTime = b.modified_at || b.created_at || 0;
  switch (Number(config.settings.folderSort)) {
    case 1: return folderSearchCollator.compare(b.name, a.name);
    case 2: return aTime - bTime;
    case 3: return bTime - aTime;
    default: return folderSearchCollator.compare(a.name, b.name);
  }
};

let lastFolderScanRefresh = 0;
async function refreshExpandedAlbumFoldersDuringScan(album: Album) {
  if (!album.is_expanded) return;
  const now = Date.now();
  if (now - lastFolderScanRefresh < 3000) return;
  lastFolderScanRefresh = now;

  if (album.is_accessible === false) {
    if (!album.children) await loadCachedAlbumTree(album);
    return;
  }

  const subFolders = await fetchFolder(album.path, false, config.settings.folderSort);
  if (!subFolders) return;

  if (!album.children || album.children.length === 0) {
    album.children = [subFolders];
  } else {
    const rootNode = album.children[0];
    if (rootNode) {
      rootNode.has_subfolders = subFolders.has_subfolders;
      if (subFolders.children) {
        if (!rootNode.children || rootNode.children.length === 0) {
          rootNode.children = subFolders.children;
        } else {
          const existingMap = new Map(rootNode.children.map((c: Folder) => [c.path, c]));
          let changed = false;
          for (const newChild of subFolders.children) {
            const existing = existingMap.get(newChild.path);
            if (!existing) {
              rootNode.children.push(newChild);
              changed = true;
            } else if (existing.has_subfolders !== newChild.has_subfolders) {
              existing.has_subfolders = newChild.has_subfolders;
            }
          }
          if (changed) {
            rootNode.children.sort(compareFolders);
          }
        }
      }
    }
  }
}

function buildFilteredFolderTree(folders: AlbumFolderRecord[], visiblePaths: string[]) {
  const visible = new Set(visiblePaths);
  const nodes = new Map<string, Folder>();
  const roots: Folder[] = [];

  for (const folder of folders) {
    if (visible.has(folder.path)) nodes.set(folder.path, { ...folder, is_expanded: false, children: [] });
  }
  for (const folder of nodes.values()) {
    const separatorIndex = Math.max(folder.path.lastIndexOf('/'), folder.path.lastIndexOf('\\'));
    const parentPath = separatorIndex >= 0 ? folder.path.slice(0, separatorIndex) : '';
    const parent = nodes.get(parentPath);
    if (parent) parent.children?.push(folder);
    else roots.push(folder);
  }
  const sortTree = (children: Folder[]) => {
    children.sort(compareFolders);
    for (const child of children) sortTree(child.children || []);
  };
  sortTree(roots);
  return roots;
}

async function loadCachedAlbumTree(album: Album) {
  const records = (await getAllAlbumFolders() || [])
    .filter((folder: AlbumFolderRecord) => Number(folder.album_id) === Number(album.id));
  const root = records.find((folder: AlbumFolderRecord) => folder.path === album.path) || {
    id: -Number(album.id),
    album_id: Number(album.id),
    name: getFolderName(album.path),
    path: album.path,
    has_subfolders: records.length > 0,
  };
  const folders = [root, ...records.filter((folder: AlbumFolderRecord) => folder.path !== album.path)];
  album.children = buildFilteredFolderTree(folders, folders.map((folder: AlbumFolderRecord) => folder.path));
}

const filteredAlbumResults = computed<FilteredAlbumResult[]>(() => {
  const query = normalizedFolderSearch.value;
  if (!query && !favoriteFoldersOnly.value) {
    return albums.value.map(album => ({
      album,
      rootFolderMatches: false,
      visibleFolderPaths: [],
      matchedFolderPaths: [],
      hasMatches: false,
      folderTree: [],
    }));
  }

  return albums.value.flatMap((album) => {
    const albumFolders = folderSearchFoldersByAlbum.value.get(Number(album.id)) || [];
    const rootFolder = albumFolders.find(folder => folder.path === album.path) || {
      id: -Number(album.id),
      album_id: Number(album.id),
      name: getFolderName(album.path),
      path: album.path,
    };
    const rootFolderMatches = query.length > 0
      && (!favoriteFoldersOnly.value || rootFolder.is_favorite)
      && getFolderName(album.path).toLocaleLowerCase().includes(query);
    const rootFolderVisible = rootFolderMatches
      || (!query && favoriteFoldersOnly.value && rootFolder.is_favorite);
    const folders = albumFolders.filter(folder => folder.path !== album.path);
    const matchingFolders = favoriteFoldersOnly.value
      ? folders.filter(folder => folder.is_favorite)
      : folders;
    const folderPaths = getFolderSearchPaths(matchingFolders, album.path, query);
    const visibleFolderPaths = rootFolderVisible || folderPaths.hasMatches
      ? [rootFolder.path, ...folderPaths.visibleFolderPaths]
      : [];
    const matchedFolderPaths = rootFolderMatches
      ? [rootFolder.path, ...folderPaths.matchedFolderPaths]
      : folderPaths.matchedFolderPaths;
    return rootFolderVisible || folderPaths.hasMatches
      ? [{
        album,
        rootFolderMatches,
        ...folderPaths,
        hasMatches: rootFolderVisible || folderPaths.hasMatches,
        visibleFolderPaths,
        matchedFolderPaths,
        folderTree: buildFilteredFolderTree([rootFolder, ...folders], visibleFolderPaths),
      }]
      : [];
  });
});

const visibleAlbums = computed(() => filteredAlbumResults.value.map(result => result.album));
const getFilteredAlbumResult = (albumId: number) =>
  filteredAlbumResults.value.find(result => Number(result.album.id) === Number(albumId));
const getVisibleFolderPaths = (albumId: number) => getFilteredAlbumResult(albumId)?.visibleFolderPaths || [];
const getMatchedFolderPaths = (albumId: number) => getFilteredAlbumResult(albumId)?.matchedFolderPaths || [];
const getFilteredFolderTree = (albumId: number) => getFilteredAlbumResult(albumId)?.folderTree || [];
const shouldShowFilteredFolderTree = (albumId: number) => {
  const result = getFilteredAlbumResult(albumId);
  return isFolderFiltering.value && Boolean(result?.hasMatches);
};

async function loadFolderSearchFolders() {
  const request = ++folderSearchRequest;
  isFolderSearchLoading.value = true;
  try {
    const folders = await getAllAlbumFolders();
    if (request === folderSearchRequest && isFolderFiltering.value) {
      folderSearchFolders.value = folders || [];
    }
  } finally {
    if (request === folderSearchRequest) isFolderSearchLoading.value = false;
  }
}

function refreshFolderSearchFolders() {
  folderSearchFolders.value = [];
  folderSearchRequest += 1;
  isFolderSearchLoading.value = false;
  if (isFolderFiltering.value) void loadFolderSearchFolders();
}

const updateFolderPath = (folders: Folder[] | undefined, oldPath: string, newPath: string) => {
  if (!folders) return;
  const oldPathPrefix = `${oldPath}${oldPath.includes('\\') ? '\\' : '/'}`;
  for (const folder of folders) {
    if (folder.path === oldPath) {
      folder.path = newPath;
      folder.name = newPath.split(/[\\/]/).filter(Boolean).pop() || folder.name;
    } else if (folder.path.startsWith(oldPathPrefix)) {
      folder.path = `${newPath}${folder.path.slice(oldPath.length)}`;
    }
    updateFolderPath(folder.children, oldPath, newPath);
  }
};
const albumContextMenus = ref<Record<number, any>>({});

function handleAlbumContextMenu(album: Album, event: MouseEvent) {
  void (async () => {
    await clickAlbum(album);
    albumContextMenus.value[album.id]?.open?.(event.clientX, event.clientY);
  })();
}

const getAlbumById = (id: number) =>
  albums.value.find(album => Number(album.id) === Number(id));
const isReorderingAlbum = (album: Album) => Number(album.id) === reorderingAlbumId.value;
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
  album.is_accessible = await checkAlbumAccessibility(album.id);
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

const handleImportComplete = async () => {
  if (!importAlbum.value) return;
  const album = getAlbumById(Number(importAlbum.value.id));
  const updated = await recountAlbum(Number(importAlbum.value.id));
  if (album && updated) Object.assign(album, updated);
  if (album?.is_expanded) await expandAlbum(album, true);
  await refreshAlbumVisibleCounts();
  await tauriEmit('import-files-added', { albumId: Number(importAlbum.value.id) });
};

// Get menu items for a specific album (function for lazy evaluation)
const getMoreMenuItems = async (album: any) => {
  const isAccessible = await refreshAlbumAccess(album);
  return [
    {
      label: localeMsg.value.menu.album.edit,
      icon: IconEdit,
      action: () => openAlbumEdit(album.id)
    },
    {
      label: "-",   // separator
      action: () => {}
    },
    {
      label: `${localeMsg.value.import_organize.import}…`,
      icon: IconDownload,
      disabled: !isAccessible,
      action: () => { importAlbum.value = album; }
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
      label: localeMsg.value.menu.album.reorder,
      icon: IconOrder,
      action: () => {
        reorderingAlbumId.value = isReorderingAlbum(album) ? null : album.id;
      }
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
  document.addEventListener('pointerdown', handleReorderOutsidePointerDown, true);
  if (albums.value.length === 0) {
    albums.value = await getAllAlbums(true);
    void refreshAlbumVisibleCounts();
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

  unlistenAlbumFolderPathsMigrated = await listen('album-folder-paths-migrated', (event: any) => {
    for (const migration of event.payload || []) {
      const albumId = Number(migration?.albumId || 0);
      const oldPath = String(migration?.oldPath || '');
      const newPath = String(migration?.newPath || '');
      const album = getAlbumById(albumId);
      if (!album || !oldPath || !newPath) continue;
      updateFolderPath(album.children, oldPath, newPath);
      if (
        selection.albumId.value === albumId &&
        !selection.selected.value &&
        selection.folderPath.value === oldPath
      ) {
        selection.folderPath.value = newPath;
      }
    }
    refreshFolderSearchFolders();
  });

  // listen for index progress
  unlistenIndexProgress = await listenIndexProgress(async (event: any) => {
    const { album_id, current, total } = event.payload;
    const album = getAlbumById(album_id);
    if (album) {
      album.indexed = current;
      album.total = total;
      if (album.is_expanded) {
        await refreshExpandedAlbumFoldersDuringScan(album);
      }
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
        album.file_types = updatedAlbum.file_types;
        album.small_image_filter = updatedAlbum.small_image_filter;
        album.excluded_folders = updatedAlbum.excluded_folders;
        album.total = updatedAlbum.total;
        album.cover_file_id = updatedAlbum.cover_file_id;
        album.last_scan_time = updatedAlbum.last_scan_time;
        album.last_scan_count = updatedAlbum.last_scan_count;
        album.skipped_count = updatedAlbum.skipped_count;
        album.skipped_size = updatedAlbum.skipped_size;
        album.failed_count = updatedAlbum.failed_count;
        album.failed_size = updatedAlbum.failed_size;
        album.merged_count = updatedAlbum.merged_count;
        album.merged_size = updatedAlbum.merged_size;
        
        // Reload the cover thumbnail
        await loadAlbumCover(album_id, album.cover_file_id ?? null);
        
        // Refresh folder tree if album is expanded (to show newly indexed folders)
        if (album.is_expanded) {
          await expandAlbum(album, true); // forceRefresh = true
        }
      }
    }
    refreshFolderSearchFolders();
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
      album.skipped_count = updatedAlbum.skipped_count;
      album.skipped_size = updatedAlbum.skipped_size;
      album.failed_count = updatedAlbum.failed_count;
      album.failed_size = updatedAlbum.failed_size;
      album.merged_count = updatedAlbum.merged_count;
      album.merged_size = updatedAlbum.merged_size;
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
    refreshFolderSearchFolders();
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


watch(() => [config.main.sidebarIndex, libConfig.activePane], () => {
  if (isMainPane.value && libConfig.activePane === 'main' && config.main.sidebarIndex === SIDEBAR.ALBUM) {
    void refreshAlbumVisibleCounts();
  }
});

watch(isFolderFiltering, (filtering) => {
  if (!filtering) {
    refreshFolderSearchFolders();
    return;
  }
  reorderingAlbumId.value = null;
  void loadFolderSearchFolders();
});

onBeforeUnmount(() => {
  document.removeEventListener('pointerdown', handleReorderOutsidePointerDown, true);
  if (unlistenAlbumCoverChanged) unlistenAlbumCoverChanged();
  if (unlistenExpandAlbumFolder) unlistenExpandAlbumFolder();
  if (unlistenIndexProgress) unlistenIndexProgress();
  if (unlistenIndexFinished) unlistenIndexFinished();
  if (unlistenAlbumsRefreshed) unlistenAlbumsRefreshed();
  if (unlistenAlbumFolderPathsMigrated) unlistenAlbumFolderPathsMigrated();
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
    refreshFolderSearchFolders();
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
const clickEditAlbum = async (folderPathParam: string, newName: string, newDescription: string, isNew: boolean,
  filters: { fileTypes: number; smallImageFilter: number; excludedFolders: string[] }, filtersChanged: boolean) => {
  if (albumEditBusy.value) return;
  albumEditBusy.value = true;
  let interruptedScan: { id: number; libraryId: string; position: number; status: number; paused: boolean } | null = null;
  try {
    if (isNew) {
      const newAlbum = await addAlbum(folderPathParam, newName, newDescription, filters);
      if (!newAlbum) throw new Error('Could not create album');
      config.leftPanel.show = true;
      albums.value.push(newAlbum);
      clickAlbum(newAlbum);
      showAlbumEdit.value = false;
      await tauriEmit('albums-refreshed');
      await tauriEmit('library-total-refreshed');
      await clickIndexAlbum(newAlbum.id);
    } else {
      const current = editingAlbum.value;
      const scopeChanged = current?.file_types !== filters.fileTypes
        || Number(current?.small_image_filter || 0) !== filters.smallImageFilter
        || JSON.stringify([...(current?.excluded_folders || [])].sort()) !== JSON.stringify(filters.excludedFolders);
      if (scopeChanged) {
        const id = editingAlbumId.value;
        const position = getAlbumQueueIndex(id, libConfig.index.albumQueue as any[]);
        if (position >= 0) interruptedScan = {
          id, position, libraryId: libConfig._libraryId,
          status: Number(libConfig.index.status), paused: isAlbumPaused(id),
        };
        await clickCancelIndexAlbum(id);
      }
      const result = await editAlbum(editingAlbumId.value, newName, newDescription, filters);
      if (!result) throw new Error('Could not save album');
      if (current) Object.assign(current, { name: newName, description: newDescription,
        file_types: filters.fileTypes, small_image_filter: filters.smallImageFilter, excluded_folders: filters.excludedFolders });
      if (scopeChanged && current && filters.excludedFolders.some(folder => {
        const root = `${current.path.replace(/[\\/]$/, '')}/${folder}`.replaceAll('\\', '/');
        const selected = String(selection.folderPath.value || '').replaceAll('\\', '/');
        return selected === root || selected.startsWith(`${root}/`);
      })) clickAlbum(current);
      showAlbumEdit.value = false;
      await tauriEmit('album-updated', { albumId: editingAlbumId.value, name: newName, description: newDescription, filtersChanged });
      if (filtersChanged) {
        libConfig.clearLazySidebarCounts();
        await tauriEmit('albums-refreshed', { albums: await getAllAlbums(), refreshFolders: true });
        await refreshAlbumVisibleCounts();
        await tauriEmit('library-total-refreshed');
      }
      if (scopeChanged) await clickIndexAlbum(editingAlbumId.value);
    }
  } catch (error) {
    console.error('Saving album failed:', error);
    // Restore only scans interrupted by this save, never an already-paused album.
    if (interruptedScan && interruptedScan.libraryId === libConfig._libraryId
      && getAlbumQueueIndex(interruptedScan.id, libConfig.index.albumQueue as any[]) === -1) {
      const { id, position, status, paused } = interruptedScan;
      const queue = libConfig.index.albumQueue as number[];
      // Another album may already be running; keep that queue head in place.
      queue.splice(Math.min(queue.length, Math.max(queue.length ? 1 : 0, position)), 0, id);
      if (!paused) removePausedAlbum(id);
      if (status === 1) syncIndexStatus();
      else if (queue.length === 1) libConfig.index.status = status;
    }
    useToast().error(t('album.edit.save_failed'));
  } finally {
    albumEditBusy.value = false;
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
  const removedAlbumIndex = albums.value.findIndex(album => Number(album.id) === Number(albumId));
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
    if (reorderingAlbumId.value === albumId) {
      reorderingAlbumId.value = null;
    }
    showAlbumEdit.value = false; // Close the edit dialog if it's open

    tauriEmit('albums-refreshed');
    tauriEmit('library-total-refreshed');

    const nextSelectedAlbum = removedAlbumIndex > 0
      ? albums.value[removedAlbumIndex - 1]
      : albums.value[0];
    if (nextSelectedAlbum) {
      await clickAlbum(nextSelectedAlbum);
    } else {
      selection.resetSelection();
    }
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

  selection.selectAlbum(album);
  requestAnimationFrame(() => {
    setTimeout(async () => {
      const isAccessible = await refreshAlbumAccess(album);
      if (!isAccessible) {
        if (!album.children) await loadCachedAlbumTree(album);
      } else if (!album.children) {
        const subFolders = await fetchFolder(album.path, false, config.settings.folderSort);
        if (subFolders) {
          album.children = [subFolders];
          // A leaf album still has a useful root-folder node for folder
          // actions and context. Show it when the album is selected.
          if (!subFolders.has_subfolders) {
            album.is_expanded = true;
          }
        }
      }
    }, 0);
  });
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
  
  await refreshAlbumAccess(album);
  if (album.is_expanded && album.is_accessible === false) {
    if (!album.children || forceRefresh) await loadCachedAlbumTree(album);
    return;
  }
  if (album.is_expanded && (!album.children || forceRefresh)) {
    const subFolders = await fetchFolder(album.path, false, config.settings.folderSort);
    if(subFolders) {
      album.children = [subFolders];
    }
  }
};

const toggleAlbumExpansion = async (album: Album) => {
  await expandAlbum(album);
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
  try {
    await reorderAlbums(albums.value.map((album, displayOrder) => ({ id: album.id, displayOrder })));
  } catch (error) {
    console.error('Failed to reorder albums:', error);
    const restoredAlbums = await getAllAlbums();
    if (Array.isArray(restoredAlbums)) {
      albums.value = restoredAlbums;
      await loadAlbumCovers();
    }
  }
}

function handleReorderOutsidePointerDown(event: PointerEvent) {
  if (event.button !== 0 || reorderingAlbumId.value === null || isDragging.value) return;
  if (event.target instanceof Element && event.target.closest('[data-reordering-album="true"]')) return;
  reorderingAlbumId.value = null;
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

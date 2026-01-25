<template>
    <!-- albums -->
    <ul v-if="albums.length > 0" class="flex-1 overflow-x-hidden overflow-y-auto rounded-box">
      
      <!-- title -->
      <div v-if="isMainPane" class="px-2 h-10 flex items-center text-sm text-base-content/30 cursor-default whitespace-nowrap">
        <span class="flex-1">{{ $t('album.album_list') }}</span>
        <TButton v-if="isEditList" 
          :icon="IconRestore"
          :selected="true"
          :buttonSize="'small'"
          @click="clickCloseEditList"
        />
      </div>
      
      <!-- drag to change albums' display order -->
      <VueDraggable 
        v-model="albums" 
        :disabled="!isMainPane || !isEditList"
        group="album-folder"
        :handle="'.drag-handle'" 
        :animation="200"
        @start="onDragStart"
        @end="onDragEnd" 
      >
        <li v-for="album in albums" :key="album.id">
          <div 
            :class="[
              'mx-1 p-1 h-12 flex items-center rounded-box whitespace-nowrap cursor-pointer group transition-all duration-200 ease-in-out', 
              selection.albumId.value === album.id && !isEditList 
                ? (selection.selected.value ? 'text-primary bg-base-100 hover:bg-base-100' : 'text-primary')
                : 'hover:text-base-content hover:bg-base-100/30',
            ]"
            @click="clickAlbum(album)"
            @dblclick="dlbClickAlbum(album)"
          >
            <div v-if="album.cover_file_id && albumCovers[album.id]" class="w-10 h-10 mr-2 shrink-0" @click.stop="dlbClickAlbum(album)">
              <img :src="albumCovers[album.id]" class="w-full h-full object-cover rounded-box">
            </div>
            <div v-else class="skeleton w-10 h-10 mr-2 shrink-0"></div>

            <div class="flex flex-col overflow-hidden" :class="{'text-base-content/30': isEditList }">  
              <div class="overflow-hidden whitespace-pre text-ellipsis">
                {{ album.name }} 
              </div>
              <div v-if="album.description" class="text-xs overflow-hidden whitespace-nowrap text-ellipsis">
                {{ album.description }}
              </div>
            </div>

            <!-- Right side: Count and Status Icons -->
            <div class="ml-auto pl-1 flex items-center justify-center text-xs text-base-content/30">
              <!-- <TButton v-if="album.indexed !== undefined && album.total !== undefined && album.indexed < album.total" 
                :icon="IconUpdate"
                :iconClasses="(libConfig.index.albumQueue as any).includes(album.id) ? ['animate-spin'] : []"
                :buttonSize="'small'"
                @click="clickIndexAlbum(album.id)"
              /> -->
              <div v-if="album.indexed !== undefined && album.total !== undefined && album.indexed < album.total" @click="clickIndexAlbum(album.id)">
                <component :is="libConfig.index.albumQueue[0] === album.id ? IconIndexRunning : IconIndexWaiting" class="mx-1 w-4 h-4 hover:text-base-content" />
              </div>
              <span v-else>
                {{ album.total.toLocaleString() }}
              </span>
            </div>  

            <div class="flex flex-row items-center text-base-content/30">
              <div v-if="isMainPane && !isEditList"
                :class="[
                  selection.albumId.value === album.id && selection.selected.value ? '' : 'hidden group-hover:block'
                ]"
              >
                <ContextMenu
                  :iconMenu="IconMore"
                  :menuItems="() => getMoreMenuItems(album)"
                  :smallIcon="true"
                />
              </div>
              <!-- dragging handle -->
              <div v-if="isEditList" class="drag-handle">
                <TButton 
                  :icon="IconDragHandle"
                  :buttonSize="'small'"
                  :selected="true"
                />
              </div>
            </div>
          </div>
          <transition
            enter-active-class="transition-all duration-200 ease-out overflow-hidden"
            enter-from-class="max-h-0"
            enter-to-class="max-h-96"
          >
            <div v-if="album.is_expanded && !isEditList && !(libConfig.index.albumQueue as any).includes(album.id)" class="mx-2 my-1 py-1 rounded-box bg-base-300/70">
              <AlbumFolder 
                :children="album.children" 
                :albumId="album.id"
                :rootPath="album.path"
              />
            </div>
          </transition>
        </li>
      </VueDraggable>

    </ul>

    <!-- No Albums Found Message -->
    <div v-else-if="!isLoading && !isEditList" class="mt-8 px-2 flex flex-col items-center justify-center gap-2 text-base-content/30">
      <span class="text-center">{{ $t('album.no_albums.title') }}</span>
      <span class="text-sm text-center">{{ $t('album.no_albums.description') }}</span>
      <button class="btn btn-primary rounded-box" @click="clickNewAlbum">
        <IconAdd class="w-5 h-5" />
        {{ $t('menu.album.add') }}
      </button>
    </div>

    <!-- edit album information -->
    <AlbumEdit
      v-if="showAlbumEdit"
      :isNewAlbum="isNewAlbum"
      :albumId="isNewAlbum ? 0 : selection.albumId.value"
      :inputName="isNewAlbum ? '' : selectedAlbum?.name"
      :inputDescription="isNewAlbum ? '' : selectedAlbum?.description"
      :albumPath="isNewAlbum ? '' : selectedAlbum?.path"
      :albumCoverFileId="isNewAlbum ? null : selectedAlbum?.cover_file_id"
      :createdAt="isNewAlbum ? '' : formatTimestamp(selectedAlbum?.created_at, $t('format.date_time'))"
      :modifiedAt="isNewAlbum ? '' : formatTimestamp(selectedAlbum?.modified_at, $t('format.date_time'))"
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

import { ref, watch, computed, onMounted, onBeforeUnmount } from 'vue';
import { useI18n } from 'vue-i18n';
import { VueDraggable } from 'vue-draggable-plus'
import { listen } from '@tauri-apps/api/event';
import { config, libConfig } from '@/common/config';
import { useUIStore } from '@/stores/uiStore';
import { scrollToFolder, formatTimestamp } from '@/common/utils';
import { getAllAlbums, setDisplayOrder, addAlbum, editAlbum, removeAlbum, 
         fetchFolder, expandFinalFolder, getFileThumb,
         getAlbum, listenIndexProgress, listenIndexFinished } from '@/common/api';
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
  IconDragHandle,
  IconEdit,
  IconRemove,
  IconUpdate,
  IconRestore,
  IconOrder,
  IconIndexReady,
  IconIndexRunning,
  IconIndexWaiting,
} from '@/common/icons';

const props = defineProps<{
  selectionSource: SelectionSource;
}>();

/// i18n
const { locale, messages } = useI18n();
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

let unlistenKeydown: () => void;

let unlistenAlbumCoverChanged: () => void;
let unlistenExpandAlbumFolder: (() => void) | undefined;
let unlistenIndexProgress: (() => void) | undefined;
let unlistenIndexFinished: (() => void) | undefined;

// Computed to check if we're in main album pane
const isMainPane = computed(() => props.selectionSource === 'album');

// message boxes
const showAlbumEdit = ref(false);           // show edit album
const showRemoveAlbumMsgbox = ref(false);   // show remove album

const albums = ref<Album[]>([]);
const albumCovers = ref<Record<number, string>>({});
const isNewAlbum = ref(false);
const isEditList = ref(false);  // edit album list
const isLoading = ref(true);    // loading albums
const isDragging = ref(false);  // dragging albums

const getAlbumById = (id: number) => albums.value.find(album => album.id === id);
const selectedAlbum = computed(() => getAlbumById(selection.albumId.value)) || {};

// Get menu items for a specific album (function for lazy evaluation)
const getMoreMenuItems = (album: any) => {
  return [
    {
      label: localeMsg.value.menu.album.edit,
      icon: IconEdit,
      action: () => {
        showAlbumEdit.value = true;
        isNewAlbum.value = false;
      }
    },
    {
      label: localeMsg.value.menu.album.index,
      icon: IconIndexReady,
      action: () => {
        clickIndexAlbum(album.id);
      }
    },
    {
      label: "-",   // separator
      action: () => {}
    },
    {
      label: localeMsg.value.menu.album.reorder,
      icon: IconOrder,
      action: () => {
        isEditList.value = true;
        uiStore.pushInputHandler('AlbumList-edit');
      }
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
const loadAlbumCover = async (albumId: number, coverFileId: number | null) => {
  if (coverFileId) {
    try {
      const thumb = await getFileThumb(coverFileId, "", 0, 0, config.settings.thumbnailSize, false);
      if (thumb && thumb.error_code === 0) {
        albumCovers.value[albumId] = `data:image/jpeg;base64,${thumb.thumb_data_base64}`;
      }
    } catch (error) {
      console.error(`Failed to load cover for album ${albumId}:`, error);
    }
  } else {
    delete albumCovers.value[albumId];
  }
};

const loadAlbumCovers = async () => {
  for (const album of albums.value) {
    await loadAlbumCover(album.id, album.cover_file_id);
  }
};

onMounted( async () => {
  unlistenKeydown = await listen('global-keydown', handleKeyDown);

  if (albums.value.length === 0) {
    albums.value = await getAllAlbums(true);
    await loadAlbumCovers();
    isLoading.value = false;

    if (selection.albumId.value > 0) {
      // expand and select the current album and folder
      clickFinalSubFolder(selection.albumId.value, selection.folderPath.value);
    }
  }

  // If in selection mode (MoveTo/CopyTo), auto-expand all albums
  if (!isMainPane.value && albums.value.length > 0) {
    Promise.all(albums.value.map(album => expandAlbum(album, true)));
  }

  // listen for album-cover-changed event
  unlistenAlbumCoverChanged = await listen('album-cover-changed', async (event: any) => {
    const { albumId: eventAlbumId, fileId } = event.payload;
    const album = getAlbumById(eventAlbumId);
    if (album) {
      if (fileId) {
        // manual update
        album.cover_file_id = fileId;
      } else {
        // indexing finished update, reload album to get new cover
        const updatedAlbums = await getAllAlbums(true);
        const updatedAlbum = updatedAlbums.find(a => a.id === eventAlbumId);
        if (updatedAlbum) {
          album.cover_file_id = updatedAlbum.cover_file_id;
        }
      }
      
      // Update the cover in albumCovers
      await loadAlbumCover(eventAlbumId, album.cover_file_id);
    }
  });

  // listen for expand-album-folder event (from Content.vue "Find Album Folder" action)
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
        
        // Reload the cover thumbnail
        await loadAlbumCover(album_id, album.cover_file_id);
      }
    }
  });

});

onBeforeUnmount(() => {
  if (unlistenKeydown) unlistenKeydown();
  if (unlistenAlbumCoverChanged) unlistenAlbumCoverChanged();
  if (unlistenExpandAlbumFolder) unlistenExpandAlbumFolder();
  if (unlistenIndexProgress) unlistenIndexProgress();
  if (unlistenIndexFinished) unlistenIndexFinished();
});

// Update album count in config
watch(() => albums.value.length, (newCount) => {
  config.main.albumCount = newCount;
});

/// Add a new album
const clickNewAlbum = async () => {
  showAlbumEdit.value = true;
  isNewAlbum.value = true;
};

// Refresh albums function
const refreshAlbums = async () => {
  isLoading.value = true;
  try {
    albums.value = await getAllAlbums(true);
  } catch (error) {
    console.error('Failed to refresh albums:', error);
  } finally {
    isLoading.value = false;
    
    selection.albumId.value = 0;      // show all files
    selection.folderPath.value = "";
    selection.selected.value = false;
  }
};

/// edit album information or add new album
const clickEditAlbum = async (folderPathParam: string, newName: string, newDescription: string, isNew: boolean) => {
  if (isNew) {
    // Add new album
    const newAlbum = await addAlbum(folderPathParam);
    if (newAlbum) {
      // Update album name and description if different from folder name
      if (newName !== newAlbum.name || newDescription) {
        await editAlbum(newAlbum.id, newName, newDescription, false);
        newAlbum.name = newName;
        newAlbum.description = newDescription;
      }
      albums.value.push(newAlbum);
      clickAlbum(newAlbum);
      showAlbumEdit.value = false;
      
      // add the new album to the index queue
      libConfig.index.albumQueue.push(newAlbum.id);   
    }
  } else {
    // Edit existing album
    const result = await editAlbum(selection.albumId.value, newName, newDescription);
    if(result && selectedAlbum.value) {
      selectedAlbum.value.name = newName;
      selectedAlbum.value.description = newDescription;
      showAlbumEdit.value = false;
    }
  }
};

/// Index an album
const clickIndexAlbum = async (albumId: number) => {
  if (!(libConfig.index.albumQueue as any).includes(albumId)) {
    libConfig.index.albumQueue.push(albumId);
    libConfig.index.status = 1;
  }
}

/// Remove an album from the list
const clickRemoveAlbum = async () => {
  const removedAlbum = await removeAlbum(selection.albumId.value);
  if(removedAlbum) {
    showRemoveAlbumMsgbox.value = false;

    // remove the album from the list
    albums.value = albums.value.filter(album => album.id !== selection.albumId.value);
    showAlbumEdit.value = false; // Close the edit dialog if it's open

    selection.resetSelection();
  }
};

/// click a album to select it
const clickAlbum = async (album: Album) => {
  if(isEditList.value) {
    return;
  }

  // In MoveTo dialog, disable album selection and toggle expansion instead
  if (!isMainPane.value) {
    expandAlbum(album);
    return;
  }

  selection.selectAlbum(album);
};

/// dlb click album to select it and expand/collapse its folders
const dlbClickAlbum = async (album: any) => {
  clickAlbum(album);
  expandAlbum(album);
};

/// click album icon to expand or collapse next level folders
const expandAlbum = async (album: any, forceRefresh = false) => {
  const willExpand = forceRefresh ? true : !album.is_expanded;
  
  // Collapse all other albums when expanding one (accordion behavior)
  // Only enabled in Main Pane to keep UI clean. In MoveTo dialog, allow multiple expansions.
  if (willExpand && isMainPane.value) {
    albums.value.forEach(a => {
      if (a.id !== album.id) {
        a.is_expanded = false;
      }
    });
  }
  
  album.is_expanded = willExpand; 
  
  if (album.is_expanded && (!album.children || forceRefresh)) {
    const subFolders = await fetchFolder(album.path, false);
    if(subFolders) {
      album.children = [subFolders];
    }
  }
};

/// click folder to select
const clickFolder = async (albumIdVal: number, folder: Folder) => {
  console.log('AlbumList.vue-clickFolder:', folder);
  await selection.selectFolder(albumIdVal, folder);
};

/// click the final sub-folder to select it
const clickFinalSubFolder = async (albumIdVal: number, folderPathVal: string) => {

  console.log('AlbumList.vue-clickFinalSubFolder:', albumIdVal, folderPathVal);
  let album = getAlbumById(albumIdVal);
  if(!album) {
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
};

const onDragEnd = async () => {
  isDragging.value = false;
  
  // update the display order of albums
  for (let i = 0; i < albums.value.length; i++) {
    await setDisplayOrder(albums.value[i].id, i);
  }
}

const clickCloseEditList = () => {
  isEditList.value = false;
  uiStore.removeInputHandler('AlbumList-edit');
};

const handleKeyDown = (event: KeyboardEvent) => {
  if (isEditList.value && event.payload.key === 'Escape') {
    clickCloseEditList();
  }
};

// Expose methods
defineExpose({ 
  isEditList,
  clickNewAlbum,
  refreshAlbums,
  clickFinalSubFolder,
});

</script>
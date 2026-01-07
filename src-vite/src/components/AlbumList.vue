<template>
    <!-- albums -->
    <ul v-if="albums.length > 0" class="flex-1 overflow-x-hidden overflow-y-auto rounded-box">
      
      <!-- title -->
      <div v-if="componentId === 0" class="px-2 h-10 flex items-center text-sm text-base-content/30 cursor-default whitespace-nowrap">
        {{ $t('album.album_list') }}
      </div>
      
      <!-- drag to change albums' display order -->
      <VueDraggable 
        v-model="albums" 
        :disabled="componentId !== 0 || !isEditList"
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
              config.album.id === album.id && !isEditList 
                ? (config.album.selected ? 'text-primary bg-base-100 hover:bg-base-100' : 'text-primary')
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
            <div class="ml-auto pl-1 mr-1 flex flex-col items-end justify-center text-xs text-base-content/30 space-y-0.5">
               <span v-if="album.total > 0">
                  {{ album.scanned >= album.total ? album.total.toLocaleString() : album.scanned.toLocaleString() + '/' + album.total.toLocaleString() }}
               </span>
               <div class="flex flex-row items-center space-x-1">
                  <IconFavorite v-if="album.is_favorite" class="w-4 h-4 min-mt-0" />
                  <IconHide v-if="album.is_hidden" class="w-4 h-4 min-mt-0" />
               </div>
            </div>  

            <div class="flex flex-row items-center text-base-content/30">
              <div v-if="componentId === 0 && !isEditList && !config.scan.albumQueue.includes(album.id)"
                :class="[
                  config.album.id === album.id && config.album.selected ? '' : 'hidden group-hover:block'
                ]"
              >
                <ContextMenu
                  :iconMenu="IconMore"
                  :menuItems="() => getMoreMenuItems(album)"
                  :smallIcon="true"
                />
              </div>
              <!-- when scanning, replace context menu with Scanning Icon -->
              <IconUpdate v-if="config.scan.albumQueue.includes(album.id)" 
                :class="['mx-1 w-4 h-4', config.scan.albumQueue[0] === album.id ? 'animate-spin' : '']" 
              />
              <!-- dragging handle -->
              <TButton v-if="isEditList" 
                class="drag-handle"
                :icon="IconDragHandle"
                :buttonSize="'small'"
                :selected="true"
              />
            </div>
          </div>
          <div v-if="album.is_expanded && !isEditList && !config.scan.albumQueue.includes(album.id)"  class="mx-2 my-1 py-1 rounded-box bg-base-300/70">
            <AlbumFolder 
              :children="album.children" 
              :isHiddenAlbum="album.is_hidden ? true : false"
              :albumId="album.id"
              :folderPath="album.path"
              :componentId="componentId"
            />
          </div>
        </li>
      </VueDraggable>

    </ul>

    <!-- No Albums Found Message -->
    <div v-else-if="!isLoading && !isEditList" class="mt-8 px-2 flex flex-col items-center justify-center gap-2 text-base-content/30">
      <span class="text-center">{{ $t('album.no_albums.title') }}</span>
      <span class="text-sm text-center">{{ $t('album.no_albums.description') }}</span>
      <button class="btn btn-primary" @click="clickNewAlbum">
        <IconAdd class="w-5 h-5" />
        {{ $t('menu.album.add') }}
      </button>
    </div>

    <!-- edit album information -->
    <AlbumEdit
      v-if="showAlbumEdit"
      :isNewAlbum="isNewAlbum"
      :albumId="isNewAlbum ? 0 : config.album.id"
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
      :errorMessage="errorMessage"
      @ok="clickNewFolder"
      @cancel="showNewFolderMsgbox = false"
      @reset="errorMessage = ''"
    />

    <ToolTip ref="toolTipRef" />

</template>

<script setup lang="ts">

import { ref, watch, computed, onMounted, onBeforeUnmount } from 'vue';
import { listen, emit as tauriEmit } from '@tauri-apps/api/event';
import { useI18n } from 'vue-i18n';
import { VueDraggable } from 'vue-draggable-plus'
import { config } from '@/common/config';
import { isMac, scrollToFolder, formatTimestamp } from '@/common/utils';
import { getAllAlbums, setDisplayOrder, addAlbum, editAlbum, removeAlbum, 
         createFolder, selectFolder, fetchFolder, expandFinalFolder, revealFolder, setFolderFavorite, getFileThumb,
         getAlbum, listenScanProgress, listenScanFinished } from '@/common/api';

import AlbumFolder from '@/components/AlbumFolder.vue';
import AlbumEdit from '@/components/AlbumEdit.vue';
import ContextMenu from '@/components/ContextMenu.vue';
import MessageBox from '@/components/MessageBox.vue';
import ToolTip from '@/components/ToolTip.vue';
import TButton from '@/components/TButton.vue';

import {
  IconAdd,
  IconNewFolder,
  IconMore,
  IconDragHandle,
  IconEdit,
  IconRemove,
  IconUnhide,
  IconHide,
  IconFavorite,
  IconUnFavorite,
  IconUpdate,
  IconRestore,
} from '@/common/icons';

const props = defineProps({
  componentId: {  // 0: album pane; 1: move/copy to mode(select destination folder)
    type: Number,
    default: 0
  }
});

/// i18n
const { locale, messages } = useI18n();
const localeMsg = computed(() => messages.value[locale.value] as any);

let unlistenAlbumCoverChanged: () => void;
let unlistenScanProgress: (() => void) | undefined;
let unlistenScanFinished: (() => void) | undefined;

// message boxes
const showAlbumEdit = ref(false);           // show edit album
const showRemoveAlbumMsgbox = ref(false);   // show remove album
const showNewFolderMsgbox = ref(false);     // show new folder
const errorMessage = ref('');

const toolTipRef = ref(null);

const albums = ref<any[]>([]);
const albumCovers = ref<Record<number, string>>({});
const isNewAlbum = ref(false);
const isEditList = ref(false);  // edit album list
const isLoading = ref(true);    // loading albums
const isDragging = ref(false);  // dragging albums

const getAlbumById = (id: number) => albums.value.find(album => album.id === id);
const selectedAlbum = computed(() => getAlbumById(config.album.id)) || {};

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
      label: localeMsg.value.menu.album.scan,
      icon: IconUpdate,
      action: () => {
        if (!config.scan.albumQueue.includes(album.id)) {
          config.scan.albumQueue.push(album.id);
          config.scan.status = 1;
        }
      }
    },
    {
      label: !album?.is_hidden ? localeMsg.value.menu.album.exclude_from_search : localeMsg.value.menu.album.include_in_search,
      icon: !album?.is_hidden ? IconHide : IconUnhide,
      action: () => {
        toggleHidden();
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
  if (albums.value.length === 0) {
    albums.value = await getAllAlbums(true);
    await loadAlbumCovers();
    isLoading.value = false;

    if (config.album.id > 0) {
      // expand and select the current album and folder
      clickFinalSubFolder(config.album.id, config.album.folderPath);
    }
  }

  // listen for album-cover-changed event
  unlistenAlbumCoverChanged = await listen('album-cover-changed', async (event: any) => {
    const { albumId, fileId } = event.payload;
    const album = getAlbumById(albumId);
    if (album) {
      if (fileId) {
        // manual update
        album.cover_file_id = fileId;
      } else {
        // scan finished update, reload album to get new cover
         const updatedAlbums = await getAllAlbums(true);
         const updatedAlbum = updatedAlbums.find(a => a.id === albumId);
         if (updatedAlbum) {
             album.cover_file_id = updatedAlbum.cover_file_id;
         }
      }
      
      // Update the cover in albumCovers
      await loadAlbumCover(albumId, album.cover_file_id);
    }
  });

  // listen for scan progress
  unlistenScanProgress = await listenScanProgress(async (event: any) => {
    const { album_id, current, total } = event.payload;
    const album = getAlbumById(album_id);
    if (album) {
      album.scanned = current;
      album.total = total;
    }
  });

  // listen for scan finished
  unlistenScanFinished = await listenScanFinished(async (event: any) => {
    const { album_id } = event.payload;
    const album = getAlbumById(album_id);
    if (album) {
      const updatedAlbum = await getAlbum(album_id);
      if (updatedAlbum) {
        album.scanned = updatedAlbum.scanned;
        album.total = updatedAlbum.total;
        album.cover_file_id = updatedAlbum.cover_file_id;
        
        // Reload the cover thumbnail
        await loadAlbumCover(album_id, album.cover_file_id);
      }
    }
  });

});

onBeforeUnmount(() => {
  if (unlistenAlbumCoverChanged) unlistenAlbumCoverChanged();
  if (unlistenScanProgress) unlistenScanProgress();
  if (unlistenScanFinished) unlistenScanFinished();
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
    
    config.album.id = 0;      // show all files
    config.album.folderPath = "";
    config.album.selected = false;
  }
};

/// edit album information or add new album
const clickEditAlbum = async (folderPath: string, newName: string, newDescription: string, isNew: boolean) => {
  if (isNew) {
    // Add new album
    const newAlbum = await addAlbum(folderPath);
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
      
      // add the new album to the scan queue
      config.scan.albumQueue.push(newAlbum.id);   
    }
  } else {
    // Edit existing album
    const result = await editAlbum(config.album.id, newName, newDescription, selectedAlbum.value.is_hidden ?? false);
    if(result) {
      selectedAlbum.value.name = newName;
      selectedAlbum.value.description = newDescription;
      showAlbumEdit.value = false;
    }
  }
};

/// Remove an album from the list
const clickRemoveAlbum = async () => {
  const removedAlbum = await removeAlbum(config.album.id);
  if(removedAlbum) {
    showRemoveAlbumMsgbox.value = false;

    // remove the album from the list
    albums.value = albums.value.filter(album => album.id !== config.album.id);
    showAlbumEdit.value = false; // Close the edit dialog if it's open

    config.album.id = 0;
    config.album.folderPath = "";
    config.album.selected = false;
  }
};

/// click a album to select it
const clickAlbum = async (album: any) => {
  if(isEditList.value) {
    return;
  }

  config.album.id = album.id;
  config.album.folderPath = album.path;
  config.album.selected = true;
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
  if (willExpand) {
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
const clickFolder = async (albumId: number, folder: any) => {
  console.log('AlbumList.vue-clickFolder:', folder);
  const selectedFolder = await selectFolder(albumId, folder.path);
  if(selectedFolder) {
    config.album.id = albumId;
    config.album.folderId = selectedFolder.id;
    config.album.folderPath = selectedFolder.path;
    config.album.selected = false;

  } else {
    toolTipRef.value.showTip(localeMsg.value.msgbox.select_folder.error);
  }
};

/// click the final sub-folder to select it
const clickFinalSubFolder = async (albumId: number, folderPath: string) => {

  console.log('AlbumList.vue-clickFinalSubFolder:', albumId, folderPath);
  let album = getAlbumById(albumId);
  if(!album) {
    return;
  }

  if (config.album.selected) {  // album is selected
    clickAlbum(album);
  } else {    // album's sub-folder is selected
    // expand the album's folder
    await expandAlbum(album, true);

    // recursively expand the final sub-folder path
    expandFinalFolder(album, folderPath).then((folder) => {
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

// toggle hidden album
const toggleHidden = async () => {
  selectedAlbum.value.is_hidden = !selectedAlbum.value.is_hidden;
  await editAlbum(selectedAlbum.value.id, selectedAlbum.value.name, selectedAlbum.value.description, selectedAlbum.value.is_hidden);
};

// Expose methods
defineExpose({ 
  isEditList,
  clickNewAlbum,
  refreshAlbums,
});

</script>
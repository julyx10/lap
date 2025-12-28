<template>
    <!-- albums -->
    <ul v-if="albums.length > 0" class="flex-1 overflow-x-hidden overflow-y-auto rounded-box">

      <!-- drag to change albums' display order -->
      <VueDraggable 
        v-model="albums" 
        :disabled="componentId !== 0 && !isEditList"
        group="album-folder"
        :handle="'.drag-handle'" 
        :animation="200"
        @start="onDragStart"
        @end="onDragEnd" 
      >
        <li v-for="album in albums" :key="album.id">
          <div 
            :class="[
              'mx-1 p-1 h-10 flex items-center rounded-box whitespace-nowrap cursor-pointer group', 
              selectedFolderId === album.folderId && !isEditList ? 'text-primary bg-base-100 hover:bg-base-100' : (config.index.albumQueue.includes(album.id) ? '' : 'hover:text-base-content hover:bg-base-100/30'),
              config.index.albumQueue.includes(album.id) ? 'text-base-content/30 pointer-events-none' : ''
            ]"
            @click="clickAlbum(album)"
            @dblclick="dlbClickAlbum(album)"
          >
            <!-- Indexing Icon -->
            <IconUpdate v-if="config.index.albumQueue.includes(album.id)" 
              :class="['mx-1 w-5 h-5', config.index.albumQueue[0] === album.id ? 'animate-spin' : '']" 
            />
            <component v-else :is="album.is_expanded && !isEditList ? IconFolderExpanded : IconFolder" 
              class="mx-1 w-5 h-5 cursor-pointer shrink-0" 
              @click.stop="expandAlbum(album)"
            />

            <div class="overflow-hidden whitespace-pre text-ellipsis">
              {{ album.name }}
              <span v-if="config.index.albumQueue.includes(album.id)" class="text-xs">
                 {{  config.index.albumQueue[0] === album.id 
                     ? `(${$t('search.indexing')} ${config.index.indexed.toLocaleString()} / ${config.index.total.toLocaleString()})` 
                     : `(${$t('search.indexing')})` 
                 }}
              </span>
            </div>
            <div class="ml-auto flex flex-row items-center text-base-content/30">
              <TButton v-if="album.is_hidden" 
                :icon="IconHide"
                :disabled="true"
                :buttonSize="'small'"
              />
              <TButton v-if="album.is_favorite" 
                :icon="IconFavorite"
                :disabled="true"
                :buttonSize="'small'"
              />
              <TButton v-if="isEditList" 
                class="drag-handle"
                :icon="IconDragHandle"
                :buttonSize="'small'"
              />
              <ContextMenu v-else-if="componentId === 0 && !isDragging && !config.index.albumQueue.includes(album.id)"
                :class="['',
                  !selectedFolderId || selectedFolderId != album.folderId ? 'invisible group-hover:visible' : ''
                ]"
                :iconMenu="IconMore"
                :menuItems="() => getMoreMenuItems(album)"
                :smallIcon="true"
              />
              <TButton v-if="config.index.albumQueue.includes(album.id)" 
                class="pointer-events-auto"
                :icon="IconRestore"
                :buttonSize="'small'"
                @click="cancelIndexing(album.id)"
              />
            </div>
          </div>
          <AlbumFolder v-if="album.is_expanded && !isEditList"
            :children="album.children" 
            :rootAlbumId="album.id"
            :isHiddenAlbum="album.is_hidden ? true : false"
            :albumId="selectedAlbumId"
            :folderId="selectedFolderId"
            :folderPath="selectedFolderPath"
            :componentId="componentId"
          />
        </li>
      </VueDraggable>

    </ul>

    <!-- No Albums Found Message -->
    <div v-else-if="!isLoading && !isEditList" class="flex-1 flex flex-col items-center justify-center text-base-content/30">
      <button class="btn btn-primary" @click="clickNewAlbum">
        <IconAdd class="w-5 h-5" />
        {{ $t('menu.album.add') }}
      </button>

      <span class="m-4 text-center">{{ $t('tooltip.not_found.albums') }}</span>
    </div>

    <!-- edit album information -->
    <AlbumEdit
      v-if="showAlbumEdit"
      :isNewAlbum="isNewAlbum"
      :albumId="isNewAlbum ? 0 : albumId"
      :inputName="isNewAlbum ? '' : getAlbumById(albumId)?.name"
      :inputDescription="isNewAlbum ? '' : getAlbumById(albumId)?.description"
      :albumPath="isNewAlbum ? '' : getAlbumById(albumId)?.path"
      :createdAt="isNewAlbum ? '' : formatTimestamp(getAlbumById(albumId)?.created_at, $t('format.date_time'))"
      :modifiedAt="isNewAlbum ? '' : formatTimestamp(getAlbumById(albumId)?.modified_at, $t('format.date_time'))"
      @ok="clickAlbumInfo"
      @cancel="showAlbumEdit = false"
    />

    <!-- Remove album dialog -->
    <MessageBox
      v-if="showRemoveAlbumMsgbox"
      :title="$t('msgbox.remove_album.title')"
      :message="$t('msgbox.remove_album.content', { album: getAlbumById(albumId).name })"
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
         createFolder, selectFolder, fetchFolder, expandFinalFolder, revealFolder, setFolderFavorite } from '@/common/api';

import AlbumFolder from '@/components/AlbumFolder.vue';
import AlbumEdit from '@/components/AlbumEdit.vue';
import ContextMenu from '@/components/ContextMenu.vue';
import MessageBox from '@/components/MessageBox.vue';
import ToolTip from '@/components/ToolTip.vue';
import TButton from '@/components/TButton.vue';

import {
  IconAdd,
  IconFolder,
  IconFolderExpanded,
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
  albumId: {    // album id
    type: Number,
    required: false
  },
  folderId: {   // folder id
    type: Number,
    required: false
  },
  folderPath: { // folder path
    type: String,
    required: false
  },
  componentId: {  // 0: album pane; 1: move/copy to mode(select destination folder)
    type: Number,
    default: 0
  }
});

/// i18n
const { locale, messages } = useI18n();
const localeMsg = computed(() => messages.value[locale.value]);

let unlistenSelectFolder: () => void;

const selectedAlbumId = ref(0);
const selectedFolderId = ref(0);
const selectedFolderPath = ref('');

// message boxes
const showAlbumEdit = ref(false);           // show edit album
const showRemoveAlbumMsgbox = ref(false);   // show remove album
const showNewFolderMsgbox = ref(false);     // show new folder
const errorMessage = ref('');

const toolTipRef = ref(null);

const albums = ref<any[]>([]);
const isNewAlbum = ref(false);
const isEditList = ref(false);  // edit album list
const isLoading = ref(true);    // loading albums
const isDragging = ref(false);  // dragging albums

const getAlbumById = (id: number) => albums.value.find(album => album.id === id);

const emit = defineEmits(['update:albumId', 'update:folderId', 'update:folderPath']);

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
      label: localeMsg.value.menu.album.update_index,
      icon: IconUpdate,
      action: () => {
        if (!config.index.albumQueue.includes(album.id)) {
          config.index.albumQueue.push(album.id);
          config.index.status = 1;
        }
      }
    },
    {
      label: !album?.is_hidden ? localeMsg.value.menu.album.exclude_from_search : localeMsg.value.menu.album.include_in_search,
      icon: !album?.is_hidden ? IconHide : IconUnhide,
      action: () => {
        toggleHidden(album);
      }
    },
    {
      label: localeMsg.value.menu.album.remove,
      icon: IconRemove,
      action: () => {
        showRemoveAlbumMsgbox.value = true;
      }
    },
    {
      label: "-",   // separator
      action: () => {}
    },
    {
      label: localeMsg.value.menu.file.new_folder,
      icon: IconNewFolder,
      action: () => {
        showNewFolderMsgbox.value = true;
      }
    },
    {
      label: isMac ? localeMsg.value.menu.file.reveal_in_finder : localeMsg.value.menu.file.reveal_in_file_explorer,
      // icon: IconExternal,
      action: () => {
        revealFolder(album.path);
      }
    },
    {
      label: "-",   // separator
      action: () => {}
    },
    {
      label: !album?.is_favorite ? localeMsg.value.menu.meta.favorite : localeMsg.value.menu.meta.unfavorite,
      icon: !album?.is_favorite ? IconFavorite : IconUnFavorite,
      action: () => {
        toggleFavorite(album);
      }
    },
  ];
};

onMounted( async () => {
  if (albums.value.length === 0) {
    albums.value = await getAllAlbums(true);
    isLoading.value = false;

    if (props.albumId > 0) {
      // expand and select the current album and folder
      clickFinalSubFolder(props.albumId, props.folderPath);
    }
  }

  // listen for messages from AlbumFolder component
  unlistenSelectFolder = await listen('message-from-select-folder', async(event) => {
    console.log('listen - message-from-select-folder:', event);
    const { message } = event.payload;
    switch (message) {
      case 'click-folder':
      case 'rename-folder':
      case 'trash-folder':
        if(event.payload.componentId === props.componentId) {
          selectedAlbumId.value = event.payload.albumId;
          selectedFolderId.value = event.payload.folderId;
          selectedFolderPath.value = event.payload.folderPath;
        };
        break;
      case 'refresh-folder':
        for (let album of albums.value) {
          if(album.id === config.destFolder.albumId) {
            clickFinalSubFolder(album.id, event.payload.folderPath);  // select the sub-folder
            break;
          }
        }
        break;
      default:
        break;
    }
  });

});

onBeforeUnmount(() => {
  unlistenSelectFolder();
});

watch(() => [ props.albumId, props.folderId, props.folderPath ], ([ newAlbumId, newFolderId, newFolderPath ]) => {
  selectedAlbumId.value = Number(newAlbumId) || 0;
  selectedFolderId.value = Number(newFolderId) || 0;
  selectedFolderPath.value = String(newFolderPath || '');
}, { immediate: true });

watch(() => [ selectedAlbumId.value, selectedFolderId.value, selectedFolderPath.value ], ([ newAlbumId, newFolderId, newFolderPath ]) => {
  emit('update:albumId', newAlbumId);
  emit('update:folderId', newFolderId);
  emit('update:folderPath', newFolderPath);
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
    selectedAlbumId.value = -1;
    selectedFolderId.value = -1;
    selectedFolderPath.value = "";
  }
};

/// edit album information or add new album
const clickAlbumInfo = async (folderPath, newName, newDescription, isNew) => {
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
      
      // add the new album to the index queue
      config.index.albumQueue.push(newAlbum.id);   
    }
  } else {
    // Edit existing album
    let album = getAlbumById(selectedAlbumId.value);
    const result = await editAlbum(selectedAlbumId.value, newName, newDescription, album.is_hidden ?? false);
    if(result) {
      album.name = newName;
      album.description = newDescription;
      showAlbumEdit.value = false;
    }
  }
};

/// Remove an album from the list
const clickRemoveAlbum = async () => {
  const removedAlbum = await removeAlbum(selectedAlbumId.value);
  if(removedAlbum) {
    showRemoveAlbumMsgbox.value = false;

    // remove the album from the list
    albums.value = albums.value.filter(album => album.id !== selectedAlbumId.value);
    showAlbumEdit.value = false; // Close the edit dialog if it's open

    selectedAlbumId.value = 0;
    selectedFolderId.value = 0;
    selectedFolderPath.value = "";
  }
};

/// click a album to select it
const clickAlbum = async (album) => {
  if(isEditList.value) {
    return;
  }

  const selectedFolder = await selectFolder(album.id, album.path);
  if(selectedFolder) {
    // insert a new property(album.folderId) 
    album.folderId = selectedFolder.id;

    selectedAlbumId.value = album.id;
    selectedFolderId.value = selectedFolder.id;
    selectedFolderPath.value = selectedFolder.path;
  }
};

/// dlb click album to select it and expand/collapse its folders
const dlbClickAlbum = async (album) => {
  clickAlbum(album);
  expandAlbum(album);
};

/// cancel indexing
const cancelIndexing = async (albumId) => {
  const index = config.index.albumQueue.indexOf(albumId);
  if (index === -1) return;

  if (index === 0) {
    // Active one: remove and restart processing for next
    config.index.albumQueue.shift();
    config.index.indexed = 0;
    config.index.total = 0;
    
    // reset status
    config.index.status = 0;
    
    // trigger Content.vue to process next using global event
    await tauriEmit('trigger-next-album');
    
  } else {
    // Pending one: just remove
    config.index.albumQueue.splice(index, 1);
  }
};

/// click album icon to expand or collapse next level folders
const expandAlbum = async (album, forceRefresh = false) => {
  album.is_expanded = forceRefresh ? true : !album.is_expanded; 
  
  if (album.is_expanded && (!album.children || forceRefresh)) {
    const subFolders = await fetchFolder(album.path, false);
    if(subFolders) {
      album.children = subFolders.children;
    }
  }
};

/// Create new folder
const clickNewFolder = async (folderName) => {
  const newFolderPath = await createFolder(selectedFolderPath.value, folderName);
  if(newFolderPath) {
    let album = getAlbumById(selectedAlbumId.value);
    let newFolder = -1;

    if(!album.children) {
      await expandAlbum(album);
      newFolder = album.children.find(folder => folder.name === folderName);
    } else {
      album.is_expanded = true;
      album.children.push({ name: folderName, path: newFolderPath });
      newFolder = album.children[album.children.length - 1];
    }

    // select the new folder
    clickFolder(album.id, newFolder).then(() => {
      // scroll to the new folder
      scrollToFolder(newFolder.id);
    });

    // close the message box
    showNewFolderMsgbox.value = false;

    errorMessage.value = '';
  } else {
    errorMessage.value = localeMsg.value.msgbox.new_folder.error;
  }
};

/// click folder to select
const clickFolder = async (albumId, folder) => {
  console.log('AlbumList.vue-clickFolder:', folder);
  const selectedFolder = await selectFolder(albumId, folder.path);
  if(selectedFolder) {
    selectedAlbumId.value = albumId;
    selectedFolderId.value = selectedFolder.id;
    selectedFolderPath.value = selectedFolder.path;

    // insert new property 'id' to folder object
    folder.id = selectedFolder.id;
  } else {
    toolTipRef.value.showTip(localeMsg.value.msgbox.select_folder.error);
  }
};

/// click the final sub-folder to select it
const clickFinalSubFolder = async (albumId, folderPath) => {

  console.log('AlbumList.vue-clickFinalSubFolder:', albumId, folderPath);
  let album = getAlbumById(albumId);
  if(!album) {
    return;
  }

  if (album.path === folderPath) {  // album is selected
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

// toggle favorite album
const toggleFavorite = async (album?: any) => {
  // If album not provided, fallback to selectedAlbumId
  if (!album) {
    album = getAlbumById(selectedAlbumId.value);
  }
  if (album) {
    album.is_favorite = !album.is_favorite;
    // An album is essentially a folder, and usually album.folderId (if set) would be the one. 
    // However, 'addAlbum' logic suggests album.path is the root. 
    // Important: The user said "album has a folder record in afolder db".
    // Usually albums table has a folder_id or we use the folder associated with it.
    // If 'album' object from 'getAllAlbums' join fetches folder info (like is_favorite), 
    // we need to know the 'folderId' corresponding to the album root.
    // 'getAllAlbums' in api.js calls 'get_all_albums'.
    // If we look at 'addAlbum' -> 'add_album', it takes 'folderPath'.
    // Let's assume album.id IS NOT the folder id. 
    // But 'selectFolder(album.id, album.path)' returns a selectedFolder with 'id'.
    // Let's check 'clickAlbum'. It calls 'selectFolder' and sets 'album.folderId = selectedFolder.id'.
    // So 'album.folderId' should function after selection.
    
    // If album.folderId is missing (e.g. contextual menu on unselected album?), we might need to fetch it?
    // But context menu shows for 'selectedAlbumId'. So we likely have clicked it.
    
    if (album.folderId) {
       await setFolderFavorite(album.folderId, album.is_favorite);
    } else {
       // Fallback: Try to get folder id from path or select it implicitly? 
       // For now, let's assume valid selection populated folderId.
       const selectedFolder = await selectFolder(album.id, album.path);
       if(selectedFolder) {
         album.folderId = selectedFolder.id;
         await setFolderFavorite(album.folderId, album.is_favorite);
       }
    }
  }
};

// toggle hidden album
const toggleHidden = async (album?: any) => {
  // If album not provided, fallback to selectedAlbumId
  if (!album) {
    album = getAlbumById(selectedAlbumId.value);
  }
  if (album) {
    album.is_hidden = !album.is_hidden;
    await editAlbum(album.id, album.name, album.description, album.is_hidden);
  }
};

// Expose methods
defineExpose({ 
  isEditList,
  clickNewAlbum,
  refreshAlbums,
});

</script>
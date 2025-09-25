<template>
    <!-- albums -->
    <ul v-if="albums.length > 0" class="flex-1 overflow-x-hidden overflow-y-auto rounded-lg">

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
              'my-1 mr-1 pr-1 h-8 flex flex-row items-center rounded border-l-2 whitespace-nowrap cursor-pointer group', 
              selectedFolderId === album.folderId && !isEditList ? 'bg-base-content/10 border-primary' : 'hover:bg-base-content/10 border-transparent',
              album.is_hidden ? 'text-base-content/30' : '',
              { 
                'hover:bg-base-content/10': !isDragging && !isEditList,
                'text-base-content': selectedAlbumId === album.id && !isEditList, 
              }
            ]"
            @click="clickAlbum(album)"
            @dblclick="dlbClickAlbum(album)"
          >
            <TButton v-if="isEditList" 
              :icon="IconRemove"
              :buttonSize="'small'"
              :buttonClasses="'text-error'"
              @click.stop="removingAlbumId = album.id; showRemoveMsgbox = true"
            />
            <component v-else :is="album.is_expanded ? IconFolderExpanded : IconFolder" 
              class="mx-1 w-5 h-5 t-icon-animate hover:text-base-content cursor-pointer shrink-0" 
              @click.stop="expandAlbum(album)"
            />

            <div class="overflow-hidden whitespace-pre text-ellipsis">
              {{ album.name }}
            </div>
            <TButton v-if="isEditList" 
              class="ml-auto drag-handle"
              :icon="IconDragHandle"
              :buttonSize="'small'"
            />
            <DropDownMenu v-else-if="componentId === 0 && !isDragging"
              :class="[
                'ml-auto pl-1',
                !selectedFolderId || selectedFolderId != album.folderId ? 'invisible group-hover:visible' : ''
              ]"
              :iconMenu="IconMore"
              :menuItems="moreMenuItems"
              :smallIcon="true"
            />
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
    <div v-else-if="!isLoading" class="mt-10 flex flex-col items-center justify-center text-base-content/30">
      <button class="btn btn-primary" @click="clickNewAlbum">
        <IconAdd class="w-5 h-5" />
        {{ $t('menu.album.add') }}
      </button>

      <span class="mt-4">{{ $t('tooltip.not_found.albums') }}</span>
    </div>

    <!-- edit album information -->
    <AlbumEdit
      v-if="showAlbumEdit"
      :inputName="getAlbumById(albumId).name"
      :inputDescription="getAlbumById(albumId).description"
      :hiddenAlbum="getAlbumById(albumId).is_hidden"
      :albumPath="getAlbumById(albumId).path"
      :createdAt="formatTimestamp(getAlbumById(albumId).created_at, $t('format.date_time'))"
      :modifiedAt="formatTimestamp(getAlbumById(albumId).modified_at, $t('format.date_time'))"
      @ok="clickAlbumInfo"
      @cancel="showAlbumEdit = false"
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

    <!-- remove from albums -->
    <MessageBox
      v-if="showRemoveMsgbox"
      :title="$t('msgbox.remove_album.title')"
      :message="`${$t('msgbox.remove_album.content', { album: getAlbumById(removingAlbumId).name })}`"
      :OkText="$t('msgbox.remove_album.ok')"
      :cancelText="$t('msgbox.cancel')"
      :warningOk="true"
      @ok="clickRemoveAlbum(removingAlbumId)"
      @cancel="showRemoveMsgbox = false"
    />

    <ToolTip ref="toolTipRef" />

</template>

<script setup lang="ts">

import { ref, watch, computed, onMounted, onBeforeUnmount } from 'vue';
import { listen } from '@tauri-apps/api/event';
import { useI18n } from 'vue-i18n';
import { VueDraggable } from 'vue-draggable-plus'
import { config, isMac, scrollToFolder, formatTimestamp } from '@/common/utils';
import { getAllAlbums, setDisplayOrder, addAlbum, editAlbum, removeAlbum, 
         createFolder, selectFolder, fetchFolder, expandFinalFolder, revealFolder } from '@/common/api';

import AlbumFolder from '@/components/AlbumFolder.vue';
import AlbumEdit from '@/components/AlbumEdit.vue';
import DropDownMenu from '@/components/DropDownMenu.vue';
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
  IconRemove,
  IconEdit,
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
let unlistenContent: () => void;

const selectedAlbumId = ref(0);
const selectedFolderId = ref(0);
const selectedFolderPath = ref('');

const removingAlbumId = ref(null);  // album id to be removed

// message boxes
const showAlbumEdit = ref(false);           // show edit album
const showNewFolderMsgbox = ref(false);     // show new folder
const showRemoveMsgbox = ref(false);        // show remove album
const errorMessage = ref('');

const toolTipRef = ref(null);

const albums = ref([]);
const isEditList = ref(false);  // edit album list
const isLoading = ref(true);    // loading albums
const isDragging = ref(false);  // dragging albums

const getAlbumById = (id) => albums.value.find(album => album.id === id);

const emit = defineEmits(['update:albumId', 'update:folderId', 'update:folderPath']);

// more menuitems
const moreMenuItems = computed(() => {
  return [
    {
      label: localeMsg.value.menu.album.edit_album,
      icon: IconEdit,
      action: () => {
        showAlbumEdit.value = true;
      }
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
        revealFolder(getAlbumById(selectedAlbumId.value).path);
      }
    },
  ];
});

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
      case 'delete-folder':
        if(event.payload.componentId === props.componentId) {
          selectedAlbumId.value = event.payload.albumId;
          selectedFolderId.value = event.payload.folderId;
          selectedFolderPath.value = event.payload.folderPath;
        };
        break;
      case 'refresh-folder':
        for (let album of albums.value) {
          if(album.id === config.destAlbumId) {
            clickFinalSubFolder(album.id, event.payload.folderPath);  // select the sub-folder
            break;
          }
        }
        break;

      default:
        break;
    }
  });

  unlistenContent = await listen('message-from-content', async(event) => {
    console.log('listen - message-from-album-content:', event);
    const { message } = event.payload;
    switch (message) {
      case 'goto-folder':
        clickFinalSubFolder(event.payload.albumId, event.payload.folderPath).then (() => {
          config.sidebarIndex = 1; // show album content
        });
        break;
      default:
        break;
    }
  });
});

onBeforeUnmount(() => {
  unlistenSelectFolder();
  unlistenContent();
});

watch(() => [ props.albumId, props.folderId, props.folderPath ], ([ newAlbumId, newFolderId, newFolderPath ]) => {
  selectedAlbumId.value = newAlbumId;
  selectedFolderId.value = newFolderId;
  selectedFolderPath.value = newFolderPath;
}, { immediate: true });

watch(() => [ selectedAlbumId.value, selectedFolderId.value, selectedFolderPath.value ], ([ newAlbumId, newFolderId, newFolderPath ]) => {
  emit('update:albumId', newAlbumId);
  emit('update:folderId', newFolderId);
  emit('update:folderPath', newFolderPath);
});

/// Add a new album
const clickNewAlbum = async () => {
  const new_album = await addAlbum();
  if(new_album) {
    albums.value.push(new_album);
    clickAlbum(new_album);
  }
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
    selectedAlbumId.value = 0;
    selectedFolderId.value = 0;
    selectedFolderPath.value = "";
  }
};

/// edit album information
const clickAlbumInfo = async (newName, newDescription, newIsHidden) => {
  const result = await editAlbum(selectedAlbumId.value, newName, newDescription, newIsHidden);
  if(result) {
    let album = getAlbumById(selectedAlbumId.value);
    album.name = newName;
    album.description = newDescription;
    album.is_hidden = newIsHidden;
    showAlbumEdit.value = false;
  }
};

/// Remove an album from the list
const clickRemoveAlbum = async (albumId) => {
  const removedAlbum = await removeAlbum(albumId);
  if(removedAlbum) {
    // remove the album from the list
    albums.value = albums.value.filter(album => album.id !== albumId);
    showRemoveMsgbox.value = false;

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

// Expose methods
defineExpose({ 
  isEditList,
  clickNewAlbum,
  refreshAlbums,
});

</script>

<style scoped>
</style>
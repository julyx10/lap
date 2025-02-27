<template>
  
  <ul v-if="children && children.length > 0">
    <li v-for="(child, index) in children" :key="index" :id="'folder-' + index" class="pl-4">
      <div 
        :class="[
          'my-1 border-l-2 flex items-center whitespace-nowrap hover:bg-gray-700 cursor-pointer', 
          rootAlbumId === selectedAlbumId && selectedFolderId === child.id ? 't-color-text-selected t-color-bg-selected border-sky-500 transition-colors duration-300' : 'border-gray-900'
        ]" 
        @update="scrollToItem(index)"
        @click="clickFolder(rootAlbumId, child)"
        @dblclick="clickExpandFolder($event, child)"
      >
        <span v-if="child.children && child.children.length == 0" class="w-6"></span>
        <IconRight v-else
          :class="[
            'p-1 t-icon-size flex-shrink-0 transition-transform', 
            child.is_expanded && child.children && child.children.length > 0 ? 'rotate-90' : ''
          ]"
          @click="clickExpandFolder($event, child)"
        />
        <span 
          :class="[
            'flex-1 min-w-0', 
            componentId === 0 && selectedFolderId === child.id ? 'mask-fade-right' : ''
          ]"
        >{{ child.name }}</span>
        <DropDownMenu v-if="componentId === 0 && selectedFolderId === child.id"
          class="t-color-bg-selected"
          :iconMenu="IconMore"
          :menuItems="moreMenuItems"
        />
      </div>
      <SelectFolder v-if="child.is_expanded" 
        :key="child.id"
        :children="child.children" 
        :rootAlbumId="rootAlbumId"
        :albumId="selectedAlbumId"
        :folderId="selectedFolderId"
        :folderPath="selectedFolderPath"
        :componentId="componentId"
      />
    </li>
  </ul>

  <!-- rename album -->
  <MessageBox
    v-if="showRenameMsgbox"
    :title="$t('msgbox_rename_folder_title')"
    :message="$t('msgbox_rename_folder_content')"
    :showInput="true"
    :inputText="getFolderById(selectedFolderId).name"
    :OkText="$t('msgbox_rename_folder_ok')"
    :cancelText="$t('msgbox_cancel')"
    @ok="clickRenameConfirm"
    @cancel="showRenameMsgbox = false"
  />

  <!-- move to -->
  <MoveTo
    v-if="showMoveTo"
    :title="`${$t('msgbox_move_to_title', { source: shortenFilename(getFolderById(selectedFolderId).name) })}`"
    :message="$t('msgbox_move_to_content')"
    :OkText="$t('msgbox_move_to_ok')"
    :cancelText="$t('msgbox_cancel')"
    @ok="clickCopyToConfirm"
    @cancel="showMoveTo = false"
  />

  <!-- copy to -->
  <MoveTo
    v-if="showCopyTo"
    :title="`${$t('msgbox_copy_to_title', { source: shortenFilename(getFolderById(selectedFolderId).name) })}`"
    :message="$t('msgbox_copy_to_content')"
    :OkText="$t('msgbox_copy_to_ok')"
    :cancelText="$t('msgbox_cancel')"
    @ok="clickCopyToConfirm"
    @cancel="showCopyTo = false"
  />

  <!-- new folder -->
  <MessageBox
    v-if="showNewFolderMsgbox"
    :title="$t('msgbox_new_folder_title')"
    :message="$t('msgbox_new_folder_content')"
    :showInput="true"
    :inputText="''"
    :OkText="$t('msgbox_new_folder_ok')"
    :cancelText="$t('msgbox_cancel')"
    @ok="clickNewFolderConfirm"
    @cancel="showNewFolderMsgbox = false"
  />

  <!-- delete folder -->
  <MessageBox
    v-if="showDeleteMsgbox"
    :title="$t('msgbox_delete_folder_title')"
    :message="`${$t('msgbox_delete_folder_content', { folder: getFolderById(selectedFolderId).name })}`"
    :OkText="$t('msgbox_delete_folder_ok')"
    :cancelText="$t('msgbox_cancel')"
    :warningOk="true"
    @ok="clickDeleteConfirm"
    @cancel="showDeleteMsgbox = false"
  />

</template>

<script setup lang="ts">

import { ref, watch, computed, onMounted } from 'vue';
import { emit } from '@tauri-apps/api/event';
import { useI18n } from 'vue-i18n';
import { openShellFolder, shortenFilename } from '@/common/utils';
import { addFolder, expandFolder } from '@/common/api';

import SelectFolder from '@/components/SelectFolder.vue';
import DropDownMenu from '@/components/DropDownMenu.vue';
import MoveTo from '@/components/MoveTo.vue';
import MessageBox from '@/components/MessageBox.vue';

// folder icon
import IconRight from '@/assets/arrow-right.svg';
import IconMore from '@/assets/more.svg';
import IconRefresh from '@/assets/refresh.svg';
import IconCopyTo from '@/assets/copy-to.svg';
import IconMoveTo from '@/assets/move-to.svg';
import IconRename from '@/assets/rename.svg';
import IconDelete from '@/assets/trash.svg';
import IconNewFolder from '@/assets/folder-plus.svg';
import IconOpenFolder from '@/assets/external.svg';

const props = defineProps({
  children: {       // subfolders
    type: Array,
    required: false,
  },
  rootAlbumId: {    // root album id
    type: Number, 
    required: true,
  },
  albumId: {    // selected album id (v-model value)
    type: Number, 
    required: true,
  },
  folderId: {   // selected folder id (v-model value)
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
const localeMsg = computed(() => messages.value[locale.value]);

// current selected folder
const selectedAlbumId = ref(0);
const selectedFolderId = ref(0);
const selectedFolderPath = ref('');

// message boxes
const showRenameMsgbox = ref(false);
const showMoveTo = ref(false);
const showCopyTo = ref(false);
const showNewFolderMsgbox = ref(false);
const showDeleteMsgbox = ref(false);

const getFolderById = (id) => props.children.find(child => child.id === id);

// more menuitems
const moreMenuItems = computed(() => {
  return [
    {
      label: localeMsg.value.menu_item_refresh,
      icon: IconRefresh,
      action: () => {
      }
    },
    {
      label: localeMsg.value.menu_item_rename,
      icon: IconRename,
      action: () => {
        showRenameMsgbox.value = true;
      }
    },
    {
      label: "-",   // separator
      action: null
    },    
    {
      label: localeMsg.value.menu_item_move_to,
      icon: IconMoveTo,
      action: () => {
        showMoveTo.value = true;
      }
    },
    {
      label: localeMsg.value.menu_item_copy_to,
      // icon: IconCopyTo,
      action: () => {
        showCopyTo.value = true;
      }
    },
    {
      label: localeMsg.value.menu_item_new_folder,
      icon: IconNewFolder,
      action: () => {
        showNewFolderMsgbox.value = true;
      }
    },
    {
      label: localeMsg.value.menu_item_open_folder,
      icon: IconOpenFolder,
      action: () => {
        openShellFolder(selectedFolderPath.value);
      }
    },
    {
      label: "-",   // separator
      action: null
    },    
    {
      label: localeMsg.value.menu_item_delete,
      icon: IconDelete,
      action: () => {
        showDeleteMsgbox.value = true;
      }
    }
  ];
});

onMounted(() => {
  // scrollToItem(selectedFolderId.value);
});


watch(() => [ props.albumId, props.folderId, props.folderPath ], ([ newAlbumId, newFolderId, newFolderPath ]) => {
  selectedAlbumId.value = newAlbumId;
  selectedFolderId.value = newFolderId;
  selectedFolderPath.value = newFolderPath;
}, { immediate: true });

/// click folder to select
const clickFolder = async (albumId, folder) => {
  console.log('SelectFolder.vue-clickFolder:', albumId, folder);
  const newFolder = await addFolder(albumId, 0, folder.path); // parentId: 0 is root folder(album)
  if (newFolder) {
    selectedAlbumId.value = albumId;
    selectedFolderId.value = newFolder.id;
    selectedFolderPath.value = newFolder.path;

    // insert new property 'id' to folder object
    folder.id = newFolder.id;
    
    emit('message-from-select-folder', { 
      albumId: selectedAlbumId.value, 
      folderId: selectedFolderId.value, 
      folderPath: selectedFolderPath.value,
      componentId: props.componentId
    });
  }
};

/// click expand icon to toggle folder expansion
const clickExpandFolder = async (event: Event, folder) => {
  event.stopPropagation();    // Prevents clickFolder() from being triggered

  folder.is_expanded = !folder.is_expanded;

  if (folder.is_expanded && !folder.children) {
    const subFolders = await expandFolder(folder.path, false);
    if(subFolders) {
      folder.children = subFolders.children;
    }
  }
};

/// Rename an album
const clickRenameConfirm = async (value) => {
  try {
    showRenameMsgbox.value = false;
  } catch (error) {
    console.error('Failed to rename folder:', error);
  }
};

const clickCopyToConfirm = async (value) => {
  try {
    showMoveTo.value = false;
  } catch (error) {
    console.error('Failed to move folder:', error);
  }
};

/// Create new folder
const clickNewFolderConfirm = async (value) => {
  try {
    showNewFolderMsgbox.value = false;
  } catch (error) {
    console.error('Failed to create new folder:', error);
  }
};

/// delete folder
const clickDeleteConfirm = async () => {
  try {
    showDeleteMsgbox.value = false;
  } catch (error) {
    console.error('Failed to delete folder:', error);
  }
};

// make the selected item always visible in a scrollable container
function scrollToItem(index) {
  const item = document.getElementById(`folder-${index}`);
  if (item) {
    item.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
  }
};

</script>

<style scoped>
.mask-fade-right {
  mask-image: linear-gradient(to left, transparent 0%, black 24px);
}
</style>
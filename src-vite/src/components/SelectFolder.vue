<template>
  
  <ul v-if="children && children.length > 0">
    <li v-for="(child, index) in children" :key="index" :id="'folder-' + index" class="pl-4">
      <div 
        :class="[
          'my-1 border-l-2 flex items-center whitespace-nowrap hover:bg-gray-700 cursor-pointer group rounded-r', 
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
        <input v-if="isRenamingFolder && selectedFolderId === child.id"
          ref="folderInputRef"
          type="text"
          maxlength="255"
          class="px-1 w-full border t-color-border t-input-color-bg t-input-focus rounded"
          v-model="child.name"
          @keydown.enter = "clickRenameFolder(child.name)"
          @keydown.esc = "handleEscKey"
          @blur = "clickRenameFolder(child.name)"
        > 
        <span v-else
          :class="[
            'flex-1 min-w-0 mask-fade-right', 
            // componentId === 0 && selectedFolderId === child.id ? '' : ''
          ]"
        >{{ child.name }}</span>

        <DropDownMenu v-if="componentId === 0 && !isRenamingFolder"
          class="hidden group-hover:block t-color-bg-selected rounded-r"
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

  <!-- rename folder -->
  <!-- <MessageBox
    v-if="showRenameMsgbox"
    :title="$t('msgbox_rename_folder_title')"
    :message="$t('msgbox_rename_folder_content')"
    :showInput="true"
    :inputText="getFolderById(selectedFolderId).name"
    :OkText="$t('msgbox_rename_folder_ok')"
    :cancelText="$t('msgbox_cancel')"
    @ok="clickRenameFolder"
    @cancel="showRenameMsgbox = false"
  /> -->

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
    @ok="clickNewFolder"
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

import { ref, watch, nextTick, computed, onMounted } from 'vue';
import { emit } from '@tauri-apps/api/event';
import { useI18n } from 'vue-i18n';
import { openShellFolder, shortenFilename, isValidFileName } from '@/common/utils';
import { createFolder, renameFolder, selectFolder, expandFolder } from '@/common/api';

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

const isRenamingFolder = ref(false);
const folderInputRef = ref([]);     // input text box ref
const originalFolderName = ref(''); // restore original folder name when cancel renaming(press ESC)

// message boxes
// const showRenameMsgbox = ref(false);
const showMoveTo = ref(false);
const showCopyTo = ref(false);
const showNewFolderMsgbox = ref(false);
const showDeleteMsgbox = ref(false);

const getFolderById = (id) => props.children.find(child => child.id === id);

// more menuitems
const moreMenuItems = computed(() => {
  return [
    // {
    //   label: localeMsg.value.menu_item_refresh,
    //   icon: IconRefresh,
    //   action: () => {
    //   }
    // },
    {
      label: localeMsg.value.menu_item_new_folder,
      icon: IconNewFolder,
      action: () => {
        showNewFolderMsgbox.value = true;
      }
    },
    {
      label: localeMsg.value.menu_item_rename,
      icon: IconRename,
      action: () => {
        isRenamingFolder.value = true;
        originalFolderName.value = getFolderById(selectedFolderId.value).name;
        nextTick(() => {
          if (folderInputRef.value) {
            folderInputRef.value[0].focus();    // array of input elements
          }
        });
      }
    },
    {
      label: localeMsg.value.menu_item_delete,
      icon: IconDelete,
      action: () => {
        showDeleteMsgbox.value = true;
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
      label: "-",   // separator
      action: null
    },
    {
      label: localeMsg.value.menu_item_reveal_in_file_explorer,
      // icon: IconOpenFolder,
      action: () => {
        openShellFolder(selectedFolderPath.value);
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
  const selectedFolder = await selectFolder(albumId, 0, folder.path); // parentId: 0 is root folder(album)
  if (selectedFolder) {
    selectedAlbumId.value = albumId;
    selectedFolderId.value = selectedFolder.id;
    selectedFolderPath.value = selectedFolder.path;

    // insert new property 'id' to folder object
    folder.id = selectedFolder.id;
    
    emit('message-from-select-folder', { 
      albumId: selectedAlbumId.value, 
      folderId: selectedFolderId.value, 
      folderPath: selectedFolderPath.value,
      componentId: props.componentId
    });
  }
};

/// click expand icon to toggle folder expansion
const clickExpandFolder = async (event: Event, folder, alwaysExpand = false) => {
  event.stopPropagation();    // Prevents clickFolder() from being triggered

  folder.is_expanded = alwaysExpand ? alwaysExpand : !folder.is_expanded;

  if (folder.is_expanded && !folder.children) {
    const subFolders = await expandFolder(folder.path, false);
    if(subFolders) {
      folder.children = subFolders.children;
    }
  }
};

/// Rename folder
const clickRenameFolder = async (newFolderName) => {
  console.log('SelectFolder.vue-clickRenameFolder:', newFolderName);

  // verfify new folder name is valid
  if (!newFolderName || newFolderName.trim().length === 0 || !isValidFileName(newFolderName)) {
    console.log('SelectFolder.vue-clickRenameFolder: invalid folder name');
    return;
  }
  if (newFolderName != originalFolderName.value) {
    const newFolderPath = await renameFolder(selectedFolderPath.value, newFolderName);
    if(newFolderPath) {
      let folder = getFolderById(selectedFolderId.value);
      folder.name = newFolderName;
      updateFolderPath(folder, selectedFolderPath.value, newFolderPath);

      // update selected folder path
      selectedFolderPath.value = newFolderPath;
      emit('message-from-select-folder', { 
        albumId: selectedAlbumId.value, 
        folderId: selectedFolderId.value, 
        folderPath: selectedFolderPath.value,
        componentId: props.componentId
      });
    }
  }
  isRenamingFolder.value = false;
};

/// handle ESC key to cancel renaming folder
const handleEscKey = (event) => {
  event.preventDefault();

  let folder = getFolderById(selectedFolderId.value);
  folder.name = originalFolderName.value;

  isRenamingFolder.value = false; 
};

/// rename folder path and children paths
function updateFolderPath(folder, oldpath, newPath) {
    folder.path = newPath + folder.path.slice(oldpath.length);

    if (folder.children) {
        folder.children.forEach(child => {
            updateFolderPath(child, oldpath, newPath); // recursive
        });
    }
}

const clickCopyToConfirm = async (value) => {
  try {
    showMoveTo.value = false;
  } catch (error) {
    console.error('Failed to move folder:', error);
  }
};

/// Create new folder
const clickNewFolder = async (value) => {
  const newFolderPath = await createFolder(selectedFolderPath.value, value);
  if(newFolderPath) {
    let folder = getFolderById(selectedFolderId.value);
    if (!folder.children) folder.children = [];
    folder.children.push({ name: value, path: newFolderPath });
    showNewFolderMsgbox.value = false;

    await clickExpandFolder(new Event('click'), folder, true);
    clickFolder(selectedAlbumId.value, folder.children[folder.children.length - 1]);
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
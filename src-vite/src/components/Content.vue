<template>

  <div class="flex-1 flex flex-col">

    <!-- title bar -->
    <div class="px-4 pt-1 min-h-12 flex flex-row flex-wrap items-center justify-between select-none" data-tauri-drag-region>

      <!-- title -->
      <div class="breadcrumbs mr-2" data-tauri-drag-region>
        <ul>
          <li v-for="(item, index) in contentTitle.split(' > ')"><a>{{ item }}</a></li>
        </ul>
      </div>

      <!-- toolbar -->
      <div class="ml-auto h-6 flex flex-row items-center space-x-2 shrink-0">

        <!-- search box - filter file name -->
        <SearchBox ref="searchBoxRef" v-model="config.searchText" @click.stop="selectMode = false" /> 

        <!-- select mode -->
        <div tabindex="-1"
          :class="[
            'px-2 py-1 h-8 flex flex-row items-center rounded-md border border-content focus:outline-none text-sm shrink-0 cursor-pointer',
            selectMode ? 'border-primary' : 'border-base-content/30 hover:border-base-content/70'
          ]"
          @click="handleSelectMode(true)"
        >
          <TButton v-if="selectMode"
            :icon="IconClose"
            :buttonSize="'small'"
            @click.stop="handleSelectMode(false)" 
          />
          <span class="px-1">{{ selectMode ? $t('toolbar.filter.select_count', { count: selectedCount }) : $t('toolbar.filter.select_mode') }}</span>
          <DropDownMenu v-if="selectMode"
            :iconMenu="IconArrowDown"
            :menuItems="moreMenuItems"
            :smallIcon="true"
            @click.stop
          />
        </div>

        
        <!-- file type options -->
        <DropDownSelect
          :options="fileTypeOptions"
          :defaultIndex="config.searchFileType"
          @select="handleFileTypeSelect"
        />

        <!-- sort type options -->
        <DropDownSelect
          :options="sortOptions"
          :defaultIndex="config.sortType"
          :extendOptions="sortExtendOptions"
          :defaultExtendIndex="config.sortOrder"
          @select="handleSortTypeSelect"
        />

        <!-- magic button -->
        <TButton
          :icon="IconMagic"
          :buttonSize="'medium'"
          @click="clickAI"
        />
        
        <!-- preview -->
        <TButton
          :icon="config.showPreview ? IconPreview : IconPreviewOff"
          :buttonSize="'medium'"
          @click="config.showPreview = !config.showPreview"
        />

      </div>
    </div>

    <ProgressBar v-if="fileList.length > 0" :percent="Number(((thumbCount / fileList.length) * 100).toFixed(0))" />

    <div ref="divListView" class="mt-1 flex-1 flex flex-row overflow-hidden">
      <div class="flex-1 flex flex-col">
        <!-- grid view -->
        <GridView ref="gridViewRef"
          v-model:selectItemIndex="selectedItemIndex"
          :fileList="fileList"
          :showFolderFiles="showFolderFiles"
          :selectMode="selectMode"
        />
        
        <!-- status bar -->
        <div v-if="config.showStatusBar" 
          class="p-2 min-h-8 flex flex-row items-center justify-start text-sm select-none cursor-default"
        >
          <component 
            :is="showFolderFiles ? IconFolder : IconFile"
            class="t-icon-size-xs shrink-0" 
          />
          <div class="pl-1 pr-4 whitespace-nowrap">
            {{ $t('statusbar.files_summary', { count: totalCount.toLocaleString(), size: formatFileSize(totalSize) }) }} 
          </div>

          <component v-if="selectedItemIndex >= 0"
            :is="selectMode ? IconCheckAll : IconChecked" 
            class="t-icon-size-xs shrink-0" 
          />
          <div v-if="selectedItemIndex >=0" 
            class="px-1 w-0 flex-1 overflow-hidden whitespace-nowrap text-ellipsis"
          >
            {{
              selectMode 
                ? $t('toolbar.filter.select_count', { count: selectedCount.toLocaleString() }) + ' (' + formatFileSize(selectedSize) + ')'
                : fileList[selectedItemIndex]?.name + ' (' + formatFileSize(fileList[selectedItemIndex]?.size) + ')' 
            }}
          </div>
        </div>

      </div>

      <!-- splitter -->
      <div v-if="config.showPreview" 
        class="w-1 hover:bg-primary cursor-ew-resize transition-colors" 
        @mousedown="startDragging"
      ></div>

      <!-- preview pane -->
      <transition
        enter-active-class="transition-transform duration-200"
        leave-active-class="transition-transform duration-200"
        enter-from-class="translate-x-full"
        enter-to-class="translate-x-0"
        leave-from-class="translate-x-0"
        leave-to-class="translate-x-full"
      >
        <div v-show="config.showPreview" ref="previewDiv" 
          class="p-1 rounded-ss-lg overflow-hidden bg-base-200"
          :style="{ width: config.previewPaneWidth + '%' }"
        >
          <div v-if="selectedItemIndex >= 0 && selectedItemIndex < fileList.length"
            class="relative"
            :style="{ width: previewPaneSize.width + 'px', height: previewPaneSize.height + 'px' }"
            @dblclick="openImageViewer(selectedItemIndex, true)"
          >
            <Image v-if="fileList[selectedItemIndex]?.file_type === 1"
              ref="imageRef" 
              :src="imageSrc" 
              :rotate="fileList[selectedItemIndex]?.rotate ?? 0" 
              :isZoomFit="true"
            />
            <Video v-else-if="fileList[selectedItemIndex]?.file_type === 2"
              :src="fileList[selectedItemIndex]?.file_path"
            ></Video>

            <!-- comments -->
            <div v-if="config.showComment && fileList[selectedItemIndex]?.comments?.length > 0" 
              class="absolute flex m-2 p-2 bottom-0 left-0 right-0 text-sm bg-base-100 opacity-60 rounded-lg" 
            >
              <IconComment class="t-icon-size-sm shrink-0 mr-2"></IconComment>
              {{ fileList[selectedItemIndex]?.comments }}
            </div>
          </div>

          <div v-else class="h-full flex items-center justify-center">
            <p >{{ $t('tooltip.not_found.files') }}</p>
          </div>
      
        </div>
      </transition>

    </div>
  </div>

  <!-- rename -->
  <MessageBox
    v-if="showRenameMsgbox"
    :title="$t('msgbox.rename_file.title')"
    :message="$t('msgbox.rename_file.content')"
    :showInput="true"
    :inputText="renamingFileName.name"
    :needValidateInput="true"
    :OkText="$t('msgbox.rename_file.ok')"
    :cancelText="$t('msgbox.cancel')"
    :errorMessage="errorMessage"
    @ok="clickRenameFile"
    @cancel="showRenameMsgbox = false"
    @reset="errorMessage = ''"
  />

  <!-- move to -->
  <MoveTo
    v-if="showMoveTo"
    :title="`${$t('msgbox.move_to.title', { source: selectMode ? $t('toolbar.filter.select_count', { count: selectedCount.toLocaleString() }) : fileList[selectedItemIndex].name })}`"
    :message="$t('msgbox.move_to.content')"
    :OkText="$t('msgbox.move_to.ok')" 
    :cancelText="$t('msgbox.cancel')"
    @ok="clickMoveTo"
    @cancel="showMoveTo = false"
  />

  <!-- copy to -->
  <MoveTo
    v-if="showCopyTo"
    :title="`${$t('msgbox.copy_to.title', { source: selectMode ? $t('toolbar.filter.select_count', { count: selectedCount.toLocaleString() }) : fileList[selectedItemIndex].name })}`"
    :message="$t('msgbox.copy_to.content')"
    :OkText="$t('msgbox.copy_to.ok')"
    :cancelText="$t('msgbox.cancel')"
    @ok="clickCopyTo"
    @cancel="showCopyTo = false"
  />

  <!-- delete -->
  <MessageBox
    v-if="showDeleteMsgbox"
    :title="$t('msgbox.delete_file.title')"
    :message="`${$t('msgbox.delete_file.content', { file: selectMode ? $t('toolbar.filter.select_count', { count: selectedCount.toLocaleString() }) : fileList[selectedItemIndex].name })}`"
    :OkText="$t('msgbox.delete_file.ok')"
    :cancelText="$t('msgbox.cancel')"
    :warningOk="true"
    @ok="clickDeleteFile"
    @cancel="showDeleteMsgbox = false"
  />

  <!-- comment -->
  <MessageBox
    v-if="showCommentMsgbox"
    :title="$t('msgbox.edit_comment.title')"
    :message="`${$t('msgbox.edit_comment.content', { file: fileList[selectedItemIndex].name })}`"
    :showInput="true"
    :inputText="fileList[selectedItemIndex]?.comments ?? ''"
    :multiLine="true"
    :OkText="$t('msgbox.ok')"
    :cancelText="$t('msgbox.cancel')"
    @ok="clickEditComment"
    @cancel="showCommentMsgbox = false"
  />

  <ToolTip ref="toolTipRef" />

</template>

<script setup lang="ts">

import { ref, watch, computed, onMounted, onBeforeUnmount, onUnmounted } from 'vue';
import { emit, listen } from '@tauri-apps/api/event';
import { WebviewWindow } from '@tauri-apps/api/webviewWindow';
import { useI18n } from 'vue-i18n';
import { getAlbum, getDbFiles, getFolderFiles, getTagName,
         copyImage, renameFile, moveFile, copyFile, deleteFile, editFileComment, getFileThumb, revealFolder, getFileImage,
         setFileFavorite, setFileRotate } from '@/common/api';
import { config, isWin, isMac, setTheme,
         formatFileSize, formatDate, getCalendarDateRange, getRelativePath, 
         extractFileName, combineFileName, getFolderPath } from '@/common/utils';

import SearchBox from '@/components/SearchBox.vue';
import DropDownSelect from '@/components/DropDownSelect.vue';
import DropDownMenu from '@/components/DropDownMenu.vue';
import ProgressBar from '@/components/ProgressBar.vue';
import GridView  from '@/components/GridView.vue';
import Image from '@/components/Image.vue';
import Video from '@/components/Video.vue';
import MessageBox from '@/components/MessageBox.vue';
import MoveTo from '@/components/MoveTo.vue';
import ToolTip from '@/components/ToolTip.vue';
import TButton from '@/components/TButton.vue';

import {
  IconPreview,
  IconPreviewOff,
  IconArrowDown,
  IconClose,
  IconCheckAll,
  IconCheckNone,
  IconFavorite,
  IconUnFavorite,
  IconMoveTo,
  IconTrash,
  IconFile,
  IconFolder,
  IconSearch,
  IconChecked,
  IconComment,
  IconMagic,
  IconTag,
} from '@/common/icons';

const props = defineProps({
  titlebar: String
});

/// i18n
const { locale, messages } = useI18n();
const localeMsg = computed(() => messages.value[locale.value]);

// title of the content 
const contentTitle = ref("");   

// progress bar
const thumbCount = ref(0);      // thumbnail count (from 0 to fileList.length)

// file list view
const divListView = ref(null);

// file list
const fileList = ref([]);
const totalCount = ref(0);   // total files' count
const totalSize = ref(0);     // total files' size
const selectedItemIndex = ref(-1);
const fileListOffset = ref(0); // offset of the file list

// config.favoriteFolderId = 0: means favorite files
const showFolderFiles = computed(() =>
 config.sidebarIndex === 2 || (config.sidebarIndex === 1 && config.favoriteFolderId !== 0)
);

// mutil select mode
const selectMode = ref(false);
const selectedCount = ref(0);
const selectedSize = ref(0);  // selected files size

// preview 
const isDraggingSplitter = ref(false);      // dragging splitter to resize preview pane
const previewDiv = ref(null);
const previewPaneSize = ref({ width: 100, height: 100 });
const imageSrc = ref('');         // preview image source

// message box
const showRenameMsgbox = ref(false);  // show rename message box
const renamingFileName = ref({}); // extract the file name to {name, ext}

const showMoveTo = ref(false);
const showCopyTo = ref(false);
const showDeleteMsgbox = ref(false);
const showCommentMsgbox = ref(false);
const errorMessage = ref('');

// grid view
const gridViewRef = ref(null);

const toolTipRef = ref(null);
const searchBoxRef = ref(null);

let resizeObserver;

let unlistenGridView: () => void;
let unlistenImageViewer: () => void;

// more menuitems
const moreMenuItems = computed(() => {
  return [
    {
      label: localeMsg.value.menu.select_all,
      icon: IconCheckAll,
      action: () => {
        for (let i = 0; i < fileList.value.length; i++) {
          fileList.value[i].isSelected = true;
        }
        selectMode.value = true;
      }
    },
    {
      label: localeMsg.value.menu.select_none,
      icon: IconCheckNone,
      action: () => {
        for (let i = 0; i < fileList.value.length; i++) {
          fileList.value[i].isSelected = false;
        }
        selectMode.value = true;
      }
    },
    {
      label: localeMsg.value.menu.select_invert,
      action: () => {
        for (let i = 0; i < fileList.value.length; i++) {
          fileList.value[i].isSelected = !fileList.value[i].isSelected;
        }
        selectMode.value = true;
      }
    },
    {
      label: "-",   // separator
      action: null
    },
    {
      label: localeMsg.value.menu.move_to,
      icon: IconMoveTo,
      disabled: selectedCount.value === 0,
      action: () => {
        showMoveTo.value = true;
      }
    },
    {
      label: localeMsg.value.menu.copy_to,
      disabled: selectedCount.value === 0,
      action: () => {
        showCopyTo.value = true;
      }
    },
    {
      label: localeMsg.value.menu.trash,
      icon: IconTrash,
      disabled: selectedCount.value === 0,
      action: () => {
        showDeleteMsgbox.value = true;
      }
    },
    {
      label: "-",   // separator
      action: () => {}
    },
    {
      label: localeMsg.value.menu.favorite,
      icon: IconFavorite,
      disabled: selectedCount.value === 0,
      action: () => {
        selectModeSetFavorites(true);
      }
    },
    {
      label: localeMsg.value.menu.unfavorite,
      icon: IconUnFavorite,
      disabled: selectedCount.value === 0,
      action: () => {
        selectModeSetFavorites(false);
      }
    },
    {
      label: localeMsg.value.menu.tag,
      icon: IconTag,
      disabled: selectedCount.value === 0,
      action: () => {
        // clickTag();
      }
    },
  ];
});

const handleEscapeKey = (e: KeyboardEvent) => {
  if (e.key === 'Escape' && selectMode.value) {
    handleSelectMode(false);
  }
};

onMounted( async() => {
  // FIXME: ResizeObserver loop completed with undelivered notifications.
  resizeObserver = new ResizeObserver(entries => {
    for (let entry of entries) {
      previewPaneSize.value = {
        width: entry.contentRect.width,
        height: entry.contentRect.height,
      };
    }
  });

  if (previewDiv.value) {
    // Observe preview pane size changes
    resizeObserver.observe(previewDiv.value);
  }

  // Add escape key listener
  window.addEventListener('keydown', handleEscapeKey);

  unlistenGridView = await listen('message-from-grid-view', (event) => {
    const { message } = event.payload;
    console.log('content - message-from-grid-view:', message);
    switch (message) {
      case 'search':
        searchBoxRef.value.focusInput();
        break;
      case 'select':
        fileList.value[event.payload.index].isSelected = !fileList.value[event.payload.index].isSelected;
        break;
      case 'open':
        openImageViewer(selectedItemIndex.value, true);
        break;
      case 'copy': // copy image to clipboard
        copyImage(fileList.value[selectedItemIndex.value].file_path).then(() => {
          toolTipRef.value.showTip(localeMsg.value.tooltip.copy_image.success);
        });
        break;
      case 'rename':
        renamingFileName.value = extractFileName(fileList.value[selectedItemIndex.value].name);
        showRenameMsgbox.value = true;
        break;
      case 'move-to':
        showMoveTo.value = true;
        break;
      case 'copy-to':
        showCopyTo.value = true;
        break;
      case 'delete':
        showDeleteMsgbox.value = true;
        break;
      case 'goto-folder':
        const albumId = fileList.value[selectedItemIndex.value].album_id;
        const folderPath = getFolderPath(fileList.value[selectedItemIndex.value].file_path);
        emit('message-from-content', { message: 'goto-folder', albumId, folderPath });
        break;
      case 'reveal':
        revealFolder(getFolderPath(fileList.value[selectedItemIndex.value].file_path));
        break;
      case 'favorite':
        toggleFavorite();
        break;
      case 'rotate':
        clickRotate();
        break;
      case 'comment':
        showCommentMsgbox.value = true;
        break;
      case 'next-page':
        if(!showFolderFiles.value) {  // offset is only available for db files
          if(fileListOffset.value + config.fileListPageSize < totalCount.value) {
            fileListOffset.value += config.fileListPageSize;
          }
        }
        break;
      default:
        break;
    }
  });

  unlistenImageViewer = await listen('message-from-image-viewer', (event) => {
    const { message } = event.payload;
    console.log('content - message-from-image-viewer:', message);
    switch (message) {
      case 'home':
        selectedItemIndex.value = 0;
        break;
      case 'end':
        selectedItemIndex.value = fileList.value.length - 1;
        break;
      case 'prev':
        selectedItemIndex.value = Math.max(selectedItemIndex.value - 1, 0);
        break;
      case 'next':
        selectedItemIndex.value = Math.min(selectedItemIndex.value + 1, fileList.value.length - 1);
        break;
      case 'delete':
        clickDeleteFile();
        break;
      case 'favorite':
        toggleFavorite();
        break;
      case 'rotate':
        clickRotate();
        break;
      default:
        break;
    }
  });
});

onBeforeUnmount(() => {
  // unlisten
  unlistenGridView();
  unlistenImageViewer();
  // Remove escape key listener
  window.removeEventListener('keydown', handleEscapeKey);
});

onUnmounted(() => {
  if (resizeObserver && previewDiv.value) {
    resizeObserver.unobserve(previewDiv.value);
    resizeObserver.disconnect();
  }
});

/// watch appearance
watch(() => config.appearance, (newAppearance) => {
  setTheme(newAppearance);
});

/// watch language
watch(() => config.language, (newLanguage) => {
    locale.value = newLanguage; // update locale based on config.language
});

/// watch for file list changes
watch(
  () => [
          config.sidebarIndex,      // toolbar index
          config.favoriteAlbumId, config.favoriteFolderId, config.favoriteFolderPath,   // favorite files and folder
          config.albumId, config.albumFolderId, config.albumFolderPath,                 // album
          config.tagId,                                                                 // tag
          config.calendarYear, config.calendarMonth, config.calendarDate,               // calendar
          config.cameraMake, config.cameraModel,                                        // camera 
          config.searchText, config.searchFileType, config.sortType, config.sortOrder,  // search and sort 
        ], 
  () => {
    fileListOffset.value = 0;   // reset file list offset
    updateContent();
  }, 
  { immediate: true }
);

// watch for file list offset
watch(() => fileListOffset.value, (newValue) => {
  if(newValue > 0) {
    updateContent();
  }
});

// watch for selected item (not in select mode)
watch(() => selectedItemIndex.value, (newIndex) => {
  updateSelectedImage(newIndex);
});

// watch for selected items in the file list (select mode)
watch(
  () => fileList.value.map(file => ({ isSelected: file.isSelected, size: file.size })),
  () => {
    selectedCount.value = fileList.value.filter(f => f.isSelected).length;
    selectedSize.value = fileList.value.reduce((total, f) => total + (f.isSelected ? f.size : 0), 0);
  }
);

// watch for show preview on/off
watch(() => config.showPreview, (newValue) => {
  if(newValue) {
    getImageSrc(selectedItemIndex.value);
  } else {
    imageSrc.value = '';
  }
  gridViewRef.value.scrollToItem(selectedItemIndex.value); 
});

async function getFileList(startDate, endDate, make, model, isFavorite, tagId, isDeleted, offset) { 
  if (offset === 0) {
    [fileList.value, totalCount.value, totalSize.value] = await getDbFiles(startDate, endDate, make, model, isFavorite, tagId, isDeleted, offset);
  } else {
    const newFiles = await getDbFiles(startDate, endDate, make, model, isFavorite, tagId, isDeleted, offset);
    fileList.value.push(...newFiles);
  }
}

async function updateContent() {
  const newIndex = config.sidebarIndex;

  if(newIndex === 0) {        // home
    contentTitle.value = localeMsg.value.sidebar.home;
    await getFileList("", "", "", "", false, 0, false, fileListOffset.value);
  } 
  else if(newIndex === 1) {   // album
    const album = await getAlbum(config.albumId);
    if(album) {
      if(config.albumFolderPath === album.path) { // current folder is root
        contentTitle.value = album.name;
      } else {
        contentTitle.value = album.name + getRelativePath(config.albumFolderPath, album.path);
      };
      [fileList.value, totalCount.value, totalSize.value] = await getFolderFiles(config.albumFolderId, config.albumFolderPath);
    } 
  }
  else if(newIndex === 2) {   // favorite
    if(config.favoriteFolderId === 0) { // 0: favorite files
      contentTitle.value = localeMsg.value.favorite.files;
      await getFileList("", "", "", "", true, 0, false, fileListOffset.value);
    } else {                // else: favorite folders
      const album = await getAlbum(config.favoriteAlbumId);
      if(album) {
        contentTitle.value = localeMsg.value.favorite.folders + getRelativePath(config.favoriteFolderPath, album.path);
      };
      [fileList.value, totalCount.value, totalSize.value] = await getFolderFiles(config.favoriteFolderId, config.favoriteFolderPath);
    }
  }
  else if(newIndex === 3) {   // tag
    if (!config.tagId) {
      contentTitle.value = localeMsg.value.sidebar.tag;
    } else {
      const tagName = await getTagName(config.tagId);
      if (tagName) {
        contentTitle.value = localeMsg.value.sidebar.tag + ' > ' + tagName;
      } else {
        contentTitle.value = localeMsg.value.sidebar.tag;
      }
    }
    await getFileList("", "", "", "", false, config.tagId, false, fileListOffset.value);
  }
  else if(newIndex === 4) {   // calendar
    if (config.calendarMonth === -1) {          // yearly
      contentTitle.value = formatDate(config.calendarYear, 1, 1, localeMsg.value.format.year);
    } else if (config.calendarDate === -1) {    // monthly
      contentTitle.value = formatDate(config.calendarYear, config.calendarMonth, 1, localeMsg.value.format.month);
    } else {                                    // daily
      contentTitle.value = formatDate(config.calendarYear, config.calendarMonth, config.calendarDate, localeMsg.value.format.date_long_with_weekdayf);
    }
    const [startDate, endDate] = getCalendarDateRange(config.calendarYear, config.calendarMonth, config.calendarDate);
    await getFileList(startDate, endDate, "", "", false, 0, false, fileListOffset.value);
  }
  else if(newIndex === 5) {   // location
    contentTitle.value = localeMsg.value.sidebar.location;
    fileList.value = [];
  }
  else if(newIndex === 6) {   // camera
    if(config.cameraModel) {
      contentTitle.value = `${config.cameraMake} > ${config.cameraModel}`;
    } else {
      contentTitle.value = `${config.cameraMake}`;
    }
    await getFileList("", "", config.cameraMake, config.cameraModel, false, 0, false, fileListOffset.value);
  } else if(newIndex === 7) { // trash
    contentTitle.value = localeMsg.value.trash.files;
    await getFileList("", "", "", "", false, 0, true, fileListOffset.value);
  }

  refreshFileList();
}

// click rename menu item
const clickRenameFile = async (newName) => {
  if(selectedItemIndex.value >= 0) {
    const file = fileList.value[selectedItemIndex.value];
    const fileName = combineFileName(newName, renamingFileName.value.ext);
    const newFilePath = await renameFile(file.id, file.file_path, fileName );
    if(newFilePath) {
      console.log('clickRenameFile:', newFilePath);
      file.name = fileName;
      file.file_path = newFilePath;
      showRenameMsgbox.value = false;
      errorMessage.value = '';
    } else {
      errorMessage.value = localeMsg.value.msgbox.rename_file.error;
    }
  }
}

// click move to menu item
const clickMoveTo = async () => {
  if (selectMode.value && selectedCount.value > 0) {    // multi-select mode
    const moves = fileList.value
      .filter(item => item.isSelected)
      .map(async item => {
        const movedFile = await moveFile(item.id, item.file_path, config.destFolderId, config.destFolderPath);
        if(movedFile) {
          console.log('clickMoveTo:', movedFile);
          removeFromFileList(fileList.value.indexOf(item));
        }
      });
    await Promise.all(moves); // parallelize DB updates
    selectMode.value = false; // exit multi-select mode
  } 
  else if(selectedItemIndex.value >= 0) {               // single select mode
    const file = fileList.value[selectedItemIndex.value];
    const movedFile = await moveFile(file.id, file.file_path, config.destFolderId, config.destFolderPath);
    if(movedFile) {
      console.log('clickMoveTo:', movedFile);
      removeFromFileList(selectedItemIndex.value);
    }
  }
  showMoveTo.value = false;
}

// click copy to menu item
const clickCopyTo = async () => {
  if (selectMode.value && selectedCount.value > 0) {    // multi-select mode
    const copies = fileList.value
      .filter(item => item.isSelected)
      .map(async item => {
        const copiedFile = await copyFile(item.file_path, config.destFolderPath);
        if(copiedFile) {
          console.log('clickCopyTo:', copiedFile);
        }
      });
    await Promise.all(copies); // parallelize DB updates
    selectMode.value = false; // exit multi-select mode
  } 
  else if(selectedItemIndex.value >= 0) {               // single select mode
    const file = fileList.value[selectedItemIndex.value];
    const copiedFile = await copyFile(file.file_path, config.destFolderPath);
    if(copiedFile) {
      console.log('clickCopyTo:', copiedFile);
    }
  }
  showCopyTo.value = false;
}

// click delete menu item
const clickDeleteFile = async () => {
  if (selectMode.value && selectedCount.value > 0) {     // multi-select mode
    const deletes = fileList.value
      .filter(item => item.isSelected)
      .map(async item => {
        const deletedFile = await deleteFile(item.id, item.file_path);
        if(deletedFile) {
          console.log('clickDeleteFile:', deletedFile);
          removeFromFileList(fileList.value.indexOf(item));
        }
      });
    await Promise.all(deletes); // parallelize DB updates
    selectMode.value = false; // exit multi-select mode
  } 
  else if(selectedItemIndex.value >= 0) {               // single select mode
    const file = fileList.value[selectedItemIndex.value];
    const deletedFile = await deleteFile(file.id, file.file_path);
    if(deletedFile) {
      console.log('clickDeleteFile:', deletedFile);
      removeFromFileList(selectedItemIndex.value);
    }
  }
  showDeleteMsgbox.value = false;
}

// remove an file item from the list and update the selected item index
function removeFromFileList(index) {
  fileList.value.splice(index, 1);
  selectedItemIndex.value = Math.min(index, fileList.value.length - 1);
}

// toggle the selected file's favorite status (selectMode = false)
const toggleFavorite = async () => {
  if (selectedItemIndex.value >= 0) {
    const item = fileList.value[selectedItemIndex.value];
    item.is_favorite = !item.is_favorite;
    
    // notify the image viewer
    emit('message-from-content', { message: 'favorite', favorite: item.is_favorite });

    // update the favorite status in the database
    await setFileFavorite(item.id, item.is_favorite);
  }
};

// set selected files' favorite status (selectMode = true)
const selectModeSetFavorites = async (isFavorite: boolean) => {
  if (selectMode.value && selectedCount.value > 0) {
    const updates = fileList.value
      .filter(item => item.isSelected)
      .map(async item => {
        item.is_favorite = isFavorite;

        // notify the image viewer
        if(selectedItemIndex.value === fileList.value.indexOf(item)) {
          emit('message-from-content', { message: 'favorite', favorite: item.is_favorite });
        }

        // update the favorite status in the database
        return setFileFavorite(item.id, isFavorite);
      }); 
    await Promise.all(updates); // parallelize DB updates
  }
}

// set file rotate
const clickRotate = async () => {
  if (selectedItemIndex.value >= 0) {
    fileList.value[selectedItemIndex.value].rotate += 90;

    // notify the image viewer
    emit('message-from-content', { message: 'rotate' });

    // update the rotate status in the database
    setFileRotate(fileList.value[selectedItemIndex.value].id, fileList.value[selectedItemIndex.value].rotate);
  }
};

// edit comment
const clickEditComment = async (newComment) => {
  if (selectedItemIndex.value >= 0) {
    const file = fileList.value[selectedItemIndex.value];
    const result = await editFileComment(file.id, newComment);
    if(result) {
      console.log('clickEditComment:', newComment);
      file.comments = newComment;
      showCommentMsgbox.value = false;
    }
  }
}

// AI
const clickAI = async () => {
  console.log('clickAI');
}

const handleSelectMode = (value) => {
  selectMode.value = value;
  if(!selectMode.value) {
    for (let i = 0; i < fileList.value.length; i++) {
      fileList.value[i].isSelected = false;
    }
  }
};

const handleFileTypeSelect = (option, extendOption) => {
  selectMode.value = false;   // exit multi-select mode
  config.searchFileType = option;
};

const handleSortTypeSelect = (option, extendOption) => {
  selectMode.value = false;   // exit multi-select mode
  config.sortType = option;
  config.sortOrder = extendOption;
};

// file type options
const fileTypeOptions = computed(() => {
  return getSelectOptions(localeMsg.value.toolbar.filter?.file_type_options);
});

// sort type options
const sortOptions = computed(() => {
  return getSelectOptions(localeMsg.value.toolbar.filter?.sort_type_options);
});

// sort extend options
const sortExtendOptions = computed(() => {
  return getSelectOptions(localeMsg.value.toolbar.filter?.sort_order_options);
});

function getSelectOptions(options) {
  const result = [];
  for (let i = 0; options && i < options.length; i++) {
    result.push({ label: options[i], value: i });
  }
  return result;
}

// get selected image source
const getImageSrc = async (index) => {
  if(index < 0 || index >= fileList.value.length) {
    imageSrc.value = '';
    return;
  }
  
  let filePath = fileList.value[index].file_path;
  console.log('getImageSrc:', filePath);
  try {
    let currentIndex = index;
    const imageBase64 = await getFileImage(filePath);
    // Check if the selected item has changed since the invocation
    if (currentIndex === index) {
      imageSrc.value = `data:image/jpeg;base64,${imageBase64}`;
    }
  } catch (error) {
    imageSrc.value = '';
    console.error('getImageSrc error:', error);
  }
}

function refreshFileList() {
  selectMode.value = false;   // exit multi-select mode

  if(fileList.value.length > 0) {
    if(fileListOffset.value === 0) {  // file list is changed
      selectedItemIndex.value = 0;
      updateSelectedImage(selectedItemIndex.value);
    }
    
    getFileListThumb(fileList.value, fileListOffset.value); 
  } else {
    selectedItemIndex.value = -1;
  }
  console.log('refreshFileList:', fileList.value);
}

// update image when the select file is changed
function updateSelectedImage(index) {
  // scroll to the selected item
  gridViewRef.value.scrollToItem(index); 

  // update the preview
  if(config.showPreview) {
    getImageSrc(index);
  }

  // update image viewer if the viewer is open
  openImageViewer(index, false);
}

// Get the thumbnail for the files
async function getFileListThumb(files, offset, concurrencyLimit = 8) {
  const result = [];
  let activeRequests = 0;
  thumbCount.value = 0;

  const getThumbForFile = async (file) => {
    console.log('getFileListThumb:', file);
    const thumb = await getFileThumb(file.id, file.file_path, file.file_type, file.e_orientation || 0, config.thumbnailImageSize);
    if(thumb) {
      file.thumbnail = `data:image/jpeg;base64,${thumb.thumb_data_base64}`;
      thumbCount.value++;
    }
    return file;
  };

  const runWithConcurrencyLimit = async (files) => {
    const queue = [];

    for (let i = offset; i < files.length; i++) {
      if (activeRequests >= concurrencyLimit) {
        await Promise.race(queue); // Wait for the first promise to complete
      }

      const filePromise = getThumbForFile(files[i])
        .then((file) => {
          // Remove the finished promise from the queue
          queue.splice(queue.indexOf(filePromise), 1);
          activeRequests--;
          return file;
        })
        .catch((error) => {
          console.log('Error fetching thumbnail:', error);
        });

      queue.push(filePromise);
      activeRequests++;
    }

    return Promise.all(queue);
  };

  result.push(...await runWithConcurrencyLimit(files));
  console.log('All thumbnails fetched successfully.');
}

// Open the image viewer window
async function openImageViewer(index: number, newViewer = false) {
  const webViewLabel = 'imageviewer';

  const fileCount = fileList.value.length;

  const file = index >= 0 && index < fileCount ? fileList.value[index] : null;
  const fileId = file ? file.id : 0;
  const encodedFilePath = file ? encodeURIComponent(file.file_path) : '';

  // preload the next image for smooth transition
  const nextFile = index + 1 >= 0 && index + 1 < fileCount ? fileList.value[index + 1] : null;
  const nextEncodedFilePath = nextFile ? encodeURIComponent(nextFile.file_path) : '';
  
  // create a new window if it doesn't exist
  let imageWindow = await WebviewWindow.getByLabel(webViewLabel);
  if (!imageWindow) {
    if (newViewer) {
      imageWindow = new WebviewWindow(webViewLabel, {
        url: `/image-viewer?fileId=${fileId}&fileIndex=${index}&fileCount=${fileCount}` + 
                           `&filePath=${encodedFilePath}&nextFilePath=${nextEncodedFilePath}`,
        title: 'Image Viewer',
        width: 1200,
        height: 800,
        minWidth: 800,
        minHeight: 600,
        resizable: true,
        decorations: isMac,
        transparent: isWin,
        ...(isMac && {
          titleBarStyle: 'Overlay',
          hiddenTitle: true,
          minimizable: false,
        }),
      });

      imageWindow.once('tauri://created', () => {
        console.log('ImageViewer window created');
      });

      imageWindow.once('tauri://close-requested', () => {
        imageWindow?.close();
        console.log('ImageViewer window is closing');
      });

      imageWindow.once('tauri://error', (e) => {
        console.error('Error creating ImageViewer window:', e);
      });
    }
  } else {    // update the existing window
    await imageWindow.emit('update-img', { 
      fileId: fileId, 
      fileIndex: index,   // selected file index
      fileCount: fileCount, // total files length
      filePath: encodedFilePath, 
      nextFilePath: nextEncodedFilePath,
    });
    if(newViewer) {
      imageWindow.show();
    }
  }
}

/// Dragging the splitter
function startDragging(event) {
  isDraggingSplitter.value = true;
  document.addEventListener('mousemove', handleMouseMove);
  document.addEventListener('mouseup', stopDragging);
}

/// stop dragging the splitter
function stopDragging() {
  isDraggingSplitter.value = false;
  document.removeEventListener('mousemove', handleMouseMove);
  document.removeEventListener('mouseup', stopDragging);
}

/// handle mouse move event
function handleMouseMove(event) {
  // console.log('handleMouseMove:', document.documentElement.clientWidth, event.clientX, leftPosition);
  if (isDraggingSplitter.value) {
    const windowWidth = document.documentElement.clientWidth - 4; // -4: border width(2px) * 2
    const leftPosition = divListView.value.getBoundingClientRect().left - 2;  // -2: border width(2px)

    // Limit width between 20% and 80%
    config.previewPaneWidth = Math.min(Math.max(((windowWidth - event.clientX)*100) / (windowWidth - leftPosition), 20), 80); 
  }
}

</script>

<style scoped>
* {
  user-select: none;
}
</style>
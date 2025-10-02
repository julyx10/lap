<template>
  <div ref="scrollContainer" 
    class="mb-1 flex-1 overflow-auto focus:outline-none" 
    tabindex="0" 
    @focus="isFocus = true"
    @blur="isFocus = false"
    @scroll="handleScroll"
  >
    <div id="gridView" 
      class="px-2 grid gap-2"
      :style="{ gridTemplateColumns: `repeat(auto-fit, minmax(${config.thumbnailSize}px, 1fr))` }"
    >
      <div 
        v-for="(file, index) in fileList" 
        :key="index"
        :id="'item-' + index"
        :class="[
          'p-2 border-2 rounded-lg hover:bg-base-100 cursor-pointer group',
          (selectMode ? file.isSelected : index === selectedIndex) ? 'border-primary' : 'border-transparent',
        ]"
        @click="clickItem(index)"
        @dblclick="openItem()"
      >
        <div class="relative flex flex-col items-center group">
          <img v-if="file.thumbnail"
            :src="file.thumbnail"
            :class="[
              'transition duration-200',
              config.thumbnailScalingOption === 0 ? 'object-contain' : '',
              config.thumbnailScalingOption === 1 ? 'object-cover rounded' : '',
              config.thumbnailScalingOption === 2 ? 'object-fill rounded' : ''
            ]"
            :style="{ 
              width: `${config.thumbnailSize}px`, height: `${config.thumbnailSize}px`, 
              transform: `rotate(${file.rotate}deg)`, 
              transition: 'transform 0.3s ease-in-out' 
            }"
            loading="lazy"
          />
          <div v-else 
            class="skeleton rounded flex items-center justify-center"
            :style="{ width: `${config.thumbnailSize}px`, height: `${config.thumbnailSize}px` }"
          > </div>
          <span class="pt-1 text-sm text-center">{{ getThumbnailText(file, config.thumbnailLabelPrimaryOption) }}</span>
          <span class="text-sm text-center">{{ getThumbnailText(file, config.thumbnailLabelSecondaryOption) }}</span>
        
          <!-- status icons -->
          <div class="absolute left-1 top-1 flex items-center gap-1 text-sm text-base-content/30">
            <IconVideo v-if="file.file_type===2" class="t-icon-size-xs"></IconVideo>
            <IconFavorite v-if="file.is_favorite" class="t-icon-size-xs"></IconFavorite>
            <IconRotate v-if="file.rotate % 360 > 0"
              class="t-icon-size-xs"
              :style="{ 
                transform: `rotate(${file.rotate}deg)`, 
                transition: 'transform 0.3s ease-in-out' 
              }"
            />
            <IconTag v-if="file.has_tags" class="t-icon-size-xs "></IconTag>
            <IconComment v-if="file.comments?.length > 0" class="t-icon-size-xs "></IconComment>              
          </div>

          <!-- select checkbox or more menu -->
          <div class="absolute right-0 top-0 flex items-center">
            <component v-if="selectMode"
              :is="file?.isSelected ? IconChecked : IconUnChecked" 
              :class="['t-icon-size-sm hover:text-base-content/70', file?.isSelected ? 'text-primary' : 'text-gray-500']" 
              @click.stop="selectItem(index)"
            />
            <DropDownMenu v-else
              :class="[
                index != selectedIndex ? 'invisible group-hover:visible' : ''
              ]"
              :iconMenu="IconMore"
              :menuItems="moreMenuItems"
              :smallIcon="true"
            />
          </div>
        </div>
      </div>
    </div>

    <div v-if="fileList.length === 0" class="flex flex-col items-center justify-center w-full h-full text-base-content/30">
      <IconSearch class="w-8 h-8" />
      <span>{{ $t('tooltip.not_found.files') }}</span>
    </div>

  </div>

</template>

<script setup lang="ts">

import { ref, watch, computed, nextTick } from 'vue';
import { emit } from '@tauri-apps/api/event';
import { useI18n } from 'vue-i18n';
import { config, isMac, shortenFilename, formatFileSize, formatDuration, formatTimestamp } from '@/common/utils';
import DropDownMenu from '@/components/DropDownMenu.vue';

import { 
  IconMore,
  IconOpen,
  IconEdit,
  IconPrint,
  IconFavorite,
  IconUnFavorite,
  IconTag,
  IconRotate,
  IconCopy,
  IconRename,
  IconMoveTo,
  IconTrash,
  IconGoto,
  IconSearch,
  IconVideo,
  IconAudio,
  IconChecked,
  IconUnChecked,
  IconComment,
} from '@/common/icons';

const props = defineProps({
  selectItemIndex: {     // v-model value
    type: Number,
    required: true,
  },
  fileList: {
    type: Array,
    required: true,
  },
  showFolderFiles: {             
    type: Boolean,
    default: false,
  },
  searchMode: {
    type: Boolean,
    default: false,
  },
  selectMode: {
    type: Boolean,
    default: false,
  },
});

/// i18n
const { locale, messages } = useI18n();
const localeMsg = computed(() => messages.value[locale.value]);

// when the grid view is focused, the keydown event is listened
const isFocus = ref(false);

const selectedIndex = ref(props.selectItemIndex);
const emitUpdate = defineEmits(['update:selectItemIndex']);

const scrollContainer = ref(null); // Ref for the scrollable element



const moreMenuItems = computed(() => {
  if (selectedIndex.value < 0 || selectedIndex.value >= props.fileList.length) {
    return [];
  }

  const file = props.fileList[selectedIndex.value];
  return [
    {
      label: localeMsg.value.menu.file.open,
      icon: IconOpen,
      shortcut: isMac ? '⌘⏎' : 'Ctrl+Enter',
      action: () => {
        openItem();
      }
    },
    {
      label: localeMsg.value.menu.file.copy,
      icon: IconCopy,
      shortcut: isMac ? '⌘C' : 'Ctrl+C',
      action: () => {
        copyItem();
      }
    },
    {
      label: "-",   // separator
      action: () => {}
    },
    {
      label: localeMsg.value.menu.file.rename,
      icon: IconRename,
      action: () => {
        renameItem();
      }
    },
    // {
    //   label: localeMsg.value.menu.edit,
    //   icon: IconEdit,
    //   action: () => {
    //     emit('message-from-grid-view', { message: 'edit' });
    //   }
    // },
    // {
    //   label: localeMsg.value.menu.print,
    //   icon: IconPrint,
    //   action: async () => {
    //     emit('message-from-grid-view', { message: 'print' });
    //   }
    // },
    {
      label: localeMsg.value.menu.file.move_to,
      icon: IconMoveTo,
      action: () => {
        moveTo();
      }
    },
    {
      label: localeMsg.value.menu.file.copy_to,
      // icon: IconCopyTo,
      action: () => {
        copyTo();
      }
    },
    {
      label: isMac ? localeMsg.value.menu.file.move_to_trash : localeMsg.value.menu.file.delete,
      icon: IconTrash,
      shortcut: isMac ? '⌘⌫' : 'Del',
      action: () => {
        deleteItem();
      }
    },
    {
      label: localeMsg.value.menu.file.goto_folder,
      disabled: props.showFolderFiles,
      icon: IconGoto,
      action: () => {
        gotoFolder();
      }
    },
    {
      label: isMac ? localeMsg.value.menu.file.reveal_in_finder : localeMsg.value.menu.file.reveal_in_file_explorer,
      // icon: IconOpenFolder,
      action: () => {
        revealItem();
      }
    },
    {
      label: "-",   // separator
      action: null
    },
    {
      label: file.is_favorite ? localeMsg.value.menu.meta.unfavorite : localeMsg.value.menu.meta.favorite,
      icon: file.is_favorite ? IconUnFavorite : IconFavorite,
      shortcut: isMac ? '⌘F' : 'Ctrl+F',
      action: () => {
        toggleFavorite();
      }
    },
    {
      label: localeMsg.value.menu.meta.tag,
      icon: IconTag,
      shortcut: isMac ? '⌘T' : 'Ctrl+T',
      action: () => {
        tagItem();
      }
    },
    {
      label: localeMsg.value.menu.meta.rotate,
      icon: IconRotate,
      shortcut: isMac ? '⌘R' : 'Ctrl+R',
      action: () => {
        rotateItem();
      }
    },
    {
      label: localeMsg.value.menu.meta.comment,
      icon: IconComment,
      action: () => {
        commentItem();
      }
    }
  ];
});

// if search mode, do not handle the keydown event
watch(() => props.searchMode, (newValue) => {
  if (newValue) {
    window.removeEventListener('keydown', handleKeyDown);
  } else {
    window.addEventListener('keydown', handleKeyDown);
  }
}, { immediate: true });

watch(() => props.selectItemIndex, (newValue) => { 
  selectedIndex.value = newValue; 
});

watch(() => selectedIndex.value, (newValue) => {
  emitUpdate('update:selectItemIndex', newValue);
  scrollToItem(newValue);
});

function clickItem(index: number) {
  selectedIndex.value = index;

  if(props.selectMode) {
    selectItem(index);
  }
}

function handleKeyDown(event) {
  const key = event.key;
  const isCmdKey = isMac ? event.metaKey : event.ctrlKey;

  if (isCmdKey && key === 'Enter') {   // Open shortcut
    event.preventDefault(); // Prevent the default action
    openItem();
  } else if (isCmdKey && key.toLowerCase() === 'c') {   // Copy shortcut
    event.preventDefault(); // Prevent the default action
    copyItem();
  } else if(isCmdKey && key.toLowerCase() === 'f') {
    event.preventDefault(); // Prevent the default action
    toggleFavorite();
  } else if(isCmdKey && key.toLowerCase() === 't') {
    event.preventDefault(); // Prevent the default action
    tagItem();
  } else if(isCmdKey && key.toLowerCase() === 'r') {
    event.preventDefault(); // Prevent the default action
    rotateItem();
  } else if((isMac && event.metaKey && key === 'Backspace') || (!isMac && key === 'Delete')) {
    event.preventDefault(); // Prevent the default action
    deleteItem();
  } else if (keyActions[key]) {
    event.preventDefault(); // Prevent the default action
    keyActions[key](); 
  }
}

// key actions
const keyActions = {
  ArrowDown: ()  => selectedIndex.value = Math.min(selectedIndex.value + getColumnCount(), props.fileList.length - 1),
  ArrowRight: () => selectedIndex.value = Math.min(selectedIndex.value + 1, props.fileList.length - 1),
  ArrowUp: ()    => selectedIndex.value = Math.max(selectedIndex.value - getColumnCount(), 0),
  ArrowLeft: ()  => selectedIndex.value = Math.max(selectedIndex.value - 1, 0),
  Home: ()       => selectedIndex.value = 0,
  End: ()        => selectedIndex.value = props.fileList.length - 1,
  // Enter: ()      => openItem(),
  '/': ()        => searchFile(),
};

function searchFile() {
  emit('message-from-grid-view', { message: 'search' });
};

function selectItem(index: number) {
  emit('message-from-grid-view', { message: 'select', index });
};

function openItem() {
  emit('message-from-grid-view', { message: 'open' });
};

function copyItem() {
  emit('message-from-grid-view', { message: 'copy' });
};

function renameItem() {
  emit('message-from-grid-view', { message: 'rename' });
};

function moveTo() {
  emit('message-from-grid-view', { message: 'move-to' });
};

function copyTo() {
  emit('message-from-grid-view', { message: 'copy-to' });
};

function deleteItem() {
  emit('message-from-grid-view', { message: 'delete' });
};

function gotoFolder() {
  emit('message-from-grid-view', { message: 'goto-folder' });
};

function revealItem() {
  emit('message-from-grid-view', { message: 'reveal' });
};

function toggleFavorite() {
  emit('message-from-grid-view', { message: 'favorite' });
};

function tagItem() {
  emit('message-from-grid-view', { message: 'tag' });
};

function rotateItem() {
  emit('message-from-grid-view', { message: 'rotate' });
};

function commentItem() {
  emit('message-from-grid-view', { message: 'comment' });
};

function nextPage() {
  emit('message-from-grid-view', { message: 'next-page' });
};

// function to get the text for the thumbnail
const getThumbnailText = (file, option) => {
  switch (option) {
    case 0:   // empty
      return '';
    case 1:   // name
      return shortenFilename(file.name);
    case 2:   // size
      return formatFileSize(file.size);
    case 3:   // dimension
      return file.width > 0 && file.height > 0 ? `${file.width}x${file.height}` : '-';
    case 4:   // duration
      return file.duration > 0 ? formatDuration(file.duration): '-';
    case 5:   // created time
      return formatTimestamp(file.created_at, localeMsg.value.format.date_time);
    case 6:   // modified time
      return formatTimestamp(file.modified_at, localeMsg.value.format.date_time);
    case 7:   // date taken
      return file.e_date_time || '-';
    default:
      return '';
  }
};

// function to handle the scroll event
function handleScroll() {
  const el = scrollContainer.value;

  // scroll to the bottom of the container
  if (el.scrollTop + el.clientHeight >= el.scrollHeight - 200) {  // -1: Small offset to handle floating point errors
    nextPage();
  }
}

// make the selected item always visible in a scrollable container
function scrollToItem(index) {
  nextTick(() => {
    const item = document.getElementById(`item-${index}`);
    if (item) {
      item.scrollIntoView({ behavior: 'auto', block: 'nearest' });
    }
  });
};

// function to get the number of columns in the grid
function getColumnCount() {
  // get the first element with the id 'gridView'
  const gridContainer = document.querySelector('#gridView');

  const computedStyle = getComputedStyle(gridContainer);
  const gridTemplateColumns = computedStyle.gridTemplateColumns;

  // Split by space to account for grid definitions
  const columnCount = gridTemplateColumns.split(' ').length;

  return columnCount;
}

defineExpose({ 
  scrollToItem,
});

</script>

<style scoped>
</style>

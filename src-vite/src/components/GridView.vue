<template>
  <div ref="scrollable" class="mb-1 flex-1 overflow-auto t-scrollbar focus:outline-none" 
    tabindex="0" @focus="isFocus = true"  @blur="isFocus = false"
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
          'p-1 border-2 rounded-lg hover:text-gray-300 hover:bg-gray-600 cursor-pointer transition duration-200 group', 
          !selectMode && index === selectedIndex ? 'border-sky-500' : 'border-gray-800',
          selectMode && file.isSelected ? 'border-sky-500' : 'border-gray-800',
        ]"
        @click="clickItem(index)"
        @dblclick="openItem()"
      >
        <div v-if="!file.is_deleted" class="relative flex flex-col items-center group">
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
            class="rounded flex items-center justify-center"
            :style="{ width: `${config.thumbnailSize}px`, height: `${config.thumbnailSize}px` }"
          >
            <IconPhoto class="size-1/2"/>
          </div>
          <span class="pt-1 text-sm text-center">{{ getThumbnailText(file, config.thumbnailLabelPrimaryOption) }}</span>
          <span class="text-sm text-center">{{ getThumbnailText(file, config.thumbnailLabelSecondaryOption) }}</span>
        
          <!-- favorite and rotate status -->
          <div class="absolute left-0 top-0 flex items-center">
            <IconFavorite v-if="file.is_favorite" class="t-icon-size-sm group-hover:text-gray-500"></IconFavorite>
            <IconRotate v-if="file.rotate % 360 > 0"
              class="t-icon-size-sm group-hover:text-gray-500"
              :style="{ 
                transform: `rotate(${file.rotate}deg)`, 
                transition: 'transform 0.3s ease-in-out' 
              }"
            />
            <IconComment v-if="file.comments?.length > 0" class="t-icon-size-sm group-hover:text-gray-500"></IconComment>
          </div>

          <!-- select checkbox or more menu -->
          <div class="absolute right-0 top-0 flex items-center">
            <component v-if="selectMode"
              :is="file?.isSelected ? IconChecked : IconUnChecked" 
              :class="['t-icon-size-sm t-icon-hover', file?.isSelected ? 'text-sky-500' : 'text-gray-500']" 
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

    <div v-if="fileList.length === 0" class="flex items-center justify-center w-full h-full">
      {{ $t('file_list_no_files') }}
    </div>

  </div>

</template>


<script setup lang="ts">

import { ref, watch, computed } from 'vue';
import { emit } from '@tauri-apps/api/event';
import { useI18n } from 'vue-i18n';
import { config, isMac, shortenFilename, formatFileSize, formatTimestamp } from '@/common/utils';
import DropDownMenu from '@/components/DropDownMenu.vue';

import { 
  IconMore,
  IconOpen,
  IconEdit,
  IconPrint,
  IconFavorite,
  IconUnFavorite,
  IconRotate,
  IconCopy,
  IconRename,
  IconMoveTo,
  IconDelete,
  IconPhoto,
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

const scrollable = ref(null); // Ref for the scrollable element

const moreMenuItems = computed(() => {
  if (selectedIndex.value < 0 || selectedIndex.value >= props.fileList.length) {
    return [];
  }

  const file = props.fileList[selectedIndex.value];
  return [
    {
      label: localeMsg.value.menu_item_open,
      icon: IconOpen,
      shortcut: isMac ? '⏎' : 'Enter',
      action: () => {
        openItem();
      }
    },
    {
      label: localeMsg.value.menu_item_copy,
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
      label: localeMsg.value.menu_item_rename,
      icon: IconRename,
      action: () => {
        renameItem();
      }
    },
    // {
    //   label: localeMsg.value.menu_item_edit,
    //   icon: IconEdit,
    //   action: () => {
    //     emit('message-from-grid-view', { message: 'edit' });
    //   }
    // },
    // {
    //   label: localeMsg.value.menu_item_print,
    //   icon: IconPrint,
    //   action: async () => {
    //     emit('message-from-grid-view', { message: 'print' });
    //   }
    // },
    {
      label: localeMsg.value.menu_item_move_to,
      icon: IconMoveTo,
      action: () => {
        moveTo();
      }
    },
    {
      label: localeMsg.value.menu_item_copy_to,
      // icon: IconCopyTo,
      action: () => {
        copyTo();
      }
    },
    {
      label: localeMsg.value.menu_item_delete,
      icon: IconDelete,
      shortcut: isMac ? '⌘⌫' : 'Del',
      action: () => {
        deleteItem();
      }
    },
    {
      label: isMac ? localeMsg.value.menu_item_reveal_in_finder : localeMsg.value.menu_item_reveal_in_file_explorer,
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
      label: file.is_favorite ? localeMsg.value.menu_item_unfavorite : localeMsg.value.menu_item_favorite,
      icon: file.is_favorite ? IconUnFavorite : IconFavorite,
      shortcut: isMac ? '⌘F' : 'Ctrl+F',
      action: () => {
        toggleFavorite();
      }
    },
    {
      label: localeMsg.value.menu_item_rotate,
      icon: IconRotate,
      shortcut: isMac ? '⌘R' : 'Ctrl+R',
      action: () => {
        rotateItem();
      }
    },
    {
      label: localeMsg.value.menu_item_comment,
      icon: IconComment,
      action: () => {
        commentItem();
      }
    }
  ];
});

// when the grid view is focused, the keydown event is listened
watch(() => isFocus.value, (newValue) => {
  if (newValue) {
    window.addEventListener('keydown', handleKeyDown);
  } else {
    window.removeEventListener('keydown', handleKeyDown);
  }
});

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

  if (isCmdKey && key.toLowerCase() === 'c') {   // Copy shortcut
    event.preventDefault(); // Prevent the default action
    copyItem();
  } else if(isCmdKey && key.toLowerCase() === 'f') {
    event.preventDefault();
    toggleFavorite();
  } else if(isCmdKey && key.toLowerCase() === 'r') {
    event.preventDefault();
    rotateItem();
  } else if((isMac && event.metaKey && key === 'Backspace') || (!isMac && key === 'Delete')) {
    event.preventDefault();
    deleteItem()
  } else if (keyActions[key]) {
    event.preventDefault(); 
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
  Enter: ()      => openItem(),
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

function revealItem() {
  emit('message-from-grid-view', { message: 'reveal' });
};

function toggleFavorite() {
  emit('message-from-grid-view', { message: 'favorite' });
};

function rotateItem() {
  emit('message-from-grid-view', { message: 'rotate' });
};

function commentItem() {
  emit('message-from-grid-view', { message: 'comment' });
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
    case 3:   // resolution
      return `${file.width}x${file.height}`;
    case 4:   // created time
      return formatTimestamp(file.created_at, localeMsg.value.date_time_format);
    case 5:   // modified time
      return formatTimestamp(file.modified_at, localeMsg.value.date_time_format);
    case 6:   // date taken
      return file.e_date_time || '-';
    default:
      return '';
  }
};

// make the selected item always visible in a scrollable container
function scrollToItem(index) {
  const item = document.getElementById(`item-${index}`);
  if (item) {
    item.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
  }
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

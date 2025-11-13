<template>
  <div ref="scrollContainer" 
    class="flex-1 overflow-auto focus:outline-none" 
    :class="{ 
      'pointer-events-none': uiStore.inputStack.length > 0,
      'rounded-r-lg': config.content.showFileInfo
    }"
    tabindex="0" 
    @focus="isFocus = true"
    @blur="isFocus = false"
    @scroll="handleScroll"
    @keydown="handleNativeKeyDown"
  >
    <div id="gridView" 
      class="p-2 grid gap-2"
      :style="{ gridTemplateColumns: `repeat(auto-fit, minmax(${config.settings.grid.size}px, 1fr))` }"
    >
      <!-- thumbnail -->
      <Thumbnail
        v-for="(file, index) in fileList" 
        :key="index"
        :id="'item-' + index"
        :file="file"
        :is-selected="selectMode ? file.isSelected : index === selectedIndex"
        :select-mode="selectMode"
        :show-folder-files="showFolderFiles"
        @clicked="clickItem(index)"
        @dblclicked="openItem()"
        @select-toggled="selectItem(index)"
        @action="handleThumbnailAction"
      />
    </div>

    <div v-if="fileList.length === 0" class="flex flex-col items-center justify-center w-full h-full text-base-content/30">
      <IconSearch class="w-8 h-8" />
      <span>{{ $t('tooltip.not_found.files') }}</span>
    </div>

  </div>

</template>

<script setup lang="ts">

import { ref, watch, computed, nextTick, onMounted, onUnmounted } from 'vue';
import { emit, listen } from '@tauri-apps/api/event';
import { useI18n } from 'vue-i18n';
import { useUIStore } from '@/stores/uiStore';
import { config, isMac } from '@/common/utils';
import Thumbnail from '@/components/Thumbnail.vue';

import { 
  IconSearch,
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
  selectMode: {
    type: Boolean,
    default: false,
  },
});

/// i18n
const { locale, messages } = useI18n();
const localeMsg = computed(() => messages.value[locale.value]);
const uiStore = useUIStore();

// when the grid view is focused, the keydown event is listened
const isFocus = ref(false);

const selectedIndex = ref(props.selectItemIndex);
const emitUpdate = defineEmits(['update:selectItemIndex']);

const scrollContainer = ref(null); // Ref for the scrollable element

let unlistenKeydown: () => void;

onMounted(async () => {
  unlistenKeydown = await listen('global-keydown', handleKeyDown);
});

onUnmounted(() => {
  if (unlistenKeydown) {
    unlistenKeydown();
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

function handleThumbnailAction(action: string) {
  const actionMap = {
    'open': openItem,
    'edit': editItem,
    'copy': copyItem,
    'update-from-file': updateItem,
    'goto-folder': gotoFolder,
    'reveal': revealItem,
    'rename': renameItem,
    'move-to': moveTo,
    'copy-to': copyTo,
    'trash': trashItem,
    'favorite': toggleFavorite,
    'tag': tagItem,
    'comment': commentItem,
    'rotate': rotateItem,
  };

  if (actionMap[action]) {
    actionMap[action]();
  }
}

function handleKeyDown(event) {
  if (uiStore.inputStack.length > 0) {
    return;
  }
  
  const { key, metaKey } = event.payload;
  const isCmdKey = isMac ? metaKey : event.payload.ctrlKey;

  if (isCmdKey && key === 'Enter') {   // Open shortcut
    openItem();
  } else if (isCmdKey && key.toLowerCase() === 'c') {   // Copy shortcut
    copyItem();
  } else if(isCmdKey && key.toLowerCase() === 'e') {
    editItem();
  } else if(isCmdKey && key.toLowerCase() === 'f') {
    toggleFavorite();
  } else if(isCmdKey && key.toLowerCase() === 't') {
    tagItem();
  } else if(isCmdKey && key.toLowerCase() === 'r') {
    rotateItem();
  } else if((isMac && metaKey && key === 'Backspace') || (!isMac && key === 'Delete')) {
    trashItem();
  } else if (keyActions[key]) {
    keyActions[key](); 
  }
}

function handleNativeKeyDown(event: KeyboardEvent) {
  // Keys that we are handling manually for navigation, to prevent default browser behavior (scrolling).
  const handledKeys = ['ArrowUp', 'ArrowDown', 'ArrowLeft', 'ArrowRight', 'Home', 'End', ' '];

  if (handledKeys.includes(event.key)) {
    event.preventDefault();
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
  if (props.fileList[selectedIndex.value].file_type !== 1) {
    return;
  }
  emit('message-from-grid-view', { message: 'copy' });
};

function editItem() {
  if (props.fileList[selectedIndex.value].file_type !== 1) {
    return;
  }
  emit('message-from-grid-view', { message: 'edit' });
};

function updateItem() {
  emit('message-from-grid-view', { message: 'update-from-file' });
}

function revealItem() {
  emit('message-from-grid-view', { message: 'reveal' });
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

function trashItem() {
  emit('message-from-grid-view', { message: 'trash' });
};

function gotoFolder() {
  emit('message-from-grid-view', { message: 'goto-folder' });
};

function toggleFavorite() {
  emit('message-from-grid-view', { message: 'favorite' });
};

function rotateItem() {
  emit('message-from-grid-view', { message: 'rotate' });
};

function tagItem() {
  emit('message-from-grid-view', { message: 'tag' });
};

function commentItem() {
  emit('message-from-grid-view', { message: 'comment' });
};

function nextPage() {
  emit('message-from-grid-view', { message: 'next-page' });
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

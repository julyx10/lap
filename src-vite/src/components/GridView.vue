<template>
  <div
    class="relative w-full h-full focus:outline-none" 
    :class="{ 
      'pointer-events-none': uiStore.inputStack.length > 0,
    }"
    tabindex="0" 
  >
    <div id="gridView" 
      class="p-2 gap-2"
      :class="{
        'grid': config.content.layout === 0,
        'absolute w-full h-full flex flex-nowrap items-center overflow-x-auto overflow-y-hidden': config.content.layout === 1
      }"
      :style="config.content.layout === 0 ? { gridTemplateColumns: `repeat(auto-fit, minmax(${config.settings.grid.size}px, 1fr))` } : { }"
    >
      <!-- thumbnail -->
      <Thumbnail
        v-for="(file, index) in fileList"
        :key="index"
        :id="'item-' + index"
        :file="file"
        :is-selected="selectMode ? file.isSelected : index === selectedItemIndex"
        :select-mode="selectMode"
        :show-folder-files="showFolderFiles"
        @clicked="$emit('item-clicked', index)"
        @dblclicked="$emit('item-dblclicked', index)"
        @select-toggled="$emit('item-select-toggled', index)"
        @action="(actionName) => $emit('item-action', { action: actionName, index: index })"
      />
    </div>

    <div v-if="fileList.length === 0" class="flex flex-col items-center justify-center w-full h-full text-base-content/30">
      <IconSearch class="w-8 h-8" />
      <span>{{ $t('tooltip.not_found.files') }}</span>
    </div>

  </div>

</template>

<script setup lang="ts">

import { watch, nextTick } from 'vue';
import { useUIStore } from '@/stores/uiStore';
import { config } from '@/common/utils';
import Thumbnail from '@/components/Thumbnail.vue';

import { 
  IconSearch,
} from '@/common/icons';

const props = defineProps({
  selectedItemIndex: {
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

defineEmits([
  'item-clicked',
  'item-dblclicked',
  'item-select-toggled',
  'item-action',
]);

const uiStore = useUIStore();

watch(() => props.selectedItemIndex, (newValue) => {
  scrollToItem(newValue);
});

// make the selected item always visible in a scrollable container
function scrollToItem(index: number) {
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
  if (!gridContainer) return 1;

  const computedStyle = getComputedStyle(gridContainer);
  const gridTemplateColumns = computedStyle.gridTemplateColumns;

  // Split by space to account for grid definitions
  const columnCount = gridTemplateColumns.split(' ').length;

  return columnCount;
}

defineExpose({ 
  scrollToItem,
  getColumnCount,
});

</script>

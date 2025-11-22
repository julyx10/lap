<template>
  <div
    ref="gridViewRef"
    class="relative w-full min-h-full focus:outline-none" 
    :class="{ 
      'pointer-events-none': uiStore.inputStack.length > 0,
    }"
    tabindex="0" 
  >
    <div v-if="fileList.length > 0" id="gridView" 
      :class="[
        config.content.layout === 0 ? 'px-1 grid' : 'absolute flex flex-nowrap items-center',
        config.content.layout === 0 && config.settings.grid.style === 0 ? 'gap-1' : '',
      ]"
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

    <div v-else class="absolute inset-0 flex flex-col items-center justify-center text-base-content/30">
      <span>{{ config.home.sidebarIndex === 1 ? $t('tooltip.not_found.folder_files') : $t('tooltip.not_found.files') }}</span>
    </div>

  </div>

</template>

<script setup lang="ts">

import { watch, ref, onMounted, onBeforeUnmount } from 'vue';
import { useUIStore } from '@/stores/uiStore';
import { config } from '@/common/config';
import Thumbnail from '@/components/Thumbnail.vue';

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

const emit = defineEmits([
  'item-clicked',
  'item-dblclicked',
  'item-select-toggled',
  'item-action',
  'request-scroll',
]);

const uiStore = useUIStore();
const gridViewRef = ref(null);
let resizeObserver: ResizeObserver | null = null;

watch(() => props.selectedItemIndex, (newValue) => {
  if (newValue !== -1) {
    emit('request-scroll', newValue);
  }
});

watch(() => config.content.layout, () => {
  if (props.selectedItemIndex !== -1) {
    emit('request-scroll', props.selectedItemIndex);
  }
});

onMounted(() => {
  if (gridViewRef.value) {
    resizeObserver = new ResizeObserver(() => {
      if (props.selectedItemIndex !== -1) {
        emit('request-scroll', props.selectedItemIndex);
      }
    });
    resizeObserver.observe(gridViewRef.value);
  }
});

onBeforeUnmount(() => {
  if (resizeObserver) {
    resizeObserver.disconnect();
  }
});

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
  getColumnCount,
});

</script>

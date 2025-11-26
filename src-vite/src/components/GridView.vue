<template>
  <div
    ref="containerRef"
    class="relative w-full h-full focus:outline-none" 
    :class="{ 
      'pointer-events-none': uiStore.inputStack.length > 0,
    }"
    tabindex="0" 
  >
    <RecycleScroller
      v-if="config.content.layout === 0 && fileList.length > 0"
      ref="scroller"
      class="scroller h-full no-scrollbar"
      :items="fileList"
      :item-size="itemSize"
      :grid-items="columnCount"
      key-field="id"
      :emit-update="true"
      v-slot="{ item, index }"
      @update="onUpdate"
      @scroll="onScroll"
    >
      <div 
        class="flex justify-center items-center h-full w-full"
        :style="{ padding: gap / 2 + 'px' }"
      >
        <!-- Debug Info -->
        <div class="absolute top-0 left-0 bg-black/50 text-white text-[10px] z-50 p-1 pointer-events-none">
          {{ index }} {{ item.isPlaceholder ? 'PH' : 'F' }} {{ item.thumbnail ? 'T' : 'NoT' }}
        </div>
        
        <Thumbnail
          v-if="item && !item.isPlaceholder"
          :id="'item-' + index"
          :file="item"
          :is-selected="selectMode ? item.isSelected : index === selectedItemIndex"
          :select-mode="selectMode"
          :show-folder-files="showFolderFiles"
          @clicked="$emit('item-clicked', index)"
          @dblclicked="$emit('item-dblclicked', index)"
          @select-toggled="$emit('item-select-toggled', index)"
          @action="(actionName) => $emit('item-action', { action: actionName, index: index })"
        />
        <div v-else class="w-full h-full bg-base-200/50 rounded animate-pulse"></div>
      </div>
    </RecycleScroller>

    <!-- Filmstrip View (Layout 1) - Horizontal -->
    <div v-else-if="config.content.layout === 1 && fileList.length > 0" 
      id="gridView"
      class="absolute flex flex-nowrap items-center h-full overflow-x-auto"
    >
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

    <!-- Empty State -->
    <div v-if="fileList.length === 0" class="absolute inset-0 flex flex-col items-center justify-center text-base-content/30">
      <span>{{ config.home.sidebarIndex === 1 ? $t('tooltip.not_found.folder_files') : $t('tooltip.not_found.files') }}</span>
    </div>

  </div>

</template>

<script setup lang="ts">

import { watch, ref, onMounted, onBeforeUnmount, computed, nextTick } from 'vue';
import { useUIStore } from '@/stores/uiStore';
import { config } from '@/common/config';
import Thumbnail from '@/components/Thumbnail.vue';
// @ts-ignore
import { RecycleScroller } from 'vue-virtual-scroller';
import 'vue-virtual-scroller/dist/vue-virtual-scroller.css';

const props = withDefaults(defineProps<{
  selectedItemIndex: number;
  fileList: any[];
  showFolderFiles?: boolean;
  selectMode?: boolean;
}>(), {
  showFolderFiles: false,
  selectMode: false,
});

const emit = defineEmits([
  'item-clicked',
  'item-dblclicked',
  'item-select-toggled',
  'item-action',
  'request-scroll',
  'visible-range-update',
  'scroll',
]);

const uiStore = useUIStore();
const containerRef = ref<HTMLElement | null>(null);
const scroller = ref<any>(null);
const columnCount = ref(4);
const containerWidth = ref(0);

const gap = 4; // Gap between items

// item height including gap
const itemSize = computed(() => {
  return config.settings.grid.size + gap;
});

let resizeObserver: ResizeObserver | null = null;

function updateColumnCount() {
  if (containerRef.value) {
    containerWidth.value = containerRef.value.clientWidth;
    const size = itemSize.value;
    if (size > 0) {
      columnCount.value = Math.max(1, Math.floor(containerWidth.value / size));
    }
  }
}

watch(() => config.settings.grid.size, () => {
  updateColumnCount();
});

watch(() => props.selectedItemIndex, (newValue) => {
  if (newValue !== -1) {
    scrollToItem(newValue);
  }
});

onMounted(() => {
  if (containerRef.value) {
    resizeObserver = new ResizeObserver(() => {
      updateColumnCount();
    });
    resizeObserver.observe(containerRef.value);
    updateColumnCount();
  }
});

onBeforeUnmount(() => {
  if (resizeObserver) {
    resizeObserver.disconnect();
  }
});

function onUpdate(startIndex: number, endIndex: number) {
  console.log(`onUpdate: ${startIndex}-${endIndex}`);
  emit('visible-range-update', { startIndex, endIndex });
}

function onScroll(e: Event) {
  emit('scroll', e);
}

function scrollToItem(index: number) {
  if (scroller.value && config.content.layout === 0) {
    const el = scroller.value.$el;
    const itemHeight = itemSize.value;
    const row = Math.floor(index / columnCount.value);
    const itemTop = row * itemHeight;
    const itemBottom = itemTop + itemHeight;
    const scrollTop = el.scrollTop;
    const clientHeight = el.clientHeight;

    if (itemTop < scrollTop) {
      el.scrollTop = itemTop;
    } else if (itemBottom > scrollTop + clientHeight) {
      el.scrollTop = itemBottom - clientHeight;
    }
  } else {
    emit('request-scroll', index);
  }
}

function scrollToPosition(scrollTop: number) {
  if (scroller.value && config.content.layout === 0) {
    scroller.value.$el.scrollTop = scrollTop;
  }
}

function getColumnCount() {
  return columnCount.value;
}

defineExpose({
  getColumnCount,
  scrollToItem,
  scrollToPosition,
});

</script>

<style scoped>
.scroller {
  height: 100%;
}
</style>

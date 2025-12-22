<template>
  <div
    ref="containerRef"
    class="w-full h-full" 
    :class="{ 
      'pointer-events-none': uiStore.inputStack.length > 0,
    }"
    @wheel="onWheel"
  >
    <VirtualScroll
      v-if="fileList.length > 0"
      ref="scroller"
      class="w-full h-full no-scrollbar"
      :class="{
        'pt-12': !config.content.showFilmStrip,
        'pb-8': !config.content.showFilmStrip && config.settings.showStatusBar,
        'pb-1': !config.content.showFilmStrip && !config.settings.showStatusBar,
      }"
      :items="fileList"
      :direction="!config.content.showFilmStrip ? 'vertical' : 'horizontal'"
      :grid-items="!config.content.showFilmStrip ? columnCount : undefined"
      :item-size="!config.content.showFilmStrip ? itemHeight : filmStripItemSize"
      :item-secondary-size="!config.content.showFilmStrip ? itemWidth : undefined"
      :key="config.content.showFilmStrip ? 'filmstrip' : 'grid'"
      key-field="id"
      :emit-update="true"
      :buffer="4"
      v-slot="{ item, index }"
      @update="onUpdate"
      @scroll="onScroll"
    >
      <!-- Debug Info -->
      <!-- <div class="absolute top-10 left-4 bg-black/50 text-white text-[10px] z-50 p-1 pointer-events-none">
        {{ index }} {{ item.isPlaceholder ? 'PH' : 'F' }} {{ item.thumbnail ? 'T' : 'NoT' }} {{ item.id }}
      </div> -->
      
      <Thumbnail
        v-if="item && !item.isPlaceholder"
        :id="'item-' + index"
        :file="item"
        :is-selected="selectMode ? item.isSelected : index === selectedItemIndex"
        :select-mode="selectMode"
        :show-folder-files="showFolderFiles"
        @clicked="$emit('item-clicked', index)"
        @dblclicked="$emit('item-dblclicked', index)"
        @select-toggled="(shiftKey) => $emit('item-select-toggled', index, shiftKey)"
        @action="(actionName) => $emit('item-action', { action: actionName, index: index })"
      />
      <div v-else class="w-full h-full bg-base-200/50 rounded animate-pulse"></div>
    </VirtualScroll>
    <!-- Empty State / Loading -->
    <div v-else class="absolute inset-0 flex flex-col items-center justify-center">
      <div v-if="loading" class="flex flex-col items-center gap-y-2">
        <span class="loading loading-spinner loading-lg text-primary"></span>
        <span class="text-sm text-base-content/50">{{ $t('tooltip.loading') }}</span>
      </div>
      <div v-else class="text-base-content/30 flex flex-col items-center">
        <span>{{ config.main.sidebarIndex === 1 ? $t('tooltip.not_found.folder_files') : $t('tooltip.not_found.files') }}</span>
      </div>
    </div>

  </div>

</template>

<script setup lang="ts">

import { watch, ref, onMounted, onBeforeUnmount, computed } from 'vue';
import { useUIStore } from '@/stores/uiStore';
import { config } from '@/common/config';
import Thumbnail from '@/components/Thumbnail.vue';
import VirtualScroll from '@/components/VirtualScroll.vue';

const props = withDefaults(defineProps<{
  selectedItemIndex: number;
  fileList: any[];
  showFolderFiles?: boolean;
  selectMode?: boolean;
  loading?: boolean;
}>(), {
  selectedItemIndex: -1,
  showFolderFiles: false,
  selectMode: false,
  loading: false,
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

const gap = 8; // Gap between items

// item width and height(including gap)
const itemWidth = computed(() => {
  if (config.settings.grid.style === 0) {
    return config.settings.grid.size + gap * 2;
  } else if (config.settings.grid.style === 1) {
    return config.settings.grid.size;
  }
  return 0;
});

const itemHeight = computed(() => {
  if (config.settings.grid.style === 0) {
    let labelHeight = 0
    if (config.settings.grid.labelPrimary > 0 ) labelHeight += 20;      // height of text-sm
    if (config.settings.grid.labelSecondary > 0 ) labelHeight += 16;    // height of text-xs
    return itemWidth.value + gap / 2 + labelHeight;
  } else if (config.settings.grid.style === 1) {
    return itemWidth.value + gap / 2;
  }
  return 0;
});

const filmStripItemSize = computed(() => {
  return config.content.filmStripPaneHeight;
});

let resizeObserver: ResizeObserver | null = null;

function updateColumnCount() {
  if (containerRef.value) {
    containerWidth.value = containerRef.value.clientWidth;
    if (itemWidth.value > 0) {
      columnCount.value = Math.max(1, Math.floor(containerWidth.value / itemWidth.value));
    }
  }
}

watch(() => [config.settings.grid.size, config.settings.grid.style], () => {
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
      if (props.selectedItemIndex !== -1) {
        scrollToItem(props.selectedItemIndex);
      }
    });
    resizeObserver.observe(containerRef.value);
    updateColumnCount();
  }
  window.addEventListener('keydown', onKeyDown);
});

onBeforeUnmount(() => {
  window.removeEventListener('keydown', onKeyDown);
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

function onWheel(e: WheelEvent) {
  if (config.content.showFilmStrip && scroller.value) {
    // If it's a vertical scroll (deltaY) and no horizontal scroll (deltaX),
    // translate it to horizontal scroll
    if (Math.abs(e.deltaY) > Math.abs(e.deltaX)) {
      scroller.value.$el.scrollLeft += e.deltaY;
      e.preventDefault(); // Prevent default vertical scrolling behavior if any
    }
  }
}

function onKeyDown(e: KeyboardEvent) {
  // Prevent default scrolling for arrow keys and spacebar
  if (['ArrowUp', 'ArrowDown', 'Space', ' '].includes(e.key)) {
    // Allow default behavior if typing in an input
    const target = e.target as HTMLElement;
    if (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA' || target.isContentEditable) {
      return;
    }
    e.preventDefault();
  }
}

function scrollToItem(index: number) {
  if (!scroller.value) return;
  
  const el = scroller.value.$el;
  
  if (!config.content.showFilmStrip) {
    const row = Math.floor(index / columnCount.value);
    const itemTop = row * itemHeight.value;
    const itemBottom = itemTop + itemHeight.value;
    const scrollTop = el.scrollTop;
    const clientHeight = el.clientHeight;
    
    // Account for top and bottom padding
    const topPadding = 48; // pt-12 = 48px
    const bottomPadding = config.settings.showStatusBar ? 32 : 4; // pb-8 = 32px, pb-1 = 4px
    
    const viewportTop = scrollTop;
    const viewportBottom = scrollTop + clientHeight - (topPadding + bottomPadding);

    // Only scroll if the item is not fully visible
    const isFullyVisible = itemTop >= viewportTop && itemBottom <= viewportBottom;
    
    if (!isFullyVisible) {
      if (itemTop < viewportTop) {
        // Item is above viewport, scroll to show it at the top
        el.scrollTop = itemTop;
      } else if (itemBottom > viewportBottom) {
        // Item is below viewport, scroll to show it at the bottom (accounting for bottom padding)
        el.scrollTop = itemBottom - clientHeight + (topPadding + bottomPadding);
      }
    }
  } else {
    // Layout 1: Horizontal, center the item
    const itemSize = filmStripItemSize.value;
    const itemLeft = index * itemSize;
    const itemCenter = itemLeft + itemSize / 2;
    const clientWidth = el.clientWidth;
    
    // Calculate target scrollLeft to center the item
    let targetScrollLeft = itemCenter - clientWidth / 2;
    
    // Clamp to bounds
    targetScrollLeft = Math.max(0, targetScrollLeft);
    const maxScrollLeft = el.scrollWidth - clientWidth;
    targetScrollLeft = Math.min(targetScrollLeft, maxScrollLeft);
    
    el.scrollTo({
      left: targetScrollLeft,
      behavior: 'smooth'
    });
  }
}

function scrollToPosition(scrollTop: number) {
  if (scroller.value && !config.content.showFilmStrip) {
    scroller.value.$el.scrollTop = scrollTop;
  }
}

function getColumnCount() {
  return columnCount.value;
}

function getScrollTop() {
  return scroller.value ? scroller.value.$el.scrollTop : 0;
}

defineExpose({
  getColumnCount,
  scrollToPosition,
  getScrollTop,
});

</script>

<style scoped>
</style>

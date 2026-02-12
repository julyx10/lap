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
        'pt-12': config.settings.grid.style !== 3,
        'pb-8': config.settings.grid.style !== 3 && config.settings.showStatusBar,
        'pb-1': config.settings.grid.style !== 3 && !config.settings.showStatusBar,
      }"
      :items="fileList"
      :direction="config.settings.grid.style === 3 ? 'horizontal' : 'vertical'"
      :grid-items="config.settings.grid.style === 3 ? 1 : columnCount"
      :item-size="config.settings.grid.style === 3 ? filmStripItemSize : itemHeight"
      :item-secondary-size="config.settings.grid.style !== 3 ? itemWidth : undefined"
      :key="config.settings.grid.style === 3 ? 'filmstrip' : 'grid'"
      :geometry="config.settings.grid.style === 2 ? layoutGeometry : undefined"
      :content-height="config.settings.grid.style === 2 ? layoutContentHeight : undefined"
      :transition="isLayoutTransitioning"
      key-field="id"
      :emit-update="true"
      :buffer="4"
      v-slot="{ item, index }"
      @update="onUpdate"
      @scroll="onScroll"
    >
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
      <div class="text-base-content/30 flex flex-col items-center">
        <span>{{ loading ? $t('tooltip.loading') : $t('tooltip.not_found.files') }}</span>
      </div>
    </div>

  </div>

</template>

<script setup lang="ts">

import { watch, ref, onMounted, onBeforeUnmount, computed, nextTick } from 'vue';
import { useUIStore } from '@/stores/uiStore';
import { config } from '@/common/config';
import Thumbnail from '@/components/Thumbnail.vue';
import VirtualScroll from '@/components/VirtualScroll.vue';
import { calculateJustifiedLayout, calculateLinearRowLayout, type Geometry } from '@/common/layout';

const props = withDefaults(defineProps<{
  selectedItemIndex: number;
  fileList: any[];
  showFolderFiles?: boolean;
  selectMode?: boolean;
  loading?: boolean;
  layoutVersion?: number;
}>(), {
  selectedItemIndex: -1,
  showFolderFiles: false,
  selectMode: false,
  loading: false,
  layoutVersion: 0,
});

const emit = defineEmits([
  'item-clicked',
  'item-dblclicked',
  'item-select-toggled',
  'item-action',
  'request-scroll',
  'visible-range-update',
  'scroll',
  'layout-update',
]);

const uiStore = useUIStore();
const containerRef = ref<HTMLElement | null>(null);
const scroller = ref<any>(null);
const columnCount = ref(4);
const containerWidth = ref(0);

// Justified Layout State
const layoutGeometry = ref<Geometry[]>([]);
const layoutContentHeight = ref(0);
const isLayoutTransitioning = ref(false);
const startGridSize = ref(0);

const gap = 8; // Gap between items

// item width and height(including gap)
const itemWidth = computed(() => {
  const { style, size } = config.settings.grid;
  if (style === 0) return size + gap * 2;
  return size;
});

const itemHeight = computed(() => {
  const { style, size } = config.settings.grid;
  if (style === 0) {
    let labelHeight = 0;
    if (config.settings.grid.labelPrimary > 0) labelHeight += 20;   // text-sm
    if (config.settings.grid.labelSecondary > 0) labelHeight += 16; // text-xs
    return itemWidth.value + gap * 0.5 + labelHeight;
  }
  if (style === 1) return itemWidth.value + gap * 0.5;
  return size;
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

function updateLayout() {
  updateColumnCount();
  
  if (config.settings.grid.style === 3) {
    layoutGeometry.value = [];
    layoutContentHeight.value = 0;
    emit('layout-update', { height: 0 });
  } else if (config.settings.grid.style === 2 && containerWidth.value > 0) {
    // Calculate Justified Layout
    const result = calculateJustifiedLayout(
      props.fileList,
      containerWidth.value,
      config.settings.grid.size, // Target row height
      0 // Use correct spacing
    );
    layoutGeometry.value = result.boxes;
    layoutContentHeight.value = result.containerHeight;
    emit('layout-update', { height: result.containerHeight });
  } else {
    layoutGeometry.value = [];
    layoutContentHeight.value = 0;
    emit('layout-update', { height: 0 });
  }
}

watch(() => [config.settings.grid.size, config.settings.grid.style], () => {
  isLayoutTransitioning.value = true;
  updateLayout();
  
  if (props.selectedItemIndex !== -1) {
    nextTick(() => {
      scrollToItem(props.selectedItemIndex);
    });
  }

  setTimeout(() => {
    isLayoutTransitioning.value = false;
  }, 500);
});

watch(() => props.fileList, () => {
  updateLayout();
});

watch(() => props.layoutVersion, () => {
  updateLayout();
});

watch(() => props.selectedItemIndex, (newValue) => {
  if (newValue !== -1) {
    scrollToItem(newValue);
  }
});

onMounted(() => {
  if (containerRef.value) {
    resizeObserver = new ResizeObserver(() => {
      // updateColumnCount(); // merged into updateLayout
      updateLayout();
      if (props.selectedItemIndex !== -1) {
        scrollToItem(props.selectedItemIndex);
      }
    });
    resizeObserver.observe(containerRef.value);
    updateLayout();

    // gesture events for macOS touchpad pinch
    containerRef.value.addEventListener('gesturestart', onGestureStart as any);
    containerRef.value.addEventListener('gesturechange', onGestureChange as any);
  }
  window.addEventListener('keydown', onKeyDown);
});

onBeforeUnmount(() => {
  window.removeEventListener('keydown', onKeyDown);
  if (containerRef.value) {
    containerRef.value.removeEventListener('gesturestart', onGestureStart as any);
    containerRef.value.removeEventListener('gesturechange', onGestureChange as any);
  }
  if (resizeObserver) {
    resizeObserver.disconnect();
  }
});

function onGestureStart(e: any) {
  e.preventDefault();
  startGridSize.value = config.settings.grid.size;
}

function onGestureChange(e: any) {
  e.preventDefault();
  if (startGridSize.value > 0) {
    let newSize = Math.round(startGridSize.value * e.scale);
    // Clamp between 120 and 360
    newSize = Math.max(120, Math.min(360, newSize));
    config.settings.grid.size = newSize;
  }
}

function onUpdate(startIndex: number, endIndex: number) {
  emit('visible-range-update', { startIndex, endIndex });
}

function onScroll(e: Event) {
  emit('scroll', e);
}

function onWheel(e: WheelEvent) {
  if (config.settings.grid.style === 3 && scroller.value) {
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
  
  if (config.settings.grid.style !== 3) {
    let itemTop = 0;
    let itemBottom = 0;

    if (config.settings.grid.style === 2 && layoutGeometry.value[index]) {
      // Justified Layout
      const box = layoutGeometry.value[index];
      itemTop = box.y;
      itemBottom = box.y + box.height;
    } else {
      // Normal Grid Logic
      const row = Math.floor(index / columnCount.value);
      itemTop = row * itemHeight.value;
      itemBottom = itemTop + itemHeight.value;
    }

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
  if (scroller.value && config.settings.grid.style !== 3) {
    scroller.value.$el.scrollTop = scrollTop;
  }
}

function getColumnCount() {
  return columnCount.value;
}

function getScrollTop() {
  return scroller.value ? scroller.value.$el.scrollTop : 0;
}

function getNextItemIndex(currentIndex: number, direction: 'up' | 'down'): number {
  if (config.settings.grid.style !== 2 || layoutGeometry.value.length === 0) {
    return -1;
  }

  const currentBox = layoutGeometry.value[currentIndex];
  if (!currentBox) return currentIndex;

  const centerX = currentBox.x + currentBox.width / 2;
  const currentY = currentBox.y;
  
  // Find all items in the target direction
  let candidates: { index: number; box: Geometry; diffY: number }[] = [];

  layoutGeometry.value.forEach((box, index) => {
    if (direction === 'down') {
      if (box.y > currentY + 1) { // +1 for tolerance
         candidates.push({ index, box, diffY: box.y - currentY });
      }
    } else {
      if (box.y < currentY - 1) { // -1 for tolerance
         candidates.push({ index, box, diffY: currentY - box.y });
      }
    }
  });

  if (candidates.length === 0) return currentIndex;

  // Find the closest row (smallest diffY)
  const minDiffY = Math.min(...candidates.map(c => c.diffY));
  
  // Filter candidates to only those in the closest row
  const rowCandidates = candidates.filter(c => Math.abs(c.diffY - minDiffY) < 5); // 5px tolerance

  // Find item with closest centerX
  let closestIndex = -1;
  let minDistX = Infinity;

  rowCandidates.forEach(c => {
    const boxCenterX = c.box.x + c.box.width / 2;
    const dist = Math.abs(boxCenterX - centerX);
    if (dist < minDistX) {
      minDistX = dist;
      closestIndex = c.index;
    }
  });

  return closestIndex !== -1 ? closestIndex : currentIndex;
}

defineExpose({
  getColumnCount,
  scrollToPosition,
  getScrollTop,
  getNextItemIndex
});

</script>

<style scoped>
</style>

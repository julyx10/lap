<template>
  <div 
    ref="scrollerRef" 
    class="virtual-scroller h-full w-full overflow-auto relative"
    @scroll.passive="onScroll"
  >
    <!-- Combined content container (in-flow) behaves as phantom spacer and positioning context -->
    <div :style="wrapperStyle">
      <div 
        v-for="poolItem in visibleItems" 
        :key="poolItem.index"
        class="absolute"
        :class="{ 'transition-all duration-500 ease-in-out': transition }"
        :style="getItemStyle(poolItem)"
      >
        <slot 
          :item="poolItem.item" 
          :index="poolItem.index"
        ></slot>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, onBeforeUnmount, nextTick, type CSSProperties } from 'vue';

const props = withDefaults(defineProps<{
  items: any[];
  itemSize: number; // Height for vertical, width for horizontal
  itemSecondarySize?: number; // Width for vertical (optional)
  gridItems?: number; // Items per row/column
  direction?: 'vertical' | 'horizontal';
  keyField?: string;
  buffer?: number;
  emitUpdate?: boolean;
  geometry?: { x: number, y: number, width: number, height: number }[]; // Pre-calculated layout
  contentHeight?: number; // Total height for pre-calculated layout
  transition?: boolean;
}>(), {
  gridItems: 1,
  direction: 'vertical',
  keyField: 'id',
  buffer: 2, // Number of rows/cols to buffer
  emitUpdate: false,
  transition: false,
});

const emit = defineEmits(['update', 'scroll']);

const scrollerRef = ref<HTMLElement | null>(null);
const scrollOffset = ref(0);
const containerSize = ref(0);

// Computed properties
const isVertical = computed(() => props.direction === 'vertical');

const totalCount = computed(() => props.items.length);

const totalRows = computed(() => Math.ceil(totalCount.value / props.gridItems));

const totalSize = computed(() => {
  if (props.contentHeight !== undefined) return props.contentHeight;
  return totalRows.value * props.itemSize;
});

const wrapperStyle = computed((): CSSProperties => {
  return isVertical.value
    ? { height: `${totalSize.value}px`, width: '100%', position: 'relative' }
    : { width: `${totalSize.value}px`, height: '100%', position: 'relative' };
});

const geometryAxisMetrics = computed(() => {
  const starts: number[] = [];
  const prefixMaxEnds: number[] = [];
  let maxEnd = 0;

  props.geometry?.forEach((box) => {
    const start = isVertical.value ? box.y : box.x;
    const size = isVertical.value ? box.height : box.width;
    starts.push(start);
    maxEnd = Math.max(maxEnd, start + size);
    prefixMaxEnds.push(maxEnd);
  });

  return { starts, prefixMaxEnds };
});

const visibleRange = computed(() => {
  if (props.geometry) {
    const viewportStart = scrollOffset.value;
    const viewportEnd = viewportStart + containerSize.value;
    const { starts, prefixMaxEnds } = geometryAxisMetrics.value;

    // Find the first item whose cumulative maximum end reaches the viewport.
    let low = 0;
    let high = prefixMaxEnds.length;
    while (low < high) {
      const mid = Math.floor((low + high) / 2);
      if (prefixMaxEnds[mid] >= viewportStart) high = mid;
      else low = mid + 1;
    }
    const start = low;

    // Item starts are monotonic for justified and shortest-column masonry
    // layouts, so the first start beyond the viewport is a valid upper bound.
    low = start;
    high = starts.length;
    while (low < high) {
      const mid = Math.floor((low + high) / 2);
      if (starts[mid] > viewportEnd) high = mid;
      else low = mid + 1;
    }
    const end = low;

    const bufferCount = props.buffer * (props.gridItems || 4);
    
    return {
      start: Math.max(0, start - bufferCount),
      end: Math.min(totalCount.value, end + bufferCount)
    };
  }

  const start = Math.floor(scrollOffset.value / props.itemSize);
  const visibleCount = Math.ceil(containerSize.value / props.itemSize);
  
  // Add buffer
  const bufferedStart = Math.max(0, start - props.buffer);
  const bufferedEnd = Math.min(totalRows.value, start + visibleCount + props.buffer);
  
  return {
    start: bufferedStart,
    end: bufferedEnd
  };
});

const visibleItems = computed(() => {
  const { start, end } = visibleRange.value;
  const result = [];
  
  if (props.geometry) {
    for (let i = start; i < end; i++) {
        if (i < props.items.length) {
            result.push({
                item: props.items[i],
                index: i,
            });
        }
    }
  } else {
    const startIndex = start * props.gridItems;
    const endIndex = Math.min(totalCount.value, end * props.gridItems);
    
    for (let i = startIndex; i < endIndex; i++) {
      result.push({
        item: props.items[i],
        index: i,
      });
    }
  }
  
  return result;
});

function getItemStyle(poolItem: { item: any, index: number }): CSSProperties {
  const { index } = poolItem;
  
  if (props.geometry && props.geometry[index]) {
    const box = props.geometry[index];
    return {
      position: 'absolute',
      top: `${box.y}px`,
      left: `${box.x}px`,
      width: `${box.width}px`,
      height: `${box.height}px`,
    };
  }

  const row = Math.floor(index / props.gridItems);
  const col = index % props.gridItems;
  
  if (isVertical.value) {
    const top = row * props.itemSize;
    const left = props.itemSecondarySize ? col * props.itemSecondarySize : 0;
    // If itemSecondarySize is not provided, we assume width is 100% / gridItems? 
    // Or simpler: let the slot handle internal sizing, we just position the wrapper
    // But GridView.vue passes itemSecondarySize (itemWidth).
    
    // If gridItems > 1, we need to know the width of each item to position 'left'.
    // The props.itemSecondarySize is passed from GridView as itemWidth.
    // So we use that.
    
    return {
      position: 'absolute',
      top: `${top}px`,
      left: `${left}px`,
      width: props.itemSecondarySize ? `${props.itemSecondarySize}px` : undefined,
      height: `${props.itemSize}px`,
    };
  } else {
    // Horizontal
    const left = row * props.itemSize;
    // For horizontal mode/Filmstrip, gridItems is usually 1, but we use row logic map to 'col' in horizontal
    // Layout 1 (Filmstrip): items flow horizontally.
    const top = 0; 
    
    return {
      position: 'absolute',
      left: `${left}px`,
      top: `${top}px`,
      height: '100%',
      width: `${props.itemSize}px`,
    };
  }
}

const emitUpdate = () => {
  if (!props.emitUpdate) return;

  if (props.geometry) {
    emit('update', visibleRange.value.start, visibleRange.value.end);
    return;
  }

  const startIndex = visibleRange.value.start * props.gridItems;
  const endIndex = Math.min(totalCount.value, visibleRange.value.end * props.gridItems);
  emit('update', startIndex, endIndex);
};

watch(() => visibleRange.value, () => {
  emitUpdate();
});

// Watch totalCount to re-emit if data changes but range is same (e.g. initial load)
watch(totalCount, () => {
    emitUpdate();
});

function findGeometryAnchor(
  geometry: { x: number, y: number, width: number, height: number }[],
  viewportStart: number,
) {
  if (geometry.length === 0) return -1;

  // Geometry starts are monotonic for the currently supported layouts.
  // If a future layout emits non-monotonic starts, use a linear/local scan.
  // Find the nearest start in O(log n).
  let low = 0;
  let high = geometry.length;
  while (low < high) {
    const mid = Math.floor((low + high) / 2);
    const start = isVertical.value ? geometry[mid].y : geometry[mid].x;
    if (start < viewportStart) low = mid + 1;
    else high = mid;
  }

  if (low === 0) return 0;
  if (low === geometry.length) return geometry.length - 1;

  const previousStart = isVertical.value ? geometry[low - 1].y : geometry[low - 1].x;
  const nextStart = isVertical.value ? geometry[low].y : geometry[low].x;
  return viewportStart - previousStart <= nextStart - viewportStart ? low - 1 : low;
}

let geometryAnchorVersion = 0;
let pendingGeometryAnchor: {
  index: number;
  viewportStart: number;
  oldStart: number;
} | null = null;

watch(
  () => props.geometry,
  async (_newGeometry, oldGeometry) => {
    const scroller = scrollerRef.value;
    if (
      !scroller ||
      !props.geometry ||
      !oldGeometry ||
      props.transition ||
      props.geometry.length !== oldGeometry.length ||
      props.geometry.length === 0
    ) {
      pendingGeometryAnchor = null;
      // Cancel any anchor correction already waiting on nextTick.
      geometryAnchorVersion++;
      return;
    }

    const viewportStart = isVertical.value ? scroller.scrollTop : scroller.scrollLeft;
    if (viewportStart <= 0) {
      pendingGeometryAnchor = null;
      // Cancel any anchor correction already waiting on nextTick.
      geometryAnchorVersion++;
      return;
    }

    if (!pendingGeometryAnchor) {
      const anchorIndex = findGeometryAnchor(oldGeometry, viewportStart);
      const oldAnchor = oldGeometry[anchorIndex];
      if (!oldAnchor) return;
      pendingGeometryAnchor = {
        index: anchorIndex,
        viewportStart,
        oldStart: isVertical.value ? oldAnchor.y : oldAnchor.x,
      };
    }

    // Keep the first old geometry as the batch baseline. Replacing it on a
    // subsequent update would lose displacement that has not been applied yet.
    const version = ++geometryAnchorVersion;
    await nextTick();
    if (version !== geometryAnchorVersion || !pendingGeometryAnchor || !props.geometry) return;

    const anchor = pendingGeometryAnchor;
    pendingGeometryAnchor = null;
    const newAnchor = props.geometry[anchor.index];
    if (!newAnchor) return;

    const currentOffset = isVertical.value ? scroller.scrollTop : scroller.scrollLeft;
    if (Math.abs(currentOffset - anchor.viewportStart) >= 0.5) return;

    const newStart = isVertical.value ? newAnchor.y : newAnchor.x;
    const delta = newStart - anchor.oldStart;
    if (!Number.isFinite(delta) || Math.abs(delta) < 0.5) return;

    const targetOffset = Math.max(0, anchor.viewportStart + delta);
    if (isVertical.value) {
      scroller.scrollTop = targetOffset;
    } else {
      scroller.scrollLeft = targetOffset;
    }
    scrollOffset.value = targetOffset;
  },
  { flush: 'pre' },
);

function onScroll(e: Event) {
  const target = e.target as HTMLElement;
  scrollOffset.value = isVertical.value ? target.scrollTop : target.scrollLeft;
  emit('scroll', e);
  // Update event is handled by watcher on visibleRange
}

function updateContainerSize() {
  if (scrollerRef.value) {
    containerSize.value = isVertical.value 
      ? scrollerRef.value.clientHeight 
      : scrollerRef.value.clientWidth;
  }
}

// Watch for direction changes to reset/update size
watch(() => props.direction, () => {
  nextTick(updateContainerSize);
});

let resizeObserver: ResizeObserver | null = null;

onMounted(() => {
  updateContainerSize();
  emitUpdate();
  resizeObserver = new ResizeObserver(updateContainerSize);
  if (scrollerRef.value) {
    resizeObserver.observe(scrollerRef.value);
  }
});

onBeforeUnmount(() => {
  resizeObserver?.disconnect();
  resizeObserver = null;
});

// Expose scroll to item / position
function scrollToPosition(value: number) {
  if (!scrollerRef.value) return;
  if (isVertical.value) scrollerRef.value.scrollTop = value;
  else scrollerRef.value.scrollLeft = value;
  scrollOffset.value = value;
}

defineExpose({
  scrollToPosition, // Simplified exposure
  $el: scrollerRef // Expose element to match old interface if needed
});

</script>

<style scoped>
.virtual-scroller {
  /* Ensure it has a position context */
  position: relative;
  /* Hide scrollbar if requested by class in parent, but component itself needs auto */
  /* .no-scrollbar usually handles hiding webkit-scrollbar */
}
</style>

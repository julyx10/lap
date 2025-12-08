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
        :key="poolItem.item[keyField]"
        class="absolute"
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
import { ref, computed, watch, onMounted, nextTick, type CSSProperties } from 'vue';

const props = withDefaults(defineProps<{
  items: any[];
  itemSize: number; // Height for vertical, width for horizontal
  itemSecondarySize?: number; // Width for vertical (optional)
  gridItems?: number; // Items per row/column
  direction?: 'vertical' | 'horizontal';
  keyField?: string;
  buffer?: number;
  emitUpdate?: boolean;
}>(), {
  gridItems: 1,
  direction: 'vertical',
  keyField: 'id',
  buffer: 2, // Number of rows/cols to buffer
  emitUpdate: false,
});

const emit = defineEmits(['update', 'scroll']);

const scrollerRef = ref<HTMLElement | null>(null);
const scrollOffset = ref(0);
const containerSize = ref(0);

// Computed properties
const isVertical = computed(() => props.direction === 'vertical');

const totalCount = computed(() => props.items.length);

const totalRows = computed(() => Math.ceil(totalCount.value / props.gridItems));

const totalSize = computed(() => totalRows.value * props.itemSize);

const wrapperStyle = computed((): CSSProperties => {
  return isVertical.value
    ? { height: `${totalSize.value}px`, width: '100%', position: 'relative' }
    : { width: `${totalSize.value}px`, height: '100%', position: 'relative' };
});

const visibleRange = computed(() => {
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
  
  const startIndex = start * props.gridItems;
  const endIndex = Math.min(totalCount.value, end * props.gridItems);
  
  for (let i = startIndex; i < endIndex; i++) {
    result.push({
      item: props.items[i],
      index: i,
    });
  }
  
  return result;
});

function getItemStyle(poolItem: { item: any, index: number }): CSSProperties {
  const { index } = poolItem;
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
  if (props.emitUpdate) {
    const startIdx = visibleRange.value.start * props.gridItems;
    const endIdx = Math.min(totalCount.value, visibleRange.value.end * props.gridItems);
    emit('update', startIdx, endIdx);
  }
};

watch(() => visibleRange.value, () => {
  emitUpdate();
});

// Watch totalCount to re-emit if data changes but range is same (e.g. initial load)
watch(totalCount, () => {
    emitUpdate();
});

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

onMounted(() => {
  updateContainerSize();
  const observer = new ResizeObserver(updateContainerSize);
  if (scrollerRef.value) {
    observer.observe(scrollerRef.value);
  }
});

// Expose scroll to item / position
function scrollToPosition(value: number) {
   if (scrollerRef.value) {
     if (isVertical.value) scrollerRef.value.scrollTop = value;
     else scrollerRef.value.scrollLeft = value;
   }
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

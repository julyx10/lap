<template>
  <div 
    :class="[
      'h-full flex flex-col items-end justify-center',
      markers.length > 0 ? 'w-13' : 'w-4'
    ]"
  >
    <IconScrollUp 
      :class="[
        'w-4 h-4 text-base-content/30',
        total > pageSize ? 'hover:text-base-content/50 cursor-pointer' : '' 
      ]" 
      @click="clickPreviousPage"
    />

    <!-- Main Area -->
    <div class="flex-1 w-full flex flex-row relative">
      
      <!-- Markers Area (Left) -->
      <div 
        class="flex-1 h-full relative cursor-row-resize"
        @mouseenter="handleMarkersMouseEnter"
        @mouseleave="handleMarkersMouseLeave"
        @mousemove="handleMarkersMouseMove"
        @click="handleMarkersClick"
      >
        <!-- Markers (Years and Months) -->
        <div v-for="(marker, index) in displayMarkers" :key="index"
          class="absolute w-full right-1 tabular-nums pointer-events-none select-none flex items-center justify-end z-10"
          :class="[
            marker.isYear ? 'text-[10px] text-base-content/70' : 'text-[9px] text-base-content/30'
          ]"
          :style="{ top: marker.top + '%', transform: 'translateY(-50%)' }"
        >
          {{ marker.label }}
        </div>

        <!-- Hover Marker -->
        <div v-if="isHovering"
          class="absolute w-full h-[2px] bg-primary/30 pointer-events-none rounded-full"
          :style="{ top: (hoverY - 1) + 'px' }"
        ></div>

        <!-- Selected Marker -->
        <div v-if="selectedTop >= 0"
          class="absolute w-full h-[2px] bg-primary/70 pointer-events-none rounded-full"
          :style="{ top: (selectedTop - 1) + 'px', transition: 'all 0.2s ease-in-out' }"
        ></div>

        <!-- Hover Tooltip -->
        <div v-if="isHovering && hoverDate"
          class="absolute right-full mr-1 px-2 py-1 bg-base-100/50 text-base-content text-xs tabular-nums whitespace-nowrap rounded-box shadow-lg backdrop-blur-sm z-20 pointer-events-none flex items-center"
          :style="{ top: hoverY + 'px', transform: 'translateY(-50%)' }"
        >
          {{ hoverDate }}
          <!-- Arrow -->
          <!-- <div class="absolute left-full top-1/2 -translate-y-1/2 border-4 border-transparent border-l-base-300"></div> -->
        </div>
      </div>

      <!-- Track Area (Right) -->
      <div ref="trackRef" 
        :class="[
          'w-3 relative rounded-full ml-1 mr-0.5', 
          total > pageSize ? 'bg-base-200 cursor-pointer' : 'bg-base-200/30'
        ]" 
        @mousedown="handleTrackClick"
      >
        <!-- Thumb -->
        <div v-if="total > pageSize"
          ref="thumbRef"
          class="absolute w-3 rounded-full bg-base-content/30 hover:bg-base-content/50 transition-colors duration-200 ease-in-out" 
          :style="{ top: thumbTop + 'px', height: thumbHeight + 'px' }"
          @mousedown.stop="handleThumbMouseDown"
        ></div>
      </div>

    </div>

    <IconScrollDown 
      :class="[
        'w-4 h-4 text-base-content/30',
        total > pageSize ? 'hover:text-base-content/50 cursor-pointer' : '' 
      ]" 
      @click="clickNextPage"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onBeforeUnmount } from 'vue';
import { IconScrollUp, IconScrollDown } from '@/common/icons';

const props = defineProps({
  total: {
    type: Number,
    default: 0
  },
  pageSize: {
    type: Number,
    default: 20
  },
  modelValue: { // Current offset (index of the first visible item)
    type: Number,
    default: 0
  },
  markers: {
    type: Array as () => Array<{ year: number | null, month: number | null, date: number | null, position: number }>, 
    default: () => []
  },
  selectedIndex: {
    type: Number,
    default: -1
  }
});

const emit = defineEmits(['update:modelValue', 'change', 'select-item']);

const trackRef = ref<HTMLElement | null>(null);
const thumbRef = ref<HTMLElement | null>(null);

// State for dragging
const isDragging = ref(false);
const startY = ref(0);
const startThumbTop = ref(0);

// State for hover
const isHovering = ref(false);
const hoverY = ref(0);
const hoverDate = ref('');

// Computed properties for thumb dimensions and position
const trackHeight = ref(0);

// Process markers to show Year/Month changes
const displayMarkers = computed(() => {
  if (!props.total || props.total === 0 || !props.markers.length) return [];
  
  const height = trackHeight.value; 
  const minLabelDistance = 12;
  const minTickDistance = 5;

  // 1. Generate Candidates
  interface Candidate {
    label: string;
    top: number;
    type: 'year' | 'month';
    pixelPos: number;
    showLabel: boolean;
    showTick: boolean;
  }

  const candidates: Candidate[] = [];
  let lastYear: number | null = null;
  let lastMonth: number | null = null;

  for (const marker of props.markers) {
    if (marker.position === undefined || marker.position === null) continue;
    
    const top = Math.min(100, (marker.position / props.total) * 100);
    const currentPixelPos = (top / 100) * height;
    
    const year = marker.year;
    const month = marker.month;

    if (year !== null && year !== lastYear) {
      candidates.push({
        label: year === 0 ? '' : String(year),
        top: top,
        type: 'year',
        pixelPos: currentPixelPos,
        showLabel: true,
        showTick: true
      });
      lastYear = year;
      lastMonth = month;
    } else if (month !== null && month !== lastMonth) {
      candidates.push({
        label: String(month).padStart(2, '0'),
        top: top,
        type: 'month',
        pixelPos: currentPixelPos,
        showLabel: true,
        showTick: true
      });
      lastMonth = month;
    }
  }

  // 2. Filter Years (First pass)
  // Determine which Years have visible labels
  let lastAcceptedYearPos = -minLabelDistance * 2;
  for (const item of candidates) {
    if (item.type === 'year') {
      if (Math.abs(item.pixelPos - lastAcceptedYearPos) >= minLabelDistance) {
        item.showLabel = true;
        lastAcceptedYearPos = item.pixelPos;
      } else {
        item.showLabel = false;
      }
      item.showTick = true; // Years always show ticks
    }
  }

  // 3. Filter Months (Second pass)
  // Check distances for labels and ticks
  let lastVisibleLabelPos = -minLabelDistance * 2;
  let lastVisibleTickPos = -minTickDistance * 2;
  
  // Pre-calculate visible years for fast look-ahead
  const visibleYearPositions = candidates
    .filter(c => c.type === 'year' && c.showLabel)
    .map(c => c.pixelPos);
  
  let nextYearIdx = 0;

  for (const item of candidates) {
    // Update next visible year pointer
    while (nextYearIdx < visibleYearPositions.length && visibleYearPositions[nextYearIdx] <= item.pixelPos) {
      nextYearIdx++;
    }
    const nextYearPos = nextYearIdx < visibleYearPositions.length ? visibleYearPositions[nextYearIdx] : Infinity;

    if (item.type === 'year') {
      if (item.showLabel) {
        lastVisibleLabelPos = item.pixelPos;
      }
      lastVisibleTickPos = item.pixelPos;
      continue;
    }

    if (item.type === 'month') {
      // Tick Visibility
      if (Math.abs(item.pixelPos - lastVisibleTickPos) >= minTickDistance) {
        item.showTick = true;
        lastVisibleTickPos = item.pixelPos;
      } else {
        item.showTick = false;
      }

      // Label Visibility
      const distPrev = Math.abs(item.pixelPos - lastVisibleLabelPos);
      const distNext = Math.abs(nextYearPos - item.pixelPos);

      if (distPrev >= minLabelDistance && distNext >= minLabelDistance) {
        item.showLabel = true;
        lastVisibleLabelPos = item.pixelPos;
      } else {
        item.showLabel = false;
      }
    }
  }

  return candidates
    .filter(item => item.showTick) // Only return items that have at least a tick
    .map(item => ({
      label: item.showLabel ? item.label : '', // Empty label if hidden
      top: item.top,
      isYear: item.type === 'year',
      pixelPos: item.pixelPos,
      showTick: item.showTick
    }));
});

// Update track height on resize
const updateTrackHeight = () => {
  if (trackRef.value) {
    trackHeight.value = trackRef.value.clientHeight;
  }
};

const resizeObserver = new ResizeObserver(updateTrackHeight);

onMounted(() => {
  if (trackRef.value) {
    resizeObserver.observe(trackRef.value);
    updateTrackHeight();
  }
  window.addEventListener('mouseup', handleMouseUp);
  window.addEventListener('mousemove', handleMouseMove);
});

onBeforeUnmount(() => {
  if (trackRef.value) {
    resizeObserver.unobserve(trackRef.value);
  }
  window.removeEventListener('mouseup', handleMouseUp);
  window.removeEventListener('mousemove', handleMouseMove);
});

// Calculate thumb height based on viewport ratio
const thumbHeight = computed(() => {
  if (props.total <= 0) return 30; // Default min height
  const ratio = props.pageSize / props.total;
  const height = Math.max(30, trackHeight.value * ratio); // Min height 30px
  return Math.min(height, trackHeight.value);
});

// Calculate thumb position based on modelValue (offset)
const thumbTop = computed(() => {
  if (props.total <= props.pageSize) return 0;
  
  const maxTop = trackHeight.value - thumbHeight.value;
  const maxOffset = props.total - props.pageSize;
  const ratio = props.modelValue / maxOffset;
  
  return Math.min(Math.max(0, ratio * maxTop), maxTop);
});

// Calculate selected item position
const selectedTop = computed(() => {
  if (props.selectedIndex < 0 || props.total <= 0) return -1;
  
  const ratio = props.selectedIndex / props.total;
  // Use the full track height for markers
  const top = ratio * trackHeight.value;
  return Math.min(top, trackHeight.value);
});

// --- Interaction Handlers ---

function clickPreviousPage() {
  const newOffset = Math.max(0, props.modelValue - props.pageSize);
  emitUpdate(newOffset);
}

function clickNextPage() {
  const maxOffset = Math.max(0, props.total - props.pageSize);
  const newOffset = Math.min(maxOffset, props.modelValue + props.pageSize);
  emitUpdate(newOffset);
}

function handleTrackClick(e: MouseEvent) {
  if (!trackRef.value) return;
  
  const rect = trackRef.value.getBoundingClientRect();
  const clickY = e.clientY - rect.top;
  
  // Center the thumb on the click position
  const halfThumb = thumbHeight.value / 2;
  const targetTop = Math.max(0, Math.min(clickY - halfThumb, trackHeight.value - thumbHeight.value));
  
  updateOffsetFromTop(targetTop);
}

function handleThumbMouseDown(e: MouseEvent) {
  isDragging.value = true;
  startY.value = e.clientY;
  startThumbTop.value = thumbTop.value;
  document.body.style.userSelect = 'none'; // Prevent selection while dragging
}

function handleMouseMove(e: MouseEvent) {
  if (!isDragging.value) return;
  
  const deltaY = e.clientY - startY.value;
  const newTop = startThumbTop.value + deltaY;
  
  updateOffsetFromTop(newTop);
}

function handleMouseUp() {
  isDragging.value = false;
  document.body.style.userSelect = '';
}

function updateOffsetFromTop(top: number) {
  const maxTop = trackHeight.value - thumbHeight.value;
  if (maxTop <= 0) return;

  const clampedTop = Math.max(0, Math.min(top, maxTop));
  const ratio = clampedTop / maxTop;
  
  const maxOffset = Math.max(0, props.total - props.pageSize);
  const newOffset = Math.round(ratio * maxOffset);
  
  emitUpdate(newOffset);
}

function handleMarkersMouseEnter() {
  isHovering.value = true;
}

function handleMarkersMouseLeave() {
  isHovering.value = false;
}

function handleMarkersMouseMove(e: MouseEvent) {
  if (!trackRef.value) return; // Use trackRef for height calculation as it's the same height
  const rect = trackRef.value.getBoundingClientRect();
  const y = e.clientY - rect.top;

  if (!props.markers.length || props.total <= 0) return;

  const targetIndex = getIndexFromY(y);
  
  // Snap hoverY to the item position
  const snappedY = (targetIndex / props.total) * trackHeight.value;
  hoverY.value = snappedY;

  // Find closest marker
  const marker = findMarkerForIndex(targetIndex);
  if (marker) {
    hoverDate.value = formatDate(marker);
  }
}

function handleMarkersClick(e: MouseEvent) {
  if (!trackRef.value) return;
  const rect = trackRef.value.getBoundingClientRect();
  const y = e.clientY - rect.top;
  
  const targetIndex = getIndexFromY(y);
  emit('select-item', targetIndex);
}

function getIndexFromY(y: number) {
  const height = trackHeight.value;
  if (height <= 0) return 0;
  
  const ratio = Math.max(0, Math.min(y / height, 1));
  return Math.min(Math.floor(ratio * props.total), props.total - 1);
}

function findMarkerForIndex(index: number) {
  let low = 0;
  let high = props.markers.length - 1;
  let res = null;

  while (low <= high) {
    const mid = Math.floor((low + high) / 2);
    if (props.markers[mid].position <= index) {
      res = props.markers[mid];
      low = mid + 1;
    } else {
      high = mid - 1;
    }
  }
  return res;
}

function formatDate(marker: { year: number | null, month: number | null, date: number | null }) {
  const y = marker.year;
  const m = marker.month;
  const d = marker.date;
  
  if (!y) return '';
  
  let str = `${y}`;
  if (m !== null) str += `-${String(m).padStart(2, '0')}`;
  if (d !== null) str += `-${String(d).padStart(2, '0')}`;
  return str;
}

function emitUpdate(newValue: number) {
  if (newValue !== props.modelValue) {
    emit('update:modelValue', newValue);
    emit('change', newValue);
  }
}

</script>
